extern crate discord;
extern crate dotenv;
#[macro_use]
extern crate serde_derive;
extern crate envy;

use discord::{Discord, State};
use discord::model::{Event};
use discord::voice::AudioSource;

use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender, Receiver};

// TODO: Add response messages

#[derive(Default)]
struct Dj {
  controls: HashMap<u64, Sender<AudioSourceControl>>
}

pub enum MusicItem {
  YouTube(String)
}

fn main() {
  println!("Loading .env");

  dotenv::dotenv().ok();

  println!("Reading environment variables");

  let environment: Environment = envy::from_env().expect("Invalid or missing environment variables");

  println!("Creating bot");

  let discord = Discord::from_bot_token(&environment.discord_bot_token).expect("could not start discord from bot token");
  let (mut connection, ready) = discord.connect().expect("could not connect");
  let mut state = State::new(ready);

  let mut dj = Dj::default();

  loop {
    let event = match connection.recv_event() {
      Ok(e) => e,
      Err(e) => {
        println!("could not receive event: {}", e);
        continue;
      }
    };

    state.update(&event);

    let valid_commands = &["play", "stop", "clear", "pause", "resume"];

    match event {
      Event::MessageCreate(message) => {
        let params: Vec<&str> = message.content.split_whitespace().collect();
        if params.is_empty() || params[0].len() < 2 {
          continue;
        }

        let command: String = params[0].to_lowercase().chars().skip(1).collect();
        if !params[0].starts_with('!') || !valid_commands.contains(&command.as_ref()) {
          continue;
        }

        let (server_id, voice_channel) = match state.find_voice_user(message.author.id) {
          Some((Some(server_id), voice_channel)) => (server_id, voice_channel),
          _ => {
            if let Err(e) = discord.send_embed(message.channel_id, "", |e| e.description("Must be in a public voice channel to DJ.")) {
              println!("could not send message: {}", e);
            }
            continue;
          }
        };

        let voice = connection.voice(Some(server_id));

        match command.as_ref() {
          "play" => {
            if params.len() < 2 {
              if let Err(e) = discord.send_embed(message.channel_id, "", |e| e.description("Please provide a YouTube URL to play.")) {
                println!("could not send message: {}", e);
              }
              continue;
            }
            let url = params[1];
            if let Some((_, controls)) = dj.controls.iter().find(|&(id, _)| *id == voice_channel.0) {
              if let Err(e) = controls.send(AudioSourceControl::AddQueue(url.to_string())) {
                println!("could not send control: {}", e);
              }
              continue;
            }
            let source = match discord::voice::open_ytdl_stream(url) {
              Ok(s) => s,
              Err(e) => {
                println!("could not open source: {}", e);
                if let Err(e) = discord.send_embed(message.channel_id, "", |e| e.description("Could not open video for streaming.")) {
                  println!("could not send message: {}", e);
                }
                continue;
              }
            };

            voice.connect(voice_channel);

            let (tx, rx) = channel();
            dj.controls.insert(voice_channel.0, tx);
            let buf_source = BufferedQueuedAudioSource::new(source, rx);
            voice.play(Box::new(buf_source));
          },
          "stop" => {
            let control = match dj.controls.get(&voice_channel.0) {
              Some(c) => c,
              None => continue
            };
            if let Err(e) = control.send(AudioSourceControl::Stop) {
              println!("couldn't send control: {}", e);
            }
          },
          "pause" => {
            let control = match dj.controls.get(&voice_channel.0) {
              Some(c) => c,
              None => continue
            };
            if let Err(e) = control.send(AudioSourceControl::Pause) {
              println!("couldn't send control: {}", e);
            }
          },
          "resume" => {
            let control = match dj.controls.get(&voice_channel.0) {
              Some(c) => c,
              None => continue
            };
            if let Err(e) = control.send(AudioSourceControl::Resume) {
              println!("couldn't send control: {}", e);
            }
          },
          "clear" => {
            let control = match dj.controls.get(&voice_channel.0) {
              Some(c) => c,
              None => continue
            };
            if let Err(e) = control.send(AudioSourceControl::ClearQueue) {
              println!("couldn't send control: {}", e);
            }
          },
          _ => continue
        }
      },
      Event::VoiceStateUpdate(server_id, _) => {
        let chan = match connection.voice(server_id).current_channel() {
          Some(c) => c,
          None => continue
        };
        if let Some(server_id) = server_id {
          if let Some(srv) = state.servers().iter().find(|srv| srv.id == server_id) {
            if srv.voice_states.iter().filter(|vs| vs.channel_id == Some(chan)).count() <= 1 {
              connection.voice(Some(server_id)).disconnect();
            }
          }
        }
      },
      _ => {}
    }
  }
}

