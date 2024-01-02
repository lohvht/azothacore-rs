/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `unit_power_bar`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `unit_power_bar` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `Cost` text DEFAULT NULL,
  `OutOfError` text DEFAULT NULL,
  `ToolTip` text DEFAULT NULL,
  `RegenerationPeace` float NOT NULL DEFAULT 0,
  `RegenerationCombat` float NOT NULL DEFAULT 0,
  `FileDataID1` int(11) NOT NULL DEFAULT 0,
  `FileDataID2` int(11) NOT NULL DEFAULT 0,
  `FileDataID3` int(11) NOT NULL DEFAULT 0,
  `FileDataID4` int(11) NOT NULL DEFAULT 0,
  `FileDataID5` int(11) NOT NULL DEFAULT 0,
  `FileDataID6` int(11) NOT NULL DEFAULT 0,
  `Color1` int(11) NOT NULL DEFAULT 0,
  `Color2` int(11) NOT NULL DEFAULT 0,
  `Color3` int(11) NOT NULL DEFAULT 0,
  `Color4` int(11) NOT NULL DEFAULT 0,
  `Color5` int(11) NOT NULL DEFAULT 0,
  `Color6` int(11) NOT NULL DEFAULT 0,
  `StartInset` float NOT NULL DEFAULT 0,
  `EndInset` float NOT NULL DEFAULT 0,
  `StartPower` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Flags` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CenterPower` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `BarType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinPower` int(10) unsigned NOT NULL DEFAULT 0,
  `MaxPower` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `unit_power_bar` WRITE;
/*!40000 ALTER TABLE `unit_power_bar` DISABLE KEYS */;
/*!40000 ALTER TABLE `unit_power_bar` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

