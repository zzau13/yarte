#![cfg(feature = "json")]
mod json {
    use serde::Serialize;
    use yarte::{Template, TemplateText};
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

    #[cfg(feature = "fixed")]
    mod fixed {
        use super::*;
        use std::mem::MaybeUninit;
        use yarte::{TemplateFixed, TemplateFixedText};

        #[derive(TemplateFixed)]
        #[template(src = "{{ @json f }}")]
        struct JsonTemplateF {
            f: Json,
        }

        #[derive(TemplateFixed)]
        #[template(src = "{{ @json_pretty f }}")]
        struct JsonPrettyTemplateF {
            f: Json,
        }

        #[derive(TemplateFixedText)]
        #[template(src = "{{ @json f }}", print = "code")]
        struct JsonTemplateFT {
            f: Json,
        }

        #[derive(TemplateFixedText)]
        #[template(src = "{{ @json_pretty f }}")]
        struct JsonPrettyTemplateFT {
            f: Json,
        }

        #[test]
        fn json() {
            let f = Json { f: 1 };
            let t = JsonTemplateF { f };

            let mut buf = [MaybeUninit::uninit(); 1024];
            assert_eq!(
                serde_json::to_string(&f).unwrap().as_bytes(),
                t.call(&mut buf).unwrap()
            );

            let t = JsonPrettyTemplateF { f };
            let mut buf = [MaybeUninit::uninit(); 1024];
            assert_eq!(
                serde_json::to_string_pretty(&f).unwrap().as_bytes(),
                t.call(&mut buf).unwrap()
            );

            let t = JsonTemplateFT { f };
            let mut buf = [MaybeUninit::uninit(); 1024];
            assert_eq!(
                serde_json::to_string(&f).unwrap().as_bytes(),
                t.call(&mut buf).unwrap()
            );

            let t = JsonPrettyTemplateFT { f };
            let mut buf = [MaybeUninit::uninit(); 1024];
            assert_eq!(
                serde_json::to_string_pretty(&f).unwrap().as_bytes(),
                t.call(&mut buf).unwrap()
            );
        }
    }
}
