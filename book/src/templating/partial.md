# Partial

Partials is the tool that Yarte provides for  template composition and is a one line expression of type 
`{{> partial_path scope attr=val,...}}`. The performance **is the same** as writing it and the same as an inline code. 
The partials and their arguments are generated as much as possible within the same `&'static str`. 


## Path
The path of a partial is the file path with respect to the file using the partial. Also the `config` file can be used to 
create aliases (explained in the aliasing section). The partial template file will use the context used in attribute 
scope.


## Attributes
Attributes in partials are assignation where right-hand side if the equal sign must be an expression of type path, 
field, or index. These attributes will be used to reference expression's values and use them inside the partial. 
In a partial, Yarte will first try to look the value in the attributes and if there is no existing attribute, the given 
scope must have it.

*__Note__: In this section we are making reference to attributes which are assignations (not attribute `scope` or 
`path`).


## Scope
Attribute `scope` is the only attribute that is not an assignation and the only one that a partial must have. 
- If `scope` is not given the default context will be the parent's, otherwise `scope` can only be an expression of type 
`path`, `field`, or `index`.

- When attribute `scope` is given, the parent scope cannot be access using `super::`.

The following partial will use file `..tempaltes/templates/partial.hbs`, and the parent scope to fill he template:
```handlebars
{{> ../templates/partial }}
```

Now the same file will be used but since a scope is defined, the parent scope will be overriding `expr_scope`:
```handlebars
{{> ../templates expr_scope }}
```

Overriding super top scope (`self` always reference parent scope)
Literals are put inline and pre-escaped when specified (`{{ }}`).
```handlebars
{{> partial var = bar, lit = "foo" }}
```

Overriding parent scope and override super (`self` always reference `expr_scope`)
Literals are put inline and pre-escaped when specified (`{{ }}`).
```handlebars
{{> partial expr_scope, var = bar, lit = "foo" }}
```

## Partial Block
This block syntax may also be used to pass templates to the partial, which 
can be executed by the specially named partial, `@partial-block`. A template of

```handlebars
{{#> layout ~}}
  My Content
{{~/layout }}
```

with the layout partial containing

```handlebars
Site Content {{> @partial-block }}
```

Would render
```
Site Content My Content
```

When called in this manner, the block will execute under the context of the partial at the time of the call. 
Depth-ed paths and block parameters operate relative to the partial block rather than the partial template.

```handlebars
{{#each children }}
  {{#> childEntry }}
    {{value}}
  {{/childEntry }}
{{/each }}
```

Will render `this.value` from this template, not the partial.

## Recursion
> TODO

## Aliasing
Aliasing is used to make life easier to developers when referencing to a partial template. This is done in the 
configuration file `yarte.toml`. 

This is explained in more detail in section [Config File](../config.md).

*Note: Aliases make reference to `dir` in the configuration file.
