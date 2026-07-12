use serde::Serialize;
use tokio_postgres::Client;

use crate::config;
use crate::pg::{self, quote_ident};
use crate::secrets;

const TABLES_SQL: &str = "\
SELECT n.nspname, c.relname
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind IN ('r', 'p')
  AND NOT c.relispartition
  AND n.nspname NOT LIKE 'pg\\_%'
  AND n.nspname <> 'information_schema'
ORDER BY 1, 2";

const VIEWS_SQL: &str = "\
SELECT n.nspname, c.relname, c.relkind::text
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind IN ('v', 'm')
  AND n.nspname NOT LIKE 'pg\\_%'
  AND n.nspname <> 'information_schema'
ORDER BY 1, 2";

const FUNCTIONS_SQL: &str = "\
SELECT n.nspname, p.proname
FROM pg_proc p
JOIN pg_namespace n ON n.oid = p.pronamespace
WHERE n.nspname NOT LIKE 'pg\\_%'
  AND n.nspname <> 'information_schema'
  AND NOT EXISTS (
    SELECT 1 FROM pg_depend d
    WHERE d.objid = p.oid AND d.deptype = 'e'
  )
ORDER BY 1, 2";

const POLICIES_SQL: &str = "\
SELECT schemaname, tablename, policyname, cmd
FROM pg_policies
ORDER BY 1, 2, 3";

const SCHEMAS_SQL: &str = "\
SELECT nspname
FROM pg_namespace
WHERE nspname NOT LIKE 'pg\\_%'
  AND nspname <> 'information_schema'
ORDER BY (nspname <> 'public')::int, nspname";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    pub name: String,
    pub rows: i64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PolicyInfo {
    pub table: String,
    pub name: String,
    pub cmd: String,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaInfo {
    pub name: String,
    pub tables: Vec<TableInfo>,
    pub views: Vec<String>,
    pub functions: Vec<String>,
    pub policies: Vec<PolicyInfo>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub pg_version: String,
    pub schemas: Vec<SchemaInfo>,
    pub total_tables: usize,
    pub total_rows: i64,
}

async fn open_client(app: &tauri::AppHandle, id: &str) -> Result<(config::ConnectionMeta, Client), String> {
    let meta = config::get_connection(app, id)?;
    let password = secrets::get_password(id)?;
    let client = pg::connect(&meta, &password).await?;
    Ok((meta, client))
}

/// All regular tables with exact row counts (free-tier DBs are small enough).
pub async fn tables_with_counts(client: &Client) -> Result<Vec<(String, String, i64)>, String> {
    let rows = client
        .query(TABLES_SQL, &[])
        .await
        .map_err(|e| format!("Tabellen konnten nicht gelistet werden: {e}"))?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let schema: String = row.get(0);
        let table: String = row.get(1);
        let count_sql = format!(
            "SELECT count(*) FROM {}.{}",
            quote_ident(&schema),
            quote_ident(&table)
        );
        // -1 = not readable with this role (e.g. some Supabase-internal tables)
        let count = match client.query_one(&count_sql, &[]).await {
            Ok(r) => r.get::<_, i64>(0),
            Err(_) => -1,
        };
        out.push((schema, table, count));
    }
    Ok(out)
}

