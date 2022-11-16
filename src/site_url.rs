use eyre::Result;

use url::{ParseError, Url};

#[must_use]
pub fn build_book_page_url(path: &str) -> Result<Url, ParseError> {
    log::trace!("building book page url with path: {}", path);
    build_books_toscrape_url("catalogue/")?.join(path.trim_start_matches("../"))
}

#[must_use]
pub fn build_books_toscrape_url(path: &str) -> Result<Url, ParseError> {
    const HOMEPAGE: &str = "https://books.toscrape.com/";

    log::trace!("Building url with path: {}", path);
    Url::parse(HOMEPAGE)?.join(path)
}

#[must_use]
pub fn build_catelogue_url(page: u32) -> Url {
    #[allow(clippy::expect_used)]
    build_books_toscrape_url(&format!("catalogue/page-{}.html", page))
        .expect("Any u32 should parse correctly.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_catelogue_url() -> Result<(), ParseError> {
        assert_eq!(
            "https://books.toscrape.com/catalogue/page-1.html",
            build_catelogue_url(1).to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/catalogue/page-2.html",
            build_catelogue_url(2).to_string()
        );
        assert_eq!(
            "https://books.toscrape.com/catalogue/page-3.html",
            build_catelogue_url(3).to_string()
        );
        Ok(())
    }

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
}
