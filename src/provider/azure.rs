
// SPDX-License-Identifier: LGPL-3.0-or-later

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const AZURE_METADATA_BASE: &str = "http://169.254.169.254/metadata/instance";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureMetadata {
    pub compute: AzureCompute,
    pub network: AzureNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureCompute {
    pub name: String,
    pub location: String,
    pub vm_id: String,
    pub vm_size: String,
    #[serde(default)]
    pub zone: String,
    #[serde(default)]
    pub subscription_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureNetwork {
    pub interface: Vec<AzureInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureInterface {
    pub mac_address: String,
    pub ipv4: AzureIpv4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureIpv4 {
    pub ip_address: Vec<AzureIpAddress>,
    pub subnet: Vec<AzureSubnet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureIpAddress {
    pub private_ip_address: String,
    pub public_ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSubnet {
    pub address: String,
    pub prefix: String,
}

pub struct Azure {
    metadata: Option<AzureMetadata>,
    api_version: String,
}

impl Azure {
    pub fn new(config: &crate::conf::AzureCloudConfig) -> Self {
        Self {
            metadata: None,
            api_version: config.api_version.clone(),
        }
    }

    fn parse_ipv4_addresses_from_metadata_by_mac(&self, mac: &str) -> HashMap<String, bool> {
        let mut addresses = HashMap::new();

        if let Some(ref meta) = self.metadata {
            for iface in &meta.network.interface {
                if iface.mac_address.eq_ignore_ascii_case(mac) {
                    if let Some(subnet) = iface.ipv4.subnet.first() {
                        let prefix = &subnet.prefix;
                        for ip_addr in &iface.ipv4.ip_address {
                            let cidr = format!("{}/{}", ip_addr.private_ip_address, prefix);
                            addresses.insert(cidr, true);
                        }
                    }
                    break;
                }
            }
        }

        addresses
    }
}

#[async_trait::async_trait]
impl super::CloudProvider for Azure {
    async fn fetch_cloud_metadata(&mut self) -> Result<()> {
        let url = format!("{}?api-version={}", AZURE_METADATA_BASE, self.api_version);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("Metadata", "true")
            .send()
            .await?;

        self.metadata = Some(response.json::<AzureMetadata>().await?);
        Ok(())
    }

    async fn configure_network_from_cloud_meta(&self, env: &mut super::Environment) -> Result<()> {
        for (mac, link) in &env.links.links_by_mac {
            let addresses = self.parse_ipv4_addresses_from_metadata_by_mac(mac);
            if !addresses.is_empty() {
                super::network::configure_network(env, link, addresses, None, None).await?;
            }
        }
        Ok(())
    }

    async fn save_cloud_metadata(&self) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            let path = format!("{}/azure", crate::conf::SYSTEM_STATE_DIR);
            crate::system::create_and_save_json(&path, &meta.compute)?;
        }
        Ok(())
    }

    async fn link_save_cloud_metadata(&self, env: &super::Environment) -> Result<()> {
        if let Some(ref meta) = self.metadata {
            for iface in &meta.network.interface {
                if let Some(link) = env.links.links_by_mac.get(&iface.mac_address) {
                    let path = format!("{}/{}", crate::conf::LINK_STATE_DIR, link.name);
                    crate::system::create_and_save_json(&path, iface)?;
                }
            }
        }
        Ok(())
    }
}
