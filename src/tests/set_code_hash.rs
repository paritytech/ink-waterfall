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

//! Tests for the `set-code-hash` example.

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

#[waterfall_test(example = "set-code-hash")]
async fn set_code_hash_works(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path =
        utils::example_path("upgradeable-contracts/set-code-hash/Cargo.toml");
    let incrementer_bundle =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let incrementer_addr = ui
        .execute_upload(Upload::new(incrementer_bundle.clone()).caller("ALICE"))
        .await?;

    let manifest_path = utils::example_path(
        "upgradeable-contracts/set-code-hash/updated-incrementer/Cargo.toml",
    );
    let updated_incrementer_bundle =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let updated_incrementer_hash =
        utils::extract_hash_from_contract_bundle(&updated_incrementer_bundle);
    let _updated_incrementer_addr = ui
        .execute_upload(Upload::new(updated_incrementer_bundle.clone()))
        .await?;

    ui.execute_transaction(Call::new(&incrementer_addr, "inc"))
        .await
        .expect("failed to `submit_transaction`");
    assert_eq!(
        ui.execute_rpc(Call::new(&incrementer_addr, "get")).await?,
        "1"
    );

    let call = Call::new(&incrementer_addr, "set_code")
        .push_value("code_hash", &updated_incrementer_hash);
    ui.execute_transaction(call).await.expect("must work");

    ui.execute_transaction(Call::new(&incrementer_addr, "inc"))
        .await
        .expect("failed to `submit_transaction`");
    assert_eq!(
        ui.execute_rpc(Call::new(&incrementer_addr, "get")).await?,
        "5"
    );

    Ok(())
}
