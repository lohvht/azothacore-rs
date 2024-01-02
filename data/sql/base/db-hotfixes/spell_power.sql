/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `spell_power`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `spell_power` (
  `ManaCost` int(11) NOT NULL DEFAULT 0,
  `PowerCostPct` float NOT NULL DEFAULT 0,
  `PowerPctPerSecond` float NOT NULL DEFAULT 0,
  `RequiredAuraSpellID` int(11) NOT NULL DEFAULT 0,
  `PowerCostMaxPct` float NOT NULL DEFAULT 0,
  `OrderIndex` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PowerType` tinyint(4) NOT NULL DEFAULT 0,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `ManaCostPerLevel` int(11) NOT NULL DEFAULT 0,
  `ManaPerSecond` int(11) NOT NULL DEFAULT 0,
  `OptionalCost` int(10) unsigned NOT NULL DEFAULT 0,
  `PowerDisplayID` int(10) unsigned NOT NULL DEFAULT 0,
  `AltPowerBarID` int(11) NOT NULL DEFAULT 0,
  `SpellID` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `spell_power` WRITE;
/*!40000 ALTER TABLE `spell_power` DISABLE KEYS */;
/*!40000 ALTER TABLE `spell_power` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

