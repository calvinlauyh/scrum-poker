-- Your SQL goes here
CREATE TABLE public.games (
    uuid VARCHAR PRIMARY KEY,
    room_uuid VARCHAR,
    sequence INTEGER,
    title VARCHAR NOT NULL,
    description VARCHAR NULL,
    created_at TIMESTAMP NOT NULL,
    last_updated_at TIMESTAMP NOT NULL
);
