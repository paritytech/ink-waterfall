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

use fantoccini::{
    error::CmdError,
    Client,
    ClientBuilder,
    Locator,
};
use psutil::process::processes;
use regex::Regex;
use serde_json::{
    self,
    map::Map,
    value::Value,
};
use std::{
    path::PathBuf,
    process,
};

/// Holds everything necessary to interact with the `canvas-ui`.
pub struct CanvasUi {
    client: Client,
    geckodriver: process::Child,
}

impl CanvasUi {
    /// Creates a new `CanvasUi` instance.
    ///
    /// As part of this set-up a `geckodriver` instance is spawned to a free port.
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        assert_canvas_node_running();

        // the output is unfortunately always printed
        // https://users.rust-lang.org/t/cargo-test-printing-println-output-from-child-threads/11627
        // https://github.com/rust-lang/rust/issues/35136
        let port = format!("{}", portpicker::pick_unused_port().expect("no free port"));
        log::info!("Picked free port {:?} for geckodriver instance", port);
        let geckodriver = process::Command::new("geckodriver")
            .args(&["--port", &port, "--log", "fatal"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("geckodriver can not be spawned");

        // connect to webdriver instance that is listening on port 4444
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
    /// It would be better to have this in `CanvasUi::Drop`, but this is not possible
    /// due to the async nature of the `client.close()` method.
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !closing_enabled() {
            log::info!(
                "keeping client open due to env variable `WATERFALL_CLOSE_BROWSER`"
            );
            return Ok(())
        }
        self.client.close().await?;
        Ok(())
    }

    /// Returns the balance postfix numbers.
    pub async fn balance_postfix(
        &mut self,
        account: String,
    ) -> Result<u128, Box<dyn std::error::Error>> {
        self.client
            .goto(
                "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts",
            )
            .await?;
        std::thread::sleep(std::time::Duration::from_secs(3));

        let path = format!(
            "//div[. = '{}']/ancestor::tr//span[@class = 'ui--FormatBalance-postfix']",
            account
        );
        let txt = self
            .client
            .find(Locator::XPath(&path))
            .await?
            .text()
            .await?;
        Ok(txt.parse::<u128>().expect("failed parsing"))
    }

    /// Uploads the contract behind `contract_path`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    pub async fn execute_upload(
        &mut self,
        upload_input: Upload,
    ) -> Result<String, Box<dyn std::error::Error>> {
        log::info!("opening {:?}", url("/#/upload"));
        self.client.goto(&url("/#/upload")).await?;

        // We wait until the settings are visible to make sure the page is ready
        log::info!("waiting for settings to become visible");
        self.client
            .wait_for_find(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await?;

        // We should get rid of this `sleep`. The problem is that the "Skip Intro" button
        // sometimes appears after a bit of time and sometimes it doesn't (if it was already
        // clicked away during the session).
        std::thread::sleep(std::time::Duration::from_secs(3));

        log::info!("click skip intro button, if it is available");
        if let Ok(skip_button) = self
            .client
            .find(Locator::XPath("//button[contains(text(),'Skip Intro')]"))
            .await
        {
            log::info!("found skip button");
            skip_button.click().await?;
        } else {
            // The "Skip Intro" button is not always there, e.g. if multiple contracts
            // are deployed subsequently in the same browser session by one test.
            eprintln!("did not find 'Skip Intro' button, ignoring it.");
        }

        log::info!("click settings");
        self.client
            .find(Locator::Css(".app--SideBar-settings"))
            .await?
            .click()
            .await?;

        log::info!("click local node");
        self.client
            .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await?
            .click()
            .await?;

        log::info!("click upload");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Upload & Instantiate Contract')]",
            ))
            .await?
            .click()
            .await?;

        log::info!("injecting jquery");
        let inject = String::from(
            "(function (){\
                    var d = document;\
                    if (!d.getElementById('jquery')) {\
                        var s = d.createElement('script');\
                        s.src = 'https://code.jquery.com/jquery-3.6.0.min.js';\
                        s.id = 'jquery';\
                        d.body.appendChild(s);\
                        (function() {\
                            var nTimer = setInterval(function() {\
                                if (window.jQuery) {\
                                    $('body').append('<div id=\"jquery-ready\"></div');\
                                    clearInterval(nTimer);\
                                }\
                            }, 100);\
                        })();\
                    }\
                })();",
        );
        self.client.execute(&*inject, Vec::new()).await?;

        log::info!("waiting for jquery");
        self.client
            .wait_for_find(Locator::Css("#jquery-ready"))
            .await?;

        log::info!("click combobox");
        self.client
            .execute("$('[role=combobox]').click()", Vec::new())
            .await?;

        log::info!("click alice");
        self.client
            .execute("$('[name=alice]').click()", Vec::new())
            .await?;

