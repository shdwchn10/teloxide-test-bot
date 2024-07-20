use teloxide::{
    prelude::*,
    types::{
        ChatBoostRemoved, ChatBoostUpdated, LinkPreviewOptions, MessageEntityKind,
        MessageReactionCountUpdated, MessageReactionUpdated, ReactionType, ReactionTypeKind,
        ReplyParameters,
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
        .branch(Update::filter_chat_boost().endpoint(
            |upd: ChatBoostUpdated, bot: Bot| async move {
                bot.send_message(upd.chat.id, format!("ChatBoostUpdated: {:#?}", upd))
                    .await?;
                Ok(())
            },
        ))
        .branch(Update::filter_removed_chat_boost().endpoint(
            |upd: ChatBoostRemoved, bot: Bot| async move {
                bot.send_message(upd.chat.id, format!("ChatBoostRemoved: {:#?}", upd))
                    .await?;
                Ok(())
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
                                .reply_parameters(ReplyParameters {
                                    message_id: msg.id,
                                    chat_id: None,
                                    allow_sending_without_reply: None,
                                    quote: None,
                                })
                                .await?;
                            Ok(())
                        }
                        Commands::Boosts { user_id } => {
                            bot.send_message(
                                msg.chat.id,
                                format!(
                                    "User `{user_id}` boosts: {:?}",
                                    bot.get_user_chat_boosts(msg.chat.id, UserId(user_id))
                                        .await?
                                ),
                            )
                            .reply_parameters(ReplyParameters {
                                message_id: msg.id,
                                chat_id: None,
                                allow_sending_without_reply: None,
                                quote: None,
                            })
                            .await?;
                            Ok(())
                        }
                    }
                }),
        )
        .branch(
            Update::filter_message()
                .branch(
                    Message::filter_story().endpoint(|msg: Message, bot: Bot| async move {
                        bot.send_message(
                            msg.chat.id,
                            format!("I hate stories! Story: {:#?}", msg.story().unwrap()),
                        )
                        .reply_parameters(ReplyParameters {
                            message_id: msg.id,
                            chat_id: None,
                            allow_sending_without_reply: None,
                            quote: None,
                        })
                        .await?;
                        Ok(())
                    }),
                )
                .branch(Message::filter_reply_to_story().endpoint(
                    |msg: Message, bot: Bot| async move {
                        bot.send_message(
                            msg.chat.id,
                            format!("Reply to Story: {:#?}", msg.reply_to_story().unwrap()),
                        )
                        .await?;
                        Ok(())
                    },
                ))
                .branch(Message::filter_boost_added().endpoint(
                    |msg: Message, bot: Bot| async move {
                        bot.send_message(
                            msg.chat.id,
                            format!("Chat Boost Added: {:#?}", msg.boost_added().unwrap()),
                        )
                        .await?;
                        Ok(())
                    },
                ))
                .branch(
                    dptree::filter(|cfg: ConfigParameters, msg: Message| {
                        msg.from
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
                                MaintainerCommands::Tba71 => {
                                    bot.send_message(
                                        msg.chat.id,
                                        format!(
                                            "unrestrict_boost_count: {:?}, custom_emoji_sticker_set_name: {:?}",
                                            bot.get_chat(msg.chat.id).await?.unrestrict_boost_count(), bot.get_chat(msg.chat.id).await?.custom_emoji_sticker_set_name()
                                        ),
                                    )
                                    .reply_parameters(ReplyParameters {
                                        message_id: msg.id,
                                        chat_id: None,
                                        allow_sending_without_reply: None,
                                        quote: None,
                                    })
                                    .await?;
                                    Ok(())
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
                        .reply_parameters(ReplyParameters {
                            message_id: msg.id,
                            chat_id: None,
                            allow_sending_without_reply: None,
                            quote: None,
                        })
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
                )
                .branch(
                    dptree::filter(|msg: Message| {
                        msg.sender_boost_count.map(|c| c > 0).unwrap_or(false)
                    })
                    .endpoint(|msg: Message, bot: Bot| async move {
                        bot.send_message(
                            msg.chat.id,
                            format!(
                                "This user gave us {} boosts!",
                                msg.sender_boost_count.unwrap()
                            ),
                        )
                        .reply_parameters(ReplyParameters {
                            message_id: msg.id,
                            chat_id: None,
                            allow_sending_without_reply: None,
                            quote: None,
                        })
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
