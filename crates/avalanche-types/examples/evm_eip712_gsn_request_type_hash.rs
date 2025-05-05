// 删除未使用的导入
// use std::{env::args, io};
// use avalanche_types::evm::eip712::gsn;

/// "registerRequestType(string typeName, string typeSuffix)" "my name" "my suffix"
/// cargo run --example evm_eip712_gsn_request_type_hash --features="evm" -- "my name" "my suffix"
/// ref. <https://github.com/opengsn/gsn/blob/master/packages/contracts/src/forwarder/Forwarder.sol>
fn main() {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let request_type_hash = ethers_core::types::H256::from_slice(
        &ethers_core::utils::keccak256(
            b"ForwardRequest(address from,address to,uint256 value,uint256 gas,uint256 nonce,bytes data)",
        ),
    );

    // 使用内联格式化参数
    log::info!("request type hash: {request_type_hash:x}");
}
