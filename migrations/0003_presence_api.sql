CREATE TABLE user_presence_kv (
    user_id BIGINT UNSIGNED NOT NULL,
    kv_key VARCHAR(255) NOT NULL,
    kv_value TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, kv_key),
    CONSTRAINT fk_user_presence_kv_user
        FOREIGN KEY (user_id) REFERENCES users(id)
        ON DELETE CASCADE
);
