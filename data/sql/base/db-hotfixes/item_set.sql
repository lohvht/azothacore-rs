/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `item_set`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `item_set` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `ItemID1` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID2` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID3` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID4` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID5` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID6` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID7` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID8` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID9` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID10` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID11` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID12` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID13` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID14` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID15` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID16` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemID17` int(10) unsigned NOT NULL DEFAULT 0,
  `RequiredSkillRank` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RequiredSkill` int(10) unsigned NOT NULL DEFAULT 0,
  `SetFlags` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `item_set` WRITE;
/*!40000 ALTER TABLE `item_set` DISABLE KEYS */;
/*!40000 ALTER TABLE `item_set` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

