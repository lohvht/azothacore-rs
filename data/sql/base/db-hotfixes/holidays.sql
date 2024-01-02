/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `holidays`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `holidays` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Date1` int(10) unsigned NOT NULL DEFAULT 0,
  `Date2` int(10) unsigned NOT NULL DEFAULT 0,
  `Date3` int(10) unsigned NOT NULL DEFAULT 0,
  `Date4` int(10) unsigned NOT NULL DEFAULT 0,
  `Date5` int(10) unsigned NOT NULL DEFAULT 0,
  `Date6` int(10) unsigned NOT NULL DEFAULT 0,
  `Date7` int(10) unsigned NOT NULL DEFAULT 0,
  `Date8` int(10) unsigned NOT NULL DEFAULT 0,
  `Date9` int(10) unsigned NOT NULL DEFAULT 0,
  `Date10` int(10) unsigned NOT NULL DEFAULT 0,
  `Date11` int(10) unsigned NOT NULL DEFAULT 0,
  `Date12` int(10) unsigned NOT NULL DEFAULT 0,
  `Date13` int(10) unsigned NOT NULL DEFAULT 0,
  `Date14` int(10) unsigned NOT NULL DEFAULT 0,
  `Date15` int(10) unsigned NOT NULL DEFAULT 0,
  `Date16` int(10) unsigned NOT NULL DEFAULT 0,
  `Duration1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration5` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration6` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration7` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration8` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration9` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Duration10` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Region` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Looping` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags6` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags7` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags8` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags9` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFlags10` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Priority` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `CalendarFilterType` tinyint(4) NOT NULL DEFAULT 0,
  `Flags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `HolidayNameID` int(10) unsigned NOT NULL DEFAULT 0,
  `HolidayDescriptionID` int(10) unsigned NOT NULL DEFAULT 0,
  `TextureFileDataID1` int(11) NOT NULL DEFAULT 0,
  `TextureFileDataID2` int(11) NOT NULL DEFAULT 0,
  `TextureFileDataID3` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `holidays` WRITE;
/*!40000 ALTER TABLE `holidays` DISABLE KEYS */;
/*!40000 ALTER TABLE `holidays` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

