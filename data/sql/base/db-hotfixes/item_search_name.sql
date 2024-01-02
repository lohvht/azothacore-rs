/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `item_search_name`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `item_search_name` (
  `AllowableRace` bigint(20) NOT NULL DEFAULT 0,
  `Display` text DEFAULT NULL,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags1` int(11) NOT NULL DEFAULT 0,
  `Flags2` int(11) NOT NULL DEFAULT 0,
  `Flags3` int(11) NOT NULL DEFAULT 0,
  `ItemLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `OverallQualityID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ExpansionID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RequiredLevel` tinyint(4) NOT NULL DEFAULT 0,
  `MinFactionID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinReputation` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AllowableClass` int(11) NOT NULL DEFAULT 0,
  `RequiredSkill` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RequiredSkillRank` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RequiredAbility` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `item_search_name` WRITE;
/*!40000 ALTER TABLE `item_search_name` DISABLE KEYS */;
/*!40000 ALTER TABLE `item_search_name` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

