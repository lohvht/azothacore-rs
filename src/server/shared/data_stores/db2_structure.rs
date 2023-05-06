use wow_db2_proc_macros::WDC1;

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6A529F37)]
pub struct ManifestInterfaceActionIcon {
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x069F44E5)]
pub struct GarrItemLevelUpgradeData {
    #[id]
    pub id: u32,
    pub i1: u32,
    pub i2: u32,
    pub i3: u32,
    pub i4: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2C4BE18C)]
pub struct Achievement {
    pub title:            String,
    pub description:      String,
    pub reward:           String,
    pub flags:            i32,
    /// -1 = none
    pub instance_id:      i16,
    /// its Achievement parent (can`t start while parent uncomplete, use its Criteria if don`t have own, use its progress on begin)
    pub supercedes:       i16,
    pub category:         i16,
    #[parent]
    pub ui_order:         i16,
    /// referenced achievement (counting of all completed criterias)
    pub shares_criteria:  i16,
    /// -1 = all, 0 = horde, 1 = alliance
    pub faction:          i8,
    pub points:           i8,
    /// need this count of completed criterias (own or referenced achievement criterias)
    pub minimum_criteria: i8,
    #[id]
    pub id:               u32,
    pub icon_file_id:     i32,
    pub criteria_tree:    u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x81D6D250)]
pub struct AnimKit {
    pub id: u32,
    pub one_shot_duration: u32,
    pub one_shot_stop_anim_kit_id: u16,
    pub low_def_anim_kit_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x50AA43EE)]
pub struct AreaGroupMember {
    pub id:            u32,
    pub area_id:       u16,
    #[parent]
    pub area_group_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0CA01129)]
pub struct AreaTable {
    pub id: u32,
    pub zone_name: String,
    pub area_name: String,
    pub flags: [i32; 2],
    pub ambient_multiplier: f32,
    pub continent_id: u16,
    pub parent_area_id: u16,
    pub area_bit: i16,
    pub ambience_id: u16,
    pub zone_music: u16,
    pub intro_sound: u16,
    pub liquid_type_id: [u16; 4],
    pub uw_zone_music: u16,
    pub uw_ambience: u16,
    pub pvp_combat_world_state_id: i16,
    pub sound_provider_pref: u8,
    pub sound_provider_pref_underwater: u8,
    pub exploration_level: i8,
    pub faction_group_mask: u8,
    pub mount_flags: u8,
    pub wild_battle_pet_level_min: u8,
    pub wild_battle_pet_level_max: u8,
    pub wind_settings_id: u8,
    pub uw_intro_sound: u32,
}

impl AreaTable {
    // helpers
    pub fn is_sanctuary(&self) -> bool {
        if self.continent_id == 609 {
            return true;
        }
        const AREA_FLAG_SANCTUARY: i32 = 0x00000800;
        (self.flags[0] & AREA_FLAG_SANCTUARY) != 0
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x378573E8)]
pub struct AreaTrigger {
    pub pos: [f32; 3],
    pub radius: f32,
    pub box_length: f32,
    pub box_width: f32,
    pub box_height: f32,
    pub box_yaw: f32,
    #[parent]
    pub continent_id: i16,
    pub phase_id: i16,
    pub phase_group_id: i16,
    pub shape_id: i16,
    pub area_trigger_action_set_id: i16,
    pub phase_use_flags: i8,
    pub shape_type: i8,
    pub flags: i8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCCFBD16E)]
pub struct ArmorLocation {
    pub id:              u32,
    pub clothmodifier:   f32,
    pub leathermodifier: f32,
    pub chainmodifier:   f32,
    pub platemodifier:   f32,
    pub modifier:        f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x76CF31A8)]
pub struct Artifact {
    pub id: u32,
    pub name: String,
    pub ui_bar_overlay_color: i32,
    pub ui_bar_background_color: i32,
    pub ui_name_color: i32,
    pub ui_texture_kit_id: u16,
    pub chr_specialization_id: u16,
    pub artifact_category_id: u8,
    pub flags: u8,
    pub ui_model_scene_id: u32,
    pub spell_visual_kit_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xAEED7395)]
pub struct ArtifactAppearance {
    pub name: String,
    pub ui_swatch_color: i32,
    pub ui_model_saturation: f32,
    pub ui_model_opacity: f32,
    pub override_shapeshift_display_id: u32,
    #[parent]
    pub artifact_appearance_set_id: u16,
    pub ui_camera_id: u16,
    pub display_index: u8,
    pub item_appearance_modifier_id: u8,
    pub flags: u8,
    pub override_shapeshift_form_id: u8,
    #[id]
    pub id: u32,
    pub unlock_player_condition_id: u32,
    pub ui_item_appearance_id: u32,
    pub ui_alt_item_appearance_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x53DFED74)]
pub struct ArtifactAppearanceSet {
    pub name: String,
    pub description: String,
    pub ui_camera_id: u16,
    pub alt_hand_ui_camera_id: u16,
    pub display_index: u8,
    pub forge_attachment_override: i8,
    pub flags: u8,
    #[id]
    pub id: u32,
    #[parent]
    pub artifact_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x21328475)]
pub struct ArtifactCategory {
    pub id:                  u32,
    pub xp_mult_currency_id: i16,
    pub xp_mult_curve_id:    i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x45240818)]
pub struct ArtifactPower {
    pub pos:                  [f32; 2],
    #[parent]
    pub artifact_id:          u8,
    pub flags:                u8,
    pub max_purchasable_rank: u8,
    pub tier:                 u8,
    #[id]
    pub id:                   u32,
    pub label:                i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE179618C)]
pub struct ArtifactPowerLink {
    pub id:      u32,
    pub power_a: u16,
    pub power_b: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2D6AF006)]
pub struct ArtifactPowerPicker {
    pub id:                  u32,
    pub player_condition_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA87EACC4)]
pub struct ArtifactPowerRank {
    pub id:                   u32,
    pub spell_id:             i32,
    pub aura_points_override: f32,
    pub item_bonus_list_id:   u16,
    pub rank_index:           u8,
    #[parent]
    pub artifact_power_id:    u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x86397302)]
pub struct ArtifactQuestXP {
    pub id:         u32,
    pub difficulty: [u32; 10],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x1A5A50B9)]
pub struct ArtifactTier {
    pub id: u32,
    pub artifact_tier: u32,
    pub max_num_traits: u32,
    pub max_artifact_knowledge: u32,
    pub knowledge_player_condition: u32,
    pub minimum_empower_knowledge: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x52839A77)]
pub struct ArtifactUnlock {
    pub id:                  u32,
    pub item_bonus_list_id:  u16,
    pub power_rank:          u8,
    pub power_id:            u32,
    pub player_condition_id: u32,
    #[parent]
    pub artifact_id:         u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x51CFEEFF)]
pub struct AuctionHouse {
    pub id:               u32,
    pub name:             String,
    /// id of faction.dbc for player factions associated with city
    pub faction_id:       u16,
    pub deposit_rate:     u8,
    pub consignment_rate: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xEA0AC2AA)]
pub struct BankBagSlotPrices {
    pub id:   u32,
    pub cost: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF779B6E5)]
pub struct BannedAddons {
    pub id:      u32,
    pub name:    String,
    pub version: String,
    pub flags:   u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x670C71AE)]
pub struct BarberShopStyle {
    pub display_name:  String,
    pub description:   String,
    pub cost_modifier: f32,
    /// value 0 -> hair, value 2 -> facialhair
    pub typ:           u8,
    pub race:          u8,
    pub sex:           u8,
    /// real ID to hair/facial hair
    pub data:          u8,
    #[id]
    pub id:            u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xBDE74E1D)]
pub struct BattlePetBreedQuality {
    pub id:               u32,
    pub state_multiplier: f32,
    pub quality_enum:     u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x68D5C999)]
pub struct BattlePetBreedState {
    pub id:                  u32,
    pub value:               u16,
    pub battle_pet_state_id: u8,
    #[parent]
    pub battle_pet_breed_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x8A3D97A4)]
pub struct BattlePetSpecies {
    pub source_text: String,
    pub description: String,
    pub creature_id: i32,
    pub icon_file_data_id: i32,
    pub summon_spell_id: i32,
    pub flags: u16,
    pub pet_type_enum: u8,
    pub source_type_enum: i8,
    #[id]
    pub id: u32,
    pub card_ui_model_scene_id: i32,
    pub loadout_ui_model_scene_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x8F958D5C)]
pub struct BattlePetSpeciesState {
    pub id:                    u32,
    pub value:                 i32,
    pub battle_pet_state_id:   u8,
    #[parent]
    pub battle_pet_species_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD8AAA088)]
pub struct BattlemasterList {
    pub id: u32,
    pub name: String,
    pub game_type: String,
    pub short_description: String,
    pub long_description: String,
    pub icon_file_data_id: i32,
    pub map_id: [i16; 16],
    pub holiday_world_state: i16,
    pub required_player_condition_id: i16,
    pub instance_type: i8,
    pub groups_allowed: i8,
    pub max_group_size: i8,
    pub min_level: i8,
    pub max_level: i8,
    pub rated_players: i8,
    pub min_players: i8,
    pub max_players: i8,
    pub flags: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x51BF0C33)]
pub struct BroadcastText {
    pub id:               u32,
    pub text:             String,
    pub text1:            String,
    pub emote_id:         [u16; 3],
    pub emote_delay:      [u16; 3],
    pub emotes_id:        u16,
    pub language_id:      u8,
    pub flags:            u8,
    pub condition_id:     i32,
    pub sound_entries_id: [u32; 2],
}

#[allow(dead_code, non_camel_case_types)]
#[derive(WDC1, Default, Debug)]
#[layout_hash(0x9F4272BF)]
pub struct Cfg_Regions {
    pub id:                u32,
    pub tag:               String,
    /// Date of first raid reset, all other resets are calculated as this date plus interval
    pub raidorigin:        u32,
    pub challenge_origin:  u32,
    pub region_id:         u16,
    pub region_group_mask: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x47D79688)]
pub struct CharacterFacialHairStyles {
    pub id:           u32,
    pub geoset:       [i32; 5],
    pub race_id:      u8,
    pub sex_id:       u8,
    pub variation_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4F08B5F3)]
pub struct CharBaseSection {
    pub id: u32,
    pub variation_enum: u8,
    pub resolution_variation_enum: u8,
    pub layout_res_type: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE349E55B)]
pub struct CharSections {
    pub id:                    u32,
    pub material_resources_id: [i32; 3],
    pub flags:                 i16,
    pub race_id:               i8,
    pub sex_id:                i8,
    pub base_section:          i8,
    pub variation_index:       i8,
    pub color_index:           i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0EEBEE24)]
pub struct CharStartOutfit {
    pub id:             u32,
    pub item_id:        [i32; 24],
    /// Pet Model ID for starting pet
    pub pet_display_id: u32,
    pub class_id:       u8,
    pub sex_id:         u8,
    pub outfit_id:      u8,
    /// Pet Family Entry for starting pet
    pub pet_family_id:  u8,
    #[parent]
    pub race_id:        u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x7A58AA5F)]
pub struct CharTitles {
    pub id:      u32,
    pub name:    String,
    pub name1:   String,
    pub mask_id: i16,
    pub flags:   i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x1A325E80)]
pub struct ChatChannels {
    pub id:            u32,
    pub name:          String,
    pub shortcut:      String,
    pub flags:         i32,
    pub faction_group: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6F7AB8E7)]
pub struct ChrClasses {
    pub pet_name_token: String,
    pub name: String,
    pub name_female: String,
    pub name_male: String,
    pub filename: String,
    pub create_screen_file_data_id: u32,
    pub select_screen_file_data_id: u32,
    pub low_res_screen_file_data_id: u32,
    pub icon_file_data_id: u32,
    pub starting_level: i32,
    pub flags: u16,
    pub cinematic_sequence_id: u16,
    pub default_spec: u16,
    pub display_power: u8,
    pub spell_class_set: u8,
    pub attack_power_per_strength: u8,
    pub attack_power_per_agility: u8,
    pub ranged_attack_power_per_agility: u8,
    pub primary_stat_priority: u8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xAF977B23)]
pub struct ChrClassesXPowerTypes {
    pub id:         u32,
    pub power_type: u8,
    #[parent]
    pub class_id:   u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x51C511F9)]
pub struct ChrRaces {
    pub client_prefix: String,
    pub client_file_string: String,
    pub name: String,
    pub name_female: String,
    pub name_lowercase: String,
    pub name_female_lowercase: String,
    pub flags: i32,
    pub male_display_id: u32,
    pub female_display_id: u32,
    pub create_screen_file_data_id: i32,
    pub select_screen_file_data_id: i32,
    pub male_customize_offset: [f32; 3],
    pub female_customize_offset: [f32; 3],
    pub low_res_screen_file_data_id: i32,
    pub starting_level: i32,
    pub ui_display_order: i32,
    pub faction_id: i16,
    pub res_sickness_spell_id: i16,
    pub splash_sound_id: i16,
    pub cinematic_sequence_id: i16,
    pub base_language: i8,
    pub creature_type: i8,
    pub alliance: i8,
    pub race_related: i8,
    pub unaltered_visual_race_id: i8,
    pub char_component_texture_layout_id: i8,
    pub default_class_id: i8,
    pub neutral_race_id: i8,
    pub display_race_id: i8,
    pub char_component_tex_layout_hi_res_id: i8,
    #[id]
    pub id: u32,
    pub high_res_male_display_id: u32,
    pub high_res_female_display_id: u32,
    pub heritage_armor_achievement_id: i32,
    pub male_skeleton_file_data_id: i32,
    pub female_skeleton_file_data_id: i32,
    pub altered_form_start_visual_kit_id: [u32; 3],
    pub altered_form_finish_visual_kit_id: [u32; 3],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x3D86B8F7)]
pub struct ChrSpecialization {
    pub name:                  String,
    pub female_name:           String,
    pub description:           String,
    pub mastery_spell_id:      [i32; 2],
    #[parent]
    pub class_id:              i8,
    pub order_index:           i8,
    pub pet_talent_type:       i8,
    pub role:                  i8,
    pub primary_stat_priority: i8,
    #[id]
    pub id:                    u32,
    pub spell_icon_file_id:    i32,
    pub flags:                 u32,
    pub anim_replacements:     i32,
}

impl ChrSpecialization {
    pub fn is_pet_specialization(&self) -> bool {
        self.class_id == 0
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0062B0F4)]
pub struct CinematicCamera {
    pub id:            u32,
    /// Sound ID       (voiceover for cinematic)
    pub sound_id:      u32,
    /// Position in map used for basis for M2 co-ordinates
    pub origin:        [f32; 3],
    /// Orientation in map used for basis for M2 co-
    pub origin_facing: f32,
    /// Model
    pub file_data_id:  u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x470FDA8C)]
pub struct CinematicSequences {
    pub id:       u32,
    pub sound_id: u32,
    pub camera:   [u16; 8],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x032B137B)]
pub struct ConversationLine {
    pub id: u32,
    pub broadcast_text_id: u32,
    pub spell_visual_kit_id: u32,
    pub additional_duration: i32,
    pub next_conversation_line_id: u16,
    pub anim_kit_id: u16,
    pub speech_type: u8,
    pub start_animation: u8,
    pub end_animation: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x406268DF)]