#[tauri::command]
pub async fn get_overview(app: tauri::AppHandle, id: String) -> Result<Overview, String> {
    let (meta, client) = open_client(&app, &id).await?;
    let err = |e: tokio_postgres::Error| pg::friendly_error(&e.to_string(), &meta);

    let pg_version: String = client
        .query_one("SELECT version()", &[])
        .await
        .map_err(err)?
        .get(0);

    let schema_rows = client.query(SCHEMAS_SQL, &[]).await.map_err(err)?;
    let mut schemas: Vec<SchemaInfo> = schema_rows
        .iter()
        .map(|r| SchemaInfo {
            name: r.get(0),
            tables: vec![],
            views: vec![],
            functions: vec![],
            policies: vec![],
        })
        .collect();

    let find = |schemas: &mut Vec<SchemaInfo>, name: &str| -> Option<usize> {
        schemas.iter().position(|s| s.name == name)
    };

    for (schema, table, rows) in tables_with_counts(&client).await? {
        if let Some(i) = find(&mut schemas, &schema) {
            schemas[i].tables.push(TableInfo { name: table, rows });
        }
    }
    for row in client.query(VIEWS_SQL, &[]).await.map_err(err)? {
        let schema: String = row.get(0);
        if let Some(i) = find(&mut schemas, &schema) {
            schemas[i].views.push(row.get(1));
        }
    }
    for row in client.query(FUNCTIONS_SQL, &[]).await.map_err(err)? {
        let schema: String = row.get(0);
        if let Some(i) = find(&mut schemas, &schema) {
            schemas[i].functions.push(row.get(1));
        }
    }
    for row in client.query(POLICIES_SQL, &[]).await.map_err(err)? {
        let schema: String = row.get(0);
        if let Some(i) = find(&mut schemas, &schema) {
            schemas[i].policies.push(PolicyInfo {
                table: row.get(1),
                name: row.get(2),
                cmd: row.try_get::<_, String>(3).unwrap_or_default(),
            });
        }
    }

    let total_tables = schemas.iter().map(|s| s.tables.len()).sum();
    let total_rows = schemas
        .iter()
        .flat_map(|s| &s.tables)
        .map(|t| t.rows.max(0))
        .sum();

    Ok(Overview {
        pg_version,
        schemas,
        total_tables,
        total_rows,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    pub name: String,
    pub data_type: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TablePage {
    pub columns: Vec<ColumnInfo>,
    pub rows: Vec<Vec<Option<String>>>,
    pub total_rows: i64,
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn get_table_page(
    app: tauri::AppHandle,
    id: String,
    schema: String,
    table: String,
    limit: i64,
    offset: i64,
    sort_column: Option<String>,
    sort_desc: bool,
) -> Result<TablePage, String> {
    let (_meta, client) = open_client(&app, &id).await?;

    let col_rows = client
        .query(
            "SELECT column_name, data_type FROM information_schema.columns\n             WHERE table_schema = $1 AND table_name = $2\n             ORDER BY ordinal_position",
            &[&schema, &table],
        )
        .await
        .map_err(|e| e.to_string())?;
    if col_rows.is_empty() {
        return Err(format!("Tabelle {schema}.{table} wurde nicht gefunden."));
    }
    let columns: Vec<ColumnInfo> = col_rows
        .iter()
        .map(|r| ColumnInfo {
            name: r.get(0),
            data_type: r.get(1),
        })
        .collect();

    let qualified = format!("{}.{}", quote_ident(&schema), quote_ident(&table));
    let total_rows: i64 = client
        .query_one(&format!("SELECT count(*) FROM {qualified}"), &[])
        .await
        .map_err(|e| e.to_string())?
        .get(0);

    // Every column is cast to text so arbitrary types can be displayed.
    let select_list = columns
        .iter()
        .map(|c| format!("{}::text", quote_ident(&c.name)))
        .collect::<Vec<_>>()
        .join(", ");

    let order = match sort_column {
        Some(col) if columns.iter().any(|c| c.name == col) => format!(
            " ORDER BY {} {}",
            quote_ident(&col),
            if sort_desc { "DESC" } else { "ASC" }
        ),
        _ => String::new(),
    };

    let limit = limit.clamp(1, 500);
    let offset = offset.max(0);
    let sql = format!("SELECT {select_list} FROM {qualified}{order} LIMIT {limit} OFFSET {offset}");
    let data_rows = client.query(&sql, &[]).await.map_err(|e| e.to_string())?;

    let rows = data_rows
        .iter()
        .map(|row| {
            (0..columns.len())
                .map(|i| row.try_get::<_, Option<String>>(i).unwrap_or(None))
                .collect()
        })
        .collect();

    Ok(TablePage {
        columns,
        rows,
        total_rows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live test against a local Postgres. Run with:
    /// TEST_DB_URL=postgres://dbcloner:test1234@localhost/dbcloner_test \
    ///   cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn browse_queries_against_live_db() {
        let url = std::env::var("TEST_DB_URL").expect("TEST_DB_URL not set");
        let cfg: tokio_postgres::Config = url.parse().unwrap();
        let (client, conn) = cfg.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(conn);

        let tables = tables_with_counts(&client).await.unwrap();
        assert!(
            tables.iter().any(|(s, t, r)| s == "public" && t == "todos" && *r == 1234),
            "todos mit 1234 Zeilen erwartet, got {tables:?}"
        );
        assert!(tables.iter().any(|(s, t, r)| s == "app" && t == "users" && *r == 42));

        let views = client.query(VIEWS_SQL, &[]).await.unwrap();
        assert!(views.iter().any(|r| r.get::<_, String>(1) == "open_todos"));

        let functions = client.query(FUNCTIONS_SQL, &[]).await.unwrap();
        assert!(functions.iter().any(|r| r.get::<_, String>(1) == "touch"));

        let policies = client.query(POLICIES_SQL, &[]).await.unwrap();
        assert!(policies.iter().any(|r| r.get::<_, String>(2) == "users_self"));

        let schemas = client.query(SCHEMAS_SQL, &[]).await.unwrap();
        assert!(schemas.iter().any(|r| r.get::<_, String>(0) == "app"));
        assert!(!schemas.iter().any(|r| r.get::<_, String>(0) == "information_schema"));
    }
}
