# Yarte Wasm application
A simple single thread reactor pattern implementation

### Implement without yarte derive
The cycle of App methods is:
- `init`:
    - `App.__hydrate(&mut self, _addr: &'static Addr<Self>)`
    - `Update`
- `on message`:
    - enqueue message
    - is ready? -> `update`
- `update`
    - pop message? -> `App.__dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>)`
    - is queue empty?  -> `App.__render(&mut self, _addr: &'static Addr<Self>)`
    - is queue not empty? -> `update`. It is **not** recursive.