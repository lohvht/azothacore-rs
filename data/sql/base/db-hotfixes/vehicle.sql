/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `vehicle`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `vehicle` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags` int(11) NOT NULL DEFAULT 0,
  `TurnSpeed` float NOT NULL DEFAULT 0,
  `PitchSpeed` float NOT NULL DEFAULT 0,
  `PitchMin` float NOT NULL DEFAULT 0,
  `PitchMax` float NOT NULL DEFAULT 0,
  `MouseLookOffsetPitch` float NOT NULL DEFAULT 0,
  `CameraFadeDistScalarMin` float NOT NULL DEFAULT 0,
  `CameraFadeDistScalarMax` float NOT NULL DEFAULT 0,
  `CameraPitchOffset` float NOT NULL DEFAULT 0,
  `FacingLimitRight` float NOT NULL DEFAULT 0,
  `FacingLimitLeft` float NOT NULL DEFAULT 0,
  `CameraYawOffset` float NOT NULL DEFAULT 0,
  `SeatID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID4` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID5` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID6` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID7` smallint(5) unsigned NOT NULL DEFAULT 0,
  `SeatID8` smallint(5) unsigned NOT NULL DEFAULT 0,
  `VehicleUIIndicatorID` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PowerDisplayID1` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PowerDisplayID2` smallint(5) unsigned NOT NULL DEFAULT 0,
  `PowerDisplayID3` smallint(5) unsigned NOT NULL DEFAULT 0,
  `FlagsB` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `UiLocomotionType` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `MissileTargetingID` int(11) NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `vehicle` WRITE;
/*!40000 ALTER TABLE `vehicle` DISABLE KEYS */;
/*!40000 ALTER TABLE `vehicle` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

