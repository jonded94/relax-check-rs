mod async_utils;
mod monitors;

use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install recorder/exporter");

    env_logger::init();

    monitors::carolus::occupancy_loop(Duration::from_secs(60 * 5)).await;
}
