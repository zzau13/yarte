# Yarte Wasm application
A simple single thread reactor pattern implementation

### Implement without yarte derive
The cycle of App methods is:
- `init`:
    - `__hydrate(&mut self, _addr: &'static Addr<Self>)`
- `on message`:
    - enqueue message
    - is ready? -> `update`
- `update`
    - pop message? -> `__dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>)`
    - is queue empty?  -> `__render(&mut self, _addr: &'static Addr<Self>)`
    - is queue not empty? -> `update`