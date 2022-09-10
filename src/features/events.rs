use crate::{
  core::commands::{CallBackParams, CallbackReturn},
  database::{NewEvent, INSTANCE},
};
use chrono::{prelude::*, Duration};
use procedural_macros::command;
use regex::Regex;
use serenity::{
  http,
  model::id::{ChannelId, UserId},
  prelude::Mentionable,
};
use std::sync::Arc;
use std::{thread, time};

lazy_static! {
  static ref TIME_INPUT_REGEX: Regex = Regex::new(
    r#"^([0-9]{1,4})((m(inutes?)?)|(h(ours?)?)|(d(ays?)?(([0-9]{2})[:h]([0-9]{2})?)?))$"#
  )
  .expect("unable to create regex");
}

#[command]
pub async fn remind_me(params: CallBackParams) -> CallbackReturn {
  let input_date = &params.args[1];

  if let Some(captures) = TIME_INPUT_REGEX.captures(input_date) {
    let number = captures
      .get(1)
      .unwrap()
      .as_str()
      .parse::<u16>()
      .expect("unable to parse number value from regex");

    let mut trigger_date: Option<NaiveDateTime> = None;
    let now = chrono::offset::Local::now().naive_local();
    for i in [3, 5, 7] {
      if captures.get(i).is_some() {
        match i {
          3 => {
            trigger_date = Some(now + Duration::minutes(number.into()));
          }
          5 => {
            trigger_date = Some(now + Duration::hours(number.into()));
          }
          7 => {
            if let Some(hours) = captures.get(10) {
              let hours = hours.as_str().parse().expect("unable to parse hours");
              if let Some(minutes) = captures.get(11) {
                let minutes = minutes.as_str().parse().expect("unable to parse hours");
                trigger_date = Some(
                  now
                    .with_hour(hours)
                    .unwrap()
                    .with_minute(minutes)
                    .expect("unable to parse minutes")
                    + Duration::days(number.into()),
                );
              } else {
                trigger_date = Some(now.with_hour(hours).unwrap() + Duration::days(number.into()));
              }
            } else {
              trigger_date = Some(now + Duration::days(number.into()));
            }
          }
          _ => panic!("captures matches missing case"),
        }
        break;
      }
    }
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
      trigger_date: trigger_date.unwrap(),
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
    let now = chrono::offset::Local::now().naive_local();
    for event in events {
      let time_since_trigger = now - event.trigger_date;
      let event_id = event.id;

      if time_since_trigger > Duration::seconds(0) {
        let http_clone = http.clone();
        // I don't known why i need to do this
        // The other threads just seem to die if i don't spawn here (the bot even disconnect)
        // And it needs awaiting because other wise when there multiple spawn only one is executed
        let spawn_result = tokio::spawn(async move {
          ChannelId(event.channel as u64)
            .say(
              http_clone,
              format!(
                "{} {}",
                UserId(event.author as u64).mention(),
                event.content
              ),
            )
            .await
            .expect("unable to send event");
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
