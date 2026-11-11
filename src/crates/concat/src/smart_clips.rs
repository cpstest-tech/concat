// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Auto-Highlight / Smart Clips engine (CapCut-style).
//!
//! Pipeline in 3 main phases:
//! - Step A: Transcription pre-processing (`format_transcript_for_llm`), assigning
//!   sequential IDs to transcript segments and building a fast lookup map.
//! - Step B: Google Gemini caller (`generate_smart_clips`) with strict JSON schema
//!   and structured outputs to discover viral moments (defaults to `gemini-3.8-flash`).
//! - Step C: Timeline post-processing and mapping (`map_clips_to_timeline`), validating
//!   IDs, adding safety boundary buffers (-0.1s / +0.15s), and generating Concat commands.

use std::collections::HashMap;
use std::time::Duration;

use concat_project::Command;
use concat_speech::transcribe::Segment;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Minimum safety clip duration in seconds.
const MIN_DURATION: f64 = 0.5;

/// Pre-processed transcript with sequential IDs and string representations.
#[derive(Clone, Debug)]
pub struct FormattedTranscript {
    /// Formatted representation for LLM: `[ID: {id} | {start}s - {end}s] {text}`
    pub formatted_text: String,
    /// Fast in-memory lookup map from ID to original transcription segment.
    pub segment_map: HashMap<usize, Segment>,
    /// Total duration covered by the transcript in seconds.
    pub total_duration: f64,
}

/// A raw cut segment returned by the LLM containing segment IDs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawCutSegment {
    /// Starting transcript segment ID.
    pub start_id: usize,
    /// Ending transcript segment ID (inclusive).
    pub end_id: usize,
}

/// A raw smart clip returned by Gemini Structured Outputs.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawSmartClip {
    /// Catchy viral title for the clip.
    pub title: String,
    /// Predicted virality score (1 - 100).
    pub virality_score: f64,
    /// Why this clip was selected (hook, payoff, pacing).
    pub editorial_reason: String,
    /// Segments that compose this clip (supports multi-cut to remove fluff/silence).
    pub segments: Vec<RawCutSegment>,
}

/// Structured response schema returned by Gemini.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeminiSmartClipsResponse {
    /// High-level summary of the video content.
    pub video_summary: String,
    /// Extracted highlight clips.
    pub clips: Vec<RawSmartClip>,
}

/// A single cut segment mapped to real media source timestamps.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MappedCutSegment {
    /// Timestamp where this cut starts in the source media (seconds).
    pub source_start: f64,
    /// Duration of this cut (seconds).
    pub duration: f64,
}

/// A processed highlight clip ready for preview and timeline placement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessedSmartClip {
    /// Catchy title.
    pub title: String,
    /// Virality score (1 - 100).
    pub virality_score: u32,
    /// Editorial explanation.
    pub editorial_reason: String,
    /// Sum of durations of all its cut segments (seconds).
    pub total_duration: f64,
    /// The individual cuts making up this clip (with padding buffer applied).
    pub segments: Vec<MappedCutSegment>,
}

/// Target duration constraint presets for clip extraction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetDurationPreset {
    /// Clips under 30 seconds (ideal for Reels / Shorts / TikTok).
    Short,
    /// Clips between 30 and 60 seconds.
    Medium,
    /// Clips between 60 and 90 seconds.
    Long,
}

impl TargetDurationPreset {
    /// (min_duration_seconds, max_duration_seconds)
    pub fn range_seconds(self) -> (f64, f64) {
        match self {
            Self::Short => (10.0, 30.0),
            Self::Medium => (30.0, 60.0),
            Self::Long => (60.0, 90.0),
        }
    }

    /// Label description for prompts or UI.
    pub fn label(self) -> &'static str {
        match self {
            Self::Short => "< 30s",
            Self::Medium => "30s - 60s",
            Self::Long => "60s - 90s",
        }
    }
}

/// Default model on Google AI Studio Free Tier (1M token context, high-quality editorial reasoning).
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.8-flash";
/// Fast fallback model for large transcripts or quota limits (1M token context, ultra-fast).
pub const FALLBACK_GEMINI_MODEL: &str = "gemini-3.5-flash-lite";

/// Configuration options for the smart clips analysis.
#[derive(Clone, Debug)]
pub struct SmartClipsConfig {
    /// Maximum number of clips to return (e.g. 3, 5, 10).
    pub max_clips: usize,
    /// Target duration preset.
    pub duration_preset: TargetDurationPreset,
    /// Video duration in seconds (boundary limit for buffers).
    pub video_duration: f64,
    /// Optional user prompt / directions / topic focus.
    pub custom_directions: Option<String>,
    /// Model ID to invoke (e.g. "gemini-3.8-flash", "gemini-3.5-flash-lite").
    pub model: String,
}

