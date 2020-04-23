@helpers are a group of functions implemented on `Formatter` to format most common template structures.
Are made to avoid reallocation by creating a single output stream on a single highly abstract writer.
Adapt to the language of handlebars by hinting at common expressions headed by the character '@' and 
the name of `@helper`

Per example:
```handlebars
{{ @json obj }}
```
