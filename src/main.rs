use teloxide::{prelude::*, RequestError};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dispatching features bot...");

    let bot = Bot::from_env();

    let handler = Update::filter_channel_post().endpoint(|msg: Message, bot: Bot| async move {
        if !msg.text().unwrap_or_default().starts_with("MessageKind: ") {
            bot.send_message(msg.chat.id, format!("MessageKind: {:#?}", msg.kind))
                .await?;
        }

        Ok::<(), RequestError>(())
    });

    Dispatcher::builder(bot, handler)
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
