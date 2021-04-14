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

pub mod canvas_ui;
pub mod cargo_contract;

use std::path::PathBuf;

/// Returns the full path to the ink! example directory for `example`.
pub fn example_path(example: &str) -> PathBuf {
    let examples_path = std::env::var("INK_EXAMPLES_PATH")
        .expect("env variable INK_EXAMPLES_PATH must be set");
    let path = PathBuf::from(examples_path);
    path.join(example)
}
