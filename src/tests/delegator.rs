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
        extract_hash_from_contract_bundle,
    },
};
use lang_macro::waterfall_test;

#[waterfall_test]
async fn delegator_works(mut ui: Ui) -> Result<()> {
    // given
    let accumulator_path =
        cargo_contract::build(&utils::example_path("delegator/accumulator/Cargo.toml"))
            .expect("accumulator build failed");
    let accumulator_hash = extract_hash_from_contract_bundle(&accumulator_path);

    let adder_path =
        cargo_contract::build(&utils::example_path("delegator/adder/Cargo.toml"))
            .expect("adder build failed");
    let adder_hash = extract_hash_from_contract_bundle(&adder_path);

    let subber_path =
        cargo_contract::build(&utils::example_path("delegator/subber/Cargo.toml"))
            .expect("subber build failed");
    let subber_hash = extract_hash_from_contract_bundle(&subber_path);

    let delegator_path =
        cargo_contract::build(&utils::example_path("delegator/Cargo.toml"))
            .expect("delegator build failed");

    let _accumulator_addr = ui.execute_upload(Upload::new(accumulator_path)).await?;
    let _adder_addr = ui.execute_upload(Upload::new(adder_path)).await?;
    let _subber_addr = ui.execute_upload(Upload::new(subber_path)).await?;

    // when
    let delegator_addr = ui
        .execute_upload(
            Upload::new(delegator_path)
                .endowment("100000", "Unit")
                .push_initial_value("accumulatorCodeHash", &accumulator_hash)
                .push_initial_value("adderCodeHash", &adder_hash)
                .push_initial_value("subberCodeHash", &subber_hash),
        )
        .await?;

    // then
    assert_eq!(
        ui.execute_rpc(Call::new(&delegator_addr, "get")).await?,
        "0"
    );
    ui.execute_transaction(
        Call::new(&delegator_addr, "change")
            .push_value("by: i32", "13")
            .max_gas("5000"),
    )
    .await
    .expect("failed to execute transaction");
    assert_eq!(
        ui.execute_rpc(Call::new(&delegator_addr, "get").max_gas("5000"))
            .await?,
        "13"
    );
    ui.execute_transaction(Call::new(&delegator_addr, "switch").max_gas("5000"))
        .await
        .expect("failed to execute transaction");
    ui.execute_transaction(
        Call::new(&delegator_addr, "change")
            .push_value("by: i32", "3")
            .max_gas("5000"),
    )
    .await
    .expect("failed to execute transaction");
    assert_eq!(
        ui.execute_rpc(Call::new(&delegator_addr, "get").max_gas("5000"))
            .await?,
        "10"
    );
    Ok(())
}
