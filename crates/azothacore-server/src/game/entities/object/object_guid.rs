use std::{
    any::Any,
    cmp,
    fmt::{Debug, Display},
    marker::PhantomData,
    sync::atomic::AtomicU64,
};

use azothacore_common::{az_error, bevy_app::ToFromEntity, AzError, AzResult, HexFmt};
use azothacore_database::database_env::{CharacterDatabase, WorldDatabase};
use bevy::{
    ecs::system::{Res, SystemParam},
    prelude::Entity,
};
use flagset::{flags, FlagSet};

use crate::{
    game::world::CurrentRealm,
    shared::{
        id_generators::{DBIDGenerator, IDGenerator, IDGeneratorTrait},
        realms::Realm,
    },
};

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
}

const fn global_guid<H: HighGlobal>(counter: u64) -> ObjectGuid<H> {
    let high = H::ID as u64;
    ObjectGuid::<H> {
        high:     high << 58,
        low:      counter,
        _phantom: PhantomData,
    }
}

pub trait ObjectGuidGlobal<H: HighGlobal> {
    fn global(counter: u64) -> ObjectGuid<H> {
        global_guid(counter)
    }
}

pub trait HighGlobal: HighGuidTrait {}

impl<H: HighGlobal> ObjectGuidGlobal<H> for ObjectGuid<H> {}

pub trait ObjectGuidRealmSpecific<H: HighGuidTrait> {
    fn realm_specific(realm: &Realm, counter: u64) -> ObjectGuid<H> {
        let typ: u64 = H::ID.into();
        ObjectGuid::<H> {
            high:     (typ << 58) | (((realm.id.realm & 0x1FFF) as u64) << 42),
            low:      counter,
            _phantom: Default::default(),
        }
    }
}

pub trait RealmRelated {}

pub trait HighRealmSpecific: HighGuidTrait + RealmRelated {}

impl<H: HighRealmSpecific> ObjectGuidRealmSpecific<H> for ObjectGuid<H> {}

impl<H: ?Sized + RealmRelated> ObjectGuid<H> {
    /// ObjectGuid::GetRealmId in TC / AC
    pub const fn realm_id(&self) -> u32 {
        ((self.high >> 42) & 0x1FFF) as u32
    }
}

fn map_specific_object_guid<H: HighGuidTrait>(realm: &Realm, subtype: u8, map_id: u16, server_id: u32, entry: u32, counter: u64) -> ObjectGuid<H> {
    let typ: u64 = H::ID.into();
    let high = (typ << 58)
        | (((realm.id.realm & 0x1FFF) as u64) << 42)
        | (((map_id & 0x1FFF) as u64) << 29)
        | (((entry & 0x7FFFFF) as u64) << 6)
        | ((subtype & 0x3F) as u64);
    let low = (((server_id & 0xFFFFFF) as u64) << 40) | (counter & 0xFFFFFFFFFF);
    ObjectGuid::<H> {
        high,
        low,
        _phantom: Default::default(),
    }
}
pub trait MapRelated {}

pub trait HighMapSpecific: HighMapSpecificWithSubType {}
pub trait HighMapSpecificWithSubType: HighGuidTrait + RealmRelated + MapRelated {}

pub trait ObjectGuidMapSpecific<H: HighMapSpecific> {
    fn map_specific(realm: &Realm, map_id: u16, entry: u32, counter: u64) -> ObjectGuid<H> {
        map_specific_object_guid(realm, 0, map_id, 0, entry, counter)
    }
}

pub trait ObjectGuidMapSpecificWithSubType<H: HighMapSpecificWithSubType> {
    fn map_specific_with_subtype(realm: &Realm, subtype: u8, map_id: u16, entry: u32, counter: u64) -> ObjectGuid<H> {
        map_specific_object_guid(realm, subtype, map_id, 0, entry, counter)
    }
}

