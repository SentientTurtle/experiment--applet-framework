use chrono::{NaiveDate, NaiveDateTime};
use crate::data::Color3;
use crate::dom::{DomElement, element};
use js_sys::{ArrayBuffer, Uint8Array};
use wasm_bindgen_futures::JsFuture;

pub use wasm_bindgen::{JsValue, JsCast};
pub use wasm_bindgen::prelude::Closure;
pub use js_sys::{Function, Reflect};
pub use web_sys::{
    Document,
    DocumentFragment,
    HtmlElement,
    HtmlFormElement,
    FormData
};
pub use wasm_bindgen_futures::spawn_local;

/// Form declaration macro
///
/// In format of:
/// <pre>
/// FormStruct(form_element_id) -> FormDataStruct {
///     submit = "Submit button text".to_string(),
///     input_element_id = InputType {
///         input_field: value
///     },
///     input_2_element_id = InputType {
///         input_field: value
///     },
///     ...
///     input_n_element_id = InputType { ... }
/// }
/// </pre>
///
/// Concrete examples are available in the "applet-example" subproject
#[macro_export]
macro_rules! web_form {
    (
        $form_name:ident($form_id:ident) -> $result_name:ident {
            submit = $submit_value:expr,
            $($input_id:ident = $input:tt {
                $($field:ident: $value:expr),*
            }),+
        }
    ) => {
        /// Macro-generated form struct
        struct $form_name {
            $($input_id: $input),+,
            submit: $crate::form::Submit,
            on_submit_callback: fn($result_name)
        }

        impl $crate::dom::DomElement for $form_name {
            fn to_nodes(&self, document: &$crate::form::Document) -> Result<$crate::form::DocumentFragment, $crate::form::JsValue> {
                let form = $crate::form::JsCast::dyn_into::<$crate::form::HtmlElement>(document.create_element("form")?)
                    .expect("newly created element must be a HtmlElement");
                form.set_attribute("id", stringify!($form_id))?;
                form.set_attribute("onsubmit", "return false;")?;   // Set onsubmit to cancel the form submission; So that our "proper" eventhandler does not have to handle this
                $(form.append_child(&*$crate::dom::DomElement::to_nodes(&self.$input_id, document)?)?;)+
                form.append_child(&*$crate::dom::DomElement::to_nodes(&self.submit, document)?)?;


                let callback = self.on_submit_callback.clone();
                let closure_box: Box<dyn Fn(&$crate::form::JsValue) -> ()> = Box::new(
                    move |event| {
                        let event = event.clone();
                        $crate::form::spawn_local((async move |event: $crate::form::JsValue| {
                            let result: Result<(), $crate::form::JsValue> = try {
                                let target = $crate::form::Reflect::get(&event, &$crate::form::JsValue::from_str("target"))?;
                                let form_data = $crate::form::FormData::new_with_form(
                                    $crate::form::JsCast::unchecked_ref::<$crate::form::HtmlFormElement>(&target)
                                )?;
                                let data = $result_name {
                                    $($input_id: <$input as $crate::form::FormInput>::parse(form_data.get(stringify!($input_id))).await?),+
                                };
                                callback(data)
                            };
                            match result {
                                Ok(()) => (),
                                Err(err) => panic!("error during form submission: {:?}", err)
                            }
                        })(event))
                    });

                let closure = $crate::form::Closure::wrap(closure_box)
                    .into_js_value();

                form.add_event_listener_with_callback(
                    "submit",
                    $crate::form::JsCast::unchecked_ref::<$crate::form::Function>(&closure)  // We can cast Closures to JS Functions
                )?;

                let fragment = document.create_document_fragment();
                fragment.append_child(&*form)?;
                Ok(fragment)
            }
        }

        impl $crate::form::HTMLForm for $form_name {
            type Output = $result_name;

            fn new(on_submit: fn(Self::Output)) -> Self {
                $form_name {
                    submit: Submit {
                        form: stringify!($form_id),
                        name: "submit",
                        value: $submit_value
                    },
                    on_submit_callback: on_submit,
                    $($input_id: $input {
                        form: stringify!($form_id),
                        name: stringify!($input_id),
                        $($field: $value),*
                    },)+
                }
            }
        }

        /// Macro-generated form data struct
        struct $result_name {
            $($input_id: <$input as $crate::form::FormInput>::Output),+
        }
    };
}

/// Trait for HTML Forms generated by the web_form! macro.
pub trait HTMLForm: DomElement {
    /// Submitted data
    type Output;


    /// Constructs a new instance of this form
    ///
    /// # Arguments
    ///
    /// * `on_submit`: Callback for form submission
    ///
    /// returns: Self
    fn new(on_submit: fn(Self::Output)) -> Self;
}

