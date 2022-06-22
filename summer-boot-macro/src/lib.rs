//! 运行时宏处理
//!
//! # main
//! 使用运行时宏来设置summerboot async运行时。参见[main]宏文档。
//!
//! # auto_scan
//! 提供了基础的`auto_scan`功能用于发现并自动注册路由。
//!
//! # post、get、delete、put、patch、head、options、connect、trace
//! 提供了简单的路由宏标注。
//!
//!

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::fs;
use std::io::Read;
use syn::{
    parse_file, parse_macro_input, parse_quote, punctuated::Punctuated, Item, ItemFn, Lit, Meta,
    NestedMeta, Token,
};

/// 用于匹配项目根目录下的 `Cargo.toml` 文件。
/// 匹配规则为：
/// 1. workspace下的member的数组格式
/// 2. 在package下的name字段
#[derive(Debug, Deserialize)]
struct ConfWorkSpace {
    workspace: Option<Member>,
    package: Option<Name>,
}

/// 匹配workspace下的member数组格式
#[derive(Debug, Deserialize)]
struct Member {
    members: Option<Vec<String>>,
}

/// 匹配package下的name字段
#[derive(Debug, Deserialize)]
struct Name {
    name: String,
}

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
    let mut input = parse_macro_input!(item as ItemFn);
    let attrs = &input.attrs;
    let vis = &input.vis;
    let sig = &mut input.sig;
    let body = &input.block;
    let _name = &sig.ident;

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
    })
    .into()
}

/// 完成 summer_boot 项目下的自动扫描功能
/// # Examples
/// ```rust
/// #[summer_boot::auto_scan]
/// fn main() {
///     println!("stmt 0");
///     let app = summer_boot::new();
///                                   // <---- 在这里开始自动注入路由, 印象该注入位置的代码为`input.block.stmts.insert(2, parse_quote! {`
///     println!("stmt 2");
/// }
/// ```
#[proc_macro_attribute]
pub fn auto_scan(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut project = Vec::<String>::new();

    // 找到需要扫描的路径
    let mut cargo_toml = fs::File::open("Cargo.toml").unwrap();
    let mut content = String::new();
    cargo_toml.read_to_string(&mut content).unwrap();

    // 根据包类型分别处理
    if let Ok(conf_work_space) = toml::from_str::<ConfWorkSpace>(&content) {
        if let Some(workspace) = conf_work_space.workspace {
            if let Some(members) = workspace.members {
                for member in members {
                    project.push(format!("{}/{}", member, "src"));
                }
            }
        } else if project.len() == 0 {
            if let Some(_) = conf_work_space.package {
                project.push("src".to_string());
            }
        }
    }

    let mut input = parse_macro_input!(input as ItemFn);

    // 开始扫描
    for path in project {
        scan(&path, &mut input);
    }

    // 构建新的函数结构，增加函数行
    TokenStream::from(input.into_token_stream())
}

