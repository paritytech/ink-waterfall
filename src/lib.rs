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

#[cfg(test)]
mod uis;

#[cfg(test)]
mod utils;

#[cfg(test)]
mod tests;

use std::{
    cell::RefCell,
    sync::Once,
};

/// We use this to only initialize `env_logger` once.
pub static INIT: Once = Once::new();

// We save the name of the currently executing test here.
thread_local! {
    pub static TEST_NAME: RefCell<String> = RefCell::new(String::from("no test"));
}
