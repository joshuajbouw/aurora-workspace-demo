use aurora_workspace::contract::EthProverConfig;
use aurora_workspace::{types::AccountId, EvmContract, InitConfig};
use std::str::FromStr;
use workspaces::network::Sandbox;
use workspaces::types::{KeyType, SecretKey};
use workspaces::Worker;

const EVM_ACCOUNT_ID: &str = "aurora.test.near";
const AURORA_LOCAL_CHAIN_ID: u64 = 1313161556;
const OWNER_ACCOUNT_ID: &str = "owner.test.near";
const PROVER_ACCOUNT_ID: &str = "prover.test.near";

pub async fn init_and_deploy_contract(worker: &Worker<Sandbox>) -> anyhow::Result<EvmContract> {
    let sk = SecretKey::from_random(KeyType::ED25519);
    let evm_account = worker
        .create_tla(AccountId::from_str(EVM_ACCOUNT_ID)?, sk)
        .await?
        .into_result()?;
    let eth_prover_config = EthProverConfig::default();
    let init_config = InitConfig {
        owner_id: AccountId::from_str(OWNER_ACCOUNT_ID)?,
        prover_id: AccountId::from_str(PROVER_ACCOUNT_ID)?,
        eth_prover_config: Some(eth_prover_config),
        chain_id: AURORA_LOCAL_CHAIN_ID.into(),
    };
    let wasm = std::fs::read("./res/aurora-testnet.wasm")?;
    // create contract
    let contract = EvmContract::deploy_and_init(evm_account, init_config, wasm).await?;

    Ok(contract)
}
