ALTER TABLE tags RENAME TO tags_old;
CREATE TABLE tags (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  server_id INTEGER NOT NULL,
  character_id INTEGER NOT NULL,
  character VARCHAR NOT NULL,
  server VARCHAR NOT NULL
);
INSERT INTO tags (id, user_id, server_id, character_id, character, server)
  SELECT id, user_id, server_id, character_id, character, server FROM tags_old;
DROP TABLE tags_old;
