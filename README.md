# Readme

The idea of this project is to have proper end-to-end tests for this pipeline:
```
ink! ➜
     cargo-contract ➜
                    canvas-ui ➜
                              canvas-node
```

The `HEAD` of the `master` branch is used for every component.


## Required dependencies

* [`canvas-node`](https://paritytech.github.io/ink-docs/getting-started/setup#installing-the-canvas-node)
* [`cargo-contract`](https://paritytech.github.io/ink-docs/getting-started/setup#ink-cli)
* [The ink! repository](https://github.com/paritytech/ink)
* [`geckodriver`](https://github.com/mozilla/geckodriver/) - packages are available in some package managers.
* Firefox


## Run it locally

```
export INK_EXAMPLES_PATH=/path/to/ink/examples/
canvas --tmp --dev > /tmp/canvas.log 2>&1 &

# by default you will see the Firefox GUI and the interactions with it
cargo test 

# …you can also start the tests headless though, then you won't see anything
cargo test --features headless
```