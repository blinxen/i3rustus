use crate::netlink::NetlinkAttribute;
use crate::utils::macros::walk_to_number;
use crate::utils::walking_vec::WalkingVec;

use crate::netlink::netlink_header::parse_attributes;

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/genetlink.h#L13
#[derive(Debug, PartialEq)]
pub struct GenericNetlinkMessageHeader {
    pub cmd: u8,
    pub version: u8,
    pub reserverd: u16,
    pub attributes: Vec<NetlinkAttribute>,
}

impl GenericNetlinkMessageHeader {
    pub fn build(cmd: i32, attributes: Vec<NetlinkAttribute>) -> Self {
        Self {
            cmd: cmd as u8,
            version: 1,
            reserverd: 0,
            attributes,
        }
    }

    // Calculate the actual size including padding
    pub fn size(&self) -> usize {
        // u8 * 2 =  the number of byte we need to store command, version
        let mut size = (std::mem::size_of::<u8>() * 2) + std::mem::size_of::<u16>();
        for attribute in self.attributes.iter() {
            size += attribute.size();
        }
        size
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.push(self.cmd);
        buffer.push(self.version);
        buffer.extend(self.reserverd.to_le_bytes());
        for attribute in self.attributes.iter() {
            buffer.extend(attribute.serialize());
        }

        buffer
    }

    pub fn deserialize(buffer: &mut WalkingVec) -> Self {
        Self {
            cmd: walk_to_number!(buffer, u8),
            version: walk_to_number!(buffer, u8),
            reserverd: walk_to_number!(buffer, u16),
            attributes: parse_attributes(buffer),
        }
    }
}
