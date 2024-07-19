use teloxide::{
    prelude::*,
    types::{
        LinkPreviewOptions, MessageEntityKind, MessageReactionCountUpdated, MessageReactionUpdated,
        ReactionType, ReactionTypeKind,
    },
    utils::command::BotCommands,
    RequestError,
};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let bot = Bot::from_env();

    let parameters = ConfigParameters {
        bot_maintainer: UserId(1459074222),
    };

    let handler = dptree::entry()
        .inspect(|u: Update| {
            eprintln!("{u:#?}"); // Print the update to the console with inspect
        })
        .branch(Update::filter_message_reaction_updated().endpoint(
            |msg: MessageReactionUpdated, bot: Bot| async move {
                bot.send_message(msg.chat.id, format!("MessageReactionUpdated: {:#?}", msg))
                    .await?;
                Ok::<(), RequestError>(())
            },
        ))
        .branch(Update::filter_message_reaction_count_updated().endpoint(
            |msg: MessageReactionCountUpdated, bot: Bot| async move {
                bot.send_message(
                    msg.chat.id,
                    format!("MessageReactionCountUpdated: {:#?}", msg),
                )
                .await?;
                Ok::<(), RequestError>(())
            },
        ))
        .branch(
            Update::filter_channel_post()
                .filter_command::<Commands>()
                .endpoint(|msg: Message, bot: Bot, cmd: Commands| async move {
                    match cmd {
                        Commands::Reactions => {
                            let text = match bot.get_chat(msg.chat.id).await?.available_reactions {
                                Some(r) => format!("Available reactions: {r:?}"),
                                None => "All reactions are available".to_owned(),
                            };

                            bot.send_message(msg.chat.id, text)
                                .reply_to_message_id(msg.id)
                                .await?;
                            Ok(())
                        }
                    }
                }),
        )
        .branch(
            Update::filter_message()
                .branch(
                    dptree::filter(|cfg: ConfigParameters, msg: Message| {
                        msg.from()
                            .map(|user| user.id == cfg.bot_maintainer)
                            .unwrap_or_default()
                    })
                    .filter_command::<MaintainerCommands>()
                    .endpoint(
                        |msg: Message, bot: Bot, cmd: MaintainerCommands| async move {
                            match cmd {
                                MaintainerCommands::Rights { user_id } => {
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
                        },
                    ),
                )
                .branch(
                    dptree::filter(|msg: Message| {
                        let Some(text) = msg.text() else { return false };
                        text.contains('🌭')
                    })
                    .endpoint(|msg: Message, bot: Bot| async move {
                        bot.set_message_reaction(msg.chat.id, msg.id)
                            .reaction(vec![ReactionType {
                                kind: ReactionTypeKind::Emoji {
                                    emoji: '🌭'.into()
                                },
                            }])
                            .is_big(true)
                            .await?;
                        Ok(())
                    }),
                )
                .branch(
                    dptree::filter(|msg: Message| {
                        let Some(entities) = msg.entities() else {
                            return false;
                        };
                        entities.iter().any(|e| {
                            matches!(
                                e.kind,
                                MessageEntityKind::Url | MessageEntityKind::TextLink { .. }
                            )
                        })
                    })
                    .endpoint(|msg: Message, bot: Bot| async move {
                        let link_preview_options = msg.link_preview_options();
                        bot.send_message(
                            msg.chat.id,
                            format!("LinkPreviewOptions: {link_preview_options:#?}"),
                        )
                        .reply_to_message_id(msg.id)
                        .link_preview_options(
                            link_preview_options
                                .unwrap_or(&LinkPreviewOptions {
                                    is_disabled: None,
                                    url: None,
                                    prefer_small_media: None,
                                    prefer_large_media: None,
                                    show_above_text: None,
                                })
                                .clone(),
                        )
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
    Rights { user_id: u64 },
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Commands {
    Reactions,
}
