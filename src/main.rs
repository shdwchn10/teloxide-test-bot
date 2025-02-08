use teloxide::{
    prelude::*, sugar::request::RequestReplyExt, types::ParseMode, utils::render::Renderer,
    RequestError,
};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    pretty_env_logger::init();

    let bot = Bot::from_env();

    let handler = dptree::entry()
        .inspect(|u: Update| {
            eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_message().branch(Message::filter_text().endpoint(
                |msg: Message, bot: Bot| async move {
                    let render =
                        Renderer::new(msg.text().unwrap(), msg.entities().unwrap_or_default());

                    bot.send_message(msg.chat.id, msg.text().unwrap())
                        .reply_to(msg.id)
                        .entities(msg.entities().unwrap_or_default().to_owned())
                        .await?;
                    bot.send_message(msg.chat.id, render.as_html())
                        .reply_to(msg.id)
                        .parse_mode(ParseMode::Html)
                        .await?;
                    bot.send_message(msg.chat.id, render.as_markdown())
                        .reply_to(msg.id)
                        .parse_mode(ParseMode::MarkdownV2)
                        .await?;

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
