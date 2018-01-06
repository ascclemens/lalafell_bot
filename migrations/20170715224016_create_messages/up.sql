CREATE TABLE messages (
  id SERIAL PRIMARY KEY,
  message_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  content TEXT NOT NULL
)
