# Conditional helper

## If helper

The conditional helper must start with an `if` block, followed by the  condition, 
`{{#if condition}}`, where `condition` is a valid rust expression (otherwise an 
error will be thrown). Inside an `if` block, users can use as many `else if` 
statements as they want  and one `else` statement to create basic logic in the 
template, without using `#`, for example, `{{else}}` or `{{else if condotion}}`.
In order to close the `if` block, following the helper syntax, `{{/if}}` is used.
 
```handlebars
{{#if isLiked}}
  Liked!
{{else if isSeen}}
  Seen!
{{else}}
  Sorry ...
{{/if}}
```
In the example above if variable `isLiked` is interpreted as `true`, `Liked!` 
will be parsed. If `isLiked` is interpreted as `false` and `isSeen` as `true`
then `Seen!`, otherwise `Sorry...` will be shown. So having conditional around 
your HTML code is as intuitive as it should be.

## Unless helper

The `unless` helper is equivalent to a negated `if` statement, for that reason, negated `unless` statements
are not allowed and and error will be prompt.

```handlebars
{{#unless isAdministrator-}} 
  Ask administrator.
{{~/unless}}
```
