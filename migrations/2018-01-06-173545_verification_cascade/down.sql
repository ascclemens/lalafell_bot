PRAGMA foreign_keys = OFF;
ALTER TABLE verifications RENAME TO old_verifications;

PRAGMA foreign_keys = ON;
CREATE TABLE verifications (
  id INTEGER PRIMARY KEY NOT NULL,
  tag_id INTEGER NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT 'f',
  verification_string VARCHAR,

  FOREIGN KEY(tag_id) REFERENCES tags(id)
);

INSERT INTO verifications SELECT * FROM old_verifications;

DROP TABLE old_verifications;
