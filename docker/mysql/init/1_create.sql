CREATE DATABASE IF NOT EXISTS userdb1 CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE userdb1;

CREATE TABLE IF NOT EXISTS `todo` (
  `id` varchar(64) NOT NULL,
  `text` varchar(256) NOT NULL,
  `done` bool NOT NULL,
  `user_id` varchar(64) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_bin;

CREATE TABLE IF NOT EXISTS `user` (
  `id` varchar(64) NOT NULL,
  `name` varchar(256) NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_bin;