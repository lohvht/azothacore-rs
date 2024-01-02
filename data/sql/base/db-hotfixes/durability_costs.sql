/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `durability_costs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `durability_costs` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost5` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost6` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost7` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost8` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost9` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost10` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost11` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost12` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost13` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost14` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost15` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost16` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost17` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost18` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost19` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost20` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeaponSubClassCost21` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost5` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost6` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost7` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ArmorSubClassCost8` smallint(5) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `durability_costs` WRITE;
/*!40000 ALTER TABLE `durability_costs` DISABLE KEYS */;
/*!40000 ALTER TABLE `durability_costs` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

