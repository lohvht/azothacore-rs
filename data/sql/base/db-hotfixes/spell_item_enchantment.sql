/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `spell_item_enchantment`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `spell_item_enchantment` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `EffectArg1` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectArg2` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectArg3` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectScalingPoints1` float NOT NULL DEFAULT 0,
  `EffectScalingPoints2` float NOT NULL DEFAULT 0,
  `EffectScalingPoints3` float NOT NULL DEFAULT 0,
  `TransmogCost` int(10) unsigned NOT NULL DEFAULT 0,
  `IconFileDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectPointsMin1` smallint(6) NOT NULL DEFAULT 0,
  `EffectPointsMin2` smallint(6) NOT NULL DEFAULT 0,
  `EffectPointsMin3` smallint(6) NOT NULL DEFAULT 0,
  `ItemVisual` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Flags` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RequiredSkillID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `RequiredSkillRank` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ItemLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Charges` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Effect1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Effect2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Effect3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ConditionID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ScalingClass` tinyint(4) NOT NULL DEFAULT 0,
  `ScalingClassRestricted` tinyint(4) NOT NULL DEFAULT 0,
  `TransmogPlayerConditionID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `spell_item_enchantment` WRITE;
/*!40000 ALTER TABLE `spell_item_enchantment` DISABLE KEYS */;
/*!40000 ALTER TABLE `spell_item_enchantment` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

