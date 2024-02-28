use libc::{NLMSG_DONE, NLMSG_ERROR, RTM_NEWADDR};

use crate::{
    netlink::generic_netlink_header::GenericNetlinkMessageHeader,
    netlink::interface_address_message::InterfaceAddressMessage,
    netlink::netlink_attribute::NetlinkAttribute,
    utils::{macros::walk_to_number, walking_vec::WalkingVec},
};

use super::constants::NETLINK_HEADER_SIZE;

pub fn get_attribute(
    attributes: &Vec<NetlinkAttribute>,
    attribute_type: i32,
) -> Option<&NetlinkAttribute> {
    let mut attribute = None;

    for attr in attributes {
        if attr.attribute_type as i32 == attribute_type {
            attribute = Some(attr);
            break;
        }
    }

    attribute
}

pub fn parse_attributes(buffer: &mut WalkingVec) -> Vec<NetlinkAttribute> {
    let mut attributes = Vec::new();

    loop {
        let attribute = NetlinkAttribute::deserialize(buffer);
        attributes.push(attribute);
        if buffer.reached_end() {
            break;
        }
    }

    attributes
}

// https://github.com/torvalds/linux/blob/master/include/uapi/linux/netlink.h#L52
#[derive(Debug)]
pub struct NetlinkMessageHeader {
    pub length: u32,
    pub message_type: u16,
    pub flags: u16,
    pub sequence_number: u32,
    pub pid: u32,
    pub payload: Payload,
}

#[derive(Debug, PartialEq)]
pub enum Payload {
    GenericNetlink(GenericNetlinkMessageHeader),
    RtmGetAddr(InterfaceAddressMessage),
    Done(i32),
    Error(i32),
}

impl Payload {
    pub fn size(&self) -> usize {
        match self {
            Payload::Done(_) | Payload::Error(_) => std::mem::size_of::<i32>(),
            Payload::GenericNetlink(p) => p.size(),
            Payload::RtmGetAddr(p) => p.size(),
        }
    }
}

impl NetlinkMessageHeader {
    pub fn build(message_type: i32, flags: i32, payload: Payload) -> Self {
        // Length with padding
        let length = (NETLINK_HEADER_SIZE + payload.size()) as u32;
        Self {
            length,
            message_type: message_type as u16,
            flags: flags as u16,
            sequence_number: 0,
            pid: std::process::id(),
            payload,
        }
    }

    pub fn serialize(self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.extend(self.length.to_le_bytes());
        buffer.extend(self.message_type.to_le_bytes());
        buffer.extend(self.flags.to_le_bytes());
        buffer.extend(self.sequence_number.to_le_bytes());
        buffer.extend(self.pid.to_le_bytes());
        match &self.payload {
            Payload::GenericNetlink(message) => buffer.extend(message.serialize()),
            Payload::RtmGetAddr(message) => buffer.extend(message.serialize()),
            _ => unimplemented!("This is not needed for now"),
        }

        buffer
    }

    pub fn deserialize(walkable_buffer: &mut WalkingVec) -> Self {
        let length = walk_to_number!(walkable_buffer, u32);
        let message_type = walk_to_number!(walkable_buffer, u16);
        let flags = walk_to_number!(walkable_buffer, u16);
        let sequence_number = walk_to_number!(walkable_buffer, u32);
        let pid = walk_to_number!(walkable_buffer, u32);
        // Limit buffer that can be read by the other message types
        // This is required since the amount of netlink attributes is not known
        // and we don't want to read more than the length of the netlink header
        let mut limited_walking_buffer = WalkingVec {
            buffer: walkable_buffer.buffer[walkable_buffer.position
                ..walkable_buffer.position + length as usize - NETLINK_HEADER_SIZE]
                .to_vec(),
            position: 0,
        };
        // Before trying to parse the payload data
        // We want to check whether the the current message is an ACK message or the last part of
        // a multi message response
        let payload = if message_type as i32 == NLMSG_DONE {
            Payload::Done(walk_to_number!(walkable_buffer, i32))
        } else if message_type as i32 == NLMSG_ERROR {
            Payload::Error(walk_to_number!(walkable_buffer, i32))
        } else if message_type == RTM_NEWADDR {
            Payload::RtmGetAddr(InterfaceAddressMessage::deserialize(
                &mut limited_walking_buffer,
            ))
        } else {
            Payload::GenericNetlink(GenericNetlinkMessageHeader::deserialize(
                &mut limited_walking_buffer,
            ))
        };

        Self {
            length,
            message_type,
            flags,
            sequence_number,
            pid,
            payload,
        }
    }
}