#[derive(Debug, Deserialize)]
pub struct Environment {
  #[serde(rename = "lb_discord_bot_token")]
  pub discord_bot_token: String,
}

pub enum AudioSourceControl {
  Pause,
  Resume,
  Stop,
  AddQueue(String),
  ClearQueue
}

struct BufferedQueuedAudioSource {
  controls: Receiver<AudioSourceControl>,
  source: Box<AudioSource>,
  buffer: Box<[i16]>,
  queue: Vec<String>,
  position: usize,
  capacity: usize,
  paused: bool,
  finished: bool
}

impl BufferedQueuedAudioSource {
  fn new(source: Box<AudioSource>, controls: Receiver<AudioSourceControl>) -> Self {
    BufferedQueuedAudioSource {
      controls: controls,
      source: source,
      buffer: vec![0; 61440].into_boxed_slice(),
      queue: Vec::default(),
      position: 0,
      capacity: 0,
      paused: false,
      finished: false
    }
  }
}

impl AudioSource for BufferedQueuedAudioSource {
  fn is_stereo(&mut self) -> bool {
    self.source.is_stereo()
  }

  fn read_frame(&mut self, buffer: &mut [i16]) -> Option<usize> {
    self.read_controls();
    self.handle_queue();
    if self.finished {
      return Some(0);
    }
    if self.paused {
      for byte in buffer.iter_mut() {
        *byte = 0;
      }
      return Some(buffer.len());
    }
    self.read(buffer)
  }
}

impl BufferedQueuedAudioSource {
  fn read_controls(&mut self) {
    while let Ok(control) = self.controls.try_recv() {
      match control {
        AudioSourceControl::Resume => self.paused = false,
        AudioSourceControl::Pause => self.paused = true,
        AudioSourceControl::AddQueue(s) => self.queue.push(s),
        AudioSourceControl::ClearQueue => self.queue.clear(),
        AudioSourceControl::Stop => {
          self.queue.clear();
          self.finished = true;
        }
      }
    }
  }

  fn handle_queue(&mut self) {
    if !self.finished || self.queue.is_empty() {
      return;
    }
    while !self.queue.is_empty() {
      let first = self.queue.remove(0);
      match discord::voice::open_ytdl_stream(&first) {
        Ok(s) => {
          self.source = s;
          self.finished = false;
          break;
        },
        Err(e) => {
          println!("could not open ytdl stream in queue: {}", e);
          continue;
        }
      }
    }
  }

  fn read(&mut self, buffer: &mut [i16]) -> Option<usize> {
    if self.position == self.capacity && buffer.len() >= self.buffer.len() {
      return self.source.read_frame(buffer);
    }
    let nread = {
      let rem = match self.fill_buff() {
        Some(s) => s,
        None => return None
      };
      let read = std::cmp::min(buffer.len(), rem.len());
      for i in 0..read {
        buffer[i] = rem[i];
      }
      read
    };
    self.consume(nread);
    Some(nread)
  }

  fn fill_buff(&mut self) -> Option<&[i16]> {
    if self.position >= self.capacity {
      self.capacity = match self.source.read_frame(&mut self.buffer) {
        Some(s) => s,
        None => return None
      };
      // FIXME: This should happen when read_frame turns None, but discord-rs has a bug
      // https://github.com/SpaceManiac/discord-rs/issues/107
      if self.capacity == 0 && !self.finished {
        self.finished = true;
      }
      self.position = 0;
    }
    Some(&self.buffer[self.position..self.capacity])
  }

  fn consume(&mut self, amt: usize) {
    self.position = std::cmp::min(self.position + amt, self.capacity);
  }
}
