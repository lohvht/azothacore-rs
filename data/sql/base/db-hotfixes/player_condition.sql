/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `player_condition`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `player_condition` (
  `RaceMask` bigint(20) NOT NULL DEFAULT 0,
  `FailureDescription` text DEFAULT NULL,
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ClassMask` int(11) NOT NULL DEFAULT 0,
  `Gender` tinyint(4) NOT NULL DEFAULT 0,
  `NativeGender` tinyint(4) NOT NULL DEFAULT 0,
  `SkillLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `LanguageID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinLanguage` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxLanguage` int(11) NOT NULL DEFAULT 0,
  `MaxFactionID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxReputation` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ReputationLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrentPvpFaction` tinyint(4) NOT NULL DEFAULT 0,
  `MinPVPRank` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxPVPRank` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PvpMedal` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PrevQuestLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrQuestLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrentCompletedQuestLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `SpellLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemFlags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AuraSpellLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `WorldStateExpressionID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `WeatherID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PartyStatus` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LifetimeMaxPVPRank` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AchievementLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `LfgLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `AreaLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `QuestKillLogic` int(10) unsigned NOT NULL DEFAULT 0,
  `MinExpansionLevel` tinyint(4) NOT NULL DEFAULT 0,
  `MaxExpansionLevel` tinyint(4) NOT NULL DEFAULT 0,
  `MinExpansionTier` tinyint(4) NOT NULL DEFAULT 0,
  `MaxExpansionTier` tinyint(4) NOT NULL DEFAULT 0,
  `MinGuildLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaxGuildLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PhaseUseFlags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PhaseID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PhaseGroupID` int(10) unsigned NOT NULL DEFAULT 0,
  `MinAvgItemLevel` int(11) NOT NULL DEFAULT 0,
  `MaxAvgItemLevel` int(11) NOT NULL DEFAULT 0,
  `MinAvgEquippedItemLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxAvgEquippedItemLevel` smallint(5) unsigned NOT NULL DEFAULT 0,
  `ChrSpecializationIndex` tinyint(4) NOT NULL DEFAULT 0,
  `ChrSpecializationRole` tinyint(4) NOT NULL DEFAULT 0,
  `PowerType` tinyint(4) NOT NULL DEFAULT 0,
  `PowerTypeComp` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PowerTypeValue` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ModifierTreeID` int(10) unsigned NOT NULL DEFAULT 0,
  `WeaponSubclassMask` int(11) NOT NULL DEFAULT 0,
  `SkillID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SkillID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SkillID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SkillID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinSkill1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinSkill2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinSkill3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinSkill4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxSkill1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxSkill2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxSkill3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MaxSkill4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `MinFactionID1` int(10) unsigned NOT NULL DEFAULT 0,
  `MinFactionID2` int(10) unsigned NOT NULL DEFAULT 0,
  `MinFactionID3` int(10) unsigned NOT NULL DEFAULT 0,
  `MinReputation1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinReputation2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MinReputation3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `PrevQuestID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PrevQuestID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PrevQuestID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PrevQuestID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrQuestID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrQuestID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrQuestID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrQuestID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrentCompletedQuestID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrentCompletedQuestID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrentCompletedQuestID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrentCompletedQuestID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SpellID1` int(11) NOT NULL DEFAULT 0,
  `SpellID2` int(11) NOT NULL DEFAULT 0,
  `SpellID3` int(11) NOT NULL DEFAULT 0,
  `SpellID4` int(11) NOT NULL DEFAULT 0,
  `ItemID1` int(11) NOT NULL DEFAULT 0,
  `ItemID2` int(11) NOT NULL DEFAULT 0,
  `ItemID3` int(11) NOT NULL DEFAULT 0,
  `ItemID4` int(11) NOT NULL DEFAULT 0,
  `ItemCount1` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemCount2` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemCount3` int(10) unsigned NOT NULL DEFAULT 0,
  `ItemCount4` int(10) unsigned NOT NULL DEFAULT 0,
  `Explored1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Explored2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Time1` int(10) unsigned NOT NULL DEFAULT 0,
  `Time2` int(10) unsigned NOT NULL DEFAULT 0,
  `AuraSpellID1` int(11) NOT NULL DEFAULT 0,
  `AuraSpellID2` int(11) NOT NULL DEFAULT 0,
  `AuraSpellID3` int(11) NOT NULL DEFAULT 0,
  `AuraSpellID4` int(11) NOT NULL DEFAULT 0,
  `AuraStacks1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AuraStacks2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AuraStacks3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `AuraStacks4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Achievement1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Achievement2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Achievement3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `Achievement4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LfgStatus1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgStatus2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgStatus3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgStatus4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgCompare1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgCompare2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgCompare3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgCompare4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LfgValue1` int(10) unsigned NOT NULL DEFAULT 0,
  `LfgValue2` int(10) unsigned NOT NULL DEFAULT 0,
  `LfgValue3` int(10) unsigned NOT NULL DEFAULT 0,
  `LfgValue4` int(10) unsigned NOT NULL DEFAULT 0,
  `AreaID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AreaID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AreaID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AreaID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `CurrencyID1` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyID2` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyID3` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyID4` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyCount1` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyCount2` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyCount3` int(10) unsigned NOT NULL DEFAULT 0,
  `CurrencyCount4` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillMonster1` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillMonster2` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillMonster3` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillMonster4` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillMonster5` int(10) unsigned NOT NULL DEFAULT 0,
  `QuestKillMonster6` int(10) unsigned NOT NULL DEFAULT 0,
  `MovementFlags1` int(11) NOT NULL DEFAULT 0,
  `MovementFlags2` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `player_condition` WRITE;
/*!40000 ALTER TABLE `player_condition` DISABLE KEYS */;
/*!40000 ALTER TABLE `player_condition` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