impl<H: HighMapSpecific> ObjectGuidMapSpecific<H> for ObjectGuid<H> {}
impl<H: HighMapSpecificWithSubType> ObjectGuidMapSpecificWithSubType<H> for ObjectGuid<H> {}

impl<H: ?Sized + MapRelated> ObjectGuid<H> {
    /// ObjectGuid::GetMapId in TC / AC
    pub const fn map_id(&self) -> u32 {
        ((self.high >> 29) & 0x1FFF) as u32
    }

    /// ObjectGuid::GetEntry in TC / AC
    pub const fn entry(&self) -> u32 {
        ((self.high >> 6) & 0x7FFFFF) as u32
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ObjectGuid<H: ?Sized = ()> {
    low:      u64,
    /// High u64 layout (in general, could be different for different high types)
    /// 2nd row denotes the mask
    ///
    /// hightype realm_id         map_id        entry                   subtype
    /// XXXXXX   XXXXXXXXXXXXXXXX XXXXXXXXXXXXX XXXXXXXXXXXXXXXXXXXXXXX XXXXXX
    /// 111111   0001111111111111 1111111111111 11111111111111111111111 111111
    ///
    high:     u64,
    _phantom: PhantomData<H>,
}

impl<H1, H2> PartialOrd<ObjectGuid<H2>> for ObjectGuid<H1> {
    fn partial_cmp(&self, other: &ObjectGuid<H2>) -> Option<cmp::Ordering> {
        let hi_cmp = self.high.cmp(&other.high);
        if !matches!(hi_cmp, cmp::Ordering::Equal) {
            return Some(hi_cmp);
        }
        Some(self.low.cmp(&other.low))
    }
}

impl<H> Ord for ObjectGuid<H> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.partial_cmp(other).expect("always expect order for ObjectGUIDs to be total")
    }
}

impl<H1, H2> PartialEq<ObjectGuid<H2>> for ObjectGuid<H1> {
    fn eq(&self, other: &ObjectGuid<H2>) -> bool {
        self.high == other.high && self.low == other.low
    }
}

impl<H> Eq for ObjectGuid<H> {}

impl<H: HighGuidTrait + 'static> Display for ObjectGuid<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "GUID Full: {high:#016X}{low:#016X} Type: {typ}",
            high = HexFmt(self.high.to_le_bytes()),
            low = HexFmt(self.low.to_le_bytes()),
            typ = H::TYPE_ID
        )?;
        if let Some(this) = self.as_map_related() {
            let entry = this.entry();
            if self.raw_hightype() == HighGuidPet::ID {
                write!(f, " Pet number: {entry}")?;
            } else {
                write!(f, " Entry: {entry}")?;
            }
        }

        write!(f, " Low: {}", self.counter())?;
        Ok(())
    }
}

impl<H: HighGuidTrait> TryFrom<ObjectGuid> for ObjectGuid<H> {
    type Error = AzError;

    fn try_from(value: ObjectGuid) -> Result<Self, Self::Error> {
        let ht = value.raw_hightype();
        let ObjectGuid { low, high, .. } = value;
        if ht != H::ID {
            Err(az_error!("wrong high_guid type {ht}"))
        } else {
            Ok(ObjectGuid::<H> {
                low,
                high,
                _phantom: Default::default(),
            })
        }
    }
}

impl TryFrom<&[u8]> for ObjectGuid {
    type Error = AzError;

    /// ObjectGuid::SetRawValue in TC
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 16 {
            return Err(az_error!("value passed in is not at least length 16. {value:?}"));
        }

        let lo_bytes = &value[..8];
        let hi_bytes = &value[8..];
        Ok(Self {
            low:      u64::from_le_bytes(lo_bytes.try_into()?),
            high:     u64::from_le_bytes(hi_bytes.try_into()?),
            _phantom: Default::default(),
        })
    }
}

impl<H: HighGuidTrait> TryFrom<&[u8]> for ObjectGuid<H> {
    type Error = AzError;

    /// ObjectGuid::SetRawValue in TC
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        ObjectGuid::<()>::try_from(value).and_then(|o| o.try_into())
    }
}

