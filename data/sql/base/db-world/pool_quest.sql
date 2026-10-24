/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `pool_quest`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `pool_quest` (
  `entry` int(10) unsigned NOT NULL DEFAULT 0,
  `pool_entry` mediumint(8) unsigned NOT NULL DEFAULT 0,
  `description` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`entry`),
  KEY `idx_guid` (`entry`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `pool_quest` WRITE;
/*!40000 ALTER TABLE `pool_quest` DISABLE KEYS */;
INSERT INTO `pool_quest` VALUES
(11377,359,'Revenge is Tasty'),
(11379,359,'Super Hot Stew'),
(11380,359,'Manalicious'),
(11381,359,'Soup for the Soul'),
(11665,360,'Crocolisks in the City'),
(11666,360,'Bait Bandits'),
(11667,360,'The One That Got Away'),
(11668,360,'Shrimpin\' Ain\'t Easy'),
(11669,360,'Felblood Fillet'),
(12958,363,'Shipment: Blood Jade Amulet'),
(12959,363,'Shipment: Glowing Ivory Figurine'),
(12960,363,'Shipment: Wicked Sun Brooch'),
(12961,363,'Shipment: Intricate Bone Figurine'),
(12962,363,'Shipment: Bright Armor Relic'),
(12963,363,'Shipment: Shifting Sun Curio'),
(13100,362,'Infused Mushroom Meatloaf'),
(13101,362,'Convention at the Legerdemain'),
(13102,362,'Sewer Stew'),
(13103,362,'Cheese for Glowergold'),
(13107,362,'Mustard Dogs!'),
(13830,361,'The Ghostfish'),
(13832,361,'Jewel Of The Sewers'),
(13833,361,'Blood Is Thicker'),
(13834,361,'Dangerously Delicious'),
(13836,361,'Disarmed!'),
(25105,348,'Nibbler! No!'),
(25154,348,'A Present for Lila'),
(25155,348,'Ogrezonians in the Mood'),
(25156,348,'Elemental Goo'),
(25157,348,'The Latest Fashion!'),
(25158,349,'Nibbler! No!'),
(25159,349,'The Latest Fashion!'),
(25160,349,'A Present for Lila'),
(25161,349,'Ogrezonians in the Mood'),
(25162,349,'Elemental Goo'),
(26153,351,'Penny\'s Pumpkin Pancakes'),
(26177,351,'Feeling Crabby?'),
(26183,351,'Rock Lobster'),
(26190,351,'A Fisherman\'s Feast'),
(26192,351,'Orphans Like Cookies Too!'),
(26414,350,'Hitting a Walleye'),
(26420,350,'Diggin\' For Worms'),
(26442,350,'Rock Lobster'),
(26488,350,'Big Gulp'),
(26536,350,'Thunder Falls'),
(26543,347,'Clammy Hands'),
(26556,347,'No Dumping Allowed'),
(26557,347,'A Staggering Effort'),
(26572,347,'A Golden Opportunity'),
(26588,347,'A Furious Catch'),
(29313,357,'The Secret to Perfect Kimchi'),
(29314,357,'Remembering the Ancestors'),
(29315,355,'Fungus Among Us'),
(29316,357,'Back to Basics'),
(29317,356,'Fish Head'),
(29318,357,'Ribs for the Sentinels'),
(29319,356,'Tadpole Terror'),
(29320,356,'Like Pike?'),
(29321,358,'Happy as a Clam Digger'),
(29322,356,'Time for Slime'),
(29323,358,'Stocking Up'),
(29324,358,'The Sister\'s Pendant'),
(29325,358,'A Slippery Snack'),
(29332,355,'Lily, Oh Lily'),
(29333,355,'Escargot A Go-Go'),
(29334,355,'Roach Coach'),
(29342,353,'Cold Water Fishing'),
(29343,353,'One fer the Ages'),
(29344,353,'Fish fer Squrky'),
(29345,346,'Pond Predators'),
(29346,346,'The Rings the Thing'),
(29347,353,'Live Bait'),
(29348,346,'The Race to Restock'),
(29349,346,'Craving Crayfish'),
(29350,353,'The Gnomish Bait-o-Matic'),
(29351,352,'A Round for the Guards'),
(29352,352,'A Fowl Shortage'),
(29353,352,'Keepin\' the Haggis Flowin\''),
(29354,346,'Shiny Baubles'),
(29355,352,'Can\'t Get Enough Spice Bread'),
(29356,352,'I Need to Cask a Favor'),
(29357,357,'Spice Bread Aplenty'),
(29358,354,'Pining for Nuts'),
(29359,358,'An Old Favorite'),
(29360,355,'Would You Like Some Flies With That?'),
(29361,356,'Moat Monster!'),
(29362,354,'\'Magic\' Mushrooms'),
(29363,354,'Mulgore Spice Bread'),
(29364,354,'Corn Mash'),
(29365,354,'Perfectly Picked Portions'),
(31332,364,'Lesson 1: Sliced Peaches'),
(31333,364,'Lesson 2: Instant Noodles'),
(31334,364,'Lesson 3: Toasted Fish Jerky'),
(31335,364,'Lesson 4: Dried Needle Mushrooms'),
(31336,364,'Lesson 5: Pounded Rice Cake'),
(31337,364,'A Token of Appreciation');
/*!40000 ALTER TABLE `pool_quest` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

