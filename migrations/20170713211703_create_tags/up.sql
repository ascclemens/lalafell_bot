CREATE TABLE tags (
  id INTEGER PRIMARY KEY NOT NULL,
  user_id TEXT NOT NULL,
  server_id TEXT NOT NULL,
  character_id TEXT NOT NULL,
  character VARCHAR NOT NULL,
  server VARCHAR NOT NULL,
  last_updated BIGINT NOT NULL
)
