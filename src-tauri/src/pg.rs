//! TLS connection handling for Supabase Postgres.
//!
//! Supabase signs its database certificates with its own CA, so we mirror
//! libpq's `sslmode=require` semantics: the connection is always encrypted,
//! but the certificate chain is not verified against the system trust store.

use std::sync::Arc;
use std::time::Duration;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use tokio_postgres::Client;

use crate::config::ConnectionMeta;

#[derive(Debug)]
struct AcceptAnyCert(Arc<rustls::crypto::CryptoProvider>);

impl ServerCertVerifier for AcceptAnyCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn tls_connector() -> tokio_postgres_rustls::MakeRustlsConnect {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyCert(provider)))
        .with_no_client_auth();
    tokio_postgres_rustls::MakeRustlsConnect::new(config)
}

pub async fn connect(meta: &ConnectionMeta, password: &str) -> Result<Client, String> {
    let mut cfg = tokio_postgres::Config::new();
    cfg.host(&meta.host)
        .port(meta.port)
        .user(&meta.user)
        .password(password)
        .dbname(&meta.dbname)
        .application_name("db-cloner")
        .connect_timeout(Duration::from_secs(15));

    let (client, connection) = cfg
        .connect(tls_connector())
        .await
        .map_err(|e| friendly_error(&e.to_string(), meta))?;

    tauri::async_runtime::spawn(async move {
        let _ = connection.await;
    });
    Ok(client)
}

/// Maps low-level connection errors to actionable German messages,
/// including the Supabase IPv6 trap.
pub fn friendly_error(err: &str, meta: &ConnectionMeta) -> String {
    let lower = err.to_lowercase();
    if lower.contains("password authentication failed") {
        return "Anmeldung fehlgeschlagen: Benutzername oder Passwort ist falsch.".into();
    }
    if meta.host.starts_with("db.")
        && meta.host.ends_with(".supabase.co")
        && (lower.contains("failed to lookup")
            || lower.contains("network unreachable")
            || lower.contains("connection refused")
            || lower.contains("timed out")
            || lower.contains("no route to host"))
    {
        return format!(
            "Verbindung zu {} fehlgeschlagen ({err}). Hinweis: Die Supabase-Direktverbindung ist nur über IPv6 erreichbar. \
             Nutze stattdessen den Session-Pooler-Connection-String (Dashboard → Connect → Session pooler, Host *.pooler.supabase.com, Port 5432).",
            meta.host
        );
    }
    if lower.contains("timed out") {
        return format!("Zeitüberschreitung beim Verbinden mit {}:{}.", meta.host, meta.port);
    }
    format!("Verbindungsfehler: {err}")
}

/// Quotes a SQL identifier (schema, table, column name).
pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('\"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_ident_escapes_quotes() {
        assert_eq!(quote_ident("simple"), "\"simple\"");
        assert_eq!(quote_ident("we\"ird"), "\"we\"\"ird\"");
    }

    /// Verifies the rustls connector still falls back to plaintext against a
    /// non-TLS local server (SslMode::Prefer semantics). Run with:
    /// TEST_PG_HOST=localhost TEST_PG_USER=dbcloner TEST_PG_PASSWORD=test1234 \
    ///   TEST_PG_DB=dbcloner_test cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn connect_against_live_db() {
        let meta = ConnectionMeta {
            host: std::env::var("TEST_PG_HOST").expect("TEST_PG_HOST not set"),
            user: std::env::var("TEST_PG_USER").unwrap_or("postgres".into()),
            dbname: std::env::var("TEST_PG_DB").unwrap_or("postgres".into()),
            ..Default::default()
        };
        let password = std::env::var("TEST_PG_PASSWORD").unwrap_or_default();
        let client = connect(&meta, &password).await.unwrap();
        let row = client.query_one("SELECT 1 + 1", &[]).await.unwrap();
        assert_eq!(row.get::<_, i32>(0), 2);
    }

    #[test]
    fn ipv6_hint_for_direct_supabase_host() {
        let meta = ConnectionMeta {
            host: "db.abcdefgh.supabase.co".into(),
            ..Default::default()
        };
        let msg = friendly_error("error connecting to server: Network unreachable", &meta);
        assert!(msg.contains("Session-Pooler"));
    }
}
