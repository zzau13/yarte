# Comments

In order to add HTML comments to your Yarte code use `!--` or `!` after the opening templating 
tag and use `--}}` or `}}`, respectively, as a closing clause.

```handlebars
{{!   Comments can be written  }}
{{!--  in two different ways --}}
```

Comments will appear on the debug output. In release, comments are removed and stream is optimized.
When parsed, whitespaces around the comment block will be ignored.