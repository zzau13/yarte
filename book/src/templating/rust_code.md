# Rust code

Yarte provides you with the possibility to use basic raw rust code within 
the HTML files. There are three important facts to take in consideration 
when using rust code in your template, the scope they act upon, resolve ident of variables.


- The usage if this feature is limited by its context, meaning that created variables of a scope, will live only in that scope. Also, a valid rust expression, so it will act like Rust.

- Resolve:
  - Resolution uses hierarchical nomenclature system, and its most important function is to 'translate' names depending on the context
    where it lives. Contexts are defined by the root, helpers, partials, and rust blocks. Created variables in these blocks
    will be removed from the scope when block finishes.
    If a variable that already existed is redefined, the first will be overwritten by the second one, losing the 
    original value. Be careful with pre-defined variables like `first`, `this`, `index`, `index0`, `_index_[0-9]+` or `_n` at tuple context to 
    make reference to the n-th item.
  
  - Constants and static variables must be upper-cased with underscores, `N_ITER`.
  - Paths of type `\*\*::\*\*::\*\*` can be use without using reserved word `super`.
  - `self` refers to the root scope (first parent). Note that in partials this would be the current partial in use.
  - Substitution will take into account `super`, locals, etc. So keep track of the context created.
  
```handlebars
Hello, {{#each conditions}}
    {{~#if let Some(check) = cond }}
        {{~#if check }}
            {{ let cond = if check { "&foo" } else { "&"} }}
            {{
                if check {
                    cond
                } else if let Some(cond) = key.cond {
                    if cond {
                        "1"
                    } else {
                        "2"
                    }
                } else {
                   "for"
                }
            }}
        {{~ else if let Some(_) = cond }}
        {{~ else if let Some(cond) = key.check }}
            {{~#if cond ~}}
                baa
            {{~/if }}
        {{~ else ~}}
            {{ cond.is_some() }}
        {{~/if~}}
        {{ cond.is_some() && true }}
    {{~else if let Some(cond) = check }}
        {{~#if cond ~}}
            bar
        {{~/if}}
    {{~ else ~}}
        None
    {{~/if}}
{{~/each}}!
```

```handlebars
{{ unsafe { s.get_unchecked(0) } }}
```
