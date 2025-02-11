use teloxide::{dispatching::UpdateHandlerTracingExt, prelude::*, RequestError};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .instrument_with(|update: Update| {
                    tracing::info_span!(
                        "Handle update",
                        username = update.from().and_then(|user| user.username.clone())
                    )
                })
                .branch(
                    Message::filter_text().endpoint(|msg: Message, bot: Bot| async move {
                        bot.send_message(msg.chat.id, msg.text().unwrap()).await?;

                        Ok::<(), RequestError>(())
                    }),
                ),
        );

    Dispatcher::builder(bot.clone(), handler)
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
