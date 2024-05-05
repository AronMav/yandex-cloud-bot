use std::env::var;
use std::sync::Arc;

use teloxide::prelude::*;

use crate::{chat_server::ChatServer,
            handler::handle,
            handler::callback_handler};

mod chat_server;
mod db;
pub mod handler;

#[tokio::main]
async fn main() {
    let log_path = var("LOG_PATH").expect("LOG_PATH is not set");
    log4rs::init_file(log_path, Default::default()).unwrap();
    run().await;
}

async fn run() {
    log::info!("Starting yandex-cloud-bot");

    let bot = Bot::from_env();
    let chat_server = Arc::new(ChatServer::new(
        var("DB_PATH").expect("$DB_PATH is not set"),
        var("BOT_NAME").expect("$BOT_NAME is not set"),
        var("YAUTH").expect("$YAUTH is not set"),
        var("ACCESS_KEY").expect("$ACCESS_KEY is not set"),
        var("ADMIN_ID").expect("$ADMIN_ID is not set")
            .parse().expect("$ADMIN_ID is not u64"),
        var("ROOT_DIR").expect("$ROOT_DIR is not set"),
        var("TMP_DIR").expect("$TMP_DIR is not set"),
    ));

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(handle))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![chat_server])
        .build()
        .dispatch()
        .await;

    log::info!("Closing bot... Goodbye!");
}
