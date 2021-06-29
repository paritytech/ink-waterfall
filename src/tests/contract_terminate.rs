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

//! Tests for the `contract-terminate `example.

use crate::{
    uis::{
        Call,
        Error,
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
async fn contract_terminate_works(mut ui: Ui) -> Result<()> {
    // given
    let manifest_path = utils::example_path("contract-terminate/Cargo.toml");
    let contract_file =
        cargo_contract::build(&manifest_path).expect("contract build failed");
    let contract_addr = ui.execute_upload(Upload::new(contract_file)).await?;

    // when
    let events = ui
        .execute_transaction(Call::new(&contract_addr, "terminate_me"))
        .await
        .expect("failed to execute transaction");
    assert!(events.contains("system.KilledAccount"));
    assert!(events.contains("balances.Transfer"));
    assert!(events.contains("contracts.Terminated"));

    // then
    let err = ui
        .execute_transaction(Call::new(&contract_addr, "terminate_me"))
        .await
        .expect_err("successfully executed transaction, but expected it to_fail");
    match err {
        Error::ExtrinsicFailed(events) => {
            assert!(events.contains("contracts.ContractNotFound"))
        }
        err => panic!("encountered unexpected {:?}", err),
    }
    Ok(())
}
