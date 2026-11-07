// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! Auto-Highlight / Smart Clips pane controller.
//!
//! Manages the Smart Clips modal UI state, orchestrates transcription and
//! Gemini 2.5 Flash calls, and dispatches timeline commands to add generated
//! clips to the timeline.

use std::sync::Arc;

use concat_project::Command;
use concat_speech::transcribe::Segment;
use slint::SharedString;

use crate::host::{on_ui_in_project, spawn_in_project};
use crate::panes::Msg;
use crate::panes::settings::installed;
use crate::smart_clips::{
    DEFAULT_GEMINI_MODEL, FALLBACK_GEMINI_MODEL, GeminiSmartClipsResponse, ProcessedSmartClip,
    SmartClipsConfig, TargetDurationPreset, build_all_clips_commands, build_clip_commands,
    format_transcript_for_llm, generate_smart_clips, map_clips_to_timeline,
};
use crate::studio::Studio;
use crate::ui::{SmartClipData, SmartClipsSheetData};

/// Source audio/video option for smart clip extraction.
#[derive(Clone, Debug)]
pub struct SmartClipSource {
    pub id: String,
    pub label: String,
    pub media_id: String,
    pub duration: f64,
}

/// Messages for the Smart Clips pane.
#[derive(Clone, Debug)]
pub enum SmartClipsMsg {
    Open,
    Close,
    SourceChanged(i32),
    DurationPresetChanged(i32),
    MaxClipsPresetChanged(i32),
    ModelPresetChanged(i32),
    CustomModelEdited(String),
    CustomDirectionsEdited(String),
    ApiKeyEdited(String),
    BeginAnalysis,
    Cancel,
    TranscriptionProgress(f32, String),
    TranscriptionFinished(Result<Vec<Segment>, String>),
    GeminiFinished(Result<GeminiSmartClipsResponse, String>),
    AddClipToTimeline(i32),
    InsertAllClips,
    ResetAnalysis,
}

/// State for the Smart Clips dialog and analysis pipeline.
#[derive(Default)]
pub struct SmartClipsPane {
    pub open: bool,
    pub running: bool,
    pub stage: String,
    pub progress: f32,
    pub message: String,
    pub source_index: usize,
    /// 0: <30s, 1: 30s-60s, 2: 60s-90s
    pub duration_preset: usize,
    /// 0: 3 clips, 1: 5 clips, 2: 10 clips
    pub max_clips_preset: usize,
    /// 0: gemini-3.8-flash, 1: gemini-3.5-flash-lite, 2: custom
    pub model_preset: usize,
    /// Custom model name if model_preset == 2
    pub custom_model: String,
    /// User specified topic focus, directions or specific moment search
    pub custom_directions: String,
    pub api_key: String,
    pub results: Vec<ProcessedSmartClip>,
    pub media_id: Option<String>,
    pub video_duration: f64,
    /// Cached formatted transcript while analysis runs
    formatted_transcript: Option<crate::smart_clips::FormattedTranscript>,
}

impl SmartClipsPane {
    /// Returns the active Gemini model identifier.
    pub fn resolved_model_name(&self) -> String {
        match self.model_preset {
            0 => DEFAULT_GEMINI_MODEL.to_owned(),
            1 => FALLBACK_GEMINI_MODEL.to_owned(),
            _ => {
                let trimmed = self.custom_model.trim();
                if trimmed.is_empty() {
                    DEFAULT_GEMINI_MODEL.to_owned()
                } else {
                    trimmed.to_owned()
                }
            }
        }
    }

    /// Opens the Smart Clips dialog, loading preferences and identifying sources.
    pub fn open(&mut self, studio: &Studio) {
        self.open = true;
        self.running = false;
        self.stage.clear();
        self.progress = 0.0;
        self.message.clear();

        // Load Gemini API key from preferences or environment
        if self.api_key.trim().is_empty() {
            if let Some(key) = &studio.prefs.gemini_api_key {
                self.api_key = key.clone();
            } else if let Ok(env_key) = std::env::var("GEMINI_API_KEY") {
                self.api_key = env_key;
            }
        }

        // Load model choice from preferences
        if let Some(model) = &studio.prefs.gemini_model {
            match model.as_str() {
                DEFAULT_GEMINI_MODEL => self.model_preset = 0,
                FALLBACK_GEMINI_MODEL => self.model_preset = 1,
                custom => {
                    self.model_preset = 2;
                    self.custom_model = custom.to_owned();
                }
            }
        }
        if let Some(custom) = &studio.prefs.gemini_custom_model {
            self.custom_model = custom.clone();
        }

        // Set selected source to sole selection or default to first
        let sources = get_sources(studio);
        if let Some(sel) = studio.sole_selection() {
            if let Some(idx) = sources.iter().position(|s| s.id == sel) {
                self.source_index = idx;
            }
        }
    }

