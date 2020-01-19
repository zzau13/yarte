#![allow(clippy::into_iter_on_ref)]
#![cfg(feature = "wasm")]

use serde::Serialize;
use yarte::Template;

#[derive(Serialize)]
struct Fortune {
    id: i32,
    message: String,
}

#[derive(Template, Serialize)]
#[template(path = "html/fortune.hbs", mode = "iso")]
#[msg(enum Msg {
    Unit
})]
struct WasmServer {
    fortunes: Vec<Fortune>,
}

#[test]
fn wasm_server() {
    let t = WasmServer {
        fortunes: vec![
            Fortune {
                id: 0,
                message: "foo".to_string(),
            },
            Fortune {
                id: 1,
                message: "bar".to_string(),
            },
        ],
    };

    assert_eq!(
        t.call().unwrap(),
        "<!DOCTYPE html><html><head><title>Fortunes</title>\
         <script>function get_state(){return JSON.stringify({\"fortunes\":[{\"id\":0,\"message\":\"foo\"},{\"id\":1,\"message\":\"bar\"}]});}</script>\
         <script type=\"module\">import init from \'./pkg/example.js\';async function run(){await init()}</script>\
         </head><body><table><tr><th>id</th><th>message</th></tr><tr><td>0</td><td>foo</td>\
         </tr><tr><td>1</td><td>bar</td></tr></table></body></html>"
    )
}
