# Config File

An example of a config file `yarte.toml`:
```toml
# root dir of templates
[main]
dir = "templates"

# Alias for partials. In call, change the start of partial path with one of this, if exist.
[partials]
alias = "./deep/more/deep"

[debug]
# prettyprint themes, put anything for options
theme = "zenburn"
number_line = true
grid = true
```
The `main` attribute makes reference main configuration variables. `dir` is used to define the path 
to the directory where all the templates are located.
The `partial` attribute makes references to the partials. In the example above the namespace `alias`
is used to make reference to path `templates/deep/more/deep` since the relative path is the one defined in `dir`.

`debug` is used so developers can anticipate the code that will be generated in a clear manner.
A `theme` can be defined so that the code in command line looks more appealing using pre-built themes.
Other debugging features can be set as `number_line` to draw code number line, or `grid` to see the code in a grid.
