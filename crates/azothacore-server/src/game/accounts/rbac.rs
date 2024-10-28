//! Role Based Access Control related definition
//!
//! This file contains all the structs and enums used to implement
//! Role Based Access Control
//!
//! RBAC Rules:
//! - Pemission: Defines an authorisation to perform certain operation.
//! - Role: Set of permissions.
//! - Group: Set of roles.
//! - An Account can have multiple groups, roles and permissions.
//! - Account Groups can only be granted or revoked
//! - Account Roles and Permissions can be granted, denied or revoked
//! - Grant: Assignment of the object (role/permission) and allow it
//! - Deny: Assignment of the object (role/permission) and deny it
//! - Revoke: Removal of the object (role/permission) no matter if it was granted or denied
//! - Global Permissions are computed as:
//!       Group Grants + Role Grants + User Grans - Role Grants - User Grants
//! - Groups, Roles and Permissions can be assigned by realm
//!

use std::collections::BTreeSet;

// use azothacore_common::AccountTypes;
// use azothacore_database::{
//     database_env::{LoginDatabase, LoginPreparedStmts},
//     params,
// };
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
// use tracing::{debug, instrument, trace};

// use super::account_mgr::ACCOUNT_MGR;

pub type RawRbacPermId = Result<RbacPermId, u32>;

/// IDs from rbac_permissions in enum form, u32 that cannot be turned into this
/// enum are usually from `rbac_linked_permissions``
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
pub enum RbacPermId {
    InstantLogout = 1,
    SkipQueue = 2,
    JoinNormalBg = 3,
    JoinRandomBg = 4,
    JoinArenas = 5,
    JoinDungeonFinder = 6,
    //  7 - reuse
    //  8 - reuse
    //  9 - reuse
    UseCharacterTemplates = 10,
    LogGmTrade = 11,
    SkipCheckCharacterCreationDemonHunter = 12,
    SkipCheckInstanceRequiredBosses = 13,
    SkipCheckCharacterCreationTeammask = 14,
    SkipCheckCharacterCreationClassmask = 15,
    SkipCheckCharacterCreationRacemask = 16,
    SkipCheckCharacterCreationReservedname = 17,
    SkipCheckCharacterCreationDeathKnight = 18, // deprecated since Draenor DON'T reuse
    SkipCheckChatChannelReq = 19,
    SkipCheckDisableMap = 20,
    SkipCheckMoreTalentsThanAllowed = 21,
    SkipCheckChatSpam = 22,
    SkipCheckOverspeedPing = 23,
    TwoSideCharacterCreation = 24,
    TwoSideInteractionChat = 25,
    TwoSideInteractionChannel = 26,
    TwoSideInteractionMail = 27,
    TwoSideWhoList = 28,
    TwoSideAddFriend = 29,
    CommandsSaveWithoutDelay = 30,
    CommandsUseUnstuckWithArgs = 31,
    CommandsBeAssignedTicket = 32,
    CommandsNotifyCommandNotFoundError = 33,
    CommandsAppearInGmList = 34,
    WhoSeeAllSecLevels = 35,
    CanFilterWhispers = 36,
    ChatUseStaffBadge = 37,
    ResurrectWithFullHps = 38,
    RestoreSavedGmState = 39,
    AllowGmFriend = 40,
    UseStartGmLevel = 41,
    OpcodeWorldTeleport = 42,
    OpcodeWhois = 43,
    ReceiveGlobalGmTextmessage = 44,
    SilentlyJoinChannel = 45,
    ChangeChannelNotModerator = 46,
    CheckForLowerSecurity = 47,
    CommandsPinfoCheckPersonalData = 48,
    EmailConfirmForPassChange = 49,
    MayCheckOwnEmail = 50,
    AllowTwoSideTrade = 51,

