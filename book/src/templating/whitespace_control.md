# Whitespace control

Yarte provides the possibility to erase the unneeded blanks when templating using the character `~` 
in any block. Only first white characters and last whitespaces of a block will be ignored. Characters interpreted as whitespaces will as specified in [rust](https://doc.rust-lang.org/reference/whitespace.html).

Let's say 
we have a struct define as follows:

```rust
#[derive(Template)]
#[template(path = "hello.html")]
struct CardTemplate<'a> {
    users: Vec<User<'a>>,
}

struct User<'a> {
    valid: bool,
    name: &'a str,
}
```

and we create a new `CardTemplate`
```rust
let t = CardTemplate {
    users: vec![
        User { 
            name: "Tom",
            valid: true,
        },
    ],
};
```
Now we will use `~` in an `if` statement and ignore unnecessary whitespaces. 
```handlebars
{{~# each users ~}}
    {{~# if valid ~}}
        <h1>Hello, {{ name }}</h1>
    {{~/if }}
{{~/each}}
```

This will output 
```text
<h1>Hello, Tom</h1> 
```

In the other hand if we don't ignore whitespaces:
```handlebars
{{~# each users ~}}
    {{# if valid }}
        <h1>Hello, {{ name }}</h1>
    {{/if }}
{{~/each}}
```

then this will be the output 
```text

    <h1>Hello, Tom</h1> 
    
```

## Special cases
The are some especial cases where Yarte will ignore whitespaces before and after in some special cases by 
default. These cases are when writing `comments`, `locals`(such as `let` expressions), and whitespaces at 
the end of the file