        log::info!("uploading {:?}", upload_input.contract_path);
        let mut upload = self
            .client
            .find(Locator::Css(".ui--InputFile input"))
            .await?;
        upload
            .send_keys(&upload_input.contract_path.display().to_string())
            .await?;
        self.client
            .execute("$(\".ui--InputFile input\").trigger('change')", Vec::new())
            .await?;

        // I used this for local debugging to make tooltips disappear if the cursor
        // was placed unfortunately.
        // log::info!("click sidebar");
        // self.client
        // .wait_for_find(Locator::XPath(
        // "//div[@class = 'app--SideBar']",
        // ))
        // .await?
        // .click();

        log::info!("click settings");
        self.client
            .find(Locator::Css(".app--SideBar-settings"))
            .await?
            .click()
            .await?;
        log::info!("click settings");
        self.client
            .find(Locator::Css(".app--SideBar-settings"))
            .await?
            .click()
            .await?;

        // We should get rid of this `sleep`
        std::thread::sleep(std::time::Duration::from_millis(500));

        log::info!("click details");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Constructor Details')]",
            ))
            .await?
            .click()
            .await?;

        for (key, value) in upload_input.initial_values.iter() {
            log::info!("inserting '{}' into input field '{}'", value, key);
            let path = format!(
                "//label/*[contains(text(),'{}')]/ancestor::div[1]//*/input",
                key
            );
            let mut input = self.client.find(Locator::XPath(&path)).await?;
            // we need to clear the default `0x000...` input from the field
            input.clear().await?;
            input.send_keys(&value).await?;
        }

        log::info!("set endowment to {}", upload_input.endowment);
        let mut input = self
            .client
            .find(Locator::XPath(
                "//label/*[contains(text(),'Endowment')]/ancestor::div[1]//*/input",
            ))
            .await?;
        input.clear().await?;
        input.send_keys(&upload_input.endowment).await?;

        log::info!("click endowment list box");
        self.client
            .wait_for_find(Locator::XPath("//label/*[contains(text(),'Endowment')]/ancestor::div[1]//*/div[@role='listbox']"))
            .await?;

        log::info!(
            "click endowment unit option {}",
            upload_input.endowment_unit
        );
        let path = format!(
            "//div[@role='option']/span[contains(text(),'{}')]",
            upload_input.endowment_unit
        );
        self.client.wait_for_find(Locator::XPath(&path)).await?;

        log::info!("Check 'Unique Instantiation Salt' checkbox");
        let path = "//*[contains(text(),'Unique Instantiation Salt')]/ancestor::div[1]//div[contains(@class, 'ui--Toggle')]/div";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        log::info!("click instantiate");
        self.client
            .execute("$(\"button:contains('Instantiate')\").click()", Vec::new())
            .await?;

        log::info!("click sign and submit");
        self.client
            .execute(
                "$(\"button:contains('Sign & Submit')\").click()",
                Vec::new(),
            )
            .await?;

        // h1: Contract successfully instantiated
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Contract successfully instantiated')]",
            ))
            .await?;

        log::info!("click dismiss");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss all notifications')]",
            ))
            .await?
            .click()
            .await?;

        // wait for disappearance animation to finish instead
        // otherwise the notifications might occlude buttons
        log::info!("wait for animation to finish");
        self.client
            .execute("$('.ui--Status').hide()", Vec::new())
            .await?;

        log::info!("click execute");
        self.client
            .find(Locator::XPath(
                "//button[contains(text(),'Execute Contract')]",
            ))
            .await?
            .click()
            .await?;

        let base_url = url("");
        let re = Regex::new(&format!("{}/#/execute/([0-9a-zA-Z]+)/0", base_url))
            .expect("invalid regex");
        let curr_client_url = self.client.current_url().await?;
        let captures = re
            .captures(curr_client_url.as_str())
            .expect("contract address cannot be extracted from website");
        let addr = captures.get(1).expect("no capture group").as_str();
        log::info!("contract address {:?}", addr);
        Ok(String::from(addr))
    }

    /// Executes the RPC call `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    pub async fn execute_rpc(
        &mut self,
        call: Call,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}/0", url("/#/execute/"), call.contract_address);
        self.client.goto(url.as_str()).await?;

        // open listbox for methods
        log::info!("click listbox");
        self.client
            .find(Locator::XPath(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
            ))
            .await?
            .click()
            .await?;

        // click `method`
        log::info!("choose {:?}", call.method);
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'{}')]", call.method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // Open listbox
        log::info!("Open listbox for rpc vs. transaction");
        let path = "//*[contains(text(),'Send as RPC call')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        // Send as RPC call
        log::info!("select 'Send as RPC call'");
        let path = "//*[contains(text(),'Send as RPC call')]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        // possibly set max gas
        if let Some(max_gas) = call.max_gas_allowed {
            // click checkbox
            log::info!("unset 'use estimated gas' checkbox");
            let path = "//*[contains(text(),'use estimated gas')]/ancestor::div[1]/div";
            self.client
                .find(Locator::XPath(path))
                .await?
                .click()
                .await?;

            log::info!("{}", &format!("entering max gas {:?}", max_gas));
            let path = "//*[contains(text(),'Max Gas Allowed')]/ancestor::div[1]/div//input[@type = 'text']";
            self.client
                .find(Locator::XPath(path))
                .await?
                .clear()
                .await?;
            self.client
                .find(Locator::XPath(path))
                .await?
                .send_keys(&max_gas)
                .await?;
        }

        // possibly add values
        for (key, value) in call.values {
            log::info!("{}", &format!("entering {:?} into {:?}", &value, &key));
            let path = format!(
                "//*[contains(text(),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
                key
            );
            self.client
                .find(Locator::XPath(&path))
                .await?
                .clear()
                .await?;
            self.client
                .find(Locator::XPath(&path))
                .await?
                .send_keys(&value)
                .await?;
        }

        // click call
        log::info!("click call");
        self.client
            .find(Locator::XPath("//button[contains(text(),'Call')]"))
            .await?
            .click()
            .await?;

        // wait for outcomes
        let mut el = self.client.wait_for_find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']/div[1]")).await?;
        let txt = el.text().await?;
        log::info!("outcomes value {:?}", txt);
        Ok(txt)
    }

    /// Executes the transaction `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    pub async fn execute_transaction(&mut self, call: Call) -> Result<Events, Error> {
        let url = format!("{}{}/0", url("/#/execute/"), call.contract_address);
        self.client.goto(url.as_str()).await?;
        self.client.refresh().await?;

        // open listbox for methods
        log::info!("click listbox");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
            ))
            .await?
            .click()
            .await?;

        // click `method`
        log::info!("choose {:?}", call.method);
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'{}')]", call.method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // Open listbox
        log::info!("open listbox for rpc vs. transaction");
        let path = "//*[contains(text(),'Send as transaction')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        // Send as transaction
        log::info!("select 'Send as transaction'");
        let path = "//*[contains(text(),'Send as transaction')]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        if let Some(caller) = call.caller {
            // open listbox for accounts
            log::info!("click listbox for accounts");
            self.client
                .wait_for_find(Locator::XPath(
                    "//*[contains(text(),'Call from Account')]/ancestor::div[1]/div",
                ))
                .await?
                .click()
                .await?;

            // choose caller
            log::info!("choose {:?}", caller);
            let path = format!("//div[@name = '{}']", caller.to_lowercase());
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        // Possibly add payment
        if let Some(payment) = call.payment {
            // Open listbox
            log::info!("open listbox for payment units");
            let path = format!("//*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]", payment.unit);
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;

            log::info!("click payment unit option {}", payment.unit);
            let path = format!(
                "//div[@role='option']/span[contains(text(),'{}')]/ancestor::div[1]",
                payment.unit
            );
            self.client
                .wait_for_find(Locator::XPath(&path))
                .await?
                .click()
                .await?;

            log::info!("{}", &format!("entering payment {:?}", payment.payment));
            let path = "//*[contains(text(),'Payment')]/ancestor::div[1]/div//input[@type = 'text']";
            self.client
                .find(Locator::XPath(path))
                .await?
                .clear()
                .await?;
            self.client
                .find(Locator::XPath(path))
                .await?
                .send_keys(&payment.payment)
                .await?;
        }

        // possibly set max gas
        if let Some(max_gas) = call.max_gas_allowed {
            // click checkbox
            log::info!("unset 'use estimated gas' checkbox");
            let path = "//*[contains(text(),'use estimated gas')]/ancestor::div[1]/div";
            self.client
                .find(Locator::XPath(path))
                .await?
                .click()
                .await?;

            log::info!("{}", &format!("entering max gas {:?}", max_gas));
            let path = "//*[contains(text(),'Max Gas Allowed')]/ancestor::div[1]/div//input[@type = 'text']";
            self.client
                .find(Locator::XPath(path))
                .await?
                .clear()
                .await?;
            self.client
                .find(Locator::XPath(path))
                .await?
                .send_keys(&max_gas)
                .await?;
        }

        // possibly add values
        for (key, value) in call.values {
            log::info!("{}", &format!("entering {:?} into {:?}", &value, &key));
            let path = format!(
                "//*[contains(text(),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
                key
            );
            self.client
                .find(Locator::XPath(&path))
                .await?
                .clear()
                .await?;
            self.client
                .find(Locator::XPath(&path))
                .await?
                .send_keys(&value)
                .await?;
        }

        // click call
        log::info!("click call");
        self.client
            .find(Locator::XPath("//button[contains(text(),'Call')]"))
            .await?
            .click()
            .await?;

        // wait for notification to show up
        self.client
            .wait_for_find(Locator::XPath(
                "//div[@class = 'status' and contains(text(), 'queued')]",
            ))
            .await?;

        // click sign and submit
        log::info!("sign and submit");
        self.client
            .find(Locator::XPath("//button[contains(text(),'Sign & Submit')]"))
            .await?
            .click()
            .await?;

        // maybe assert?
        log::info!("waiting for either success or failure notification");
        self.client.wait_for_find(
            Locator::XPath("//div[@class = 'status']/ancestor::div/div[@class = 'header' and (contains(text(), 'ExtrinsicSuccess') or contains(text(), 'ExtrinsicFailed'))]")
        ).await?;

        // extract all status messages
        let statuses = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']",
            ))
            .await?;
        log::info!("found {:?} status messages", statuses.len());
        let mut statuses_processed = Vec::new();
        for mut el in statuses {
            let header = el
                .find(Locator::XPath("div[@class = 'header']"))
                .await?
                .text()
                .await?;
            let status = el
                .find(Locator::XPath("div[@class = 'status']"))
                .await?
                .text()
                .await?;
            log::info!("found status message {:?} with {:?}", header, status);
            statuses_processed.push(Event { header, status });
        }
        let events = Events::new(statuses_processed);

        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss all notifications')]",
            ))
            .await?
            .click()
            .await?;

        let success = events.contains("system.ExtrinsicSuccess");
        let failure = events.contains("system.ExtrinsicFailed");
        match (success, failure) {
            (true, false) => Ok(events),
            (false, true) => Err(Error::ExtrinsicFailed(events)),
            (false, false) => panic!("ERROR: Neither 'ExtrinsicSuccess' nor 'ExtrinsicFailed' was found in status messages!"),
            (true, true) => panic!("ERROR: Both 'ExtrinsicSuccess' nor 'ExtrinsicFailed' was found in status messages!"),
        }
    }
}

