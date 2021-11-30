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
/// async fn works(mut ui: Ui) -> Result<()> {
///     let _contract_addr = ui.upload(contract_file).await?;
///     Ok(())
/// }
/// ```
#[proc_macro_attribute]
pub fn waterfall_test(waterfall_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let example = waterfall_attrs
        .into_iter()
        .find_map(|item| {
            if let proc_macro::TokenTree::Literal(lit) = item {
                return Some(format!("{}", lit.to_string().replace("\"", "")))
            }
            None
        })
        .expect("example param must exist");

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
            log::debug!("setting up test for {}", stringify!(#fn_name));
            crate::TEST_NAME.with(|test_name| {
                let str = format!("example: {}, test: {}", #example, stringify!(#fn_name));
                *test_name.borrow_mut() = String::from(str);
            });
            crate::INIT.call_once(|| {
                env_logger::init();
            });

            use crate::uis::ContractsUi;
            log::debug!("creating new ui for {}", stringify!(#fn_name));
            let mut ui = Ui::new().await?;
            log::debug!("invoking block for {}", stringify!(#fn_name));
            let __ret = {
                #block
            };
            ui.shutdown().await?;
            log::debug!("shutdown for {} complete", stringify!(#fn_name));
            __ret
        }
    };
    res.into()
}
