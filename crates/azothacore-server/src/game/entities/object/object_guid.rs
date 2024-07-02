use std::{
    cmp,
    fmt::{Debug, Display},
    mem::transmute,
};

use azothacore_common::HexFmt;
use flagset::{flags, FlagSet};

use crate::shared::realms::Realm;

flags! {
    pub enum TypeId: u16 {
        Object        = 0x0001,
        Item          = 0x0002,
        Container     = 0x0004,
        Unit          = 0x0008,
        Player        = 0x0010,
        Gameobject    = 0x0020,
        Dynamicobject = 0x0040,
        Corpse        = 0x0080,
        Areatrigger   = 0x0100,
        Sceneobject   = 0x0200,
        Conversation  = 0x0400,
        Seer          = (TypeId::Player | TypeId::Unit | TypeId::Dynamicobject).bits()
    }
}

impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl TypeId {
    pub fn mask(self) -> FlagSet<Self> {
        self.into()
    }

    // pub fn value(&self) -> u16 {
    //     let mut bits = self.mask().bits();
    //     let mut res = 0;
    //     while bits >= 1 {
    //         res += 1;
    //         bits >>= 1;
    //     }
    //     res - 1
    // }
}
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
#[repr(u8)]
pub enum HighGuid {
    Null = 0,
    Uniq = 1,
    Player = 2,
    Item = 3,
    WorldTransaction = 4,
    StaticDoor = 5, //NYI
    Transport = 6,
    Conversation = 7,
    Creature = 8,
    Vehicle = 9,
    Pet = 10,
    GameObject = 11,
    DynamicObject = 12,
    AreaTrigger = 13,
    Corpse = 14,
    LootObject = 15,
    SceneObject = 16,
    Scenario = 17,
    AIGroup = 18,
    DynamicDoor = 19,
    ClientActor = 20, //NYI
    Vignette = 21,
    CallForHelp = 22,
    AIResource = 23,
    AILock = 24,
    AILockTicket = 25,
    ChatChannel = 26,
    Party = 27,
    Guild = 28,
    WowAccount = 29,
    BNetAccount = 30,
    GMTask = 31,
    MobileSession = 32, //NYI
    RaidGroup = 33,
    Spell = 34,
    Mail = 35,
    WebObj = 36,    //NYI
    LFGObject = 37, //NYI
    LFGList = 38,   //NYI
    UserRouter = 39,
    PVPQueueGroup = 40,
    UserClient = 41,
    PetBattle = 42, //NYI
    UniqUserClient = 43,
    BattlePet = 44,
    CommerceObj = 45,
    ClientSession = 46,
    Cast = 47,
}

impl Display for HighGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl HighGuid {
    /// THIS FUNCTION IS UNSAFE! but its one of the only ways to easily
    /// derive the u8 value from enum, since HighGuid has is [repr(u8)]
    ///
    /// We could also use `as u8` syntax
    pub const fn to_id(&self) -> u8 {
        *self as u8
    }

    /// THIS FUNCTION IS UNSAFE! but its one of the only ways to easily
    /// derive the enum given a u8 value, since HighGuid has is [repr(u8)]
    ///
    ///
    pub const fn from_id(v: u8) -> Self {
        if v > Self::Cast as u8 {
            panic!("high guid should never fail to become take from primitive")
        }
        unsafe { transmute(v) }
    }

    pub const fn global(&self) -> bool {
        use HighGuid::*;
        matches!(
            *self,
            Uniq | Party
                | WowAccount
                | BNetAccount
                | GMTask
                | RaidGroup
                | Spell
                | Mail
                | UserRouter
                | PVPQueueGroup
                | UserClient
                | UniqUserClient
                | BattlePet
                | CommerceObj
                | ClientSession
        )
    }

    pub const fn realm_specific(&self) -> bool {
        use HighGuid::*;
        matches!(
            self,
            Player
            // This is not exactly correct, there are 2 more unknown parts in
            // highguid: (high >> 10 & 0xFF), (high >> 18 & 0xFFFFFF)
            | Item
            | Guild
            // Special case
            // Global transports are loaded from `transports` table, RealmSpecific part is
            // used for them.
            // after worldserver finishes loading, no more global transports can be created,
            // only the ones existing within instances that never change maps
            // here is where MapSpecific comes into play - each map takes over the responsibility
            // to generate transport guids
            // on top of this, regular elevators (GAMEOBJECT_TYPE_TRANSPORT) must also use
            // Transport highguid type, otherwise client will reject seeing other
            // players on them
            | Transport
        )
    }

