use std::time::Duration;

use teloxide::{
    prelude::*,
    types::{ChatMemberUpdated, InputPollOption, LivePeriod, ParseMode},
    utils::command::BotCommands,
    RequestError,
};
use tokio::time;

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
        .branch(Update::filter_chat_member().endpoint(
            |msg: ChatMemberUpdated, bot: Bot| async move {
                bot.send_message(msg.chat.id, format!("ChatMemberUpdated: {:#?}", msg))
                    .await?;
                Ok::<(), RequestError>(())
            },
        ))
        .branch(
            Update::filter_message()
                .branch(
                    Message::filter_location().endpoint(|msg: Message, bot: Bot| async move {
                        bot.send_message(
                            msg.chat.id,
                            format!("Location: {:#?}", msg.location().unwrap()),
                        )
                        .await?;
                        Ok::<(), RequestError>(())
                    }),
                )
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
                                MaintainerCommands::Location => {
                                    let loc_id = bot
                                        .send_location(
                                            msg.chat.id,
                                            38.950504885601845,
                                            -77.1457524495,
                                        )
                                        .live_period(LivePeriod::from_u32(60))
                                        .await?
                                        .id;

                                    time::sleep(Duration::from_secs(3)).await;

                                    bot.edit_message_live_location(
                                        msg.chat.id,
                                        loc_id,
                                        39.108889,
                                        -76.771389,
                                    )
                                    .await?;

                                    Ok::<(), RequestError>(())
                                }
                                MaintainerCommands::GetMax => {
                                    bot.send_message(
                                        msg.chat.id,
                                        format!(
                                            "Chat max_reaction_count: {:#?}",
                                            bot.get_chat(msg.chat.id).await?.max_reaction_count
                                        ),
                                    )
                                    .await?;
                                    Ok(())
                                }
                            }
                        },
                    ),
                )
                .branch(
                    Message::filter_poll().endpoint(|msg: Message, bot: Bot| async move {
                        let poll = msg.poll().unwrap();
                        let input_poll_options = poll.options.iter().map(|opt| InputPollOption {
                            text: format!("cloned: {}", opt.text),
                            text_parse_mode: Some(ParseMode::Html),
                            text_entities: opt.text_entities.clone(),
                        });
                        bot.send_poll(msg.chat.id, poll.question.clone(), input_poll_options)
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
    Location,
    GetMax,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Commands {}
