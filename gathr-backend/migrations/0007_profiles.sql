ALTER TABLE users ADD COLUMN bio TEXT;

ALTER TABLE users
  ADD CONSTRAINT users_avatar_media_fk
  FOREIGN KEY (avatar_media_id) REFERENCES media(id) ON DELETE SET NULL;