pub struct CreatureDisplayInfo {
    #[id]
    pub id: u32,
    pub creature_model_scale: f32,
    pub model_id: u16,
    pub npc_sound_id: u16,
    pub size_class: i8,
    pub flags: u8,
    pub gender: i8,
    pub extended_display_info_id: i32,
    pub portrait_texture_file_data_id: i32,
    pub creature_model_alpha: u8,
    pub sound_id: u16,
    pub player_override_scale: f32,
    pub portrait_creature_display_info_id: i32,
    pub blood_id: u8,
    pub particle_color_id: u16,
    pub creature_geoset_data: u32,
    pub object_effect_package_id: u16,
    pub anim_replacement_set_id: u16,
    pub unarmed_weapon_type: i8,
    pub state_spell_visual_kit_id: i32,
    /// scale of not own player pets inside dungeons/raids/scenarios
    pub pet_instance_scale: f32,
    pub mount_poof_spell_visual_kit_id: i32,
    pub texture_variation_file_data_id: [i32; 3],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6DF98EF6)]
pub struct CreatureDisplayInfoExtra {
    pub id: u32,
    pub bake_material_resources_id: i32,
    pub hd_bake_material_resources_id: i32,
    pub display_race_id: i8,
    pub display_sex_id: i8,
    pub display_class_id: i8,
    pub skin_id: i8,
    pub face_id: i8,
    pub hair_style_id: i8,
    pub hair_color_id: i8,
    pub facial_hair_id: i8,
    pub custom_display_option: [u8; 3],
    pub flags: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE2DC5126)]
pub struct CreatureFamily {
    pub id:              u32,
    pub name:            String,
    pub min_scale:       f32,
    pub max_scale:       f32,
    pub icon_file_id:    i32,
    pub skill_line:      [i16; 2],
    pub pet_food_mask:   i16,
    pub min_scale_level: i8,
    pub max_scale_level: i8,
    pub pet_talent_type: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x983BD312)]
pub struct CreatureModelData {
    pub id: u32,
    pub model_scale: f32,
    pub footprint_texture_length: f32,
    pub footprint_texture_width: f32,
    pub footprint_particle_scale: f32,
    pub collision_width: f32,
    pub collision_height: f32,
    pub mount_height: f32,
    pub geo_box: [f32; 6],
    pub world_effect_scale: f32,
    pub attached_effect_scale: f32,
    pub missile_collision_radius: f32,
    pub missile_collision_push: f32,
    pub missile_collision_raise: f32,
    pub override_loot_effect_scale: f32,
    pub override_name_scale: f32,
    pub override_selection_radius: f32,
    pub tamed_pet_base_scale: f32,
    pub hover_height: f32,
    pub flags: u32,
    pub file_data_id: u32,
    pub size_class: u32,
    pub blood_id: u32,
    pub footprint_texture_id: u32,
    pub foley_material_id: u32,
    pub footstep_camera_effect_id: u32,
    pub death_thud_camera_effect_id: u32,
    pub sound_id: u32,
    pub creature_geoset_data_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x7BA9D2F8)]
pub struct CreatureType {
    pub id:    u32,
    pub name:  String,
    pub flags: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA87A5BB9)]
pub struct Criteria {
    pub id: u32,
    pub asset: u32,
    pub start_asset: i32,
    pub fail_asset: i32,
    pub modifier_tree_id: u32,
    pub start_timer: u16,
    pub eligibility_world_state_id: i16,
    pub typ: u8,
    pub start_event: u8,
    pub fail_event: u8,
    pub flags: u8,
    pub eligibility_world_state_value: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0A1B99C2)]
pub struct CriteriaTree {
    pub id:          u32,
    pub description: String,
    pub amount:      i32,
    pub flags:       i16,
    pub operator:    i8,
    pub criteria_id: u32,
    pub parent:      u32,
    pub order_index: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6CC25CBF)]
pub struct CurrencyTypes {
    pub id:                     u32,
    pub name:                   String,
    pub description:            String,
    pub max_qty:                u32,
    pub max_earnable_per_week:  u32,
    pub flags:                  u32,
    pub category_id:            u8,
    pub spell_category:         u8,
    pub quality:                u8,
    pub inventory_icon_file_id: i32,
    pub spell_weight:           u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x17EA5154)]
pub struct Curve {
    pub id:    u32,
    pub typ:   u8,
    pub flags: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF36752EB)]
pub struct CurvePoint {
    pub id:          u32,
    pub pos:         [f32; 2],
    pub curve_id:    u16,
    pub order_index: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x1092C9AF)]
pub struct DestructibleModelData {
    pub id: u32,
    pub state0_wmo: u16,
    pub state1_wmo: u16,
    pub state2_wmo: u16,
    pub state3_wmo: u16,
    pub heal_effect_speed: u16,
    pub state0_impact_effect_doodad_set: i8,
    pub state0_ambient_doodad_set: u8,
    pub state0_name_set: i8,
    pub state1_destruction_doodad_set: i8,
    pub state1_impact_effect_doodad_set: i8,
    pub state1_ambient_doodad_set: u8,
    pub state1_name_set: i8,
    pub state2_destruction_doodad_set: i8,
    pub state2_impact_effect_doodad_set: i8,
    pub state2_ambient_doodad_set: u8,
    pub state2_name_set: i8,
    pub state3_init_doodad_set: u8,
    pub state3_ambient_doodad_set: u8,
    pub state3_name_set: i8,
    pub eject_direction: u8,
    pub do_not_highlight: u8,
    pub heal_effect: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x92302BB8)]
pub struct Difficulty {
    pub id: u32,
    pub name: String,
    pub group_size_health_curve_id: u16,
    pub group_size_dmg_curve_id: u16,
    pub group_size_spell_points_curve_id: u16,
    pub fallback_difficulty_id: u8,
    pub instance_type: u8,
    pub min_players: u8,
    pub max_players: u8,
    pub old_enum_value: i8,
    pub flags: u8,
    pub toggle_difficulty_id: u8,
    pub item_context: u8,
    pub order_index: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB04A2596)]
pub struct DungeonEncounter {
    pub name:                String,
    pub creature_display_id: i32,
    #[parent]
    pub map_id:              i16,
    pub difficulty_id:       i8,
    pub bit:                 i8,
    pub flags:               u8,
    #[id]
    pub id:                  u32,
    pub order_index:         i32,
    pub spell_icon_file_id:  i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x8447966A)]
pub struct DurabilityCosts {
    pub id:                    u32,
    pub weapon_sub_class_cost: [u16; 21],
    pub armor_sub_class_cost:  [u16; 8],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6F64793D)]
pub struct DurabilityQuality {
    pub id:   u32,
    pub data: f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x14467F27)]
pub struct Emotes {
    pub id:                    u32,
    pub race_mask:             i64,
    pub emote_slash_command:   String,
    pub emote_flags:           u32,
    pub spell_visual_kit_id:   u32,
    pub anim_id:               i16,
    pub emote_spec_proc:       u8,
    pub class_mask:            i32,
    pub emote_spec_proc_param: u32,
    pub event_sound_id:        u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE85AFA10)]
pub struct EmotesText {
    pub id:       u32,
    pub name:     String,
    pub emote_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6DFAF9BC)]
pub struct EmotesTextSound {
    pub id:             u32,
    pub race_id:        u8,
    pub sex_id:         u8,
    pub class_id:       u8,
    pub sound_id:       u32,
    #[parent]
    pub emotes_text_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6BFE8737)]
pub struct Faction {
    pub reputation_race_mask:  [i64; 4],
    pub name:                  String,
    pub description:           String,
    #[id]
    pub id:                    u32,
    pub reputation_base:       [i32; 4],
    /// Faction outputs rep * ParentFactionModOut as spillover reputation
    pub parent_faction_mod:    [f32; 2],
    pub reputation_max:        [i32; 4],
    pub reputation_index:      i16,
    pub reputation_class_mask: [i16; 4],
    pub reputation_flags:      [u16; 4],
    pub parent_faction_id:     u16,
    pub paragon_faction_id:    u16,
    /// The highest rank the faction will profit from incoming spillover
    pub parent_faction_cap:    [u8; 2],
    pub expansion:             u8,
    pub flags:                 u8,
    pub friendship_rep_id:     u8,
}

impl Faction {
    pub fn can_have_reputation(&self) -> bool {
        self.reputation_index >= 0
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6F1D2135)]
pub struct FactionTemplate {
    pub id:            u32,
    pub faction:       u16,
    pub flags:         u16,
    pub enemies:       [u16; 4],
    pub friend:        [u16; 4],
    pub faction_group: u8,
    pub friend_group:  u8,
    pub enemy_group:   u8,
}

// impl FactionTemplate {
//     // helpers
//     pub fn is_friendly_to(&self, entry: &FactionTemplate) -> bool
//     {
//         if self == entry {
//             return true;
//         }
//         if (entry.faction > 0)
//         {
//             for e in self.enemies {
//                 if e == entry.faction {
//                     return false
//                 }
//             }
//             for e in self.friend {
//                 if e == entry.faction {
//                     return true
//                 }
//             }
//         }
//         (self.friend_group & entry.faction_group) > 0|| (self.faction_group & entry.friend_group) > 0
//     }
//     pub fn is_hostile_to(&self, entry: &FactionTemplate) -> bool
//     {
//         if (self == entry){
//             return false;
//         }
//         if (entry.faction> 0)
//         {
//             for e in self.enemies {
//                 if e == entry.faction {
//                     return true
//                 }
//             }
//             for e in self.friend {
//                 if e == entry.faction {
//                     return false
//                 }
//             }
//         }
//         return (self.enemy_group & entry.faction_group) != 0;
//     }
//     pub fn is_hostile_to_players(&self, ) -> bool { return (self.enemy_group & FACTION_MASK_PLAYER) !=0; }
//     pub fn is_neutral_to_all(&self, ) -> bool
//     {
//         for e in self.enemies {
//             if e != 0 {
//                 return false
//             }
//         }
//         return self.enemy_group == 0 && FriendGroup == 0;
//     }
//     pub fn is_contested_guard_faction(&self, ) -> bool { return (self.flags & FACTION_TEMPLATE_FLAG_CONTESTED_GUARD) != 0; }
//     pub fn should_spar_attack(&self, ) -> bool { return (self.flags & FACTION_TEMPLATE_ENEMY_SPAR) != 0; }
// }

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x9F2098D1)]
pub struct GameObjectDisplayInfo {
    pub id: u32,
    pub file_data_id: i32,
    pub geo_box: [f32; 6],
    pub override_loot_effect_scale: f32,
    pub override_name_scale: f32,
    pub object_effect_package_id: i16,
}

// pub struct GameObjectsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sfffihhhhbbi";
//         static const: u8,arraySizes[12] = { 1, 3, 4, 1, 8, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(11, 12, , types, arraySizes, 5);
//         return &instance;
//     }
// };

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x597E8643)]
pub struct GameObjects {
    pub name:            String,
    pub pos:             [f32; 3],
    pub rot:             [f32; 4],
    pub scale:           f32,
    pub prop_value:      [i32; 8],
    #[parent]
    pub owner_id:        u16,
    pub display_id:      u16,
    pub phase_id:        u16,
    pub phase_group_id:  u16,
    pub phase_use_flags: u8,
    pub type_id:         u8,
    #[id]
    pub id:              u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x5DF95DBD)]
pub struct GarrAbility {
    pub name: String,
    pub description: String,
    pub icon_file_data_id: i32,
    pub flags: u16,
    pub faction_change_garr_ability_id: u16,
    pub garr_ability_category_id: u8,
    pub garr_follower_type_id: u8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x200F9858)]
pub struct GarrBuilding {
    pub id: u32,
    pub alliance_name: String,
    pub horde_name: String,
    pub description: String,
    pub tooltip: String,
    pub horde_game_object_id: i32,
    pub alliance_game_object_id: i32,
    pub icon_file_data_id: i32,
    pub currency_type_id: u16,
    pub horde_ui_texture_kit_id: u16,
    pub alliance_ui_texture_kit_id: u16,
    pub alliance_scene_script_package_id: u16,
    pub horde_scene_script_package_id: u16,
    pub garr_ability_id: u16,
    pub bonus_garr_ability_id: u16,
    pub gold_cost: u16,
    pub garr_site_id: u8,
    pub building_type: u8,
    pub upgrade_level: u8,
    pub flags: u8,
    pub shipment_capacity: u8,
    pub garr_type_id: u8,
    pub build_seconds: i32,
    pub currency_qty: i32,
    pub max_assignments: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF45B6227)]
pub struct GarrBuildingPlotInst {
    pub map_offset: [f32; 2],
    pub ui_texture_atlas_member_id: u16,
    pub garr_site_level_plot_inst_id: u16,
    #[parent]
    pub garr_building_id: u8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x194CD478)]
pub struct GarrClassSpec {
    pub class_spec: String,
    pub class_spec_male: String,
    pub class_spec_female: String,
    pub ui_texture_atlas_member_id: u16,
    pub garr_foll_item_set_id: u16,
    pub follower_class_limit: u8,
    pub flags: u8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xAAB75E04)]
pub struct GarrFollower {
    pub horde_source_text: String,
    pub alliance_source_text: String,
    pub title_name: String,
    pub horde_creature_id: i32,
    pub alliance_creature_id: i32,
    pub horde_icon_file_data_id: i32,
    pub alliance_icon_file_data_id: i32,
    pub horde_slotting_broadcast_text_id: u32,
    pub ally_slotting_broadcast_text_id: u32,
    pub horde_garr_foll_item_set_id: u16,
    pub alliance_garr_foll_item_set_id: u16,
    pub item_level_weapon: u16,
    pub item_level_armor: u16,
    pub horde_ui_texture_kit_id: u16,
    pub alliance_ui_texture_kit_id: u16,
    pub garr_follower_type_id: u8,
    pub horde_garr_foll_race_id: u8,
    pub alliance_garr_foll_race_id: u8,
    pub quality: u8,
    pub horde_garr_class_spec_id: u8,
    pub alliance_garr_class_spec_id: u8,
    pub follower_level: u8,
    pub gender: u8,
    pub flags: u8,
    pub horde_source_type_enum: i8,
    pub alliance_source_type_enum: i8,
    pub garr_type_id: u8,
    pub vitality: u8,
    pub chr_class_id: u8,
    pub horde_flavor_garr_string_id: u8,
    pub alliance_flavor_garr_string_id: u8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x996447F1)]
pub struct GarrFollowerXAbility {
    pub id:               u32,
    pub garr_ability_id:  u16,
    pub faction_index:    u8,
    #[parent]
    pub garr_follower_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE12049E0)]
pub struct GarrPlot {
    pub id: u32,
    pub name: String,
    pub alliance_construct_obj_id: i32,
    pub horde_construct_obj_id: i32,
    pub ui_category_id: u8,
    pub plot_type: u8,
    pub flags: u8,
    pub upgrade_requirement: [u32; 2],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x3F77A6FA)]
pub struct GarrPlotBuilding {
    pub id:               u32,
    pub garr_plot_id:     u8,
    pub garr_building_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB708BB37)]
pub struct GarrPlotInstance {
    pub id:           u32,
    pub name:         String,
    pub garr_plot_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD3979C38)]
