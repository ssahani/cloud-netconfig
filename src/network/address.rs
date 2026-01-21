
// SPDX-License-Identifier: LGPL-3.0-or-later

use anyhow::{Context, Result};
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use std::collections::HashMap;
use std::net::IpAddr;

pub async fn address_add(if_index: u32, address: &str) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    // Parse the CIDR notation
    let parts: Vec<&str> = address.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid address format, expected CIDR notation"));
    }

    let ip: IpAddr = parts[0].parse()?;
    let prefix_len: u8 = parts[1].parse()?;

    // Add address - ignore if it already exists
    let result = handle
        .address()
        .add(if_index, ip, prefix_len)
        .execute()
        .await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("File exists") || err_str.contains("EEXIST") {
                // Address already exists, this is okay
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add address: {}", e))
            }
        }
    }
}

pub async fn address_set(name: &str, address: &str) -> Result<()> {
    let if_index = super::get_link_index_by_name(name).await?;

    // Parse the CIDR notation
    let parts: Vec<&str> = address.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid address format, expected CIDR notation"));
    }

    let ip: IpAddr = parts[0].parse()?;
    let prefix_len: u8 = parts[1].parse()?;

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    // Set replaces existing address
    handle
        .address()
        .add(if_index, ip, prefix_len)
        .replace()
        .execute()
        .await
        .ok(); // Silently ignore errors

    Ok(())
}

pub async fn get_ipv4_addresses(if_name: &str) -> Result<HashMap<String, bool>> {
    let if_index = super::get_link_index_by_name(if_name).await?;

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut addresses = HashMap::new();
    let mut addr_stream = handle.address().get().set_link_index_filter(if_index).execute();

    while let Some(addr_msg) = addr_stream.try_next().await? {
        // Check if it's IPv4
        if addr_msg.header.family == 2 {
            // AF_INET = 2
            let ip = addr_msg
                .nlas
                .iter()
                .find_map(|nla| {
                    if let netlink_packet_route::address::nlas::Nla::Address(addr) = nla {
                        if addr.len() == 4 {
                            Some(format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            if !ip.is_empty() {
                let cidr = format!("{}/{}", ip, addr_msg.header.prefix_len);
                addresses.insert(cidr, true);
            }
        }
    }

    Ok(addresses)
}

pub async fn address_remove(if_name: &str, address: &str) -> Result<()> {
    let if_index = super::get_link_index_by_name(if_name).await?;

    // Parse the CIDR notation
    let parts: Vec<&str> = address.split('/').collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Invalid address format, expected CIDR notation"));
    }

    let ip: IpAddr = parts[0].parse()?;
    let prefix_len: u8 = parts[1].parse()?;

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    // Silently ignore errors
    handle
        .address()
        .del(if_index, ip, prefix_len)
        .execute()
        .await
        .ok();

    Ok(())
}
