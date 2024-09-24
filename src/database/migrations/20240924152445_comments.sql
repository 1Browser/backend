CREATE TABLE comments
(
    id         uuid PRIMARY KEY     DEFAULT uuid_generate_v4(),
    url        text        NOT NULL,
    selector   text        NOT NULL,
    origin     text,
    user_id    uuid REFERENCES users (id),
    content    text        NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX ON comments (url, created_at DESC);