impl<H: HighGuidTrait> From<ObjectGuid<H>> for ObjectGuid {
    fn from(value: ObjectGuid<H>) -> Self {
        let ObjectGuid { low, high, .. } = value;
        Self {
            low,
            high,
            ..Default::default()
        }
    }
}

impl Default for ObjectGuid {
    fn default() -> Self {
        Self::EMPTY
    }
}

impl ObjectGuid {
    /// ObjectGuid::Empty in TC / AC
    pub const EMPTY: Self = Self {
        low:      0,
        high:     0,
        _phantom: PhantomData,
    };

    /// Unpack from the given buffer, see [`Self::pack_into()`] for more info abput packing order
    pub fn unpack_from<B>(buf: &mut B) -> Self
    where
        B: bytes::Buf,
    {
        let low_mask = buf.get_u8();
        let high_mask = buf.get_u8();
        let low = unpack_u64(low_mask, buf);
        let high = unpack_u64(high_mask, buf);
        Self {
            low,
            high,
            _phantom: Default::default(),
        }
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

impl<H> ObjectGuid<H> {
    /// ObjectGuid::GetHigh in TC / AC
    const fn raw_hightype(&self) -> u8 {
        ((self.high >> 58) & 0x3F) as u8
    }

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

    pub fn is_empty(&self) -> bool {
        *self == ObjectGuid::EMPTY
    }

    pub fn is_creature(&self) -> bool {
        self.raw_hightype() == HighGuidCreature::ID
    }

    pub fn is_pet(&self) -> bool {
        self.raw_hightype() == HighGuidPet::ID
    }

    pub fn is_vehicle(&self) -> bool {
        self.raw_hightype() == HighGuidVehicle::ID
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
        !self.is_empty() && self.raw_hightype() == HighGuidPlayer::ID
    }

    pub fn is_unit(&self) -> bool {
        self.is_any_type_creature() || self.is_player()
    }

    pub fn is_item(&self) -> bool {
        self.raw_hightype() == HighGuidItem::ID
    }

    pub fn is_game_object(&self) -> bool {
        self.raw_hightype() == HighGuidGameObject::ID
    }

    pub fn is_dynamic_object(&self) -> bool {
        self.raw_hightype() == HighGuidDynamicObject::ID
    }

    pub fn is_corpse(&self) -> bool {
        self.raw_hightype() == HighGuidCorpse::ID
    }

    pub fn is_area_trigger(&self) -> bool {
        self.raw_hightype() == HighGuidAreaTrigger::ID
    }

    pub fn is_mo_transport(&self) -> bool {
        self.raw_hightype() == HighGuidTransport::ID
    }

    pub fn is_any_type_game_object(&self) -> bool {
        self.is_game_object() || self.is_mo_transport()
    }

    pub fn is_party(&self) -> bool {
        self.raw_hightype() == HighGuidParty::ID
    }

    pub fn is_guild(&self) -> bool {
        self.raw_hightype() == HighGuidGuild::ID
    }

    pub fn is_scene_object(&self) -> bool {
        self.raw_hightype() == HighGuidSceneObject::ID
    }

    pub fn is_conversation(&self) -> bool {
        self.raw_hightype() == HighGuidConversation::ID
    }

    pub fn is_cast(&self) -> bool {
        self.raw_hightype() == HighGuidCast::ID
    }

    pub const fn raw_value(&self) -> [u8; 16] {
        (((self.low as u128) << 64) + (self.high as u128)).to_le_bytes()
    }
}

impl<H: 'static> ObjectGuid<H> {
    fn as_map_related(&self) -> Option<&ObjectGuid<dyn MapRelated>> {
        // This is where the downcasting happens
        let c = self as &dyn Any;
        c.downcast_ref()
    }

    fn counter(&self) -> u64 {
        if self.as_map_related().is_some() {
            self.low & 0x000000FFFFFFFFFF
        } else {
            self.low
        }
    }
}

impl<H: HighGuidTrait> ObjectGuid<H> {
    pub fn try_unpack_from<B>(buf: &mut B) -> AzResult<Self>
    where
        B: bytes::Buf,
    {
        let non_high_guid = ObjectGuid::unpack_from(buf);
        non_high_guid.try_into()
    }
}

impl ObjectGuid<HighGuidUniq> {
    pub const TRADE_ITEM: Self = global_guid(10);
}

impl ToFromEntity for ObjectGuid<HighGuidParty> {
    fn from_entity(entity: Entity) -> Self {
        Self::global(entity.to_bits())
    }

