use futures::future;
use futures::{stream, StreamExt};

use env_logger::Env;

use crate::book::Book;
use crate::site_url::{build_book_page_url, build_catelogue_url};
use eyre::{eyre, Result};
use std::num::ParseIntError;
use url::Url;

pub mod book;
pub mod site_url;

#[must_use]
pub fn make_selector(selector: &str) -> scraper::Selector {
    scraper::Selector::parse(selector).expect("Invalid selector string provided")
}

#[must_use]
pub fn parse_int(input: &str) -> Result<u32, ParseIntError> {
    log::debug!("Attempting to parse input {}", input);
    input
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<u32>()
}

#[cfg(test)]
mod tests {
    use super::parse_int;

    #[test]
    fn test_parse_int() {
        assert_eq!(Ok(19), parse_int("In stock (19 available)"));
        assert_eq!(Ok(0), parse_int("Out of stock (0 available)"));
        assert!(parse_int("Out of stock").is_err());
        assert!(parse_int("In stock ( available)").is_err());
    }
}

pub async fn get_page(url: Url) -> Result<String> {
    log::info!("Making GET request to: {}", url);
    let resp = reqwest::get(url).await?;
    if !resp.status().is_success() {
        return Err(eyre!(
            "Received non success status code: {}",
            resp.status().as_u16()
        ));
    }
    Ok(resp.text().await?)
}

pub async fn get_html(url: Url) -> Result<scraper::Html> {
    let resp_text = get_page(url).await?;
    Ok(scraper::Html::parse_document(&resp_text))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let book_url_selector = make_selector("article.product_pod a[title]");

    // We can just iterate through the pages on the catelogue pages.
    // will generate sequential urls and stop consuming when we get a 404
    let pages: Vec<scraper::Html> = stream::iter(1..)
        .map(build_catelogue_url)
        .map(get_html)
        .buffered(10)
        .take_while(|page| future::ready(page.is_ok()))
        .map(std::result::Result::unwrap)
        .collect()
        .await;

    let book_urls = pages
        .iter()
        .flat_map(|page| page.select(&book_url_selector))
        .filter_map(|d| build_book_page_url(d.value().attr("href")?).ok());

    let books = stream::iter(book_urls)
        .map(get_html)
        .buffer_unordered(10)
        .map(|page| Book::from_html(&page?))
        .collect::<Vec<Result<Book>>>()
        .await;
    for book in &books {
        match book {
            Ok(book) => log::info!("{book:?}"),
            Err(_) => (),
        }
    }
    log::info!("Number of books scraped: {}", books.len());
    Ok(())
}
