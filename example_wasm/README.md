# Work in progress

## Build 
```bash
wasm-pack build --release --target web 
```


## Generate App Documentation
You can generate the documentation on the BlackBox 
to be able to modify it outside the automatic render cycle
by message
```bash
cargo doc --target wasm32-unknown-unknown --open --no-deps
```