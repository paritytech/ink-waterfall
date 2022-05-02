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

//! Tests for the `forward-calls` example.

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

#[waterfall_test(example = "forward-calls")]
async fn forward_calls_works(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("flipper/Cargo.toml");
    let flipper_bundle =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let flipper_addr = ui
        .execute_upload(Upload::new(flipper_bundle.clone()))
        .await?;

    let manifest_path =
        utils::example_path("upgradeable-contracts/forward-calls/Cargo.toml");
    let forward_calls_bundle =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let forward_calls_addr = ui
        .execute_upload(
            Upload::new(forward_calls_bundle.clone())
                .caller("ALICE")
                .push_initial_value("forward_to", &flipper_addr),
        )
        .await?;

    let manifest_path = utils::example_path("contract-transfer/Cargo.toml");
    let transfer_bundle =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let transfer_addr = ui
        .execute_upload(Upload::new(transfer_bundle.clone()))
        .await?;

    ui.update_metadata(&forward_calls_addr, &flipper_bundle)
        .await?;

    ui.execute_transaction(Call::new(&forward_calls_addr, "flip"))
        .await
        .expect("failed to `submit_transaction`");
    assert_eq!(
        ui.execute_rpc(Call::new(&flipper_addr, "get")).await?,
        "true"
    );

    ui.execute_transaction(Call::new(&forward_calls_addr, "flip"))
        .await
        .expect("failed to `submit_transaction`");
    assert_eq!(
        ui.execute_rpc(Call::new(&flipper_addr, "get")).await?,
        "false"
    );

    ui.update_metadata(&forward_calls_addr, &forward_calls_bundle)
        .await?;

    // TODO has to be disabled until https://github.com/polkadot-js/apps/issues/6603 is fixed.
    if false {
        let call = Call::new(&forward_calls_addr, "change_forward_address")
            .caller("BOB")
            .push_value("new_address", &transfer_addr);
        ui.execute_transaction(call)
            .await
            .expect_err("must fail due to caller not being Alice");
    }

    let call = Call::new(&forward_calls_addr, "change_forward_address")
        .caller("ALICE")
        .push_value("new_address", &transfer_addr);
    ui.execute_transaction(call).await.expect("must work");

    ui.update_metadata(&forward_calls_addr, &transfer_bundle)
        .await?;

    // TODO has to be disabled until https://github.com/polkadot-js/apps/issues/5823 is fixed
    if false {
        let result = ui
            .execute_transaction(
                Call::new(&forward_calls_addr, "was_it_ten")
                    .caller("DAVE")
                    .payment("10", "pico"),
            )
            .await;
        assert!(result.is_ok());
        assert!(utils::node_log_contains("received payment: 10\n"));
    }
    // TODO this is actually an issue because all tests use the same log, so if
    // some other test makes this log appear we mistake it for an entry of this test.
    assert!(utils::node_log_contains("received payment: 10\n"));

    Ok(())
}
