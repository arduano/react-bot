use std::rc::Rc;

use config::get_react_roles;
use midnight::{
    client::DiscordClient,
    twilight::{
        gateway::{Event, Intents},
        http::request::channel::reaction::RequestReactionType,
        model::{
            channel::ReactionType,
            guild::Guild,
            id::{marker::EmojiMarker, Id},
        },
    },
};

mod config;

fn as_request_reaction_type<'a>(react: &'a ReactionType) -> RequestReactionType<'a> {
    match react {
        ReactionType::Custom { name, id, .. } => RequestReactionType::Custom {
            name: name.as_deref(),
            id: *id,
        },
        ReactionType::Unicode { name } => RequestReactionType::Unicode { name: &name },
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    run().await
}

async fn run() {
    dotenv::dotenv().ok();

    let configs = config::read_config();

    let token = std::env::var("TOKEN").expect("TOKEN not set");

    let http = DiscordClient::new(token.clone());

    println!("Initializing reactions...");

    // Initialize the reactions
    for config in configs.iter() {
        let message = http.message(config.channel, config.message).await.unwrap();

        let channel = http.channel(config.channel).await.unwrap();
        let guild_id = channel.guild_id.expect("Channel is not a guild");

        let guild = http.guild(guild_id).await.unwrap();

        // Add missing reactions
        for (emoji_name, _) in config.react_map.iter() {
            let emoji = get_emoji(&guild, emoji_name);
            let my_react = message.reactions.iter().find(|e| e.emoji == emoji && e.me);
            if my_react.is_none() {
                http.add_reaction(
                    config.channel,
                    config.message,
                    &as_request_reaction_type(&emoji),
                )
                .await
                .unwrap();
            }
        }
    }

    let http = Rc::new(http);
    let configs = Rc::new(configs);

    println!("Initialized, starting the gateway cluster...");

    let intents = Intents::GUILDS | Intents::GUILD_MESSAGE_REACTIONS;
    midnight::run_discord_event_loop_or_panic(token, intents, move |e| {
        let configs = configs.clone();
        let http = http.clone();

        match e {
            Event::ShardConnected(_)
            | Event::ShardConnecting(_)
            | Event::ShardDisconnected(_)
            | Event::ShardIdentifying(_)
            | Event::ShardReconnecting(_)
            | Event::ShardResuming(_) => println!("Event: {:?}", e.kind()),
            _ => {}
        }

        async move {
            match e {
                Event::ReactionAdd(react) => {
                    let roles =
                        get_react_roles(&configs, react.message_id, react.channel_id, &react.emoji);

                    let roles = match roles {
                        Some(roles) => roles,
                        None => return,
                    };

                    for role in roles.iter() {
                        http.add_role(react.guild_id.unwrap(), react.user_id, role)
                            .await
                            .map_err(|err| {
                                println!("Error adding role: {:?}", err);
                            })
                            .ok();
                    }
                }
                Event::ReactionRemove(react) => {
                    let roles =
                        get_react_roles(&configs, react.message_id, react.channel_id, &react.emoji);

                    let roles = match roles {
                        Some(roles) => roles,
                        None => return,
                    };

                    for role in roles.iter() {
                        http.remove_role(react.guild_id.unwrap(), react.user_id, role)
                            .await
                            .map_err(|err| {
                                println!("Error removing role: {:?}", err);
                            })
                            .ok();
                    }
                }

                _ => {}
            }
        }
    })
    .await;
}

pub fn get_emoji(guild: &Guild, emoji_name_or_id: &str) -> ReactionType {
    match emoji_name_or_id.parse::<Id<EmojiMarker>>() {
        Ok(id) => {
            // Custom emoji
            let guild_emoji = guild.emojis.iter().find(|emoji| emoji.id == id);
            let emoji = match guild_emoji {
                Some(emoji) => emoji,
                None => panic!("Emoji {id} not found"),
            };
            ReactionType::Custom {
                animated: emoji.animated,
                id,
                name: Some(emoji.name.clone()),
            }
        }
        Err(_) => {
            // Unicode emoji
            ReactionType::Unicode {
                name: emoji_name_or_id.to_string(),
            }
        }
    }
}
