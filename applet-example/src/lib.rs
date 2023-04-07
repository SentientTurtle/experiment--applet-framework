#![feature(async_closure)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(try_blocks)]
use std::panic;
use applet_framework::{Applet, applet_entrypoint, web_form};
use applet_framework::dom::DomElement;


#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

use applet_framework::form::{HTMLForm, Text, File, Submit};

web_form!(TestForm(test_form) -> TestInput {
    submit = "HELLO!".to_string(),
    input_data = File {
        accept: ".xml",
        label: "Input file:".to_string(),
        multiple: false
    },
    text = Text {
        label: "TEST2".to_string(),
        value: "Test value".to_string()
    }
});

applet_entrypoint!(TestApplet);
pub struct TestApplet {}

impl Applet for TestApplet {
    fn new() -> Self {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        TestApplet {}
    }

    fn content(&self) -> Box<dyn DomElement> {
        Box::new(TestForm::new(|input| {
            alert(&*format!("Selected file is {} bytes", input.input_data.len()));
        }))
    }
}