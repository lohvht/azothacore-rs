/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `liquid_type`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `liquid_type` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Name` text DEFAULT NULL,
  `Texture1` text DEFAULT NULL,
  `Texture2` text DEFAULT NULL,
  `Texture3` text DEFAULT NULL,
  `Texture4` text DEFAULT NULL,
  `Texture5` text DEFAULT NULL,
  `Texture6` text DEFAULT NULL,
  `SpellID` int(10) unsigned NOT NULL DEFAULT 0,
  `MaxDarkenDepth` float NOT NULL DEFAULT 0,
  `FogDarkenIntensity` float NOT NULL DEFAULT 0,
  `AmbDarkenIntensity` float NOT NULL DEFAULT 0,
  `DirDarkenIntensity` float NOT NULL DEFAULT 0,
  `ParticleScale` float NOT NULL DEFAULT 0,
  `Color1` int(11) NOT NULL DEFAULT 0,
  `Color2` int(11) NOT NULL DEFAULT 0,
  `Float1` float NOT NULL DEFAULT 0,
  `Float2` float NOT NULL DEFAULT 0,
  `Float3` float NOT NULL DEFAULT 0,
  `Float4` float NOT NULL DEFAULT 0,
  `Float5` float NOT NULL DEFAULT 0,
  `Float6` float NOT NULL DEFAULT 0,
  `Float7` float NOT NULL DEFAULT 0,
  `Float8` float NOT NULL DEFAULT 0,
  `Float9` float NOT NULL DEFAULT 0,
  `Float10` float NOT NULL DEFAULT 0,
  `Float11` float NOT NULL DEFAULT 0,
  `Float12` float NOT NULL DEFAULT 0,
  `Float13` float NOT NULL DEFAULT 0,
  `Float14` float NOT NULL DEFAULT 0,
  `Float15` float NOT NULL DEFAULT 0,
  `Float16` float NOT NULL DEFAULT 0,
  `Float17` float NOT NULL DEFAULT 0,
  `Float18` float NOT NULL DEFAULT 0,
  `Int1` int(10) unsigned NOT NULL DEFAULT 0,
  `Int2` int(10) unsigned NOT NULL DEFAULT 0,
  `Int3` int(10) unsigned NOT NULL DEFAULT 0,
  `Int4` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags` smallint(5) unsigned NOT NULL DEFAULT 0,
  `LightID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SoundBank` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ParticleMovement` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `ParticleTexSlots` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MaterialID` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FrameCountTexture1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FrameCountTexture2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FrameCountTexture3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FrameCountTexture4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FrameCountTexture5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `FrameCountTexture6` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `SoundID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `liquid_type` WRITE;
/*!40000 ALTER TABLE `liquid_type` DISABLE KEYS */;
/*!40000 ALTER TABLE `liquid_type` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

