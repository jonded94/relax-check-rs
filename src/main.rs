mod async_utils;
mod monitors;

use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("RELAX_CHECK_PORT")
        .map(|port_str| {
            port_str
                .parse()
                .unwrap_or_else(|_| panic!("Could not parse port '{port_str}' as an integer"))
        })
        .unwrap_or(9000);
    let interval = std::env::var("RELAX_CHECK_INTERVAL")
        .map(|interval_str| {
            Duration::from_secs(interval_str.parse().unwrap_or_else(|_| {
                panic!("Could not parse interval '{interval_str}' as an integer")
            }))
        })
        .unwrap_or(Duration::from_secs(60 * 5));

    let builder = PrometheusBuilder::new()
        .with_http_listener(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port));
    builder
        .install()
        .expect("failed to install recorder/exporter");

    env_logger::init();

    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    handles.push(tokio::spawn(monitors::carolus::occupancy_loop(interval)));

    async_utils::wait_for_shutdown(&handles).await;
}
