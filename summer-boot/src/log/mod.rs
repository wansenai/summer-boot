//! 事件日志记录类型
//! 
//!
//! # Examples
//!
//! ```
//! use summer_boot::log;
//!
//! log::start();
//!
//! log::info!("Hello James");
//! log::debug!("{} eat rice", "James");
//! log::error!("this is an error!");
//! log::info!("{} are win", "test", {
//!     key_1: "value1",
//!     key_2: "value2",
//! });
//! ```

pub use kv_log_macro::{debug, error, info, log, trace, warn};
pub use kv_log_macro::{max_level, Level};

mod logging_system;

pub use femme::LevelFilter;

pub use logging_system::LoggingSystem;

/// 开启日志记录
pub fn start() {
    femme::start();
    crate::log::info!("Logger started");
    crate::log::info!("
    _____                                       ____              _   
   / ____|                                     |  _ \\            | |  
  | (___  _   _ _ __ ___  _ __ ___   ___ _ __  | |_) | ___   ___ | |_ 
   \\___ \\| | | | '_ ` _ \\| '_ ` _ \\ / _ \\ '__| |  _ < / _ \\ / _ \\| __|
   ____) | |_| | | | | | | | | | | |  __/ |    | |_) | (_) | (_) | |_ 
  |_____/ \\__,_|_| |_| |_|_| |_| |_|\\___|_|    |____/ \\___/ \\___/ \\__|
                                                                      
  :: Summer Boot Version::             (0.1.0)                                                                    
 ");
}

/// 使用日志级别开启日志记录
pub fn with_level(level: LevelFilter) {
    femme::with_level(level);
    crate::log::info!("Logger started", { level: format!("{}", level) });
    crate::log::info!("
    _____                                       ____              _   
   / ____|                                     |  _ \\            | |  
  | (___  _   _ _ __ ___  _ __ ___   ___ _ __  | |_) | ___   ___ | |_ 
   \\___ \\| | | | '_ ` _ \\| '_ ` _ \\ / _ \\ '__| |  _ < / _ \\ / _ \\| __|
   ____) | |_| | | | | | | | | | | |  __/ |    | |_) | (_) | (_) | |_ 
  |_____/ \\__,_|_| |_| |_|_| |_| |_|\\___|_|    |____/ \\___/ \\___/ \\__|
                                                                      
  :: Summer Boot Version::             (0.1.0)                                                                    
 ");
}
