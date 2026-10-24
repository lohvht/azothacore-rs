/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `spell_areatrigger_rollpitchyaw`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `spell_areatrigger_rollpitchyaw` (
  `SpellMiscId` int(10) unsigned NOT NULL,
  `X` float NOT NULL DEFAULT 0,
  `Y` float NOT NULL DEFAULT 0,
  `Z` float NOT NULL DEFAULT 0,
  `TargetX` float NOT NULL DEFAULT 0,
  `TargetY` float NOT NULL DEFAULT 0,
  `TargetZ` float NOT NULL DEFAULT 0,
  `VerifiedBuild` int(10) unsigned NOT NULL DEFAULT 0,
  PRIMARY KEY (`SpellMiscId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `spell_areatrigger_rollpitchyaw` WRITE;
/*!40000 ALTER TABLE `spell_areatrigger_rollpitchyaw` DISABLE KEYS */;
INSERT INTO `spell_areatrigger_rollpitchyaw` VALUES
(5167,0,0,0,0,0,370.01,26124);
/*!40000 ALTER TABLE `spell_areatrigger_rollpitchyaw` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

