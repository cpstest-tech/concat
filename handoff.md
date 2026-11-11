# Handoff: compilare e provare ConCat su questa macchina

Note raccolte durante le modifiche al workflow della timeline e alla schermata iniziale. Sono specifiche del checkout e dell'ambiente Linux usati qui; non sostituiscono le istruzioni generali in `README.md` e `src/README.md`.

## La cosa più importante: la directory Cargo è `src/`

La root Git è `/home/paolo/ConCat-Edit/concat`, ma il workspace Rust e il suo `Cargo.toml` sono in `src/`. Eseguire i comandi Cargo dalla root dà `could not find Cargo.toml`.

```sh
cd /home/paolo/ConCat-Edit/concat/src
cargo fmt --all -- --check
cargo check -p concat
```

Per applicare rustfmt, togliere `--check`. `git diff --check` può essere eseguito dalla root o da `src/`.

## Build locale che ha funzionato

In questo ambiente il build del profilo `release` aveva fallito durante il link per librerie ONNX duplicate (runtime linkato staticamente più runtime dinamico). Per compilare l'app localmente è stato necessario indicare l'ONNX Runtime già presente sul sistema e preferire il link dinamico; il profilo `quick` evita inoltre il costo del profilo di distribuzione.

```sh
cd /home/paolo/ConCat-Edit/concat/src
ORT_LIB_LOCATION=/usr/lib \
ORT_PREFER_DYNAMIC_LINK=1 \
cargo build -p concat --profile quick -j 2
```

Il binario prodotto è:

```text
src/target/quick/concat
```

L'ultimo build con questo comando è terminato con successo. La compilazione può richiedere un paio di minuti anche con la cache calda. Ci sono warning in dipendenze e codice generato (per esempio `concat-media`, `concat-host` e `whisper-rs-sys`); non hanno impedito il build.

Se il percorso ONNX non esiste su un'altra macchina, non copiarlo alla cieca: individuare la libreria ONNX Runtime installata o usare il percorso di build documentato per quella piattaforma. Il percorso qui sopra è locale, non portabile.

## Installazione locale usata

Il launcher in `~/.local/bin/concat` avvia `~/.local/libexec/concat/concat` e imposta `LD_LIBRARY_PATH` verso la stessa directory ONNX Runtime. Per aggiornare il binario senza lasciare un eseguibile scritto a metà, è stato copiato prima su un file temporaneo e poi rinominato:

```sh
install -m 0755 src/target/quick/concat ~/.local/libexec/concat/concat.new
mv ~/.local/libexec/concat/concat.new ~/.local/libexec/concat/concat
~/.local/bin/concat --version
```

Queste directory sono fuori dal checkout: l'installazione richiede permessi appropriati. Non serve disinstallare il launcher; si sostituisce il binario che già usa. Dopo l'aggiornamento occorre riavviare un'istanza dell'app già aperta. La versione mostrata finora è `concat 0.2.8` anche per build locali, quindi `--version` conferma l'avvio ma non distingue le modifiche non rilasciate.

## Verifiche e stato del checkout

- `cargo fmt --all -- --check` e `git diff --check` passano con successo (0 errori).
- Il build `quick` dell'app è passato e il binario è stato installato in `~/.local/libexec/concat/concat`.
- `concat --smoke` è stato eseguito con successo (NVIDIA RTX 2060 Discrete GPU Vulkan attiva, Whisper su CPU AMD Ryzen 7 7700).
- `cargo test -p concat-core` (64 test) e `cargo test -p concat-project` (162 test) passano con successo.
- `cargo test -p concat-effects` (72 test) passa con successo.
- `cargo test -p concat --lib` (162 test passati, inclusi i test su animazioni captions, i18n e cards).
- Risolto il bug delle animazioni che tornavano a "NONE" al click: `clip_commands` in `studio.rs` non includeva il confronto `after.animations != before.animations` per emettere `Command::UpdateClip`.
- Aggiunto il selettore di animazioni (In / Out / Combo, preset testo come word-reveal, typewriter, wave-loop, pop, fade, slide, zoom, e cursore durata) in `TextPanel` (`text-panel.slint`).
- Aggiunte animazioni automatiche ai sottotitoli parola per parola (preset `pop` con rimbalzo dinamico su scala e durata calcolata per ogni singola parola) e aggiunto campo `animations` a `Command::AddTextClip`.
- Risolti errori di compilazione Slint (`captions-editor-closed`, `captions-edit-row` non esposti in `app.slint` e conversione `ModelRc` per `rows` in `panes/captions.rs`).
- Risolto errore "Preview failed: no GPU or software renderer is available to draw with - install Mesa's Vulkan drivers (lavapipe)": `--smoke` armava il tripwire di startup che a causa di `leave(0)` non veniva disarmato, inducendo `safe_start` a bloccare l'uso della GPU NVIDIA GeForce RTX 2060 forzando `softwareGpu: true`. Disattivato l'armamento del tripwire durante `--smoke`, aggiunto fallback hardware trasparente se non esiste rasterizzatore software e ripristinato `softwareGpu: false` in `~/.config/app.concat.editor/settings.json`.

