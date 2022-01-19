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
use fantoccini::{
    error,
    Client,
    Locator,
};
use std::path::PathBuf;

#[async_trait]
impl ContractsUi for crate::uis::Ui {
    /// Returns the balance postfix numbers.
    async fn balance_postfix(&mut self, account: String) -> Result<u128> {
        let log_id = format!("{} {}", test_name(), account.clone());
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

        std::thread::sleep(std::time::Duration::from_secs(6));

        let path = format!(
            "//div[. = '{}']/ancestor::tr//span[@class = 'ui--FormatBalance-postfix']",
            account
        );
        let txt = self
            .client
            .wait()
            .for_element(Locator::XPath(&path))
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
        let log_id = format!("{}", test_name(),);
        log::info!(
            "[{}] opening url for upload of {}: {:?}",
            log_id,
            upload_input
                .contract_path
                .file_name()
                .expect("file name must exist")
                .to_str()
                .expect("conversion must work"),
            url()
        );
        self.client.goto(&url()).await?;

        // Firefox might not load if the website at that address is already open due to e.g.
        // a prior `execute_transaction` call in the test. Hence we refresh just to be sure
        // that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        log::info!("[{}] click upload", log_id);
        self.click(Locator::XPath(
            "//button[contains(text(),'Upload & deploy code')]",
        ))
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
                .wait()
                .for_element(Locator::XPath(
                    "//*[contains(text(),'deployment account')]/ancestor::div[1]",
                ))
                .await?
                .click()
                .await?;

            std::thread::sleep(std::time::Duration::from_secs(3));

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
            .wait()
            .for_element(Locator::XPath(
                "//label[contains(text(), 'code bundle name')]",
            ))
            .await?;

        log::info!(
            "[{}] click next on {:?}",
            log_id,
            upload_input.contract_path
        );
        self.client
            .find(Locator::XPath("//button[contains(text(), 'Next')]"))
            .await?
            .click()
            .await?;

        if let Some(constructor) = &upload_input.constructor {
            log::info!("[{}] click constructor list box", log_id);
            self.client
                .wait().for_element(Locator::XPath(
                    "//*[contains(text(),'deployment constructor')]/ancestor::div[1]//*/div[@role='listbox']"
                ))
                .await?.click().await?;

            log::info!("[{}] click constructor option {}", log_id, constructor);
            let path = format!(
                "//span[@class = 'ui--MessageSignature-name' and contains(normalize-space(text()),'{}')]",
                constructor
            );
            self.client
                .wait()
                .for_element(Locator::XPath(&path))
                .await?
                .click()
                .await?;
        }

        for (key, value) in upload_input.initial_values.iter() {
            // if the value is `Yes` or `No` we assume it's a listbox with a boolean
            let mut value = transform_value(&value);
            if value == "Yes" || value == "No" {
                log::info!("[{}] opening dropdown list '{}'", log_id, key);
                let path = format!(
                    "//label/*[contains(normalize-space(text()),'{}')]/ancestor::div[1]",
                    key
                );
                self.client
                    .find(Locator::XPath(&path))
                    .await?
                    .click()
                    .await?;

                log::info!("[{}] chossing option '{}''", log_id, value);
                let path = format!("//label/*[contains(normalize-space(text()),'{}')]/ancestor::div[1]//*/div[@role = 'option']/span[text() = '{}']", key, value);
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
                    format!("//*[contains(normalize-space(text()),'{}')]/ancestor::div[1]//*/input", key);
                let mut input = self.client.find(Locator::XPath(&path)).await?;
                // we need to clear a possible default input from the field
                input.clear().await?;
                value.push('\n');
                input.send_keys(&value).await?;
            }
        }

        for (key, value) in upload_input.items.iter() {
            log::info!("[{}] adding item '{}' for '{}'", log_id, value, key);
            let add_item = format!("//div[contains(normalize-space(text()),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//div[contains(normalize-space(text()),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&format!("{}\n", value)).await?;
        }

        log::info!("[{}] click deploy", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Deploy')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] click sign and submit", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
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
            std::thread::sleep(std::time::Duration::from_secs(3));
            res = self.client.find(
                Locator::XPath("//div[contains(@class, 'ui--Status')]//*/div[@class = 'status' and not(contains(text(),'ready') or contains(text(),'usurped') or contains(text(),'outdated'))]")
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
                if waited == 20 {
                    log::info!(
                        "[{}] timed out on waiting for {:?} upload! next recursion.",
                        log_id,
                        upload_input.contract_path
                    );
                    return self.execute_upload(upload_input.clone()).await
                } else {
                    log::info!(
                        "[{}] timed out on waiting for {:?} upload after {}! sleeping.",
                        log_id,
                        upload_input.contract_path,
                        waited
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
                "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']//div[@class = 'header']//div",
            ))
            .await?;
        let mut statuses_processed = Vec::new();
        for mut el in statuses {
            // the switch of status vs. header is intentional here
            let txt = el.html(true).await?.to_string().replace("\"", "");
            statuses_processed.push(Event {
                // TODO remove `header` as a field altogether
                header: String::from(""),
                status: txt,
            });
        }
        for status in &statuses_processed {
            log::info!("[{}] upload: found status {:?}", log_id, status,);
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
        } else if events.contains("outdated") {
            log::info!(
                "[{}] found outdated for upload of {:?}! trying again!",
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
            .wait()
            .for_element(Locator::XPath(
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
            .wait()
            .for_element(Locator::XPath(
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
        let log_id = format!("{} {}", test_name(), call.method.clone());

        let url = url();
        log::info!(
            "[{}] opening url for rpc {:?}: {:?}",
            log_id,
            call.method,
            url
        );
        self.client.goto(&url).await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//div[@class = 'menuSection']"))
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
            .wait()
            .for_element(Locator::XPath(
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
            .wait()
            .for_element(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // possibly set values
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
            let add_item = format!("//div[contains(normalize-space(text()),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//div[contains(normalize-space(text()),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&format!("{}\n", &value)).await?;
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
            .wait().for_element(Locator::XPath("//div[contains(text(),'Call results')]/ancestor::div[1]/ancestor::div[1]/div[@class = 'ui--Expander-content']"))
            .await?;

        log::info!("[{}] read outcome", log_id);
        let mut ret_value = self
            .client
            .wait()
            .for_element(Locator::XPath(
                "(//div[contains(@class, 'ui--output')])[last()]/div",
            ))
            .await?
            .text()
            .await?;

        log::info!("[{}] read outcome type", log_id);
        let ret_type = self
            .client
            .wait()
            .for_element(Locator::XPath(
                "(//span[@class = 'ui--MessageSignature-returnType'])[last()]",
            ))
            .await?
            .text()
            .await?;

        if ret_type.contains("AccountId")
            && ret_value != "<none>"
            && ret_value != "<empty>"
        {
            // convert hash account id to mnemonic name
            log::info!("[{}] attempting to resolve {}", log_id, &ret_value);
            ret_value = name_to_address(&ret_value).expect("address for name must exist");
            log::info!("[{}] resolved to {}", log_id, &ret_value);
        }

        if ret_value == "<none>" {
            ret_value = "None".to_string();
        }

        log::info!("[{}] outcome value is {:?}", log_id, ret_value);
        log::info!("[{}] outcome type value is {:?}", log_id, ret_type);
        Ok(ret_value)
    }

    /// Executes the transaction `call`.
    ///
    /// # Developer Note
    ///
    /// This method must not make any assumptions about the state of the Ui before
    /// the method is invoked. It must e.g. open the upload page right at the start.
    async fn execute_transaction(&mut self, call: Call) -> TransactionResult<Events> {
        let log_id = format!("{} {}", test_name(), call.method.clone());

        let url = url();
        log::info!(
            "[{}] opening url for executing transaction {:?}: {:?}",
            log_id,
            call.method,
            url
        );
        self.client.goto(&url).await?;

        // Firefox might not load if the website at that address is already open, hence we refresh
        // just to be sure that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(6));

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
            .wait()
            .for_element(Locator::XPath(
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
        let path = format!("//span[@class = 'ui--MessageSignature-name' and (text() = '{}')]/ancestor::div[1]/ancestor::div[1]/ancestor::div[1]/button", call.method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        log::info!("[{}] waiting for exec details to appear", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//h1[text() = 'Call a contract']"))
            .await?;

        if let Some(caller) = &call.caller {
            // open listbox for accounts
            log::info!("[{}] click listbox for accounts", log_id);
            self.client
                .wait()
                .for_element(Locator::XPath(
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

        // possibly add payment
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
                .wait()
                .for_element(Locator::XPath(&path))
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
                .wait()
                .for_element(Locator::XPath(max_gas_input_path))
                .await?;

            std::thread::sleep(std::time::Duration::from_secs(3));

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
        for (key, value) in call.values.clone() {
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
                    "//div[contains(@class, 'ui--Params')]//*[contains(normalize-space(text()),'{}')]/ancestor::div[1]/div//input[@type = 'text']",
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

        std::thread::sleep(std::time::Duration::from_secs(2));

        // possibly add items
        for (key, value) in call.items.iter() {
            log::info!("[{}] adding item '{}' for '{}'", log_id, value, key);
            let add_item = format!("//div[contains(normalize-space(text()),'{}')]/ancestor::div[1]/ancestor::div[1]/*/button[contains(text(), 'Add item')]", key);
            self.client
                .find(Locator::XPath(&add_item))
                .await?
                .click()
                .await?;

            let last_item = format!("//div[contains(normalize-space(text()),'{}')]/ancestor::div[1]/ancestor::div[1]/*/div[@class = 'ui--Params-Content']/div[last()]//input", key);
            let mut input = self.client.find(Locator::XPath(&last_item)).await?;
            // we need to clear a possible default input from the field
            input.clear().await?;
            input.send_keys(&format!("{}\n", &value)).await?;
        }

        std::thread::sleep(std::time::Duration::from_secs(3));

        log::info!("[{}] get estimated gas", log_id);
        let max_gas_input_path = "//*[contains(text(),'max gas allowed')]/ancestor::div[1]/div//input[@type = 'text']";
        let max_gas_input = self
            .client
            .wait()
            .for_element(Locator::XPath(max_gas_input_path))
            .await?
            .attr("value")
            .await?;
        log::info!(
            "[{}] estimated gas for transaction is {}",
            log_id,
            max_gas_input.expect("estimated gas must exist")
        );

        log::info!("[{}] click execute", log_id);
        self.client
            .find(Locator::XPath("//button[contains(text(),'Execute')]"))
            .await?
            .click()
            .await?;

        log::info!("[{}] click sign and submit", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
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
            std::thread::sleep(std::time::Duration::from_secs(3));
            res = self.client.find(
                Locator::XPath("//div[contains(@class, 'ui--Status')]//*/div[contains(text(),'system.ExtrinsicSuccess') or contains(text(), 'system.ExtrinsicFailed')]")
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
                        "//div[contains(@class, 'ui--Status')]//div[@class = 'desc']//div[@class = 'header']//div",
                    ))
                    .await?;
                log::info!(
                    "[{}] transaction retry: found {:?} status messages for {:?}",
                    log_id,
                    statuses.len(),
                    call.method
                );
                for mut el in statuses {
                    let txt = el.html(true).await?.to_string().replace("\"", "");
                    log::info!("[{}] transaction retry, text: {:?}", log_id, txt,);
                }

                if waited == 20 {
                    log::info!(
                        "[{}] timed out on waiting for {:?} transaction after {}! next recursion.",
                        log_id,
                        call.method,
                        waited
                    );
                    return self.execute_transaction(call.clone()).await
                } else {
                    log::info!(
                        "[{}] timed out on waiting for {:?} transaction after {}! sleeping.",
                        log_id,
                        call.method,
                        waited
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
            let mut contents = el
                .find_all(Locator::XPath(
                    "//div[contains(@class, 'header') or contains(@class, 'status')]",
                ))
                .await?;
            for content in contents.iter_mut() {
                let status = content.html(true).await?;
                log::info!("[{}] found status message {:?}", log_id, status);
                statuses_processed.push(Event {
                    header: String::from(""),
                    status,
                });
            }
        }
        let events = Events::new(statuses_processed);

        log::info!("[{}] click dismiss", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
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
        } else if events.contains("outdated") {
            log::info!(
                "[{}] found outdated for transaction of {:?}! trying again!",
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

    /// Updates the metadata which the UI uses for interacting with the contract
    /// at `contract_addr` to `new_abi`.
    async fn update_metadata(
        &mut self,
        contract_addr: &String,
        new_abi: &PathBuf,
    ) -> Result<String> {
        let log_id = format!("{}", test_name(),);
        log::info!(
            "[{}] opening url for updating metadata of {}: {:?}",
            log_id,
            contract_addr,
            url()
        );
        self.client.goto(&url()).await?;

        // Firefox might not load if the website at that address is already open due to e.g.
        // a prior `execute_transaction` call in the test. Hence we refresh just to be sure
        // that it's a clean, freshly loaded page in front of us.
        self.client.refresh().await?;

        log::info!("[{}] waiting for page to become visible", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath("//div[@class = 'menuSection']"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(3));

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

        log::info!("[{}] click 'Add an existing contract'", log_id);
        self.client
            .wait()
            .for_element(Locator::XPath(
                "//button[contains(text(),'Add an existing contract')]",
            ))
            .await?
            .click()
            .await?;

        log::info!("[{}] entering contract address {:?}", log_id, contract_addr);
        let path = "//input[@data-testid = 'contract address']";
        self.client
            .find(Locator::XPath(path))
            .await?
            .clear()
            .await?;
        self.client
            .find(Locator::XPath(path))
            .await?
            .send_keys(&contract_addr)
            .await?;

        log::info!("[{}] uploading {:?}", log_id, new_abi);
        let mut upload = self
            .client
            .find(Locator::XPath("//input[@type = 'file']"))
            .await?;
        upload.send_keys(&new_abi.display().to_string()).await?;
        self.client
            .execute("$(\"input[type = 'file']\").trigger('change')", Vec::new())
            .await?;

        log::info!(
            "[{}] wait for upload of {:?} to be finished",
            log_id,
            new_abi
        );
        self.client
            .wait()
            .for_element(Locator::XPath("//div[contains(text(), 'Constructors (')]"))
            .await?;

        std::thread::sleep(std::time::Duration::from_secs(2));

        log::info!("[{}] click save on {:?}", log_id, new_abi);
        self.client
            .find(Locator::XPath("//button[contains(text(), 'Save')]"))
            .await?
            .click()
            .await?;

        Ok(String::from(""))
    }
}

impl crate::uis::Ui {
    /// Clicks on the `locator`, if an error occurs we retry ten times with a sleep
    /// of two seconds in between.
    ///
    /// This was introduced to retry on these spurious UI errors:
    ///
    /// ```json
    /// Standard(WebDriverError { error: ElementClickIntercepted,
    /// message: "Element <button class=\"ui--Button hasLabel Button-sc-l9wqp0-0 fUpXVx\">
    /// is not clickable at point (750,216) because another element
    /// <div class=\"ui--InputFile error InputFile-sc-vhlvx4-0 jqSBqi\"> obscures it",
    /// stack: "", delete_session: false })
    /// ```
    async fn click(
        &mut self,
        locator: Locator<'_>,
    ) -> std::result::Result<Client, error::CmdError> {
        let mut possibly_err =
            self.client.wait().for_element(locator).await?.click().await;

        const MAX_ATTEMPTS: usize = 10;
        let mut attempt = 0;
        while possibly_err.is_err() && attempt < MAX_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_secs(2));
            possibly_err = self.client.wait().for_element(locator).await?.click().await;
            attempt = attempt + 1;
        }

        possibly_err
    }
}

/// Returns the UI's base URL.
///
/// If the env variable `UI_URL` is set that one is taken, otherwise the default
/// `https://polkadot.js.org` is returned.
fn base_url() -> String {
    let base_url =
        std::env::var("UI_URL").unwrap_or(String::from("https://polkadot.js.org"));

    // strip a possibly ending `/` from he URL, since a URL like `http://foo//bar`
    // can cause issues.
    let mut url = base_url.trim_end_matches('/').to_string();
    url.push_str(&format!(
        "/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A{}#/",
        utils::node_port()
    ));
    url
}

/// Returns the URL to the `path` in the UI.
///
/// Defaults to https://paritytech.github.io/canvas-ui as the base URL.
fn url() -> String {
    format!("{}contracts", base_url())
}

/// Returns the address for a given `name`.
fn name_to_address(name: &str) -> Option<String> {
    match name {
        "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY" => Some("ALICE".to_string()),
        "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty" => Some("BOB".to_string()),
        "5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y" => Some("CHARLIE".to_string()),
        "5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy" => Some("DAVE".to_string()),
        "5HGjWAeFDfFCWPsjFQdVV2Msvz2XtMktvgocEZcCj68kUMaw" => Some("EVE".to_string()),
        _ => None,
    }
}

fn transform_value(value: &str) -> String {
    match value {
        "true" => String::from("Yes"),
        "false" => String::from("No"),
        _ => value.to_string(),
    }
}