pub struct GarrSiteLevel {
    pub id:                 u32,
    pub town_hall_ui_pos:   [f32; 2],
    pub map_id:             u16,
    pub ui_texture_kit_id:  u16,
    pub upgrade_movie_id:   u16,
    pub upgrade_cost:       u16,
    pub upgrade_gold_cost:  u16,
    pub garr_level:         u8,
    pub garr_site_id:       u8,
    pub max_building_level: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC4E74201)]
pub struct GarrSiteLevelPlotInst {
    pub id:                    u32,
    pub ui_marker_pos:         [f32; 2],
    #[parent]
    pub garr_site_level_id:    u16,
    pub garr_plot_instance_id: u8,
    pub ui_marker_size:        u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x84558CAB)]
pub struct GemProperties {
    pub id:             u32,
    pub typ:            u32,
    pub enchant_id:     u16,
    pub min_item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xEA228DFA)]
pub struct GlyphBindableSpell {
    pub id:                  u32,
    pub spell_id:            i32,
    #[parent]
    pub glyph_properties_id: i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD0046829)]
pub struct GlyphProperties {
    pub id: u32,
    pub spell_id: u32,
    pub spell_icon_id: u16,
    pub glyph_type: u8,
    pub glyph_exclusive_category_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDD6481CE)]
pub struct GlyphRequiredSpec {
    pub id:                    u32,
    pub chr_specialization_id: u16,
    #[parent]
    pub glyph_properties_id:   u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCC0CEFF1)]
pub struct GuildColorBackground {
    pub id:    u32,
    pub red:   u8,
    pub green: u8,
    pub blue:  u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCC0CEFF1)]
pub struct GuildColorBorder {
    pub id:    u32,
    pub red:   u8,
    pub green: u8,
    pub blue:  u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCC0CEFF1)]
pub struct GuildColorEmblem {
    pub id:    u32,
    pub red:   u8,
    pub green: u8,
    pub blue:  u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC15D6E9F)]
pub struct GuildPerkSpells {
    pub id:       u32,
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x36887C6F)]
pub struct Heirloom {
    pub source_text: String,
    pub item_id: i32,
    pub legacy_item_id: i32,
    pub legacy_upgraded_item_id: i32,
    pub static_upgraded_item_id: i32,
    pub upgrade_item_id: [i32; 3],
    pub upgrade_item_bonus_list_id: [u16; 3],
    pub flags: u8,
    pub source_type_enum: i8,
    #[id]
    pub id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x7C3E60FC)]
pub struct Holidays {
    #[id]
    pub id:                     u32,
    /// dates in unix time starting at January, 1, 2000
    pub date:                   [u32; 16],
    pub duration:               [u16; 10],
    pub region:                 u16,
    pub looping:                u8,
    pub calendar_flags:         [u8; 10],
    pub priority:               u8,
    pub calendar_filter_type:   i8,
    pub flags:                  u8,
    pub holiday_name_id:        u32,
    pub holiday_description_id: u32,
    pub texture_file_data_id:   [i32; 3],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x1F7A850F)]
pub struct ImportPriceArmor {
    pub id:               u32,
    pub cloth_modifier:   f32,
    pub leather_modifier: f32,
    pub chain_modifier:   f32,
    pub plate_modifier:   f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6F64793D)]
pub struct ImportPriceQuality {
    pub id:   u32,
    pub data: f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6F64793D)]
pub struct ImportPriceShield {
    pub id:   u32,
    pub data: f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6F64793D)]
pub struct ImportPriceWeapon {
    pub id:   u32,
    pub data: f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0DFCC83D)]
pub struct Item {
    pub id: u32,
    pub icon_file_data_id: i32,
    pub class_id: u8,
    pub subclass_id: u8,
    pub sound_override_subclass_id: i8,
    pub material: u8,
    pub inventory_type: u8,
    pub sheathe_type: u8,
    pub item_group_sounds_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x06D35A59)]
pub struct ItemAppearance {
    pub id: u32,
    pub item_display_info_id: i32,
    pub default_icon_file_data_id: i32,
    pub ui_order: i32,
    pub display_type: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x85642CC0)]
pub struct ItemArmorQuality {
    pub id:         u32,
    pub qualitymod: [f32; 7],
    pub item_level: i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC2186F95)]
pub struct ItemArmorShield {
    pub id:         u32,
    pub quality:    [f32; 7],
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x45C396DD)]
pub struct ItemArmorTotal {
    pub id:         u32,
    pub cloth:      f32,
    pub leather:    f32,
    pub mail:       f32,
    pub plate:      f32,
    pub item_level: i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x96663ABF)]
pub struct ItemBagFamily {
    pub id:   u32,
    pub name: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE12FB1A0)]
pub struct ItemBonus {
    pub id: u32,
    pub value: [i32; 3],
    pub parent_item_bonus_list_id: u16,
    pub typ: u8,
    pub order_index: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDFBF5AC9)]
pub struct ItemBonusListLevelDelta {
    pub item_level_delta: i16,
    #[id]
    pub id:               u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x84FE93B7)]
pub struct ItemBonusTreeNode {
    pub id: u32,
    pub child_item_bonus_tree_id: u16,
    pub child_item_bonus_list_id: u16,
    pub child_item_level_selector_id: u16,
    pub item_context: u8,
    #[parent]
    pub parent_item_bonus_tree_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB6940674)]
pub struct ItemChildEquipment {
    pub id:                    u32,
    pub child_item_id:         i32,
    pub child_item_equip_slot: u8,
    #[parent]
    pub parent_item_id:        i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA1E4663C)]
pub struct ItemClass {
    pub id:             u32,
    pub class_name:     String,
    pub price_modifier: f32,
    pub class_id:       i8,
    pub flags:          u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE2FF5688)]
pub struct ItemCurrencyCost {
    pub id:      u32,
    #[parent(inline)]
    pub item_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC2186F95)]
pub struct ItemDamageAmmo {
    pub id:         u32,
    pub quality:    [f32; 7],
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC2186F95)]
pub struct ItemDamageOneHand {
    pub id:         u32,
    pub quality:    [f32; 7],
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC2186F95)]
pub struct ItemDamageOneHandCaster {
    pub id:         u32,
    pub quality:    [f32; 7],
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC2186F95)]
pub struct ItemDamageTwoHand {
    pub id:         u32,
    pub quality:    [f32; 7],
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC2186F95)]
pub struct ItemDamageTwoHandCaster {
    pub id:         u32,
    pub quality:    [f32; 7],
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC0D926CC)]
pub struct ItemDisenchantLoot {
    pub id:             u32,
    pub min_level:      u16,
    pub max_level:      u16,
    pub skill_required: u16,
    pub subclass:       i8,
    pub quality:        u8,
    pub expansion_id:   i8,
    #[parent]
    pub class:          u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA390FA40)]
pub struct ItemEffect {
    pub id: u32,
    pub spell_id: i32,
    pub cool_down_m_sec: i32,
    pub category_cool_down_m_sec: i32,
    pub charges: i16,
    pub spell_category_id: u16,
    pub chr_specialization_id: u16,
    pub legacy_slot_index: u8,
    pub trigger_type: i8,
    #[parent]
    pub parent_item_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC31F4DEF)]
pub struct ItemExtendedCost {
    pub id:                    u32,
    /// required item id
    pub item_id:               [i32; 5],
    /// required curency count
    pub currency_count:        [u32; 5],
    /// required count of 1st item
    pub item_count:            [u16; 5],
    /// required personal arena rating
    pub required_arena_rating: u16,
    /// required curency id
    pub currency_id:           [u16; 5],
    /// arena slot restrictions (min slot value)
    pub arena_bracket:         u8,
    pub min_faction_id:        u8,
    pub min_reputation:        u8,
    pub flags:                 u8,
    pub required_achievement:  u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x8143060E)]
pub struct ItemLevelSelector {
    pub id: u32,
    pub min_item_level: u16,
    pub item_level_selector_quality_set_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB7174A51)]
pub struct ItemLevelSelectorQuality {
    pub id: u32,
    pub quality_item_bonus_list_id: i32,
    pub quality: i8,
    #[parent]
    pub parent_ils_quality_set_id: i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x20055BA8)]
pub struct ItemLevelSelectorQualitySet {
    pub id:        u32,
    pub ilvl_rare: i16,
    pub ilvl_epic: i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB6BB188D)]
pub struct ItemLimitCategory {
    pub id:       u32,
    pub name:     String,
    pub quantity: u8,
    pub flags:    u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDE8EAD49)]
pub struct ItemLimitCategoryCondition {
    pub id: u32,
    pub add_quantity: i8,
    pub player_condition_id: u32,
    #[parent]
    pub parent_item_limit_category_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE64FD18B)]
pub struct ItemModifiedAppearance {
    #[parent]
    pub item_id: i32,
    #[id]
    pub id: u32,
    pub item_appearance_modifier_id: u8,
    pub item_appearance_id: u16,
    pub order_index: u8,
    pub transmog_source_type_enum: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4BD234D7)]
pub struct ItemPriceBase {
    pub id:         u32,
    pub armor:      f32,
    pub weapon:     f32,
    pub item_level: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB67375F8)]
pub struct ItemRandomProperties {
    pub id:          u32,
    pub name:        String,
    pub enchantment: [u16; 5],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x95CAB825)]
pub struct ItemRandomSuffix {
    pub id:             u32,
    pub name:           String,
    pub enchantment:    [u16; 5],
    pub allocation_pct: [u16; 5],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2D4B72FA)]
pub struct ItemSearchName {
    pub allowable_race:      i64,
    pub display:             String,
    #[id]
    pub id:                  u32,
    pub flags:               [i32; 3],
    pub item_level:          u16,
    pub overall_quality_id:  u8,
    pub expansion_id:        u8,
    pub required_level:      i8,
    pub min_faction_id:      u16,
    pub min_reputation:      u8,
    pub allowable_class:     i32,
    pub required_skill:      u16,
    pub required_skill_rank: u16,
    pub required_ability:    u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x847FF58A)]
pub struct ItemSet {
    pub id:                  u32,
    pub name:                String,
    pub item_id:             [u32; 17],
    pub required_skill_rank: u16,
    pub required_skill:      u32,
    pub set_flags:           u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF65D0AF8)]
pub struct ItemSetSpell {
    pub id:          u32,
    pub spell_id:    u32,
    pub chr_spec_id: u16,
    pub threshold:   u8,
    #[parent]
    pub item_set_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4007DE16)]
pub struct ItemSparse {
    pub id: u32,
    pub allowable_race: i64,
    pub display: String,
    pub display1: String,
    pub display2: String,
    pub display3: String,
    pub description: String,
    pub flags: [i32; 4],
    pub price_random_value: f32,
    pub price_variance: f32,
    pub vendor_stack_count: u32,
    pub buy_price: u32,
    pub sell_price: u32,
    pub required_ability: u32,
    pub max_count: i32,
    pub stackable: i32,
    pub stat_percent_editor: [i32; 10],
    pub stat_percentage_of_socket: [f32; 10],
    pub item_range: f32,
    pub bag_family: u32,
    pub quality_modifier: f32,
    pub duration_in_inventory: u32,
    pub dmg_variance: f32,
    pub allowable_class: i16,
    pub item_level: u16,
    pub required_skill: u16,
    pub required_skill_rank: u16,
    pub min_faction_id: u16,
    pub item_stat_value: [i16; 10],
    pub scaling_stat_distribution_id: u16,
    pub item_delay: u16,
    pub page_id: u16,
    pub start_quest_id: u16,
    pub lock_id: u16,
    pub random_select: u16,
    pub item_random_suffix_group_id: u16,
    pub item_set: u16,
    pub zone_bound: u16,
    pub instance_bound: u16,
    pub totem_category_id: u16,
    pub socket_match_enchantment_id: u16,
    pub gem_properties: u16,
    pub limit_category: u16,
    pub required_holiday: u16,
    pub required_transmog_holiday: u16,
    pub item_name_description_id: u16,
    pub overall_quality_id: u8,
    pub inventory_type: u8,
    pub required_level: i8,
    pub required_pvp_rank: u8,
    pub required_pvp_medal: u8,
    pub min_reputation: u8,
    pub container_slots: u8,
    pub stat_modifier_bonus_stat: [i8; 10],
    pub damage_damage_type: u8,
    pub bonding: u8,
    pub language_id: u8,
    pub page_material_id: u8,
    pub material: u8,
    pub sheathe_type: u8,
    pub socket_type: [u8; 3],
    pub spell_weight_category: u8,
    pub spell_weight: u8,
    pub artifact_id: u8,
    pub expansion_id: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB17B7986)]
pub struct ItemSpec {
    pub id:                u32,
    pub specialization_id: u16,
    pub min_level:         u8,
    pub max_level:         u8,
    #[parent]
    pub item_type:         u8,
    pub primary_stat:      u8,
    pub secondary_stat:    u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE499CD2A)]
pub struct ItemSpecOverride {
    pub id:      u32,
    pub spec_id: u16,
    #[parent]
    pub item_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x8F3A4137)]
pub struct ItemUpgrade {
    pub id:                   u32,
    pub currency_amount:      u32,
    pub prerequisite_id:      u16,
    pub currency_type:        u16,
    pub item_upgrade_path_id: u8,
    pub item_level_increment: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x87C4B605)]
pub struct ItemXBonusTree {
    pub id:                 u32,
    pub item_bonus_tree_id: u16,
    #[parent]
    pub item_id:            i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x5B214E82)]
pub struct Keychain {
    pub id:  u32,
    pub key: [u8; 32],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF02081A0)]
pub struct LFGDungeons {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub flags: i32,
    pub min_gear: f32,
    pub max_level: u16,
    pub target_level_max: u16,
    pub map_id: i16,
    pub random_id: u16,
    pub scenario_id: u16,
    pub final_encounter_id: u16,
    pub bonus_reputation_amount: u16,
    pub mentor_item_level: u16,
    pub required_player_condition_id: u16,
    pub min_level: u8,
    pub target_level: u8,
    pub target_level_min: u8,
    pub difficulty_id: u8,
    pub type_id: u8,
    pub faction: i8,
    pub expansion_level: u8,
    pub order_index: u8,
    pub group_id: u8,
    pub count_tank: u8,
    pub count_healer: u8,
    pub count_damage: u8,
    pub min_count_tank: u8,
    pub min_count_healer: u8,
    pub min_count_damage: u8,
    pub subtype: u8,
    pub mentor_char_level: u8,
    pub icon_texture_file_id: i32,
    pub rewards_bg_texture_file_id: i32,
    pub popup_bg_texture_file_id: i32,
}

impl LFGDungeons {
    pub fn entry(&self) -> u32 {
        self.id + ((self.type_id as u32) << 24)
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x25025A13)]
pub struct Light {
    pub id:                 u32,
    pub game_coords:        [f32; 3],
    pub game_falloff_start: f32,
    pub game_falloff_end:   f32,
    pub continent_id:       i16,
    pub light_params_id:    [u16; 8],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x3313BBF3)]