    /// Handles a message directed to this pane.
    pub fn update(&mut self, msg: SmartClipsMsg, studio: &mut Studio) {
        match msg {
            SmartClipsMsg::Open => {
                self.open(studio);
            }
            SmartClipsMsg::Close => {
                self.open = false;
                self.running = false;
            }
            SmartClipsMsg::SourceChanged(index) => {
                self.source_index =
                    (index.max(0) as usize).min(get_sources(studio).len().saturating_sub(1));
            }
            SmartClipsMsg::DurationPresetChanged(index) => {
                self.duration_preset = (index.max(0) as usize).min(2);
            }
            SmartClipsMsg::MaxClipsPresetChanged(index) => {
                self.max_clips_preset = (index.max(0) as usize).min(2);
            }
            SmartClipsMsg::ModelPresetChanged(index) => {
                self.model_preset = (index.max(0) as usize).min(2);
                let model_name = self.resolved_model_name();
                studio.prefs.gemini_model = Some(model_name);
                studio.prefs.save(&studio.host.dirs);
            }
            SmartClipsMsg::CustomModelEdited(text) => {
                self.custom_model = text.clone();
                studio.prefs.gemini_custom_model = Some(text.clone());
                if self.model_preset == 2 {
                    studio.prefs.gemini_model = Some(text);
                }
                studio.prefs.save(&studio.host.dirs);
            }
            SmartClipsMsg::CustomDirectionsEdited(text) => {
                self.custom_directions = text;
            }
            SmartClipsMsg::ApiKeyEdited(text) => {
                self.api_key = text.clone();
                studio.prefs.gemini_api_key = Some(text);
                studio.prefs.save(&studio.host.dirs);
            }
            SmartClipsMsg::BeginAnalysis => {
                self.begin_analysis(studio);
            }
            SmartClipsMsg::Cancel => {
                studio.host.transcriber.cancel();
                self.running = false;
                self.stage.clear();
                self.progress = 0.0;
            }
            SmartClipsMsg::TranscriptionProgress(progress, stage) => {
                self.progress = progress * 0.5; // Transcription is 0% - 50%
                self.stage = stage;
            }
            SmartClipsMsg::TranscriptionFinished(result) => match result {
                Ok(segments) => {
                    if segments.is_empty() {
                        self.running = false;
                        self.message =
                            "Nessun discorso o voce rilevata nella traccia audio selezionata."
                                .to_owned();
                        return;
                    }
                    self.on_transcript_ready(segments, studio);
                }
                Err(error) => {
                    self.running = false;
                    self.message = format!("Trascrizione non riuscita: {error}");
                }
            },
            SmartClipsMsg::GeminiFinished(result) => {
                self.running = false;
                match result {
                    Ok(response) => {
                        let formatted = self.formatted_transcript.take();
                        if let Some(formatted) = formatted {
                            let mapped = map_clips_to_timeline(
                                &response.clips,
                                &formatted.segment_map,
                                self.video_duration,
                            );
                            if mapped.is_empty() {
                                self.message =
                                    "Gemini non ha estratto clip conformi ai vincoli richiesti."
                                        .to_owned();
                            } else {
                                let count = mapped.len();
                                let model = self.resolved_model_name();
                                self.results = mapped;
                                studio.notify(
                                    &format!("Estratte {count} clip virali con {model}!"),
                                    false,
                                );
                            }
                        }
                    }
                    Err(error) => {
                        self.message = error;
                    }
                }
            }
            SmartClipsMsg::AddClipToTimeline(index) => {
                let idx = index as usize;
                if let Some(clip) = self.results.get(idx).cloned() {
                    let media_id = self.media_id.clone().or_else(|| {
                        get_sources(studio)
                            .get(self.source_index)
                            .map(|s| s.media_id.clone())
                    });

                    let Some(media_id) = media_id else {
                        studio.notify("Media di origine non trovato.", true);
                        return;
                    };

                    // Create a dedicated new track at the top
                    let track_id = studio.apply(Command::AddTrack { bottom: false });
                    let Some(track_id) = track_id else {
                        studio.notify("Impossibile creare una nuova traccia.", true);
                        return;
                    };

                    let commands = build_clip_commands(&media_id, &track_id, 0.0, &clip);
                    if !commands.is_empty() {
                        studio.apply(Command::Batch { commands });
                        studio.notify(
                            &format!("Clip '{}' aggiunta alla timeline!", clip.title),
                            false,
                        );
                        self.open = false;
                    }
                }
            }
            SmartClipsMsg::InsertAllClips => {
                if self.results.is_empty() {
                    return;
                }

                let media_id = self.media_id.clone().or_else(|| {
                    get_sources(studio)
                        .get(self.source_index)
                        .map(|s| s.media_id.clone())
                });

                let Some(media_id) = media_id else {
                    studio.notify("Media di origine non trovato.", true);
                    return;
                };

                let track_id = studio.apply(Command::AddTrack { bottom: false });
                let Some(track_id) = track_id else {
                    studio.notify("Impossibile creare una nuova traccia.", true);
                    return;
                };

                let count = self.results.len();
                let commands = build_all_clips_commands(&media_id, &track_id, 0.0, &self.results);
                if !commands.is_empty() {
                    studio.apply(Command::Batch { commands });
                    studio.notify(&format!("Inserite {count} clip virali in timeline!"), false);
                    self.open = false;
                }
            }
            SmartClipsMsg::ResetAnalysis => {
                self.results.clear();
                self.message.clear();
                self.running = false;
            }
        }
    }

