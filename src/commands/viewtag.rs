  fn viewtag_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    if params.is_empty() {
      return Ok(false);
    }
    let channel = self.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => return Err("channel was not public".into())
    };
    let who = params[0];
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      match who.parse::<u64>() {
        Ok(n) => UserId(n),
        Err(_) => return Ok(false)
      }
    } else {
      message.mentions[0].id
    };

    let user = {
      let database = self.database.lock().unwrap();
      database.autotags.users.iter().find(|u| u.user_id == who.0 && u.server_id == server_id.0).cloned()
    };

    let msg = match user {
      Some(u) => format!("{} is {} on {}.", who.mention(), u.character, u.server),
      None => format!("{} is not tagged.", who.mention())
    };
    self.discord.send_embed(message.channel_id, "", |e| e.description(&msg)).chain_err(|| "could not send embed")?;
    Ok(true)
  }
