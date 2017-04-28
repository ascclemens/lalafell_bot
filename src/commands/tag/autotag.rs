  fn autotag_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    let channel = self.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => return Err("channel was not public".into())
    };
    let mut state_option = self.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => return Err("could not find server for channel".into())
    };

    if params.len() < 3 {
      return Ok(false);
    }

    let ff_server = params[0];
    let name = params[1..].join(" ");

    let (msg, emoji) = match self.search_tag(message.author.id, server, ff_server, &name)? {
      Some(error) => (Some(error), ReactionEmoji::Unicode(String::from("\u{274c}"))),
      None => (None, ReactionEmoji::Unicode(String::from("\u{2705}")))
    };
    if let Some(msg) = msg {
      self.discord.send_embed(message.channel_id, "", |f| f.description(&msg)).chain_err(|| "could not send embed")?;
    }
    self.discord.add_reaction(message.channel_id, message.id, emoji).chain_err(|| "could not add reaction")?;
    Ok(true)
  }
