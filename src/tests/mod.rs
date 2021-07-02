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

mod contract_terminate;
mod contract_transfer;
mod delegator;
mod dns;
mod erc20;

#[cfg(not(feature = "polkadot-js-ui"))]
mod erc721;

mod flipper;
mod incrementer;
mod multisig_plain;
mod rand_extension;

// TODO only enable for `canvas-ui` once https://github.com/paritytech/canvas-ui/issues/105 is fixed
#[cfg(feature = "polkadot-js-ui")]
mod trait_erc20;

mod trait_flipper;
