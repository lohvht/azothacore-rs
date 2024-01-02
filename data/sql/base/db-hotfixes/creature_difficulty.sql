/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `creature_difficulty`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `creature_difficulty` (
  `ID` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `CreatureID` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `Flags1` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags2` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags3` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags4` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags5` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags6` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags7` int(10) unsigned NOT NULL DEFAULT 0,
  `FactionTemplateID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Expansion` tinyint(4) NOT NULL DEFAULT 0,
  `MinLevel` tinyint(4) NOT NULL DEFAULT 0,
  `MaxLevel` tinyint(4) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(5) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `creature_difficulty` WRITE;
/*!40000 ALTER TABLE `creature_difficulty` DISABLE KEYS */;
/*!40000 ALTER TABLE `creature_difficulty` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

