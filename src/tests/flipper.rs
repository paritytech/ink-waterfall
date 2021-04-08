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

use crate::utils::canvas_ui::CanvasUI;

#[tokio::test]
async fn works() -> Result<(), Box<dyn std::error::Error>> {
    // given
    let mut canvas_ui = CanvasUI::new().await?;
    let contract_addr = canvas_ui
        .upload(
            "/ci-cache/ink-waterfall/targets/master/run/ink/flipper.contract"
        )
        .await?;
    assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get").await?, "false");

    // when
    canvas_ui
        .execute_transaction(&contract_addr, "flip")
        .await?;

    // then
    assert_eq!(canvas_ui.execute_rpc(&contract_addr, "get").await?, "true");
    canvas_ui.close().await?;
    Ok(())
}
