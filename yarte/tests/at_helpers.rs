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
        #[template(src = "{{ @json f }}")]
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
            let mut buf: [u8; 1024] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            let b = t.call(&mut buf).unwrap();
            assert_eq!(serde_json::to_vec(&f).unwrap(), buf[..b].to_vec());

            let t = JsonPrettyTemplateF { f };
            let mut buf: [u8; 1024] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            let b = t.call(&mut buf).unwrap();
            assert_eq!(serde_json::to_vec_pretty(&f).unwrap(), buf[..b].to_vec());

            let t = JsonTemplateFT { f };
            let mut buf: [u8; 1024] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            let b = t.call(&mut buf).unwrap();
            assert_eq!(serde_json::to_vec(&f).unwrap(), buf[..b].to_vec());

            let t = JsonPrettyTemplateFT { f };
            let mut buf: [u8; 1024] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
            let b = t.call(&mut buf).unwrap();
            assert_eq!(serde_json::to_vec_pretty(&f).unwrap(), buf[..b].to_vec());
        }
    }
}
