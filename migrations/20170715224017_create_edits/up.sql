CREATE TABLE edits (
  id SERIAL PRIMARY KEY,
  message_id INTEGER NOT NULL REFERENCES messages(id),
  content TEXT NOT NULL
)
