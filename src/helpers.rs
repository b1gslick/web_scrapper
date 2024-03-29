pub mod url_helper {
    use article_scraper::ArticleScraper;
    use headless_chrome::{Browser, LaunchOptionsBuilder};
    use regex::Regex;
    // use std::any::type_name;
    use serde::{Deserialize, Serialize};
    use std::{
        env,
        fs::File,
        io::{self, BufRead, BufReader},
    };
    use url::Url;

    #[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
    pub struct News {
        pub title: String,
        pub url: String,
    }

    pub async fn get_news(
        checked_url: &str,
        kw: Vec<String>,
    ) -> Result<News, crate::types::error::Error> {
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
                        return Ok(make_news(&title, checked_url));
                    }
                    if html_body.contains(word) {
                        return Ok(make_news(&title, checked_url));
                    }
                }
                Err(crate::types::error::Error::NewNotFound)
            }
            Err(error) => {
                println!("Can't parse url {} with \n{:?}", url, error);
                Err(crate::types::error::Error::ParseUrl)
            }
        }
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
        if url.contains("www.pnp.ru") {
            if url.contains(".html") {
                return true;
            }
            return false;
        }
        let re = Regex::new(r"^((http[s]?|ftp):\/)?\/?([^:\/\s]+)(:([^\/]*))?((\/\w+)*\/)([\w\-\.]+[^#?\s]+)(\?([^#]*))?(#(.*))?$").unwrap();
        let Some(_urls) = re.captures(url) else {
            return false;
        };
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
            match Browser::new(launch_options) {
                Ok(nb) => {
                    let tab = nb.new_tab()?;
                    tab.navigate_to(url)?.wait_until_navigated()?;
                    let html_body = tab.wait_for_element("body").unwrap().get_content().unwrap();
                    let document = scraper::Html::parse_document(&html_body);

                    for node in document.tree.nodes() {
                        if node.value().is_text() {
                            for anc in node.ancestors() {
                                if anc.value().is_element() {
                                    if let Some(url_path) =
                                        anc.value().as_element().unwrap().attr("href")
                                    {
                                        let full_news_path = build_url(url_path, url);
                                        if is_has_ban_word(url_path)
                                            && !url_path.contains("www.pnp.ru")
                                        {
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
                Err(err) => {
                    log::error!("{}", err);
                    continue;
                }
            }
        }
        Ok(urls)
    }

    fn is_has_ban_word(url: &str) -> bool {
        let ban_words: Vec<String> =
            lines_from_file("black_words.txt").expect("Could not load lines");
        for bw in ban_words.iter() {
            if url.contains(bw) {
                return true;
            }
        }
        false
    }
    pub fn lines_from_file(filename: &str) -> io::Result<Vec<String>> {
        let mut cur_dir = env::current_dir()?;
        cur_dir.push("src");
        cur_dir.push("config");
        cur_dir.push(filename);
        BufReader::new(File::open(cur_dir)?).lines().collect()
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
