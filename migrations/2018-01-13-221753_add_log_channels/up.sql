CREATE TABLE log_channels (
  server_id BIGINT NOT NULL,
  channel_id BIGINT NOT NULL,
  PRIMARY KEY(server_id, channel_id)
)
