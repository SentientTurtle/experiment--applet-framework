pub use web_sys::{Document, DocumentFragment, HtmlElement};
pub use wasm_bindgen::{JsValue, JsCast};

macro_rules! element {
    ($document:expr, $tag:expr) => {
        $crate::dom::JsCast::dyn_into::<$crate::dom::HtmlElement>($document.create_element($tag)?)
            .expect("newly created element must be a HtmlElement")
    };
    ($document:expr, $tag:expr; $( $attribute:expr => $value:expr ),*) => {
        {
            let element = element!($document, $tag);
            $(element.set_attribute($attribute, $value)?;)*
            element
        }
    };
    ($document:expr, $tag:expr; $( $attribute:expr => $value:expr ),*; Text = $text:expr) => {
        {
            let element = element!($document, $tag);
            $(element.set_attribute($attribute, $value)?;)*
            element.set_inner_text($text);
            element
        }
    };
}

pub(crate) use {element};

/// Trait for DOM/HTML element types
pub trait DomElement {
    /// create the nodes for this type, collected into a document fragment
    ///
    /// provides JsValue error types for error propagation
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue>;
}

/// Source for Applet Style
pub enum AppletStyle {
    /// Static CSS, generally used with include_str! macro
    IncludeFile(&'static str),
    /// Dynamically generated CSS
    String(String),
    /// Default provided CSS, see /rsc/default.css
    DEFAULT,
    /// No style
    NONE
}

impl AppletStyle {
    /// String slice for CSS
    pub fn as_str(&self) -> &str {
        match self {
            AppletStyle::IncludeFile(file) => *file,
            AppletStyle::String(string) => string.as_str(),
            AppletStyle::DEFAULT => include_str!("./../../rsc/default.css"),
            AppletStyle::NONE => ""
        }
    }
}

impl DomElement for AppletStyle {
    /// Creates <style> element with content set through innerText
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        if let AppletStyle::NONE = self {
            Ok(document.create_document_fragment())
        } else {
            let style = element!(document, "style");
            style.set_inner_text(self.as_str());
            let fragment = document.create_document_fragment();
            fragment.append_child(&*style)?;
            Ok(fragment)
        }
    }
}