    pub const fn map_specific(&self) -> bool {
        use HighGuid::*;
        matches!(
            *self,
            WorldTransaction
            | Conversation
            | Creature
            | Vehicle
            | Pet
            | GameObject
            | DynamicObject
            | AreaTrigger
            | Corpse
            | LootObject
            | SceneObject
            | Scenario
            | AIGroup
            | DynamicDoor
            | Vignette
            | CallForHelp
            | AIResource
            | AILock
            | AILockTicket
            | Cast
            // Special case
            // Global transports are loaded from `transports` table, RealmSpecific part is
            // used for them.
            // after worldserver finishes loading, no more global transports can be created,
            // only the ones existing within instances that never change maps
            // here is where MapSpecific comes into play - each map takes over the responsibility
            // to generate transport guids
            // on top of this, regular elevators (GAMEOBJECT_TYPE_TRANSPORT) must also use
            // Transport highguid type, otherwise client will reject seeing other
            // players on them
            | Transport
        )
    }

    pub fn get_type_id(&self) -> TypeId {
        match *self {
            HighGuid::Item => TypeId::Item,
            HighGuid::Creature | HighGuid::Pet | HighGuid::Vehicle => TypeId::Unit,
            HighGuid::Player => TypeId::Player,
            HighGuid::GameObject | HighGuid::Transport => TypeId::Gameobject,
            HighGuid::DynamicObject => TypeId::Dynamicobject,
            HighGuid::Corpse => TypeId::Corpse,
            HighGuid::AreaTrigger => TypeId::Areatrigger,
            HighGuid::SceneObject => TypeId::Sceneobject,
            HighGuid::Conversation => TypeId::Conversation,
            _ => TypeId::Object,
        }
    }
}

// TODO: Cleanup the Global, MapSpecific, RealmSpecific traits => Use derive macro to generalise the mapping
// TODO: Also cleanup the generics, <const H: u8> pattern into just <const H: HighGuid>. This is pending on this feature
//       feature(adt_const_params) to be stabilised, check here: https://github.com/rust-lang/rust/issues/95174

pub trait Global<const H: u8> {
    //     template<HighGuid type>
    //     static typename std::enable_if<ObjectGuidTraits<type>::Global, ObjectGuid>::type Create(LowType counter) { return Global(type, counter); }
    const TYPE: HighGuid = {
        let hi = HighGuid::from_id(H);
        if hi.global() {
            hi
        } else {
            panic!("high guid global does not allow any other values")
        }
    };

    fn create(counter: u64) -> ObjectGuid {
        ObjectGuid::global(Self::TYPE, counter)
    }
}

impl Global<{ HighGuid::Uniq as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::Party as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::WowAccount as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::BNetAccount as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::GMTask as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::RaidGroup as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::Spell as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::Mail as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::UserRouter as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::PVPQueueGroup as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::UserClient as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::UniqUserClient as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::BattlePet as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::CommerceObj as u8 }> for ObjectGuid {}
impl Global<{ HighGuid::ClientSession as u8 }> for ObjectGuid {}

pub trait RealmSpecific<const H: u8> {
    //     template<HighGuid type>
    //     static typename std::enable_if<ObjectGuidTraits<type>::RealmSpecific, ObjectGuid>::type Create(LowType counter) { return RealmSpecific(type, counter); }
    const TYPE: HighGuid = {
        let hi = HighGuid::from_id(H);
        if hi.realm_specific() {
            hi
        } else {
            panic!("realm specific high guid type does not allow any other values")
        }
    };

    fn create(realm: &Realm, counter: u64) -> ObjectGuid {
        ObjectGuid::realm_specific(realm, Self::TYPE, counter)
    }
}

