
// SPDX-License-Identifier: LGPL-3.0-or-later

mod azure;
mod ec2;
mod gcp;
mod network;
mod watch;

pub use azure::*;
pub use ec2::*;
pub use gcp::*;
pub use network::*;
pub use watch::*;

use crate::cloud::CloudProvider as CloudKind;
use crate::network::{Links, Route, RoutingPolicyRule};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[async_trait::async_trait]
pub trait CloudProvider: Send + Sync {
    async fn fetch_cloud_metadata(&mut self) -> Result<()>;
    async fn configure_network_from_cloud_meta(&self, env: &mut Environment) -> Result<()>;
    async fn save_cloud_metadata(&self) -> Result<()>;
    async fn link_save_cloud_metadata(&self, env: &Environment) -> Result<()>;
}

pub struct Environment {
    pub kind: CloudKind,
    pub provider: Box<dyn CloudProvider>,
    pub links: Links,
    pub route_table: u32,
    pub addresses_by_mac: HashMap<String, HashMap<String, bool>>,
    pub routes_by_index: HashMap<u32, Route>,
    pub routing_rules_by_address_from: HashMap<String, RoutingPolicyRule>,
    pub routing_rules_by_address_to: HashMap<String, RoutingPolicyRule>,
    pub mutex: Arc<Mutex<()>>,
}

impl Environment {
    pub fn new(kind: CloudKind, config: &crate::conf::Config) -> Option<Self> {
        let provider: Box<dyn CloudProvider> = match kind {
            CloudKind::Azure => Box::new(Azure::new(&config.cloud.azure)),
            CloudKind::AWS => Box::new(EC2::new(&config.cloud.aws)),
            CloudKind::GCP => Box::new(GCP::new(&config.cloud.gcp)),
            _ => return None,
        };

        Some(Self {
            kind,
            provider,
            links: Links::new(),
            route_table: config.network.routing.table_base,
            addresses_by_mac: HashMap::new(),
            routes_by_index: HashMap::new(),
            routing_rules_by_address_from: HashMap::new(),
            routing_rules_by_address_to: HashMap::new(),
            mutex: Arc::new(Mutex::new(())),
        })
    }
}

pub async fn acquire_cloud_metadata(env: &mut Environment) -> Result<()> {
    let _lock = env.mutex.lock().unwrap();

    env.links = crate::network::acquire_links().await?;
    env.provider.fetch_cloud_metadata().await?;

    Ok(())
}

pub async fn configure_network_metadata(env: &mut Environment) -> Result<()> {
    let _lock = env.mutex.lock().unwrap();
    env.provider.configure_network_from_cloud_meta(env).await
}

pub async fn save_metadata(env: &Environment) -> Result<()> {
    env.provider.save_cloud_metadata().await?;
    env.provider.link_save_cloud_metadata(env).await?;
    Ok(())
}
