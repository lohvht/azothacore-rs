# Legion engine backlog (data waiting for engine support)

World data imported from AshamaneCore (ADB_735.10) and LegionCore is limited to what the
TrinityCore 7.3.5/26972 engine (`trinitycore-dockerfile/TrinityCore/`) can represent, because
Azothacore ports that engine. The items below are real 7.3.5 content that was left out or kept
inert because TC 7.3.5 has no support for it. Revisit them when the matching Rust system is built.
The counts are from the 2026-10-24 import. The converters and reports live in
`.agents/plans/legion-{adb-rebase,lc-fill}/` (gitignored, local only).

## Reference implementations

- **Base: TC 7.3.5.** Keep its enum numbering (SmartAI events/actions/targets, condition types and
  source types, gossip option types, `flags_extra`) so imported TC-format data stays valid.
- **Missing feature:** add it as an extension of the TC 7.3.5 enum, in a reserved range after the
  TC 7.3.5 values. Look for references in this order:
  1. **TC `master`** (`TrinityCore-master/`). It is maintained and has since implemented much of
     what the Legion forks invented, e.g. `SMART_ACTION_CREATE_CONVERSATION`,
     `BECOME_PERSONAL_CLONE_FOR_PLAYER`, `CONDITION_SCENARIO_STEP`, `CONDITION_PLAYER_CONDITION`,
     and currency loot via `ItemType`. Take names and semantics from it, but check that the
     behaviour matches 7.3.5. Master also **renumbered** condition types (e.g. `OBJECT_ENTRY_GUID`
     is 31 in 7.3.5 and 51 in master), so its data must be mapped by name too.
  2. **AshamaneCore.** Same TC 7.3.5 base, so its additions port cleanly. It adds SmartAI actions
     200–215 and the target `INVOKER_SUMMON` (100).
  3. **LegionCore.** It has the most complete Legion behaviour but a divergent codebase (UWoW
     lineage), so use it as a behavioural reference only.
- When an extension lands, rerun the converter with the new mapping. The skipped rows are
  reproducible from the reports.

## 1. SmartAI

- Imported LegionCore blocks use only TC 7.3.5 values (mapped by name), so they are representable.
- **ADB base, inert today:** 5 rows use AshamaneCore-only actions (`START_CONVERSATION` ×4,
  `CAST_SPELL_OFFSET` ×1).
- **LegionCore, not imported:** 1625 blocks use LegionCore-only features. The most used:

  | feature | blocks |
  |---|---|
  | `UPDATE_ACHIEVEMENT_CRITERIA` | 515 |
  | `SUMMON_CONVERSATION` (master: `CREATE_CONVERSATION`) | 504 |
  | `SUMMON_ADD_PLR_PERSONNAL_VISIBILE` (personal summons; master: personal clones) | 355 |
  | LegionCore eventobjects (source type + event) | 354 |
  | `CIRCLE_PATH` (AshamaneCore has it) | 105 |
  | `MOVE_FORWARD` | 69 |
  | targets `HOSTILE_RANDOM_PLAYER` / `_NOT_TOP_PLAYER` | 65 |
  | `CHECK_DIST_TO_HOME` | 44 |
  | `SET_INGAME_PHASE_MASK` (LegionCore phase system: drop) | 39 |
  | scenario / areatrigger / boss / LFG / currency / quest-complete actions | < 25 each |

- **LegionCore guid scripts:** 546 blocks were skipped because their spawn wasn't imported (it was
  matched to an existing ADB spawn). They could be attached by remapping the GUIDs through the
  position match.

## 2. Conditions and gossip

- **LegionCore-only condition types, not imported:**
  - `OBJECT_ENTRY` 3520 rows. It is probably TC's `OBJECT_ENTRY_GUID` under its old name; verify
    the parameters and map it.
  - `QUEST_OBJECTIVE_DONE` 743 (master: `QUEST_OBJECTIVE_PROGRESS`).
  - `SCENARION_STEP` 555 (master: `SCENARIO_STEP`).
  - `SPEC_ID` 258, `ARTIFACT_LEVEL` 72, `CURRENCY` 65, world quest, scene, artifact power, class
    hall: < 50 each.
- **LegionCore-only source types:**
  - `PHASE_DEFINITION` 5233: LegionCore's phase system. Not needed; TC uses `phase_area` + conditions.
  - `QUEST_SHOW_MARK` 369, `LOOT_ITEM` 306, player choice responses, vignettes, seamless teleport.
- **Gossip option types TC 7.3.5 lacks** (39 options): garrison shipment / tradeskill, artifact
  respec, allied race details, mailbox, player choice, scenario, class hall upgrade.

## 3. Phasing

- LegionCore spawns that list several phases without a matching 7.3.5 `PhaseGroup` are imported
  with **server-side PhaseGroups** (IDs from 10001) in `hotfixes.phase_x_phase_group`. TC 7.3.5
  loads that table into `sPhaseXPhaseGroupStore`, and ADB already uses the trick (group 10000).
  The Rust DB2 loader must merge hotfix rows into the store for these spawns to show.
- **Not representable:** 346 creatures and 228 gameobjects use only LegionCore's old `phaseMask`
  (Cataclysm-style: Gilneas, Lost Isles, some Draenor). LegionCore's `phase_definitions` maps only
  3 of them to phase IDs. ADB/TDB carry their own phasing for those zones, so this is low priority.

## 4. Scripts (C++ AI, spells)

- These follow the source priority in the root `CLAUDE.md`: port from AshamaneCore first.
- **LegionCore scripts we have no script for:**
  - 2527 creature templates, of which 380 now have LegionCore SmartAI. The most affected maps are
    Isle of Thunder 1064, Tanaan intro 1265, Vault of the Wardens 1468 (Demon Hunter start),
    Seat of the Triumvirate 1753, and the Lost Isles 648.
  - 192 gameobject templates and 1995 spells (`spell_script_names`).
- **ADB references we haven't found an AshamaneCore script for** (approximate, from a name grep):
  182 creature and 263 spell script names. Check them when porting.

## 5. Loot

- **Currency loot.** TC 7.3.5 has none; TC master supports it through `ItemType` in `*_loot_template`.
  - LegionCore: 4834 rows over 46 currencies (negative `item`), e.g. Order Resources 1220,
    Lesser Charm 738, Ancient Mana 1155, Apexis 823, Garrison Resources 824, Curious Coin 1275,
    Nethershards 1226, Timeless Coin 777.
  - ADB: 5 rows. Both sets were dropped at import.
  - Adopt master's `ItemType` column when loot is implemented, then re-import.

## 6. Hotfixes and LegionCore-only tables

- **LegionCore hotfix rows absent from ours:**
  - `broadcast_text` 1971. Mostly 7.3.x Blizzard IDs that are probably in the client DB2 already;
    88 are custom IDs.
  - `item` 1273 (1132 exist in the client), `spell` 260 (259 are LegionCore custom server-side
    spells used by its scripts), `criteria` 137.
  - Import only when a ported script needs them.
- **Not imported:** LegionCore's own areatrigger, conversation, scenario, world-quest, battlepay
  and eventobject tables. AshamaneCore has equivalents for some (`areatrigger`, `world_quest*`,
  `zone_scripts`, kept as ADB-only tables).
- **ADB:** 36 server-side areatrigger teleports with negative IDs were dropped.
