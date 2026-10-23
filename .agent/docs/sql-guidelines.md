# SQL guidelines

## Adding SQL Updates

TODO: Lift from azerothcore's guidelines => but eventually want to transition our raw `.sql` files into proper sqlx migrations instead.

TODO: Make sure that the sqlx migrations are somewhat comparable to the existing azerothcore / trinitycore migration solutions

3. Conventions (linted): every `INSERT` preceded by a matching `DELETE` (idempotency); spawn `DELETE` and `UPDATE` (`creature`, `gameobject`) target that table alone (no join, comma list or schema qualifier) and filter on both `id` and `guid` with `=`/`IN`/`BETWEEN`, never `OR`; no double semicolons; no multiple blank lines; InnoDB engine.

## Data conventions (TODO: ADAPT TO 7.3.5 specific detailsm, below are lifted from AzerothCore 3.3.5)

- Set `flags_extra` on every `difficulty_entry_*` template, not just the base entry: which of the two the engine reads varies by call site.
- `smart_scripts` edits always rewrite the full block — `DELETE` + `INSERT` of every row for the `(entryorguid, source_type)` pair, with the `DELETE` matching both columns — never a partial `UPDATE`, not even for a comment-only fix.
- Sniff-backed changes stamp `VerifiedBuild` (the sniff's client build) on every row the sniff validated, including rows the fix doesn't otherwise touch.
- `creature_template_model`: rows were generated from the old `modelid1`-`modelid4` columns, and that migration copied `creature_template.VerifiedBuild` onto every `Idx`, so equal probabilities are that import's default rather than a randomization claim. Judge a stamp by its build, not its `Idx`: a legacy build on the extra rows is migration noise, while later sniff passes stamp `Idx` >= 1 legitimately. Authority is the creature query response's per-index `Probability`, which Classic and retail sniffs carry and 3.3.5 clients omit. Suppress a model by setting its `Probability` to 0 and keeping the row, never by deleting it; keep a creature's only row at 1, since the loader reads a zero total as equal chance and resets every row to 1.0.
- `creature_immunities`: negative ids are curated shared sets — reference them via `creature_template.CreatureImmunitiesId`, never edit them or allocate new ones. Positive ids are single-creature sets — reuse an existing set only on an exact match; to extend a creature's immunities, insert a superset under a new id and point the creature's `CreatureImmunitiesId` at it.

## The four databases

- `azcore_auth` — accounts, realm list, IP/account bans, session keys. Shared across all realms.
- `azcore_characters` — per-character state: characters, inventory, in-progress quests, mail, guilds, arena teams, achievements. One per realm.
- `azcore_world` — static game content: creature/gameobject/item/quest templates, spawn lists, loot tables, SmartAI scripts, gossip, conditions. Read-mostly; rebuilt from SQL.
- `azcore_hotfixes` — server-side overrides of the client's DB2 data (`item`, `item_sparse`, `spell_*`, `broadcast_text`, …), plus `hotfix_data`, which lists the changed records the client is told to re-fetch. One table per DB2 store, with `*_locale` siblings for localized strings. See [Hotfixes](#hotfixes).

## Hotfixes

Mirrors Blizzard's mechanism for changing client data without a client patch. Reference: TrinityCore `DB2Manager::LoadHotfixData` (`src/server/game/DataStores/DB2Stores.cpp`) and `src/server/game/Handlers/HotfixHandler.cpp`.

- **Server side:** each DB2 store loads the extracted `.db2` file, then layers the matching hotfixes table on top (`DB2StorageBase::LoadFromDB`). A row there adds a new record or replaces the client record with the same ID. Locale strings load as enUS from the base table; other locales come from `<table>_locale`.
- **Client side:** `hotfix_data (Id, TableHash, RecordId, Deleted)` is sent to the client on login (`SMSG_AVAILABLE_HOTFIXES`). The client compares it against its `DBCache.bin` and requests missing or outdated records (`CMSG_DB_QUERY_BULK` / `CMSG_HOTFIX_REQUEST`), which the server answers from its stores (`SMSG_DB_REPLY` / `SMSG_HOTFIX_RESPONSE`).
- `TableHash` is the client's hash of the DB2 name, not a table ID. A hash with no matching store is logged and skipped.
- `Deleted = 1` removes the record from the server's store and tells the client to drop it.

Conventions:

- Every added or changed row in a DB2 override table needs a matching `hotfix_data` row (same `TableHash` + `RecordId`). Without it the server's copy and the client's copy disagree. The client keeps its cached version, so tooltips, names and spell data may not match what the server actually uses.
- Allocate a new `hotfix_data.Id` per change instead of reusing an existing one. The client uses the Id to tell whether it already has a hotfix, so a reused Id may never be re-fetched.
- Data that's only ever used by the server belongs in `azcore_world`. Use `azcore_hotfixes` only when the client has to see the change.
