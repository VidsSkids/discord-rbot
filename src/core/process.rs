//! Handle the connection with discord and it's events.
use crate::database;
use log::error;
use rand;
use serenity::{
    cache, http,
    model::channel::Message,
    model::id::{ChannelId, UserId},
    prelude::*,
};
use std::{str::FromStr, sync::Arc};

use super::commands::{
    CallBackParams, ATTACKED, COMMANDS_LIST, CONTAIN_MSG_LIST, CONTAIN_REACTION_LIST, TAG_MSG_LIST,
};

lazy_static! {
    pub static ref HTTP_STATIC: RwLock<Option<Arc<http::Http>>> = RwLock::new(None);
    pub static ref CACHE: RwLock<cache::CacheRwLock> = RwLock::new(cache::CacheRwLock::default());
}

fn allowed_channel(
    command_channel: Option<ChannelId>,
    message_channel: ChannelId,
    ctx: &Context,
) -> bool {
    match command_channel {
        Some(ref chan) => {
            if chan != &message_channel {
                message_channel
                    .say(&ctx.http, format!("I am not allowed to issue this command in this channel ! Use {} instead.", chan.mention())).unwrap();
                return false;
            }
            true
        }
        None => true,
    }
}

fn allowed_user(expected: database::Role, user: &database::User) -> bool {
    let role = match database::Role::from_str(&*user.role) {
        Err(e) => {
            println!("Error {}", e);
            return false;
        }
        Ok(role) => (role),
    };

    role as u32 >= expected as u32
}

pub fn process_command(message_split: &[&str], message: &Message, ctx: &Context) -> bool {
    for (key, command) in COMMANDS_LIST.iter() {
        if *key == message_split[0] && allowed_channel(command.channel, message.channel_id, ctx) {
            {
                let db_instance = database::INSTANCE.read().unwrap();
                let user: &database::User = db_instance
                    .user_search(*message.author.id.as_u64())
                    .unwrap();
                if !allowed_user(command.permission, &user) {
                    message
                        .channel_id
                        .send_message(&ctx.http, |m| {
                            m.content(format!(
                                "You({}) are not allowed to run this command",
                                user.role
                            ))
                        })
                        .unwrap();
                    return true;
                }
            }
            // We remove default arguments: author and command name from the total
            let arguments = message_split.len() - 2;
            let result = if arguments >= command.argument_min && arguments <= command.argument_max {
                let params = CallBackParams {
                    args: message_split,
                    message,
                    context: ctx,
                };
                (command.exec)(params)
            } else {
                Ok(Some(format!("Usage: {}", command.usage.clone())))
            };

            match result {
                Ok(option) => match option {
                    Some(reply) => {
                        message.reply(&ctx.http, reply).unwrap();
                    }
                    None => {}
                },
                Err(err) => {
                    message
                        .reply(&ctx.http, "Bipboop this is broken <@173013989180178432>")
                        .unwrap();
                    error!("Command Error: {} => {}", key, err);
                }
            }
            return true;
        }
    }
    false
}

pub fn process_tag_msg(message_split: &[&str], message: &Message, ctx: &Context) -> bool {
    for (key, reaction) in TAG_MSG_LIST.iter() {
        if *key == message_split[0] {
            message.channel_id.say(&ctx.http, reaction).unwrap();
            return true;
        }
    }
    false
}

pub fn process_contains(message: &Message, ctx: &Context) {
    for (key, text) in CONTAIN_MSG_LIST.iter() {
        if message.content.contains(key) {
            message.channel_id.say(&ctx.http, *text).unwrap();
        }
    }

    for (key, reaction) in CONTAIN_REACTION_LIST.iter() {
        if message.content.contains(key) {
            message.react(ctx, *reaction).unwrap();
        }
    }
}

pub fn split_args(input: &str) -> Vec<&str> {
    let mut count = 0;
    let message_split_quote: Vec<&str> = input.split('"').collect();
    let mut result: Vec<&str> = Vec::new();
    for msg in message_split_quote {
        if msg.is_empty() {
            continue;
        }
        count += 1;
        if (count % 2) == 0 {
            result.push(msg);
        } else {
            let mut message_split_space: Vec<&str> = msg.trim().split(' ').collect();
            result.append(&mut message_split_space);
        }
    }
    result
}

const CATS: [&str; 12] = [
    "😺", "😸", "😹", "😻", "😼", "😽", "🙀", "😿", "😾", "🐈", "🐁", "🐭",
];
const KEYS: [&str; 8] = ["🔑", "🗝", "🔏", "🔐", "🔒", "🔓", "🖱", "👓"];

const HERDINGCHATTE: ChannelId = ChannelId(570275817804791809);
const CYBERGOD: ChannelId = ChannelId(588666452849065994);
const TESTBOT: ChannelId = ChannelId(555206410619584519);
/// Anoying other channels
pub fn annoy_channel(ctx: &Context, message: &Message) {
    if message.channel_id == HERDINGCHATTE {
        let random_active = rand::random::<usize>() % 10;
        if random_active == 0 {
            let random_icon = rand::random::<usize>() % CATS.len();
            message.react(ctx, CATS[random_icon]).unwrap();
        }
    }
    if message.channel_id == CYBERGOD {
        let random_active = rand::random::<usize>() % 10;
        if random_active == 0 {
            let random_icon = rand::random::<usize>() % KEYS.len();
            message.react(ctx, KEYS[random_icon]).unwrap();
        }
    }
    if message.channel_id == TESTBOT {
        let random_active = rand::random::<usize>() % 10;
        if random_active == 0 {
            let random_icon = rand::random::<usize>() % KEYS.len();
            message.react(ctx, KEYS[random_icon]).unwrap();
        }
    }
}

const FILTERED: [&str; 1] = ["🔥"];
const PM: UserId = UserId(365228504817729539);
pub fn filter_outannoying_messages(ctx: &Context, message: &Message) {
    if message.author.id != PM {
        return;
    }
    for annoying in FILTERED.iter() {
        if message.content.replace(annoying, "").trim().is_empty() {
            println!("Has been filtered !");
            let _ = message.delete(ctx);
        }
    }
}

pub fn personal_attack(ctx: &Context, message: &Message) {
    if message.author.mention() == *ATTACKED.read() {
        const ANNOYING: [&str; 11] = [
            "🐧", "💩", "🍌", "💣", "👾", "🐔", "📛", "🔥", "‼", "⚡", "⚠",
        ];
        let random1 = rand::random::<usize>() % ANNOYING.len();
        let random2 = rand::random::<usize>() % ANNOYING.len();
        message.react(ctx, ANNOYING[random1]).unwrap();
        message.react(ctx, ANNOYING[random2]).unwrap();
    }
}

pub fn database_update(message: &Message) {
    let mut db_instance = database::INSTANCE.write().unwrap();
    let author_id = *message.author.id.as_u64() as i64;
    if !db_instance.users.iter().any(|e| e.discordid == author_id) {
        db_instance.user_add(author_id, &*database::Role::Guest.to_string());
    }
    db_instance.message_add(
        *message.id.as_u64() as i64,
        author_id,
        &message.content,
        *message.channel_id.as_u64() as i64,
    );
}
