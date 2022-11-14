use futures::{stream, StreamExt};
use std::num::ParseIntError;

use env_logger::Env;
use eyre::{eyre, Result};
use log;
use url::{ParseError, Url};

#[derive(Debug)]
#[allow(dead_code)]
struct Book {
    title: String,
    upc: String,
    price: String,
    available: u32,
    reviews: u32,
    rating: u8,
}

impl Book {
    pub const fn new(
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
        match book_page.select(&sel).next() {
            Some(elem) => Ok(elem.text().collect()),
            None => {
                log::warn!("Failed to extract title from book page");
                Err(eyre!("Failed to extract title from book page"))
            }
        }
    }

    fn extract_upc(book_page: &scraper::Html) -> Result<String> {
        let sel = make_selector("tbody tr:first-of-type td");
        match book_page.select(&sel).next() {
            Some(elem) => Ok(elem.text().collect()),
            None => {
                log::warn!("Failed to extract upc from book page");
                Err(eyre!("Failed to extract upc from book page"))
            }
        }
    }

    fn extract_price(book_page: &scraper::Html) -> Result<String> {
        let sel = make_selector("div[class$='product_main']  p[class^='price']");
        match book_page.select(&sel).next() {
            Some(elem) => Ok(elem.text().collect()),
            None => {
                log::warn!("Failed to extract price from book page");
                Err(eyre!("Failed to extract price from book page"))
            }
        }
    }

    fn extract_available(book_page: &scraper::Html) -> Result<u32> {
        let sel = make_selector("div[class$='product_main'] p[class^='instock']");
        let text: String = match book_page.select(&sel).next() {
            Some(elem) => elem.text().collect(),
            None => {
                log::warn!("Failed to availability from book page");
                return Err(eyre!("Failed to availability from book page"));
            }
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
            Some(p) => Ok(p as u8),
            None => Err(eyre!("Could not convert text to string")),
        }
    }
}

fn make_selector(selector: &str) -> scraper::Selector {
    scraper::Selector::parse(selector).unwrap()
}

fn parse_int(input: &str) -> Result<u32, ParseIntError> {
    log::debug!("Attempting to parse input {}", input);
    input
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse::<u32>()
}

async fn get_html(url: Url) -> Result<scraper::Html> {
    log::debug!("Requesting resource at url {}", url);
    let resp = reqwest::get(url).await?;
    log::debug!("Parsing result from url: {}", resp.url());
    let resp_text = resp.text().await?;
    Ok(scraper::Html::parse_document(&resp_text))
}

fn get_category_paths(page: &scraper::Html) -> Vec<&str> {
    let category_elements = scraper::Selector::parse("ul.nav-list ul a[href]");
    match category_elements {
        Ok(elems) => page
            .select(&elems)
            .filter_map(move |d| d.value().attr("href"))
            .collect(),
        Err(_) => Vec::new(),
    }
}

fn build_book_page_url(path: &str) -> Result<Url, ParseError> {
    log::trace!("Building book page url with path: {}", path);
    build_books_toscrape_url("catalogue/")?.join(path.trim_start_matches("../"))
}

fn build_books_toscrape_url(path: &str) -> Result<Url, ParseError> {
    log::trace!("Building url with path: {}", path);
    const HOMEPAGE: &str = "https://books.toscrape.com/";
    Url::parse(HOMEPAGE)?.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_books_toscrape_url() -> Result<(), ParseError> {
        assert_eq!(
            "https://books.toscrape.com/a",
            build_books_toscrape_url("a")?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/abc",
            build_books_toscrape_url("abc")?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/multiple/paths",
            build_books_toscrape_url("multiple/paths")?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/multiple/paths",
            build_books_toscrape_url("/multiple/paths")?.to_string()
        );
        Ok(())
    }

    #[test]
    fn test_build_book_page_url() -> Result<(), ParseError> {
        assert_eq!(
            "https://books.toscrape.com/catalogue/its-only-the-himalayas_981/index.html",
            build_book_page_url("../../../its-only-the-himalayas_981/index.html")?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/catalogue/full-moon-over-noahs-ark-an-odyssey-to-mount-ararat-and-beyond_811/index.html",
            build_book_page_url(
            "../../../full-moon-over-noahs-ark-an-odyssey-to-mount-ararat-and-beyond_811/index.html"
            )?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/catalogue/a",
            build_book_page_url("a")?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/catalogue/abc",
            build_book_page_url("abc")?.to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/catalogue/multiple/paths",
            build_book_page_url("multiple/paths")?.to_string()
        );
        Ok(())
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(Ok(19), parse_int("In stock (19 available)"));
        assert_eq!(Ok(0), parse_int("Out of stock (0 available)"));
        assert!(parse_int("Out of stock").is_err());
        assert!(parse_int("In stock ( available)").is_err());
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let book_url_selector = make_selector("article.product_pod a[title]");

    let document = get_html(build_books_toscrape_url("").expect("Getting base url must be valid"))
        .await
        .unwrap_or_else(|error| panic!("Problem requesting home page {:?}", error));

    let category_paths = get_category_paths(&document);
    log::debug!("Category Paths: {category_paths:?}");
    let category_urls = category_paths
        .iter()
        .filter_map(|x| build_books_toscrape_url(x).ok());
    log::debug!("Category urls: {category_urls:?}");

    let pages: Vec<scraper::Html> = stream::iter(category_urls)
        .filter_map(|url| async {
            log::info!("Getting category page {url}");
            get_html(url).await.ok()
        })
        .collect()
        .await;

    let book_urls = pages
        .iter()
        .flat_map(|page| page.select(&book_url_selector))
        .filter_map(move |d| d.value().attr("href"))
        .filter_map(|x| build_book_page_url(x).ok());

    let books = stream::iter(book_urls)
        .filter_map(|url| async {
            log::info!("Getting book page at {url}");
            get_html(url).await.ok()
        })
        .map(|page| Book::from_html(&page))
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
