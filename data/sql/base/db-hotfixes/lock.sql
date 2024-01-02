/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `lock`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `lock` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Index1` int(11) NOT NULL DEFAULT 0,
  `Index2` int(11) NOT NULL DEFAULT 0,
  `Index3` int(11) NOT NULL DEFAULT 0,
  `Index4` int(11) NOT NULL DEFAULT 0,
  `Index5` int(11) NOT NULL DEFAULT 0,
  `Index6` int(11) NOT NULL DEFAULT 0,
  `Index7` int(11) NOT NULL DEFAULT 0,
  `Index8` int(11) NOT NULL DEFAULT 0,
  `Skill1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill5` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill6` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill7` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Skill8` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Type1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type6` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type7` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Type8` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action6` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action7` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Action8` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `lock` WRITE;
/*!40000 ALTER TABLE `lock` DISABLE KEYS */;
/*!40000 ALTER TABLE `lock` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

