use std::{
    fs::{self, File},
    io::{self, BufReader, Error, ErrorKind, Read, Write},
    path::Path,
};

use rcgen::{
    date_time_ymd, BasicConstraints, Certificate, CertificateParams, CertificateSigningRequest,
    DistinguishedName, DnType, IsCa, KeyPair,
};
use rsa::{pkcs1::LineEnding, pkcs8::EncodePrivateKey, RsaPrivateKey};
use rustls_pemfile::{read_one, Item};

/// Type alias for DER-encoded private key with static lifetime.
type PrivateKeyDer = rustls::pki_types::PrivateKeyDer<'static>;

/// Type alias for DER-encoded certificate with static lifetime.
type CertificateDer = rustls::pki_types::CertificateDer<'static>;

/// Represents a certificate authority.
/// CA acts as a trusted third party.
/// See: <https://en.wikipedia.org/wiki/Certificate_authority>
/// See: <https://github.com/djc/sign-cert-remote/blob/main/src/main.rs>
pub struct Ca {
    pub cert: Certificate,
}

impl Ca {
    /// Creates a new certificate authority
    ///
    /// # Arguments
    /// * `common_name` - The common name for the certificate
    ///
    /// # Errors
    /// Returns an error if certificate generation fails
    pub fn new(common_name: &str) -> io::Result<Self> {
        let cert_params = default_params(None, Some(common_name.to_string()), true)?;
        let cert = generate(Some(cert_params))?;
        Ok(Self { cert })
    }

    /// Creates a new certificate authority with custom parameters
    ///
    /// # Arguments
    /// * `cert_params` - Optional certificate parameters
    ///
    /// # Errors
    /// Returns an error if certificate generation fails or parameters are invalid
    pub fn new_with_parameters(cert_params: Option<CertificateParams>) -> io::Result<Self> {
        let cert = generate(cert_params)?;
        Ok(Self { cert })
    }

    /// Save the certificate and private key to files
    ///
    /// # Arguments
    /// * `overwrite` - Whether to overwrite existing files
    /// * `key_path` - Optional path to save the private key
    /// * `cert_path` - Optional path to save the certificate
    ///
    /// # Errors
    /// Returns error if:
    /// * File paths already exist and overwrite is false
    /// * Failed to write files
    pub fn save(
        &self,
        overwrite: bool,
        key_path: Option<&str>,
        cert_path: Option<&str>,
    ) -> io::Result<(String, String)> {
        let key_path = if let Some(p) = key_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("key path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".key"))?
        };

        let cert_path = if let Some(p) = cert_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("cert path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".cert"))?
        };

        // ref. "crypto/tls.parsePrivateKey"
        // ref. "crypto/x509.MarshalPKCS8PrivateKey"
        let key_contents = self.cert.serialize_private_key_pem();
        let mut key_file = File::create(&key_path)?;
        key_file.write_all(key_contents.as_bytes())?;
        log::info!("saved key '{key_path}' ({}-byte)", key_contents.len());

        let cert_contents = self
            .cert
            .serialize_pem()
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {e}")))?;
        let mut cert_file = File::create(&cert_path)?;
        cert_file.write_all(cert_contents.as_bytes())?;
        log::info!("saved cert '{cert_path}' ({}-byte)", cert_contents.len());

        Ok((key_path, cert_path))
    }

