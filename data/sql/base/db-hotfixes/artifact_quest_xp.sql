/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `artifact_quest_xp`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `artifact_quest_xp` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty1` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty2` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty3` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty4` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty5` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty6` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty7` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty8` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty9` int(10) unsigned NOT NULL DEFAULT 0,
  `Difficulty10` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `artifact_quest_xp` WRITE;
/*!40000 ALTER TABLE `artifact_quest_xp` DISABLE KEYS */;
/*!40000 ALTER TABLE `artifact_quest_xp` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

