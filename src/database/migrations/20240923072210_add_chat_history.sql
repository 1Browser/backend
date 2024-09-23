CREATE TABLE chat
(
    user_id         uuid        NOT NULL,
    article_url     text        NOT NULL,
    article_content text        NOT NULL,
    message         text        NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),

    CONSTRAINT chat_pk PRIMARY KEY (user_id, article_url)
);