    fn to_entity(self) -> Entity {
        Entity::from_bits(self.counter())
    }
}

pub trait HighGuidTrait {
    const ID: u8;
    const TYPE_ID: TypeId = TypeId::Object;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidNull;
impl HighGuidTrait for HighGuidNull {
    const ID: u8 = 0;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidUniq;
impl HighGuidTrait for HighGuidUniq {
    const ID: u8 = 1;
}
impl HighGlobal for HighGuidUniq {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidPlayer;
impl HighGuidTrait for HighGuidPlayer {
    const ID: u8 = 2;
    const TYPE_ID: TypeId = TypeId::Player;
}
impl HighRealmSpecific for HighGuidPlayer {}
impl RealmRelated for HighGuidPlayer {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidItem;
impl HighGuidTrait for HighGuidItem {
    const ID: u8 = 3;
    const TYPE_ID: TypeId = TypeId::Item;
}
/// This is not exactly correct, there are 2 more unknown parts in
/// highguid: (high >> 10 & 0xFF), (high >> 18 & 0xFFFFFF)
impl HighRealmSpecific for HighGuidItem {}
impl RealmRelated for HighGuidItem {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidWorldTransaction;
impl HighGuidTrait for HighGuidWorldTransaction {
    const ID: u8 = 4;
}
impl HighMapSpecificWithSubType for HighGuidWorldTransaction {}
impl HighMapSpecific for HighGuidWorldTransaction {}
impl RealmRelated for HighGuidWorldTransaction {}
impl MapRelated for HighGuidWorldTransaction {}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidStaticDoor;
impl HighGuidTrait for HighGuidStaticDoor {
    const ID: u8 = 5;
}
/// HighGuid::Transport in TC.
/// HighGuid::Mo_Transport in AC.
///
/// Special case
/// Global transports are loaded from `transports` table, RealmSpecific part is
/// used for them.
///
/// after worldserver finishes loading, no more global transports can be created,
/// only the ones existing within instances that never change maps
///
/// here is where MapSpecific comes into play - each map takes over the responsibility
/// to generate transport guids
/// on top of this, regular elevators (GAMEOBJECT_TYPE_TRANSPORT) must also use
/// Transport highguid type, otherwise client will reject seeing other
/// players on them
#[derive(Debug, Clone, Copy)]
pub struct HighGuidTransport;
impl HighGuidTrait for HighGuidTransport {
    const ID: u8 = 6;
    const TYPE_ID: TypeId = TypeId::Gameobject;
}
impl HighRealmSpecific for HighGuidTransport {}
impl HighMapSpecificWithSubType for HighGuidTransport {}
impl RealmRelated for HighGuidTransport {}
impl MapRelated for HighGuidTransport {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidConversation;
impl HighGuidTrait for HighGuidConversation {
    const ID: u8 = 7;
    const TYPE_ID: TypeId = TypeId::Conversation;
}
impl HighMapSpecificWithSubType for HighGuidConversation {}
impl HighMapSpecific for HighGuidConversation {}
impl RealmRelated for HighGuidConversation {}
impl MapRelated for HighGuidConversation {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidCreature;
impl HighGuidTrait for HighGuidCreature {
    const ID: u8 = 8;
    const TYPE_ID: TypeId = TypeId::Unit;
}
impl HighMapSpecificWithSubType for HighGuidCreature {}
impl HighMapSpecific for HighGuidCreature {}
impl RealmRelated for HighGuidCreature {}
impl MapRelated for HighGuidCreature {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidVehicle;
impl HighGuidTrait for HighGuidVehicle {
    const ID: u8 = 9;
    const TYPE_ID: TypeId = TypeId::Unit;
}
impl HighMapSpecificWithSubType for HighGuidVehicle {}
impl HighMapSpecific for HighGuidVehicle {}
impl RealmRelated for HighGuidVehicle {}
impl MapRelated for HighGuidVehicle {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidPet;
impl HighGuidTrait for HighGuidPet {
    const ID: u8 = 10;
    const TYPE_ID: TypeId = TypeId::Unit;
}
impl HighMapSpecificWithSubType for HighGuidPet {}
impl HighMapSpecific for HighGuidPet {}
impl RealmRelated for HighGuidPet {}
impl MapRelated for HighGuidPet {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidGameObject;
impl HighGuidTrait for HighGuidGameObject {
    const ID: u8 = 11;
    const TYPE_ID: TypeId = TypeId::Gameobject;
}
impl HighMapSpecificWithSubType for HighGuidGameObject {}
impl HighMapSpecific for HighGuidGameObject {}
impl RealmRelated for HighGuidGameObject {}
impl MapRelated for HighGuidGameObject {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidDynamicObject;
impl HighGuidTrait for HighGuidDynamicObject {
    const ID: u8 = 12;
    const TYPE_ID: TypeId = TypeId::Dynamicobject;
}
impl HighMapSpecificWithSubType for HighGuidDynamicObject {}
impl HighMapSpecific for HighGuidDynamicObject {}
impl RealmRelated for HighGuidDynamicObject {}
impl MapRelated for HighGuidDynamicObject {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidAreaTrigger;
impl HighGuidTrait for HighGuidAreaTrigger {
    const ID: u8 = 13;
    const TYPE_ID: TypeId = TypeId::Areatrigger;
}
impl HighMapSpecificWithSubType for HighGuidAreaTrigger {}
impl HighMapSpecific for HighGuidAreaTrigger {}
impl RealmRelated for HighGuidAreaTrigger {}
impl MapRelated for HighGuidAreaTrigger {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidCorpse;
impl HighGuidTrait for HighGuidCorpse {
    const ID: u8 = 14;
    const TYPE_ID: TypeId = TypeId::Corpse;
}
impl HighMapSpecificWithSubType for HighGuidCorpse {}
impl HighMapSpecific for HighGuidCorpse {}
impl RealmRelated for HighGuidCorpse {}
impl MapRelated for HighGuidCorpse {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidLootObject;
impl HighGuidTrait for HighGuidLootObject {
    const ID: u8 = 15;
}
impl HighMapSpecificWithSubType for HighGuidLootObject {}
impl HighMapSpecific for HighGuidLootObject {}
impl RealmRelated for HighGuidLootObject {}
impl MapRelated for HighGuidLootObject {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidSceneObject;
impl HighGuidTrait for HighGuidSceneObject {
    const ID: u8 = 16;
    const TYPE_ID: TypeId = TypeId::Sceneobject;
}
impl HighMapSpecificWithSubType for HighGuidSceneObject {}
impl HighMapSpecific for HighGuidSceneObject {}
impl RealmRelated for HighGuidSceneObject {}
impl MapRelated for HighGuidSceneObject {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidScenario;
impl HighGuidTrait for HighGuidScenario {
    const ID: u8 = 17;
}
impl HighMapSpecificWithSubType for HighGuidScenario {}
impl HighMapSpecific for HighGuidScenario {}
impl RealmRelated for HighGuidScenario {}
impl MapRelated for HighGuidScenario {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidAIGroup;
impl HighGuidTrait for HighGuidAIGroup {
    const ID: u8 = 18;
}
impl HighMapSpecificWithSubType for HighGuidAIGroup {}
impl HighMapSpecific for HighGuidAIGroup {}
impl RealmRelated for HighGuidAIGroup {}
impl MapRelated for HighGuidAIGroup {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidDynamicDoor;
impl HighGuidTrait for HighGuidDynamicDoor {
    const ID: u8 = 19;
}
impl HighMapSpecificWithSubType for HighGuidDynamicDoor {}
impl HighMapSpecific for HighGuidDynamicDoor {}
impl RealmRelated for HighGuidDynamicDoor {}
impl MapRelated for HighGuidDynamicDoor {}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidClientActor;
impl HighGuidTrait for HighGuidClientActor {
    const ID: u8 = 20;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidVignette;
impl HighGuidTrait for HighGuidVignette {
    const ID: u8 = 21;
}
impl HighMapSpecificWithSubType for HighGuidVignette {}
impl HighMapSpecific for HighGuidVignette {}
impl RealmRelated for HighGuidVignette {}
impl MapRelated for HighGuidVignette {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidCallForHelp;
impl HighGuidTrait for HighGuidCallForHelp {
    const ID: u8 = 22;
}
impl HighMapSpecificWithSubType for HighGuidCallForHelp {}
impl HighMapSpecific for HighGuidCallForHelp {}
impl RealmRelated for HighGuidCallForHelp {}
impl MapRelated for HighGuidCallForHelp {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidAIResource;
impl HighGuidTrait for HighGuidAIResource {
    const ID: u8 = 23;
}
impl HighMapSpecificWithSubType for HighGuidAIResource {}
impl HighMapSpecific for HighGuidAIResource {}
impl RealmRelated for HighGuidAIResource {}
impl MapRelated for HighGuidAIResource {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidAILock;
impl HighGuidTrait for HighGuidAILock {
    const ID: u8 = 24;
}
impl HighMapSpecificWithSubType for HighGuidAILock {}
impl HighMapSpecific for HighGuidAILock {}
impl RealmRelated for HighGuidAILock {}
impl MapRelated for HighGuidAILock {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidAILockTicket;
impl HighGuidTrait for HighGuidAILockTicket {
    const ID: u8 = 25;
}
impl HighMapSpecificWithSubType for HighGuidAILockTicket {}
impl HighMapSpecific for HighGuidAILockTicket {}
impl RealmRelated for HighGuidAILockTicket {}
impl MapRelated for HighGuidAILockTicket {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidChatChannel;
impl HighGuidTrait for HighGuidChatChannel {
    const ID: u8 = 26;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidParty;
impl HighGuidTrait for HighGuidParty {
    const ID: u8 = 27;
}
impl HighGlobal for HighGuidParty {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidGuild;
impl HighGuidTrait for HighGuidGuild {
    const ID: u8 = 28;
}
impl HighRealmSpecific for HighGuidGuild {}
impl RealmRelated for HighGuidGuild {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidWowAccount;
impl HighGuidTrait for HighGuidWowAccount {
    const ID: u8 = 29;
}
impl HighGlobal for HighGuidWowAccount {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidBNetAccount;
impl HighGuidTrait for HighGuidBNetAccount {
    const ID: u8 = 30;
}
impl HighGlobal for HighGuidBNetAccount {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidGMTask;
impl HighGuidTrait for HighGuidGMTask {
    const ID: u8 = 31;
}
impl HighGlobal for HighGuidGMTask {}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidMobileSession;
impl HighGuidTrait for HighGuidMobileSession {
    const ID: u8 = 32;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidRaidGroup;
impl HighGuidTrait for HighGuidRaidGroup {
    const ID: u8 = 33;
}
impl HighGlobal for HighGuidRaidGroup {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidSpell;
impl HighGuidTrait for HighGuidSpell {
    const ID: u8 = 34;
}
impl HighGlobal for HighGuidSpell {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidMail;
impl HighGuidTrait for HighGuidMail {
    const ID: u8 = 35;
}
impl HighGlobal for HighGuidMail {}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidWebObj;
impl HighGuidTrait for HighGuidWebObj {
    const ID: u8 = 36;
}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidLFGObject;
impl HighGuidTrait for HighGuidLFGObject {
    const ID: u8 = 37;
}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidLFGList;
impl HighGuidTrait for HighGuidLFGList {
    const ID: u8 = 38;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidUserRouter;
impl HighGuidTrait for HighGuidUserRouter {
    const ID: u8 = 39;
}
impl HighGlobal for HighGuidUserRouter {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidPVPQueueGroup;
impl HighGuidTrait for HighGuidPVPQueueGroup {
    const ID: u8 = 40;
}
impl HighGlobal for HighGuidPVPQueueGroup {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidUserClient;
impl HighGuidTrait for HighGuidUserClient {
    const ID: u8 = 41;
}
impl HighGlobal for HighGuidUserClient {}

/// NYI
#[derive(Debug, Clone, Copy)]
pub struct HighGuidPetBattle;
impl HighGuidTrait for HighGuidPetBattle {
    const ID: u8 = 42;
}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidUniqUserClient;
impl HighGuidTrait for HighGuidUniqUserClient {
    const ID: u8 = 43;
}
impl HighGlobal for HighGuidUniqUserClient {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidBattlePet;
impl HighGuidTrait for HighGuidBattlePet {
    const ID: u8 = 44;
}
impl HighGlobal for HighGuidBattlePet {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidCommerceObj;
impl HighGuidTrait for HighGuidCommerceObj {
    const ID: u8 = 45;
}
impl HighGlobal for HighGuidCommerceObj {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidClientSession;
impl HighGuidTrait for HighGuidClientSession {
    const ID: u8 = 46;
}
impl HighGlobal for HighGuidClientSession {}

#[derive(Debug, Clone, Copy)]
pub struct HighGuidCast;
impl HighGuidTrait for HighGuidCast {
    const ID: u8 = 47;
}
impl HighMapSpecificWithSubType for HighGuidCast {}
impl HighMapSpecific for HighGuidCast {}
impl RealmRelated for HighGuidCast {}
impl MapRelated for HighGuidCast {}

pub type ObjectGuidLowGenerator<H> = IDGenerator<H, AtomicU64, u64>;

impl DBIDGenerator<CharacterDatabase, u64> for ObjectGuidLowGenerator<HighGuidPlayer> {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(guid), 0) AS UNSIGNED INT)+1 FROM characters";
}
impl DBIDGenerator<CharacterDatabase, u64> for ObjectGuidLowGenerator<HighGuidItem> {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(guid), 0) AS UNSIGNED INT)+1 FROM item_instance";
}
impl DBIDGenerator<WorldDatabase, u64> for ObjectGuidLowGenerator<HighGuidTransport> {
    const DB_SELECT_MAX_ID_QUERY: &str = "SELECT CAST(COALESCE(MAX(guid), 0) AS UNSIGNED INT)+1 FROM transports";
}

#[derive(SystemParam)]
pub struct GlobalObjectGuidGenerator<'w, H: HighGlobal + Send + Sync + 'static>(Res<'w, ObjectGuidLowGenerator<H>>);

impl<H: HighGlobal + Send + Sync + 'static> GlobalObjectGuidGenerator<'_, H> {
    pub fn generate(&self) -> AzResult<ObjectGuid<H>> {
        self.0.generate().map(|v| ObjectGuid::<H>::global(v))
    }
}

#[derive(SystemParam)]
pub struct RealmSpecificObjectGuidGenerator<'w, H: HighRealmSpecific + Send + Sync + 'static> {
    low_gen:       Res<'w, ObjectGuidLowGenerator<H>>,
    current_realm: Res<'w, CurrentRealm>,
}

impl<H: HighRealmSpecific + Send + Sync + 'static> RealmSpecificObjectGuidGenerator<'_, H> {
    pub fn generate(&self) -> AzResult<ObjectGuid<H>> {
        self.low_gen.generate().map(|v| ObjectGuid::<H>::realm_specific(&self.current_realm, v))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::Debug,
        net::{Ipv4Addr, SocketAddr},
    };

    use azothacore_common::AccountTypes;
    use ipnet::IpNet;

    use crate::{
        game::entities::object::object_guid::{
            HighGuidDynamicObject,
            HighGuidItem,
            HighGuidTransport,
            HighGuidWowAccount,
            HighMapSpecificWithSubType,
            ObjectGuid,
            ObjectGuidGlobal,
            ObjectGuidMapSpecific,
            ObjectGuidMapSpecificWithSubType,
            ObjectGuidRealmSpecific,
            RealmRelated,
        },
        shared::realms::{BnetRealmHandle, Realm, RealmFlags, RealmType},
    };

    // <H: HighGlobal> ObjectGuidTrait<H> for ObjectGuid<H>

    fn assert_guid<H: Debug + 'static>(guid: ObjectGuid<H>, counter: u64, empty: bool, raw: [u8; 16], packed: Vec<u8>) {
        assert_eq!(guid.counter(), counter);
        assert_eq!(guid.is_empty(), empty);
        assert_eq!(guid.raw_value(), raw);
        let mut our_packed = vec![];
        guid.pack_into(&mut our_packed);
        assert_eq!(our_packed, packed);

        let their_guid = ObjectGuid::unpack_from(&mut packed.as_slice());
        assert_eq!(guid, their_guid);
    }

    fn assert_guid_map_specific<H: HighMapSpecificWithSubType>(guid: ObjectGuid<H>, entry: u32, map: u32) {
        assert_eq!(guid.entry(), entry);
        assert_eq!(guid.map_id(), map);
    }

    fn assert_guid_realm_specific<H: RealmRelated>(guid: ObjectGuid<H>, realm: u32) {
        assert_eq!(guid.realm_id(), realm);
    }

    #[test]
    fn create_guids() {
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

        assert_guid(ObjectGuid::EMPTY, 0, true, [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], vec![0, 0]);

        assert_guid(
            ObjectGuid::<HighGuidWowAccount>::global(1111),
            1111,
            false,
            [0, 0, 0, 0, 0, 0, 0, 116, 87, 4, 0, 0, 0, 0, 0, 0],
            vec![0b00000011, 0b10000000, 87, 4, 116],
        );

        let guid = ObjectGuid::<HighGuidItem>::realm_specific(&current_realm, 2222);
        assert_guid(
            guid,
            2222,
            false,
            [0, 0, 0, 0, 0, 236, 1, 12, 174, 8, 0, 0, 0, 0, 0, 0],
            vec![0b00000011, 0b11100000, 174, 8, 236, 1, 12],
        );
        assert_guid_realm_specific(guid, current_realm.id.realm);

        let guid = ObjectGuid::<HighGuidDynamicObject>::map_specific(&current_realm, 1, 2, 3333);
        assert_guid(
            guid,
            3333,
            false,
            [128, 0, 0, 32, 0, 236, 1, 48, 5, 13, 0, 0, 0, 0, 0, 0],
            vec![0b00000011, 0b11101001, 5, 13, 128, 32, 236, 1, 48],
        );
        assert_guid_realm_specific(guid, current_realm.id.realm);
        assert_guid_map_specific(guid, 2, 1);

        let guid = ObjectGuid::<HighGuidTransport>::map_specific_with_subtype(&current_realm, 4, 2, 3, 4444);
        assert_guid(
            guid,
            4444,
            false,
            [196, 0, 0, 64, 0, 236, 1, 24, 92, 17, 0, 0, 0, 0, 0, 0],
            vec![0b00000011, 0b11101001, 92, 17, 196, 64, 236, 1, 24],
        );
        assert_guid_realm_specific(guid, current_realm.id.realm);
        assert_guid_map_specific(guid, 3, 2);
    }
}
