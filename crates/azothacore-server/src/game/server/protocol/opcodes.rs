/// Player state
pub enum SessionStatus {
    /// Player authenticated (_player == NULL, m_playerRecentlyLogout = false or will be reset before handler call, m_GUID have garbage)
    Authed,
    /// Player in game (_player != NULL, m_GUID == _player->GetGUID(), inWorld())
    LoggedIn,
    /// Player transferring to another map (_player != NULL, m_GUID == _player->GetGUID(), !inWorld())
    Transfer,
    /// _player != NULL or _player == NULL && m_playerRecentlyLogout && m_playerLogout, m_GUID store last _player guid)
    LoggedInOrRecentlyLogout,
    /// Opcode not accepted from client (deprecated or server side only)
    Never,
    /// Opcode not handled yet
    Unhandled,
}

/// PacketProcessing, this is mainly leftover metadata from TC / AC implementation to serve
/// as a reference. May or not not be used for now
pub enum PacketProcessing {
    ///TC Notes: process packet whenever we receive it - mostly for non-handled or non-implemented packets
    Inplace,
    ///TC Notes: packet is not thread-safe - process it in World::UpdateSessions()
    Threadunsafe,
    ///TC Notes: packet is thread-safe - process it in Map::Update()
    Threadsafe,
}

#[derive(Clone)]
pub enum ConnectionType {
    Realm,
    Instance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_derive::FromPrimitive)]
#[repr(u16)]
pub enum OpcodeClient {
    AcceptGuildInvite = 0x35FC,
    AcceptLevelGrant = 0x34FD,
    AcceptTrade = 0x315A,
    AcceptWargameInvite = 0x35E0,
    ActivateTaxi = 0x34AE,
    AddonList = 0x35D8,
    AddBattlenetFriend = 0x365A,
    AddFriend = 0x36CF,
    AddIgnore = 0x36D3,
    AddToy = 0x328B,
    AdventureJournalOpenQuest = 0x31F9,
    AdventureJournalStartQuest = 0x332E,
    AlterAppearance = 0x34F9,
    AreaSpiritHealerQuery = 0x34B3,
    AreaSpiritHealerQueue = 0x34B4,
    AreaTrigger = 0x31CD,
    ArtifactAddPower = 0x31A5,
    ArtifactAddRelicTalent = 0x31A8,
    ArtifactAttunePreviewRelic = 0x31A9,
    ArtifactAttuneSocketedRelic = 0x31AA,
    ArtifactSetAppearance = 0x31A7,
    AssignEquipmentSetSpec = 0x3200,
    AttackStop = 0x324D,
    AttackSwing = 0x324C,
    AuctionHelloRequest = 0x34CF,
    AuctionListBidderItems = 0x34D5,
    AuctionListItems = 0x34D2,
    AuctionListOwnerItems = 0x34D4,
    AuctionListPendingSales = 0x34D7,
    AuctionPlaceBid = 0x34D6,
    AuctionRemoveItem = 0x34D1,
    AuctionReplicateItems = 0x34D3,
    AuctionSellItem = 0x34D0,
    AuthContinuedSession = 0x3766,
    AuthSession = 0x3765,
    AutobankItem = 0x3996,
    AutobankReagent = 0x3998,
    AutostoreBankItem = 0x3997,
    AutostoreBankReagent = 0x3999,
    AutoEquipItem = 0x399A,
    AutoEquipItemSlot = 0x399F,
    AutoStoreBagItem = 0x399B,
    BankerActivate = 0x34B6,
    BattlefieldLeave = 0x3171,
    BattlefieldList = 0x317D,
    BattlefieldPort = 0x3529,
    BattlemasterHello = 0x32A1,
    BattlemasterJoin = 0x3524,
    BattlemasterJoinArena = 0x3525,
    BattlemasterJoinBrawl = 0x3527,
    BattlemasterJoinSkirmish = 0x3526,
    BattlenetChallengeResponse = 0x36D2,
    BattlenetRequest = 0x36F6,
    BattlenetRequestRealmListTicket = 0x36F7,
    BattlePayAckFailedResponse = 0x36CA,
    BattlePayConfirmPurchaseResponse = 0x36C9,
    BattlePayDistributionAssignToTarget = 0x36C0,
    BattlePayGetProductList = 0x36BA,
    BattlePayGetPurchaseList = 0x36BB,
    BattlePayQueryClassTrialBoostResult = 0x36C3,
    BattlePayRequestCharacterBoostUnrevoke = 0x36C1,
    BattlePayRequestCurrentVasTransferQueues = 0x3708,
    BattlePayRequestPriceInfo = 0x3707,
    BattlePayRequestVasCharacterQueueTime = 0x3709,
    BattlePayStartPurchase = 0x36F2,
    BattlePayStartVasPurchase = 0x36F3,
    BattlePayTrialBoostCharacter = 0x36C2,
    BattlePayValidateBnetVasTransfer = 0x370A,
    BattlePetClearFanfare = 0x312C,
    BattlePetDeletePet = 0x3624,
    BattlePetDeletePetCheat = 0x3625,
    BattlePetModifyName = 0x3627,
    BattlePetRequestJournal = 0x3623,
    BattlePetRequestJournalLock = 0x3622,
    BattlePetSetBattleSlot = 0x362B,
    BattlePetSetFlags = 0x362F,
    BattlePetSummon = 0x3628,
    BattlePetUpdateDisplayNotify = 0x31D7,
    BattlePetUpdateNotify = 0x31D6,
    BeginTrade = 0x3157,
    BinderActivate = 0x34B5,
    BlackMarketBidOnItem = 0x3531,
    BlackMarketOpen = 0x352F,
    BlackMarketRequestItems = 0x3530,
    BugReport = 0x3686,
    BusyTrade = 0x3158,
    BuyBackItem = 0x34A7,
    BuyBankSlot = 0x34B7,
    BuyItem = 0x34A6,
    BuyReagentBank = 0x34B8,
    BuyWowTokenConfirm = 0x36EB,
    BuyWowTokenStart = 0x36EA,
    CageBattlePet = 0x31E8,
    CalendarAddEvent = 0x367D,
    CalendarComplain = 0x3679,
    CalendarCopyEvent = 0x3678,
    CalendarEventInvite = 0x3672,
    CalendarEventModeratorStatus = 0x3676,
    CalendarEventRsvp = 0x3674,
    CalendarEventSignUp = 0x367B,
    CalendarEventStatus = 0x3675,
    CalendarGet = 0x366F,
    CalendarGetEvent = 0x3670,
    CalendarGetNumPending = 0x367A,
    CalendarGuildFilter = 0x3671,
    CalendarRemoveEvent = 0x3677,
    CalendarRemoveInvite = 0x3673,
    CalendarUpdateEvent = 0x367E,
    CancelAura = 0x31AC,
    CancelAutoRepeatSpell = 0x34EB,
    CancelCast = 0x3291,
    CancelChannelling = 0x325C,
    CancelGrowthAura = 0x3261,
    CancelMasterLootRoll = 0x3208,
    CancelModSpeedNoControlAuras = 0x31AB,
    CancelMountAura = 0x3272,
    CancelQueuedSpell = 0x317E,
    CancelTempEnchantment = 0x34F6,
    CancelTrade = 0x315C,
    CanDuel = 0x3662,
    CanRedeemWowTokenForBalance = 0x3706,
    CastSpell = 0x328E,
    ChallengeModeRequestLeaders = 0x308F,
    ChallengeModeRequestMapStats = 0x308E,
    ChangeBagSlotFlag = 0x3312,
    ChangeMonumentAppearance = 0x32F4,
    ChangeSubGroup = 0x364C,
    CharacterRenameRequest = 0x36BE,
    CharCustomize = 0x368E,
    CharDelete = 0x369B,
    CharRaceOrFactionChange = 0x3694,
    ChatAddonMessageChannel = 0x37D0,
    ChatAddonMessageGuild = 0x37D4,
    ChatAddonMessageInstanceChat = 0x37F3,
    ChatAddonMessageOfficer = 0x37D6,
    ChatAddonMessageParty = 0x37EF,
    ChatAddonMessageRaid = 0x37F1,
    ChatAddonMessageWhisper = 0x37D2,
    ChatChannelAnnouncements = 0x37E7,
    ChatChannelBan = 0x37E5,
    ChatChannelDeclineInvite = 0x37EA,
    ChatChannelDisplayList = 0x37DA,
    ChatChannelInvite = 0x37E3,
    ChatChannelKick = 0x37E4,
    ChatChannelList = 0x37D9,
    ChatChannelModerate = 0x37DE,
    ChatChannelModerator = 0x37DF,
    ChatChannelMute = 0x37E1,
    ChatChannelOwner = 0x37DD,
    ChatChannelPassword = 0x37DB,
    ChatChannelSetOwner = 0x37DC,
    ChatChannelSilenceAll = 0x37E8,
    ChatChannelUnban = 0x37E6,
    ChatChannelUnmoderator = 0x37E0,
    ChatChannelUnmute = 0x37E2,
    ChatChannelUnsilenceAll = 0x37E9,
    ChatJoinChannel = 0x37C8,
    ChatLeaveChannel = 0x37C9,
    ChatMessageAfk = 0x37D7,
    ChatMessageChannel = 0x37CF,
    ChatMessageDnd = 0x37D8,
    ChatMessageEmote = 0x37EC,
    ChatMessageGuild = 0x37D3,
    ChatMessageInstanceChat = 0x37F2,
    ChatMessageOfficer = 0x37D5,
    ChatMessageParty = 0x37EE,
    ChatMessageRaid = 0x37F0,
    ChatMessageRaidWarning = 0x37F4,
    ChatMessageSay = 0x37EB,
    ChatMessageWhisper = 0x37D1,
    ChatMessageYell = 0x37ED,
    ChatRegisterAddonPrefixes = 0x37CD,
    ChatReportFiltered = 0x37CC,
    ChatReportIgnored = 0x37CB,
    ChatUnregisterAllAddonPrefixes = 0x37CE,
    CheckRafEmailEnabled = 0x36CB,
    CheckWowTokenVeteranEligibility = 0x36E9,
    ChoiceResponse = 0x3293,
    ClearRaidMarker = 0x31A1,
    ClearTradeItem = 0x315E,
    ClientPortGraveyard = 0x352B,
    CloseInteraction = 0x3496,
    CollectionItemSetFavorite = 0x3632,
    CommentatorEnable = 0x35F0,
    CommentatorEnterInstance = 0x35F4,
    CommentatorExitInstance = 0x35F5,
    CommentatorGetMapInfo = 0x35F1,
    CommentatorGetPlayerCooldowns = 0x35F3,
    CommentatorGetPlayerInfo = 0x35F2,
    CommentatorStartWargame = 0x35EF,
    Complaint = 0x366C,
    CompleteCinematic = 0x3549,
    CompleteMovie = 0x34E1,
    ConfirmArtifactRespec = 0x31A6,
    ConfirmRespecWipe = 0x3202,
    ConnectToFailed = 0x35D4,
    ContributionContribute = 0x3558,
    ContributionGetState = 0x3559,
    ConversationLineStarted = 0x354A,
    ConvertConsumptionTime = 0x36F9,
    ConvertRaid = 0x364E,
    CreateCharacter = 0x3643,
    CreateShipment = 0x32E0,
    DbQueryBulk = 0x35E4,
    DeclineGuildInvites = 0x3522,
    DeclinePetition = 0x3538,
    DeleteEquipmentSet = 0x3510,
    DelFriend = 0x36D0,
    DelIgnore = 0x36D4,
    DepositReagentBank = 0x331B,
    DestroyItem = 0x3285,
    DfBootPlayerVote = 0x3615,
    DfGetJoinStatus = 0x3613,
    DfGetSystemInfo = 0x3612,
    DfJoin = 0x3608,
    DfLeave = 0x3611,
    DfProposalResponse = 0x3607,
    DfReadyCheckResponse = 0x3618,
    DfSetRoles = 0x3614,
    DfTeleport = 0x3616,
    DiscardedTimeSyncAcks = 0x3A3C,
    DismissCritter = 0x34FF,
    DoMasterLootRoll = 0x3207,
    DoReadyCheck = 0x3633,
    DuelResponse = 0x34E6,
    EjectPassenger = 0x3230,
    Emote = 0x3545,
    EnableEncryptionAck = 0x3767,
    EnableNagle = 0x376B,
    EnableTaxiNode = 0x34AC,
    EngineSurvey = 0x36E3,
    EnumCharacters = 0x35E8,
    EnumCharactersDeletedByClient = 0x36DD,
    FarSight = 0x34EC,
    GameObjReportUse = 0x34F3,
    GameObjUse = 0x34F2,
    GarrisonAssignFollowerToBuilding = 0x32CB,
    GarrisonCancelConstruction = 0x32BC,
    GarrisonCheckUpgradeable = 0x330E,
    GarrisonCompleteMission = 0x3301,
    GarrisonGenerateRecruits = 0x32CE,
    GarrisonGetBuildingLandmarks = 0x32DC,
    GarrisonGetMissionReward = 0x3334,
    GarrisonMissionBonusRoll = 0x3303,
    GarrisonPurchaseBuilding = 0x32B8,
    GarrisonRecruitFollower = 0x32D0,
    GarrisonRemoveFollower = 0x32F8,
    GarrisonRemoveFollowerFromBuilding = 0x32CC,
    GarrisonRenameFollower = 0x32CD,
    GarrisonRequestBlueprintAndSpecializationData = 0x32B7,
    GarrisonRequestClassSpecCategoryInfo = 0x32D5,
    GarrisonRequestLandingPageShipmentInfo = 0x32DF,
    GarrisonRequestShipmentInfo = 0x32DE,
    GarrisonResearchTalent = 0x32D1,
    GarrisonSetBuildingActive = 0x32B9,
    GarrisonSetFollowerFavorite = 0x32C9,
    GarrisonSetFollowerInactive = 0x32C5,
    GarrisonSetRecruitmentPreferences = 0x32CF,
    GarrisonStartMission = 0x3300,
    GarrisonSwapBuildings = 0x32BD,
    GenerateRandomCharacterName = 0x35E7,
    GetChallengeModeRewards = 0x3683,
    GetGarrisonInfo = 0x32B2,
    GetItemPurchaseData = 0x3533,
    GetMirrorImageData = 0x3289,
    GetPvpOptionsEnabled = 0x35EE,
    GetRemainingGameTime = 0x36EC,
    GetTrophyList = 0x32F1,
    GetUndeleteCharacterCooldownStatus = 0x36DF,
    GmTicketAcknowledgeSurvey = 0x3692,
    GmTicketGetCaseStatus = 0x3691,
    GmTicketGetSystemStatus = 0x3690,
    GossipSelectOption = 0x3497,
    GrantLevel = 0x34FB,
    GuildAddBattlenetFriend = 0x308D,
    GuildAddRank = 0x3064,
    GuildAssignMemberRank = 0x305F,
    GuildAutoDeclineInvitation = 0x3061,
    GuildBankActivate = 0x34B9,
    GuildBankBuyTab = 0x34C8,
    GuildBankDepositMoney = 0x34CA,
    GuildBankLogQuery = 0x3082,
    GuildBankQueryTab = 0x34C7,
    GuildBankRemainingWithdrawMoneyQuery = 0x3083,
    GuildBankSetTabText = 0x3086,
    GuildBankSwapItems = 0x34BA,
    GuildBankTextQuery = 0x3087,
    GuildBankUpdateTab = 0x34C9,
    GuildBankWithdrawMoney = 0x34CB,
    GuildChallengeUpdateRequest = 0x307B,
    GuildChangeNameRequest = 0x307E,
    GuildDeclineInvitation = 0x3060,
    GuildDelete = 0x3068,
    GuildDeleteRank = 0x3065,
    GuildDemoteMember = 0x305E,
    GuildEventLogQuery = 0x3085,
    GuildGetAchievementMembers = 0x3071,
    GuildGetRanks = 0x306D,
    GuildGetRoster = 0x3073,
    GuildInviteByName = 0x3606,
    GuildLeave = 0x3062,
    GuildMemberSendSorRequest = 0x308C,
    GuildNewsUpdateSticky = 0x306E,
    GuildOfficerRemoveMember = 0x3063,
    GuildPermissionsQuery = 0x3084,
    GuildPromoteMember = 0x305D,
    GuildQueryMembersForRecipe = 0x306B,
    GuildQueryMemberRecipes = 0x3069,
    GuildQueryNews = 0x306C,
    GuildQueryRecipes = 0x306A,
    GuildReplaceGuildMaster = 0x3088,
    GuildSetAchievementTracking = 0x306F,
    GuildSetFocusedAchievement = 0x3070,
    GuildSetGuildMaster = 0x36C5,
    GuildSetMemberNote = 0x3072,
    GuildSetRankPermissions = 0x3067,
    GuildShiftRank = 0x3066,
    GuildUpdateInfoText = 0x3075,
    GuildUpdateMotdText = 0x3074,
    HearthAndResurrect = 0x350C,
    HotfixRequest = 0x35E5,
    IgnoreTrade = 0x3159,
    InitiateRolePoll = 0x35DA,
    InitiateTrade = 0x3156,
    Inspect = 0x352D,
    InspectPvp = 0x36A1,
    InstanceLockResponse = 0x3511,
    ItemPurchaseRefund = 0x3534,
    ItemTextQuery = 0x330F,
    JoinPetBattleQueue = 0x31D4,
    JoinRatedBattleground = 0x3176,
    KeepAlive = 0x367F,
    KeyboundOverride = 0x3219,
    LearnPvpTalents = 0x3557,
    LearnTalents = 0x3556,
    LeaveGroup = 0x3649,
    LeavePetBattleQueue = 0x31D5,
    LfgListApplyToGroup = 0x360C,
    LfgListCancelApplication = 0x360D,
    LfgListDeclineApplicant = 0x360E,
    LfgListGetStatus = 0x360A,
    LfgListInviteApplicant = 0x360F,
    LfgListInviteResponse = 0x3610,
    LfgListJoin = 0x3345,
    LfgListLeave = 0x3609,
    LfgListSearch = 0x360B,
    LfgListUpdateRequest = 0x3346,
    LfGuildAddRecruit = 0x361B,
    LfGuildBrowse = 0x361D,
    LfGuildDeclineRecruit = 0x3078,
    LfGuildGetApplications = 0x3079,
    LfGuildGetGuildPost = 0x3076,
    LfGuildGetRecruits = 0x3077,
    LfGuildRemoveRecruit = 0x307A,
    LfGuildSetGuildPost = 0x361C,
    ListInventory = 0x34A4,
    LiveRegionAccountRestore = 0x36B9,
    LiveRegionCharacterCopy = 0x36B8,
    LiveRegionGetAccountCharacterList = 0x36B7,
    LoadingScreenNotify = 0x35F8,
    LoadSelectedTrophy = 0x32F2,
    LogoutCancel = 0x34DC,
    LogoutInstant = 0x34DD,
    LogoutRequest = 0x34DB,
    LogDisconnect = 0x3769,
    LogStreamingError = 0x376D,
    LootItem = 0x3205,
    LootMoney = 0x3204,
    LootRelease = 0x3209,
    LootRoll = 0x320A,
    LootUnit = 0x3203,
    LowLevelRAID1 = 0x369F,
    LowLevelRAID2 = 0x3518,
    MailCreateTextItem = 0x353F,
    MailDelete = 0x321B,
    MailGetList = 0x353A,
    MailMarkAsRead = 0x353E,
    MailReturnToSender = 0x3655,
    MailTakeItem = 0x353C,
    MailTakeMoney = 0x353B,
    MasterLootItem = 0x3206,
    MinimapPing = 0x364B,
    MissileTrajectoryCollision = 0x3189,
    MountClearFanfare = 0x312D,
    MountSetFavorite = 0x3631,
    MountSpecialAnim = 0x3273,
    MoveApplyMovementForceAck = 0x3A12,
    MoveChangeTransport = 0x3A2C,
    MoveChangeVehicleSeats = 0x3A31,
    MoveDismissVehicle = 0x3A30,
    MoveDoubleJump = 0x39EB,
    MoveEnableDoubleJumpAck = 0x3A1B,
    MoveEnableSwimToFlyTransAck = 0x3A21,
    MoveFallLand = 0x39F9,
    MoveFallReset = 0x3A16,
    MoveFeatherFallAck = 0x3A19,
    MoveForceFlightBackSpeedChangeAck = 0x3A2B,
    MoveForceFlightSpeedChangeAck = 0x3A2A,
    MoveForcePitchRateChangeAck = 0x3A2F,
    MoveForceRootAck = 0x3A0B,
    MoveForceRunBackSpeedChangeAck = 0x3A09,
    MoveForceRunSpeedChangeAck = 0x3A08,
    MoveForceSwimBackSpeedChangeAck = 0x3A1F,
    MoveForceSwimSpeedChangeAck = 0x3A0A,
    MoveForceTurnRateChangeAck = 0x3A20,
    MoveForceUnrootAck = 0x3A0C,
    MoveForceWalkSpeedChangeAck = 0x3A1E,
    MoveGravityDisableAck = 0x3A32,
    MoveGravityEnableAck = 0x3A33,
    MoveHeartbeat = 0x3A0D,
    MoveHoverAck = 0x3A10,
    MoveJump = 0x39EA,
    MoveKnockBackAck = 0x3A0F,
    MoveRemoveMovementForces = 0x3A14,
    MoveRemoveMovementForceAck = 0x3A13,
    MoveSetCanFlyAck = 0x3A24,
    MoveSetCanTurnWhileFallingAck = 0x3A22,
    MoveSetCollisionHeightAck = 0x3A36,
    MoveSetFacing = 0x3A06,
    MoveSetFly = 0x3A25,
    MoveSetIgnoreMovementForcesAck = 0x3A23,
    MoveSetPitch = 0x3A07,
    MoveSetRunMode = 0x39F2,
    MoveSetVehicleRecIdAck = 0x3A11,
    MoveSetWalkMode = 0x39F3,
    MoveSplineDone = 0x3A15,
    MoveStartAscend = 0x3A26,
    MoveStartBackward = 0x39E5,
    MoveStartDescend = 0x3A2D,
    MoveStartForward = 0x39E4,
    MoveStartPitchDown = 0x39F0,
    MoveStartPitchUp = 0x39EF,
    MoveStartStrafeLeft = 0x39E7,
    MoveStartStrafeRight = 0x39E8,
    MoveStartSwim = 0x39FA,
    MoveStartTurnLeft = 0x39EC,
    MoveStartTurnRight = 0x39ED,
    MoveStop = 0x39E6,
    MoveStopAscend = 0x3A27,
    MoveStopPitch = 0x39F1,
    MoveStopStrafe = 0x39E9,
    MoveStopSwim = 0x39FB,
    MoveStopTurn = 0x39EE,
    MoveTeleportAck = 0x39F8,
    MoveTimeSkipped = 0x3A18,
    MoveToggleCollisionCheat = 0x3A05,
    MoveWaterWalkAck = 0x3A1A,
    NeutralPlayerSelectFaction = 0x31CA,
    NextCinematicCamera = 0x3548,
    ObjectUpdateFailed = 0x317F,
    ObjectUpdateRescued = 0x3180,
    OfferPetition = 0x36AF,
    OpeningCinematic = 0x3547,
    OpenItem = 0x3310,
    OpenMissionNpc = 0x32D7,
    OpenShipmentNpc = 0x32DD,
    OpenTradeskillNpc = 0x32E8,
    OptOutOfLoot = 0x34FA,
    PartyInvite = 0x3602,
    PartyInviteResponse = 0x3603,
    PartyUninvite = 0x3647,
    PetitionBuy = 0x34CD,
    PetitionRenameGuild = 0x36C6,
    PetitionShowList = 0x34CC,
    PetitionShowSignatures = 0x34CE,
    PetAbandon = 0x3490,
    PetAction = 0x348E,
    PetBattleFinalNotify = 0x31D9,
    PetBattleInput = 0x3640,
    PetBattleQueueProposeMatchResult = 0x321A,
    PetBattleQuitNotify = 0x31D8,
    PetBattleReplaceFrontPet = 0x3641,
    PetBattleRequestPvp = 0x31D2,
    PetBattleRequestUpdate = 0x31D3,
    PetBattleRequestWild = 0x31D0,
    PetBattleScriptErrorNotify = 0x31DA,
    PetBattleWildLocationFail = 0x31D1,
    PetCancelAura = 0x3491,
    PetCastSpell = 0x328D,
    PetRename = 0x3685,
    PetSetAction = 0x348D,
    PetSpellAutocast = 0x3492,
    PetStopAttack = 0x348F,
    Ping = 0x3768,
    PlayerLogin = 0x35EA,
    ProtocolMismatch = 0x376E,
    PushQuestToParty = 0x34A2,
    PvpLogData = 0x317A,
    PvpPrestigeRankUp = 0x3332,
    QueryBattlePetName = 0x3268,
    QueryCorpseLocationFromClient = 0x3660,
    QueryCorpseTransport = 0x3661,
    QueryCountdownTimer = 0x31A4,
    QueryCreature = 0x3262,
    QueryGameObject = 0x3263,
    QueryGarrisonCreatureName = 0x3269,
    QueryGuildInfo = 0x368D,
    QueryInspectAchievements = 0x3506,
    QueryNextMailTime = 0x353D,
    QueryNpcText = 0x3264,
    QueryPageText = 0x3266,
    QueryPetition = 0x326A,
    QueryPetName = 0x3267,
    QueryPlayerName = 0x368B,
    QueryQuestCompletionNpcs = 0x3173,
    QueryQuestInfo = 0x3265,
    QueryQuestRewards = 0x3336,
    QueryRealmName = 0x368C,
    QueryScenarioPoi = 0x3656,
    QueryTime = 0x34DA,
    QueryVoidStorage = 0x319D,
    QuestConfirmAccept = 0x34A1,
    QuestGiverAcceptQuest = 0x349B,
    QuestGiverChooseReward = 0x349D,
    QuestGiverCompleteQuest = 0x349C,
    QuestGiverHello = 0x3499,
    QuestGiverQueryQuest = 0x349A,
    QuestGiverRequestReward = 0x349E,
    QuestGiverStatusMultipleQuery = 0x34A0,
    QuestGiverStatusQuery = 0x349F,
    QuestLogRemoveQuest = 0x3532,
    QuestPoiQuery = 0x36B0,
    QuestPushResult = 0x34A3,
    QueuedMessagesEnd = 0x376C,
    QuickJoinAutoAcceptRequests = 0x3705,
    QuickJoinRequestInvite = 0x3704,
    QuickJoinRespondToInvite = 0x3703,
    QuickJoinSignalToastDisplayed = 0x3702,
    RaidOrBattlegroundEngineSurvey = 0x36E4,
    RandomRoll = 0x3654,
    ReadyCheckResponse = 0x3634,
    ReadItem = 0x3311,
    ReclaimCorpse = 0x34DF,
    RecruitAFriend = 0x36CC,
    RedeemWowTokenConfirm = 0x36EE,
    RedeemWowTokenStart = 0x36ED,
    RemoveNewItem = 0x3339,
    ReorderCharacters = 0x35E9,
    RepairItem = 0x34F0,
    ReplaceTrophy = 0x32F3,
    RepopRequest = 0x352A,
    ReportClientVariables = 0x36FF,
    ReportEnabledAddons = 0x36FE,
    ReportKeybindingExecutionCounts = 0x3700,
    ReportPvpPlayerAfk = 0x34F8,
    RequestAccountData = 0x3695,
    RequestAreaPoiUpdate = 0x3338,
    RequestBattlefieldStatus = 0x35DC,
    RequestCategoryCooldowns = 0x317C,
    RequestCemeteryList = 0x3174,
    RequestConquestFormulaConstants = 0x32A4,
    RequestConsumptionConversionInfo = 0x36F8,
    RequestCrowdControlSpell = 0x352E,
    RequestForcedReactions = 0x31FE,
    RequestGuildPartyState = 0x31A3,
    RequestGuildRewardsList = 0x31A2,
    RequestHonorStats = 0x3179,
    RequestLfgListBlacklist = 0x3295,
    RequestPartyJoinUpdates = 0x35F7,
    RequestPartyMemberStats = 0x3653,
    RequestPetInfo = 0x3493,
    RequestPlayedTime = 0x326D,
    RequestPvpBrawlInfo = 0x3191,
    RequestPvpRewards = 0x3190,
    RequestRaidInfo = 0x36C7,
    RequestRatedBattlefieldInfo = 0x35E3,
    RequestResearchHistory = 0x3167,
    RequestStabledPets = 0x3494,
    RequestVehicleExit = 0x322B,
    RequestVehicleNextSeat = 0x322D,
    RequestVehiclePrevSeat = 0x322C,
    RequestVehicleSwitchSeat = 0x322E,
    RequestWorldQuestUpdate = 0x3337,
    RequestWowTokenMarketPrice = 0x36E6,
    ResetChallengeMode = 0x31FB,
    ResetChallengeModeCheat = 0x31FC,
    ResetInstances = 0x3668,
    ResurrectResponse = 0x3684,
    RevertMonumentAppearance = 0x32F5,
    RideVehicleInteract = 0x322F,
    SaveCufProfiles = 0x318A,
    SaveEquipmentSet = 0x350F,
    SaveGuildEmblem = 0x3299,
    ScenePlaybackCanceled = 0x3216,
    ScenePlaybackComplete = 0x3215,
    SceneTriggerEvent = 0x3217,
    SelfRes = 0x3535,
    SellItem = 0x34A5,
    SellWowTokenConfirm = 0x36E8,
    SellWowTokenStart = 0x36E7,
    SendContactList = 0x36CE,
    SendMail = 0x35FA,
    SendSorRequestViaAddress = 0x3620,
    SendTextEmote = 0x348A,
    SetAchievementsHidden = 0x321C,
    SetActionBarToggles = 0x3536,
    SetActionButton = 0x3635,
    SetActiveMover = 0x3A37,
    SetAdvancedCombatLogging = 0x32A5,
    SetAssistantLeader = 0x364F,
    SetBackpackAutosortDisabled = 0x3314,
    SetBankAutosortDisabled = 0x3315,
    SetBankBagSlotFlag = 0x3313,
    SetContactNotes = 0x36D1,
    SetCurrencyFlags = 0x3169,
    SetDifficultyId = 0x3218,
    SetDungeonDifficulty = 0x3682,
    SetEveryoneIsAssistant = 0x3617,
    SetFactionAtWar = 0x34E2,
    SetFactionInactive = 0x34E4,
    SetFactionNotAtWar = 0x34E3,
    SetInsertItemsLeftToRight = 0x3317,
    SetLfgBonusFactionId = 0x3294,
    SetLootMethod = 0x3648,
    SetLootSpecialization = 0x3543,
    SetPartyAssignment = 0x3651,
    SetPartyLeader = 0x364A,
    SetPetSlot = 0x3168,
    SetPlayerDeclinedNames = 0x368A,
    SetPreferredCemetery = 0x3175,
    SetPvp = 0x329D,
    SetRaidDifficulty = 0x36DB,
    SetRole = 0x35D9,
    SetSavedInstanceExtend = 0x3688,
    SetSelection = 0x352C,
    SetSheathed = 0x348B,
    SetSortBagsRightToLeft = 0x3316,
    SetTaxiBenchmarkMode = 0x34F7,
    SetTitle = 0x3271,
    SetTradeCurrency = 0x3160,
    SetTradeGold = 0x315F,
    SetTradeItem = 0x315D,
    SetUsingPartyGarrison = 0x32D9,
    SetWatchedFaction = 0x34E5,
    ShowTradeSkill = 0x36BF,
    SignPetition = 0x3537,
    SilencePartyTalker = 0x3652,
    SocketGems = 0x34EF,
    SortBags = 0x3318,
    SortBankBags = 0x3319,
    SortReagentBankBags = 0x331A,
    SpellClick = 0x3498,
    SpiritHealerActivate = 0x34B2,
    SplitItem = 0x399E,
    StandStateChange = 0x3188,
    StartChallengeMode = 0x354E,
    StartSpectatorWarGame = 0x35DF,
    StartWarGame = 0x35DE,
    SummonResponse = 0x366A,
    SupportTicketSubmitBug = 0x3645,
    SupportTicketSubmitComplaint = 0x3644,
    SupportTicketSubmitSuggestion = 0x3646,
    SurrenderArena = 0x3172,
    SuspendCommsAck = 0x3764,
    SuspendTokenResponse = 0x376A,
    SwapInvItem = 0x399D,
    SwapItem = 0x399C,
    SwapSubGroups = 0x364D,
    SwapVoidItem = 0x319F,
    TabardVendorActivate = 0x329A,
    TalkToGossip = 0x3495,
    TaxiNodeStatusQuery = 0x34AB,
    TaxiQueryAvailableNodes = 0x34AD,
    TaxiRequestEarlyLanding = 0x34AF,
    TimeAdjustmentResponse = 0x3A3B,
    TimeSyncResponse = 0x3A38,
    TimeSyncResponseDropped = 0x3A3A,
    TimeSyncResponseFailed = 0x3A39,
    ToggleDifficulty = 0x3657,
    TogglePvp = 0x329C,
    TotemDestroyed = 0x34FE,
    TradeSkillSetFavorite = 0x3335,
    TrainerBuySpell = 0x34B1,
    TrainerList = 0x34B0,
    TransmogrifyItems = 0x3192,
    TurnInPetition = 0x3539,
    Tutorial = 0x36DC,
    TwitterCheckStatus = 0x312A,
    TwitterConnect = 0x3127,
    TwitterDisconnect = 0x312B,
    TwitterPost = 0x331C,
    UiTimeRequest = 0x369A,
    UnacceptTrade = 0x315B,
    UndeleteCharacter = 0x36DE,
    UnlearnSkill = 0x34E9,
    UnlearnSpecialization = 0x31A0,
    UnlockVoidStorage = 0x319C,
    UpdateAccountData = 0x3696,
    UpdateAreaTriggerVisual = 0x3290,
    UpdateClientSettings = 0x3664,
    UpdateMissileTrajectory = 0x3A3E,
    UpdateRaidTarget = 0x3650,
    UpdateSpellVisual = 0x328F,
    UpdateVasPurchaseStates = 0x36F4,
    UpdateWowTokenAuctionableList = 0x36EF,
    UpdateWowTokenCount = 0x36E5,
    UpgradeGarrison = 0x32AD,
    UpgradeItem = 0x321D,
    UsedFollow = 0x3185,
    UseCritterItem = 0x3235,
    UseEquipmentSet = 0x3995,
    UseItem = 0x328A,
    UseToy = 0x328C,
    ViolenceLevel = 0x3183,
    VoidStorageTransfer = 0x319E,
    WardenData = 0x35EC,
    Who = 0x3681,
    WhoIs = 0x3680,
    WorldPortResponse = 0x35F9,
    WrapItem = 0x3994,
    // CMSG_BF_MGR_ENTRY_INVITE_RESPONSE = 0xBADD,
    // CMSG_BF_MGR_QUEUE_INVITE_RESPONSE = 0xBADD,
    // CMSG_BF_MGR_QUEUE_EXIT_REQUEST = 0xBADD,
}

