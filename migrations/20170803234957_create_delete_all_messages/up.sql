CREATE TABLE delete_all_messages (
  id INTEGER PRIMARY KEY NOT NULL,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  after INTEGER NOT NULL,
  exclude BLOB NOT NULL
)
