/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `creature`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `creature` (
  `ID` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `ItemID1` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `ItemID2` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `ItemID3` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `Mount` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `DisplayID1` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `DisplayID2` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `DisplayID3` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `DisplayID4` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `DisplayIDProbability1` float NOT NULL DEFAULT 0,
  `DisplayIDProbability2` float NOT NULL DEFAULT 0,
  `DisplayIDProbability3` float NOT NULL DEFAULT 0,
  `DisplayIDProbability4` float NOT NULL DEFAULT 0,
  `Name` text NOT NULL,
  `FemaleName` text NOT NULL,
  `SubName` text NOT NULL,
  `FemaleSubName` text NOT NULL,
  `Type` mediumint(3) unsigned NOT NULL DEFAULT 0,
  `Family` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Classification` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `InhabitType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(5) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `creature` WRITE;
/*!40000 ALTER TABLE `creature` DISABLE KEYS */;
/*!40000 ALTER TABLE `creature` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

