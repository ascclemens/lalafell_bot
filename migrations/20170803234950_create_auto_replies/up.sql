CREATE TABLE auto_replies (
  id INTEGER PRIMARY KEY NOT NULL,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  message TEXT NOT NULL,
  on_join BOOLEAN NOT NULL
)