    // Free space for core permissions (till 149)
    // Roles (Permissions with delegated permissions) use 199 and descending
    CommandRbac = 200,
    CommandRbacAcc = 201,
    CommandRbacAccPermList = 202,
    CommandRbacAccPermGrant = 203,
    CommandRbacAccPermDeny = 204,
    CommandRbacAccPermRevoke = 205,
    CommandRbacList = 206,
    CommandBnetAccount = 207,
    CommandBnetAccountCreate = 208,
    CommandBnetAccountLockCountry = 209,
    CommandBnetAccountLockIp = 210,
    CommandBnetAccountPassword = 211,
    CommandBnetAccountSet = 212,
    CommandBnetAccountSetPassword = 213,
    CommandBnetAccountLink = 214,
    CommandBnetAccountUnlink = 215,
    CommandBnetAccountCreateGame = 216,
    CommandAccount = 217,
    CommandAccountAddon = 218,
    CommandAccountCreate = 219,
    CommandAccountDelete = 220,
    CommandAccountLock = 221,
    CommandAccountLockCountry = 222,
    CommandAccountLockIp = 223,
    CommandAccountOnlineList = 224,
    CommandAccountPassword = 225,
    CommandAccountSet = 226,
    CommandAccountSetAddon = 227,
    CommandAccountSetGmlevel = 228,
    CommandAccountSetPassword = 229,
    CommandAchievement = 230,
    CommandAchievementAdd = 231,
    CommandArena = 232,
    CommandArenaCaptain = 233,
    CommandArenaCreate = 234,
    CommandArenaDisband = 235,
    CommandArenaInfo = 236,
    CommandArenaLookup = 237,
    CommandArenaRename = 238,
    CommandBan = 239,
    CommandBanAccount = 240,
    CommandBanCharacter = 241,
    CommandBanIp = 242,
    CommandBanPlayeraccount = 243,
    CommandBaninfo = 244,
    CommandBaninfoAccount = 245,
    CommandBaninfoCharacter = 246,
    CommandBaninfoIp = 247,
    CommandBanlist = 248,
    CommandBanlistAccount = 249,
    CommandBanlistCharacter = 250,
    CommandBanlistIp = 251,
    CommandUnban = 252,
    CommandUnbanAccount = 253,
    CommandUnbanCharacter = 254,
    CommandUnbanIp = 255,
    CommandUnbanPlayeraccount = 256,
    CommandBf = 257,
    CommandBfStart = 258,
    CommandBfStop = 259,
    CommandBfSwitch = 260,
    CommandBfTimer = 261,
    CommandBfEnable = 262,
    CommandAccountEmail = 263,
    CommandAccountSetSec = 264,
    CommandAccountSetSecEmail = 265,
    CommandAccountSetSecRegmail = 266,
    CommandCast = 267,
    CommandCastBack = 268,
    CommandCastDist = 269,
    CommandCastSelf = 270,
    CommandCastTarget = 271,
    CommandCastDest = 272,
    CommandCharacter = 273,
    CommandCharacterCustomize = 274,
    CommandCharacterChangefaction = 275,
    CommandCharacterChangerace = 276,
    CommandCharacterDeleted = 277,
    CommandCharacterDeletedDelete = 278,
    CommandCharacterDeletedList = 279,
    CommandCharacterDeletedRestore = 280,
    CommandCharacterDeletedOld = 281,
    CommandCharacterErase = 282,
    CommandCharacterLevel = 283,
    CommandCharacterRename = 284,
    CommandCharacterReputation = 285,
    CommandCharacterTitles = 286,
    CommandLevelup = 287,
    CommandPdump = 288,
    CommandPdumpLoad = 289,
    CommandPdumpWrite = 290,
    CommandCheat = 291,
    CommandCheatCasttime = 292,
    CommandCheatCooldown = 293,
    CommandCheatExplore = 294,
    CommandCheatGod = 295,
    CommandCheatPower = 296,
    CommandCheatStatus = 297,
    CommandCheatTaxi = 298,
    CommandCheatWaterwalk = 299,
    CommandDebug = 300,
    CommandDebugAnim = 301,
    CommandDebugAreatriggers = 302,
    CommandDebugArena = 303,
    CommandDebugBg = 304,
    CommandDebugEntervehicle = 305,
    CommandDebugGetitemstate = 306,
    CommandDebugGetitemvalue = 307,
    CommandDebugGetvalue = 308,
    CommandDebugHostil = 309,
    CommandDebugItemexpire = 310,
    CommandDebugLootrecipient = 311,
    CommandDebugLos = 312,
    CommandDebugMod32Value = 313,
    CommandDebugMoveflags = 314,
    CommandDebugPlay = 315,
    CommandDebugPlayCinematic = 316,
    CommandDebugPlayMovie = 317,
    CommandDebugPlaySound = 318,
    CommandDebugSend = 319,
    CommandDebugSendBuyerror = 320,
    CommandDebugSendChannelnotify = 321,
    CommandDebugSendChatmessage = 322,
    CommandDebugSendEquiperror = 323,
    CommandDebugSendLargepacket = 324,
    CommandDebugSendOpcode = 325,
    CommandDebugSendQinvalidmsg = 326,
    CommandDebugSendQpartymsg = 327,
    CommandDebugSendSellerror = 328,
    CommandDebugSendSetphaseshift = 329,
    CommandDebugSendSpellfail = 330,
    CommandDebugSetaurastate = 331,
    CommandDebugSetbit = 332,
    CommandDebugSetitemvalue = 333,
    CommandDebugSetvalue = 334,
    CommandDebugSetvid = 335,
    CommandDebugSpawnvehicle = 336,
    CommandDebugThreat = 337,
    CommandDebugUpdate = 338,
    CommandDebugUws = 339,
    CommandWpgps = 340,
    CommandDeserter = 341,
    CommandDeserterBg = 342,
    CommandDeserterBgAdd = 343,
    CommandDeserterBgRemove = 344,
    CommandDeserterInstance = 345,
    CommandDeserterInstanceAdd = 346,
    CommandDeserterInstanceRemove = 347,
    CommandDisable = 348,
    CommandDisableAdd = 349,
    CommandDisableAddCriteria = 350,
    CommandDisableAddBattleground = 351,
    CommandDisableAddMap = 352,
    CommandDisableAddMmap = 353,
    CommandDisableAddOutdoorpvp = 354,
    CommandDisableAddQuest = 355,
    CommandDisableAddSpell = 356,
    CommandDisableAddVmap = 357,
    CommandDisableRemove = 358,
    CommandDisableRemoveCriteria = 359,
    CommandDisableRemoveBattleground = 360,
    CommandDisableRemoveMap = 361,
    CommandDisableRemoveMmap = 362,
    CommandDisableRemoveOutdoorpvp = 363,
    CommandDisableRemoveQuest = 364,
    CommandDisableRemoveSpell = 365,
    CommandDisableRemoveVmap = 366,
    CommandEvent = 367,
    CommandEventActivelist = 368,
    CommandEventStart = 369,
    CommandEventStop = 370,
    CommandGm = 371,
    CommandGmChat = 372,
    CommandGmFly = 373,
    CommandGmIngame = 374,
    CommandGmList = 375,
    CommandGmVisible = 376,
    CommandGo = 377,
    CommandGoCreature = 378,
    CommandGoGraveyard = 379,
    CommandGoGrid = 380,
    CommandGoObject = 381,
    CommandGoTaxinode = 382,
    CommandGoTicket = 383, // deprecated since Draenor DON'T reuse
    CommandGoTrigger = 384,
    CommandGoXyz = 385,
    CommandGoZonexy = 386,
    CommandGobject = 387,
    CommandGobjectActivate = 388,
    CommandGobjectAdd = 389,
    CommandGobjectAddTemp = 390,
    CommandGobjectDelete = 391,
    CommandGobjectInfo = 392,
    CommandGobjectMove = 393,
    CommandGobjectNear = 394,
    CommandGobjectSet = 395,
    CommandGobjectSetPhase = 396,
    CommandGobjectSetState = 397,
    CommandGobjectTarget = 398,
    CommandGobjectTurn = 399,
    CommandDebugTransport = 400,
    CommandGuild = 401,
    CommandGuildCreate = 402,
    CommandGuildDelete = 403,
    CommandGuildInvite = 404,
    CommandGuildUninvite = 405,
    CommandGuildRank = 406,
    CommandGuildRename = 407,
    CommandHonor = 408,
    CommandHonorAdd = 409,
    CommandHonorAddKill = 410,
    CommandHonorUpdate = 411,
    CommandInstance = 412,
    CommandInstanceListbinds = 413,
    CommandInstanceUnbind = 414,
    CommandInstanceStats = 415,
    CommandInstanceSavedata = 416,
    CommandLearn = 417,
    CommandLearnAll = 418,
    CommandLearnAllMy = 419,
    CommandLearnAllMyClass = 420,
    CommandLearnAllMyPettalents = 421,
    CommandLearnAllMySpells = 422,
    CommandLearnAllMyTalents = 423,
    CommandLearnAllGm = 424,
    CommandLearnAllCrafts = 425,
    CommandLearnAllDefault = 426,
    CommandLearnAllLang = 427,
    CommandLearnAllRecipes = 428,
    CommandUnlearn = 429,
    CommandLfg = 430,
    CommandLfgPlayer = 431,
    CommandLfgGroup = 432,
    CommandLfgQueue = 433,
    CommandLfgClean = 434,
    CommandLfgOptions = 435,
    CommandList = 436,
    CommandListCreature = 437,
    CommandListItem = 438,
    CommandListObject = 439,
    CommandListAuras = 440,
    CommandListMail = 441,
    CommandLookup = 442,
    CommandLookupArea = 443,
    CommandLookupCreature = 444,
    CommandLookupEvent = 445,
    CommandLookupFaction = 446,
    CommandLookupItem = 447,
    CommandLookupItemset = 448,
    CommandLookupObject = 449,
    CommandLookupQuest = 450,
    CommandLookupPlayer = 451,
    CommandLookupPlayerIp = 452,
    CommandLookupPlayerAccount = 453,
    CommandLookupPlayerEmail = 454,
    CommandLookupSkill = 455,
    CommandLookupSpell = 456,
    CommandLookupSpellId = 457,
    CommandLookupTaxinode = 458,
    CommandLookupTele = 459,
    CommandLookupTitle = 460,
    CommandLookupMap = 461,
    CommandAnnounce = 462,
    CommandChannel = 463,
    CommandChannelSet = 464,
    CommandChannelSetOwnership = 465,
    CommandGmannounce = 466,
    CommandGmnameannounce = 467,
    CommandGmnotify = 468,
    CommandNameannounce = 469,
    CommandNotify = 470,
    CommandWhispers = 471,
    CommandGroup = 472,
    CommandGroupLeader = 473,
    CommandGroupDisband = 474,
    CommandGroupRemove = 475,
    CommandGroupJoin = 476,
    CommandGroupList = 477,
    CommandGroupSummon = 478,
    CommandPet = 479,
    CommandPetCreate = 480,
    CommandPetLearn = 481,
    CommandPetUnlearn = 482,
    CommandSend = 483,
    CommandSendItems = 484,
    CommandSendMail = 485,
    CommandSendMessage = 486,
    CommandSendMoney = 487,
    CommandAdditem = 488,
    CommandAdditemset = 489,
    CommandAppear = 490,
    CommandAura = 491,
    CommandBank = 492,
    CommandBindsight = 493,
    CommandCombatstop = 494,
    CommandCometome = 495,
    CommandCommands = 496,
    CommandCooldown = 497,
    CommandDamage = 498,
    CommandDev = 499,
    CommandDie = 500,
    CommandDismount = 501,
    CommandDistance = 502,
    CommandFlusharenapoints = 503,
    CommandFreeze = 504,
    CommandGps = 505,
    CommandGuid = 506,
    CommandHelp = 507,
    CommandHidearea = 508,
    CommandItemmove = 509,
    CommandKick = 510,
    CommandLinkgrave = 511,
    CommandListfreeze = 512,
    CommandMaxskill = 513,
    CommandMovegens = 514,
    CommandMute = 515,
    CommandNeargrave = 516,
    CommandPinfo = 517,
    CommandPlayall = 518,
    CommandPossess = 519,
    CommandRecall = 520,
    CommandRepairitems = 521,
    CommandRespawn = 522,
    CommandRevive = 523,
    CommandSaveall = 524,
    CommandSave = 525,
    CommandSetskill = 526,
    CommandShowarea = 527,
    CommandSummon = 528,
    CommandUnaura = 529,
    CommandUnbindsight = 530,
    CommandUnfreeze = 531,
    CommandUnmute = 532,
    CommandUnpossess = 533,
    CommandUnstuck = 534,
    CommandWchange = 535,
    CommandMmap = 536,
    CommandMmapLoadedtiles = 537,
    CommandMmapLoc = 538,
    CommandMmapPath = 539,
    CommandMmapStats = 540,
    CommandMmapTestarea = 541,
    CommandMorph = 542,
    CommandDemorph = 543,
    CommandModify = 544,
    CommandModifyArenapoints = 545,
    CommandModifyBit = 546,
    CommandModifyDrunk = 547,
    CommandModifyEnergy = 548,
    CommandModifyFaction = 549,
    CommandModifyGender = 550,
    CommandModifyHonor = 551,
    CommandModifyHp = 552,
    CommandModifyMana = 553,
    CommandModifyMoney = 554,
    CommandModifyMount = 555,
    CommandModifyPhase = 556,
    CommandModifyRage = 557,
    CommandModifyReputation = 558,
    CommandModifyRunicpower = 559,
    CommandModifyScale = 560,
    CommandModifySpeed = 561,
    CommandModifySpeedAll = 562,
    CommandModifySpeedBackwalk = 563,
    CommandModifySpeedFly = 564,
    CommandModifySpeedWalk = 565,
    CommandModifySpeedSwim = 566,
    CommandModifySpell = 567,
    CommandModifyStandstate = 568,
    CommandModifyTalentpoints = 569,
    CommandNpc = 570,
    CommandNpcAdd = 571,
    CommandNpcAddFormation = 572,
    CommandNpcAddItem = 573,
    CommandNpcAddMove = 574,
    CommandNpcAddTemp = 575,
    CommandNpcDelete = 576,
    CommandNpcDeleteItem = 577,
    CommandNpcFollow = 578,
    CommandNpcFollowStop = 579,
    CommandNpcSet = 580,
    CommandNpcSetAllowmove = 581,
    CommandNpcSetEntry = 582,
    CommandNpcSetFactionid = 583,
    CommandNpcSetFlag = 584,
    CommandNpcSetLevel = 585,
    CommandNpcSetLink = 586,
    CommandNpcSetModel = 587,
    CommandNpcSetMovetype = 588,
    CommandNpcSetPhase = 589,
    CommandNpcSetSpawndist = 590,
    CommandNpcSetSpawntime = 591,
    CommandNpcSetData = 592,
    CommandNpcInfo = 593,
    CommandNpcNear = 594,
    CommandNpcMove = 595,
    CommandNpcPlayemote = 596,
    CommandNpcSay = 597,
    CommandNpcTextemote = 598,
    CommandNpcWhisper = 599,
    CommandNpcYell = 600,
    CommandNpcTame = 601,
    CommandQuest = 602,
    CommandQuestAdd = 603,
    CommandQuestComplete = 604,
    CommandQuestRemove = 605,
    CommandQuestReward = 606,
    CommandReload = 607,
    CommandReloadAccessRequirement = 608,
    CommandReloadCriteriaData = 609,
    CommandReloadAchievementReward = 610,
    CommandReloadAll = 611,
    CommandReloadAllAchievement = 612,
    CommandReloadAllArea = 613,
    CommandReloadBroadcastText = 614,
    CommandReloadAllGossip = 615,
    CommandReloadAllItem = 616,
    CommandReloadAllLocales = 617,
    CommandReloadAllLoot = 618,
    CommandReloadAllNpc = 619,
    CommandReloadAllQuest = 620,
    CommandReloadAllScripts = 621,
    CommandReloadAllSpell = 622,
    CommandReloadAreatriggerInvolvedrelation = 623,
    CommandReloadAreatriggerTavern = 624,
    CommandReloadAreatriggerTeleport = 625,
    CommandReloadAuctions = 626,
    CommandReloadAutobroadcast = 627,
    CommandReloadCommand = 628,
    CommandReloadConditions = 629,
    CommandReloadConfig = 630,
    CommandReloadBattlegroundTemplate = 631,
    CommandMutehistory = 632,
    CommandReloadCreatureLinkedRespawn = 633,
    CommandReloadCreatureLootTemplate = 634,
    CommandReloadCreatureOnkillReputation = 635,
    CommandReloadCreatureQuestender = 636,
    CommandReloadCreatureQueststarter = 637,
    CommandReloadCreatureSummonGroups = 638,
    CommandReloadCreatureTemplate = 639,
    CommandReloadCreatureText = 640,
    CommandReloadDisables = 641,
    CommandReloadDisenchantLootTemplate = 642,
    CommandReloadEventScripts = 643,
    CommandReloadFishingLootTemplate = 644,
    CommandReloadGraveyardZone = 645,
    CommandReloadGameTele = 646,
    CommandReloadGameobjectQuestender = 647,
    CommandReloadGameobjectQuestLootTemplate = 648,
    CommandReloadGameobjectQueststarter = 649,
    CommandReloadSupportSystem = 650,
    CommandReloadGossipMenu = 651,
    CommandReloadGossipMenuOption = 652,
    CommandReloadItemEnchantmentTemplate = 653,
    CommandReloadItemLootTemplate = 654,
    CommandReloadItemSetNames = 655,
    CommandReloadLfgDungeonRewards = 656,
    CommandReloadLocalesAchievementReward = 657,
    CommandReloadLocalesCreture = 658,
    CommandReloadLocalesCretureText = 659,
    CommandReloadLocalesGameobject = 660,
    CommandReloadLocalesGossipMenuOption = 661,
    CommandReloadLocalesItem = 662, // deprecated since Draenor DON'T reuse
    CommandReloadLocalesItemSetName = 663,
    CommandReloadLocalesNpcText = 664, // deprecated since Draenor DON'T reuse
    CommandReloadLocalesPageText = 665,
    CommandReloadLocalesPointsOfInterest = 666,
    CommandReloadQuestLocale = 667,
    CommandReloadMailLevelReward = 668,
    CommandReloadMailLootTemplate = 669,
    CommandReloadMillingLootTemplate = 670,
    CommandReloadNpcSpellclickSpells = 671,
    CommandReloadTrainer = 672,
    CommandReloadNpcVendor = 673,
    CommandReloadPageText = 674,
    CommandReloadPickpocketingLootTemplate = 675,
    CommandReloadPointsOfInterest = 676,
    CommandReloadProspectingLootTemplate = 677,
    CommandReloadQuestPoi = 678,
    CommandReloadQuestTemplate = 679,
    CommandReloadRbac = 680,
    CommandReloadReferenceLootTemplate = 681,
    CommandReloadReservedName = 682,
    CommandReloadReputationRewardRate = 683,
    CommandReloadSpilloverTemplate = 684,
    CommandReloadSkillDiscoveryTemplate = 685,
    CommandReloadSkillExtraItemTemplate = 686,
    CommandReloadSkillFishingBaseLevel = 687,
    CommandReloadSkinningLootTemplate = 688,
    CommandReloadSmartScripts = 689,
    CommandReloadSpellRequired = 690,
    CommandReloadSpellArea = 691,
    CommandReloadSpellBonusData = 692, // deprecated since Draenor DON'T reuse
    CommandReloadSpellGroup = 693,
    CommandReloadSpellLearnSpell = 694,
    CommandReloadSpellLootTemplate = 695,
    CommandReloadSpellLinkedSpell = 696,
    CommandReloadSpellPetAuras = 697,
    CommandCharacterChangeaccount = 698,
    CommandReloadSpellProc = 699,
    CommandReloadSpellScripts = 700,
    CommandReloadSpellTargetPosition = 701,
    CommandReloadSpellThreats = 702,
    CommandReloadSpellGroupStackRules = 703,
    CommandReloadAzothaString = 704,
    CommandReloadWardenAction = 705,
    CommandReloadWaypointScripts = 706,
    CommandReloadWaypointData = 707,
    CommandReloadVehicleAccesory = 708,
    CommandReloadVehicleTemplateAccessory = 709,
    CommandReset = 710,
    CommandResetAchievements = 711,
    CommandResetHonor = 712,
    CommandResetLevel = 713,
    CommandResetSpells = 714,
    CommandResetStats = 715,
    CommandResetTalents = 716,
    CommandResetAll = 717,
    CommandServer = 718,
    CommandServerCorpses = 719,
    CommandServerExit = 720,
    CommandServerIdlerestart = 721,
    CommandServerIdlerestartCancel = 722,
    CommandServerIdleshutdown = 723,
    CommandServerIdleshutdownCancel = 724,
    CommandServerInfo = 725,
    CommandServerPlimit = 726,
    CommandServerRestart = 727,
    CommandServerRestartCancel = 728,
    CommandServerSet = 729,
    CommandServerSetClosed = 730,
    CommandServerSetDifftime = 731,
    CommandServerSetLoglevel = 732,
    CommandServerSetMotd = 733,
    CommandServerShutdown = 734,
    CommandServerShutdownCancel = 735,
    CommandServerMotd = 736,
    CommandTele = 737,
    CommandTeleAdd = 738,
    CommandTeleDel = 739,
    CommandTeleName = 740,
    CommandTeleGroup = 741,
    CommandTicket = 742,
    CommandTicketAssign = 743,        // deprecated since Draenor DON'T reuse
    CommandTicketClose = 744,         // deprecated since Draenor DON'T reuse
    CommandTicketClosedlist = 745,    // deprecated since Draenor DON'T reuse
    CommandTicketComment = 746,       // deprecated since Draenor DON'T reuse
    CommandTicketComplete = 747,      // deprecated since Draenor DON'T reuse
    CommandTicketDelete = 748,        // deprecated since Draenor DON'T reuse
    CommandTicketEscalate = 749,      // deprecated since Draenor DON'T reuse
    CommandTicketEscalatedlist = 750, // deprecated since Draenor DON'T reuse
    CommandTicketList = 751,          // deprecated since Draenor DON'T reuse
    CommandTicketOnlinelist = 752,    // deprecated since Draenor DON'T reuse
    CommandTicketReset = 753,
    CommandTicketResponse = 754,         // deprecated since Draenor DON'T reuse
    CommandTicketResponseAppend = 755,   // deprecated since Draenor DON'T reuse
    CommandTicketResponseAppendln = 756, // deprecated since Draenor DON'T reuse
    CommandTicketTogglesystem = 757,
    CommandTicketUnassign = 758, // deprecated since Draenor DON'T reuse
    CommandTicketViewid = 759,   // deprecated since Draenor DON'T reuse
    CommandTicketViewname = 760, // deprecated since Draenor DON'T reuse
    CommandTitles = 761,
    CommandTitlesAdd = 762,
    CommandTitlesCurrent = 763,
    CommandTitlesRemove = 764,
    CommandTitlesSet = 765,
    CommandTitlesSetMask = 766,
    CommandWp = 767,
    CommandWpAdd = 768,
    CommandWpEvent = 769,
    CommandWpLoad = 770,
    CommandWpModify = 771,
    CommandWpUnload = 772,
    CommandWpReload = 773,
    CommandWpShow = 774,
    CommandModifyCurrency = 775,
    CommandDebugPhase = 776,
    CommandMailbox = 777,
    CommandAhbot = 778,
    CommandAhbotItems = 779,
    CommandAhbotItemsGray = 780,
    CommandAhbotItemsWhite = 781,
    CommandAhbotItemsGreen = 782,
    CommandAhbotItemsBlue = 783,
    CommandAhbotItemsPurple = 784,
    CommandAhbotItemsOrange = 785,
    CommandAhbotItemsYellow = 786,
    CommandAhbotRatio = 787,
    CommandAhbotRatioAlliance = 788,
    CommandAhbotRatioHorde = 789,
    CommandAhbotRatioNeutral = 790,
    CommandAhbotRebuild = 791,
    CommandAhbotReload = 792,
    CommandAhbotStatus = 793,
    CommandGuildInfo = 794,
    CommandInstanceSetBossState = 795,
    CommandInstanceGetBossState = 796,
    CommandPvpstats = 797,
    CommandModifyXp = 798,
    CommandGoBugTicket = 799,
    CommandGoComplaintTicket = 800,
    CommandGoSuggestionTicket = 801,
    CommandTicketBug = 802,
    CommandTicketComplaint = 803,
    CommandTicketSuggestion = 804,
    CommandTicketBugAssign = 805,
    CommandTicketBugClose = 806,
    CommandTicketBugClosedlist = 807,
    CommandTicketBugComment = 808,
    CommandTicketBugDelete = 809,
    CommandTicketBugList = 810,
    CommandTicketBugUnassign = 811,
    CommandTicketBugView = 812,
    CommandTicketComplaintAssign = 813,
    CommandTicketComplaintClose = 814,
    CommandTicketComplaintClosedlist = 815,
    CommandTicketComplaintComment = 816,
    CommandTicketComplaintDelete = 817,
    CommandTicketComplaintList = 818,
    CommandTicketComplaintUnassign = 819,
    CommandTicketComplaintView = 820,
    CommandTicketSuggestionAssign = 821,
    CommandTicketSuggestionClose = 822,
    CommandTicketSuggestionClosedlist = 823,
    CommandTicketSuggestionComment = 824,
    CommandTicketSuggestionDelete = 825,
    CommandTicketSuggestionList = 826,
    CommandTicketSuggestionUnassign = 827,
    CommandTicketSuggestionView = 828,
    CommandTicketResetAll = 829,
    CommandBnetAccountListGameAccounts = 830,
    CommandTicketResetBug = 831,
    CommandTicketResetComplaint = 832,
    CommandTicketResetSuggestion = 833,
    CommandGoQuest = 834,
    CommandDebugLoadcells = 835,
    CommandDebugBoundary = 836,
    CommandNpcEvade = 837,
    CommandPetLevel = 838,
    CommandServerShutdownForce = 839,
    CommandServerRestartForce = 840,
    CommandNeargraveyard = 841,
    CommandReloadCharacterTemplate = 842,
    CommandReloadQuestGreeting = 843,
    CommandScene = 844,
    CommandSceneDebug = 845,
    CommandScenePlay = 846,
    CommandScenePlayPackage = 847,
    CommandSceneCancel = 848,
    CommandListScenes = 849,
    CommandReloadSceneTemplate = 850,
    CommandReloadAreatriggerTemplate = 851,
    CommandGoOffset = 852,
    CommandReloadConversationTemplate = 853,
    CommandDebugConversation = 854,
    CommandNpcSpawngroup = 856,             // reserved for dynamic_spawning
    CommandNpcDespawngroup = 857,           // reserved for dynamic_spawning
    CommandGobjectSpawngroup = 858,         // reserved for dynamic_spawning
    CommandGobjectDespawngroup = 859,       // reserved for dynamic_spawning
    CommandListRespawns = 860,              // reserved for dynamic_spawning
    CommandGroupSet = 861,                  // reserved
    CommandGroupAssistant = 862,            // reserved
    CommandGroupMaintank = 863,             // reserved
    CommandGroupMainassist = 864,           // reserved
    CommandNpcShowloot = 865,               // reserved
    CommandListSpawnpoints = 866,           // reserved
    CommandReloadQuestGreetingLocale = 867, // reserved
    CommandModifyPower = 868,
    CommandDebugSendPlayerChoice = 869,
    CommandDebugThreatinfo = 870,    // reserved
    CommandDebugInstancespawn = 871, // reserved
    CommandServerDebug = 872,
}

