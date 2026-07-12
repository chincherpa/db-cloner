# pg-tools

In dieses Verzeichnis legt `scripts/fetch-pg-tools` die PostgreSQL-Client-Tools
(`bin/pg_dump`, `bin/pg_restore` samt Bibliotheken). Der Ordner wird beim
Bundling als Tauri-Resource in die App eingebettet.

Ist der Ordner leer, fällt die App auf ein im PATH installiertes
`pg_dump`/`pg_restore` zurück.
