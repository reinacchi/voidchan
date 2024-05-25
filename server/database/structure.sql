CREATE TABLE files (
    id VARCHAR(5) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    buffer LONGTEXT NOT NULL,
    mimetype VARCHAR(255) NOT NULL,
    nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    uploader VARCHAR(255) NOT NULL,
    PRIMARY KEY (created_at)
);

CREATE TABLE posts (
    id INT AUTO_INCREMENT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    buffer LONGTEXT NOT NULL,
    mimetype VARCHAR(255) NOT NULL,
    uploader VARCHAR(255) NOT NULL,
    status ENUM("approved", "pending", "disapproved") DEFAULT "approved" NOT NULL,
    tags JSON,
    comments JSON,
    favourites INT DEFAULT 0,
    size INT NOT NULL,
    rating ENUM("explicit", "erotica", "suggestive", "safe") NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE tags (
    id INT AUTO_INCREMENT PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    category ENUM(
        "general",
        "meta",
        "character",
        "copyright",
        "artist"
    ) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL,
    description TEXT,
    aliases JSON,
    creator_id VARCHAR(255) NOT NULL
);

CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    auth_key VARCHAR(255) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    display_name VARCHAR(255),
    kudos INT DEFAULT 0,
    clearance_levels JSON,
    INDEX (username),
    INDEX (email)
);
