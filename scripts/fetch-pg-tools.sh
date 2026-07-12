#!/usr/bin/env bash
# Lädt die PostgreSQL-Client-Tools (pg_dump, pg_restore) nach
# src-tauri/pg-tools/, damit sie als Tauri-Resource mitgebündelt werden.
#
# Linux:  nutzt die zonky embedded-postgres-binaries von Maven Central
# macOS:  nutzt die EDB "binaries only"-ZIPs
# (Windows: scripts/fetch-pg-tools.ps1 verwenden)
set -euo pipefail

PG_VERSION="${PG_VERSION:-17.5.0}"
EDB_VERSION="${EDB_VERSION:-17.5-1}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$ROOT/src-tauri/pg-tools"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$DEST"

case "$(uname -s)" in
  Linux)
    ARCH="$(uname -m)"
    case "$ARCH" in
      x86_64) ZARCH="amd64" ;;
      aarch64) ZARCH="arm64v8" ;;
      *) echo "Nicht unterstützte Architektur: $ARCH" >&2; exit 1 ;;
    esac
    URL="https://repo1.maven.org/maven2/io/zonky/test/postgres/embedded-postgres-binaries-linux-${ZARCH}/${PG_VERSION}/embedded-postgres-binaries-linux-${ZARCH}-${PG_VERSION}.jar"
    echo "Lade $URL"
    curl -fL "$URL" -o "$TMP/pg.jar"
    (cd "$TMP" && unzip -q pg.jar && tar -xJf postgres-linux-*.txz -C "$TMP" 2>/dev/null || tar -xJf "$TMP"/postgres-*.txz -C "$TMP")
    mkdir -p "$DEST/bin" "$DEST/lib"
    cp "$TMP/bin/pg_dump" "$TMP/bin/pg_restore" "$DEST/bin/"
    cp -r "$TMP/lib/." "$DEST/lib/"
    ;;
  Darwin)
    URL="https://get.enterprisedb.com/postgresql/postgresql-${EDB_VERSION}-osx-binaries.zip"
    echo "Lade $URL"
    curl -fL "$URL" -o "$TMP/pg.zip"
    unzip -q "$TMP/pg.zip" -d "$TMP"
    mkdir -p "$DEST/bin" "$DEST/lib"
    cp "$TMP/pgsql/bin/pg_dump" "$TMP/pgsql/bin/pg_restore" "$DEST/bin/"
    cp -r "$TMP/pgsql/lib/." "$DEST/lib/"
    ;;
  *)
    echo "Für Windows bitte scripts/fetch-pg-tools.ps1 verwenden." >&2
    exit 1
    ;;
esac

chmod +x "$DEST/bin/"* 2>/dev/null || true
echo "Fertig: $DEST"
"$DEST/bin/pg_dump" --version
