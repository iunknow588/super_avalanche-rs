pub mod export;
pub mod import;
pub mod transfer;

use crate::{errors::Result, jsonrpc::client::x as client_x, key, txs, wallet};

impl<T> wallet::Wallet<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone + Send + Sync,
{
    #[must_use]
    pub fn x(&self) -> X<T> {
        X {
            inner: self.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct X<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone + Send + Sync,
{
    pub inner: crate::wallet::Wallet<T>,
}

impl<T> X<T>
where
    T: key::secp256k1::ReadOnly + key::secp256k1::SignOnly + Clone + Send + Sync,
{
    /// Fetches the current balance of the wallet owner from the specified HTTP endpoint.
    /// 查询指定 endpoint 上 X 链余额。
    ///
    /// # Errors
    /// 查询失败时返回错误。
    ///
    /// # Panics
    ///
    /// Panics if the result is None.
    pub async fn balance_with_endpoint(&self, http_rpc: &str) -> Result<u64> {
        let resp = client_x::get_balance(http_rpc, &self.inner.x_address).await?;
        let cur_balance = resp
            .result
            .expect("unexpected None GetBalanceResult")
            .balance;
        Ok(cur_balance)
    }

    /// Fetches the current balance of the wallet owner from all endpoints
    /// in the same order of "`self.http_rpcs`".
    /// 获取 X 链所有资产余额。
    ///
    /// # Errors
    /// 查询失败时返回错误。
    pub async fn balances(&self) -> Result<Vec<u64>> {
        let mut balances = Vec::new();
        for http_rpc in &self.inner.base_http_urls {
            let balance = self.balance_with_endpoint(http_rpc).await?;
            balances.push(balance);
        }
        Ok(balances)
    }

    /// Fetches the current balance of the wallet owner.
    /// 获取 X 链 AVAX 余额。
    ///
    /// # Errors
    /// 查询失败时返回错误。
    pub async fn balance(&self) -> Result<u64> {
        self.balance_with_endpoint(&self.inner.pick_base_http_url().1)
            .await
    }

    /// Fetches UTXOs for "X" chain.
    /// TODO: cache this like avalanchego
    /// 获取 X 链的 UTXOs。
    ///
    /// # Errors
    /// 获取失败时返回错误。
    pub async fn utxos(&self) -> Result<Vec<txs::utxo::Utxo>> {
        // ref. https://github.com/ava-labs/avalanchego/blob/v1.7.9/wallet/chain/p/builder.go
        // ref. https://github.com/ava-labs/avalanchego/blob/v1.7.9/vms/platformvm/add_validator_tx.go#L263
        // ref. https://github.com/ava-labs/avalanchego/blob/v1.7.9/vms/platformvm/spend.go#L39 "stake"
        // ref. https://github.com/ava-labs/subnet-cli/blob/6bbe9f4aff353b812822af99c08133af35dbc6bd/client/p.go#L355 "AddValidator"
        // ref. https://github.com/ava-labs/subnet-cli/blob/6bbe9f4aff353b812822af99c08133af35dbc6bd/client/p.go#L614 "stake"
        let resp =
            client_x::get_utxos(&self.inner.pick_base_http_url().1, &self.inner.p_address).await?;
        let utxos = resp
            .result
            .ok_or_else(|| crate::errors::Error::UnexpectedNone("GetUtxosResult".to_string()))?
            .utxos
            .ok_or_else(|| {
                crate::errors::Error::UnexpectedNone("Utxos from GetUtxosResult".to_string())
            })?;
        Ok(utxos)
    }

    /// 构建 X 链转账交易。
    #[must_use]
    pub fn transfer(&self) -> transfer::Tx<T> {
        transfer::Tx::new(self)
    }

    /// 构建 X 链跨链导出交易。
    #[must_use]
    pub fn export(&self) -> export::Tx<T> {
        export::Tx::new(self)
    }

    /// 构建 X 链跨链导入交易。
    #[must_use]
    pub fn import(&self) -> import::Tx<T> {
        import::Tx::new(self)
    }
}