/// Shorter alias of OpcodeClient
pub type CMSG = OpcodeClient;

pub struct ClientOpcodeInfo {
    pub status:     SessionStatus,
    pub processing: PacketProcessing,
}

macro_rules! define_client_handler {
    ( $this:expr; $( $cmsg:ident, $status:ident, $processing:ident );+ $(;)? ) => {
        match $this {
            $(
                CMSG::$cmsg => ClientOpcodeInfo { status: SessionStatus::$status, processing: PacketProcessing::$processing },
            )+
        }
    };
}

impl OpcodeClient {
    pub const fn info(&self) -> ClientOpcodeInfo {
        define_client_handler!(
        *self;
        AcceptGuildInvite, LoggedIn, Threadunsafe;
        AcceptLevelGrant, LoggedIn, Threadunsafe; //, &WorldSession::HandleAcceptGrantLevel);
        AcceptTrade, LoggedIn, Threadunsafe; //, &WorldSession::HandleAcceptTradeOpcode);
        AcceptWargameInvite, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ActivateTaxi, LoggedIn, Threadsafe; //,   &WorldSession::HandleActivateTaxiOpcode);
        AddonList, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        AddBattlenetFriend, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        AddFriend, LoggedIn, Threadunsafe; //, &WorldSession::HandleAddFriendOpcode);
        AddIgnore, LoggedIn, Threadunsafe; //, &WorldSession::HandleAddIgnoreOpcode);
        AddToy, LoggedIn, Threadunsafe; //, &WorldSession::HandleAddToy);
        AdventureJournalOpenQuest, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        AdventureJournalStartQuest, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        AlterAppearance, LoggedIn, Threadunsafe; //, &WorldSession::HandleAlterAppearance);
        AreaSpiritHealerQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleAreaSpiritHealerQueryOpcode);
        AreaSpiritHealerQueue, LoggedIn, Threadunsafe; //, &WorldSession::HandleAreaSpiritHealerQueueOpcode);
        AreaTrigger, LoggedIn, Threadunsafe; //, &WorldSession::HandleAreaTriggerOpcode);
        ArtifactAddPower, LoggedIn, Threadunsafe; //, &WorldSession::HandleArtifactAddPower);
        ArtifactAddRelicTalent, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ArtifactAttunePreviewRelic, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ArtifactAttuneSocketedRelic, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ArtifactSetAppearance, LoggedIn, Threadunsafe; //, &WorldSession::HandleArtifactSetAppearance);
        AttackStop, LoggedIn, Inplace; //,      &WorldSession::HandleAttackStopOpcode);
        AttackSwing, LoggedIn, Inplace; //,      &WorldSession::HandleAttackSwingOpcode);
        AuctionHelloRequest, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionHelloOpcode);
        AuctionListBidderItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionListBidderItems);
        AuctionListItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionListItems);
        AuctionListOwnerItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionListOwnerItems);
        AuctionListPendingSales, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionListPendingSales);
        AuctionPlaceBid, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionPlaceBid);
        AuctionRemoveItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionRemoveItem);
        AuctionReplicateItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleReplicateItems);
        AuctionSellItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAuctionSellItem);
        AuthContinuedSession, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        AuthSession, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        AutobankItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAutoBankItemOpcode);
        AutobankReagent, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        AutostoreBankItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAutoStoreBankItemOpcode);
        AutostoreBankReagent, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        AutoEquipItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAutoEquipItemOpcode);
        AutoEquipItemSlot, LoggedIn, Threadunsafe; //, &WorldSession::HandleAutoEquipItemSlotOpcode);
        AutoStoreBagItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAutoStoreBagItemOpcode);
        BankerActivate, LoggedIn, Threadunsafe; //, &WorldSession::HandleBankerActivateOpcode);
        BattlefieldLeave, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlefieldLeaveOpcode);
        BattlefieldList, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlefieldListOpcode);
        BattlefieldPort, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattleFieldPortOpcode);
        BattlemasterHello, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlemasterHelloOpcode);
        BattlemasterJoin, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlemasterJoinOpcode);
        BattlemasterJoinArena, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlemasterJoinArena);
        BattlemasterJoinBrawl, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlemasterJoinSkirmish, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BattlenetChallengeResponse, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BattlenetRequest, Authed, Threadunsafe; //, &WorldSession::HandleBattlenetRequest);
        BattlenetRequestRealmListTicket, Authed, Threadunsafe; //, &WorldSession::HandleBattlenetRequestRealmListTicket);
        BattlePayAckFailedResponse, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayConfirmPurchaseResponse, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayDistributionAssignToTarget, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayGetProductList, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayGetPurchaseList, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayQueryClassTrialBoostResult, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayRequestCharacterBoostUnrevoke, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayRequestCurrentVasTransferQueues, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayRequestVasCharacterQueueTime, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayRequestPriceInfo, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayStartPurchase, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayStartVasPurchase, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayTrialBoostCharacter, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePayValidateBnetVasTransfer, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePetClearFanfare, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        BattlePetDeletePet, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlePetDeletePet);
        BattlePetDeletePetCheat, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BattlePetModifyName, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlePetModifyName);
        BattlePetRequestJournal, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlePetRequestJournal);
        BattlePetRequestJournalLock, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BattlePetSetBattleSlot, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlePetSetBattleSlot);
        BattlePetSetFlags, LoggedIn, Threadunsafe; //, &WorldSession::HandleBattlePetSetFlags);
        BattlePetSummon, LoggedIn, Inplace; //,      &WorldSession::HandleBattlePetSummon);
        BattlePetUpdateDisplayNotify, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BattlePetUpdateNotify, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BeginTrade, LoggedIn, Threadunsafe; //, &WorldSession::HandleBeginTradeOpcode);
        //BfMgrEntryInviteResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleBfEntryInviteResponse);
        //BfMgrQueueExitRequest, LoggedIn, Threadunsafe; //, &WorldSession::HandleBfQueueExitRequest);
        //BfMgrQueueInviteResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleBfQueueInviteResponse);
        BinderActivate, LoggedIn, Threadunsafe; //, &WorldSession::HandleBinderActivateOpcode);
        BlackMarketBidOnItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleBlackMarketBidOnItem);
        BlackMarketOpen, LoggedIn, Threadunsafe; //, &WorldSession::HandleBlackMarketOpen);
        BlackMarketRequestItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleBlackMarketRequestItems);
        BugReport, LoggedIn, Threadunsafe; //, &WorldSession::HandleBugReportOpcode);
        BusyTrade, LoggedIn, Threadunsafe; //, &WorldSession::HandleBusyTradeOpcode);
        BuyBackItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleBuybackItem);
        BuyBankSlot, LoggedIn, Threadunsafe; //, &WorldSession::HandleBuyBankSlotOpcode);
        BuyItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleBuyItemOpcode);
        BuyReagentBank, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BuyWowTokenConfirm, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        BuyWowTokenStart, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CageBattlePet, LoggedIn, Threadunsafe; //, &WorldSession::HandleCageBattlePet);
        CalendarAddEvent, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarAddEvent);
        CalendarComplain, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarComplain);
        CalendarCopyEvent, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarCopyEvent);
        CalendarEventInvite, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarEventInvite);
        CalendarEventModeratorStatus, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarEventModeratorStatus);
        CalendarEventRsvp, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarEventRsvp);
        CalendarEventSignUp, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarEventSignup);
        CalendarEventStatus, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarEventStatus);
        CalendarGet, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarGetCalendar);
        CalendarGetEvent, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarGetEvent);
        CalendarGetNumPending, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarGetNumPending);
        CalendarGuildFilter, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarGuildFilter);
        CalendarRemoveEvent, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarRemoveEvent);
        CalendarRemoveInvite, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarEventRemoveInvite);
        CalendarUpdateEvent, LoggedIn, Threadunsafe; //, &WorldSession::HandleCalendarUpdateEvent);
        CancelAura, LoggedIn, Threadunsafe; //, &WorldSession::HandleCancelAuraOpcode);
        CancelAutoRepeatSpell, LoggedIn, Threadunsafe; //, &WorldSession::HandleCancelAutoRepeatSpellOpcode);
        CancelCast, LoggedIn, Threadsafe; //,   &WorldSession::HandleCancelCastOpcode);
        CancelChannelling, LoggedIn, Threadunsafe; //, &WorldSession::HandleCancelChanneling);
        CancelGrowthAura, LoggedIn, Threadunsafe; //, &WorldSession::HandleCancelGrowthAuraOpcode);
        CancelMasterLootRoll, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CancelModSpeedNoControlAuras, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CancelMountAura, LoggedIn, Threadunsafe; //, &WorldSession::HandleCancelMountAuraOpcode);
        CancelQueuedSpell, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CancelTempEnchantment, LoggedIn, Threadunsafe; //, &WorldSession::HandleCancelTempEnchantmentOpcode);
        CancelTrade, LoggedInOrRecentlyLogout, Threadunsafe; //, &WorldSession::HandleCancelTradeOpcode);
        CanDuel, LoggedIn, Threadunsafe; //, &WorldSession::HandleCanDuel);
        CanRedeemWowTokenForBalance, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        CastSpell, LoggedIn, Threadsafe; //,   &WorldSession::HandleCastSpellOpcode);
        ChallengeModeRequestLeaders, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChallengeModeRequestMapStats, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChangeBagSlotFlag, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChangeMonumentAppearance, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChangeSubGroup, LoggedIn, Threadunsafe; //, &WorldSession::HandleChangeSubGroupOpcode);
        CharacterRenameRequest, Authed, Threadunsafe; //, &WorldSession::HandleCharRenameOpcode);
        CharCustomize, Authed, Threadunsafe; //, &WorldSession::HandleCharCustomizeOpcode);
        CharDelete, Authed, Threadunsafe; //, &WorldSession::HandleCharDeleteOpcode);
        CharRaceOrFactionChange, Authed, Threadunsafe; //, &WorldSession::HandleCharRaceOrFactionChangeOpcode);
        ChatAddonMessageChannel, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageChannelOpcode);
        ChatAddonMessageGuild, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageOpcode);
        ChatAddonMessageInstanceChat, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageOpcode);
        ChatAddonMessageOfficer, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageOpcode);
        ChatAddonMessageParty, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageOpcode);
        ChatAddonMessageRaid, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageOpcode);
        ChatAddonMessageWhisper, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatAddonMessageWhisperOpcode);
        ChatChannelAnnouncements, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelCommand);
        ChatChannelBan, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelDeclineInvite, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelCommand);
        ChatChannelDisplayList, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelCommand);
        ChatChannelInvite, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelKick, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelList, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelCommand);
        ChatChannelModerate, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChatChannelModerator, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelMute, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelOwner, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelCommand);
        ChatChannelPassword, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPassword);
        ChatChannelSetOwner, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelSilenceAll, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelUnban, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelUnmoderator, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelUnmute, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatChannelUnsilenceAll, LoggedIn, Threadunsafe; //, &WorldSession::HandleChannelPlayerCommand);
        ChatJoinChannel, LoggedIn, Threadunsafe; //, &WorldSession::HandleJoinChannel);
        ChatLeaveChannel, LoggedIn, Threadunsafe; //, &WorldSession::HandleLeaveChannel);
        ChatMessageAfk, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageAFKOpcode);
        ChatMessageChannel, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageChannelOpcode);
        ChatMessageDnd, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageDNDOpcode);
        ChatMessageEmote, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageEmoteOpcode);
        ChatMessageGuild, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageInstanceChat, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageOfficer, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageParty, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageRaid, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageRaidWarning, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageSay, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatMessageWhisper, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageWhisperOpcode);
        ChatMessageYell, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatMessageOpcode);
        ChatRegisterAddonPrefixes, LoggedIn, Threadunsafe; //, &WorldSession::HandleAddonRegisteredPrefixesOpcode);
        ChatReportFiltered, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChatReportIgnored, LoggedIn, Threadunsafe; //, &WorldSession::HandleChatIgnoredOpcode);
        ChatUnregisterAllAddonPrefixes, LoggedIn, Threadunsafe; //, &WorldSession::HandleUnregisterAllAddonPrefixesOpcode);
        CheckRafEmailEnabled, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CheckWowTokenVeteranEligibility, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ChoiceResponse, LoggedIn, Inplace; //,      &WorldSession::HandlePlayerChoiceResponse);
        ClearRaidMarker, LoggedIn, Threadunsafe; //, &WorldSession::HandleClearRaidMarker);
        ClearTradeItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleClearTradeItemOpcode);
        ClientPortGraveyard, LoggedIn, Threadunsafe; //, &WorldSession::HandlePortGraveyard);
        CloseInteraction, LoggedIn, Threadunsafe; //, &WorldSession::HandleCloseInteraction);
        CollectionItemSetFavorite, LoggedIn, Threadunsafe; //, &WorldSession::HandleCollectionItemSetFavorite);
        CommentatorEnable, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CommentatorEnterInstance, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CommentatorExitInstance, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CommentatorGetMapInfo, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CommentatorGetPlayerCooldowns, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CommentatorGetPlayerInfo, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        CommentatorStartWargame, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        Complaint, LoggedIn, Threadunsafe; //, &WorldSession::HandleComplaint);
        CompleteCinematic, LoggedIn, Threadunsafe; //, &WorldSession::HandleCompleteCinematic);
        CompleteMovie, LoggedIn, Threadunsafe; //, &WorldSession::HandleCompleteMovie);
        ConfirmArtifactRespec, LoggedIn, Threadunsafe; //, &WorldSession::HandleConfirmArtifactRespec);
        ConfirmRespecWipe, LoggedIn, Threadunsafe; //, &WorldSession::HandleConfirmRespecWipeOpcode);
        ConnectToFailed, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        ContributionContribute, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ContributionGetState, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ConversationLineStarted, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ConvertConsumptionTime, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ConvertRaid, LoggedIn, Threadunsafe; //, &WorldSession::HandleConvertRaidOpcode);
        CreateCharacter, Authed, Threadunsafe; //, &WorldSession::HandleCharCreateOpcode);
        CreateShipment, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        DbQueryBulk, Authed, Inplace; //,      &WorldSession::HandleDBQueryBulk);
        DeclineGuildInvites, LoggedIn, Threadunsafe; //, &WorldSession::HandleDeclineGuildInvites);
        DeclinePetition, LoggedIn, Threadunsafe; //, &WorldSession::HandleDeclinePetition);
        DeleteEquipmentSet, LoggedIn, Threadunsafe; //, &WorldSession::HandleDeleteEquipmentSet);
        DelFriend, LoggedIn, Threadunsafe; //, &WorldSession::HandleDelFriendOpcode);
        DelIgnore, LoggedIn, Threadunsafe; //, &WorldSession::HandleDelIgnoreOpcode);
        DepositReagentBank, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        DestroyItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleDestroyItemOpcode);
        DfBootPlayerVote, LoggedIn, Threadunsafe; //, &WorldSession::HandleLfgSetBootVoteOpcode);
        DfGetJoinStatus, Unhandled, Threadsafe; //,   &WorldSession::HandleDFGetJoinStatus);
        DfGetSystemInfo, LoggedIn, Threadsafe; //,   &WorldSession::HandleDFGetSystemInfo);
        DfJoin, LoggedIn, Threadunsafe; //, &WorldSession::HandleLfgJoinOpcode);
        DfLeave, LoggedIn, Threadunsafe; //, &WorldSession::HandleLfgLeaveOpcode);
        DfProposalResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleLfgProposalResultOpcode);
        DfReadyCheckResponse, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        DfSetRoles, LoggedIn, Threadunsafe; //, &WorldSession::HandleLfgSetRolesOpcode);
        DfTeleport, LoggedIn, Threadunsafe; //, &WorldSession::HandleLfgTeleportOpcode);
        DiscardedTimeSyncAcks, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        DismissCritter, LoggedIn, Threadunsafe; //, &WorldSession::HandleDismissCritter);
        DoMasterLootRoll, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        DoReadyCheck, LoggedIn, Threadunsafe; //, &WorldSession::HandleDoReadyCheckOpcode);
        DuelResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleDuelResponseOpcode);
        EjectPassenger, LoggedIn, Threadunsafe; //, &WorldSession::HandleEjectPassenger);
        Emote, LoggedIn, Threadunsafe; //, &WorldSession::HandleEmoteOpcode);
        EnableEncryptionAck, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        EnableNagle, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        EnableTaxiNode, LoggedIn, Threadsafe; //,   &WorldSession::HandleEnableTaxiNodeOpcode);
        EngineSurvey, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        EnumCharacters, Authed, Threadunsafe; //, &WorldSession::HandleCharEnumOpcode);
        EnumCharactersDeletedByClient, Authed, Threadunsafe; //, &WorldSession::HandleCharUndeleteEnumOpcode);
        FarSight, LoggedIn, Threadunsafe; //, &WorldSession::HandleFarSightOpcode);
        GameObjReportUse, LoggedIn, Threadunsafe; //, &WorldSession::HandleGameobjectReportUse);
        GameObjUse, LoggedIn, Threadunsafe; //, &WorldSession::HandleGameObjectUseOpcode);
        GarrisonAssignFollowerToBuilding, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonCancelConstruction, LoggedIn, Threadunsafe; //, &WorldSession::HandleGarrisonCancelConstruction);
        GarrisonCheckUpgradeable, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonCompleteMission, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonGenerateRecruits, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonGetBuildingLandmarks, LoggedIn, Threadunsafe; //, &WorldSession::HandleGarrisonGetBuildingLandmarks);
        GarrisonGetMissionReward, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        GarrisonMissionBonusRoll, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonPurchaseBuilding, LoggedIn, Threadunsafe; //, &WorldSession::HandleGarrisonPurchaseBuilding);
        GarrisonRecruitFollower, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonRemoveFollower, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonRemoveFollowerFromBuilding, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonRenameFollower, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonRequestBlueprintAndSpecializationData, LoggedIn, Threadunsafe; //, &WorldSession::HandleGarrisonRequestBlueprintAndSpecializationData);
        GarrisonRequestClassSpecCategoryInfo, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        GarrisonRequestLandingPageShipmentInfo, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonRequestShipmentInfo, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonResearchTalent, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        GarrisonSetBuildingActive, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonSetFollowerFavorite, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonSetFollowerInactive, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonSetRecruitmentPreferences, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonStartMission, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GarrisonSwapBuildings, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GenerateRandomCharacterName, Authed, Threadunsafe; //, &WorldSession::HandleRandomizeCharNameOpcode);
        GetChallengeModeRewards, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GetGarrisonInfo, LoggedIn, Threadunsafe; //, &WorldSession::HandleGetGarrisonInfo);
        GetItemPurchaseData, LoggedIn, Threadunsafe; //, &WorldSession::HandleGetItemPurchaseData);
        GetMirrorImageData, LoggedIn, Threadunsafe; //, &WorldSession::HandleMirrorImageDataRequest);
        GetPvpOptionsEnabled, LoggedIn, Inplace; //,      &WorldSession::HandleGetPVPOptionsEnabled);
        GetRemainingGameTime, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GetTrophyList, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GetUndeleteCharacterCooldownStatus, Authed, Threadunsafe; //, &WorldSession::HandleGetUndeleteCooldownStatus);
        GmTicketAcknowledgeSurvey, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GmTicketGetCaseStatus, LoggedIn, Inplace; //,      &WorldSession::HandleGMTicketGetCaseStatusOpcode);
        GmTicketGetSystemStatus, LoggedIn, Inplace; //,      &WorldSession::HandleGMTicketSystemStatusOpcode);
        GossipSelectOption, LoggedIn, Threadunsafe; //, &WorldSession::HandleGossipSelectOptionOpcode);
        GrantLevel, LoggedIn, Threadunsafe; //, &WorldSession::HandleGrantLevel);
        GuildAddBattlenetFriend, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildAddRank, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildAddRank);
        GuildAssignMemberRank, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildAssignRank);
        GuildAutoDeclineInvitation, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildBankActivate, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankActivate);
        GuildBankBuyTab, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankBuyTab);
        GuildBankDepositMoney, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankDepositMoney);
        GuildBankLogQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankLogQuery);
        GuildBankQueryTab, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankQueryTab);
        GuildBankRemainingWithdrawMoneyQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankMoneyWithdrawn);
        GuildBankSetTabText, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankSetTabText);
        GuildBankSwapItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankSwapItems);
        GuildBankTextQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankTextQuery);
        GuildBankUpdateTab, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankUpdateTab);
        GuildBankWithdrawMoney, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildBankWithdrawMoney);
        GuildChallengeUpdateRequest, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildChallengeUpdateRequest);
        GuildChangeNameRequest, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildDeclineInvitation, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildDeclineInvitation);
        GuildDelete, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildDelete);
        GuildDeleteRank, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildDeleteRank);
        GuildDemoteMember, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildDemoteMember);
        GuildEventLogQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildEventLogQuery);
        GuildGetAchievementMembers, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildGetAchievementMembers);
        GuildGetRanks, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildGetRanks);
        GuildGetRoster, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildGetRoster);
        GuildInviteByName, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildInviteByName);
        GuildLeave, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildLeave);
        GuildMemberSendSorRequest, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildNewsUpdateSticky, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildNewsUpdateSticky);
        GuildOfficerRemoveMember, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildOfficerRemoveMember);
        GuildPermissionsQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildPermissionsQuery);
        GuildPromoteMember, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildPromoteMember);
        GuildQueryMembersForRecipe, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildQueryMemberRecipes, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildQueryNews, LoggedIn, Inplace; //,      &WorldSession::HandleGuildQueryNews);
        GuildQueryRecipes, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildReplaceGuildMaster, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildReplaceGuildMaster);
        GuildSetAchievementTracking, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildSetAchievementTracking);
        GuildSetFocusedAchievement, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildSetFocusedAchievement);
        GuildSetGuildMaster, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildSetGuildMaster);
        GuildSetMemberNote, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildSetMemberNote);
        GuildSetRankPermissions, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildSetRankPermissions);
        GuildShiftRank, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        GuildUpdateInfoText, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildUpdateInfoText);
        GuildUpdateMotdText, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildUpdateMotdText);
        HearthAndResurrect, LoggedIn, Threadunsafe; //, &WorldSession::HandleHearthAndResurrect);
        HotfixRequest, Authed, Threadunsafe; //, &WorldSession::HandleHotfixRequest);
        IgnoreTrade, LoggedIn, Threadunsafe; //, &WorldSession::HandleIgnoreTradeOpcode);
        InitiateRolePoll, LoggedIn, Threadunsafe; //, &WorldSession::HandleInitiateRolePoll);
        InitiateTrade, LoggedIn, Threadunsafe; //, &WorldSession::HandleInitiateTradeOpcode);
        Inspect, LoggedIn, Threadunsafe; //, &WorldSession::HandleInspectOpcode);
        InspectPvp, LoggedIn, Threadunsafe; //, &WorldSession::HandleInspectPVP);
        InstanceLockResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleInstanceLockResponse);
        ItemPurchaseRefund, LoggedIn, Threadunsafe; //, &WorldSession::HandleItemRefund);
        ItemTextQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleItemTextQuery);
        JoinPetBattleQueue, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        JoinRatedBattleground, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        KeepAlive, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        KeyboundOverride, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LearnPvpTalents, LoggedIn, Threadunsafe; //, &WorldSession::HandleLearnPvpTalentsOpcode);
        LearnTalents, LoggedIn, Threadunsafe; //, &WorldSession::HandleLearnTalentsOpcode);
        LeaveGroup, LoggedIn, Threadunsafe; //, &WorldSession::HandleLeaveGroupOpcode);
        LeavePetBattleQueue, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListApplyToGroup, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListCancelApplication, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListDeclineApplicant, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListGetStatus, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListInviteApplicant, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListInviteResponse, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListJoin, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListLeave, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListSearch, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfgListUpdateRequest, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LfGuildAddRecruit, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderAddRecruit);
        LfGuildBrowse, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderBrowse);
        LfGuildDeclineRecruit, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderDeclineRecruit);
        LfGuildGetApplications, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderGetApplications);
        LfGuildGetGuildPost, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderGetGuildPost);
        LfGuildGetRecruits, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderGetRecruits);
        LfGuildRemoveRecruit, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderRemoveRecruit);
        LfGuildSetGuildPost, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildFinderSetGuildPost);
        ListInventory, LoggedIn, Threadunsafe; //, &WorldSession::HandleListInventoryOpcode);
        LiveRegionAccountRestore, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        LiveRegionCharacterCopy, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        LiveRegionGetAccountCharacterList, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        LoadingScreenNotify, Authed, Threadunsafe; //, &WorldSession::HandleLoadScreenOpcode);
        LoadSelectedTrophy, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LogoutCancel, LoggedIn, Threadunsafe; //, &WorldSession::HandleLogoutCancelOpcode);
        LogoutInstant, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LogoutRequest, LoggedIn, Threadunsafe; //, &WorldSession::HandleLogoutRequestOpcode);
        LogDisconnect, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        LogStreamingError, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LootItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleAutostoreLootItemOpcode);
        LootMoney, LoggedIn, Threadunsafe; //, &WorldSession::HandleLootMoneyOpcode);
        LootRelease, LoggedIn, Threadunsafe; //, &WorldSession::HandleLootReleaseOpcode);
        LootRoll, LoggedIn, Threadunsafe; //, &WorldSession::HandleLootRoll);
        LootUnit, LoggedIn, Threadunsafe; //, &WorldSession::HandleLootOpcode);
        LowLevelRAID1, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        LowLevelRAID2, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        MailCreateTextItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleMailCreateTextItem);
        MailDelete, LoggedIn, Threadunsafe; //, &WorldSession::HandleMailDelete);
        MailGetList, LoggedIn, Threadunsafe; //, &WorldSession::HandleGetMailList);
        MailMarkAsRead, LoggedIn, Threadunsafe; //, &WorldSession::HandleMailMarkAsRead);
        MailReturnToSender, LoggedIn, Threadunsafe; //, &WorldSession::HandleMailReturnToSender);
        MailTakeItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleMailTakeItem);
        MailTakeMoney, LoggedIn, Threadunsafe; //, &WorldSession::HandleMailTakeMoney);
        MasterLootItem, Unhandled, Threadunsafe; //, &WorldSession::HandleLootMasterGiveOpcode);
        MinimapPing, LoggedIn, Threadunsafe; //, &WorldSession::HandleMinimapPingOpcode);
        MissileTrajectoryCollision, LoggedIn, Threadunsafe; //, &WorldSession::HandleMissileTrajectoryCollision);
        MountClearFanfare, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        MountSetFavorite, LoggedIn, Threadunsafe; //, &WorldSession::HandleMountSetFavorite);
        MountSpecialAnim, LoggedIn, Threadunsafe; //, &WorldSession::HandleMountSpecialAnimOpcode);
        MoveApplyMovementForceAck, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        MoveChangeTransport, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveChangeVehicleSeats, LoggedIn, Threadunsafe; //, &WorldSession::HandleMoveChangeVehicleSeats);
        MoveDismissVehicle, LoggedIn, Threadunsafe; //, &WorldSession::HandleMoveDismissVehicle);
        MoveDoubleJump, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveEnableDoubleJumpAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveEnableSwimToFlyTransAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveFallLand, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveFallReset, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveFeatherFallAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveForceFlightBackSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceFlightSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForcePitchRateChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceRootAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveForceRunBackSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceRunSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceSwimBackSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceSwimSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceTurnRateChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveForceUnrootAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveForceWalkSpeedChangeAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleForceSpeedChangeAck);
        MoveGravityDisableAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveGravityEnableAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveHeartbeat, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveHoverAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveJump, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveKnockBackAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMoveKnockBackAck);
        MoveRemoveMovementForces, Unhandled, Threadsafe; //,   &WorldSession::Handle_NULL);
        MoveRemoveMovementForceAck, Unhandled, Threadsafe; //,   &WorldSession::Handle_NULL);
        MoveSetCanFlyAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveSetCanTurnWhileFallingAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveSetCollisionHeightAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleSetCollisionHeightAck);
        MoveSetFacing, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveSetFly, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveSetIgnoreMovementForcesAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        MoveSetPitch, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveSetRunMode, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveSetVehicleRecIdAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMoveSetVehicleRecAck);
        MoveSetWalkMode, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveSplineDone, LoggedIn, Threadsafe; //,   &WorldSession::HandleMoveSplineDoneOpcode);
        MoveStartAscend, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartBackward, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartDescend, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartForward, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartPitchDown, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartPitchUp, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartStrafeLeft, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartStrafeRight, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartSwim, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartTurnLeft, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStartTurnRight, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStop, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStopAscend, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStopPitch, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStopStrafe, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStopSwim, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveStopTurn, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementOpcodes);
        MoveTeleportAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMoveTeleportAck);
        MoveTimeSkipped, LoggedIn, Inplace; //,      &WorldSession::HandleMoveTimeSkippedOpcode);
        MoveToggleCollisionCheat, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        MoveWaterWalkAck, LoggedIn, Threadsafe; //,   &WorldSession::HandleMovementAckMessage);
        NeutralPlayerSelectFaction, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        NextCinematicCamera, LoggedIn, Threadunsafe; //, &WorldSession::HandleNextCinematicCamera);
        ObjectUpdateFailed, LoggedIn, Threadunsafe; //, &WorldSession::HandleObjectUpdateFailedOpcode);
        ObjectUpdateRescued, LoggedIn, Inplace; //,      &WorldSession::HandleObjectUpdateRescuedOpcode);
        OfferPetition, LoggedIn, Threadunsafe; //, &WorldSession::HandleOfferPetition);
        OpeningCinematic, LoggedIn, Threadunsafe; //, &WorldSession::HandleOpeningCinematic);
        OpenItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleOpenItemOpcode);
        OpenMissionNpc, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        OpenShipmentNpc, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        OpenTradeskillNpc, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        OptOutOfLoot, LoggedIn, Threadunsafe; //, &WorldSession::HandleOptOutOfLootOpcode);
        PartyInvite, LoggedIn, Threadunsafe; //, &WorldSession::HandlePartyInviteOpcode);
        PartyInviteResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandlePartyInviteResponseOpcode);
        PartyUninvite, LoggedIn, Threadunsafe; //, &WorldSession::HandlePartyUninviteOpcode);
        PetitionBuy, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetitionBuy);
        PetitionRenameGuild, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetitionRenameGuild);
        PetitionShowList, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetitionShowList);
        PetitionShowSignatures, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetitionShowSignatures);
        PetAbandon, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetAbandon);
        PetAction, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetAction);
        PetBattleFinalNotify, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleInput, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleQueueProposeMatchResult, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleQuitNotify, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleReplaceFrontPet, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleRequestPvp, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleRequestUpdate, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleRequestWild, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleScriptErrorNotify, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PetBattleWildLocationFail, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        PetCancelAura, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetCancelAuraOpcode);
        PetCastSpell, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetCastSpellOpcode);
        PetRename, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetRename);
        PetSetAction, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetSetAction);
        PetSpellAutocast, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetSpellAutocastOpcode);
        PetStopAttack, LoggedIn, Threadunsafe; //, &WorldSession::HandlePetStopAttack);
        Ping, Never, Inplace; //,      &WorldSession::Handle_EarlyProccess);
        PlayerLogin, Authed, Threadunsafe; //, &WorldSession::HandlePlayerLoginOpcode);
        ProtocolMismatch, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        PushQuestToParty, LoggedIn, Threadunsafe; //, &WorldSession::HandlePushQuestToParty);
        PvpLogData, LoggedIn, Threadunsafe; //, &WorldSession::HandlePVPLogDataOpcode);
        PvpPrestigeRankUp, LoggedIn, Threadunsafe; //, &WorldSession::HandlePvpPrestigeRankUp);
        QueryBattlePetName, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        QueryCorpseLocationFromClient, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryCorpseLocation);
        QueryCorpseTransport, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryCorpseTransport);
        QueryCountdownTimer, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        QueryCreature, LoggedIn, Inplace; //,      &WorldSession::HandleCreatureQuery);
        QueryGameObject, LoggedIn, Inplace; //,      &WorldSession::HandleGameObjectQueryOpcode);
        QueryGarrisonCreatureName, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        QueryGuildInfo, Authed, Threadunsafe; //, &WorldSession::HandleGuildQueryOpcode);
        QueryInspectAchievements, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryInspectAchievements);
        QueryNextMailTime, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryNextMailTime);
        QueryNpcText, LoggedIn, Threadunsafe; //, &WorldSession::HandleNpcTextQueryOpcode);
        QueryPageText, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryPageText);
        QueryPetition, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryPetition);
        QueryPetName, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryPetName);
        QueryPlayerName, LoggedIn, Threadunsafe; //, &WorldSession::HandleNameQueryOpcode);
        QueryQuestCompletionNpcs, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryQuestCompletionNPCs);
        QueryQuestInfo, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestQueryOpcode);
        QueryQuestRewards, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        QueryRealmName, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryRealmName);
        QueryScenarioPoi, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryScenarioPOI);
        QueryTime, LoggedIn, Threadunsafe; //, &WorldSession::HandleQueryTimeOpcode);
        QueryVoidStorage, LoggedIn, Threadunsafe; //, &WorldSession::HandleVoidStorageQuery);
        QuestConfirmAccept, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestConfirmAccept);
        QuestGiverAcceptQuest, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverAcceptQuestOpcode);
        QuestGiverChooseReward, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverChooseRewardOpcode);
        QuestGiverCompleteQuest, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverCompleteQuest);
        QuestGiverHello, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverHelloOpcode);
        QuestGiverQueryQuest, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverQueryQuestOpcode);
        QuestGiverRequestReward, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverRequestRewardOpcode);
        QuestGiverStatusMultipleQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestgiverStatusMultipleQuery);
        QuestGiverStatusQuery, LoggedIn, Inplace; //,      &WorldSession::HandleQuestgiverStatusQueryOpcode);
        QuestLogRemoveQuest, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestLogRemoveQuest);
        QuestPoiQuery, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestPOIQuery);
        QuestPushResult, LoggedIn, Threadunsafe; //, &WorldSession::HandleQuestPushResult);
        QueuedMessagesEnd, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        QuickJoinAutoAcceptRequests, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        QuickJoinRequestInvite, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        QuickJoinRespondToInvite, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        QuickJoinSignalToastDisplayed, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        RaidOrBattlegroundEngineSurvey, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        RandomRoll, LoggedIn, Threadunsafe; //, &WorldSession::HandleRandomRollOpcode);
        ReadyCheckResponse, LoggedIn, Inplace; //,      &WorldSession::HandleReadyCheckResponseOpcode);
        ReadItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleReadItem);
        ReclaimCorpse, LoggedIn, Threadunsafe; //, &WorldSession::HandleReclaimCorpse);
        RecruitAFriend, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RedeemWowTokenConfirm, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RedeemWowTokenStart, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RemoveNewItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleRemoveNewItem);
        ReorderCharacters, Authed, Threadunsafe; //, &WorldSession::HandleReorderCharacters);
        RepairItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleRepairItemOpcode);
        ReplaceTrophy, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RepopRequest, LoggedIn, Threadunsafe; //, &WorldSession::HandleRepopRequest);
        ReportClientVariables, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ReportEnabledAddons, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ReportKeybindingExecutionCounts, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ReportPvpPlayerAfk, LoggedIn, Threadunsafe; //, &WorldSession::HandleReportPvPAFK);
        RequestAccountData, Authed, Threadunsafe; //, &WorldSession::HandleRequestAccountData);
        RequestAreaPoiUpdate, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        RequestBattlefieldStatus, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestBattlefieldStatusOpcode);
        RequestCategoryCooldowns, LoggedIn, Inplace; //,      &WorldSession::HandleRequestCategoryCooldowns);
        RequestCemeteryList, LoggedIn, Inplace; //,      &WorldSession::HandleRequestCemeteryList);
        RequestConquestFormulaConstants, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RequestConsumptionConversionInfo, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        RequestCrowdControlSpell, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        RequestForcedReactions, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestForcedReactionsOpcode);
        RequestGuildPartyState, LoggedIn, Threadunsafe; //, &WorldSession::HandleGuildRequestPartyState);
        RequestGuildRewardsList, LoggedIn, Inplace; //,      &WorldSession::HandleRequestGuildRewardsList);
        RequestHonorStats, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestHonorStatsOpcode);
        RequestLfgListBlacklist, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RequestPartyJoinUpdates, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestPartyJoinUpdates);
        RequestPartyMemberStats, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestPartyMemberStatsOpcode);
        RequestPetInfo, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestPetInfo);
        RequestPlayedTime, LoggedIn, Threadunsafe; //, &WorldSession::HandlePlayedTime);
        RequestPvpBrawlInfo, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        RequestPvpRewards, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestPvpReward);
        RequestRaidInfo, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestRaidInfoOpcode);
        RequestRatedBattlefieldInfo, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestRatedBattlefieldInfo);
        RequestResearchHistory, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RequestStabledPets, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestStabledPets);
        RequestVehicleExit, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestVehicleExit);
        RequestVehicleNextSeat, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestVehicleNextSeat);
        RequestVehiclePrevSeat, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestVehiclePrevSeat);
        RequestVehicleSwitchSeat, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestVehicleSwitchSeat);
        RequestWorldQuestUpdate, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestWorldQuestUpdate);
        RequestWowTokenMarketPrice, LoggedIn, Threadunsafe; //, &WorldSession::HandleRequestWowTokenMarketPrice);
        ResetChallengeMode, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ResetChallengeModeCheat, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        ResetInstances, LoggedIn, Threadunsafe; //, &WorldSession::HandleResetInstancesOpcode);
        ResurrectResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleResurrectResponse);
        RevertMonumentAppearance, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        RideVehicleInteract, LoggedIn, Threadunsafe; //, &WorldSession::HandleRideVehicleInteract);
        SaveCufProfiles, LoggedIn, Inplace; //,      &WorldSession::HandleSaveCUFProfiles);
        SaveEquipmentSet, LoggedIn, Threadunsafe; //, &WorldSession::HandleEquipmentSetSave);
        SaveGuildEmblem, LoggedIn, Threadunsafe; //, &WorldSession::HandleSaveGuildEmblem);
        ScenePlaybackCanceled, LoggedIn, Threadunsafe; //, &WorldSession::HandleScenePlaybackCanceled);
        ScenePlaybackComplete, LoggedIn, Threadunsafe; //, &WorldSession::HandleScenePlaybackComplete);
        SceneTriggerEvent, LoggedIn, Threadunsafe; //, &WorldSession::HandleSceneTriggerEvent);
        SelfRes, LoggedIn, Threadunsafe; //, &WorldSession::HandleSelfResOpcode);
        SellItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleSellItemOpcode);
        SellWowTokenConfirm, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SellWowTokenStart, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SendContactList, LoggedIn, Threadunsafe; //, &WorldSession::HandleContactListOpcode);
        SendMail, LoggedIn, Threadunsafe; //, &WorldSession::HandleSendMail);
        SendSorRequestViaAddress, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SendTextEmote, LoggedIn, Threadunsafe; //, &WorldSession::HandleTextEmoteOpcode);
        SetAchievementsHidden, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetActionBarToggles, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetActionBarToggles);
        SetActionButton, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetActionButtonOpcode);
        SetActiveMover, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetActiveMoverOpcode);
        SetAdvancedCombatLogging, LoggedIn, Inplace; //,      &WorldSession::HandleSetAdvancedCombatLogging);
        SetAssistantLeader, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetAssistantLeaderOpcode);
        SetBackpackAutosortDisabled, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetBankAutosortDisabled, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetBankBagSlotFlag, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetContactNotes, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetContactNotesOpcode);
        SetCurrencyFlags, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetDifficultyId, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetDungeonDifficulty, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetDungeonDifficultyOpcode);
        SetEveryoneIsAssistant, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetEveryoneIsAssistant);
        SetFactionAtWar, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetFactionAtWar);
        SetFactionInactive, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetFactionInactiveOpcode);
        SetFactionNotAtWar, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetFactionNotAtWar);
        SetInsertItemsLeftToRight, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetLfgBonusFactionId, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetLootMethod, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetLootMethodOpcode);
        SetLootSpecialization, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetLootSpecialization);
        SetPartyAssignment, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetPartyAssignment);
        SetPartyLeader, LoggedIn, Inplace; //,      &WorldSession::HandleSetPartyLeaderOpcode);
        SetPetSlot, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetPlayerDeclinedNames, Authed, Threadunsafe; //, &WorldSession::HandleSetPlayerDeclinedNames);
        SetPreferredCemetery, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetPvp, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetPvP);
        SetRaidDifficulty, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetRaidDifficultyOpcode);
        SetRole, LoggedIn, Inplace; //,      &WorldSession::HandleSetRoleOpcode);
        SetSavedInstanceExtend, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetSavedInstanceExtend);
        SetSelection, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetSelectionOpcode);
        SetSheathed, LoggedIn, Inplace; //,      &WorldSession::HandleSetSheathedOpcode);
        SetSortBagsRightToLeft, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetTaxiBenchmarkMode, LoggedIn, Inplace; //,      &WorldSession::HandleSetTaxiBenchmark);
        SetTitle, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetTitleOpcode);
        SetTradeCurrency, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetTradeCurrencyOpcode);
        SetTradeGold, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetTradeGoldOpcode);
        SetTradeItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetTradeItemOpcode);
        SetUsingPartyGarrison, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SetWatchedFaction, LoggedIn, Threadunsafe; //, &WorldSession::HandleSetWatchedFactionOpcode);
        ShowTradeSkill, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SignPetition, LoggedIn, Threadunsafe; //, &WorldSession::HandleSignPetition);
        SilencePartyTalker, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SocketGems, LoggedIn, Threadunsafe; //, &WorldSession::HandleSocketGems);
        SortBags, LoggedIn, Threadunsafe; //, &WorldSession::HandleSortBags);
        SortBankBags, LoggedIn, Threadunsafe; //, &WorldSession::HandleSortBankBags);
        SortReagentBankBags, LoggedIn, Threadunsafe; //, &WorldSession::HandleSortReagentBankBags);
        SpellClick, LoggedIn, Threadunsafe; //, &WorldSession::HandleSpellClick);
        SpiritHealerActivate, LoggedIn, Threadunsafe; //, &WorldSession::HandleSpiritHealerActivate);
        SplitItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleSplitItemOpcode);
        StandStateChange, LoggedIn, Threadunsafe; //, &WorldSession::HandleStandStateChangeOpcode);
        StartChallengeMode, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        StartSpectatorWarGame, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        StartWarGame, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SummonResponse, LoggedIn, Threadunsafe; //, &WorldSession::HandleSummonResponseOpcode);
        SupportTicketSubmitBug, LoggedIn, Threadunsafe; //, &WorldSession::HandleSupportTicketSubmitBug);
        SupportTicketSubmitComplaint, LoggedIn, Threadunsafe; //, &WorldSession::HandleSupportTicketSubmitComplaint);
        SupportTicketSubmitSuggestion, LoggedIn, Threadunsafe; //, &WorldSession::HandleSupportTicketSubmitSuggestion);
        SurrenderArena, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        SuspendCommsAck, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        SuspendTokenResponse, Transfer, Threadunsafe; //, &WorldSession::HandleSuspendTokenResponse);
        SwapInvItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleSwapInvItemOpcode);
        SwapItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleSwapItem);
        SwapSubGroups, LoggedIn, Threadunsafe; //, &WorldSession::HandleSwapSubGroupsOpcode);
        SwapVoidItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleVoidSwapItem);
        TabardVendorActivate, LoggedIn, Threadunsafe; //, &WorldSession::HandleTabardVendorActivateOpcode);
        TalkToGossip, LoggedIn, Threadunsafe; //, &WorldSession::HandleGossipHelloOpcode);
        TaxiNodeStatusQuery, LoggedIn, Threadsafe; //,   &WorldSession::HandleTaxiNodeStatusQueryOpcode);
        TaxiQueryAvailableNodes, LoggedIn, Threadsafe; //,   &WorldSession::HandleTaxiQueryAvailableNodesOpcode);
        TaxiRequestEarlyLanding, LoggedIn, Threadsafe; //,   &WorldSession::HandleTaxiRequestEarlyLanding);
        TimeAdjustmentResponse, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        TimeSyncResponse, LoggedIn, Inplace; //,      &WorldSession::HandleTimeSyncResponse);
        TimeSyncResponseDropped, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        TimeSyncResponseFailed, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        ToggleDifficulty, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        TogglePvp, LoggedIn, Threadunsafe; //, &WorldSession::HandleTogglePvP);
        TotemDestroyed, LoggedIn, Threadunsafe; //, &WorldSession::HandleTotemDestroyed);
        TradeSkillSetFavorite, Unhandled, Threadunsafe; //, &WorldSession::Handle_NULL);
        TrainerBuySpell, LoggedIn, Threadunsafe; //, &WorldSession::HandleTrainerBuySpellOpcode);
        TrainerList, LoggedIn, Threadunsafe; //, &WorldSession::HandleTrainerListOpcode);
        TransmogrifyItems, LoggedIn, Threadunsafe; //, &WorldSession::HandleTransmogrifyItems);
        TurnInPetition, LoggedIn, Threadunsafe; //, &WorldSession::HandleTurnInPetition);
        Tutorial, LoggedIn, Threadunsafe; //, &WorldSession::HandleTutorialFlag);
        TwitterCheckStatus, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        TwitterConnect, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        TwitterDisconnect, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        TwitterPost, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UiTimeRequest, Authed, Inplace; //,      &WorldSession::HandleUITimeRequest);
        UnacceptTrade, LoggedIn, Threadunsafe; //, &WorldSession::HandleUnacceptTradeOpcode);
        UndeleteCharacter, Authed, Threadunsafe; //, &WorldSession::HandleCharUndeleteOpcode);
        UnlearnSkill, LoggedIn, Threadunsafe; //, &WorldSession::HandleUnlearnSkillOpcode);
        UnlearnSpecialization, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UnlockVoidStorage, LoggedIn, Threadunsafe; //, &WorldSession::HandleVoidStorageUnlock);
        UpdateAccountData, Authed, Threadunsafe; //, &WorldSession::HandleUpdateAccountData);
        UpdateAreaTriggerVisual, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UpdateClientSettings, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UpdateMissileTrajectory, LoggedIn, Threadunsafe; //, &WorldSession::HandleUpdateMissileTrajectory);
        UpdateRaidTarget, LoggedIn, Threadunsafe; //, &WorldSession::HandleUpdateRaidTargetOpcode);
        UpdateSpellVisual, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UpdateVasPurchaseStates, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UpdateWowTokenAuctionableList, LoggedIn, Threadunsafe; //, &WorldSession::HandleUpdateListedAuctionableTokens);
        UpdateWowTokenCount, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UpgradeGarrison, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UpgradeItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleUpgradeItem);
        UsedFollow, Unhandled, Inplace; //,      &WorldSession::Handle_NULL);
        UseCritterItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleUseCritterItem);
        UseEquipmentSet, LoggedIn, Threadunsafe; //, &WorldSession::HandleUseEquipmentSet);
        UseItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleUseItemOpcode);
        UseToy, LoggedIn, Threadunsafe; //, &WorldSession::HandleUseToy);
        ViolenceLevel, Authed, Inplace; //,      &WorldSession::HandleViolenceLevel);
        VoidStorageTransfer, LoggedIn, Threadunsafe; //, &WorldSession::HandleVoidStorageTransfer);
        WardenData, Unhandled, Threadunsafe; //, &WorldSession::HandleWardenData);
        Who, LoggedIn, Threadunsafe; //, &WorldSession::HandleWhoOpcode);
        WhoIs, LoggedIn, Threadunsafe; //, &WorldSession::HandleWhoIsOpcode);
        WorldPortResponse, Transfer, Threadunsafe; //, &WorldSession::HandleMoveWorldportAckOpcode);
        WrapItem, LoggedIn, Threadunsafe; //, &WorldSession::HandleWrapItem);
        AssignEquipmentSetSpec, Unhandled, Threadunsafe; // NOTE: TC doesn't seem to handle this at all
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_derive::FromPrimitive)]
#[repr(u16)]
pub enum OpcodeServer {
    AbortNewWorld = 0x25AD,
    AccountCriteriaUpdate = 0x2652,
    AccountDataTimes = 0x2749,
    AccountMountUpdate = 0x25C3,
    AccountToysUpdate = 0x25C4,
    AchievementDeleted = 0x271E,
    AchievementEarned = 0x2660,
    ActivateTaxiReply = 0x26A6,
    ActiveGlyphs = 0x2C53,
    AddBattlenetFriendResponse = 0x265A,
    AddItemPassive = 0x25BF,
    AddLossOfControl = 0x2696,
    AddRunePower = 0x26E2,
    AdjustSplineDuration = 0x25E8,
    AeLootTargets = 0x262C,
    AeLootTargetAck = 0x262D,
    AiReaction = 0x26DF,
    AllAccountCriteria = 0x2570,
    AllAchievementData = 0x256F,
    AllGuildAchievements = 0x29B8,
    ArchaeologySurveryCast = 0x2586,
    AreaPoiUpdate = 0x2848,
    AreaSpiritHealerTime = 0x2782,
    AreaTriggerDenied = 0x269D,
    AreaTriggerNoCorpse = 0x2755,
    AreaTriggerRePath = 0x263F,
    AreaTriggerReShape = 0x263C,
    ArenaCrowdControlSpells = 0x264E,
    ArenaError = 0x2711,
    ArenaPrepOpponentSpecializations = 0x2665,
    ArtifactForgeOpened = 0x27E2,
    ArtifactKnowledge = 0x27EA,
    ArtifactRespecConfirm = 0x27E5,
    ArtifactTraitsRefunded = 0x27E6,
    ArtifactXpGain = 0x282D,
    AttackerStateUpdate = 0x27CF,
    AttackStart = 0x266D,
    AttackStop = 0x266E,
    AttackSwingError = 0x2733,
    AttackSwingLandedLog = 0x2734,
    AuctionClosedNotification = 0x2728,
    AuctionCommandResult = 0x2725,
    AuctionHelloResponse = 0x2723,
    AuctionListBidderItemsResult = 0x272C,
    AuctionListItemsResult = 0x272A,
    AuctionListOwnerItemsResult = 0x272B,
    AuctionListPendingSalesResult = 0x272D,
    AuctionOutbidNotification = 0x2727,
    AuctionOwnerBidNotification = 0x2729,
    AuctionReplicateResponse = 0x2724,
    AuctionWonNotification = 0x2726,
    AuraPointsDepleted = 0x2C23,
    AuraUpdate = 0x2C22,
    AuthChallenge = 0x3048,
    AuthResponse = 0x256C,
    AvailableHotfixes = 0x25A1,
    BanReason = 0x26B2,
    BarberShopResult = 0x26E8,
    BattlefieldList = 0x2594,
    BattlefieldPortDenied = 0x259A,
    BattlefieldStatusActive = 0x2590,
    BattlefieldStatusFailed = 0x2593,
    BattlefieldStatusNeedConfirmation = 0x258F,
    BattlefieldStatusNone = 0x2592,
    BattlefieldStatusQueued = 0x2591,
    BattlefieldStatusWaitForGroups = 0x25A5,
    BattlegroundInfoThrottled = 0x259B,
    BattlegroundInit = 0x27A0,
    BattlegroundPlayerJoined = 0x2598,
    BattlegroundPlayerLeft = 0x2599,
    BattlegroundPlayerPositions = 0x2595,
    BattlegroundPoints = 0x279F,
    BattlenetChallengeAbort = 0x27CE,
    BattlenetChallengeStart = 0x27CD,
    BattlenetNotification = 0x2843,
    BattlenetRealmListTicket = 0x2845,
    BattlenetResponse = 0x2842,
    BattlenetSetSessionState = 0x2844,
    BattlePayAckFailed = 0x27C6,
    BattlePayBattlePetDelivered = 0x27BB,
    BattlePayConfirmPurchase = 0x27C5,
    BattlePayDeliveryEnded = 0x27B9,
    BattlePayDeliveryStarted = 0x27B8,
    BattlePayDistributionUpdate = 0x27B7,
    BattlePayGetDistributionListResponse = 0x27B5,
    BattlePayGetProductListResponse = 0x27B3,
    BattlePayGetPurchaseListResponse = 0x27B4,
    BattlePayMountDelivered = 0x27BA,
    BattlePayPurchaseUpdate = 0x27C4,
    BattlePayStartDistributionAssignToTargetResponse = 0x27C2,
    BattlePayStartPurchaseResponse = 0x27C1,
    BattlePaySubscriptionChanged = 0x2864,
    BattlePayVasBnetTransferValidationResult = 0x285B,
    BattlePayVasBoostConsumed = 0x27B6,
    BattlePayVasCharacterList = 0x2830,
    BattlePayVasCharacterQueueStatus = 0x2859,
    BattlePayVasPurchaseComplete = 0x2833,
    BattlePayVasPurchaseList = 0x2834,
    BattlePayVasPurchaseStarted = 0x2832,
    BattlePayVasRealmList = 0x2831,
    BattlePayVasTransferQueueStatus = 0x2858,
    BattlePetsHealed = 0x2609,
    BattlePetCageDateError = 0x26A0,
    BattlePetDeleted = 0x2606,
    BattlePetError = 0x2655,
    BattlePetJournal = 0x2605,
    BattlePetJournalLockAcquired = 0x2603,
    BattlePetJournalLockDenied = 0x2604,
    BattlePetLicenseChanged = 0x260A,
    BattlePetMaxCountChanged = 0x2601,
    BattlePetRestored = 0x2608,
    BattlePetRevoked = 0x2607,
    BattlePetTrapLevel = 0x2600,
    BattlePetUpdates = 0x25FF,
    BinderConfirm = 0x2739,
    BindPointUpdate = 0x257C,
    BlackMarketBidOnItemResult = 0x2644,
    BlackMarketOpenResult = 0x2642,
    BlackMarketOutbid = 0x2645,
    BlackMarketRequestItemsResult = 0x2643,
    BlackMarketWon = 0x2646,
    BonusRollEmpty = 0x2662,
    BossKillCredit = 0x27C0,
    BreakTarget = 0x266C,
    BuyFailed = 0x26F1,
    BuySucceeded = 0x26F0,
    CacheInfo = 0x2743,
    CacheVersion = 0x2742,
    CalendarClearPendingAction = 0x26C6,
    CalendarCommandResult = 0x26C7,
    CalendarEventInitialInvites = 0x26B6,
    CalendarEventInvite = 0x26B7,
    CalendarEventInviteAlert = 0x26B8,
    CalendarEventInviteModeratorStatus = 0x26BB,
    CalendarEventInviteNotes = 0x26C0,
    CalendarEventInviteNotesAlert = 0x26C1,
    CalendarEventInviteRemoved = 0x26BC,
    CalendarEventInviteRemovedAlert = 0x26BD,
    CalendarEventInviteStatus = 0x26B9,
    CalendarEventInviteStatusAlert = 0x26BA,
    CalendarEventRemovedAlert = 0x26BE,
    CalendarEventUpdatedAlert = 0x26BF,
    CalendarRaidLockoutAdded = 0x26C2,
    CalendarRaidLockoutRemoved = 0x26C3,
    CalendarRaidLockoutUpdated = 0x26C4,
    CalendarSendCalendar = 0x26B4,
    CalendarSendEvent = 0x26B5,
    CalendarSendNumPending = 0x26C5,
    CameraEffect = 0x2767,
    CancelAutoRepeat = 0x2712,
    CancelCombat = 0x2731,
    CancelOrphanSpellVisual = 0x2C46,
    CancelScene = 0x2654,
    CancelSpellVisual = 0x2C44,
    CancelSpellVisualKit = 0x2C48,
    CanDuelResult = 0x2676,
    CastFailed = 0x2C56,
    CategoryCooldown = 0x2C16,
    ChallengeModeAllMapStats = 0x2622,
    ChallengeModeComplete = 0x2620,
    ChallengeModeMapStatsUpdate = 0x2623,
    ChallengeModeNewPlayerRecord = 0x2625,
    ChallengeModeRequestLeadersResult = 0x2624,
    ChallengeModeReset = 0x261F,
    ChallengeModeRewards = 0x2621,
    ChallengeModeStart = 0x261D,
    ChallengeModeUpdateDeathCount = 0x261E,
    ChangePlayerDifficultyResult = 0x2735,
    ChannelList = 0x2BC3,
    ChannelNotify = 0x2BC0,
    ChannelNotifyJoined = 0x2BC1,
    ChannelNotifyLeft = 0x2BC2,
    CharacterClassTrialCreate = 0x2804,
    CharacterInventoryOverflowWarning = 0x2863,
    CharacterItemFixup = 0x2854,
    CharacterLoginFailed = 0x2744,
    CharacterObjectTestResponse = 0x27CC,
    CharacterRenameResult = 0x27A5,
    CharacterUpgradeComplete = 0x2803,
    CharacterUpgradeQueued = 0x2802,
    CharacterUpgradeSpellTierSet = 0x25F4,
    CharacterUpgradeStarted = 0x2801,
    CharacterUpgradeUnrevokeResult = 0x2805,
    CharCustomize = 0x2719,
    CharCustomizeFailed = 0x2718,
    CharFactionChangeResult = 0x27EE,
    Chat = 0x2BAD,
    ChatAutoResponded = 0x2BB8,
    ChatDown = 0x2BBD,
    ChatIgnoredAccountMuted = 0x2BAC,
    ChatIsDown = 0x2BBE,
    ChatNotInParty = 0x2BB2,
    ChatPlayerAmbiguous = 0x2BB0,
    ChatPlayerNotfound = 0x2BB7,
    ChatReconnect = 0x2BBF,
    ChatRestricted = 0x2BB3,
    ChatServerMessage = 0x2BC4,
    CheatIgnoreDimishingReturns = 0x2C12,
    CheckWargameEntry = 0x259E,
    ClearAllSpellCharges = 0x2C27,
    ClearBossEmotes = 0x25CD,
    ClearCooldown = 0x26E4,
    ClearCooldowns = 0x2C26,
    ClearLossOfControl = 0x2698,
    ClearSpellCharges = 0x2C28,
    ClearTarget = 0x26DB,
    CoinRemoved = 0x262B,
    CombatEventFailed = 0x266F,
    CommentatorMapInfo = 0x2746,
    CommentatorPlayerInfo = 0x2747,
    CommentatorStateChanged = 0x2745,
    ComplaintResult = 0x26D3,
    CompleteShipmentResponse = 0x27DE,
    ConnectTo = 0x304D,
    ConquestFormulaConstants = 0x27C7,
    ConsoleWrite = 0x2651,
    ConsumptionConversionInfoResponse = 0x2849,
    ConsumptionConversionResult = 0x284A,
    ContactList = 0x27CA,
    ControlUpdate = 0x2664,
    CooldownCheat = 0x277B,
    CooldownEvent = 0x26E3,
    CorpseLocation = 0x266B,
    CorpseReclaimDelay = 0x278E,
    CorpseTransportQuery = 0x2751,
    CreateChar = 0x273E,
    CreateShipmentResponse = 0x27DD,
    CriteriaDeleted = 0x271D,
    CriteriaUpdate = 0x2717,
    CrossedInebriationThreshold = 0x26EC,
    CustomLoadScreen = 0x25E3,
    DailyQuestsReset = 0x2A80,
    DamageCalcLog = 0x280C,
    DbReply = 0x25A0,
    DeathReleaseLoc = 0x2705,
    DefenseMessage = 0x2BB6,
    DeleteChar = 0x273F,
    DestroyArenaUnit = 0x2784,
    DestructibleBuildingDamage = 0x2732,
    DifferentInstanceFromParty = 0x258A,
    DisenchantCredit = 0x25BC,
    Dismount = 0x26DA,
    DismountResult = 0x257B,
    DispelFailed = 0x2C30,
    DisplayGameError = 0x25B5,
    DisplayPlayerChoice = 0x26A1,
    DisplayPromotion = 0x2668,
    DisplayQuestPopup = 0x2A9D,
    DisplayToast = 0x2638,
    DontAutoPushSpellsToActionBar = 0x25F6,
    DropNewConnection = 0x304C,
    DuelComplete = 0x2674,
    DuelCountdown = 0x2673,
    DuelInBounds = 0x2672,
    DuelOutOfBounds = 0x2671,
    DuelRequested = 0x2670,
    DuelWinner = 0x2675,
    DurabilityDamageDeath = 0x278A,
    Emote = 0x280D,
    EnableBarberShop = 0x26E7,
    EnableEncryption = 0x3049,
    EnchantmentLog = 0x2752,
    EncounterEnd = 0x27BF,
    EncounterStart = 0x27BE,
    EnumCharactersResult = 0x2582,
    EnvironmentalDamageLog = 0x2C21,
    EquipmentSetId = 0x26DC,
    ExpectedSpamRecords = 0x2BB1,
    ExplorationExperience = 0x27A2,
    FactionBonusInfo = 0x2766,
    FailedPlayerCondition = 0x25E2,
    FeatureSystemStatus = 0x25D1,
    FeatureSystemStatusGlueScreen = 0x25D2,
    FeignDeathResisted = 0x2787,
    FishEscaped = 0x26F9,
    FishNotHooked = 0x26F8,
    FlightSplineSync = 0x2DF7,
    ForcedDeathUpdate = 0x2706,
    ForceAnim = 0x2794,
    ForceObjectRelink = 0x2667,
    FriendStatus = 0x27CB,
    GameObjectActivateAnimKit = 0x25D6,
    GameObjectCustomAnim = 0x25D7,
    GameObjectDespawn = 0x25D8,
    GameObjectPlaySpellVisual = 0x2C4B,
    GameObjectPlaySpellVisualKit = 0x2C4A,
    GameObjectResetState = 0x275D,
    GameObjectSetState = 0x2841,
    GameObjectUiAction = 0x275A,
    GameSpeedSet = 0x26AA,
    GameTimeSet = 0x274B,
    GameTimeUpdate = 0x274A,
    GarrisonAddFollowerResult = 0x2902,
    GarrisonAddMissionResult = 0x2906,
    GarrisonAssignFollowerToBuildingResult = 0x2918,
    GarrisonBuildingActivated = 0x28FB,
    GarrisonBuildingLandmarks = 0x292C,
    GarrisonBuildingRemoved = 0x28F4,
    GarrisonBuildingSetActiveSpecializationResult = 0x28F6,
    GarrisonClearAllFollowersExhaustion = 0x2916,
    GarrisonCompleteMissionResult = 0x2909,
    GarrisonCreateResult = 0x28FC,
    GarrisonDeleteResult = 0x2920,
    GarrisonFollowerChangedAbilities = 0x2914,
    GarrisonFollowerChangedDurability = 0x2904,
    GarrisonFollowerChangedItemLevel = 0x2913,
    GarrisonFollowerChangedStatus = 0x2915,
    GarrisonFollowerChangedXp = 0x2912,
    GarrisonIsUpgradeableResult = 0x2929,
    GarrisonLandingPageShipmentInfo = 0x27E0,
    GarrisonLearnBlueprintResult = 0x28F7,
    GarrisonLearnSpecializationResult = 0x28F5,
    GarrisonListFollowersCheatResult = 0x2905,
    GarrisonListMissionsCheatResult = 0x292A,
    GarrisonMissionAreaBonusAdded = 0x2910,
    GarrisonMissionBonusRollResult = 0x290C,
    GarrisonMissionRewardResponse = 0x292D,
    GarrisonMissionUpdateCanStart = 0x2911,
    GarrisonNumFollowerActivationsRemaining = 0x2917,
    GarrisonOpenArchitect = 0x2921,
    GarrisonOpenMissionNpc = 0x2923,
    GarrisonOpenRecruitmentNpc = 0x291C,
    GarrisonOpenTalentNpc = 0x291D,
    GarrisonOpenTradeskillNpc = 0x2922,
    GarrisonPlaceBuildingResult = 0x28F3,
    GarrisonPlotPlaced = 0x28F1,
    GarrisonPlotRemoved = 0x28F2,
    GarrisonRecallPortalLastUsedTime = 0x290A,
    GarrisonRecallPortalUsed = 0x290B,
    GarrisonRecruitmentFollowersGenerated = 0x291E,
    GarrisonRecruitFollowerResult = 0x291F,
    GarrisonRemoteInfo = 0x28FA,
    GarrisonRemoveFollowerFromBuildingResult = 0x2919,
    GarrisonRemoveFollowerResult = 0x2903,
    GarrisonRequestBlueprintAndSpecializationDataResult = 0x28F9,
    GarrisonStartMissionResult = 0x2907,
    GarrisonUnlearnBlueprintResult = 0x28F8,
    GarrisonUpgradeResult = 0x28FD,
    GenerateRandomCharacterNameResult = 0x2583,
    GetAccountCharacterListResult = 0x27A3,
    GetDisplayedTrophyListResponse = 0x2928,
    GetGarrisonInfoResult = 0x28F0,
    GetShipmentsOfTypeResponse = 0x27DF,
    GetShipmentInfoResponse = 0x27DB,
    GetTrophyListResponse = 0x2808,
    GmPlayerInfo = 0x277A,
    GmRequestPlayerInfo = 0x25ED,
    GmTicketCaseStatus = 0x26CC,
    GmTicketSystemStatus = 0x26CB,
    GodMode = 0x2738,
    GossipComplete = 0x2A96,
    GossipMessage = 0x2A97,
    GossipPoi = 0x27D8,
    GroupActionThrottled = 0x259C,
    GroupDecline = 0x27D3,
    GroupDestroyed = 0x27D5,
    GroupInviteConfirmation = 0x2855,
    GroupNewLeader = 0x2649,
    GroupUninvite = 0x27D4,
    GuildAchievementDeleted = 0x29C5,
    GuildAchievementEarned = 0x29C4,
    GuildAchievementMembers = 0x29C7,
    GuildBankLogQueryResults = 0x29DF,
    GuildBankQueryResults = 0x29DE,
    GuildBankRemainingWithdrawMoney = 0x29E0,
    GuildBankTextQueryResult = 0x29E3,
    GuildChallengeCompleted = 0x29D3,
    GuildChallengeUpdate = 0x29D2,
    GuildChangeNameResult = 0x29DD,
    GuildCommandResult = 0x29BA,
    GuildCriteriaDeleted = 0x29C6,
    GuildCriteriaUpdate = 0x29C3,
    GuildEventBankContentsChanged = 0x29F5,
    GuildEventBankMoneyChanged = 0x29F4,
    GuildEventDisbanded = 0x29EB,
    GuildEventLogQueryResults = 0x29E2,
    GuildEventMotd = 0x29EC,
    GuildEventNewLeader = 0x29EA,
    GuildEventPlayerJoined = 0x29E8,
    GuildEventPlayerLeft = 0x29E9,
    GuildEventPresenceChange = 0x29ED,
    GuildEventRanksUpdated = 0x29EE,
    GuildEventRankChanged = 0x29EF,
    GuildEventTabAdded = 0x29F0,
    GuildEventTabDeleted = 0x29F1,
    GuildEventTabModified = 0x29F2,
    GuildEventTabTextChanged = 0x29F3,
    GuildFlaggedForRename = 0x29DC,
    GuildInvite = 0x29CA,
    GuildInviteDeclined = 0x29E6,
    GuildInviteExpired = 0x29E7,
    GuildItemLooted = 0x29D4,
    GuildKnownRecipes = 0x29BE,
    GuildMembersWithRecipe = 0x29BF,
    GuildMemberDailyReset = 0x29E4,
    GuildMemberRecipes = 0x29BD,
    GuildMemberUpdateNote = 0x29C9,
    GuildMoved = 0x29DA,
    GuildMoveStarting = 0x29D9,
    GuildNameChanged = 0x29DB,
    GuildNews = 0x29C1,
    GuildNewsDeleted = 0x29C2,
    GuildPartyState = 0x29CB,
    GuildPermissionsQueryResults = 0x29E1,
    GuildRanks = 0x29C8,
    GuildReputationReactionChanged = 0x29CC,
    GuildReset = 0x29D8,
    GuildRewardList = 0x29C0,
    GuildRoster = 0x29BB,
    GuildRosterUpdate = 0x29BC,
    GuildSendRankChange = 0x29B9,
    HealthUpdate = 0x26FC,
    HighestThreatUpdate = 0x270C,
    HotfixMessage = 0x25A2,
    HotfixResponse = 0x25A3,
    InitializeFactions = 0x2765,
    InitialSetup = 0x257F,
    InitWorldStates = 0x278B,
    InspectHonorStats = 0x25B2,
    InspectPvp = 0x2761,
    InspectResult = 0x264D,
    InstanceEncounterChangePriority = 0x27F4,
    InstanceEncounterDisengageUnit = 0x27F3,
    InstanceEncounterEnd = 0x27FC,
    InstanceEncounterEngageUnit = 0x27F2,
    InstanceEncounterGainCombatResurrectionCharge = 0x27FE,
    InstanceEncounterInCombatResurrection = 0x27FD,
    InstanceEncounterObjectiveComplete = 0x27F7,
    InstanceEncounterObjectiveStart = 0x27F6,
    InstanceEncounterObjectiveUpdate = 0x27FB,
    InstanceEncounterPhaseShiftChanged = 0x27FF,
    InstanceEncounterSetAllowingRelease = 0x27FA,
    InstanceEncounterSetSuppressingRelease = 0x27F9,
    InstanceEncounterStart = 0x27F8,
    InstanceEncounterTimerStart = 0x27F5,
    InstanceGroupSizeChanged = 0x2736,
    InstanceInfo = 0x2650,
    InstanceReset = 0x26AF,
    InstanceResetFailed = 0x26B0,
    InstanceSaveCreated = 0x27BD,
    InvalidatePageText = 0x2701,
    InvalidatePlayer = 0x26D2,
    InvalidPromotionCode = 0x2795,
    InventoryChangeFailure = 0x2763,
    IsQuestCompleteResponse = 0x2A83,
    ItemChanged = 0x2720,
    ItemCooldown = 0x280B,
    ItemEnchantTimeUpdate = 0x2797,
    ItemExpirePurchaseRefund = 0x25B1,
    ItemPurchaseRefundResult = 0x25AF,
    ItemPushResult = 0x2637,
    ItemTimeUpdate = 0x2796,
    KickReason = 0x282F,
    LearnedSpells = 0x2C4D,
    LearnPvpTalentsFailed = 0x25EA,
    LearnTalentsFailed = 0x25E9,
    LevelUpdate = 0x2587,
    LevelUpInfo = 0x271F,
    LfgBootPlayer = 0x2A35,
    LfgDisabled = 0x2A33,
    LfgInstanceShutdownCountdown = 0x2A25,
    LfgJoinResult = 0x2A1C,
    LfgListJoinResult = 0x2A1D,
    LfgListSearchResults = 0x2A1E,
    LfgListSearchStatus = 0x2A1F,
    LfgListUpdateBlacklist = 0x2A2A,
    LfgListUpdateStatus = 0x2A26,
    LfgOfferContinue = 0x2A34,
    LfgPartyInfo = 0x2A36,
    LfgPlayerInfo = 0x2A37,
    LfgPlayerReward = 0x2A38,
    LfgProposalUpdate = 0x2A2D,
    LfgQueueStatus = 0x2A20,
    LfgReadyCheckResult = 0x2A3A,
    LfgReadyCheckUpdate = 0x2A22,
    LfgRoleCheckUpdate = 0x2A21,
    LfgSlotInvalid = 0x2A30,
    LfgTeleportDenied = 0x2A32,
    LfgUpdateStatus = 0x2A24,
    LfGuildApplicantListChanged = 0x29D5,
    LfGuildApplications = 0x29D1,
    LfGuildApplicationsListChanged = 0x29D6,
    LfGuildBrowse = 0x29CE,
    LfGuildCommandResult = 0x29D0,
    LfGuildPost = 0x29CD,
    LfGuildRecruits = 0x29CF,
    LiveRegionAccountRestoreResult = 0x27B1,
    LiveRegionCharacterCopyResult = 0x27AF,
    LiveRegionGetAccountCharacterListResult = 0x27A4,
    LoadCufProfiles = 0x25CE,
    LoadEquipmentSet = 0x274D,
    LoadSelectedTrophyResult = 0x2809,
    LoginSetTimeSpeed = 0x274C,
    LoginVerifyWorld = 0x25AC,
    LogoutCancelAck = 0x26AE,
    LogoutComplete = 0x26AD,
    LogoutResponse = 0x26AC,
    LogXpGain = 0x271B,
    LootAllPassed = 0x2635,
    LootList = 0x2783,
    LootMoneyNotify = 0x2630,
    LootRelease = 0x262F,
    LootReleaseAll = 0x262E,
    LootRemoved = 0x2629,
    LootResponse = 0x2628,
    LootRoll = 0x2632,
    LootRollsComplete = 0x2634,
    LootRollWon = 0x2636,
    LossOfControlAuraUpdate = 0x2695,
    MailCommandResult = 0x2658,
    MailListResult = 0x2798,
    MailQueryNextTimeResult = 0x2799,
    MapObjectivesInit = 0x27A1,
    MapObjEvents = 0x25D9,
    MasterLootCandidateList = 0x2633,
    MessageBox = 0x2575,
    MinimapPing = 0x26F7,
    MirrorImageComponentedData = 0x2C14,
    MirrorImageCreatureData = 0x2C13,
    MissileCancel = 0x25DA,
    ModifyChargeRecoverySpeed = 0x27A8,
    ModifyCooldown = 0x27A6,
    ModifyCooldownRecoverySpeed = 0x27A7,
    ModifyPartyRange = 0x2786,
    Motd = 0x2BAF,
    MountResult = 0x257A,
    MoveApplyMovementForce = 0x2DE1,
    MoveDisableCollision = 0x2DDD,
    MoveDisableDoubleJump = 0x2DCB,
    MoveDisableGravity = 0x2DDB,
    MoveDisableTransitionBetweenSwimAndFly = 0x2DDA,
    MoveEnableCollision = 0x2DDE,
    MoveEnableDoubleJump = 0x2DCA,
    MoveEnableGravity = 0x2DDC,
    MoveEnableTransitionBetweenSwimAndFly = 0x2DD9,
    MoveKnockBack = 0x2DD1,
    MoveRemoveMovementForce = 0x2DE2,
    MoveRoot = 0x2DC7,
    MoveSetActiveMover = 0x2DA3,
    MoveSetCanFly = 0x2DD3,
    MoveSetCanTurnWhileFalling = 0x2DD5,
    MoveSetCollisionHeight = 0x2DDF,
    MoveSetCompoundState = 0x2DE3,
    MoveSetFeatherFall = 0x2DCD,
    MoveSetFlightBackSpeed = 0x2DC3,
    MoveSetFlightSpeed = 0x2DC2,
    MoveSetHovering = 0x2DCF,
    MoveSetIgnoreMovementForces = 0x2DD7,
    MoveSetLandWalk = 0x2DCC,
    MoveSetNormalFall = 0x2DCE,
    MoveSetPitchRate = 0x2DC6,
    MoveSetRunBackSpeed = 0x2DBF,
    MoveSetRunSpeed = 0x2DBE,
    MoveSetSwimBackSpeed = 0x2DC1,
    MoveSetSwimSpeed = 0x2DC0,
    MoveSetTurnRate = 0x2DC5,
    MoveSetVehicleRecId = 0x2DE0,
    MoveSetWalkSpeed = 0x2DC4,
    MoveSetWaterWalk = 0x2DC9,
    MoveSkipTime = 0x2DE4,
    MoveSplineDisableCollision = 0x2DE9,
    MoveSplineDisableGravity = 0x2DE7,
    MoveSplineEnableCollision = 0x2DEA,
    MoveSplineEnableGravity = 0x2DE8,
    MoveSplineRoot = 0x2DE5,
    MoveSplineSetFeatherFall = 0x2DEB,
    MoveSplineSetFlightBackSpeed = 0x2DBA,
    MoveSplineSetFlightSpeed = 0x2DB9,
    MoveSplineSetFlying = 0x2DF5,
    MoveSplineSetHover = 0x2DED,
    MoveSplineSetLandWalk = 0x2DF0,
    MoveSplineSetNormalFall = 0x2DEC,
    MoveSplineSetPitchRate = 0x2DBD,
    MoveSplineSetRunBackSpeed = 0x2DB6,
    MoveSplineSetRunMode = 0x2DF3,
    MoveSplineSetRunSpeed = 0x2DB5,
    MoveSplineSetSwimBackSpeed = 0x2DB8,
    MoveSplineSetSwimSpeed = 0x2DB7,
    MoveSplineSetTurnRate = 0x2DBC,
    MoveSplineSetWalkMode = 0x2DF4,
    MoveSplineSetWalkSpeed = 0x2DBB,
    MoveSplineSetWaterWalk = 0x2DEF,
    MoveSplineStartSwim = 0x2DF1,
    MoveSplineStopSwim = 0x2DF2,
    MoveSplineUnroot = 0x2DE6,
    MoveSplineUnsetFlying = 0x2DF6,
    MoveSplineUnsetHover = 0x2DEE,
    MoveTeleport = 0x2DD2,
    MoveUnroot = 0x2DC8,
    MoveUnsetCanFly = 0x2DD4,
    MoveUnsetCanTurnWhileFalling = 0x2DD6,
    MoveUnsetHovering = 0x2DD0,
    MoveUnsetIgnoreMovementForces = 0x2DD8,
    MoveUpdate = 0x2DAE,
    MoveUpdateApplyMovementForce = 0x2DB2,
    MoveUpdateCollisionHeight = 0x2DAD,
    MoveUpdateFlightBackSpeed = 0x2DAA,
    MoveUpdateFlightSpeed = 0x2DA9,
    MoveUpdateKnockBack = 0x2DB0,
    MoveUpdatePitchRate = 0x2DAC,
    MoveUpdateRemoveMovementForce = 0x2DB3,
    MoveUpdateRunBackSpeed = 0x2DA5,
    MoveUpdateRunSpeed = 0x2DA4,
    MoveUpdateSwimBackSpeed = 0x2DA8,
    MoveUpdateSwimSpeed = 0x2DA7,
    MoveUpdateTeleport = 0x2DAF,
    MoveUpdateTurnRate = 0x2DAB,
    MoveUpdateWalkSpeed = 0x2DA6,
    NeutralPlayerFactionSelectResult = 0x25F1,
    NewTaxiPath = 0x26A7,
    NewWorld = 0x25AB,
    NotifyDestLocSpellCast = 0x2C43,
    NotifyMissileTrajectoryCollision = 0x26D1,
    NotifyMoney = 0x25AE,
    NotifyReceivedMail = 0x2659,
    OfferPetitionError = 0x26E0,
    OnCancelExpectedRideVehicleAura = 0x271C,
    OnMonsterMove = 0x2DA2,
    OpenAlliedRaceDetailsGiver = 0x2837,
    OpenContainer = 0x2764,
    OpenLfgDungeonFinder = 0x2A31,
    OpenShipmentNpcFromGossip = 0x27DA,
    OpenShipmentNpcResult = 0x27DC,
    OpenTransmogrifier = 0x2836,
    OverrideLight = 0x26E6,
    PageText = 0x2759,
    PartyCommandResult = 0x27D7,
    PartyInvite = 0x25CF,
    PartyKillLog = 0x279D,
    PartyMemberState = 0x279B,
    PartyMemberStateUpdate = 0x279A,
    PartyUpdate = 0x260B,
    PauseMirrorTimer = 0x274F,
    PendingRaidLock = 0x2730,
    PetitionAlreadySigned = 0x25B8,
    PetitionRenameGuildResponse = 0x29F7,
    PetitionShowList = 0x26E9,
    PetitionShowSignatures = 0x26EA,
    PetitionSignResults = 0x278F,
    PetActionFeedback = 0x278D,
    PetActionSound = 0x26C9,
    PetAdded = 0x25A8,
    PetBattleChatRestricted = 0x2618,
    PetBattleDebugQueueDumpResponse = 0x269C,
    PetBattleFinalizeLocation = 0x2611,
    PetBattleFinalRound = 0x2616,
    PetBattleFinished = 0x2617,
    PetBattleFirstRound = 0x2613,
    PetBattleInitialUpdate = 0x2612,
    PetBattleMaxGameLengthWarning = 0x2619,
    PetBattlePvpChallenge = 0x2610,
    PetBattleQueueProposeMatch = 0x2656,
    PetBattleQueueStatus = 0x2657,
    PetBattleReplacementsMade = 0x2615,
    PetBattleRequestFailed = 0x260F,
    PetBattleRoundResult = 0x2614,
    PetBattleSlotUpdates = 0x2602,
    PetCastFailed = 0x2C57,
    PetClearSpells = 0x2C24,
    PetDismissSound = 0x26CA,
    PetGodMode = 0x26A4,
    PetGuids = 0x2741,
    PetLearnedSpells = 0x2C4F,
    PetMode = 0x2589,
    PetNameInvalid = 0x26EE,
    PetSlotUpdated = 0x2588,
    PetSpellsMessage = 0x2C25,
    PetStableList = 0x25A9,
    PetStableResult = 0x25AA,
    PetTameFailure = 0x26DD,
    PetUnlearnedSpells = 0x2C50,
    PhaseShiftChange = 0x2577,
    PlayedTime = 0x2708,
    PlayerBound = 0x257D,
    PlayerSaveGuildEmblem = 0x29F6,
    PlayerSkinned = 0x2788,
    PlayerTabardVendorActivate = 0x279C,
    PlayMusic = 0x27AB,
    PlayObjectSound = 0x27AC,
    PlayOneShotAnimKit = 0x2772,
    PlayOrphanSpellVisual = 0x2C47,
    PlayScene = 0x2653,
    PlaySound = 0x27AA,
    PlaySpeakerbotSound = 0x27AD,
    PlaySpellVisual = 0x2C45,
    PlaySpellVisualKit = 0x2C49,
    PlayTimeWarning = 0x273A,
    Pong = 0x304E,
    PowerUpdate = 0x26FD,
    PrestigeAndHonorInvoluntarilyChanged = 0x2758,
    PreRessurect = 0x27A9,
    PrintNotification = 0x25E1,
    ProcResist = 0x279E,
    ProposeLevelGrant = 0x2710,
    PushSpellToActionBar = 0x2C51,
    PvpCredit = 0x2716,
    PvpLogData = 0x25B3,
    PvpOptionsEnabled = 0x25B6,
    PvpSeason = 0x25D3,
    QueryBattlePetNameResponse = 0x2703,
    QueryCreatureResponse = 0x26FA,
    QueryGameObjectResponse = 0x26FB,
    QueryGarrisonCreatureNameResponse = 0x292B,
    QueryGuildInfoResponse = 0x29E5,
    QueryItemTextResponse = 0x280A,
    QueryNpcTextResponse = 0x26FE,
    QueryPageTextResponse = 0x2700,
    QueryPetitionResponse = 0x2704,
    QueryPetNameResponse = 0x2702,
    QueryPlayerNameResponse = 0x26FF,
    QueryQuestInfoResponse = 0x2A95,
    QueryQuestRewardResponse = 0x2846,
    QueryTimeResponse = 0x271A,
    QuestCompletionNpcResponse = 0x2A81,
    QuestConfirmAccept = 0x2A8E,
    QuestForceRemoved = 0x2A9B,
    QuestGiverInvalidQuest = 0x2A84,
    QuestGiverOfferRewardMessage = 0x2A93,
    QuestGiverQuestComplete = 0x2A82,
    QuestGiverQuestDetails = 0x2A91,
    QuestGiverQuestFailed = 0x2A85,
    QuestGiverQuestListMessage = 0x2A99,
    QuestGiverQuestTurnInFailure = 0x2852,
    QuestGiverRequestItems = 0x2A92,
    QuestGiverStatus = 0x2A9A,
    QuestGiverStatusMultiple = 0x2A90,
    QuestLogFull = 0x2A86,
    QuestPoiChanged = 0x2A9F,
    QuestPoiQueryResponse = 0x2A9C,
    QuestPushResult = 0x2A8F,
    QuestSpawnTrackingUpdate = 0x2A9E,
    QuestUpdateAddCredit = 0x2A8B,
    QuestUpdateAddCreditSimple = 0x2A8C,
    QuestUpdateAddPvpCredit = 0x2A8D,
    QuestUpdateComplete = 0x2A88,
    QuestUpdateCompleteBySpell = 0x2A87,
    QuestUpdateFailed = 0x2A89,
    QuestUpdateFailedTimer = 0x2A8A,
    RafEmailEnabledResponse = 0x27C8,
    RaidDifficultySet = 0x27EF,
    RaidGroupOnly = 0x27F1,
    RaidInstanceMessage = 0x2BB4,
    RaidMarkersChanged = 0x25B9,
    RandomRoll = 0x264C,
    RatedBattlefieldInfo = 0x25A6,
    ReadyCheckCompleted = 0x260E,
    ReadyCheckResponse = 0x260D,
    ReadyCheckStarted = 0x260C,
    ReadItemResultFailed = 0x27EB,
    ReadItemResultOk = 0x27E1,
    RealmLookupInformation = 0x2810,
    RealmQueryResponse = 0x26E5,
    RecruitAFriendResponse = 0x27C9,
    ReferAFriendExpired = 0x2762,
    ReferAFriendFailure = 0x26EB,
    RefreshComponent = 0x2678,
    RefreshSpellHistory = 0x2C2C,
    RemoveItemPassive = 0x25C0,
    RemoveLossOfControl = 0x2697,
    ReplaceTrophyResponse = 0x2807,
    ReportPvpPlayerAfkResult = 0x26D9,
    RequestAddonList = 0x265F,
    RequestCemeteryListResponse = 0x259D,
    RequestPvpBrawlInfoResponse = 0x25D5,
    RequestPvpRewardsResponse = 0x25D4,
    ResearchComplete = 0x2585,
    ResetAreaTrigger = 0x2640,
    ResetCompressionContext = 0x304F,
    ResetFailedNotify = 0x26E1,
    ResetRangedCombatTimer = 0x2713,
    ResetWeeklyCurrency = 0x2574,
    RespecWipeConfirm = 0x2626,
    RespondInspectAchievements = 0x2571,
    ResumeCastBar = 0x2C3E,
    ResumeComms = 0x304B,
    ResumeToken = 0x25BE,
    ResurrectRequest = 0x257E,
    ResyncRunes = 0x273D,
    RoleChangedInform = 0x258C,
    RoleChosen = 0x2A39,
    RolePollInform = 0x258D,
    RuneRegenDebug = 0x25C8,
    ScenarioBoot = 0x27EC,
    ScenarioCompleted = 0x282C,
    ScenarioPois = 0x264F,
    ScenarioProgressUpdate = 0x2648,
    ScenarioSetShouldShowCriteria = 0x283A,
    ScenarioSpellUpdate = 0x2839,
    ScenarioState = 0x2647,
    SceneObjectEvent = 0x25F7,
    SceneObjectPetBattleFinalRound = 0x25FC,
    SceneObjectPetBattleFinished = 0x25FD,
    SceneObjectPetBattleFirstRound = 0x25F9,
    SceneObjectPetBattleInitialUpdate = 0x25F8,
    SceneObjectPetBattleReplacementsMade = 0x25FB,
    SceneObjectPetBattleRoundResult = 0x25FA,
    ScriptCast = 0x2C55,
    SellResponse = 0x26EF,
    SendItemPassives = 0x25C1,
    SendKnownSpells = 0x2C2A,
    SendRaidTargetUpdateAll = 0x264A,
    SendRaidTargetUpdateSingle = 0x264B,
    SendSpellCharges = 0x2C2D,
    SendSpellHistory = 0x2C2B,
    SendUnlearnSpells = 0x2C2E,
    ServerFirstAchievement = 0x2BBC,
    ServerFirstAchievements = 0x266A,
    ServerTime = 0x26AB,
    SetupCurrency = 0x2572,
    SetupResearchHistory = 0x2584,
    SetAiAnimKit = 0x2771,
    SetAllTaskProgress = 0x27D1,
    SetAnimTier = 0x2775,
    SetCurrency = 0x2573,
    SetDfFastLaunchResult = 0x2A2E,
    SetDungeonDifficulty = 0x26CD,
    SetFactionAtWar = 0x273C,
    SetFactionNotVisible = 0x276C,
    SetFactionStanding = 0x276D,
    SetFactionVisible = 0x276B,
    SetFlatSpellModifier = 0x2C36,
    SetForcedReactions = 0x275C,
    SetItemPurchaseData = 0x25B0,
    SetLootMethodFailed = 0x2816,
    SetMaxWeeklyQuantity = 0x25B7,
    SetMeleeAnimKit = 0x2774,
    SetMovementAnimKit = 0x2773,
    SetPctSpellModifier = 0x2C37,
    SetPetSpecialization = 0x2641,
    SetPlayerDeclinedNamesResult = 0x2707,
    SetPlayHoverAnim = 0x25CC,
    SetProficiency = 0x2776,
    SetSpellCharges = 0x2C29,
    SetTaskComplete = 0x27D2,
    SetTimeZoneInformation = 0x269F,
    SetVehicleRecId = 0x272F,
    ShowAdventureMap = 0x2835,
    ShowBank = 0x26A8,
    ShowMailbox = 0x27ED,
    ShowNeutralPlayerFactionSelectUi = 0x25F0,
    ShowTaxiNodes = 0x26F6,
    ShowTradeSkillResponse = 0x27B2,
    SocketGems = 0x2768,
    SocketGemsFailure = 0x2769,
    SortBagsResult = 0x2824,
    SorStartExperienceIncomplete = 0x25F2,
    SpecializationChanged = 0x25EC,
    SpecialMountAnim = 0x26C8,
    SpecInvoluntarilyChanged = 0x2757,
    SpellAbsorbLog = 0x2C1F,
    SpellCategoryCooldown = 0x2C17,
    SpellChannelStart = 0x2C34,
    SpellChannelUpdate = 0x2C35,
    SpellCooldown = 0x2C15,
    SpellDamageShield = 0x2C31,
    SpellDelayed = 0x2C3F,
    SpellDispellLog = 0x2C1A,
    SpellEnergizeLog = 0x2C1C,
    SpellExecuteLog = 0x2C40,
    SpellFailedOther = 0x2C54,
    SpellFailure = 0x2C52,
    SpellFailureMessage = 0x2C59,
    SpellGo = 0x2C39,
    SpellHealAbsorbLog = 0x2C1E,
    SpellHealLog = 0x2C1D,
    SpellInstakillLog = 0x2C33,
    SpellInterruptLog = 0x2C20,
    SpellMissLog = 0x2C41,
    SpellNonMeleeDamageLog = 0x2C32,
    SpellOrDamageImmune = 0x2C2F,
    SpellPeriodicAuraLog = 0x2C1B,
    SpellPrepare = 0x2C38,
    SpellStart = 0x2C3A,
    SpiritHealerConfirm = 0x2754,
    StandStateUpdate = 0x275B,
    StartElapsedTimer = 0x261A,
    StartElapsedTimers = 0x261C,
    StartLootRoll = 0x2631,
    StartMirrorTimer = 0x274E,
    StartTimer = 0x25BB,
    StopElapsedTimer = 0x261B,
    StopMirrorTimer = 0x2750,
    StopSpeakerbotSound = 0x27AE,
    StreamingMovies = 0x25BA,
    SummonCancel = 0x26D8,
    SummonRaidMemberValidateFailed = 0x258E,
    SummonRequest = 0x2760,
    SupercededSpells = 0x2C4C,
    SuspendComms = 0x304A,
    SuspendToken = 0x25BD,
    TalentsInvoluntarilyReset = 0x2756,
    TaxiNodeStatus = 0x26A5,
    TextEmote = 0x26A3,
    ThreatClear = 0x270F,
    ThreatRemove = 0x270E,
    ThreatUpdate = 0x270D,
    TimeAdjustment = 0x2DA1,
    TimeSyncRequest = 0x2DA0,
    TitleEarned = 0x270A,
    TitleLost = 0x270B,
    TotemCreated = 0x26F2,
    TotemMoved = 0x26F3,
    TradeStatus = 0x2581,
    TradeUpdated = 0x2580,
    TrainerBuyFailed = 0x2715,
    TrainerList = 0x2714,
    TransferAborted = 0x2740,
    TransferPending = 0x25E5,
    TransmogCollectionUpdate = 0x25C6,
    TransmogSetCollectionUpdate = 0x25C7,
    TriggerCinematic = 0x280E,
    TriggerMovie = 0x26F4,
    TurnInPetitionResult = 0x2791,
    TutorialFlags = 0x2800,
    TutorialHighlightSpell = 0x2840,
    TutorialUnhighlightSpell = 0x283F,
    TwitterStatus = 0x2FFD,
    UiTime = 0x2753,
    UndeleteCharacterResponse = 0x2811,
    UndeleteCooldownStatusResponse = 0x2812,
    UnlearnedSpells = 0x2C4E,
    UpdateAccountData = 0x2748,
    UpdateActionButtons = 0x25F5,
    UpdateCelestialBody = 0x285E,
    UpdateCharacterFlags = 0x2806,
    UpdateExpansionLevel = 0x2663,
    UpdateGameTimeState = 0x2865,
    UpdateInstanceOwnership = 0x26D0,
    UpdateLastInstance = 0x26B1,
    UpdateObject = 0x280F,
    UpdateTalentData = 0x25EB,
    UpdateTaskProgress = 0x27D0,
    UpdateWeeklySpellUsage = 0x2C19,
    UpdateWorldState = 0x278C,
    UserlistAdd = 0x2BB9,
    UserlistRemove = 0x2BBA,
    UserlistUpdate = 0x2BBB,
    UseEquipmentSetResult = 0x2792,
    VendorInventory = 0x25CA,
    VignetteUpdate = 0x27B0,
    VoidItemSwapResponse = 0x25DF,
    VoidStorageContents = 0x25DC,
    VoidStorageFailed = 0x25DB,
    VoidStorageTransferChanges = 0x25DD,
    VoidTransferResult = 0x25DE,
    WaitQueueFinish = 0x256E,
    WaitQueueUpdate = 0x256D,
    WardenData = 0x2576,
    WargameRequestSuccessfullySentToOpponent = 0x25B4,
    Weather = 0x26CF,
    WeeklySpellUsage = 0x2C18,
    Who = 0x2BAE,
    WhoIs = 0x26CE,
    WorldQuestUpdate = 0x2847,
    WorldServerInfo = 0x25C2,
    WorldText = 0x282E,
    WowTokenAuctionSold = 0x281C,
    WowTokenBuyRequestConfirmation = 0x281E,
    WowTokenBuyResultConfirmation = 0x281F,
    WowTokenCanRedeemForBalanceResult = 0x2856,
    WowTokenCanVeteranBuyResult = 0x281D,
    WowTokenDistributionGlueUpdate = 0x2817,
    WowTokenDistributionUpdate = 0x2818,
    WowTokenMarketPriceResponse = 0x2819,
    WowTokenRedeemGameTimeUpdated = 0x2820,
    WowTokenRedeemRequestConfirmation = 0x2821,
    WowTokenRedeemResult = 0x2822,
    WowTokenSellRequestConfirmation = 0x281A,
    WowTokenSellResultConfirmation = 0x281B,
    WowTokenUpdateAuctionableListResponse = 0x2823,
    XpGainAborted = 0x25E0,
    XpGainEnabled = 0x27F0,
    ZoneUnderAttack = 0x2BB5,
    // Opcodes that are not generated automatically
    // SMSG_ACCOUNT_HEIRLOOM_UPDATE                      = 0xBADD, // no client handler
    // SMSG_ITEM_UPGRADE_RESULT                          = 0xBADD, // no client handler
    CompressedPacket = 0x3052,
    MultiplePackets = 0x3051,
    // Deleted opcodes, here only to allow compile
    // SMSG_ARENA_TEAM_STATS                             = 0xBADD,
    // SMSG_BUY_BANK_SLOT_RESULT                         = 0xBADD,
    // SMSG_BF_MGR_EJECTED                               = 0xBADD,
    // SMSG_BF_MGR_ENTERING                              = 0xBADD,
    // SMSG_BF_MGR_ENTRY_INVITE                          = 0xBADD,
    // SMSG_BF_MGR_QUEUE_INVITE                          = 0xBADD,
    // SMSG_BF_MGR_QUEUE_REQUEST_RESPONSE                = 0xBADD,
}

