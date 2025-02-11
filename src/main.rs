use teloxide::{prelude::*, utils::command::BotCommands, RequestError};

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
            Update::filter_message().branch(
                dptree::filter(|cfg: ConfigParameters, msg: Message| {
                    msg.from()
                        .map(|user| user.id == cfg.bot_maintainer)
                        .unwrap_or_default()
                })
                .filter_command::<MaintainerCommands>()
                .endpoint(
                    |msg: Message, bot: Bot, cmd: MaintainerCommands| async move {
                        match cmd {
                            MaintainerCommands::Delete => {
                                match msg.reply_to_message() {
                                    Some(msg) => {
                                        let res = bot.delete_message(msg.chat.id, msg.id).await;

                                        if let Err(e) = res {
                                            bot.send_message(msg.chat.id, format!("{e:#?}"))
                                                .await?;
                                        }
                                    }
                                    None => {
                                        bot.send_message(
                                            msg.chat.id,
                                            "Reply to message which you want to delete!",
                                        )
                                        .await?;
                                    }
                                }

                                Ok::<(), RequestError>(())
                            }
                        }
                    },
                ),
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
    Delete,
}
