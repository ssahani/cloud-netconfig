
// SPDX-License-Identifier: LGPL-3.0-or-later

use anyhow::{anyhow, Result};
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct Route {
    pub table: u32,
    pub if_index: u32,
    pub gw: String,
}

pub async fn get_default_ipv4_gateway() -> Result<String> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut route_stream = handle.route().get(rtnetlink::IpVersion::V4).execute();

    while let Some(route_msg) = route_stream.try_next().await? {
        // Look for default route (0.0.0.0/0)
        let is_default = route_msg.header.destination_prefix_length == 0;

        if is_default {
            if let Some(gw) = route_msg.nlas.iter().find_map(|nla| {
                if let netlink_packet_route::route::nlas::Nla::Gateway(addr) = nla {
                    if addr.len() == 4 {
                        Some(format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }) {
                return Ok(gw);
            }
        }
    }

    Err(anyhow!("Default gateway not found"))
}

pub async fn get_default_ipv4_gateway_by_link(if_index: u32) -> Result<String> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut route_stream = handle.route().get(rtnetlink::IpVersion::V4).execute();

    while let Some(route_msg) = route_stream.try_next().await? {
        // Check if this is a default route for the specific link
        let is_default = route_msg.header.destination_prefix_length == 0;

        let matches_link = route_msg.nlas.iter().any(|nla| {
            if let netlink_packet_route::route::nlas::Nla::Oif(index) = nla {
                *index == if_index
            } else {
                false
            }
        });

        if is_default && matches_link {
            if let Some(gw) = route_msg.nlas.iter().find_map(|nla| {
                if let netlink_packet_route::route::nlas::Nla::Gateway(addr) = nla {
                    if addr.len() == 4 {
                        Some(format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }) {
                return Ok(gw);
            }
        }
    }

    Err(anyhow!("Default gateway not found for link {}", if_index))
}

pub async fn get_ipv4_gateway_by_link(if_index: u32) -> Result<String> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut route_stream = handle.route().get(rtnetlink::IpVersion::V4).execute();

    while let Some(route_msg) = route_stream.try_next().await? {
        let matches_link = route_msg.nlas.iter().any(|nla| {
            if let netlink_packet_route::route::nlas::Nla::Oif(index) = nla {
                *index == if_index
            } else {
                false
            }
        });

        if matches_link {
            if let Some(gw) = route_msg.nlas.iter().find_map(|nla| {
                if let netlink_packet_route::route::nlas::Nla::Gateway(addr) = nla {
                    if addr.len() == 4 {
                        Some(format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }) {
                return Ok(gw);
            }
        }
    }

    Err(anyhow!("Gateway not found for link {}", if_index))
}

pub async fn route_add(route: &Route) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let gw: Ipv4Addr = route.gw.parse()?;

    let result = handle
        .route()
        .add()
        .v4()
        .gateway(gw)
        .output_interface(route.if_index)
        .table(route.table)
        .execute()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("File exists") || err_str.contains("EEXIST") {
                // Route already exists, this is okay
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add route: {}", e))
            }
        }
    }
}

pub async fn route_remove(route: &Route) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let gw: Ipv4Addr = route.gw.parse()?;

    handle
        .route()
        .del()
        .v4()
        .gateway(gw)
        .output_interface(route.if_index)
        .table(route.table)
        .execute()
        .await?;

    Ok(())
}