/// Trait for form &lt;input&gt; elements
pub trait FormInput: DomElement {
    /// Rust datatype for this input
    ///
    /// This type should represent all input that validates the HTML-enforced input constraints
    type Output;

    /// Parse the `value` field of the input to rust value
    /// This function may still receive invalid or undefined values, and should not panic on doing so.
    ///
    /// # Arguments
    ///
    /// * `value`: JS value provided by &lt;input&gt; element
    ///
    /// returns: Result<Self::Output, JsValue>
    async fn parse(value: JsValue) -> Result<Self::Output, JsValue>;
}

/// &lt;input type='checkbox'&gt;
pub struct Checkbox {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub default: bool,
}

impl DomElement for Checkbox {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let checkbox = element!(
            document, "input";
            "type" => "checkbox",
            "name" => self.name,
            "id" => &*id
        );
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        if self.default {
            checkbox.set_attribute("checked", "")?;
        }

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*checkbox)?;
        div.append_child(&*label)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for Checkbox {
    type Output = bool;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        Ok(value.eq(&JsValue::from_str("on")))
    }
}

/// &lt;input type='color'&gt;
pub struct Color {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub default: Option<Color3>,
}

impl DomElement for Color {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        let color_picker = element!(
                document, "input";
                "type" => "color",
                "name" => self.name,
                "id" => &*id
            );
        if let Some(default_color) = self.default {
            color_picker.set_attribute("value", &*default_color.as_css_hex())?;
        }

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*label)?;
        div.append_child(&*color_picker)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for Color {
    type Output = Color3;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        value.as_string()
            .as_ref()
            .map(String::as_str)
            .and_then(Color3::parse_from_hex)
            .ok_or(JsValue::from_str("color input value was not valid color"))
    }
}

/// &lt;input type='date'&gt;
pub struct Date {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub default: Option<chrono::NaiveDate>,
    pub min: Option<chrono::NaiveDate>,
    pub max: Option<chrono::NaiveDate>,
}

impl DomElement for Date {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        let date_picker = element!(
            document, "input";
            "type" => "date",
            "name" => self.name,
            "id" => &*id
        );

        if let Some(default_date) = self.default {
            date_picker.set_attribute("value", &*default_date.format("%Y-%m-%d").to_string())?;
        }

        if let Some(minimum_date) = self.min {
            date_picker.set_attribute("min", &*minimum_date.format("%Y-%m-%d").to_string())?;
        }

        if let Some(maximum_date) = self.max {
            date_picker.set_attribute("max", &*maximum_date.format("%Y-%m-%d").to_string())?;
        }

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*label)?;
        div.append_child(&*date_picker)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for Date {
    type Output = chrono::NaiveDate;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        NaiveDate::parse_from_str(
            &*value.as_string().ok_or(JsValue::from_str("date input value was not valid date"))?,
            "%Y-%m-%d",  // TODO: Deal with other possible date formats (And fallback to text input)
        )
            .map_err(|_| JsValue::from_str("date input value was not valid date"))
    }
}

/// &lt;input type='datetime-local'&gt;
pub struct DateTime {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub default: Option<chrono::NaiveDateTime>,
    pub min: Option<chrono::NaiveDateTime>,
    pub max: Option<chrono::NaiveDateTime>,
}

impl DomElement for DateTime {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        let date_picker = element!(
            document, "input";
            "type" => "datetime-local",
            "name" => self.name,
            "id" => &*id
        );

        if let Some(default_date) = self.default {
            date_picker.set_attribute("value", &*default_date.format("%Y-%m-%dT%H:%M").to_string())?;
        }

        if let Some(minimum_date) = self.min {
            date_picker.set_attribute("min", &*minimum_date.format("%Y-%m-%dT%H:%M").to_string())?;
        }

        if let Some(maximum_date) = self.max {
            date_picker.set_attribute("max", &*maximum_date.format("%Y-%m-%dT%H:%M").to_string())?;
        }

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*label)?;
        div.append_child(&*date_picker)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for DateTime {
    type Output = chrono::NaiveDateTime;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        NaiveDateTime::parse_from_str(
            &*value.as_string().ok_or(JsValue::from_str("datetime input value was not valid datetime"))?,
            "%Y-%m-%dT%H:%M",  // TODO: Deal with other possible date formats (And fallback to text input)
        )
            .map_err(|_| JsValue::from_str("datetime input value was not valid datetime"))
    }
}


// pub struct Email;

