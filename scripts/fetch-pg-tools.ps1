# Lädt die PostgreSQL-Client-Tools (pg_dump, pg_restore) nach
# src-tauri\pg-tools\, damit sie als Tauri-Resource mitgebündelt werden.
# Quelle: EDB "binaries only"-ZIP für Windows x64.
$ErrorActionPreference = "Stop"

$EdbVersion = if ($env:EDB_VERSION) { $env:EDB_VERSION } else { "17.5-1" }
$Root = Split-Path -Parent $PSScriptRoot
$Dest = Join-Path $Root "src-tauri\pg-tools"
$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("pgtools-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $Tmp | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $Dest "bin") | Out-Null

$Url = "https://get.enterprisedb.com/postgresql/postgresql-$EdbVersion-windows-x64-binaries.zip"
Write-Host "Lade $Url"
$Zip = Join-Path $Tmp "pg.zip"
Invoke-WebRequest -Uri $Url -OutFile $Zip
Expand-Archive -Path $Zip -DestinationPath $Tmp

# pg_dump/pg_restore brauchen die DLLs aus demselben bin-Verzeichnis.
Copy-Item (Join-Path $Tmp "pgsql\bin\pg_dump.exe") (Join-Path $Dest "bin")
Copy-Item (Join-Path $Tmp "pgsql\bin\pg_restore.exe") (Join-Path $Dest "bin")
Copy-Item (Join-Path $Tmp "pgsql\bin\*.dll") (Join-Path $Dest "bin")

Remove-Item -Recurse -Force $Tmp
Write-Host "Fertig: $Dest"
& (Join-Path $Dest "bin\pg_dump.exe") --version
