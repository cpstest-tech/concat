# Stato del Merge Upstream (`https://github.com/jub0t/concat` -> `paolo/custom-workflow`)

## ✅ COMPLETATO, TESTATO E COMMITTATO
1. **`concat-core`**, **`concat-effects`**, **`concat-text`**, **`concat-project`** (Blocco 1 & 2):
   - `be75d58d`, `c63c7ecf`, `2b594e8e`, `f55451e6`, `ab8607dd`, `20dcd9aa`, `758da5a8`, `64b25e90`, `3298aa08`, `268cd601`, `1d04d147`, `5437d53d`.
2. **`concat-media`** & **`concat-host`** (Blocco 1 & 2 — Passo A + Passo B completati):
   - `7b2d30ea` (Nearest-then-exact scrubbing nel `ReaderPool` + fix riordino B-frame H.264 in `keyframes_only` su `feed.rs` e clamp DTS in `probe.rs`)
   - `ccc6131c`, `7dc91a92`, `8ee4f93e`, `d02b7565`, `1dac19b7` (Poster/Thumbnail/Waveform disk cache)
   - `bac31a3f`, `e8c546ac`, `2b6eb230` (Proxy & Cutout performance)
   - `39ebb75e` (SVG `<defs>` CSS classes & `.svgz`)
   - `75919f5e` (Scrubbing a 60fps `nearest` + `Lane::Refine` in `concat-export`, `concat-host`, `panes/monitor.rs` preservando il decode parallelo multi-layer e la cancellazione dei prefetch obsoleti)
   - `f19fda34` (Idle memory trim `release_freed()` e chiusura reader inattivi)
3. **GPU & Export Pipeline (`concat-vision`, `concat-render`, `concat-export`, `concat-media`) — Passo B COMPLETATO**:
   - `aa38b9d5` + `84c0e625` + `7db37f2a` + `94f4255f`: Upscaler Real-ESRGAN su compute shader GPU (`upscale.rs`, `compact.rs`), riuso frame duplicati ed enhance solo dello span usato della clip (`ReplaceClipMedia { offset }`).
   - `9e69bbcf` + `e938cfdf`: Hardware H.264 encoding (mantenendo `libx264` e `libx265` come software fallback!), thread dedicato per l'encoder (`Writer`), conversione YUV (`NV12`/`I420`) direttamente su GPU (`YUV_SHADER`, `draw_planes`) e pipelining readback multi-frame (`Compositor::submit`/`collect`, `IN_FLIGHT`), preservando l'anteprima live 640px (`Yuv420::thumbnail`) nel dialogo di esportazione.
   - `f0587664`, `359839d7`, `7b29c0f9`, `76988c49`, `fe2be741`, `167a7331`: Soglie VRAM (`MEMORY_BUDGET`), recovery GPU loss senza crash driver, transizioni concatenate senza collisione tracce (`lane_spacing`), `timeline_extent` esatto.

---

## ⏳ COSA MANCA DA FARE ALLA RIPRESA (Passo C)

### Passo C — Captions (Silero VAD + Language Picker) & Miglioramenti UI/Timeline Compatibili (Blocco 4)
1. **`f1a5f4d3` + `2922614c` + `1ffd4f41` + `e2b20c27` + `153cce29`**:
   - Fix timestamp Whisper dopo le pause (`concat-speech`), Silero VAD v6.2.0 (`ggml-silero-v6.2.0.bin`), CPU backend init prima del VAD, e selettore lingua parlata nel pannello Captions (`captions-panel.slint`, `window.slint`, `studio.rs`), **preservando** l'opzione `"all"` ("Intera timeline") e le animazioni `pop` parola per parola.
   - Fix URL archivi Kokoro (`153cce29`) mantenendo intatti Kokoro e Pocket TTS.
2. **UI & Timeline compatibili**:
   - `4b1b2b05` (Scrittura atomica di `settings.json`)
   - `e2be568a` (Blocco scorciatoie globali nei campi di testo e modali)
   - `b3054fae` (Scorrimento automatico cursore in `Field`)
   - `cdbdf074` + `b613ba62` (Stabilità tab Inspector durante playback)
   - `4e1eb116` (Barra tab timeline scorrevole con `+` fisso)
   - `f12b9b2a` (Auto-scroll progressivo ai bordi durante il drag e scrollbar 12px)
   - `a4838588` (Maniglie fade-in/fade-out audio agli angoli delle clip)
   - `bfb39ca4` (Pulsante forbici "quick-split" sulla playhead sopra la clip selezionata)
   - `23897dd8` (Freccia Su/Giù per saltare al taglio precedente/successivo + aspect ratio `4:5` e `3:4`)
   - `25cf5073` (Importazione progetti CapCut direttamente dalla schermata iniziale)
3. **Pulizia finale**:
   - Rimuovere questo file `MERGE_STATUS.md`, aggiornare `handoff.md`, eseguire `cargo test` e `cargo fmt --all -- --check`.
