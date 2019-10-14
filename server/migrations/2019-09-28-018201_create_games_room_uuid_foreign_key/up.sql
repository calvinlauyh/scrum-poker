-- Your SQL goes here
ALTER TABLE public.games
    ADD CONSTRAINT room_uuid_foreign_key FOREIGN KEY (room_uuid) REFERENCES public.rooms (uuid);