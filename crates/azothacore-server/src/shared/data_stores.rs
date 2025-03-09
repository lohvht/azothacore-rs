pub mod db2_loader;
pub mod db2_structure;
pub mod dbc_enums;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io,
    ops::Deref,
    path::Path,
    str::FromStr,
    time::Instant,
};

use azothacore_common::{
    az_error,
    bevy_app::{AzStartupFailedEvent, TokioRuntime},
    collision::management::vmap_mgr2::LiquidFlagsGetter,
    configuration::{ConfigMgr, DataDirConfig},
    deref_boilerplate,
    utils::buffered_file_open,
    AzError,
    AzResult,
    Locale,
    MapLiquidTypeFlag,
};
use azothacore_database::{database_env::HotfixDatabase, DbAcquire, DbDriver};
use bevy::{
    app::App,
    ecs::system::SystemParam,
    prelude::{
        not,
        on_event,
        Commands,
        Event,
        EventReader,
        EventWriter,
        IntoSystemConfigs,
        IntoSystemSetConfigs,
        Real,
        Res,
        Resource,
        Startup,
        SystemSet,
        Time,
    },
};
use db2_loader::DB2DatabaseLoader;
use dbc_enums::{
    CharBaseSectionVariation,
    CharSectionType,
    Class,
    ClassError,
    DifficultyFlag,
    DifficultyID,
    Gender,
    ItemClassID,
    MapType,
    Power,
    QuestPackageFilter,
    Race,
    TaxiNodeFlags,
    WorldMapTransformsFlags,
};
use flagset::FlagSet;
use nalgebra::Vector2;
use num::FromPrimitive;
use regex::{Regex, RegexBuilder};
use sqlx::{Database, FromRow};
use tracing::{error, info, warn};
use wow_db2::DB2;

use crate::{
    game::world::WorldConfig,
    shared::data_stores::{db2_loader::DB2FileLoader, db2_structure::*, dbc_enums::BATTLE_PET_SPECIES_MAX_ID},
};

#[derive(Resource)]
pub struct DB2Storage<D: DB2> {
    table_hash: u32,
    records:    BTreeMap<u32, D>,
}

impl<D: DB2> Deref for DB2Storage<D> {
    type Target = BTreeMap<u32, D>;

    fn deref(&self) -> &Self::Target {
        &self.records
    }
}

fn db2_file_error(e: io::Error) -> AzError {
    az_error!(e).context("db2 file exists but unable to process it properly, extracted file might be from wrong client version")
}

fn db2_database_error(e: io::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("loading db2 info from database failed; err={e}"))
}

fn open_db2_file_loader<D: DB2 + From<wow_db2::DB2RawRecord>, P: AsRef<Path>>(db2_dir: P, locale: Locale) -> AzResult<DB2FileLoader<D>> {
    let file = buffered_file_open(db2_dir.as_ref().join(locale.to_string()).join(D::db2_file()))
        .map_err(|e| az_error!(e).context("unable to upen db2 file, extracted file not exists"))?;
    DB2FileLoader::<D>::from_reader(file, locale).map_err(db2_file_error)
}

impl<D: DB2 + From<wow_db2::DB2RawRecord> + for<'r> FromRow<'r, <DbDriver as Database>::Row> + Send + Unpin> DB2Storage<D> {
    /// LoadDB2 in TC / LoadDBC in AC
    pub async fn load<'e, P: AsRef<Path>, A: DbAcquire<'e>>(
        db2_dir: P,
        hotfix_db: A,
        default_locale: Locale,
        available_db2_locales: impl Iterator<Item = Locale>,
    ) -> AzResult<Self> {
        let default_locale_db2 = open_db2_file_loader::<D, _>(&db2_dir, default_locale)?;
        let table_hash = default_locale_db2.header.table_hash;
        let mut db2_data = default_locale_db2
            .produce_data()
            .map_err(db2_file_error)?
            .map(|v| (v.id(), v))
            .collect::<BTreeMap<_, _>>();

        let mut hotfix_db = hotfix_db
            .acquire()
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("unable to upen connect to database to get db2 details; err={e}")))?;
        DB2DatabaseLoader::load(&mut *hotfix_db, &mut db2_data).await.map_err(db2_database_error)?;
        // DB2DatabaseLoader::load() always loads strings into enUS locale, other locales are expected to have data in corresponding _locale tables
        // so we need to make additional call to load that data in case said locale
        DB2DatabaseLoader::load_localised_strings(&mut *hotfix_db, &mut db2_data)
            .await
            .map_err(db2_database_error)?;

        for l in available_db2_locales {
            if default_locale == l {
                continue;
            }
            let localised_file_loader = match open_db2_file_loader::<D, _>(&db2_dir, l) {
                Ok(f) => f,
                Err(e) => {
                    warn!(target:"db2",cause=?e, "error loading localised db2 files");
                    continue;
                },
            };
            let localised_db2_data = match localised_file_loader.produce_raw_data() {
                Err(e) => {
                    warn!(target:"db2",cause=?e, "error producing localised db2 records");
                    continue;
                },
                Ok(vs) => vs.map(|v| (v.id, v)).collect::<BTreeMap<_, _>>(),
            };
            for d in db2_data.values_mut() {
                if let Some(other) = localised_db2_data.get(&d.id()) {
                    d.merge_strs(other);
                }
            }
        }

        Ok(Self { table_hash, records: db2_data })
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum DB2StoresMgrSet {
    Start,
    LoadStores,
    SetupStoreHelpers,
}

/// Encapsulates the whole of DB2Manager::LoadStores in TC, LoadDBCStores
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InitDB2MgrSet;

macro_rules! load_db2 {
    ( $app:expr, $db2_type:ty ) => {{
        $app.add_systems(Startup, load_db2::<$db2_type>.in_set(DB2StoresMgrSet::LoadStores));
    }};
}

fn load_db2_before() {
    //- Load DB2s
    info!(target = "server.loading", "Initialising DB2 stores...");
}

