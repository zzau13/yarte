# Raw

If escaping Yarte code is ever needed, the `{{R }} {{/R }}` block can be used and Yarte code
inside the block will be escaped like in the following example: 

```handlebars
  {{~R }}{{#each example}}{{/each}}   {{~/R }}
```
will be render to:
```text
{{#each example}}{{/each}}
```
