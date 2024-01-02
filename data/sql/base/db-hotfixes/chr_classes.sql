/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `chr_classes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `chr_classes` (
  `PetNameToken` text DEFAULT NULL,
  `Name` text DEFAULT NULL,
  `NameFemale` text DEFAULT NULL,
  `NameMale` text DEFAULT NULL,
  `Filename` text DEFAULT NULL,
  `CreateScreenFileDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `SelectScreenFileDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `LowResScreenFileDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `IconFileDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `StartingLevel` int(11) NOT NULL DEFAULT 0,
  `Flags` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CinematicSequenceID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `DefaultSpec` smallint(5) unsigned NOT NULL DEFAULT 0,
  `DisplayPower` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `SpellClassSet` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AttackPowerPerStrength` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AttackPowerPerAgility` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RangedAttackPowerPerAgility` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PrimaryStatPriority` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `chr_classes` WRITE;
/*!40000 ALTER TABLE `chr_classes` DISABLE KEYS */;
/*!40000 ALTER TABLE `chr_classes` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