impl Default for SmartClipsConfig {
    fn default() -> Self {
        Self {
            max_clips: 5,
            duration_preset: TargetDurationPreset::Short,
            video_duration: 3600.0,
            custom_directions: None,
            model: DEFAULT_GEMINI_MODEL.to_owned(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STEP A: Pre-processor della Trascrizione (`formatTranscriptForLLM`)
// ─────────────────────────────────────────────────────────────────────────────

/// Converts raw transcription segments into an indexed representation with unique IDs.
///
/// Ensures the LLM reasons over discrete, bounded token IDs rather than floating point timestamps.
pub fn format_transcript_for_llm(segments: &[Segment]) -> FormattedTranscript {
    let mut segment_map = HashMap::with_capacity(segments.len());
    let mut formatted_text = String::new();
    let mut max_end = 0.0f64;

    for (id, seg) in segments.iter().enumerate() {
        let text = seg.text.trim();
        if text.is_empty() {
            continue;
        }
        segment_map.insert(id, seg.clone());
        formatted_text.push_str(&format!(
            "[ID: {} | {:.2}s - {:.2}s] {}\n",
            id, seg.start, seg.end, text
        ));
        if seg.end > max_end {
            max_end = seg.end;
        }
    }

    FormattedTranscript {
        formatted_text,
        segment_map,
        total_duration: max_end,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// STEP B: Servizio Gemini con Structured Outputs (`generateSmartClips`)
// ─────────────────────────────────────────────────────────────────────────────

/// Calls Google Gemini via REST API with a strict JSON response schema.
pub fn generate_smart_clips(
    api_key: &str,
    formatted: &FormattedTranscript,
    config: &SmartClipsConfig,
) -> Result<GeminiSmartClipsResponse, String> {
    let api_key = api_key.trim().trim_matches('"').trim_matches('\'');
    if api_key.is_empty() {
        return Err("Chiave API di Google Gemini non configurata. Inseriscila nelle impostazioni o nella variabile d'ambiente GEMINI_API_KEY.".to_owned());
    }

    let (min_dur, max_dur) = config.duration_preset.range_seconds();
    let max_clips = config.max_clips.max(1);

    let directions_clause = if let Some(directions) = &config.custom_directions {
        let trimmed = directions.trim();
        if !trimmed.is_empty() {
            format!(
                "\n\nDIREZIONI SPECIFICHE E FOCUS RICHIESTO DALL'UTENTE:\n\
                L'utente desidera concentrarsi su questi temi o momenti specifici:\n\
                \"{trimmed}\"\n\
                DARE PRIORITÀ ad estrarre clip incentrate su questi momenti/argomenti richiesti, purché coerenti e di impatto."
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let system_prompt = format!(
        "Sei un Senior Video Editor e Viral Strategist per TikTok, Reels e Shorts. Analizza la trascrizione indicizzata fornita ed estrai fino a {max_clips} clip ad alto potenziale virale.\n\
        REGOLE:\n\
        1. Ogni clip deve essere autonoma (avere senso compiuto da sola).\n\
        2. Inizia sempre con un Hook forte (primi 3 secondi) e termina con una frase conclusiva netta.\n\
        3. La durata totale di ogni clip (somma dei suoi segmenti) deve essere compresa tra {min_dur:.0} e {max_dur:.0} secondi.\n\
        4. Se all'interno di un bel discorso c'è una divagazione noiosa, dividi la clip in più `segments` consecutivi saltando gli ID inutili.\n\
        5. Usa ESCLUSIVAMENTE gli `id` interi presenti nella trascrizione per definire `start_id` e `end_id`.{directions_clause}"
    );

    let user_directions_intro = if let Some(directions) = &config.custom_directions {
        let trimmed = directions.trim();
        if !trimmed.is_empty() {
            format!("\nFocus / direzioni specifiche richieste: \"{trimmed}\"\n")
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let user_prompt = format!(
        "Ecco la trascrizione completa indicizzata del video:{user_directions_intro}\n\n{}\n\nGenera ora le migliori clip virali seguendo lo schema JSON richiesto.",
        formatted.formatted_text
    );

    // Gemini 2.5 Flash structured outputs schema
    let response_schema = json!({
        "type": "OBJECT",
        "properties": {
            "video_summary": {
                "type": "STRING",
                "description": "Breve sintesi editoriale dell'intero video"
            },
            "clips": {
                "type": "ARRAY",
                "description": "Lista delle migliori clip virali estratte",
                "items": {
                    "type": "OBJECT",
                    "properties": {
                        "title": {
                            "type": "STRING",
                            "description": "Titolo breve, accattivante ed esplicito per la clip"
                        },
                        "virality_score": {
                            "type": "NUMBER",
                            "description": "Punteggio di viralità previsto da 1 a 100"
                        },
                        "editorial_reason": {
                            "type": "STRING",
                            "description": "Spiegazione strategica del motivo per cui la clip funziona (hook, payoff, ritmo)"
                        },
                        "segments": {
                            "type": "ARRAY",
                            "description": "Array di 1 o più segmenti consecutivi che compongono la clip (multi-cut)",
                            "items": {
                                "type": "OBJECT",
                                "properties": {
                                    "start_id": {
                                        "type": "INTEGER",
                                        "description": "ID del segmento di trascrizione di inizio (incluso)"
                                    },
                                    "end_id": {
                                        "type": "INTEGER",
                                        "description": "ID del segmento di trascrizione di fine (incluso)"
                                    }
                                },
                                "required": ["start_id", "end_id"]
                            }
                        }
                    },
                    "required": ["title", "virality_score", "editorial_reason", "segments"]
                }
            }
        },
        "required": ["video_summary", "clips"]
    });

    let request_body = json!({
        "systemInstruction": {
            "parts": [
                { "text": system_prompt }
            ]
        },
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": user_prompt }
                ]
            }
        ],
        "generationConfig": {
            "responseMimeType": "application/json",
            "responseSchema": response_schema,
            "temperature": 0.3
        }
    });

    let model = if config.model.trim().is_empty() {
        DEFAULT_GEMINI_MODEL
    } else {
        config.model.trim()
    };

    let url =
        format!("https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent");

    let body_bytes = serde_json::to_vec(&request_body)
        .map_err(|e| format!("Impossibile serializzare la richiesta JSON: {e}"))?;

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(120))
        .build();

    let response = agent
        .post(&url)
        .set("Content-Type", "application/json")
        .set("x-goog-api-key", api_key)
        .send_bytes(&body_bytes);

    let response = match response {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, resp)) => {
            let error_text = resp
                .into_string()
                .unwrap_or_else(|_| "Risposta d'errore illeggibile".to_owned());

            // Extract Google's structured error message if available
            if let Ok(json_err) = serde_json::from_str::<serde_json::Value>(&error_text) {
                if let Some(msg) = json_err.pointer("/error/message").and_then(|v| v.as_str()) {
                    return Err(format!("Errore API Gemini (HTTP {code}): {msg}"));
                }
            }

            return Err(format!("Errore API Gemini (HTTP {code}): {error_text}"));
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(format!(
                "Errore di rete durante la chiamata a Gemini: {err}"
            ));
        }
    };

    let response_text = response
        .into_string()
        .map_err(|e| format!("Impossibile leggere la risposta da Gemini: {e}"))?;

    let root_json: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Risposta Gemini non conforme a JSON: {e}"))?;

    let candidate_text = root_json
        .pointer("/candidates/0/content/parts/0/text")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "Nessun contenuto valido ricevuto da Gemini: {}",
                root_json
                    .pointer("/promptFeedback")
                    .map(|f| f.to_string())
                    .unwrap_or_else(|| response_text.chars().take(200).collect())
            )
        })?;

    let parsed_clips: GeminiSmartClipsResponse =
        serde_json::from_str(candidate_text).map_err(|e| {
            format!("Formato JSON structured output non valido: {e} (contenuto: {candidate_text})")
        })?;

    Ok(parsed_clips)
}

// ─────────────────────────────────────────────────────────────────────────────
// STEP C: Post-Processing & Mapper per la Timeline (`mapClipsToTimeline`)
// ─────────────────────────────────────────────────────────────────────────────

/// Maps raw LLM clips with discrete IDs to timeline-ready clips with real media timestamps.
///
/// - Validates start_id and end_id exist and start_id <= end_id.
/// - Applies safety audio padding: -0.1s at start, +0.15s at end, clamped to video bounds.
/// - Sorts clips descending by `virality_score`.
pub fn map_clips_to_timeline(
    raw_clips: &[RawSmartClip],
    segment_map: &HashMap<usize, Segment>,
    video_duration: f64,
) -> Vec<ProcessedSmartClip> {
    let mut processed: Vec<ProcessedSmartClip> = raw_clips
        .iter()
        .filter_map(|raw| {
            let mut mapped_segments = Vec::new();
            let mut clip_total_dur = 0.0f64;

            for cut in &raw.segments {
                if cut.start_id > cut.end_id {
                    continue;
                }
                let Some(first_seg) = segment_map.get(&cut.start_id) else {
                    continue;
                };
                let Some(last_seg) = segment_map.get(&cut.end_id) else {
                    continue;
                };

                // Apply safety margins: -0.1s at beginning and +0.15s at end to avoid cutting words
                let buffered_start = (first_seg.start - 0.1).max(0.0);
                let buffered_end = (last_seg.end + 0.15).min(video_duration);

                let duration = (buffered_end - buffered_start).max(MIN_DURATION);
                clip_total_dur += duration;

                mapped_segments.push(MappedCutSegment {
                    source_start: buffered_start,
                    duration,
                });
            }

            if mapped_segments.is_empty() {
                return None;
            }

            let score = raw.virality_score.round().clamp(1.0, 100.0) as u32;

            Some(ProcessedSmartClip {
                title: raw.title.clone(),
                virality_score: score,
                editorial_reason: raw.editorial_reason.clone(),
                total_duration: clip_total_dur,
                segments: mapped_segments,
            })
        })
        .collect();

    // Sort by virality_score descending (best clips first)
    processed.sort_by(|a, b| b.virality_score.cmp(&a.virality_score));

    processed
}

/// Builds ConCat timeline commands to insert a single smart clip onto a track.
///
/// If multiple segments exist (multi-cut), they are placed consecutively starting at `start_time`.
pub fn build_clip_commands(
    media_id: &str,
    track_id: &str,
    start_time: f64,
    clip: &ProcessedSmartClip,
) -> Vec<Command> {
    let mut commands = Vec::with_capacity(clip.segments.len());
    let mut current_start = start_time;

    for (idx, seg) in clip.segments.iter().enumerate() {
        let label = if clip.segments.len() > 1 {
            format!("{} (Part {})", clip.title, idx + 1)
        } else {
            clip.title.clone()
        };

        commands.push(Command::AddClipSpan {
            media_id: media_id.to_owned(),
            track_id: track_id.to_owned(),
            start: current_start,
            source_start: seg.source_start,
            duration: seg.duration,
            name: Some(label),
            ripple: false,
        });

        current_start += seg.duration;
    }

    commands
}

/// Builds ConCat timeline commands to insert all smart clips sequentially into a track.
pub fn build_all_clips_commands(
    media_id: &str,
    track_id: &str,
    start_time: f64,
    clips: &[ProcessedSmartClip],
) -> Vec<Command> {
    let mut commands = Vec::new();
    let mut current_start = start_time;

    for clip in clips {
        for (idx, seg) in clip.segments.iter().enumerate() {
            let label = if clip.segments.len() > 1 {
                format!("{} (Part {})", clip.title, idx + 1)
            } else {
                clip.title.clone()
            };

            commands.push(Command::AddClipSpan {
                media_id: media_id.to_owned(),
                track_id: track_id.to_owned(),
                start: current_start,
                source_start: seg.source_start,
                duration: seg.duration,
                name: Some(label),
                ripple: false,
            });

            current_start += seg.duration;
        }
    }

    commands
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_transcript() {
        let segments = vec![
            Segment {
                start: 0.0,
                end: 2.5,
                text: "Benvenuti in questo video!".to_string(),
            },
            Segment {
                start: 2.6,
                end: 5.0,
                text: "Oggi vi mostro una novità clamorosa.".to_string(),
            },
        ];

        let formatted = format_transcript_for_llm(&segments);
        assert_eq!(formatted.segment_map.len(), 2);
        assert!(formatted.formatted_text.contains("[ID: 0 | 0.00s - 2.50s]"));
        assert!(formatted.formatted_text.contains("[ID: 1 | 2.60s - 5.00s]"));
        assert!((formatted.total_duration - 5.0).abs() < 1e-4);
    }

    #[test]
    fn test_map_clips_to_timeline() {
        let mut map = HashMap::new();
        map.insert(
            0,
            Segment {
                start: 1.0,
                end: 4.0,
                text: "Intro".to_string(),
            },
        );
        map.insert(
            1,
            Segment {
                start: 4.5,
                end: 8.0,
                text: "Climax".to_string(),
            },
        );

        let raw = vec![RawSmartClip {
            title: "Super Hook".to_string(),
            virality_score: 95.0,
            editorial_reason: "Strong hook".to_string(),
            segments: vec![RawCutSegment {
                start_id: 0,
                end_id: 1,
            }],
        }];

        let mapped = map_clips_to_timeline(&raw, &map, 10.0);
        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].virality_score, 95);
        assert_eq!(mapped[0].segments.len(), 1);
        // Start should be padded: 1.0 - 0.1 = 0.9
        assert!((mapped[0].segments[0].source_start - 0.9).abs() < 1e-4);
        // End should be padded: 8.0 + 0.15 = 8.15. Duration: 8.15 - 0.9 = 7.25
        assert!((mapped[0].segments[0].duration - 7.25).abs() < 1e-4);
    }
}
