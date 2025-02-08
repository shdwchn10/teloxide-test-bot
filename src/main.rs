use teloxide::{
    prelude::*,
    sugar::{bot::BotMessagesExt, request::RequestReplyExt},
    types::{
        ChatBoostRemoved, ChatBoostUpdated, InputFile, LinkPreviewOptions, MessageEntityKind,
        MessageReactionCountUpdated, MessageReactionUpdated, ParseMode, ReactionType,
    },
    utils::{command::BotCommands, render::Renderer},
    RequestError,
};

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();

    pretty_env_logger::init();

    let bot = Bot::from_env();

    let parameters = ConfigParameters {
        bot_maintainer: UserId(1459074222),
    };

    let handler = dptree::entry()
        .inspect(|u: Update| {
            eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|msg: Message| msg.effect_id().is_some()).endpoint(
                        |msg: Message, bot: Bot| async move {
                            let effect = msg.effect_id().unwrap();
                            bot.send_message(msg.chat.id, format!("Effect: '{}'", effect))
                                .reply_to(msg.id)
                                .message_effect_id(effect)
                                .await?;
                            Ok::<(), RequestError>(())
                        },
                    ),
                )
                .branch(
                    dptree::filter(|msg: Message| {
                        msg.show_caption_above_media() && msg.photo().is_some()
                    })
                    .endpoint(|msg: Message, bot: Bot| async move {
                        let photo = msg.photo().unwrap();
                        bot.send_photo(msg.chat.id, InputFile::file_id(photo[0].file.id.clone()))
                            .caption(msg.caption().unwrap_or_default())
                            .show_caption_above_media(msg.show_caption_above_media())
                            .reply_to(msg.id)
                            .await?;
                        Ok(())
                    }),
                )
                .branch(
                    Message::filter_text().endpoint(|msg: Message, bot: Bot| async move {
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

                        Ok(())
                    }),
                ),
        );

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![parameters])
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

#[derive(Clone)]
struct ConfigParameters {
    bot_maintainer: UserId,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum MaintainerCommands {
    #[command(parse_with = "split")]
    Rights {
        user_id: u64,
    },
    Tba71,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Commands {
    Reactions,
    #[command(parse_with = "split")]
    Boosts {
        user_id: u64,
    },
}
