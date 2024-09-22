CREATE EXTENSION "uuid-ossp";

CREATE TABLE users
(
    id         uuid PRIMARY KEY     DEFAULT uuid_generate_v4(),
    email      text UNIQUE NOT NULL,
    avatar     text        NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
