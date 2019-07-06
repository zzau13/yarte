# HTML

Yarte HTML-escapes values returned by a `{{ expression }}`. 
If you don't want Yarte to escape a value, use the 
"triple-stash", `{{{`. For example having the following 
struct:

```rust
let t = CardTemplate {
  title: "All about <p> Tags",
  body: "<p>This is a post about &lt;p&gt; tags</p>"
};
```
and the following template:

```handlebars
<div class="entry">
  <h1>{{title}}</h1>
  <div class="body">
    {{{body}}}
  </div>
</div>
```

will result in:
    
```handlebars
<div class="entry">
  <h1>All About &lt;p&gt; Tags</h1>
  <div class="body">
    <p>This is a post about &lt;p&gt; tags</p>
  </div>
</div>
```