    /// Initiates transcription or immediately runs LLM if subtitles/captions already exist.
    fn begin_analysis(&mut self, studio: &mut Studio) {
        if self.api_key.trim().is_empty() {
            self.message = "Inserisci una chiave API di Gemini valida.".to_owned();
            return;
        }

        let sources = get_sources(studio);
        let Some(source) = sources.get(self.source_index).cloned() else {
            self.message = "Seleziona una clip video/audio valida dalla timeline.".to_owned();
            return;
        };

        self.media_id = Some(source.media_id.clone());
        self.video_duration = source.duration.max(1.0);
        self.message.clear();
        self.running = true;
        self.progress = 0.05;

        // Check if captions or subtitles already exist on the timeline!
        let title_cues = concat_project::srt::title_cues(studio.timeline(), None);
        if !title_cues.is_empty() {
            log::info!(
                "smart_clips: using {} existing subtitle cues from timeline",
                title_cues.len()
            );
            let segments: Vec<Segment> = title_cues
                .into_iter()
                .map(|cue| Segment {
                    start: cue.start,
                    end: cue.end,
                    text: cue.text,
                })
                .collect();
            self.on_transcript_ready(segments, studio);
            return;
        }

        // Otherwise transcribe the clip's audio using Whisper
        let Some(clip) = studio.clip(&source.id).cloned() else {
            self.message = "Clip non più presente nella timeline.".to_owned();
            self.running = false;
            return;
        };

        let Some(media) = studio.project().media_by_id(&clip.media_id).cloned() else {
            self.message = "File multimediale associato non trovato.".to_owned();
            self.running = false;
            return;
        };

        let installed_models = installed(&studio.settings.transcribers);
        let Some(model) = installed_models
            .iter()
            .find(|m| m.active)
            .or_else(|| installed_models.first())
        else {
            self.message =
                "Nessun modello Whisper installato. Scaricalo da Impostazioni > Trascrizione."
                    .to_owned();
            self.running = false;
            return;
        };

        self.stage = "Trascrizione vocale in corso con Whisper...".to_owned();

        let request = concat_speech::transcribe::TranscribeRequest {
            path: media.path,
            audio_stream: clip.audio_stream,
            source_start: clip.source_start,
            window: clip.duration * clip.speed,
            model_id: model.id.clone(),
            word_timestamps: false,
            language: studio.prefs.caption_language.clone(),
        };

        let dirs = studio.host.dirs.clone();
        let transcriber = Arc::clone(&studio.host.transcriber);
        let epoch = crate::host::project_epoch();

        spawn_in_project(
            move || {
                transcriber.transcribe(&dirs, &request, move |_stage| {
                    on_ui_in_project(epoch, move |studio, _, _| {
                        studio.handle(Msg::SmartClips(SmartClipsMsg::TranscriptionProgress(
                            0.3,
                            "Trascrizione in corso...".to_owned(),
                        )));
                    });
                })
            },
            |studio, _, _, result| {
                studio.handle(Msg::SmartClips(SmartClipsMsg::TranscriptionFinished(
                    result,
                )));
            },
        );
    }

