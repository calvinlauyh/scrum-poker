-- Your SQL goes here
CREATE TABLE public.game_hands (
    id SERIAL PRIMARY KEY,
    game_uuid VARCHAR,
    user_uuid VARCHAR,
    card VARCHAR NOT NULL,
    last_updated_at TIMESTAMP NOT NULL,
    FOREIGN KEY(game_uuid) REFERENCES games(uuid),
    FOREIGN KEY(user_uuid) REFERENCES users(uuid)
);
