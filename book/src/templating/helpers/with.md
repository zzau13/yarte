# With helper

The `with` helper sets the scope/context to be any specified structure, using syntax
`{{#with context}}  {{/with}}`.

For example, in the following lines of code we want to set the context inside the `with` block
to be `author`, defined in the template instead of the 'main' context.

```rust
let author = Author {
    name: "J. R. R. Tolkien"
};
```

```handlebars
{{#with author}}
  <p>{{name}}</p>
{{/with}}
```
