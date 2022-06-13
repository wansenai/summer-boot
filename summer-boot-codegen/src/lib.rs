//! 运行时宏处理
//! # Runtime Setup
//! 使用运行时宏来设置summerboot async运行时。参见[main]宏文档。
//!
//! ```
//! #[summer_boot_codegen::main] // 或者 `#[summer_boot::main]`
//! async fn main() {
//!     async { println!("Hello world"); }.await
//! }
//! ```
//!

use proc_macro::{TokenStream};
use quote::quote;

/// 用于标记 summer_boot web 的入口点
/// # Examples
/// ```
/// #[summer_boot::main]
/// async fn main() {
///     async { println!("Hello world"); }.await
/// }
/// ```
#[proc_macro_attribute]
pub fn main(_: TokenStream, item: TokenStream) -> TokenStream {
    todo!("检测item格式是否正确");

    let mut input = syn::parse_macro_input!(item as syn::ItemFn);
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &mut input.sig;
    let body = &input.block;
    let name = &sig.ident;

    if sig.asyncness.is_none() {
        return syn::Error::new_spanned(sig.fn_token, "仅支持 async fn")
            .to_compile_error()
            .into();
    }
    sig.asyncness = None;

    (quote! {
        #(#attrs)*
        #vis #sig {
            summer_boot::rt::SummerRuntime::new()
            .block_on(async move { #body });
        }
    }).into()
}

