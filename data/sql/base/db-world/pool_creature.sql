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
(52323,1107,0,'Bro\'Gaz the Clanless - Spawn 1'),
(52324,1107,0,'Bro\'Gaz the Clanless - Spawn 2'),
(52325,1107,0,'Bro\'Gaz the Clanless - Spawn 3'),
(52326,1107,0,'Bro\'Gaz the Clanless - Spawn 4'),
(52355,368,0,'Eredar Soul-Eater - Spawn 1'),
(52356,368,0,'Eredar Soul-Eater - Spawn 2'),
(54652,1211,0,'Meshlok the Harvester placeholder (Maraudon)'),
(63724,11637,0,'Defenders at Bloodmyst Isle 3'),
(63725,11635,0,'Defenders at Bloodmyst Isle 1'),
(63726,11636,0,'Defenders at Bloodmyst Isle 2'),
(63727,11638,0,'Defenders at Bloodmyst Isle 4'),
(72770,60003,0,'Lake Frog (33224) - Spawn 1'),
(72771,60003,0,'Lake Frog (33224) - Spawn 2'),
(72772,60003,0,'Lake Frog (33224) - Spawn 3'),
(72773,60003,0,'Lake Frog (33224) - Spawn 4'),
(72774,60003,0,'Lake Frog (33224) - Spawn 5'),
(72775,60003,0,'Lake Frog (33224) - Spawn 6'),
(83351,1506,0,'Auchenai Vindicator - Group 7'),
(83352,1505,0,'Auchenai Soulpriest - Group 6'),
(83353,1504,0,'Auchenai Soulpriest - Group 5'),
(83354,1505,0,'Auchenai Vindicator - Group 6'),
(83355,1504,0,'Auchenai Vindicator - Group 5'),
(83356,1506,0,'Auchenai Soulpriest - Group 7'),
(83357,1507,0,'Auchenai Soulpriest - Group 8'),
(83358,1501,0,'Auchenai Soulpriest - Group 2'),
(83359,1501,0,'Auchenai Vindicator - Group 2'),
(83360,1503,0,'Auchenai Soulpriest - Group 4'),
(83361,1502,0,'Auchenai Soulpriest - Group 3'),
(83362,1502,0,'Auchenai Vindicator - Group 3'),
(83363,1500,0,'Auchenai Soulpriest - Group 1'),
(83364,1503,0,'Auchenai Vindicator - Group 4'),
(83365,1500,0,'Auchenai Vindicator - Group 1'),
(83366,1507,0,'Auchenai Vindicator - Group 8'),
(83367,1508,0,'Auchenai Vindicator - Group 9'),
(83368,1508,0,'Auchenai Soulpriest - Group 9'),
(83370,1509,0,'Auchenai Vindicator - Group 10'),
(83371,1509,0,'Auchenai Soulpriest - Group 10'),
(84395,11636,0,'Defenders at Bloodmyst Isle 2'),
(84396,11637,0,'Defenders at Bloodmyst Isle 3'),
(84397,11635,0,'Defenders at Bloodmyst Isle 1'),
(84428,11638,0,'Defenders at Bloodmyst Isle 4'),
(85382,1073,25,'Okrek - Spawnlocation 1'),
(85405,1073,25,'Okrek - Spawnlocation 2'),
(85564,1073,25,'Okrek - Spawnlocation 3'),
(85565,1073,25,'Okrek - Spawnlocation 4'),
(87572,369,0,'Ethereum Jailor'),
(87573,369,0,'Ethereum Jailor'),
(87574,369,0,'Ethereum Jailor'),
(87575,369,0,'Ethereum Jailor'),
(87576,369,0,'Ethereum Jailor'),
(87577,369,0,'Ethereum Jailor'),
(87578,369,0,'Ethereum Jailor'),
(87579,369,0,'Ethereum Jailor'),
(87580,369,0,'Ethereum Jailor'),
(87581,369,0,'Ethereum Jailor'),
(87582,369,0,'Ethereum Jailor'),
(87583,369,0,'Ethereum Jailor'),
(87584,369,0,'Ethereum Jailor'),
(87585,369,0,'Ethereum Jailor'),
(87586,369,0,'Ethereum Jailor'),
(87587,369,0,'Ethereum Jailor'),
(87588,369,0,'Ethereum Jailor'),
(87589,369,0,'Ethereum Jailor'),
(87590,369,0,'Ethereum Jailor'),
(87591,369,0,'Ethereum Jailor'),
(87592,369,0,'Ethereum Jailor'),
(151826,7001,0,'Webbed Crusader Spawn 1'),
(151827,7002,0,'Webbed Crusader Spawn 2'),
(151828,7003,0,'Webbed Crusader Spawn 3'),
(151829,7004,0,'Webbed Crusader Spawn 4'),
(151830,7005,0,'Webbed Crusader Spawn 5'),
(151831,7006,0,'Webbed Crusader Spawn 6'),
(151832,7007,0,'Webbed Crusader Spawn 7'),
(151833,7008,0,'Webbed Crusader Spawn 8'),
(151834,7009,0,'Webbed Crusader Spawn 9'),
(151835,7010,0,'Webbed Crusader Spawn 10'),
(151836,7011,0,'Webbed Crusader Spawn 11'),
(151837,7012,0,'Webbed Crusader Spawn 12'),
(151838,7013,0,'Webbed Crusader Spawn 13'),
(151839,7014,0,'Webbed Crusader Spawn 14'),
(151895,1074,20,'Ambassador Jerrikar - Spawnlocation 1'),
(151896,1074,20,'Ambassador Jerrikar - Spawnlocation 2'),
(151897,1074,20,'Ambassador Jerrikar - Spawnlocation 3'),
(151898,1074,20,'Ambassador Jerrikar - Spawnlocation 4'),
(151899,1074,20,'Ambassador Jerrikar - Spawnlocation 5'),
(151900,1075,33,'Chief Engineer Lorthander - Spawnlocation 1'),
(151901,1075,33,'Chief Engineer Lorthander - Spawnlocation 2'),
(151902,1075,34,'Chief Engineer Lorthander - Spawnlocation 3'),
(151903,1076,100,'Crippler - Spawnlocation 1'),
(151909,1078,25,'Ever-Core the Punisher Spawnlocation 1'),
(151910,1078,25,'Ever-Core the Punisher Spawnlocation 2'),
(151911,1078,25,'Ever-Core the Punisher Spawnlocation 3'),
(151912,1078,25,'Ever-Core the Punisher Spawnlocation 4'),
(151913,1079,0,'Fulgore Spawnlocation 1'),
(151914,1079,0,'Fulgore Spawnlocation 2'),
(151915,1080,25,'Goretooth Spawnlocation 1'),
(151916,1080,25,'Goretooth Spawnlocation 2'),
(151917,1080,25,'Goretooth Spawnlocation 3'),
(151918,1080,25,'Goretooth Spawnlocation 4'),
(151919,1081,50,'Hemathion Spawnlocation 1'),
(151920,1081,50,'Hemathion Spawnlocation 2'),
(151921,1082,50,'Mekthorg the Wild Spawnlocation 1'),
(151922,1082,50,'Mekthorg the Wild Spawnlocation 2'),
(151923,1083,20,'Speaker Margrom Spawnlocation 1'),
(151924,1083,20,'Speaker Margrom Spawnlocation 2'),
(151925,1083,20,'Speaker Margrom Spawnlocation 3'),
(151926,1083,20,'Speaker Margrom Spawnlocation 4'),
(151927,1083,20,'Speaker Margrom Spawnlocation 5'),
(151928,1084,100,'Voidhunter Yar Spawnlocation 1'),
(151929,1085,25,'Vorakem Doomspeaker Spawnlocation 1'),
(151930,1085,25,'Vorakem Doomspeaker Spawnlocation 2'),
(151931,1085,25,'Vorakem Doomspeaker Spawnlocation 3'),
(151932,1085,25,'Vorakem Doomspeaker Spawnlocation 4'),
(151933,1079,0,'Fulgore Spawnlocation 2'),
(151934,1086,0,'Old Crystalbark - Spawnlocation 1'),
(151935,1086,0,'Old Crystalbark - Spawnlocation 2'),
(151936,1086,0,'Old Crystalbark - Spawnlocation 3'),
(151937,1086,0,'Old Crystalbark - Spawnlocation 4'),
(151938,1087,100,'Fumblub Gearwind - Spawnlocation 1'),
(151939,1088,0,'Icehorn - Spawnlocation 1'),
(151940,1088,0,'Icehorn Spawnlocation 2'),
(151941,1088,0,'Icehorn Spawnlocation 3'),
(151942,1089,100,'Crazed Indu le Survivor - Spawnlocation 1'),
(151943,1090,0,'Scarlet Highlord Daion Spawnlocation 1'),
(151944,1090,0,'Scarlet Highlord Daion Spawnlocation 2'),
(151945,1090,0,'Scarlet Highlord Daion Spawnlocation 3'),
(151946,1090,0,'Scarlet Highlord Daion Spawnlocation 4'),
(151947,1091,0,'Perobas the Bloodthirster Spawnlocation 1'),
(151948,1091,0,'Perobas the Bloodthirster Spawnlocation 2'),
(151949,1091,0,'Perobas the Bloodthirster Spawnlocation 3'),
(151950,1092,0,'Vigdis the War Maiden Spawnlocation 1'),
(151951,1092,0,'Vigdis the War Maiden Spawnlocation 2'),
(151952,1092,0,'Vigdis the War Maiden Spawnlocation 3'),
(151953,1092,0,'Vigdis the War Maiden Spawnlocation 4'),
(151954,1092,0,'Vigdis the War Maiden Spawnlocation 5'),
(151955,1092,0,'Vigdis the War Maiden Spawnlocation 6'),
(151956,1093,0,'King Pin Spawnlocation 1'),
(151957,1093,0,'King Pin Spawnlocation 2'),
(151958,1093,0,'King Pin Spawnlocation 3'),
(151959,1093,0,'King Pin Spawnlocation 4'),
(151960,1093,0,'King Pin Spawnlocation 5'),
(151961,1094,0,'Tukemuth Spawnlocation 1'),
(151962,1094,0,'Tukemuth Spawnlocation 2'),
(151963,1094,0,'Tukemuth Spawnlocation 3'),
(151964,1094,0,'Tukemuth Spawnlocation 4'),
(151965,1094,0,'Tukemuth Spawnlocation 5'),
(151966,1094,0,'Tukemuth Spawnlocation 6'),
(151967,1094,0,'Tukemuth Spawnlocation 7'),
(151968,1094,0,'Tukemuth Spawnlocation 8'),
(151969,1095,0,'Grocklar Spawnlocation 1'),
(151970,1095,0,'Grocklar Spawnlocation 2'),
(151971,1095,0,'Grocklar Spawnlocation 3'),
(151972,1095,0,'Grocklar Spawnlocation 4'),
(151973,1095,0,'Grocklar Spawnlocation 5'),
(151974,1095,0,'Grocklar Spawnlocation 6'),
(151975,1096,0,'Seething Hate Spawnlocation 1'),
(151976,1096,0,'Seething Hate Spawnlocation 2'),
(151977,1096,0,'Seething Hate Spawnlocation 3'),
(151978,1097,0,'Syreian the Bonecarver Spawnlocation 1'),
(151979,1097,0,'Syreian the Bonecarver Spawnlocation 2'),
(151980,1097,0,'Syreian the Bonecarver Spawnlocation 3'),
(151981,1097,0,'Syreian the Bonecarver Spawnlocation 4'),
(151982,1097,0,'Syreian the Bonecarver Spawnlocation 5'),
(151983,1097,0,'Syreian the Bonecarver Spawnlocation 6'),
(151984,1098,0,'Hildana Deathstealer Spawnlocation 1'),
(151985,1098,0,'Hildana Deathstealer Spawnlocation 2'),
(151986,1098,0,'Hildana Deathstealer Spawnlocation 3'),
(151987,1098,0,'Hildana Deathstealer Spawnlocation 4'),
(151988,1099,0,'High Thane Jorfus Spawnlocation 1'),
(151989,1099,0,'High Thane Jorfus Spawnlocation 2'),
(151990,1099,0,'High Thane Jorfus Spawnlocation 3'),
(151991,1100,0,'Terror Spinner Spawnlocation 1'),
(151992,1100,0,'Terror Spinner Spawnlocation 2'),
(151993,1100,0,'Terror Spinner Spawnlocation 3'),
(151995,1101,0,'Griegen Spawnlocation 1'),
(151996,1101,0,'Griegen Spawnlocation 2'),
(151997,1101,0,'Griegen Spawnlocation 3'),
(151998,1101,0,'Griegen Spawnlocation 4'),
(151999,1101,0,'Griegen Spawnlocation 5'),
(152000,1101,0,'Griegen Spawnlocation 6'),
(152001,1101,0,'Griegen Spawnlocation 7'),
(152002,1102,0,'King Krush Spawnlocation 1'),
(152003,1102,0,'King Krush Spawnlocation 2'),
(152004,1103,0,'Aotona Spawnlocation 1'),
(152005,1103,0,'Aotona Spawnlocation 2'),
(152006,1103,0,'Aotona Spawnlocation 3'),
(152007,1103,0,'Aotona Spawnlocation 4'),
(152008,1103,0,'Aotona Spawnlocation 5'),
(152009,1103,0,'Aotona Spawnlocation 6'),
(152010,1104,0,'Dirkee Spawnlocation 1'),
(152011,1104,0,'Dirkee Spawnlocation 2'),
(152012,1104,0,'Dirkee Spawnlocation 3'),
(152013,1104,0,'Dirkee Spawnlocation 4'),
(152014,1105,0,'Putridus the Ancient Spawnlocation 1'),
(152015,1105,0,'Putridus the Ancient Spawnlocation 2'),
(152016,1105,0,'Putridus the Ancient Spawnlocation 3'),
(152017,1105,0,'Putridus the Ancient Spawnlocation 4'),
(152018,1105,0,'Putridus the Ancient Spawnlocation 5'),
(152019,1106,0,'Zul Drak Sentinel Spawnlocation 1'),
(152020,1106,0,'Zul Drak Sentinel Spawnlocation 2'),
(152168,4993,0,'Arctic Cloud - Stormpikes'),
(152169,4993,0,'Arctic Cloud - Stormpikes'),
(152170,4993,0,'Arctic Cloud - Stormpikes'),
(152171,4993,0,'Arctic Cloud - Stormpikes'),
(152172,4993,0,'Arctic Cloud - Stormpikes'),
(152173,4993,0,'Arctic Cloud - Stormpikes'),
(152174,4993,0,'Arctic Cloud - Stormpikes'),
(152175,4993,0,'Arctic Cloud - Stormpikes'),
(152176,4993,0,'Arctic Cloud - Stormpikes'),
(152177,4993,0,'Arctic Cloud - Stormpikes'),
(152178,4993,0,'Arctic Cloud - Stormpikes'),
(152179,4993,0,'Arctic Cloud - Stormpikes'),
(152180,4993,0,'Arctic Cloud - Stormpikes'),
(152181,4993,0,'Arctic Cloud - Stormpikes'),
(152182,4993,0,'Arctic Cloud - Stormpikes'),
(152183,4993,0,'Arctic Cloud - Stormpikes'),
(152184,4993,0,'Arctic Cloud - Stormpikes'),
(152185,4993,0,'Arctic Cloud - Stormpikes'),
(152186,4993,0,'Arctic Cloud - Stormpikes'),
(152187,4993,0,'Arctic Cloud - Stormpikes'),
(152188,4994,0,'Arctic Cloud - Dragonsblight'),
(152189,4994,0,'Arctic Cloud - Dragonsblight'),
(152190,4994,0,'Arctic Cloud - Dragonsblight'),
(152191,4994,0,'Arctic Cloud - Dragonsblight'),
(152192,4994,0,'Arctic Cloud - Dragonsblight'),
(152193,4994,0,'Arctic Cloud - Dragonsblight'),
(152194,4994,0,'Arctic Cloud - Dragonsblight'),
(152195,4994,0,'Arctic Cloud - Dragonsblight'),
(152196,4994,0,'Arctic Cloud - Dragonsblight'),
(152197,4994,0,'Arctic Cloud - Dragonsblight'),
(152198,4994,0,'Arctic Cloud - Dragonsblight'),
(152199,4994,0,'Arctic Cloud - Dragonsblight'),
(152200,4994,0,'Arctic Cloud - Dragonsblight'),
(152201,4994,0,'Arctic Cloud - Dragonsblight'),
(152202,4994,0,'Arctic Cloud - Dragonsblight'),
(152203,4994,0,'Arctic Cloud - Dragonsblight'),
(152204,4994,0,'Arctic Cloud - Dragonsblight'),
(152205,4994,0,'Arctic Cloud - Dragonsblight'),
(152206,4994,0,'Arctic Cloud - Dragonsblight'),
(152207,4994,0,'Arctic Cloud - Dragonsblight'),
(152208,4995,0,'Arctic Cloud - Icecrow'),
(152209,4995,0,'Arctic Cloud - Icecrow'),
(152210,4995,0,'Arctic Cloud - Icecrow'),
(152211,4995,0,'Arctic Cloud - Icecrow'),
(152212,4995,0,'Arctic Cloud - Icecrow'),
(152213,4995,0,'Arctic Cloud - Icecrow'),
(152214,4995,0,'Arctic Cloud - Icecrow'),
(152215,4995,0,'Arctic Cloud - Icecrow'),
(152216,4995,0,'Arctic Cloud - Icecrow'),
(152217,4995,0,'Arctic Cloud - Icecrow'),
(152218,4995,0,'Arctic Cloud - Icecrow'),
(152219,4996,0,'Cinder Cloud'),
(152220,4996,0,'Cinder Cloud'),
(152221,4996,0,'Cinder Cloud'),
(152222,4996,0,'Cinder Cloud'),
(152223,4996,0,'Cinder Cloud'),
(152224,4996,0,'Cinder Cloud'),
(152225,4996,0,'Cinder Cloud'),
(152226,4996,0,'Cinder Cloud'),
(152227,4996,0,'Cinder Cloud'),
(152228,4996,0,'Cinder Cloud'),
(152229,4996,0,'Cinder Cloud'),
(152230,4996,0,'Cinder Cloud'),
(152231,4997,0,'Steam Cloud - Sholazar'),
(152232,4997,0,'Steam Cloud - Sholazar'),
(152233,4997,0,'Steam Cloud - Sholazar'),
(152234,4997,0,'Steam Cloud - Sholazar'),
(152235,4997,0,'Steam Cloud - Sholazar'),
(152236,4997,0,'Steam Cloud - Sholazar'),
(152237,4997,0,'Steam Cloud - Sholazar'),
(152238,4997,0,'Steam Cloud - Sholazar'),
(152239,4997,0,'Steam Cloud - Sholazar'),
(152240,4997,0,'Steam Cloud - Sholazar'),
(152241,4997,0,'Steam Cloud - Sholazar'),
(152242,4997,0,'Steam Cloud - Sholazar'),
(152243,4997,0,'Steam Cloud - Sholazar'),
(152244,4997,0,'Steam Cloud - Sholazar'),
(152245,4997,0,'Steam Cloud - Sholazar'),
(152246,4997,0,'Steam Cloud - Sholazar'),
(152247,4997,0,'Steam Cloud - Sholazar'),
(152248,4997,0,'Steam Cloud - Sholazar'),
(152249,4997,0,'Steam Cloud - Sholazar'),
(152250,4997,0,'Steam Cloud - Sholazar'),
(152251,4997,0,'Steam Cloud - Sholazar'),
(152252,4997,0,'Steam Cloud - Sholazar'),
(152253,4997,0,'Steam Cloud - Sholazar'),
(152254,4997,0,'Steam Cloud - Sholazar'),
(152255,4998,0,'Steam Cloud - Borean Tundra'),
(152256,4998,0,'Steam Cloud - Borean Tundra'),
(152257,4998,0,'Steam Cloud - Borean Tundra'),
(152258,4998,0,'Steam Cloud - Borean Tundra'),
(160506,1211,30,'Meshlok the Harvester (Maraudon)'),
(160509,1211,0,'Meshlok the Harvester placeholder (Maraudon)'),
(200126,60000,0,'Gondria (33776) - Spawn 6'),
(200127,60000,0,'Gondria (33776) - Spawn 5'),
(200128,60000,0,'Gondria (33776) - Spawn 4'),
(200129,60000,0,'Gondria (33776) - Spawn 3'),
(200130,60000,0,'Gondria (33776) - Spawn 2'),
(200131,60000,0,'Gondria (33776) - Spawn 1'),
(200132,60001,0,'Loquenahak (32517) - Spawn 1'),
(200133,60001,0,'Loquenahak (32517) - Spawn 2'),
(200134,60001,0,'Loquenahak (32517) - Spawn 3'),
(200135,60001,0,'Loquenahak (32517) - Spawn 4'),
(200136,60001,0,'Loquenahak (32517) - Spawn 5'),
(200137,60001,0,'Loquenahak (32517) - Spawn 6'),
(200138,60001,0,'Loquenahak (32517) - Spawn 7'),
(200139,60001,0,'Loquenahak (32517) - Spawn 8'),
(202441,32630,0,'Vyragosa (32630) - Spawn 1'),
(202442,32630,0,'Vyragosa (32630) - Spawn 2'),
(202443,32630,0,'Vyragosa (32630) - Spawn 3'),
(202444,32630,0,'Vyragosa (32630) - Spawn 4'),
(202445,32630,0,'Vyragosa (32630) - Spawn 5'),
(202446,32630,0,'Vyragosa (32630) - Spawn 6'),
(202447,32630,0,'Vyragosa (32630) - Spawn 7'),
(202448,32630,0,'Vyragosa (32630) - Spawn 8'),
(202449,32630,0,'Vyragosa (32630) - Spawn 9'),
(202450,32630,0,'Vyragosa (32630) - Spawn 10'),
(202451,32630,0,'Vyragosa (32630) - Spawn 11'),
(202452,32630,0,'Vyragosa (32630) - Spawn 12'),
(202453,32630,0,'Vyragosa (32630) - Spawn 13'),
(202454,32630,0,'Vyragosa (32630) - Spawn 14'),
(202455,32630,0,'Vyragosa (32630) - Spawn 15'),
(202456,32630,0,'Vyragosa (32630) - Spawn 16'),
(202457,32630,0,'Vyragosa (32630) - Spawn 17'),
(202458,32630,0,'Vyragosa (32630) - Spawn 18'),
(202459,32630,0,'Vyragosa (32630) - Spawn 19'),
(202460,32630,0,'Vyragosa (32630) - Spawn 20'),
(202461,32491,0,'Time-Lost Proto Drake (32491) - Spawn 1'),
(202462,32491,0,'Time-Lost Proto Drake (32491) - Spawn 2'),
(202463,32491,0,'Time-Lost Proto Drake (32491) - Spawn 3'),
(202464,32491,0,'Time-Lost Proto Drake (32491) - Spawn 4'),
(202465,32491,0,'Time-Lost Proto Drake (32491) - Spawn 5'),
(202466,32491,0,'Time-Lost Proto Drake (32491) - Spawn 6'),
(202467,32491,0,'Time-Lost Proto Drake (32491) - Spawn 7'),
(202468,32491,0,'Time-Lost Proto Drake (32491) - Spawn 8'),
(202469,32491,0,'Time-Lost Proto Drake (32491) - Spawn 9'),
(202470,32491,0,'Time-Lost Proto Drake (32491) - Spawn 10'),
(202471,32491,0,'Time-Lost Proto Drake (32491) - Spawn 11'),
(202472,32491,0,'Time-Lost Proto Drake (32491) - Spawn 12'),
(202473,32491,0,'Time-Lost Proto Drake (32491) - Spawn 13'),
(202474,32491,0,'Time-Lost Proto Drake (32491) - Spawn 14'),
(202475,32491,0,'Time-Lost Proto Drake (32491) - Spawn 15'),
(202476,32491,0,'Time-Lost Proto Drake (32491) - Spawn 16'),
(202477,32491,0,'Time-Lost Proto Drake (32491) - Spawn 17'),
(202478,32491,0,'Time-Lost Proto Drake (32491) - Spawn 18'),
(202479,32491,0,'Time-Lost Proto Drake (32491) - Spawn 19'),
(202480,32491,0,'Time-Lost Proto Drake (32491) - Spawn 20'),
(202602,32630,0,'Vyragosa (32630) - Spawn 21'),
(203506,202481,12,'Meshlok the Harvester (12237)'),
(203522,202481,0,'trigger for Meshlok (12999)'),
(207266,7001,0,'Webbed Crusader Spawn 1'),
(207267,7002,0,'Webbed Crusader Spawn 2'),
(207268,7003,0,'Webbed Crusader Spawn 3'),
(207269,7004,0,'Webbed Crusader Spawn 4'),
(207270,7005,0,'Webbed Crusader Spawn 5'),
(207271,7006,0,'Webbed Crusader Spawn 6'),
(207272,7007,0,'Webbed Crusader Spawn 7'),
(207273,7008,0,'Webbed Crusader Spawn 8'),
(207274,7009,0,'Webbed Crusader Spawn 9'),
(207275,7010,0,'Webbed Crusader Spawn 10'),
(207276,7011,0,'Webbed Crusader Spawn 11'),
(207277,7012,0,'Webbed Crusader Spawn 12'),
(207278,7013,0,'Webbed Crusader Spawn 13'),
(207279,7014,0,'Webbed Crusader Spawn 14'),
(359116,51001,0,'Blazing Elemental'),
(359117,51002,0,'Blazing Elemental'),
(359118,51003,0,'Blazing Elemental'),
(359119,51004,0,'Blazing Elemental'),
(359120,51005,0,'Blazing Elemental'),
(359121,51006,0,'Blazing Elemental'),
(359122,51007,0,'Blazing Elemental'),
(359123,51008,0,'Blazing Elemental'),
(359124,51009,0,'Blazing Elemental'),
(359125,51010,0,'Blazing Elemental'),
(359126,51011,0,'Blazing Elemental'),
(359127,51012,0,'Blazing Elemental'),
(359128,51013,0,'Blazing Elemental'),
(359129,51014,0,'Blazing Elemental'),
(359130,51015,0,'Blazing Elemental'),
(359131,51016,0,'Blazing Elemental'),
(359132,51017,0,'Blazing Elemental'),
(359133,51018,0,'Blazing Elemental'),
(359134,51019,0,'Blazing Elemental'),
(359135,51020,0,'Blazing Elemental'),
(359136,51021,0,'Blazing Elemental'),
(359137,51022,0,'Blazing Elemental'),
(359138,51023,0,'Blazing Elemental'),
(359139,51024,0,'Blazing Elemental'),
(359140,51025,0,'Blazing Elemental'),
(359141,51026,0,'Blazing Elemental'),
(359142,51027,0,'Blazing Elemental'),
(359143,51028,0,'Blazing Elemental'),
(359144,51029,0,'Blazing Elemental'),
(359145,51030,0,'Blazing Elemental'),
(359146,51031,0,'Blazing Elemental'),
(359147,51032,0,'Blazing Elemental'),
(359148,51033,0,'Blazing Elemental'),
(359149,51034,0,'Blazing Elemental'),
(359150,51035,0,'Blazing Elemental'),
(359151,51036,0,'Blazing Elemental'),
(359152,51037,0,'Blazing Elemental'),
(359153,51038,0,'Blazing Elemental'),
(359154,51039,0,'Blazing Elemental'),
(359155,51040,0,'Blazing Elemental'),
(359156,51041,0,'Blazing Elemental'),
(359157,51042,0,'Blazing Elemental'),
(359158,51043,0,'Blazing Elemental'),
(359159,51044,0,'Blazing Elemental'),
(359160,51045,0,'Blazing Elemental'),
(359161,51046,0,'Blazing Elemental'),
(359162,51047,0,'Blazing Elemental'),
(359163,51048,0,'Blazing Elemental'),
(359164,51049,0,'Blazing Elemental'),
(359165,51050,0,'Blazing Elemental'),
(359166,51051,0,'Blazing Elemental'),
(359167,51052,0,'Blazing Elemental'),
(359168,51053,0,'Blazing Elemental'),
(359169,51054,0,'Blazing Elemental'),
(359170,51055,0,'Blazing Elemental'),
(359171,51056,0,'Blazing Elemental'),
(359172,51057,0,'Blazing Elemental'),
(359173,51058,0,'Blazing Elemental'),
(359174,51059,0,'Blazing Elemental'),
(359175,51060,0,'Blazing Elemental'),
(359176,51061,0,'Blazing Elemental'),
(359177,51062,0,'Blazing Elemental'),
(359178,51063,0,'Blazing Elemental'),
(359179,51001,0,'Inferno Elemental'),
(359180,51002,0,'Inferno Elemental'),
(359181,51003,0,'Inferno Elemental'),
(359182,51004,0,'Inferno Elemental'),
(359183,51005,0,'Inferno Elemental'),
(359184,51006,0,'Inferno Elemental'),
(359185,51007,0,'Inferno Elemental'),
(359186,51008,0,'Inferno Elemental'),
(359187,51009,0,'Inferno Elemental'),
(359188,51010,0,'Inferno Elemental'),
(359189,51011,0,'Inferno Elemental'),
(359190,51012,0,'Inferno Elemental'),
(359191,51013,0,'Inferno Elemental'),
(359192,51014,0,'Inferno Elemental'),
(359193,51015,0,'Inferno Elemental'),
(359194,51016,0,'Inferno Elemental'),
(359195,51017,0,'Inferno Elemental'),
(359196,51018,0,'Inferno Elemental'),
(359197,51019,0,'Inferno Elemental'),
(359198,51020,0,'Inferno Elemental'),
(359199,51021,0,'Inferno Elemental'),
(359200,51022,0,'Inferno Elemental'),
(359201,51023,0,'Inferno Elemental'),
(359202,51024,0,'Inferno Elemental'),
(359203,51025,0,'Inferno Elemental'),
(359204,51026,0,'Inferno Elemental'),
(359205,51027,0,'Inferno Elemental'),
(359206,51028,0,'Inferno Elemental'),
(359207,51029,0,'Inferno Elemental'),
(359208,51030,0,'Inferno Elemental'),
(359209,51031,0,'Inferno Elemental'),
(359210,51032,0,'Inferno Elemental'),
(359211,51033,0,'Inferno Elemental'),
(359212,51034,0,'Inferno Elemental'),
(359213,51035,0,'Inferno Elemental'),
(359214,51036,0,'Inferno Elemental'),
(359215,51037,0,'Inferno Elemental'),
(359216,51038,0,'Inferno Elemental'),
(359217,51039,0,'Inferno Elemental'),
(359218,51040,0,'Inferno Elemental'),
(359219,51041,0,'Inferno Elemental'),
(359220,51042,0,'Inferno Elemental'),
(359221,51043,0,'Inferno Elemental'),
(359222,51044,0,'Inferno Elemental'),
(359223,51045,0,'Inferno Elemental'),
(359224,51046,0,'Inferno Elemental'),
(359225,51047,0,'Inferno Elemental'),
(359226,51048,0,'Inferno Elemental'),
(359227,51049,0,'Inferno Elemental'),
(359228,51050,0,'Inferno Elemental'),
(359229,51051,0,'Inferno Elemental'),
(359230,51052,0,'Inferno Elemental'),
(359231,51053,0,'Inferno Elemental'),
(359232,51054,0,'Inferno Elemental'),
(359233,51055,0,'Inferno Elemental'),
(359234,51056,0,'Inferno Elemental'),
(359235,51057,0,'Inferno Elemental'),
(359236,51058,0,'Inferno Elemental'),
(359237,51059,0,'Inferno Elemental'),
(359238,51060,0,'Inferno Elemental'),
(359239,51061,0,'Inferno Elemental'),
(359240,51062,0,'Inferno Elemental'),
(359241,51063,0,'Inferno Elemental'),
(359281,51001,0,'Magma Elemental'),
(359282,51002,0,'Magma Elemental'),
(359283,51003,0,'Magma Elemental'),
(359284,51004,0,'Magma Elemental'),
(359285,51005,0,'Magma Elemental'),
(359286,51006,0,'Magma Elemental'),
(359287,51007,0,'Magma Elemental'),
(359288,51008,0,'Magma Elemental'),
(359289,51009,0,'Magma Elemental'),
(359290,51010,0,'Magma Elemental'),
(359291,51011,0,'Magma Elemental'),
(359292,51012,0,'Magma Elemental'),
(359293,51013,0,'Magma Elemental'),
(359294,51014,0,'Magma Elemental'),
(359295,51015,0,'Magma Elemental'),
(359296,51016,0,'Magma Elemental'),
(359297,51017,0,'Magma Elemental'),
(359298,51018,0,'Magma Elemental'),
(359299,51019,0,'Magma Elemental'),
(359300,51020,0,'Magma Elemental'),
(359301,51021,0,'Magma Elemental'),
(359302,51022,0,'Magma Elemental'),
(359303,51023,0,'Magma Elemental'),
(359304,51024,0,'Magma Elemental'),
(359305,51025,0,'Magma Elemental'),
(359306,51026,0,'Magma Elemental'),
(359307,51027,0,'Magma Elemental'),
(359308,51028,0,'Magma Elemental'),
(359309,51029,0,'Magma Elemental'),
(359310,51030,0,'Magma Elemental'),
(359311,51031,0,'Magma Elemental'),
(359312,51032,0,'Magma Elemental'),
(359313,51033,0,'Magma Elemental'),
(359314,51034,0,'Magma Elemental'),
(359315,51035,0,'Magma Elemental'),
(359316,51036,0,'Magma Elemental'),
(359317,51037,0,'Magma Elemental'),
(359318,51038,0,'Magma Elemental'),
(359319,51039,0,'Magma Elemental'),
(359320,51040,0,'Magma Elemental'),
(359321,51041,0,'Magma Elemental'),
(359322,51042,0,'Magma Elemental'),
(359323,51043,0,'Magma Elemental'),
(359324,51044,0,'Magma Elemental'),
(359325,51045,0,'Magma Elemental'),
(359326,51046,0,'Magma Elemental'),
(359327,51047,0,'Magma Elemental'),
(359328,51048,0,'Magma Elemental'),
(359329,51049,0,'Magma Elemental'),
(359330,51050,0,'Magma Elemental'),
(359331,51051,0,'Magma Elemental'),
(359332,51052,0,'Magma Elemental'),
(359333,51053,0,'Magma Elemental'),
(359334,51054,0,'Magma Elemental'),
(359335,51055,0,'Magma Elemental'),
(359336,51056,0,'Magma Elemental'),
(359337,51057,0,'Magma Elemental'),
(359338,51058,0,'Magma Elemental'),
(359339,51059,0,'Magma Elemental'),
(359340,51060,0,'Magma Elemental'),
(359341,51061,0,'Magma Elemental'),
(359342,51062,0,'Magma Elemental'),
(359343,51063,0,'Magma Elemental'),
(359405,51064,0,'Twilight Dark Shaman'),
(359406,51065,0,'Twilight Dark Shaman'),
(359407,51066,0,'Twilight Dark Shaman'),
(359408,51067,0,'Twilight Dark Shaman'),
(359409,51068,0,'Twilight Dark Shaman'),
(359410,51069,0,'Twilight Dark Shaman'),
(359411,51070,0,'Twilight Dark Shaman'),
(359412,51071,0,'Twilight Dark Shaman'),
(359413,51072,0,'Twilight Dark Shaman'),
(359414,51073,0,'Twilight Dark Shaman'),
(359415,51074,0,'Twilight Dark Shaman'),
(359416,51075,0,'Twilight Dark Shaman'),
(359417,51076,0,'Twilight Dark Shaman'),
(359418,51077,0,'Twilight Dark Shaman'),
(359419,51078,0,'Twilight Dark Shaman'),
(359420,51079,0,'Twilight Dark Shaman'),
(359421,51080,0,'Twilight Dark Shaman'),
(359422,51081,0,'Twilight Dark Shaman'),
(359423,51082,0,'Twilight Dark Shaman'),
(359424,51083,0,'Twilight Dark Shaman'),
(359425,51084,0,'Twilight Dark Shaman'),
(359426,51085,0,'Twilight Dark Shaman'),
(359427,51086,0,'Twilight Dark Shaman'),
(359428,51087,0,'Twilight Dark Shaman'),
(359429,51088,0,'Twilight Dark Shaman'),
(359430,51089,0,'Twilight Dark Shaman'),
(359431,51090,0,'Twilight Dark Shaman'),
(359432,51091,0,'Twilight Dark Shaman'),
(359433,51092,0,'Twilight Dark Shaman'),
(359434,51093,0,'Twilight Dark Shaman'),
(359435,51094,0,'Twilight Dark Shaman'),
(359436,51095,0,'Twilight Dark Shaman'),
(359437,51096,0,'Twilight Dark Shaman'),
(359438,51097,0,'Twilight Dark Shaman'),
(359439,51064,0,'Twilight Fire Guard'),
(359440,51065,0,'Twilight Fire Guard'),
(359441,51066,0,'Twilight Fire Guard'),
(359442,51067,0,'Twilight Fire Guard'),
(359443,51068,0,'Twilight Fire Guard'),
(359444,51069,0,'Twilight Fire Guard'),
(359445,51070,0,'Twilight Fire Guard'),
(359446,51071,0,'Twilight Fire Guard'),
(359447,51072,0,'Twilight Fire Guard'),
(359448,51073,0,'Twilight Fire Guard'),
(359449,51074,0,'Twilight Fire Guard'),
(359450,51075,0,'Twilight Fire Guard'),
(359451,51076,0,'Twilight Fire Guard'),
(359452,51077,0,'Twilight Fire Guard'),
(359453,51078,0,'Twilight Fire Guard'),
(359454,51079,0,'Twilight Fire Guard'),
(359455,51080,0,'Twilight Fire Guard'),
(359456,51081,0,'Twilight Fire Guard'),
(359457,51082,0,'Twilight Fire Guard'),
(359458,51083,0,'Twilight Fire Guard'),
(359459,51084,0,'Twilight Fire Guard'),
(359460,51085,0,'Twilight Fire Guard'),
(359461,51086,0,'Twilight Fire Guard'),
(359462,51087,0,'Twilight Fire Guard'),
(359463,51088,0,'Twilight Fire Guard'),
(359464,51089,0,'Twilight Fire Guard'),
(359465,51090,0,'Twilight Fire Guard'),
(359466,51091,0,'Twilight Fire Guard'),
(359467,51092,0,'Twilight Fire Guard'),
(359468,51093,0,'Twilight Fire Guard'),
(359469,51094,0,'Twilight Fire Guard'),
(359470,51095,0,'Twilight Fire Guard'),
(359471,51096,0,'Twilight Fire Guard'),
(359472,51097,0,'Twilight Fire Guard'),
(359473,51064,0,'Twilight Geomancer'),
(359474,51065,0,'Twilight Geomancer'),
(359475,51066,0,'Twilight Geomancer'),
(359476,51067,0,'Twilight Geomancer'),
(359477,51068,0,'Twilight Geomancer'),
(359478,51069,0,'Twilight Geomancer'),
(359479,51070,0,'Twilight Geomancer'),
(359480,51071,0,'Twilight Geomancer'),
(359481,51072,0,'Twilight Geomancer'),
(359482,51073,0,'Twilight Geomancer'),
(359483,51074,0,'Twilight Geomancer'),
(359484,51075,0,'Twilight Geomancer'),
(359485,51076,0,'Twilight Geomancer'),
(359486,51077,0,'Twilight Geomancer'),
(359487,51078,0,'Twilight Geomancer'),
(359488,51079,0,'Twilight Geomancer'),
(359489,51080,0,'Twilight Geomancer'),
(359490,51081,0,'Twilight Geomancer'),
(359491,51082,0,'Twilight Geomancer'),
(359492,51083,0,'Twilight Geomancer'),
(359493,51084,0,'Twilight Geomancer'),
(359494,51085,0,'Twilight Geomancer'),
(359495,51086,0,'Twilight Geomancer'),
(359496,51087,0,'Twilight Geomancer'),
(359497,51088,0,'Twilight Geomancer'),
(359498,51089,0,'Twilight Geomancer'),
(359499,51090,0,'Twilight Geomancer'),
(359500,51091,0,'Twilight Geomancer'),
(359501,51092,0,'Twilight Geomancer'),
(359502,51093,0,'Twilight Geomancer'),
(359503,51094,0,'Twilight Geomancer'),
(359504,51095,0,'Twilight Geomancer'),
(359505,51096,0,'Twilight Geomancer'),
(359506,51097,0,'Twilight Geomancer'),
(359778,51098,0,'Dark Iron Digmaster'),
(359779,51099,0,'Dark Iron Digmaster'),
(359780,51100,0,'Dark Iron Digmaster'),
(359781,51101,0,'Dark Iron Digmaster'),
(359782,51102,0,'Dark Iron Digmaster'),
(359783,51103,0,'Dark Iron Digmaster'),
(359784,51104,0,'Dark Iron Digmaster'),
(359785,51105,0,'Dark Iron Digmaster'),
(359787,51098,0,'Dark Blacksmith'),
(359788,51099,0,'Dark Blacksmith'),
(359789,51100,0,'Dark Blacksmith'),
(359790,51101,0,'Dark Blacksmith'),
(359791,51102,0,'Dark Blacksmith'),
(359792,51103,0,'Dark Blacksmith'),
(359793,51104,0,'Dark Blacksmith'),
(359794,51105,0,'Dark Blacksmith'),
(360275,361001,0,'Deadwood Den Watcher'),
(360276,361002,0,'Deadwood Den Watcher'),
(360277,361003,0,'Deadwood Den Watcher'),
(360278,361004,0,'Deadwood Den Watcher'),
(360279,361005,0,'Deadwood Den Watcher'),
(360280,361006,0,'Deadwood Den Watcher'),
(360281,361007,0,'Deadwood Den Watcher'),
(360282,361008,0,'Deadwood Den Watcher'),
(360283,361009,0,'Deadwood Den Watcher'),
(360284,361010,0,'Deadwood Den Watcher'),
(360285,361011,0,'Deadwood Den Watcher'),
(360286,361012,0,'Deadwood Den Watcher'),
(360287,361013,0,'Deadwood Den Watcher'),
(360288,361014,0,'Deadwood Den Watcher'),
(360289,361015,0,'Deadwood Den Watcher'),
(360290,361016,0,'Deadwood Den Watcher'),
(360291,361017,0,'Deadwood Den Watcher'),
(360292,361018,0,'Deadwood Den Watcher'),
(360293,361019,0,'Deadwood Den Watcher'),
(360294,361020,0,'Deadwood Den Watcher'),
(360295,361021,0,'Deadwood Den Watcher'),
(360296,361022,0,'Deadwood Den Watcher'),
(360297,361023,0,'Deadwood Den Watcher'),
(360298,361024,0,'Deadwood Den Watcher'),
(360299,361025,0,'Deadwood Den Watcher'),
(360300,361026,0,'Deadwood Den Watcher'),
(360301,361027,0,'Deadwood Den Watcher'),
(360302,361028,0,'Deadwood Den Watcher'),
(360303,361029,0,'Deadwood Den Watcher'),
(360304,361030,0,'Deadwood Den Watcher'),
(360305,361031,0,'Deadwood Den Watcher'),
(360306,361032,0,'Deadwood Den Watcher'),
(360307,361033,0,'Deadwood Den Watcher'),
(360308,361034,0,'Deadwood Den Watcher'),
(361530,361001,0,'Deadwood Avenger'),
(361531,361002,0,'Deadwood Avenger'),
(361532,361003,0,'Deadwood Avenger'),
(361533,361004,0,'Deadwood Avenger'),
(361534,361005,0,'Deadwood Avenger'),
(361535,361006,0,'Deadwood Avenger'),
(361536,361007,0,'Deadwood Avenger'),
(361537,361008,0,'Deadwood Avenger'),
(361538,361009,0,'Deadwood Avenger'),
(361539,361010,0,'Deadwood Avenger'),
(361540,361011,0,'Deadwood Avenger'),
(361541,361012,0,'Deadwood Avenger'),
(361542,361013,0,'Deadwood Avenger'),
(361543,361014,0,'Deadwood Avenger'),
(361544,361015,0,'Deadwood Avenger'),
(361545,361016,0,'Deadwood Avenger'),
(361546,361017,0,'Deadwood Avenger'),
(361547,361018,0,'Deadwood Avenger'),
(361548,361019,0,'Deadwood Avenger'),
(361549,361020,0,'Deadwood Avenger'),
(361550,361021,0,'Deadwood Avenger'),
(361551,361022,0,'Deadwood Avenger'),
(361552,361023,0,'Deadwood Avenger'),
(361553,361024,0,'Deadwood Avenger'),
(361554,361025,0,'Deadwood Avenger'),
(361555,361026,0,'Deadwood Avenger'),
(361556,361027,0,'Deadwood Avenger'),
(361557,361028,0,'Deadwood Avenger'),
(361558,361029,0,'Deadwood Avenger'),
(361559,361030,0,'Deadwood Avenger'),
(361560,361031,0,'Deadwood Avenger'),
(361561,361032,0,'Deadwood Avenger'),
(361562,361033,0,'Deadwood Avenger'),
(361563,361034,0,'Deadwood Avenger'),
(361564,361001,0,'Deadwood Shaman'),
(361565,361002,0,'Deadwood Shaman'),
(361566,361003,0,'Deadwood Shaman'),
(361567,361004,0,'Deadwood Shaman'),
(361568,361005,0,'Deadwood Shaman'),
(361569,361006,0,'Deadwood Shaman'),
(361570,361007,0,'Deadwood Shaman'),
(361571,361008,0,'Deadwood Shaman'),
(361572,361009,0,'Deadwood Shaman'),
(361573,361010,0,'Deadwood Shaman'),
(361574,361011,0,'Deadwood Shaman'),
(361575,361012,0,'Deadwood Shaman'),
(361576,361013,0,'Deadwood Shaman'),
(361577,361014,0,'Deadwood Shaman'),
(361578,361015,0,'Deadwood Shaman'),
(361579,361016,0,'Deadwood Shaman'),
(361580,361017,0,'Deadwood Shaman'),
(361581,361018,0,'Deadwood Shaman'),
(361582,361019,0,'Deadwood Shaman'),
(361583,361020,0,'Deadwood Shaman'),
(361584,361021,0,'Deadwood Shaman'),
(361585,361022,0,'Deadwood Shaman'),
(361586,361023,0,'Deadwood Shaman'),
(361587,361024,0,'Deadwood Shaman'),
(361588,361025,0,'Deadwood Shaman'),
(361589,361026,0,'Deadwood Shaman'),
(361590,361027,0,'Deadwood Shaman'),
(361591,361028,0,'Deadwood Shaman'),
(361592,361029,0,'Deadwood Shaman'),
(361593,361030,0,'Deadwood Shaman'),
(361594,361031,0,'Deadwood Shaman'),
(361595,361032,0,'Deadwood Shaman'),
(361596,361033,0,'Deadwood Shaman'),
(361597,361034,0,'Deadwood Shaman'),
(361694,645001,0,'Twilight Torturer'),
(361695,645002,0,'Twilight Torturer'),
(361696,645003,0,'Twilight Torturer'),
(361697,645004,0,'Twilight Torturer'),
(361698,645005,0,'Twilight Torturer'),
(361699,645006,0,'Twilight Torturer'),
(361700,645007,0,'Twilight Torturer'),
(361701,645008,0,'Twilight Torturer'),
(361702,645009,0,'Twilight Torturer'),
(361703,645010,0,'Twilight Torturer'),
(361704,645011,0,'Twilight Torturer'),
(361705,645012,0,'Twilight Torturer'),
(361706,645013,0,'Twilight Torturer'),
(361707,645001,0,'Twilight Sadist'),
(361708,645002,0,'Twilight Sadist'),
(361709,645003,0,'Twilight Sadist'),
(361710,645004,0,'Twilight Sadist'),
(361711,645005,0,'Twilight Sadist'),
(361712,645006,0,'Twilight Sadist'),
(361713,645007,0,'Twilight Sadist'),
(361714,645008,0,'Twilight Sadist'),
(361715,645009,0,'Twilight Sadist'),
(361716,645010,0,'Twilight Sadist'),
(361717,645011,0,'Twilight Sadist'),
(361718,645012,0,'Twilight Sadist'),
(361719,645013,0,'Twilight Sadist'),
(361720,645014,0,'Mad Prisoner'),
(361721,645015,0,'Mad Prisoner'),
(361722,645016,0,'Mad Prisoner'),
(361723,645017,0,'Mad Prisoner'),
(361724,645018,0,'Mad Prisoner'),
(361725,645014,0,'Crazed Sadist'),
(361726,645015,0,'Crazed Sadist'),
(361727,645016,0,'Crazed Sadist'),
(361728,645017,0,'Crazed Sadist'),
(361729,645018,0,'Crazed Sadist');
/*!40000 ALTER TABLE `pool_creature` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
