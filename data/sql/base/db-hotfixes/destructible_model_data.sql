/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `destructible_model_data`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `destructible_model_data` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `State0Wmo` smallint(5) unsigned NOT NULL DEFAULT 0,
  `State1Wmo` smallint(5) unsigned NOT NULL DEFAULT 0,
  `State2Wmo` smallint(5) unsigned NOT NULL DEFAULT 0,
  `State3Wmo` smallint(5) unsigned NOT NULL DEFAULT 0,
  `HealEffectSpeed` smallint(5) unsigned NOT NULL DEFAULT 0,
  `State0ImpactEffectDoodadSet` tinyint(4) NOT NULL DEFAULT 0,
  `State0AmbientDoodadSet` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `State0NameSet` tinyint(4) NOT NULL DEFAULT 0,
  `State1DestructionDoodadSet` tinyint(4) NOT NULL DEFAULT 0,
  `State1ImpactEffectDoodadSet` tinyint(4) NOT NULL DEFAULT 0,
  `State1AmbientDoodadSet` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `State1NameSet` tinyint(4) NOT NULL DEFAULT 0,
  `State2DestructionDoodadSet` tinyint(4) NOT NULL DEFAULT 0,
  `State2ImpactEffectDoodadSet` tinyint(4) NOT NULL DEFAULT 0,
  `State2AmbientDoodadSet` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `State2NameSet` tinyint(4) NOT NULL DEFAULT 0,
  `State3InitDoodadSet` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `State3AmbientDoodadSet` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `State3NameSet` tinyint(4) NOT NULL DEFAULT 0,
  `EjectDirection` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `DoNotHighlight` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `HealEffect` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `destructible_model_data` WRITE;
/*!40000 ALTER TABLE `destructible_model_data` DISABLE KEYS */;
/*!40000 ALTER TABLE `destructible_model_data` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

