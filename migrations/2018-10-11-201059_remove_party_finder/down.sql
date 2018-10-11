CREATE TABLE party_finder_configs (
  server_id BIGINT NOT NULL,
  channel_id BIGINT NOT NULL,
  message_id BIGINT NOT NULL,
  role_id BIGINT NOT NULL,
  emoji TEXT NOT NULL,
  timeout BIGINT NOT NULL,

  PRIMARY KEY (server_id, channel_id)
)
