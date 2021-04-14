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

use proc_macro::TokenStream;
use quote::quote;

/// The macro is used to do some initial set-up for a waterfall test and handle
/// the shutdown at the end of a test.
///
/// # Usage
///
/// ```no_compile
/// #[waterfall_test]
/// async fn works(mut canvas_ui: CanvasUi) -> Result<()> {
///     let _contract_addr = canvas_ui.upload(contract_file).await?;
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn waterfall_test(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn =
        syn::parse2::<syn::ItemFn>(item.into()).expect("no item_fn can be parsed");
    let fn_name = &item_fn.sig.ident;
    let block = &item_fn.block;
    let fn_return_type = &item_fn.sig.output;
    let vis = &item_fn.vis;
    let attrs = &item_fn.attrs;
    let ret = match fn_return_type {
        syn::ReturnType::Default => quote! {},
        syn::ReturnType::Type(rarrow, ret_type) => quote! { #rarrow #ret_type },
    };
    let res = quote! {
        #( #attrs )*
        #[tokio::test]
        async #vis fn #fn_name () #ret {
            // hack to get a timeout for async tests running.
            // this is necessary so that the ci doesn't wait forever to fail, thus
            // enabling faster feedback cycles.
            let __orig_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |panic_info| {
                // invoke the default handler and exit the process
                __orig_hook(panic_info);
            }));
            std::thread::spawn(|| {
                let timeout: String = std::env::var("WATERFALL_TIMEOUT_SECS_PER_TEST")
                    .unwrap_or(String::from("180")); // 3 * 60 = three minutes
                let timeout: u64 = timeout.parse::<u64>()
                    .expect("unable to parse WATERFALL_TEST_TIMEOUT into u64");
                std::thread::sleep(std::time::Duration::from_secs(timeout));

                std::process::Command::new("pkill")
                    .args(&["-9", "-f", "geckodriver"])
                    .output()
                    .expect("can not execute pkill");

                panic!("The test '{}' didn't finish in time.", stringify!(#fn_name));
            });

            let mut canvas_ui = CanvasUi::new().await?;
            let __ret = {
                #block
            };
            canvas_ui.shutdown().await?;
            __ret
        }
    };
    res.into()
}
