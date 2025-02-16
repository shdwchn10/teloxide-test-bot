use teloxide::{
    adaptors::{throttle::Limits, trace::Settings, CacheMe, Throttle, Trace},
    prelude::*,
    sugar::bot::BotMessagesExt,
    RequestError,
};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    pretty_env_logger::init();

    let bot: Trace<Throttle<CacheMe<Bot>>> = Bot::from_env()
        .cache_me()
        .throttle(Limits::default())
        .trace(Settings::all());

    let handler = dptree::entry()
        .inspect(|u: Update| {
            eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_message().branch(Message::filter_text().endpoint(
                |msg: Message, bot: Trace<Throttle<CacheMe<Bot>>>| async move {
                    let user_id = if let Ok(user_id) = msg.text().unwrap_or_default().parse::<u64>()
                    {
                        UserId(user_id)
                    } else {
                        return Ok(());
                    };

                    if bot.is_user_deactivated(user_id).await {
                        bot.send_message(msg.chat.id, "User is deactivated!")
                            .await?;
                    }

                    Ok::<(), RequestError>(())
                },
            )),
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
