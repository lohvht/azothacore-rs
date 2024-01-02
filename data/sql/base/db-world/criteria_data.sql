/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `criteria_data`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `criteria_data` (
  `criteria_id` mediumint(8) NOT NULL,
  `type` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `value1` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `value2` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `ScriptName` char(64) NOT NULL DEFAULT '',
  PRIMARY KEY (`criteria_id`,`type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci COMMENT='Achievment system';
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `criteria_data` WRITE;
/*!40000 ALTER TABLE `criteria_data` DISABLE KEYS */;
INSERT INTO `criteria_data` VALUES
(4244,11,0,0,'achievement_hadronox_denied'),
(5258,0,57064,0,''),
(7604,11,0,0,'achievement_thaddius_shocking'),
(7605,11,0,0,'achievement_thaddius_shocking'),
(11118,16,404,0,''),
(11119,16,404,0,''),
(11120,16,404,0,''),
(11121,16,404,0,''),
(11122,16,404,0,''),
(11123,16,404,0,''),
(11124,16,404,0,''),
(11125,16,404,0,''),
(11126,16,404,0,''),
(11127,16,404,0,''),
(11134,5,66303,0,''),
(11134,16,404,0,''),
(11135,5,66303,0,''),
(11135,16,404,0,''),
(11136,5,66303,0,''),
(11136,16,404,0,''),
(11137,5,66303,0,''),
(11137,16,404,0,''),
(11138,5,66303,0,''),
(11138,16,404,0,''),
(11139,5,66303,0,''),
(11139,16,404,0,''),
(11140,5,66303,0,''),
(11140,16,404,0,''),
(11141,5,66303,0,''),
(11141,16,404,0,''),
(11142,5,66303,0,''),
(11142,16,404,0,''),
(17577,11,0,0,'achievement_share_the_pain');
/*!40000 ALTER TABLE `criteria_data` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

