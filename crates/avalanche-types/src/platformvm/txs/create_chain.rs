use crate::{codec, errors::Result, hash, ids, key, txs};
use serde::{Deserialize, Serialize};

/// `CreateChainTx` is a transaction that creates a new chain.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#CreateChainTx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx>
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#UnsignedTx>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Tx {
    /// The transaction ID is empty for unsigned tx
    /// as long as "avax.BaseTx.Metadata" is "None".
    /// Once Metadata is updated with signing and "Tx.Initialize",
    /// `Tx.ID()` is non-empty.
    pub base_tx: txs::Tx,
    pub subnet_id: ids::Id,
    pub chain_name: String,
    pub vm_id: ids::Id,
    pub fx_ids: Option<Vec<ids::Id>>,
    pub genesis_data: Vec<u8>,
    pub subnet_auth: key::secp256k1::txs::Input,

    /// To be updated after signing.
    pub creds: Vec<key::secp256k1::txs::Credential>,
}

impl Default for Tx {
    fn default() -> Self {
        Self {
            base_tx: txs::Tx::default(),
            subnet_id: ids::Id::empty(),
            chain_name: String::new(),
            vm_id: ids::Id::empty(),
            fx_ids: None,
            genesis_data: Vec::new(),
            subnet_auth: key::secp256k1::txs::Input::default(),
            creds: Vec::new(),
        }
    }
}

impl Tx {
    #[must_use]
    pub fn new(base_tx: txs::Tx) -> Self {
        Self {
            base_tx,
            ..Self::default()
        }
    }

    /// Returns the transaction ID.
    /// Only non-empty if the embedded metadata is updated
    /// with the signing process.
    ///
    /// # Panics
    ///
    /// Panics if `self.base_tx.metadata` is `Some` but cannot be unwrapped.
    #[must_use]
    pub fn tx_id(&self) -> ids::Id {
        if self.base_tx.metadata.is_some() {
            let m = self.base_tx.metadata.clone().unwrap();
            m.id
        } else {
            ids::Id::default()
        }
    }

    #[must_use]
    pub fn type_name() -> String {
        "platformvm.CreateChainTx".to_string()
    }

    /// Returns the type ID for this transaction.
    ///
    /// # Panics
    ///
    /// Panics if the type name is not found in the codec registry.
    #[must_use]
    pub fn type_id() -> u32 {
        u32::try_from(*(codec::P_TYPES.get(&Self::type_name()).unwrap())).unwrap()
    }

    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm/txs#Tx.Sign>
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/utils/crypto#PrivateKeyED25519.SignHash>
    /// Signs the transaction with the provided signers.
    ///
    /// # Panics
    ///
    /// Panics if `self.fx_ids` is `Some` but cannot be unwrapped.
    ///
    /// # Errors
    ///
    /// Returns an error if the signing process fails.
    #[allow(clippy::too_many_lines)]
    pub async fn sign<T: key::secp256k1::SignOnly + Send + Sync>(
        &mut self,
        signers: Vec<Vec<T>>,
    ) -> Result<()> {
        // marshal "unsigned tx" with the codec version
        let type_id = Self::type_id();
        let packer = self.base_tx.pack(codec::VERSION, type_id)?;

        // "avalanchego" marshals the whole struct again for signed bytes
        // even when the underlying "unsigned_tx" is already once marshaled
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#Tx.Sign
        //
        // reuse the underlying packer to avoid marshaling the unsigned tx twice
        // just marshal the next fields in the struct and pack them all together
        // in the existing packer
        let base = packer.take_bytes();
        packer.set_bytes(&base);

        // pack the second field "subnet_id" in the struct
        packer.pack_bytes(self.subnet_id.as_ref())?;

        // pack the third field "chain_name" in the struct
        packer.pack_str(&self.chain_name)?;

        // pack the fourth field "vm_id" in the struct
        packer.pack_bytes(self.vm_id.as_ref())?;

        // pack the fifth field "fx_ids" in the struct
        if self.fx_ids.is_some() {
            let fx_ids = self.fx_ids.as_ref().unwrap();
            packer.pack_u32(u32::try_from(fx_ids.len()).unwrap())?;
            for fx_id in fx_ids {
                packer.pack_bytes(fx_id.as_ref())?;
            }
        } else {
            packer.pack_u32(0_u32)?;
        }

        // pack the sixth field "genesis_data" in the struct
        // []byte is reflected as "reflect.Slice" in avalanchego
        // thus encode its length
        packer.pack_u32(u32::try_from(self.genesis_data.len()).unwrap())?;
        packer.pack_bytes(&self.genesis_data)?;

        // pack the seventh field "subnet_auth" in the struct
        let subnet_auth_type_id = key::secp256k1::txs::Input::type_id();
        packer.pack_u32(subnet_auth_type_id)?;
        packer.pack_u32(u32::try_from(self.subnet_auth.sig_indices.len()).unwrap())?;
        for sig_idx in &self.subnet_auth.sig_indices {
            packer.pack_u32(*sig_idx)?;
        }

        // take bytes just for hashing computation
        let tx_bytes_with_no_signature = packer.take_bytes();
        packer.set_bytes(&tx_bytes_with_no_signature);

        // compute sha256 for marshaled "unsigned tx" bytes
        // IMPORTANT: take the hash only for the type "platformvm.AddValidatorTx" unsigned tx
        // not other fields -- only hash "platformvm.AddValidatorTx.*" but not "platformvm.Tx.Creds"
        // ref. https://pkg.go.dev/github.com/ava-labs/avalanchego/vms/platformvm#UnsignedAddValidatorTx
        let tx_bytes_hash = hash::sha256(&tx_bytes_with_no_signature);

        // number of of credentials
        let creds_len = u32::try_from(signers.len()).unwrap();
        // pack the fourth field in the struct
        packer.pack_u32(creds_len)?;

        // sign the hash with the signers (in case of multi-sig)
        // and combine all signatures into a secp256k1fx credential
        self.creds = Vec::new();
        for keys in &signers {
            let mut sigs: Vec<Vec<u8>> = Vec::new();
            for k in keys {
                let sig = k.sign_digest(&tx_bytes_hash).await?;
                sigs.push(Vec::from(sig));
            }

            let cred = key::secp256k1::txs::Credential { signatures: sigs };

            // add a new credential to "Tx"
            self.creds.push(cred);
        }
        if creds_len > 0 {
            // pack each "cred" which is "secp256k1fx.Credential"
            // marshal type ID for "secp256k1fx.Credential"
            let cred_type_id = key::secp256k1::txs::Credential::type_id();
            for cred in &self.creds {
                // marshal type ID for "secp256k1fx.Credential"
                packer.pack_u32(cred_type_id)?;

                // marshal fields for "secp256k1fx.Credential"
                packer.pack_u32(u32::try_from(cred.signatures.len()).unwrap())?;
                for sig in &cred.signatures {
                    packer.pack_bytes(sig)?;
                }
            }
        }
        let tx_bytes_with_signatures = packer.take_bytes();
        let tx_id = hash::sha256(&tx_bytes_with_signatures);

        // update "BaseTx.Metadata" with id/unsigned bytes/bytes
        // ref. "avalanchego/vms/platformvm.Tx.Sign"
        // ref. "avalanchego/vms/components/avax.BaseTx.Metadata.Initialize"
        self.base_tx.metadata = Some(txs::Metadata {
            id: ids::Id::from_slice(&tx_id),
            tx_bytes_with_no_signature: tx_bytes_with_no_signature.to_vec(),
            tx_bytes_with_signatures: tx_bytes_with_signatures.to_vec(),
        });

        Ok(())
    }
}

