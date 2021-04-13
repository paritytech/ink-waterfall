# Readme

End-to-end tests for
```
ink! ➜
     cargo-contract ➜
                    canvas-ui ➜
                              canvas-node
```

The `HEAD` of the `master` branch is used for every component.


## Required dependencies

* `canvas-node`
* `cargo-contract`
* The ink! repository: `git clone --depth 1 https://github.com/paritytech/ink.git`.
* `geckodriver`
* Firefox


## Run it locally

```
export INK_EXAMPLES_PATH=/path/to/ink/examples/
canvas --tmp --dev > /tmp/canvas.log 2>&1 &
geckodriver --port 4444 &

cargo test 

# or if you want to have it headless
cargo test --features headless
```