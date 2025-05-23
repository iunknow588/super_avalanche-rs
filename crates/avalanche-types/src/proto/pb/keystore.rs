#[allow(clippy::wildcard_imports)]
#[allow(clippy::missing_docs_in_private_items)]
#[allow(clippy::missing_errors_doc)]
#[allow(clippy::missing_panics_doc)]
#[allow(clippy::missing_const_for_fn)]
#[allow(clippy::default_trait_access)]
#[allow(clippy::used_underscore_items)]
// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDatabaseRequest {
    #[prost(string, tag="1")]
    pub username: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub password: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDatabaseResponse {
    /// server_addr is the address of the gRPC server hosting the Database service
    #[prost(string, tag="2")]
    pub server_addr: ::prost::alloc::string::String,
}
/// Encoded file descriptor set for the `keystore` package
pub const FILE_DESCRIPTOR_SET: &[u8] = &[
    0x0a, 0xe4, 0x07, 0x0a, 0x17, 0x6b, 0x65, 0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x2f, 0x6b, 0x65,
    0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x08, 0x6b, 0x65,
    0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x22, 0x4c, 0x0a, 0x12, 0x47, 0x65, 0x74, 0x44, 0x61, 0x74,
    0x61, 0x62, 0x61, 0x73, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x12, 0x1a, 0x0a, 0x08,
    0x75, 0x73, 0x65, 0x72, 0x6e, 0x61, 0x6d, 0x65, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x08,
    0x75, 0x73, 0x65, 0x72, 0x6e, 0x61, 0x6d, 0x65, 0x12, 0x1a, 0x0a, 0x08, 0x70, 0x61, 0x73, 0x73,
    0x77, 0x6f, 0x72, 0x64, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09, 0x52, 0x08, 0x70, 0x61, 0x73, 0x73,
    0x77, 0x6f, 0x72, 0x64, 0x22, 0x3c, 0x0a, 0x13, 0x47, 0x65, 0x74, 0x44, 0x61, 0x74, 0x61, 0x62,
    0x61, 0x73, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x1f, 0x0a, 0x0b, 0x73,
    0x65, 0x72, 0x76, 0x65, 0x72, 0x5f, 0x61, 0x64, 0x64, 0x72, 0x18, 0x02, 0x20, 0x01, 0x28, 0x09,
    0x52, 0x0a, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72, 0x41, 0x64, 0x64, 0x72, 0x4a, 0x04, 0x08, 0x01,
    0x10, 0x02, 0x32, 0x56, 0x0a, 0x08, 0x4b, 0x65, 0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x12, 0x4a,
    0x0a, 0x0b, 0x47, 0x65, 0x74, 0x44, 0x61, 0x74, 0x61, 0x62, 0x61, 0x73, 0x65, 0x12, 0x1c, 0x2e,
    0x6b, 0x65, 0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x2e, 0x47, 0x65, 0x74, 0x44, 0x61, 0x74, 0x61,
    0x62, 0x61, 0x73, 0x65, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x1d, 0x2e, 0x6b, 0x65,
    0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x2e, 0x47, 0x65, 0x74, 0x44, 0x61, 0x74, 0x61, 0x62, 0x61,
    0x73, 0x65, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x42, 0x33, 0x5a, 0x31, 0x67, 0x69,
    0x74, 0x68, 0x75, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x61, 0x76, 0x61, 0x2d, 0x6c, 0x61, 0x62,
    0x73, 0x2f, 0x61, 0x76, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x68, 0x65, 0x67, 0x6f, 0x2f, 0x70, 0x72,
    0x6f, 0x74, 0x6f, 0x2f, 0x70, 0x62, 0x2f, 0x6b, 0x65, 0x79, 0x73, 0x74, 0x6f, 0x72, 0x65, 0x4a,
    0x9d, 0x05, 0x0a, 0x06, 0x12, 0x04, 0x00, 0x00, 0x16, 0x01, 0x0a, 0x08, 0x0a, 0x01, 0x0c, 0x12,
    0x03, 0x00, 0x00, 0x12, 0x0a, 0x08, 0x0a, 0x01, 0x02, 0x12, 0x03, 0x02, 0x00, 0x11, 0x0a, 0x08,
    0x0a, 0x01, 0x08, 0x12, 0x03, 0x04, 0x00, 0x48, 0x0a, 0x09, 0x0a, 0x02, 0x08, 0x0b, 0x12, 0x03,
    0x04, 0x00, 0x48, 0x0a, 0x0a, 0x0a, 0x02, 0x06, 0x00, 0x12, 0x04, 0x06, 0x00, 0x08, 0x01, 0x0a,
    0x0a, 0x0a, 0x03, 0x06, 0x00, 0x01, 0x12, 0x03, 0x06, 0x08, 0x10, 0x0a, 0x0b, 0x0a, 0x04, 0x06,
    0x00, 0x02, 0x00, 0x12, 0x03, 0x07, 0x02, 0x44, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x00,
    0x01, 0x12, 0x03, 0x07, 0x06, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x00, 0x02, 0x12,
    0x03, 0x07, 0x12, 0x24, 0x0a, 0x0c, 0x0a, 0x05, 0x06, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x07,
    0x2f, 0x42, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x00, 0x12, 0x04, 0x0a, 0x00, 0x0d, 0x01, 0x0a, 0x0a,
    0x0a, 0x03, 0x04, 0x00, 0x01, 0x12, 0x03, 0x0a, 0x08, 0x1a, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00,
    0x02, 0x00, 0x12, 0x03, 0x0b, 0x02, 0x16, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x05,
    0x12, 0x03, 0x0b, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x01, 0x12, 0x03,
    0x0b, 0x09, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00, 0x02, 0x00, 0x03, 0x12, 0x03, 0x0b, 0x14,
    0x15, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x00, 0x02, 0x01, 0x12, 0x03, 0x0c, 0x02, 0x16, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x00, 0x02, 0x01, 0x05, 0x12, 0x03, 0x0c, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x00, 0x02, 0x01, 0x01, 0x12, 0x03, 0x0c, 0x09, 0x11, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x00,
    0x02, 0x01, 0x03, 0x12, 0x03, 0x0c, 0x14, 0x15, 0x0a, 0x0a, 0x0a, 0x02, 0x04, 0x01, 0x12, 0x04,
    0x0f, 0x00, 0x16, 0x01, 0x0a, 0x0a, 0x0a, 0x03, 0x04, 0x01, 0x01, 0x12, 0x03, 0x0f, 0x08, 0x1b,
    0x0a, 0xce, 0x01, 0x0a, 0x03, 0x04, 0x01, 0x09, 0x12, 0x03, 0x13, 0x02, 0x0d, 0x1a, 0xc1, 0x01,
    0x20, 0x72, 0x65, 0x73, 0x65, 0x72, 0x76, 0x65, 0x64, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x62, 0x61,
    0x63, 0x6b, 0x77, 0x61, 0x72, 0x64, 0x20, 0x63, 0x6f, 0x6d, 0x70, 0x61, 0x74, 0x69, 0x62, 0x69,
    0x6c, 0x69, 0x74, 0x79, 0x0a, 0x20, 0x61, 0x76, 0x61, 0x6c, 0x61, 0x6e, 0x63, 0x68, 0x65, 0x67,
    0x6f, 0x20, 0x3c, 0x3d, 0x76, 0x31, 0x2e, 0x37, 0x2e, 0x39, 0x20, 0x75, 0x73, 0x65, 0x64, 0x20,
    0x74, 0x68, 0x65, 0x20, 0x66, 0x69, 0x65, 0x6c, 0x64, 0x20, 0x22, 0x31, 0x22, 0x20, 0x61, 0x73,
    0x20, 0x61, 0x6e, 0x20, 0x69, 0x64, 0x20, 0x74, 0x6f, 0x20, 0x69, 0x64, 0x65, 0x6e, 0x74, 0x69,
    0x66, 0x79, 0x20, 0x74, 0x68, 0x65, 0x20, 0x67, 0x52, 0x50, 0x43, 0x20, 0x73, 0x65, 0x72, 0x76,
    0x65, 0x72, 0x0a, 0x20, 0x61, 0x64, 0x64, 0x72, 0x65, 0x73, 0x73, 0x20, 0x77, 0x68, 0x69, 0x63,
    0x68, 0x20, 0x73, 0x65, 0x72, 0x76, 0x65, 0x64, 0x20, 0x74, 0x68, 0x65, 0x20, 0x44, 0x61, 0x74,
    0x61, 0x62, 0x61, 0x73, 0x65, 0x20, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x20, 0x76, 0x69,
    0x61, 0x20, 0x74, 0x68, 0x65, 0x20, 0x6e, 0x6f, 0x77, 0x20, 0x72, 0x65, 0x6d, 0x6f, 0x76, 0x65,
    0x64, 0x20, 0x73, 0x65, 0x72, 0x76, 0x69, 0x63, 0x65, 0x20, 0x62, 0x72, 0x6f, 0x6b, 0x65, 0x72,
    0x0a, 0x0a, 0x0b, 0x0a, 0x04, 0x04, 0x01, 0x09, 0x00, 0x12, 0x03, 0x13, 0x0b, 0x0c, 0x0a, 0x0c,
    0x0a, 0x05, 0x04, 0x01, 0x09, 0x00, 0x01, 0x12, 0x03, 0x13, 0x0b, 0x0c, 0x0a, 0x0c, 0x0a, 0x05,
    0x04, 0x01, 0x09, 0x00, 0x02, 0x12, 0x03, 0x13, 0x0b, 0x0c, 0x0a, 0x59, 0x0a, 0x04, 0x04, 0x01,
    0x02, 0x00, 0x12, 0x03, 0x15, 0x02, 0x19, 0x1a, 0x4c, 0x20, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72,
    0x5f, 0x61, 0x64, 0x64, 0x72, 0x20, 0x69, 0x73, 0x20, 0x74, 0x68, 0x65, 0x20, 0x61, 0x64, 0x64,
    0x72, 0x65, 0x73, 0x73, 0x20, 0x6f, 0x66, 0x20, 0x74, 0x68, 0x65, 0x20, 0x67, 0x52, 0x50, 0x43,
    0x20, 0x73, 0x65, 0x72, 0x76, 0x65, 0x72, 0x20, 0x68, 0x6f, 0x73, 0x74, 0x69, 0x6e, 0x67, 0x20,
    0x74, 0x68, 0x65, 0x20, 0x44, 0x61, 0x74, 0x61, 0x62, 0x61, 0x73, 0x65, 0x20, 0x73, 0x65, 0x72,
    0x76, 0x69, 0x63, 0x65, 0x0a, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x05, 0x12, 0x03,
    0x15, 0x02, 0x08, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x01, 0x12, 0x03, 0x15, 0x09,
    0x14, 0x0a, 0x0c, 0x0a, 0x05, 0x04, 0x01, 0x02, 0x00, 0x03, 0x12, 0x03, 0x15, 0x17, 0x18, 0x62,
    0x06, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x33,
];
include!("keystore.tonic.rs");
// @@protoc_insertion_point(module)