-- This file should undo anything in `up.sql`
ALTER TABLE public.games DROP CONSTRAINT room_current_game_uuid_foreign_key;
