use log::LevelFilter;
use log4rs::{append::{file::FileAppender}, config::{Appender, Logger, Root}, encode::pattern::PatternEncoder, Config};
use std::path::Path;

pub struct LoggerInitArgs {
    pub debug_path_string: String,
    pub info_path_string: String,
    pub warn_path_string: String,
    pub error_path_string: String,
}

pub struct CustomLogger {}

impl CustomLogger {
    pub fn init(logger_init_args: LoggerInitArgs) {

        let debug_path = Path::new(&logger_init_args.debug_path_string);
        let info_path = Path::new(&logger_init_args.info_path_string);
        let warn_path = Path::new(&logger_init_args.warn_path_string);
        let error_path = Path::new(&logger_init_args.error_path_string);

        if debug_path.exists() {
            let _ = std::fs::remove_file(debug_path);
        } 
        if info_path.exists() {
            let _ = std::fs::remove_file(info_path);
        } 
        if warn_path.exists() {
            let _ = std::fs::remove_file(warn_path);
        } 
        if error_path.exists() {
            let _ = std::fs::remove_file(error_path);
        } 

        let debug_ap = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(debug_path).unwrap();
        let info_ap = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(info_path).unwrap();
        let warn_ap = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(warn_path).unwrap();
        let error_ap = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{l} - {m}\n")))
            .build(error_path).unwrap();

        let config = Config::builder()
            .appender(Appender::builder().build("debug_ap", Box::new(debug_ap)))
            .appender(Appender::builder().build("info_ap", Box::new(info_ap)))
            .appender(Appender::builder().build("warn_ap", Box::new(warn_ap)))
            .appender(Appender::builder().build("error_ap", Box::new(error_ap)))
            .logger(
                Logger::builder()
                    .appender("debug_ap")
                    .build("debug", LevelFilter::Debug),
                )
            .logger(
                Logger::builder()
                    .appender("info_ap")
                    .build("info", LevelFilter::Info),
                )
            .logger(
                Logger::builder()
                    .appender("warn_ap")
                    .build("warn", LevelFilter::Warn),
                )
            .logger(
                Logger::builder()
                    .appender("error_ap")
                    .build("error", LevelFilter::Error),
                )
            .build(Root::builder().build(LevelFilter::Debug))
            .unwrap();


        let _handle = log4rs::init_config(config).unwrap();
    }
}

#[macro_export]
macro_rules! logd {
    ($($args:tt),*) => {
        #[cfg(any(all(debug_assertions, not(feature="no_logging")), feature = "log_debug"))]
        {
            log::debug!(target: "debug", $($args),*);
        }
    };
}

#[macro_export]
macro_rules! logi {
    ($($args:tt),*) => {
        #[cfg(any(all(debug_assertions, not(feature="no_logging")), feature = "log_info"))]
        {
            log::info!(target: "info", $($args),*);
        }
    };
}

#[macro_export]
macro_rules! logw {
    ($($args:tt),*) => {
        #[cfg(any(all(debug_assertions, not(feature="no_logging")), feature = "log_warn"))]
        {
            log::warn!(target: "warn", $($args),*);
        }
    };
}

#[macro_export]
macro_rules! loge {
    ($($args:tt),*) => {
        #[cfg(any(all(debug_assertions, not(feature="no_logging")), feature = "log_error"))]
        {
            log::error!(target: "error", $($args),*);
        }
        
    };
}