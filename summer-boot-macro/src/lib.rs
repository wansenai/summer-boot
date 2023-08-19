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
use serde_json::Value;
use std::io::Read;
use std::path::Path;
use std::{fmt::format, fs};
use syn::{
    parse_file, parse_macro_input, parse_quote, punctuated::Punctuated, AttributeArgs, Item,
    ItemFn, Lit, Meta, NestedMeta, Pat, Stmt, Token,
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

/// 完成 summer_boot 项目下的自动扫描功能，会先扫描找到`summer_boot::run();`
/// 函数，然后在此处进行装配活动。也可以手动增加过滤路径或过滤文件。
/// 如果增加过滤路径，需要在末尾添加 `/`，如果增加过滤文件，需要在末尾添加 `.rs`。
///
/// 注意：如果需要在此处添加运行时，必须在当前宏的后面配置，否则无法完成装配
/// # Examples
/// ```rust
/// // #[summer_boot::auto_scan]
/// // #[summer_boot::auto_scan("summer-boot-tests/src/lib.rs")]
/// fn main() {
///     summer_boot::run();
/// }
/// ```
#[proc_macro_attribute]
pub fn auto_scan(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut project = Vec::<String>::new();
    let mut filter_paths = Vec::<String>::new();
    // 找到需要扫描的路径
    let mut cargo_toml = fs::File::open("Cargo.toml").expect("Cargo Toml文件找不到");
    let mut content = String::new();

    cargo_toml
        .read_to_string(&mut content)
        .expect("Cargo内容为空");

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

    // 解析宏信息
    let args = parse_macro_input!(args as AttributeArgs);
    for arg in args {
        if let NestedMeta::Lit(Lit::Str(project)) = arg {
            filter_paths.push(project.value());
        }
    }

    // 解析函数体
    let mut input = parse_macro_input!(input as ItemFn);

    // 查找主函数的位置和是否存在变量名
    // 未找到则直接退出宏处理
    // 变量名不存在则默认添加app
    // 如果存在则会返回出来，供后续使用
    if let Some((master_index, master_name)) = scan_master_fn(&mut input) {
        // 解析yaml文件
        let mut listener_addr = String::from("0.0.0.0:");
        let mut app_context_path = String::from("");
        let config = summer_boot_autoconfigure::load_conf();
        if let Some(config) = config {
            let read_server = serde_json::to_string(&config.server).expect("读取服务配置文件失败");
            let v: Value = serde_json::from_str(&read_server).expect("读取服务配置文件失败");
            let port = v["port"].to_string();
            let context_path = v["context_path"].to_string();
            listener_addr.push_str(&port);
            app_context_path.push_str(&context_path);
        }

        // 开始扫描
        for path in project {
            scan_method(
                &path,
                &filter_paths,
                &mut input,
                &app_context_path,
                (master_index, &master_name),
            );
        }

        // 配置listen
        input.block.stmts.push(parse_quote! {
            #master_name.listen(#listener_addr).await.expect("配置listen失败");
        });
    }

    // 构建新的函数结构，增加函数行
    TokenStream::from(input.into_token_stream())
}

// 扫描函数，找到主函数
// 返回主函数所在的位置索引，并判断是否存在变量名
// 如果存在，则找到并返回
// 如果不存在，则删除默认主函数，添加新的主函数
fn scan_master_fn(input: &mut ItemFn) -> Option<(i32, Ident)> {
    let mut master_index: i32 = -1;
    let mut master_name = Ident::new("app", Span::call_site());

    for (index, stmt) in (&mut input.block.stmts).iter_mut().enumerate() {
        let master = stmt.to_token_stream().to_string();
        if let Some(_) = master.find("summer_boot :: run()") {
            master_index = index as i32;
        }
    }
    if master_index < 0 {
        None
    } else {
        if let Stmt::Local(local) = &input.block.stmts[master_index as usize] {
            // 函数存在变量，需要获取变量名称
            let pat = &local.pat;

            if let Pat::Ident(pat_ident) = pat {
                let name = pat_ident.ident.to_string();
                master_name = Ident::new(&name, Span::call_site());
            }
        } else {
            // 函数不存在变量，需要手动添加
            // TODO 目前相对简单，删除当前函数，并添加指定的函数即可，后续建议修改
            input.block.stmts.remove(master_index as usize);
            input.block.stmts.insert(
                master_index as usize,
                parse_quote! {
                    let mut app = summer_boot::run();
                },
            )
        }

        Some((master_index, master_name))
    }
}

// 判断是否是目录，如果是路径则需要循环递归处理，
// 如果是文件则直接处理
// 处理过程中会将函数调用函数拼接，然后插入到指定的位置 下标+1 的位置
fn scan_method(
    path: &str,
    filter_paths: &Vec<String>,
    input_token_stream: &mut ItemFn,
    context_path: &str,
    (mut master_index, master_name): (i32, &Ident),
) {
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();
                if file_path.is_file() {
                    if let Some(extension) = file_path.extension() {
                        if extension == "rs" {
                            if filter_paths.iter().any(|p| path.contains(p)) {
                                return;
                            }
                            // 如果是文件，则处理内部细节
                            let content = fs::read_to_string(entry.path()).expect("处理内部细节");
                            // 解析文件
                            let ast = parse_file(&content).expect("解析文件失败");
                            let items = ast.items;
                            for item in items {
                                if let Item::Fn(item) = item {
                                    // 处理函数中的函数名，指定宏信息
                                    for attr in item.attrs {
                                        // 遍历所有宏信息
                                        if let Meta::List(meta) =
                                            attr.parse_meta().expect("所有所有宏信息")
                                        {
                                            // 判断宏是否为指定的宏
                                            let attr_path = meta.path.to_token_stream().to_string();

                                            let method = config_req_type(&attr_path);
                                            if method.is_none() {
                                                continue;
                                            }
                                            let method =
                                                method.expect("是否为指定的宏").to_token_stream();

                                            // 获取函数全路径名
                                            let fn_name: &String = &item.sig.ident.to_string();
                                            let fn_path_token_stream = config_function_path(
                                                &file_path.to_str().unwrap_or("文件为空"),
                                                fn_name,
                                            );

                                            // 如果是 summer_boot 的宏信息，则处理
                                            let attr_url = meta
                                                .nested
                                                .into_iter()
                                                .next()
                                                .expect("summer_boot 的宏信息");
                                            if let NestedMeta::Lit(Lit::Str(url)) = attr_url {
                                                let url = url.value();
                                                let url = format!("{}{}", context_path, url)
                                                    .replace("\"", "")
                                                    .replace("//", "/");

                                                if input_token_stream.block.stmts.len() < 1 {
                                                    // 如果注入的方法中没有任何代码，则不操作
                                                    break;
                                                } else {
                                                    // 添加，注意下标加 1
                                                    master_index += 1;
                                                    input_token_stream.block.stmts.insert(
                                                    master_index as usize,
                                                    parse_quote! {
                                                        #master_name.at(#url).#method(#fn_path_token_stream);
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
            }
        }
    }
}

// 配置函数全路径
// 根据相对项目的绝对路径找到函数调用的全路径链
// 注意：目前无法完成文件中mod下的函数调用，无法找到
fn config_function_path(path: &str, fu_name: &str) -> proc_macro2::TokenStream {
    let mut fn_path_idents = Punctuated::<Ident, Token![::]>::new();
    fn_path_idents.push(Ident::new("crate", Span::call_site()));

    // 配置函数路径
    let names: Vec<&str> = path
        [path.find("src").expect("转换src") + 4..path.rfind(".rs").expect("转换rs后缀")]
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
            pub fn $method(_args: TokenStream, input: TokenStream) -> TokenStream {

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