/// &lt;input type='file'&gt;
///
/// Currently handles no-selected-file by yielding a 0-sized slice
pub struct File {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub accept: &'static str,
    pub multiple: bool,
}

impl DomElement for File {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        let file_select = element!(
            document, "input";
            "type" => "file",
            "id" => &*id,
            "name" => self.name,
            "accept" => self.accept
        );
        if self.multiple {
            file_select.set_attribute("multiple", "")?;
        }

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*label)?;
        div.append_child(&*file_select)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for File {
    type Output = Box<[u8]>;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        let file = value.dyn_into::<web_sys::File>()
            .map_err(|_| JsValue::from_str("file input value was not valid file"))?;
        let buff: ArrayBuffer = JsFuture::from(file.array_buffer())
            .await?
            .dyn_into()
            .expect("array_buffer() must return array buffer");

        let u8_array = Uint8Array::new(&*buff);

        Ok(u8_array.to_vec().into_boxed_slice())
    }
}

// pub struct Hidden;
// pub struct Image;   // Use a regular submit
// pub struct Month;

/// &lt;input type='number'&gt;
pub struct Number {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub default: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
}

impl DomElement for Number {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        let number = element!(
            document, "input";
            "type" => "number",
            "name" => self.name,
            "id" => &*id
        );

        if let Some(default) = self.default {
            number.set_attribute("value", &*format!("{}", default))?;
        }

        if let Some(min) = self.min {
            number.set_attribute("min", &*format!("{}", min))?;
        }

        if let Some(max) = self.max {
            number.set_attribute("max", &*format!("{}", max))?;
        }

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*label)?;
        div.append_child(&*number)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for Number {
    type Output = f64;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        f64::try_from(value)
            .map_err(|_| JsValue::from_str("number input value was not number"))
    }
}

// pub struct Password;

/// &lt;input type='radio'&gt;
///
/// Caution: The data type for this input may return any string, and should not be displayed.
pub struct Radio<const N: usize> {
    pub form: &'static str,
    pub name: &'static str,
    pub label: [String; N],
    pub value: [&'static str; N]    // TODO: Replace value with enum type to remove XSS risk
}

impl<const N: usize> DomElement for Radio<N> {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let fragment = document.create_document_fragment();

        for (label, value) in self.label.iter().zip(&self.value) {
            let id = format!("{}-{}", self.form, self.name);
                let label = element!(
                document, "label";
                "for" => &*id;
                Text = &*label
            );
            let radio = element!(
                document, "input";
                "type" => "file",
                "id" => &*id,
                "name" => self.name,
                "value" => value
            );

            let div = document.create_element("div")?;
            div.set_attribute("class", "form-group")?;
            div.append_child(&*label)?;
            div.append_child(&*radio)?;
            fragment.append_child(&*div)?;
        }
        Ok(fragment)
    }
}

impl<const N: usize>  FormInput for Radio<N> {
    type Output = String;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        value.as_string()
            .ok_or(JsValue::from_str("radio input value was not valid string"))
    }
}

// pub struct Range;
// pub struct Search;

/// &lt;input type='submit'&gt;
///
/// Special case in the web_form! macro, and does not need to be added
pub struct Submit {
    pub form: &'static str,
    pub name: &'static str,
    pub value: String
}

impl DomElement for Submit {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let submit = element!(
            document, "input";
            "type" => "submit",
            "id" => &*format!("{}-{}", self.form, self.name),
            "value" => &*self.value
        );

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*submit)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

// pub struct Telephone;

/// &lt;input type='text'&gt;
///
/// Warning: Provides direct user-input String. Subject to XSS risks
pub struct Text {
    pub form: &'static str,
    pub name: &'static str,
    pub label: String,
    pub value: String
}

impl DomElement for Text {
    fn to_nodes(&self, document: &Document) -> Result<DocumentFragment, JsValue> {
        let id = format!("{}-{}", self.form, self.name);
        let label = element!(
            document, "label";
            "for" => &*id;
            Text = &*self.label
        );
        let text = element!(
            document, "input";
            "type" => "text",
            "id" => &*id,
            "name" => self.name,
            "value" => &*self.value
        );

        let div = document.create_element("div")?;
        div.set_attribute("class", "form-group")?;
        div.append_child(&*label)?;
        div.append_child(&*text)?;
        let fragment = document.create_document_fragment();
        fragment.append_child(&*div)?;
        Ok(fragment)
    }
}

impl FormInput for Text {
    type Output = String;

    async fn parse(value: JsValue) -> Result<Self::Output, JsValue> {
        value.as_string().ok_or(JsValue::from_str("text input value was not string"))
    }
}

// pub struct ClockTime;

// pub struct URL;

// pub struct Week;

