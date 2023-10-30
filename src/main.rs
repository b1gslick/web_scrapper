use mobot::*;

#[derive(Clone, Default, BotState)]
struct Options {
    links: Vec<String>,
    key_words: Vec<String>,
    old_news: Vec<News>,
}

#[derive(Clone, Default, PartialEq)]
struct News {
    title: String,
    url: String,
}

async fn help(_e: Event, _state: State<Options>) -> Result<Action, anyhow::Error> {
    Ok(Action::ReplyText(format!(
        "\n
        ===============================================\n
        Hello! For use me need to provide some options:\n
        /add - add url, you can print it with ',' separator\n
        /key_words - add key word for scrapping text into urls, also can use ',' for provide several word\n
        /urls - check all urls which use for scrapping\n
        /kw - check all words for scrapping\n
        /del_url url - delete current url\n
        /del_w word - delete word from key_words\n
        /scan - scan urls with key words\n
    ===============================================\n
        "
    )))
}

async fn handle_any(e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    match e.update {
        Update::Message(message) => {
            let text = message.clone().text.unwrap();
            if text.contains("/add") {
                let mut state = state.get().write().await;
                let striped_text = text.replace("/add", "");
                let urls: Vec<String> = striped_text
                    .trim()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                for url in urls.iter() {
                    state.links.push(url.to_string());
                }
                Ok(Action::ReplyText(format!("Added: {}", striped_text)))
            } else if text.contains("/key_words") {
                let mut state = state.get().write().await;
                let striped_text = text.replace("/key_words", "");
                let key_words: Vec<String> = striped_text
                    .trim()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                for kew_word in key_words.iter() {
                    state.key_words.push(kew_word.to_string());
                }
                Ok(Action::ReplyText(format!("Added: {}", striped_text)))
            } else {
                Ok(Action::Done)
            }
        }
        Update::EditedMessage(message) => Ok(Action::ReplyText(format!(
            "This function not impelemted yet {}",
            message.text.unwrap()
        ))),
        _ => {
            unreachable!()
        }
    }
}

async fn check_urls(_e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    let state = state.get().read().await;
    Ok(Action::ReplyText(format!("Got urls: {:?}", state.links)))
}

async fn check_key_words(_e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    let state = state.get().read().await;
    Ok(Action::ReplyText(format!(
        "Got words: {:?}",
        state.key_words
    )))
}

async fn delete(e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;
    let message = e.update.get_message_or_post()?.clone();
    let text = message.clone().text.unwrap();
    if text.contains("/del_url") {
        let striped_text = text.replace("/del_url", "");
        let index = state
            .links
            .iter()
            .position(|x| *x == striped_text.trim())
            .unwrap();
        state.links.remove(index);
        Ok(Action::ReplyText(format!(
            "Deleted: {}, \n New urls list is: {:?}",
            message.text.unwrap(),
            state.links
        )))
    } else if text.contains("/del_w") {
        let striped_text = text.replace("/del_w", "");
        let index = state
            .key_words
            .iter()
            .position(|x| *x == striped_text.trim())
            .unwrap();
        state.key_words.remove(index);
        Ok(Action::ReplyText(format!(
            "Deleted: {}, \n New words list is: {:?}",
            message.text.unwrap(),
            state.key_words
        )))
    } else {
        unreachable!()
    }
}

async fn scan(e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    let mut state = state.get().write().await;
    let mut result_news: Vec<News> = vec![];
    for url in state.links.clone().iter() {
        let browser = headless_chrome::Browser::default().unwrap();
        let tab = browser.new_tab().unwrap();

        tab.navigate_to(url)?.wait_until_navigated()?;

        let html_body = tab.wait_for_element("body").unwrap().get_content().unwrap();

        let document = scraper::Html::parse_document(&html_body);

        for node in document.tree.nodes() {
            if node.value().is_text() {
                for kw in state.clone().key_words.iter() {
                    if node.value().as_text().unwrap().contains(kw)
                        && !node.value().as_text().unwrap().contains("img")
                    {
                        if node.value().as_text().unwrap().len() > 300 {
                            continue;
                        }
                        for anc in node.ancestors().into_iter() {
                            if anc.value().is_element() {
                                let url_path = anc.value().as_element().unwrap().attr("href");
                                if url_path.is_some() {
                                    let striped_url = url_path.unwrap().replace("esg/", "");
                                    let path_url = url.replace("/ekologiya", "");
                                    let full_path = format!("{}{}", path_url, striped_url);
                                    let news = News {
                                        title: node.value().as_text().unwrap().trim().to_string(),
                                        url: full_path.to_string(),
                                    };
                                    if !result_news.contains(&news)
                                        && !state.old_news.contains(&news)
                                    {
                                        result_news.push(news.clone());
                                        state.old_news.push(news.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    for news in result_news.iter() {
        e.send_message(format!("{}: {}", news.title, news.url))
            .await?;
    }
    Ok(Action::Done)
}

#[tokio::main]
async fn main() {
    let client = Client::new(std::env::var("TOKEN").unwrap());

    Router::new(client)
        .add_route(Route::Message(Matcher::Prefix("/urls".into())), check_urls)
        .add_route(
            Route::Message(Matcher::Prefix("/kw".into())),
            check_key_words,
        )
        .add_route(Route::Message(Matcher::Prefix("/del_url".into())), delete)
        .add_route(Route::Message(Matcher::Prefix("/del_w".into())), delete)
        .add_route(Route::Message(Matcher::Prefix("/help".into())), help)
        .add_route(Route::Message(Matcher::Prefix("/scan".into())), scan)
        .add_route(Route::Message(Matcher::Any), handle_any)
        .add_route(Route::EditedMessage(Matcher::Any), handle_any)
        .add_route(Route::Default, handlers::log_handler)
        .start()
        .await;
}