pub fn db2_mgr_plugin(app: &mut App) {
    app.add_event::<DB2LoadStartEvent>()
        .add_systems(Startup, load_db2_before.in_set(DB2StoresMgrSet::Start));
    load_db2!(app, Achievement);
    load_db2!(app, AnimKit);
    load_db2!(app, AreaGroupMember);
    load_db2!(app, AreaTable);
    load_db2!(app, AreaTrigger);
    load_db2!(app, ArmorLocation);
    load_db2!(app, Artifact);
    load_db2!(app, ArtifactAppearance);
    load_db2!(app, ArtifactAppearanceSet);
    load_db2!(app, ArtifactCategory);
    load_db2!(app, ArtifactPower);
    load_db2!(app, ArtifactPowerLink);
    load_db2!(app, ArtifactPowerPicker);
    load_db2!(app, ArtifactPowerRank);
    load_db2!(app, ArtifactTier);
    load_db2!(app, ArtifactUnlock);
    load_db2!(app, AuctionHouse);
    load_db2!(app, BankBagSlotPrices);
    load_db2!(app, BannedAddons);
    load_db2!(app, BarberShopStyle);
    load_db2!(app, BattlePetBreedQuality);
    load_db2!(app, BattlePetBreedState);
    load_db2!(app, BattlePetSpecies);
    load_db2!(app, BattlePetSpeciesState);
    load_db2!(app, BattlemasterList);
    load_db2!(app, BroadcastText);
    load_db2!(app, Cfg_Regions);
    load_db2!(app, CharacterFacialHairStyles);
    load_db2!(app, CharBaseSection);
    load_db2!(app, CharSections);
    load_db2!(app, CharStartOutfit);
    load_db2!(app, CharTitles);
    load_db2!(app, ChatChannels);
    load_db2!(app, ChrClasses);
    load_db2!(app, ChrClassesXPowerTypes);
    load_db2!(app, ChrRaces);
    load_db2!(app, ChrSpecialization);
    load_db2!(app, CinematicCamera);
    load_db2!(app, CinematicSequences);
    load_db2!(app, ConversationLine);
    load_db2!(app, CreatureDisplayInfo);
    load_db2!(app, CreatureDisplayInfoExtra);
    load_db2!(app, CreatureFamily);
    load_db2!(app, CreatureModelData);
    load_db2!(app, CreatureType);
    load_db2!(app, Criteria);
    load_db2!(app, CriteriaTree);
    load_db2!(app, CurrencyTypes);
    load_db2!(app, Curve);
    load_db2!(app, CurvePoint);
    load_db2!(app, DestructibleModelData);
    load_db2!(app, Difficulty);
    load_db2!(app, DungeonEncounter);
    load_db2!(app, DurabilityCosts);
    load_db2!(app, DurabilityQuality);
    load_db2!(app, Emotes);
    load_db2!(app, EmotesText);
    load_db2!(app, EmotesTextSound);
    load_db2!(app, Faction);
    load_db2!(app, FactionTemplate);
    load_db2!(app, GameObjects);
    load_db2!(app, GameObjectDisplayInfo);
    load_db2!(app, GarrAbility);
    load_db2!(app, GarrBuilding);
    load_db2!(app, GarrBuildingPlotInst);
    load_db2!(app, GarrClassSpec);
    load_db2!(app, GarrFollower);
    load_db2!(app, GarrFollowerXAbility);
    load_db2!(app, GarrPlotBuilding);
    load_db2!(app, GarrPlot);
    load_db2!(app, GarrPlotInstance);
    load_db2!(app, GarrSiteLevel);
    load_db2!(app, GarrSiteLevelPlotInst);
    load_db2!(app, GemProperties);
    load_db2!(app, GlyphBindableSpell);
    load_db2!(app, GlyphProperties);
    load_db2!(app, GlyphRequiredSpec);
    load_db2!(app, GuildColorBackground);
    load_db2!(app, GuildColorBorder);
    load_db2!(app, GuildColorEmblem);
    load_db2!(app, GuildPerkSpells);
    load_db2!(app, Heirloom);
    load_db2!(app, Holidays);
    load_db2!(app, ImportPriceArmor);
    load_db2!(app, ImportPriceQuality);
    load_db2!(app, ImportPriceShield);
    load_db2!(app, ImportPriceWeapon);
    load_db2!(app, ItemAppearance);
    load_db2!(app, ItemArmorQuality);
    load_db2!(app, ItemArmorShield);
    load_db2!(app, ItemArmorTotal);
    load_db2!(app, ItemBagFamily);
    load_db2!(app, ItemBonus);
    load_db2!(app, ItemBonusListLevelDelta);
    load_db2!(app, ItemBonusTreeNode);
    load_db2!(app, ItemChildEquipment);
    load_db2!(app, ItemClass);
    load_db2!(app, ItemCurrencyCost);
    load_db2!(app, ItemDamageAmmo);
    load_db2!(app, ItemDamageOneHand);
    load_db2!(app, ItemDamageOneHandCaster);
    load_db2!(app, ItemDamageTwoHand);
    load_db2!(app, ItemDamageTwoHandCaster);
    load_db2!(app, ItemDisenchantLoot);
    load_db2!(app, ItemEffect);
    load_db2!(app, Item);
    load_db2!(app, ItemExtendedCost);
    load_db2!(app, ItemLevelSelector);
    load_db2!(app, ItemLevelSelectorQuality);
    load_db2!(app, ItemLevelSelectorQualitySet);
    load_db2!(app, ItemLimitCategory);
    load_db2!(app, ItemLimitCategoryCondition);
    load_db2!(app, ItemModifiedAppearance);
    load_db2!(app, ItemPriceBase);
    load_db2!(app, ItemRandomProperties);
    load_db2!(app, ItemRandomSuffix);
    load_db2!(app, ItemSearchName);
    load_db2!(app, ItemSet);
    load_db2!(app, ItemSetSpell);
    load_db2!(app, ItemSparse);
    load_db2!(app, ItemSpec);
    load_db2!(app, ItemSpecOverride);
    load_db2!(app, ItemUpgrade);
    load_db2!(app, ItemXBonusTree);
    load_db2!(app, Keychain);
    load_db2!(app, LFGDungeons);
    load_db2!(app, Light);
    load_db2!(app, LiquidType);
    load_db2!(app, Lock);
    load_db2!(app, MailTemplate);
    load_db2!(app, Map);
    load_db2!(app, MapDifficulty);
    load_db2!(app, ModifierTree);
    load_db2!(app, MountCapability);
    load_db2!(app, Mount);
    load_db2!(app, MountTypeXCapability);
    load_db2!(app, MountXDisplay);
    load_db2!(app, Movie);
    load_db2!(app, NameGen);
    load_db2!(app, NamesProfanity);
    load_db2!(app, NamesReserved);
    load_db2!(app, NamesReservedLocale);
    load_db2!(app, OverrideSpellData);
    load_db2!(app, Phase);
    load_db2!(app, PhaseXPhaseGroup);
    load_db2!(app, PlayerCondition);
    load_db2!(app, PowerDisplay);
    load_db2!(app, PowerType);
    load_db2!(app, PrestigeLevelInfo);
    load_db2!(app, PVPDifficulty);
    load_db2!(app, PVPItem);
    load_db2!(app, PvpReward);
    load_db2!(app, PvpTalent);
    load_db2!(app, PvpTalentUnlock);
    load_db2!(app, QuestFactionReward);
    load_db2!(app, QuestMoneyReward);
    load_db2!(app, QuestPackageItem);
    load_db2!(app, QuestSort);
    load_db2!(app, QuestV2);
    load_db2!(app, QuestXP);
    load_db2!(app, RandPropPoints);
    load_db2!(app, RewardPack);
    load_db2!(app, RewardPackXCurrencyType);
    load_db2!(app, RewardPackXItem);
    load_db2!(app, RulesetItemUpgrade);
    load_db2!(app, SandboxScaling);
    load_db2!(app, ScalingStatDistribution);
    load_db2!(app, Scenario);
    load_db2!(app, ScenarioStep);
    load_db2!(app, SceneScript);
    load_db2!(app, SceneScriptGlobalText);
    load_db2!(app, SceneScriptPackage);
    load_db2!(app, SceneScriptText);
    load_db2!(app, SkillLine);
    load_db2!(app, SkillLineAbility);
    load_db2!(app, SkillRaceClassInfo);
    load_db2!(app, SoundKit);
    load_db2!(app, SpecializationSpells);
    load_db2!(app, Spell);
    load_db2!(app, SpellAuraOptions);
    load_db2!(app, SpellAuraRestrictions);
    load_db2!(app, SpellCastTimes);
    load_db2!(app, SpellCastingRequirements);
    load_db2!(app, SpellCategories);
    load_db2!(app, SpellCategory);
    load_db2!(app, SpellClassOptions);
    load_db2!(app, SpellCooldowns);
    load_db2!(app, SpellDuration);
    load_db2!(app, SpellEffect);
    load_db2!(app, SpellEquippedItems);
    load_db2!(app, SpellFocusObject);
    load_db2!(app, SpellInterrupts);
    load_db2!(app, SpellItemEnchantment);
    load_db2!(app, SpellItemEnchantmentCondition);
    load_db2!(app, SpellLearnSpell);
    load_db2!(app, SpellLevels);
    load_db2!(app, SpellMisc);
    load_db2!(app, SpellPower);
    load_db2!(app, SpellPowerDifficulty);
    load_db2!(app, SpellProcsPerMinute);
    load_db2!(app, SpellProcsPerMinuteMod);
    load_db2!(app, SpellRadius);
    load_db2!(app, SpellRange);
    load_db2!(app, SpellReagents);
    load_db2!(app, SpellScaling);
    load_db2!(app, SpellShapeshift);
    load_db2!(app, SpellShapeshiftForm);
    load_db2!(app, SpellTargetRestrictions);
    load_db2!(app, SpellTotems);
    load_db2!(app, SpellXSpellVisual);
    load_db2!(app, SummonProperties);
    load_db2!(app, TactKey);
    load_db2!(app, Talent);
    load_db2!(app, TaxiNodes);
    load_db2!(app, TaxiPath);
    load_db2!(app, TaxiPathNode);
    load_db2!(app, TotemCategory);
    load_db2!(app, Toy);
    load_db2!(app, TransmogHoliday);
    load_db2!(app, TransmogSet);
    load_db2!(app, TransmogSetGroup);
    load_db2!(app, TransmogSetItem);
    load_db2!(app, TransportAnimation);
    load_db2!(app, TransportRotation);
    load_db2!(app, UnitPowerBar);
    load_db2!(app, Vehicle);
    load_db2!(app, VehicleSeat);
    load_db2!(app, WMOAreaTable);
    load_db2!(app, WorldEffect);
    load_db2!(app, WorldMapArea);
    load_db2!(app, WorldMapOverlay);
    load_db2!(app, WorldMapTransforms);
    load_db2!(app, WorldSafeLocs);

    app.add_systems(Startup, load_db2_store_after.in_set(DB2StoresMgrSet::SetupStoreHelpers));
    app.configure_sets(
        Startup,
        ((
            DB2StoresMgrSet::Start,
            DB2StoresMgrSet::LoadStores,
            DB2StoresMgrSet::SetupStoreHelpers.run_if(not(on_event::<AzStartupFailedEvent>)),
        )
            .chain()
            .in_set(InitDB2MgrSet),),
    );
}

