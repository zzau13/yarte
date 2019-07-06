# Each helper

In order to iterate over a vector of objects, an `each` helper can be used, with the following syntax:

```handlebars
{{#each into_iter}} 
    {{~# if first ~}}
        {{ index }} 
    {{~ else ~}}
        {{ index0 }} 
    {{~/if }} {{ this }} 
{{~/each}}
```

Associated variables such as  `this`, `first`, `index`, `index0` and struct fields are automatically generated
and can be used without declaring them.
