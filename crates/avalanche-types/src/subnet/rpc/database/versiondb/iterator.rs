//! Database Iterator management implementation for versiondb.
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::subnet::rpc::{
    database::{self, iterator::BoxedIterator},
    errors::Error,
};

/// Iterator iterates over a versionbd database's key/value pairs.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#Iterator>
pub struct Iterator {
    /// The underlying database iterator
    iterator: BoxedIterator,
    /// Keys from the in-memory database
    keys: Vec<Vec<u8>>,
    /// Values from the in-memory database
    values: Vec<ValueDelete>,
    /// Error that occurred during iteration
    error: Option<io::Error>,
    /// Whether the database is closed
    closed: Arc<AtomicBool>,
    /// Current key
    key: Vec<u8>,
    /// Current value
    value: Vec<u8>,
    /// Whether the iterator has been initialized
    initialized: Arc<AtomicBool>,
    /// Whether the iterator has been exhausted
    exhausted: Arc<AtomicBool>,
}

#[derive(Clone, Debug)]
pub struct ValueDelete {
    pub value: Vec<u8>,
    pub delete: bool,
}

impl Iterator {
    pub fn new_boxed(
        keys: Vec<Vec<u8>>,
        values: Vec<ValueDelete>,
        closed: Arc<AtomicBool>,
        iterator: BoxedIterator,
    ) -> BoxedIterator {
        Box::new(Self {
            keys,
            values,
            error: None,
            closed,
            initialized: Arc::new(AtomicBool::new(false)),
            exhausted: Arc::new(AtomicBool::new(false)),
            iterator,
            key: vec![],
            value: vec![],
        })
    }
}

#[tonic::async_trait]
impl database::iterator::Iterator for Iterator {
    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn next(&mut self) -> io::Result<bool> {
        // set an error if the underlying database has been closed
        if self.closed.load(Ordering::Relaxed) {
            self.keys.clear();
            self.values.clear();
            self.error = Some(Error::DatabaseClosed.to_err());
            return Ok(false);
        }

        // initialize iterator
        if !self.initialized.load(Ordering::Relaxed) {
            let exhausted = !self.iterator.next().await?;
            self.exhausted.store(exhausted, Ordering::Relaxed);
            self.initialized.store(true, Ordering::Relaxed);
        }

        loop {
            if self.exhausted.load(Ordering::Relaxed) && self.keys.is_empty() {
                self.key.clear();
                self.value.clear();

                return Ok(false);
            }

            if self.exhausted.load(Ordering::Relaxed) {
                let next_key = self.keys.first().unwrap().clone();
                let next_value = self.values.first().unwrap().clone();

                self.keys[0].clear();
                self.keys = self.keys[1..].to_vec();
                self.values[0].value.clear();
                self.values = self.values[1..].to_vec();

                if !next_value.delete {
                    self.key = next_key;
                    self.value = next_value.value;

                    return Ok(true);
                }
            }

            if self.keys.is_empty() {
                self.key = self.iterator.key().await?.to_vec();
                self.value = self.iterator.value().await?.to_vec();
                let exhausted = !self.iterator.next().await?;
                self.exhausted.store(exhausted, Ordering::Relaxed);

                return Ok(true);
            }

            let mem_key = self.keys.first().unwrap().clone();
            let mem_value = self.values.first().unwrap().clone();
            let db_key = self.iterator.key().await?.to_vec();

            if mem_key.lt(&db_key) {
                self.keys[0].clear();
                self.keys = self.keys[1..].to_vec();
                self.values[0].value.clear();
                self.values = self.values[1..].to_vec();

                if !mem_value.delete {
                    self.key = mem_key;
                    self.value.clone_from(&mem_value.value);

                    return Ok(true);
                }
            }

            if db_key.lt(&mem_key) {
                self.key.clone_from(&db_key);
                self.value = self.iterator.value().await?.to_vec();
                let exhausted = !self.iterator.next().await?;
                self.exhausted.store(exhausted, Ordering::Relaxed);

                return Ok(true);
            }

            self.keys[0].clear();
            self.keys = self.keys[1..].to_vec();
            self.values[0].value.clear();
            self.values = self.values[1..].to_vec();

            let exhausted = !self.iterator.next().await?;
            self.exhausted.store(exhausted, Ordering::Relaxed);

            if !mem_value.delete {
                mem_key.clone_into(&mut self.key);
                self.value.clone_from(&mem_value.value);
                return Ok(true);
            }
        }
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn error(&mut self) -> io::Result<()> {
        if let Some(err) = &self.error {
            return Err(io::Error::new(err.kind(), err.to_string()));
        }

        self.iterator.error().await
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn key(&self) -> io::Result<&[u8]> {
        Ok(&self.key)
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn value(&self) -> io::Result<&[u8]> {
        Ok(&self.value)
    }

    /// Implements the \[`crate::subnet::rpc::database::Iterator`\] trait.
    async fn release(&mut self) {
        self.key.clear();
        self.value.clear();
        self.keys.clear();
        self.values.clear();
        self.iterator.release().await;
    }
}
