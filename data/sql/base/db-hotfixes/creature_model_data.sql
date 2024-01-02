/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `creature_model_data`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `creature_model_data` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `ModelScale` float NOT NULL DEFAULT 0,
  `FootprintTextureLength` float NOT NULL DEFAULT 0,
  `FootprintTextureWidth` float NOT NULL DEFAULT 0,
  `FootprintParticleScale` float NOT NULL DEFAULT 0,
  `CollisionWidth` float NOT NULL DEFAULT 0,
  `CollisionHeight` float NOT NULL DEFAULT 0,
  `MountHeight` float NOT NULL DEFAULT 0,
  `GeoBox1` float NOT NULL DEFAULT 0,
  `GeoBox2` float NOT NULL DEFAULT 0,
  `GeoBox3` float NOT NULL DEFAULT 0,
  `GeoBox4` float NOT NULL DEFAULT 0,
  `GeoBox5` float NOT NULL DEFAULT 0,
  `GeoBox6` float NOT NULL DEFAULT 0,
  `WorldEffectScale` float NOT NULL DEFAULT 0,
  `AttachedEffectScale` float NOT NULL DEFAULT 0,
  `MissileCollisionRadius` float NOT NULL DEFAULT 0,
  `MissileCollisionPush` float NOT NULL DEFAULT 0,
  `MissileCollisionRaise` float NOT NULL DEFAULT 0,
  `OverrideLootEffectScale` float NOT NULL DEFAULT 0,
  `OverrideNameScale` float NOT NULL DEFAULT 0,
  `OverrideSelectionRadius` float NOT NULL DEFAULT 0,
  `TamedPetBaseScale` float NOT NULL DEFAULT 0,
  `HoverHeight` float NOT NULL DEFAULT 0,
  `Flags` int(10) unsigned NOT NULL DEFAULT 0,
  `FileDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `SizeClass` int(10) unsigned NOT NULL DEFAULT 0,
  `BloodID` int(10) unsigned NOT NULL DEFAULT 0,
  `FootprintTextureID` int(10) unsigned NOT NULL DEFAULT 0,
  `FoleyMaterialID` int(10) unsigned NOT NULL DEFAULT 0,
  `FootstepCameraEffectID` int(10) unsigned NOT NULL DEFAULT 0,
  `DeathThudCameraEffectID` int(10) unsigned NOT NULL DEFAULT 0,
  `SoundID` int(10) unsigned NOT NULL DEFAULT 0,
  `CreatureGeosetDataID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `creature_model_data` WRITE;
/*!40000 ALTER TABLE `creature_model_data` DISABLE KEYS */;
/*!40000 ALTER TABLE `creature_model_data` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