pub struct LiquidType {
    pub id:                   u32,
    pub name:                 String,
    pub texture:              [String; 6],
    pub spell_id:             u32,
    pub max_darken_depth:     f32,
    pub fog_darken_intensity: f32,
    pub amb_darken_intensity: f32,
    pub dir_darken_intensity: f32,
    pub particle_scale:       f32,
    pub color:                [i32; 2],
    pub float:                [f32; 18],
    pub int:                  [u32; 4],
    pub flags:                u16,
    pub light_id:             u16,
    /// used to be "type", maybe needs fixing (works well for now)
    pub sound_bank:           u8,
    pub particle_movement:    u8,
    pub particle_tex_slots:   u8,
    pub material_id:          u8,
    pub frame_count_texture:  [u8; 6],
    pub sound_id:             u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDAC7F42F)]
pub struct Lock {
    pub id:     u32,
    pub index:  [i32; 8],
    pub skill:  [u16; 8],
    pub typ:    [u8; 8],
    pub action: [u8; 8],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x25C8D6CC)]
pub struct MailTemplate {
    pub id:   u32,
    pub body: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF568DF12)]
pub struct Map {
    pub id:                     u32,
    pub directory:              String,
    pub map_name:               String,
    /// Horde
    pub map_description0:       String,
    /// Alliance
    pub map_description1:       String,
    pub pvp_short_description:  String,
    pub pvp_long_description:   String,
    pub flags:                  [i32; 2],
    pub minimap_icon_scale:     f32,
    /// entrance coordinates in ghost mode  (in most cases = normal entrance)
    pub corpse:                 [f32; 2],
    pub area_table_id:          u16,
    pub loading_screen_id:      i16,
    /// map_id of entrance map in ghost mode (continent always and in most cases = normal entrance)
    pub corpse_map_id:          i16,
    pub time_of_day_override:   i16,
    pub parent_map_id:          i16,
    pub cosmetic_parent_map_id: i16,
    pub wind_settings_id:       i16,
    pub instance_type:          u8,
    pub map_type:               u8,
    pub expansion_id:           u8,
    pub max_players:            u8,
    pub time_offset:            u8,
}

// enum MapTypes                                               // Lua_IsInInstance
// {
//                                     /// none
//     Common          = 0,
//                                     /// party
//     Instance        = 1,
//                                     /// raid
//     Raid            = 2,
//                                     /// pvp
//     Battleground    = 3,
//                                      /// arena
//     Arena           = 4,
// }

// enum MapFlags
// {
//     CanToggleDifficulty  = 0x0100,
//     /// All difficulties share completed encounters lock, not bound to a single instance id
//     /// heroic difficulty flag overrides it and uses instance id bind
//     FlexLocking           = 0x8000,
//     Garrison               = 0x4000000,
// }

// impl Map {

//     // Helpers
//     pub fn expansion(&self) -> u8 { return self.expansion_id }

//     pub fn is_dungeon(&self, ) { return (self.instance_type == MapTypes::Instance as u8 || self.instance_type == MapTypes::Raid as u8 || self.instance_type == MapTypes::Scenario as u8) && !IsGarrison(); }
//     pub fn is_non_raid_dungeon(&self, ) { return self.instance_type == MapTypes::Instance as u8; }
//     pub fn instanceable(&self, ) { return self.instance_type == MapTypes::Instance as u8 || self.instance_type == MapTypes::Raid as u8 || self.instance_type == MapTypes::Battleground as u8 || self.instance_type == MapTypes::Arena as u8 || self.instance_type == MapTypes::Scenario as u8; }
//     pub fn is_raid(&self, ) { return self.instance_type == MapTypes::Raid as u8; }
//     pub fn is_battleground(&self, ) { return self.instance_type == MapTypes::Battleground as u8; }
//     pub fn is_battle_arena(&self, ) { return self.instance_type == MapTypes::Arena as u8; }
//     pub fn is_battleground_or_arena(&self, ) { return self.instance_type == MapTypes::Battleground as u8 || self.instance_type == MapTypes::Arena as u8; }
//     pub fn is_world_map(&self, ) { return self.instance_type == MapTypes::Common as u8; }

//     pub fn get_entrance_pos(&self) -> Option<(i16, f32, f32)>
//     {
//         if (self.corpse_map_id < 0) {
//             return None;
//         }
//         Some((self.corpse_map_id, self.corpse[0], self.corpse[1]))
//     }

//     pub fn is_continent(&self, ) -> bool
//     {
//         return self.id == 0 || self.id == 1 || self.id == 530 || self.id == 571 || self.id == 870 || self.id == 1116 || self.id == 1220;
//     }

//     pub fn is_dynamic_difficulty_map(&self, ) { return (self.flags[0] & MapFlags::CanToggleDifficulty as i32) != 0; }
//     pub fn is_garrison(&self, ) { return (self.flags[0] & MapFlags::Garrison as i32) != 0; }
// }

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2B3B759E)]
pub struct MapDifficulty {
    pub id:                     u32,
    /// m_message_lang (text showed when transfer to map failed)
    pub message:                String,
    pub difficulty_id:          u8,
    pub reset_interval:         u8,
    pub max_players:            u8,
    pub lock_id:                u8,
    pub flags:                  u8,
    pub item_context:           u8,
    pub item_context_picker_id: u32,
    #[parent]
    pub map_id:                 u16,
}

impl MapDifficulty {
    pub fn get_raid_duration(&self) -> u32 {
        if self.reset_interval == 1 {
            return 86400;
        }
        if self.reset_interval == 2 {
            return 604800;
        }
        0
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x7718AFC2)]
pub struct ModifierTree {
    pub id:              u32,
    pub asset:           i32,
    pub secondary_asset: i32,
    pub parent:          u32,
    pub typ:             u8,
    pub tertiary_asset:  i8,
    pub operator:        i8,
    pub amount:          i8,
}

pub enum MountFlags {
    /// Player becomes the mount himself
    SelfMount = 0x02,
    FactionSpecific = 0x04,
    PreferredSwimming = 0x10,
    PreferredWaterWalking = 0x20,
    HideIfUnknown = 0x40,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4D812F19)]
pub struct Mount {
    pub name:                  String,
    pub description:           String,
    pub source_text:           String,
    pub source_spell_id:       i32,
    pub mount_fly_ride_height: f32,
    pub mount_type_id:         u16,
    pub flags:                 u16,
    pub source_type_enum:      i8,
    #[id]
    pub id:                    u32,
    pub player_condition_id:   u32,
    pub ui_model_scene_id:     i32,
}

impl Mount {
    pub fn is_self_mount(&self) -> bool {
        (self.flags & MountFlags::SelfMount as u16) != 0
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB0D11D52)]
pub struct MountCapability {
    pub req_spell_known_id: i32,
    pub mod_spell_aura_id:  i32,
    pub req_riding_skill:   u16,
    pub req_area_id:        u16,
    pub req_map_id:         i16,
    pub flags:              u8,
    #[id]
    pub id:                 u32,
    pub req_spell_aura_id:  u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA34A8445)]
pub struct MountTypeXCapability {
    pub id:                  u32,
    #[parent]
    pub mount_type_id:       u16,
    pub mount_capability_id: u16,
    pub order_index:         u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD59B9FE4)]
pub struct MountXDisplay {
    pub id: u32,
    pub creature_display_info_id: i32,
    pub player_condition_id: u32,
    #[parent]
    pub mount_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF3E9AE3B)]
pub struct Movie {
    pub id:                    u32,
    pub audio_file_data_id:    u32,
    pub subtitle_file_data_id: u32,
    pub volume:                u8,
    pub key_id:                u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2EF936CD)]
pub struct NameGen {
    pub id:      u32,
    pub name:    String,
    pub race_id: u8,
    pub sex:     u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDFB56E0E)]
pub struct NamesProfanity {
    pub id:       u32,
    pub name:     String,
    pub language: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE4923C1F)]
pub struct NamesReserved {
    pub id:   u32,
    pub name: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC1403093)]
pub struct NamesReservedLocale {
    pub id:          u32,
    pub name:        String,
    pub locale_mask: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x9417628C)]
pub struct OverrideSpellData {
    pub id: u32,
    pub spells: [i32; 10],
    pub player_action_bar_file_data_id: i32,
    pub flags: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0043219C)]
pub struct Phase {
    pub id:    u32,
    pub flags: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x66517AF6)]
pub struct PhaseXPhaseGroup {
    pub id:             u32,
    pub phase_id:       u16,
    #[parent]
    pub phase_group_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x5B3DA113)]
pub struct PlayerCondition {
    pub race_mask: i64,
    pub failure_description: String,
    #[id]
    pub id: u32,
    pub flags: u8,
    pub min_level: u16,
    pub max_level: u16,
    pub class_mask: i32,
    pub gender: i8,
    pub native_gender: i8,
    pub skill_logic: u32,
    pub language_id: u8,
    pub min_language: u8,
    pub max_language: i32,
    pub max_faction_id: u16,
    pub max_reputation: u8,
    pub reputation_logic: u32,
    pub current_pvp_faction: i8,
    pub min_pvp_rank: u8,
    pub max_pvp_rank: u8,
    pub pvp_medal: u8,
    pub prev_quest_logic: u32,
    pub curr_quest_logic: u32,
    pub current_completed_quest_logic: u32,
    pub spell_logic: u32,
    pub item_logic: u32,
    pub item_flags: u8,
    pub aura_spell_logic: u32,
    pub world_state_expression_id: u16,
    pub weather_id: u8,
    pub party_status: u8,
    pub lifetime_max_pvp_rank: u8,
    pub achievement_logic: u32,
    pub lfg_logic: u32,
    pub area_logic: u32,
    pub currency_logic: u32,
    pub quest_kill_id: u16,
    pub quest_kill_logic: u32,
    pub min_expansion_level: i8,
    pub max_expansion_level: i8,
    pub min_expansion_tier: i8,
    pub max_expansion_tier: i8,
    pub min_guild_level: u8,
    pub max_guild_level: u8,
    pub phase_use_flags: u8,
    pub phase_id: u16,
    pub phase_group_id: u32,
    pub min_avg_item_level: i32,
    pub max_avg_item_level: i32,
    pub min_avg_equipped_item_level: u16,
    pub max_avg_equipped_item_level: u16,
    pub chr_specialization_index: i8,
    pub chr_specialization_role: i8,
    pub power_type: i8,
    pub power_type_comp: u8,
    pub power_type_value: u8,
    pub modifier_tree_id: u32,
    pub weapon_subclass_mask: i32,
    pub skill_id: [u16; 4],
    pub min_skill: [u16; 4],
    pub max_skill: [u16; 4],
    pub min_faction_id: [u32; 3],
    pub min_reputation: [u8; 3],
    pub prev_quest_id: [u16; 4],
    pub curr_quest_id: [u16; 4],
    pub current_completed_quest_id: [u16; 4],
    pub spell_id: [i32; 4],
    pub item_id: [i32; 4],
    pub item_count: [u32; 4],
    pub explored: [u16; 2],
    pub time: [u32; 2],
    pub aura_spell_id: [i32; 4],
    pub aura_stacks: [u8; 4],
    pub achievement: [u16; 4],
    pub lfg_status: [u8; 4],
    pub lfg_compare: [u8; 4],
    pub lfg_value: [u32; 4],
    pub area_id: [u16; 4],
    pub currency_id: [u32; 4],
    pub currency_count: [u32; 4],
    pub quest_kill_monster: [u32; 6],
    pub movement_flags: [i32; 2],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xFD152E5B)]
pub struct PowerDisplay {
    pub id:                     u32,
    pub global_string_base_tag: String,
    pub actual_type:            u8,
    pub red:                    u8,
    pub green:                  u8,
    pub blue:                   u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0C3844E1)]
pub struct PowerType {
    pub id: u32,
    pub name_global_string_tag: String,
    pub cost_global_string_tag: String,
    pub regen_peace: f32,
    pub regen_combat: f32,
    pub max_base_power: i16,
    pub regen_interrupt_time_ms: i16,
    pub flags: i16,
    pub power_type_enum: i8,
    pub min_power: i8,
    pub center_power: i8,
    pub default_power: i8,
    pub display_modifier: i8,
}

enum PrestigeLevelInfoFlags {
    /// Prestige levels with this flag won't be included to calculate max prestigelevel.
    Disabled = 0x01,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA7B2D559)]
pub struct PrestigeLevelInfo {
    pub id: u32,
    pub name: String,
    pub badge_texture_file_data_id: i32,
    pub prestige_level: u8,
    pub flags: u8,
}

impl PrestigeLevelInfo {
    pub fn is_disabled(&self) -> bool {
        (self.flags & PrestigeLevelInfoFlags::Disabled as u8) != 0
    }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x970B5E15)]
pub struct PVPDifficulty {
    pub id:          u32,
    pub range_index: u8,
    pub min_level:   u8,
    pub max_level:   u8,
    #[parent]
    pub map_id:      u16,
    // // helpers
    // BattlegroundBracketId GetBracketId() const { return BattlegroundBracketId(RangeIndex); }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xBD449801)]
pub struct PVPItem {
    pub id:               u32,
    pub item_id:          i32,
    pub item_level_delta: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x72F4C016)]
pub struct PvpReward {
    pub id:             u32,
    pub honor_level:    i32,
    pub prestige_level: i32,
    pub reward_pack_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x6EB51740)]
pub struct PvpTalent {
    pub id:                  u32,
    pub description:         String,
    pub spell_id:            i32,
    pub overrides_spell_id:  i32,
    pub action_bar_spell_id: i32,
    pub tier_id:             i32,
    pub column_index:        i32,
    pub flags:               i32,
    pub class_id:            i32,
    pub spec_id:             i32,
    pub role:                i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x465C83BC)]
pub struct PvpTalentUnlock {
    pub id:           u32,
    pub tier_id:      i32,
    pub column_index: i32,
    pub honor_level:  i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB0E02541)]
pub struct QuestFactionReward {
    pub id:         u32,
    pub difficulty: [i16; 10],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x86397302)]
pub struct QuestMoneyReward {
    pub id:         u32,
    pub difficulty: [u32; 10],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCF9401CF)]
pub struct QuestPackageItem {
    pub id:            u32,
    pub item_id:       i32,
    pub package_id:    u16,
    pub display_type:  u8,
    pub item_quantity: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xAD7072C6)]
pub struct QuestSort {
    pub id:             u32,
    pub sort_name:      String,
    pub ui_order_index: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x70495C9B)]
pub struct QuestV2 {
    pub id:              u32,
    pub unique_bit_flag: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCB76B4C0)]
pub struct QuestXP {
    pub id:         u32,
    pub difficulty: [u16; 10],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4E2C0BCC)]
pub struct RandPropPoints {
    pub id:       u32,
    pub epic:     [u32; 5],
    pub superior: [u32; 5],
    pub good:     [u32; 5],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDB6CC0AB)]
pub struct RewardPack {
    pub id: u32,
    pub money: u32,
    pub artifact_xp_multiplier: f32,
    pub artifact_xp_difficulty: i8,
    pub artifact_xp_category_id: u8,
    pub char_title_id: i32,
    pub treasure_picker_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x217E6712)]
pub struct RewardPackXCurrencyType {
    pub id:               u32,
    pub currency_type_id: u32,
    pub quantity:         i32,
    #[parent]
    pub reward_pack_id:   u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x74F6B9BD)]
pub struct RewardPackXItem {
    pub id:             u32,
    pub item_id:        i32,
    pub item_quantity:  i32,
    #[parent]
    pub reward_pack_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xFB641AE0)]
