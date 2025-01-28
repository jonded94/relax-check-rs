use backoff::exponential::ExponentialBackoff;
use backoff::{future::retry_notify, SystemClock};
use log::warn;

pub async fn fetch_url(url: &str) -> Result<String, reqwest::Error> {
    retry_notify(
        ExponentialBackoff::<SystemClock>::default(),
        || async { Ok(reqwest::get(url).await?.text().await?) },
        |e, dur| warn!("Could not retrieve {url} at {dur:?}: {e}"),
    )
    .await
}
