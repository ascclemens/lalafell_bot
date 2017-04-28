  fn tag_command(&self, message: &Message, params: &[&str]) -> Result<bool> {
    let channel = self.discord.get_channel(message.channel_id).chain_err(|| "could not get channel for message")?;
    let server_id = match channel {
      Channel::Public(c) => c.server_id,
      _ => return Err("channel was not public".into())
    };
    let user = self.discord.get_member(server_id, message.author.id).chain_err(|| "could not get member for message")?;
    let mut state_option = self.state.lock().unwrap();
    let state = state_option.as_mut().unwrap();
    let server = match state.servers().iter().find(|x| x.id == server_id) {
      Some(s) => s,
      None => return Err("could not find server for channel".into())
    };
    if server.owner_id != message.author.id {
      let roles = &server.roles;
      let user_roles: Option<Vec<&Role>> = user.roles.iter()
        .map(|r| roles.iter().find(|z| z.id == *r))
        .collect();
      match user_roles {
        Some(ur) => {
          let can_manage_roles = ur.iter()
            .any(|r| r.permissions.contains(permissions::MANAGE_ROLES));
          if !can_manage_roles {
            return Ok(false);
          }
        },
        None => return Ok(false)
      }
    }

    if params.len() < 3 {
      return Ok(false);
    }

    let who = params[0];
    let who = if !who.starts_with("<@") && !who.ends_with('>') && message.mentions.len() != 1 {
      match who.parse::<u64>() {
        Ok(n) => UserId(n),
        Err(_) => return Ok(false)
      }
    } else {
      message.mentions[0].id
    };
    let ff_server = params[1];
    let name = params[2..].join(" ");

    let (msg, emoji) = match self.search_tag(who, server, ff_server, &name)? {
      Some(error) => (Some(error), ReactionEmoji::Unicode(String::from("\u{274c}"))),
      None => (None, ReactionEmoji::Unicode(String::from("\u{2705}")))
    };
    if let Some(msg) = msg {
      self.discord.send_embed(message.channel_id, "", |f| f.description(&msg)).chain_err(|| "could not send embed")?;
    }
    self.discord.add_reaction(message.channel_id, message.id, emoji).chain_err(|| "could not add reaction")?;
    Ok(true)
  }
