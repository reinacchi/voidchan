ALTER TABLE users
    ADD COLUMN discord_user_id VARCHAR(32) NULL UNIQUE AFTER api_token,
    ADD COLUMN preferred_url_mode VARCHAR(8) NOT NULL DEFAULT 'v' AFTER discord_user_id,
    ADD COLUMN preferred_hex_colour VARCHAR(7) NOT NULL DEFAULT '#7289da' AFTER preferred_url_mode,
    ADD COLUMN is_blacklisted BOOLEAN NOT NULL DEFAULT FALSE AFTER preferred_hex_colour;

ALTER TABLE files
    ADD COLUMN uploader_id BIGINT UNSIGNED NULL AFTER uploader,
    ADD CONSTRAINT fk_files_uploader
        FOREIGN KEY (uploader_id) REFERENCES users(id)
        ON DELETE SET NULL;
