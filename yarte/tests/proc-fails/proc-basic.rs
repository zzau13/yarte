use yarte::*;

fn main() {
    let _ = auto!(ywrite_html!(String, "{{ @foo }}"));
}
