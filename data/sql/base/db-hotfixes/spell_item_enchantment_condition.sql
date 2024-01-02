/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `spell_item_enchantment_condition`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `spell_item_enchantment_condition` (
  `ID` int(10) unsigned NOT NULL DEFAULT 0,
  `LtOperand1` int(10) unsigned NOT NULL DEFAULT 0,
  `LtOperand2` int(10) unsigned NOT NULL DEFAULT 0,
  `LtOperand3` int(10) unsigned NOT NULL DEFAULT 0,
  `LtOperand4` int(10) unsigned NOT NULL DEFAULT 0,
  `LtOperand5` int(10) unsigned NOT NULL DEFAULT 0,
  `LtOperandType1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LtOperandType2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LtOperandType3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LtOperandType4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `LtOperandType5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Operator1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Operator2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Operator3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Operator4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Operator5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperandType1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperandType2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperandType3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperandType4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperandType5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperand1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperand2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperand3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperand4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `RtOperand5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Logic1` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Logic2` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Logic3` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Logic4` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `Logic5` tinyint(3) unsigned NOT NULL DEFAULT 0,
  `VerifiedBuild` smallint(6) NOT NULL DEFAULT 0,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3 COLLATE=utf8mb3_general_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

LOCK TABLES `spell_item_enchantment_condition` WRITE;
/*!40000 ALTER TABLE `spell_item_enchantment_condition` DISABLE KEYS */;
/*!40000 ALTER TABLE `spell_item_enchantment_condition` ENABLE KEYS */;
UNLOCK TABLES;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

