use std::rc::Rc;

use config::get_react_roles;
use futures::{Future, StreamExt};
use twilight_gateway::{cluster::ShardScheme, Cluster, Event, EventType, Intents};
use twilight_http::request::channel::reaction::RequestReactionType;
use twilight_model::channel::{Channel, ReactionType};

mod config;
mod http;

pub async fn run_socket_event_cluster<
    H: 'static + Fn(Event) -> F,
    F: 'static + Future<Output = ()>,
>(
    token: String,
    handler: H,
) {
    let intents = Intents::GUILDS | Intents::GUILD_MESSAGE_REACTIONS;

    let scheme = ShardScheme::Auto;

    let (cluster, mut events) = Cluster::builder(token, intents)
        .shard_scheme(scheme)
        .build()
        .await
        .expect("Failed to create cluster");

    tokio::spawn(async move {
        cluster.up().await;
    });

    while let Some((id, event)) = events.next().await {
        let epic = match event.kind() {
            EventType::ShardConnected
            | EventType::ShardConnecting
            | EventType::ShardDisconnected
            | EventType::ShardIdentifying
            | EventType::ShardReconnecting
            | EventType::ShardResuming => true,
            _ => false,
        };

        if epic {
            println!("Shard: {}, Event: {:?}", id, event.kind());
        }

        handler(event).await;
    }
}

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
    dotenv::dotenv().ok();

    let configs = config::read_config();

    let token = std::env::var("TOKEN").expect("TOKEN not set");

    let http = http::HttpApi::new(token.clone());

    // Initialize the reactions
    for config in configs.iter() {
        let message = http.get_message(config.channel, config.message).await;

        let channel = http.get_channel(config.channel).await;
        let channel = match channel {
            Channel::Guild(guild) => guild,
            _ => panic!("Channel is not a guild"),
        };

        let guild = http.get_guild(channel.guild_id().unwrap()).await;

        // Add missing reactions
        for (emoji_name, _) in config.react_map.iter() {
            let emoji = http.get_emoji(&guild, emoji_name).await;
            let my_react = message.reactions.iter().find(|e| e.emoji == emoji && e.me);
            if my_react.is_none() {
                http.add_reaction(
                    config.channel,
                    config.message,
                    as_request_reaction_type(&emoji),
                )
                .await;
            }
        }
    }

    let http = Rc::new(http);
    let configs = Rc::new(configs);

    run_socket_event_cluster(token, move |e| {
        let configs = configs.clone();
        let http = http.clone();

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
                            .await;
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
                            .await;
                    }
                }

                _ => {}
            }
        }
    })
    .await;
}
