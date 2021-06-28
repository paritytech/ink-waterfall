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

//! Tests for the `ERC-721 `example.

use crate::{
    uis::{
        Call,
        Ui,
        Upload,
    },
    utils::{
        self,
        cargo_contract,
    },
};
use lang_macro::waterfall_test;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[waterfall_test]
async fn erc721(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("erc721/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");

    let contract_addr = ui.execute_upload(Upload::new(contract_file)).await?;

    ui.execute_transaction(
        Call::new(&contract_addr, "mint")
            .caller("ALICE")
            .push_value("id", "123"),
    )
    .await
    .expect("`mint` must succeed");
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of")
                .push_value("owner", "ALICE")
                .caller("ALICE")
        )
        .await?,
        "1"
    );

    // TODO https://github.com/paritytech/canvas-ui/issues/84 needs to be fixed first
    // assert_eq!(
    // canvas_ui
    // .execute_rpc(
    // Call::new(&contract_addr, "owner_of").push_value("id", "123")
    // )
    // .await?,
    // "ALICE"
    // );

    ui.execute_transaction(
        Call::new(&contract_addr, "transfer")
            .caller("ALICE")
            .push_value("destination", "BOB")
            .push_value("id", "123"),
    )
    .await
    .expect("`transfer` must succeed");
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "ALICE")
        )
        .await?,
        "0"
    );
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "BOB")
        )
        .await?,
        "1"
    );
    ui.execute_transaction(
        Call::new(&contract_addr, "approve")
            .caller("BOB")
            .push_value("to", "CHARLIE")
            .push_value("id", "123"),
    )
    .await
    .expect("`approve` must succeed");
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "ALICE")
        )
        .await?,
        "0"
    );

    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "BOB")
        )
        .await?,
        "1"
    );
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "CHARLIE")
        )
        .await?,
        "0"
    );

    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "DAVE")
        )
        .await?,
        "0"
    );

    ui.execute_transaction(
        Call::new(&contract_addr, "transfer_from")
            .caller("CHARLIE")
            .push_value("from", "BOB")
            .push_value("to", "DAVE")
            .push_value("id", "123"),
    )
    .await
    .expect("`transfer_from` must succeed");
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "BOB")
        )
        .await?,
        "0"
    );
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "CHARLIE")
        )
        .await?,
        "0"
    );
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "DAVE")
        )
        .await?,
        "1"
    );
    ui.execute_transaction(
        Call::new(&contract_addr, "burn")
            .caller("DAVE")
            .push_value("id", "123"),
    )
    .await
    .expect("`burn` must succeed");
    assert_eq!(
        ui.execute_rpc(
            Call::new(&contract_addr, "balance_of").push_value("owner", "DAVE")
        )
        .await?,
        "0"
    );

    Ok(())
}
