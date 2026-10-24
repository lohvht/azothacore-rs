/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `zone_scripts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `zone_scripts` (
  `zoneId` int(10) unsigned NOT NULL,
  `scriptname` char(64) NOT NULL DEFAULT '',
  PRIMARY KEY (`zoneId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci ROW_FORMAT=DYNAMIC;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `zone_scripts` WRITE;
/*!40000 ALTER TABLE `zone_scripts` DISABLE KEYS */;
INSERT INTO `zone_scripts` VALUES
(4,'zone_blasted_lands'),
(41,'zone_deadwind_pass'),
(139,'zone_western_plaguelands'),
(491,'zone_razorfen_kraul'),
(6723,'zone_tanaan_jungle_100'),
(8392,'zone_legion_dalaran_underbelly'),
(8574,'zone_argus_krokuun'),
(8701,'zone_argus_macaree'),
(8899,'zone_argus_antoran_wastes');
/*!40000 ALTER TABLE `zone_scripts` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

