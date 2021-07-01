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

use crate::uis::{
    Call,
    ContractsUi,
    Error,
    Event,
    Events,
    Upload,
};
use async_trait::async_trait;
use fantoccini::Locator;
use regex::Regex;

#[async_trait]
impl ContractsUi for crate::uis::Ui {
    /// Returns the address for a given `name`.
    async fn name_to_address(
        &mut self,
        name: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.client
            .goto(
                // TODO doesn't work with differen URI!
                "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts",
            )
            .await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("waiting for page to become visible");
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        log::info!("checking account {:?}", name);
        self.client
            .find(Locator::XPath(&format!("//div[text() = '{}']", name)))
            .await?
            .click()
            .await?;

        log::info!("getting address");
        let addr = self
            .client
            .find(Locator::XPath("//div[@class = 'ui--AddressMenu-addr']"))
            .await?
            .text()
            .await?;
        log::info!("got address {}", addr);
        Ok(addr)
    }

    /// Returns the balance postfix numbers.
    async fn balance_postfix(
        &mut self,
        account: String,
    ) -> Result<u128, Box<dyn std::error::Error>> {
        log::info!("getting balance_postfix for {:?}", account);
        self.client
            .goto(
                // TODO doesn't work with differen URI!
                "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts",
            )
            .await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("waiting for page to become visible");
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        let path = format!(
            "//div[. = '{}']/ancestor::tr//span[@class = 'ui--FormatBalance-postfix']",
            account
        );
        let txt = self
            .client
            .wait_for_find(Locator::XPath(&path))
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
    async fn execute_upload(
        &mut self,
        upload_input: Upload,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let foo = upload_input.contract_path.clone();
        log::info!("opening url for upload: {:?} {:?}", url("/#/upload"), foo);

        self.client.goto(&url("/#/upload")).await?;

        // We wait until the settings are visible to make sure the page is ready
        log::info!("waiting for settings to become visible {:?}", foo);
        self.client
            .wait_for_find(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await?;

        // We should get rid of this `sleep`. The problem is that the "Skip Intro" button
        // sometimes appears after a bit of time and sometimes it doesn't (if it was already
        // clicked away during the session).
        std::thread::sleep(std::time::Duration::from_secs(2));

        log::info!("click skip intro button, if it is available {:?}", foo);
        if let Ok(skip_button) = self
            .client
            .find(Locator::XPath("//button[contains(text(),'Skip Intro')]"))
            .await
        {
            log::info!("found skip button {:?}", foo);
            skip_button.click().await?;
        } else {
            // The "Skip Intro" button is not always there, e.g. if multiple contracts
            // are deployed subsequently in the same browser session by one test.
            log::info!("did not find 'Skip Intro' button, ignoring it. {:?}", foo);
        }

        log::info!("click settings 1 {:?}", foo);
        self.client
            .find(Locator::Css(".app--SideBar-settings"))
            .await?
            .click()
            .await?;

        log::info!("click local node {:?}", foo);
        self.client
            .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await?
            .click()
            .await?;

        log::info!("click upload {:?}", foo);
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Upload & Instantiate Contract')]",
            ))
            .await?
            .click()
            .await?;

        log::info!("injecting jquery {:?}", foo);
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

        log::info!("waiting for jquery {:?}", foo);
        self.client
            .wait_for_find(Locator::Css("#jquery-ready"))
            .await?;

        log::info!("click combobox {:?}", foo);
        self.client
            .execute("$('[role=combobox]').click()", Vec::new())
            .await?;

        log::info!("click alice {:?}", foo);
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

        log::info!("click settings 2 {:?}", foo);
        self.client
            .find(Locator::Css(".app--SideBar-settings"))
            .await?
            .click()
            .await?;

        // We should get rid of this `sleep`
        std::thread::sleep(std::time::Duration::from_secs(1));

        log::info!("click details {:?}", foo);
        self.client
            .find(Locator::XPath(
                "//*[contains(text(),'Constructor Details')]",
            ))
            .await?
            .click()
            .await?;

        if let Some(caller) = &upload_input.caller {
            // open listbox for accounts
            log::info!("click listbox for accounts {:?}", foo);
            self.client
                .wait_for_find(Locator::XPath(
                    "//*[contains(text(),'instantiation account')]/ancestor::div[1]/div",
                ))
                .await?
                .click()
                .await?;

            // choose caller
            log::info!("choose {:?} {:?}", caller, foo);
            let path = format!("//div[@name = '{}']", caller.to_lowercase());
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        for (key, value) in upload_input.initial_values.iter() {
            log::info!("inserting '{}' into input field '{}' {:?}", value, key, foo);
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
            log::info!("adding item '{}' for '{}' {:?}", value, key, foo);
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
            log::info!("click constructor list box {:?}", foo);
            self.client
                .wait_for_find(Locator::XPath(
                    "//label/*[contains(text(),'Instantiation Constructor')]/ancestor::div[1]//*/div[@role='listbox']"
                ))
                .await?.click().await?;

            log::info!("click constructor option {} {:?}", constructor, foo);
            let path = format!(
                "//span[@class = 'ui--MessageSignature-name' and contains(text(),'{}')]",
                constructor
            );
            self.client
                .wait_for_find(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        log::info!("set endowment to {} {:?}", upload_input.endowment, foo);
        let mut input = self
            .client
            .find(Locator::XPath(
                "//label/*[contains(text(),'Endowment')]/ancestor::div[1]//*/input",
            ))
            .await?;
        input.clear().await?;
        input.send_keys(&upload_input.endowment).await?;

        log::info!("click endowment list box {:?}", foo);
        self.client
            .wait_for_find(Locator::XPath("//label/*[contains(text(),'Endowment')]/ancestor::div[1]//*/div[@role='listbox']"))
            .await?;

        log::info!(
            "click endowment unit option {} {:?}",
            upload_input.endowment_unit,
            foo
        );
        let path = format!(
            "//div[@role='option']/span[contains(text(),'{}')]",
            upload_input.endowment_unit
        );
        self.client.wait_for_find(Locator::XPath(&path)).await?;

        // the react toggle button cannot be clicked if it is not in view
        self.client
            .execute(
                "$(':contains(\"Unique Instantiation Salt\")')[0].scrollIntoView();",
                Vec::new(),
            )
            .await?;
        std::thread::sleep(std::time::Duration::from_secs(1));

        log::info!("check 'Unique Instantiation Salt' checkbox {:?}", foo);
        let path = "//*[contains(text(),'Unique Instantiation Salt')]/ancestor::div[1]//div[contains(@class,'ui--Toggle')]/div";
        self.client
            .find(Locator::XPath(path))
            .await?
            .click()
            .await?;

        log::info!("click instantiate {:?}", foo);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Instantiate')]"))
            .await?
            .click()
            .await?;

        log::info!("click sign and submit {:?}", foo);
        self.client
            .wait_for_find(Locator::XPath("//button[contains(text(),'Sign & Submit')]"))
            .await?
            .click()
            .await?;

        log::info!(
            "upload: waiting for either success or failure notification {:?}",
            foo
        );
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss') or contains(text(),'usurped')]",
            ))
            .await?;

        // extract all status messages
        let statuses = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']",
            ))
            .await?;
        log::info!(
            "found {:?} status messages for {:?}",
            statuses.len(),
            upload_input.contract_path
        );
        let mut statuses_processed = Vec::new();
        for mut el in statuses {
            log::info!("text {:?}", el.text().await?);
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
                "found status message {:?} with {:?} for {:?}",
                header,
                status,
                upload_input.contract_path
            );
            statuses_processed.push(Event { header, status });
        }
        let events = Events::new(statuses_processed);

        if events.contains("Priority is too low") {
            log::info!(
                "found priority too low during upload of {:?}! trying again!",
                upload_input.contract_path
            );
            return self.execute_upload(upload_input.clone()).await
        } else if events.contains("usurped") {
            log::info!(
                "found usurped for upload of {:?}! trying again!",
                upload_input.contract_path
            );
            return self.execute_upload(upload_input.clone()).await
        } else {
            log::info!(
                "did not find priority too low in {:?} status messages",
                events.events.len()
            );
        }
        assert!(
            events.contains("system.ExtrinsicSuccess"),
            "uploading contract must succeed"
        );

        log::info!("dismiss notifications {:?}", foo);
        self.client
            .wait_for_find(Locator::XPath("//*[contains(text(),'Dismiss')]"))
            .await?
            .click()
            .await?;

        log::info!("click execute {:?}", foo);
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
        log::info!("contract address {:?} {:?}", addr, foo);
        Ok(String::from(addr))
    }

    /// Executes the RPC call `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_rpc(
        &mut self,
        call: Call,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!("{}{}/0", url("/#/execute/"), call.contract_address);
        log::info!("opening url for rpc {:?}: {:?}", call.method, url);
        self.client.goto(url.as_str()).await?;

        // hack to get around a failure of the ui for the multisig tests.
        // the ui fails displaying the flipper contract execution page, but
        // it strangely works if tried again after some time.
        log::info!("sleep for {}", url);
        std::thread::sleep(std::time::Duration::from_secs(2));

        self.client.refresh().await?;
        self.client.goto(url.as_str()).await?;

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
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[text() = '{}']", call.method);
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
            log::info!("unset 'use estimated gas' checkbox if it exists");
            let path = "//*[contains(text(),'use estimated gas')]/ancestor::div[1]/div";
            let checkbox = self.client.find(Locator::XPath(path)).await;

            if let Ok(checkbox) = checkbox {
                log::info!("unsetting 'use estimated gas' checkbox - it exists");
                checkbox.click().await?;
            }

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
        for (key, mut value) in call.values {
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
            value.push('\n');
            self.client
                .find(Locator::XPath(&path))
                .await?
                .send_keys(&value)
                .await?
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
        let mut txt = el.text().await?;
        log::info!("outcomes value {:?}", txt);
        if txt == "0x000000…00000000" {
            txt = String::from("<empty>");
        }
        Ok(txt)
    }

    /// Executes the transaction `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_transaction(&mut self, call: Call) -> Result<Events, Error> {
        let url = format!("{}{}/0", url("/#/execute/"), call.contract_address);
        log::info!(
            "opening url for executing transaction {:?}: {:?}",
            call.method,
            url
        );
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
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[text() = '{}']", call.method);
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

        if let Some(caller) = &call.caller {
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
        if let Some(max_gas) = &call.max_gas_allowed {
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
        for (key, value) in &call.values {
            log::info!("{}", &format!("entering {:?} into {:?}", &value, &key));
            let path = format!(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/following-sibling::div[1]//*[contains(text(),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
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

        log::info!("transaction: waiting for either success or failure notification");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss') or contains(text(),'usurped')]",
            ))
            .await?;

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

        if events.contains("Priority is too low") {
            log::info!(
                "found priority too low during transaction execution of {:?}! trying again!",
                call.method
            );
            return self.execute_transaction(call.clone()).await
        } else if events.contains("usurped") {
            log::info!(
                "found usurped for transaction {:?}! trying again!",
                call.method
            );
            return self.execute_transaction(call.clone()).await
        } else {
            log::info!(
                "did not find priority too low in {:?} status messages",
                events.events.len()
            );
        }

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
/// Returns the URL to the `path` in the UI.
///
/// Defaults to https://paritytech.github.io/canvas-ui as the base URL.
fn url(path: &str) -> String {
    let base_url: String = std::env::var("UI_URL")
        .unwrap_or(String::from("https://paritytech.github.io/canvas-ui"));

    // strip a possibly ending `/` from he URL, since a URL like `http://foo//bar`
    // can cause issues.
    let base_url = base_url.trim_end_matches('/');

    String::from(format!("{}{}", base_url, path))
}
