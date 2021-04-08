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
    ClientBuilder,
    Locator,
};
use serde_json;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn works() -> Result<(), Box<dyn std::error::Error>> {
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

    eprintln!("click action button");
    client
        .wait_for_find(Locator::Css(".actions button"))
        .await?
        .click()
        .await?;

    eprintln!("click settings");
    client
        .find(Locator::Css(".app--SideBar-settings"))
        .await?
        .click()
        .await?;

    eprintln!("click local node");
    client
        .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
        .await?
        .click()
        .await?;

    // TODO
    sleep(Duration::from_millis(2000)).await;

    eprintln!("click upload");
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

    eprintln!("click combobox");
    client
        .execute("$('[role=combobox]').click()", Vec::new())
        .await?;

    eprintln!("click alice");
    client
        .execute("$('[name=alice]').click()", Vec::new())
        .await?;

    let mut upload = client.find(Locator::Css(".ui--InputFile input")).await?;
    upload
        .send_keys("/ci-cache/ink-waterfall/targets/master/run/ink/flipper.contract")
        .await?;
    client
        .execute("$(\".ui--InputFile input\").trigger('change')", Vec::new())
        .await?;
    eprintln!("click details");
    client
        .execute(
            "$(\":contains('Constructor Details')\").click()",
            Vec::new(),
        )
        .await?;
    eprintln!("click instantiate");
    client
        .execute("$(\"button:contains('Instantiate')\").click()", Vec::new())
        .await?;
    eprintln!("click sign and submit");
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

    eprintln!("click dismiss");
    client
        .wait_for_find(Locator::XPath(
            "//*[contains(text(),'Dismiss all notifications')]",
        ))
        .await?
        .click()
        .await?;

    // wait for disappearance animation to finish instead
    // otherwise the notifications might occlude buttons
    eprintln!("wait for animation to finish");
    client
        .execute("$('.ui--Status').hide()", Vec::new())
        .await?;

    eprintln!("click execute");
    client
        .find(Locator::XPath(
            "//button[contains(text(),'Execute Contract')]",
        ))
        .await?
        .click()
        .await?;

    // open listbox for methods
    eprintln!("click listbox");
    client
        .find(Locator::XPath(
            "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
        ))
        .await?
        .click()
        .await?;

    // click get
    eprintln!("choose get");
    client.find(Locator::XPath("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'get')]")).await?.click().await?;

    // click call
    eprintln!("click call");
    client
        .find(Locator::XPath("//button[contains(text(),'Call')]"))
        .await?
        .click()
        .await?;

    // must contain false
    client.wait_for_find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']//*[text() = 'false']")).await?;

    // clear all
    eprintln!("click clear all");
    client
        .find(Locator::XPath("//*[text() = 'Clear all']"))
        .await?
        .click()
        .await?;

    // open listbox for methods
    eprintln!("open listbox");
    client
        .find(Locator::XPath(
            "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
        ))
        .await?
        .click()
        .await?;

    // click flip
    eprintln!("click flip");
    client.find(Locator::XPath("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'flip')]")).await?.click().await?;

    // click call
    eprintln!("click call");
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
