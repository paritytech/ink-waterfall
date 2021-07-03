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

#[cfg(not(feature = "polkadot-js-ui"))]
pub mod canvas_ui;

#[cfg(feature = "polkadot-js-ui")]
pub mod polkadot_js;

#[cfg(feature = "polkadot-js-ui")]
use convert_case::{
    Case,
    Casing,
};

use async_trait::async_trait;
use fantoccini::{
    error::CmdError,
    Client,
    ClientBuilder,
};
use lazy_static::lazy_static;
use serde_json::{
    self,
    map::Map,
    value::Value,
};
use std::{
    path::PathBuf,
    process,
    sync::Mutex,
};

lazy_static! {
    static ref PICKED_PORTS: Mutex<Vec<u16>> = Mutex::new(vec![]);
}

#[async_trait]
pub trait ContractsUi {
    /// Returns the address for a given `name`.
    async fn name_to_address(
        &mut self,
        name: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Returns the balance postfix numbers.
    async fn balance_postfix(
        &mut self,
        account: String,
    ) -> Result<u128, Box<dyn std::error::Error>>;

    /// Uploads the contract behind `contract_path`.
    async fn execute_upload(
        &mut self,
        upload_input: Upload,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Executes the RPC call `call`.
    async fn execute_rpc(
        &mut self,
        call: Call,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Executes the transaction `call`.
    async fn execute_transaction(&mut self, call: Call) -> Result<Events, Error>;
}

/// Holds everything necessary to interact with the user interface.
pub struct Ui {
    client: Client,
    geckodriver: process::Child,
}

impl Ui {
    /// Creates a new `Ui` instance.
    ///
    /// As part of this set-up a `geckodriver` instance is spawned to a free port.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        crate::utils::assert_canvas_node_running();

        let mut port = None;
        for retry in 0..10 {
            let port_candidate = portpicker::pick_unused_port().expect("no free port");
            log::info!("picked free port candidate {}", port_candidate);

            // add this port to a global variable and check that no other thread has yet chosen it.
            let mut picked_ports =
                PICKED_PORTS.lock().expect("failed locking `PICKED_PORTS`");
            log::info!("picked ports {:?}", picked_ports);
            if !picked_ports.contains(&port_candidate) {
                picked_ports.push(port_candidate);
                port = Some(port_candidate);
                break
            } else {
                log::info!("port {} was already chosen by another thread, picking another one (try {})", port_candidate, retry);
            }
        }
        let port = port.expect("no free port could be determined!");
        log::info!("picked free port {} for geckodriver instance", port);

        // the output is unfortunately always printed
        // https://users.rust-lang.org/t/cargo-test-printing-println-output-from-child-threads/11627
        // https://github.com/rust-lang/rust/issues/35136
        let geckodriver = process::Command::new("geckodriver")
            .args(&["--port", &port.to_string(), "--log", "fatal"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("geckodriver can not be spawned");

        // connect to webdriver instance that is listening on that port
        let client = ClientBuilder::native()
            .capabilities(get_capabilities())
            .connect(&format!("http://localhost:{}", port))
            .await?;
        Ok(Self {
            client,
            geckodriver,
        })
    }

    /// Closes the `client`.
    ///
    /// It would be better to have this in `Ui::Drop`, but this is not possible
    /// due to the async nature of the `client.close()` method.
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !closing_enabled() {
            log::info!(
                "keeping client open due to env variable `WATERFALL_CLOSE_BROWSER`"
            );
            return Ok(())
        }
        log::debug!("closing client");
        self.client.close().await?;
        log::debug!("closed client");
        Ok(())
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        if !closing_enabled() {
            log::info!(
                "keeping browser open due to env variable `WATERFALL_CLOSE_BROWSER`"
            );
            return
        }
        // We kill the `geckodriver` instance here and not in `Ui::shutdown()`.
        // The reason is that if a test fails (e.g. due to an assertion), then the test
        // will be interrupted and the shutdown method at the end of a test will not
        // be reached, but this drop will.
        log::debug!("killing geckodriver");
        self.geckodriver
            .kill()
            .expect("unable to kill geckodriver, it probably wasn't running");
        log::debug!("killed geckodriver");
    }
}

#[derive(Debug)]
pub enum Error {
    ExtrinsicFailed(Events),
    Other(Box<dyn std::error::Error>),
}

impl From<CmdError> for Error {
    fn from(cmd_err: CmdError) -> Self {
        Error::Other(Box::new(cmd_err))
    }
}

#[derive(Clone, Debug)]
pub struct Payment {
    /// The payment.
    payment: String,
    /// The unit of payment.
    unit: String,
}

#[derive(Debug)]
pub struct Event {
    /// The header text returned in a status event by the UI.
    header: String,
    /// The status text returned in a status event by the UI.
    status: String,
}

#[derive(Debug)]
pub struct Events {
    /// The events returned by the UI as a result of a RPC call or a transaction.
    events: Vec<Event>,
}

impl Events {
    /// Creates a new `Events` instance.
    pub fn new(events: Vec<Event>) -> Self {
        Self { events }
    }

    /// Returns `true` if the `event` is contained in these events.
    pub fn contains(&self, event: &str) -> bool {
        self.events
            .iter()
            .any(|evt| evt.header.contains(event) || evt.status.contains(event))
    }
}

#[derive(Clone)]
pub struct Call {
    /// Address of the contract.
    contract_address: String,
    /// Method to execute.
    method: String,
    /// Maximum gas allowed.
    max_gas_allowed: Option<String>,
    /// Values to pass along.
    values: Vec<(String, String)>,
    /// The payment to send with the call.
    payment: Option<Payment>,
    /// The account from which to execute the call.
    caller: Option<String>,
}

impl Call {
    /// Creates a new `Transaction` instance.
    pub fn new(contract_address: &str, method: &str) -> Self {
        let method = method.to_string();

        // the `polkadot-js` ui displays method names in camel-case
        #[cfg(feature = "polkadot-js-ui")]
        let method = method.to_case(Case::Camel);

        Self {
            contract_address: contract_address.to_string(),
            method,
            max_gas_allowed: None,
            values: Vec::new(),
            payment: None,
            caller: None,
        }
    }

    /// Adds an initial value.
    ///
    /// TODO: Make `val` an enum of `Boolean` and `String`.
    pub fn push_value(mut self, key: &str, val: &str) -> Self {
        self.values.push((key.to_string(), val.to_string()));
        self
    }

    /// Sets the maximum gas allowed.
    pub fn max_gas(mut self, max_gas: &str) -> Self {
        self.max_gas_allowed = Some(max_gas.to_string());
        self
    }

    /// Sets the payment submitted with the call.
    pub fn payment(mut self, payment: &str, unit: &str) -> Self {
        self.payment = Some(Payment {
            payment: payment.to_string(),
            unit: unit.to_string(),
        });
        self
    }

    /// Sets the account from which to execute the call.
    pub fn caller(mut self, caller: &str) -> Self {
        self.caller = Some(caller.to_string());
        self
    }
}

#[derive(Clone)]
pub struct Upload {
    /// Path to the contract which should be uploaded.
    contract_path: PathBuf,
    /// Values to instantiate the contract with.
    initial_values: Vec<(String, String)>,
    /// Items to add as instantiatiation values.
    items: Vec<(String, String)>,
    /// Initial endowment of the contract.
    endowment: String,
    /// Unit for initial endowment of the contract.
    endowment_unit: String,
    /// Maximum allowed gas.
    #[allow(dead_code)]
    max_allowed_gas: String,
    /// The constructor to use. If not specified the default selected one is used.
    constructor: Option<String>,
    /// The caller to use. If not specified the default selected one is used.
    caller: Option<String>,
}

impl Upload {
    /// Creates a new `Upload` instance.
    pub fn new(contract_path: PathBuf) -> Self {
        Self {
            contract_path,
            initial_values: Vec::new(),
            items: Vec::new(),
            endowment: "1000".to_string(),
            endowment_unit: "Unit".to_string(),
            max_allowed_gas: "5000".to_string(),
            constructor: None,
            caller: None,
        }
    }

    /// Adds an initial value.
    ///
    /// TODO: Make `val` an enum of `Boolean` and `String`.
    pub fn push_initial_value(mut self, key: &str, val: &str) -> Self {
        self.initial_values.push((key.to_string(), val.to_string()));
        self
    }

    /// Adds an item.
    pub fn add_item(mut self, key: &str, val: &str) -> Self {
        self.items.push((key.to_string(), val.to_string()));
        self
    }

    /// Sets the contract path.
    #[allow(dead_code)]
    pub fn contract_path(mut self, path: PathBuf) -> Self {
        self.contract_path = path;
        self
    }

    /// Sets the initial endowment.
    pub fn endowment(mut self, endowment: &str, unit: &str) -> Self {
        self.endowment = endowment.to_string();
        self.endowment_unit = unit.to_string();
        self
    }

    /// Sets the max allowed gas.
    #[allow(dead_code)]
    pub fn max_allowed_gas(mut self, max: &str) -> Self {
        self.max_allowed_gas = max.to_string();
        self
    }

    /// Sets the constructor to use for instantiation.
    pub fn constructor(mut self, constructor: &str) -> Self {
        self.constructor = Some(constructor.to_string());
        self
    }

    /// Sets the caller to use for instantiation.
    pub fn caller(mut self, caller: &str) -> Self {
        self.caller = Some(caller.to_string());
        self
    }
}

/// Returns `true` if the shutdown procedure should be executed after a test run.
/// This mostly involves closing the browser.
///
/// Returns `false` if the environment variable `WATERFALL_CLOSE_BROWSER` is set to `false`.
fn closing_enabled() -> bool {
    std::env::var("WATERFALL_CLOSE_BROWSER")
        .unwrap_or("true".to_string())
        .parse()
        .expect("unable to parse `WATERFALL_CLOSE_BROWSER` into `bool`")
}

/// Returns the capabilities with which the `fantoccini::Client` is instantiated.
#[cfg(feature = "headless")]
fn get_capabilities() -> Map<String, Value> {
    let mut caps = Map::new();
    let opts = serde_json::json!({ "args": ["--headless"] });
    caps.insert("moz:firefoxOptions".to_string(), opts.clone());
    caps
}

/// Returns the capabilities with which the `fantoccini::Client` is instantiated.
#[cfg(not(feature = "headless"))]
fn get_capabilities() -> Map<String, Value> {
    Map::new()
}