#[derive(SystemParam)]
pub struct DB2Stores<'w> {
    pub achievement_store: Res<'w, DB2Storage<Achievement>>,
    pub anim_kit_store: Res<'w, DB2Storage<AnimKit>>,
    pub area_group_member_store: Res<'w, DB2Storage<AreaGroupMember>>,
    pub area_table_store: Res<'w, DB2Storage<AreaTable>>,
    pub area_trigger_store: Res<'w, DB2Storage<AreaTrigger>>,
    pub armor_location_store: Res<'w, DB2Storage<ArmorLocation>>,
    pub artifact_store: Res<'w, DB2Storage<Artifact>>,
    pub artifact_appearance_store: Res<'w, DB2Storage<ArtifactAppearance>>,
    pub artifact_appearance_set_store: Res<'w, DB2Storage<ArtifactAppearanceSet>>,
    pub artifact_category_store: Res<'w, DB2Storage<ArtifactCategory>>,
    pub artifact_power_store: Res<'w, DB2Storage<ArtifactPower>>,
    pub artifact_power_link_store: Res<'w, DB2Storage<ArtifactPowerLink>>,
    pub artifact_power_picker_store: Res<'w, DB2Storage<ArtifactPowerPicker>>,
    pub artifact_power_rank_store: Res<'w, DB2Storage<ArtifactPowerRank>>,
    pub artifact_tier_store: Res<'w, DB2Storage<ArtifactTier>>,
    pub artifact_unlock_store: Res<'w, DB2Storage<ArtifactUnlock>>,
    pub auction_house_store: Res<'w, DB2Storage<AuctionHouse>>,
    pub bank_bag_slot_prices_store: Res<'w, DB2Storage<BankBagSlotPrices>>,
    pub banned_addons_store: Res<'w, DB2Storage<BannedAddons>>,
    pub barber_shop_style_store: Res<'w, DB2Storage<BarberShopStyle>>,
    pub battle_pet_breed_quality_store: Res<'w, DB2Storage<BattlePetBreedQuality>>,
    pub battle_pet_breed_state_store: Res<'w, DB2Storage<BattlePetBreedState>>,
    pub battle_pet_species_store: Res<'w, DB2Storage<BattlePetSpecies>>,
    pub battle_pet_species_state_store: Res<'w, DB2Storage<BattlePetSpeciesState>>,
    pub battlemaster_list_store: Res<'w, DB2Storage<BattlemasterList>>,
    pub broadcast_text_store: Res<'w, DB2Storage<BroadcastText>>,
    pub cfg_regions_store: Res<'w, DB2Storage<Cfg_Regions>>,
    pub character_facial_hair_styles_store: Res<'w, DB2Storage<CharacterFacialHairStyles>>,
    pub char_base_section_store: Res<'w, DB2Storage<CharBaseSection>>,
    pub char_sections_store: Res<'w, DB2Storage<CharSections>>,
    pub char_start_outfit_store: Res<'w, DB2Storage<CharStartOutfit>>,
    pub char_titles_store: Res<'w, DB2Storage<CharTitles>>,
    pub chat_channels_store: Res<'w, DB2Storage<ChatChannels>>,
    pub chr_classes_store: Res<'w, DB2Storage<ChrClasses>>,
    pub chr_classes_x_power_types_store: Res<'w, DB2Storage<ChrClassesXPowerTypes>>,
    pub chr_races_store: Res<'w, DB2Storage<ChrRaces>>,
    pub chr_specialization_store: Res<'w, DB2Storage<ChrSpecialization>>,
    pub cinematic_camera_store: Res<'w, DB2Storage<CinematicCamera>>,
    pub cinematic_sequences_store: Res<'w, DB2Storage<CinematicSequences>>,
    pub conversation_line_store: Res<'w, DB2Storage<ConversationLine>>,
    pub creature_display_info_store: Res<'w, DB2Storage<CreatureDisplayInfo>>,
    pub creature_display_info_extra_store: Res<'w, DB2Storage<CreatureDisplayInfoExtra>>,
    pub creature_family_store: Res<'w, DB2Storage<CreatureFamily>>,
    pub creature_model_data_store: Res<'w, DB2Storage<CreatureModelData>>,
    pub creature_type_store: Res<'w, DB2Storage<CreatureType>>,
    pub criteria_store: Res<'w, DB2Storage<Criteria>>,
    pub criteria_tree_store: Res<'w, DB2Storage<CriteriaTree>>,
    pub currency_types_store: Res<'w, DB2Storage<CurrencyTypes>>,
    pub curve_store: Res<'w, DB2Storage<Curve>>,
    pub curve_point_store: Res<'w, DB2Storage<CurvePoint>>,
    pub destructible_model_data_store: Res<'w, DB2Storage<DestructibleModelData>>,
    pub difficulty_store: Res<'w, DB2Storage<Difficulty>>,
    pub dungeon_encounter_store: Res<'w, DB2Storage<DungeonEncounter>>,
    pub durability_costs_store: Res<'w, DB2Storage<DurabilityCosts>>,
    pub durability_quality_store: Res<'w, DB2Storage<DurabilityQuality>>,
    pub emotes_store: Res<'w, DB2Storage<Emotes>>,
    pub emotes_text_store: Res<'w, DB2Storage<EmotesText>>,
    pub emotes_text_sound_store: Res<'w, DB2Storage<EmotesTextSound>>,
    pub faction_store: Res<'w, DB2Storage<Faction>>,
    pub faction_template_store: Res<'w, DB2Storage<FactionTemplate>>,
    pub game_objects_store: Res<'w, DB2Storage<GameObjects>>,
    pub game_object_display_info_store: Res<'w, DB2Storage<GameObjectDisplayInfo>>,
    pub garr_ability_store: Res<'w, DB2Storage<GarrAbility>>,
    pub garr_building_store: Res<'w, DB2Storage<GarrBuilding>>,
    pub garr_building_plot_inst_store: Res<'w, DB2Storage<GarrBuildingPlotInst>>,
    pub garr_class_spec_store: Res<'w, DB2Storage<GarrClassSpec>>,
    pub garr_follower_store: Res<'w, DB2Storage<GarrFollower>>,
    pub garr_follower_x_ability_store: Res<'w, DB2Storage<GarrFollowerXAbility>>,
    pub garr_plot_building_store: Res<'w, DB2Storage<GarrPlotBuilding>>,
    pub garr_plot_store: Res<'w, DB2Storage<GarrPlot>>,
    pub garr_plot_instance_store: Res<'w, DB2Storage<GarrPlotInstance>>,
    pub garr_site_level_store: Res<'w, DB2Storage<GarrSiteLevel>>,
    pub garr_site_level_plot_inst_store: Res<'w, DB2Storage<GarrSiteLevelPlotInst>>,
    pub gem_properties_store: Res<'w, DB2Storage<GemProperties>>,
    pub glyph_bindable_spell_store: Res<'w, DB2Storage<GlyphBindableSpell>>,
    pub glyph_properties_store: Res<'w, DB2Storage<GlyphProperties>>,
    pub glyph_required_spec_store: Res<'w, DB2Storage<GlyphRequiredSpec>>,
    pub guild_color_background_store: Res<'w, DB2Storage<GuildColorBackground>>,
    pub guild_color_border_store: Res<'w, DB2Storage<GuildColorBorder>>,
    pub guild_color_emblem_store: Res<'w, DB2Storage<GuildColorEmblem>>,
    pub guild_perk_spells_store: Res<'w, DB2Storage<GuildPerkSpells>>,
    pub heirloom_store: Res<'w, DB2Storage<Heirloom>>,
    pub holidays_store: Res<'w, DB2Storage<Holidays>>,
    pub import_price_armor_store: Res<'w, DB2Storage<ImportPriceArmor>>,
    pub import_price_quality_store: Res<'w, DB2Storage<ImportPriceQuality>>,
    pub import_price_shield_store: Res<'w, DB2Storage<ImportPriceShield>>,
    pub import_price_weapon_store: Res<'w, DB2Storage<ImportPriceWeapon>>,
    pub item_appearance_store: Res<'w, DB2Storage<ItemAppearance>>,
    pub item_armor_quality_store: Res<'w, DB2Storage<ItemArmorQuality>>,
    pub item_armor_shield_store: Res<'w, DB2Storage<ItemArmorShield>>,
    pub item_armor_total_store: Res<'w, DB2Storage<ItemArmorTotal>>,
    pub item_bag_family_store: Res<'w, DB2Storage<ItemBagFamily>>,
    pub item_bonus_store: Res<'w, DB2Storage<ItemBonus>>,
    pub item_bonus_list_level_delta_store: Res<'w, DB2Storage<ItemBonusListLevelDelta>>,
    pub item_bonus_tree_node_store: Res<'w, DB2Storage<ItemBonusTreeNode>>,
    pub item_child_equipment_store: Res<'w, DB2Storage<ItemChildEquipment>>,
    pub item_class_store: Res<'w, DB2Storage<ItemClass>>,
    pub item_currency_cost_store: Res<'w, DB2Storage<ItemCurrencyCost>>,
    pub item_damage_ammo_store: Res<'w, DB2Storage<ItemDamageAmmo>>,
    pub item_damage_one_hand_store: Res<'w, DB2Storage<ItemDamageOneHand>>,
    pub item_damage_one_hand_caster_store: Res<'w, DB2Storage<ItemDamageOneHandCaster>>,
    pub item_damage_two_hand_store: Res<'w, DB2Storage<ItemDamageTwoHand>>,
    pub item_damage_two_hand_caster_store: Res<'w, DB2Storage<ItemDamageTwoHandCaster>>,
    pub item_disenchant_loot_store: Res<'w, DB2Storage<ItemDisenchantLoot>>,
    pub item_effect_store: Res<'w, DB2Storage<ItemEffect>>,
    pub item_store: Res<'w, DB2Storage<Item>>,
    pub item_extended_cost_store: Res<'w, DB2Storage<ItemExtendedCost>>,
    pub item_level_selector_store: Res<'w, DB2Storage<ItemLevelSelector>>,
    pub item_level_selector_quality_store: Res<'w, DB2Storage<ItemLevelSelectorQuality>>,
    pub item_level_selector_quality_set_store: Res<'w, DB2Storage<ItemLevelSelectorQualitySet>>,
    pub item_limit_category_store: Res<'w, DB2Storage<ItemLimitCategory>>,
    pub item_limit_category_condition_store: Res<'w, DB2Storage<ItemLimitCategoryCondition>>,
    pub item_modified_appearance_store: Res<'w, DB2Storage<ItemModifiedAppearance>>,
    pub item_price_base_store: Res<'w, DB2Storage<ItemPriceBase>>,
    pub item_random_properties_store: Res<'w, DB2Storage<ItemRandomProperties>>,
    pub item_random_suffix_store: Res<'w, DB2Storage<ItemRandomSuffix>>,
    pub item_search_name_store: Res<'w, DB2Storage<ItemSearchName>>,
    pub item_set_store: Res<'w, DB2Storage<ItemSet>>,
    pub item_set_spell_store: Res<'w, DB2Storage<ItemSetSpell>>,
    pub item_sparse_store: Res<'w, DB2Storage<ItemSparse>>,
    pub item_spec_store: Res<'w, DB2Storage<ItemSpec>>,
    pub item_spec_override_store: Res<'w, DB2Storage<ItemSpecOverride>>,
    pub item_upgrade_store: Res<'w, DB2Storage<ItemUpgrade>>,
    pub item_x_bonus_tree_store: Res<'w, DB2Storage<ItemXBonusTree>>,
    pub keychain_store: Res<'w, DB2Storage<Keychain>>,
    pub lfg_dungeons_store: Res<'w, DB2Storage<LFGDungeons>>,
    pub light_store: Res<'w, DB2Storage<Light>>,
    pub liquid_type_store: Res<'w, DB2Storage<LiquidType>>,
    pub lock_store: Res<'w, DB2Storage<Lock>>,
    pub mail_template_store: Res<'w, DB2Storage<MailTemplate>>,
    pub map_store: Res<'w, DB2Storage<Map>>,
    pub map_difficulty_store: Res<'w, DB2Storage<MapDifficulty>>,
    pub modifier_tree_store: Res<'w, DB2Storage<ModifierTree>>,
    pub mount_capability_store: Res<'w, DB2Storage<MountCapability>>,
    pub mount_store: Res<'w, DB2Storage<Mount>>,
    pub mount_type_x_capability_store: Res<'w, DB2Storage<MountTypeXCapability>>,
    pub mount_x_display_store: Res<'w, DB2Storage<MountXDisplay>>,
    pub movie_store: Res<'w, DB2Storage<Movie>>,
    pub name_gen_store: Res<'w, DB2Storage<NameGen>>,
    pub names_profanity_store: Res<'w, DB2Storage<NamesProfanity>>,
    pub names_reserved_store: Res<'w, DB2Storage<NamesReserved>>,
    pub names_reserved_locale_store: Res<'w, DB2Storage<NamesReservedLocale>>,
    pub override_spell_data_store: Res<'w, DB2Storage<OverrideSpellData>>,
    pub phase_store: Res<'w, DB2Storage<Phase>>,
    pub phase_x_phase_group_store: Res<'w, DB2Storage<PhaseXPhaseGroup>>,
    pub player_condition_store: Res<'w, DB2Storage<PlayerCondition>>,
    pub power_display_store: Res<'w, DB2Storage<PowerDisplay>>,
    pub power_type_store: Res<'w, DB2Storage<PowerType>>,
    pub prestige_level_info_store: Res<'w, DB2Storage<PrestigeLevelInfo>>,
    pub pvp_difficulty_store: Res<'w, DB2Storage<PVPDifficulty>>,
    pub pvp_item_store: Res<'w, DB2Storage<PVPItem>>,
    pub pvp_reward_store: Res<'w, DB2Storage<PvpReward>>,
    pub pvp_talent_store: Res<'w, DB2Storage<PvpTalent>>,
    pub pvp_talent_unlock_store: Res<'w, DB2Storage<PvpTalentUnlock>>,
    pub quest_faction_reward_store: Res<'w, DB2Storage<QuestFactionReward>>,
    pub quest_money_reward_store: Res<'w, DB2Storage<QuestMoneyReward>>,
    pub quest_package_item_store: Res<'w, DB2Storage<QuestPackageItem>>,
    pub quest_sort_store: Res<'w, DB2Storage<QuestSort>>,
    pub quest_v2_store: Res<'w, DB2Storage<QuestV2>>,
    pub quest_xp_store: Res<'w, DB2Storage<QuestXP>>,
    pub rand_prop_points_store: Res<'w, DB2Storage<RandPropPoints>>,
    pub reward_pack_store: Res<'w, DB2Storage<RewardPack>>,
    pub reward_pack_x_currency_type_store: Res<'w, DB2Storage<RewardPackXCurrencyType>>,
    pub reward_pack_x_item_store: Res<'w, DB2Storage<RewardPackXItem>>,
    pub ruleset_item_upgrade_store: Res<'w, DB2Storage<RulesetItemUpgrade>>,
    pub sandbox_scaling_store: Res<'w, DB2Storage<SandboxScaling>>,
    pub scaling_stat_distribution_store: Res<'w, DB2Storage<ScalingStatDistribution>>,
    pub scenario_store: Res<'w, DB2Storage<Scenario>>,
    pub scenario_step_store: Res<'w, DB2Storage<ScenarioStep>>,
    pub scene_script_store: Res<'w, DB2Storage<SceneScript>>,
    pub scene_script_global_text_store: Res<'w, DB2Storage<SceneScriptGlobalText>>,
    pub scene_script_package_store: Res<'w, DB2Storage<SceneScriptPackage>>,
    pub scene_script_text_store: Res<'w, DB2Storage<SceneScriptText>>,
    pub skill_line_store: Res<'w, DB2Storage<SkillLine>>,
    pub skill_line_ability_store: Res<'w, DB2Storage<SkillLineAbility>>,
    pub skill_race_class_info_store: Res<'w, DB2Storage<SkillRaceClassInfo>>,
    pub sound_kit_store: Res<'w, DB2Storage<SoundKit>>,
    pub specialization_spells_store: Res<'w, DB2Storage<SpecializationSpells>>,
    pub spell_store: Res<'w, DB2Storage<Spell>>,
    pub spell_aura_options_store: Res<'w, DB2Storage<SpellAuraOptions>>,
    pub spell_aura_restrictions_store: Res<'w, DB2Storage<SpellAuraRestrictions>>,
    pub spell_cast_times_store: Res<'w, DB2Storage<SpellCastTimes>>,
    pub spell_casting_requirements_store: Res<'w, DB2Storage<SpellCastingRequirements>>,
    pub spell_categories_store: Res<'w, DB2Storage<SpellCategories>>,
    pub spell_category_store: Res<'w, DB2Storage<SpellCategory>>,
    pub spell_class_options_store: Res<'w, DB2Storage<SpellClassOptions>>,
    pub spell_cooldowns_store: Res<'w, DB2Storage<SpellCooldowns>>,
    pub spell_duration_store: Res<'w, DB2Storage<SpellDuration>>,
    pub spell_effect_store: Res<'w, DB2Storage<SpellEffect>>,
    pub spell_equipped_items_store: Res<'w, DB2Storage<SpellEquippedItems>>,
    pub spell_focus_object_store: Res<'w, DB2Storage<SpellFocusObject>>,
    pub spell_interrupts_store: Res<'w, DB2Storage<SpellInterrupts>>,
    pub spell_item_enchantment_store: Res<'w, DB2Storage<SpellItemEnchantment>>,
    pub spell_item_enchantment_condition_store: Res<'w, DB2Storage<SpellItemEnchantmentCondition>>,
    pub spell_learn_spell_store: Res<'w, DB2Storage<SpellLearnSpell>>,
    pub spell_levels_store: Res<'w, DB2Storage<SpellLevels>>,
    pub spell_misc_store: Res<'w, DB2Storage<SpellMisc>>,
    pub spell_power_store: Res<'w, DB2Storage<SpellPower>>,
    pub spell_power_difficulty_store: Res<'w, DB2Storage<SpellPowerDifficulty>>,
    pub spell_procs_per_minute_store: Res<'w, DB2Storage<SpellProcsPerMinute>>,
    pub spell_procs_per_minute_mod_store: Res<'w, DB2Storage<SpellProcsPerMinuteMod>>,
    pub spell_radius_store: Res<'w, DB2Storage<SpellRadius>>,
    pub spell_range_store: Res<'w, DB2Storage<SpellRange>>,
    pub spell_reagents_store: Res<'w, DB2Storage<SpellReagents>>,
    pub spell_scaling_store: Res<'w, DB2Storage<SpellScaling>>,
    pub spell_shapeshift_store: Res<'w, DB2Storage<SpellShapeshift>>,
    pub spell_shapeshift_form_store: Res<'w, DB2Storage<SpellShapeshiftForm>>,
    pub spell_target_restrictions_store: Res<'w, DB2Storage<SpellTargetRestrictions>>,
    pub spell_totems_store: Res<'w, DB2Storage<SpellTotems>>,
    pub spell_x_spell_visual_store: Res<'w, DB2Storage<SpellXSpellVisual>>,
    pub summon_properties_store: Res<'w, DB2Storage<SummonProperties>>,
    pub tact_key_store: Res<'w, DB2Storage<TactKey>>,
    pub talent_store: Res<'w, DB2Storage<Talent>>,
    pub taxi_nodes_store: Res<'w, DB2Storage<TaxiNodes>>,
    pub taxi_path_store: Res<'w, DB2Storage<TaxiPath>>,
    pub taxi_path_node_store: Res<'w, DB2Storage<TaxiPathNode>>,
    pub totem_category_store: Res<'w, DB2Storage<TotemCategory>>,
    pub toy_store: Res<'w, DB2Storage<Toy>>,
    pub transmog_holiday_store: Res<'w, DB2Storage<TransmogHoliday>>,
    pub transmog_set_store: Res<'w, DB2Storage<TransmogSet>>,
    pub transmog_set_group_store: Res<'w, DB2Storage<TransmogSetGroup>>,
    pub transmog_set_item_store: Res<'w, DB2Storage<TransmogSetItem>>,
    pub transport_animation_store: Res<'w, DB2Storage<TransportAnimation>>,
    pub transport_rotation_store: Res<'w, DB2Storage<TransportRotation>>,
    pub unit_power_bar_store: Res<'w, DB2Storage<UnitPowerBar>>,
    pub vehicle_store: Res<'w, DB2Storage<Vehicle>>,
    pub vehicle_seat_store: Res<'w, DB2Storage<VehicleSeat>>,
    pub wmo_area_table_store: Res<'w, DB2Storage<WMOAreaTable>>,
    pub world_effect_store: Res<'w, DB2Storage<WorldEffect>>,
    pub world_map_area_store: Res<'w, DB2Storage<WorldMapArea>>,
    pub world_map_overlay_store: Res<'w, DB2Storage<WorldMapOverlay>>,
    pub world_map_transforms_store: Res<'w, DB2Storage<WorldMapTransforms>>,
    pub world_safe_locs_store: Res<'w, DB2Storage<WorldSafeLocs>>,
}

