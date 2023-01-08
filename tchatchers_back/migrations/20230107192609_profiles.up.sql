-- Add up migration script here
CREATE TABLE PROFILE (
	id SERIAL PRIMARY KEY,
	name VARCHAR NOT NULL UNIQUE
);

INSERT INTO PROFILE(id, name) 
VALUES (1, 'user'), (2, 'moderator'), (3, 'admin');

ALTER TABLE CHATTER
ADD COLUMN profile_id INTEGER NOT NULL DEFAULT 1,
ADD CONSTRAINT fk_profile FOREIGN KEY(profile_id) REFERENCES PROFILE(id) ON DELETE SET DEFAULT;