    /// Issues a certificate in PEM format.
    ///
    /// # Errors
    /// Returns error if:
    /// - CSR PEM parsing fails
    /// - Certificate signing fails
    pub fn issue_cert(&self, csr_pem: &str) -> io::Result<String> {
        log::info!("issuing a cert for CSR");
        let csr = CertificateSigningRequest::from_pem(csr_pem).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed CertificateSigningRequest::from_pem {e}"),
            )
        })?;
        csr.serialize_pem_with_signer(&self.cert).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed serialize_pem_with_signer {e}"),
            )
        })
    }

    /// Issues and saves a certificate in PEM format.
    /// Returns the issued cert in PEM format, and the saved cert file path.
    ///
    /// # Errors
    /// Returns error if:
    /// - Certificate path already exists and overwrite is false
    /// - CSR parsing fails
    /// - Certificate signing fails
    /// - File operations fail
    pub fn issue_and_save_cert(
        &self,
        csr_pem: &str,
        overwrite: bool,
        cert_path: Option<&str>,
    ) -> io::Result<(String, String)> {
        let cert_path = if let Some(p) = cert_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".csr.pem"))?
        };

        log::info!("saving the issued certificate in '{cert_path}'");
        let issued_cert = self.issue_cert(csr_pem)?;
        let mut issued_cert_file = File::create(&cert_path)?;
        issued_cert_file.write_all(issued_cert.as_bytes())?;
        log::info!("saved cert '{cert_path}' ({}-byte)", issued_cert.len());

        Ok((issued_cert, cert_path))
    }
}

/// Represents a certificate signing request entity.
/// ref. <https://en.wikipedia.org/wiki/Certificate_signing_request>
/// ref. <https://github.com/djc/sign-cert-remote/blob/main/src/main.rs>
pub struct CsrEntity {
    pub cert: Certificate,
    pub csr_pem: String,
}

impl CsrEntity {
    /// Creates a new CSR with the given common name.
    ///
    /// # Errors
    /// Returns error if certificate generation fails
    pub fn new(common_name: &str) -> io::Result<Self> {
        let cert_params = default_params(None, Some(common_name.to_string()), false)?;
        let (cert, csr_pem) = generate_csr(cert_params)?;
        Ok(Self { cert, csr_pem })
    }

    /// Creates a new CSR with custom certificate parameters.
    ///
    /// # Errors
    /// Returns error if certificate generation fails
    pub fn new_with_parameters(cert_params: CertificateParams) -> io::Result<Self> {
        let (cert, csr_pem) = generate_csr(cert_params)?;
        Ok(Self { cert, csr_pem })
    }

    /// Saves the CSR to a file.
    ///
    /// # Errors
    /// Returns error if:
    /// - File already exists and overwrite is false
    /// - File operations fail
    pub fn save_csr(&self, overwrite: bool, csr_path: Option<&str>) -> io::Result<String> {
        let csr_path = if let Some(p) = csr_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".csr.pem"))?
        };

        let mut csr_file = File::create(&csr_path)?;
        csr_file.write_all(self.csr_pem.as_bytes())?;
        log::info!("saved CSR '{csr_path}' ({}-byte)", self.csr_pem.len());

        Ok(csr_path)
    }

    /// Saves all CSR components to files.
    ///
    /// # Errors
    /// Returns error if any file operation fails
    pub fn save(
        &self,
        overwrite: bool,
        csr_key_path: Option<&str>,
        csr_cert_path: Option<&str>,
        csr_path: Option<&str>,
    ) -> io::Result<(String, String, String)> {
        let csr_key_path = if let Some(p) = csr_key_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR key path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".key"))?
        };

        let csr_cert_path = if let Some(p) = csr_cert_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR cert path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".cert"))?
        };

        let csr_path = if let Some(p) = csr_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".csr.pem"))?
        };

        // ref. "crypto/tls.parsePrivateKey"
        // ref. "crypto/x509.MarshalPKCS8PrivateKey"
        let csr_key_contents = self.cert.serialize_private_key_pem();
        let mut csr_key_file = File::create(&csr_key_path)?;
        csr_key_file.write_all(csr_key_contents.as_bytes())?;
        log::info!(
            "saved key '{csr_key_path}' ({}-byte)",
            csr_key_contents.len()
        );

        let csr_cert_contents = self
            .cert
            .serialize_pem()
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {e}")))?;
        let mut cert_file = File::create(&csr_cert_path)?;
        cert_file.write_all(csr_cert_contents.as_bytes())?;
        log::info!(
            "saved cert '{csr_cert_path}' ({}-byte)",
            csr_cert_contents.len()
        );

        // ref. "crypto/tls.parsePrivateKey"
        // ref. "crypto/x509.MarshalPKCS8PrivateKey"
        let mut csr_file = File::create(&csr_path)?;
        csr_file.write_all(self.csr_pem.as_bytes())?;
        log::info!("saved CSR '{csr_path}' ({}-byte)", self.csr_pem.len());

        Ok((csr_key_path, csr_cert_path, csr_path))
    }
}

