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

//! Tests for the `seal_ecdsa` example.

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

#[waterfall_test(example = "seal_ecdsa")]
async fn seal_ecdsa_recover(mut ui: Ui) -> Result<()> {
    // given
    let contract_path =
        cargo_contract::build(&utils::example_path("seal-ecdsa/Cargo.toml"))
            .expect("contract build failed");

    // when
    let addr = ui.execute_upload(Upload::new(contract_path)).await?;

    // then
    const SIGNATURE: [u8; 65] = [
        227, 196, 116, 146, 193, 140, 84, 25, 67, 95, 64, 189, 114, 44, 214, 52, 74, 32,
        9, 185, 163, 194, 83, 85, 255, 125, 97, 140, 103, 93, 122, 219, 107, 154, 147,
        255, 52, 202, 121, 212, 229, 69, 34, 15, 237, 152, 68, 253, 242, 3, 127, 226,
        186, 226, 16, 105, 214, 211, 126, 23, 204, 171, 208, 175, 27,
    ];

    const MESSSAGE_HASH: [u8; 32] = [
        167, 124, 116, 195, 220, 156, 244, 20, 243, 69, 1, 98, 189, 205, 79, 108, 213,
        78, 65, 65, 230, 30, 17, 37, 184, 220, 237, 135, 1, 209, 101, 229,
    ];

    const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
        2, 97, 42, 251, 200, 102, 43, 116, 48, 33, 219, 192, 115, 0, 250, 20, 90, 182,
        249, 92, 249, 154, 93, 168, 75, 81, 252, 149, 98, 128, 103, 248, 221,
    ];

    let recovered_pk = ui
        .execute_rpc(
            Call::new(&addr, "recover")
                .push_value("signature", &format!("0x{}", hex::encode(SIGNATURE)))
                .push_value("message_hash", &format!("0x{}", hex::encode(MESSSAGE_HASH))),
        )
        .await?;
    assert_eq!(
        recovered_pk,
        format!("0x{}", hex::encode(EXPECTED_COMPRESSED_PUBLIC_KEY))
    );

    Ok(())
}

#[waterfall_test(example = "seal_ecdsa")]
async fn seal_ecdsa_to_eth_address(mut ui: Ui) -> Result<()> {
    // given
    let contract_path =
        cargo_contract::build(&utils::example_path("seal-ecdsa/Cargo.toml"))
            .expect("contract build failed");

    // when
    let addr = ui.execute_upload(Upload::new(contract_path)).await?;

    // then
    const PUB_KEY: [u8; 33] = [
        2, 97, 42, 251, 200, 102, 43, 116, 48, 33, 219, 192, 115, 0, 250, 20, 90, 182,
        249, 92, 249, 154, 93, 168, 75, 81, 252, 149, 98, 128, 103, 248, 221,
    ];

    const EXPECTED_ETH_ADDRESS: [u8; 20] = [
        76, 189, 93, 201, 157, 117, 248, 100, 17, 91, 55, 147, 239, 207, 152, 134, 24,
        192, 194, 18,
    ];

    let eth_addr = ui
        .execute_rpc(
            Call::new(&addr, "to_eth_address")
                .push_value("pub_key", &format!("0x{}", hex::encode(PUB_KEY))),
        )
        .await?;
    assert_eq!(eth_addr, format!("0x{}", hex::encode(EXPECTED_ETH_ADDRESS)));

    Ok(())
}
