use std::sync::Arc;

use teloxide::{
    prelude::*,
    types::{InlineQueryResult, InlineQueryResultCachedPhoto},
    RequestError,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    pretty_env_logger::init();

    let bot = Bot::from_env();

    let file_ids: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

    let handler = dptree::entry()
        .branch(
            Update::filter_message().branch(Message::filter_photo().endpoint(
                |msg: Message, file_ids: Arc<Mutex<Vec<String>>>| async move {
                    let mut file_ids = file_ids.lock().await;

                    file_ids.push(msg.photo().unwrap().first().unwrap().file.id.clone());

                    Ok::<(), RequestError>(())
                },
            )),
        )
        .branch(Update::filter_inline_query().endpoint(
            |bot: Bot, query: InlineQuery, file_ids: Arc<Mutex<Vec<String>>>| async move {
                let file_ids = file_ids.lock().await;

                bot.answer_inline_query(
                    query.id,
                    file_ids.iter().enumerate().map(|(i, photo_file_id)| {
                        InlineQueryResult::CachedPhoto(InlineQueryResultCachedPhoto::new(
                            i.to_string(),
                            photo_file_id,
                        ))
                    }),
                )
                .await?;

                Ok::<(), RequestError>(())
            },
        ));

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![file_ids])
        .default_handler(|upd| async move {
            log::warn!("Unhandled update: {:?}", upd);
        })
        .error_handler(LoggingErrorHandler::with_custom_text(
            "An error has occurred in the dispatcher",
        ))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