// 判断是否是目录，如果是路径则需要循环处理，
// 如果是文件则直接处理
fn scan(path: &str, input: &mut ItemFn) {
    let mut file = fs::File::open(path).unwrap();
    let file_type = file.metadata().unwrap();
    if file_type.is_dir() {
        // 获取当前文件夹下的所有文件
        let mut files = fs::read_dir(path).unwrap();

        // 循环里面的所有文件
        while let Some(file) = files.next() {
            let file = file.unwrap();
            // TODO 过滤带test文件夹的扫描
            scan(&file.path().to_str().unwrap(), input);
        }
    } else {
        // 判断文件名后缀是否是.rs
        if path.ends_with(".rs") {
            // 如果是文件，则处理内部细节
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();

            // 解析文件
            let ast = parse_file(&content).unwrap();
            let items = ast.items;
            for item in items {
                if let Item::Fn(item) = item {
                    // 处理函数中的函数名，指定宏信息
                    for attr in item.attrs {
                        // 遍历所有宏信息
                        if let Meta::List(meta) = attr.parse_meta().unwrap() {
                            // 判断宏是否为指定的宏
                            let attr_path = meta.path.to_token_stream().to_string();

                            let method = config_req_type(&attr_path);
                            if method.is_none() {
                                continue;
                            }
                            let method = method.unwrap().to_token_stream();

                            // 获取函数全路径名
                            let fn_name = &item.sig.ident.to_string();
                            let fn_path_token_stream = config_function_path(&path, fn_name);

                            // 如果是 summer_boot 的宏信息，则处理
                            let attr_url = meta.nested.into_iter().next().unwrap();
                            if let NestedMeta::Lit(Lit::Str(url)) = attr_url {
                                let url = url.value();
                                if input.block.stmts.len() < 1 {
                                    // 如果注入的方法中没有任何代码，则不操作
                                    break;
                                } else {
                                    // 添加
                                    input.block.stmts.insert(
                                        2,
                                        parse_quote! {
                                            app.at(#url).#method(#fn_path_token_stream);
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// 配置函数全路径
fn config_function_path(path: &str, fu_name: &str) -> proc_macro2::TokenStream {
    let mut fn_path_idents = Punctuated::<Ident, Token![::]>::new();
    fn_path_idents.push(Ident::new("crate", Span::call_site()));

    // 配置函数路径
    let names: Vec<&str> = (&path[path.find("src/").unwrap() + 4..path.rfind(".rs").unwrap()])
        .split("/")
        .collect();

    let len = names.len();
    for (index, name) in names.into_iter().enumerate() {
        if (index + 1) == len {
            // 最后一个文件名称如果是main、lib、test则不需要加入路径
            match name {
                "main" | "mod" | "lib" => {
                    break;
                }
                _ => {}
            }
        }
        if !name.is_empty() {
            // 配置文件包名
            fn_path_idents.push(Ident::new(name, Span::call_site()));
        }
    }
    // 配置函数名称
    fn_path_idents.push(Ident::new(fu_name, Span::call_site()));

    fn_path_idents.to_token_stream()
}

// 配置请求类型
fn config_req_type(attr_path: &str) -> Option<Ident> {
    if attr_path == "summer_boot_macro :: get"
        || attr_path == "summer_boot :: get"
        || attr_path == "get"
        || attr_path == "summer_boot_macro :: head"
        || attr_path == "summer_boot :: head"
        || attr_path == "head"
        || attr_path == "summer_boot_macro :: put"
        || attr_path == "summer_boot :: put"
        || attr_path == "put"
        || attr_path == "summer_boot_macro :: post"
        || attr_path == "summer_boot :: post"
        || attr_path == "post"
        || attr_path == "summer_boot_macro :: delete"
        || attr_path == "summer_boot :: delete"
        || attr_path == "delete"
        || attr_path == "summer_boot_macro :: head"
        || attr_path == "summer_boot :: head"
        || attr_path == "head"
        || attr_path == "summer_boot_macro :: options"
        || attr_path == "summer_boot :: options"
        || attr_path == "options"
        || attr_path == "summer_boot_macro :: connect"
        || attr_path == "summer_boot :: connect"
        || attr_path == "connect"
        || attr_path == "summer_boot_macro :: patch"
        || attr_path == "summer_boot :: patch"
        || attr_path == "patch"
        || attr_path == "summer_boot_macro :: trace"
        || attr_path == "summer_boot :: trace"
        || attr_path == "trace"
    {
        if attr_path.starts_with("summer_boot_macro ::") {
            return Some(Ident::new(
                &attr_path["summer_boot_macro :: ".len()..],
                Span::call_site(),
            ));
        } else if attr_path.starts_with("summer_boot ::") {
            return Some(Ident::new(
                &attr_path["summer_boot :: ".len()..],
                Span::call_site(),
            ));
        } else {
            return Some(Ident::new(attr_path, Span::call_site()));
        }
    } else {
        return None;
    }
}

macro_rules! doc_comment {
    ($x:expr; $($tt:tt)*) => {
        #[doc = $x]
        $($tt)*
    };
}

macro_rules! method_macro {
    (
        $($method:ident,)+
    ) => {
        $(doc_comment! {
concat!("
# 功能
创建路由接口，用于`summer_boot.new()`的返回值使用，
该函数提供了对应方法`summer_boot/src/web2/gateway/routes.rs`文件下的所有路由方法，

# 支持的路由如下：
- get
- head
- put
- post
- delete
- options
- connect
- patch
- trace

# 例子：
```rust
# use summer_boot::{Request, Result};
#[summer_boot_macro::", stringify!($method), r#"("/")]
async fn example(mut req: Request<()>) -> Result {
    Ok(format!("Hello World").into())
}
```
"#);
            #[proc_macro_attribute]
            pub fn $method(args: TokenStream, input: TokenStream) -> TokenStream {

                let mut input = parse_macro_input!(input as ItemFn);
                let attrs = &input.attrs;
                let vis = &input.vis;
                let sig = &mut input.sig;
                let body = &input.block;
                let _name = &sig.ident;
                if sig.asyncness.is_none() {
                    return syn::Error::new_spanned(sig.fn_token, "仅支持 async fn")
                        .to_compile_error()
                        .into();
                }

                (quote! {
                    #(#attrs)*
                    #vis #sig
                        #body
                }).into()
            }
        })+
    };
}

method_macro!(get, head, put, post, delete, patch, trace, options, connect,);