impl Drop for CanvasUi {
    fn drop(&mut self) {
        if !closing_enabled() {
            log::info!(
                "keeping browser open due to env variable `WATERFALL_CLOSE_BROWSER`"
            );
            return
        }
        // We kill the `geckodriver` instance here and not in `CanvasUi::shutdown()`.
        // The reason is that if a test fails (e.g. due to an assertion), then the test
        // will be interrupted and the shutdown method at the end of a test will not
        // be reached, but this drop will.
        self.geckodriver
            .kill()
            .expect("unable to kill geckodriver, it probably wasn't running");
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

#[derive(Debug)]
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
            .any(|evt| evt.header == event || evt.status == event)
    }
}

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
        Self {
            contract_address: contract_address.to_string(),
            method: method.to_string(),
            max_gas_allowed: None,
            values: Vec::new(),
            payment: None,
            caller: None,
        }
    }

    /// Adds an initial value.
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

pub struct Upload {
    /// Path to the contract which should be uploaded.
    contract_path: PathBuf,
    /// Values to instantiate the contract with.
    initial_values: Vec<(String, String)>,
    /// Initial endowment of the contract.
    endowment: String,
    /// Unit for initial endowment of the contract.
    endowment_unit: String,
    /// Maximum allowed gas.
    #[allow(dead_code)]
    max_allowed_gas: String,
}

