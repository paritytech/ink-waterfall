# Readme

[![ci-result][a1]][a2]

[a1]: https://gitlab.parity.io/parity/ink-waterfall/badges/master/pipeline.svg
[a2]: https://gitlab.parity.io/parity/ink-waterfall/pipelines

This project contains end-to-end tests for this pipeline:

```
ink! ➜
   cargo-contract ➜
             canvas-ui || polkadot-js ➜
                                 canvas-node
```


## How the tests in this repository work

* They build [the ink! examples](https://github.com/paritytech/ink/tree/master/examples)
  using [`cargo-contract`](https://github.com/paritytech/cargo-contract).
* The resulting `.contract` file is deployed on a local blockchain instance of
  [`canvas-node`](https://github.com/paritytech/canvas-node).
* The deployment is done using either the [`canvas-ui`](https://github.com/paritytech/canvas-ui)
  or [`polkadot-js`](https://github.com/polkadot-js/apps).
* This is done by emulating browser interactions in Firefox (clicking, uploading, …).
* After successful deployment more browser interactions with the contract are
  conducted, in order to assert that the contract behaves as expected.
* The `master` branch of all these components is used.


## Required dependencies

* [`cargo-contract`](https://github.com/paritytech/cargo-contract#installation) with its dependencies
  `binaryen` and `rust-src`.
* [`geckodriver`](https://github.com/mozilla/geckodriver/) - is required for emulating interactions with
  a browser. Packages are available in some package managers, binary releases are available
  [in the repository](https://github.com/mozilla/geckodriver/releases).
* [`canvas-node`](https://paritytech.github.io/ink-docs/getting-started/setup#installing-the-canvas-node)
* [The ink! repository](https://github.com/paritytech/ink)
* Firefox

For the UI either the [`canvas-ui`](https://github.com/paritytech/canvas-ui) 
or the [`polkadot-js`](https://github.com/polkadot-js/apps) UI is an optional
requirement. By default the published versions of those projects are used
([https://paritytech.github.io/canvas-ui](https://polkadot.js.org/apps/#/), 
[https://polkadot.js.org/apps/#/](https://polkadot.js.org/apps/#/)).


## Run it locally

```bash
export INK_EXAMPLES_PATH=/path/to/ink/examples/
canvas --tmp --dev > /tmp/canvas.log 2>&1 &

# By default you will see the Firefox GUI and the
# tests interacting with it.
cargo test 

# …you can also start the tests headless though, then
# you won't see anything.
cargo test --features headless

# Handy for debugging:

# You can prevent the test suite from closing the browser
# window. Then you can still interact with the browser after
# the test failed/succeeded. 
export WATERFALL_CLOSE_BROWSER=false

# Setting the number of parallel jobs to `1` makes it easier
# to follow the tests interacting with the browser.
cargo test --jobs 1
```

By default, the `canvas-ui` published at [https://paritytech.github.io/canvas-ui](https://paritytech.github.io/canvas-ui)
(i.e. the `gh-pages` branch) will be used. But you can also use a local instance:

```bash
git clone --depth 1 https://github.com/paritytech/canvas-ui.git
cd canvas-ui/
yarn install
yarn start > /tmp/canvas-ui.log 2>&1 &
cd ..

# Check that the ui is ready and a `200 OK` is returned.
curl -I http://localhost:3000/

export UI_URL="http://localhost:3000"
cargo test
```

If you want to use the `polkadot-js` UI instead you need to
supply `--features polkadot-js-ui` to `cargo test`.


## Environment variables

* `INK_EXAMPLES_PATH` ‒ Path to the ink! examples folder.
* `UI_URL` ‒ URL of the UI to use.
* `WATERFALL_CLOSE_BROWSER` ‒ Do not close browser window at the end of a test run.
* `WATERFALL_SKIP_CONTRACT_BUILD` ‒ Do not build the contracts, re-use existing artifacts
  from their `target` folder.
* `RUST_LOG` ‒ Use `RUST_LOG=info` to get output on what the tests are doing.
* `CANVAS_PORT` ‒ Port under which the `canvas-node` is running, defaults to `9944`.


## Known issues

The tooltips which show the result of a contract upload or transaction
(`ExtrinsicSuccess`, …) disappear after some time. If too many UI tests
are run at the same time the tooltips might disappear before the test
is finished processing them.

The test will then fail with a `NoSucheElement` error, indicating that
the DOM element is no longer available.

The easiest fix for this is to limit the number of concurrent test
threads via `cargo test --jobs 4`.