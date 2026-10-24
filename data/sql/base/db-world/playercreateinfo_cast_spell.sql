/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `playercreateinfo_cast_spell`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `playercreateinfo_cast_spell` (
  `raceMask` int(10) unsigned NOT NULL DEFAULT 0,
  `classMask` int(10) unsigned NOT NULL DEFAULT 0,
  `spell` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `note` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`raceMask`,`classMask`,`spell`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `playercreateinfo_cast_spell` WRITE;
/*!40000 ALTER TABLE `playercreateinfo_cast_spell` DISABLE KEYS */;
INSERT INTO `playercreateinfo_cast_spell` VALUES
(0,256,688,'Warlock - Summon Imp'),
(0,2048,90266,'Demon Hunter - Fly'),
(1,4,79597,'Human - Hunter - Young Wolf'),
(2,4,79598,'Orc - Hunter - Young Boar'),
(4,4,79593,'Dwarf - Hunter - Young Bear'),
(8,4,79602,'Night Elf - Hunter - Young Cat'),
(16,4,79600,'Undead - Hunter - Young Widow'),
(16,925,73523,'Undead - Rigor Mortis'),
(32,4,79603,'Tauren - Hunter - Young Tallstrider'),
(64,4,153724,'Gnome - Hunter - Mechanical Bunny'),
(128,4,79599,'Troll - Hunter - Young Raptor'),
(256,4,79595,'Goblin - Hunter - Young Crab'),
(256,477,77534,'Goblin - Pack Hobgoblin'),
(512,4,79594,'Blood Elf - Hunter - Young Dragonhawk'),
(1024,4,79601,'Draenei - Hunter - Young Moth'),
(2097152,4,79596,'Worgen - Hunter - Young Mastiff'),
(2097152,1437,72792,'Worgen - Learn Racials'),
(2097152,1437,72857,'Worgen - Learn Two Forms'),
(2097152,1437,95759,'Worgen - Learn Darkflight'),
(8388608,1,108059,'Pandaren - Warrior - Remove weapon'),
(8388608,4,107924,'Pandaren - Hunter - Wise Turtle'),
(8388608,8,108058,'Pandaren - Rogue - Remove weapon'),
(8388608,16,108057,'Pandaren - Priest - Remove weapon'),
(8388608,64,108056,'Pandaren - Shaman - Remove weapon'),
(8388608,128,108055,'Pandaren - Mage - Remove weapon'),
(8388608,512,108060,'Pandaren - Monk - Remove weapon'),
(8388608,512,166646,'Pandaren - Monk - Windwalking');
/*!40000 ALTER TABLE `playercreateinfo_cast_spell` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

