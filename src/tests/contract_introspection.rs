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

//! Tests for the `contract_introspection` example.

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

#[waterfall_test(example = "contract_introspection")]
#[ignore]
async fn contract_instrospection_works(mut ui: Ui) -> Result<()> {
    // given
    let contract_path =
        cargo_contract::build(&utils::example_path("contract-introspection/Cargo.toml"))
            .expect("contract build failed");

    // when
    let addr = ui.execute_upload(Upload::new(contract_path)).await?;

    // then
    // the method is called directly via the ui.
    assert_eq!(
        ui.execute_rpc(Call::new(&addr, "is_caller_contract"))
            .await?,
        "false"
    );
    // the `is_caller_contract` method is called indirectly from this contract method.
    assert_eq!(
        ui.execute_rpc(Call::new(&addr, "calls_is_caller_contract"))
            .await?,
        "true"
    );

    // the method is called directly via the ui.
    assert_eq!(
        ui.execute_rpc(Call::new(&addr, "is_caller_origin")).await?,
        "true"
    );
    // the `is_caller_origin` method is called indirectly from this contract method.
    assert_eq!(
        ui.execute_rpc(Call::new(&addr, "calls_is_caller_origin"))
            .await?,
        "false"
    );

    Ok(())
}
