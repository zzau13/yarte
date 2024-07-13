#![cfg(feature = "json")]
mod json {
    use serde::Serialize;
    use yarte::{Serialize as YSerialize, Template, TemplateText};
    // TODO: remove at helpers
    use yarte_helpers::at_helpers::*;
    #[derive(Serialize, YSerialize, Clone, Copy)]
    struct Json {
        f: usize,
    }

    #[derive(Serialize, YSerialize)]
    struct JsonN {
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

    #[derive(TemplateText)]
    #[template(src = "{{ @json f }}")]
    struct JsonTemplateT {
        f: Json,
    }

    #[derive(TemplateText)]
    #[template(src = "{{ @json_pretty f }}")]
    struct JsonPrettyTemplateT {
        f: Json,
    }

    #[test]
    fn json() {
        let f = Json { f: 1 };
        let t = JsonTemplate { f };
        assert_eq!(serde_json::to_string(&f).unwrap(), t.call().unwrap());

        let t = JsonPrettyTemplate { f };
        assert_eq!(serde_json::to_string_pretty(&f).unwrap(), t.call().unwrap());

        let t = JsonTemplateT { f };
        assert_eq!(serde_json::to_string(&f).unwrap(), t.call().unwrap());

        let t = JsonPrettyTemplateT { f };
        assert_eq!(serde_json::to_string_pretty(&f).unwrap(), t.call().unwrap());
    }

    #[cfg(feature = "bytes-buf")]
    mod bytes_buf {
        use super::*;
        use yarte::{TemplateBytes, TemplateBytesText};

        #[derive(TemplateBytes)]
        #[template(src = "{{ @json f }}")]
        struct JsonTemplateF {
            f: Json,
        }

        #[derive(TemplateBytes)]
        #[template(src = "{{ @json f }}")]
        struct JsonTemplateN {
            f: JsonN,
        }

        #[derive(TemplateBytesText)]
        #[template(src = "{{ @json f }}", print = "code")]
        struct JsonTemplateFT {
            f: Json,
        }

        #[test]
        fn json() {
            let f = Json { f: 1 };
            let t = JsonTemplateF { f };

            assert_eq!(serde_json::to_string(&f).unwrap(), t.ccall::<String>(0));

            let t = JsonTemplateFT { f };
            assert_eq!(serde_json::to_string(&f).unwrap(), t.ccall::<String>(0));

            let t = JsonTemplateN { f: JsonN { f: 1 } };
            assert_eq!(serde_json::to_string(&f).unwrap(), t.ccall::<String>(0));
        }
    }
}
