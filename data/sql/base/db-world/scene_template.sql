/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `scene_template`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `scene_template` (
  `SceneId` int(10) unsigned NOT NULL,
  `Flags` int(10) unsigned NOT NULL DEFAULT 0,
  `ScriptPackageID` int(10) unsigned NOT NULL DEFAULT 0,
  `ScriptName` varchar(64) NOT NULL DEFAULT '',
  PRIMARY KEY (`SceneId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `scene_template` WRITE;
/*!40000 ALTER TABLE `scene_template` DISABLE KEYS */;
INSERT INTO `scene_template` VALUES
(35,25,186,''),
(37,1,25,''),
(59,1,195,''),
(60,11,154,''),
(66,9,200,''),
(68,13,202,''),
(72,1,212,''),
(73,1,211,''),
(74,0,214,''),
(75,17,213,''),
(77,11,41,''),
(79,5,68,''),
(81,0,221,''),
(82,0,222,''),
(83,0,223,''),
(84,0,224,''),
(91,1,241,''),
(94,11,248,''),
(109,1,263,''),
(110,11,264,''),
(111,1,69,''),
(189,0,449,''),
(276,1,567,''),
(424,16,685,''),
(490,16,746,''),
(502,20,752,''),
(506,17,756,''),
(508,17,758,''),
(521,16,759,''),
(522,27,760,''),
(602,27,788,''),
(610,16,800,''),
(612,27,801,''),
(613,16,802,''),
(621,16,806,''),
(623,16,810,''),
(624,16,813,''),
(625,16,812,''),
(628,16,1029,''),
(629,16,817,''),
(630,16,808,''),
(632,16,822,''),
(634,16,824,''),
(635,16,825,''),
(636,16,826,''),
(637,16,827,''),
(638,16,828,''),
(640,16,830,''),
(648,16,838,''),
(652,16,842,''),
(653,16,843,''),
(658,0,846,''),
(659,16,847,''),
(660,16,848,''),
(666,16,844,''),
(667,20,854,''),
(668,16,855,''),
(669,27,856,''),
(670,16,859,''),
(672,16,858,''),
(673,16,861,''),
(674,16,862,''),
(675,5,1361,''),
(679,16,867,''),
(680,16,868,''),
(688,16,873,''),
(689,20,871,''),
(691,4,879,''),
(692,4,880,''),
(693,4,881,''),
(694,16,940,''),
(695,16,884,''),
(696,16,886,''),
(697,16,885,''),
(708,16,883,''),
(709,16,887,''),
(719,17,896,''),
(720,21,898,''),
(723,20,893,''),
(724,16,894,''),
(727,16,903,''),
(730,16,908,''),
(731,16,909,''),
(732,25,910,''),
(733,16,1018,''),
(734,16,907,''),
(739,0,915,''),
(740,16,912,''),
(741,0,917,''),
(742,0,918,''),
(743,26,919,''),
(753,16,922,''),
(754,16,923,''),
(756,17,925,''),
(757,16,927,''),
(758,16,928,''),
(770,16,933,''),
(771,16,934,''),
(772,16,937,''),
(782,16,938,''),
(786,5,1352,''),
(788,16,942,''),
(795,20,945,''),
(796,20,946,''),
(797,16,947,''),
(799,16,950,''),
(801,25,952,''),
(802,17,955,''),
(803,25,956,''),
(811,16,961,''),
(812,16,962,''),
(815,16,965,''),
(818,25,986,''),
(824,20,976,''),
(831,16,981,''),
(855,16,998,''),
(864,20,1017,''),
(877,11,1030,''),
(878,16,1031,''),
(880,20,1033,''),
(886,11,1036,''),
(889,16,1040,''),
(890,16,1041,''),
(907,5,1358,''),
(910,5,1359,''),
(919,20,1371,''),
(935,16,1378,''),
(936,16,1379,''),
(937,16,1380,''),
(965,16,1387,''),
(994,16,1403,''),
(1012,17,1441,''),
(1022,27,1405,''),
(1023,0,1431,''),
(1046,0,1434,''),
(1047,0,1433,''),
(1092,16,1477,''),
(1125,21,1500,''),
(1141,24,1508,''),
(1146,17,1518,''),
(1181,16,1550,'scene_deathwing_simulator'),
(1194,1,1560,''),
(1223,20,1588,''),
(1246,1,1594,''),
(1269,21,1717,''),
(1279,4,1628,''),
(1280,4,1629,''),
(1281,4,1630,''),
(1287,25,1638,''),
(1301,27,1641,''),
(1304,16,1642,''),
(1305,16,1645,''),
(1311,25,1649,''),
(1312,21,1650,''),
(1324,11,1653,''),
(1326,20,1655,''),
(1327,25,1657,''),
(1328,17,1658,''),
(1335,26,1661,''),
(1339,16,1665,''),
(1341,16,1667,''),
(1350,17,1676,''),
(1351,52,1677,''),
(1356,62,1678,''),
(1362,20,1669,''),
(1368,17,1687,''),
(1373,62,1691,''),
(1387,20,1669,''),
(1426,23,1704,''),
(1435,23,1709,''),
(1438,31,1713,''),
(1445,16,1722,''),
(1448,31,1727,''),
(1449,58,1728,''),
(1450,27,1653,''),
(1452,16,1731,''),
(1455,25,1733,''),
(1458,16,1737,''),
(1459,11,1714,''),
(1460,11,1725,''),
(1461,11,1726,''),
(1467,16,1747,''),
(1472,25,1746,''),
(1474,16,1750,''),
(1481,25,1757,''),
(1483,16,1760,''),
(1485,16,1761,''),
(1494,25,1762,''),
(1495,17,1764,''),
(1496,20,1765,''),
(1497,16,1766,''),
(1499,21,1768,''),
(1531,16,1775,''),
(1673,16,1855,''),
(1674,16,1858,''),
(1688,17,1872,''),
(1751,27,1878,''),
(1761,27,1886,''),
(1771,27,1879,''),
(1773,27,1880,''),
(1775,27,1928,''),
(1801,17,1905,''),
(1805,27,1883,''),
(1806,27,1887,''),
(1808,27,1884,''),
(1818,20,1959,''),
(1842,27,1928,'');
/*!40000 ALTER TABLE `scene_template` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