pub struct RulesetItemUpgrade {
    pub id:              u32,
    pub item_id:         i32,
    pub item_upgrade_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x5200B7F5)]
pub struct SandboxScaling {
    pub id:        u32,
    pub min_level: i32,
    pub max_level: i32,
    pub flags:     i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDED48286)]
pub struct ScalingStatDistribution {
    pub id: u32,
    pub player_level_to_item_level_curve_id: u16,
    pub min_level: i32,
    pub max_level: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD052232A)]
pub struct Scenario {
    pub id:            u32,
    pub name:          String,
    pub area_table_id: u16,
    pub flags:         u8,
    pub typ:           u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x201B0EFC)]
pub struct ScenarioStep {
    pub id:              u32,
    pub description:     String,
    pub title:           String,
    #[parent]
    pub scenario_id:     u16,
    /// Used in conjunction with Proving Grounds scenarios, when sequencing steps (Not using step order?)
    pub supersedes:      u16,
    pub reward_quest_id: u16,
    pub order_index:     u8,
    pub flags:           u8,
    pub criteriatreeid:  u32,
    /// Bonus step can only be completed if scenario is in the step specified in this field
    pub related_step:    i32,
    // // helpers
    // bool IsBonusObjective() const
    // {
    //     return self.flags & SCENARIO_STEP_FLAG_BONUS_OBJECTIVE;
    // }
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC694B81E)]
pub struct SceneScript {
    pub id:                    u32,
    pub first_scene_script_id: u16,
    pub next_scene_script_id:  u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB9F8FDF1)]
pub struct SceneScriptGlobalText {
    pub id:     u32,
    pub name:   String,
    pub script: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x96663ABF)]
pub struct SceneScriptPackage {
    pub id:   u32,
    pub name: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB9F8FDF1)]
pub struct SceneScriptText {
    pub id:     u32,
    pub name:   String,
    pub script: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x3F7E88AF)]
pub struct SkillLine {
    pub id:                   u32,
    pub display_name:         String,
    pub description:          String,
    pub alternate_verb:       String,
    pub flags:                u16,
    pub category_id:          i8,
    pub can_link:             i8,
    pub spell_icon_file_id:   i32,
    pub parent_skill_line_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x97B5A653)]
pub struct SkillLineAbility {
    pub race_mask: i64,
    #[id]
    pub id: u32,
    pub spell: i32,
    pub supercedes_spell: i32,
    #[parent]
    pub skill_line: i16,
    pub trivial_skill_line_rank_high: i16,
    pub trivial_skill_line_rank_low: i16,
    pub unique_bit: i16,
    pub trade_skill_category_id: i16,
    pub num_skill_ups: i8,
    pub class_mask: i32,
    pub min_skill_line_rank: i16,
    pub acquire_method: i8,
    pub flags: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x9752C2CE)]
pub struct SkillRaceClassInfo {
    pub id:            u32,
    pub race_mask:     i64,
    #[parent]
    pub skill_id:      i16,
    pub flags:         u16,
    pub skill_tier_id: i16,
    pub availability:  i8,
    pub min_level:     i8,
    pub class_mask:    i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0E9CB7AE)]
pub struct SoundKit {
    #[id]
    pub id: u32,
    pub volume_float: f32,
    pub min_distance: f32,
    pub distance_cutoff: f32,
    pub flags: u16,
    pub sound_entries_advanced_id: u16,
    pub sound_type: u8,
    pub dialog_type: u8,
    pub eax_def: u8,
    pub volume_variation_plus: f32,
    pub volume_variation_minus: f32,
    pub pitch_variation_plus: f32,
    pub pitch_variation_minus: f32,
    pub pitch_adjust: f32,
    pub bus_overwrite_id: u16,
    pub max_instances: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xAE3436F3)]
pub struct SpecializationSpells {
    pub description:        String,
    pub spell_id:           i32,
    pub overrides_spell_id: i32,
    #[parent]
    pub spec_id:            u16,
    pub display_order:      u8,
    #[id]
    pub id:                 u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2273DFFF)]
pub struct Spell {
    pub id:               u32,
    pub name:             String,
    pub name_subtext:     String,
    pub description:      String,
    pub aura_description: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE05BE94F)]
pub struct SpellAuraOptions {
    pub id: u32,
    pub proc_charges: i32,
    pub proc_type_mask: i32,
    pub proc_category_recovery: i32,
    pub cumulative_aura: u16,
    pub spell_procs_per_minute_id: u16,
    pub difficulty_id: u8,
    pub proc_chance: u8,
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x7CDF3311)]
pub struct SpellAuraRestrictions {
    pub id: u32,
    pub caster_aura_spell: i32,
    pub target_aura_spell: i32,
    pub exclude_caster_aura_spell: i32,
    pub exclude_target_aura_spell: i32,
    pub difficulty_id: u8,
    pub caster_aura_state: u8,
    pub target_aura_state: u8,
    pub exclude_caster_aura_state: u8,
    pub exclude_target_aura_state: u8,
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4129C6A4)]
pub struct SpellCastTimes {
    pub id:        u32,
    pub base:      i32,
    pub minimum:   i32,
    pub per_level: i16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD8B56E5D)]
pub struct SpellCastingRequirements {
    pub id:                   u32,
    pub spell_id:             i32,
    pub min_faction_id:       u16,
    pub required_areas_id:    u16,
    pub requires_spell_focus: u16,
    pub facing_caster_flags:  u8,
    pub min_reputation:       i8,
    pub required_aura_vision: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x14E916CC)]
pub struct SpellCategories {
    pub id: u32,
    pub category: i16,
    pub start_recovery_category: i16,
    pub charge_category: i16,
    pub difficulty_id: u8,
    pub defense_type: i8,
    pub dispel_type: i8,
    pub mechanic: i8,
    pub prevention_type: i8,
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xEA60E384)]
pub struct SpellCategory {
    pub id:                   u32,
    pub name:                 String,
    pub charge_recovery_time: i32,
    pub flags:                i8,
    pub uses_per_week:        u8,
    pub max_charges:          i8,
    pub type_mask:            i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x80FBD67A)]
pub struct SpellClassOptions {
    pub id:               u32,
    pub spell_id:         i32,
    pub spell_class_mask: [u32; 4],
    pub spell_class_set:  u8,
    pub modal_next_spell: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCA8D8B3C)]
pub struct SpellCooldowns {
    pub id:                     u32,
    pub category_recovery_time: i32,
    pub recovery_time:          i32,
    pub start_recovery_time:    i32,
    pub difficulty_id:          u8,
    #[parent]
    pub spell_id:               i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0D6C9082)]
pub struct SpellDuration {
    pub id:                 u32,
    pub duration:           i32,
    pub max_duration:       i32,
    pub duration_per_level: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x3244098B)]
pub struct SpellEffect {
    #[id]
    pub id: u32,
    pub effect: u32,
    pub effect_base_points: i32,
    pub effect_index: i32,
    pub effect_aura: i32,
    pub difficulty_id: i32,
    pub effect_amplitude: f32,
    pub effect_aura_period: i32,
    pub effect_bonus_coefficient: f32,
    pub effect_chain_amplitude: f32,
    pub effect_chain_targets: i32,
    pub effect_die_sides: i32,
    pub effect_item_type: i32,
    pub effect_mechanic: i32,
    pub effect_points_per_resource: f32,
    pub effect_real_points_per_level: f32,
    pub effect_trigger_spell: i32,
    pub effect_pos_facing: f32,
    pub effect_attributes: i32,
    pub bonus_coefficient_from_ap: f32,
    pub pvp_multiplier: f32,
    pub coefficient: f32,
    pub variance: f32,
    pub resource_coefficient: f32,
    pub group_size_base_points_coefficient: f32,
    pub effect_spell_class_mask: [u32; 4],
    pub effect_misc_value: [i32; 2],
    pub effect_radius_index: [u32; 2],
    pub implicit_target: [u32; 2],
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCE628176)]
pub struct SpellEquippedItems {
    pub id: u32,
    pub spell_id: i32,
    pub equipped_item_inv_types: i32,
    pub equipped_item_subclass: i32,
    pub equipped_item_class: i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x96663ABF)]
pub struct SpellFocusObject {
    pub id:   u32,
    pub name: String,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2FA8EA94)]
pub struct SpellInterrupts {
    pub id: u32,
    pub difficulty_id: u8,
    pub interrupt_flags: i16,
    pub aura_interrupt_flags: [i32; 2],
    pub channel_interrupt_flags: [i32; 2],
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x80DEA734)]
pub struct SpellItemEnchantment {
    pub id: u32,
    pub name: String,
    pub effect_arg: [u32; 3],
    pub effect_scaling_points: [f32; 3],
    pub transmog_cost: u32,
    pub icon_file_data_id: u32,
    pub effect_points_min: [i16; 3],
    pub item_visual: u16,
    pub flags: u16,
    pub required_skill_id: u16,
    pub required_skill_rank: u16,
    pub item_level: u16,
    pub charges: u8,
    pub effect: [u8; 3],
    pub condition_id: u8,
    pub min_level: u8,
    pub max_level: u8,
    pub scaling_class: i8,
    pub scaling_class_restricted: i8,
    pub transmog_player_condition_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB9C16961)]
pub struct SpellItemEnchantmentCondition {
    pub id:              u32,
    pub lt_operand:      [u32; 5],
    pub lt_operand_type: [u8; 5],
    pub operator:        [u8; 5],
    pub rt_operand_type: [u8; 5],
    pub rt_operand:      [u8; 5],
    pub logic:           [u8; 5],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x153EBA26)]
pub struct SpellLearnSpell {
    pub id:                 u32,
    pub spell_id:           i32,
    pub learn_spell_id:     i32,
    pub overrides_spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x9E7D1CCD)]
pub struct SpellLevels {
    pub id:                     u32,
    pub base_level:             i16,
    pub max_level:              i16,
    pub spell_level:            i16,
    pub difficulty_id:          u8,
    pub max_passive_aura_level: u8,
    #[parent]
    pub spell_id:               i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCDC114D5)]
pub struct SpellMisc {
    pub id: u32,
    pub casting_time_index: u16,
    pub duration_index: u16,
    pub range_index: u16,
    pub school_mask: u8,
    pub spell_icon_file_data_id: i32,
    pub speed: f32,
    pub active_icon_file_data_id: i32,
    pub launch_delay: f32,
    pub difficulty_id: u8,
    pub attributes: [i32; 14],
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x8E5E46EC)]
pub struct SpellPower {
    pub mana_cost:              i32,
    pub power_cost_pct:         f32,
    pub power_pct_per_second:   f32,
    pub required_aura_spell_id: i32,
    pub power_cost_max_pct:     f32,
    pub order_index:            u8,
    pub power_type:             i8,
    #[id]
    pub id:                     u32,
    pub mana_cost_per_level:    i32,
    pub mana_per_second:        i32,
    /// Spell uses [ManaCost, ManaCost+ManaCostAdditional] power - affects tooltip parsing as multiplier on SpellEffectEntry::EffectPointsPerResource
    ///   only SPELL_EFFECT_WEAPON_DAMAGE_NOSCHOOL, SPELL_EFFECT_WEAPON_PERCENT_DAMAGE, SPELL_EFFECT_WEAPON_DAMAGE, SPELL_EFFECT_NORMALIZED_WEAPON_DMG
    pub optional_cost:          u32,
    pub power_display_id:       u32,
    pub alt_power_bar_id:       i32,
    #[parent]
    pub spell_id:               i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x74714FF7)]
pub struct SpellPowerDifficulty {
    pub difficulty_id: u8,
    pub order_index:   u8,
    #[id]
    pub id:            u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4BC1931B)]
pub struct SpellProcsPerMinute {
    pub id:             u32,
    pub base_proc_rate: f32,
    pub flags:          u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2503C18B)]
pub struct SpellProcsPerMinuteMod {
    pub id: u32,
    pub coeff: f32,
    pub param: i16,
    pub typ: u8,
    #[parent]
    pub spell_procs_per_minute_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC12E5C90)]
pub struct SpellRadius {
    pub id:               u32,
    pub radius:           f32,
    pub radius_per_level: f32,
    pub radius_min:       f32,
    pub radius_max:       f32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDE2E3F8E)]
pub struct SpellRange {
    pub id:                 u32,
    pub display_name:       String,
    pub display_name_short: String,
    pub range_min:          [f32; 2],
    pub range_max:          [f32; 2],
    pub flags:              u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0463C688)]
pub struct SpellReagents {
    pub id:            u32,
    pub spell_id:      i32,
    pub reagent:       [i32; 8],
    pub reagent_count: [i16; 8],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF67A5719)]
pub struct SpellScaling {
    pub id:                     u32,
    pub spell_id:               i32,
    pub scales_from_item_level: i16,
    pub class:                  i32,
    pub min_scaling_level:      u32,
    pub max_scaling_level:      u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xA461C24D)]
pub struct SpellShapeshift {
    pub id:                 u32,
    pub spell_id:           [i32; 2],
    pub shapeshift_exclude: [i32; 2],
    pub shapeshift_mask:    [i32; 2],
    pub stance_bar_order:   i8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x130819AF)]
pub struct SpellShapeshiftForm {
    pub id:                  u32,
    pub name:                String,
    pub damage_variance:     f32,
    pub flags:               i32,
    pub combat_round_time:   i16,
    pub mount_type_id:       u16,
    pub creature_type:       i8,
    pub bonus_action_bar:    i8,
    pub attack_icon_file_id: i32,
    pub creature_display_id: [u32; 4],
    pub preset_spell_id:     [u32; 8],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x7B330026)]
pub struct SpellTargetRestrictions {
    pub id:                   u32,
    pub cone_degrees:         f32,
    pub width:                f32,
    pub targets:              i32,
    pub target_creature_type: i16,
    pub difficulty_id:        u8,
    pub max_targets:          u8,
    pub max_target_level:     u32,
    #[parent]
    pub spell_id:             i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xEC0C4866)]
pub struct SpellTotems {
    pub id: u32,
    pub spell_id: i32,
    pub totem: [i32; 2],
    pub required_totem_category_id: [u16; 2],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x4F4B8A2A)]
pub struct SpellXSpellVisual {
    pub spell_visual_id: u32,
    #[id]
    pub id: u32,
    pub probability: f32,
    pub caster_player_condition_id: u16,
    pub caster_unit_condition_id: u16,
    pub viewer_player_condition_id: u16,
    pub viewer_unit_condition_id: u16,
    pub spell_icon_file_id: i32,
    pub active_icon_file_id: i32,
    pub flags: u8,
    pub difficulty_id: u8,
    pub priority: u8,
    #[parent]
    pub spell_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xFB8338FC)]
pub struct SummonProperties {
    pub id:      u32,
    pub flags:   i32,
    pub control: i32,
    pub faction: i32,
    pub title:   i32,
    pub slot:    i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF0F98B62)]
pub struct TactKey {
    pub id:  u32,
    pub key: [u8; 16],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xE8850B48)]
pub struct Talent {
    pub id:                 u32,
    pub description:        String,
    pub spell_id:           u32,
    pub overrides_spell_id: u32,
    pub spec_id:            u16,
    pub tier_id:            u8,
    pub column_index:       u8,
    pub flags:              u8,
    pub category_mask:      [u8; 2],
    pub class_id:           u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB46C6A8B)]
