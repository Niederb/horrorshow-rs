//! # Horrorshow
//!
//! An html templating library.
//!
//! ## Example:
//!
//! ```
//! # #[macro_use] extern crate horrorshow;
//! # fn main() {
//! let actual = html! {
//!     html {
//!         head {
//!             title { : "Hello world!" }
//!         }
//!         body {
//!             // attributes
//!             h1(id="heading") {
//!                 // Insert escaped text
//!                 : "Hello! This is <html />"
//!             }
//!             p {
//!                 // Insert raw text (unescaped)
//!                 : raw!("Let's <i>count</i> to 10!")
//!             }
//!             ol(id="count") {
//!                 // run some inline code...
//!                 |mut tmpl| for i in 0..10 {
//!                     // append to the current template.
//!                     // store output because rust bug #25753
//!                     tmpl = tmpl << html! {
//!                         li {
//!                             // format some text
//!                             #{"{}", i+1 }
//!                         }
//!                     };
//!                 }
//!             }
//!             // You need semi-colons for tags without children.
//!             br; br;
//!             p {
//!                 : "Easy!"
//!             }
//!         }
//!     }
//! }.render();
//!
//! let expected = "<html><head><title>Hello world!</title></head><body><h1 id=\"heading\">Hello! This is &lt;html /&gt;</h1><p>Let's <i>count</i> to 10!</p><ol id=\"count\"><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li><li>6</li><li>7</li><li>8</li><li>9</li><li>10</li></ol><br /><br /><p>Easy!</p></body></html>";
//! assert_eq!(expected, actual);
//!
//! # }
//! ```
//!
//! Or cleaned up:
//!
//! ```html
//! <html>
//!   <head>
//!     <title>Hello world!</title>
//!   </head>
//!   <body>
//!     <h1 id="heading">Hello!</h1>
//!     <p>Let's count to 10!</p>
//!     <ol id="count">
//!       <li>1</li>
//!       <li>2</li>
//!       <li>3</li>
//!       <li>4</li>
//!       <li>5</li>
//!       <li>6</li>
//!       <li>7</li>
//!       <li>8</li>
//!       <li>9</li>
//!       <li>10</li>
//!     </ol>
//!     <br /><br />
//!     <p>Easy!</p>
//!   </body>
//! </html>
//! ```
//!
//! ## Usage
//!
//! Inside an html template, the following expressions are valid:
//!
//! * `some_tag;` -- Insert a the tag `some_tag`.
//!
//! * `some_tag(attr=rust_expresion,...);` -- Insert a the tag `some_tag` with the specified
//!    attributes. The attribute values will be evaluated as rust expressions at runtime.
//!
//! * `some_tag { ... }` -- Insert a the tag `some_tag` and recursivly evaluate the `...`.
//!
//! * `some_tag(...) { ... }` -- Same as above but with custom attributes.
//!
//! * `: rust_expression`, `: { rust_code }` -- Evaluate the expression or block and insert result
//! current position. To insert literal html, mark it as raw with the `raw!` macro.
//!
//! * `#{"format_str", rust_expressions... }` -- Format the arguments according to `format_str` and insert the
//! result at the current position.
//!
//! * `|tmpl| rust_expression`, `|tmpl| { rust_code }` -- Evaluate the expression or block. This is
//! actually a closure so the block/expression can append to the current template through `tmpl`
//! (of type `&mut TemplateBuilder`).
use std::fmt;
use std::io;

#[macro_use]
mod html;


/// A component that can be appended to a template.
///
/// In a perfect world, I'd just use the Display but the string format system is REALLY slow.
pub trait TemplateComponent {
    #[inline]
    fn render_into<'a>(self, tmpl: &mut TemplateBuilder<'a>);
}

/// A template renderer.
pub struct Renderer<F> {
    renderer: F,
    expected_size: usize,
}

impl<F> Renderer<F> where F: FnOnce(&mut TemplateBuilder) {
    /// Render this template into a string.
    #[inline]
    pub fn render(self) -> Result<String, fmt::Error> {
        let mut writer = String::with_capacity(self.expected_size);
        self.render_fmt(&mut writer).and(Ok(writer))
    }

    #[inline]
    pub fn render_fmt(self, writer: &mut fmt::Write) -> Result<(), fmt::Error> {
        let mut tmpl = TemplateBuilder::new_fmt(writer);
        self.render_into(&mut tmpl);
        match tmpl.0 {
            TemplateWriter::Fmt { error, .. } => match error {
                Some(e) => Err(e),
                None => Ok(()),
            },
            _ => panic!(),
        }
    }

    #[inline]
    pub fn render_io(self, writer: &mut io::Write) -> Result<(), io::Error> {
        let mut tmpl = TemplateBuilder::new_io(writer);
        self.render_into(&mut tmpl);
        match tmpl.0 {
            TemplateWriter::Io { error, .. } => match error {
                Some(e) => Err(e),
                None => Ok(()),
            },
            _ => panic!(),
        }
    }
}

impl<F> TemplateComponent for Renderer<F> where F: FnOnce(&mut TemplateBuilder) {
    #[inline]
    fn render_into(self, tmpl: &mut TemplateBuilder) {
        (self.renderer)(tmpl)
    }
}

