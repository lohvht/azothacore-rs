use std::collections::BTreeMap;

use crate::{
    game::server::{
        packet::{ClientWorldPacket, ServerWorldPacket},
        protocol::opcodes::{CMSG, OpcodeClient, OpcodeServer, SMSG},
    },
    shared::{
        packets::{AppendBufferBytes, ByteBuffer, ByteBufferError, ReadBufferBytes},
        shared_defines::Expansion,
    },
};

pub struct PingClientPacket {
    pub serial:  u32,
    pub latency: u32,
}

impl ClientWorldPacket for PingClientPacket {
    const OPCODE: OpcodeClient = CMSG::Ping;

    fn read(p: &mut ByteBuffer) -> Result<Self, ByteBufferError> {
        Ok(Self {
            serial:  p.read()?,
            latency: p.read()?,
        })
    }
}

pub struct PongServerPacket {
    pub serial: u32,
}

impl ServerWorldPacket for PongServerPacket {
    const INITIAL_CAP: usize = 4;
    const OPCODE: OpcodeServer = SMSG::Pong;

    fn write(&self) -> ByteBuffer {
        let mut buf = Self::init_buffer();
        buf.append(self.serial);
        buf
    }
}

pub struct AuthChallengeServerPacket {
    pub challenge:     [u8; 16],
    pub dos_challenge: [u8; 32],
    pub dos_zero_bits: u8,
}

impl ServerWorldPacket for AuthChallengeServerPacket {
    const INITIAL_CAP: usize = 4 + 32 + 1;
    const OPCODE: OpcodeServer = SMSG::AuthChallenge;

    fn write(&self) -> ByteBuffer {
        let mut buf = Self::init_buffer();
        buf.append(self.dos_challenge);
        buf.append(self.challenge);
        buf.append(self.dos_zero_bits);

        buf
    }
}

#[derive(Clone)]
pub struct AuthSessionClientPacket {
    pub build:             u16,
    pub build_type:        i8,
    pub region_id:         u32,
    pub battlegroup_id:    u32,
    pub realm_id:          u32,
    pub local_challenge:   [u8; 16],
    pub digest:            [u8; 24],
    pub dos_response:      u64,
    pub realm_join_ticket: String,
    pub use_ipv6:          bool,
}

impl ClientWorldPacket for AuthSessionClientPacket {
    const OPCODE: OpcodeClient = CMSG::AuthSession;

    fn read(p: &mut ByteBuffer) -> Result<Self, ByteBufferError> {
        let dos_response = p.read()?;
        let build = p.read()?;
        let build_type = p.read()?;
        let region_id = p.read()?;
        let battlegroup_id = p.read()?;
        let realm_id = p.read()?;
        let local_challenge = p.read()?;
        let digest = p.read()?;
        let use_ipv6 = p.read_bit()?;

        let realm_join_ticket_size: u32 = p.read()?;
        let realm_join_ticket = if realm_join_ticket_size == 0 {
            String::new()
        } else {
            p.read_string_with_length(realm_join_ticket_size as usize)?
        };
        Ok(Self {
            dos_response,
            build,
            build_type,
            region_id,
            battlegroup_id,
            realm_id,
            local_challenge,
            digest,
            use_ipv6,
            realm_join_ticket,
        })
    }
}

pub struct AuthWaitInfo {
    ///< position of the account in the login queue
    pub wait_count: u32,
    ///< Wait time in login queue in minutes, if sent queued and this value is 0 client displays "unknown time"
    pub wait_time:  u32,
    ///< true if the account has a forced character migration pending. @todo implement
    pub has_fcm:    bool,
}

impl AppendBufferBytes<&AuthWaitInfo> for ByteBuffer {
    fn append(&mut self, v: &AuthWaitInfo) {
        self.append(v.wait_count);
        self.append(v.wait_time);
        self.write_bit(v.has_fcm);
        self.flush_bits();
    }
}

pub struct VirtualRealmNameInfo {
    ///< true if the realm is the same as the account's home realm
    pub is_local:              bool,
    ///< @todo research
    pub is_internal_realm:     bool,
    ///< the name of the realm
    pub realm_name_actual:     String,
    ///< the name of the realm without spaces
    pub realm_name_normalised: String,
}

impl AppendBufferBytes<&VirtualRealmNameInfo> for ByteBuffer {
    fn append(&mut self, v: &VirtualRealmNameInfo) {
        self.write_bit(v.is_local);
        self.write_bit(v.is_internal_realm);
        self.write_bits(v.realm_name_actual.len() as u64, 8);
        self.write_bits(v.realm_name_normalised.len() as u64, 8);
        self.flush_bits();

        self.write_string(&v.realm_name_actual);
        self.write_string(&v.realm_name_normalised);
    }
}

pub struct VirtualRealmInfo {
    ///< the virtual address of this realm, constructed as RealmHandle::Region << 24 | RealmHandle::Battlegroup << 16 | RealmHandle::Index
    pub realm_address:   u32,
    pub realm_name_info: VirtualRealmNameInfo,
}

