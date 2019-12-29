use annotate_snippets::{
    display_list::DisplayList,
    formatter::DisplayListFormatter,
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};

use yarte_config::Config;
use yarte_parser::source_map::Span;

use crate::helpers::Sources;

// TODO: Display errors ?
pub struct ErrorMessage {
    pub message: String,
    pub span: Span,
}

// TODO: improve
// - print all line with len limits
pub fn emitter(sources: Sources, config: &Config, mut errors: Vec<ErrorMessage>) -> ! {
    let mut prefix = config.get_dir().clone();
    prefix.pop();

    errors.sort_unstable_by(|a, b| a.span.lo.cmp(&b.span.lo));
    let slices = errors
        .into_iter()
        .map(|err| {
            let origin = err.span.file_path();
            let (lo, hi) = err.span.range_in_line();
            let start = err.span.start();
            let source = sources
                .get(&origin)
                .unwrap()
                .get(lo..hi)
                .unwrap()
                .to_string();
            let origin = origin
                .strip_prefix(&prefix)
                .unwrap()
                .to_string_lossy()
                .to_string();

            Slice {
                source,
                line_start: start.line,
                origin: Some(origin),
                annotations: vec![SourceAnnotation {
                    range: (start.column, hi),
                    label: err.message,
                    annotation_type: AnnotationType::Error,
                }],
                fold: false,
            }
        })
        .collect();

    let s = Snippet {
        title: Some(Annotation {
            id: None,
            label: None,
            annotation_type: AnnotationType::Error,
        }),
        footer: vec![],
        slices,
    };

    let dl = DisplayList::from(s);
    let dlf = DisplayListFormatter::new(true, false);

    // TODO: decide when output is better
    panic!("{}", dlf.format(&dl))
    //    eprintln!("{}", dlf.format(&dl))
    //    struct Panickier;
    //    resume_unwind(Box::new(Panickier))
}
