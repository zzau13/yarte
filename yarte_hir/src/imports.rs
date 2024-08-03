use std::{fs::read_to_string, mem, path::Path, rc::Rc};

use yarte_helpers::config::Config;
use yarte_parser::{
    parse, ErrorMessage, Helper, MResult, Node, OwnParsed, PError, Parsed, Partial, PartialBlock,
    SNode,
};

// TODO: Error
fn get_nodes(src: &str, path: Rc<Path>) -> MResult<Vec<SNode<'static>>, PError> {
    let src = unsafe { mem::transmute::<&str, &'static str>(src) };
    let nodes = parse(path, src.trim_end())?;

    Ok(nodes)
}

// TODO: Error
fn get_nodes_from_path(path: Rc<Path>) -> MResult<(String, Vec<SNode<'static>>), PError> {
    // TODO: error message
    let src = read_to_string(Rc::clone(&path)).expect("exist file");

    let nodes = get_nodes(&src, path)?;
    Ok((src, nodes))
}

fn add_nodes(
    on_path: Rc<Path>,
    path: &str,
    ctx: Parsed,
    c: &Config,
    stack: &mut Vec<(Rc<Path>, String, Vec<SNode<'static>>)>,
    errors: &mut Vec<ErrorMessage<PError>>,
) {
    let path = c.resolve_partial(Rc::clone(&on_path), path);
    if ctx.get(&path).is_none() {
        match get_nodes_from_path(Rc::clone(&path)) {
            Ok((src, nodes)) => {
                stack.push((path, src, nodes));
            }
            Err(e) => errors.push(e),
        }
    }
}

pub fn resolve_imports(
    src: String,
    path: Rc<Path>,
    c: &Config,
    ctx: &mut OwnParsed,
) -> Result<(), Vec<ErrorMessage<PError>>> {
    let nodes = match get_nodes(&src, Rc::clone(&path)).map_err(|e| vec![e]) {
        Ok(nodes) => nodes,
        Err(e) => {
            ctx.insert(path, (src, vec![]));
            return Err(e);
        }
    };
    let mut stack = vec![(path, src, nodes)];
    let mut errors = vec![];

    while let Some((on_path, src, nodes)) = stack.pop() {
        let mut stack_nodes = vec![];
        stack_nodes.push(&nodes);
        while let Some(nodes) = stack_nodes.pop() {
            for node in nodes {
                match node.t() {
                    Node::Helper(helper) => match helper.as_ref() {
                        Helper::Each(_, _, nodes) => {
                            stack_nodes.push(nodes);
                        }
                        Helper::If((_, _, if_nodes), else_ifs, else_) => {
                            stack_nodes.push(if_nodes);
                            for (_, _, nodes) in else_ifs {
                                stack_nodes.push(nodes)
                            }
                            if let Some((_, nodes)) = else_ {
                                stack_nodes.push(nodes)
                            }
                        }
                        Helper::With(_, _, nodes) => stack_nodes.push(nodes),
                        Helper::Unless(_, _, nodes) => stack_nodes.push(nodes),
                        Helper::Defined(_, _, _, nodes) => stack_nodes.push(nodes),
                    },
                    Node::Partial(Partial(_, path, _)) => {
                        add_nodes(
                            Rc::clone(&on_path),
                            path.t(),
                            ctx,
                            c,
                            &mut stack,
                            &mut errors,
                        );
                    }
                    Node::PartialBlock(PartialBlock(_, path, _, ref nodes)) => {
                        stack_nodes.push(nodes);
                        add_nodes(
                            Rc::clone(&on_path),
                            path.t(),
                            ctx,
                            c,
                            &mut stack,
                            &mut errors,
                        );
                    }
                    _ => {}
                }
            }
        }
        ctx.insert(on_path, (src, nodes));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
