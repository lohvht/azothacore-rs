/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `faction`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `faction` (
  `ReputationRaceMask1` bigint(20) NOT NULL DEFAULT 0,
  `ReputationRaceMask2` bigint(20) NOT NULL DEFAULT 0,
  `ReputationRaceMask3` bigint(20) NOT NULL DEFAULT 0,
  `ReputationRaceMask4` bigint(20) NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `Description` text DEFAULT NULL,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `ReputationBase1` int(11) NOT NULL DEFAULT 0,
  `ReputationBase2` int(11) NOT NULL DEFAULT 0,
  `ReputationBase3` int(11) NOT NULL DEFAULT 0,
  `ReputationBase4` int(11) NOT NULL DEFAULT 0,
  `ParentFactionMod1` float NOT NULL DEFAULT 0,
  `ParentFactionMod2` float NOT NULL DEFAULT 0,
  `ReputationMax1` int(11) NOT NULL DEFAULT 0,
  `ReputationMax2` int(11) NOT NULL DEFAULT 0,
  `ReputationMax3` int(11) NOT NULL DEFAULT 0,
  `ReputationMax4` int(11) NOT NULL DEFAULT 0,
  `ReputationIndex` smallint(6) NOT NULL DEFAULT 0,
  `ReputationClassMask1` smallint(6) NOT NULL DEFAULT 0,
  `ReputationClassMask2` smallint(6) NOT NULL DEFAULT 0,
  `ReputationClassMask3` smallint(6) NOT NULL DEFAULT 0,
  `ReputationClassMask4` smallint(6) NOT NULL DEFAULT 0,
  `ReputationFlags1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ReputationFlags2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ReputationFlags3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ReputationFlags4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ParentFactionID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ParagonFactionID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ParentFactionCap1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ParentFactionCap2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Expansion` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Flags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FriendshipRepID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `faction` WRITE;
/*!40000 ALTER TABLE `faction` DISABLE KEYS */;
/*!40000 ALTER TABLE `faction` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

