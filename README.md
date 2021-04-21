# Readme

[![ci-result][a1]][a2]

[a1]: https://gitlab.parity.io/parity/ink/badges/master/pipeline.svg
[a2]: https://gitlab.parity.io/parity/ink/pipelines?ref=master

The idea of this project is to have proper end-to-end tests for this pipeline:
```
ink! ➜
   cargo-contract ➜
                canvas-ui ➜
                         canvas-node
```

This way we want to always ensure that our components work properly together.
The CI for this project currently tests this pipeline for ink!'s `flipper` example.


## How it Works

* The tests in this repository use the `HEAD` of the `master` branch of all these components.
* They build the ink! examples using `cargo-contract`.
* The resulting `.contract` file is deployed on a local `canvas-node` instance using
  the `canvas-ui`.
* This is done by emulating browser interactions with the `canvas-ui` (such as clicking,
  uploading files, …).
* After successful deployment some more browser interactions with the contract are emulated,
  in order to assert that the contract behaves as expected.
  

## Required dependencies

* [`cargo-contract`](https://github.com/paritytech/cargo-contract#installation) with its dependencies
  `binaryen` and `rust-src`.
* [`geckodriver`](https://github.com/mozilla/geckodriver/) - is required for emulating interactions with
  a browser. Packages are available in some package managers, binary releases are available
  [in the repository](https://github.com/mozilla/geckodriver/releases).
* [`canvas-node`](https://paritytech.github.io/ink-docs/getting-started/setup#installing-the-canvas-node)
* [The ink! repository](https://github.com/paritytech/ink)
* Firefox

The [`canvas-ui`](https://github.com/paritytech/canvas-ui) is an optional requirement, by default
the [published version](https://paritytech.github.io/canvas-ui) is used.


## Run it locally

```bash
export INK_EXAMPLES_PATH=/path/to/ink/examples/
canvas --tmp --dev > /tmp/canvas.log 2>&1 &

# by default you will see the Firefox GUI and the
# tests interacting with it
cargo test 

# …you can also start the tests headless though, then
# you won't see anything
cargo test --features headless

# handy for debugging:
# you can prevent the test suite from closing the browser
# window. then you can still interact with the browser after
# the test failed/succeeded. 
export WATERFALL_CLOSE_BROWSER=false
cargo test
```

By default, the `canvas-ui` published at [https://paritytech.github.io/canvas-ui](https://paritytech.github.io/canvas-ui)
(i.e. the `gh-pages` branch) will be used. But you can also use a local instance:

```bash
git clone --depth 1 https://github.com/paritytech/canvas-ui.git
cd canvas-ui/
yarn install
yarn start > /tmp/canvas-ui.log 2>&1 &
cd ..

# check that the ui is ready and a `200 OK` is returned
curl -I http://localhost:3000/

export CANVAS_UI_URL="http://localhost:3000"
cargo test
```

## Environment variables

* `INK_EXAMPLES_PATH` ‒ Path to the ink! examples folder.
* `CANVAS_UI_URL` ‒ URL of the `canvas-ui`.
* `WATERFALL_TIMEOUT_SECS_PER_TEST` ‒ The number of seconds each test is allowed to take.
  This is necessary so that the CI fails early and doesn't wait for e.g. the Gitlab timeout,
  just because some UI element has changed its name.
* `WATERFALL_CLOSE_BROWSER` ‒ Do not close browser window at the end of a test run.
