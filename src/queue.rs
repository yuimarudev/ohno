use crate::voice::{OutputKind, VoiceClient};
use crate::Context;
use poise::serenity_prelude::{
  model::{channel::GuildChannel, id::ChannelId},
  Mutex as PoiseMutex,
};
use songbird::input::{codec::Codec, reader::Reader, Input};
use std::{env, sync::Arc};

#[derive(Debug)]
pub struct Queue {
  pub channel_id: ChannelId,
  pub handler: Arc<PoiseMutex<songbird::Call>>,
}

impl Queue {
  pub async fn play(&self, text: String) {
    let voice_client = VoiceClient::new(
      env::var("apiKey").unwrap(),
      env::var("region").unwrap(),
      OutputKind::Raw48KHz16BitMonoPcm,
    );
    let speech_result = voice_client.speech(text).await;
    if let Ok(bytes) = speech_result {
      let reader = Reader::from(bytes.to_vec());
      let kind = Codec::Pcm;
      let input = Input::new(false, reader, kind, songbird::input::Container::Raw, None);

      self.handler.lock().await.play_source(input);
    }
  }

  pub async fn new(ctx: Context<'_>, vc: GuildChannel) -> Result<Self, ()> {
    let guild_id = ctx.guild_id().unwrap();
    let manager = songbird::get(ctx.discord())
      .await
      .expect("Songbird Voice client placed in at initialisation.")
      .clone();
    let (handler, result) = manager.join(guild_id, vc.id).await;
    match result {
      Ok(()) => Ok(Self {
        channel_id: ctx.channel_id(),
        handler,
      }),
      Err(e) => {
        ctx.say(format!("Error: {}", e)).await.ok();
        Err(())
      }
    }
  }
}
