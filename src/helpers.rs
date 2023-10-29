pub mod url_helper {
    use article_scraper::ArticleScraper;
    use headless_chrome::{Browser, LaunchOptionsBuilder};
    use regex::Regex;
    // use std::any::type_name;
    use url::Url;

    #[derive(Clone, Default, PartialEq)]
    pub struct News {
        pub title: String,
        pub url: String,
    }

    pub async fn get_news(urls: Vec<String>, kw: Vec<String>) -> Result<Vec<News>, anyhow::Error> {
        let mut result: Vec<News> = vec![];
        for checked_url in urls.iter() {
            let article_scraper = ArticleScraper::new(None).await;
            let url = Url::parse(checked_url).unwrap();
            let client = reqwest::Client::new();
            let article = article_scraper.parse(&url, false, &client, None).await;
            match article {
                Ok(article) => {
                    let title = article.title.unwrap_or_else(|| "not found".to_string());
                    let html_body = article.html.unwrap_or_else(|| "not found".to_string());
                    for word in kw.iter() {
                        if title.contains(word) {
                            result.push(make_news(&title, checked_url));
                        }
                        if html_body.contains(word) {
                            result.push(make_news(&title, checked_url));
                        }
                    }
                }
                Err(..) => {
                    println!("Can't parse url {}", url);
                    continue;
                }
            }
        }

        Ok(result)
    }

    pub fn make_news(title: &str, url: &str) -> News {
        News {
            title: title.to_string(),
            url: url.to_string(),
        }
    }

    pub fn build_url(url_href: &str, url_original: &str) -> std::string::String {
        let mut full_path = format!("{}{}", cut_domain(url_original), url_href);
        if url_href.contains("pnp.ru") {
            full_path = url_href.to_string()
        }
        full_path
    }

    pub fn is_valid_url(url: &str) -> bool {
        let re = Regex::new(r"^((http[s]?|ftp):\/)?\/?([^:\/\s]+)(:([^\/]*))?((\/\w+)*\/)([\w\-\.]+[^#?\s]+)(\?([^#]*))?(#(.*))?$").unwrap();
        let Some(_urls) = re.captures(url) else {return false};
        true
    }

    fn cut_domain(url: &str) -> &str {
        let re =
            Regex::new(r"http(?:s)?:\/\/(?:[\w-]+\.)*([\w-]{1,63})(?:\.(?:\w{3}|\w{2}))(?:$|)")
                .unwrap();
        re.find(url).unwrap().as_str()
    }

    pub fn get_urls(
        ulrs_for_check: Vec<String>,
        already_checked: &[String],
    ) -> Result<Vec<String>, anyhow::Error> {
        let mut urls: Vec<String> = vec![];
        for url in ulrs_for_check.iter() {
            let launch_options = LaunchOptionsBuilder::default()
                .headless(true)
                .sandbox(false)
                .build()
                .unwrap();
            let browser = Browser::new(launch_options).unwrap();
            let tab = browser.new_tab().unwrap();
            tab.set_default_timeout(std::time::Duration::from_secs(120));
            tab.navigate_to(url)?.wait_until_navigated()?;
            let html_body = tab.wait_for_element("body").unwrap().get_content().unwrap();
            let document = scraper::Html::parse_document(&html_body);

            for node in document.tree.nodes() {
                if node.value().is_text() {
                    for anc in node.ancestors() {
                        if anc.value().is_element() {
                            if let Some(url_path) = anc.value().as_element().unwrap().attr("href") {
                                let full_news_path = build_url(url_path, url);
                                if is_has_ban_word(url_path) {
                                    continue;
                                }

                                if is_valid_url(&full_news_path)
                                    && !already_checked.contains(&full_news_path)
                                    && !urls.contains(&full_news_path)
                                {
                                    urls.push(full_news_path.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(urls)
    }

    fn is_has_ban_word(url: &str) -> bool {
        let ban_words: Vec<&str> = vec![
            "https://",
            "tel:+",
            "mailto:",
            "utm_source=",
            "/archive",
            "/info/",
            "/copyright",
            "/info/",
            "http://",
            "/tass-today",
            "/press",
            "/contacts",
            "/career",
            "/ads",
            "/pravila-citirovaniya",
            "/recommend",
            "/daily/",
            "/authors/",
            "/rubric/",
            "/rusfond.ru",
            "?from=tag",
            "?from=logo",
            "#",
            "/lk/",
            "/LK/",
            "/ad",
            "/regions",
            "/fm/player",
            "/t.me/",
            "/vk.com/",
            "/companynews",
            "//",
            "/politika-obrabotki-personalnyh-dannyh",
        ];
        for bw in ban_words.iter() {
            if url.contains(bw) {
                return true;
            }
        }
        false
    }
    // fn type_of<T>(_: T) -> &'static str {
    //     type_name::<T>()
    // }
    //
    //
    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_cut_captures_url() {
            let urls: Vec<(&str, &str)> = [
                (
                    "https://www.kommersant.ru/theme/941",
                    "https://www.kommersant.ru",
                ),
                ("https://tass.ru/ekologiya", "https://tass.ru"),
                ("https://www.vedomosti.ru/esg", "https://www.vedomosti.ru"),
                (
                    "https://www.vedomosti.ru/ecology",
                    "https://www.vedomosti.ru",
                ),
                ("https://www.pnp.ru/social/", "https://www.pnp.ru"),
            ]
            .to_vec();
            for url in urls.iter() {
                assert_eq!(cut_domain(url.0), url.1)
            }
        }

        #[test]
        fn test_build_correct_vedomosti() {
            let url_original = "https://www.vedomosti.ru/ecology";
            let url_href = "/ecology/regulation/columns/2023/05/25/976859-vremya-ubirat-za-soboi";
            let expected_url = "https://www.vedomosti.ru/ecology/regulation/columns/2023/05/25/976859-vremya-ubirat-za-soboi";

            let full_path = build_url(url_href, url_original);

            assert_eq!(full_path, expected_url);
        }

        #[test]
        fn test_build_pnp() {
            let url_original = "https://www.pnp.ru/social/";
            let url_href = "https://www.pnp.ru/social/v-rospotrebnadzore-otkryli-goryachuyu-liniyu-po-uslugam-taksi-i-karsheringa.html";
            let expected_url = "https://www.pnp.ru/social/v-rospotrebnadzore-otkryli-goryachuyu-liniyu-po-uslugam-taksi-i-karsheringa.html";

            let full_path = build_url(url_href, url_original);

            assert_eq!(full_path, expected_url);
        }

        #[test]
        fn test_regexp_good() {
            let good_url = "https://www.vedomosti.ru/ecology";
            assert!(is_valid_url(good_url));
        }
        #[test]
        fn test_regexp_bad() {
            let bad_urls: Vec<&str> = vec![
                "https://www.vedomosti.rutel:+74959563458",
                "https://www.vedomosti.ruhttps://flipboard.com/@vedomosti",
            ];
            for uri in bad_urls.iter() {
                assert!(!is_valid_url(uri));
            }
        }

        #[test]
        fn test_is_has_ban_word() {
            let bad_urls: Vec<&str> = vec!["tel:+74959563458", "https://flipboard.com/@vedomosti"];

            for uri in bad_urls.iter() {
                assert!(is_has_ban_word(uri));
            }
        }
    }
}
