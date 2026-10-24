/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `playerchoice_response_locale`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `playerchoice_response_locale` (
  `ChoiceId` int(11) NOT NULL,
  `ResponseId` int(11) NOT NULL,
  `locale` varchar(4) NOT NULL,
  `Header` varchar(511) NOT NULL DEFAULT '',
  `Answer` varchar(511) NOT NULL DEFAULT '',
  `Description` varchar(2047) NOT NULL DEFAULT '',
  `Confirmation` varchar(127) NOT NULL DEFAULT '',
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ChoiceId`,`ResponseId`,`locale`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `playerchoice_response_locale` WRITE;
/*!40000 ALTER TABLE `playerchoice_response_locale` DISABLE KEYS */;
INSERT INTO `playerchoice_response_locale` VALUES
(231,478,'frFR','Dévastation','Dévastation','Conserver votre spécialisation : Dévastation.\n\nLes chasseurs de démons Dévastation peuvent utiliser Métamorphose pour se transformer en démons ailés. Ils se concentrent sur les dégâts en mêlée.\n\nChoisir Dévastation confère |cFFFF0000|Hspell:162264|h[Métamorphose]|h|r et |cFFFF0000|Hspell:185164|h[Maîtrise : Présence démoniaque]|h|r.','',25549),
(231,479,'frFR','Vengeance','Vengeance','Changer de spécialisation : Vengeance.\n\nLes chasseurs de démons Vengeance peuvent utiliser Métamorphose pour se transformer en démons couverts de pointes. Ils se concentrent sur le rôle de tank.\n\nChoisir Vengeance confère |cFFFF0000|Hspell:187827|h[Métamorphose]|h|r et |cFFFF0000|Hspell:203747|h[Maîtrise : Gangresang]|h|r.\n\nVotre technique |cFFFF0000|Hspell:193921|h[Rayon accablant]|h|r est remplacée par |cFFFF0000|Hspell:203720|h[Pointes démoniaques]|h|r.','',25549);
/*!40000 ALTER TABLE `playerchoice_response_locale` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

