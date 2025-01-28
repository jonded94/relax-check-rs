use crate::async_utils;
use anyhow::Context;
use core::time::Duration;
use log::{info, warn};
use metrics::{describe_gauge, gauge, Gauge, Unit};
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
enum CarlousOccupancyParsingError {
    #[error("Could not find occupancy value '{0}'")]
    OccupancyValueNotFound(&'static str),
    #[error("Could not find tag '{0}'")]
    TagNotFound(&'static str),
    #[error("Tag '{0}' has no text")]
    TagHasNoText(&'static str),
    #[error("Tag '{0}' has unexpected text '{1}'")]
    TagHasUnexpectedText(&'static str, String),
}

#[derive(Debug)]
struct Occupancy {
    thermalbad: u8,
    saunawelt: u8,
    parkhaus: u8,
}

impl Occupancy {
    fn from_hashmap(map: HashMap<&str, u8>) -> Result<Self, CarlousOccupancyParsingError> {
        Ok(Occupancy {
            thermalbad: *map.get("Thermalbad").ok_or(
                CarlousOccupancyParsingError::OccupancyValueNotFound("Thermalbad"),
            )?,
            saunawelt: *map.get("Saunawelt").ok_or(
                CarlousOccupancyParsingError::OccupancyValueNotFound("Saunawelt"),
            )?,
            parkhaus: *map.get("Parkhaus").ok_or(
                CarlousOccupancyParsingError::OccupancyValueNotFound("Parkhaus"),
            )?,
        })
    }
}

#[derive(Debug)]
struct OccupancyGauges {
    thermalbad: Gauge,
    saunawelt: Gauge,
    parkhaus: Gauge,
}

impl OccupancyGauges {
    fn new() -> Self {
        // Macro now takes both struct field and display name component
        macro_rules! build_gauges {
            ($($field:ident => $display_name:literal),+) => {
                Self {
                    $(
                        $field: {
                            // Generate metric name from field identifier
                            let metric_name = concat!("carolus_", stringify!($field), "_ratio");

                            // Generate human-readable description from display name
                            let description = concat!(
                                "Occupancy of the ",
                                $display_name,
                                " of Carolus Thermen Aachen as percentage."
                            );

                            let gauge = gauge!(metric_name);
                            describe_gauge!(metric_name, Unit::Percent, description);
                            gauge
                        },
                    )+
                }
            };
        }

        build_gauges! {
            thermalbad => "Thermalbad",
            saunawelt => "Saunawelt",
            parkhaus => "Parkhaus"
        }
    }

    fn set(&self, occupancy: Occupancy) {
        self.thermalbad.set(occupancy.thermalbad);
        self.saunawelt.set(occupancy.saunawelt);
        self.parkhaus.set(occupancy.parkhaus);
    }
}

const OCCUPANCY_PAGE: &str = "https://carolus-thermen.de/auslastung/";

static ITEMS_SELECTOR: Lazy<Selector> =
    Lazy::new(|| Selector::parse("div.modulAuslastugsGrid div.item-aus.center").unwrap());
static P_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("p").unwrap());
static SPAN_SELECTOR: Lazy<Selector> = Lazy::new(|| Selector::parse("span").unwrap());

fn get_current_occupancy(html_page: &str) -> Result<Occupancy, CarlousOccupancyParsingError> {
    // Parse document with owned String input
    let fragment = Html::parse_document(html_page);
    let mut data = HashMap::new();

    for item in fragment.select(&ITEMS_SELECTOR) {
        // Extract category using static string checks
        let category = item
            .select(&P_SELECTOR)
            .next()
            .ok_or(CarlousOccupancyParsingError::TagNotFound("p"))?
            .text()
            .next()
            .ok_or(CarlousOccupancyParsingError::TagHasNoText("p"))?
            .trim();

        // Extract value from span
        let value_text = item
            .select(&SPAN_SELECTOR)
            .next()
            .ok_or(CarlousOccupancyParsingError::TagNotFound("span"))?
            .text()
            .next()
            .ok_or(CarlousOccupancyParsingError::TagHasNoText("span"))?
            .trim();

        let value = value_text
            .strip_suffix(" %")
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| {
                CarlousOccupancyParsingError::TagHasUnexpectedText(
                    "span",
                    value_text.to_string(), // Only copy when creating error
                )
            })?;

        data.insert(category, value);
    }

    Occupancy::from_hashmap(data)
}

pub async fn occupancy_loop(duration: Duration) {
    let gauges = OccupancyGauges::new();
    loop {
        let occupancy_maybe: anyhow::Result<Occupancy> = async {
            let page = async_utils::fetch_url(OCCUPANCY_PAGE)
                .await
                .context("Failed to fetch Carolus page")?;
            get_current_occupancy(&page).context("Failed to parse Carolus page")
        }
        .await;
        match occupancy_maybe {
            Ok(occupancy) => {
                info!("Set occupancy {occupancy:?}");
                gauges.set(occupancy);
            }
            Err(e) => warn!("Error occupancy loop: {}", e),
        }
        tokio::time::sleep(duration).await;
    }
}
