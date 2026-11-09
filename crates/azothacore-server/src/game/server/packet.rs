use crate::{
    game::server::protocol::opcodes::{OpcodeClient, OpcodeServer},
    shared::packets::{ByteBuffer, ByteBufferError},
};

/// TC: WorldPackets::ServerPacket
pub trait ServerWorldPacket {
    const OPCODE: OpcodeServer;
    const INITIAL_CAP: usize = 200;

    fn init_buffer() -> ByteBuffer {
        ByteBuffer::with_capacity(Self::INITIAL_CAP)
    }

    /// TC: WorldPackets::ServerPacket::Write()
    fn write(&self) -> ByteBuffer;
}

/// TC: WorldPackets::ClientPacket
pub trait ClientWorldPacket: Sized {
    const OPCODE: OpcodeClient;

    /// TC: WorldPackets::ClientPacket::Read()
    fn read(p: &mut ByteBuffer) -> Result<Self, ByteBufferError>;
}
