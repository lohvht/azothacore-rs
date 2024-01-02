/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `spell_effect`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `spell_effect` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Effect` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectBasePoints` int(11) NOT NULL DEFAULT 0,
  `EffectIndex` int(11) NOT NULL DEFAULT 0,
  `EffectAura` int(11) NOT NULL DEFAULT 0,
  `DifficultyID` int(11) NOT NULL DEFAULT 0,
  `EffectAmplitude` float NOT NULL DEFAULT 0,
  `EffectAuraPeriod` int(11) NOT NULL DEFAULT 0,
  `EffectBonusCoefficient` float NOT NULL DEFAULT 0,
  `EffectChainAmplitude` float NOT NULL DEFAULT 0,
  `EffectChainTargets` int(11) NOT NULL DEFAULT 0,
  `EffectDieSides` int(11) NOT NULL DEFAULT 0,
  `EffectItemType` int(11) NOT NULL DEFAULT 0,
  `EffectMechanic` int(11) NOT NULL DEFAULT 0,
  `EffectPointsPerResource` float NOT NULL DEFAULT 0,
  `EffectRealPointsPerLevel` float NOT NULL DEFAULT 0,
  `EffectTriggerSpell` int(11) NOT NULL DEFAULT 0,
  `EffectPosFacing` float NOT NULL DEFAULT 0,
  `EffectAttributes` int(11) NOT NULL DEFAULT 0,
  `BonusCoefficientFromAP` float NOT NULL DEFAULT 0,
  `PvpMultiplier` float NOT NULL DEFAULT 0,
  `Coefficient` float NOT NULL DEFAULT 0,
  `Variance` float NOT NULL DEFAULT 0,
  `ResourceCoefficient` float NOT NULL DEFAULT 0,
  `GroupSizeBasePointsCoefficient` float NOT NULL DEFAULT 0,
  `EffectSpellClassMask1` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectSpellClassMask2` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectSpellClassMask3` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectSpellClassMask4` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectMiscValue1` int(11) NOT NULL DEFAULT 0,
  `EffectMiscValue2` int(11) NOT NULL DEFAULT 0,
  `EffectRadiusIndex1` int(10) unsigned NOT NULL DEFAULT 0,
  `EffectRadiusIndex2` int(10) unsigned NOT NULL DEFAULT 0,
  `ImplicitTarget1` int(10) unsigned NOT NULL DEFAULT 0,
  `ImplicitTarget2` int(10) unsigned NOT NULL DEFAULT 0,
  `SpellID` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `spell_effect` WRITE;
/*!40000 ALTER TABLE `spell_effect` DISABLE KEYS */;
/*!40000 ALTER TABLE `spell_effect` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

