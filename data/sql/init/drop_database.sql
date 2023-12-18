REVOKE ALL PRIVILEGES, GRANT OPTION FROM 'azcore'@'localhost';

REVOKE ALL PRIVILEGES, GRANT OPTION FROM 'azcore'@'%';

DROP USER 'azcore'@'localhost';

DROP USER 'azcore'@'%';

DROP DATABASE IF EXISTS `azcore_world`;

DROP DATABASE IF EXISTS `azcore_characters`;

DROP DATABASE IF EXISTS `azcore_auth`;

DROP DATABASE IF EXISTS `azcore_hotfixes`;