pub type SMSG = OpcodeServer;

pub struct ServerOpcodeInfo {
    pub status:          SessionStatus,
    pub connection_type: ConnectionType,
}

macro_rules! define_server_opcode_handler {
    ( $this:expr; $( $cmsg:ident, $status:ident, $conn_typ:ident );+ $(;)? ) => {{
        const _: () = {
            $(
                assert!(
                    !SMSG::$cmsg.instance_only() || matches!(ConnectionType::$conn_typ, ConnectionType::Instance),
                    concat!("instance only opcode ", stringify!($cmsg), " must use ConnectionType::Instance"),
                );
            )+
        };
        const _: () = {
            $(
                assert!(
                    matches!(SessionStatus::$status, SessionStatus::Never | SessionStatus::Unhandled),
                    concat!("server opcode ", stringify!($cmsg), " only accepts SessionStatus::Never or SessionStatus::Unhandled"),
                );
            )+
        };
        match $this {
            $(
                SMSG::$cmsg => ServerOpcodeInfo { status: SessionStatus::$status, connection_type: ConnectionType::$conn_typ },
            )+
        }
    }};
}

impl OpcodeServer {
    /// IsInstanceOnlyOpcode in TC
    pub const fn instance_only(&self) -> bool {
        match *self {
            SMSG::QuestGiverStatus | // ClientQuest
            SMSG::DuelRequested | // Client
            SMSG::DuelInBounds | // Client
            SMSG::QueryTimeResponse | // Client
            SMSG::DuelWinner | // Client
            SMSG::DuelComplete | // Client
            SMSG::DuelOutOfBounds | // Client
            SMSG::AttackStop | // Client
            SMSG::AttackStart | // Client
            SMSG::MountResult // Client
            => true,
            _ => false,
        }
    }

