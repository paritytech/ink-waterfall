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
    Event,
    Events,
    Result,
    TransactionError,
    TransactionResult,
    Upload,
};
use async_trait::async_trait;
use fantoccini::Locator;

#[async_trait]
impl ContractsUi for crate::uis::Ui {
    /// Returns the address for a given `name`.
    async fn name_to_address(&mut self, name: &str) -> Result<String> {
        let log_id = name.clone();
        self.client
            .goto(
                // TODO doesn't work with differen URI!
                "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts",
            )
            .await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        log::info!("[{}] checking account {:?}", log_id, name);
        self.client
            .find(Locator::XPath(&format!("//div[text() = '{}']", name)))
            .await?
            .click()
            .await?;

        log::info!("[{}] getting address", log_id);
        let addr = self
            .client
            .find(Locator::XPath("//div[@class = 'ui--AddressMenu-addr']"))
            .await?
            .text()
            .await?;
        log::info!("[{}] got address {}", log_id, addr);
        Ok(addr)
    }

    /// Returns the balance postfix numbers.
    async fn balance_postfix(&mut self, account: String) -> Result<u128> {
        let log_id = account.clone();
        self.client
            .goto(
                // TODO doesn't work with differen URI!
                "https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts",
            )
            .await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(3));

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
        log::info!("[{}] found balance {} for account {}", log_id, txt, account);
        Ok(txt.parse::<u128>().expect("failed parsing"))
    }

    /// Uploads the contract behind `contract_path`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_upload(&mut self, upload_input: Upload) -> Result<String> {
        let log_id = format!(
            "{}",
            upload_input
                .contract_path
                .file_name()
                .expect("file name must exist")
                .to_str()
                .expect("conversion must work")
        );
        log::info!("[{}] opening url for upload: {:?}", log_id, url());
        self.client.goto(&url()).await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        log::info!("[{}] click settings", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']/*[1]"))
            .await?
            .click()
            .await?;

        // if we can already select the 'Local Node' we will.
        let maybe_local_node = self
            .client
            .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await;
        if maybe_local_node.is_err() {
            log::info!("[{}] click development", log_id);
            self.client
                .find(Locator::XPath("//*[contains(text(),'Development')]"))
                .await?
                .click()
                .await?;

            log::info!("[{}] select local node", log_id);
            self.client
                .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
                .await?
                .click()
                .await?;

            log::info!("[{}] click switch", log_id);
            self.client
                .find(Locator::XPath("//button[contains(text(),'Switch')]"))
                .await?
                .click()
                .await?;
        } else {
            log::info!("[{}] close settings", log_id);
            self.client
                .wait_for_find(Locator::XPath(
                    "//div[contains(@class, 'ui--Sidebar')]/*[2]",
                ))
                .await?
                .click()
                .await?;
        }

        log::info!("[{}] waiting for local node page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        log::info!("[{}] opening url for upload: {:?}", log_id, url());
        self.client.goto(&url()).await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(1));

        log::info!("[{}] click upload", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//button[contains(text(),'Upload & deploy code')]",
            ))
            .await?
            .click()
            .await?;

        log::info!("[{}] injecting jquery", log_id);
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

        log::info!("[{}] waiting for jquery", log_id);
        self.client
            .wait_for_find(Locator::Css("#jquery-ready"))
            .await?;

        log::info!("[{}] uploading {:?}", log_id, upload_input.contract_path);
        let mut upload = self
            .client
            .find(Locator::XPath("//input[@type = 'file']"))
            .await?;
        upload
            .send_keys(&upload_input.contract_path.display().to_string())
            .await?;
        self.client
            .execute("$(\"input[type = 'file']\").trigger('change')", Vec::new())
            .await?;

        if let Some(caller) = &upload_input.caller {
            let caller = caller.to_lowercase();
            // open listbox for accounts
            log::info!("[{}] click listbox for accounts", log_id);
            self.client
                .wait_for_find(Locator::XPath(
                    "//*[contains(text(),'deployment account')]/ancestor::div[1]",
                ))
                .await?
                .click()
                .await?;

            std::thread::sleep(std::time::Duration::from_secs(1));

            // choose caller
            log::info!("[{}] choose {:?}", log_id, caller);
            let path = format!("//div[@name = '{}']", caller);
            self.client
                .find(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        log::info!(
            "[{}] wait for upload of {:?} to be finished",
            log_id,
            upload_input.contract_path
        );
        self.client
            .wait_for_find(Locator::XPath(
                "//label[contains(text(), 'code bundle name')]",
            ))
            .await?;

        log::info!(
            "[{}] click next on {:?}",
            log_id,
            upload_input.contract_path
        );
        self.client
            .find(Locator::XPath(
                "//div[@class = 'actions']//button[contains(text(), 'Next')]",
            ))
            .await?
            .click()
            .await?;

        if let Some(constructor) = &upload_input.constructor {
            log::info!("[{}] click constructor list box", log_id);
            self.client
                .wait_for_find(Locator::XPath(
                    "//*[contains(text(),'deployment constructor')]/ancestor::div[1]//*/div[@role='listbox']"
                ))
                .await?.click().await?;

            log::info!("[{}] click constructor option {}", log_id, constructor);
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

        for (key, value) in upload_input.initial_values.iter() {
            // if the value is `Yes` or `No` we assume it's a listbox with a boolean
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
                let path = format!("//label/*[contains(text(),'initValue')]/ancestor::div[1]//*/div[@role = 'option']/span[text() = '{}']", value);
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .click()
                    .await?;
            } else {
                log::info!(
                    "[{}] inserting '{}' into input field '{}'",
                    log_id,
                    value,
                    key
                );
                let path =
                    format!("//*[contains(text(),'{}')]/ancestor::div[1]//*/input", key);
                let mut input = self.client.find(Locator::XPath(&path)).await?;
                // we need to clear a possible default input from the field
                input.clear().await?;
                input.send_keys(&value).await?;
            }
        }

        for (key, value) in upload_input.items.iter() {
            log::info!("[{}] adding item '{}' for '{}'", log_id, value, key);
            let add_item = format!("//div[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//div[contains(text(),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&value).await?;
        }

        log::info!("[{}] set endowment to {}", log_id, upload_input.endowment);
        let mut input = self
            .client
            .find(Locator::XPath(
                "//div/*[contains(text(),'endowment')]/ancestor::div[1]//*/input",
            ))
            .await?;
        input.clear().await?;
        input.send_keys(&upload_input.endowment).await?;

        log::info!("[{}] click endowment list box", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div/*[contains(text(),'endowment')]/ancestor::div[1]//*/div[@role='listbox']"))
            .await?;

        log::info!(
            "[{}] click endowment unit option {}",
            log_id,
            upload_input.endowment_unit
        );
        let path = format!(
            "//div[@role='option']/span[contains(text(),'{}')]",
            upload_input.endowment_unit
        );
        self.client.wait_for_find(Locator::XPath(&path)).await?;

        log::info!("[{}] click deploy", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Deploy')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] click sign and submit", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//button[contains(text(),'Sign and Submit')]",
            ))
            .await?
            .click()
            .await?;

        log::info!(
            "[{}] upload: waiting for either success or failure notification {:?}",
            log_id,
            upload_input.contract_path
        );

        let mut res;
        for waited in 0..21 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            res = self.client.find(
                Locator::XPath("//div[contains(@class, 'ui--Status')]//*/div[@class = 'status' and not(contains(text(),'ready') or contains(text(),'usurped'))]")
            ).await;
            if res.is_ok() {
                log::info!(
                    "[{}] upload: status contains {:?}",
                    log_id,
                    self.client
                        .find(Locator::XPath("//div[contains(@class, 'ui--Status')]"))
                        .await?
                        .text()
                        .await?
                );
                log::info!(
                    "[{}] upload: found status {:?}",
                    log_id,
                    res.expect("res must exist here").text().await?
                );
                log::info!(
                    "[{}] upload: success for {:?} after waiting {}",
                    log_id,
                    upload_input.contract_path,
                    waited,
                );
                break
            } else {
                log::info!(
                    "[{}] upload retry: failed for {:?} after waiting {}",
                    log_id,
                    upload_input.contract_path,
                    waited,
                );

                let statuses = self
                    .client
                    .find_all(Locator::XPath(
                        "//div[contains(@class, 'ui--Status')]//div[@class = 'desc' or @class = 'header']",
                    ))
                    .await?;
                log::info!(
                    "[{}] upload retry: found {:?} status messages for {:?}",
                    log_id,
                    statuses.len(),
                    upload_input.contract_path
                );
                for mut el in statuses {
                    log::info!("[{}] upload retry, text: {:?}", log_id, el.text().await?);
                }

                if waited == 20 {
                    log::info!(
                        "[{}] timed out on waiting for {:?} upload! next recursion.",
                        log_id,
                        upload_input.contract_path
                    );
                    return self.execute_upload(upload_input.clone()).await
                } else {
                    log::info!(
                        "[{}] timed out on waiting for {:?} upload! sleeping.",
                        log_id,
                        upload_input.contract_path
                    );
                }
            }
        }

        log::info!(
            "[{}] upload: extracting status messages {:?}",
            log_id,
            upload_input.contract_path
        );
        let statuses = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']",
            ))
            .await?;
        log::info!(
            "[{}] upload: found {:?} status messages {:?}",
            log_id,
            statuses.len(),
            upload_input.contract_path
        );
        let mut statuses_processed = Vec::new();
        for mut el in statuses {
            // the switch of status vs. header is intentional here
            el.find(Locator::XPath("div[@class = 'header']"))
                .await?
                .text()
                .await?
                .split("\n")
                .for_each(|status| {
                    log::info!(
                        "[{}] found status message {:?} {:?}",
                        log_id,
                        status,
                        upload_input.contract_path
                    );
                    statuses_processed.push(Event {
                        header: String::from(""),
                        status: status.to_string(),
                    });
                });
        }
        let events = Events::new(statuses_processed);
        if events.contains("Priority is too low") {
            log::info!(
                "[{}] found priority too low during upload of {:?}! trying again!",
                log_id,
                upload_input.contract_path
            );
            return self.execute_upload(upload_input.clone()).await
        } else if events.contains("usurped") {
            log::info!(
                "[{}] found usurped for upload of {:?}! trying again!",
                log_id,
                upload_input.contract_path
            );
            return self.execute_upload(upload_input.clone()).await
        } else {
            log::info!(
                "[{}] did not find priority too low in {:?} status messages {:?}",
                log_id,
                events.events.len(),
                upload_input.contract_path
            );
        }
        assert!(
            events.contains("system.ExtrinsicSuccess"),
            "upload must have succeeded, but events contain only {:?}",
            events.events
        );

        log::info!("[{}] click dismiss", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss all notifications')]",
            ))
            .await?
            .click()
            .await?;

        // wait for disappearance animation to finish instead
        // otherwise the notifications might occlude buttons
        log::info!("[{}] wait for animation to finish", log_id);
        self.client
            .execute("$('.ui--Status').hide()", Vec::new())
            .await?;

        log::info!(
            "[{}] click on recently added contract in list (the last one)",
            log_id
        );
        self.client
            .find(Locator::XPath(
                "(//div[contains(@class, 'ui--AccountName')])[last()]",
            ))
            .await?
            .click()
            .await?;

        log::info!("[{}] getting contract address", log_id);
        let addr = self
            .client
            .find(Locator::XPath("//div[@class = 'ui--AddressMenu-addr']"))
            .await?
            .text()
            .await?;
        log::info!("[{}] contract address {:?}", log_id, addr);

        log::info!("[{}] close sidebar", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//div[contains(@class, 'ui--Sidebar')]/button",
            ))
            .await?
            .click()
            .await?;

        Ok(String::from(addr))
    }

    /// Executes the RPC call `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_rpc(&mut self, call: Call) -> Result<String> {
        let log_id = call.method.clone();

        let url = format!("{}", url());
        log::info!(
            "[{}] opening url for rpc {:?}: {:?}",
            log_id,
            call.method,
            url
        );
        self.client.goto(url.as_str()).await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        // iterate through the list and see which of the entries has the correct address
        let contracts_in_list = self
            .client
            .find_all(Locator::XPath("//div[contains(@class, 'ui--AccountName')]"))
            .await?
            .len();
        log::info!("[{}] found {} contracts in list", log_id, contracts_in_list);
        assert!(
            contracts_in_list > 0,
            "there must be more than zero contracts in the list!"
        );

        let mut contract_index = None;
        for index in (0..contracts_in_list + 1).rev() {
            log::info!("[{}] checking contract {:?}", log_id, index);
            self.client
                .find(Locator::XPath(&format!(
                    "(//div[contains(@class, 'ui--AccountName')])[{}]",
                    index
                )))
                .await?
                .click()
                .await?;

            log::info!("[{}] getting contract address", log_id);
            let addr = self
                .client
                .find(Locator::XPath("//div[@class = 'ui--AddressMenu-addr']"))
                .await?
                .text()
                .await?;
            log::info!("[{}] contract address {}", log_id, addr);

            log::info!(
                "[{}] comparing {} == {}",
                log_id,
                addr,
                call.contract_address
            );
            if addr == call.contract_address {
                log::info!("[{}] found contract address at index {:?}", log_id, index);
                contract_index = Some(index);
                break
            }
        }

        let index = contract_index.expect("index must exist");
        log::info!("[{}] close sidebar", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//div[contains(@class, 'ui--Sidebar')]/button",
            ))
            .await?
            .click()
            .await?;

        log::info!("[{}] opening detail view for contract {:?}", log_id, index);
        self.client
            .find(Locator::XPath(&format!(
                "(//div[contains(@class, 'ui--Messages')])[{}]",
                index
            )))
            .await?
            .click()
            .await?;

        // assert that only one expanded method view exists
        let expanded_views = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Expander-content')]",
            ))
            .await?
            .len();
        assert!(
            expanded_views == 1,
            "found too many expanded views ({})!",
            expanded_views
        );

        // click `method`
        log::info!("[{}] try to find result for {:?}", log_id, call.method);
        let path = format!("//span[@class = 'ui--MessageSignature-name' and (text() = '{}' or text() = '{}')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]/div[contains(@class, 'result')]//div[@class = 'ui--Param-text ']", call.method, call.method.to_lowercase());
        let possibly_rpc_result = self.client.find(Locator::XPath(&path)).await;

        // if the rpc can be executed without params (e.g. `get(&self)`)
        // the result is already shown
        if possibly_rpc_result.is_ok() {
            let result = possibly_rpc_result?.text().await?;
            log::info!(
                "[{}] found result for {:?}: {:?}",
                log_id,
                call.method,
                result
            );
            return Ok(result)
        }

        // otherwise we have to execute the rpc and set the params
        log::info!("[{}] open rpc param details", log_id);
        let path = format!("//span[@class = 'ui--MessageSignature-name' and (text() = '{}' or text() = '{}')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]/button", call.method, call.method.to_lowercase());
        self.client
            .wait_for_find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // possibly set values
        for (key, mut value) in call.values {
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

        // click call
        log::info!("[{}] click read", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Read')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] wait for outcome to appear", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[contains(text(),'Call results')]/ancestor::div[1]/ancestor::div[1]/div[@class = 'ui--Expander-content']"))
            .await?;

        log::info!("[{}] read outcome", log_id);
        let mut el = self
            .client
            .wait_for_find(Locator::XPath(
                "(//div[contains(@class, 'ui--output')])[last()]/div",
            ))
            .await?;
        let txt = el.text().await?;
        log::info!("[{}] outcomes value {:?}", log_id, txt);
        Ok(txt)
    }

    /// Executes the transaction `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_transaction(&mut self, call: Call) -> TransactionResult<Events> {
        let log_id = call.method.clone();

        let url = format!("{}", url());
        log::info!(
            "[{}] opening url for executing transaction {:?}: {:?}",
            log_id,
            call.method,
            url
        );
        self.client.goto(url.as_str()).await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait_for_find(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        // iterate through the list and see which of the entries has the correct address
        let contracts_in_list = self
            .client
            .find_all(Locator::XPath("//div[contains(@class, 'ui--AccountName')]"))
            .await?
            .len();
        log::info!("[{}] found {} contracts in list", log_id, contracts_in_list);
        assert!(
            contracts_in_list > 0,
            "there must be more than zero contracts in the list!"
        );

        let mut contract_index = None;
        for index in (0..contracts_in_list + 1).rev() {
            log::info!("[{}] checking contract {:?}", log_id, index);
            self.client
                .find(Locator::XPath(&format!(
                    "(//div[contains(@class, 'ui--AccountName')])[{}]",
                    index
                )))
                .await?
                .click()
                .await?;

            log::info!("[{}] getting contract address", log_id);
            let addr = self
                .client
                .find(Locator::XPath("//div[@class = 'ui--AddressMenu-addr']"))
                .await?
                .text()
                .await?;
            log::info!("[{}] contract address {}", log_id, addr);

            log::info!(
                "[{}] comparing {} == {}",
                log_id,
                addr,
                call.contract_address
            );
            if addr == call.contract_address {
                log::info!("[{}] found contract address at index {:?}", log_id, index);
                contract_index = Some(index);
                break
            }
        }

        let index = contract_index.expect("index must exist");
        log::info!("[{}] close sidebar", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//div[contains(@class, 'ui--Sidebar')]/button",
            ))
            .await?
            .click()
            .await?;

        log::info!("[{}] opening detail view for contract {:?}", log_id, index);
        self.client
            .find(Locator::XPath(&format!(
                "(//div[contains(@class, 'ui--Messages')])[{}]",
                index
            )))
            .await?
            .click()
            .await?;

        // assert that only one expanded method view exists
        let expanded_views = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Expander-content')]",
            ))
            .await?
            .len();
        assert!(
            expanded_views == 1,
            "found too many expanded views ({})!",
            expanded_views
        );

        log::info!("[{}] open exec details", log_id);
        let path = format!("//span[@class = 'ui--MessageSignature-name' and (text() = '{}' or text() = '{}')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]/button", call.method, call.method.to_lowercase());
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        log::info!("[{}] waiting for exec details to appear", log_id);
        self.client
            .wait_for_find(Locator::XPath("//h1[text() = 'Call a contract']"))
            .await?;

        if let Some(caller) = &call.caller {
            // open listbox for accounts
            log::info!("[{}] click listbox for accounts", log_id);
            self.client
                .wait_for_find(Locator::XPath(
                    "//*[contains(text(),'call from account')]/ancestor::div[1]/div",
                ))
                .await?
                .click()
                .await?;

            // enter the caller
            log::info!("[{}] entering {:?} into listbox", log_id, caller);
            let path = format!(
                "//*[contains(text(),'call from account')]/ancestor::div[1]//input"
            );
            self.client
                .find(Locator::XPath(&path))
                .await?
                .clear()
                .await?;
            self.client
                .find(Locator::XPath(&path))
                .await?
                .send_keys(&format!("{}\n", caller))
                .await?;
        }

        // Possibly add payment
        if let Some(payment) = &call.payment {
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
                .wait_for_find(Locator::XPath(&path))
                .await?
                .click()
                .await?;

            log::info!("[{}] entering payment {:?}", log_id, payment.payment);
            let path = "//*[contains(text(),'value')]/ancestor::div[1]/div//input[@type = 'text']";
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
            // the box shows only up if the rpc can determine the gas costs (not if the rpc
            // e.g. results in `ContractTrapped`).
            log::info!("[{}] possibly unset 'use estimated gas' checkbox", log_id);
            let max_gas_input_path = "//*[contains(text(),'max gas allowed')]/ancestor::div[1]/div//input[@type = 'text']";
            let mut max_gas_input = self
                .client
                .wait_for_find(Locator::XPath(max_gas_input_path))
                .await?;

            std::thread::sleep(std::time::Duration::from_secs(1));

            let path = "//*[contains(text(),'use estimated gas')]/ancestor::div[1]/div";
            let possibly_estimated_gas = self.client.find(Locator::XPath(path)).await;
            if let Ok(el) = possibly_estimated_gas {
                log::info!("[{}] unsetting 'use estimated gas' checkbox", log_id);
                el.click().await?;
            } else {
                log::info!("[{}] no 'use estimated gas' checkbox found", log_id);
            }

            log::info!("[{}] entering max gas {:?}", log_id, max_gas);
            max_gas_input.clear().await?;
            max_gas_input.send_keys(&max_gas).await?;
        }

        // possibly set values
        for (key, mut value) in call.values.clone() {
            log::info!("[{}] entering {:?} into {:?}", log_id, &value, &key);
            let path = format!(
                "//div[contains(@class, 'ui--Params')]//*[contains(text(),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
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

        std::thread::sleep(std::time::Duration::from_secs(2));

        log::info!("[{}] click execute", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Execute')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] click sign and submit", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//button[contains(text(),'Sign and Submit')]",
            ))
            .await?
            .click()
            .await?;

        log::info!(
            "[{}] transaction: waiting for either success or failure notification",
            log_id
        );
        let mut res;
        for waited in 0..21 {
            std::thread::sleep(std::time::Duration::from_secs(1));
            res = self.client.find(
                Locator::XPath("//div[contains(@class, 'ui--Status')]//*/div[@class = 'status' and not(contains(text(),'ready') or contains(text(),'usurped'))]")
            ).await;
            if res.is_ok() {
                log::info!(
                    "[{}] transaction: success for {:?} after waiting {}",
                    log_id,
                    call.method,
                    waited,
                );
                break
            } else {
                log::info!(
                    "[{}] transaction: waited for {:?} for {}",
                    log_id,
                    call.method,
                    waited,
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
                    log::info!(
                        "[{}] transaction retry, text: {:?}",
                        log_id,
                        el.text().await?
                    );
                }

                if waited == 20 {
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

        log::info!("[{}] transaction: extracting status messages", log_id);
        let statuses = self
            .client
            .find_all(Locator::XPath(
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']",
            ))
            .await?;
        log::info!(
            "[{}] transaction: found {:?} status messages",
            log_id,
            statuses.len()
        );
        let mut statuses_processed = Vec::new();
        for mut el in statuses {
            el.find(Locator::XPath("div[@class = 'status']"))
                .await?
                .text()
                .await?
                .split("\n")
                .for_each(|status| {
                    log::info!("[{}] found status message {:?}", log_id, status);
                    statuses_processed.push(Event {
                        header: String::from(""),
                        status: status.to_string(),
                    });
                });
            el.find(Locator::XPath("div[@class = 'header']"))
                .await?
                .text()
                .await?
                .split("\n")
                .for_each(|status| {
                    log::info!("[{}] found status message {:?}", log_id, status);
                    statuses_processed.push(Event {
                        header: String::from(""),
                        status: status.to_string(),
                    });
                });
        }
        let events = Events::new(statuses_processed);

        log::info!("[{}] click dismiss", log_id);
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss all notifications')]",
            ))
            .await?
            .click()
            .await?;

        if events.contains("Priority is too low") {
            log::info!(
                "[{}] found priority too low during transaction execution of {:?}! trying again!",
                log_id,
                call.method
            );
            return self.execute_transaction(call.clone()).await
        } else if events.contains("usurped") {
            log::info!(
                "[{}] found usurped for transaction of {:?}! trying again!",
                log_id,
                call.method
            );
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
            (true, false) => Ok(events),
            (false, true) => Err(TransactionError::ExtrinsicFailed(events)),
            (false, false) => panic!("ERROR: Neither 'ExtrinsicSuccess' nor 'ExtrinsicFailed' was found in status messages!"),
            (true, true) => panic!("ERROR: Both 'ExtrinsicSuccess' nor 'ExtrinsicFailed' was found in status messages!"),
        }
    }
}
/// Returns the URL to the `path` in the UI.
///
/// Defaults to https://paritytech.github.io/canvas-ui as the base URL.
fn url() -> String {
    let base_url: String =
        std::env::var("UI_URL").unwrap_or(String::from("https://polkadot.js.org"));

    // strip a possibly ending `/` from he URL, since a URL like `http://foo//bar`
    // can cause issues.
    let base_url = base_url.trim_end_matches('/');

    String::from(format!("{}{}", base_url, "/apps/#/contracts"))
}
