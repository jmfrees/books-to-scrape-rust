use crate::make_selector;
use crate::parse_int;
use eyre::{eyre, Result};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Book {
    title: String,
    upc: String,
    price: String,
    available: u32,
    reviews: u32,
    rating: u8,
}

impl Book {
    const fn new(
        title: String,
        upc: String,
        price: String,
        available: u32,
        reviews: u32,
        rating: u8,
    ) -> Self {
        Self {
            title,
            upc,
            price,
            available,
            reviews,
            rating,
        }
    }
    pub fn from_html(page: &scraper::Html) -> Result<Self> {
        Ok(Self::new(
            Self::extract_title(page)?,
            Self::extract_upc(page)?,
            Self::extract_price(page)?,
            Self::extract_available(page)?,
            Self::extract_reviews(page)?,
            Self::extract_rating(page)?,
        ))
    }

    fn extract_title(book_page: &scraper::Html) -> Result<String> {
        let sel = make_selector("div[class$='product_main'] h1");
        book_page.select(&sel).next().map_or_else(
            || {
                log::warn!("Failed to extract title from book page");
                Err(eyre!("Failed to extract title from book page"))
            },
            |elem| Ok(elem.text().collect()),
        )
    }

    fn extract_upc(book_page: &scraper::Html) -> Result<String> {
        let sel = make_selector("tbody tr:first-of-type td");
        book_page.select(&sel).next().map_or_else(
            || {
                log::warn!("Failed to extract upc from book page");
                Err(eyre!("Failed to extract upc from book page"))
            },
            |elem| Ok(elem.text().collect()),
        )
    }

    fn extract_price(book_page: &scraper::Html) -> Result<String> {
        let sel = make_selector("div[class$='product_main']  p[class^='price']");
        book_page.select(&sel).next().map_or_else(
            || {
                log::warn!("Failed to extract price from book page");
                Err(eyre!("Failed to extract price from book page"))
            },
            |elem| Ok(elem.text().collect()),
        )
    }

    fn extract_available(book_page: &scraper::Html) -> Result<u32> {
        let sel = make_selector("div[class$='product_main'] p[class^='instock']");
        let text = if let Some(elem) = book_page.select(&sel).next() {
            elem.text().collect::<String>()
        } else {
            log::warn!("Failed to availability from book page");
            return Err(eyre!("Failed to availability from book page"));
        };
        Ok(parse_int(&text).unwrap_or(0))
    }

    fn extract_reviews(book_page: &scraper::Html) -> Result<u32> {
        let sel = make_selector("tbody tr:last-of-type td");
        let text: String = match book_page.select(&sel).next() {
            Some(elem) => elem.text().collect(),
            None => return Err(eyre!("Failed to extract title from book page")),
        };
        Ok(parse_int(&text).unwrap_or(0))
    }

    fn extract_rating(book_page: &scraper::Html) -> Result<u8> {
        let ratings: Vec<&str> = vec!["Zero", "One", "Two", "Three", "Four", "Five"];
        let sel = make_selector("div[class$='product_main'] p[class^='star-rating']");
        let rating = match book_page.select(&sel).next() {
            Some(elem) => elem
                .value()
                .attr("class")
                .unwrap_or("")
                .split(' ')
                .last()
                .unwrap_or(""),
            None => return Err(eyre!("Failed to extract rating from book page")),
        };

        match ratings.iter().position(|&s| s == rating) {
            Some(p) => Ok(u8::try_from(p)?),
            None => Err(eyre!("Could not convert text to string")),
        }
    }
}
