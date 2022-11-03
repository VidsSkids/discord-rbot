use crate::{
  core::commands::{CallBackParams, CallbackReturn},
  database::{NewEvent, INSTANCE},
};
use chrono::{prelude::*, Duration};
use chrono_tz::{Europe::Paris, Tz};
use log::info;
use procedural_macros::command;
use regex::{Captures, Regex};
use serenity::{
  http,
  model::{
    id::{ChannelId, UserId},
    prelude::{Message, Reaction},
  },
  prelude::{Context, Mentionable},
};
use std::sync::{Arc, Mutex};
use std::{thread, time};

#[derive(Clone)]
struct RemindMeSnooze {
  pub author: u64,
  pub content: String,
  pub channel: u64,
  pub message_id: u64,
}

lazy_static! {
  static ref TIME_INPUT_REGEX: Regex = Regex::new(
    r#"^([0-9]{1,4})((m(inutes?)?)|(h(ours?)?)|(d(ays?)?(([0-9]{2})[:h]([0-9]{2})?)?))$"#
  )
  .expect("unable to create regex");
  static ref SNOOZE: Mutex<Vec<RemindMeSnooze>> = Mutex::new(Vec::new());
  static ref SNOOZE_MESSAGE: Mutex<Vec<Message>> = Mutex::new(Vec::new());
}

fn extract_date(captures: &Captures) -> Option<DateTime<Tz>> {
  let number = captures
    .get(1)
    .unwrap()
    .as_str()
    .parse::<u16>()
    .expect("unable to parse number value from regex");

  let mut trigger_date: Option<DateTime<_>> = None;
  // Using paris time so we convert correctly when setting hours or minutes
  // Paris.with_hour(10) => NaiveDateTime.hour == 8 because of Tz +2
  let now_paris = Paris.from_utc_datetime(&Utc::now().naive_utc());
  for i in [3, 5, 7] {
    if captures.get(i).is_some() {
      match i {
        3 => {
          trigger_date = Some(
            now_paris
              .checked_add_signed(Duration::minutes(number.into()))
              .unwrap(),
          );
        }
        5 => {
          trigger_date = Some(now_paris + Duration::hours(number.into()));
        }
        7 => {
          if let Some(hours) = captures.get(10) {
            let hours = hours.as_str().parse().expect("unable to parse hours");
            let minutes = captures.get(11).map_or(0, |c| c.as_str().parse().unwrap());
            trigger_date = Some(
              now_paris
                .with_hour(hours)
                .unwrap()
                .with_minute(minutes)
                .unwrap()
                + Duration::days(number.into()),
            );
          } else {
            trigger_date = Some(now_paris + Duration::days(number.into()));
          }
        }
        _ => panic!("captures matches missing case"),
      }
      break;
    }
  }
  trigger_date
}

#[command]
pub async fn remind_me(params: CallBackParams) -> CallbackReturn {
  let input_date = &params.args[1];

  if let Some(captures) = TIME_INPUT_REGEX.captures(input_date) {
    let trigger_date: Option<DateTime<_>> = extract_date(&captures);
    if trigger_date.is_none() {
      return Ok(Some("missing time denominator".to_string()));
    }
    let content = &params.args[2];
    if content.len() > 1900 {
      return Ok(Some("Your message is too long".to_string()));
    }
    let mut db_instance = INSTANCE.write().unwrap();
    db_instance.event_add(NewEvent {
      author: params.message.author.id.0 as i64,
      channel: params.message.channel_id.0 as i64,
      content,
      trigger_date: trigger_date.unwrap().naive_utc(),
    });
    Ok(Some(":ok:".to_string()))
  } else {
    Ok(Some("the time parameter is invalid".to_string()))
  }
}

