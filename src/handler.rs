use std::path::PathBuf;
use std::{error::Error, sync::Arc};

use crate::chat_server::ChatServer;
use serde::Deserialize;
use teloxide::{
    prelude::*,
    types::{InputFile, ParseMode},
    utils::command::BotCommands,
};

#[derive(BotCommands, PartialEq, Debug)]
#[command(rename_rule = "lowercase", description = "*Описание команд*")]
enum Command {
    #[command(description = "Начало диалога")]
    Start,
    #[command(description = "Активация бота\\.\
    \n`/key \\<Ключ активации\\>`")]
    Key(String),
}

#[derive(Deserialize)]
pub struct Json {
    href: String,
}

pub async fn handle(
    bot: Bot,
    msg: Message,
    cs: Arc<ChatServer>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let text = match msg.text() {
        Some(text) => text,
        None => return Ok(()),
    };

    let mut response = String::from("");

    if let Ok(command) = Command::parse(text, cs.bot_name.as_str()) {
        response = match command {
            Command::Start => {
                let mut str = String::from("Команда не доступна в группе");
                if msg.chat.id.0 > 0i64 {
                    str = Command::descriptions().to_string();
                }
                str
            }

            Command::Key(key) => {
                let mut str;
                if cs.access_is_allowed(&msg.chat.id.to_string()).await? {
                    str = String::from("Доступ разрешен");
                } else {
                    if key == cs.access_key {
                        let username = msg.chat.username().unwrap_or_else(|| "");
                        let first_name = msg.chat.first_name().unwrap_or_else(|| "");
                        let last_name = msg.chat.last_name().unwrap_or_else(|| "");

                        cs.add_new_user(
                            msg.chat.id.to_string(),
                            &username,
                            &first_name,
                            &last_name,
                        )
                        .await?;
                        let admin_user_id: UserId = UserId(cs.admin_id);
                        let admin_chat_id: ChatId = ChatId::from(admin_user_id);
                        str = format!("Новый пользователь: {username}, {first_name}, {last_name}");
                        bot.send_message(admin_chat_id, str).await?;

                        str = String::from("Доступ разрешен");
                    } else {
                        str = String::from("Доступ разрешен");
                    }
                }
                str
            }
        }
    } else {
        if cs.access_is_allowed(&msg.chat.id.to_string()).await? {
            let paths = cs.get_paths(text).await?;

            let keyboard = cs.make_keyboard(paths).await?;
            bot.send_message(msg.chat.id, "Найденные обработки:")
                .reply_markup(keyboard)
                .await?;
        } else {
            bot.send_message(msg.chat.id, "Доступ запрешен").await?;
        }
    }

    if !response.is_empty() {
        bot.send_message(msg.chat.id, response.replace("_", "\\_"))
            .parse_mode(ParseMode::MarkdownV2)
            .await?;
    }

    Ok(())
}

pub async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    cs: Arc<ChatServer>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let is_file = cs.get_file_path(&q.data.clone().unwrap().as_str()).await?;
    let chat = q.message.clone().unwrap().chat().clone();

    let url = format!(
        "https://cloud-api.yandex.net/v1/disk/resources/download?path=disk:{}{}",
        cs.root_dir, is_file.path
    );

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", &cs.yauth)
        .send()
        .await?;

    let json: Json = response.json().await.unwrap_or_else(|_e| Json {
        href: "".to_string(),
    });

    let name = &is_file.name.as_str().replace("/", "_");
    let tmp_path = format!("{}/{}", cs.tmp_dir, name);
    match cs.download_file(json.href.as_str(), &tmp_path).await {
        Ok(_) => log::info!("File saved successfully"),
        Err(e) => log::info!("Error while downloading file: {}", e),
    }

    let input_file: InputFile = InputFile::file(PathBuf::from(&tmp_path));

    bot.send_document(chat.id.clone(), input_file).await?;

    match cs.delete_file(tmp_path.as_str()).await {
        Ok(_) => log::info!("File removed successfully"),
        Err(e) => log::info!("Error while removing file: {}", e),
    }

    let username = &chat.username().unwrap_or_else(|| "");
    let first_name = &chat.first_name().unwrap_or_else(|| "");
    let last_name = &chat.last_name().unwrap_or_else(|| "");

    cs.log_user_activity(
        &is_file.path,
        &chat.id.to_string(),
        username,
        first_name,
        last_name,
    )
    .await
    .expect("Не удалось записать выполнить логирование");

    let text: String = String::from("Файл обработки");
    bot.answer_callback_query(&q.id).await?;

    if !&q.id.is_empty() {
        bot.edit_message_text(chat.id, q.clone().message.unwrap().id(), text)
            .await?;
    } else if let Some(id) = &q.inline_message_id {
        bot.edit_message_text_inline(id, text).await?;
    }

    log::info!("Uploaded file: {}", is_file.path);

    Ok(())
}