impl DB2Storage<WorldMapTransforms> {
    /// DB2Manager::DeterminaAlternateMapPosition in TC
    fn determine_alternate_map_position(&self, map_id: u32, mut x: f32, mut y: f32, z: f32) -> (u32, Vector2<f32>) {
        let mut transformation = None::<&WorldMapTransforms>;
        for e in self.values() {
            if map_id != u32::from(e.map_id) {
                continue;
            }
            if e.area_id > 0 {
                continue;
            }
            if e.flags & FlagSet::from(WorldMapTransformsFlags::Dungeon).bits() > 0 {
                continue;
            }
            let region_min = &e.region_min_max[..3];
            let region_max = &e.region_min_max[3..];
            if region_min[0] > x || region_max[0] < x {
                continue;
            }
            if region_min[1] > y || region_max[1] < y {
                continue;
            }
            if region_min[2] > z || region_max[2] < z {
                continue;
            }
            if transformation.is_none() || transformation.is_some_and(|t| t.priority < e.priority) {
                transformation = Some(e);
            }
        }
        let Some(transformation) = transformation else {
            return (map_id, Vector2::new(x, y));
        };
        let region_min = &transformation.region_min_max[..3];
        let new_map_id = transformation.new_map_id.into();
        if (transformation.region_scale - 1.0).abs() > 0.001 {
            x = (x - region_min[0]) * transformation.region_scale + region_min[0];
            y = (y - region_min[1]) * transformation.region_scale + region_min[1];
        }
        let new_pos = Vector2::new(x + transformation.region_offset[0], y + transformation.region_offset[1]);

        (new_map_id, new_pos)
    }
}

#[derive(Resource, Default)]
pub struct AreaGroupMembers(pub BTreeMap<u32, Vec<u32>>);
deref_boilerplate!(AreaGroupMembers, BTreeMap<u32, Vec<u32>>, 0);
#[derive(Resource, Default)]
pub struct ArtifactPowers(pub BTreeMap<u32, Vec<ArtifactPower>>);
deref_boilerplate!(ArtifactPowers, BTreeMap<u32, Vec<ArtifactPower>>, 0);
#[derive(Resource, Default)]
pub struct ArtifactPowerLinks(pub BTreeMap<u32, BTreeSet<u32>>);
deref_boilerplate!(ArtifactPowerLinks, BTreeMap<u32, BTreeSet<u32>>, 0);

#[derive(Resource, Default)]
pub struct ArtifactPowerRanks(pub BTreeMap<(u32, u8), ArtifactPowerRank>);
deref_boilerplate!(ArtifactPowerRanks, BTreeMap<(u32, u8), ArtifactPowerRank>, 0);

#[derive(Resource, Default)]
pub struct CharFacialHairStyles(pub BTreeSet<(u8, u8, u32)>);
deref_boilerplate!(CharFacialHairStyles, BTreeSet<(u8, u8, u32)>, 0);

#[derive(Resource, Default)]
pub struct CharsSections(pub BTreeMap<(u8, u8, CharBaseSectionVariation), Vec<CharSections>>);
deref_boilerplate!(CharsSections, BTreeMap<(u8, u8, CharBaseSectionVariation), Vec<CharSections>>, 0);

#[derive(Resource, Default)]
pub struct CharStartOutfits(pub BTreeMap<(u8, u8, u8), CharStartOutfit>);
deref_boilerplate!(CharStartOutfits, BTreeMap<(u8, u8, u8), CharStartOutfit>, 0);

#[derive(Resource, Default)]
pub struct PowersByClass(pub BTreeMap<Class, BTreeMap<Power, u32>>);
deref_boilerplate!(PowersByClass, BTreeMap<Class, BTreeMap<Power, u32>>, 0);

#[derive(Resource, Default)]
pub struct ChrSpecializationByIndexContainer(pub BTreeMap<Class, BTreeMap<i8, ChrSpecialization>>);
deref_boilerplate!(ChrSpecializationByIndexContainer, BTreeMap<Class, BTreeMap<i8, ChrSpecialization>>, 0);

#[derive(Resource, Default)]
pub struct CurvePointsContainer(pub BTreeMap<u32, Vec<CurvePoint>>);
deref_boilerplate!(CurvePointsContainer, BTreeMap<u32, Vec<CurvePoint>>, 0);

#[derive(Resource, Default)]
pub struct EmotesTextSoundContainer(pub BTreeMap<(u32, Race, Gender, Class), EmotesTextSound>);
deref_boilerplate!(EmotesTextSoundContainer, BTreeMap<(u32, Race, Gender, Class), EmotesTextSound>, 0);

#[derive(Resource, Default)]
pub struct FactionTeamContainer(pub BTreeMap<u32, Vec<u32>>);
deref_boilerplate!(FactionTeamContainer, BTreeMap<u32, Vec<u32>>, 0);

#[derive(Resource, Default)]
pub struct HeirloomItemsContainer(pub BTreeMap<u32, Heirloom>);
deref_boilerplate!(HeirloomItemsContainer, BTreeMap<u32, Heirloom>, 0);

#[derive(Resource, Default)]
pub struct GlyphBindableSpellsContainer(pub BTreeMap<u32 /*glyphPropertiesId*/, Vec<u32>>);
deref_boilerplate!(GlyphBindableSpellsContainer, BTreeMap<u32 /*glyphPropertiesId*/, Vec<u32>>, 0);

#[derive(Resource, Default)]
pub struct GlyphRequiredSpecsContainer(pub BTreeMap<u32 /*glyphPropertiesId*/, Vec<u32>>);
deref_boilerplate!(GlyphRequiredSpecsContainer, BTreeMap<u32 /*glyphPropertiesId*/, Vec<u32>>, 0);

#[derive(Resource, Default)]
pub struct ItemBonusListContainer(pub BTreeMap<u32 /*bonusListId*/, Vec<ItemBonus>>);
deref_boilerplate!(ItemBonusListContainer, BTreeMap<u32 /*bonusListId*/, Vec<ItemBonus>>, 0);

#[derive(Resource, Default)]
pub struct ItemBonusListLevelDeltaContainer(pub BTreeMap<i16, u32>);
deref_boilerplate!(ItemBonusListLevelDeltaContainer, BTreeMap<i16, u32>, 0);

#[derive(Resource, Default)]
pub struct ItemBonusTreeContainer(pub BTreeMap<u32, BTreeSet<ItemBonusTreeNode>>);
deref_boilerplate!(ItemBonusTreeContainer, BTreeMap<u32, BTreeSet<ItemBonusTreeNode>>, 0);

#[derive(Resource, Default)]
pub struct ItemChildEquipmentContainer(pub BTreeMap<u32 /*itemId*/, ItemChildEquipment>);
deref_boilerplate!(ItemChildEquipmentContainer, BTreeMap<u32 /*itemId*/, ItemChildEquipment>, 0);

#[derive(Resource, Default)]
pub struct ItemClassByOldEnumContainer(pub BTreeMap<ItemClassID, ItemClass>);
deref_boilerplate!(ItemClassByOldEnumContainer, BTreeMap<ItemClassID, ItemClass>, 0);

