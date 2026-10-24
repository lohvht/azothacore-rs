/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `gossip_menu_option_locale`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `gossip_menu_option_locale` (
  `MenuId` int(10) unsigned NOT NULL DEFAULT 0,
  `OptionIndex` int(10) unsigned NOT NULL DEFAULT 0,
  `Locale` varchar(4) NOT NULL,
  `OptionText` text DEFAULT NULL,
  `BoxText` text DEFAULT NULL,
  PRIMARY KEY (`MenuId`,`OptionIndex`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `gossip_menu_option_locale` WRITE;
/*!40000 ALTER TABLE `gossip_menu_option_locale` DISABLE KEYS */;
INSERT INTO `gossip_menu_option_locale` VALUES
(5853,0,'ruRU','Обучи меня.',''),
(5855,0,'ruRU','Обучи меня.',''),
(10841,0,'ruRU','Можно мне взглянуть на товар?',''),
(11741,0,'frFR','Larosse, j\'ai besoin de te demander quelque chose de génial. J\'ai besoin de te couper la tête pour prouver ma loyauté envers les Boucaniers de la Voile sanglante, alors ils vont me permettre d\'infiltrer leurs rangs et, je l\'espère, de découvrir leurs vraies intentions.\r\n\r\nBaie du Butin a besoin de toi, Bossy ... MAINTENANT plus que jamais.',NULL),
(11741,1,'frFR','Meuuh.',NULL),
(12180,11,'ruRU','Расскажи мне подробнее о профессиях, связанных с собирательством и добычей ресурсов.',''),
(12180,12,'ruRU','Расскажи мне подробнее о ремесленных профессиях.',''),
(12185,11,'ruRU','Расскажи мне о травничестве.',''),
(12185,12,'ruRU','Расскажи мне о горном деле.',''),
(12185,13,'ruRU','Расскажи мне о снятии шкур.',''),
(12185,14,'ruRU','Расскажи мне подробнее о ремесленных профессиях.',''),
(12188,11,'ruRU','Обучи меня травничеству.',''),
(12188,12,'ruRU','Расскажи мне об алхимии.',''),
(12188,13,'ruRU','Расскажи мне о начертании.',''),
(12188,14,'ruRU','Расскажи мне подробнее о профессиях, связанных с собирательством и добычей ресурсов.',''),
(12188,15,'ruRU','Расскажи мне подробнее о ремесленных профессиях.',''),
(20457,1,'frFR','Je suis prêt, commencez la téléportation de Dalaran vers les îles Brisées.',NULL),
(63908,1,'frFR','Niuzao... nous devons le sauver !',NULL);
/*!40000 ALTER TABLE `gossip_menu_option_locale` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

