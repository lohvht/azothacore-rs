/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `instance_reset`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `instance_reset` (
  `mapid` smallint(5) unsigned NOT NULL DEFAULT 0,
  `difficulty` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `resettime` int(10) unsigned NOT NULL DEFAULT 0,
  PRIMARY KEY (`mapid`,`difficulty`),
  KEY `difficulty` (`difficulty`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `instance_reset` WRITE;
/*!40000 ALTER TABLE `instance_reset` DISABLE KEYS */;
INSERT INTO `instance_reset` VALUES
(33,2,1426996800),
(36,2,1426996800),
(249,3,1427515200),
(249,4,1427515200),
(269,2,1426996800),
(409,9,1427515200),
(469,9,1427515200),
(509,3,1427169600),
(531,9,1427515200),
(532,3,1427515200),
(533,3,1427515200),
(533,4,1427515200),
(534,4,1427515200),
(540,2,1426996800),
(542,2,1426996800),
(543,2,1426996800),
(544,4,1427515200),
(545,2,1426996800),
(546,2,1426996800),
(547,2,1426996800),
(548,4,1427515200),
(550,4,1427515200),
(552,2,1426996800),
(553,2,1426996800),
(554,2,1426996800),
(555,2,1426996800),
(556,2,1426996800),
(557,2,1426996800),
(558,2,1426996800),
(560,2,1426996800),
(564,4,1427515200),
(565,4,1427515200),
(568,2,1426996800),
(574,2,1426996800),
(575,2,1426996800),
(576,2,1426996800),
(578,2,1426996800),
(580,4,1427515200),
(585,2,1426996800),
(595,2,1426996800),
(598,2,1426996800),
(599,2,1426996800),
(600,2,1426996800),
(601,2,1426996800),
(602,2,1426996800),
(603,3,1427515200),
(603,4,1427515200),
(604,2,1426996800),
(608,2,1426996800),
(615,3,1427515200),
(615,4,1427515200),
(616,3,1427515200),
(616,4,1427515200),
(619,2,1426996800),
(624,3,1427515200),
(624,4,1427515200),
(631,3,1427515200),
(631,4,1427515200),
(631,5,1427515200),
(631,6,1427515200),
(632,2,1426996800),
(643,2,1426996800),
(644,2,1426996800),
(645,2,1426996800),
(649,3,1427515200),
(649,4,1427515200),
(649,5,1427515200),
(649,6,1427515200),
(650,2,1426996800),
(657,2,1426996800),
(658,2,1426996800),
(668,2,1426996800),
(669,3,1427515200),
(669,4,1427515200),
(669,5,1427515200),
(669,6,1427515200),
(670,2,1426996800),
(671,3,1427515200),
(671,4,1427515200),
(671,5,1427515200),
(671,6,1427515200),
(720,3,1427515200),
(720,4,1427515200),
(720,5,1427515200),
(720,6,1427515200),
(724,3,1427515200),
(724,4,1427515200),
(724,5,1427515200),
(724,6,1427515200),
(725,2,1426996800),
(754,3,1427515200),
(754,4,1427515200),
(754,5,1427515200),
(754,6,1427515200),
(755,2,1426996800),
(757,3,1427515200),
(757,4,1427515200),
(757,5,1427515200),
(757,6,1427515200),
(859,2,1426996800),
(938,2,1426996800),
(939,2,1426996800),
(940,2,1426996800),
(959,2,1426996800),
(960,2,1426996800),
(961,2,1426996800),
(962,2,1426996800),
(967,3,1427515200),
(967,4,1427515200),
(967,5,1427515200),
(967,6,1427515200),
(994,2,1426996800),
(996,3,1427515200),
(996,4,1427515200),
(996,5,1427515200),
(996,6,1427515200),
(1001,2,1426996800),
(1004,2,1426996800),
(1007,2,1426996800),
(1008,3,1427515200),
(1008,4,1427515200),
(1008,5,1427515200),
(1008,6,1427515200),
(1009,3,1427515200),
(1009,4,1427515200),
(1009,5,1427515200),
(1009,6,1427515200),
(1011,2,1426996800),
(1098,3,1427515200),
(1098,4,1427515200),
(1098,5,1427515200),
(1098,6,1427515200),
(1136,14,1427515200),
(1136,15,1427515200),
(1136,16,1427515200),
(1175,2,1426996800),
(1176,2,1426996800),
(1182,2,1426996800),
(1195,2,1426996800),
(1205,14,1427515200),
(1205,15,1427515200),
(1205,16,1427515200),
(1208,2,1426996800),
(1209,2,1426996800),
(1228,14,1427515200),
(1228,15,1427515200),
(1228,16,1427515200),
(1279,2,1426996800),
(1358,2,1426996800);
/*!40000 ALTER TABLE `instance_reset` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
