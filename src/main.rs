use mobot::*;

use crate::db::databse_mod::get_old_data;
use crate::db::databse_mod::write_data;
use crate::db::databse_mod::DbOptions;
use crate::helpers::url_helper::get_news;
use crate::helpers::url_helper::get_urls;
use crate::helpers::url_helper::lines_from_file;
use crate::helpers::url_helper::News;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

pub mod db;
pub mod helpers;
pub mod types;

#[derive(Clone, Default, BotState, Serialize, Deserialize, Debug)]
pub struct Options {
    pub name: String,
    pub links: Vec<String>,
    pub key_words: Vec<String>,
    pub old_news: Vec<News>,
    pub already_checked: Vec<String>,
    pub db_options: DbOptions,
}

async fn start_ecology(_e: Event, state: State<Options>) -> Result<Action, anyhow::Error> {
    let initial_url: Vec<String> = lines_from_file("urls.txt").expect("Can't read urls file");
    let initial_key_words: Vec<String> = lines_from_file("kw.txt").expect("Can't read kw file");
    let mut state = state.get().write().await;
    state.links = initial_url.clone();
    state.key_words = initial_key_words.clone();

    write_to_db(state.clone()).await;
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
                if striped_text.is_empty() {
                    return Ok(Action::ReplyText("Вы добавили пустой урл".to_string()));
                }

                let urls: Vec<String> = striped_text
                    .trim()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();

                for url in urls.iter() {
                    state.links.push(url.to_string());
                }
                write_to_db(state.clone()).await;
                Ok(Action::ReplyText(format!("Added: {}", striped_text)))
            } else if text.contains("/key_words") {
                let mut state = state.get().write().await;
                let striped_text = text.replace("/key_words", "");
                if striped_text.is_empty() {
                    return Ok(Action::ReplyText("Вы добавили пустое слово".to_string()));
                }
                let key_words: Vec<String> = striped_text
                    .trim()
                    .split(',')
                    .map(|s| s.replace(['"', '\\'], ""))
                    .collect();

                for kew_word in key_words.iter() {
                    state.key_words.push(kew_word.to_string());
                }
                write_to_db(state.clone()).await;
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
        write_to_db(state.clone()).await;
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
        write_to_db(state.clone()).await;

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
    if state.links.is_empty() {
        e.send_message(
            "Список сайтов для поиска пустой... 
            \nчто бы добавить сайты нужно ввести команду /add <url>
            \n для более детальной информации введите /help",
        )
        .await?;
        return Ok(Action::Done);
    };
    if state.key_words.is_empty() {
        e.send_message(
            "Список ключевых слов для поиска пустой... 
            \nчто бы добавить новые слова нужно ввести команду /key_words <word>
            \n для более детальной информации введите /help",
        )
        .await?;
        return Ok(Action::Done);
    }
    e.send_message("Начинаю поиск новостей...").await?;
    match get_urls(state.links.clone(), &state.already_checked) {
        Ok(urls_for_check) => {
            for url in urls_for_check {
                match get_news(&url, state.key_words.clone()).await {
                    Ok(news) => {
                        state.already_checked.push(url.to_string());
                        if !state.old_news.contains(&news) {
                            state.old_news.push(news.clone());
                            e.send_message(format!("{}: {}", news.title, news.url))
                                .await?;
                        }
                    }
                    Err(error) => {
                        log::error! {"Catch error for url {}\n {:?}",url, error}
                    }
                }
            }
            e.send_message("поиск завершен").await?;
        }
        Err(_) => {
            e.send_message("Поиск завершен c ошибкой, пожалуйста повторите попытку.")
                .await?;
        }
    }
    write_to_db(state.clone()).await;

    Ok(Action::Done)
}

async fn write_to_db(state: Options) {
    let state_to_db = Options {
        name: "eco".to_string(),
        links: state.links.clone(),
        key_words: state.key_words.clone(),
        old_news: state.old_news.clone(),
        already_checked: state.already_checked.clone(),
        db_options: state.db_options.clone(),
    };
    let result = write_data(state_to_db, state.db_options.clone()).await;
    log::info!("Write to database {:?}", result);
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv().ok();

    let client = Client::new(std::env::var("TOKEN").unwrap());
    let db_name = std::env::var("DB_NAME").unwrap_or("web-finder".to_string());
    let collection_name = std::env::var("COL_NAME").unwrap_or("options".to_string());
    let db_options = DbOptions {
        db_name,
        collection_name,
    };
    let result = get_old_data(db_options.clone()).await;
    log::info!("{:?}", result);
    let loaded_state = match result {
        Ok(state) => state,
        Err(_) => Options {
            name: "ecology".to_string(),
            links: vec![],
            key_words: vec![],
            old_news: vec![],
            already_checked: vec![],
            db_options: db_options.clone(),
        },
    };
    log::info!("Loaded state from db: {:?}", loaded_state);

    Router::new(client)
        .with_state(loaded_state)
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