## Dettagli del progetto che avrebbero evitato tentativi

- I crate Rust sono sotto `src/crates/`; l'app grafica è `src/crates/concat` e la UI è scritta in Slint sotto `src/crates/concat/ui/`.
- Le interazioni della schermata iniziale passano da callback Slint in `start.slint` e `app.slint` ai messaggi Rust in `panes/start.rs`. Quando si aggiunge o modifica una callback, verificare tutta la catena: componente, schermata, `app.slint` e handler Rust.
- La preview della Media Pool usa una sorgente temporanea; il flag UI `has-picture` era invece calcolato solo dalle clip della timeline. Per diagnosticare un monitor vuoto, controllare sia il rendering sia lo stato UI che decide se mostrare il placeholder.
- L'ordine delle tracce è bottom-first in alcune operazioni. Non dedurre che `tracks.first()` sia la traccia visivamente superiore: verificare i commenti e i test del modello prima di cambiare l'ordine.
- I comandi pubblicati nel README sono pensati per lo sviluppo generale o per la distribuzione. Per iterare velocemente su questa macchina, `quick` è stato il profilo pratico; per una build di release seguire le istruzioni e i requisiti di linking aggiornati del repository, non assumere che il comando locale sia il packaging ufficiale.

## Animazioni In / Out / Combo: architettura e prossimi passi

- Il progetto salva gli slot in `Clip::animations` (`entrance`, `exit`, `combo`) come preset dichiarativi con `id`, `duration` e `intensity`. `ClipAnimations::tidy` mantiene Combo alternativa a In/Out e limita le due durate perché non si sovrappongano. Le modifiche dell'inspector passano dall'echo/commit dello Studio, così il gesto è raggruppato nell'undo/redo.
- La valutazione dei preset vive in `concat-core/src/animate.rs` (`evaluate_preset` e `PresetId`). Restituisce offset per posizione, scala, rotazione e opacità; `concat-core/src/timeline.rs` li applica sopra curve manuali e proprietà base. Il renderer legge il risultato da `transform_at` / `opacity_at`, quindi preview ed export usano la stessa valutazione.
- Per foto e video, `concat-export/src/flatten.rs` trasferisce i preset e `resolve.rs` li converte nel vocabolario del core. Per aggiungere un preset, aggiornare l'ID e la formula in `PresetId` / `evaluate_preset`, poi esporlo nel pannello Video e nelle traduzioni inglese e italiana.
- Blur non è implementato: il compositor attuale non espone un blur per singolo livello. Gli indicatori di durata sulla timeline sono anch'essi ancora da aggiungere.
- **Prossimo lavoro: animazioni per il testo.** Il modello `Clip` può già contenere gli slot, ma al momento l'inspector Animazioni è mostrato solo a foto e video. I testi vengono esclusi dal flattener normale e rasterizzati separatamente da `concat-host/src/titles.rs`; il loro `ExportClip` non copia ancora `clip.animations` in `preset_animations`. Per attivare le animazioni testo, aggiungere l'accesso UI per `ClipKind::Text`, pubblicare gli stessi campi in `SelectedClipData` e trasferire gli slot nel `ExportClip` del rasterizzatore titoli. Il motore core condiviso può quindi valutarli senza creare una seconda implementazione della preview/export.
- La modifica delle animazioni foto/video è stata scritta senza build né test: va ancora compilata e provata sul checkout. Non considerare verificati Slint, rendering o round-trip del progetto fino a quel controllo.

