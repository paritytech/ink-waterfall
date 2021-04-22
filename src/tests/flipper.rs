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

//! Tests for the `flipper `example.

use crate::utils::{
    self,
    canvas_ui::{
        CanvasUi,
        Transaction,
        Upload,
    },
    cargo_contract,
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[waterfall_test]
async fn flipper_works(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let manifest_path = utils::example_path("flipper/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = canvas_ui.execute_upload(Upload::new(contract_file)).await?;
    // assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get", None).await?, "false");
    assert_eq!(
        canvas_ui.execute_rpc(&contract_addr, "get", None).await?,
        "false"
    );

    // when
    canvas_ui
        .execute_transaction(Transaction::new(&contract_addr, "flip"))
        .await?;

    // then
    assert_eq!(
        canvas_ui.execute_rpc(&contract_addr, "get", None).await?,
        "true"
    );

    Ok(())
}
