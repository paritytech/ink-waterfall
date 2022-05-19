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
        161, 234, 203, 74, 147, 96, 51, 212, 5, 174, 231, 9, 142, 48, 137, 201, 162, 118,
        192, 67, 239, 16, 71, 216, 125, 86, 167, 139, 70, 7, 86, 241, 33, 87, 154, 251,
        81, 29, 160, 4, 176, 239, 88, 211, 244, 232, 232, 52, 211, 234, 100, 115, 230,
        47, 80, 44, 152, 166, 62, 50, 8, 13, 86, 175, 28,
    ];
    const MESSSAGE_HASH: [u8; 32] = [
        162, 28, 244, 179, 96, 76, 244, 178, 188, 83, 230, 248, 143, 106, 77, 117, 239,
        95, 244, 171, 65, 95, 62, 153, 174, 166, 182, 28, 130, 73, 196, 208,
    ];
    const EXPECTED_COMPRESSED_PUBLIC_KEY: [u8; 33] = [
        2, 121, 190, 102, 126, 249, 220, 187, 172, 85, 160, 98, 149, 206, 135, 11, 7, 2,
        155, 252, 219, 45, 206, 40, 217, 89, 242, 129, 91, 22, 248, 23, 152,
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
        2, 141, 181, 91, 5, 219, 134, 192, 177, 120, 108, 164, 159, 9, 93, 118, 52, 76,
        158, 96, 86, 178, 240, 39, 1, 167, 231, 243, 194, 10, 171, 253, 145,
    ];
    const EXPECTED_ETH_ADDRESS: [u8; 20] = [
        9, 35, 29, 167, 177, 154, 1, 111, 158, 87, 109, 35, 177, 98, 119, 6, 47, 77, 70,
        168,
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