impl RealmSpecific<{ HighGuid::Player as u8 }> for ObjectGuid {}
impl RealmSpecific<{ HighGuid::Item as u8 }> for ObjectGuid {}
impl RealmSpecific<{ HighGuid::Guild as u8 }> for ObjectGuid {}
impl RealmSpecific<{ HighGuid::Transport as u8 }> for ObjectGuid {}

pub trait MapSpecificWithSubType<const H: u8> {
    const TYPE: HighGuid = {
        let hi = HighGuid::from_id(H);
        if hi.map_specific() {
            hi
        } else {
            panic!("map specific high guid does not allow any other values")
        }
    };
    //     template<HighGuid type>
    //     static typename std::enable_if<ObjectGuidTraits<type>::MapSpecific, ObjectGuid>::type Create(uint8 subType, uint16 mapId, uint32 entry, LowType counter) { return MapSpecific(type, subType, mapId, 0, entry, counter); }

    // NOTE: hirogoro@22jan2024: Not implementing the below, we just sub in `subType == 0`, so that we dont need to maintain another set of traits + impls
    //     template<HighGuid type>
    //     static typename std::enable_if<ObjectGuidTraits<type>::MapSpecific && type != HighGuid::Transport, ObjectGuid>::type Create(uint16 mapId, uint32 entry, LowType counter) { return MapSpecific(type, 0, mapId, 0, entry, counter); }

    fn create(realm: &Realm, subtype: u8, map_id: u16, entry: u32, counter: u64) -> ObjectGuid {
        ObjectGuid::map_specific(realm, Self::TYPE, subtype, map_id, 0, entry, counter)
    }
}

impl MapSpecificWithSubType<{ HighGuid::WorldTransaction as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Conversation as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Creature as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Vehicle as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Pet as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::GameObject as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::DynamicObject as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::AreaTrigger as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Corpse as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::LootObject as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::SceneObject as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Scenario as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::AIGroup as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::DynamicDoor as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Vignette as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::CallForHelp as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::AIResource as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::AILock as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::AILockTicket as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Cast as u8 }> for ObjectGuid {}
impl MapSpecificWithSubType<{ HighGuid::Transport as u8 }> for ObjectGuid {}

pub trait MapSpecific<const H: u8>: MapSpecificWithSubType<H> {
    const TYPE: HighGuid = {
        let hi = HighGuid::from_id(H);
        if hi.map_specific() && !matches!(hi, HighGuid::Transport) {
            hi
        } else {
            panic!("map specific high guid does not allow any other values")
        }
    };
    fn create(realm: &Realm, map_id: u16, entry: u32, counter: u64) -> ObjectGuid {
        ObjectGuid::map_specific(realm, <Self as MapSpecific<H>>::TYPE, 0, map_id, 0, entry, counter)
    }
}

impl MapSpecific<{ HighGuid::WorldTransaction as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Conversation as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Creature as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Vehicle as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Pet as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::GameObject as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::DynamicObject as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::AreaTrigger as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Corpse as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::LootObject as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::SceneObject as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Scenario as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::AIGroup as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::DynamicDoor as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Vignette as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::CallForHelp as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::AIResource as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::AILock as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::AILockTicket as u8 }> for ObjectGuid {}
impl MapSpecific<{ HighGuid::Cast as u8 }> for ObjectGuid {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectGuid {
    pub low:  u64,
    /// High u64 layout (in general, could be different for different high types)
    /// 2nd row denotes the mask
    ///
    /// hightype realm_id         map_id        entry                   subtype
    /// XXXXXX   XXXXXXXXXXXXXXXX XXXXXXXXXXXXX XXXXXXXXXXXXXXXXXXXXXXX XXXXXX
    /// 111111   0001111111111111 1111111111111 11111111111111111111111 111111
    ///
    pub high: u64,
}

impl PartialOrd for ObjectGuid {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ObjectGuid {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let hi_cmp = self.high.cmp(&other.high);
        if !matches!(hi_cmp, cmp::Ordering::Equal) {
            return hi_cmp;
        }
        self.low.cmp(&other.low)
    }
}

impl Display for ObjectGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GUID Full: {high:#016X}{low:#016X} Type: {typ}",
            high = HexFmt(self.high.to_le_bytes()),
            low = HexFmt(self.low.to_le_bytes()),
            typ = self.get_type_id()
        )?;

