/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `phase_area`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `phase_area` (
  `AreaId` int(10) unsigned NOT NULL,
  `PhaseId` int(10) unsigned NOT NULL,
  `Comment` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`AreaId`,`PhaseId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `phase_area` WRITE;
/*!40000 ALTER TABLE `phase_area` DISABLE KEYS */;
INSERT INTO `phase_area` VALUES
(14,169,'Durotar - always see unphased creatures'),
(14,1164,'Durotar - after quest 31450 complete and before quest 31012 taken'),
(14,1165,'Durotar - after quest 31450 complete and before quest 31012 taken'),
(41,5829,'Dalaran above Karazhan - Legion start quests'),
(4720,169,'Lost Isles default phase'),
(4720,170,'Lost Isles - First Isles - Part 1'),
(4720,171,'Lost Isles - First Isles - Part 2'),
(4720,172,'Lost Isles - First Isles - Part 3'),
(4720,179,'Lost Isles - First Isles - Part 4 Final'),
(4720,180,'Lost Isles - Main Island - Part 1'),
(4720,181,'Lost Isles - Main Island - Part 2'),
(4720,182,'Lost Isles - Main Island - Part 3'),
(4720,183,'Lost Isles - Main Island - Part 4'),
(4720,184,'Lost Isles - Main Island - Part 5'),
(4720,185,'Lost Isles - Main Island - Part 6'),
(4737,169,'Kezan default phase'),
(4737,383,'Kezan Burning 1'),
(4737,384,'Kezan Burning 2'),
(4765,379,'KTC headquarter party'),
(4765,380,'KTC headquarter party pirates'),
(4765,381,'KTC headquarter Gallywix'),
(4765,382,'KTC headquarter Post-Gallywix'),
(5140,169,'Highbank phase after quest 28598 complete'),
(5140,361,'Highbank phase before quest 28598 complete'),
(5424,169,'Obsidian Breakers phase after quest 28598 complete'),
(5424,361,'Obsidian Breakers phase before quest 28598 complete'),
(5736,1323,'The Wandering Isle - before quest 29423 rewarded'),
(5736,1324,'The Wandering Isle - after quest 29423 rewarded and before quest 29679 rewarded'),
(5736,1325,'The Wandering Isle - after quest 29679 rewarded and before quest 29775 rewarded'),
(5736,1326,'The Wandering Isle - after quest 29775 rewarded and before quest 29787 taken'),
(5736,1327,'The Wandering Isle - quest 29787 taken, complete or rewarded'),
(5737,879,'Pei-Wu Forest - after quest 29792 first objective complete and before quest 29792 rewarded'),
(5737,903,'Pei-Wu Forest - after quest 29792 rewarded and before quest 30589 taken'),
(5737,1835,'Pei-Wu Forest - after quest 30589 complete or rewarded and before quest 29798 taken'),
(5820,1027,'Temple of Five Dawns - after quest 29521 completed or rewarded - see Fire spirit'),
(5820,1028,'Temple of Five Dawns - after quest 29775 completed or rewarded - see Water spirit'),
(5820,1029,'Temple of Five Dawns - after quest 29776 completed or rewarded - see Earth spirit'),
(5820,1030,'Temple of Five Dawns - after quest 29791 completed or rewarded - see Air spirit'),
(5825,592,'The Dawning Valley - warrior'),
(5825,593,'The Dawning Valley - mage'),
(5825,594,'The Dawning Valley - hunter'),
(5825,595,'The Dawning Valley - priest'),
(5825,596,'The Dawning Valley - rogue'),
(5825,597,'The Dawning Valley - shaman'),
(5825,598,'The Dawning Valley - monk'),
(5825,629,'The Dawning Valley - before quest 29419 complete'),
(5825,630,'The Dawning Valley - after quest 29419 complete'),
(5826,964,'The Singing Pools - after quest 29663 complete and before quest 29676 rewarded'),
(5827,1510,'The Rows - quest 29768 taken and before complete'),
(5828,878,'Mandori Village - quest 29792 taken'),
(5828,879,'Mandori Village - quest 29792 complete'),
(5829,524,'Ridge of Laughing Winds - quest 29786 taken'),
(5829,536,'Ridge of Laughing Winds - after quest 29786 complete or rewarded'),
(5829,1430,'Ridge of Laughing Winds - after quest 29776 rewarded and quest 29786 not taken, complete or rewarded'),
(5829,1526,'Ridge of Laughing Winds - after quest 29786 complete or rewarded'),
(5829,1836,'Ridge of Laughing Winds - before quest 29785 complete or rewarded'),
(5830,1430,'Morning Breeze Village - after quest 29776 rewarded and quest 29786 not taken, complete or rewarded'),
(5830,1518,'Morning Breeze Village - after quest 29776 rewarded and before quest 29786 taken, complete or rewarded'),
(5830,1519,'Morning Breeze Village - x'),
(5830,1523,'Morning Breeze Village - quest 29780 rewarded'),
(5830,1836,'Morning Breeze Village - before quest 29785 complete or rewarded'),
(5831,1430,'Morning Breeze Village - after quest 29776 rewarded and quest 29786 not taken, complete or rewarded'),
(5831,1714,'Fe-Feng Village - before quest 29782 complete or rewarded'),
(5832,1527,'The Wood of Staves - after quest 29787 rewarded and before quest 29790 taken'),
(5832,1885,'The Wood of Staves - before quest 29791 completed or rewarded'),
(5833,543,'Wreck of the Skyseeker - after quest 29798 available'),
(5833,544,'Wreck of the Skyseeker - after quest 30767 complete or rewarded'),
(5833,545,'Wreck of the Skyseeker - after quest 29799 rewarded'),
(5833,993,'Wreck of the Skyseeker - after quest 29798 complete (delayed) and before quest 30767 complete'),
(5833,1835,'Wreck of the Skyseeker - after quest 30589 complete or rewarded and before quest 29798 taken'),
(5834,592,'Shang Xi Training Grounds - warrior'),
(5834,593,'Shang Xi Training Grounds - mage'),
(5834,594,'Shang Xi Training Grounds - hunter'),
(5834,595,'Shang Xi Training Grounds - priest'),
(5834,596,'Shang Xi Training Grounds - rogue'),
(5834,597,'Shang Xi Training Grounds - shaman'),
(5834,598,'Shang Xi Training Grounds - monk'),
(5837,1429,'The Dawning Span - before quest 29776 rewarded'),
(5837,1430,'The Dawning Span - after quest 29776 rewarded and quest 29786 not taken, complete or rewarded'),
(5843,592,'Tranquil Grotto - warrior'),
(5843,593,'Tranquil Grotto - mage'),
(5843,594,'Tranquil Grotto - hunter'),
(5843,595,'Tranquil Grotto - priest'),
(5843,596,'Tranquil Grotto - rogue'),
(5843,597,'Tranquil Grotto - shaman'),
(5843,598,'Tranquil Grotto - monk'),
(5848,838,'Cave of Meditation - quest 29414 taken'),
(5849,504,'Shrine of Inner-Light - quest 29421 taken'),
(5849,631,'Shrine of Inner-Light - quest 29422 taken'),
(5849,632,'Shrine of Inner-Light - after quest 29422 complete and before quest 29423 taken'),
(5859,1527,'The Elders\' Path - after quest 29787 rewarded and before quest 29790 taken'),
(5859,1885,'The Elders\' Path - before quest 29791 completed or rewarded'),
(5860,964,'Liang\'s Retreat - after quest 29663 complete and before quest 29676 rewarded'),
(5881,1827,'The Dai-Lo Farmstead - before quest 29774 complete'),
(5884,1510,'Ki-Han Brewery - quest 29768 taken and before complete'),
(5886,524,'Chamber of Whispers - quest 29786 taken'),
(5886,1430,'Chamber of Whispers - after quest 29776 rewarded and quest 29786 not taken, complete or rewarded'),
(5886,1836,'Chamber of Whispers - before quest 29785 complete or rewarded'),
(5944,903,'Forlorn Hut - after quest 29792 rewarded and before quest 30589 taken'),
(5946,1430,'Morning Breeze Village - after quest 29776 rewarded and quest 29786 not taken, complete or rewarded'),
(5946,1518,'Morning Breeze Lake - after quest 29776 rewarded and before quest 29786 taken, complete or rewarded'),
(5946,1519,'Morning Breeze Lake - x'),
(5946,1523,'Morning Breeze Lake - quest 29780 rewarded'),
(7025,169,'Tanaan Default Phase'),
(7705,169,'Mardum Default Phase');
/*!40000 ALTER TABLE `phase_area` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

