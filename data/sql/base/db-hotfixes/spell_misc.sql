/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `spell_misc`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `spell_misc` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `CastingTimeIndex` smallint(5) unsigned NOT NULL DEFAULT 0,
  `DurationIndex` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RangeIndex` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SchoolMask` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `SpellIconFileDataID` int(11) NOT NULL DEFAULT 0,
  `Speed` float NOT NULL DEFAULT 0,
  `ActiveIconFileDataID` int(11) NOT NULL DEFAULT 0,
  `LaunchDelay` float NOT NULL DEFAULT 0,
  `DifficultyID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Attributes1` int(11) NOT NULL DEFAULT 0,
  `Attributes2` int(11) NOT NULL DEFAULT 0,
  `Attributes3` int(11) NOT NULL DEFAULT 0,
  `Attributes4` int(11) NOT NULL DEFAULT 0,
  `Attributes5` int(11) NOT NULL DEFAULT 0,
  `Attributes6` int(11) NOT NULL DEFAULT 0,
  `Attributes7` int(11) NOT NULL DEFAULT 0,
  `Attributes8` int(11) NOT NULL DEFAULT 0,
  `Attributes9` int(11) NOT NULL DEFAULT 0,
  `Attributes10` int(11) NOT NULL DEFAULT 0,
  `Attributes11` int(11) NOT NULL DEFAULT 0,
  `Attributes12` int(11) NOT NULL DEFAULT 0,
  `Attributes13` int(11) NOT NULL DEFAULT 0,
  `Attributes14` int(11) NOT NULL DEFAULT 0,
  `SpellID` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `spell_misc` WRITE;
/*!40000 ALTER TABLE `spell_misc` DISABLE KEYS */;
/*!40000 ALTER TABLE `spell_misc` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

