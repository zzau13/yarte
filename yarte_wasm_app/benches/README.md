# Benches

Implementation of [`js-framework-benchmark`](https://github.com/krausest/js-framework-benchmark) with yarte

```bash
wasm-pack build --target web --release

python3 -m http.server
```

### Bundle
```bash
180K example_bg.wasm
 60K example_bg.wasm.gz
```

# Partial result of proof of concept
> On intel i7-7700HQ

### Non Keyed
##### Duration in milliseconds ± 95% confidence interval (Slowdown = Duration / Fastest)
![](shot-1.png)

##### Startup metrics (lighthouse with mobile simulation)
![](shot-2.png)

##### Memory allocation in MBs ± 95% confidence interval
![](shot-3.png)

