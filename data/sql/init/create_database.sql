CREATE DATABASE `azcore_world` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE DATABASE `azcore_characters` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE DATABASE `azcore_auth` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE DATABASE `azcore_hotfixes` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

CREATE USER 'azcore'@'localhost' IDENTIFIED BY 'azcore' WITH MAX_QUERIES_PER_HOUR 0 MAX_CONNECTIONS_PER_HOUR 0 MAX_UPDATES_PER_HOUR 0;

GRANT USAGE ON * . * TO 'azcore'@'localhost';

GRANT ALL PRIVILEGES ON `azcore_world` . * TO 'azcore'@'localhost' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON `azcore_characters` . * TO 'azcore'@'localhost' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON `azcore_auth` . * TO 'azcore'@'localhost' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON `azcore_hotfixes` . * TO 'azcore'@'localhost' WITH GRANT OPTION;

CREATE USER 'azcore'@'%' IDENTIFIED BY 'azcore' WITH MAX_QUERIES_PER_HOUR 0 MAX_CONNECTIONS_PER_HOUR 0 MAX_UPDATES_PER_HOUR 0;

GRANT USAGE ON * . * TO 'azcore'@'%';

GRANT ALL PRIVILEGES ON `azcore_world` . * TO 'azcore'@'%' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON `azcore_characters` . * TO 'azcore'@'%' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON `azcore_auth` . * TO 'azcore'@'%' WITH GRANT OPTION;

GRANT ALL PRIVILEGES ON `azcore_hotfixes` . * TO 'azcore'@'%' WITH GRANT OPTION;

FLUSH PRIVILEGES;
