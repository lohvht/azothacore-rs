/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `cinematic_sequences`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `cinematic_sequences` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `SoundID` int(10) unsigned NOT NULL DEFAULT 0,
  `Camera1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera5` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera6` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera7` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Camera8` smallint(5) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `cinematic_sequences` WRITE;
/*!40000 ALTER TABLE `cinematic_sequences` DISABLE KEYS */;
/*!40000 ALTER TABLE `cinematic_sequences` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