/// `RUST_LOG=debug` cargo test --package avalanche-types --lib -- `platformvm::txs::create_chain::test_create_chain_tx_serialization_with_one_signer` --exact --show-output
#[test]
fn test_create_chain_tx_serialization_with_one_signer() {
    use crate::ids::short;

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    let mut tx = Tx {
        base_tx: txs::Tx {
            network_id: 1_000_000,
            transferable_outputs: Some(vec![txs::transferable::Output {
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                    0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                    0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                    0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
                ])),
                transfer_output: Some(key::secp256k1::txs::transfer::Output {
                    amount: 0x02c6_874d_5c56_f500,
                    output_owners: key::secp256k1::txs::OutputOwners {
                        locktime: 0x00,
                        threshold: 0x01,
                        addresses: vec![short::Id::from_slice(&<Vec<u8>>::from([
                            0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, //
                            0x81, 0x42, 0xc6, 0xc2, 0xa7, 0x83, 0xef, 0x87, 0x1d, 0xe9, //
                        ]))],
                    },
                }),
                ..txs::transferable::Output::default()
            }]),
            transferable_inputs: Some(vec![txs::transferable::Input {
                utxo_id: txs::utxo::Id {
                    tx_id: ids::Id::from_slice(&<Vec<u8>>::from([
                        0x4e, 0x02, 0x63, 0x73, 0xef, 0x9f, 0x0f, 0xaf, 0xf6, 0x24, //
                        0x11, 0xc7, 0x15, 0x80, 0x8b, 0x28, 0x00, 0x60, 0x32, 0xce, //
                        0x82, 0x9e, 0x1c, 0xb5, 0xb0, 0x46, 0xb9, 0xc8, 0x83, 0xae, //
                        0xfb, 0xbc,
                    ])),
                    output_index: 0,
                    ..txs::utxo::Id::default()
                },
                asset_id: ids::Id::from_slice(&<Vec<u8>>::from([
                    0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, //
                    0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, //
                    0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, //
                    0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, 0x80, 0x14, //
                ])),
                transfer_input: Some(key::secp256k1::txs::transfer::Input {
                    amount: 0x02c6_874d_624c_d600,
                    sig_indices: vec![0],
                }),
                ..txs::transferable::Input::default()
            }]),
            ..txs::Tx::default()
        },
        subnet_id: ids::Id::from_slice(&<Vec<u8>>::from([
            0xda, 0x77, 0x6a, 0xb0, 0xf6, 0x10, 0x01, 0x8e, 0x60, 0xa5, //
            0x0a, 0xc5, 0xb1, 0x48, 0x9a, 0x4d, 0xcd, 0xe0, 0x25, 0xf1, //
            0xf4, 0xa5, 0x62, 0x60, 0xc4, 0x4b, 0x86, 0x19, 0x46, 0x05, //
            0x0f, 0x11,
        ])),
        chain_name: String::from("subnetevm"),
        vm_id: ids::Id::from_slice(&<Vec<u8>>::from([
            0x73, 0x75, 0x62, 0x6e, 0x65, 0x74, 0x65, 0x76, 0x6d, 0x00, //
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
            0x00, 0x00, //
        ])),
        genesis_data: <Vec<u8>>::from([
            0x7b, 0x22, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a, 0x7b, 0x22, 0x63, 0x68,
            0x61, 0x69, 0x6e, 0x49, 0x64, 0x22, 0x3a, 0x32, 0x30, 0x30, 0x30, 0x37, 0x37, 0x37,
            0x2c, 0x22, 0x68, 0x6f, 0x6d, 0x65, 0x73, 0x74, 0x65, 0x61, 0x64, 0x42, 0x6c, 0x6f,
            0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x65, 0x69, 0x70, 0x31, 0x35, 0x30, 0x42,
            0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x65, 0x69, 0x70, 0x31, 0x35,
            0x30, 0x48, 0x61, 0x73, 0x68, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x32, 0x30, 0x38, 0x36,
            0x37, 0x39, 0x39, 0x61, 0x65, 0x65, 0x62, 0x65, 0x61, 0x65, 0x31, 0x33, 0x35, 0x63,
            0x32, 0x34, 0x36, 0x63, 0x36, 0x35, 0x30, 0x32, 0x31, 0x63, 0x38, 0x32, 0x62, 0x34,
            0x65, 0x31, 0x35, 0x61, 0x32, 0x63, 0x34, 0x35, 0x31, 0x33, 0x34, 0x30, 0x39, 0x39,
            0x33, 0x61, 0x61, 0x63, 0x66, 0x64, 0x32, 0x37, 0x35, 0x31, 0x38, 0x38, 0x36, 0x35,
            0x31, 0x34, 0x66, 0x30, 0x22, 0x2c, 0x22, 0x65, 0x69, 0x70, 0x31, 0x35, 0x35, 0x42,
            0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x65, 0x69, 0x70, 0x31, 0x35,
            0x38, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x62, 0x79, 0x7a,
            0x61, 0x6e, 0x74, 0x69, 0x75, 0x6d, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30,
            0x2c, 0x22, 0x63, 0x6f, 0x6e, 0x73, 0x74, 0x61, 0x6e, 0x74, 0x69, 0x6e, 0x6f, 0x70,
            0x6c, 0x65, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x70, 0x65,
            0x74, 0x65, 0x72, 0x73, 0x62, 0x75, 0x72, 0x67, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22,
            0x3a, 0x30, 0x2c, 0x22, 0x69, 0x73, 0x74, 0x61, 0x6e, 0x62, 0x75, 0x6c, 0x42, 0x6c,
            0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x6d, 0x75, 0x69, 0x72, 0x47, 0x6c,
            0x61, 0x63, 0x69, 0x65, 0x72, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c,
            0x22, 0x73, 0x75, 0x62, 0x6e, 0x65, 0x74, 0x45, 0x56, 0x4d, 0x54, 0x69, 0x6d, 0x65,
            0x73, 0x74, 0x61, 0x6d, 0x70, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x66, 0x65, 0x65, 0x43,
            0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a, 0x7b, 0x22, 0x67, 0x61, 0x73, 0x4c, 0x69,
            0x6d, 0x69, 0x74, 0x22, 0x3a, 0x32, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2c,
            0x22, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x52, 0x61,
            0x74, 0x65, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x69, 0x6e, 0x42, 0x61, 0x73, 0x65,
            0x46, 0x65, 0x65, 0x22, 0x3a, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x2c, 0x22, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x47, 0x61, 0x73, 0x22, 0x3a,
            0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2c, 0x22, 0x62, 0x61, 0x73,
            0x65, 0x46, 0x65, 0x65, 0x43, 0x68, 0x61, 0x6e, 0x67, 0x65, 0x44, 0x65, 0x6e, 0x6f,
            0x6d, 0x69, 0x6e, 0x61, 0x74, 0x6f, 0x72, 0x22, 0x3a, 0x34, 0x38, 0x2c, 0x22, 0x6d,
            0x69, 0x6e, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x47, 0x61, 0x73, 0x43, 0x6f, 0x73, 0x74,
            0x22, 0x3a, 0x30, 0x2c, 0x22, 0x6d, 0x61, 0x78, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x47,
            0x61, 0x73, 0x43, 0x6f, 0x73, 0x74, 0x22, 0x3a, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x2c, 0x22, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x47, 0x61, 0x73, 0x43, 0x6f,
            0x73, 0x74, 0x53, 0x74, 0x65, 0x70, 0x22, 0x3a, 0x35, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x7d, 0x2c, 0x22, 0x63, 0x6f, 0x6e, 0x74, 0x72, 0x61, 0x63, 0x74, 0x44, 0x65, 0x70,
            0x6c, 0x6f, 0x79, 0x65, 0x72, 0x41, 0x6c, 0x6c, 0x6f, 0x77, 0x4c, 0x69, 0x73, 0x74,
            0x43, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x6c, 0x6f, 0x63,
            0x6b, 0x54, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x22, 0x3a, 0x30, 0x2c,
            0x22, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x41, 0x64, 0x64, 0x72, 0x65, 0x73, 0x73, 0x65,
            0x73, 0x22, 0x3a, 0x5b, 0x22, 0x30, 0x78, 0x38, 0x64, 0x62, 0x39, 0x37, 0x43, 0x37,
            0x63, 0x45, 0x63, 0x45, 0x32, 0x34, 0x39, 0x63, 0x32, 0x62, 0x39, 0x38, 0x62, 0x44,
            0x43, 0x30, 0x32, 0x32, 0x36, 0x43, 0x63, 0x34, 0x43, 0x32, 0x41, 0x35, 0x37, 0x42,
            0x46, 0x35, 0x32, 0x46, 0x43, 0x22, 0x2c, 0x22, 0x30, 0x78, 0x36, 0x31, 0x33, 0x30,
            0x34, 0x30, 0x61, 0x32, 0x33, 0x39, 0x42, 0x44, 0x66, 0x43, 0x46, 0x31, 0x31, 0x30,
            0x39, 0x36, 0x39, 0x66, 0x65, 0x63, 0x42, 0x34, 0x31, 0x63, 0x36, 0x66, 0x39, 0x32,
            0x45, 0x41, 0x33, 0x35, 0x31, 0x35, 0x43, 0x30, 0x22, 0x2c, 0x22, 0x30, 0x78, 0x30,
            0x61, 0x36, 0x33, 0x61, 0x43, 0x43, 0x33, 0x37, 0x33, 0x35, 0x65, 0x38, 0x32, 0x35,
            0x44, 0x37, 0x44, 0x31, 0x33, 0x32, 0x34, 0x33, 0x46, 0x44, 0x37, 0x36, 0x62, 0x41,
            0x64, 0x34, 0x39, 0x33, 0x33, 0x31, 0x62, 0x61, 0x45, 0x30, 0x45, 0x22, 0x2c, 0x22,
            0x30, 0x78, 0x32, 0x66, 0x63, 0x39, 0x32, 0x32, 0x42, 0x65, 0x65, 0x39, 0x30, 0x32,
            0x35, 0x32, 0x30, 0x63, 0x34, 0x36, 0x38, 0x31, 0x63, 0x35, 0x62, 0x62, 0x64, 0x39,
            0x37, 0x39, 0x30, 0x38, 0x43, 0x37, 0x32, 0x37, 0x36, 0x36, 0x34, 0x65, 0x35, 0x36,
            0x22, 0x2c, 0x22, 0x30, 0x78, 0x30, 0x43, 0x38, 0x35, 0x66, 0x32, 0x37, 0x35, 0x35,
            0x30, 0x63, 0x61, 0x62, 0x33, 0x31, 0x32, 0x37, 0x46, 0x42, 0x36, 0x44, 0x61, 0x38,
            0x34, 0x45, 0x36, 0x44, 0x44, 0x63, 0x65, 0x43, 0x66, 0x33, 0x34, 0x32, 0x37, 0x32,
            0x66, 0x44, 0x30, 0x22, 0x5d, 0x7d, 0x7d, 0x2c, 0x22, 0x6e, 0x6f, 0x6e, 0x63, 0x65,
            0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22, 0x74, 0x69, 0x6d, 0x65, 0x73,
            0x74, 0x61, 0x6d, 0x70, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22, 0x65,
            0x78, 0x74, 0x72, 0x61, 0x44, 0x61, 0x74, 0x61, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30,
            0x30, 0x22, 0x2c, 0x22, 0x67, 0x61, 0x73, 0x4c, 0x69, 0x6d, 0x69, 0x74, 0x22, 0x3a,
            0x22, 0x30, 0x78, 0x31, 0x33, 0x31, 0x32, 0x64, 0x30, 0x30, 0x22, 0x2c, 0x22, 0x64,
            0x69, 0x66, 0x66, 0x69, 0x63, 0x75, 0x6c, 0x74, 0x79, 0x22, 0x3a, 0x22, 0x30, 0x78,
            0x30, 0x22, 0x2c, 0x22, 0x6d, 0x69, 0x78, 0x48, 0x61, 0x73, 0x68, 0x22, 0x3a, 0x22,
            0x30, 0x78, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x2c, 0x22, 0x63,
            0x6f, 0x69, 0x6e, 0x62, 0x61, 0x73, 0x65, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x2c, 0x22, 0x61,
            0x6c, 0x6c, 0x6f, 0x63, 0x22, 0x3a, 0x7b, 0x22, 0x30, 0x43, 0x38, 0x35, 0x66, 0x32,
            0x37, 0x35, 0x35, 0x30, 0x63, 0x61, 0x62, 0x33, 0x31, 0x32, 0x37, 0x46, 0x42, 0x36,
            0x44, 0x61, 0x38, 0x34, 0x45, 0x36, 0x44, 0x44, 0x63, 0x65, 0x43, 0x66, 0x33, 0x34,
            0x32, 0x37, 0x32, 0x66, 0x44, 0x30, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61,
            0x6e, 0x63, 0x65, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32,
            0x64, 0x63, 0x63, 0x38, 0x30, 0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x22, 0x7d, 0x2c, 0x22, 0x30, 0x61, 0x36, 0x33, 0x61, 0x43, 0x43, 0x33,
            0x37, 0x33, 0x35, 0x65, 0x38, 0x32, 0x35, 0x44, 0x37, 0x44, 0x31, 0x33, 0x32, 0x34,
            0x33, 0x46, 0x44, 0x37, 0x36, 0x62, 0x41, 0x64, 0x34, 0x39, 0x33, 0x33, 0x31, 0x62,
            0x61, 0x45, 0x30, 0x45, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63,
            0x65, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63,
            0x63, 0x38, 0x30, 0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x22, 0x7d, 0x2c, 0x22, 0x32, 0x66, 0x63, 0x39, 0x32, 0x32, 0x42, 0x65, 0x65, 0x39,
            0x30, 0x32, 0x35, 0x32, 0x30, 0x63, 0x34, 0x36, 0x38, 0x31, 0x63, 0x35, 0x62, 0x62,
            0x64, 0x39, 0x37, 0x39, 0x30, 0x38, 0x43, 0x37, 0x32, 0x37, 0x36, 0x36, 0x34, 0x65,
            0x35, 0x36, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22,
            0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38,
            0x30, 0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d,
            0x2c, 0x22, 0x36, 0x31, 0x33, 0x30, 0x34, 0x30, 0x61, 0x32, 0x33, 0x39, 0x42, 0x44,
            0x66, 0x43, 0x46, 0x31, 0x31, 0x30, 0x39, 0x36, 0x39, 0x66, 0x65, 0x63, 0x42, 0x34,
            0x31, 0x63, 0x36, 0x66, 0x39, 0x32, 0x45, 0x41, 0x33, 0x35, 0x31, 0x35, 0x43, 0x30,
            0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22, 0x3a, 0x22,
            0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38, 0x30, 0x63,
            0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, 0x2c, 0x22,
            0x38, 0x64, 0x62, 0x39, 0x37, 0x43, 0x37, 0x63, 0x45, 0x63, 0x45, 0x32, 0x34, 0x39,
            0x63, 0x32, 0x62, 0x39, 0x38, 0x62, 0x44, 0x43, 0x30, 0x32, 0x32, 0x36, 0x43, 0x63,
            0x34, 0x43, 0x32, 0x41, 0x35, 0x37, 0x42, 0x46, 0x35, 0x32, 0x46, 0x43, 0x22, 0x3a,
            0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22, 0x3a, 0x22, 0x30, 0x78,
            0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38, 0x30, 0x63, 0x64, 0x32,
            0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, 0x7d, 0x2c, 0x22, 0x6e,
            0x75, 0x6d, 0x62, 0x65, 0x72, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22,
            0x67, 0x61, 0x73, 0x55, 0x73, 0x65, 0x64, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22,
            0x2c, 0x22, 0x70, 0x61, 0x72, 0x65, 0x6e, 0x74, 0x48, 0x61, 0x73, 0x68, 0x22, 0x3a,
            0x22, 0x30, 0x78, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
            0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d,
        ]),
        subnet_auth: key::secp256k1::txs::Input {
            sig_indices: vec![0],
        },
        ..Tx::default()
    };

    let test_key = key::secp256k1::private_key::Key::from_cb58(
        "PrivateKey-2kqWNDaqUKQyE4ZsV5GLCGeizE6sHAJVyjnfjXoXrtcZpK9M67",
    )
    .expect("failed to load private key");
    let keys1: Vec<key::secp256k1::private_key::Key> = vec![test_key.clone()];
    let keys2: Vec<key::secp256k1::private_key::Key> = vec![test_key];
    let signers: Vec<Vec<key::secp256k1::private_key::Key>> = vec![keys1, keys2];
    ab!(tx.sign(signers)).expect("failed to sign");
    let tx_metadata = tx.base_tx.metadata.clone().unwrap();
    let tx_bytes_with_signatures = tx_metadata.tx_bytes_with_signatures;
    assert_eq!(
        tx.tx_id().to_string(),
        "2nWs4EB5gmBz99pn4Vck3dBjnPysv44HRiXvNQNpQUonfTNsTf"
    );

    let expected_signed_bytes: &[u8] = &[
        // codec version
        0x00, 0x00, //
        //
        // platformvm.UnsignedCreateChainTx type ID
        0x00, 0x00, 0x00, 0x0f, //
        //
        // network id
        0x00, 0x0f, 0x42, 0x40, //
        //
        // blockchain id
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // outs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.asset_id
        0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, 0xe6, 0x89, 0x61, 0x8e, 0x87, 0x21, 0xe0,
        0x4a, 0xe8, 0x5e, 0xa5, 0x74, 0xc7, 0xa1, 0x5a, 0x79, 0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47,
        0x80, 0x14, //
        //
        // NOTE: fx_id is serialize:"false"
        //
        // "outs[0]" secp256k1fx.TransferOutput type ID
        0x00, 0x00, 0x00, 0x07, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.amount
        0x02, 0xc6, 0x87, 0x4d, 0x5c, 0x56, 0xf5, 0x00, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.locktime
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.threshold
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.addrs.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "outs[0]" TransferableOutput.out.key::secp256k1::txs::transfer::Output.output_owners.addrs[0]
        0x65, 0x84, 0x4a, 0x05, 0x40, 0x5f, 0x36, 0x62, 0xc1, 0x92, 0x81, 0x42, 0xc6, 0xc2, 0xa7,
        0x83, 0xef, 0x87, 0x1d, 0xe9, //
        //
        // ins.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.utxo_id.tx_id
        0x4e, 0x02, 0x63, 0x73, 0xef, 0x9f, 0x0f, 0xaf, 0xf6, 0x24, //
        0x11, 0xc7, 0x15, 0x80, 0x8b, 0x28, 0x00, 0x60, 0x32, 0xce, //
        0x82, 0x9e, 0x1c, 0xb5, 0xb0, 0x46, 0xb9, 0xc8, 0x83, 0xae, //
        0xfb, 0xbc, //
        //
        // "ins[0]" TransferableInput.utxo_id.output_index
        0x00, 0x00, 0x00, 0x00, //
        //
        // "ins[0]" TransferableInput.asset_id
        0x88, 0xee, 0xc2, 0xe0, 0x99, 0xc6, 0xa5, 0x28, 0xe6, 0x89, //
        0x61, 0x8e, 0x87, 0x21, 0xe0, 0x4a, 0xe8, 0x5e, 0xa5, 0x74, //
        0xc7, 0xa1, 0x5a, 0x79, 0x68, 0x64, 0x4d, 0x14, 0xd5, 0x47, //
        0x80, 0x14, //
        //
        // "ins[0]" secp256k1fx.TransferInput type ID
        0x00, 0x00, 0x00, 0x05, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.amount
        0x02, 0xc6, 0x87, 0x4d, 0x62, 0x4c, 0xd6, 0x00, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // "ins[0]" TransferableInput.input.key::secp256k1::txs::transfer::Input.sig_indices[0]
        0x00, 0x00, 0x00, 0x00, //
        //
        // memo.len()
        0x00, 0x00, 0x00, 0x00, //
        //
        // subnet_id
        0xda, 0x77, 0x6a, 0xb0, 0xf6, 0x10, 0x01, 0x8e, 0x60, 0xa5, //
        0x0a, 0xc5, 0xb1, 0x48, 0x9a, 0x4d, 0xcd, 0xe0, 0x25, 0xf1, //
        0xf4, 0xa5, 0x62, 0x60, 0xc4, 0x4b, 0x86, 0x19, 0x46, 0x05, //
        0x0f, 0x11, //
        //
        // chain name "length"
        0x00, 0x09, //
        //
        // chain name
        0x73, 0x75, 0x62, 0x6e, 0x65, 0x74, 0x65, 0x76, 0x6d, //
        //
        // vm id
        0x73, 0x75, 0x62, 0x6e, 0x65, 0x74, 0x65, 0x76, 0x6d, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, //
        0x00, 0x00, //
        //
        // fx_ids.len()
        0x00, 0x00, 0x00, 0x00, //
        //
        // genesis_data.len()
        0x00, 0x00, 0x06, 0x1f, //
        //
        // genesis
        0x7b, 0x22, 0x63, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a, 0x7b, 0x22, 0x63, 0x68, 0x61,
        0x69, 0x6e, 0x49, 0x64, 0x22, 0x3a, 0x32, 0x30, 0x30, 0x30, 0x37, 0x37, 0x37, 0x2c, 0x22,
        0x68, 0x6f, 0x6d, 0x65, 0x73, 0x74, 0x65, 0x61, 0x64, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22,
        0x3a, 0x30, 0x2c, 0x22, 0x65, 0x69, 0x70, 0x31, 0x35, 0x30, 0x42, 0x6c, 0x6f, 0x63, 0x6b,
        0x22, 0x3a, 0x30, 0x2c, 0x22, 0x65, 0x69, 0x70, 0x31, 0x35, 0x30, 0x48, 0x61, 0x73, 0x68,
        0x22, 0x3a, 0x22, 0x30, 0x78, 0x32, 0x30, 0x38, 0x36, 0x37, 0x39, 0x39, 0x61, 0x65, 0x65,
        0x62, 0x65, 0x61, 0x65, 0x31, 0x33, 0x35, 0x63, 0x32, 0x34, 0x36, 0x63, 0x36, 0x35, 0x30,
        0x32, 0x31, 0x63, 0x38, 0x32, 0x62, 0x34, 0x65, 0x31, 0x35, 0x61, 0x32, 0x63, 0x34, 0x35,
        0x31, 0x33, 0x34, 0x30, 0x39, 0x39, 0x33, 0x61, 0x61, 0x63, 0x66, 0x64, 0x32, 0x37, 0x35,
        0x31, 0x38, 0x38, 0x36, 0x35, 0x31, 0x34, 0x66, 0x30, 0x22, 0x2c, 0x22, 0x65, 0x69, 0x70,
        0x31, 0x35, 0x35, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x65, 0x69,
        0x70, 0x31, 0x35, 0x38, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x62,
        0x79, 0x7a, 0x61, 0x6e, 0x74, 0x69, 0x75, 0x6d, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a,
        0x30, 0x2c, 0x22, 0x63, 0x6f, 0x6e, 0x73, 0x74, 0x61, 0x6e, 0x74, 0x69, 0x6e, 0x6f, 0x70,
        0x6c, 0x65, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x70, 0x65, 0x74,
        0x65, 0x72, 0x73, 0x62, 0x75, 0x72, 0x67, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30,
        0x2c, 0x22, 0x69, 0x73, 0x74, 0x61, 0x6e, 0x62, 0x75, 0x6c, 0x42, 0x6c, 0x6f, 0x63, 0x6b,
        0x22, 0x3a, 0x30, 0x2c, 0x22, 0x6d, 0x75, 0x69, 0x72, 0x47, 0x6c, 0x61, 0x63, 0x69, 0x65,
        0x72, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x73, 0x75, 0x62, 0x6e,
        0x65, 0x74, 0x45, 0x56, 0x4d, 0x54, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d, 0x70, 0x22,
        0x3a, 0x30, 0x2c, 0x22, 0x66, 0x65, 0x65, 0x43, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a,
        0x7b, 0x22, 0x67, 0x61, 0x73, 0x4c, 0x69, 0x6d, 0x69, 0x74, 0x22, 0x3a, 0x32, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x2c, 0x22, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x42, 0x6c,
        0x6f, 0x63, 0x6b, 0x52, 0x61, 0x74, 0x65, 0x22, 0x3a, 0x32, 0x2c, 0x22, 0x6d, 0x69, 0x6e,
        0x42, 0x61, 0x73, 0x65, 0x46, 0x65, 0x65, 0x22, 0x3a, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x2c, 0x22, 0x74, 0x61, 0x72, 0x67, 0x65, 0x74, 0x47, 0x61, 0x73,
        0x22, 0x3a, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2c, 0x22, 0x62, 0x61,
        0x73, 0x65, 0x46, 0x65, 0x65, 0x43, 0x68, 0x61, 0x6e, 0x67, 0x65, 0x44, 0x65, 0x6e, 0x6f,
        0x6d, 0x69, 0x6e, 0x61, 0x74, 0x6f, 0x72, 0x22, 0x3a, 0x34, 0x38, 0x2c, 0x22, 0x6d, 0x69,
        0x6e, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x47, 0x61, 0x73, 0x43, 0x6f, 0x73, 0x74, 0x22, 0x3a,
        0x30, 0x2c, 0x22, 0x6d, 0x61, 0x78, 0x42, 0x6c, 0x6f, 0x63, 0x6b, 0x47, 0x61, 0x73, 0x43,
        0x6f, 0x73, 0x74, 0x22, 0x3a, 0x31, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x2c, 0x22,
        0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x47, 0x61, 0x73, 0x43, 0x6f, 0x73, 0x74, 0x53, 0x74, 0x65,
        0x70, 0x22, 0x3a, 0x35, 0x30, 0x30, 0x30, 0x30, 0x30, 0x7d, 0x2c, 0x22, 0x63, 0x6f, 0x6e,
        0x74, 0x72, 0x61, 0x63, 0x74, 0x44, 0x65, 0x70, 0x6c, 0x6f, 0x79, 0x65, 0x72, 0x41, 0x6c,
        0x6c, 0x6f, 0x77, 0x4c, 0x69, 0x73, 0x74, 0x43, 0x6f, 0x6e, 0x66, 0x69, 0x67, 0x22, 0x3a,
        0x7b, 0x22, 0x62, 0x6c, 0x6f, 0x63, 0x6b, 0x54, 0x69, 0x6d, 0x65, 0x73, 0x74, 0x61, 0x6d,
        0x70, 0x22, 0x3a, 0x30, 0x2c, 0x22, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x41, 0x64, 0x64, 0x72,
        0x65, 0x73, 0x73, 0x65, 0x73, 0x22, 0x3a, 0x5b, 0x22, 0x30, 0x78, 0x38, 0x64, 0x62, 0x39,
        0x37, 0x43, 0x37, 0x63, 0x45, 0x63, 0x45, 0x32, 0x34, 0x39, 0x63, 0x32, 0x62, 0x39, 0x38,
        0x62, 0x44, 0x43, 0x30, 0x32, 0x32, 0x36, 0x43, 0x63, 0x34, 0x43, 0x32, 0x41, 0x35, 0x37,
        0x42, 0x46, 0x35, 0x32, 0x46, 0x43, 0x22, 0x2c, 0x22, 0x30, 0x78, 0x36, 0x31, 0x33, 0x30,
        0x34, 0x30, 0x61, 0x32, 0x33, 0x39, 0x42, 0x44, 0x66, 0x43, 0x46, 0x31, 0x31, 0x30, 0x39,
        0x36, 0x39, 0x66, 0x65, 0x63, 0x42, 0x34, 0x31, 0x63, 0x36, 0x66, 0x39, 0x32, 0x45, 0x41,
        0x33, 0x35, 0x31, 0x35, 0x43, 0x30, 0x22, 0x2c, 0x22, 0x30, 0x78, 0x30, 0x61, 0x36, 0x33,
        0x61, 0x43, 0x43, 0x33, 0x37, 0x33, 0x35, 0x65, 0x38, 0x32, 0x35, 0x44, 0x37, 0x44, 0x31,
        0x33, 0x32, 0x34, 0x33, 0x46, 0x44, 0x37, 0x36, 0x62, 0x41, 0x64, 0x34, 0x39, 0x33, 0x33,
        0x31, 0x62, 0x61, 0x45, 0x30, 0x45, 0x22, 0x2c, 0x22, 0x30, 0x78, 0x32, 0x66, 0x63, 0x39,
        0x32, 0x32, 0x42, 0x65, 0x65, 0x39, 0x30, 0x32, 0x35, 0x32, 0x30, 0x63, 0x34, 0x36, 0x38,
        0x31, 0x63, 0x35, 0x62, 0x62, 0x64, 0x39, 0x37, 0x39, 0x30, 0x38, 0x43, 0x37, 0x32, 0x37,
        0x36, 0x36, 0x34, 0x65, 0x35, 0x36, 0x22, 0x2c, 0x22, 0x30, 0x78, 0x30, 0x43, 0x38, 0x35,
        0x66, 0x32, 0x37, 0x35, 0x35, 0x30, 0x63, 0x61, 0x62, 0x33, 0x31, 0x32, 0x37, 0x46, 0x42,
        0x36, 0x44, 0x61, 0x38, 0x34, 0x45, 0x36, 0x44, 0x44, 0x63, 0x65, 0x43, 0x66, 0x33, 0x34,
        0x32, 0x37, 0x32, 0x66, 0x44, 0x30, 0x22, 0x5d, 0x7d, 0x7d, 0x2c, 0x22, 0x6e, 0x6f, 0x6e,
        0x63, 0x65, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22, 0x74, 0x69, 0x6d, 0x65,
        0x73, 0x74, 0x61, 0x6d, 0x70, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22, 0x65,
        0x78, 0x74, 0x72, 0x61, 0x44, 0x61, 0x74, 0x61, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x30,
        0x22, 0x2c, 0x22, 0x67, 0x61, 0x73, 0x4c, 0x69, 0x6d, 0x69, 0x74, 0x22, 0x3a, 0x22, 0x30,
        0x78, 0x31, 0x33, 0x31, 0x32, 0x64, 0x30, 0x30, 0x22, 0x2c, 0x22, 0x64, 0x69, 0x66, 0x66,
        0x69, 0x63, 0x75, 0x6c, 0x74, 0x79, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22,
        0x6d, 0x69, 0x78, 0x48, 0x61, 0x73, 0x68, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x22, 0x2c, 0x22, 0x63, 0x6f, 0x69, 0x6e, 0x62, 0x61, 0x73, 0x65, 0x22, 0x3a, 0x22,
        0x30, 0x78, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x2c, 0x22,
        0x61, 0x6c, 0x6c, 0x6f, 0x63, 0x22, 0x3a, 0x7b, 0x22, 0x30, 0x43, 0x38, 0x35, 0x66, 0x32,
        0x37, 0x35, 0x35, 0x30, 0x63, 0x61, 0x62, 0x33, 0x31, 0x32, 0x37, 0x46, 0x42, 0x36, 0x44,
        0x61, 0x38, 0x34, 0x45, 0x36, 0x44, 0x44, 0x63, 0x65, 0x43, 0x66, 0x33, 0x34, 0x32, 0x37,
        0x32, 0x66, 0x44, 0x30, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65,
        0x22, 0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38,
        0x30, 0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, 0x2c,
        0x22, 0x30, 0x61, 0x36, 0x33, 0x61, 0x43, 0x43, 0x33, 0x37, 0x33, 0x35, 0x65, 0x38, 0x32,
        0x35, 0x44, 0x37, 0x44, 0x31, 0x33, 0x32, 0x34, 0x33, 0x46, 0x44, 0x37, 0x36, 0x62, 0x41,
        0x64, 0x34, 0x39, 0x33, 0x33, 0x31, 0x62, 0x61, 0x45, 0x30, 0x45, 0x22, 0x3a, 0x7b, 0x22,
        0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62,
        0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38, 0x30, 0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, 0x2c, 0x22, 0x32, 0x66, 0x63, 0x39, 0x32, 0x32, 0x42,
        0x65, 0x65, 0x39, 0x30, 0x32, 0x35, 0x32, 0x30, 0x63, 0x34, 0x36, 0x38, 0x31, 0x63, 0x35,
        0x62, 0x62, 0x64, 0x39, 0x37, 0x39, 0x30, 0x38, 0x43, 0x37, 0x32, 0x37, 0x36, 0x36, 0x34,
        0x65, 0x35, 0x36, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22,
        0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38, 0x30,
        0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, 0x2c, 0x22,
        0x36, 0x31, 0x33, 0x30, 0x34, 0x30, 0x61, 0x32, 0x33, 0x39, 0x42, 0x44, 0x66, 0x43, 0x46,
        0x31, 0x31, 0x30, 0x39, 0x36, 0x39, 0x66, 0x65, 0x63, 0x42, 0x34, 0x31, 0x63, 0x36, 0x66,
        0x39, 0x32, 0x45, 0x41, 0x33, 0x35, 0x31, 0x35, 0x43, 0x30, 0x22, 0x3a, 0x7b, 0x22, 0x62,
        0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37,
        0x64, 0x32, 0x64, 0x63, 0x63, 0x38, 0x30, 0x63, 0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x22, 0x7d, 0x2c, 0x22, 0x38, 0x64, 0x62, 0x39, 0x37, 0x43, 0x37, 0x63,
        0x45, 0x63, 0x45, 0x32, 0x34, 0x39, 0x63, 0x32, 0x62, 0x39, 0x38, 0x62, 0x44, 0x43, 0x30,
        0x32, 0x32, 0x36, 0x43, 0x63, 0x34, 0x43, 0x32, 0x41, 0x35, 0x37, 0x42, 0x46, 0x35, 0x32,
        0x46, 0x43, 0x22, 0x3a, 0x7b, 0x22, 0x62, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x65, 0x22, 0x3a,
        0x22, 0x30, 0x78, 0x35, 0x32, 0x62, 0x37, 0x64, 0x32, 0x64, 0x63, 0x63, 0x38, 0x30, 0x63,
        0x64, 0x32, 0x65, 0x34, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, 0x7d, 0x2c, 0x22,
        0x6e, 0x75, 0x6d, 0x62, 0x65, 0x72, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c, 0x22,
        0x67, 0x61, 0x73, 0x55, 0x73, 0x65, 0x64, 0x22, 0x3a, 0x22, 0x30, 0x78, 0x30, 0x22, 0x2c,
        0x22, 0x70, 0x61, 0x72, 0x65, 0x6e, 0x74, 0x48, 0x61, 0x73, 0x68, 0x22, 0x3a, 0x22, 0x30,
        0x78, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30,
        0x30, 0x30, 0x30, 0x30, 0x30, 0x22, 0x7d, //
        //
        // "secp256k1fx.Input" type id
        0x00, 0x00, 0x00, 0x0a, //
        //
        // secp256k1fx.Input.sig_indices.len()
        0x00, 0x00, 0x00, 0x01, //
        //
        // secp256k1fx.Input.sig_indices[0]
        0x00, 0x00, 0x00, 0x00, //
        //
        // number of credentials
        0x00, 0x00, 0x00, 0x02, //
        //
        // struct field type ID "fx::Credential.cred"
        // "secp256k1fx.Credential" type ID
        0x00, 0x00, 0x00, 0x09, //
        //
        // number of signers ("fx::Credential.cred.sigs.len()")
        0x00, 0x00, 0x00, 0x01, //
        //
        // first 65-byte signature
        0x02, 0x4d, 0xc2, 0x09, 0xa2, 0x56, 0x39, 0x8f, 0x13, 0x63, //
        0xb0, 0xb6, 0xd4, 0x67, 0x0e, 0xec, 0xac, 0x21, 0x46, 0x7f, //
        0xa5, 0xe1, 0x66, 0x12, 0xe6, 0x04, 0x5b, 0x68, 0x88, 0x1a, //
        0x6d, 0x26, 0x58, 0x04, 0x33, 0x80, 0x93, 0xa2, 0x5d, 0x8f, //
        0x2e, 0x72, 0xfa, 0x01, 0x73, 0x4f, 0x31, 0x94, 0x0c, 0x17, //
        0xc3, 0x8a, 0x55, 0xbf, 0x0b, 0x30, 0x03, 0xcf, 0xb4, 0x5a, //
        0x43, 0x93, 0xeb, 0xbe, 0x01, //
        //
        // struct field type ID "fx::Credential.cred"
        // "secp256k1fx.Credential" type ID
        0x00, 0x00, 0x00, 0x09, //
        //
        // number of signers ("fx::Credential.cred.sigs.len()")
        0x00, 0x00, 0x00, 0x01, //
        //
        // first 65-byte signature
        0x02, 0x4d, 0xc2, 0x09, 0xa2, 0x56, 0x39, 0x8f, 0x13, 0x63, //
        0xb0, 0xb6, 0xd4, 0x67, 0x0e, 0xec, 0xac, 0x21, 0x46, 0x7f, //
        0xa5, 0xe1, 0x66, 0x12, 0xe6, 0x04, 0x5b, 0x68, 0x88, 0x1a, //
        0x6d, 0x26, 0x58, 0x04, 0x33, 0x80, 0x93, 0xa2, 0x5d, 0x8f, //
        0x2e, 0x72, 0xfa, 0x01, 0x73, 0x4f, 0x31, 0x94, 0x0c, 0x17, //
        0xc3, 0x8a, 0x55, 0xbf, 0x0b, 0x30, 0x03, 0xcf, 0xb4, 0x5a, //
        0x43, 0x93, 0xeb, 0xbe, 0x01, //
    ];
    // for c in &signed_bytes {
    //     print!("{:#02x},", *c);
    // }
    assert!(cmp_manager::eq_vectors(
        expected_signed_bytes,
        &tx_bytes_with_signatures
    ));
}