        let entry = self.get_entry();
        if entry != 0 {
            if self.is_pet() {
                write!(f, " Pet number: {entry}")?;
            } else {
                write!(f, " Entry: {entry}")?;
            }
        }
        write!(f, " Low: {}", self.get_counter())?;
        Ok(())
    }
}

impl ObjectGuid {
    pub const EMPTY: Self = Self { low: 0, high: 0 };
    pub const TRADE_ITEM: Self = Self::global(HighGuid::Uniq, 10);

    /// Used when serialising to be used to be send as part of a wow packet.
    /// it will pack the GUID such that only the bytes that are set are sent.
    ///
    /// Expected packets are of the following order
    ///
    /// lo_mask: u8 =>      the bits of the low u64 that are set, i.e. if 0b01101001,
    ///                     then the  0-th, 3-rd, 5-th and 6-th bytes of the low guid,
    ///                     in that order are sent
    /// hi_mask: u8 =>      Similar to lo_mask, see that
    /// lo_bytes: u8*n =>   where n is the number of 1s set in lo_mask
    /// hi_bytes: u8*n =>   similar to lo_bytes
    ///
    /// This is the equivalent function in TC
    /// ByteBuffer& operator<<(ByteBuffer& buf, ObjectGuid const& guid)
    pub fn pack_into<B>(&self, buf: &mut B)
    where
        B: bytes::BufMut,
    {
        let mut packed = Vec::with_capacity(16);
        let low_mask = pack_u64(self.low, &mut packed);
        let high_mask = pack_u64(self.high, &mut packed);

        buf.put_u8(low_mask);
        buf.put_u8(high_mask);
        buf.put_slice(&packed);
    }

    /// Unpack from the given buffer, see [`Self::pack_into()`] for more info abput packing order
    pub fn unpack_from<B>(buf: &mut B) -> Self
    where
        B: bytes::Buf,
    {
        let low_mask = buf.get_u8();
        let high_mask = buf.get_u8();
        let low = unpack_u64(low_mask, buf);
        let high = unpack_u64(high_mask, buf);
        Self { low, high }
    }

    const fn global(typ: HighGuid, counter: u64) -> Self {
        Self {
            high: (typ.to_id() as u64) << 58,
            low:  counter,
        }
    }

    fn realm_specific(realm: &Realm, typ: HighGuid, counter: u64) -> Self {
        Self {
            high: (typ.to_id() as u64) << 58 | ((realm.id.realm & 0x1FFF) as u64) << 42,
            low:  counter,
        }
    }

    fn map_specific(realm: &Realm, typ: HighGuid, subtype: u8, map_id: u16, server_id: u32, entry: u32, counter: u64) -> Self {
        Self::create(typ, subtype, realm.id.realm, map_id, server_id, entry, counter)
    }

    /// creates the GUID from all its relevant parts
    pub const fn create(typ: HighGuid, subtype: u8, realm_id: u32, map_id: u16, server_id: u32, entry: u32, counter: u64) -> Self {
        let high = ((typ.to_id() as u64) << 58)
            | (((realm_id & 0x1FFF) as u64) << 42)
            | (((map_id & 0x1FFF) as u64) << 29)
            | (((entry & 0x7FFFFF) as u64) << 6)
            | ((subtype & 0x3F) as u64);
        let low = (((server_id & 0xFFFFFF) as u64) << 40) | (counter & 0xFFFFFFFFFF);
        Self { high, low }
    }

    pub const fn get_raw_value(&self) -> [u8; 16] {
        (((self.low as u128) << 64) + (self.high as u128)).to_le_bytes()
    }

    //     void SetRawValue(std::vector<uint8> const& guid);

    pub const fn get_high(&self) -> HighGuid {
        HighGuid::from_id(((self.high >> 58) & 0x3F) as u8)
    }

    pub const fn get_realm_id(&self) -> u32 {
        ((self.high >> 42) & 0x1FFF) as u32
    }

    pub const fn get_map_id(&self) -> u32 {
        ((self.high >> 29) & 0x1FFF) as u32
    }

