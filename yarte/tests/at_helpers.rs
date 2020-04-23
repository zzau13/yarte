#![cfg(feature = "json")]
mod json {
    use serde::Serialize;
    use yarte::Template;
    #[derive(Serialize, Clone, Copy)]
    struct Json {
        f: usize,
    }

    #[derive(Template)]
    #[template(src = "{{ @json f }}")]
    struct JsonTemplate {
        f: Json,
    }

    #[derive(Template)]
    #[template(src = "{{ @json_pretty f }}")]
    struct JsonPrettyTemplate {
        f: Json,
    }

    #[test]
    fn json() {
        let f = Json { f: 1 };
        let t = JsonTemplate { f };
        assert_eq!(serde_json::to_string(&f).unwrap(), t.call().unwrap());

        let t = JsonPrettyTemplate { f };
        assert_eq!(serde_json::to_string_pretty(&f).unwrap(), t.call().unwrap());
    }
}
