# Let 
Let statements it can only be used with `{{` not with `{{{`, and tries to imitate rust `let` statement.

>A `let` statement introduces a new set of variables, given by a pattern. The pattern is followed optionally by a type annotation and then optionally by an initializer expression. When no type annotation is given, the compiler will infer the type, or signal an error if insufficient type information is available for definite inference. Any variables introduced by a variable declaration are visible from the point of declaration until the end of the enclosing block scope.
> -- <cite>[Rust Documentation][1]</cite>

[1]:https://doc.rust-lang.org/reference/statements.html#let-statements

Unlike rust code block, locals will live in his parent context and can overwrite previously defined variables, it's 
very important to take this into account. Also is important to know when parsing locals, whitespaces before and after 
the block will be ignored.

```handlebars
{{ let doubled = a.iter().map(|x| x * 2).collect::<Vec<_>>() }}
{{ let doubled: Vec<usize> = a.iter().map(|x| x * 2).collect() }}

{{#each doubled ~}}
    {{ this + 1 }}
{{~/each}}
```


Inside the expression you can use tuple and slice decomposition, mutability and types:
```handlebars
{{ let a = |a: &str| a.repeat(2) }}

{{ let mut a = name.chars() }}

{{ let (mut h, t)  = name.split_at(1) }}
```
