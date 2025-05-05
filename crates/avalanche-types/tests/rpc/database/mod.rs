mod batch;
mod concurrency;
mod iterator;

use std::{io::ErrorKind, time::Duration};

pub use super::serve_test_database;
use avalanche_types::subnet::rpc::database::{
    corruptabledb::Database as CorruptableDb,
    manager::{versioned_database::VersionedDatabase, DatabaseManager},
    memdb::Database as MemDb,
    rpcdb::{client::DatabaseClient, server::Server as RpcDb},
    Closer, KeyValueReaderWriterDeleter,
};
use semver::Version;
use tokio::net::TcpListener;
use tonic::transport::Channel;

#[tokio::test]
#[allow(clippy::significant_drop_tightening)]
async fn rpcdb_mutation_test() {
    let _result = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let bar_value = b"bar".to_vec();
    let baz_data = b"baz".to_vec();

    let db = MemDb::new_boxed();
    let server = RpcDb::new_boxed(db);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_test_database(server, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = DatabaseClient::new(
        Channel::builder(format!("http://{addr}").parse().unwrap())
            .connect()
            .await
            .unwrap(),
    );

    log::info!("put foo:bar");
    let resp = client.put(b"foo", b"bar").await;
    assert!(resp.is_ok());

    log::info!("get foo:bar");
    let resp = client.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, bar_value.clone());

    // verify valid response from cloning client
    let mut client = client.clone();

    log::info!("put foo:baz");
    let resp = client.put(b"foo", b"baz").await;
    assert!(resp.is_ok());

    log::info!("get foo:baz");
    let resp = client.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, baz_data.clone());

    log::info!("has foo true");
    let resp = client.has(b"foo").await;
    assert!(resp.is_ok());
    assert!(resp.unwrap());

    log::info!("get fool error not found");
    let resp = client.get(b"fool").await;
    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().kind(), ErrorKind::NotFound);

    log::info!("has fool false");
    let resp = client.has(b"fool").await;
    assert!(resp.is_ok());
    assert!(!resp.unwrap());

    log::info!("close client");
    let resp = client.close().await;
    assert!(resp.is_ok());

    log::info!("get foo error closed");
    let resp = client.get(b"foo").await;
    assert!(resp.is_err());
    assert!(resp.unwrap_err().to_string().contains("database closed"));
    drop(client);
}

#[tokio::test]
async fn corruptibledb_mutation_test() {
    let _result = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let bar_value = b"bar".to_vec();

    let memdb = MemDb::new_boxed();
    let mut corruptible = CorruptableDb::new_boxed(memdb);

    log::info!("put foo:bar");
    let resp = corruptible.put(b"foo", b"bar").await;
    assert!(resp.is_ok());

    log::info!("get foo:bar");
    let resp = corruptible.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, bar_value.clone());

    log::info!("put foo:baz");
    let resp = corruptible.put(b"foo", b"baz").await;
    assert!(resp.is_ok());

    log::info!("has foo true");
    let resp = corruptible.has(b"foo").await;
    assert!(resp.is_ok());
    assert!(resp.unwrap());

    log::info!("get fool error not found");
    let resp = corruptible.get(b"fool").await;
    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().kind(), ErrorKind::NotFound);

    log::info!("has fool false");
    let resp = corruptible.has(b"fool").await;
    assert!(resp.is_ok());
    assert!(!resp.unwrap());

    log::info!("close client");
    let resp = corruptible.close().await;
    assert!(resp.is_ok());

    log::info!("get foo error closed");
    let resp = corruptible.put(b"foo", b"baz").await;
    assert!(resp.is_err());
    assert!(resp.unwrap_err().to_string().contains("database closed"));
}

#[tokio::test]
#[allow(clippy::significant_drop_tightening)]
async fn test_rpcdb_corruptible() {
    let _result = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let bar_value = b"bar".to_vec();
    let baz_data = b"baz".to_vec();

    let memdb = MemDb::new_boxed();
    let rpc_server = RpcDb::new_boxed(memdb);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_test_database(rpc_server, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = DatabaseClient::new(
        Channel::builder(format!("http://{addr}").parse().unwrap())
            .connect()
            .await
            .unwrap(),
    );

    log::info!("put foo:bar");
    let resp = client.put(b"foo", b"bar").await;
    assert!(resp.is_ok());

    log::info!("get foo:bar");
    let resp = client.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, bar_value.clone());

    // verify valid response from cloning client
    let mut client = client.clone();

    log::info!("put foo:baz");
    let resp = client.put(b"foo", b"baz").await;
    assert!(resp.is_ok());

    log::info!("get foo:baz");
    let resp = client.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, baz_data.clone());

    log::info!("has foo true");
    let resp = client.has(b"foo").await;
    assert!(resp.is_ok());
    assert!(resp.unwrap());

    log::info!("get fool error not found");
    let resp = client.get(b"fool").await;
    assert!(resp.is_err());
    assert_eq!(resp.unwrap_err().kind(), ErrorKind::NotFound);

    log::info!("has fool false");
    let resp = client.has(b"fool").await;
    assert!(resp.is_ok());
    assert!(!resp.unwrap());

    log::info!("close client");
    let resp = client.close().await;
    assert!(resp.is_ok());

    log::info!("get foo error closed");
    let resp = client.get(b"foo").await;
    assert!(resp.is_err());
    assert!(resp.unwrap_err().to_string().contains("database closed"));
    drop(client);
}

#[tokio::test]
async fn test_db_manager() {
    use avalanche_types::subnet::rpc::database::manager::Manager;
    let _result = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let bar_value = b"bar".to_vec();
    let baz_data = b"baz".to_vec();

    let memdb = MemDb::new_boxed();
    let rpc_server = RpcDb::new_boxed(memdb);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_test_database(rpc_server, listener).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let vdb = VersionedDatabase::new(
        DatabaseClient::new_boxed(
            Channel::builder(format!("http://{addr}").parse().unwrap())
                .connect()
                .await
                .unwrap(),
        ),
        Version::new(0, 0, 1),
    );

    let databases: Vec<VersionedDatabase> = vec![vdb];

    let manager = DatabaseManager::from_databases(databases);
    let current = manager.current().await.unwrap();

    let mut client = current.db;

    log::info!("put foo:bar");
    let resp = client.put(b"foo", b"bar").await;
    assert!(resp.is_ok());

    log::info!("get foo:bar");
    let resp = client.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, bar_value.clone());

    // verify valid response from cloning client
    let mut client = client.clone();

    log::info!("put foo:baz");
    let resp = client.put(b"foo", b"baz").await;
    assert!(resp.is_ok());

    log::info!("get foo:baz");
    let resp = client.get(b"foo").await;
    let value = resp.unwrap();
    assert_eq!(value, baz_data.clone());

    log::info!("close all db with manager");
    let _result = manager.close().await;

    log::info!("get foo error closed");
    let resp = client.get(b"foo").await;
    assert!(resp.is_err());
    assert!(resp.unwrap_err().to_string().contains("database closed"));
}
