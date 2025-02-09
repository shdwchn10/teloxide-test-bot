use teloxide::{
    adaptors::{throttle::Limits, DefaultParseMode, Throttle},
    prelude::*,
    sugar::request::RequestReplyExt,
    types::ParseMode,
    RequestError,
};

type MyBot = DefaultParseMode<Throttle<Bot>>;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    pretty_env_logger::init();

    let bot: MyBot = Bot::from_env()
        .throttle(Limits::default())
        .parse_mode(ParseMode::Html);

    let handler = dptree::entry()
        .inspect(|u: Update| {
            eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_message().endpoint(|msg: Message, bot: MyBot| async move {
                bot.send_message(msg.chat.id, "Test")
                    .reply_to(msg.id)
                    .await?;
                Ok::<(), RequestError>(())
            }),
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
