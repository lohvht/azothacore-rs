/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `vehicle_seat`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `vehicle_seat` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `Flags` int(11) NOT NULL DEFAULT 0,
  `FlagsB` int(11) NOT NULL DEFAULT 0,
  `FlagsC` int(11) NOT NULL DEFAULT 0,
  `AttachmentOffsetX` float NOT NULL DEFAULT 0,
  `AttachmentOffsetY` float NOT NULL DEFAULT 0,
  `AttachmentOffsetZ` float NOT NULL DEFAULT 0,
  `EnterPreDelay` float NOT NULL DEFAULT 0,
  `EnterSpeed` float NOT NULL DEFAULT 0,
  `EnterGravity` float NOT NULL DEFAULT 0,
  `EnterMinDuration` float NOT NULL DEFAULT 0,
  `EnterMaxDuration` float NOT NULL DEFAULT 0,
  `EnterMinArcHeight` float NOT NULL DEFAULT 0,
  `EnterMaxArcHeight` float NOT NULL DEFAULT 0,
  `ExitPreDelay` float NOT NULL DEFAULT 0,
  `ExitSpeed` float NOT NULL DEFAULT 0,
  `ExitGravity` float NOT NULL DEFAULT 0,
  `ExitMinDuration` float NOT NULL DEFAULT 0,
  `ExitMaxDuration` float NOT NULL DEFAULT 0,
  `ExitMinArcHeight` float NOT NULL DEFAULT 0,
  `ExitMaxArcHeight` float NOT NULL DEFAULT 0,
  `PassengerYaw` float NOT NULL DEFAULT 0,
  `PassengerPitch` float NOT NULL DEFAULT 0,
  `PassengerRoll` float NOT NULL DEFAULT 0,
  `VehicleEnterAnimDelay` float NOT NULL DEFAULT 0,
  `VehicleExitAnimDelay` float NOT NULL DEFAULT 0,
  `CameraEnteringDelay` float NOT NULL DEFAULT 0,
  `CameraEnteringDuration` float NOT NULL DEFAULT 0,
  `CameraExitingDelay` float NOT NULL DEFAULT 0,
  `CameraExitingDuration` float NOT NULL DEFAULT 0,
  `CameraOffsetX` float NOT NULL DEFAULT 0,
  `CameraOffsetY` float NOT NULL DEFAULT 0,
  `CameraOffsetZ` float NOT NULL DEFAULT 0,
  `CameraPosChaseRate` float NOT NULL DEFAULT 0,
  `CameraFacingChaseRate` float NOT NULL DEFAULT 0,
  `CameraEnteringZoom` float NOT NULL DEFAULT 0,
  `CameraSeatZoomMin` float NOT NULL DEFAULT 0,
  `CameraSeatZoomMax` float NOT NULL DEFAULT 0,
  `UiSkinFileDataID` int(11) NOT NULL DEFAULT 0,
  `EnterAnimStart` smallint(6) NOT NULL DEFAULT 0,
  `EnterAnimLoop` smallint(6) NOT NULL DEFAULT 0,
  `RideAnimStart` smallint(6) NOT NULL DEFAULT 0,
  `RideAnimLoop` smallint(6) NOT NULL DEFAULT 0,
  `RideUpperAnimStart` smallint(6) NOT NULL DEFAULT 0,
  `RideUpperAnimLoop` smallint(6) NOT NULL DEFAULT 0,
  `ExitAnimStart` smallint(6) NOT NULL DEFAULT 0,
  `ExitAnimLoop` smallint(6) NOT NULL DEFAULT 0,
  `ExitAnimEnd` smallint(6) NOT NULL DEFAULT 0,
  `VehicleEnterAnim` smallint(6) NOT NULL DEFAULT 0,
  `VehicleExitAnim` smallint(6) NOT NULL DEFAULT 0,
  `VehicleRideAnimLoop` smallint(6) NOT NULL DEFAULT 0,
  `EnterAnimKitID` smallint(6) NOT NULL DEFAULT 0,
  `RideAnimKitID` smallint(6) NOT NULL DEFAULT 0,
  `ExitAnimKitID` smallint(6) NOT NULL DEFAULT 0,
  `VehicleEnterAnimKitID` smallint(6) NOT NULL DEFAULT 0,
  `VehicleRideAnimKitID` smallint(6) NOT NULL DEFAULT 0,
  `VehicleExitAnimKitID` smallint(6) NOT NULL DEFAULT 0,
  `CameraModeID` smallint(6) NOT NULL DEFAULT 0,
  `AttachmentID` tinyint(4) NOT NULL DEFAULT 0,
  `PassengerAttachmentID` tinyint(4) NOT NULL DEFAULT 0,
  `VehicleEnterAnimBone` tinyint(4) NOT NULL DEFAULT 0,
  `VehicleExitAnimBone` tinyint(4) NOT NULL DEFAULT 0,
  `VehicleRideAnimLoopBone` tinyint(4) NOT NULL DEFAULT 0,
  `VehicleAbilityDisplay` tinyint(4) NOT NULL DEFAULT 0,
  `EnterUISoundID` int(10) unsigned NOT NULL DEFAULT 0,
  `ExitUISoundID` int(10) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `vehicle_seat` WRITE;
/*!40000 ALTER TABLE `vehicle_seat` DISABLE KEYS */;
/*!40000 ALTER TABLE `vehicle_seat` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

