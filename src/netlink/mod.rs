mod constants;
mod generic_netlink_header;
mod interface_address_message;
mod netlink_attribute;
mod netlink_header;

use libc::{
    bind, c_void, connect, recv, sa_family_t, send, sockaddr, sockaddr_nl, socket, socklen_t,
    AF_NETLINK, AF_UNSPEC, CTRL_ATTR_FAMILY_ID, CTRL_ATTR_FAMILY_NAME, CTRL_CMD_GETFAMILY,
    GENL_ID_CTRL, IFA_LOCAL, NETLINK_GENERIC, NETLINK_ROUTE, NLMSG_DONE, NLMSG_ERROR, NLM_F_ACK,
    NLM_F_DUMP, NLM_F_REQUEST, RTM_GETADDR, RT_SCOPE_UNIVERSE, SOCK_RAW,
};
use std::ffi::CString;
use std::io::{Error as IOError, ErrorKind};
use std::os::unix::io::RawFd;

use crate::netlink::constants::*;
use crate::netlink::generic_netlink_header::GenericNetlinkMessageHeader;
use crate::netlink::interface_address_message::InterfaceAddressMessage;
use crate::netlink::netlink_attribute::NetlinkAttribute;
use crate::netlink::netlink_header::{NetlinkMessageHeader, Payload};
use crate::utils::walking_vec::WalkingVec;

// This is the maximum length that a netlink message can have
// https://git.kernel.org/pub/scm/linux/kernel/git/netdev/net-next.git/commit/?id=d35c99ff77ecb2eb239731b799386f3b3637a31e
const MAX_NETLINK_MESSAGE_SIZE: usize = 32768;
const WIRELESS_SUBSYSTEM_NAME: &str = "nl80211\0";

pub struct InterfaceInformation {
    pub ssid: String,
    pub ip: String,
    pub frequency: f32,
    pub bitrate: u16,
}

pub struct BSSInformation {
    pub ssid: String,
    pub frequency: f32,
}

#[derive(Debug)]
pub struct Netlink {
    generic_netlink_socket: RawFd,
    netlink_route_socket: RawFd,
    nl_80211_family_id: Result<i32, IOError>,
}

impl Netlink {
    // Create a generic netlink socket
    pub fn new() -> Result<Self, IOError> {
        // This socket is used to retrieve: SSID, Frequency, bitrate
        let generic_netlink_socket = unsafe { socket(AF_NETLINK, SOCK_RAW, NETLINK_GENERIC) };
        // This socket is used to retrieve: Local IP Address
        let netlink_route_socket = unsafe { socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE) };

        if generic_netlink_socket < 0 || netlink_route_socket < 0 {
            return Err(IOError::last_os_error());
        }
        let (address_ptr, address_length) = Self::socket_address();

        if unsafe { bind(generic_netlink_socket, address_ptr, address_length) } < 0
            || unsafe { bind(netlink_route_socket, address_ptr, address_length) } < 0
        {
            return Err(IOError::last_os_error());
        }

        if unsafe { connect(generic_netlink_socket, address_ptr, address_length) } < 0
            || unsafe { connect(netlink_route_socket, address_ptr, address_length) } < 0
        {
            return Err(IOError::last_os_error());
        }
        let nl_80211_family_id = Self::get_80211_family_id(generic_netlink_socket);