/// `RUST_LOG=debug` cargo test --all-features --lib -- `x509::test_csr` --exact
/// --show-output
#[test]
#[allow(clippy::too_many_lines)]
fn test_csr() {
    use std::process::{Command, Stdio};

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let ca = Ca::new("ca.hello.com").unwrap();
    let (ca_key_path, ca_cert_path) = ca.save(true, None, None).unwrap();
    let openssl_args = vec![
        "x509".to_string(),
        "-text".to_string(),
        "-noout".to_string(),
        "-in".to_string(),
        ca_cert_path.to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl x509 with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            println!(
                "openssl output {} bytes:\n{}\n",
                output.stdout.len(),
                String::from_utf8(output.stdout).unwrap()
            );
        }
        Err(e) => {
            log::warn!("failed to run openssl {e}");
        }
    }

    let csr_entity = CsrEntity::new("entity.hello.com").unwrap();
    log::info!("csr_entity.csr:\n\n{}", csr_entity.csr_pem);
    let (csr_key_path, csr_cert_path, csr_path) = csr_entity.save(true, None, None, None).unwrap();
    log::info!("csr_key_path: {csr_key_path}");
    log::info!("csr_cert_path: {csr_cert_path}");
    log::info!("csr_path: {csr_path}");
    let openssl_args = vec![
        "x509".to_string(),
        "-text".to_string(),
        "-noout".to_string(),
        "-in".to_string(),
        csr_cert_path.to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl x509 with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            println!(
                "openssl output {} bytes:\n{}\n",
                output.stdout.len(),
                String::from_utf8(output.stdout).unwrap()
            );
        }
        Err(e) => {
            log::warn!("failed to run openssl {e}");
        }
    }

    let issued_cert = ca.issue_cert(&csr_entity.csr_pem).unwrap();
    log::info!("issued_cert:\n\n{issued_cert}");

    let (issued_cert, issued_cert_path) = ca
        .issue_and_save_cert(&csr_entity.csr_pem, true, None)
        .unwrap();
    log::info!("issued_cert:\n\n{issued_cert}");
    log::info!("issued_cert issued_cert_path: {issued_cert_path}");
    let openssl_args = vec![
        "x509".to_string(),
        "-text".to_string(),
        "-noout".to_string(),
        "-in".to_string(),
        issued_cert_path.to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl x509 with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            println!(
                "openssl output {} bytes:\n{}\n",
                output.stdout.len(),
                String::from_utf8(output.stdout).unwrap()
            );
        }
        Err(e) => {
            log::warn!("failed to run openssl {e}");
        }
    }

    fs::remove_file(ca_key_path).unwrap();
    fs::remove_file(&ca_cert_path).unwrap();

    fs::remove_file(&csr_key_path).unwrap();
    fs::remove_file(&csr_cert_path).unwrap();
    fs::remove_file(&csr_path).unwrap();

    fs::remove_file(&issued_cert_path).unwrap();
}

/// Generates a X509 certificate pair.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/staking#NewCertAndKeyBytes>
///
/// See <https://github.com/ava-labs/avalanche-types/blob/ad1730ed193cf1cd5056f23d130c3defc897cab5/avalanche-types/src/cert.rs>
/// to use "openssl" crate.
///
/// # Errors
/// Returns error if certificate generation fails
pub fn generate(params: Option<CertificateParams>) -> io::Result<Certificate> {
    let cert_params = if let Some(p) = params {
        p
    } else {
        default_params(None, None, false)?
    };
    Certificate::from_params(cert_params).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed to generate certificate {e}"),
        )
    })
}

