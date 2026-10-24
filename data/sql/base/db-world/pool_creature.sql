/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `pool_creature`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `pool_creature` (
  `guid` bigint(20) unsigned NOT NULL DEFAULT 0,
  `pool_entry` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `chance` float unsigned NOT NULL DEFAULT 0,
  `description` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`guid`),
  KEY `idx_guid` (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `pool_creature` WRITE;
/*!40000 ALTER TABLE `pool_creature` DISABLE KEYS */;
INSERT INTO `pool_creature` VALUES
(2186,462,0,'Vultros-Spawn1'),
(3419,573,0,'Foe Reaper 4000-Spawn21'),
(3695,572,0,'Leprithus'),
(18439,38453,0,'Arcturis'),
(18632,38453,0,'Arcturis'),
(56918,573,0,'Foe Reaper 4000-Spawn20'),
(57255,573,0,'Foe Reaper 4000-Spawn19'),
(58224,573,0,'Foe Reaper 4000-Spawn18'),
(58229,573,0,'Foe Reaper 4000-Spawn17'),
(59219,573,0,'Foe Reaper 4000-Spawn16'),
(61880,573,0,'Foe Reaper 4000-Spawn15'),
(61885,573,0,'Foe Reaper 4000-Spawn14'),
(61900,573,0,'Foe Reaper 4000-Spawn13'),
(61909,573,0,'Foe Reaper 4000-Spawn12'),
(61915,573,0,'Foe Reaper 4000-Spawn11'),
(61916,573,0,'Foe Reaper 4000-Spawn10'),
(61937,573,0,'Foe Reaper 4000-Spawn9'),
(61938,573,0,'Foe Reaper 4000-Spawn8'),
(75887,573,0,'Foe Reaper 4000-Spawn7'),
(75888,573,0,'Foe Reaper 4000-Spawn6'),
(81471,33776,0,'Gondria'),
(82536,573,0,'Foe Reaper 4000-Spawn5'),
(86265,35189,0,'Skoll'),
(86266,35189,0,'Skoll'),
(86267,35189,0,'Skoll'),
(87053,573,0,'Foe Reaper 4000-Spawn4'),
(88424,573,0,'Foe Reaper 4000-Spawn3'),
(88595,33776,0,'Gondria'),
(91035,32517,0,'Loquenahak'),
(91036,32517,0,'Loquenahak'),
(91037,32517,0,'Loquenahak'),
(91038,32517,0,'Loquenahak'),
(91039,32517,0,'Loquenahak'),
(91040,32517,0,'Loquenahak'),
(91041,32517,0,'Loquenahak'),
(97253,573,0,'Foe Reaper 4000-Spawn2'),
(97255,573,0,'Foe Reaper 4000-Spawn1'),
(116946,462,0,'Vultros-Spawn16'),
(116952,462,0,'Vultros-Spawn15'),
(117452,462,0,'Vultros-Spawn14'),
(117453,462,0,'Vultros-Spawn13'),
(117454,462,0,'Vultros-Spawn12'),
(117455,462,0,'Vultros-Spawn11'),
(117456,462,0,'Vultros-Spawn10'),
(117457,462,0,'Vultros-Spawn9'),
(117458,462,0,'Vultros-Spawn8'),
(123509,33776,0,'Gondria'),
(123510,33776,0,'Gondria'),
(126879,572,0,'Leprithus'),
(126972,462,0,'Vultros-Spawn7'),
(126973,462,0,'Vultros-Spawn6'),
(126974,462,0,'Vultros-Spawn5'),
(126975,462,0,'Vultros-Spawn4'),
(126976,462,0,'Vultros-Spawn3'),
(126977,462,0,'Vultros-Spawn2'),
(127764,33776,0,'Gondria'),
(127765,33776,0,'Gondria'),
(214702,573,0,'Foe Reaper 4000-Spawn22'),
(316894,572,0,'Leprithus'),
(371487,50138,0,'Karoma'),
(10954517,50138,0,'Karoma'),
(10954518,50138,0,'Karoma'),
(10954519,50138,0,'Karoma'),
(10954520,50138,0,'Karoma'),
(10954523,50051,0,'Clampant Fantôme'),
(10954524,50051,0,'Clampant Fantôme'),
(10954525,50051,0,'Clampant Fantôme'),
(10954526,50051,0,'Clampant Fantôme'),
(20549920,113694,0,'Pashya'),
(20549921,113694,0,'Pashya'),
(20549922,113694,0,'Pashya'),
(20549923,113694,0,'Pashya');
/*!40000 ALTER TABLE `pool_creature` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