## Implementazioni recenti (Bug UX, Export, Feature)

### 1. Drag & drop affidabile e diretto sulla timeline
- **Media -> Timeline**: il drag hover non resettava lo stato in caso di uscita prematura o rilascio fuori bersaglio; aggiunta callback `drag-hover-ended` che azzera `state.drop` nello `Studio`, unificando la verifica sincrona dei payload ammessi (`is-timeline-payload`).
- **Spostamento clip in timeline**: avvolto il corpo del `Clip` in `lanes.slint` con un `DragArea` (payload `"clip:{id}"`). Lo `Studio` riconosce il payload e genera un `Command::MoveClips`.

### 2. Import asincrono non bloccante
- `rfd::FileDialog` in `host::after_dialog` bloccava il thread UI durante l'apertura del selettore nativo. Spostato `ask()` su `std::thread::spawn`.
- Il probing dei metadati media ora emette `MediaMsg::ImportProgress(f32)` e visualizza una barra di caricamento progressiva accentata dentro il pulsante Importa (`media-pane.slint`), evitando il freeze del compositor.

### 3. Timeline magnetica per-traccia
- Aggiunto `magnetic: bool` a `Track` in `concat-project/src/model.rs` (default `false`, default `true` solo per la traccia primaria "T1").
- Aggiunto `TrackFlag::Magnetic` e pulsante calamita (`Glyph.magnet`) negli header traccia (`track-header.slint`).
- `close_gaps` e i gesti di ripple trimming/delete ora controllano `track.magnetic` (tramite `studio.is_track_magnetic` / `studio.is_clip_track_magnetic`), salvando la preferenza nel file di progetto.

### 4. Creazione autonoma di tracce (Drop collision avoidance)
- In `studio.rs::place`, se la traccia di destinazione ha già clip nell'intervallo temporale `[start, start + duration)` (`!timeline.is_free(...)`), viene invocato automaticamente `Command::AddTrack { bottom: false }` posizionando il clip sulla nuova traccia.

### 5. Sottotitoli su tutto il montato (Intera Timeline)
- Aggiunta sorgente `"all"` ("Intera timeline") in `panes/captions.rs`.
- Se selezionata, appiattisce l'intera timeline (`concat_export::flatten_timeline`), filtra le tracce audio/video con volume $>0$, genera un mix fedele a tagli e livelli via `concat_media::audio::mix_to_file` e lo invia a Whisper con offset 0.0.
- Alla ricezione dei segmenti, crea in automatico una traccia sottotitoli dedicata in cima alla timeline (`Command::AddTrack`) e vi inserisce i blocchi di testo.

### 6. Rimozione sfondo / Cutout migliorata
- **Smoothing temporale tra frame**: `Mask::blend(a, b, t)` calcola l'interpolazione lineare a virgola fissa tra i keyframe a 10 fps in `store.rs::mask_at` e `store.rs::resolved`, eliminando lo scatto/flicker a 10 fps su video a 30/60 fps.
- **Raffinamento bordi e capelli**: `Mask::refined(&self)` sopprime il rumore di fondo ($<12$), assicura opacità piena sull'interno del soggetto ($>242$) e applica una curva S smoothstep Hermite con anti-aliasing sui contorni di transizione, rimuovendo la scalettatura su capelli e sagome.

### 7. Migliorie Export
- **Bitrate miglior video**: aggiunta opzione "Corrisponde al miglior video del progetto" in `dialogs/export.slint` (`ExportMsg::MatchBestMedia`), che imposta bitrate, risoluzione e framerate in base alla sorgente con qualità più elevata.
- **Anteprima live durante il render**: `concat_export::Reporter` cattura i frame campionati a 640px con cadenza fluida (~4-6 frame/s) prima dell'emissione del progresso, mostrati in grande player dedicato nel dialogo di esportazione a tutto schermo.
- **Schermata fine export e apertura nel programma**: creata vista dedicata a completamento render: titolo "Il video è pronto", grande anteprima visiva, pulsante principale "Apri nell'editor" (carica il video esportato direttamente nel monitor player di ConCat e avvia la riproduzione), pulsante secondario "Apri con player" (player di sistema esterno) e "Mostra nella cartella".