/// Generates a certificate and returns the certificate and the CSR.
/// ref. <https://github.com/djc/sign-cert-remote/blob/main/src/main.rs>
///
/// # Errors
/// Returns error if certificate generation or CSR serialization fails
pub fn generate_csr(params: CertificateParams) -> io::Result<(Certificate, String)> {
    let cert = generate(Some(params))?;
    let csr = cert.serialize_request_pem().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed to serialize_request_pem {e}"),
        )
    })?;
    Ok((cert, csr))
}

/// Generates a X509 certificate pair and writes them as PEM files.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/staking#NewCertAndKeyBytes>
///
/// See <https://github.com/ava-labs/avalanche-types/blob/ad1730ed193cf1cd5056f23d130c3defc897cab5/avalanche-types/src/cert.rs>
/// to use "openssl" crate.
///
/// # Errors
/// Returns error if file operations fail or certificate generation fails
pub fn generate_and_write_pem(
    params: Option<CertificateParams>,
    key_path: &str,
    cert_path: &str,
) -> io::Result<()> {
    log::info!("generating key '{key_path}', cert '{cert_path}' (PEM format)");
    if Path::new(key_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("key path '{key_path}' already exists"),
        ));
    }
    if Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("cert path '{cert_path}' already exists"),
        ));
    }

    let cert = generate(params)?;

    // ref. "crypto/tls.parsePrivateKey"
    // ref. "crypto/x509.MarshalPKCS8PrivateKey"
    let key_contents = cert.serialize_private_key_pem();
    let mut key_file = File::create(key_path)?;
    key_file.write_all(key_contents.as_bytes())?;
    log::info!("saved key '{key_path}' ({}-byte)", key_contents.len());

    let cert_contents = cert
        .serialize_pem()
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {e}")))?;

    let mut cert_file = File::create(cert_path)?;
    cert_file.write_all(cert_contents.as_bytes())?;
    log::info!("saved cert '{cert_path}' ({}-byte)", cert_contents.len());

    Ok(())
}

/// Loads the key and certificate from the PEM-encoded files.
/// # Errors
/// Returns error if file operations fail
pub fn load_pem_to_vec(key_path: &str, cert_path: &str) -> io::Result<(Vec<u8>, Vec<u8>)> {
    log::info!("loading PEM from key path '{key_path}' and cert '{cert_path}' (as PEM)");

    if !Path::new(key_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("key path '{key_path}' does not exist"),
        ));
    }
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("cert path '{cert_path}' does not exist"),
        ));
    }

    Ok((read_vec(key_path)?, read_vec(cert_path)?))
}

/// Use RSA for Apple M*.
/// ref. <https://github.com/sfackler/rust-native-tls/issues/225>
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
fn default_sig_algo() -> String {
    "PKCS_RSA_SHA256".to_string()
}

#[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
fn default_sig_algo() -> String {
    "PKCS_ECDSA_P256_SHA256".to_string()
}