impl AppendBufferBytes<&VirtualRealmInfo> for ByteBuffer {
    fn append(&mut self, v: &VirtualRealmInfo) {
        self.append(v.realm_address);
        self.append(&v.realm_name_info);
    }
}

pub struct BillingInfo {
    pub billing_plan: u32,
    pub time_remain:  u32,
    pub unknown735:   u32,
    pub in_game_room: bool,
}

pub struct AuthSuccessInfo {
    ///< the current expansion of this account, the possible values are in @ref Expansions
    pub account_expansion_level: u8,
    ///< the current server expansion, the possible values are in @ref Expansions
    pub active_expansion_level: u8,
    ///< affects the return value of the GetBillingTimeRested() client API call, it is the number of seconds you have left until the experience points and loot you receive from creatures and quests is reduced. It is only used in the Asia region in retail, it's not implemented in TC and will probably never be.
    pub time_rested: u32,
    ///< a special identifier made from the Index, BattleGroup and Region.
    pub virtual_realm_address: u32,
    ///< @todo research
    pub time_seconds_until_pc_kick: u32,
    ///< this is probably used for the ingame shop. @todo implement
    pub currency_id: u32,
    pub time: i32,

    pub billing: BillingInfo,

    ///< list of realms connected to this one (inclusive) @todo implement
    pub virtual_realms: Vec<VirtualRealmInfo>,

    // ///< list of pre-made character templates. // TODO: Leave out until RBAC_PERM_USE_CHARACTER_TEMPLATES
    // pub templates:      Vec<CharacterTemplate>,
    ///< the minimum AccountExpansion required to select the classes
    pub available_classes: BTreeMap<u8, Expansion>,

    pub is_expansion_trial:       bool,
    ///< forces the client to always use a character template when creating a new character. @see Templates. @todo implement
    pub force_character_template: bool,
    ///< number of horde players in this realm. @todo implement
    pub num_players_horde:        Option<u16>,
    ///< number of alliance players in this realm. @todo implement
    pub num_players_alliance:     Option<u16>,
}

pub struct AuthResponseServerPacket {
    ///< contains the packet data in case that it has account information (It is never set when WaitInfo is set), otherwise its contents are undefined.
    pub success_info: Option<AuthSuccessInfo>,
    ///< contains the queue wait information in case the account is in the login queue.
    pub wait_info:    Option<AuthWaitInfo>,
    ///< the result of the authentication process, possible values are @ref BattlenetRpcErrorCode
    pub result:       u32,
}

impl ServerWorldPacket for AuthResponseServerPacket {
    const INITIAL_CAP: usize = 132;
    const OPCODE: OpcodeServer = SMSG::AuthResponse;

    fn write(&self) -> ByteBuffer {
        let mut buf = Self::init_buffer();

        buf.append(self.result);
        buf.write_bit(self.success_info.is_some());
        buf.write_bit(self.wait_info.is_some());
        buf.flush_bits();

        if let Some(info) = &self.success_info {
            buf.append(info.virtual_realm_address);
            buf.append(info.virtual_realms.len() as u32);
            buf.append(info.time_rested);
            buf.append(info.active_expansion_level);
            buf.append(info.account_expansion_level);
            buf.append(info.time_seconds_until_pc_kick);
            buf.append(info.available_classes.len() as u32);
            buf.append(0u32); // buf.append(info.templates.len() as u32); // TODO: Leave out until RBAC_PERM_USE_CHARACTER_TEMPLATES
            buf.append(info.currency_id);
            buf.append(info.time);
            for (klass, exp) in &info.available_classes {
                buf.append(*klass); // the current class
                buf.append(exp.into_u8()); // the required Expansion
            }
            buf.write_bit(info.is_expansion_trial);
            buf.write_bit(info.force_character_template);
            buf.write_bit(info.num_players_horde.is_some());
            buf.write_bit(info.num_players_alliance.is_some());
            buf.flush_bits();
            buf.append(info.billing.billing_plan);
            buf.append(info.billing.time_remain);
            buf.append(info.billing.unknown735);
            // 3x same bit is not a mistake - preserves legacy client behavior of BillingPlanFlags::SESSION_IGR
            buf.write_bit(info.billing.in_game_room); // inGameRoom check in function checking which lua event to fire when remaining time is near end - BILLING_NAG_DIALOG vs IGR_BILLING_NAG_DIALOG
            buf.write_bit(info.billing.in_game_room); // inGameRoom lua return from Script_GetBillingPlan
            buf.write_bit(info.billing.in_game_room); // not used anywhere in the client
            buf.flush_bits();
            if let Some(h) = info.num_players_horde {
                buf.append(h);
            }
            if let Some(a) = info.num_players_alliance {
                buf.append(a);
            }
            for virtual_realm in &info.virtual_realms {
                buf.append(virtual_realm);
            }
        }

        if let Some(w) = &self.wait_info {
            buf.append(w);
        }

        // TODO: Leave out until RBAC_PERM_USE_CHARACTER_TEMPLATES
        // for (CharacterTemplate const* templat : SuccessInfo->Templates)
        // {
        //     _worldPacket << uint32(templat->TemplateSetId);
        //     _worldPacket << uint32(templat->Classes.size());
        //     for (CharacterTemplateClass const& templateClass : templat->Classes)
        //     {
        //         _worldPacket << uint8(templateClass.ClassID);
        //         _worldPacket << uint8(templateClass.FactionGroup);
        //     }

        //     _worldPacket.WriteBits(templat->Name.length(), 7);
        //     _worldPacket.WriteBits(templat->Description.length(), 10);
        //     _worldPacket.FlushBits();

        //     _worldPacket.WriteString(templat->Name);
        //     _worldPacket.WriteString(templat->Description);
        // }

        buf
    }
}

