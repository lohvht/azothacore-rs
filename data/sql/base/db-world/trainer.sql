/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `trainer`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `trainer` (
  `Id` int(10) unsigned NOT NULL DEFAULT 0,
  `Type` tinyint(2) unsigned NOT NULL DEFAULT 2,
  `Greeting` text DEFAULT NULL,
  `VerifiedBuild` smallint(5) DEFAULT 0,
  PRIMARY KEY (`Id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `trainer` WRITE;
/*!40000 ALTER TABLE `trainer` DISABLE KEYS */;
INSERT INTO `trainer` VALUES
(10,2,'I can teach you how to use a fishing pole to catch fish.',26124),
(24,0,'Welcome!',24015),
(27,2,'Care to learn how to turn the ore that you find into weapons and metal armor?',26124),
(29,2,'Greetings!  Can I teach you how to cut precious gems and craft jewelry?',26124),
(37,2,'Care to learn how to turn the ore that you find into weapons and metal armor?',25996),
(40,0,'Hello, hunter!  Ready for some training?',23420),
(46,0,'Hello!  Can I teach you something?',26124),
(48,2,'Greetings!  Can I teach you how to cut precious gems and craft jewelry?',26124),
(51,2,'Enchanting is the art of improving existing items through magic. ',26124),
(56,2,'Greetings!  Can I teach you how to turn beast hides into armor?',26124),
(59,2,'With alchemy you can turn found herbs into healing and other types of potions.',26124),
(62,2,'Enchanting is the art of improving existing items through magic. ',26124),
(63,2,'Would you like to learn the intricacies of inscription?',26124),
(80,2,'Care to learn how to turn the ore that you find into weapons and metal armor?',26124),
(91,2,'You have not lived until you have dug deep into the earth.',26124),
(102,2,'Engineering is very simple once you grasp the basics.',26124),
(103,2,'Greetings!  Can I teach you how to turn beast hides into armor?',26124),
(105,2,'With alchemy you can turn found herbs into healing and other types of potions.',26124),
(114,0,'Welcome!',26822),
(117,2,'Greetings!  Can I teach you how to turn found cloth into cloth armor?',26124),
(122,2,'The herbs of Northrend can be brewed into powerful potions.',26124),
(125,2,'Enchanting is the art of improving existing items through magic. ',24015),
(129,0,'Welcome!',25961),
(133,2,'Searching for herbs requires both knowledge and instinct.',26124),
(136,2,'Can I teach you how to turn the meat you find on beasts into a feast?',26124),
(137,2,'Hello!  Can I teach you something?',23420),
(144,0,'Welcome!',25996),
(148,0,'Welcome!',25881),
(149,0,'Welcome!',26124),
(157,2,'Greetings!  Can I teach you how to turn found cloth into cloth armor?',25996),
(160,2,'Here, let me show you how to bind those wounds....',26124),
(163,2,'Greetings!  Can I teach you how to turn found cloth into cloth armor?',26124),
(196,2,'It requires a steady hand to remove the leather from a slain beast.',26124),
(373,2,'Hi.',26124),
(386,2,'Do you wish to learn how to fly?',26124),
(387,2,'Would you like to learn the intricacies of inscription?',26124),
(388,2,'Searching for herbs requires both knowledge and instinct.',26124),
(389,2,'You have not lived until you have dug deep into the earth.',26124),
(390,2,'It requires a steady hand to remove the leather from a slain beast.',26124),
(405,2,'Engineering is very simple once you grasp the basics.',23420),
(406,2,'Engineering is very simple once you grasp the basics.',26822),
(407,2,'Engineering is very simple once you grasp the basics.',26124),
(424,2,'Searching for herbs requires both knowledge and instinct.',24015),
(554,2,'Hello!  Can I teach you something?',24015),
(580,0,'No greeting.',26124),
(582,2,'Test - greeting',23420),
(608,2,'Greetings!  Can I teach you how to cut precious gems and craft jewelry?',23420),
(695,2,'Care to learn how to turn the ore that you find into weapons and metal armor?',23420),
(774,2,'Greetings! I specialize in cloakweaving. Would you like to train?',23420),
(783,2,'Hi.',23420),
(786,2,'Would you like to learn the intricacies of inscription?',24015),
(789,2,'Would you like to learn the intricacies of inscription?',24015),
(790,2,'Would you like to learn the intricacies of inscription?',23420),
(791,2,'Would you like to learn the intricacies of inscription?',25881);
/*!40000 ALTER TABLE `trainer` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

