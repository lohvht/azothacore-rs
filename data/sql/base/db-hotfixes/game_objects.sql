/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `game_objects`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `game_objects` (
  `Name` text DEFAULT NULL,
  `Pos1` float NOT NULL DEFAULT 0,
  `Pos2` float NOT NULL DEFAULT 0,
  `Pos3` float NOT NULL DEFAULT 0,
  `Rot1` float NOT NULL DEFAULT 0,
  `Rot2` float NOT NULL DEFAULT 0,
  `Rot3` float NOT NULL DEFAULT 0,
  `Rot4` float NOT NULL DEFAULT 0,
  `Scale` float NOT NULL DEFAULT 0,
  `PropValue1` int(11) NOT NULL DEFAULT 0,
  `PropValue2` int(11) NOT NULL DEFAULT 0,
  `PropValue3` int(11) NOT NULL DEFAULT 0,
  `PropValue4` int(11) NOT NULL DEFAULT 0,
  `PropValue5` int(11) NOT NULL DEFAULT 0,
  `PropValue6` int(11) NOT NULL DEFAULT 0,
  `PropValue7` int(11) NOT NULL DEFAULT 0,
  `PropValue8` int(11) NOT NULL DEFAULT 0,
  `OwnerID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `DisplayID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PhaseID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PhaseGroupID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PhaseUseFlags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `TypeID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `game_objects` WRITE;
/*!40000 ALTER TABLE `game_objects` DISABLE KEYS */;
/*!40000 ALTER TABLE `game_objects` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

