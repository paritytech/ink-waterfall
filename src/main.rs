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
    Client,
    ClientBuilder,
    Locator,
};
use serde_json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to webdriver instance that is listening on port 4444
    let mut caps = serde_json::map::Map::new();
    let opts = serde_json::json!({ "args": ["--headless"] });
    caps.insert("moz:firefoxOptions".to_string(), opts.clone());
    let mut client = ClientBuilder::native()
        .capabilities(caps)
        .connect("http://localhost:4444")
        .await?;

    client
        .goto("https://paritytech.github.io/canvas-ui/#/upload")
        .await?;

    client
        .wait_for_find(Locator::Css(".actions button"))
        .await?
        .click()
        .await?;
    client
        .find(Locator::Css(".app--SideBar-settings"))
        .await?
        .click()
        .await?;

    client
        .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
        .await?
        .click()
        .await?;

    sleep(Duration::from_millis(2000)).await;
    client
        .find(Locator::XPath(
            "//*[contains(text(),'Upload & Instantiate Contract')]",
        ))
        .await?
        .click()
        .await?;
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
    client.execute(&*inject, Vec::new()).await?;

    client.wait_for_find(Locator::Css("#jquery-ready")).await?;

    client
        .execute("$('[role=combobox]').click()", Vec::new())
        .await?;
    client
        .execute("$('[name=alice]').click()", Vec::new())
        .await?;

    let mut upload = client.find(Locator::Css(".ui--InputFile input")).await?;
    upload.send_keys("/builds/parity/ink-waterfall/examples/flipper/target/ink/flipper.contract").await?;
    client
        .execute("$(\".ui--InputFile input\").trigger('change')", Vec::new())
        .await?;
    client
        .execute(
            "$(\":contains('Constructor Details')\").click()",
            Vec::new(),
        )
        .await?;
    client
        .execute("$(\"button:contains('Instantiate')\").click()", Vec::new())
        .await?;
    client
        .execute(
            "$(\"button:contains('Sign & Submit')\").click()",
            Vec::new(),
        )
        .await?;

    // h1: Contract successfully instantiated
    client
        .wait_for_find(Locator::XPath(
            "//*[contains(text(),'Contract successfully instantiated')]",
        ))
        .await?;

    client
        .wait_for_find(Locator::XPath(
            "//*[contains(text(),'Dismiss all notifications')]",
        ))
        .await?
        .click()
        .await?;

    client
        .find(Locator::XPath("//*[contains(text(),'Execute Contract')]"))
        .await?
        .click()
        .await?;

    // open listbox for methods
    client
        .find(Locator::XPath(
            "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
        ))
        .await?
        .click()
        .await?;

    // click get
    client.find(Locator::XPath("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'get')]")).await?.click().await?;

    // click call
    client
        .find(Locator::XPath("//button[contains(text(),'Call')]"))
        .await?
        .click()
        .await?;

    // must contain false
    client.wait_for_find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']//*[text() = 'false']")).await?;

    // clear all
    client
        .find(Locator::XPath("//*[text() = 'Clear all']"))
        .await?
        .click()
        .await?;

    // open listbox for methods
    client
        .find(Locator::XPath(
            "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
        ))
        .await?
        .click()
        .await?;

    // click flip
    client.find(Locator::XPath("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'flip')]")).await?.click().await?;

    // click call
    client
        .find(Locator::XPath("//button[contains(text(),'Call')]"))
        .await?
        .click()
        .await?;

    // wait for notification to show up
    client
        .wait_for_find(Locator::XPath(
            "//div[@class = 'status' and contains(text(), 'queued')]",
        ))
        .await?;

    // click sign and submit
    eprintln!("sign and submit");
    client
        .find(Locator::XPath("//button[contains(text(),'Sign & Submit')]"))
        .await?
        .click()
        .await?;

    // maybe assert?
    eprintln!("waiting for success notification");
    client.wait_for_find(Locator::XPath("//div[@class = 'status']/ancestor::div/div[@class = 'header' and contains(text(), 'ExtrinsicSuccess')]")).await?;
    client
        .wait_for_find(Locator::XPath(
            "//*[contains(text(),'Dismiss all notifications')]",
        ))
        .await?
        .click()
        .await?;

    // open listbox for methods
    client
        .find(Locator::XPath(
            "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
        ))
        .await?
        .click()
        .await?;

    // click get
    eprintln!("click get");
    client.find(Locator::XPath("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'get')]")).await?.click().await?;

    // click call
    eprintln!("click call");
    client
        .find(Locator::XPath("//button[contains(text(),'Call')]"))
        .await?
        .click()
        .await?;

    // must contain true
    eprintln!("wait for true");
    client.wait_for_find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']//*[text() = 'true']")).await?;

    // Then close the browser window.
    client.close().await?;

    Ok(())
}
