/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `character_equipmentsets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `character_equipmentsets` (
  `guid` bigint(20) NOT NULL DEFAULT 0,
  `setguid` bigint(20) NOT NULL AUTO_INCREMENT,
  `setindex` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `name` varchar(31) NOT NULL,
  `iconname` varchar(100) NOT NULL,
  `ignore_mask` int(11) unsigned NOT NULL DEFAULT 0,
  `AssignedSpecIndex` int(11) NOT NULL DEFAULT -1,
  `item0` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item1` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item2` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item3` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item4` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item5` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item6` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item7` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item8` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item9` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item10` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item11` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item12` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item13` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item14` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item15` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item16` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item17` bigint(20) unsigned NOT NULL DEFAULT 0,
  `item18` bigint(20) unsigned NOT NULL DEFAULT 0,
  PRIMARY KEY (`setguid`),
  UNIQUE KEY `idx_set` (`guid`,`setguid`,`setindex`),
  KEY `Idx_setindex` (`setindex`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `character_equipmentsets` WRITE;
/*!40000 ALTER TABLE `character_equipmentsets` DISABLE KEYS */;
/*!40000 ALTER TABLE `character_equipmentsets` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

