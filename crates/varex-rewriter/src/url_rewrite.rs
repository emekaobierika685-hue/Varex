// crates/varex-rewriter/src/url_rewrite.rs
use url::Url;

const PASSTHROUGH: &[&str] = &["data:", "blob:", "javascript:", "mailto:", "about:", "tel:", "sms:"];

pub struct UrlRewriter { pub prefix: String }
impl UrlRewriter {
    pub fn new(prefix: impl Into<String>) -> Self { Self { prefix: prefix.into().trim_end_matches('/').to_string() } }
    pub fn rewrite_absolute(&self, abs: &str) -> String { format!("{}/{}", self.prefix, abs) }
    pub fn resolve_and_rewrite(&self, base: &Url, rel: &str) -> String {
        let trimmed = rel.trim();
        if trimmed.is_empty() { return String::new(); }
        let lower = trimmed.to_ascii_lowercase();
        if PASSTHROUGH.iter().any(|p| lower.starts_with(p)) || trimmed.starts_with('#') { return trimmed.to_string(); }
        match base.join(trimmed) { Ok(abs) => self.rewrite_absolute(abs.as_str()), Err(_) => trimmed.to_string() }
    }
}
pub fn rewrite_srcset(rw: &UrlRewriter, base: &Url, srcset: &str) -> String {
    let mut out = String::with_capacity(srcset.len());
    for candidate in srcset.split(',') {
        let candidate = candidate.trim(); if candidate.is_empty() { continue; }
        let mut parts = candidate.splitn(2, char::is_whitespace);
        let url = parts.next().unwrap_or(""); let descriptor = parts.next().unwrap_or("");
        if !out.is_empty() { out.push_str(", "); }
        out.push_str(&rw.resolve_and_rewrite(base, url));
        if !descriptor.is_empty() { out.push(' '); out.push_str(descriptor); }
    }
    out
}
