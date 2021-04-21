// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use regex::Regex;
use std::{
    path::PathBuf,
    process::Command,
};

/// Run cargo with the supplied args
///
/// If successful, returns the stdout bytes
pub(crate) fn build(manifest_path: &PathBuf) -> Result<PathBuf, String> {
    assert_wasm_opt_available();

    let mut dir = manifest_path.clone();
    dir.pop(); // pop `Cargo.toml` from the path

    let output = Command::new("cargo")
        .arg("+nightly")
        .arg("contract")
        .arg("build")
        .arg("--manifest-path=Cargo.toml")
        .current_dir(dir)
        .output()
        .map_err(|err| format!("oh no - {:?}", err))
        .expect("failed to execute process");

    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).expect("string conversion failed");
        // extract the path to the resulting `.contract` from the output
        let re_path = Regex::new(
            r"Your contract artifacts are ready. You can find them in:\n([A-Za-z0-9_\-/]+)\n"
        )
            .expect("invalid regex");
        let captures = re_path
            .captures(&stdout)
            .ok_or("regex does not match the command output")
            .map_err(|err| format!("{}: '{:?}'", err, stdout))?;
        let directory = captures.get(1).expect("no capture group found").as_str();

        // extract the basename to the resulting `.contract`
        let re_basename =
            Regex::new(r"\- ([A-Za-z0-9_\-/]+).contract \(code \+ metadata\)\n")
                .expect("invalid regex");
        let captures = re_basename
            .captures(&stdout)
            .ok_or("regex does not match the command output")
            .map_err(|err| format!("{}: '{:?}'", err, stdout))?;
        let basename = captures.get(1).expect("no capture group found").as_str();
        let path = PathBuf::from(directory).join(format!("{}.contract", basename));
        log::info!("Path to the resulting contract bundle: {:?}", path);
        Ok(path)
    } else {
        let stderr = String::from_utf8(output.stderr).expect("string conversion failed");
        Err(format!(
            "Failed with exit code: {:?} and '{:?}'",
            output.status.code(),
            stderr
        ))
    }
}

/// Asserts that `wasm-opt` is available.
fn assert_wasm_opt_available() {
    assert!(
        which::which("wasm-opt").is_ok(),
        "ERROR: The `wasm-opt` binary cannot be found!\n\n\
        Please check that it is installed and in your `PATH`.\n\n\
        See the `cargo-contract` readme for instructions on how to install it:\n\
        https://github.com/paritytech/cargo-contract."
    );
}
