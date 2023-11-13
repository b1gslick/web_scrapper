use mobot::*;

use crate::helpers::url_helper::get_news;
use crate::helpers::url_helper::get_urls;
use crate::helpers::url_helper::News;

pub mod helpers;


#[derive(Clone, Default, BotState)]
struct Options {
    links: Vec<String>,
    key_words: Vec<String>,
    old_news: Vec<News>,
    already_checked: Vec<String>,
}

async fn start_ecology(_e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    let initial_url: Vec<String> = std::env::var("URLS")
        .unwrap()
        .split(',')
        .map(str::to_string)
        .collect();
    let initial_key_words: Vec<String> = std::env::var("KW")
        .unwrap()
        .split(',')
        .map(str::to_string)
        .collect();
    let mut state = state.get().write().await;
    state.links = initial_url.clone();
    state.key_words = initial_key_words.clone();
    Ok(Action::ReplyText(format!(
        "\n
        Привет загружена стартовая конфигурация для новостей по экологии!\n
        Сайты для поиска: {:?}\n
        Ключевые слова: {:?}\n
    ",
        &initial_url, &initial_key_words
    )))
}

async fn help(_e: Event, _state: State<Options>) -> Result<Action, anyhow::Error> {
    Ok(Action::ReplyText(
        "\n
        ===============================================\n
        Привет! Я бот который поможет тебе искать новости на сайтах по ключевым словам \n
        Для того что бы начать со мной работать, можно выбрать уже готовые настройки   \n
        или настроить поиск самостоятельно, я еще маленький и только учуь поэтому по всем n\
        ошибкам можно писать сюда @tim1106\n\n\n
          Для того что бы начать работу со мной, нужно выполнить несколько команд вот они:\n
        /start - вызовет эту команду \n
        /help - вызовет эту команду \n
        /eco - начать работу с готовыми настройками для новостей по экологии\n
        /add - добавить новый сайт, так же можно добавлять несколько используя разделитель ','\n
        /key_words - добавить ключевое слово для поиска, можно задать несколько используя разделитель ','\n
        /scan - Начать поиск новостей\n
        /urls - посмотреть уже добавленные сайты\n
        /kw - посмотреть используемые ключевые слова\n
        /del_url url - удалить сайт, пожалуйста введите полный урл\n
        /del_w word - удалить ключевое слово\n
    ===============================================\n
        ".to_string()
    ))
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
    e.send_message("Начинаю сканирование новостей").await?;
    let mut state = state.get().write().await;
    match get_urls(state.links.clone(), &state.already_checked) {
        Ok(urls_for_check) => {
            for for_check_url in urls_for_check.iter() {
                state.already_checked.push(for_check_url.to_string())
            }

            let result_news: Vec<News> = get_news(urls_for_check.clone(), state.key_words.clone())
                .await
                .unwrap();

            for news in result_news.iter() {
                if !state.old_news.contains(news) {
                    state.old_news.push(news.clone());
                    e.send_message(format!("{}: {}", news.title, news.url))
                        .await?;
                }
            }
            e.send_message("Сканироание завершено!").await?;
            if result_news.is_empty() {
                e.send_message("Новых новостей не найдено, пожалуйста попробуйте позже.")
                    .await?;
            }
        }
        Err(urls_for_check) => {
            e.send_message("Сканироание завершено c ошибкой, пожалуйста повторите попытку.")
                .await?;
            println! {"Catch error {:?}", urls_for_check}
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
    env_logger::init();

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
        .add_route(Route::Message(Matcher::Prefix("/start".into())), help)
        .add_route(
            Route::Message(Matcher::Prefix("/eco".into())),
            start_ecology,
        )
        .add_route(Route::Message(Matcher::Prefix("/scan".into())), scan)
        .add_route(Route::Message(Matcher::Any), handle_any)
        .add_route(Route::EditedMessage(Matcher::Any), handle_any)
        .add_route(Route::Default, handlers::log_handler)
        .start()
        .await;
}