        Ok(Self {
            generic_netlink_socket,
            netlink_route_socket,
            nl_80211_family_id,
        })
    }

    // Create a netlink socket address
    fn socket_address() -> (*const sockaddr, socklen_t) {
        let mut address: sockaddr_nl = unsafe { std::mem::zeroed() };
        address.nl_family = AF_NETLINK as sa_family_t;
        // The destination is the kernel, so we don't
        // really need to set this to something usefull
        address.nl_pid = 0;
        // Unicast
        address.nl_groups = 0;
        (
            &address as *const sockaddr_nl as *const sockaddr,
            std::mem::size_of::<sockaddr>() as socklen_t,
        )
    }

    // Retrive the subsystem family ID for nl80211
    fn get_80211_family_id(socket: RawFd) -> Result<i32, IOError> {
        let mut family_id = i32::MIN;

        let genl_header = GenericNetlinkMessageHeader::build(
            CTRL_CMD_GETFAMILY,
            vec![NetlinkAttribute::build(
                CTRL_ATTR_FAMILY_NAME,
                WIRELESS_SUBSYSTEM_NAME.as_bytes().to_vec(),
            )],
        );
        let response = Self::request(
            socket,
            GENL_ID_CTRL,
            NLM_F_REQUEST | NLM_F_ACK,
            Payload::GenericNetlink(genl_header),
        )?;

        if let Payload::GenericNetlink(message) = &response[0].payload {
            let family_id_attribute =
                netlink_header::get_attribute(&message.attributes, CTRL_ATTR_FAMILY_ID);
            if let Some(family_id_attribute) = family_id_attribute {
                family_id =
                    u16::from_le_bytes(family_id_attribute.data.clone().try_into().unwrap()) as i32;
            }
        }

        if family_id == i32::MIN {
            Err(IOError::new(
                ErrorKind::Other,
                "Could not retrieve nl80211 family ID",
            ))
        } else {
            Ok(family_id)
        }
    }

    fn request(
        socket: RawFd,
        netlink_message_type: i32,
        flags: i32,
        payload: Payload,
    ) -> Result<Vec<NetlinkMessageHeader>, IOError> {
        let mut result_buffer = Vec::new();
        // Create netlink header
        let input_buffer =
            NetlinkMessageHeader::build(netlink_message_type, flags, payload).serialize();
        // Send and receive answer from socket
        unsafe {
            send(
                socket,
                input_buffer.as_ptr() as *const c_void,
                input_buffer.len(),
                0,
            );
            loop {
                // Temporary buffer that will hold the current response
                let mut buffer = vec![0; MAX_NETLINK_MESSAGE_SIZE];
                let mut bytes_read: u32 = 0;
                let response_size = recv(
                    socket,
                    buffer.as_mut_ptr() as *mut c_void,
                    MAX_NETLINK_MESSAGE_SIZE,
                    0,
                );

                // Our input buffer is initialized with the maximum netlink message size
                // So we truncate the result to only contain the actual response bytes
                buffer.truncate(response_size as usize);
                // Create a vector which we can walk through
                // The idea here is that we don't want to manipulate the original response
                // vector, since this would require to allocate vectors that are not needed
                let mut walkable_buffer = WalkingVec {
                    buffer,
                    position: 0,
                };
                // In a single response, there could be multiple netlink message headers
                loop {
                    // Break out of loop if we have finished reading all bytebytes
                    if bytes_read == response_size as u32 {
                        break;
                    }
                    let header = NetlinkMessageHeader::deserialize(&mut walkable_buffer);
                    // Set original walking vec position to the length of the header length
                    // This is required because we create a new walking vec in
                    // NetlinkMessageHeader::deserialize to make sure that deserialization
                    // stops after we reached the end of the header
                    // And the reason deserialization does not "just" stop from itself, is because
                    // we look for netlink attributes recursively and only stop at the end of
                    // the passed walking vec
                    // We could simply pass the header length to all other deserialization methods
                    // and let them do the checking but this is cleaner in my (humble) opinion
                    walkable_buffer.position += (header.length as usize) - NETLINK_HEADER_SIZE;
                    bytes_read += header.length;
                    result_buffer.push(header);
                }
                if let Some(last_header) = result_buffer.last() {
                    // Error + error code 0 is the ACK messag
                    if last_header.message_type as i32 == NLMSG_DONE
                        || (last_header.message_type as i32 == NLMSG_ERROR)
                            && last_header.payload == Payload::Error(0)
                    {
                        break;
                    } else if last_header.message_type as i32 == NLMSG_ERROR
                        && last_header.payload != Payload::Error(0)
                    {
                        log::error!(
                            "Error occured with netlink request of type {}",
                            netlink_message_type
                        );
                        result_buffer.clear();
                        break;
                    }
                }
            }
        }

        if result_buffer.is_empty() {
            Err(IOError::new(
                ErrorKind::Other,
                "No netlink response could be found",
            ))
        } else {
            Ok(result_buffer)
        }
    }

    fn get_interface_index(&self, interface_name: &str) -> Result<u32, IOError> {
        let interface_index = unsafe {
            let if_name = CString::new(interface_name).unwrap();
            libc::if_nametoindex(if_name.as_ptr() as *const libc::c_char)
        };

        if interface_index == 0 {
            Err(IOError::new(
                ErrorKind::Other,
                "Could not retrieve interface index",
            ))
        } else {
            Ok(interface_index)
        }
    }

    fn interface_bss_information(&self, interface_name: &str) -> Result<BSSInformation, IOError> {
        let interface_index = self.get_interface_index(interface_name)?;
        let mut bss = BSSInformation {
            ssid: String::new(),
            frequency: 0.0,
        };

        let genl_header = GenericNetlinkMessageHeader::build(
            NL80211_CMD_GET_SCAN,
            vec![NetlinkAttribute::build(
                NL80211_ATTR_IFINDEX,
                interface_index.to_le_bytes().to_vec(),
            )],
        );

        if let Ok(nl_80211_family_id) = self.nl_80211_family_id.as_ref() {
            let response = Self::request(
                self.generic_netlink_socket,
                *nl_80211_family_id,
                NLM_F_REQUEST | NLM_F_DUMP | NLM_F_ACK,
                Payload::GenericNetlink(genl_header),
            )?;

            // Iterate over all received BSS results
            for message in response.iter() {
                if let Payload::GenericNetlink(message) = &message.payload {
                    // Search for a BSS attribute
                    let bss_attribute =
                        netlink_header::get_attribute(&message.attributes, NL80211_ATTR_BSS);
                    if bss_attribute.is_none() {
                        // We did not find a BSS attribute--> ignore this message
                        continue;
                    }
                    // Parse the nested attributes in the BSS attribute
                    let bss_attributes = netlink_header::parse_attributes(&mut WalkingVec {
                        buffer: bss_attribute.unwrap().data.to_owned(),
                        position: 0,
                    });
                    if bss_attributes.is_empty() {
                        // No attributes found --> ignore this message
                        continue;
                    }

                    let bss_status =
                        netlink_header::get_attribute(&bss_attributes, NL80211_BSS_STATUS);
                    if let Some(bss_status) = bss_status {
                        let status =
                            u32::from_le_bytes(bss_status.data.clone().try_into().unwrap());
                        if status != NL80211_BSS_STATUS_ASSOCIATED
                            && status != NL80211_BSS_STATUS_IBSS_JOINED
                        {
                            // We are not connected to this BSS --> ignore this message
                            continue;
                        }
                    } else {
                        // No status could be found --> ignore this message
                        continue;
                    }

                    let bss_information_elements = netlink_header::get_attribute(
                        &bss_attributes,
                        NL80211_BSS_INFORMATION_ELEMENTS,
                    );
                    if let Some(bss_information_elements) = bss_information_elements {
                        // Based on https://github.com/i3/i3status/blob/main/src/print_wireless_info.c#L141
                        let mut ies = bss_information_elements.data.to_owned();
                        while ies.len() > 2 && ies[0] != 0 {
                            ies = ies[(ies[1] as usize + 2)..].to_owned();
                        }

                        if ies.len() < 2 || ies.len() < ies[1] as usize + 2 {
                            break;
                        };

                        let ssid_len = ies[1] as usize;
                        let ssid_bytes = &ies[2..][..ssid_len];

                        bss.ssid = String::from_utf8_lossy(ssid_bytes).into_owned();
                    }

                    let bss_freq =
                        netlink_header::get_attribute(&bss_attributes, NL80211_BSS_FREQUENCY);
                    if let Some(bss_freq) = bss_freq {
                        // Frequency is in megahertz, but we want it in gigahertz
                        bss.frequency =
                            u32::from_le_bytes(bss_freq.data.clone().try_into().unwrap()) as f32
                                / 1000.0;
                    }

                    // We found the Access Point that we are connected to
                    if !bss.ssid.is_empty() {
                        break;
                    }
                }
            }
        }

        // We only care about the SSID here
        // If frequency is not set correctly, then we don't care
        if bss.ssid.is_empty() {
            Err(IOError::new(ErrorKind::Other, "Could not determine SSID"))
        } else {
            Ok(bss)
        }
    }

    fn interface_ip(&self, interface_name: &str) -> Result<String, IOError> {
        let mut ip = String::new();
        let interface_index = self.get_interface_index(interface_name)?;

        let message = InterfaceAddressMessage::build(
            AF_UNSPEC as u8,
            0,
            0,
            RT_SCOPE_UNIVERSE,
            interface_index,
            Vec::new(),
        );

        let response = Self::request(
            self.netlink_route_socket,
            RTM_GETADDR as i32,
            NLM_F_REQUEST | NLM_F_DUMP,
            Payload::RtmGetAddr(message),
        )?;

        for message in response.iter() {
            if let Payload::RtmGetAddr(message) = &message.payload {
                // Only read messages that contain information about the specified interface
                if message.index == interface_index {
                    let interface_address_local =
                        netlink_header::get_attribute(&message.attributes, IFA_LOCAL as i32);
                    if let Some(interface_address_local) = interface_address_local {
                        ip = format!(
                            "{}.{}.{}.{}",
                            interface_address_local.data[0],
                            interface_address_local.data[1],
                            interface_address_local.data[2],
                            interface_address_local.data[3]
                        );
                    }
                }
            }
        }

        if ip.is_empty() {
            Err(IOError::new(
                ErrorKind::Other,
                "Could not retrieve interface IP",
            ))
        } else {
            Ok(ip)
        }
    }

    fn interface_tx_rate(&self, interface_name: &str) -> Result<u16, IOError> {
        let mut bitrate = 0;
        let interface_index = self.get_interface_index(interface_name)?;

        let genl_header = GenericNetlinkMessageHeader::build(
            NL80211_CMD_GET_STATION,
            vec![NetlinkAttribute::build(
                NL80211_ATTR_IFINDEX,
                interface_index.to_le_bytes().to_vec(),
            )],
        );

        if let Ok(nl_80211_family_id) = self.nl_80211_family_id.as_ref() {
            let response = Self::request(
                self.generic_netlink_socket,
                *nl_80211_family_id,
                NLM_F_REQUEST | NLM_F_DUMP | NLM_F_ACK,
                Payload::GenericNetlink(genl_header),
            )?;

            if let Payload::GenericNetlink(message) = &response[0].payload {
                let station_info =
                    netlink_header::get_attribute(&message.attributes, NL80211_ATTR_STA_INFO);
                if let Some(station_info) = station_info {
                    // Parse the nested attributes
                    let station_info_attributes =
                        netlink_header::parse_attributes(&mut WalkingVec {
                            buffer: station_info.data.to_owned(),
                            position: 0,
                        });
                    if let Some(rate_info) = netlink_header::get_attribute(
                        &station_info_attributes,
                        NL80211_STA_INFO_TX_BITRATE,
                    ) {
                        let rate_info_attributes =
                            netlink_header::parse_attributes(&mut WalkingVec {
                                buffer: rate_info.data.to_owned(),
                                position: 0,
                            });
                        if let Some(raw_bitrate) = netlink_header::get_attribute(
                            &rate_info_attributes,
                            NL80211_RATE_INFO_BITRATE,
                        ) {
                            bitrate =
                                u16::from_le_bytes(raw_bitrate.data.clone().try_into().unwrap())
                                    as u32
                                    * 100
                                    / 1000;
                        }
                    }
                }
            }
        }

        Ok(bitrate as u16)
    }

    pub fn interface_information(
        &self,
        interface_name: &str,
    ) -> Result<InterfaceInformation, IOError> {
        let bss = self.interface_bss_information(interface_name)?;
        Ok(InterfaceInformation {
            ssid: bss.ssid,
            frequency: bss.frequency,
            ip: self.interface_ip(interface_name)?,
            bitrate: self.interface_tx_rate(interface_name)?,
        })
    }
}
