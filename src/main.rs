mod config;
mod utils;
mod widgets;

use config::I3Config;
use log::LevelFilter;
use utils::logger::Logger;
use widgets::battery_life::Battery;
use widgets::cpu_stats::CpuUsage;
use widgets::cpu_stats::CpuUsageType;
use widgets::disk_stats::Disk;
use widgets::memory_stats::MemoryUsage;
use widgets::network_information::NetworkInformation;
use widgets::network_information::NetworkType;
use widgets::time::Time;

fn main() {
    // Set logger
    let logger = Logger::new();
    if let Err(error) =
        log::set_boxed_logger(logger).map(|()| log::set_max_level(LevelFilter::Error))
    {
        println!("Enable to set logger: {}", error);
    }

    let mut final_config = I3Config::new(vec![
        Box::new(NetworkInformation::new(NetworkType::Wlan)),
        Box::new(NetworkInformation::new(NetworkType::Ethernet)),
        Box::new(Battery::new()),
        Box::new(CpuUsage::new(CpuUsageType::CpuLoad)),
        Box::new(CpuUsage::new(CpuUsageType::Percentage)),
        Box::new(MemoryUsage::new()),
        Box::new(Disk::new(String::from("root"), String::from("/"))),
        Box::new(Time::new()),
    ]);

    final_config.init();
}