    pub const fn get_entry(&self) -> u32 {
        ((self.high >> 6) & 0x7FFFFF) as u32
    }

    pub const fn get_counter(&self) -> u64 {
        self.low & 0x000000FFFFFFFFFF
    }

    pub const fn get_max_counter(_high: HighGuid) -> u64 {
        0xFFFFFFFFFF
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }

    pub fn is_creature(&self) -> bool {
        self.get_high() == HighGuid::Creature
    }

    pub fn is_pet(&self) -> bool {
        self.get_high() == HighGuid::Pet
    }

    pub fn is_vehicle(&self) -> bool {
        self.get_high() == HighGuid::Vehicle
    }

    pub fn is_creature_or_pet(&self) -> bool {
        self.is_creature() || self.is_pet()
    }

    pub fn is_creature_or_vehicle(&self) -> bool {
        self.is_creature() || self.is_vehicle()
    }

    pub fn is_any_type_creature(&self) -> bool {
        self.is_creature() || self.is_pet() || self.is_vehicle()
    }

    pub fn is_player(&self) -> bool {
        !self.is_empty() && self.get_high() == HighGuid::Player
    }

    pub fn is_unit(&self) -> bool {
        self.is_any_type_creature() || self.is_player()
    }

    pub fn is_item(&self) -> bool {
        self.get_high() == HighGuid::Item
    }

    pub fn is_game_object(&self) -> bool {
        self.get_high() == HighGuid::GameObject
    }

    pub fn is_dynamic_object(&self) -> bool {
        self.get_high() == HighGuid::DynamicObject
    }

    pub fn is_corpse(&self) -> bool {
        self.get_high() == HighGuid::Corpse
    }

    pub fn is_area_trigger(&self) -> bool {
        self.get_high() == HighGuid::AreaTrigger
    }

    pub fn is_mo_transport(&self) -> bool {
        self.get_high() == HighGuid::Transport
    }

    pub fn is_any_type_game_object(&self) -> bool {
        self.is_game_object() || self.is_mo_transport()
    }

    pub fn is_party(&self) -> bool {
        self.get_high() == HighGuid::Party
    }

    pub fn is_guild(&self) -> bool {
        self.get_high() == HighGuid::Guild
    }

    pub fn is_scene_object(&self) -> bool {
        self.get_high() == HighGuid::SceneObject
    }

    pub fn is_conversation(&self) -> bool {
        self.get_high() == HighGuid::Conversation
    }

    pub fn is_cast(&self) -> bool {
        self.get_high() == HighGuid::Cast
    }

    pub fn get_type_id(&self) -> TypeId {
        self.get_high().get_type_id()
    }
}

fn pack_u64(mut value: u64, result: &mut Vec<u8>) -> u8 {
    let mut mask = 0;
    let mut i = 0;
    while value != 0 {
        // This process is basically packing set bytes in the little endian order
        let b = (value & u8::MAX as u64) as u8;
        if b > 0 {
            mask |= 1 << i;
            result.push(b);
        }
        value >>= 8;
        i += 1;
    }
    mask
}

