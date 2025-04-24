use std::{ops::Div, str::FromStr};

use avalanche_types::{
    errors::Result,
    jsonrpc::client::{evm as avalanche_sdk_evm, p as avalanche_sdk_p, x as avalanche_sdk_x},
    key, units,
};
use rand::{seq::SliceRandom, thread_rng};

pub struct LoadedKeysWithBalance {
    pub key_infos: Vec<key::secp256k1::Info>,

    pub x_addrs: Vec<String>,
    pub x_balances: Vec<u64>,

    pub p_addrs: Vec<String>,
    pub p_balances: Vec<u64>,

    pub c_addrs: Vec<String>,
    pub c_balances: Vec<primitive_types::U256>,
}

impl LoadedKeysWithBalance {
    pub fn new(key_infos: Vec<key::secp256k1::Info>) -> Self {
        let mut loaded_keys = Self {
            key_infos,
            x_addrs: Vec::new(),
            x_balances: Vec::new(),
            p_addrs: Vec::new(),
            p_balances: Vec::new(),
            c_addrs: Vec::new(),
            c_balances: Vec::new(),
        };

        loaded_keys.x_addrs = loaded_keys
            .key_infos
            .iter()
            .map(|k| k.addresses.get(&network_id).unwrap().x.clone())
            .collect();

        loaded_keys.p_addrs = loaded_keys
            .key_infos
            .iter()
            .map(|k| k.addresses.get(&network_id).unwrap().p.clone())
            .collect();

        loaded_keys.c_addrs = loaded_keys
            .key_infos
            .iter()
            .map(|k| k.eth_address.clone())
            .collect();

        loaded_keys
    }

    pub async fn load_balances(&mut self, network_id: u32, http_rpc: &str) -> io::Result<()> {
        if self.permute_keys {
            self.permute();
        }

        let (x_balances, p_balances, c_balances) = if network_id == 1 {
            get_mainnet_balances(&self.key_infos, http_rpc).await?
        } else {
            get_local_balances(&self.key_infos, http_rpc).await?
        };

        self.x_balances = x_balances;
        self.p_balances = p_balances;
        self.c_balances = c_balances;

        Ok(())
    }

    pub fn permute(&mut self) {
        self.key_infos.shuffle(&mut thread_rng());
        self.x_addrs.shuffle(&mut thread_rng());
        self.p_addrs.shuffle(&mut thread_rng());
        self.c_addrs.shuffle(&mut thread_rng());
    }
}

/// Load the signing hot keys and fetch their balances.
/// TODO: parallelize fetch
pub async fn load_keys_with_balance(
    key_infos: Vec<key::secp256k1::Info>,
    permute_keys: bool,
    network_id: u32,
    http_rpc: &str,
) -> io::Result<LoadedKeysWithBalance> {
    let mut loaded_keys = LoadedKeysWithBalance::new(key_infos);
    if permute_keys {
        loaded_keys.permute();
    }

    let (x_balances, p_balances, c_balances) = if network_id == 1 {
        get_mainnet_balances(&loaded_keys.key_infos, http_rpc).await?
    } else {
        get_local_balances(&loaded_keys.key_infos, http_rpc).await?
    };

    loaded_keys.x_balances = x_balances;
    loaded_keys.p_balances = p_balances;
    loaded_keys.c_balances = c_balances;

    Ok(loaded_keys)
}
