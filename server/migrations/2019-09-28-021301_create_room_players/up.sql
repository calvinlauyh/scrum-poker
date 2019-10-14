-- Your SQL goes here
CREATE TABLE public.room_players (
    room_uuid VARCHAR,
    player_uuid VARCHAR PRIMARY KEY,
    joined_at TIMESTAMP NOT NULL,
    left_at TIMESTAMP NULL,
    FOREIGN KEY (room_uuid) REFERENCES public.rooms(uuid),
    FOREIGN KEY (player_uuid) REFERENCES public.users(uuid)
);
