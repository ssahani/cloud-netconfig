
// SPDX-License-Identifier: LGPL-3.0-or-later

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GCP_METADATA_ENDPOINT: &str = "http://metadata.google.internal/computeMetadata/v1/?recursive=true";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCPMetadata {
    pub instance: GCPInstance,
    pub project: GCPProject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GCPInstance {
    pub id: String,
    pub hostname: String,
    pub machine_type: String,
    pub network_interfaces: Vec<GCPNetworkInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCPNetworkInterface {
    pub mac: String,
    pub ip: String,
    pub subnetmask: String,
    pub gateway: String,
    pub mtu: u32,
    #[serde(default)]
    pub ip_aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GCPProject {
    pub project_id: String,
    pub numeric_project_id: i64,
}

pub struct GCP {
    metadata: Option<GCPMetadata>,
    recursive: bool,
}

impl GCP {
    pub fn new(config: &crate::conf::GcpCloudConfig) -> Self {
        Self {
            metadata: None,
            recursive: config.recursive,
        }
    }

    fn parse_ipv4_gateway_from_metadata_by_mac(&self, mac: &str) -> Option<String> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if iface.mac.eq_ignore_ascii_case(mac) {
                    return Some(iface.gateway.clone());
                }
            }
        }
        None
    }

    fn parse_link_mtu_from_metadata_by_mac(&self, mac: &str) -> Option<u32> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if iface.mac.eq_ignore_ascii_case(mac) {
                    return Some(iface.mtu);
                }
            }
        }
        None
    }

    fn parse_ipv4_addresses_from_metadata_by_mac(&self, mac: &str) -> HashMap<String, bool> {
        let mut addresses = HashMap::new();

        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if iface.mac.eq_ignore_ascii_case(mac) {
                    // Convert subnet mask to CIDR prefix
                    let prefix = self.subnet_mask_to_cidr(&iface.subnetmask);

                    // Add primary IP
                    let cidr = format!("{}/{}", iface.ip, prefix);
                    addresses.insert(cidr, true);

                    // Add IP aliases
                    for alias in &iface.ip_aliases {
                        let alias_cidr = format!("{}/{}", alias, prefix);
                        addresses.insert(alias_cidr, true);
                    }
                    break;
                }
            }
        }

        addresses
    }

    fn subnet_mask_to_cidr(&self, mask: &str) -> u8 {
        // Simple conversion from subnet mask to CIDR prefix length
        let parts: Vec<u8> = mask.split('.').filter_map(|s| s.parse().ok()).collect();
        if parts.len() != 4 {
            return 24; // default
        }

        let mut prefix = 0;
        for octet in parts {
            prefix += octet.count_ones();
        }
        prefix as u8
    }
}

#[async_trait::async_trait]
impl super::CloudProvider for GCP {
    async fn fetch_cloud_metadata(&mut self) -> Result<()> {
        let client = reqwest::Client::new();
        let response = client
            .get(GCP_METADATA_ENDPOINT)
            .header("Metadata-Flavor", "Google")
            .send()
            .await?;

        self.metadata = Some(response.json::<GCPMetadata>().await?);
        Ok(())
    }

    async fn configure_network_from_cloud_meta(&self, env: &mut super::Environment) -> Result<()> {
        for (mac, link) in &env.links.links_by_mac {
            let addresses = self.parse_ipv4_addresses_from_metadata_by_mac(mac);
            if !addresses.is_empty() {
                let gateway = self.parse_ipv4_gateway_from_metadata_by_mac(mac);
                let mtu = self.parse_link_mtu_from_metadata_by_mac(mac);

                super::network::configure_network(env, link, addresses, gateway, mtu).await?;
            }
        }
        Ok(())
    }

    async fn save_cloud_metadata(&self) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            let path = format!("{}/gcp", crate::conf::SYSTEM_STATE_DIR);
            crate::system::create_and_save_json(&path, &meta.instance)?;
        }
        Ok(())
    }

    async fn link_save_cloud_metadata(&self, env: &super::Environment) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.instance.network_interfaces {
                if let Some(link) = env.links.links_by_mac.get(&iface.mac) {
                    let path = format!("{}/{}", crate::conf::LINK_STATE_DIR, link.name);
                    crate::system::create_and_save_json(&path, iface)?;
                }
            }
        }
        Ok(())
    }
}
