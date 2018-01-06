CREATE TABLE channel_configs (
  id SERIAL PRIMARY KEY,
  server_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  image_dump_allowed BOOLEAN
)
