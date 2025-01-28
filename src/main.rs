mod async_utils;
mod carolus;

use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let builder = PrometheusBuilder::new();
    builder
        .install()
        .expect("failed to install recorder/exporter");

    env_logger::init();

    carolus::occupancy_loop(Duration::from_secs(60 * 5)).await;
}
