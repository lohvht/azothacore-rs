/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `playerchoice_response_reward`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `playerchoice_response_reward` (
  `ChoiceId` int(11) NOT NULL,
  `ResponseId` int(11) NOT NULL,
  `TitleId` int(11) NOT NULL DEFAULT 0,
  `PackageId` int(11) NOT NULL DEFAULT 0,
  `SkillLineId` int(10) unsigned NOT NULL DEFAULT 0,
  `SkillPointCount` int(10) unsigned NOT NULL DEFAULT 0,
  `ArenaPointCount` int(10) unsigned NOT NULL DEFAULT 0,
  `HonorPointCount` int(10) unsigned NOT NULL DEFAULT 0,
  `Money` bigint(20) unsigned NOT NULL DEFAULT 0,
  `Xp` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ChoiceId`,`ResponseId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `playerchoice_response_reward` WRITE;
/*!40000 ALTER TABLE `playerchoice_response_reward` DISABLE KEYS */;
/*!40000 ALTER TABLE `playerchoice_response_reward` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

