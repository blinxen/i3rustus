use crate::netlink::netlink_header::parse_attributes;
use crate::{
    netlink::NetlinkAttribute,
    utils::{macros::walk_to_number, walking_vec::WalkingVec},
};

// https://elixir.bootlin.com/linux/latest/source/include/uapi/linux/if_addr.h#L8
#[derive(Debug, PartialEq)]
pub struct InterfaceAddressMessage {
    pub family: u8,
    pub prefix_length: u8,
    pub flags: u8,
    pub scope: u8,
    pub index: u32,
    pub attributes: Vec<NetlinkAttribute>,
}
impl InterfaceAddressMessage {
    pub fn build(
        family: u8,
        prefix_length: u8,
        flags: u8,
        scope: u8,
        index: u32,
        attributes: Vec<NetlinkAttribute>,
    ) -> Self {
        Self {
            family,
            prefix_length,
            flags,
            scope,
            index,
            attributes,
        }
    }

    // Calculate the actual size
    pub fn size(&self) -> usize {
        // u8 * 4 =  the number of byte we need to store family, prefix_length, flags, scope
        let mut size = (std::mem::size_of::<u8>() * 4) + std::mem::size_of::<u32>();
        for attribute in self.attributes.iter() {
            size += attribute.size();
        }
        size
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = vec![self.family, self.prefix_length, self.flags, self.scope];
        buffer.extend(self.index.to_le_bytes());
        for attribute in self.attributes.iter() {
            buffer.extend(attribute.serialize());
        }

        buffer
    }

    pub fn deserialize(buffer: &mut WalkingVec) -> Self {
        let family = walk_to_number!(buffer, u8);
        let prefix_length = walk_to_number!(buffer, u8);
        let flags = walk_to_number!(buffer, u8);
        let scope = walk_to_number!(buffer, u8);
        let index = walk_to_number!(buffer, u32);

        Self {
            family,
            prefix_length,
            flags,
            scope,
            index,
            attributes: parse_attributes(buffer),
        }
    }
}
