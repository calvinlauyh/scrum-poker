-- Your SQL goes here
CREATE TABLE public.users (
    uuid VARCHAR PRIMARY KEY,
    email VARCHAR(256) UNIQUE NULL,
    name VARCHAR(255) NULL,
    created_at TIMESTAMP NOT NULL,
    last_updated_at TIMESTAMP NOT NULL
);
