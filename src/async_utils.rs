use backoff::exponential::ExponentialBackoff;
use backoff::{future::retry_notify, SystemClock};
use log::warn;

pub async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    retry_notify(
        ExponentialBackoff::<SystemClock>::default(),
        || async { Ok(reqwest::get(url).await?.text().await?) },
        |e, dur| warn!("Could not retrieve {url} at {dur:?}: {e:?}"),
    )
    .await
}

pub async fn wait_for_shutdown<T>(handles: &Vec<tokio::task::JoinHandle<T>>) {
    // Graceful shutdown handler
    let shutdown = async {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\nCtrl-C received, shutting down...");
    };

    // Run until shutdown signal
    shutdown.await;

    // Cancel all tasks and wait for completion
    for handle in handles {
        handle.abort();
    }
}
