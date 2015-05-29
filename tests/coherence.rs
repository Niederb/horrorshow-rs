#[macro_use]
extern crate horrorshow;

use horrorshow::{RenderOnce, Render, RenderMut, TemplateBuilder};
use horrorshow::prelude::*;

struct Test;

impl RenderOnce for Test {
    fn render_once(self, t: &mut TemplateBuilder) {
        self.render(t);
    }
}


impl RenderMut for Test {
    fn render_mut(&mut self, t: &mut TemplateBuilder) {
        self.render(t);
    }
}


impl Render for Test {
    fn render(&self, t: &mut TemplateBuilder) {
        t.write_str("Test");
    }
}

#[test]
fn test_coherence() {
    assert_eq!((html! {
        |t| t << Test;
        |t| t << &mut Test;
        |t| t << &Test;
    }).into_string(), "TestTestTest");
}
