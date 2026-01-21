
// SPDX-License-Identifier: LGPL-3.0-or-later

use anyhow::Result;
use futures::stream::TryStreamExt;
use rtnetlink::new_connection;
use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct RoutingPolicyRule {
    pub from: Option<String>,
    pub to: Option<String>,
    pub table: u32,
}

pub async fn routing_policy_rule_add(rule: &RoutingPolicyRule) -> Result<()> {
    // Check if we have multiple interfaces
    let links = super::acquire_links().await?;
    if links.links_by_mac.len() < 2 {
        return Ok(());
    }

    // Check if rule already exists
    if rule_exists(rule).await? {
        return Ok(());
    }

    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut rule_request = handle.rule().add();

    // Set source address if specified
    if let Some(ref from) = rule.from {
        let ip: IpAddr = from.parse()?;
        rule_request = rule_request.source_prefix(ip, 32);
    }

    // Set destination address if specified
    if let Some(ref to) = rule.to {
        let ip: IpAddr = to.parse()?;
        rule_request = rule_request.destination_prefix(ip, 32);
    }

    // Set table
    rule_request = rule_request.table(rule.table);

    // Execute
    let result = rule_request.execute().await;

    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let err_str = format!("{}", e);
            if err_str.contains("File exists") || err_str.contains("EEXIST") {
                Ok(())
            } else {
                Err(anyhow::anyhow!("Failed to add routing policy rule: {}", e))
            }
        }
    }
}

pub async fn routing_policy_rule_remove(rule: &RoutingPolicyRule) -> Result<()> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut rule_request = handle.rule().del();

    // Set source address if specified
    if let Some(ref from) = rule.from {
        let ip: IpAddr = from.parse()?;
        rule_request = rule_request.source_prefix(ip, 32);
    }

    // Set destination address if specified
    if let Some(ref to) = rule.to {
        let ip: IpAddr = to.parse()?;
        rule_request = rule_request.destination_prefix(ip, 32);
    }

    // Set table
    rule_request = rule_request.table(rule.table);

    // Execute
    rule_request.execute().await?;

    Ok(())
}

async fn rule_exists(rule: &RoutingPolicyRule) -> Result<bool> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut rule_stream = handle.rule().get(rtnetlink::IpVersion::V4).execute();

    while let Some(rule_msg) = rule_stream.try_next().await? {
        // Check table
        let table = rule_msg.header.table as u32;
        if table != rule.table {
            continue;
        }

        // Extract source and destination prefixes
        let mut src_ip: Option<String> = None;
        let mut dst_ip: Option<String> = None;

        for nla in &rule_msg.nlas {
            match nla {
                netlink_packet_route::rule::nlas::Nla::Source(addr) if addr.len() == 4 => {
                    src_ip = Some(format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]));
                }
                netlink_packet_route::rule::nlas::Nla::Destination(addr) if addr.len() == 4 => {
                    dst_ip = Some(format!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]));
                }
                _ => {}
            }
        }

        // Check if this rule matches
        let from_matches = match (&rule.from, &src_ip) {
            (Some(from), Some(src)) => from == src,
            (None, None) => true,
            _ => false,
        };

        let to_matches = match (&rule.to, &dst_ip) {
            (Some(to), Some(dst)) => to == dst,
            (None, None) => true,
            _ => false,
        };

        if from_matches && to_matches {
            return Ok(true);
        }
    }

    Ok(false)
}
