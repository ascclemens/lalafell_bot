create table temporary_roles (
  id serial primary key,
  guild_id bigint not null,
  user_id bigint not null,
  role_id bigint not null,
  message_id bigint not null,
  channel_id bigint,
  messages integer,
  expires_on bigint
)