/// Creates default certificate parameters with optional signature algorithm and common name.
///
/// # Arguments
/// * `sig_algo` - Optional signature algorithm name
/// * `common_name` - Optional common name for the certificate
/// * `is_ca` - Whether this is a CA certificate
///
/// # Returns
/// Returns `CertificateParams` with default values and specified options
///
/// # Errors
/// Returns error if:
/// * The signature algorithm is not supported
/// * Key pair generation fails
/// * Certificate parameter initialization fails
#[allow(clippy::needless_pass_by_value)]
pub fn default_params(
    sig_algo: Option<String>,
    common_name: Option<String>,
    is_ca: bool,
) -> io::Result<CertificateParams> {
    let mut cert_params = CertificateParams::default();

    let sa = sig_algo
        .as_ref()
        .map_or_else(default_sig_algo, ToString::to_string);
    log::info!("generating parameter with signature algorithm '{sa}'");

    let key_pair = match sa.as_str() {
        "PKCS_RSA_SHA256" => {
            cert_params.alg = &rcgen::PKCS_RSA_SHA256;
            let mut rng = rand::thread_rng();
            let private_key = RsaPrivateKey::new(&mut rng, 2048)
                .map_err(|e| Error::new(ErrorKind::Other, format!("failed to generate key {e}")))?;
            let key = private_key
                .to_pkcs8_pem(LineEnding::CRLF)
                .map_err(|e| Error::new(ErrorKind::Other, format!("failed to convert key {e}")))?;
            KeyPair::from_pem(&key).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to generate PKCS_RSA_SHA256 key pair {e}"),
                )
            })?
        }

        // this works on linux with avalanchego
        "PKCS_ECDSA_P256_SHA256" => {
            cert_params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
            KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to generate PKCS_ECDSA_P256_SHA256 key pair {e}"),
                )
            })?
        }

        // this fails avalanchego peer IP verification (e.g., incorrect signature)
        //
        // currently, "avalanchego" only signs the IP with "crypto.SHA256"
        // ref. "avalanchego/network/ip_signer.go.newIPSigner"
        // ref. "avalanchego/network/peer/ip.go UnsignedIP.Sign" with "crypto.SHA256"
        //
        // TODO: support sha384/512 signatures in avalanchego node
        "PKCS_ECDSA_P384_SHA384" => {
            cert_params.alg = &rcgen::PKCS_ECDSA_P384_SHA384;
            KeyPair::generate(&rcgen::PKCS_ECDSA_P384_SHA384).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to generate PKCS_ECDSA_P384_SHA384 key pair {e}"),
                )
            })?
        }

        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("unknown signature algorithm {sa}"),
            ))
        }
    };
    cert_params.key_pair = Some(key_pair);

    cert_params.not_before = date_time_ymd(2023, 5, 1);
    cert_params.not_after = date_time_ymd(5000, 1, 1);

    cert_params.distinguished_name = DistinguishedName::new();
    cert_params
        .distinguished_name
        .push(DnType::CountryName, "US");
    cert_params
        .distinguished_name
        .push(DnType::StateOrProvinceName, "NY");
    cert_params
        .distinguished_name
        .push(DnType::OrganizationName, "Test Org");
    if let Some(cm) = &common_name {
        cert_params
            .distinguished_name
            .push(DnType::CommonName, cm.to_string());
    } else {
        cert_params
            .distinguished_name
            .push(DnType::CommonName, "test common name");
    }

    if is_ca {
        cert_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    }

    Ok(cert_params)
}

/// `RUST_LOG=debug` cargo test --all-features --lib -- `x509::test_pem` --exact
/// --show-output
#[test]
fn test_pem() {
    use std::process::{Command, Stdio};

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let tmp_dir = tempfile::tempdir().unwrap();

    let key_path = tmp_dir.path().join(random_manager::secure_string(20));
    let key_path = key_path.as_os_str().to_str().unwrap();
    let mut key_path = String::from(key_path);
    key_path.push_str(".key");

    let cert_path = random_manager::tmp_path(10, Some(".pem")).unwrap();

    generate_and_write_pem(None, &key_path, &cert_path).unwrap();
    load_pem_to_vec(&key_path, &cert_path).unwrap();

    let key_contents = fs::read(&key_path).unwrap();
    let key_contents = String::from_utf8(key_contents).unwrap();
    log::info!("key {key_contents}");
    log::info!("key: {} bytes", key_contents.len());

    // openssl x509 -in [cert_path] -text -noout
    let cert_contents = fs::read(&cert_path).unwrap();
    let cert_contents = String::from_utf8(cert_contents).unwrap();
    log::info!("cert {cert_contents}");
    log::info!("cert: {} bytes", cert_contents.len());

    let openssl_args = vec![
        "x509".to_string(),
        "-in".to_string(),
        cert_path.to_string(),
        "-text".to_string(),
        "-noout".to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            log::info!(
                "openssl output:\n{}\n",
                String::from_utf8(output.stdout).unwrap()
            );
        }
        Err(e) => {
            log::warn!("failed to run openssl {e}");
        }
    }

    let (key, cert) = load_pem_key_cert_to_der(&key_path, &cert_path).unwrap();
    log::info!("loaded key: {key:?}");
    log::info!("loaded cert: {cert:?}");

    let serial = load_pem_cert_serial(&cert_path).unwrap();
    log::info!("serial: {serial:?}");

    fs::remove_file(&key_path).unwrap();
    fs::remove_file(&cert_path).unwrap();
}

