#![feature(async_closure)]
#![feature(async_fn_in_trait)]
#![feature(concat_idents)]

pub mod dom;
pub mod form;
pub mod data;

/// Module containing applet-initialisation logic
///
/// applet_entrypoint! macro handles all initialisation logic for end users,
pub mod applet_init {
    use crate::Applet;
    pub use web_sys::ShadowRoot;
    pub use wasm_bindgen::JsValue;
    use crate::dom::DomElement;

    /// Macro to define entrypoint for applet struct
    ///
    /// Usage: Takes a type implementing the Applet trait
    ///
    /// Note: Only one entrypoint may exists
    #[macro_export]
    macro_rules! applet_entrypoint {
        ($applet:ty) => {
            use wasm_bindgen::prelude::wasm_bindgen;
            /// Entrypoint for Applet, only one may exist
            #[wasm_bindgen]
            pub fn __applet_entrypoint(root: $crate::applet_init::ShadowRoot) -> Result<(), $crate::applet_init::JsValue> {
                $crate::applet_init::init::<$applet>(root)
            }
        };
    }

    /// Applet initialisation function, generally used indirectly through the applet_entrypoint! macro
    ///
    /// An __applet_entrypoint function matching this function's signature must be exported in the WASM-binary
    /// If not using the applet_entrypoint! a wrapper function must be generated to select the applet type through generic type T
    ///
    /// # Arguments
    ///
    /// * `root`: Shadowroot in which the applet is loaded
    ///
    /// returns: Result<(), JsValue>
    pub fn init<T: Applet>(root: ShadowRoot) -> Result<(), JsValue> {
        let window = web_sys::window().expect("applet must be initialised within browser window");
        let document = window.document().expect("window must have document");
        let applet = T::new();

        let style = applet.style().to_nodes(&document)?;
        let content = applet.content()
            .to_nodes(&document)?;

        root.append_child(&*style)?;
        root.append_child(&*content)?;
        Ok(())
    }
}

use crate::dom::{AppletStyle, DomElement};

/// Trait for applets
pub trait Applet {
    fn new() -> Self;
    /// HTML-content of the applet. Currently only called once and not refreshed
    fn content(&self) -> Box<dyn DomElement>;
    /// CSS Style of the element, minimal default provided
    fn style(&self) -> AppletStyle {
        AppletStyle::DEFAULT
    }
}