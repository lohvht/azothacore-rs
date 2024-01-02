/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `keychain`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `keychain` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Key1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key6` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key7` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key8` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key9` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key10` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key11` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key12` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key13` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key14` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key15` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key16` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key17` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key18` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key19` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key20` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key21` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key22` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key23` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key24` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key25` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key26` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key27` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key28` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key29` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key30` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key31` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Key32` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `keychain` WRITE;
/*!40000 ALTER TABLE `keychain` DISABLE KEYS */;
/*!40000 ALTER TABLE `keychain` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

