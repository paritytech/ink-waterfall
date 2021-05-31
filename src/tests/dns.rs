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

use crate::utils::{
    self,
    canvas_ui::{
        Call,
        CanvasUi,
        Upload,
    },
    cargo_contract,
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[waterfall_test]
async fn dns_works(mut canvas_ui: CanvasUi) -> Result<()> {
    // given
    let manifest_path = utils::example_path("dns/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = canvas_ui.execute_upload(Upload::new(contract_file)).await?;

    // when registering and setting an adress and name
    let name = "0xCAFEBABE";
    let address = "EVE";
    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "register")
                .caller("ALICE")
                .push_value("name", name),
        )
        .await
        .expect("failed to execute `register` transaction");
    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "set_address")
                .caller("ALICE")
                .push_value("name", name)
                .push_value("newAddress", address),
        )
        .await
        .expect("failed to execute `set_address` transaction");

    // then the name must resolve to the address
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "get_address")
                    .caller("EVE")
                    .push_value("name", name)
            )
            .await?,
        address
    );

    // when trying to set the address from a different caller (BOB) the transaction must fail
    let address2 = "DAVE";
    assert!(canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "set_address")
                .caller("BOB")
                .push_value("name", name)
                .push_value("new_address", address2)
        )
        .await
        .is_err());

    // but if the owner is transferred to BOB he must be able to set the address
    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "transfer")
                .caller("ALICE")
                .push_value("name", name)
                .push_value("to", "BOB"),
        )
        .await
        .expect("failed to execute `transfer` to BOB transaction");
    canvas_ui
        .execute_transaction(
            Call::new(&contract_addr, "set_address")
                .caller("BOB")
                .push_value("name", name)
                .push_value("newAddress", address2),
        )
        .await
        .expect("failed to execute `set_address` transaction from BOB");
    assert_eq!(
        canvas_ui
            .execute_rpc(
                Call::new(&contract_addr, "get_address")
                    .caller("EVE")
                    .push_value("name", name)
            )
            .await?,
        address2
    );
    Ok(())
}