pub struct TaxiNodes {
    pub id: u32,
    pub name: String,
    pub pos: [f32; 3],
    pub mount_creature_id: [i32; 2],
    pub map_offset: [f32; 2],
    pub facing: f32,
    pub flight_map_offset: [f32; 2],
    pub continent_id: u16,
    pub condition_id: u16,
    pub character_bit_number: u16,
    pub flags: u8,
    pub ui_texture_kit_id: i32,
    pub special_icon_condition_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xF44E2BF5)]
pub struct TaxiPath {
    #[parent]
    pub from_taxi_node: u16,
    pub to_taxi_node:   u16,
    #[id]
    pub id:             u32,
    pub cost:           u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xD38E8C01)]
pub struct TaxiPathNode {
    pub loc:                [f32; 3],
    #[parent]
    pub path_id:            u16,
    pub continent_id:       u16,
    pub node_index:         u8,
    #[id]
    pub id:                 u32,
    pub flags:              u8,
    pub delay:              u32,
    pub arrival_event_id:   u16,
    pub departure_event_id: u16,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x20B9177A)]
pub struct TotemCategory {
    pub id:                  u32,
    pub name:                String,
    pub totem_category_mask: i32,
    pub totem_category_type: u8,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x5409C5EA)]
pub struct Toy {
    pub source_text:      String,
    pub item_id:          i32,
    pub flags:            u8,
    pub source_type_enum: i8,
    #[id]
    pub id:               u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xB420EB18)]
pub struct TransmogHoliday {
    #[id]
    pub id: u32,
    pub required_transmog_holiday: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xBEDFD7D1)]
pub struct TransmogSet {
    pub name: String,
    #[parent]
    pub parent_transmog_set_id: u16,
    pub ui_order: i16,
    pub expansion_id: u8,
    #[id]
    pub id: u32,
    pub flags: i32,
    pub tracking_quest_id: u32,
    pub class_mask: i32,
    pub item_name_description_id: i32,
    pub transmog_set_group_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xCD072FE5)]
pub struct TransmogSetGroup {
    pub name: String,
    #[id]
    pub id:   u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x0E96B3A2)]
pub struct TransmogSetItem {
    #[id]
    pub id: u32,
    #[parent]
    pub transmog_set_id: u32,
    pub item_modified_appearance_id: u32,
    pub flags: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x99987ED)]
pub struct TransportAnimation {
    pub id:           u32,
    pub time_index:   u32,
    pub pos:          [f32; 3],
    pub sequence_id:  u8,
    #[parent]
    pub transport_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x72035AA9)]
pub struct TransportRotation {
    pub id:              u32,
    pub time_index:      u32,
    pub rot:             [f32; 4],
    #[parent]
    pub game_objects_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x626C94CD)]
pub struct UnitPowerBar {
    pub id:                  u32,
    pub name:                String,
    pub cost:                String,
    pub out_of_error:        String,
    pub tool_tip:            String,
    pub regeneration_peace:  f32,
    pub regeneration_combat: f32,
    pub file_data_id:        [i32; 6],
    pub color:               [i32; 6],
    pub start_inset:         f32,
    pub end_inset:           f32,
    pub start_power:         u16,
    pub flags:               u16,
    pub center_power:        u8,
    pub bar_type:            u8,
    pub min_power:           u32,
    pub max_power:           u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x1606C582)]
pub struct Vehicle {
    pub id: u32,
    pub flags: i32,
    pub turn_speed: f32,
    pub pitch_speed: f32,
    pub pitch_min: f32,
    pub pitch_max: f32,
    pub mouse_look_offset_pitch: f32,
    pub camera_fade_dist_scalar_min: f32,
    pub camera_fade_dist_scalar_max: f32,
    pub camera_pitch_offset: f32,
    pub facing_limit_right: f32,
    pub facing_limit_left: f32,
    pub camera_yaw_offset: f32,
    pub seat_id: [u16; 8],
    pub vehicle_ui_indicator_id: u16,
    pub power_display_id: [u16; 3],
    pub flags_b: u8,
    pub ui_locomotion_type: u8,
    pub missile_targeting_id: i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x242E0ECD)]
pub struct VehicleSeat {
    pub id: u32,
    pub flags: i32,
    pub flags_b: i32,
    pub flags_c: i32,
    pub attachment_offset: [f32; 3],
    pub enter_pre_delay: f32,
    pub enter_speed: f32,
    pub enter_gravity: f32,
    pub enter_min_duration: f32,
    pub enter_max_duration: f32,
    pub enter_min_arc_height: f32,
    pub enter_max_arc_height: f32,
    pub exit_pre_delay: f32,
    pub exit_speed: f32,
    pub exit_gravity: f32,
    pub exit_min_duration: f32,
    pub exit_max_duration: f32,
    pub exit_min_arc_height: f32,
    pub exit_max_arc_height: f32,
    pub passenger_yaw: f32,
    pub passenger_pitch: f32,
    pub passenger_roll: f32,
    pub vehicle_enter_anim_delay: f32,
    pub vehicle_exit_anim_delay: f32,
    pub camera_entering_delay: f32,
    pub camera_entering_duration: f32,
    pub camera_exiting_delay: f32,
    pub camera_exiting_duration: f32,
    pub camera_offset: [f32; 3],
    pub camera_pos_chase_rate: f32,
    pub camera_facing_chase_rate: f32,
    pub camera_entering_zoom: f32,
    pub camera_seat_zoom_min: f32,
    pub camera_seat_zoom_max: f32,
    pub ui_skin_file_data_id: i32,
    pub enter_anim_start: i16,
    pub enter_anim_loop: i16,
    pub ride_anim_start: i16,
    pub ride_anim_loop: i16,
    pub ride_upper_anim_start: i16,
    pub ride_upper_anim_loop: i16,
    pub exit_anim_start: i16,
    pub exit_anim_loop: i16,
    pub exit_anim_end: i16,
    pub vehicle_enter_anim: i16,
    pub vehicle_exit_anim: i16,
    pub vehicle_ride_anim_loop: i16,
    pub enter_anim_kit_id: i16,
    pub ride_anim_kit_id: i16,
    pub exit_anim_kit_id: i16,
    pub vehicle_enter_anim_kit_id: i16,
    pub vehicle_ride_anim_kit_id: i16,
    pub vehicle_exit_anim_kit_id: i16,
    pub camera_mode_id: i16,
    pub attachment_id: i8,
    pub passenger_attachment_id: i8,
    pub vehicle_enter_anim_bone: i8,
    pub vehicle_exit_anim_bone: i8,
    pub vehicle_ride_anim_loop_bone: i8,
    pub vehicle_ability_display: i8,
    pub enter_ui_sound_id: u32,
    pub exit_ui_sound_id: u32,
    // bool CanEnterOrExit() const
    // {
    //     return ((self.flags & VEHICLE_SEAT_FLAG_CAN_ENTER_OR_EXIT) != 0 ||
    //             //If it has anmation for enter/ride, means it can be entered/exited by logic
    //             (self.flags & (VEHICLE_SEAT_FLAG_HAS_LOWER_ANIM_FOR_ENTER | VEHICLE_SEAT_FLAG_HAS_LOWER_ANIM_FOR_RIDE)) != 0);
    // }
    // bool CanSwitchFromSeat() const { return (self.flags & VEHICLE_SEAT_FLAG_CAN_SWITCH) != 0; }
    // bool IsUsableByOverride() const
    // {
    //     return (self.flags & (VEHICLE_SEAT_FLAG_UNCONTROLLED | VEHICLE_SEAT_FLAG_UNK18)
    //                                 || (FlagsB & (VEHICLE_SEAT_FLAG_B_USABLE_FORCED | VEHICLE_SEAT_FLAG_B_USABLE_FORCED_2 |
    //                                     VEHICLE_SEAT_FLAG_B_USABLE_FORCED_3 | VEHICLE_SEAT_FLAG_B_USABLE_FORCED_4)));
    // }
    // bool IsEjectable() const { return (FlagsB & VEHICLE_SEAT_FLAG_B_EJECTABLE) != 0; }
}

#[derive(WDC1, Default, Debug)]
// #[derive(Default)]
#[layout_hash(0x4616C893)]
pub struct WMOAreaTable {
    pub area_name: String,
    ///  used in group WMO
    pub wmo_group_id: i32,
    pub ambience_id: u16,
    pub zone_music: u16,
    pub intro_sound: u16,
    pub area_table_id: u16,
    pub uw_intro_sound: u16,
    ///  used in adt file
    pub name_set_id: u8,
    pub sound_provider_pref: u8,
    pub sound_provider_pref_underwater: u8,
    pub flags: u8,
    pub uw_zone_music: u8,
    #[id]
    pub id: u32,
    pub inline_wmo_id: u32,
    ///  used in root WMO
    #[parent]
    pub wmo_id: u16,
}

// impl WDC1 for WMOAreaTable {
//     pub fn id_index() -> Option<usize> {
//         Some(12)
//     }

//     pub fn layout_hash() -> u32 {
//         0x4616C893
//     }

//     pub fn num_fields() -> usize {
//         14
//     }

//     pub fn produce_entry<W>(&mut self, fl: FileLoader<W>, record_number: usize, raw_record: &[u8]) -> std::io::Result<()>
//     where
//         W: WDC1,
//     {
//         self.area_name
//         self.wmo_group_id
//         self.ambience_id
//         self.zone_music
//         self.intro_sound
//         self.area_table_id
//         self.uw_intro_sound
//         self.name_set_id
//         self.sound_provider_pref
//         self.sound_provider_pref_underwater
//         self.flags
//         self.uw_zone_music
//         self.id = fl.record_get_id(raw_record, record_number)?;
//         self.inline_wmo_id
//         self.wmo_id
//     }
// }

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x2E9B9BFD)]
pub struct WorldEffect {
    pub id: u32,
    pub target_asset: i32,
    pub combat_condition_id: u16,
    pub target_type: u8,
    pub when_to_display: u8,
    pub quest_feedback_effect_id: u32,
    pub player_condition_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xC7E90019)]
pub struct WorldMapArea {
    pub area_name: String,
    pub loc_left: f32,
    pub loc_right: f32,
    pub loc_top: f32,
    pub loc_bottom: f32,
    pub flags: u32,
    pub map_id: i16,
    pub area_id: u16,
    pub display_map_id: i16,
    pub default_dungeon_floor: u16,
    pub parent_world_map_id: u16,
    pub level_range_min: u8,
    pub level_range_max: u8,
    pub bounty_set_id: u8,
    pub bounty_display_location: u8,
    #[id]
    pub id: u32,
    pub visibility_player_condition_id: u32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0xDC4B6AF3)]
pub struct WorldMapOverlay {
    pub texture_name:        String,
    #[id]
    pub id:                  u32,
    pub texture_width:       u16,
    pub texture_height:      u16,
    /// idx in WorldMapArea.dbc
    #[parent]
    pub map_area_id:         u32,
    pub offset_x:            i32,
    pub offset_y:            i32,
    pub hit_rect_top:        i32,
    pub hit_rect_left:       i32,
    pub hit_rect_bottom:     i32,
    pub hit_rect_right:      i32,
    pub player_condition_id: u32,
    pub flags:               u32,
    pub area_id:             [u32; 4],
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x99FB4B71)]
pub struct WorldMapTransforms {
    pub id:                 u32,
    pub region_min_max:     [f32; 6],
    pub region_offset:      [f32; 2],
    pub region_scale:       f32,
    #[parent]
    pub map_id:             u16,
    pub area_id:            u16,
    pub new_map_id:         u16,
    pub new_dungeon_map_id: u16,
    pub new_area_id:        u16,
    pub flags:              u8,
    pub priority:           i32,
}

#[derive(WDC1, Default, Debug)]
#[layout_hash(0x605EA8A6)]
pub struct WorldSafeLocs {
    pub id:        u32,
    pub area_name: String,
    pub loc:       [f32; 3],
    pub facing:    f32,
    pub continent: u16,
}

// struct SpellProcsPerMinuteModMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fhbh";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, , types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct AreaTriggerActionSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "h";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x5DA480BD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AreaTriggerBoxMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "f";
//         static const: u8,arraySizes[1] = { 3 };
//         static DB2Meta instance(-1, 1, 0x602CFDA6, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AreaTriggerCylinderMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fff";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x26D4052D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AreaTriggerSphereMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "f";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x9141AC7F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct Achievement_CategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(3, 4, 0xED226BC9, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct AdventureJournalMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sssssiihhhhhhbbbbbbbii";
//         static const: u8,arraySizes[22] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1 };
//         static DB2Meta instance(-1, 22, 0xB2FFA8DD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AdventureMapPOIMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssfibiiiiiiii";
//         static const: u8,arraySizes[13] = { 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 13, 0x0C288A82, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AlliedRaceMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiiii";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(1, 8, 0xB13ABE04, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AlliedRaceRacialAbilityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssbii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x9EBF9B09, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct AnimReplacementMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhih";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(3, 5, 0x2C8B0F35, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct AnimReplacementSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x3761247A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AnimationDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihhb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x03182786, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AreaFarClipOverrideMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iffii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(4, 5, 0xEB5921CC, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AreaPOIMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssifiihhhhbbiiii";
//         static const: u8,arraySizes[16] = { 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 16, 0xB161EE90, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AreaPOIStateMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbbih";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x673BDA80, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct AnimKitBoneSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbbbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xFE4B9B1F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AnimKitBoneSetAliasMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xEA8B67BC, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AnimKitConfigMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x8A70ED4C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AnimKitConfigBoneSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x3D9B3BA7, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct AnimKitPriorityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x5E93C107, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct AnimKitReplacementMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhih";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(3, 5, 0x0735DB83, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct AnimKitSegmentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiifihhhhhhbbbbbbi";
//         static const: u8,arraySizes[18] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 18, 0x08F09B89, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct BattlePetAbilityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssihbbi";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x0F29944D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BattlePetAbilityEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhhhbi";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 6, 1, 1 };
//         static DB2Meta instance(6, 7, 0x5D30EBC5, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct BattlePetAbilityStateMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x0E40A884, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct BattlePetAbilityTurnMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhbbbi";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(5, 6, 0xCB063F4F, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct BattlePetDisplayOverrideMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiib";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xDE5129EA, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BattlePetEffectPropertiesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shb";
//         static const: u8,arraySizes[3] = { 6, 1, 6 };
//         static DB2Meta instance(-1, 3, 0x56070751, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BattlePetNPCTeamMemberMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x4423F004, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BattlePetSpeciesXAbilityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbh";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x9EE27D6A, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct BattlePetStateMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x1797AB4A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BattlePetVisualMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihhhbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x097E0F6C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BeamEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iffihhhhhh";
//         static const: u8,arraySizes[10] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0x42C18603, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BoneWindModifierModelMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x577A0772, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct BoneWindModifiersMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ff";
//         static const: u8,arraySizes[2] = { 3, 1 };
//         static DB2Meta instance(-1, 2, 0xB4E7449E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct BountyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihhib";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xE76E716C, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct BountySetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hi";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x96B908A5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CameraEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xF6AB4622, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CameraEffectEntryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffffhbbbbbbh";
//         static const: u8,arraySizes[16] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 16, 0xC5105557, types, arraySizes, 15);
//         return &instance;
//     }
// };

