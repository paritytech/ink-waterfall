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

use crate::{
    uis::{
        Call,
        ContractsUi,
        Event,
        Events,
        Result,
        TransactionError,
        TransactionResult,
        Upload,
    },
    utils::{
        self,
        test_name,
    },
};
use async_trait::async_trait;
use fantoccini::Locator;
use rand::Rng;
use regex::Regex;

#[async_trait]
impl ContractsUi for crate::uis::Ui {
    /// Returns the balance postfix numbers.
    async fn balance_postfix(&mut self, account: String) -> Result<u128> {
        // The `canvas-ui` doesn't display the balance, so we need to piggy-back
        // on `polkadot-js`.
        let log_id = format!("{} {}", test_name(), account.clone());
        log::info!("[{}] getting balance_postfix for {:?}", log_id, account);
        self.client
            .goto(&format!(
                "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A{}#/accounts",
                utils::node_port()
            ))
            .await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        let path = format!(
            "//div[. = '{}']/ancestor::tr//span[@class = 'ui--FormatBalance-postfix']",
            account
        );
        let balance = self
            .client
            .wait()
            .for_element(Locator::XPath(&path))
            .await?
            .text()
            .await?;
        log::info!("[{}] extracted balance {:?} for account", log_id, balance);
        Ok(balance.parse::<u128>().expect("failed parsing"))
    }

    /// Uploads the contract behind `contract_path`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_upload(&mut self, upload_input: Upload) -> Result<String> {
        let log_id = format!("{}", test_name());
        log::info!(
            "[{}] opening url for upload of {}: {:?}",
            log_id,
            upload_input
                .contract_path
                .file_name()
                .expect("file name must exist")
                .to_str()
                .expect("conversion must work"),
            url("upload")
        );

        self.client.goto(&url("upload")).await?;

        log::info!("[{}] waiting for settings to become visible", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await?;

        // We should get rid of this `sleep`. The problem is that the "Skip Intro" button
        // sometimes appears after a bit of time and sometimes it doesn't (if it was already
        // clicked away during the session).
        std::thread::sleep(std::time::Duration::from_secs(2));

        log::info!("[{}] click skip intro button, if it is available", log_id);
        if let Ok(skip_button) = self
            .client
            .find(Locator::XPath("//button[contains(text(),'Skip Intro')]"))
            .await
        {
            log::info!("[{}] found skip button", log_id);
            skip_button.click().await?;
        } else {
            // The "Skip Intro" button is not always there, e.g. if multiple contracts
            // are deployed subsequently in the same browser session by one test.
            log::info!(
                "[{}] did not find 'Skip Intro' button, ignoring it.",
                log_id
            );
        }

        log::info!("[{}] click upload", log_id);
        self.client
            .find(Locator::XPath(
                "//*[contains(text(),'Upload & Instantiate Contract')]",
            ))
            .await?
            .click()
            .await?;

        log::info!("[{}] injecting jquery", log_id);
        // The `inject` script will retry to load jQuery every 10 seconds.
        // This is because the CI sometimes has spurious network errors.
        let inject = String::from(
            "(function (){\
                    var d = document;\
                    if (!d.getElementById('jquery')) {\
                        function load_jquery() {\
                            var d = document;\
                            var s = d.createElement('script');\
                            s.src = 'https://code.jquery.com/jquery-3.6.0.min.js';\
                            s.id = 'jquery';\
                            d.body.appendChild(s);\
                        }\
                        var jTimer = setInterval(function() {\
                            load_jquery();\
                        }, 10000);\
                        load_jquery();\
                        (function() {\
                            var nTimer = setInterval(function() {\
                                if (window.jQuery) {\
                                    $('body').append('<div id=\"jquery-ready\"></div');\
                                    clearInterval(nTimer);\
                                    clearInterval(jTimer);\
                                }\
                            }, 100);\
                        })();\
                    }\
                })();",
        );
        self.client.execute(&*inject, Vec::new()).await?;

        log::info!("[{}] waiting for jquery", log_id);
        self.client
            .wait()
            .for_element(Locator::Css("#jquery-ready"))
            .await?;

        log::info!("[{}] click combobox", log_id);
        self.client
            .execute("$('[role=combobox]').click()", Vec::new())
            .await?;

        log::info!("[{}] click alice", log_id);
        self.client
            .execute("$('[name=alice]').click()", Vec::new())
            .await?;

        log::info!("[{}] set input {:?}", log_id, upload_input.contract_path);
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

        log::info!("[{}] click details", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
                "//*[contains(text(),'Constructor Details')]",
            ))
            .await?
            .click()
            .await?;

