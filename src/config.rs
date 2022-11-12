use std::{path::PathBuf, str::FromStr};

use indexmap::IndexMap;
use midnight::twilight::model::{
    channel::ReactionType,
    id::{
        marker::{ChannelMarker, MessageMarker, RoleMarker},
        Id,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum VecOrSingle {
    Single(Id<RoleMarker>),
    Vec(Vec<Id<RoleMarker>>),
}

pub enum VecOrSingleIter<'a> {
    Single(std::iter::Once<Id<RoleMarker>>),
    Vec(std::slice::Iter<'a, Id<RoleMarker>>),
}

impl VecOrSingle {
    pub fn iter<'a>(&'a self) -> VecOrSingleIter<'a> {
        match self {
            Self::Single(id) => VecOrSingleIter::Single(std::iter::once(*id)),
            Self::Vec(ids) => VecOrSingleIter::Vec(ids.iter()),
        }
    }
}

impl<'a> Iterator for VecOrSingleIter<'a> {
    type Item = Id<RoleMarker>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Single(iter) => iter.next(),
            Self::Vec(iter) => iter.next().cloned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub channel: Id<ChannelMarker>,
    pub message: Id<MessageMarker>,

    #[serde(rename = "reactMap")]
    // Using IndexMap to preserve the order
    pub react_map: IndexMap<String, VecOrSingle>,
}

impl Config {
    pub fn get_roles_for_react(&self, react: &ReactionType) -> Option<&VecOrSingle> {
        match react {
            ReactionType::Unicode { name } => self.react_map.get(name),
            ReactionType::Custom { id, .. } => self.react_map.get(&id.to_string()),
        }
    }
}

pub fn read_config() -> Vec<Config> {
    let config_path = PathBuf::from_str("./config.json").unwrap();
    let config_file = std::fs::read_to_string(config_path).unwrap();
    let config: Vec<Config> = serde_json::from_str(&config_file).unwrap();
    config
}

pub fn get_react_roles<'a>(
    configs: &'a Vec<Config>,
    message_id: Id<MessageMarker>,
    channel_id: Id<ChannelMarker>,
    emoji: &ReactionType,
) -> Option<&'a VecOrSingle> {
    let config = configs
        .iter()
        .find(|conf| conf.message == message_id && conf.channel == channel_id);

    let config = match config {
        Some(conf) => conf,
        None => return None,
    };

    let roles = config.get_roles_for_react(&emoji);

    roles
}
