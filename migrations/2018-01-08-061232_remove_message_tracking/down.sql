CREATE TABLE messages (
  id SERIAL PRIMARY KEY,
  message_id TEXT NOT NULL,
  channel_id TEXT NOT NULL,
  content TEXT NOT NULL
);
CREATE TABLE edits (
  id SERIAL PRIMARY KEY,
  message_id INTEGER NOT NULL REFERENCES messages(id),
  content TEXT NOT NULL
);