        if let Some(caller) = &upload_input.caller {
            // open listbox for accounts
            log::info!("[{}] click listbox for accounts", log_id);
            self.client
                .wait()
                .for_element(Locator::XPath(
                    "//*[contains(text(),'instantiation account')]/ancestor::div[1]/div",
                ))
                .await?
                .click()
                .await?;

            // choose caller
            log::info!("[{}] choose {:?}", log_id, caller);
            let path = format!("//div[@name = '{}']", caller.to_lowercase());
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        for (key, value) in upload_input.initial_values.iter() {
            log::info!(
                "[{}] inserting '{}' into input field '{}'",
                log_id,
                value,
                key
            );
            let path = format!(
                "//label/*[contains(text(),'{}')]/ancestor::div[1]//*/input",
                key
            );
            let mut input = self.client.find(Locator::XPath(&path)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&value).await?;
        }

        for (key, value) in upload_input.items.iter() {
            log::info!("[{}] adding item '{}' for '{}'", log_id, value, key);
            let add_item = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&value).await?;
        }

        if let Some(ref constructor) = upload_input.constructor {
            log::info!("[{}] click constructor list box", log_id);
            self.client
                .wait().for_element(Locator::XPath(
                    "//label/*[contains(text(),'Instantiation Constructor')]/ancestor::div[1]//*/div[@role='listbox']"
                ))
                .await?.click().await?;

            log::info!("[{}] click constructor option {}", log_id, constructor);
            let path = format!(
                "//span[@class = 'ui--MessageSignature-name' and contains(text(),'{}')]",
                constructor
            );
            self.client
                .wait()
                .for_element(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        log::info!("[{}] set endowment to {}", log_id, upload_input.endowment);
        let mut input = self
            .client
            .find(Locator::XPath(
                "//label/*[contains(text(),'Endowment')]/ancestor::div[1]//*/input",
            ))
            .await?;
        input.clear().await?;
        input.send_keys(&upload_input.endowment).await?;

        log::info!("[{}] click endowment list box", log_id);
        self.client
            .wait().for_element(Locator::XPath("//label/*[contains(text(),'Endowment')]/ancestor::div[1]//*/div[@role='listbox']"))
            .await?;

        log::info!(
            "[{}] click endowment unit option {}",
            log_id,
            upload_input.endowment_unit,
        );
        let path = format!(
            "//div[@role='option']/span[contains(text(),'{}')]",
            upload_input.endowment_unit
        );
        self.client
            .wait()
            .for_element(Locator::XPath(&path))
            .await?;

        // the react toggle button cannot be clicked if it is not in view
        self.client
            .execute(
                "$(':contains(\"Unique Instantiation Salt\")')[0].scrollIntoView();",
                Vec::new(),
            )
            .await?;
        std::thread::sleep(std::time::Duration::from_secs(3));

        log::info!("[{}] check 'Unique Instantiation Salt' checkbox", log_id);
        let path = "//*[contains(text(),'Unique Instantiation Salt')]/ancestor::div[1]//div[contains(@class,'ui--Toggle')]/div";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        log::info!("[{}] click instantiate", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Instantiate')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] click sign and submit", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//button[contains(text(),'Sign & Submit')]"))
            .await?
            .click()
            .await?;

        log::info!(
            "[{}] upload: waiting for either success or failure notification",
            log_id
        );

        let mut res;
        for retry in 0..21 {
            std::thread::sleep(std::time::Duration::from_secs(3));
            res = self.client.find(
                Locator::XPath("//*[contains(text(),'Dismiss') or contains(text(),'usurped') or contains(text(),'Priority is too low')]")
            ).await;
            if res.is_ok() {
                log::info!("[{}] upload: success on try {}", log_id, retry,);
                break
            } else {
                log::info!(
                    "[{}] upload: try {} - waiting for either success or failure notification",
                    log_id,
                    retry,
                );

                let statuses = self
                    .client
                    .find_all(Locator::XPath(
                        "//div[contains(@class, 'ui--Status')]//div[@class = 'desc' or @class = 'header']",
                    ))
                    .await?;
                log::info!(
                    "[{}] upload retry: found {} status messages",
                    log_id,
                    statuses.len(),
                );
                for mut el in statuses {
                    log::info!("[{}] upload retry, text: {:?}", log_id, el.text().await?);
                }

                if retry == 20 {
                    log::info!(
                        "[{}] timed out on waiting for upload! next recursion.",
                        log_id,
                    );
                    return self.execute_upload(upload_input.clone()).await
                } else {
                    log::info!("[{}] timed out on waiting for upload! sleeping.", log_id,);
                }
            }
        }

        // extract all status messages
        let statuses = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']",
            ))
            .await?;
        log::info!("[{}] found {} status messages", log_id, statuses.len(),);
        let mut statuses_processed = Vec::new();
        for mut el in statuses {
            log::info!("[{}] text {:?}", log_id, el.text().await?);
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
            log::info!(
                "[{}] found status message {:?} with {:?}",
                log_id,
                header,
                status,
            );
            statuses_processed.push(Event { header, status });
        }
        let events = Events::new(statuses_processed);

        if events.contains("Priority is too low") {
            log::info!(
                "[{}] found priority too low during upload! trying again!",
                log_id
            );
            return self.execute_upload(upload_input.clone()).await
        } else if events.contains("usurped") {
            log::info!("[{}] found usurped for upload! trying again!", log_id);
            return self.execute_upload(upload_input.clone()).await
        } else {
            log::info!(
                "[{}] did not find priority too low in {} status messages",
                log_id,
                events.events.len()
            );
        }
        assert!(
            events.contains("system.ExtrinsicSuccess"),
            "uploading contract must succeed"
        );

        log::info!("[{}] dismiss notifications", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//*[contains(text(),'Dismiss')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] click execute", log_id);
        self.client
            .find(Locator::XPath(
                "//button[contains(text(),'Execute Contract')]",
            ))
            .await?
            .click()
            .await?;

        let re = Regex::new("/execute/([0-9a-zA-Z]+)/0").expect("invalid regex");
        let client_url = self.client.current_url().await?;
        let url_fragment = client_url.fragment().expect("fragment must exist in url");
        log::info!("[{}] url fragment {:?}", log_id, url_fragment);
        let captures = re
            .captures(url_fragment)
            .expect("contract address cannot be extracted from client url");
        let addr = captures.get(1).expect("no capture group").as_str();
        log::info!("[{}] contract address {:?}", log_id, addr);
        Ok(String::from(addr))
    }

    /// Executes the RPC call `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_rpc(&mut self, call: Call) -> Result<String> {
        let log_id = format!("{} {}", test_name(), call.method.clone());

        let url = format!("{}{}/0", url("execute/"), call.contract_address);
        log::info!(
            "[{}] opening url for rpc {:?}: {:?}",
            log_id,
            call.method,
            url
        );
        self.client.goto(url.as_str()).await?;

        // hack to get around a failure of the ui for the multisig tests.
        // the ui fails displaying the flipper contract execution page, but
        // it strangely works if tried again after some time.
        log::info!("[{}] sleep for {}", log_id, url);
        std::thread::sleep(std::time::Duration::from_secs(2));

        self.client.refresh().await?;
        self.client.goto(url.as_str()).await?;

        // open listbox for methods
        log::info!("[{}] click listbox", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
            ))
            .await?
            .click()
            .await?;

