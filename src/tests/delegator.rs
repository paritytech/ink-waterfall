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
    self,
    canvas_ui::{
        CanvasUi,
        UploadInput,
    },
    cargo_contract,
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// we ignore the test for the ci, since the delegator upload is
// currently broken: https://github.com/paritytech/canvas-ui/issues/95
#[ignore]
#[waterfall_test]
async fn delegator_works(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let accumulator_path =
        cargo_contract::build(&utils::example_path("delegator/accumulator/Cargo.toml"))
            .expect("accumulator build failed");
    let adder_path =
        cargo_contract::build(&utils::example_path("delegator/adder/Cargo.toml"))
            .expect("adder build failed");
    let subber_path =
        cargo_contract::build(&utils::example_path("delegator/subber/Cargo.toml"))
            .expect("subber build failed");
    let delegator_path =
        cargo_contract::build(&utils::example_path("delegator/Cargo.toml"))
            .expect("delegator build failed");

    let _accumulator_addr = canvas_ui.upload(UploadInput::new(accumulator_path)).await?;
    let _adder_addr = canvas_ui.upload(UploadInput::new(adder_path)).await?;
    let _subber_addr = canvas_ui.upload(UploadInput::new(subber_path)).await?;

    let accumulator_hash = String::from(
        "0x694f690cebfca6ada3e548747b9e9438f4a277c77e8dc66bbdbfc441d921b3c7",
    );
    let adder_hash = String::from(
        "0x74de7cebd87ffc53621987d0ec18f610867dfd22b56ee8f7162e3a7bbef99bd4",
    );
    let subber_hash = String::from(
        "0x27b8eebfe9e80ae0d90f7f7b60f4acfe530b7f1025e5b462e49719757a88f0d4",
    );

    // when
    let delegator_addr = canvas_ui
        .upload(
            UploadInput::new(delegator_path)
                .endowment("1000000", "Unit")
                .push_initial_value("accumulatorCodeHash", &accumulator_hash)
                .push_initial_value("adderCodeHash", &adder_hash)
                .push_initial_value("subberCodeHash", &subber_hash),
        )
        .await?;

    // then
    // interactions with the contract…
    assert_eq!(canvas_ui.execute_rpc(&delegator_addr, "get").await?, "0");

    Ok(())
}
