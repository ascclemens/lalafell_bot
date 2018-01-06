CREATE TABLE delete_all_messages (
  id SERIAL PRIMARY KEY,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  after INTEGER NOT NULL,
  exclude BYTEA NOT NULL
)