/// Loads the TLS key and certificate from the PEM-encoded files, as DER.
/// # Errors
/// Returns error if file operations fail
/// # Panics
/// Panics if PEM parsing fails
pub fn load_pem_key_cert_to_der(
    key_path: &str,
    cert_path: &str,
) -> io::Result<(PrivateKeyDer, CertificateDer)> {
    log::info!("loading PEM from key path '{key_path}' and cert '{cert_path}' (to DER)");
    if !Path::new(key_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path {key_path} does not exist"),
        ));
    }
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path {cert_path} does not exist"),
        ));
    }

    // ref. "tls.Certificate.Leaf.Raw" in Go
    // ref. "tls.X509KeyPair"
    // ref. "x509.ParseCertificate/parseCertificate"
    // ref. "x509.Certificate.Leaf"
    //
    // use openssl::x509::X509;
    // let pub_key_contents = fs::read(cert_file_path)?;
    // let pub_key = X509::from_pem(&pub_key_contents.to_vec())?;
    // let pub_key_der = pub_key.to_der()?;
    //
    // use pem;
    // let pub_key_contents = fs::read(cert_file_path)?;
    // let pub_key = pem::parse(&pub_key_contents.to_vec()).unwrap();
    // let pub_key_der = pub_key.contents;

    let key_file = File::open(key_path)?;
    let mut reader = BufReader::new(key_file);
    let pem_read = read_one(&mut reader)?;
    let key = {
        match pem_read.unwrap() {
            Item::X509Certificate(_) => {
                log::warn!("key path {key_path} has unexpected certificate");
                None
            }
            Item::Crl(_) => {
                log::warn!("key path {key_path} has unexpected CRL");
                None
            }
            Item::Pkcs1Key(key) => {
                log::info!("loaded PKCS1 key");
                Some(PrivateKeyDer::from(key))
            }
            Item::Pkcs8Key(key) => {
                log::info!("loaded PKCS8 key");
                Some(PrivateKeyDer::from(key))
            }
            Item::Sec1Key(key) => {
                log::info!("loaded EC key");
                Some(PrivateKeyDer::from(key))
            }
            _ => None,
        }
    };
    let Some(key_der) = key else {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("key path '{key_path}' found no key"),
        ));
    };

    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    let pem_read = read_one(&mut reader)?;
    let cert = {
        match pem_read.unwrap() {
            Item::X509Certificate(cert) => Some(cert),
            Item::Pkcs1Key(_) | Item::Pkcs8Key(_) | Item::Sec1Key(_) => {
                log::warn!("cert path '{cert_path}' has unexpected private key");
                None
            }
            Item::Crl(_) => {
                log::warn!("cert path '{cert_path}' has unexpected CRL");
                None
            }
            _ => None,
        }
    };
    let Some(cert_der) = cert else {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' found no cert"),
        ));
    };

    Ok((key_der, cert_der))
}

