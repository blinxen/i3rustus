mod config;
mod i3_status;
mod netlink;
mod utils;
mod widget_executor;
mod widgets;

use i3_status::I3Status;
use log::LevelFilter;
use utils::logger::Logger;

#[actix_rt::main]
async fn main() {
    // Set logger
    let logger = Logger::new();
    if let Err(error) =
        log::set_boxed_logger(logger).map(|()| log::set_max_level(LevelFilter::Error))
    {
        println!("Enable to set logger: {}", error);
    }

    let mut i3status = I3Status::new();

    i3status.init().await;
}
