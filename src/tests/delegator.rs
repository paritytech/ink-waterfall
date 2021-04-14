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

//! Tests for the `delegator `example.

use crate::utils::{
    canvas_ui::CanvasUi,
    cargo_contract,
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn build_all() {}

// #[waterfall_test]
// async fn works(mut canvas_ui: CanvasUi) -> Result<()> {
// given
// let manifest_path = crate::utils::example_path("flipper/Cargo.toml");
// let contract_file =
// cargo_contract::build(&manifest_path).expect("contract build failed");
//
// let contract_addr = canvas_ui.upload(contract_file).await?;
// assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get").await?, "false");
//
// when
// canvas_ui
// .execute_transaction(&contract_addr, "flip")
// .await?;
//
// then
// assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get").await?, "true");
//
// Ok(())
// }
//
//
