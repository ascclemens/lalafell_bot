CREATE TABLE channel_configs (
  id INTEGER PRIMARY KEY NOT NULL,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  image_dump_allowed BOOLEAN
)
