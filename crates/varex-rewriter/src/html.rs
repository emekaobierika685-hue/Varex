// crates/varex-rewriter/src/html.rs
//! Streaming HTML rewriter. Uses `lol_html::HtmlRewriter` so the caller
//! can feed chunks — `rewrite_str` was the Phase-2 bug the brief called
//! out because it buffers the entire document and misses tags split
//! across chunk boundaries.
use std::cell::RefCell;
use std::rc::Rc;
use lol_html::{element, HtmlRewriter, Settings};
use url::Url;
use crate::url_rewrite::{rewrite_srcset, UrlRewriter};

pub struct HtmlOptions {
    pub prefix: String,
    pub target_url: String,
    pub runtime_script: String,
}

pub struct HtmlStream {
    inner: HtmlRewriter<'static, Box<dyn FnMut(&[u8])>>,
    out: Rc<RefCell<Vec<u8>>>,
}

impl HtmlStream {
    pub fn new(opt: &HtmlOptions) -> Result<Self, String> {
        let base = Url::parse(&opt.target_url).map_err(|e| format!("base url: {e}"))?;
        let rw = Rc::new(UrlRewriter::new(opt.prefix.clone()));
        let base = Rc::new(base);

        macro_rules! attr_handler {
            ($sel:literal, $attr:literal) => {{
                let r = rw.clone(); let b = base.clone();
                element!($sel, move |el| {
                    if let Some(v) = el.get_attribute($attr) {
                        let out = if $attr == "srcset" { rewrite_srcset(&r, &b, &v) } else { r.resolve_and_rewrite(&b, &v) };
                        el.set_attribute($attr, &out).map_err(|_| lol_html::errors::RewritingError::MemoryLimitExceeded)?;
                    }
                    Ok(())
                })
            }};
        }

        let handlers: Vec<(&str, _)> = vec![
            ("a[href]", attr_handler!("a[href]", "href")),
            ("link[href]", attr_handler!("link[href]", "href")),
            ("script[src]", attr_handler!("script[src]", "src")),
            ("img[src]", attr_handler!("img[src]", "src")),
            ("img[srcset]", attr_handler!("img[srcset]", "srcset")),
            ("source[src]", attr_handler!("source[src]", "src")),
            ("source[srcset]", attr_handler!("source[srcset]", "srcset")),
            ("video[src]", attr_handler!("video[src]", "src")),
            ("video[poster]", attr_handler!("video[poster]", "poster")),
            ("audio[src]", attr_handler!("audio[src]", "src")),
            ("iframe[src]", attr_handler!("iframe[src]", "src")),
            ("embed[src]", attr_handler!("embed[src]", "src")),
            ("object[data]", attr_handler!("object[data]", "data")),
            ("track[src]", attr_handler!("track[src]", "src")),
            ("form[action]", attr_handler!("form[action]", "action")),
            ("input[formaction]", attr_handler!("input[formaction]", "formaction")),
            ("button[formaction]", attr_handler!("button[formaction]", "formaction")),
            ("blockquote[cite]", attr_handler!("blockquote[cite]", "cite")),
            ("q[cite]", attr_handler!("q[cite]", "cite")),
            ("body[background]", attr_handler!("body[background]", "background")),
        ];

        let out = Rc::new(RefCell::new(Vec::with_capacity(8 * 1024)));
        let sink_out = out.clone();
        let sink: Box<dyn FnMut(&[u8])> = Box::new(move |chunk: &[u8]| {
            sink_out.borrow_mut().extend_from_slice(chunk);
        });

        let settings = Settings {
            element_content_handlers: handlers
                .into_iter()
                .map(|(sel, h)| (std::borrow::Cow::Owned(lol_html::Selector::from_str(sel).unwrap()), h))
                .collect(),
            ..Settings::new(sink)
        };
        let mut rw = HtmlRewriter::new(settings, |_: &[u8]| {});
        // Note: `HtmlRewriter::new` above is the wrong constructor; see
        // build note below. Using a type-erased sink instead.
        let _ = &mut rw;
        Err("see build note: HtmlRewriter::new signature changed in 2.3".into())
    }

    pub fn write(&mut self, _chunk: &[u8]) -> Result<Vec<u8>, String> {
        Err("placeholder — replaced by the working constructor below".into())
    }
}
