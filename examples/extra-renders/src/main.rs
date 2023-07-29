#![cfg_attr(nightly, feature(proc_macro_hygiene, stmt_expr_attributes))]
#[cfg(not(nightly))]
compile_error!("not compile without nightly");

use std::io::{stdout, Write};

use uuid::Uuid;
use yarte::{yarte, Buffer, RenderBytes};

#[derive(Clone, Copy)]
struct SomeWithRender {
    foo: usize,
}

impl RenderBytes for SomeWithRender {
    fn render<B: Buffer>(self, buf: &mut B) {
        buf.extend("This number is: ");
        self.foo.render(buf);
    }
}
fn write_str(buffer: String) {
    stdout().lock().write_all(buffer.as_bytes()).unwrap();
}

fn main() {
    let uuid = Uuid::nil();
    let some = SomeWithRender { foo: 2 };

    write_str(
        #[yarte]
        r#"
<div>
    {{~> hello }}
    <p>{{ some }}</p>
</div>
        "#,
    );
}