/// Raw content.
///
/// When rendered, raw content will not be escaped.
pub struct Raw<S: AsRef<str>>(S);

impl<S> Raw<S> where S: AsRef<str> {
    /// Mark as raw.
    #[inline]
    pub fn new(content: S) -> Raw<S> {
        Raw(content)
    }
}

/// Mark a string as a raw. The string will not be rendered.
#[macro_export]
macro_rules! raw {
    ($e:expr) => { $crate::Raw::new($e) }
}

impl<S> TemplateComponent for Raw<S> where S: AsRef<str> {
    #[inline]
    fn render_into(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_raw(self.0.as_ref())
    }
}

impl<'a> TemplateComponent for &'a str {
    #[inline]
    fn render_into(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(self)
    }
}

impl<'a> TemplateComponent for &'a String {
    #[inline]
    fn render_into(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(&self)
    }
}

impl TemplateComponent for String {
    #[inline]
    fn render_into(self, tmpl: &mut TemplateBuilder) {
        tmpl.write_str(&self)
    }
}

impl<'a, 'b, T> std::ops::Shl<T> for &'a mut TemplateBuilder<'b> where T: TemplateComponent {
    type Output = &'a mut TemplateBuilder<'b>;
    /// Render the component into the template.
    ///
    /// Note: If writing to the template fails, this method will neither panic nor return errors.
    /// Instead, no more data will be written to the template and horrorshow abort template
    /// rendering (return an error) when it re-gains control.
    #[inline]
    fn shl(self, component: T) -> &'a mut TemplateBuilder<'b> {
        component.render_into(self);
        self
    }
}

/// TemplateBuilder builder.
pub struct TemplateBuilder<'a>(TemplateWriter<'a>);

enum TemplateWriter<'a> {
    Io {
        writer: &'a mut io::Write,
        error: Option<io::Error>,
    },
    Fmt {
        writer: &'a mut fmt::Write,
        error: Option<fmt::Error>,
    }
}

#[doc(hidden)]
pub fn __new_renderer<F: FnOnce(&mut TemplateBuilder)>(expected_size: usize, f: F) -> Renderer<F> {
    Renderer {
        renderer: f,
        expected_size: expected_size,
    }
}

impl<'a> TemplateBuilder<'a> {
    #[inline]
    fn new_fmt(w: &mut fmt::Write) -> TemplateBuilder {
        TemplateBuilder(TemplateWriter::Fmt { writer: w, error: None })
    }
    #[inline]
    fn new_io(w: &mut io::Write) -> TemplateBuilder {
        TemplateBuilder(TemplateWriter::Io { writer: w, error: None })
    }
    /// Append a raw string to the template.
    #[inline]
    pub fn write_raw(&mut self, text: &str) {
        use TemplateWriter::*;
        use std::fmt::Write;
        match self.0 {
            Io { ref mut writer, ref mut error } => {
                if error.is_some() { return; }
                if let Err(e) = writer.write_all(text.as_bytes()) {
                    *error = Some(e);
                }
            },
            Fmt {ref mut writer, ref mut error } => {
                if error.is_some() { return; }
                // TODO: error::Error not implemented for fmt::Error
                if let Err(e) = writer.write_str(text) {
                    *error = Some(e);
                }
            },
        }
    }

    #[inline]
    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        use std::fmt::Write;
        self.0.write_fmt(args);
    }

    #[inline]
    pub fn write_str(&mut self, text: &str) {
        use std::fmt::Write;
        let _ = self.0.write_str(text);
    }
}

impl<'a> fmt::Write for TemplateWriter<'a> {
    /// Escape and write a string to the template.
    #[inline]
    fn write_str(&mut self, text: &str) -> fmt::Result {
        use TemplateWriter::*;
        match self {
            &mut Io { ref mut writer, ref mut error } => {
                if error.is_some() { return Ok(()); }
                for b in text.bytes() {
                    if let Err(e) = match b {
                        b'&' => writer.write_all("&amp;".as_bytes()),
                        b'"' => writer.write_all("&quot;".as_bytes()),
                        b'<' => writer.write_all("&lt;".as_bytes()),
                        b'>' => writer.write_all("&gt;".as_bytes()),
                        _ => writer.write_all(&[b] as &[u8]),
                    } {
                        *error = Some(e);
                        break;
                    }
                }
            },
            &mut Fmt { ref mut writer, ref mut error } => {
                if error.is_some() { return Ok(()); }
                for c in text.chars() {
                    if let Err(e) = match c {
                        '&' => writer.write_str("&amp;"),
                        '"' => writer.write_str("&quot;"),
                        '<' => writer.write_str("&lt;"),
                        '>' => writer.write_str("&gt;"),
                        _ => writer.write_char(c),
                    } {
                        *error = Some(e);
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

// We shouldn't need this but without it I get the folloowing error:
// error: unexpected token: `an interpolated tt`
#[macro_export]
#[doc(hidden)]
macro_rules! __horrorshow_block_identity {
    ($b:block) => { $b };
}
