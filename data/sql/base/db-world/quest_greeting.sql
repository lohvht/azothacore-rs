/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `quest_greeting`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `quest_greeting` (
  `ID` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `Type` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `GreetEmoteType` smallint(5) unsigned NOT NULL DEFAULT 0,
  `GreetEmoteDelay` int(10) unsigned NOT NULL DEFAULT 0,
  `Greeting` text DEFAULT NULL,
  `VerifiedBuild` smallint(5) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`,`Type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `quest_greeting` WRITE;
/*!40000 ALTER TABLE `quest_greeting` DISABLE KEYS */;
INSERT INTO `quest_greeting` VALUES
(344,0,5,0,'Redridge is awash in chaos!',15595),
(392,0,0,0,'Do not be alarmed, $r.  I have long since passed from this land but I intend no harm to your kind.  I have witnessed too much death in my time.  My only wish now is for peace.  Perhaps you can help my cause.',15595),
(900,0,6,0,'What business brings you before the Court of Lakeshire and the Honorable Magistrate Solomon?',15595),
(1500,0,0,0,'I hope you\'re well, all things considered.$B$BSit for a spell, and hear my tale.  It\'s a tragedy, of course, but one I hope will end in revenge!',15595),
(1515,0,0,0,'The Scarlet Crusade is encroaching on our homeland.  The foolish zealots do not realize that the loyal servants of The Dark Lady shall see to their demise.',15595),
(2080,0,1,0,'The creation of Teldrassil was a grand achievement, but now the world must shift to regain its balance.',15595),
(3337,0,0,0,'The heft of an axe, the battlecry of your allies, the spray of blood in your face. These are the things a warrior craves, $n. I will carve out the barrens with my sword in the name of the Horde.',15595),
(3391,0,1,0,'Thrall paid me and my boys well for helping out with the construction of Orgrimmar, so I decided to set up a port here. We do most of our business through Booty Bay and Baron Revilgaz.',15595),
(3519,0,0,0,'I, Arynia Cloudsbreak, have been tasked with protecting the sanctity of the Oracle Grove.',15595),
(3567,0,1,0,'Well met, $n. It is good to see that $cs like yourself are taking an active part in protecting the groves.',15595),
(4791,0,1,0,'We may not be in open war with the Alliance, but blood is still shed between us.',15595),
(5767,0,1,0,'Our only hope is to create something good from an already bad situation.',15595),
(18183,0,0,0,'',19243),
(19309,0,0,0,'',19243),
(24139,0,0,0,'',15595),
(27337,0,0,0,'',15595),
(28444,0,0,0,'',19243),
(28911,0,0,0,'',19243),
(29053,0,0,0,'',19243),
(31082,0,0,0,'',19243),
(34675,0,0,0,'',19342),
(35094,0,0,0,'',15595),
(40109,0,0,0,'',15595),
(43738,0,0,0,'Lord Harris has sent me here to collect reagents. Our work cannot be delayed...',15595),
(45315,0,66,0,'<Onslaught salutes.>',15595),
(48062,0,0,0,'',15595),
(48069,0,6,0,'Yes?',15595),
(48070,0,6,0,'What is it that you want?',15595),
(48071,0,0,0,'',15595),
(48358,0,66,0,'Lok\'tar!',15595),
(48360,0,5,0,'For the Horde!',15595),
(48361,0,25,0,'Hellscream\'s eyes are upon you!',15595),
(48363,0,0,0,'',15595),
(50048,0,0,0,'',19243),
(73097,0,0,0,'They will regret bringing me here alive.',19243),
(74163,0,0,0,'',19243),
(74223,0,0,0,'Many orcs have fallen trying to prove themselves against the might of the gronn. Signs of their passing litter the stones below us.',19243),
(75119,0,0,0,'The Light is screaming for aid.',19342),
(75121,0,0,0,'The Sunsworn will not let Auchindoun fall.',19243),
(75127,0,0,0,'The Iron Horde is a plague upon this land!',19342),
(75177,0,113,0,'There is much to do yet.',19243),
(75392,0,0,0,'The spirits within Auchindoun grow restless. Darkness is near.',19342),
(75896,0,0,0,'It is too late. Aruuna is lost.',19342),
(75913,0,0,0,'Thank you, we could not have escaped without your help.',19342),
(76609,0,0,0,'Throm-Ka, $c. The ogres of Dreadmaul will pay with blood for their foolish actions.',19243),
(76665,0,1,0,'I\'ve been waiting for your arrival, adventurer.',19342),
(77167,0,0,0,'Let\'s do this.',19342),
(77928,0,0,0,'Yo.',19243),
(78323,0,0,0,'You got my back, right?',19243),
(79047,0,0,0,'I owe you a great debt for what you\'ve done this day. I see my debts through. If you have need of me, you have only to call.',19243),
(79281,0,0,0,'',19243),
(79393,0,2,0,'At your service, $n.',19243),
(79618,0,0,0,'',19342),
(79978,0,0,0,'',19243),
(79979,0,0,0,'',19342),
(80001,0,0,0,'',19243),
(80157,0,0,0,'Seeing is not, in fact, believing.',19342),
(80193,0,2,0,'At your service, $n.',19243),
(80389,0,2,0,'At your service, $n.',19243),
(80390,0,2,0,'At your service, $n.',19243),
(80508,0,0,0,'The shadows are restless.',19342),
(80553,0,2,0,'At your service, $n.',19243),
(80648,0,0,0,'Speak, mortal. The raven god is listening.',19342),
(81144,0,0,0,'It took some work getting up here...',19342),
(81361,0,0,0,'It was quite a trial getting up here...',19243),
(81588,0,1,0,'This is all my fault ya know. I gave them dwarves coordinates to where I thought we could get a good listenin\' post on the Blackrock orcs.$b$bI dinna realize just how much the locals could disrupt mole machine navigation.',19342),
(81601,0,0,0,'I canna\' believe them botani were fattening me up just to mulch me.$b$bI may not be a farmer but I gotta expect there are more optimal ways to feed a plant.',19342),
(82126,0,0,0,'My mind... everything is so fuzzy.',19342),
(82274,0,0,0,'The botani wanted to fatten me up! No one fattens Cutter up but Cutter. Nobody!$b$bExcept for maybe Kaz\'s cooking.$b$bBut nobody else!',19243),
(82334,0,0,0,'The botani wanted to fatten me up! No one fattens Cutter up but Cutter. Nobody!$b$bExcept for maybe Kaz\'s cooking.$b$bBut nobody else!',19243),
(82569,0,0,0,'This dwarf didn\'t make it out of Tangleheart alive, but her journal remains behind.',19342),
(82574,0,30,0,'Hey hey, commander! This dwarf I found is provin\' useful already.',19243),
(82575,0,3,0,'Lots o\' lumber to be had in Gorgrond. Always need more, though.',19342),
(82713,0,0,0,'Yeah, yeah, I crashed. Big whoop.',19243),
(82786,0,0,0,'In case you haven\'t noticed, I\'m busy here!',19243),
(83773,0,0,0,'This is no land fer a dwarf.',19342),
(85130,0,11,0,'You stuck it to \'em didn\'t ya, commander?',19342),
(85601,0,0,0,'The botani wanted to fatten me up! No one fattens Cutter up but Cutter. Nobody!$b$bExcept for maybe Kaz\'s cooking.$b$bBut nobody else!',19243),
(86355,0,0,0,'Speak, mortal. The raven god is listening.',19342),
(87391,0,0,0,'I can offer the twisting of time for a variety of prices. Three seals can be obtained this week, but four options I present.  Choose wisely.',19342),
(232400,1,0,0,'Attention, Commander $n. The following notices require your attention.',19342),
(233099,1,0,0,'This poster is a list of bounties put up by citizens of Axefall.',19243),
(233100,1,0,0,'This poster is a list of bounties put up by citizens of Southport.',19342);
/*!40000 ALTER TABLE `quest_greeting` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

