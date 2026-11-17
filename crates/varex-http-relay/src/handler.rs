// crates/varex-http-relay/src/handler.rs
use std::sync::Arc;
use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::{HeaderName, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use std::net::SocketAddr;
use tracing::debug;
use varex_core::{ssrf::SsrfPolicy, VarexError};

pub struct RelayState {
    pub client: Arc<super::client::UpstreamClient>,
    pub ssrf: Arc<SsrfPolicy>,
}

pub async fn relay_handler(
    State(st): State<Arc<RelayState>>, ConnectInfo(peer): ConnectInfo<SocketAddr>, req: Request,
) -> Response {
    let target = match req.headers().get("x-varex-target") {
        Some(h) => match h.to_str() { Ok(s) => s.to_string(), Err(_) => return err(StatusCode::BAD_REQUEST, "bad target header") },
        None => return err(StatusCode::BAD_REQUEST, "missing x-varex-target"),
    };
    let method = req.method().clone();
    let body_bytes = if method == Method::HEAD { bytes::Bytes::new() } else {
        match axum::body::to_bytes(req.into_body(), 1024 * 1024 * 1024).await {
            Ok(b) => b, Err(e) => return err(StatusCode::BAD_REQUEST, &format!("body: {e}")),
        }
    };
    match proxy(st, peer, &target, method, body_bytes).await {
        Ok(r) => r,
        Err(e) => {
            debug!(%peer, %target, error=%e, "relay failed");
            let status = match e {
                VarexError::HostDenied(_) | VarexError::PortDenied { .. } | VarexError::AddressBlocked { .. } => StatusCode::FORBIDDEN,
                _ => StatusCode::BAD_GATEWAY,
            };
            err(status, &e.to_string())
        }
    }
}

async fn proxy(
    st: Arc<RelayState>, _peer: SocketAddr, target: &str, method: Method, body: bytes::Bytes,
) -> varex_core::Result<Response> {
    let url: url::Url = target.parse().map_err(|e| VarexError::Config(format!("url: {e}")))?;
    let host = url.host_str().ok_or_else(|| VarexError::Config("no host".into()))?;
    let port = url.port_or_known_default().unwrap_or(443);
    st.ssrf.check_port(port)?;
    st.ssrf.check_hostname(host)?;

    #[cfg(feature = "tls-impersonate")]
    let (status, headers, stream) = {
        use wreq::Body as WBody;
        let mut rb = st.client.inner.request(
            wreq::Method::from_bytes(method.as_str().as_bytes()).map_err(|e| VarexError::Config(e.to_string()))?,
            url.as_str(),
        );
        rb = rb.header("accept", "*/*").header("accept-language", "en-US,en;q=0.9");
        let resp = rb.body(WBody::from(body)).send().await
            .map_err(|e| VarexError::Transport(format!("wreq: {e}")))?;
        let status = resp.status();
        let mut hm = http::HeaderMap::new();
        for (n, v) in resp.headers().iter() { if !strip(n.as_str()) { hm.insert(n.clone(), v.clone()); } }
        hm.insert("x-varex-transport", HeaderValue::from_static("http-relay"));
        let s = resp.bytes_stream().map(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
        (status, hm, Box::pin(s) as std::pin::Pin<Box<dyn futures::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>> + Send>>)
    };
    #[cfg(all(feature = "plain", not(feature = "tls-impersonate")))]
    let (status, headers, stream) = {
        let mut rb = st.client.inner.request(
            reqwest::Method::from_bytes(method.as_str().as_bytes()).map_err(|e| VarexError::Config(e.to_string()))?,
            url.as_str(),
        );
        rb = rb.header("accept", "*/*").header("accept-language", "en-US,en;q=0.9");
        let resp = rb.body(body).send().await
            .map_err(|e| VarexError::Transport(format!("reqwest: {e}")))?;
        let status = resp.status();
        let mut hm = http::HeaderMap::new();
        for (n, v) in resp.headers().iter() { if !strip(n.as_str()) { hm.insert(n.clone(), v.clone()); } }
        hm.insert("x-varex-transport", HeaderValue::from_static("http-relay"));
        let s = resp.bytes_stream().map(|r| r.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
        (status, hm, Box::pin(s) as std::pin::Pin<Box<dyn futures::Stream<Item = std::result::Result<bytes::Bytes, std::io::Error>> + Send>>)
    };

    let mut out = Response::new(Body::from_stream(stream));
    *out.status_mut() = status;
    *out.headers_mut() = headers;
    Ok(out)
}

fn strip(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(),
        "content-security-policy" | "content-security-policy-report-only"
        | "x-frame-options" | "strict-transport-security"
        | "cross-origin-opener-policy" | "cross-origin-embedder-policy"
        | "cross-origin-resource-policy" | "permissions-policy"
        | "content-length" | "transfer-encoding" | "connection"
        | "keep-alive" | "proxy-authenticate" | "proxy-authorization"
        | "te" | "trailer" | "upgrade")
}

fn err(status: StatusCode, msg: &str) -> Response {
    let body = serde_json::json!({ "error": msg }).to_string();
    (status, [(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"))], body).into_response()
}
