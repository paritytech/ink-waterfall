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

//! Tests for the `incrementer `example.

use crate::{
    uis::{
        Call,
        Result,
        Ui,
        Upload,
    },
    utils::{
        self,
        cargo_contract,
    },
};
use lang_macro::waterfall_test;

#[waterfall_test]
async fn incrementer_works(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("incrementer/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
        .execute_upload(Upload::new(contract_file).push_initial_value("initValue", "13"))
        .await?;
    assert_eq!(
        ui.execute_rpc(Call::new(&contract_addr, "get")).await?,
        "13"
    );

    // when
    ui.execute_transaction(Call::new(&contract_addr, "inc").push_value("by", "14"))
        .await
        .expect("failed to execute transaction");

    // then
    assert_eq!(
        ui.execute_rpc(Call::new(&contract_addr, "get")).await?,
        "27"
    );
    Ok(())
}

#[waterfall_test]
async fn default_constructor(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("incrementer/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    // when
    let contract_addr = ui
        .execute_upload(Upload::new(contract_file).constructor("default"))
        .await?;

    // then
    assert_eq!(ui.execute_rpc(Call::new(&contract_addr, "get")).await?, "0");
    Ok(())
}
