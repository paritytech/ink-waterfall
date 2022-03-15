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

//! Tests for the `upgradeable-contract` example.

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

#[waterfall_test(example = "upgradeable-contract")]
async fn upgradeable_contract_works(mut ui: Ui) -> Result<()> {
    // given
    // build and upload all contracts and extract necessary info
    let manifest_path =
        utils::example_path("upgradeable-contract/upgradeable-flipper/Cargo.toml");
    let upgradeable_flipper =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let upgradeable_flipper_hash =
        utils::extract_hash_from_contract_bundle(&upgradeable_flipper);
    let _addr = ui
        .execute_upload(Upload::new(upgradeable_flipper.clone()))
        .await?;

    let manifest_path = utils::example_path("upgradeable-contract/Cargo.toml");
    let upgradeable_contract =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let upgradeable_contract_addr = ui
        .execute_upload(
            Upload::new(upgradeable_contract.clone())
                .caller("ALICE")
                .push_initial_value("forward_to", &upgradeable_flipper_hash),
        )
        .await?;

    let manifest_path = utils::example_path("contract-transfer/Cargo.toml");
    let contract_transfer =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let contract_transfer_hash =
        utils::extract_hash_from_contract_bundle(&contract_transfer);
    let _addr = ui
        .execute_upload(Upload::new(contract_transfer.clone()))
        .await?;

    // when
    // …we change the metadata of the contract to the one of `upgradeable-flipper`.
    ui.update_metadata(&upgradeable_contract_addr, &upgradeable_flipper)
        .await?;

    // then
    // …executing some `upgradeable-flipper` rpc calls and transactions on the
    // `upgradeable-contract` must work.
    assert_eq!(
        ui.execute_rpc(Call::new(&upgradeable_contract_addr, "get"))
            .await?,
        "false"
    );
    ui.execute_transaction(Call::new(&upgradeable_contract_addr, "flip"))
        .await
        .expect("failed to `submit_transaction`");
    assert_eq!(
        ui.execute_rpc(Call::new(&upgradeable_contract_addr, "get"))
            .await?,
        "true"
    );

    ui.execute_transaction(Call::new(&upgradeable_contract_addr, "flip"))
        .await
        .expect("failed to `submit_transaction`");
    assert_eq!(
        ui.execute_rpc(Call::new(&upgradeable_contract_addr, "get"))
            .await?,
        "false"
    );

    // when
    // …we switch the metadata back from `upgradeable-flipper` to `upgradeable-contract`.
    ui.update_metadata(&upgradeable_contract_addr, &upgradeable_contract)
        .await?;

    // TODO has to be disabled until https://github.com/polkadot-js/apps/issues/6603 is fixed.
    if false {
        // then
        // …nobody but the `admin` can modify the hash to which is delegated.
        let call = Call::new(&upgradeable_contract_addr, "change_delegate_code")
            .caller("BOB")
            .push_value("new_code_hash", &contract_transfer_hash);
        ui.execute_transaction(call)
            .await
            .expect_err("must fail due to caller not being Alice");
    }

    // TODO has to be disabled until https://github.com/polkadot-js/apps/issues/5823 is fixed
    if false {
        // when
        // …we switch the delegation to `contract-transfer`.
        let call = Call::new(&upgradeable_contract_addr, "change_delegate_code")
            .caller("ALICE")
            .push_value("new_code_hash", &contract_transfer_hash);
        ui.execute_transaction(call).await.expect("must work");

        // we have to switch the metadata to `contract-transfer`
        ui.update_metadata(&upgradeable_contract_addr, &contract_transfer)
            .await?;

        // then
        // …it must be possible to transfer value to the contract.
        let result = ui
            .execute_transaction(
                Call::new(&upgradeable_contract_addr, "was_it_ten")
                    .caller("DAVE")
                    .payment("10", "pico"),
            )
            .await;
        assert!(result.is_ok());

        // TODO this is actually an issue because all tests use the same log, so if
        // some other test makes this log appear we mistake it for an entry of this test.
        assert!(utils::node_log_contains("received payment: 10\n"));
    }

    Ok(())
}