#[derive(Resource, Default)]
pub struct ItemIDsWithCurrencyCost(pub BTreeSet<u32>);
deref_boilerplate!(ItemIDsWithCurrencyCost, BTreeSet<u32>, 0);

#[derive(Resource, Default)]
pub struct ItemLimitCategoryConditionContainer(pub BTreeMap<u32, Vec<ItemLimitCategoryCondition>>);
deref_boilerplate!(ItemLimitCategoryConditionContainer, BTreeMap<u32, Vec<ItemLimitCategoryCondition>>, 0);

#[derive(Resource, Default)]
pub struct ItemLevelSelectorQualityContainer(pub BTreeMap<u32 /*itemLevelSelectorQualitySetId*/, BTreeSet<ItemLevelSelectorQuality>>);
deref_boilerplate!(ItemLevelSelectorQualityContainer, BTreeMap<u32 /*itemLevelSelectorQualitySetId*/, BTreeSet<ItemLevelSelectorQuality>>, 0);

#[derive(Resource, Default)]
pub struct ItemModifiedAppearanceByItemContainer(pub BTreeMap<(u32, u8) /*itemId | appearanceMod << 24*/, ItemModifiedAppearance>);
deref_boilerplate!(ItemModifiedAppearanceByItemContainer, BTreeMap<(u32, u8) /*itemId | appearanceMod << 24*/, ItemModifiedAppearance>, 0);

#[derive(Resource, Default)]
pub struct ItemToBonusTreeContainer(pub BTreeMap<u32 /*itemId*/, Vec<u32> /*bonusTreeId*/>);
deref_boilerplate!(ItemToBonusTreeContainer, BTreeMap<u32 /*itemId*/, Vec<u32> /*bonusTreeId*/>, 0);

#[derive(Resource, Default)]
pub struct ItemSetSpellContainer(pub BTreeMap<u32, Vec<ItemSetSpell>>);
deref_boilerplate!(ItemSetSpellContainer, BTreeMap<u32, Vec<ItemSetSpell>>, 0);

#[derive(Resource, Default)]
pub struct ItemSpecOverridesContainer(pub BTreeMap<u32, Vec<ItemSpecOverride>>);
deref_boilerplate!(ItemSpecOverridesContainer, BTreeMap<u32, Vec<ItemSpecOverride>>, 0);

#[derive(Resource, Default)]
pub struct MapDifficultyContainer(pub BTreeMap<u32, BTreeMap<u8, MapDifficulty>>);
deref_boilerplate!(MapDifficultyContainer, BTreeMap<u32, BTreeMap<u8, MapDifficulty>>, 0);

#[derive(Resource, Default)]
pub struct MountsBySpellIDContainer(pub BTreeMap<u32, Mount>);
deref_boilerplate!(MountsBySpellIDContainer, BTreeMap<u32, Mount>, 0);

#[derive(Resource, Default)]
pub struct MountCapabilitiesByTypeContainer(pub BTreeMap<u32, BTreeSet<MountTypeXCapability>>);
deref_boilerplate!(MountCapabilitiesByTypeContainer, BTreeMap<u32, BTreeSet<MountTypeXCapability>>, 0);

#[derive(Resource, Default)]
pub struct MountDisplaysContainer(pub BTreeMap<u32, Vec<MountXDisplay>>);
deref_boilerplate!(MountDisplaysContainer, BTreeMap<u32, Vec<MountXDisplay>>, 0);

#[derive(Resource, Default)]
pub struct NameGenContainer(pub BTreeMap<Race, BTreeMap<Gender, NameGen>>);
deref_boilerplate!(NameGenContainer, BTreeMap<Race, BTreeMap<Gender, NameGen>>, 0);

#[derive(Resource, Default)]
pub struct NameValidationRegexContainer(pub BTreeMap<Locale, Vec<Regex>>);
deref_boilerplate!(NameValidationRegexContainer, BTreeMap<Locale, Vec<Regex>>, 0);

#[derive(Resource, Default)]
pub struct NameReservedRegexContainer(pub BTreeMap<Locale, Vec<Regex>>);
deref_boilerplate!(NameReservedRegexContainer, BTreeMap<Locale, Vec<Regex>>, 0);

#[derive(Resource, Default)]
pub struct PhaseGroupContainer(pub BTreeMap<u32, Vec<u32>>);
deref_boilerplate!(PhaseGroupContainer, BTreeMap<u32, Vec<u32>>, 0);

#[derive(Resource, Default)]
pub struct PowerTypesContainer(pub BTreeMap<Power, PowerType>);
deref_boilerplate!(PowerTypesContainer, BTreeMap<Power, PowerType>, 0);

#[derive(Resource, Default)]
pub struct PvpItemBonusContainer(pub BTreeMap<u32, u8>);
deref_boilerplate!(PvpItemBonusContainer, BTreeMap<u32, u8>, 0);

#[derive(Resource, Default)]
pub struct PvpRewardPackContainer(pub BTreeMap<(u32 /*prestige level*/, u32 /*honor level*/), u32>);
deref_boilerplate!(PvpRewardPackContainer, BTreeMap<(u32 /*prestige level*/, u32 /*honor level*/), u32>, 0);

#[derive(Resource, Default)]
pub struct PvpTalentsByPosition(pub BTreeMap<Class, BTreeMap<u32, BTreeMap<u32, Vec<PvpTalent>>>>);
deref_boilerplate!(PvpTalentsByPosition, BTreeMap<Class, BTreeMap<u32, BTreeMap<u32, Vec<PvpTalent>>>>, 0);

#[derive(Resource, Default)]
pub struct PvpTalentUnlockHonourLevel(pub BTreeMap<u32, BTreeMap<u32, u32>>);
deref_boilerplate!(PvpTalentUnlockHonourLevel, BTreeMap<u32, BTreeMap<u32, u32>>, 0);

#[derive(Resource, Default)]
pub struct QuestPackageItemContainer(pub BTreeMap<u32, (Vec<QuestPackageItem>, Vec<QuestPackageItem>)>);
deref_boilerplate!(QuestPackageItemContainer, BTreeMap<u32, (Vec<QuestPackageItem>, Vec<QuestPackageItem>)>, 0);

#[derive(Resource, Default)]
pub struct RewardPackCurrencyTypes(pub BTreeMap<u32, Vec<RewardPackXCurrencyType>>);
deref_boilerplate!(RewardPackCurrencyTypes, BTreeMap<u32, Vec<RewardPackXCurrencyType>>, 0);

#[derive(Resource, Default)]
pub struct RewardPackItems(pub BTreeMap<u32, Vec<RewardPackXItem>>);
deref_boilerplate!(RewardPackItems, BTreeMap<u32, Vec<RewardPackXItem>>, 0);

#[derive(Resource, Default)]
pub struct RulesetItemUpgradeContainer(pub BTreeMap<u32, u32>);
deref_boilerplate!(RulesetItemUpgradeContainer, BTreeMap<u32, u32>, 0);

#[derive(Resource, Default)]
pub struct SkillRaceClassInfoContainer(pub BTreeMap<u32, Vec<SkillRaceClassInfo>>);
deref_boilerplate!(SkillRaceClassInfoContainer, BTreeMap<u32, Vec<SkillRaceClassInfo>>, 0);

#[derive(Resource, Default)]
pub struct SpecializationSpellsContainer(pub BTreeMap<u32, Vec<SpecializationSpells>>);
deref_boilerplate!(SpecializationSpellsContainer, BTreeMap<u32, Vec<SpecializationSpells>>, 0);

#[derive(Resource, Default)]
pub struct SpellClassOptionsSets(pub BTreeSet<u8>);
deref_boilerplate!(SpellClassOptionsSets, BTreeSet<u8>, 0);

#[derive(Resource, Default)]
pub struct SpellPowerContainer(pub BTreeMap<u32, BTreeMap<u8, SpellPower>>);
deref_boilerplate!(SpellPowerContainer, BTreeMap<u32, BTreeMap<u8, SpellPower>>, 0);

#[derive(Resource, Default)]
pub struct SpellPowerDifficultyContainer(pub BTreeMap<u32, BTreeMap<u32, BTreeMap<u8, SpellPower>>>);
deref_boilerplate!(SpellPowerDifficultyContainer, BTreeMap<u32, BTreeMap<u32, BTreeMap<u8, SpellPower>>>, 0);

#[derive(Resource, Default)]
pub struct SpellProcsPerMinuteModContainer(pub BTreeMap<u32, Vec<SpellProcsPerMinuteMod>>);
deref_boilerplate!(SpellProcsPerMinuteModContainer, BTreeMap<u32, Vec<SpellProcsPerMinuteMod>>, 0);

#[derive(Resource, Default)]
pub struct TalentsByPosition(pub BTreeMap<Class, BTreeMap<u32, BTreeMap<u32, Vec<Talent>>>>);
deref_boilerplate!(TalentsByPosition, BTreeMap<Class, BTreeMap<u32, BTreeMap<u32, Vec<Talent>>>>, 0);

#[derive(Resource, Default)]
pub struct ToyItemIdsContainer(pub BTreeSet<u32>);
deref_boilerplate!(ToyItemIdsContainer, BTreeSet<u32>, 0);

#[derive(Resource, Default)]
pub struct TransmogSetsByItemModifiedAppearance(pub BTreeMap<u32, Vec<TransmogSet>>);
deref_boilerplate!(TransmogSetsByItemModifiedAppearance, BTreeMap<u32, Vec<TransmogSet>>, 0);

#[derive(Resource, Default)]
pub struct TransmogSetItemsByTransmogSet(pub BTreeMap<u32, Vec<TransmogSetItem>>);
deref_boilerplate!(TransmogSetItemsByTransmogSet, BTreeMap<u32, Vec<TransmogSetItem>> , 0);

#[derive(Resource, Default)]
pub struct WMOAreaTableLookupContainer(pub BTreeMap<(u16, u8, u32), WMOAreaTable>);
deref_boilerplate!(WMOAreaTableLookupContainer, BTreeMap<(u16, u8, u32), WMOAreaTable>, 0);

#[derive(Resource, Default)]
pub struct WorldMapAreaByAreaIDContainer(pub BTreeMap<u32, WorldMapArea>);
deref_boilerplate!(WorldMapAreaByAreaIDContainer, BTreeMap<u32, WorldMapArea>, 0);

#[derive(Resource, Default)]
pub struct TaxiPathSetBySource(pub BTreeMap<u32, BTreeMap<u32, TaxiPath>>);
deref_boilerplate!(TaxiPathSetBySource, BTreeMap<u32, BTreeMap<u32, TaxiPath>>, 0);

#[derive(Resource, Default)]
pub struct TaxiPathNodesByPath(pub BTreeMap<u32, BTreeMap<u32, TaxiPathNode>>);
deref_boilerplate!(TaxiPathNodesByPath, BTreeMap<u32, BTreeMap<u32, TaxiPathNode>>, 0);

#[derive(Resource, Default)]
pub struct TaxiNodesMask(pub BTreeMap<u8, u32>);
deref_boilerplate!(TaxiNodesMask, BTreeMap<u8, u32>, 0);

#[derive(Resource, Default)]
pub struct OldContinentsNodesMask(pub BTreeMap<u8, u32>);
deref_boilerplate!(OldContinentsNodesMask, BTreeMap<u8, u32>, 0);

#[derive(Resource, Default)]
pub struct HordeTaxiNodesMask(pub BTreeMap<u8, u32>);
deref_boilerplate!(HordeTaxiNodesMask, BTreeMap<u8, u32>, 0);