fn unpack_u64<B: bytes::Buf>(mask: u8, buf: &mut B) -> u64 {
    let mut val_bytes = [0u8; 8];
    for (i, b) in val_bytes.iter_mut().enumerate() {
        if (mask & (1 << i)) > 0 {
            *b = buf.get_u8();
        }
    }
    u64::from_le_bytes(val_bytes)
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use azothacore_common::AccountTypes;
    use ipnet::IpNet;

    use super::HighGuid;
    use crate::{
        game::entities::object::object_guid::{Global, MapSpecific, MapSpecificWithSubType, ObjectGuid, RealmSpecific},
        shared::realms::{BnetRealmHandle, Realm, RealmFlags, RealmType},
    };

    #[test]
    fn high_guid_to_from() {
        use HighGuid::*;
        for h in [
            Null,
            Uniq,
            Player,
            Item,
            WorldTransaction,
            StaticDoor,
            Transport,
            Conversation,
            Creature,
            Vehicle,
            Pet,
            GameObject,
            DynamicObject,
            AreaTrigger,
            Corpse,
            LootObject,
            SceneObject,
            Scenario,
            AIGroup,
            DynamicDoor,
            ClientActor,
            Vignette,
            CallForHelp,
            AIResource,
            AILock,
            AILockTicket,
            ChatChannel,
            Party,
            Guild,
            WowAccount,
            BNetAccount,
            GMTask,
            MobileSession,
            RaidGroup,
            Spell,
            Mail,
            WebObj,
            LFGObject,
            LFGList,
            UserRouter,
            PVPQueueGroup,
            UserClient,
            PetBattle,
            UniqUserClient,
            BattlePet,
            CommerceObj,
            ClientSession,
            Cast,
        ] {
            let h_in_u8 = h.to_id();
            let h_from_u8 = HighGuid::from_id(h_in_u8);
            assert_eq!(h_in_u8, h as u8);
            assert_eq!(h_from_u8, h);
        }
    }

    #[test]
    fn create_guids() {
        use HighGuid::*;
        let current_realm = Realm {
            id:                     BnetRealmHandle {
                realm:  123,
                region: 2,
                site:   1,
            },
            build:                  456,
            external_address:       SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085),
            local_address:          SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8085),
            local_network:          IpNet::with_netmask(
                std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                std::net::IpAddr::V4(Ipv4Addr::new(255, 255, 255, 0)),
            )
            .unwrap(),
            port:                   8085,
            realm_type:             RealmType::Normal,
            name:                   "TEST_CLIENT".to_string(),
            flag:                   RealmFlags::None.into(),
            timezone:               0,
            allowed_security_level: AccountTypes::SecPlayer,
            population_level:       0.0,
        };

        for (guid, high, counter, entry, realm, map, empty, raw, packed) in [
            (
                ObjectGuid::EMPTY,
                Null,
                0,
                0,
                0,
                0,
                true,
                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                vec![0, 0],
            ),
            (
                <ObjectGuid as Global   <{ HighGuid::WowAccount as u8 }>>::create(1111),
                WowAccount,
                1111,
                0,
                0,
                0,
                false,
                [0, 0, 0, 0, 0, 0, 0, 116, 87, 4, 0, 0, 0, 0, 0, 0],
                vec![0b00000011, 0b10000000, 87, 4, 116],
            ),
            (
                <ObjectGuid as RealmSpecific<{ Item as u8 }>>::create(&current_realm, 2222),
                Item,
                2222,
                0,
                current_realm.id.realm,
                0,
                false,
                [0, 0, 0, 0, 0, 236, 1, 12, 174, 8, 0, 0, 0, 0, 0, 0],
                vec![0b00000011, 0b11100000, 174, 8, 236, 1, 12],
            ),
            (
                <ObjectGuid as MapSpecific<{ DynamicObject as u8 }>>::create(&current_realm, 1, 2, 3333),
                DynamicObject,
                3333,
                2,
                current_realm.id.realm,
                1,
                false,
                [128, 0, 0, 32, 0, 236, 1, 48, 5, 13, 0, 0, 0, 0, 0, 0],
                vec![0b00000011, 0b11101001, 5, 13, 128, 32, 236, 1, 48],
            ),
            (
                <ObjectGuid as MapSpecificWithSubType<{ Transport as u8 }>>::create(&current_realm, 4, 2, 3, 4444),
                Transport,
                4444,
                3,
                current_realm.id.realm,
                2,
                false,
                [196, 0, 0, 64, 0, 236, 1, 24, 92, 17, 0, 0, 0, 0, 0, 0],
                vec![0b00000011, 0b11101001, 92, 17, 196, 64, 236, 1, 24],
            ),
        ] {
            assert_eq!(guid.get_high(), high);
            assert_eq!(guid.get_counter(), counter);
            assert_eq!(guid.get_entry(), entry);
            assert_eq!(guid.get_realm_id(), realm);
            assert_eq!(guid.get_map_id(), map);
            assert_eq!(guid.is_empty(), empty);
            assert_eq!(guid.get_raw_value(), raw);
            let mut our_packed = vec![];
            guid.pack_into(&mut our_packed);
            assert_eq!(our_packed, packed);

            let their_guid = ObjectGuid::unpack_from(&mut packed.as_slice());
            assert_eq!(guid, their_guid);
        }
    }
}
