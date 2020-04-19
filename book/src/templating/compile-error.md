# Compile error

You can throw at compile time errors:

```handlebars
{{#if const_expr }}
    {{$ "Message" }}
{{/if }}
```