#[derive(Resource, Default)]
pub struct AllianceTaxiNodesMask(pub BTreeMap<u8, u32>);
deref_boilerplate!(AllianceTaxiNodesMask, BTreeMap<u8, u32>, 0);

impl DB2Storage<Difficulty> {
    /// Player::CheckLoadedDungeonDifficultyID in TC
    pub fn check_loaded_dungeon_difficulty_id(&self, difficulty_id: u32) -> DifficultyID {
        let Some(entry) = self.get(&difficulty_id) else { return DifficultyID::Normal };
        if entry.instance_type != MapType::Instance.to_num() {
            return DifficultyID::Normal;
        }
        if entry.flags & FlagSet::from(DifficultyFlag::CanSelect).bits() == 0 {
            return DifficultyID::Normal;
        }
        DifficultyID::from_u32(difficulty_id).unwrap_or(DifficultyID::Normal)
    }

    /// Player::CheckLoadedRaidDifficultyID in TC
    pub fn check_loaded_raid_difficulty_id(&self, difficulty_id: u32) -> DifficultyID {
        let Some(entry) = self.get(&difficulty_id) else {
            return DifficultyID::NormalRaid;
        };
        if entry.instance_type != MapType::Raid.to_num() {
            return DifficultyID::NormalRaid;
        }
        if entry.flags & FlagSet::from(DifficultyFlag::CanSelect).bits() == 0 {
            return DifficultyID::NormalRaid;
        }
        if entry.flags & FlagSet::from(DifficultyFlag::Legacy).bits() > 0 {
            return DifficultyID::NormalRaid;
        }
        DifficultyID::from_u32(difficulty_id).unwrap_or(DifficultyID::NormalRaid)
    }

    /// Player::CheckLoadedLegacyRaidDifficultyID in TC
    ///
    pub fn check_loaded_legacy_raid_difficulty_id(&self, difficulty_id: u32) -> DifficultyID {
        let Some(entry) = self.get(&difficulty_id) else { return DifficultyID::_10N };
        if entry.instance_type != MapType::Raid.to_num() {
            return DifficultyID::_10N;
        }
        if entry.flags & FlagSet::from(DifficultyFlag::CanSelect).bits() == 0 {
            return DifficultyID::_10N;
        }
        if entry.flags & FlagSet::from(DifficultyFlag::Legacy).bits() == 0 {
            return DifficultyID::_10N;
        }

        DifficultyID::from_u32(difficulty_id).unwrap_or(DifficultyID::_10N)
    }
}

#[derive(SystemParam)]
/// DB2Manager in TC
pub struct DB2Mgr<'w> {
    pub stores: DB2Stores<'w>,
    pub area_group_members: Res<'w, AreaGroupMembers>,
    pub artifact_powers: Res<'w, ArtifactPowers>,
    pub artifact_power_links: Res<'w, ArtifactPowerLinks>,
    pub artifact_power_ranks: Res<'w, ArtifactPowerRanks>,
    pub character_facial_hair_styles: Res<'w, CharFacialHairStyles>,
    pub char_sections: Res<'w, CharsSections>,
    pub char_start_outfits: Res<'w, CharStartOutfits>,
    pub power_index_by_class: Res<'w, PowersByClass>,
    pub chr_specializations_by_index: Res<'w, ChrSpecializationByIndexContainer>,
    pub curve_points: Res<'w, CurvePointsContainer>,
    pub emote_text_sounds: Res<'w, EmotesTextSoundContainer>,
    pub faction_teams: Res<'w, FactionTeamContainer>,
    pub heirlooms: Res<'w, HeirloomItemsContainer>,
    pub glyph_bindable_spells: Res<'w, GlyphBindableSpellsContainer>,
    pub glyph_required_specs: Res<'w, GlyphRequiredSpecsContainer>,
    pub item_bonus_lists: Res<'w, ItemBonusListContainer>,
    pub item_level_delta_to_bonus_list_container: Res<'w, ItemBonusListLevelDeltaContainer>,
    pub item_bonus_trees: Res<'w, ItemBonusTreeContainer>,
    pub item_child_equipment: Res<'w, ItemChildEquipmentContainer>,
    pub item_class_by_old_enum: Res<'w, ItemClassByOldEnumContainer>,
    pub items_with_currency_cost: Res<'w, ItemIDsWithCurrencyCost>,
    pub item_category_conditions: Res<'w, ItemLimitCategoryConditionContainer>,
    pub item_level_quality_selector_qualities: Res<'w, ItemLevelSelectorQualityContainer>,
    pub item_modified_appearances_by_item: Res<'w, ItemModifiedAppearanceByItemContainer>,
    pub item_to_bonus_tree: Res<'w, ItemToBonusTreeContainer>,
    pub item_set_spells: Res<'w, ItemSetSpellContainer>,
    pub item_spec_overrides: Res<'w, ItemSpecOverridesContainer>,
    pub map_difficulties: Res<'w, MapDifficultyContainer>,
    pub mounts_by_spell_id: Res<'w, MountsBySpellIDContainer>,
    pub mount_capabilities_by_type: Res<'w, MountCapabilitiesByTypeContainer>,
    pub mount_displays: Res<'w, MountDisplaysContainer>,
    pub name_gen_data: Res<'w, NameGenContainer>,
    pub name_validators: Res<'w, NameValidationRegexContainer>,
    pub name_reserved_validators: Res<'w, NameReservedRegexContainer>,
    pub phases_by_group: Res<'w, PhaseGroupContainer>,
    pub power_types: Res<'w, PowerTypesContainer>,
    pub pvp_item_bonus: Res<'w, PvpItemBonusContainer>,
    pub pvp_reward_pack: Res<'w, PvpRewardPackContainer>,
    pub pvp_talents_by_position: Res<'w, PvpTalentsByPosition>,
    pub pvp_talent_unlock: Res<'w, PvpTalentUnlockHonourLevel>,
    pub quest_packages: Res<'w, QuestPackageItemContainer>,
    pub reward_pack_currency_types: Res<'w, RewardPackCurrencyTypes>,
    pub reward_pack_items: Res<'w, RewardPackItems>,
    pub ruleset_item_upgrade: Res<'w, RulesetItemUpgradeContainer>,
    pub skill_race_class_info_by_skill: Res<'w, SkillRaceClassInfoContainer>,
    pub specialization_spells_by_spec: Res<'w, SpecializationSpellsContainer>,
    pub spell_family_names: Res<'w, SpellClassOptionsSets>,
    pub spell_powers: Res<'w, SpellPowerContainer>,
    pub spell_power_difficulties: Res<'w, SpellPowerDifficultyContainer>,
    pub spell_procs_per_minute_mods: Res<'w, SpellProcsPerMinuteModContainer>,
    pub talents_by_position: Res<'w, TalentsByPosition>,
    pub toys: Res<'w, ToyItemIdsContainer>,
    pub transmog_sets_by_item_modified_appearance: Res<'w, TransmogSetsByItemModifiedAppearance>,
    pub transmog_set_items_by_transmog_set: Res<'w, TransmogSetItemsByTransmogSet>,
    pub wmo_area_table_lookup: Res<'w, WMOAreaTableLookupContainer>,
    pub world_map_area_by_area_id: Res<'w, WorldMapAreaByAreaIDContainer>,
    pub taxi_path_set_by_source: Res<'w, TaxiPathSetBySource>,
    pub taxi_path_nodes_by_path: Res<'w, TaxiPathNodesByPath>,
    pub all_taxi_nodes_mask: Res<'w, TaxiNodesMask>,
    pub old_continents_nodes_mask: Res<'w, OldContinentsNodesMask>,
    pub horde_taxi_nodes_mask: Res<'w, HordeTaxiNodesMask>,
    pub alliance_taxi_nodes_mask: Res<'w, AllianceTaxiNodesMask>,
}

