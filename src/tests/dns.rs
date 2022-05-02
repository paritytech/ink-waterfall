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

//! Tests for the `dns `example.

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

#[waterfall_test(example = "dns")]
async fn dns_works(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("dns/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui
        .execute_upload(Upload::new(contract_file).caller("ALICE"))
        .await?;

    // when registering and setting an address and name
    let name = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let owner = "EVE";
    ui.execute_transaction(
        Call::new(&contract_addr, "register")
            .caller("ALICE")
            .push_value("name", name),
    )
    .await
    .expect("failed to execute `register` transaction");
    ui.execute_transaction(
        Call::new(&contract_addr, "set_address")
            .caller("ALICE")
            .push_value("name", name)
            .push_value("newAddress", owner),
    )
    .await
    .expect("failed to execute `set_address` transaction");

    // then the name must resolve to the address
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "get_address")
                .caller("EVE")
                .push_value("name", name)
        )
        .await?,
        owner
    );

    // when trying to set the address from a different caller (BOB) the transaction must fail
    let owner2 = "DAVE";
    // TODO: enable after error forwarding has been implemented in ink! with
    // https://github.com/paritytech/ink/issues/641
    assert!(
        true || ui
            .execute_transaction(
                Call::new(&contract_addr, "set_address")
                    .caller("BOB")
                    .push_value("name", name)
                    .push_value("newAddress", owner2)
            )
            .await
            .is_err()
    );

    // but if the owner is transferred to BOB he must be able to set the address
    ui.execute_transaction(
        Call::new(&contract_addr, "transfer")
            .caller("ALICE")
            .push_value("name", name)
            .push_value("to: AccountId", "BOB"),
    )
    .await
    .expect("failed to execute `transfer` to BOB transaction");
    ui.execute_transaction(
        Call::new(&contract_addr, "set_address")
            .caller("BOB")
            .push_value("name", name)
            .push_value("newAddress", owner2),
    )
    .await
    .expect("failed to execute `set_address` transaction from BOB");
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "get_address")
                .caller("EVE")
                .push_value("name", name)
        )
        .await?,
        owner2
    );
    Ok(())
}
