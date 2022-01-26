use twilight_http::{request::channel::reaction::RequestReactionType, Client};
use twilight_model::{
    channel::{Channel, Message, ReactionType},
    guild::Guild,
    id::{
        marker::{ChannelMarker, EmojiMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
        Id,
    },
};

pub struct HttpApi {
    client: Client,
}

impl HttpApi {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(token),
        }
    }

    pub async fn get_message(
        &self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
    ) -> Message {
        self.client
            .message(channel_id, message_id)
            .exec()
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
    }

    pub async fn get_guild(&self, guild_id: Id<GuildMarker>) -> Guild {
        self.client
            .guild(guild_id)
            .exec()
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
    }

    pub async fn get_channel(&self, channel_id: Id<ChannelMarker>) -> Channel {
        self.client
            .channel(channel_id)
            .exec()
            .await
            .unwrap()
            .model()
            .await
            .unwrap()
    }

    pub async fn add_reaction<'a>(
        &self,
        channel_id: Id<ChannelMarker>,
        message_id: Id<MessageMarker>,
        reaction: RequestReactionType<'a>,
    ) {
        self.client
            .create_reaction(channel_id, message_id, &reaction)
            .exec()
            .await
            .unwrap();
    }

    pub async fn add_role<'a>(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) {
        self.client
            .add_guild_member_role(guild_id, user_id, role_id)
            .exec()
            .await
            .unwrap();
    }

    pub async fn remove_role<'a>(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) {
        self.client
            .remove_guild_member_role(guild_id, user_id, role_id)
            .exec()
            .await
            .unwrap();
    }

    pub async fn get_emoji(&self, guild: &Guild, emoji_name_or_id: &str) -> ReactionType {
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
}
