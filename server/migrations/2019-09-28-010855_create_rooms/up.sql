-- Your SQL goes here
CREATE TABLE public.rooms (
    uuid VARCHAR PRIMARY KEY,
    passphrase VARCHAR NULL,
    card_set TEXT[] NOT NULL,
    current_game_uuid VARCHAR,
    owner_uuid VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    last_updated_at TIMESTAMP NOT NULL
);