// struct CameraModeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffffhbbbbb";
//         static const: u8,arraySizes[11] = { 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 11, 0xCDB6BC2F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CastableRaidBuffsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x5BDD4028, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct CelestialBodyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiifffffffhi";
//         static const: u8,arraySizes[15] = { 1, 1, 2, 1, 1, 2, 2, 2, 1, 2, 1, 3, 1, 1, 1 };
//         static DB2Meta instance(14, 15, 0xD09BE31C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct Cfg_CategoriesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x705B82C8, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct Cfg_ConfigsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fhbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xC618392F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CharBaseInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x9E9939B8, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CharComponentTextureLayoutsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x0F515E34, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CharComponentTextureSectionsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihhhhbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0xCE76000F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CharHairGeosetsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibbbbbbbbi";
//         static const: u8,arraySizes[10] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0x33EB32D2, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct CharShipmentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiihhbb";
//         static const: u8,arraySizes[9] = { 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 9, 0xE6D3C7C1, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct CharShipmentContainerMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssihhhhhhbbbbbbi";
//         static const: u8,arraySizes[16] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 16, 0x194896E3, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CharacterFaceBoneSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibbbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x1C634076, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct CharacterLoadoutMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "lbb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x87B51673, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CharacterLoadoutItemMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ih";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x3C3D40B9, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct CharacterServiceInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sssiiiiiiii";
//         static const: u8,arraySizes[11] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 11, 0xADE120EF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ChatProfanityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x328E1FE6, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ChrClassRaceSexMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbbiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x5E29DFA1, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ChrClassTitleMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xC155DB2C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ChrClassUIDisplayMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x59A95A73, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ChrClassVillainMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xA6AC18CD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ChrCustomizationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siiiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 3, 1 };
//         static DB2Meta instance(-1, 6, 0x71833CE5, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct ChrUpgradeBucketMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hib";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(1, 3, 0xACF64A80, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ChrUpgradeBucketSpellMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ih";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xDF939031, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct ChrUpgradeTierMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(3, 4, 0x2C87937D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CloakDampeningMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffffff";
//         static const: u8,arraySizes[7] = { 5, 5, 2, 2, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0xB2DF7F2A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CombatConditionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhhhbbbbbb";
//         static const: u8,arraySizes[11] = { 1, 1, 1, 2, 2, 2, 2, 1, 2, 2, 1 };
//         static DB2Meta instance(-1, 11, 0x28D253C6, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CommentatorStartLocationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fi";
//         static const: u8,arraySizes[2] = { 3, 1 };
//         static DB2Meta instance(-1, 2, 0xEFD540EF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CommentatorTrackedCooldownMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbih";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x84985168, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct ComponentModelFileDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x25BB55A7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ComponentTextureFileDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x50C58D4F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ConfigurationWarningMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "si";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x0B350390, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ContributionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 4, 1 };
//         static DB2Meta instance(2, 6, 0x8EDF6090, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct CreatureMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssssiiifbbbb";
//         static const: u8,arraySizes[12] = { 1, 1, 1, 1, 3, 1, 4, 4, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 12, 0xCFB508A9, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CreatureDifficultyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbbbi";
//         static const: u8,arraySizes[6] = { 7, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x4291EEC6, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct CreatureDispXUiCameraMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ih";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x6E0E7C15, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CreatureDisplayInfoCondMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "liiibbiiiiiiiii";
//         static const: u8,arraySizes[15] = { 1, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1 };
//         static DB2Meta instance(-1, 15, 0x26CD44AB, types, arraySizes, 14);
//         return &instance;
//     }
// };

// struct CreatureDisplayInfoEvtMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iibi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x3FEF69BB, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct CreatureDisplayInfoTrnMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifiiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x8E687740, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct CreatureImmunitiesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibbbbbiii";
//         static const: u8,arraySizes[9] = { 2, 1, 1, 1, 1, 1, 1, 8, 16 };
//         static DB2Meta instance(-1, 9, 0x2D20050B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CreatureMovementInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "f";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x39F710E3, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CreatureSoundDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffbiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiiii";
//         static const: u8,arraySizes[37] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 5, 4 };
//         static DB2Meta instance(-1, 37, 0x7C3C39B9, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct CreatureXContributionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(0, 3, 0x3448DF58, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct CriteriaTreeXEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hi";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x929D9B0C, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct CurrencyCategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xC3735D76, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DeathThudLookupsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xD469085C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DecalPropertiesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiffffffffbbiiiii";
//         static const: u8,arraySizes[17] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(0, 17, 0xDD48C72A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DeclinedWordMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "si";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(1, 2, 0x3FF5EC3E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DeclinedWordCasesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x821A20A9, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct DeviceBlacklistMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xD956413D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DeviceDefaultSettingsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x90CFEC8C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DissolveEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffffbbiiii";
//         static const: u8,arraySizes[14] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 14, 0x566413E7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DriverBlacklistMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iihbbbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x1466ACAD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct DungeonMapMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffhhbbbi";
//         static const: u8,arraySizes[8] = { 2, 2, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(7, 8, 0xB5A245F4, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct DungeonMapChunkMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fihhh";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x7927A3A7, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct EdgeGlowEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffffffbii";
//         static const: u8,arraySizes[13] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 13, 0x083BF2C4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct EmotesTextDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x0E19BCF1, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct EnvironmentalDamageMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xC4552C14, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ExhaustionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiffffi";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(7, 8, 0xE6E16045, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct FactionGroupMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssibii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(2, 6, 0x7A7F9A51, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct FootprintTexturesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xFD6FF285, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct FootstepTerrainLookupMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x454895AE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct FriendshipRepReactionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x9C412E5B, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct FriendshipReputationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(3, 4, 0x406EE0AB, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct FullScreenEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffffffffffffffffffffffiiii";
//         static const: u8,arraySizes[27] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 27, 0x5CBF1D1B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GMSurveyAnswersMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x422747F6, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct GMSurveyCurrentSurveyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x617205BF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GMSurveyQuestionsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x9D852FDC, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GMSurveySurveysMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 15 };
//         static DB2Meta instance(-1, 1, 0x17FEF812, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GameObjectArtKitMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 3 };
//         static DB2Meta instance(-1, 2, 0x6F65BC41, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GameObjectDiffAnimMapMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x89A617CF, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct GameObjectDisplayInfoXSoundKitMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x4BBA66F2, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct GameTipsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shhb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x547E3F0F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrAbilityCategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrAbilityEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffihbbbbbbi";
//         static const: u8,arraySizes[12] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(11, 12, 0xE6A6CB99, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct GarrBuildingDoodadSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbbbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x2A861C7F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrClassSpecPlayerCondMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sibiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x06936172, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrEncounterMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siffiii";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(5, 7, 0x63EF121A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrEncounterSetXEncounterMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(0, 3, 0x3AA64423, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct GarrEncounterXMechanicMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x97080E17, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct GarrFollItemSetMemberMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbh";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xCA1C4CBF, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct GarrFollSupportSpellMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iibi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xB7DBA2D1, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct GarrFollowerLevelXPMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x1ED485E2, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrFollowerQualityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbbbbi";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0xAFF4CF7E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrFollowerSetXFollowerMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xDB0E0A17, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct GarrFollowerTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbbbbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0xD676FBC0, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrFollowerUICreatureMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifbbbh";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x7E275E96, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct GarrItemLevelUpgradeDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(0, 5, 0x069F44E5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrMechanicMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xAB49DA61, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrMechanicSetXMechanicMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(1, 3, 0x59514F7B, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct GarrMechanicTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssibi";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(4, 5, 0x6FEA569F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrMissionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sssiiffhhhbbbbbbbbbiiiiiiiiii";
//         static const: u8,arraySizes[29] = { 1, 1, 1, 1, 1, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(19, 29, 0xDDD70490, types, arraySizes, 28);
//         return &instance;
//     }
// };

// struct GarrMissionTextureMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fh";
//         static const: u8,arraySizes[2] = { 2, 1 };
//         static DB2Meta instance(-1, 2, 0x3071301C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrMissionTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xA289655E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrMissionXEncounterMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "biiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(1, 5, 0xBCB016C6, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct GarrMissionXFollowerMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x1EBABA29, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct GarrMssnBonusAbilityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fihbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x35F5AE92, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrPlotUICategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xA94645EE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrSpecializationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssifbbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 2, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x797A0F2F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrStringMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xE1C08C0C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrTalentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiibbbiiiiiiiiiiiii";
//         static const: u8,arraySizes[20] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(7, 20, 0x53D5FD16, types, arraySizes, 8);
//         return &instance;
//     }
// };

// struct GarrTalentTreeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x676CBC04, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 2 };
//         static DB2Meta instance(-1, 5, 0x7C52F3B7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrUiAnimClassInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbbiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0xDBF4633D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GarrUiAnimRaceInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffffffffb";
//         static const: u8,arraySizes[13] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 13, 0x44B9C1DE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GlobalStringsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x2CA3EA1E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GlyphExclusiveCategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xFE598FCD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GroundEffectDoodadMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x0376B2D6, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GroundEffectTextureMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbi";
//         static const: u8,arraySizes[4] = { 4, 4, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x84549F0A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GroupFinderActivityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sshhhbbbbbbbbb";
//         static const: u8,arraySizes[14] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 14, 0x3EF2F3BD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GroupFinderActivityGrpMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xC9458196, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct GroupFinderCategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x9213552F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct HelmetAnimScalingMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xB9EC1058, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct HelmetGeosetVisDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 9 };
//         static DB2Meta instance(-1, 1, 0x3B38D999, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct HighlightColorMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiibb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x5FADC5D3, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct HolidayDescriptionsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x92A95550, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct HolidayNamesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct HotfixMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x3747930B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct InvasionClientDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sfiiiiiiii";
//         static const: u8,arraySizes[10] = { 1, 2, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(2, 10, 0x4C93379F, types, arraySizes, 9);
//         return &instance;
//     }
// };

// struct ItemAppearanceXUiCameraMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x67747E15, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemContextPickerEntryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbiiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x4A6DF90B, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct ItemDisplayInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiiiiiiiiiii";
//         static const: u8,arraySizes[15] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 4, 4, 2 };
//         static DB2Meta instance(-1, 15, 0x99606089, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemDisplayInfoMaterialResMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xDEE4ED7B, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ItemDisplayXUiCameraMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ih";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xE57737B2, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemGroupSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 4 };
//         static DB2Meta instance(-1, 1, 0xDC2EE466, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemModifiedAppearanceExtraMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iibbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x77212236, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemNameDescriptionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "si";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x16760BD4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemPetFoodMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xE4923C1F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemRangedDisplayInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x687A28D1, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemSubClassMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sshbbbbbbb";
//         static const: u8,arraySizes[10] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0xDAD92A67, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct ItemSubClassMaskMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sib";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xFC1DA850, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ItemVisualsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 5 };
//         static DB2Meta instance(-1, 1, 0x485EA782, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct JournalEncounterMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssfhhhhbbii";
//         static const: u8,arraySizes[11] = { 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 11, 0x2935A0FD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct JournalEncounterCreatureMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiiihbi";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(7, 8, 0x22C79A42, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct JournalEncounterItemMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbbbi";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(5, 6, 0x39230FF9, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct JournalEncounterSectionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiiiihhhhhhbbb";
//         static const: u8,arraySizes[15] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 15, 0x13E56B12, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct JournalEncounterXDifficultyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x321FD542, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct JournalEncounterXMapLocMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbiiii";
//         static const: u8,arraySizes[6] = { 2, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x430540E4, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct JournalInstanceMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiiiihhbbi";
//         static const: u8,arraySizes[11] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(10, 11, 0x1691CC3D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct JournalItemXDifficultyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x60D9CA15, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct JournalSectionXDifficultyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x243822A7, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct JournalTierMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x8046B23F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct JournalTierXInstanceMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x9C4F4D2A, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct KeystoneAffixMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x1BCB46AA, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LFGDungeonExpansionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbbiih";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0xB41DEA61, types, arraySizes, 6);
//         return &instance;
//     }
// };

// struct LFGDungeonGroupMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x724D58E7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LFGRoleRequirementMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bih";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x7EB8A359, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct LanguageWordsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xC15912BD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LanguagesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "si";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(1, 2, 0x6FA5D0C4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LfgDungeonsGroupingMapMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x8CB35C50, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct LightDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiiiiiiiiiiiiiifffffffffiiiiiihh";
//         static const: u8,arraySizes[35] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 35, 0x2D2BA7FA, types, arraySizes, 34);
//         return &instance;
//     }
// };

// struct LightParamsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffhbbbi";
//         static const: u8,arraySizes[11] = { 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(10, 11, 0xF67DE2AF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LightSkyboxMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siib";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x8817C02C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LiquidMaterialMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x62BE0340, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LiquidObjectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffhbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xACC168A6, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LoadingScreenTaxiSplinesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffhhb";
//         static const: u8,arraySizes[5] = { 10, 10, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x4D6292C3, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LoadingScreensMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x99C0EB78, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LocaleMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x592AE13B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LocationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ff";
//         static const: u8,arraySizes[2] = { 3, 3 };
//         static DB2Meta instance(-1, 2, 0xBBC1BE7A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LockTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssssi";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(4, 5, 0xCD5E1D2F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct LookAtControllerMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffhhhhbbbbbiiiii";
//         static const: u8,arraySizes[18] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 18, 0x543C0D56, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManagedWorldStateMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiiiiii";
//         static const: u8,arraySizes[10] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(9, 10, 0xBA06FC33, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManagedWorldStateBuffMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x6D201DC7, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct ManagedWorldStateInputMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x0FC1A9B0, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManifestInterfaceActionIconMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(0, 1, 0x6A529F37, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManifestInterfaceDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ss";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x9E5F4C99, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManifestInterfaceItemIconMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(0, 1, 0x6A529F37, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManifestInterfaceTOCDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x6F7D397D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ManifestMP3Meta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(0, 1, 0x6A529F37, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct MapCelestialBodyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hih";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xBDE1C11C, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct MapChallengeModeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihhb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 3, 1 };
//         static DB2Meta instance(1, 5, 0xC5261662, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct MapDifficultyXConditionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x5F5D7102, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct MapLoadingScreenMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffiii";
//         static const: u8,arraySizes[5] = { 2, 2, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xBBE57FE4, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct MarketingPromotionsXLocaleMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siiiibb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x80362F57, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct MaterialMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "biii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x0BC8C134, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct MinorTalentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xAAEF0DF8, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct MissileTargetingMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffffffffiii";
//         static const: u8,arraySizes[12] = { 1, 1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 2 };
//         static DB2Meta instance(-1, 12, 0x2305491E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ModelAnimCloakDampeningMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x839B4263, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ModelFileDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(1, 3, 0xA395EB50, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ModelRibbonQualityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bi";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x38F764D9, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct MovieFileDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "h";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xAA16D59F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct MovieVariationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iih";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x3BFD250E, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct NPCModelItemSlotDisplayInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x11D16204, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct NPCSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 4 };
//         static DB2Meta instance(-1, 1, 0x672E1A6B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ObjectEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fhbbbbii";
//         static const: u8,arraySizes[8] = { 3, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 8, 0x6A0CF743, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ObjectEffectModifierMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbbb";
//         static const: u8,arraySizes[4] = { 4, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xA482B053, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ObjectEffectPackageElemMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x8CF043E5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct OutlineEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fiiiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x466B2BC4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PVPBracketTypesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bi";
//         static const: u8,arraySizes[2] = { 1, 4 };
//         static DB2Meta instance(-1, 2, 0x7C55E5BB, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PageTextMaterialMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PaperDollItemFrameMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x66B0597E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ParagonReputationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xD7712F98, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ParticleColorMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 3, 3, 3 };
//         static DB2Meta instance(-1, 3, 0x1576D1E1, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PathMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbbbbbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x5017579F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PathNodeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iihh";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(0, 4, 0x76615830, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PathNodePropertyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhbii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(3, 5, 0x92C03009, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PathPropertyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(3, 4, 0x3D29C266, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PhaseShiftZoneSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhhhbbbbiiii";
//         static const: u8,arraySizes[13] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 13, 0x85ACB830, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PositionerMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fhbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xE830F1B1, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PositionerStateMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbiiiiii";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 8, 0x6C975DF4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PositionerStateEntryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffhhhhbbbbi";
//         static const: u8,arraySizes[11] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 11, 0x667ED965, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct PvpScalingEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x52121A41, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct PvpScalingEffectTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct QuestFeedbackEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbbbb";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x89D55A27, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct QuestInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x4F45F445, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct QuestLineMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x8046B23F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct QuestLineXQuestMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x8FA4A9C7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct QuestObjectiveMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siibbbbh";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 8, 0xDD995180, types, arraySizes, 7);
//         return &instance;
//     }
// };