/// Loads the serial number from the PEM-encoded certificate.
/// # Errors
/// Returns error if file operations or certificate parsing fails
pub fn load_pem_cert_serial(cert_path: &str) -> io::Result<Vec<u8>> {
    log::info!("loading PEM cert '{cert_path}'");
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' does not exists"),
        ));
    }

    let cert_raw = read_vec(cert_path)?;

    let (_, parsed) = x509_parser::pem::parse_x509_pem(&cert_raw)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed parse_x509_pem {e}")))?;
    let cert = parsed.parse_x509().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed parse_x509_certificate {e}"),
        )
    })?;
    let serial = cert.serial.clone();

    Ok(serial.to_bytes_be())
}

/// Loads a PEM certificate and converts it to DER format.
///
/// # Errors
/// Returns error if:
/// - File operations fail
/// - Certificate parsing fails
///
/// # Panics
/// Panics if PEM parsing returns invalid data
pub fn load_pem_cert_to_der(cert_path: &str) -> io::Result<CertificateDer> {
    log::info!("loading PEM cert '{cert_path}' (to DER)");
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' does not exists"),
        ));
    }

    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    let pem_read = read_one(&mut reader)?;
    let cert = {
        match pem_read.unwrap() {
            Item::X509Certificate(cert) => Some(cert),
            Item::Pkcs1Key(_) | Item::Pkcs8Key(_) | Item::Sec1Key(_) => {
                log::warn!("cert path '{cert_path}' has unexpected private key");
                None
            }
            Item::Crl(_) => {
                log::warn!("cert path '{cert_path}' has unexpected CRL");
                None
            }
            _ => None,
        }
    };

    let Some(cert_der) = cert else {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' found no cert"),
        ));
    };

    Ok(cert_der)
}

/// Generates a X509 certificate pair and returns them in DER format.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/staking#NewCertAndKeyBytes>
/// # Errors
/// Returns error if certificate generation fails
pub fn generate_der(
    params: Option<CertificateParams>,
) -> io::Result<(PrivateKeyDer, CertificateDer)> {
    log::info!("generating key and cert (DER format)");

    let cert_params = if let Some(p) = params {
        p
    } else {
        default_params(None, None, false)?
    };
    let cert = Certificate::from_params(cert_params).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed to generate certificate {e}"),
        )
    })?;
    let cert_der = cert
        .serialize_der()
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {e}")))?;
    // ref. "crypto/tls.parsePrivateKey"
    // ref. "crypto/x509.MarshalPKCS8PrivateKey"
    let key_der = cert.serialize_private_key_der();
    let key_der = rustls::pki_types::PrivatePkcs8KeyDer::from(key_der);
    let key_der = key_der.into();
    let cert_der = CertificateDer::from(cert_der);

    Ok((key_der, cert_der))
}

/// Loads the TLS key and certificate from the DER-encoded files.
/// # Errors
/// Returns error if file operations fail
pub fn load_der_key_cert(
    key_path: &str,
    cert_path: &str,
) -> io::Result<(PrivateKeyDer, CertificateDer)> {
    log::info!("loading DER from key path '{key_path}' and cert '{cert_path}'");
    load_pem_key_cert_to_der(key_path, cert_path)
}

/// `RUST_LOG=debug` cargo test --all-features --lib -- `x509::test_generate_der`
/// --exact --show-output
#[test]
fn test_generate_der() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (key, cert) = generate_der(None).unwrap();
    log::info!("key: {} bytes", key.secret_der().len());
    log::info!("cert: {} bytes", cert.len());
}

/// ref. <https://doc.rust-lang.org/std/fs/fn.read.html>
fn read_vec(p: &str) -> io::Result<Vec<u8>> {
    let mut f = File::open(p)?;
    let metadata = fs::metadata(p)?;
    let mut buffer = vec![0; usize::try_from(metadata.len()).unwrap_or_default()];
    let _read_bytes = f.read(&mut buffer)?;
    Ok(buffer)
}
