PRAGMA foreign_keys = ON;
CREATE TABLE edits (
  id INTEGER PRIMARY KEY NOT NULL,
  message_id INTEGER NOT NULL,
  content TEXT NOT NULL,

  FOREIGN KEY(message_id) REFERENCES messages(id)
)