// struct QuestPOIBlobMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihhbbiii";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(0, 8, 0xEC15976E, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct QuestPOIPointMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihhi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(0, 4, 0x8CF2B119, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct QuestV2CliTaskMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "lssihhhhhhbbbbbbbbbbiiii";
//         static const: u8,arraySizes[24] = { 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(20, 24, 0x3F026A14, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct QuestXGroupActivityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x06CC45D3, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct RelicSlotTierRequirementMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x129FCC09, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct RelicTalentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x7A5963FD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ResearchBranchMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihbii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x58A3876E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ResearchFieldMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(2, 3, 0x85868B9F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ResearchProjectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssihbbiii";
//         static const: u8,arraySizes[9] = { 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(6, 9, 0xB1CAB80B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ResearchSiteMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x25F7DCC7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ResistancesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xA3EAE5AE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct RibbonQualityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffbi";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xC75DAEA8, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SDReplacementModelMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xE1F906C2, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ScenarioEventEntryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x02E80455, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SceneScriptPackageMemberMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x787A715F, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct ScheduledIntervalMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x5DD2FF46, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ScheduledWorldStateMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiiii";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 8, 0xFCB13A6A, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct ScheduledWorldStateGroupMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x21F6EE03, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ScheduledWorldStateXUniqCatMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(0, 3, 0x7EFF57FD, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ScreenEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihhhhbbbiii";
//         static const: u8,arraySizes[12] = { 1, 4, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 12, 0x4D5B91C5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ScreenLocationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SeamlessSiteMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xBFE7B9D3, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct ServerMessagesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x1C7A1347, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ShadowyEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iifffffffbbii";
//         static const: u8,arraySizes[13] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 13, 0xE909BB18, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundAmbienceMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "biii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 2 };
//         static DB2Meta instance(-1, 4, 0xB073D4B5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundAmbienceFlavorMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iih";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x2C58D929, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct SoundBusMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbbbbbih";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(6, 8, 0xB2ACDE2A, types, arraySizes, 7);
//         return &instance;
//     }
// };

// struct SoundBusOverrideMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifbbbii";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(0, 7, 0x6D887F48, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct SoundEmitterPillPointsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fh";
//         static const: u8,arraySizes[2] = { 3, 1 };
//         static DB2Meta instance(-1, 2, 0x41FCF15B, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct SoundEmittersMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sffhhbbbiiih";
//         static const: u8,arraySizes[12] = { 1, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(8, 12, 0x55A3B17E, types, arraySizes, 11);
//         return &instance;
//     }
// };

// struct SoundEnvelopeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iihhhbi";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x5B78031C, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct SoundFilterMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundFilterElemMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbb";
//         static const: u8,arraySizes[3] = { 9, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xE17AC589, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct SoundKitAdvancedMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifffffiifbiiiiiiiiiibffffbhffiiibbiiiiii";
//         static const: u8,arraySizes[40] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(0, 40, 0x73F6F023, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundKitChildMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x2827A3B5, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct SoundKitEntryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iibf";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x6ED6E26F, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct SoundKitFallbackMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xB1A5106F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundKitNameMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x96663ABF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundOverrideMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xFB7643F6, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SoundProviderPreferencesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sfffffffffffffffhhhhhbb";
//         static const: u8,arraySizes[23] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 23, 0x85F218A4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SourceInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x7C214135, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct SpamMessagesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x0D4BA7E7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellActionBarPrefMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ih";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x1EF80B2B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellActivationOverlayMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiifibbi";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 4, 1, 1, 1 };
//         static DB2Meta instance(-1, 8, 0x23568FC7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellAuraVisXChrSpecMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xA65B6A4A, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct SpellAuraVisibilityMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbii";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(2, 4, 0xA549F79C, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct SpellChainEffectsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffiiffffffffffffffffffffffffffffffffffiffffhhhhbbbbbbbbbbii";
//         static const: u8,arraySizes[60] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 1, 1, 1, 1, 1, 1, 1, 11, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3 };
//         static DB2Meta instance(-1, 60, 0x4E8FF369, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellDescriptionVariablesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xA8EDE75B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellDispelTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xE9DDA799, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellEffectEmissionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffhb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xC6E61A9B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellFlyoutMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "lssbii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x437671BD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellFlyoutItemMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xF86ADE09, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct SpellKeyboundOverrideMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sib";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x6ECA16FC, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellLabelMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x68E44736, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct SpellMechanicMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xF2075D8C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellMissileMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifffffffffffffb";
//         static const: u8,arraySizes[15] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 15, 0x1D35645E, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellMissileMotionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x6B78A45B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellProceduralEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbi";
//         static const: u8,arraySizes[3] = { 4, 1, 1 };
//         static DB2Meta instance(2, 3, 0x3E47F4EF, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellReagentsCurrencyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x90A5E5D2, types, arraySizes, 0);
//         return &instance;
//     }
// };

// struct SpellSpecialUnitEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hi";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x76989615, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffihbbiiiihiii";
//         static const: u8,arraySizes[14] = { 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 14, 0x1C1301D2, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualAnimMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x0ABD7A19, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualColorEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fifhhhhhbbi";
//         static const: u8,arraySizes[11] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 11, 0x7E5B2E66, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualEffectNameMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffiiibiii";
//         static const: u8,arraySizes[13] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 13, 0xB930A934, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualEventMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiiiiii";
//         static const: u8,arraySizes[9] = { 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 9, 0xAE75BC3C, types, arraySizes, 8);
//         return &instance;
//     }
// };

// struct SpellVisualKitMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifihh";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xDC04F488, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualKitAreaModelMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifffhb";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0xBE76E593, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct SpellVisualKitEffectMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xB78084B7, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct SpellVisualKitModelAttachMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffihbbhffffffffhhhhifi";
//         static const: u8,arraySizes[22] = { 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(2, 22, 0xBCE18649, types, arraySizes, 21);
//         return &instance;
//     }
// };

// struct SpellVisualMissileMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiffhhhhhbbiiih";
//         static const: u8,arraySizes[16] = { 1, 1, 1, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(12, 16, 0x00BA67A5, types, arraySizes, 15);
//         return &instance;
//     }
// };

// struct SpellXDescriptionVariablesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ii";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xB08E6876, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct StartupFilesMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x51FEBBB5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct Startup_StringsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ss";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xF8CDDEE7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct StationeryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bii";
//         static const: u8,arraySizes[3] = { 1, 1, 2 };
//         static DB2Meta instance(-1, 3, 0x20F6BABD, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TactKeyLookupMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 8 };
//         static DB2Meta instance(-1, 1, 0x3C1AC92A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TerrainMaterialMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x19D9496F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TerrainTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shhbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x4FE20345, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TerrainTypeSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xE4923C1F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TextureBlendSetMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifffffbbbb";
//         static const: u8,arraySizes[10] = { 3, 3, 3, 3, 3, 4, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0xA2323E0C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TextureFileDataMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iib";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(0, 3, 0xE0790D00, types, arraySizes, 1);
//         return &instance;
//     }
// };

// struct TradeSkillCategoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shhhb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x5D3ADD4D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TradeSkillItemMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xFDE283DA, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TransformMatrixMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffff";
//         static const: u8,arraySizes[5] = { 3, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xB6A2C431, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TransportPhysicsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffffff";
//         static const: u8,arraySizes[10] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0x2C1FB208, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct TrophyMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xE16151C5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UIExpansionDisplayInfoMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x73DFDEC5, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UIExpansionDisplayInfoIconMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x331022F2, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiCamFbackTransmogChrRaceMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x9FB4CC78, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiCamFbackTransmogWeaponMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x020890B7, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiCameraMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sfffhbbbi";
//         static const: u8,arraySizes[9] = { 1, 3, 3, 3, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 9, 0xCA6C98D4, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiCameraTypeMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sii";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x644732AE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiMapPOIMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifiihhi";
//         static const: u8,arraySizes[7] = { 1, 3, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(6, 7, 0x559E1F11, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiModelSceneMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bb";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xA7D62B8A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiModelSceneActorMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sfffffbiii";
//         static const: u8,arraySizes[10] = { 1, 3, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(7, 10, 0x679AC95F, types, arraySizes, 9);
//         return &instance;
//     }
// };

// struct UiModelSceneActorDisplayMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x6137F4BE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiModelSceneCameraMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sfffffffffffbbii";
//         static const: u8,arraySizes[16] = { 1, 3, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(14, 16, 0xC58AA5EC, types, arraySizes, 15);
//         return &instance;
//     }
// };

// struct UiTextureAtlasMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x9879592A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiTextureAtlasMemberMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sihhhhhb";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(1, 8, 0x81E2055F, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UiTextureKitMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x2C7E0372, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UnitBloodMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iiiiii";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0x4689A9A0, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UnitBloodLevelsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "b";
//         static const: u8,arraySizes[1] = { 3 };
//         static DB2Meta instance(-1, 1, 0x31A6BD58, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UnitConditionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ibbb";
//         static const: u8,arraySizes[4] = { 8, 1, 8, 8 };
//         static DB2Meta instance(-1, 4, 0x62802D9C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct UnitTestMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssiii";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(2, 5, 0x63B4527B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct VehicleUIIndSeatMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffbh";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x5F688502, types, arraySizes, 3);
//         return &instance;
//     }
// };

// struct VehicleUIIndicatorMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "i";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0x68486100, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct VignetteMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sffiiii";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x52E3B381, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct VirtualAttachmentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0xEC767C57, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct VirtualAttachmentCustomizationMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xC354C931, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct VocalUISoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 2 };
//         static DB2Meta instance(-1, 4, 0xED48CFA9, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WMOMinimapTextureMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihbbh";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0x8F4AE3C0, types, arraySizes, 4);
//         return &instance;
//     }
// };

// struct WbAccessControlListMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbbb";
//         static const: u8,arraySizes[5] = { 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 5, 0xBE044710, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WbCertWhitelistMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbbb";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x01D13030, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WeaponImpactSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbbiiii";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 11, 11, 11, 11 };
//         static DB2Meta instance(-1, 7, 0x774C043A, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WeaponSwingSounds2Meta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "bbi";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xD45347C3, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WeaponTrailMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ifffiffff";
//         static const: u8,arraySizes[9] = { 1, 1, 1, 1, 3, 3, 3, 3, 3 };
//         static DB2Meta instance(-1, 9, 0x49754C60, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WeaponTrailModelDefMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x7DE7C508, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct WeaponTrailParamMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffffbbbbh";
//         static const: u8,arraySizes[10] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0x9B0F7200, types, arraySizes, 9);
//         return &instance;
//     }
// };

// struct WeatherMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffffffhbbbii";
//         static const: u8,arraySizes[14] = { 2, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 14, 0x7C160B07, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WindSettingsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fffffffffb";
//         static const: u8,arraySizes[10] = { 1, 3, 1, 1, 3, 1, 3, 1, 1, 1 };
//         static DB2Meta instance(-1, 10, 0x5308550C, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WorldBossLockoutMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sh";
//         static const: u8,arraySizes[2] = { 1, 1 };
//         static DB2Meta instance(-1, 2, 0x4D7103A0, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WorldChunkSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hbbbbb";
//         static const: u8,arraySizes[6] = { 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 6, 0xD06AA126, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WorldElapsedTimerMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "sbb";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x6C026FDE, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WorldMapContinentMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ffffhhbbbbb";
//         static const: u8,arraySizes[11] = { 2, 1, 2, 2, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 11, 0x8F75E077, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WorldStateExpressionMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "s";
//         static const: u8,arraySizes[1] = { 1 };
//         static DB2Meta instance(-1, 1, 0xA69C9812, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct WorldStateUIMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ssssshhhhhhbbbiii";
//         static const: u8,arraySizes[17] = { 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(14, 17, 0x70808977, types, arraySizes, 5);
//         return &instance;
//     }
// };

// struct WorldStateZoneSoundsMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "ihhhhhhb";
//         static const: u8,arraySizes[8] = { 1, 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 8, 0xB9572D3D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct World_PVP_AreaMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "hhhhhbb";
//         static const: u8,arraySizes[7] = { 1, 1, 1, 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 7, 0x6FBBF76B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ZoneIntroMusicTableMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shbi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0x1F8417ED, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ZoneLightMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "shh";
//         static const: u8,arraySizes[3] = { 1, 1, 1 };
//         static DB2Meta instance(-1, 3, 0x3C11F38B, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ZoneLightPointMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "fbh";
//         static const: u8,arraySizes[3] = { 2, 1, 1 };
//         static DB2Meta instance(-1, 3, 0xEF93DC50, types, arraySizes, 2);
//         return &instance;
//     }
// };

// struct ZoneMusicMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "siii";
//         static const: u8,arraySizes[4] = { 1, 2, 2, 2 };
//         static DB2Meta instance(-1, 4, 0x9E2B332D, types, arraySizes, -1);
//         return &instance;
//     }
// };

// struct ZoneStoryMeta
// {
//     static DB2Meta const* Instance()
//     {
//         static char const* types = "iibi";
//         static const: u8,arraySizes[4] = { 1, 1, 1, 1 };
//         static DB2Meta instance(-1, 4, 0xEE16D6F3, types, arraySizes, 3);
//         return &instance;
//     }
// };
