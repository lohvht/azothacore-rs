/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `rand_prop_points`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `rand_prop_points` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Epic1` int(10) unsigned NOT NULL DEFAULT 0,
  `Epic2` int(10) unsigned NOT NULL DEFAULT 0,
  `Epic3` int(10) unsigned NOT NULL DEFAULT 0,
  `Epic4` int(10) unsigned NOT NULL DEFAULT 0,
  `Epic5` int(10) unsigned NOT NULL DEFAULT 0,
  `Superior1` int(10) unsigned NOT NULL DEFAULT 0,
  `Superior2` int(10) unsigned NOT NULL DEFAULT 0,
  `Superior3` int(10) unsigned NOT NULL DEFAULT 0,
  `Superior4` int(10) unsigned NOT NULL DEFAULT 0,
  `Superior5` int(10) unsigned NOT NULL DEFAULT 0,
  `Good1` int(10) unsigned NOT NULL DEFAULT 0,
  `Good2` int(10) unsigned NOT NULL DEFAULT 0,
  `Good3` int(10) unsigned NOT NULL DEFAULT 0,
  `Good4` int(10) unsigned NOT NULL DEFAULT 0,
  `Good5` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `rand_prop_points` WRITE;
/*!40000 ALTER TABLE `rand_prop_points` DISABLE KEYS */;
/*!40000 ALTER TABLE `rand_prop_points` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

