# DB Cloner

Desktop-App (Tauri 2 + Rust + Svelte 5), die Supabase-Datenbanken **vollständig lokal sichert** —
gedacht für den kostenlosen Supabase-Tier, in dem es keine automatischen Backups gibt.

![Stack](https://img.shields.io/badge/Tauri%202-Rust%20%2B%20Svelte%205-3ecf8e)

## Features

- **Verbindungsverwaltung**: Credentials aller Datenbanken in den Einstellungen; Passwörter liegen
  im OS-Schlüsselbund (Windows Credential Manager / macOS Keychain / Linux Keyring), nie im Klartext.
  Connection-String aus dem Supabase-Dashboard einfach einfügen — die App parst ihn automatisch.
- **Datenbank-Browser**: Schemas, Tabellen mit Zeilenzahlen, Views, Funktionen und RLS-Policies auf
  einen Blick; Tabellen-Daten mit Pagination und Sortierung.
- **Backup to Disk**: Ein Klick sichert die Datenbank zu 100 % — Schema, Daten, Views, Funktionen,
  Trigger, Sequenzen und RLS-Policies — per gebündeltem `pg_dump`:
  - `db.dump` — PostgreSQL Custom Format (komprimiert, vollständig, restorefähig)
  - `schema.sql` — lesbares Schema-SQL (diffbar)
  - `manifest.json` — Zeitpunkt, Dauer, Tabellen- und Zeilenzahlen, Dateigrößen
- **Backup-Historie**: alle Backups mit Größe, Dauer und Trigger; im Dateimanager öffnen, löschen.
- **Restore**: Backup per Klick in eine (andere) Datenbank zurückspielen (`pg_restore --clean
  --if-exists --no-owner`), abgesichert durch ein Bestätigungs-Modal.
- **Zeitgesteuerte Backups**: täglich / wöchentlich / alle N Stunden, mit Aufbewahrung („letzte N
  behalten“). Läuft weiter, wenn das Fenster ins Tray minimiert ist; verpasste Termine werden beim
  nächsten Start nachgeholt. OS-Benachrichtigung bei Erfolg/Fehler.
- Live-Fortschritt + Log-Konsole für jedes Backup/Restore, Dark Mode, `Strg+B` = Backup der aktiven DB.

## Setup (Entwicklung)

Voraussetzungen: [Rust](https://rustup.rs), Node.js ≥ 20, unter Linux die
[Tauri-Systempakete](https://tauri.app/start/prerequisites/).

```bash
npm install
./scripts/fetch-pg-tools.sh        # Windows: scripts/fetch-pg-tools.ps1
npm run tauri dev
```

`fetch-pg-tools` legt `pg_dump`/`pg_restore` (PostgreSQL 17) unter `src-tauri/pg-tools/` ab; beim
Bundling wandern sie als Resource in die App. Ohne sie fällt die App auf ein im `PATH`
installiertes `pg_dump` zurück.

Die App-Icons werden beim ersten `tauri dev`/`tauri build` automatisch generiert
(`scripts/gen-icon.mjs` + `tauri icon`); sie liegen nicht im Repository.

Release-Build: `npm run tauri build`

## Supabase verbinden

1. Supabase-Dashboard → dein Projekt → **Connect** (oben) → Tab **Session pooler**
2. Connection-String kopieren (`postgresql://postgres.xxxx:[PASSWORT]@aws-0-….pooler.supabase.com:5432/postgres`)
3. In DB Cloner: **Neue Verbindung** → String einfügen → **Übernehmen** → Passwort ergänzen → **Verbindung testen** → Speichern

> **Wichtig:** Die *Direktverbindung* (`db.<ref>.supabase.co:5432`) ist nur über **IPv6**
> erreichbar. Nutze in IPv4-Netzen den **Session pooler** — er funktioniert auch mit
> `pg_dump`/`pg_restore`. Den *Transaction pooler* (Port 6543) nicht für Backups verwenden.

## Was das Backup enthält — und was nicht

Enthalten ist alles, was `pg_dump` mit der `postgres`-Rolle sehen kann: alle Schemas (auch `auth`
und `storage`), Daten, Views, Materialized Views, Funktionen, Trigger, Sequenzen, Indizes,
Constraints und RLS-Policies.

Bewusst ausgenommen:

- Supabase-interne Schemas, die nicht lesbar/restorefähig sind (`realtime`, `vault`, `pgsodium`, …)
- Datenbank-Rollen samt Passwörtern (`pg_dumpall --roles-only` erfordert Superuser-Rechte)
- Supabase **Storage-Dateien** (Buckets) und die Auth-/Projekt-Konfiguration — sie liegen außerhalb
  von Postgres

Zeitgesteuerte Backups laufen nur, solange die App (ggf. im Tray) geöffnet ist.

## Entwicklung & Tests

```bash
npm run check                            # svelte-check
cd src-tauri
cargo test                               # Unit-Tests (Scheduler, Parser, …)
cargo clippy --all-targets

# Integrationstests gegen ein lokales Postgres:
TEST_DB_URL=postgres://user:pw@localhost/testdb \
TEST_PG_HOST=localhost TEST_PG_USER=user TEST_PG_PASSWORD=pw TEST_PG_DB=testdb \
  cargo test -- --ignored
```

## Architektur

```
src/                   Svelte-5-Frontend (Runes, Tailwind 4)
  lib/api.ts           typisierte Wrapper um die Tauri-Commands
  lib/stores.svelte.ts globaler App-State inkl. Job-/Event-Handling
  lib/components/      Sidebar, DatabaseView, TableGrid, HistoryView, …
src-tauri/src/
  commands/            Tauri-Commands: connections, browse, backup, restore, schedule, settings
  pg.rs                TLS-Verbindung (rustls, sslmode=require-Semantik)
  scheduler.rs         Backup-Zeitpläne (Due-Berechnung unit-getestet)
  tools.rs             findet gebündeltes pg_dump/pg_restore (Resources → PATH)
  secrets.rs           OS-Schlüsselbund (keyring)
scripts/fetch-pg-tools.{sh,ps1}   lädt die PostgreSQL-Client-Tools
```
