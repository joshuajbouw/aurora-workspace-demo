use aurora_workspace::types::output::TransactionStatus;
use aurora_workspace_demo::common;
use ethabi::Constructor;
use ethereum_tx_sign::{LegacyTransaction, Transaction};
use std::fs::File;

const ETH_RANDOM_HEX_PATH: &str = "./res/Random.hex";
const ETH_RANDOM_ABI_PATH: &str = "./res/Random.abi";
const PRIVATE_KEY: [u8; 32] = [88u8; 32];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a sandbox environment.
    let worker = workspaces::sandbox().await?;
    // Init and deploy the Aurora EVM in sandbox.
    let evm = common::init_and_deploy_contract(&worker).await?;

    // Set the contract.
    let contract = {
        let abi = File::open(ETH_RANDOM_ABI_PATH)?;
        let code = hex::decode(std::fs::read(ETH_RANDOM_HEX_PATH)?)?;
        EthContract::new(abi, code)
    };

    // Create a deploy transaction and sign it.
    let signed_deploy_tx = {
        let deploy_tx = contract.deploy_transaction(0, &[]);
        let ecdsa = deploy_tx.ecdsa(&PRIVATE_KEY).unwrap();
        deploy_tx.sign(&ecdsa)
    };

    // Submit the transaction and get the ETH address.
    let address = match evm
        .as_account()
        .submit(signed_deploy_tx)
        .max_gas()
        .transact()
        .await?
        .into_value()
        .into_result()?
    {
        TransactionStatus::Succeed(bytes) => {
            let mut address_bytes = [0u8; 20];
            address_bytes.copy_from_slice(&bytes);
            address_bytes
        }
        _ => panic!("Ahhhhhh"),
    };
    let random_contract = Random::new(contract, address);

    // Fast forward a few blocks...
    worker.fast_forward(10).await?;

    // Create a call to the Random contract and loop!
    for x in 0..20 {
        let random_tx = random_contract.random_seed_transaction(x);
        let ecdsa = random_tx.ecdsa(&PRIVATE_KEY).unwrap();
        let signed_random_tx = random_tx.sign(&ecdsa);
        if let TransactionStatus::Succeed(bytes) = evm
            .as_account()
            .submit(signed_random_tx)
            .max_gas()
            .transact()
            .await?
            .into_value()
            .into_result()?
        {
            println!("Random seed: {}", hex::encode(bytes));
        };
    }

    Ok(())
}

struct Random {
    contract: EthContract,
    address: [u8; 20],
}

impl Random {
    pub fn new(contract: EthContract, address: [u8; 20]) -> Self {
        Self { contract, address }
    }

    pub fn random_seed_transaction(&self, nonce: u128) -> LegacyTransaction {
        let data = self
            .contract
            .abi
            .function("randomSeed")
            .unwrap()
            .encode_input(&[])
            .unwrap();

        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            to: Some(self.address),
            value: Default::default(),
            data,
            gas: u64::MAX as u128,
        }
    }
}

struct EthContract {
    abi: ethabi::Contract,
    code: Vec<u8>,
}

impl EthContract {
    pub fn new(abi_file: File, code: Vec<u8>) -> Self {
        Self {
            abi: ethabi::Contract::load(abi_file).unwrap(),
            code,
        }
    }

    pub fn deploy_transaction(&self, nonce: u128, args: &[ethabi::Token]) -> LegacyTransaction {
        let data = self
            .abi
            .constructor()
            .unwrap_or(&Constructor { inputs: vec![] })
            .encode_input(self.code.clone(), args)
            .unwrap();

        LegacyTransaction {
            chain: 1313161556,
            nonce,
            gas_price: Default::default(),
            to: None,
            value: Default::default(),
            data,
            gas: u64::MAX as u128,
        }
    }
}
