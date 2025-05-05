//! Database corruption manager.
use std::{io, sync::Arc};

use super::{batch::BoxedBatch, iterator::BoxedIterator, BoxedDatabase};
use crate::subnet::rpc::{errors, utils};

use tokio::sync::Mutex;

/// Database wrapper which blocks further calls to the database at first sign of corruption.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database/corruptabledb#Database>
#[derive(Clone)]
pub struct Database {
    /// The underlying database
    db: BoxedDatabase,
    /// Stores a corrupted error if observed.
    corrupted: Arc<Mutex<utils::Errors>>,
}

impl Database {
    #[must_use]
    pub fn new_boxed(db: BoxedDatabase) -> BoxedDatabase {
        Box::new(Self {
            db,
            corrupted: Arc::new(Mutex::new(utils::Errors::new())),
        })
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::KeyValueReaderWriterDeleter for Database {
    /// Attempts to return if the database has a key with the provided value.
    async fn has(&self, key: &[u8]) -> io::Result<bool> {
        let () = self.corrupted.lock().await.err()?;

        let db = &self.db;
        let has = match db.has(key).await {
            Ok(val) => val,
            Err(err) => {
                if errors::is_corruptible(&err) {
                    self.corrupted.lock().await.add(&io::Error::new(
                        io::ErrorKind::Other,
                        format!("closed to avoid possible corruption, init error: {err}"),
                    ));
                }
                return Err(err);
            }
        };

        Ok(has)
    }

    /// Attempts to return the value that was mapped to the key that was provided.
    async fn get(&self, key: &[u8]) -> io::Result<Vec<u8>> {
        let () = self.corrupted.lock().await.err()?;

        let db = &self.db;
        let value = match db.get(key).await {
            Ok(val) => val,
            Err(err) => {
                if errors::is_corruptible(&err) {
                    self.corrupted.lock().await.add(&io::Error::new(
                        io::ErrorKind::Other,
                        format!("closed to avoid possible corruption, init error: {err}"),
                    ));
                }
                return Err(err);
            }
        };

        Ok(value)
    }

    /// Attempts to set the value this key maps to.
    async fn put(&mut self, key: &[u8], value: &[u8]) -> io::Result<()> {
        let () = self.corrupted.lock().await.err()?;

        let db = &mut self.db;
        match db.put(key, value).await {
            Ok(()) => Ok(()),
            Err(err) => {
                if errors::is_corruptible(&err) {
                    self.corrupted.lock().await.add(&io::Error::new(
                        io::ErrorKind::Other,
                        format!("closed to avoid possible corruption, init error: {err}"),
                    ));
                }
                return Err(err);
            }
        }
    }

    /// Attempts to remove any mapping from the key.
    async fn delete(&mut self, key: &[u8]) -> io::Result<()> {
        let () = self.corrupted.lock().await.err()?;

        let db = &mut self.db;
        match db.delete(key).await {
            Ok(()) => Ok(()),
            Err(err) => {
                if errors::is_corruptible(&err) {
                    self.corrupted.lock().await.add(&io::Error::new(
                        io::ErrorKind::Other,
                        format!("closed to avoid possible corruption, init error: {err}"),
                    ));
                }
                return Err(err);
            }
        }
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::Closer for Database {
    /// Attempts to close the database.
    async fn close(&self) -> io::Result<()> {
        let () = self.corrupted.lock().await.err()?;

        let db = &self.db;
        match db.close().await {
            Ok(()) => Ok(()),
            Err(err) => {
                if errors::is_corruptible(&err) {
                    self.corrupted.lock().await.add(&io::Error::new(
                        io::ErrorKind::Other,
                        format!("closed to avoid possible corruption, init error: {err}"),
                    ));
                }
                return Err(err);
            }
        }
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::health::Checkable for Database {
    /// Checks if the database has been closed.
    async fn health_check(&self) -> io::Result<Vec<u8>> {
        let () = self.corrupted.lock().await.err()?;

        let db = &self.db;
        let check = match db.health_check().await {
            Ok(val) => val,
            Err(err) => {
                if errors::is_corruptible(&err) {
                    self.corrupted.lock().await.add(&io::Error::new(
                        io::ErrorKind::Other,
                        format!("closed to avoid possible corruption, init error: {err}"),
                    ));
                }
                return Err(err);
            }
        };

        Ok(check)
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::iterator::Iteratee for Database {
    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator(&self) -> io::Result<BoxedIterator> {
        self.new_iterator_with_start_and_prefix(&[], &[]).await
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator_with_start(&self, start: &[u8]) -> io::Result<BoxedIterator> {
        self.new_iterator_with_start_and_prefix(start, &[]).await
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator_with_prefix(&self, prefix: &[u8]) -> io::Result<BoxedIterator> {
        self.new_iterator_with_start_and_prefix(&[], prefix).await
    }

    /// Implements the [`crate::subnet::rpc::database::iterator::Iteratee`] trait.
    async fn new_iterator_with_start_and_prefix(
        &self,
        start: &[u8],
        prefix: &[u8],
    ) -> io::Result<BoxedIterator> {
        let () = self.corrupted.lock().await.err()?;

        self.db
            .new_iterator_with_start_and_prefix(start, prefix)
            .await
    }
}

#[tonic::async_trait]
impl crate::subnet::rpc::database::batch::Batcher for Database {
    /// Implements the [`crate::subnet::rpc::database::batch::Batcher`] trait.
    async fn new_batch(&self) -> io::Result<BoxedBatch> {
        let db = &self.db;
        let mut corrupted = self.corrupted.lock().await;

        let batch = db.new_batch().await.map_err(|err| {
            if errors::is_corruptible(&err) {
                corrupted.add(&io::Error::new(
                    io::ErrorKind::Other,
                    format!("closed to avoid possible corruption, init error: {err}"),
                ));
            }
            err
        })?;

        Ok(batch)
    }
}

impl crate::subnet::rpc::database::Database for Database {}
