// crates/varex-rewriter/src/lib.rs
//! Public rewriter API. Phase 0 ships the URL resolver and the crate
//! skeleton so the workspace builds; Phase 2 wires the streaming
//! HTML/CSS passes.
pub mod html;
pub mod url_rewrite;

pub use html::{HtmlOptions, HtmlStream};
pub use url_rewrite::{rewrite_srcset, UrlRewriter};

#[cfg(target_arch = "wasm32")]
mod wasm_api {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen]
    pub struct Rewriter { prefix: String, target_url: String, runtime_script: String }
    #[wasm_bindgen]
    impl Rewriter {
        #[wasm_bindgen(constructor)]
        pub fn new(prefix: String, target_url: String, runtime_script: String) -> Self {
            Self { prefix, target_url, runtime_script }
        }
        pub fn rewrite_url(&self, rel: &str) -> String {
            let base = match url::Url::parse(&self.target_url) { Ok(u) => u, Err(_) => return rel.into() };
            super::UrlRewriter::new(self.prefix.clone()).resolve_and_rewrite(&base, rel)
        }
    }
}
