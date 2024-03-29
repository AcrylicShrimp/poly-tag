-- Your SQL goes here

CREATE TABLE collections (
  id UUID NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
  name TEXT NOT NULL,
  description TEXT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX ON collections(name);
CREATE INDEX ON collections(created_at ASC, id ASC);
