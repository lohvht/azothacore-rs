/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `playerchoice_response`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `playerchoice_response` (
  `ChoiceId` int(11) NOT NULL,
  `ResponseId` int(11) NOT NULL,
  `Index` int(10) unsigned NOT NULL,
  `ChoiceArtFileId` int(11) NOT NULL DEFAULT 0,
  `Header` varchar(511) NOT NULL,
  `Answer` varchar(511) NOT NULL,
  `Description` varchar(2047) NOT NULL,
  `Confirmation` varchar(127) NOT NULL,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ChoiceId`,`ResponseId`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `playerchoice_response` WRITE;
/*!40000 ALTER TABLE `playerchoice_response` DISABLE KEYS */;
INSERT INTO `playerchoice_response` VALUES
(231,478,0,1274664,'Devastation','Devastation','Continue your mastery of Havoc.\n\nHavos specialized demon hunters can use Metamorphosis to turn into winged demons. They focus on Melee DPS.\n\nChoosing Havoc grants |cFFFF0000|Hspell:162264|h[Metamorphosis]|h|r as well as |cFFFF0000|Hspell:185164|h[Mastery: Furious Havoc]|h|r.','',25549),
(231,479,1,1274665,'Vengeance','Vengeance','Change to mastery of Vengeance.\n\nVengeance specialized demon hunters can use Metamorphosis to turn into spiked demons. They focus on Tanking.\n\nChoosing Vengeance grants |cFFFF0000|Hspell:187827|h[Metamorphosis]|h|r as well as |cFFFF0000|Hspell:203747|h[Mastery: Fel Blood]|h|r.\n\nYour |cFFFF0000|Hspell:193921|h[Eye Beam]|h|r will change into |cFFFF0000|Hspell:203720|h[Demon Spikes]|h|r.','',25549);
/*!40000 ALTER TABLE `playerchoice_response` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

