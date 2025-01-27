use std::collections::HashMap;

use scraper::{Html, Selector};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RelaxCheckParsingError<'a> {
    #[error("Could not find occupancy value '{0}'")]
    OccupancyValueNotFound(&'a str),
    #[error("Could not find tag '{0}'")]
    TagNotFound(&'a str),
    #[error("Tag '{0}' has no text")]
    TagHasNoText(&'a str),
    #[error("Tag '{0}' has unexpected text '{1}'")]
    TagHasUnexpectedText(&'a str, &'a str),
}

struct Occupancy {
    thermalbad: u8,
    saunawelt: u8,
    parkhaus: u8,
}

impl Occupancy {
    fn from_hashmap<'a>(map: &HashMap<&'a str, u8>) -> Result<Self, RelaxCheckParsingError<'a>>  {
        Ok(Occupancy {
            thermalbad: *map
                .get("Thermalbad")
                .ok_or(RelaxCheckParsingError::OccupancyValueNotFound("Thermalbad"))?,
            saunawelt: *map
                .get("Saunawelt")
                .ok_or(RelaxCheckParsingError::OccupancyValueNotFound("Saunawelt"))?,
            parkhaus: *map
                .get("Parkhaus")
                .ok_or(RelaxCheckParsingError::OccupancyValueNotFound("Parkhaus"))?,
        })
    }
}


async fn get_current_occupancy<'a>(html_page: &'a str) -> Result<Occupancy, RelaxCheckParsingError<'a>> {
    let ITEMS_SELECTOR: Selector =
        Selector::parse("div.modulAuslastugsGrid div.item-aus.center").unwrap();
    let P_SELECTOR: Selector = Selector::parse("p").unwrap();
    let SPAN_SELECTOR: Selector = Selector::parse("span").unwrap();

    let fragment = Html::parse_fragment(html_page);

    let mut data = HashMap::<&str, u8>::new();

    for item in fragment.select(&ITEMS_SELECTOR) {
        let category_tag = item
            .select(&P_SELECTOR)
            .next()
            .ok_or(RelaxCheckParsingError::TagNotFound("p"))?;
        let category = category_tag
            .text()
            .next()
            .ok_or(RelaxCheckParsingError::TagHasNoText("p"))?
            .trim();

        let value_tag = item
            .select(&SPAN_SELECTOR)
            .next()
            .ok_or(RelaxCheckParsingError::TagNotFound("span"))?;
        let value_text = value_tag
            .text()
            .next()
            .ok_or(RelaxCheckParsingError::TagHasNoText("span"))?
            .trim();
        let value = value_text
            .strip_suffix(" %")
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| RelaxCheckParsingError::TagHasUnexpectedText("span", value_text))?;

        data.insert(category, value);
    }

    Occupancy::from_hashmap(&data)
}

fn main() {
    println!("Hello, world!");
    get_current_occupancy("123");
}
