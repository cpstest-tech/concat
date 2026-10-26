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
ORT_LIB_LOCATION=/home/paolo/.local/share/onnxruntime/onnxruntime-linux-x64-1.28.0/lib \
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

- `cargo fmt --all -- --check` e `git diff --check` sono passati dopo le modifiche più recenti.
- Il build `quick` dell'app è passato.
- `cargo test -p concat-project` aveva passato 162 test dopo le modifiche precedenti al modello e alle operazioni audio. Non è stato rieseguito dopo le ultime modifiche alla UI e alla preview.
- Il checkout contiene modifiche locali non committate provenienti dal lavoro svolto in questa sessione. Prima di scartare o sovrascrivere file, controllare `git status --short` e `git diff`.

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
