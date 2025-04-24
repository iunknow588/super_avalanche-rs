/// ref. <https://github.com/hyperium/tonic/tree/master/tonic-build>
fn main() {
    println!("cargo:rustc-env=PROTOC_FLAGS=--experimental_allow_proto3_optional");
    println!("cargo:rustc-env=PROTOC_ARG=--experimental_allow_proto3_optional");
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &[
                "../avalanchego-conformance/rpcpb/key.proto",
                "../avalanchego-conformance/rpcpb/message.proto",
                "../avalanchego-conformance/rpcpb/packer.proto",
                "../avalanchego-conformance/rpcpb/ping.proto",
            ],
            &["../avalanchego-conformance/rpcpb"],
        )
        .unwrap();
}
