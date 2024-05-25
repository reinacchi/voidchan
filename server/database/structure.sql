CREATE TABLE files (
    id VARCHAR(5) NOT NULL,
    date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    buffer LONGTEXT NOT NULL,
    mimetype VARCHAR(255) NOT NULL,
    nsfw BOOLEAN NOT NULL DEFAULT FALSE,
    uploader VARCHAR(255) NOT NULL,
    PRIMARY KEY (date)
);

CREATE TABLE posts (
    id INT AUTO_INCREMENT,
    createdAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    buffer LONGTEXT NOT NULL,
    mimetype VARCHAR(255) NOT NULL,
    uploader VARCHAR(255) NOT NULL,
    status ENUM("approved", "pending", "disapproved") DEFAULT "approved" NOT NULL,
    tags JSON,
    comments JSON,
    favourites INT DEFAULT 0,
    source JSON,
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
    createdAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updatedAt DATETIME NOT NULL,
    description TEXT,
    aliases JSON,
    creatorID VARCHAR(255) NOT NULL
);

CREATE TABLE users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    authKey VARCHAR(255) NOT NULL,
    createdAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    displayName VARCHAR(255),
    kudos INT DEFAULT 0,
    clearanceLevels JSON,
    INDEX (username),
    INDEX (email)
);