impl TryFrom<u32> for RbacPermId {
    type Error = u32;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if let Some(rbac_id) = FromPrimitive::from_u32(value) {
            Ok(rbac_id)
        } else {
            Err(value)
        }
    }
}

impl From<RbacPermId> for u32 {
    fn from(value: RbacPermId) -> Self {
        ToPrimitive::to_u32(&value).unwrap()
    }
}

pub type RbacCommandResult<T> = Result<T, RbacCommandError>;

#[derive(thiserror::Error, Debug)]
pub enum RbacCommandError {
    #[error("RbacCommandError: db internal error: {0}")]
    DbInternal(#[from] sqlx::Error),
    #[error("can't add RBAC command if already added")]
    CantAddAlreadyAdded,
    #[error("can't revoke RBAC comand as its not in list")]
    CantRevokeNotInList,
    #[error("RBAC is already in granted list")]
    InGrantedList,
    #[error("RBAC is already in denied list")]
    InDeniedList,
    #[error("rbac permissions ID does not exist")]
    IdDoesNotExists,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RbacPermission {
    /// id of the object
    pub id:                 RawRbacPermId,
    /// name of the object
    pub name:               String,
    /// Set of permissions
    pub linked_permissions: BTreeSet<RawRbacPermId>,
}

// /// @name RbacData
// /// @brief Contains all needed information about the acccount
// ///
// /// This class contains all the data needed to calculate the account permissions.
// /// RbacData is formed by granted and denied permissions and all the inherited permissions
// ///
// /// Calculation of current Permissions: Granted permissions - Denied permissions
// /// - Granted permissions: through linked permissions and directly assigned
// /// - Denied permissions: through linked permissions and directly assigned
// ///
// pub struct RbacData {
//     /// Account id
//     pub id:            u32,
//     /// Account name
//     pub name:          String,
//     /// RealmId Affected
//     pub realm_id:      u32,
//     /// Account SecurityLevel
//     pub sec_level:     AccountTypes,
//     /// Granted permissions
//     pub granted_perms: BTreeSet<RawRbacPermId>,
//     /// Denied permissions
//     pub denied_perms:  BTreeSet<RawRbacPermId>,
//     /// Calculated permissions, i.e. the delta of granted_perms and denied_perms
//     global_perms:      BTreeSet<RawRbacPermId>,
// }

// impl RbacData {
//     pub fn new(id: u32, name: &str, realm_id: u32, sec_level: AccountTypes) -> Self {
//         Self {
//             id,
//             name: name.to_string(),
//             realm_id,
//             sec_level,
//             granted_perms: BTreeSet::new(),
//             denied_perms: BTreeSet::new(),
//             global_perms: BTreeSet::new(),
//         }
//     }

//     ///
//     /// @name has_permission
//     /// @brief Checks if certain action is allowed
//     ///
//     /// Checks if certain action can be performed.
//     ///
//     /// @return grant or deny action
//     ///
//     /// Example Usage:
//     /// @code
//     /// bool Player::CanJoinArena(Battleground* bg)
//     /// {
//     ///     return bg->isArena() && has_permission(RBAC_PERM_JOIN_ARENA);
//     /// }
//     /// @endcode
//     ///
//     pub fn has_permission(&self, permission: RbacPermId) -> bool {
//         self.global_perms.contains(&Ok(permission))
//     }

//     /// @name grant_permission
//     /// @brief Grants a permission
//     ///
//     /// Grants a permission to the account. If realm is 0 or the permission can not be added
//     /// No save to db action will be performed.
//     ///
//     /// Fails if permission Id does not exists or permission already granted or denied
//     ///
//     /// @param permission_id permission to be granted
//     /// @param realm_id realm affected
//     ///
//     /// @return Success or failure (with reason) to grant the permission
//     ///
//     /// Example Usage:
//     /// @code
//     /// // previously defined "RbacData* rbac" with proper initialization
//     /// permission_id: RbacPermId  = 2;
//     /// if (rbac->GrantRole(permission_id) == RBAC_IN_DENIED_LIST)
//     ///     TC_LOG_DEBUG("entities.player", "Failed to grant permission %u, already denied", permission_id);
//     /// @endcode
//     ///
//     #[instrument(skip_all, target="rbac", fields(id=self.id, name=self.name, permission=?permission_id, realm=realm_id))]
//     pub async fn grant_permission(&mut self, permission_id: RawRbacPermId, realm_id: Option<u32>) -> RbacCommandResult<()> {
//         // Check if permission Id exists
//         if ACCOUNT_MGR.read().await.get_rbac_permission(permission_id).is_none() {
//             trace!("RBACData::GrantPermission: Permission does not exists");
//             return Err(RbacCommandError::IdDoesNotExists);
//         }
//         // Check if already added in denied list
//         if self.has_denied_permission(permission_id) {
//             trace!("RBACData::GrantPermission: Permission in deny list");
//             return Err(RbacCommandError::InDeniedList);
//         }

//         // Already added?
//         if self.has_granted_permission(permission_id) {
//             trace!("RBACData::GrantPermission: Permission already granted");
//             return Err(RbacCommandError::CantAddAlreadyAdded);
//         }

//         // Do not save to db when loading data from DB realm_id( = 0)
//         if let Some(realm_id) = realm_id {
//             self.save_permission(permission_id, true, realm_id).await?;
//             trace!("RBACData::GrantPermission: Ok and DB updated");
//             self.add_granted_permission(permission_id);
//             self.calculate_new_permissions();
//         } else {
//             self.add_granted_permission(permission_id);
//             trace!("RBACData::GrantPermission: Ok");
//         }

//         Ok(())
//     }

//     /// @name deny_permission
//     /// @brief Denies a permission
//     ///
//     /// Denied a permission to the account. If realm is 0 or the permission can not be added
//     /// No save to db action will be performed.
//     ///
//     /// Fails if permission Id does not exists or permission already granted or denied
//     ///
//     /// @param permission_id permission to be denied
//     /// @param realm_id realm affected
//     ///
//     /// @return Success or failure (with reason) to deny the permission
//     ///
//     /// Example Usage:
//     /// @code
//     /// // previously defined "RbacData* rbac" with proper initialization
//     /// permission_id: RbacPermId  = 2;
//     /// if (rbac->DenyRole(permission_id) == RBAC_ID_DOES_NOT_EXISTS)
//     ///     TC_LOG_DEBUG("entities.player", "Role Id %u does not exists", permission_id);
//     /// @endcode
//     ///
//     #[instrument(skip_all, target="rbac", fields(id=self.id, name=self.name, permission=?permission_id, realm=realm_id))]
//     pub async fn deny_permission(&mut self, permission_id: RawRbacPermId, realm_id: Option<u32>) -> RbacCommandResult<()> {
//         // Check if permission Id exists
//         if ACCOUNT_MGR.blocking_read().get_rbac_permission(permission_id).is_none() {
//             trace!("RBACData::DenyPermission. Permission does not exists");
//             return Err(RbacCommandError::IdDoesNotExists);
//         }

//         // Check if already added in granted list
//         if self.has_granted_permission(permission_id) {
//             trace!("RBACData::DenyPermission. Permission in grant list");
//             return Err(RbacCommandError::InGrantedList);
//         }

//         // Already added?
//         if self.has_denied_permission(permission_id) {
//             trace!("RBACData::DenyPermission. Permission already denied");
//             return Err(RbacCommandError::CantAddAlreadyAdded);
//         }

//         // Do not save to db when loading data from DB realm_id: Option<u32>( = 0)
//         if let Some(realm_id) = realm_id {
//             trace!("RBACData::DenyPermission. Ok and DB updated");
//             self.save_permission(permission_id, false, realm_id).await?;
//             self.add_denied_permission(permission_id);
//             self.calculate_new_permissions();
//         } else {
//             self.add_denied_permission(permission_id);
//             trace!("RBACData::DenyPermission. Ok");
//         }

//         Ok(())
//     }

//     /// @name revoke_permission
//     /// @brief Removes a permission
//     ///
//     /// Removes a permission from the account. If realm is 0 or the permission can not be removed
//     /// No save to db action will be performed. Any delete operation will always affect
//     /// "all realms (-1)" in addition to the realm specified
//     ///
//     /// Fails if permission not present
//     ///
//     /// @param permission_id permission to be removed
//     /// @param realm_id realm affected
//     ///
//     /// @return Success or failure (with reason) to remove the permission
//     ///
//     /// Example Usage:
//     /// @code
//     /// // previously defined "RbacData* rbac" with proper initialization
//     /// permission_id: RbacPermId  = 2;
//     /// if (rbac->RevokeRole(permission_id) == RBAC_OK)
//     ///     TC_LOG_DEBUG("entities.player", "Permission %u succesfully removed", permission_id);
//     /// @endcode
//     ///
//     #[instrument(skip_all, target="rbac", fields(id=self.id, name=self.name, permission=?permission_id, realm=realm_id))]
//     pub async fn revoke_permission(&mut self, permission_id: RawRbacPermId, realm_id: Option<u32>) -> RbacCommandResult<()> {
//         // Check if it's present in any list
//         if !self.has_granted_permission(permission_id) && !self.has_denied_permission(permission_id) {
//             trace!("RBACData::RevokePermission Not granted or revoked");
//             return Err(RbacCommandError::CantRevokeNotInList);
//         }

//         // Do not save to db when loading data from DB realm_id: Option<u32>( = 0)
//         if let Some(realm_id) = realm_id {
//             let permission_id_in_u32 = permission_id.map_or_else(|e| e, |v| v.into());
//             LoginDatabase::del_rbac_account_permission(&LoginDatabase::get(), args!(self.id, permission_id_in_u32, realm_id)).await?;
//             trace!("RBACData::RevokePermission Ok and DB updated");
//             self.remove_granted_permission(permission_id);
//             self.remove_denied_permission(permission_id);
//             self.calculate_new_permissions();
//         } else {
//             trace!("RBACData::RevokePermission. Ok");
//             self.remove_granted_permission(permission_id);
//             self.remove_denied_permission(permission_id);
//         }

//         Ok(())
//     }

//     /// Loads all permissions assigned to current account
//     #[instrument(skip_all, target="rbac", fields(id=self.id, name=self.name))]
//     pub async fn load_from_db(&mut self) -> RbacCommandResult<()> {
//         self.clear_data();

//         debug!(target:"rbac", "RBACData::LoadFromDB: Loading permissions");

//         #[derive(sqlx::FromRow)]
//         struct DbRbacAccountRow {
//             #[sqlx(rename = "permissionId")]
//             permission_id: u32,
//             #[sqlx(rename = "granted")]
//             granted:       bool,
//         }

//         // Load account permissions (granted and denied) that affect current realm
//         let result = LoginDatabase::sel_rbac_account_permissions::<_, DbRbacAccountRow>(&LoginDatabase::get(), args!(self.id, self.realm_id)).await?;
//         for DbRbacAccountRow { permission_id, granted } in result {
//             let permission_id = permission_id.try_into();
//             if granted {
//                 self.grant_permission(permission_id, None).await?;
//             } else {
//                 self.deny_permission(permission_id, None).await?;
//             }
//         }
//         // Add default permissions
//         if let Some(permissions) = ACCOUNT_MGR.read().await.get_rbac_default_permissions(self.sec_level) {
//             for permission_id in permissions {
//                 self.grant_permission(*permission_id, None).await?;
//             }
//         }
//         // Force calculation of permissions
//         self.calculate_new_permissions();
//         Ok(())
//     }

//     /// Sets security level
//     pub async fn set_security_level(&mut self, id: AccountTypes) -> RbacCommandResult<()> {
//         self.sec_level = id;
//         self.load_from_db().await
//     }

//     //     /// Returns the security level assigned
//     //     uint8 GetSecurityLevel() const { return _secLevel; }
//     // private:
//     /// Saves a permission to DB, Granted or Denied
//     async fn save_permission(&self, permission: RawRbacPermId, granted: bool, realm: u32) -> RbacCommandResult<()> {
//         let login_db = &LoginDatabase::get();
//         let permission = permission.map_or_else(|e| e, |v| v.into());
//         LoginDatabase::ins_rbac_account_permission(login_db, args!(self.id, permission, granted, realm)).await?;
//         Ok(())
//     }

//     /// Clears roles, groups and permissions - Used for reload
//     fn clear_data(&mut self) {
//         self.granted_perms.clear();
//         self.denied_perms.clear();
//         self.global_perms.clear();
//     }

//     ///
//     /// @name calculate_new_permissions
//     /// @brief Calculates new permissions
//     ///
//     /// Calculates new permissions after some change
//     /// The calculation is done Granted - Denied:
//     /// - Granted permissions: through linked permissions and directly assigned
//     /// - Denied permissions: through linked permissions and directly assigned
//     ///
//     fn calculate_new_permissions(&mut self) {
//         trace!(target:"rbac", account_id=self.id, name=self.name, "RBACData::CalculateNewPermissions");

//         // Get the list of granted permissions and replace global permissions
//         self.global_perms = self.granted_perms.clone();
//         Self::expand_permissions(&mut self.global_perms);
//         let mut revoked = self.denied_perms.clone();
//         Self::expand_permissions(&mut revoked);
//         Self::remove_permissions(&mut self.global_perms, &revoked);
//     }

//     // Auxiliar private functions - defined to allow to maintain same code even
//     // if internal structure changes.

//     /// Checks if a permission is granted
//     fn has_granted_permission(&self, permission_id: RawRbacPermId) -> bool {
//         self.granted_perms.contains(&permission_id)
//     }

//     /// Checks if a permission is denied
//     fn has_denied_permission(&self, permission_id: RawRbacPermId) -> bool {
//         self.denied_perms.contains(&permission_id)
//     }

//     /// Adds a new granted permission
//     fn add_granted_permission(&mut self, permission_id: RawRbacPermId) -> bool {
//         self.granted_perms.insert(permission_id)
//     }

//     /// Removes a granted permission
//     fn remove_granted_permission(&mut self, permission_id: RawRbacPermId) -> bool {
//         self.granted_perms.remove(&permission_id)
//     }

//     /// Adds a new denied permission
//     fn add_denied_permission(&mut self, permission_id: RawRbacPermId) {
//         self.denied_perms.insert(permission_id);
//     }

//     /// Removes a denied permission
//     fn remove_denied_permission(&mut self, permission_id: RawRbacPermId) -> bool {
//         self.denied_perms.remove(&permission_id)
//     }

//     // /// Removes a denied permission
//     // void RemoveDeniedPermission(permission_id: RbacPermId )
//     // {
//     //     self.denied_perms.erase(permission_id);
//     // }

//     // /// Removes a list of permissions from another list
//     fn remove_permissions(perms_from: &mut BTreeSet<RawRbacPermId>, perms_to_remove: &BTreeSet<RawRbacPermId>) {
//         for p in perms_to_remove {
//             perms_from.remove(p);
//         }
//     }

//     /**
//      * @name expand_permissions
//      * @brief Adds the list of linked permissions to the original list
//      *
//      * Given a list of permissions, gets all the inherited permissions
//      * @param permissions The list of permissions to expand
//      */
//     fn expand_permissions(permissions: &mut BTreeSet<RawRbacPermId>) {
//         let mut to_check = permissions.clone();
//         permissions.clear();

//         // remove the permission from original list
//         let accnt_mgr = ACCOUNT_MGR.blocking_read();
//         while let Some(perm_id) = to_check.pop_first() {
//             let Some(permission) = accnt_mgr.get_rbac_permission(perm_id) else {
//                 continue;
//             };
//             // insert into the final list (expanded list)
//             permissions.insert(perm_id);
//             // add all linked permissions (that are not already expanded) to the list of permissions to be checked
//             for linked in &permission.linked_permissions {
//                 if permissions.get(linked).is_none() {
//                     to_check.insert(*linked);
//                 }
//             }
//         }

//         debug!(target:"rbac", "RBACData::ExpandPermissions: Expanded: {perms:?}", perms=permissions);
//     }
// }
