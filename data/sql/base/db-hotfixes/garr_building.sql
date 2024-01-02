/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `garr_building`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `garr_building` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `AllianceName` text DEFAULT NULL,
  `HordeName` text DEFAULT NULL,
  `Description` text DEFAULT NULL,
  `Tooltip` text DEFAULT NULL,
  `HordeGameObjectID` int(11) NOT NULL DEFAULT 0,
  `AllianceGameObjectID` int(11) NOT NULL DEFAULT 0,
  `IconFileDataID` int(11) NOT NULL DEFAULT 0,
  `CurrencyTypeID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `HordeUiTextureKitID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AllianceUiTextureKitID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `AllianceSceneScriptPackageID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `HordeSceneScriptPackageID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `GarrAbilityID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `BonusGarrAbilityID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `GoldCost` smallint(5) unsigned NOT NULL DEFAULT 0,
  `GarrSiteID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `BuildingType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `UpgradeLevel` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Flags` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ShipmentCapacity` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `GarrTypeID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `BuildSeconds` int(11) NOT NULL DEFAULT 0,
  `CurrencyQty` int(11) NOT NULL DEFAULT 0,
  `MaxAssignments` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `garr_building` WRITE;
/*!40000 ALTER TABLE `garr_building` DISABLE KEYS */;
/*!40000 ALTER TABLE `garr_building` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

