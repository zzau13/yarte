# Meta programming

Yarte incorporates a meta programming system parallel to Rust's. 
Which means that it evaluates all Rust expressions at compilation 
time and together with the partials and the partials block create 
complex compilations in which you can use recursion, modules, 
conditional, loops, arrays and ranges.

The methods that can be used are listed in the documentation for [`v_eval`](https://docs.rs/v_eval)
