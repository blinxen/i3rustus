use crate::utils::{macros::walk_to_number, walking_vec::WalkingVec};

pub fn align_message(length: usize) -> usize {
    // Align message length to 4 bytes
    (length + 4 - 1) & !(4 - 1)
}

// https://github.com/torvalds/linux/blob/master/tools/include/uapi/linux/netlink.h#L211
#[derive(Debug, PartialEq)]
pub struct NetlinkAttribute {
    pub length: u16,
    pub attribute_type: u16,
    pub data: Vec<u8>,
}

impl NetlinkAttribute {
    pub fn build(attribute_type: i32, data: Vec<u8>) -> Self {
        Self {
            // u16 * 2 =  the number of byte we need to store length and attribute_type
            length: ((std::mem::size_of::<u16>() * 2) + data.len()) as u16,
            attribute_type: attribute_type as u16,
            data,
        }
    }

    // Calculate the actual size including padding
    pub fn size(&self) -> usize {
        align_message(self.length as usize)
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buffer = Vec::new();

        buffer.extend(self.length.to_le_bytes());
        buffer.extend(self.attribute_type.to_le_bytes());
        buffer.extend(self.data.iter());
        buffer.resize(self.size(), 0);

        buffer
    }

    pub fn deserialize(buffer: &mut WalkingVec) -> Self {
        let length = walk_to_number!(buffer, u16);
        let attribute_type = walk_to_number!(buffer, u16);
        // We calculate the data length by:
        //
        // 1. Align the NetlinkAttribute length
        // 2. Subtract the size of length and attribute_type
        //
        // This should give us the actual size of the data that this attribute contains
        let data = buffer
            .walk((length as usize) - (std::mem::size_of::<u16>() * 2))
            .to_vec();
        // Remove padding
        buffer.walk(align_message(length as usize) - (length as usize));
        Self {
            length,
            attribute_type,
            data,
        }
    }
}