    pub const fn info(&self) -> ServerOpcodeInfo {
        define_server_opcode_handler!(
        *self;
        AbortNewWorld, Unhandled, Realm;
        AccountCriteriaUpdate, Unhandled, Realm;
        AccountDataTimes, Never, Realm;
        // AccountHeirloomUpdate, Never, Instance;
        AccountMountUpdate, Never, Instance;
        AccountToysUpdate, Never, Instance;
        AchievementDeleted, Never, Instance;
        AchievementEarned, Never, Instance;
        ActivateTaxiReply, Never, Realm;
        ActiveGlyphs, Never, Realm;
        AddBattlenetFriendResponse, Unhandled, Realm;
        AddItemPassive, Unhandled, Realm;
        AddLossOfControl, Unhandled, Realm;
        AddRunePower, Never, Instance;
        AdjustSplineDuration, Unhandled, Realm;
        AeLootTargets, Never, Instance;
        AeLootTargetAck, Never, Instance;
        AiReaction, Never, Instance;
        AllAccountCriteria, Unhandled, Instance;
        AllAchievementData, Never, Instance;
        AllGuildAchievements, Never, Realm;
        ArchaeologySurveryCast, Unhandled, Realm;
        AreaPoiUpdate, Unhandled, Instance;
        AreaSpiritHealerTime, Never, Realm;
        AreaTriggerDenied, Never, Realm;
        AreaTriggerNoCorpse, Never, Realm;
        AreaTriggerRePath, Never, Realm;
        AreaTriggerReShape, Never, Realm;
        ArenaCrowdControlSpells, Unhandled, Instance;
        ArenaError, Unhandled, Realm;
        ArenaPrepOpponentSpecializations, Unhandled, Realm;
        ArtifactForgeOpened, Never, Realm;
        ArtifactKnowledge, Never, Realm;
        ArtifactRespecConfirm, Unhandled, Realm;
        ArtifactTraitsRefunded, Unhandled, Realm;
        ArtifactXpGain, Never, Realm;
        AttackerStateUpdate, Never, Instance;
        AttackStart, Never, Instance;
        AttackStop, Never, Instance;
        AttackSwingError, Never, Realm;
        AttackSwingLandedLog, Unhandled, Realm;
        AuctionClosedNotification, Never, Realm;
        AuctionCommandResult, Never, Realm;
        AuctionHelloResponse, Never, Realm;
        AuctionListBidderItemsResult, Never, Realm;
        AuctionListItemsResult, Never, Realm;
        AuctionListOwnerItemsResult, Never, Realm;
        AuctionListPendingSalesResult, Never, Realm;
        AuctionOutbidNotification, Never, Realm;
        AuctionOwnerBidNotification, Never, Realm;
        AuctionReplicateResponse, Never, Realm;
        AuctionWonNotification, Never, Realm;
        AuraPointsDepleted, Unhandled, Realm;
        AuraUpdate, Never, Instance;
        AuthChallenge, Never, Realm;
        AuthResponse, Never, Realm;
        AvailableHotfixes, Never, Realm;
        BanReason, Unhandled, Realm;
        BarberShopResult, Never, Realm;
        BattlefieldList, Never, Realm;
        BattlefieldPortDenied, Unhandled, Realm;
        BattlefieldStatusActive, Never, Realm;
        BattlefieldStatusFailed, Never, Realm;
        BattlefieldStatusNeedConfirmation, Never, Realm;
        BattlefieldStatusNone, Never, Realm;
        BattlefieldStatusQueued, Never, Realm;
        BattlefieldStatusWaitForGroups, Unhandled, Realm;
        BattlegroundInfoThrottled, Unhandled, Realm;
        BattlegroundInit, Unhandled, Realm;
        BattlegroundPlayerJoined, Never, Instance;
        BattlegroundPlayerLeft, Never, Instance;
        BattlegroundPlayerPositions, Never, Instance;
        BattlegroundPoints, Unhandled, Realm;
        BattlenetChallengeAbort, Unhandled, Realm;
        BattlenetChallengeStart, Unhandled, Realm;
        BattlenetNotification, Never, Realm;
        BattlenetRealmListTicket, Never, Realm;
        BattlenetResponse, Never, Realm;
        BattlenetSetSessionState, Never, Realm;
        BattlePayAckFailed, Unhandled, Realm;
        BattlePayBattlePetDelivered, Unhandled, Realm;
        BattlePayConfirmPurchase, Unhandled, Realm;
        BattlePayDeliveryEnded, Unhandled, Realm;
        BattlePayDeliveryStarted, Unhandled, Realm;
        BattlePayDistributionUpdate, Unhandled, Realm;
        BattlePayGetDistributionListResponse, Unhandled, Realm;
        BattlePayGetProductListResponse, Unhandled, Realm;
        BattlePayGetPurchaseListResponse, Unhandled, Realm;
        BattlePayMountDelivered, Unhandled, Realm;
        BattlePayPurchaseUpdate, Unhandled, Realm;
        BattlePayStartDistributionAssignToTargetResponse, Unhandled, Realm;
        BattlePayStartPurchaseResponse, Unhandled, Realm;
        BattlePaySubscriptionChanged, Unhandled, Realm;
        BattlePayVasBnetTransferValidationResult, Unhandled, Realm;
        BattlePayVasBoostConsumed, Unhandled, Realm;
        BattlePayVasCharacterList, Unhandled, Realm;
        BattlePayVasCharacterQueueStatus, Unhandled, Realm;
        BattlePayVasPurchaseComplete, Unhandled, Realm;
        BattlePayVasPurchaseList, Unhandled, Realm;
        BattlePayVasPurchaseStarted, Unhandled, Realm;
        BattlePayVasRealmList, Unhandled, Realm;
        BattlePayVasTransferQueueStatus, Unhandled, Realm;
        BattlePetsHealed, Unhandled, Realm;
        BattlePetCageDateError, Unhandled, Realm;
        BattlePetDeleted, Never, Realm;
        BattlePetError, Unhandled, Realm;
        BattlePetJournal, Never, Realm;
        BattlePetJournalLockAcquired, Never, Realm;
        BattlePetJournalLockDenied, Unhandled, Realm;
        BattlePetLicenseChanged, Unhandled, Realm;
        BattlePetMaxCountChanged, Unhandled, Realm;
        BattlePetRestored, Unhandled, Realm;
        BattlePetRevoked, Unhandled, Realm;
        BattlePetTrapLevel, Unhandled, Realm;
        BattlePetUpdates, Never, Realm;
        //BfMgrEjected, Never, Instance;
        //BfMgrEntering, Never, Instance;
        //BfMgrEntryInvite, Never, Realm;
        //BfMgrQueueInvite, Never, Realm;
        //BfMgrQueueRequestResponse, Never, Realm;
        BinderConfirm, Never, Realm;
        BindPointUpdate, Never, Instance;
        BlackMarketBidOnItemResult, Never, Realm;
        BlackMarketOpenResult, Never, Realm;
        BlackMarketOutbid, Never, Realm;
        BlackMarketRequestItemsResult, Never, Realm;
        BlackMarketWon, Never, Realm;
        BonusRollEmpty, Unhandled, Realm;
        BossKillCredit, Never, Instance;
        BreakTarget, Never, Realm;
        BuyFailed, Never, Realm;
        BuySucceeded, Never, Realm;
        CacheInfo, Never, Realm;
        CacheVersion, Never, Realm;
        CalendarClearPendingAction, Never, Realm;
        CalendarCommandResult, Never, Realm;
        CalendarEventInitialInvites, Never, Realm;
        CalendarEventInvite, Never, Realm;
        CalendarEventInviteAlert, Never, Realm;
        CalendarEventInviteModeratorStatus, Never, Realm;
        CalendarEventInviteNotes, Never, Realm;
        CalendarEventInviteNotesAlert, Never, Realm;
        CalendarEventInviteRemoved, Never, Realm;
        CalendarEventInviteRemovedAlert, Never, Realm;
        CalendarEventInviteStatus, Never, Realm;
        CalendarEventInviteStatusAlert, Never, Realm;
        CalendarEventRemovedAlert, Never, Realm;
        CalendarEventUpdatedAlert, Never, Realm;
        CalendarRaidLockoutAdded, Never, Realm;
        CalendarRaidLockoutRemoved, Never, Realm;
        CalendarRaidLockoutUpdated, Never, Realm;
        CalendarSendCalendar, Never, Realm;
        CalendarSendEvent, Never, Realm;
        CalendarSendNumPending, Never, Realm;
        CameraEffect, Unhandled, Realm;
        CancelAutoRepeat, Never, Realm;
        CancelCombat, Never, Realm;
        CancelOrphanSpellVisual, Never, Realm;
        CancelScene, Never, Instance;
        CancelSpellVisual, Never, Realm;
        CancelSpellVisualKit, Never, Realm;
        CanDuelResult, Never, Realm;
        CastFailed, Never, Instance;
        CategoryCooldown, Never, Instance;
        ChallengeModeAllMapStats, Unhandled, Realm;
        ChallengeModeComplete, Unhandled, Realm;
        ChallengeModeMapStatsUpdate, Unhandled, Realm;
        ChallengeModeNewPlayerRecord, Unhandled, Realm;
        ChallengeModeRequestLeadersResult, Unhandled, Realm;
        ChallengeModeRewards, Unhandled, Realm;
        ChallengeModeReset, Unhandled, Realm;
        ChallengeModeStart, Unhandled, Realm;
        ChallengeModeUpdateDeathCount, Unhandled, Realm;
        ChangePlayerDifficultyResult, Unhandled, Realm;
        ChannelList, Never, Realm;
        ChannelNotify, Never, Realm;
        ChannelNotifyJoined, Never, Realm;
        ChannelNotifyLeft, Never, Realm;
        CharacterClassTrialCreate, Unhandled, Realm;
        CharacterInventoryOverflowWarning, Never, Realm;
        CharacterItemFixup, Unhandled, Realm;
        CharacterLoginFailed, Never, Realm;
        CharacterObjectTestResponse, Unhandled, Realm;
        CharacterRenameResult, Never, Realm;
        CharacterUpgradeComplete, Unhandled, Realm;
        CharacterUpgradeQueued, Unhandled, Realm;
        CharacterUpgradeSpellTierSet, Unhandled, Realm;
        CharacterUpgradeStarted, Unhandled, Realm;
        CharacterUpgradeUnrevokeResult, Unhandled, Realm;
        CharCustomize, Never, Realm;
        CharCustomizeFailed, Never, Realm;
        CharFactionChangeResult, Never, Realm;
        Chat, Never, Realm;
        ChatAutoResponded, Unhandled, Realm;
        ChatDown, Unhandled, Realm;
        ChatIgnoredAccountMuted, Unhandled, Realm;
        ChatIsDown, Unhandled, Realm;
        ChatNotInParty, Unhandled, Realm;
        ChatPlayerAmbiguous, Never, Realm;
        ChatPlayerNotfound, Never, Realm;
        ChatReconnect, Unhandled, Realm;
        ChatRestricted, Never, Realm;
        ChatServerMessage, Never, Realm;
        CheatIgnoreDimishingReturns, Unhandled, Realm;
        CheckWargameEntry, Unhandled, Realm;
        ClearAllSpellCharges, Never, Instance;
        ClearBossEmotes, Unhandled, Realm;
        ClearCooldown, Never, Instance;
        ClearCooldowns, Never, Instance;
        ClearLossOfControl, Unhandled, Realm;
        ClearSpellCharges, Never, Instance;
        ClearTarget, Never, Realm;
        CoinRemoved, Never, Realm;
        CombatEventFailed, Unhandled, Realm;
        CommentatorMapInfo, Unhandled, Realm;
        CommentatorPlayerInfo, Unhandled, Realm;
        CommentatorStateChanged, Unhandled, Realm;
        ComplaintResult, Never, Realm;
        CompleteShipmentResponse, Unhandled, Realm;
        ConnectTo, Never, Realm;
        ConquestFormulaConstants, Unhandled, Instance;
        ConsoleWrite, Unhandled, Realm;
        ContactList, Never, Realm;
        ControlUpdate, Never, Realm;
        CooldownCheat, Unhandled, Realm;
        CooldownEvent, Never, Instance;
        CorpseLocation, Never, Realm;
        CorpseReclaimDelay, Never, Instance;
        CorpseTransportQuery, Never, Realm;
        CreateChar, Never, Realm;
        CreateShipmentResponse, Unhandled, Realm;
        CriteriaDeleted, Never, Instance;
        CriteriaUpdate, Never, Instance;
        CrossedInebriationThreshold, Never, Realm;
        CustomLoadScreen, Never, Realm;
        DailyQuestsReset, Never, Realm;
        DamageCalcLog, Unhandled, Realm;
        DbReply, Never, Realm;
        DeathReleaseLoc, Never, Realm;
        DefenseMessage, Never, Realm;
        DeleteChar, Never, Realm;
        DestroyArenaUnit, Never, Realm;
        DestructibleBuildingDamage, Never, Instance;
        DifferentInstanceFromParty, Unhandled, Realm;
        DisenchantCredit, Unhandled, Realm;
        Dismount, Never, Realm;
        DismountResult, Unhandled, Realm;
        DispelFailed, Never, Instance;
        DisplayGameError, Never, Realm;
        DisplayPlayerChoice, Never, Realm;
        DisplayPromotion, Unhandled, Realm;
        DisplayQuestPopup, Unhandled, Realm;
        DisplayToast, Unhandled, Realm;
        DontAutoPushSpellsToActionBar, Unhandled, Realm;
        DropNewConnection, Unhandled, Realm;
        DuelComplete, Never, Instance;
        DuelCountdown, Never, Realm;
        DuelInBounds, Never, Instance;
        DuelOutOfBounds, Never, Instance;
        DuelRequested, Never, Instance;
        DuelWinner, Never, Instance;
        DurabilityDamageDeath, Never, Realm;
        Emote, Never, Instance;
        EnableBarberShop, Never, Realm;
        EnableEncryption, Unhandled, Realm;
        EnchantmentLog, Unhandled, Realm;
        EncounterEnd, Unhandled, Realm;
        EncounterStart, Unhandled, Realm;
        EnumCharactersResult, Never, Realm;
        EnvironmentalDamageLog, Never, Realm;
        EquipmentSetId, Never, Instance;
        ExpectedSpamRecords, Unhandled, Realm;
        ExplorationExperience, Never, Realm;
        FactionBonusInfo, Unhandled, Instance;
        FailedPlayerCondition, Unhandled, Realm;
        FeatureSystemStatus, Never, Realm;
        FeatureSystemStatusGlueScreen, Never, Realm;
        FeignDeathResisted, Unhandled, Realm;
        FishEscaped, Never, Realm;
        FishNotHooked, Never, Realm;
        FlightSplineSync, Unhandled, Realm;
        ForcedDeathUpdate, Unhandled, Realm;
        ForceAnim, Unhandled, Realm;
        ForceObjectRelink, Unhandled, Realm;
        FriendStatus, Never, Realm;
        GameObjectActivateAnimKit, Never, Instance;
        GameObjectCustomAnim, Never, Instance;
        GameObjectDespawn, Never, Realm;
        GameObjectPlaySpellVisual, Unhandled, Realm;
        GameObjectPlaySpellVisualKit, Unhandled, Realm;
        GameObjectResetState, Unhandled, Realm;
        GameObjectSetState, Unhandled, Realm;
        GameObjectUiAction, Never, Instance;
        GameSpeedSet, Unhandled, Realm;
        GameTimeSet, Unhandled, Realm;
        GameTimeUpdate, Unhandled, Realm;
        GarrisonAddFollowerResult, Never, Instance;
        GarrisonAddMissionResult, Unhandled, Instance;
        GarrisonAssignFollowerToBuildingResult, Unhandled, Instance;
        GarrisonBuildingActivated, Never, Instance;
        GarrisonBuildingLandmarks, Never, Instance;
        GarrisonBuildingRemoved, Never, Instance;
        GarrisonBuildingSetActiveSpecializationResult, Unhandled, Instance;
        GarrisonClearAllFollowersExhaustion, Unhandled, Instance;
        GarrisonCompleteMissionResult, Unhandled, Instance;
        GarrisonCreateResult, Never, Instance;
        GarrisonDeleteResult, Never, Instance;
        GarrisonFollowerChangedAbilities, Unhandled, Instance;
        GarrisonFollowerChangedDurability, Unhandled, Instance;
        GarrisonFollowerChangedItemLevel, Unhandled, Instance;
        GarrisonFollowerChangedStatus, Unhandled, Instance;
        GarrisonFollowerChangedXp, Unhandled, Instance;
        GarrisonIsUpgradeableResult, Unhandled, Instance;
        GarrisonLandingPageShipmentInfo, Unhandled, Instance;
        GarrisonLearnBlueprintResult, Never, Instance;
        GarrisonLearnSpecializationResult, Unhandled, Instance;
        GarrisonListFollowersCheatResult, Unhandled, Instance;
        GarrisonListMissionsCheatResult, Unhandled, Instance;
        GarrisonMissionAreaBonusAdded, Unhandled, Instance;
        GarrisonMissionBonusRollResult, Unhandled, Instance;
        GarrisonMissionUpdateCanStart, Unhandled, Instance;
        GarrisonNumFollowerActivationsRemaining, Unhandled, Instance;
        GarrisonOpenArchitect, Unhandled, Instance;
        GarrisonOpenMissionNpc, Unhandled, Instance;
        GarrisonOpenRecruitmentNpc, Unhandled, Instance;
        GarrisonOpenTradeskillNpc, Unhandled, Instance;
        GarrisonPlaceBuildingResult, Never, Instance;
        GarrisonPlotPlaced, Never, Instance;
        GarrisonPlotRemoved, Never, Instance;
        GarrisonRecallPortalLastUsedTime, Unhandled, Instance;
        GarrisonRecallPortalUsed, Unhandled, Instance;
        GarrisonRecruitmentFollowersGenerated, Unhandled, Instance;
        GarrisonRecruitFollowerResult, Unhandled, Instance;
        GarrisonRemoteInfo, Never, Instance;
        GarrisonRemoveFollowerFromBuildingResult, Unhandled, Instance;
        GarrisonRemoveFollowerResult, Never, Instance;
        GarrisonRequestBlueprintAndSpecializationDataResult, Never, Instance;
        GarrisonStartMissionResult, Unhandled, Instance;
        GarrisonUnlearnBlueprintResult, Never, Instance;
        GarrisonUpgradeResult, Unhandled, Instance;
        GenerateRandomCharacterNameResult, Never, Realm;
        GetAccountCharacterListResult, Unhandled, Realm;
        GetDisplayedTrophyListResponse, Unhandled, Instance;
        GetGarrisonInfoResult, Never, Instance;
        GetShipmentsOfTypeResponse, Unhandled, Instance;
        GetShipmentInfoResponse, Unhandled, Instance;
        GetTrophyListResponse, Unhandled, Instance;
        GmPlayerInfo, Unhandled, Realm;
        GmRequestPlayerInfo, Unhandled, Realm;
        GmTicketCaseStatus, Never, Realm;
        GmTicketSystemStatus, Never, Realm;
        GodMode, Unhandled, Realm;
        GossipComplete, Never, Realm;
        GossipMessage, Never, Realm;
        GossipPoi, Never, Realm;
        GroupActionThrottled, Unhandled, Realm;
        GroupDecline, Never, Realm;
        GroupDestroyed, Never, Realm;
        GroupNewLeader, Never, Realm;
        GroupUninvite, Never, Realm;
        GuildAchievementDeleted, Never, Realm;
        GuildAchievementEarned, Never, Realm;
        GuildAchievementMembers, Never, Realm;
        GuildBankLogQueryResults, Never, Realm;
        GuildBankQueryResults, Never, Realm;
        GuildBankRemainingWithdrawMoney, Never, Realm;
        GuildBankTextQueryResult, Never, Realm;
        GuildChallengeCompleted, Unhandled, Realm;
        GuildChallengeUpdate, Never, Realm;
        GuildChangeNameResult, Unhandled, Realm;
        GuildCommandResult, Never, Realm;
        GuildCriteriaDeleted, Never, Realm;
        GuildCriteriaUpdate, Never, Realm;
        GuildEventBankContentsChanged, Never, Realm;
        GuildEventBankMoneyChanged, Never, Realm;
        GuildEventDisbanded, Never, Realm;
        GuildEventLogQueryResults, Never, Realm;
        GuildEventMotd, Never, Realm;
        GuildEventNewLeader, Never, Realm;
        GuildEventPlayerJoined, Never, Realm;
        GuildEventPlayerLeft, Never, Realm;
        GuildEventPresenceChange, Never, Realm;
        GuildEventRanksUpdated, Never, Realm;
        GuildEventRankChanged, Never, Realm;
        GuildEventTabAdded, Never, Realm;
        GuildEventTabDeleted, Unhandled, Realm;
        GuildEventTabModified, Never, Realm;
        GuildEventTabTextChanged, Never, Realm;
        GuildFlaggedForRename, Never, Realm;
        GuildInvite, Never, Realm;
        GuildInviteDeclined, Unhandled, Realm;
        GuildInviteExpired, Unhandled, Realm;
        GuildItemLooted, Unhandled, Realm;
        GuildKnownRecipes, Unhandled, Realm;
        GuildMembersWithRecipe, Unhandled, Realm;
        GuildMemberDailyReset, Never, Realm;
        GuildMemberRecipes, Unhandled, Realm;
        GuildMemberUpdateNote, Never, Realm;
        GuildMoved, Unhandled, Realm;
        GuildMoveStarting, Unhandled, Realm;
        GuildNameChanged, Never, Realm;
        GuildNews, Never, Realm;
        GuildNewsDeleted, Unhandled, Realm;
        GuildPartyState, Never, Instance;
        GuildPermissionsQueryResults, Never, Realm;
        GuildRanks, Never, Realm;
        GuildReputationReactionChanged, Unhandled, Realm;
        GuildReset, Unhandled, Realm;
        GuildRewardList, Never, Realm;
        GuildRoster, Never, Realm;
        GuildRosterUpdate, Never, Realm;
        GuildSendRankChange, Never, Realm;
        HealthUpdate, Never, Realm;
        HighestThreatUpdate, Never, Instance;
        HotfixMessage, Unhandled, Realm;
        HotfixResponse, Never, Realm;
        InitializeFactions, Never, Instance;
        InitialSetup, Never, Instance;
        InitWorldStates, Never, Instance;
        InspectHonorStats, Never, Instance;
        InspectPvp, Never, Instance;
        InspectResult, Never, Instance;
        InstanceEncounterChangePriority, Never, Instance;
        InstanceEncounterDisengageUnit, Never, Instance;
        InstanceEncounterEnd, Never, Instance;
        InstanceEncounterEngageUnit, Never, Instance;
        InstanceEncounterGainCombatResurrectionCharge, Never, Instance;
        InstanceEncounterInCombatResurrection, Never, Instance;
        InstanceEncounterObjectiveComplete, Unhandled, Instance;
        InstanceEncounterObjectiveStart, Unhandled, Instance;
        InstanceEncounterObjectiveUpdate, Unhandled, Instance;
        InstanceEncounterPhaseShiftChanged, Unhandled, Instance;
        InstanceEncounterStart, Never, Instance;
        InstanceEncounterTimerStart, Unhandled, Instance;
        InstanceGroupSizeChanged, Unhandled, Realm;
        InstanceInfo, Never, Realm;
        InstanceReset, Never, Realm;
        InstanceResetFailed, Never, Realm;
        InstanceSaveCreated, Never, Realm;
        InvalidatePageText, Unhandled, Realm;
        InvalidatePlayer, Never, Realm;
        InvalidPromotionCode, Unhandled, Realm;
        InventoryChangeFailure, Never, Realm;
        IsQuestCompleteResponse, Unhandled, Realm;
        ItemChanged, Unhandled, Realm;
        ItemCooldown, Never, Realm;
        ItemEnchantTimeUpdate, Never, Instance;
        ItemExpirePurchaseRefund, Never, Instance;
        ItemPurchaseRefundResult, Never, Instance;
        ItemPushResult, Never, Realm;
        ItemTimeUpdate, Never, Realm;
        // ItemUpgradeResult, Unhandled, Realm;
        KickReason, Unhandled, Realm;
        LearnedSpells, Never, Instance;
        LearnPvpTalentsFailed, Never, Realm;
        LearnTalentsFailed, Never, Realm;
        LevelUpdate, Unhandled, Realm;
        LevelUpInfo, Never, Realm;
        LfgBootPlayer, Never, Realm;
        LfgDisabled, Never, Realm;
        LfgInstanceShutdownCountdown, Unhandled, Realm;
        LfgJoinResult, Never, Realm;
        LfgListJoinResult, Unhandled, Realm;
        LfgListSearchResults, Unhandled, Realm;
        LfgListSearchStatus, Unhandled, Realm;
        LfgListUpdateBlacklist, Unhandled, Realm;
        LfgListUpdateStatus, Unhandled, Realm;
        LfgOfferContinue, Never, Realm;
        LfgPartyInfo, Never, Instance;
        LfgPlayerInfo, Never, Instance;
        LfgPlayerReward, Never, Realm;
        LfgProposalUpdate, Never, Realm;
        LfgQueueStatus, Never, Realm;
        LfgReadyCheckResult, Unhandled, Realm;
        LfgReadyCheckUpdate, Unhandled, Realm;
        LfgRoleCheckUpdate, Never, Realm;
        LfgSlotInvalid, Unhandled, Realm;
        LfgTeleportDenied, Never, Realm;
        LfgUpdateStatus, Never, Realm;
        LfGuildApplicantListChanged, Never, Realm;
        LfGuildApplications, Never, Realm;
        LfGuildApplicationsListChanged, Never, Realm;
        LfGuildBrowse, Never, Realm;
        LfGuildCommandResult, Unhandled, Realm;
        LfGuildPost, Never, Realm;
        LfGuildRecruits, Never, Realm;
        LiveRegionAccountRestoreResult, Unhandled, Realm;
        LiveRegionCharacterCopyResult, Unhandled, Realm;
        LiveRegionGetAccountCharacterListResult, Unhandled, Realm;
        LoadCufProfiles, Never, Instance;
        LoadEquipmentSet, Never, Instance;
        LoadSelectedTrophyResult, Unhandled, Realm;
        LoginSetTimeSpeed, Never, Instance;
        LoginVerifyWorld, Never, Instance;
        LogoutCancelAck, Never, Instance;
        LogoutComplete, Never, Realm;
        LogoutResponse, Never, Instance;
        LogXpGain, Never, Realm;
        LootAllPassed, Never, Instance;
        LootList, Never, Instance;
        LootMoneyNotify, Never, Instance;
        LootRelease, Never, Instance;
        LootReleaseAll, Never, Instance;
        LootRemoved, Never, Instance;
        LootResponse, Never, Instance;
        LootRoll, Never, Instance;
        LootRollsComplete, Never, Instance;
        LootRollWon, Never, Instance;
        LossOfControlAuraUpdate, Unhandled, Realm;
        MailCommandResult, Never, Realm;
        MailListResult, Never, Realm;
        MailQueryNextTimeResult, Never, Realm;
        MapObjectivesInit, Unhandled, Realm;
        MapObjEvents, Unhandled, Realm;
        MasterLootCandidateList, Unhandled, Realm;
        MessageBox, Unhandled, Realm;
        MinimapPing, Never, Realm;
        MirrorImageComponentedData, Never, Realm;
        MirrorImageCreatureData, Never, Realm;
        MissileCancel, Unhandled, Realm;
        ModifyChargeRecoverySpeed, Unhandled, Instance;
        ModifyCooldown, Never, Instance;
        ModifyCooldownRecoverySpeed, Unhandled, Instance;
        ModifyPartyRange, Unhandled, Instance;
        Motd, Never, Realm;
        MountResult, Unhandled, Instance;
        MoveApplyMovementForce, Unhandled, Instance;
        MoveDisableCollision, Never, Instance;
        MoveDisableDoubleJump, Never, Instance;
        MoveDisableGravity, Never, Instance;
        MoveDisableTransitionBetweenSwimAndFly, Never, Instance;
        MoveEnableCollision, Never, Instance;
        MoveEnableDoubleJump, Never, Instance;
        MoveEnableGravity, Never, Instance;
        MoveEnableTransitionBetweenSwimAndFly, Never, Instance;
        MoveKnockBack, Never, Instance;
        MoveRemoveMovementForce, Unhandled, Instance;
        MoveRoot, Never, Instance;
        MoveSetActiveMover, Never, Instance;
        MoveSetCanFly, Never, Instance;
        MoveSetCanTurnWhileFalling, Never, Instance;
        MoveSetCollisionHeight, Never, Instance;
        MoveSetCompoundState, Never, Instance;
        MoveSetFeatherFall, Never, Instance;
        MoveSetFlightBackSpeed, Never, Instance;
        MoveSetFlightSpeed, Never, Instance;
        MoveSetHovering, Never, Instance;
        MoveSetIgnoreMovementForces, Unhandled, Instance;
        MoveSetLandWalk, Never, Instance;
        MoveSetNormalFall, Never, Instance;
        MoveSetPitchRate, Never, Instance;
        MoveSetRunBackSpeed, Never, Instance;
        MoveSetRunSpeed, Never, Instance;
        MoveSetSwimBackSpeed, Never, Instance;
        MoveSetSwimSpeed, Never, Instance;
        MoveSetTurnRate, Never, Instance;
        MoveSetVehicleRecId, Never, Instance;
        MoveSetWalkSpeed, Never, Instance;
        MoveSetWaterWalk, Never, Instance;
        MoveSkipTime, Unhandled, Instance;
        MoveSplineDisableCollision, Never, Instance;
        MoveSplineDisableGravity, Never, Instance;
        MoveSplineEnableCollision, Never, Instance;
        MoveSplineEnableGravity, Never, Instance;
        MoveSplineRoot, Never, Instance;
        MoveSplineSetFeatherFall, Never, Instance;
        MoveSplineSetFlightBackSpeed, Never, Instance;
        MoveSplineSetFlightSpeed, Never, Instance;
        MoveSplineSetFlying, Never, Instance;
        MoveSplineSetHover, Never, Instance;
        MoveSplineSetLandWalk, Never, Instance;
        MoveSplineSetNormalFall, Never, Instance;
        MoveSplineSetPitchRate, Never, Instance;
        MoveSplineSetRunBackSpeed, Never, Instance;
        MoveSplineSetRunMode, Never, Instance;
        MoveSplineSetRunSpeed, Never, Instance;
        MoveSplineSetSwimBackSpeed, Never, Instance;
        MoveSplineSetSwimSpeed, Never, Instance;
        MoveSplineSetTurnRate, Never, Instance;
        MoveSplineSetWalkMode, Never, Instance;
        MoveSplineSetWalkSpeed, Never, Instance;
        MoveSplineSetWaterWalk, Never, Instance;
        MoveSplineStartSwim, Never, Instance;
        MoveSplineStopSwim, Never, Instance;
        MoveSplineUnroot, Never, Instance;
        MoveSplineUnsetFlying, Never, Instance;
        MoveSplineUnsetHover, Never, Instance;
        MoveTeleport, Never, Instance;
        MoveUnroot, Never, Instance;
        MoveUnsetCanFly, Never, Instance;
        MoveUnsetCanTurnWhileFalling, Never, Instance;
        MoveUnsetHovering, Never, Instance;
        MoveUnsetIgnoreMovementForces, Never, Instance;
        MoveUpdate, Never, Instance;
        MoveUpdateApplyMovementForce, Never, Instance;
        MoveUpdateCollisionHeight, Never, Instance;
        MoveUpdateFlightBackSpeed, Never, Instance;
        MoveUpdateFlightSpeed, Never, Instance;
        MoveUpdateKnockBack, Never, Instance;
        MoveUpdatePitchRate, Never, Instance;
        MoveUpdateRemoveMovementForce, Never, Instance;
        MoveUpdateRunBackSpeed, Never, Instance;
        MoveUpdateRunSpeed, Never, Instance;
        MoveUpdateSwimBackSpeed, Never, Instance;
        MoveUpdateSwimSpeed, Never, Instance;
        MoveUpdateTeleport, Never, Instance;
        MoveUpdateTurnRate, Never, Instance;
        MoveUpdateWalkSpeed, Never, Instance;
        NeutralPlayerFactionSelectResult, Unhandled, Realm;
        NewTaxiPath, Never, Realm;
        NewWorld, Never, Realm;
        NotifyDestLocSpellCast, Unhandled, Realm;
        NotifyMissileTrajectoryCollision, Never, Realm;
        NotifyMoney, Unhandled, Realm;
        NotifyReceivedMail, Never, Realm;
        OfferPetitionError, Never, Realm;
        OnCancelExpectedRideVehicleAura, Never, Instance;
        OnMonsterMove, Never, Instance;
        OpenAlliedRaceDetailsGiver, Unhandled, Realm;
        OpenContainer, Unhandled, Realm;
        OpenLfgDungeonFinder, Unhandled, Realm;
        OpenShipmentNpcFromGossip, Unhandled, Realm;
        OpenShipmentNpcResult, Unhandled, Realm;
        OpenTransmogrifier, Never, Instance;
        OverrideLight, Never, Realm;
        PageText, Never, Realm;
        PartyCommandResult, Never, Realm;
        PartyInvite, Never, Realm;
        PartyKillLog, Never, Realm;
        PartyMemberState, Never, Realm;
        PartyMemberStateUpdate, Never, Realm;
        PartyUpdate, Never, Realm;
        PauseMirrorTimer, Never, Realm;
        PendingRaidLock, Never, Realm;
        PetitionAlreadySigned, Never, Realm;
        PetitionRenameGuildResponse, Never, Realm;
        PetitionShowList, Never, Realm;
        PetitionShowSignatures, Never, Realm;
        PetitionSignResults, Never, Realm;
        PetActionFeedback, Unhandled, Realm;
        PetActionSound, Unhandled, Realm;
        PetAdded, Unhandled, Realm;
        PetBattleChatRestricted, Unhandled, Realm;
        PetBattleDebugQueueDumpResponse, Unhandled, Realm;
        PetBattleFinalizeLocation, Unhandled, Realm;
        PetBattleFinalRound, Unhandled, Realm;
        PetBattleFinished, Unhandled, Realm;
        PetBattleFirstRound, Unhandled, Realm;
        PetBattleInitialUpdate, Unhandled, Realm;
        PetBattleMaxGameLengthWarning, Unhandled, Realm;
        PetBattlePvpChallenge, Unhandled, Realm;
        PetBattleQueueProposeMatch, Unhandled, Realm;
        PetBattleQueueStatus, Unhandled, Realm;
        PetBattleReplacementsMade, Unhandled, Realm;
        PetBattleRequestFailed, Unhandled, Realm;
        PetBattleRoundResult, Unhandled, Realm;
        PetBattleSlotUpdates, Never, Realm;
        PetCastFailed, Never, Realm;
        PetClearSpells, Unhandled, Realm;
        PetDismissSound, Unhandled, Realm;
        PetGodMode, Unhandled, Realm;
        PetGuids, Unhandled, Realm;
        PetLearnedSpells, Never, Instance;
        PetMode, Unhandled, Realm;
        PetNameInvalid, Never, Realm;
        PetSlotUpdated, Unhandled, Realm;
        PetSpellsMessage, Never, Instance;
        PetStableList, Never, Instance;
        PetStableResult, Unhandled, Realm;
        PetTameFailure, Unhandled, Realm;
        PetUnlearnedSpells, Never, Instance;
        PhaseShiftChange, Never, Instance;
        PlayedTime, Never, Instance;
        PlayerBound, Never, Realm;
        PlayerSaveGuildEmblem, Never, Realm;
        PlayerSkinned, Unhandled, Realm;
        PlayerTabardVendorActivate, Never, Realm;
        PlayMusic, Never, Realm;
        PlayObjectSound, Never, Realm;
        PlayOneShotAnimKit, Never, Realm;
        PlayOrphanSpellVisual, Never, Realm;
        PlayScene, Never, Instance;
        PlaySound, Never, Realm;
        PlaySpeakerbotSound, Never, Realm;
        PlaySpellVisual, Never, Realm;
        PlaySpellVisualKit, Never, Realm;
        PlayTimeWarning, Unhandled, Realm;
        Pong, Never, Realm;
        PowerUpdate, Never, Realm;
        PrestigeAndHonorInvoluntarilyChanged, Unhandled, Realm;
        PreRessurect, Never, Realm;
        PrintNotification, Never, Realm;
        ProcResist, Never, Realm;
        ProposeLevelGrant, Never, Realm;
        PushSpellToActionBar, Unhandled, Realm;
        PvpCredit, Never, Realm;
        PvpLogData, Never, Instance;
        PvpOptionsEnabled, Never, Realm;
        PvpSeason, Never, Realm;
        QueryBattlePetNameResponse, Unhandled, Realm;
        QueryCreatureResponse, Never, Instance;
        QueryGameObjectResponse, Never, Realm;
        QueryGarrisonCreatureNameResponse, Unhandled, Realm;
        QueryGuildInfoResponse, Never, Realm;
        QueryItemTextResponse, Never, Realm;
        QueryNpcTextResponse, Never, Instance;
        QueryPageTextResponse, Never, Realm;
        QueryPetitionResponse, Never, Realm;
        QueryPetNameResponse, Never, Instance;
        QueryPlayerNameResponse, Never, Realm;
        QueryQuestInfoResponse, Never, Instance;
        QueryQuestRewardResponse, Unhandled, Instance;
        QueryTimeResponse, Never, Instance;
        QuestCompletionNpcResponse, Never, Instance;
        QuestConfirmAccept, Never, Realm;
        QuestForceRemoved, Unhandled, Realm;
        QuestGiverInvalidQuest, Never, Realm;
        QuestGiverOfferRewardMessage, Never, Realm;
        QuestGiverQuestComplete, Never, Realm;
        QuestGiverQuestDetails, Never, Realm;
        QuestGiverQuestFailed, Never, Realm;
        QuestGiverQuestListMessage, Never, Realm;
        QuestGiverRequestItems, Never, Realm;
        QuestGiverStatus, Never, Instance;
        QuestGiverStatusMultiple, Never, Instance;
        QuestGiverQuestTurnInFailure, Unhandled, Instance;
        QuestLogFull, Never, Realm;
        QuestPoiChanged, Unhandled, Realm;
        QuestPoiQueryResponse, Never, Realm;
        QuestPushResult, Never, Realm;
        QuestSpawnTrackingUpdate, Unhandled, Realm;
        QuestUpdateAddCredit, Never, Instance;
        QuestUpdateAddCreditSimple, Never, Instance;
        QuestUpdateAddPvpCredit, Never, Instance;
        QuestUpdateComplete, Never, Realm;
        QuestUpdateCompleteBySpell, Unhandled, Realm;
        QuestUpdateFailed, Unhandled, Realm;
        QuestUpdateFailedTimer, Never, Realm;
        RafEmailEnabledResponse, Unhandled, Realm;
        RaidDifficultySet, Never, Realm;
        RaidGroupOnly, Never, Realm;
        RaidInstanceMessage, Never, Realm;
        RaidMarkersChanged, Never, Realm;
        RandomRoll, Never, Realm;
        RatedBattlefieldInfo, Unhandled, Realm;
        ReadyCheckCompleted, Never, Realm;
        ReadyCheckResponse, Never, Realm;
        ReadyCheckStarted, Never, Realm;
        ReadItemResultFailed, Never, Realm;
        ReadItemResultOk, Never, Realm;
        RealmLookupInformation, Unhandled, Realm;
        RealmQueryResponse, Never, Realm;
        RecruitAFriendResponse, Unhandled, Realm;
        ReferAFriendExpired, Unhandled, Realm;
        ReferAFriendFailure, Never, Realm;
        RefreshComponent, Unhandled, Realm;
        RefreshSpellHistory, Unhandled, Realm;
        RemoveItemPassive, Unhandled, Realm;
        RemoveLossOfControl, Unhandled, Realm;
        ReplaceTrophyResponse, Unhandled, Realm;
        ReportPvpPlayerAfkResult, Never, Instance;
        RequestAddonList, Unhandled, Realm;
        RequestCemeteryListResponse, Never, Instance;
        RequestPvpBrawlInfoResponse, Unhandled, Realm;
        RequestPvpRewardsResponse, Unhandled, Realm;
        ResearchComplete, Unhandled, Realm;
        ResetAreaTrigger, Unhandled, Realm;
        ResetCompressionContext, Unhandled, Realm;
        ResetFailedNotify, Never, Realm;
        ResetRangedCombatTimer, Unhandled, Realm;
        ResetWeeklyCurrency, Unhandled, Realm;
        RespecWipeConfirm, Never, Instance;
        RespondInspectAchievements, Never, Instance;
        ResumeCastBar, Unhandled, Realm;
        ResumeComms, Never, Realm;
        ResumeToken, Never, Instance;
        ResurrectRequest, Never, Realm;
        ResyncRunes, Never, Realm;
        RoleChangedInform, Never, Realm;
        RoleChosen, Never, Realm;
        RolePollInform, Never, Realm;
        RuneRegenDebug, Unhandled, Realm;
        ScenarioBoot, Never, Instance;
        ScenarioCompleted, Never, Instance;
        ScenarioPois, Never, Realm;
        ScenarioProgressUpdate, Never, Instance;
        ScenarioSetShouldShowCriteria, Unhandled, Instance;
        ScenarioSpellUpdate, Unhandled, Instance;
        ScenarioState, Never, Instance;
        SceneObjectEvent, Unhandled, Realm;
        SceneObjectPetBattleFinalRound, Unhandled, Realm;
        SceneObjectPetBattleFinished, Unhandled, Realm;
        SceneObjectPetBattleFirstRound, Unhandled, Realm;
        SceneObjectPetBattleInitialUpdate, Unhandled, Realm;
        SceneObjectPetBattleReplacementsMade, Unhandled, Realm;
        SceneObjectPetBattleRoundResult, Unhandled, Realm;
        ScriptCast, Unhandled, Realm;
        SellResponse, Never, Realm;
        SendItemPassives, Unhandled, Realm;
        SendKnownSpells, Never, Instance;
        SendRaidTargetUpdateAll, Never, Realm;
        SendRaidTargetUpdateSingle, Never, Realm;
        SendSpellCharges, Never, Instance;
        SendSpellHistory, Never, Instance;
        SendUnlearnSpells, Never, Instance;
        ServerFirstAchievement, Never, Realm;
        ServerFirstAchievements, Unhandled, Realm;
        ServerTime, Unhandled, Realm;
        SetupCurrency, Never, Instance;
        SetupResearchHistory, Unhandled, Instance;
        SetAiAnimKit, Never, Instance;
        SetAllTaskProgress, Unhandled, Instance;
        SetAnimTier, Unhandled, Realm;
        SetCurrency, Never, Instance;
        SetDfFastLaunchResult, Unhandled, Realm;
        SetDungeonDifficulty, Never, Realm;
        SetFactionAtWar, Unhandled, Instance;
        SetFactionNotVisible, Never, Instance;
        SetFactionStanding, Never, Instance;
        SetFactionVisible, Never, Instance;
        SetFlatSpellModifier, Never, Instance;
        SetForcedReactions, Never, Instance;
        SetItemPurchaseData, Never, Instance;
        SetLootMethodFailed, Unhandled, Realm;
        SetMaxWeeklyQuantity, Unhandled, Realm;
        SetMeleeAnimKit, Never, Instance;
        SetMovementAnimKit, Never, Instance;
        SetPctSpellModifier, Never, Instance;
        SetPetSpecialization, Never, Realm;
        SetPlayerDeclinedNamesResult, Never, Realm;
        SetPlayHoverAnim, Never, Instance;
        SetProficiency, Never, Instance;
        SetSpellCharges, Never, Instance;
        SetTaskComplete, Unhandled, Realm;
        SetTimeZoneInformation, Never, Realm;
        SetVehicleRecId, Never, Instance;
        ShowAdventureMap, Unhandled, Realm;
        ShowBank, Never, Instance;
        ShowMailbox, Never, Realm;
        ShowNeutralPlayerFactionSelectUi, Unhandled, Realm;
        ShowTaxiNodes, Never, Realm;
        ShowTradeSkillResponse, Unhandled, Realm;
        SocketGems, Never, Instance;
        SocketGemsFailure, Unhandled, Realm;
        SorStartExperienceIncomplete, Unhandled, Realm;
        SortBagsResult, Never, Instance;
        SpecializationChanged, Unhandled, Realm;
        SpecialMountAnim, Never, Instance;
        SpecInvoluntarilyChanged, Unhandled, Realm;
        SpellAbsorbLog, Unhandled, Instance;
        SpellCategoryCooldown, Unhandled, Instance;
        SpellChannelStart, Never, Instance;
        SpellChannelUpdate, Never, Instance;
        SpellCooldown, Never, Instance;
        SpellDamageShield, Never, Instance;
        SpellDelayed, Never, Instance;
        SpellDispellLog, Never, Instance;
        SpellEnergizeLog, Never, Instance;
        SpellExecuteLog, Never, Instance;
        SpellFailedOther, Never, Instance;
        SpellFailure, Never, Instance;
        SpellGo, Never, Instance;
        SpellHealLog, Never, Instance;
        SpellInstakillLog, Never, Instance;
        SpellInterruptLog, Never, Instance;
        SpellMissLog, Never, Instance;
        SpellNonMeleeDamageLog, Never, Instance;
        SpellOrDamageImmune, Never, Instance;
        SpellPeriodicAuraLog, Never, Instance;
        SpellPrepare, Never, Instance;
        SpellStart, Never, Instance;
        SpiritHealerConfirm, Never, Realm;
        StandStateUpdate, Never, Realm;
        StartElapsedTimer, Unhandled, Instance;
        StartElapsedTimers, Unhandled, Instance;
        StartLootRoll, Never, Instance;
        StartMirrorTimer, Never, Realm;
        StartTimer, Unhandled, Realm;
        StopElapsedTimer, Unhandled, Realm;
        StopMirrorTimer, Never, Realm;
        StopSpeakerbotSound, Unhandled, Realm;
        StreamingMovies, Unhandled, Realm;
        SummonCancel, Unhandled, Realm;
        SummonRaidMemberValidateFailed, Unhandled, Realm;
        SummonRequest, Never, Instance;
        SupercededSpells, Never, Instance;
        SuspendComms, Unhandled, Realm;
        SuspendToken, Never, Instance;
        TalentsInvoluntarilyReset, Unhandled, Realm;
        TaxiNodeStatus, Never, Realm;
        TextEmote, Never, Instance;
        ThreatClear, Never, Realm;
        ThreatRemove, Never, Instance;
        ThreatUpdate, Never, Instance;
        TimeAdjustment, Unhandled, Realm;
        TimeSyncRequest, Never, Instance;
        TitleEarned, Never, Realm;
        TitleLost, Never, Realm;
        TotemCreated, Never, Realm;
        TotemMoved, Never, Realm;
        TradeStatus, Never, Instance;
        TradeUpdated, Never, Instance;
        TrainerBuyFailed, Never, Realm;
        TrainerList, Never, Instance;
        TransferAborted, Never, Realm;
        TransferPending, Never, Realm;
        TransmogCollectionUpdate, Never, Realm;
        TransmogSetCollectionUpdate, Unhandled, Realm;
        TriggerCinematic, Never, Realm;
        TriggerMovie, Never, Realm;
        TurnInPetitionResult, Never, Realm;
        TutorialFlags, Never, Realm;
        TutorialHighlightSpell, Unhandled, Realm;
        TutorialUnhighlightSpell, Unhandled, Realm;
        TwitterStatus, Unhandled, Realm;
        UiTime, Never, Realm;
        UndeleteCharacterResponse, Never, Realm;
        UndeleteCooldownStatusResponse, Never, Realm;
        UnlearnedSpells, Never, Instance;
        UpdateAccountData, Never, Realm;
        UpdateActionButtons, Never, Instance;
        UpdateCelestialBody, Unhandled, Realm;
        UpdateCharacterFlags, Unhandled, Realm;
        UpdateExpansionLevel, Unhandled, Realm;
        UpdateGameTimeState, Unhandled, Realm;
        UpdateInstanceOwnership, Never, Realm;
        UpdateLastInstance, Never, Realm;
        UpdateObject, Never, Instance;
        UpdateTalentData, Never, Instance;
        UpdateTaskProgress, Unhandled, Realm;
        UpdateWeeklySpellUsage, Unhandled, Realm;
        UpdateWorldState, Never, Instance;
        UserlistAdd, Never, Realm;
        UserlistRemove, Never, Realm;
        UserlistUpdate, Never, Realm;
        UseEquipmentSetResult, Never, Realm;
        VendorInventory, Never, Instance;
        VignetteUpdate, Unhandled, Instance;
        VoidItemSwapResponse, Never, Instance;
        VoidStorageContents, Never, Instance;
        VoidStorageFailed, Never, Instance;
        VoidStorageTransferChanges, Never, Instance;
        VoidTransferResult, Never, Instance;
        WaitQueueFinish, Never, Realm;
        WaitQueueUpdate, Never, Realm;
        WardenData, Unhandled, Realm;
        WargameRequestSuccessfullySentToOpponent, Unhandled, Realm;
        Weather, Never, Instance;
        WeeklySpellUsage, Unhandled, Instance;
        Who, Never, Realm;
        WhoIs, Never, Realm;
        WorldQuestUpdate, Never, Instance;
        WorldServerInfo, Never, Instance;
        WorldText, Unhandled, Realm;
        WowTokenAuctionSold, Unhandled, Realm;
        WowTokenBuyRequestConfirmation, Unhandled, Instance;
        WowTokenBuyResultConfirmation, Unhandled, Realm;
        WowTokenCanRedeemForBalanceResult, Unhandled, Realm;
        WowTokenCanVeteranBuyResult, Unhandled, Realm;
        WowTokenDistributionGlueUpdate, Unhandled, Realm;
        WowTokenDistributionUpdate, Unhandled, Realm;
        WowTokenMarketPriceResponse, Never, Instance;
        WowTokenRedeemGameTimeUpdated, Unhandled, Realm;
        WowTokenRedeemRequestConfirmation, Unhandled, Instance;
        WowTokenRedeemResult, Unhandled, Realm;
        WowTokenSellRequestConfirmation, Unhandled, Instance;
        WowTokenSellResultConfirmation, Unhandled, Realm;
        WowTokenUpdateAuctionableListResponse, Never, Instance;
        XpGainAborted, Unhandled, Realm;
        XpGainEnabled, Unhandled, Realm;
        ZoneUnderAttack, Never, Realm;
        CompressedPacket, Never, Realm;
        MultiplePackets, Unhandled, Realm;
        ConsumptionConversionInfoResponse, Unhandled, Realm;        // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        ConsumptionConversionResult, Unhandled, Realm;              // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        GarrisonMissionRewardResponse, Unhandled, Realm;            // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        GarrisonOpenTalentNpc, Unhandled, Realm;                    // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        GroupInviteConfirmation, Unhandled, Realm;                  // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        InstanceEncounterSetAllowingRelease, Unhandled, Realm;      // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        InstanceEncounterSetSuppressingRelease, Unhandled, Realm;   // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        SpellFailureMessage, Unhandled, Realm;                      // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        SpellHealAbsorbLog, Unhandled, Realm;                       // NOTE: NOT IN TC BUT FILLED SO THAT MATCH IS EXHAUSTIVE
        )
    }
}
