-- Your SQL goes here
ALTER TABLE public.rooms
    ADD CONSTRAINT room_current_game_uuid_foreign_key FOREIGN KEY(current_game_uuid) REFERENCES public.games(uuid);