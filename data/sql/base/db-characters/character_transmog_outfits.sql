/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `character_transmog_outfits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `character_transmog_outfits` (
  `guid` bigint(20) NOT NULL DEFAULT 0,
  `setguid` bigint(20) NOT NULL AUTO_INCREMENT,
  `setindex` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `name` varchar(128) NOT NULL,
  `iconname` varchar(256) NOT NULL,
  `ignore_mask` int(11) NOT NULL DEFAULT 0,
  `appearance0` int(10) NOT NULL DEFAULT 0,
  `appearance1` int(10) NOT NULL DEFAULT 0,
  `appearance2` int(10) NOT NULL DEFAULT 0,
  `appearance3` int(10) NOT NULL DEFAULT 0,
  `appearance4` int(10) NOT NULL DEFAULT 0,
  `appearance5` int(10) NOT NULL DEFAULT 0,
  `appearance6` int(10) NOT NULL DEFAULT 0,
  `appearance7` int(10) NOT NULL DEFAULT 0,
  `appearance8` int(10) NOT NULL DEFAULT 0,
  `appearance9` int(10) NOT NULL DEFAULT 0,
  `appearance10` int(10) NOT NULL DEFAULT 0,
  `appearance11` int(10) NOT NULL DEFAULT 0,
  `appearance12` int(10) NOT NULL DEFAULT 0,
  `appearance13` int(10) NOT NULL DEFAULT 0,
  `appearance14` int(10) NOT NULL DEFAULT 0,
  `appearance15` int(10) NOT NULL DEFAULT 0,
  `appearance16` int(10) NOT NULL DEFAULT 0,
  `appearance17` int(10) NOT NULL DEFAULT 0,
  `appearance18` int(10) NOT NULL DEFAULT 0,
  `mainHandEnchant` int(10) NOT NULL DEFAULT 0,
  `offHandEnchant` int(10) NOT NULL DEFAULT 0,
  PRIMARY KEY (`setguid`),
  UNIQUE KEY `idx_set` (`guid`,`setguid`,`setindex`),
  KEY `Idx_setindex` (`setindex`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `character_transmog_outfits` WRITE;
/*!40000 ALTER TABLE `character_transmog_outfits` DISABLE KEYS */;
/*!40000 ALTER TABLE `character_transmog_outfits` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

