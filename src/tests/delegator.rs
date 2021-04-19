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
    self,
};
use lang_macro::waterfall_test;
use crate::utils::canvas_ui::UploadInput;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[waterfall_test]
async fn delegator_works(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let accumulator =
        cargo_contract::build(&utils::example_path("delegator/accumulator/Cargo.toml")).expect("accumulator build failed");
    let adder =
        cargo_contract::build(&utils::example_path("delegator/adder/Cargo.toml")).expect("adder build failed");
    let subber =
        cargo_contract::build(&utils::example_path("delegator/subber/Cargo.toml")).expect("subber build failed");
    let delegator =
        cargo_contract::build(&utils::example_path("delegator/Cargo.toml")).expect("delegator build failed");

    let accumulator_addr = canvas_ui.upload(UploadInput::new(accumulator)).await?;
    let adder_addr = canvas_ui.upload(UploadInput::new(adder)).await?;
    let subber_addr = canvas_ui.upload(UploadInput::new(subber)).await?;

    let delegator =
        cargo_contract::build(&utils::example_path("delegator/Cargo.toml")).expect("delegator build failed");

    let accumulator_hash = String::from("0x694f690cebfca6ada3e548747b9e9438f4a277c77e8dc66bbdbfc441d921b3c7");
    let adder_hash = String::from("0x74de7cebd87ffc53621987d0ec18f610867dfd22b56ee8f7162e3a7bbef99bd4");
    let subber_hash = String::from("0x27b8eebfe9e80ae0d90f7f7b60f4acfe530b7f1025e5b462e49719757a88f0d4");

    let delegator_addr = canvas_ui
        .upload(UploadInput::new(delegator)
            .endowment("1000000", "Unit")
            .push_initial_value("accumulatorCodeHash", &accumulator_hash)
            .push_initial_value("adderCodeHash", &adder_hash)
            .push_initial_value("subberCodeHash", &subber_hash)
        )
        .await?;


    /*
    assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get").await?, "false");

    // when
    canvas_ui
        .execute_transaction(&contract_addr, "flip")
        .await?;

    // then
    assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get").await?, "true");

     */
    Ok(())
}
