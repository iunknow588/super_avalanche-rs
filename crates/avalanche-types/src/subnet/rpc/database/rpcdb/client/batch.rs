//! Database Batch management implementation for rpcdb client.
use crate::{
    proto::rpcdb::{self, database_client::DatabaseClient},
    subnet::rpc::{
        database::{
            self,
            batch::{CAPACITY_REDUCTION_FACTOR, MAX_EXCESS_CAPACITY_FACTOR},
            BoxedDatabase,
        },
        errors,
    },
};
use std::{
    collections::HashSet,
    io::{Error, ErrorKind, Result},
    sync::Arc,
};

use bytes::Bytes;
use tokio::sync::{Mutex, RwLock};
use tonic::transport::Channel;

pub const BASE_ELEMENT_SIZE: usize = 8;

/// Key-value pair with delete flag
struct KeyValue {
    /// The key
    key: Vec<u8>,
    /// The value
    value: Vec<u8>,
    /// Whether this is a delete operation
    delete: bool,
}

/// Batch is a write-only database that commits changes to its host database
/// when Write is called. Although batch is currently async and thread safe it
/// should not be used concurrently.
#[derive(Clone)]
pub struct Batch {
    /// The database client
    db: DatabaseClient<Channel>,
    /// The pending writes
    writes: Arc<RwLock<Vec<KeyValue>>>,
    /// The total size of all keys and values
    size: usize,
}

impl Batch {
    pub fn new(db: DatabaseClient<Channel>) -> Self {
        Self {
            db,
            writes: Arc::new(RwLock::new(Vec::new())),
            size: 0,
        }
    }
}

#[tonic::async_trait]
impl database::batch::Batch for Batch {
    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn put(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.writes.write().await.push(KeyValue {
            key: key.to_owned(),
            value: value.to_owned(),
            delete: false,
        });
        self.size += key.len() + value.len();
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.writes.write().await.push(KeyValue {
            key: key.to_owned(),
            value: vec![],
            delete: true,
        });
        self.size += key.len();
        Ok(())
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn size(&self) -> Result<usize> {
        Ok(self.size)
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn write(&self) -> Result<()> {
        let mut req = rpcdb::WriteBatchRequest {
            puts: vec![],
            deletes: vec![],
        };
        let writes = self.writes.read().await;
        let mut key_set: HashSet<Vec<u8>> = HashSet::with_capacity(writes.len());

        // Use the database client
        for kv in writes.iter() {
            // continue if the key already existed
            if key_set.contains(&kv.key) {
                continue;
            }
            key_set.insert(kv.key.clone());

            if kv.delete {
                req.deletes.push(rpcdb::DeleteRequest {
                    key: Bytes::from(kv.key.clone()),
                });
            } else {
                req.puts.push(rpcdb::PutRequest {
                    key: Bytes::from(kv.key.clone()),
                    value: Bytes::from(kv.value.clone()),
                });
            }
        }

        let resp = self
            .db
            .clone()
            .write_batch(req)
            .await
            .map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("batch write request failed: {e:?}"),
                )
            })?
            .into_inner();

        errors::from_i32(resp.err)
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn reset(&mut self) {
        let mut writes = self.writes.write().await;
        if writes.capacity() > writes.len() * MAX_EXCESS_CAPACITY_FACTOR {
            let kv: Vec<KeyValue> =
                Vec::with_capacity(writes.capacity() / CAPACITY_REDUCTION_FACTOR);
            *writes = kv;
        } else {
            writes.clear();
        }
        drop(writes);
        self.size = 0;
    }

    /// Implements the [`crate::subnet::rpc::database::batch::Batch`] trait.
    async fn replay(&self, k: Arc<Mutex<BoxedDatabase>>) -> Result<()> {
        let writes = self.writes.read().await;
        let mut db = k.lock().await;
        // Drop db and writes after the function completes

        for kv in writes.iter() {
            if kv.delete {
                db.delete(&kv.key).await.map_err(|e| {
                    Error::new(ErrorKind::Other, format!("replay delete failed: {e:?}"))
                })?;
            } else {
                db.put(&kv.key, &kv.value).await.map_err(|e| {
                    Error::new(ErrorKind::Other, format!("replay put failed: {e:?}"))
                })?;
            }
        }

        Ok(())
    }
}