fn load_db2_store_after(
    mut commands: Commands,
    stores: DB2Stores,
    mut ev_db2_load_started: EventReader<DB2LoadStartEvent>,
    mut ev_startup_failed: EventWriter<AzStartupFailedEvent>,
) {
    let mut earliest_start = None;
    let mut num_db2s = 0;
    for DB2LoadStartEvent(start) in ev_db2_load_started.read() {
        num_db2s += 1;
        earliest_start = match earliest_start {
            None => Some(*start),
            Some(es) if *start < es => Some(*start),
            _ => continue,
        };
    }

    let Some(earliest_start) = earliest_start else {
        info!(target:"server::loading", "no DB2 set up");
        ev_startup_failed.send_default();
        return;
    };
    let mut m = AreaGroupMembers::default();
    for e in stores.area_group_member_store.values() {
        m.entry(e.area_group_id.into()).or_default().push(e.area_id.into());
    }
    commands.insert_resource(m);

    let mut m = ArtifactPowers::default();
    for e in stores.artifact_power_store.values() {
        m.entry(e.artifact_id.into()).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = ArtifactPowerLinks::default();
    for e in stores.artifact_power_link_store.values() {
        m.entry(e.power_a.into()).or_default().insert(e.power_b.into());
        m.entry(e.power_b.into()).or_default().insert(e.power_a.into());
    }
    commands.insert_resource(m);

    let mut m = ArtifactPowerRanks::default();
    for e in stores.artifact_power_rank_store.values() {
        *m.entry((e.artifact_power_id.into(), e.rank_index)).or_default() = e.clone()
    }
    commands.insert_resource(m);

    if BATTLE_PET_SPECIES_MAX_ID < stores.battle_pet_species_store.len() {
        error!(target: "server::loading", "BATTLE_PET_SPECIES_MAX_ID {bpet_max_id} must be equal or greater than {bpet_species}",
            bpet_max_id=BATTLE_PET_SPECIES_MAX_ID,
            bpet_species=stores.battle_pet_species_store.len(),
        );
        ev_startup_failed.send_default();
        return;
    }
    let mut m = CharFacialHairStyles::default();
    for e in stores.character_facial_hair_styles_store.values() {
        m.insert((e.race_id, e.sex_id, e.variation_id.into()));
    }
    let mut section_to_base = BTreeMap::new();
    for e in stores.char_base_section_store.values() {
        let r = match CharSectionType::try_from(e.resolution_variation_enum) {
            Err(err) => {
                error!(target: "server::loading", "CharSectionType is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let s = match CharBaseSectionVariation::try_from(e.variation_enum) {
            Err(err) => {
                error!(target: "server::loading", "CharBaseSectionVariation is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        section_to_base.insert(r, s);
    }
    let mut char_sections = CharsSections::default();
    let mut added_sections = BTreeMap::new();
    for e in stores.char_sections_store.values() {
        let s = match CharSectionType::try_from(e.base_section as u8) {
            Err(err) => {
                error!(target: "server::loading", "CharSectionType is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let section_key = (e.race_id as u8, e.sex_id as u8, section_to_base[&s]);
        let section_combination = (e.variation_index, e.color_index);
        let entry = added_sections.entry(section_key).or_insert(BTreeSet::new());
        if entry.contains(&section_combination) {
            continue;
        }
        entry.insert(section_combination);
        char_sections.entry(section_key).or_default().push(e.clone());
    }
    commands.insert_resource(char_sections);

    let mut m = CharStartOutfits::default();
    for e in stores.char_start_outfit_store.values() {
        m.insert((e.race_id, e.class_id, e.sex_id), e.clone());
    }
    commands.insert_resource(m);

    let mut powers_by_classes = PowersByClass::default();
    for e in stores.chr_classes_x_power_types_store.values() {
        let class = match Class::try_from(e.class_id) {
            Err(err) => {
                error!(target: "server::loading", "Class is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let power = match Power::try_from(e.class_id as i8) {
            Err(err) => {
                error!(target: "server::loading", "Power is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let power_indexes = powers_by_classes.entry(class).or_default();
        let Ok(power_idx) = u32::try_from(power_indexes.len()) else {
            error!(target: "server::loading", "Power index is cannot be retrieved from '{class:?}' and '{power:?}', idx={idx}", idx=power_indexes.len());
            ev_startup_failed.send_default();
            return;
        };
        power_indexes.entry(power).or_insert(power_idx);
    }
    commands.insert_resource(powers_by_classes);

    let mut m = ChrSpecializationByIndexContainer::default();
    for e in stores.chr_specialization_store.values() {
        let class = match Class::try_from(e.class_id as u8) {
            Err(err) => {
                error!(target: "server::loading", "Class is invalid for chr_specialization_store; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        m.entry(class).or_default().entry(e.order_index).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = CurvePointsContainer::default();
    for e in stores.curve_point_store.values() {
        if stores.curve_store.get(&e.curve_id.into()).is_some() {
            m.entry(e.curve_id.into()).or_default().push(e.clone());
        }
    }
    for cps in m.values_mut() {
        cps.sort_by(|p1, p2| p1.order_index.cmp(&p2.order_index));
    }
    commands.insert_resource(m);
    let mut m = EmotesTextSoundContainer::default();
    for e in stores.emotes_text_sound_store.values() {
        let race = match Race::try_from(e.race_id) {
            Err(err) => {
                error!(target:"server::loading", "Race is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let gender = match Gender::try_from(e.sex_id) {
            Err(err) => {
                error!(target:"server::loading", "Sex is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let class = match Class::try_from(e.class_id) {
            Err(err) => {
                error!(target:"server::loading", "Class is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        m.entry((e.emotes_text_id.into(), race, gender, class)).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = FactionTeamContainer::default();
    for e in stores.faction_store.values() {
        if e.parent_faction_id != 0 {
            m.entry(e.parent_faction_id.into()).or_default().push(e.id);
        }
    }
    commands.insert_resource(m);

    let mut m = HeirloomItemsContainer::default();
    for e in stores.heirloom_store.values() {
        m.entry(e.item_id).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = GlyphBindableSpellsContainer::default();
    for e in stores.glyph_bindable_spell_store.values() {
        m.entry(e.glyph_properties_id.into()).or_default().push(e.spell_id);
    }
    commands.insert_resource(m);

    let mut m = GlyphRequiredSpecsContainer::default();
    for e in stores.glyph_required_spec_store.values() {
        m.entry(e.glyph_properties_id.into()).or_default().push(e.chr_specialization_id.into());
    }
    commands.insert_resource(m);

    let mut m = ItemBonusListContainer::default();
    for e in stores.item_bonus_store.values() {
        m.entry(e.parent_item_bonus_list_id.into()).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemBonusListLevelDeltaContainer::default();
    for e in stores.item_bonus_list_level_delta_store.values() {
        m.entry(e.item_level_delta).or_insert(e.id);
    }
    commands.insert_resource(m);

    let mut m = ItemBonusTreeContainer::default();
    for e in stores.item_bonus_tree_node_store.values() {
        let mut bonus_tree_node = Some(e);
        while let Some(bn) = bonus_tree_node {
            m.entry(e.parent_item_bonus_tree_id.into()).or_default().insert(bn.clone());
            bonus_tree_node = stores.item_bonus_tree_node_store.get(&bn.child_item_bonus_tree_id.into());
        }
    }
    commands.insert_resource(m);

    let mut m = ItemChildEquipmentContainer::default();
    for e in stores.item_child_equipment_store.values() {
        if m.get(&e.parent_item_id).is_some() {
            error!(target:"server::loading", "Item must have max 1 child item.");
            ev_startup_failed.send_default();
            return;
        }
        m.insert(e.parent_item_id, e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemClassByOldEnumContainer::default();
    for e in stores.item_class_store.values() {
        let item_class_id = match ItemClassID::try_from(e.class_id) {
            Err(err) => {
                error!(target:"server::loading", "unrecognised item class ID, err={err}.");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) if m.get(&v).is_some() => {
                error!(target:"server::loading", "item class ID already filled, item_class_id={v:?}, item_class={e:?}.");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        m.insert(item_class_id, e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemIDsWithCurrencyCost::default();
    for e in stores.item_currency_cost_store.values() {
        m.insert(e.item_id);
    }
    commands.insert_resource(m);

    let mut m = ItemLimitCategoryConditionContainer::default();
    for e in stores.item_limit_category_condition_store.values() {
        m.entry(e.parent_item_limit_category_id).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemLevelSelectorQualityContainer::default();
    for e in stores.item_level_selector_quality_store.values() {
        m.entry(e.parent_ils_quality_set_id.into()).or_default().insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemModifiedAppearanceByItemContainer::default();
    for e in stores.item_modified_appearance_store.values() {
        m.insert((e.item_id, e.item_appearance_modifier_id), e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemSetSpellContainer::default();
    for e in stores.item_set_spell_store.values() {
        m.entry(e.item_set_id.into()).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = ItemSpecOverridesContainer::default();
    for e in stores.item_spec_override_store.values() {
        m.entry(e.item_id).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m: ItemToBonusTreeContainer = ItemToBonusTreeContainer::default();
    for e in stores.item_x_bonus_tree_store.values() {
        m.entry(e.item_id).or_default().push(e.item_bonus_tree_id.into());
    }
    commands.insert_resource(m);

    let mut m = MapDifficultyContainer::default();
    for e in stores.map_difficulty_store.values() {
        m.entry(e.map_id.into()).or_default().entry(e.difficulty_id).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = MountsBySpellIDContainer::default();
    for e in stores.mount_store.values() {
        m.insert(e.source_spell_id, e.clone());
    }
    commands.insert_resource(m);

    let mut m = MountCapabilitiesByTypeContainer::default();
    for e in stores.mount_type_x_capability_store.values() {
        m.entry(e.mount_type_id.into()).or_default().insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = MountDisplaysContainer::default();
    for e in stores.mount_x_display_store.values() {
        m.entry(e.mount_id).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = NameGenContainer::default();
    for e in stores.name_gen_store.values() {
        let race = match Race::try_from(e.race_id) {
            Err(err) => {
                error!(target: "server::loading", "Invalid Race for name gen container; entry={e:?}, err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let gender = match Gender::try_from(e.sex) {
            Err(err) => {
                error!(target: "server::loading", "Invalid Gender for name gen container; entry={e:?}, err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        m.entry(race).or_default().entry(gender).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = NameValidationRegexContainer::default();
    for e in stores.names_profanity_store.values() {
        let name_regex = match RegexBuilder::new(&e.name).case_insensitive(true).build() {
            Err(err) => {
                error!(target: "server::loading", "Invalid regex for names_profanity_store; entry={e:?}, err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };

        if e.language >= 0 {
            let l = match Locale::try_from(e.language as u32) {
                Err(err) => {
                    error!(target: "server::loading", "Invalid Locale for names_profanity_store; entry={e:?}, err={err}");
                    ev_startup_failed.send_default();
                    return;
                },
                Ok(v) => v,
            };
            m.entry(l).or_default().push(name_regex.clone());
        } else {
            for l in FlagSet::<Locale>::full().into_iter().filter(|l| *l != Locale::none) {
                m.entry(l).or_default().push(name_regex.clone());
            }
        }
    }
    commands.insert_resource(m);

    let mut m = NameReservedRegexContainer::default();
    for e in stores.names_reserved_store.values() {
        let name_regex = match RegexBuilder::new(&e.name).case_insensitive(true).build() {
            Err(err) => {
                error!(target: "server::loading", "Invalid regex for names_profanity_store; entry={e:?}, err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        for l in FlagSet::<Locale>::full().into_iter().filter(|l| *l != Locale::none) {
            m.entry(l).or_default().push(name_regex.clone());
        }
    }
    for e in stores.names_reserved_locale_store.values() {
        let name_regex = match RegexBuilder::new(&e.name).case_insensitive(true).build() {
            Err(err) => {
                error!(target: "server::loading", "Invalid regex for names_profanity_store; entry={e:?}, err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        let locales = match FlagSet::<Locale>::new(u32::from(e.locale_mask)) {
            Err(err) => {
                error!(target: "server::loading", "Invalid regex for names_profanity_store; entry={e:?}, err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        for l in locales.into_iter().filter(|l| *l != Locale::none) {
            m.entry(l).or_default().push(name_regex.clone());
        }
    }
    commands.insert_resource(m);

    let mut m = PhaseGroupContainer::default();
    for e in stores.phase_x_phase_group_store.values() {
        if let Some(p) = stores.phase_store.get(&e.phase_id.into()) {
            m.entry(e.phase_group_id.into()).or_default().push(p.id);
        }
    }
    commands.insert_resource(m);

    let mut m = PowerTypesContainer::default();
    for e in stores.power_type_store.values() {
        let power = match Power::try_from(e.power_type_enum) {
            Err(err) => {
                error!(target: "server::loading", "Power is invalid for power_type_store; e={e:?} err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        if let Some(prev_type) = m.insert(power, e.clone()) {
            error!(target: "server::loading", "PowerType already seen for power_type_store; e={e:?}, prev_e={prev_type:?}");
            ev_startup_failed.send_default();
            return;
        }
    }
    commands.insert_resource(m);

    // TODO: hirogoro@27aug2024: Add in sSpellsByCategoryStore stuff as it seems useful from AC, PR here:
    // https://github.com/azerothcore/azerothcore-wotlk/pull/7559
    // for (auto i : sSpellStore)
    //     if (i->Category)
    //         sSpellsByCategoryStore[i->Category].emplace(false, i->Id);

    let mut m = PvpItemBonusContainer::default();
    for e in stores.pvp_item_store.values() {
        m.insert(e.item_id, e.item_level_delta);
    }
    commands.insert_resource(m);

    let mut m = PvpRewardPackContainer::default();
    for e in stores.pvp_reward_store.values() {
        m.insert((e.prestige_level, e.honor_level), e.reward_pack_id);
    }
    commands.insert_resource(m);

    let mut m = PvpTalentsByPosition::default();
    for e in stores.pvp_talent_store.values() {
        let class = match u8::try_from(e.class_id).map_err(|_| ClassError { got: e.class_id }).and_then(Class::try_from) {
            Err(err) => {
                error!(target: "server::loading", "Class is invalid for PvpTalentsByPosition; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        match class {
            Class::None => {
                for c in FlagSet::<Class>::full().into_iter().filter(|c| *c != Class::None) {
                    m.entry(c)
                        .or_default()
                        .entry(e.tier_id)
                        .or_default()
                        .entry(e.column_index)
                        .or_default()
                        .push(e.clone());
                }
            },
            c => {
                m.entry(c)
                    .or_default()
                    .entry(e.tier_id)
                    .or_default()
                    .entry(e.column_index)
                    .or_default()
                    .push(e.clone());
            },
        }
    }
    commands.insert_resource(m);

    let mut m = PvpTalentUnlockHonourLevel::default();
    for e in stores.pvp_talent_unlock_store.values() {
        m.entry(e.tier_id).or_default().insert(e.column_index, e.honor_level);
    }
    commands.insert_resource(m);

    let mut m = QuestPackageItemContainer::default();
    for e in stores.quest_package_item_store.values() {
        let (first, second) = m.entry(e.package_id.into()).or_default();
        if !QuestPackageFilter::try_from(e.display_type).is_ok_and(|f| f == QuestPackageFilter::Unmatched) {
            first.push(e.clone());
        } else {
            second.push(e.clone());
        }
    }
    commands.insert_resource(m);

    let mut m = RewardPackCurrencyTypes::default();
    for e in stores.reward_pack_x_currency_type_store.values() {
        m.entry(e.reward_pack_id).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = RewardPackItems::default();
    for e in stores.reward_pack_x_item_store.values() {
        m.entry(e.reward_pack_id).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = RulesetItemUpgradeContainer::default();
    for e in stores.ruleset_item_upgrade_store.values() {
        m.insert(e.item_id, e.item_upgrade_id.into());
    }
    commands.insert_resource(m);

    let mut m = SkillRaceClassInfoContainer::default();
    for e in stores.skill_race_class_info_store.values() {
        if stores.skill_line_store.contains_key(&e.skill_id.into()) {
            m.entry(e.skill_id.into()).or_default().push(e.clone());
        }
    }
    commands.insert_resource(m);

    let mut m = SpecializationSpellsContainer::default();
    for e in stores.specialization_spells_store.values() {
        m.entry(e.spec_id.into()).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = SpellClassOptionsSets::default();
    for e in stores.spell_class_options_store.values() {
        m.insert(e.spell_class_set);
    }
    commands.insert_resource(m);

    let mut spell_powers = SpellPowerContainer::default();
    let mut spell_power_difficulties = SpellPowerDifficultyContainer::default();
    for power in stores.spell_power_store.values() {
        if let Some(power_diff) = stores.spell_power_difficulty_store.get(&power.id) {
            let pows = spell_power_difficulties
                .entry(power.spell_id)
                .or_default()
                .entry(power_diff.difficulty_id.into())
                .or_default();
            pows.insert(power_diff.order_index, power.clone());
        } else {
            let pows = spell_powers.entry(power.id).or_default();
            pows.insert(power.order_index, power.clone());
        }
    }
    commands.insert_resource(spell_powers);
    commands.insert_resource(spell_power_difficulties);

    let mut m = SpellProcsPerMinuteModContainer::default();
    for e in stores.spell_procs_per_minute_mod_store.values() {
        m.entry(e.spell_procs_per_minute_id.into()).or_default().push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = TalentsByPosition::default();
    for e in stores.talent_store.values() {
        let class = match Class::try_from(e.class_id) {
            Err(err) => {
                error!(target: "server::loading", "Class is invalid; err={err}");
                ev_startup_failed.send_default();
                return;
            },
            Ok(v) => v,
        };
        m.entry(class)
            .or_default()
            .entry(e.tier_id.into())
            .or_default()
            .entry(e.column_index.into())
            .or_default()
            .push(e.clone());
    }
    commands.insert_resource(m);

    let mut m = TaxiPathSetBySource::default();
    for e in stores.taxi_path_store.values() {
        m.entry(e.from_taxi_node.into()).or_default().entry(e.to_taxi_node.into()).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut m = TaxiPathNodesByPath::default();
    for e in stores.taxi_path_node_store.values() {
        m.entry(e.path_id.into()).or_default().entry(e.node_index.into()).or_insert(e.clone());
    }
    commands.insert_resource(m);

    let mut all_taxi_nodes_mask = TaxiNodesMask::default();
    let mut old_continents_nodes_mask = OldContinentsNodesMask::default();
    let mut horde_taxi_nodes_mask = HordeTaxiNodesMask::default();
    let mut alliance_taxi_nodes_mask = AllianceTaxiNodesMask::default();
    // // Initialize global taxinodes mask
    // // include existed nodes that have at least single not spell base (scripted) path
    // // TODO: check if we need this bit of code from AC. These were removed in TC at this commit.
    // // https://github.com/TrinityCore/TrinityCore/commit/2c7459da6daf1d563825f3039b1c7112da2560ae
    //     std::set<uint32> spellPaths;
    //     for (SpellEntry const* sInfo : sSpellStore)
    //         for (uint8 j = 0; j < MAX_SPELL_EFFECTS; ++j)
    //             if (sInfo->Effect[j] == SPELL_EFFECT_SEND_TAXI)
    //                 spellPaths.insert(sInfo->EffectMiscValue[j]);
    for node in stores.taxi_nodes_store.values() {
        //         TaxiPathSetBySource::const_iterator src_i = sTaxiPathSetBySource.find(i);
        //         if (src_i != sTaxiPathSetBySource.end() && !src_i->second.empty())
        //         {
        //             bool ok = false;
        //             for (TaxiPathSetForSource::const_iterator dest_i = src_i->second.begin(); dest_i != src_i->second.end(); ++dest_i)
        //             {
        //                 // not spell path
        //                 if (dest_i->second->price || spellPaths.find(dest_i->second->ID) == spellPaths.end())
        //                 {
        //                     ok = true;
        //                     break;
        //                 }
        //             }

        //             if (!ok)
        //                 continue;
        //         }
        if node.flags & (TaxiNodeFlags::Alliance | TaxiNodeFlags::Horde).bits() == 0 {
            continue;
        }
        // valid taxi network node
        let field = ((node.id - 1) / 8) as u8;
        let submask = 1 << ((node.id - 1) % 8);

        *all_taxi_nodes_mask.entry(field).or_default() |= submask;
        if node.flags & FlagSet::from(TaxiNodeFlags::Horde).bits() > 0 {
            *horde_taxi_nodes_mask.entry(field).or_default() |= submask;
        }
        if node.flags & FlagSet::from(TaxiNodeFlags::Alliance).bits() > 0 {
            *alliance_taxi_nodes_mask.entry(field).or_default() |= submask;
        }
        let (node_map_id, _) = stores
            .world_map_transforms_store
            .determine_alternate_map_position(node.continent_id.into(), node.pos.x, node.pos.y, node.pos.z);
        if node_map_id < 2 {
            *old_continents_nodes_mask.entry(field).or_default() = submask;
        }
    }
    commands.insert_resource(all_taxi_nodes_mask);
    commands.insert_resource(old_continents_nodes_mask);
    commands.insert_resource(horde_taxi_nodes_mask);
    commands.insert_resource(alliance_taxi_nodes_mask);

    let mut m = ToyItemIdsContainer::default();
    for e in stores.toy_store.values() {
        m.insert(e.item_id);
    }
    commands.insert_resource(m);

    let mut transmog_sets_by_item_modified_appearance = TransmogSetsByItemModifiedAppearance::default();
    let mut transmog_set_items_by_transmog_set = TransmogSetItemsByTransmogSet::default();
    for transmog_set_item in stores.transmog_set_item_store.values() {
        let Some(set) = stores.transmog_set_store.get(&transmog_set_item.transmog_set_id) else {
            continue;
        };
        transmog_sets_by_item_modified_appearance
            .entry(transmog_set_item.item_modified_appearance_id)
            .or_default()
            .push(set.clone());
        transmog_set_items_by_transmog_set
            .entry(transmog_set_item.transmog_set_id)
            .or_default()
            .push(transmog_set_item.clone());
    }
    commands.insert_resource(transmog_sets_by_item_modified_appearance);
    commands.insert_resource(transmog_set_items_by_transmog_set);

    let mut m = WMOAreaTableLookupContainer::default();
    for e in stores.wmo_area_table_store.values() {
        m.insert((e.wmo_id, e.name_set_id, e.wmo_group_id), e.clone());
    }
    commands.insert_resource(m);

    let mut m = WorldMapAreaByAreaIDContainer::default();
    for e in stores.world_map_area_store.values() {
        m.insert(e.area_id.into(), e.clone());
    }
    commands.insert_resource(m);

    // Check loaded DB2 files proper version
    // last area added in 7.3.5 (25996)
    let check_area = stores.area_table_store.get(&9531).is_none();
    // last char title added in 7.3.5 (25996)
    let check_titles = stores.char_titles_store.get(&522).is_none();
    // last gem property added in 7.3.5 (25996)
    let check_gem_property = stores.gem_properties_store.get(&3632).is_none();
    // last item added in 7.3.5 (25996)
    let check_item = stores.item_store.get(&157831).is_none();
    // last item extended cost added in 7.3.5 (25996)
    let check_item_extended_cost = stores.item_extended_cost_store.get(&6300).is_none();
    // last map added in 7.3.5 (25996)
    let check_map = stores.map_store.get(&1903).is_none();
    // last spell added in 7.3.5 (25996)
    let check_spell = stores.spell_store.get(&263166).is_none();
    if check_area || check_titles || check_gem_property || check_item || check_item_extended_cost || check_map || check_spell {
        error!(target:"misc", "You have _outdated_ DB2 files. Please extract correct versions from current using client.");
        ev_startup_failed.send_default();
        return;
    }

    let current_time = Instant::now();
    let duration = current_time - earliest_start;
    info!(target:"server.loading", ">> Initialized {num_db2s} DB2 data stores in {duration:?}");
}

#[derive(Event)]
struct DB2LoadStartEvent(Instant);

/// LOAD_DB2 macro in TC, LOAD_DBC macro in AC
/// Loads the DB2 store from file and DB
fn load_db2<D: DB2 + From<wow_db2::DB2RawRecord> + for<'r> FromRow<'r, <DbDriver as Database>::Row> + Send + Unpin + Sync + 'static>(
    time: Res<Time<Real>>,
    mut commands: Commands,
    rt: Res<TokioRuntime>,
    cfg: Res<ConfigMgr<WorldConfig>>,
    hotfix_db: Res<HotfixDatabase>,
    mut ev_db2_load_started: EventWriter<DB2LoadStartEvent>,
    mut ev_startup_failed: EventWriter<AzStartupFailedEvent>,
) {
    ev_db2_load_started.send(DB2LoadStartEvent(time.startup() + time.elapsed()));

    let db2_dir = cfg.db2_dir();
    let available_db2_locales = match fs::read_dir(&db2_dir) {
        Err(e) => {
            error!(cause=?e, "Unable to read DB2 directory");
            ev_startup_failed.send_default();
            return;
        },
        Ok(r_dir) => r_dir.filter_map(|de| {
            let Ok(de) = de else { return None };
            let is_dir = de.file_type().ok().map(|ft| ft.is_dir()).unwrap_or(false);
            if !is_dir {
                return None;
            }
            Locale::from_str(&de.file_name().to_string_lossy()).ok()
        }),
    };

    let store = match rt.block_on(DB2Storage::<D>::load(db2_dir, &**hotfix_db, cfg.DBCLocale, available_db2_locales)) {
        Err(e) => {
            error!(cause=?e, "Unable to load DB2 {}", D::db2_file());
            ev_startup_failed.send_default();
            return;
        },
        Ok(s) => s,
    };
    commands.insert_resource(store);
}

impl LiquidFlagsGetter for DB2Storage<LiquidType> {
    /// DB2Manager::GetLiquidFlags
    fn get_liquid_flags(&self, liquid_type_id: u32) -> FlagSet<MapLiquidTypeFlag> {
        self.get(&liquid_type_id)
            .map_or_else(|| None.into(), |t| MapLiquidTypeFlag::from_liquid_type_sound_bank_unchecked(t.sound_bank))
    }
}