impl Upload {
    /// Creates a new `Upload` instance.
    pub fn new(contract_path: PathBuf) -> Self {
        Self {
            contract_path,
            initial_values: Vec::new(),
            endowment: "1000".to_string(),
            endowment_unit: "Unit".to_string(),
            max_allowed_gas: "2500".to_string(),
        }
    }

    /// Adds an initial value.
    pub fn push_initial_value(mut self, key: &str, val: &str) -> Self {
        self.initial_values.push((key.to_string(), val.to_string()));
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
}

/// Asserts that the `canvas` process is running.
fn assert_canvas_node_running() {
    let processes = processes().expect("can't get processes");
    let canvas_node_running = processes
        .into_iter()
        .filter_map(|pr| pr.ok())
        .map(|p| p.cmdline())
        .filter_map(|cmdline| cmdline.ok())
        .filter_map(|opt| opt)
        .any(|str| str.contains("canvas "));
    assert!(
        canvas_node_running,
        "ERROR: The canvas node is not running!"
    );
}

/// Returns the URL to the `path` in the Canvas UI.
///
/// Defaults to https://paritytech.github.io/canvas-ui as the base URL.
fn url(path: &str) -> String {
    let base_url: String = std::env::var("CANVAS_UI_URL")
        .unwrap_or(String::from("https://paritytech.github.io/canvas-ui"));

    // strip a possibly ending `/` from he URL, since a URL like `http://foo//bar`
    // can cause issues.
    let base_url = base_url.trim_end_matches('/');

    String::from(format!("{}{}", base_url, path))
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
