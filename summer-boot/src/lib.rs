pub mod web2;
pub mod rt;

/// 建立过程宏与summer boot的关联
macro_rules! codegen_reexport {
    ($name:ident) => {
        #[cfg(feature = "macros")]
        #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
        pub use summer_boot_codegen::$name;
    };
}

codegen_reexport!(main);