pub struct EnableEncryptionServerPacket;

impl ServerWorldPacket for EnableEncryptionServerPacket {
    const INITIAL_CAP: usize = 0;
    const OPCODE: OpcodeServer = SMSG::EnableEncryption;

    fn write(&self) -> ByteBuffer {
        Self::init_buffer()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICKET: &str = "1#1";

    /// Client side of CMSG_AUTH_SESSION, in TC AuthSession::Read order.
    fn write_auth_session(use_ipv6: bool, ticket_size: u32, ticket: &[u8]) -> ByteBuffer {
        let mut buf = ByteBuffer::with_capacity(68 + ticket.len());
        buf.append(0x0102_0304_0506_0708u64); // dos_response
        buf.append(26972u16); // build
        buf.append(0i8); // build_type
        buf.append(1u32); // region_id
        buf.append(1u32); // battlegroup_id
        buf.append(1u32); // realm_id
        buf.append(std::array::from_fn::<u8, 16, _>(|i| i as u8)); // local_challenge
        buf.append(std::array::from_fn::<u8, 24, _>(|i| 0xA0 + i as u8)); // digest
        buf.write_bit(use_ipv6);
        buf.append(ticket_size); // append flushes the pending bit byte
        buf.append(ticket);
        buf
    }

    #[test]
    fn it_reads_back_every_auth_session_field() {
        let mut buf = write_auth_session(true, TICKET.len() as u32, TICKET.as_bytes());
        let p = AuthSessionClientPacket::read(&mut buf).unwrap();

        assert_eq!(p.dos_response, 0x0102_0304_0506_0708);
        assert_eq!(p.build, 26972);
        assert_eq!(p.build_type, 0);
        assert_eq!((p.region_id, p.battlegroup_id, p.realm_id), (1, 1, 1));
        assert_eq!(p.local_challenge, std::array::from_fn(|i| i as u8));
        assert_eq!(p.digest, std::array::from_fn(|i| 0xA0 + i as u8));
        assert!(p.use_ipv6);
        assert_eq!(p.realm_join_ticket, TICKET);
    }

    /// Fixed offsets, so a bug shared by write_bit and read_bit can't hide.
    #[test]
    fn it_places_the_ipv6_bit_and_ticket_size_at_fixed_offsets() {
        let bytes: bytes::Bytes = write_auth_session(true, 3, b"1#1").into();
        assert_eq!(bytes.len(), 68 + 3);
        assert_eq!(bytes[63], 0b1000_0000); // use_ipv6 is the high bit
        assert_eq!(&bytes[64..68], &3u32.to_le_bytes());
        assert_eq!(&bytes[68..], b"1#1");
    }

    /// read::<u32> after read_bit must reset the bit position, or the size comes out wrong.
    #[test]
    fn it_reads_the_ticket_size_after_the_bit_without_ipv6() {
        let mut buf = write_auth_session(false, TICKET.len() as u32, TICKET.as_bytes());
        let p = AuthSessionClientPacket::read(&mut buf).unwrap();
        assert!(!p.use_ipv6);
        assert_eq!(p.realm_join_ticket, TICKET);
    }

    #[test]
    fn it_reads_an_empty_ticket_when_size_is_zero() {
        let mut buf = write_auth_session(false, 0, &[]);
        let p = AuthSessionClientPacket::read(&mut buf).unwrap();
        assert_eq!(p.realm_join_ticket, "");
    }

    /// Stricter than TC on purpose: TC cuts the ticket to the bytes left, AZC rejects the packet.
    #[test]
    fn it_rejects_a_ticket_longer_than_the_packet() {
        let mut buf = write_auth_session(false, 10, TICKET.as_bytes());
        assert!(matches!(AuthSessionClientPacket::read(&mut buf), Err(ByteBufferError::Position { .. })));
    }

    #[test]
    fn it_rejects_a_packet_cut_before_the_ticket_size() {
        let full: bytes::Bytes = write_auth_session(false, 3, b"1#1").into();
        let mut buf = ByteBuffer::from(&full[..66]);
        assert!(matches!(AuthSessionClientPacket::read(&mut buf), Err(ByteBufferError::Position { .. })));
    }
}
