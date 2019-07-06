# Super scope
In Yarte you will be able to call parents of the actual scope, making parent scopes
available in all child 
```handlebars
Hello, {{#each this~}}
        {{#each this.as_bytes() ~}}
            {{ super::index0 }} {{ super::super::this[0] }}
        {{~/each }}{{ super::this[0] }}
{{~/each}}!
```

```handlebars
Hello, {{#each this~}}
    {{#with this}}{{ super::hold }}{{ hold }}{{/with}}
{{~/each}}!
```

```handlebars
Hello, {{#each this~}}
    {{#with this}}{{ super::hold }}{{ hold }}{{ super::index }}{{/with}}
{{~/each}}!
```
