/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `guild_rewards`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `guild_rewards` (
  `ItemID` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `MinGuildRep` tinyint(3) unsigned DEFAULT 0,
  `RaceMask` bigint(20) unsigned DEFAULT 0,
  `Cost` bigint(20) unsigned DEFAULT 0,
  PRIMARY KEY (`ItemID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `guild_rewards` WRITE;
/*!40000 ALTER TABLE `guild_rewards` DISABLE KEYS */;
INSERT INTO `guild_rewards` VALUES
(61931,4,0,15000000),
(61935,4,0,15000000),
(61936,4,0,15000000),
(61937,4,0,15000000),
(61942,4,0,15000000),
(61958,4,0,15000000),
(62023,5,4294967295,17500000),
(62024,5,4294967295,17500000),
(62025,5,4294967295,17500000),
(62026,5,4294967295,17500000),
(62027,5,4294967295,17500000),
(62029,5,4294967295,17500000),
(62038,4,0,12000000),
(62039,4,0,12000000),
(62040,4,0,12000000),
(62286,4,0,100000000),
(62287,6,33555378,200000000),
(62298,7,18875469,15000000),
(62799,5,0,1500000),
(62800,5,0,1500000),
(63125,7,0,30000000),
(63138,7,4294967295,3000000),
(63206,5,18875469,3000000),
(63207,5,33555378,3000000),
(63352,5,18875469,1500000),
(63353,5,33555378,1500000),
(63359,5,18875469,1500000),
(63398,6,0,3000000),
(64398,5,18875469,2000000),
(64399,5,18875469,3000000),
(64400,5,33555378,1500000),
(64401,5,33555378,2000000),
(64402,5,33555378,3000000),
(65274,6,33555378,5000000),
(65360,6,18875469,5000000),
(65361,5,18875469,3000000),
(65362,5,33555378,3000000),
(65363,6,18875469,5000000),
(65364,6,33555378,5000000),
(65435,5,0,1500000),
(65498,5,0,1500000),
(67107,7,33555378,15000000),
(68136,6,18875469,200000000),
(69209,4,4294967295,1250000),
(69210,5,4294967295,2500000),
(69887,4,0,15000000),
(69888,5,4294967295,17500000),
(69892,4,0,12000000),
(71033,7,0,15000000),
(85508,5,4294967295,1000000),
(85509,5,4294967295,1000000),
(85510,5,4294967295,1000000),
(85666,7,4294967295,30000000),
(89190,5,4294967295,1500000),
(89191,7,4294967295,2000000),
(89192,5,4294967295,1500000),
(89193,7,4294967295,2000000),
(89194,5,4294967295,1500000),
(89195,7,4294967295,2000000),
(114968,6,0,3000000),
(116666,7,4294967295,40000000),
(120352,7,0,1000000);
/*!40000 ALTER TABLE `guild_rewards` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
