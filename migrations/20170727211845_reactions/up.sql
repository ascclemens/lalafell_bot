CREATE TABLE reactions (
  id INTEGER PRIMARY KEY NOT NULL,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  emoji TEXT NOT NULL,
  role TEXT NOT NULL
)
