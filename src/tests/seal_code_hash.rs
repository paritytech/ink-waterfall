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

//! Tests for the `seal_code_hash` example.

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

#[waterfall_test(example = "seal_code_hash")]
async fn delegator_works(mut ui: Ui) -> Result<()> {
    // given
    let contract_path =
        cargo_contract::build(&utils::example_path("seal-code-hash/Cargo.toml"))
            .expect("contract build failed");
    let bundle_hash = utils::extract_hash_from_contract_bundle(&contract_path);

    // when
    let addr = ui.execute_upload(Upload::new(contract_path)).await?;

    // then
    let deployed_hash = ui
        .execute_rpc(Call::new(&addr, "code_hash").push_value("account_id", &addr))
        .await?;
    let own_code_hash = ui.execute_rpc(Call::new(&addr, "own_code_hash")).await?;
    assert_eq!(own_code_hash, deployed_hash);
    assert_eq!(own_code_hash, bundle_hash);

    Ok(())
}