### 8. Perfezionamenti Drag & Drop e Layout Export (Ottobre 2026)
- **Risoluzione bug Slint DnD per spostamento clip**: Slint scarta silenziosamente il drag se `DropArea.can-drop` restituisce un'azione non inclusa nelle azioni consentite da `DragArea` (es. restituire `copy` quando `allow-copy` era `false` su `Clip`). Risolto abilitando sia `allow-copy: true` che `allow-move: true` su `Clip` in `lanes.slint` e facendo restituire `event.proposed-action` sia da `can-drop` che da `dropped`.
- **Creazione automatica nuova traccia trascinando su righello/numeri o poco sopra**: `DropArea` in `lanes.slint` e l'area di drop su `TimelineTray` intercettano i drag posizionati sul righello o poco sopra (`y < ruler-height + 12px` o sulla barra strumenti timeline). Viene attivato `is_new_track = true`. L'interfaccia mostra un'anteprima visiva con corsia dedicata evidenziata `+ Nuova traccia` e ghost del clip con badge "+ Nuova traccia: {nome}". Riportando il cursore verso il basso l'anteprima si toglie all'istante tornando alla traccia ordinaria. Rilasciando in alto viene creata una traccia in cima e piazzato/spostato il clip; rilasciando su una traccia esistente viene posizionato lì senza creare tracce indesiderate (rimosso il vecchio automatismo che creava tracce sulle collisioni ordinarie).
- **Drag & drop dal file manager direttamente sulla timeline**: Collegato il monitoraggio del cursore e degli eventi OS (`HoveredFile`, `CursorMoved`, `DroppedFile` in `DropHandler` e listener Wayland). Durante l'hover del file dal gestore file viene calcolata la posizione esatta sulla timeline tramite `Studio::timeline_drop_target`, mostrando il ghost preview del clip (e l'anteprima nuova traccia se vicino al righello). Al rilascio, il file viene aggiunto ai media e inserito istantaneamente come clip sulla timeline nella posizione e traccia esatte (o su nuova traccia in cima se rilasciato sul righello).
- **Layout di esportazione e apertura video nel programma**: `Measure` espanso a `max: 1800px` con layout responsive. Durante il rendering viene mostrato un player live widescreen ad alta risoluzione (640px) affiancato al pannello di avanzamento. A fine export viene presentata la schermata "Il video è pronto" con pulsante principale "Apri" (carica e avvia la riproduzione del file esportato direttamente nell'editor), "Apri con lettore esterno" e "Mostra nella cartella".

### 9. Upstream Merge 0.2.9 & Miglioramenti Architetturali (Novembre 2026)
- **Allineamento Upstream Completo**: Completato e verificato l'intero processo di merge da upstream `0.2.9` sul branch `paolo/custom-workflow`, mantenendo integra l'architettura personalizzata (tracce magnetiche, transizioni sul taglio `clip.transition_in`, decode Venturi zero-stall, animazioni word-reveal/pop e anteprima export responsive).
- **Importazione bozze CapCut**: Supporto nativo per importare direttamente dalla schermata iniziale i progetti CapCut (`Draft`), traducendo tracce, clip, media e transizioni direttamente sul modello timeline `clip.transition_in`.
- **Silero VAD v6.2.0 & Captions Language Picker**: Integrazione del modello Silero VAD v6.2.0 (`concat-speech`) per segmentazione vocale accurata prima di Whisper; correzione gap timestamp post-pause; aggiunta del menu a discesa per la selezione lingua nei sottotitoli mantenendo l'opzione "Intera timeline" e le animazioni pop parola per parola.
- **Strumenti Timeline Rapidi**:
  - Pulsante forbici "quick-split" fluttuante sulla playhead sopra la clip selezionata.
  - Maniglie interattive fade-in e fade-out audio sugli angoli superiori di ogni clip sulla timeline con feedback visivo.
  - Barra delle tab timeline scorrevole orizzontalmente con pulsante `+` fisso per timeline multiple.
  - Navigazione rapida tra i tagli con Freccia Su / Freccia Giù (`SelectPreviousCut` / `SelectNextCut`).
  - Nuovi preset di aspect ratio `4:5` (Instagram/Facebook) e `3:4`.
- **GPU Upscaler & Pipeline di Render**: Real-ESRGAN basato su compute shader WGSL nativo (`concat-vision`); hardware H.264 encode con failover automatico su software (`libx264`/`libx265`); conversione YUV GPU zero-copy e recovery dinamico in caso di GPU device loss.
- **Robustezza Configurazione**: Salvataggio atomico di `settings.json` con parser resiliente che ripara JSON corrotti o parziali senza azzerare le preferenze dell'utente.

### 10. Drag & Drop delle Transizioni dalla Libreria alla Timeline (Novembre 2026)
- **Causa del malfunzionamento**:
  - In `media-pane.slint`, il componente `ShelfGrid` per la scheda Transizioni (`tab == 3`) non impostava la proprietà `payload-kind: "transition"`. Di conseguenza `CatalogueCard` generava un payload vuoto e `DragArea` non avviava il trascinamento.
  - In `lib.rs`, `Payload.is_timeline_payload` non controllava il prefisso `transition:`, rifiutando a priori il drag (`DragAction.none`).
  - In `studio.rs`, `incoming` e `plan` non gestivano il payload transizione né il calcolo del punto di taglio (seam) più vicino tra clip adiacenti.
  - In `place`, mancava la logica per convertire il rilascio della transizione nell'applicazione su `clip.transition_in` con selezione automatica della transizione.
- **Risoluzione implementata**:
  - Aggiunto `payload-kind: "transition"` a `ShelfGrid` in `media-pane.slint`.
  - Abilitato `transition:` in `Payload.is-timeline-payload` e aggiunto il chip di anteprima cursore con icona `GLYPH_TRANSITION` (`lucide/blend`) in `chips.rs` e `lib.rs`.
  - Implementato `Studio::nearest_cut_for_transition`: ricerca il taglio visivo più vicino sulla traccia entro un raggio tollerante (minimo 60px o 1.5 secondi) tra due clip adiacenti.
  - In `Studio::plan`, se viene trascinata una transizione, il ghost della transizione si ancora e centra simmetricamente sul punto di taglio tra le due clip (`plan.start = cut_time - duration / 2`), bloccando la creazione su tracce nuove/vuote.
  - In `Studio::place`, il rilascio applica la transizione sulla clip di destinazione (`clip.transition_in = Some(Transition { ... })`) con durata adattata, deseleziona i clip e seleziona direttamente la transizione per consentire immediata regolazione nell'Inspector o tramite maniglie.
  - In `Studio::apply_transition` e `transition_cut`, migliorata la tolleranza: se una transizione è già selezionata, un click su un'altra transizione la sostituisce; se la playhead è vicina a un taglio o è selezionata la clip precedente/successiva, la transizione viene applicata senza errori.

### 11. Diagnostica Animazioni & Supporto Multi-Selezione Batch (Novembre 2026)
- **Diagnostica dei Log Runtime**:
  - Nessun errore di rendering, panico o crash GPU nei log (`concat-20261109-205353.log`). Il pipeline matematico di animazione (`concat-core` e `concat-render`) calcola correttamente le trasformazioni (scala, offset, opacità per pop/zoom/slide/fade).
  - Causa percepita di "non succede nulla":
    1. **Frame Skipping all'avvio del playback**: le animazioni generate dai sottotitoli automatici (`pop`) avevano durata di soli 0.18s (~11 frame a 60 fps). L'inizializzazione del decoder NVDEC all'avvio della riproduzione ha causato un'attesa di 333 ms, portando a saltare 18 frame e superando l'intera finestra dell'animazione iniziale prima che il playback si stabilizzasse (effettuando lo scrub o impostando durate standard come 0.5s l'animazione è perfettamente visibile).
    2. **Clip video senza animazioni**: in `Untitled project 10`, le clip video non avevano alcun blocco `animations` assegnato (era presente solo una transizione di taglio).
- **Miglioramento Applicazione Batch Multi-Selezione**:
  - In precedenza `Studio::clip_animation_set` in `studio.rs` considerava solo il primo elemento (`self.selection.first()`), ignorando la selezione multipla.
  - Modificato `Studio::clip_animation_set` per operare su `self.edit_targets()` (o su tutti gli elementi di `self.selection` se vuoto), iterando su tutte le clip selezionate all'interno di `begin_echo()` e `clip_commit()`.
  - Ora, selezionando più sottotitoli o clip e scegliendo un'animazione dall'Inspector, l'animazione viene applicata istantaneamente a tutti i clip selezionati in blocco.
