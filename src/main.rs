use teloxide::{prelude::*, utils::command::BotCommands, RequestError};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dispatching features bot...");

    let bot = Bot::from_env();

    let parameters = ConfigParameters {
        bot_maintainer: UserId(1459074222),
    };

    let handler = Update::filter_message().branch(
        // Filter a maintainer by a user ID.
        dptree::filter(|cfg: ConfigParameters, msg: Message| {
            msg.from()
                .map(|user| user.id == cfg.bot_maintainer)
                .unwrap_or_default()
        })
        .filter_command::<Commands>()
        .endpoint(|msg: Message, bot: Bot, cmd: Commands| async move {
            match cmd {
                Commands::Rights { user_id } => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "Rights of {user_id}: {:#?}",
                            bot.get_chat_member(msg.chat.id, UserId(user_id))
                                .await?
                                .kind
                        ),
                    )
                    .await?;
                    Ok::<(), RequestError>(())
                }
            }
        }),
    );

    Dispatcher::builder(bot, handler)
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

/// Maintainer commands
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Commands {
    /// Generate a number within range
    #[command(parse_with = "split")]
    Rights { user_id: u64 },
}
