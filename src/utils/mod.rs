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

pub mod cargo_contract;

use serde_json;
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    process::Command,
};

/// Returns the name of the test which is currently executed.
pub fn test_name() -> String {
    crate::TEST_NAME.with(|test_name| test_name.borrow().clone())
}

/// Returns the full path to the ink! example directory for `example`.
///
/// This method will first try to look the example up in `INK_EXAMPLES_PATH`.
/// If not found there, it will fall back to `./examples/` ++ `example`.
pub fn example_path(example: &str) -> PathBuf {
    let examples_path = std::env::var("INK_EXAMPLES_PATH")
        .expect("env variable `INK_EXAMPLES_PATH` must be set");
    let mut path = PathBuf::from(examples_path).join(example);

    // Check if path exists, if not assume it's a local example to `ink-waterfall`.
    // This is done as a fallback if the waterfall tests are run locally.
    // For the CI we copy all `ink-waterfall/examples/` to the `INK_EXAMPLES_PATH`.
    if !path.exists() {
        path = PathBuf::from("./examples/").join(example);
        path = path.canonicalize().unwrap_or_else(|path| {
            panic!("canonicalizing {:?} must work", path);
        });
    }

    path
}

/// Extracts the `source.hash` field from the contract bundle.
pub fn extract_hash_from_contract_bundle(path: &PathBuf) -> String {
    let file =
        File::open(path).expect(&format!("Contract file at {:?} does not exist", path));
    let reader = BufReader::new(file);
    let json: serde_json::Value = serde_json::from_reader(reader).unwrap_or_else(|err| {
        panic!("JSON at {:?} is not well-formatted: {:?}", path, err)
    });
    json.get("source")
        .expect("Unable to get 'source' field from contract JSON")
        .get("hash")
        .expect("Unable to get 'hash' field from contract JSON")
        .to_string()
        .trim_matches('"')
        .to_string()
}

/// Asserts that some process is listening at the [`node_port`].
pub fn assert_node_running() {
    let url = format!("127.0.0.1:{}", node_port());
    std::net::TcpStream::connect(&url).unwrap_or_else(|_| {
        panic!(
            "No process listening on {}. Did you start the substrate-contracts-node?",
            url
        )
    });
}

/// Returns the port under which the node is running.
pub fn node_port() -> String {
    std::env::var("NODE_PORT").unwrap_or(String::from("9944"))
}

/// Returns true if the `substrate-contracts-node` log under
/// `/tmp/substrate-contracts-node.log` contains `msg`.
pub fn node_log_contains(msg: &str) -> bool {
    let output = Command::new("grep")
        .arg("-q")
        .arg(msg)
        .arg("/tmp/substrate-contracts-node.log")
        .spawn()
        .map_err(|err| format!("ERROR while executing `grep` with {:?}: {:?}", msg, err))
        .expect("failed to execute process")
        .wait_with_output()
        .expect("failed to receive output");
    output.status.success()
}