        // click `method`
        log::info!("[{}] choose {:?}", log_id, call.method);
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[text() = '{}']", call.method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // Open listbox
        log::info!("[{}] open listbox for rpc vs. transaction", log_id);
        let path = "//*[contains(text(),'Send as RPC call')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        // Send as RPC call
        log::info!("[{}] select 'Send as RPC call'", log_id);
        let path = "//*[contains(text(),'Send as RPC call')]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        // possibly set max gas
        if let Some(max_gas) = call.max_gas_allowed {
            // click checkbox
            log::info!(
                "[{}] unset 'use estimated gas' checkbox if it exists",
                log_id
            );
            let path = "//*[contains(text(),'use estimated gas')]/ancestor::div[1]/div";
            let checkbox = self.client.find(Locator::XPath(path)).await;

            if let Ok(checkbox) = checkbox {
                log::info!(
                    "[{}] unsetting 'use estimated gas' checkbox - it exists",
                    log_id
                );
                checkbox.click().await?;
            }

            log::info!("[{}] entering max gas {:?}", log_id, max_gas);
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
            // if the value is `Yes` or `No` we assume it's a listbox with a boolean
            let mut value = transform_value(&value);
            if value == "Yes" || value == "No" {
                log::info!("[{}] opening dropdown list '{}'", log_id, key);
                let path =
                    format!("//label/*[contains(text(),'{}')]/ancestor::div[1]", key);
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .click()
                    .await?;

                log::info!("[{}] chossing option '{}''", log_id, value);
                let path = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]//*/div[@role = 'option']/span[text() = '{}']", key, value);
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .click()
                    .await?;
            } else {
                log::info!("[{}] entering {:?} into {:?}", log_id, &value, &key);
                let path = format!(
                    "//*[contains(text(),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
                    key
                );
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .clear()
                    .await?;
                value.push('\n');
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .send_keys(&value)
                    .await?
            }
        }

        // possibly add items
        for (key, value) in call.items.iter() {
            log::info!("[{}] adding item '{}' for '{}'", log_id, value, key);
            let add_item = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&value).await?;
        }

        // click call
        let mut txt = None;
        log::info!("[{}] click rpc call", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Call')]"))
            .await?
            .click()
            .await?;
        for waited in 0..21 {
            log::info!("[{}] waiting for rpc call outcome {}", log_id, waited);
            std::thread::sleep(std::time::Duration::from_secs(3));
            let el = self.client.find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']/div[1]")).await;
            if let Ok(mut el) = el {
                log::info!("[{}] found rpc call outcome", log_id);
                txt = Some(el.text().await?);
                log::info!(
                    "[{}] found rpc call outcome text {}",
                    log_id,
                    txt.clone().expect("txt exists here")
                );
                break
            }

            if waited % 5 == 0 {
                // if txt.is_none() {
                // let html = self.client.find(Locator::XPath("//div[@class = 'outcomes']")).await?.html(true).await?;
                // if html.contains("Error: OutOfGas") {
                // panic!("[{}] An `OutOfGas` error occurred for this RPC", log_id);
                // }
                // }
                log::info!("[{}] click rpc call again in {}", log_id, waited);
                self.client
                    .find(Locator::XPath("//button[contains(text(),'Call')]"))
                    .await?
                    .click()
                    .await?;
            }
        }
        let mut txt = txt.expect("[{}] no outcome txt found after retrying!");

        // wait for outcomes
        log::info!("outcomes value {:?}", txt);
        if txt == "0x000000…00000000" {
            txt = String::from("<empty>");
        }
        txt = txt
            .trim_start_matches("Some(\n")
            .trim_end_matches("\n)")
            .to_string();
        Ok(txt)
    }

    /// Executes the transaction `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_transaction(&mut self, call: Call) -> TransactionResult<Events> {
        let log_id = format!("{} {}", test_name(), call.method.clone());
        let url = url(&format!("execute/{}/0", call.contract_address));
        log::info!(
            "[{}] opening url for executing transaction {:?}: {:?}",
            log_id,
            call.method,
            url
        );
        self.client.goto(url.as_str()).await?;
        self.client.refresh().await?;

        // open listbox for methods
        log::info!("[{}] click listbox", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
            ))
            .await?
            .click()
            .await?;

        // click `method`
        log::info!("[{}] choose {:?}", log_id, call.method);
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[text() = '{}']", call.method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // Open listbox
        log::info!("[{}] open listbox for rpc vs. transaction", log_id);
        let path = "//*[contains(text(),'Send as transaction')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        // Send as transaction
        log::info!("[{}] select 'Send as transaction'", log_id);
        let path = "//*[contains(text(),'Send as transaction')]/ancestor::div[1]";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        if let Some(caller) = &call.caller {
            // open listbox for accounts
            log::info!("[{}] click listbox for accounts", log_id);
            self.client
                .wait()
                .for_element(Locator::XPath(
                    "//*[contains(text(),'Call from Account')]/ancestor::div[1]/div",
                ))
                .await?
                .click()
                .await?;

            // choose caller
            log::info!("[{}] choose {:?}", log_id, caller);
            let path = format!("//*[contains(text(),'Call from Account')]/ancestor::div[1]//div[@name = '{}']", caller.to_lowercase());
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        // Possibly add payment
        if let Some(payment) = &call.payment {
            // Open listbox
            log::info!("[{}] open listbox for payment units", log_id);
            let path = format!("//*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]", payment.unit);
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;

            log::info!("[{}] click payment unit option {}", log_id, payment.unit);
            let path = format!(
                "//div[@role='option']/span[contains(text(),'{}')]/ancestor::div[1]",
                payment.unit
            );
            self.client
                .wait()
                .for_element(Locator::XPath(&path))
                .await?
                .click()
                .await?;

            log::info!("[{}] entering payment {:?}", log_id, payment.payment);
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
        if let Some(max_gas) = &call.max_gas_allowed {
            // click checkbox
            log::info!("[{}] unset 'use estimated gas' checkbox", log_id);
            let path = "//*[contains(text(),'use estimated gas')]/ancestor::div[1]/div";
            self.client
                .find(Locator::XPath(path))
                .await?
                .click()
                .await?;

            log::info!("[{}] entering max gas {:?}", log_id, max_gas);
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
        for (key, value) in &call.values {
            // if the value is `Yes` or `No` we assume it's a listbox with a boolean
            let mut value = transform_value(&value);
            if value == "Yes" || value == "No" {
                log::info!("[{}] opening dropdown list '{}'", log_id, key);
                let path =
                    format!("//label/*[contains(text(),'{}')]/ancestor::div[1]", key);
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .click()
                    .await?;

                log::info!("[{}] chossing option '{}''", log_id, value);
                let path = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]//*/div[@role = 'option']/span[text() = '{}']", key, value);
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .click()
                    .await?;
            } else {
                log::info!("[{}] entering {:?} into {:?}", log_id, &value, &key);
                let path = format!(
                    "//*[contains(text(),'Message to Send')]/ancestor::div[1]/following-sibling::div[1]//*[contains(text(),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
                    key
                );
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .clear()
                    .await?;
                value.push('\n');
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .send_keys(&value)
                    .await?;
            }
        }

        // possibly add items
        for (key, value) in call.items.iter() {
            log::info!("[{}] adding item '{}' for '{}'", log_id, value, key);
            let add_item = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//label/*[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&value).await?;
        }

        // click call
        log::info!("[{}] transaction click call", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//button[contains(text(),'Call')]"))
            .await?
            .click()
            .await?;

        // click sign and submit
        log::info!("[{}] sign and submit", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//button[contains(text(),'Sign & Submit')]"))
            .await?
            .click()
            .await?;

        log::info!(
            "[{}] transaction: waiting for either success or failure notification",
            log_id
        );
        let mut res;
        for retry in 0..21 {
            std::thread::sleep(std::time::Duration::from_secs(3));
            res = self.client.find(
                Locator::XPath("//*[contains(text(),'Dismiss') or contains(text(),'usurped') or contains(text(),'Priority is too low')]")
            ).await;
            if res.is_ok() {
                log::info!(
                    "[{}] transaction: success on try {} for {:?}",
                    log_id,
                    retry,
                    call.method
                );
                break
            } else {
                log::info!(
                    "[{}] transaction: try {} - waiting for either success or failure notification {:?}",
                    log_id,
                    retry,
                    call.method
                );

                let statuses = self
                    .client
                    .find_all(Locator::XPath(
                        "//div[contains(@class, 'ui--Status')]//div[@class = 'desc' or @class = 'header']",
                    ))
                    .await?;
                log::info!(
                    "[{}] transaction retry: found {:?} status messages for {:?}",
                    log_id,
                    statuses.len(),
                    call.method
                );
                for mut el in statuses {
                    log::info!("transaction retry, text: {:?}", el.text().await?);
                }

                if retry == 20 {
                    log::info!(
                        "[{}] timed out on waiting for {:?} transaction! next recursion.",
                        log_id,
                        call.method
                    );
                    return self.execute_transaction(call.clone()).await
                } else {
                    log::info!(
                        "[{}] timed out on waiting for {:?} transaction! sleeping.",
                        log_id,
                        call.method
                    );
                }
            }
        }

        // extract all status messages
        let statuses = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']",
            ))
            .await?;
        log::info!("[{}] found {:?} status messages", log_id, statuses.len());
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
            log::info!(
                "[{}] found status message {:?} with {:?}",
                log_id,
                header,
                status
            );
            statuses_processed.push(Event { header, status });
        }
        let events = Events::new(statuses_processed);

        if events.contains("Priority is too low") {
            log::info!(
                "[{}] found priority too low during transaction execution of {:?}! trying again!",
                log_id,
                call.method
            );
            return self.execute_transaction(call.clone()).await
        } else if events.contains("usurped") {
            log::info!(
                "[{}] found usurped for transaction {:?}! trying again!",
                log_id,
                call.method
            );
            {
                let mut rng = rand::thread_rng();
                let rand = rng.gen_range(0..30_000);
                log::info!("[{}] sleeping for rand {:?} after usurped", log_id, rand);
                std::thread::sleep(std::time::Duration::from_millis(rand));
            }
            return self.execute_transaction(call.clone()).await
        } else {
            log::info!(
                "[{}] did not find priority too low in {:?} status messages",
                log_id,
                events.events.len()
            );
        }

        let success = events.contains("system.ExtrinsicSuccess");
        let failure = events.contains("system.ExtrinsicFailed");
        match (success, failure) {
            (true, false) => TransactionResult::Ok(events),
            (false, true) => TransactionResult::Err(TransactionError::ExtrinsicFailed(events)),
            (false, false) => panic!("ERROR: Neither 'ExtrinsicSuccess' nor 'ExtrinsicFailed' was found in status messages!"),
            (true, true) => panic!("ERROR: Both 'ExtrinsicSuccess' nor 'ExtrinsicFailed' was found in status messages!"),
        }
    }
}

/// Returns the UI's base URL.
///
/// If the env variable `UI_URL` is set that one is taken, otherwise the default
/// `https://paritytech.github.io/canvas-ui` is returned.
fn base_url() -> String {
    let base_url = std::env::var("UI_URL")
        .unwrap_or(String::from("https://paritytech.github.io/canvas-ui"));

    // strip a possibly ending `/` from he URL, since a URL like `http://foo//bar`
    // can cause issues.
    let mut url = base_url.trim_end_matches('/').to_string();
    url.push_str(&format!(
        "?rpc=ws%3A%2F%2F127.0.0.1%3A{}#/",
        utils::node_port()
    ));
    url
}

/// Returns the URL to the `path` in the UI.
///
/// Defaults to https://paritytech.github.io/canvas-ui as the base URL.
fn url(path: &str) -> String {
    format!("{}{}", base_url(), path)
}

fn transform_value(value: &str) -> String {
    match value {
        "true" => String::from("Yes"),
        "false" => String::from("No"),
        _ => value.to_string(),
    }
}