    /// Called once transcription segments are available: launches the Gemini query.
    fn on_transcript_ready(&mut self, segments: Vec<Segment>, _studio: &mut Studio) {
        let formatted = format_transcript_for_llm(&segments);
        if formatted.segment_map.is_empty() {
            self.running = false;
            self.message =
                "Nessun segmento di testo valido estratto dalla trascrizione.".to_owned();
            return;
        }

        let model = self.resolved_model_name();
        self.stage = format!("Analisi narrativa e viralità con {model}...");
        self.progress = 0.6;

        let preset = match self.duration_preset {
            0 => TargetDurationPreset::Short,
            1 => TargetDurationPreset::Medium,
            _ => TargetDurationPreset::Long,
        };

        let max_clips = match self.max_clips_preset {
            0 => 3,
            1 => 5,
            _ => 10,
        };

        let custom_directions = if self.custom_directions.trim().is_empty() {
            None
        } else {
            Some(self.custom_directions.trim().to_owned())
        };

        let config = SmartClipsConfig {
            max_clips,
            duration_preset: preset,
            video_duration: self.video_duration,
            custom_directions,
            model,
        };

        self.formatted_transcript = Some(formatted.clone());
        let api_key = self.api_key.clone();

        spawn_in_project(
            move || generate_smart_clips(&api_key, &formatted, &config),
            |studio, _, _, result| {
                studio.handle(Msg::SmartClips(SmartClipsMsg::GeminiFinished(result)));
            },
        );
    }

    /// Publishes data to the Slint UI model.
    pub fn data(&self, studio: &Studio) -> SmartClipsSheetData {
        let sources = get_sources(studio);
        let source_labels: Vec<SharedString> = sources
            .iter()
            .map(|s| SharedString::from(s.label.as_str()))
            .collect();

        let clips_data: Vec<SmartClipData> = self
            .results
            .iter()
            .enumerate()
            .map(|(idx, clip)| SmartClipData {
                index: idx as i32,
                title: SharedString::from(clip.title.as_str()),
                virality_score: clip.virality_score as i32,
                editorial_reason: SharedString::from(clip.editorial_reason.as_str()),
                duration_seconds: clip.total_duration as f32,
                duration_formatted: SharedString::from(format_duration(clip.total_duration)),
                segment_count: clip.segments.len() as i32,
            })
            .collect();

        SmartClipsSheetData {
            open: self.open,
            sources: slint::ModelRc::new(slint::VecModel::from(source_labels)),
            source: self.source_index as i32,
            duration_preset: self.duration_preset as i32,
            max_clips_preset: self.max_clips_preset as i32,
            model_preset: self.model_preset as i32,
            custom_model: SharedString::from(self.custom_model.as_str()),
            custom_directions: SharedString::from(self.custom_directions.as_str()),
            api_key: SharedString::from(self.api_key.as_str()),
            running: self.running,
            stage: SharedString::from(self.stage.as_str()),
            progress: self.progress,
            message: SharedString::from(self.message.as_str()),
            has_results: !self.results.is_empty(),
            clips: slint::ModelRc::new(slint::VecModel::from(clips_data)),
        }
    }
}

/// Helper to get all sounding clips from timeline.
fn get_sources(studio: &Studio) -> Vec<SmartClipSource> {
    let project = studio.project();
    let mut out = Vec::new();

    for clip in &studio.timeline().clips {
        if studio.clip_has_sound(clip) {
            let media_name = project
                .media_by_id(&clip.media_id)
                .map_or(clip.name.as_str(), |m| m.name.as_str());
            let label = format!("{media_name} (start: {:.1}s)", clip.start);
            out.push(SmartClipSource {
                id: clip.id.clone(),
                label,
                media_id: clip.media_id.clone(),
                duration: clip.duration,
            });
        }
    }

    out
}

/// Formats seconds into human-readable duration `XXm YYs` or `XXs`.
fn format_duration(seconds: f64) -> String {
    let s = seconds.max(0.0).round() as u64;
    let mins = s / 60;
    let secs = s % 60;
    if mins > 0 {
        format!("{mins}m {secs:02}s")
    } else {
        format!("{secs}s")
    }
}