const SLEEP_TIME_SECS: u64 = 60;
/// Every X seconds check if an event should be sent
pub async fn check_events_loop(http: Arc<http::Http>) {
  info!("running events loop");
  loop {
    let events = {
      let db_instance = INSTANCE.read().unwrap();
      db_instance.events.clone()
    };
    // Here we do not take Paris time as it's already stored as Utc in the database
    let now = Utc::now().naive_utc();
    for event in events {
      let time_since_trigger = now - event.trigger_date;
      let event_id = event.id;

      if time_since_trigger > Duration::seconds(0) {
        let http_clone = http.clone();
        // I don't known why i need to do this
        // The other threads just seem to die if i don't spawn here (the bot even disconnect)
        // And it needs awaiting because other wise when there multiple spawn only one is executed
        let spawn_result = tokio::spawn(async move {
          let message = ChannelId(event.channel as u64)
            .say(
              &http_clone,
              format!(
                "{} {}",
                UserId(event.author as u64).mention(),
                event.content
              ),
            )
            .await
            .expect("unable to send event");
          message.react(&http_clone, '⌚').await.unwrap();
        })
        .await;
        if let Err(e) = spawn_result {
          error!("error spawning event: {}", e);
        }
        {
          let mut db_instance = INSTANCE.write().unwrap();
          db_instance.event_delete(event_id);
        }
      }
    }

    thread::sleep(time::Duration::from_secs(SLEEP_TIME_SECS))
  }
}

pub async fn snooze_reaction(ctx: &Context, reaction: &Reaction, emoji: &str) {
  if emoji == "⌚" {
    let message = reaction.message(&ctx).await.unwrap();
    let reaction_author = reaction.user(&ctx).await.unwrap().id;
    let message_author = message.author.id;
    let channel = message.channel_id;
    let content = message.content.clone();
    if content.starts_with(&format!("<@{}>", reaction_author.0))
      && message_author.0 == ctx.cache.current_user_id().0
    {
      {
        let mut snooze = SNOOZE.lock().unwrap();
        if snooze.iter().cloned().any(|s| s.message_id == message.id.0) {
          return;
        }
        snooze.push(RemindMeSnooze {
          author: reaction_author.0,
          channel: channel.0,
          message_id: message.id.0,
          content: content.replace(format!("<@{}>", reaction_author.0).as_str(), ""),
        });
      }
      let message = channel
        .say(&ctx.http, "Enter the new duration please")
        .await
        .unwrap();
      SNOOZE_MESSAGE.lock().unwrap().push(message);
    }
  }
}

pub async fn snooze_message(ctx: &Context, message: &Message) {
  let message_author = message.author.id;
  let channel = message.channel_id;
  let content = message.content.clone();
  let list_snooze = SNOOZE.lock().unwrap().clone();
  if let Some(snooze) = list_snooze
    .iter()
    .find(|s| s.channel == channel.0 && s.author == message_author.0)
  {
    if let Some(captures) = TIME_INPUT_REGEX.captures(&content) {
      let trigger_date: Option<DateTime<_>> = extract_date(&captures);
      if trigger_date.is_none() {
        snooze_send_error_message(&channel, message, &ctx.http).await;
      }
      {
        let mut db_instance = INSTANCE.write().unwrap();
        db_instance.event_add(NewEvent {
          author: snooze.author as i64,
          channel: snooze.channel as i64,
          content: &snooze.content,
          trigger_date: trigger_date.unwrap().naive_utc(),
        });
      }
      channel
        .delete_message(&ctx.http, snooze.message_id)
        .await
        .unwrap();
      message.react(&ctx.http, '✅').await.unwrap();
      let mut list_snooze = SNOOZE.lock().unwrap();
      list_snooze.retain(|s| s.message_id != snooze.message_id);
      let list_snooze_message = SNOOZE_MESSAGE.lock().unwrap();
      list_snooze_message.iter().cloned().for_each(|m| {
        let http = ctx.http.clone();
        if m.channel_id == channel {
          tokio::spawn(async move {
            channel.delete_message(http, m.id.0).await.unwrap();
          });
        }
      });
    } else {
      snooze_send_error_message(&channel, message, &ctx.http).await;
    }
  }
}

async fn snooze_send_error_message(channel: &ChannelId, message: &Message, http: &Arc<http::Http>) {
  channel.delete_message(&http, message.id.0).await.unwrap();
  let message = channel
    .say(&http, "The time parameter is invalid\nTry again please")
    .await
    .unwrap();
  SNOOZE_MESSAGE.lock().unwrap().push(message);
}
