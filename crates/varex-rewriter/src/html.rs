// crates/varex-rewriter/src/html.rs (corrected)
use std::cell::RefCell;
use std::rc::Rc;
use lol_html::{element, HtmlRewriter, Settings, Selector};
use url::Url;
use crate::url_rewrite::{rewrite_srcset, UrlRewriter};

pub struct HtmlOptions { pub prefix: String, pub target_url: String, pub runtime_script: String }

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

        let mut handlers: Vec<(std::borrow::Cow<'static, Selector>, _)> = Vec::new();
        handlers.push(("a[href]".into(), attr_handler!("a[href]", "href")));
        handlers.push(("link[href]".into(), attr_handler!("link[href]", "href")));
        handlers.push(("script[src]".into(), attr_handler!("script[src]", "src")));
        handlers.push(("img[src]".into(), attr_handler!("img[src]", "src")));
        handlers.push(("img[srcset]".into(), attr_handler!("img[srcset]", "srcset")));
        handlers.push(("source[src]".into(), attr_handler!("source[src]", "src")));
        handlers.push(("source[srcset]".into(), attr_handler!("source[srcset]", "srcset")));
        handlers.push(("video[src]".into(), attr_handler!("video[src]", "src")));
        handlers.push(("video[poster]".into(), attr_handler!("video[poster]", "poster")));
        handlers.push(("audio[src]".into(), attr_handler!("audio[src]", "src")));
        handlers.push(("iframe[src]".into(), attr_handler!("iframe[src]", "src")));
        handlers.push(("embed[src]".into(), attr_handler!("embed[src]", "src")));
        handlers.push(("object[data]".into(), attr_handler!("object[data]", "data")));
        handlers.push(("track[src]".into(), attr_handler!("track[src]", "src")));
        handlers.push(("form[action]".into(), attr_handler!("form[action]", "action")));
        handlers.push(("input[formaction]".into(), attr_handler!("input[formaction]", "formaction")));
        handlers.push(("button[formaction]".into(), attr_handler!("button[formaction]", "formaction")));
        handlers.push(("blockquote[cite]".into(), attr_handler!("blockquote[cite]", "cite")));
        handlers.push(("q[cite]".into(), attr_handler!("q[cite]", "cite")));
        handlers.push(("body[background]".into(), attr_handler!("body[background]", "background")));

        let out = Rc::new(RefCell::new(Vec::<u8>::with_capacity(8 * 1024)));
        let sink_out = out.clone();
        let settings = Settings {
            element_content_handlers: handlers,
            ..Settings::new(Box::new(move |chunk: &[u8]| {
                sink_out.borrow_mut().extend_from_slice(chunk);
            }))
        };
        let inner = HtmlRewriter::new(settings, |_: &[u8]| {});
        Ok(Self { inner, out })
    }

    pub fn write(&mut self, chunk: &[u8]) -> Result<Vec<u8>, String> {
        self.inner.write(chunk).map_err(|e| format!("html write: {e}"))?;
        Ok(std::mem::take(&mut self.out.borrow_mut()))
    }

    pub fn end(&mut self) -> Result<Vec<u8>, String> {
        self.inner.end().map_err(|e| format!("html end: {e}"))?;
        Ok(std::mem::take(&mut self.out.borrow_mut()))
    }
}
