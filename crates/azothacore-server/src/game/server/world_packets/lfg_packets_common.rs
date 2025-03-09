use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::game::entities::object::object_guid::{HighGuidPlayer, ObjectGuid};

#[derive(Serialize_repr, Deserialize_repr)]
#[repr(u32)]
pub enum RideType {
    None = 0,
    Battlegrounds = 1,
    Lfg = 2,
}

/// WorldPackets::RideTicket in TC / AC
pub struct LFGRideTicket {
    /// WorldPackets::RideTicket::RequesterGuid in TC
    requester_guid: ObjectGuid<HighGuidPlayer>,
    /// WorldPackets::RideTicket::Id in TC
    queue_id:       u32,
    typ:            RideType,
    /// WorldPackets::RideTicket::Time in TC
    time:           i32,
}
