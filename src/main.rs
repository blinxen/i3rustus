mod config;
mod i3_status;
mod netlink;
mod utils;
mod widget_executor;
mod widgets;

use config::Config;
use i3_status::I3Status;
use log::LevelFilter;
use utils::logger::Logger;
use widgets::battery_life::Battery;
use widgets::brightness::Brightness;
use widgets::cpu_stats::CpuUsage;
use widgets::cpu_stats::CpuUsageType;
use widgets::disk_stats::Disk;
use widgets::memory_stats::MemoryUsage;
use widgets::network_information::NetworkInformation;
use widgets::network_information::NetworkType;
use widgets::time::Time;

#[actix_rt::main]
async fn main() {
    // Set logger
    let logger = Logger::new();
    if let Err(error) =
        log::set_boxed_logger(logger).map(|()| log::set_max_level(LevelFilter::Error))
    {
        println!("Enable to set logger: {}", error);
    }

    let config = Config::new();

    let mut i3status = I3Status::new(
        config.widget_order(),
        vec![
            Box::new(NetworkInformation::new(NetworkType::Wlan)),
            Box::new(NetworkInformation::new(NetworkType::Ethernet)),
            Box::new(Battery::new(config.battery_device_name())),
            Box::new(CpuUsage::new(CpuUsageType::CpuLoad)),
            Box::new(CpuUsage::new(CpuUsageType::Percentage)),
            Box::new(MemoryUsage::new()),
            Box::new(Disk::new(String::from("root"), String::from("/"))),
            Box::new(Time::new()),
            Box::new(Brightness::new(config.brightness_device_name())),
        ],
    );

    i3status.init().await;
}
