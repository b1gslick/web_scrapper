use mobot::*;
use scraper::{Html, Selector};

#[derive(Clone, Default, BotState)]
struct Options {
    links: Vec<String>,
    key_words: Vec<String>,
}

async fn help(e: Event, _state: State<Options>) -> Result<Action, anyhow::Error> {
    let message = e.update.get_message_or_post()?.clone();
    Ok(Action::ReplyText(format!(
        "You were type not correct message {}",
        message.text.unwrap()
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
                println!("{:?}", striped_text);

                for url in urls.iter() {
                    println!("{}", url);
                    state.links.push(url.to_string());
                    println!("{:?}", state.links)
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
                    println!("{}", kew_word);
                    state.key_words.push(kew_word.to_string());
                    println!("{:?}", state.key_words)
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
    let state = state.get().read().await;
    for url in state.links.iter() {
        let response = reqwest::blocking::get(url).unwrap().text().unwrap();

        let doc_body = Html::parse_document(&response);

        let title = Selector::parse(".titleline").unwrap();

        for title in doc_body.select(&title) {
            let titles = title.text().collect::<Vec<_>>();
            println!("{}", titles[0])
        }
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
        .add_route(Route::Message(Matcher::Any), handle_any)
        .add_route(Route::EditedMessage(Matcher::Any), handle_any)
        .add_route(Route::Message(Matcher::Prefix("/scan".into())), scan)
        .add_route(Route::Default, handlers::log_handler)
        .start()
        .await;
}
