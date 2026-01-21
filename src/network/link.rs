
// SPDX-License-Identifier: LGPL-3.0-or-later

use anyhow::{anyhow, Context, Result};
use futures::stream::TryStreamExt;
use rtnetlink::{new_connection, Handle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Link {
    pub name: String,
    pub ifindex: u32,
    pub oper_state: String,
    pub mac: String,
    pub mtu: u32,
    pub addresses: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone)]
pub struct Links {
    pub links_by_mac: HashMap<String, Link>,
}

impl Links {
    pub fn new() -> Self {
        Self {
            links_by_mac: HashMap::new(),
        }
    }
}

pub async fn acquire_links() -> Result<Links> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = Links::new();
    let mut link_stream = handle.link().get().execute();

    while let Some(link_msg) = link_stream.try_next().await? {
        let name = link_msg
            .nlas
            .iter()
            .find_map(|nla| {
                if let netlink_packet_route::link::nlas::Nla::IfName(n) = nla {
                    Some(n.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        // Skip loopback
        if name == "lo" {
            continue;
        }

        let mac = link_msg
            .nlas
            .iter()
            .find_map(|nla| {
                if let netlink_packet_route::link::nlas::Nla::Address(addr) = nla {
                    Some(format!(
                        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                        addr[0], addr[1], addr[2], addr[3], addr[4], addr[5]
                    ))
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let mtu = link_msg
            .nlas
            .iter()
            .find_map(|nla| {
                if let netlink_packet_route::link::nlas::Nla::Mtu(m) = nla {
                    Some(*m)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let oper_state = link_msg
            .nlas
            .iter()
            .find_map(|nla| {
                if let netlink_packet_route::link::nlas::Nla::OperState(state) = nla {
                    Some(format!("{:?}", state))
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let link = Link {
            name,
            ifindex: link_msg.header.index,
            oper_state,
            mac: mac.clone(),
            mtu,
            addresses: None,
        };

        links.links_by_mac.insert(mac, link);
    }

    Ok(links)
}

pub async fn get_link_mac_by_index(links: &Links, if_index: u32) -> Result<String> {
    links
        .links_by_mac
        .values()
        .find(|link| link.ifindex == if_index)
        .map(|link| link.mac.clone())
        .ok_or_else(|| anyhow!("not found"))
}

pub async fn get_link_name_by_index(if_index: u32) -> Result<String> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut link_stream = handle.link().get().match_index(if_index).execute();

    if let Some(link_msg) = link_stream.try_next().await? {
        let name = link_msg
            .nlas
            .iter()
            .find_map(|nla| {
                if let netlink_packet_route::link::nlas::Nla::IfName(n) = nla {
                    Some(n.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("Link name not found"))?;

        return Ok(name);
    }

    Err(anyhow!("Link with index {} not found", if_index))
}

pub async fn get_link_index_by_name(name: &str) -> Result<u32> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut link_stream = handle.link().get().match_name(name.to_string()).execute();

    if let Some(link_msg) = link_stream.try_next().await? {
        return Ok(link_msg.header.index);
    }

    Err(anyhow!("Link '{}' not found", name))
}

pub async fn link_set_oper_state_up(if_index: u32) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    handle
        .link()
        .set(if_index)
        .up()
        .execute()
        .await
        .context("Failed to bring link up")?;

    Ok(())
}

pub async fn link_set_mtu(if_index: u32, mtu: u32) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    handle
        .link()
        .set(if_index)
        .mtu(mtu)
        .execute()
        .await
        .context("Failed to set MTU")?;

    Ok(())
}
