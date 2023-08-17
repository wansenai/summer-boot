

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn log_print() {
        // 在使用log的时候需要调用start()方法开启log记录
        log::start();

        log::info!("Hello Summer Boot");
        
        // debug 模式下日志记录
        log::debug!("debug apps");

        log::error!("process error");

        log::warn!("warning apps");
    }

}