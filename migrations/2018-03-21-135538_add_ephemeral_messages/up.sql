create table ephemeral_messages (
  id serial primary key,
  guild_id bigint not null,
  channel_id bigint not null,
  message_id bigint not null,
  expires_on bigint not null
)
