

use ethers::{
    middleware::SignerMiddleware,
    prelude::abigen,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::{Address, U256},
};
use eyre::eyre;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::sync::Arc;

/// Your private key file path.
const ENV_PRIV_KEY_PATH: &str = "PRIV_KEY_PATH";

/// Stylus RPC endpoint url.
const ENV_RPC_URL: &str = "RPC_URL";

/// Deployed pragram address.
const ENV_PROGRAM_ADDRESS: &str = "STYLUS_PROGRAM_ADDRESS";

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let priv_key_path = std::env::var(ENV_PRIV_KEY_PATH)
        .map_err(|_| eyre!("No {} env var set", ENV_PRIV_KEY_PATH))?;
    let rpc_url =
        std::env::var(ENV_RPC_URL).map_err(|_| eyre!("No {} env var set", ENV_RPC_URL))?;
    let program_address = std::env::var(ENV_PROGRAM_ADDRESS)
        .map_err(|_| eyre!("No {} env var set", ENV_PROGRAM_ADDRESS))?;
    abigen!(
        StateTransitionVerifier,
        r#"[
            function verifyProof(uint256 p_a_1, uint256 p_a_2, uint256 p_b_1_1, uint256 p_b_1_2, uint256 p_b_2_1, uint256 p_b_2_2, uint256 p_c_1, uint256 p_c_2, uint256[1] calldata signals) external pure returns (bool)
        ]"#
    );

    let provider = Provider::<Http>::try_from(rpc_url)?;
    let address: Address = program_address.parse()?;

    let privkey = read_secret_from_file(&priv_key_path)?;
    let wallet = LocalWallet::from_str(&privkey)?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    let p_a = [
        U256::from_str_radix("0x296517e2db14f86f9c994e40c3a0d72295e8468042f6d4b1a7224f0bfdab8a56", 16).unwrap(),
        U256::from_str_radix("0x011b68c331785b2226f7642ce93832e1a7fca212ad06eef60c6e710a480e9af2", 16).unwrap(),
    ];
    let p_b = [
        [
            U256::from_str_radix("0x0dc26e733798c3af232de0f8b65965736a07604f7030456cea0c506b3fb51277", 16).unwrap(),
            U256::from_str_radix("0x0d14224f52418ceff20b6c4f9e4347f3da67a296dbb75e38ff8c9559e1f05a93", 16).unwrap(),
        ],
        [
            U256::from_str_radix("0x040726334da3ff99374359aac35db4896ad28b727b276c3a19dbddd61a2c92b0", 16).unwrap(),
            U256::from_str_radix("0x15a42bb5e94ae4cf3486c31f191805c5b921c3c2bf31ac0c47fca3e00c05e0b0", 16).unwrap(),
        ]
    ];
    let p_c = [
        U256::from_str_radix("0x2beb7559796a7614b1d9069c6a96177f0de936ec9ee42c49b8a96b4d8106789d", 16).unwrap(),
        U256::from_str_radix("0x0514fd5267ccbf14439cdc308aaad3e05f03107f3b93a72a49109e480cbdb120", 16).unwrap(),
    ];
    let signals = [
        U256::from_str_radix("0x29306e9be57e23bd4e2272889892c9f78dfe082575106749a1372cea45f00d41", 16).unwrap(),
    ];

    let verifier = StateTransitionVerifier::new(address, client);
    let result = verifier.verify_proof(p_a[0], p_a[1], p_b[0][0], p_b[0][1], p_b[1][0], p_b[1][1], p_c[0], p_c[1], signals).send().await?.await?;

    println!("result: {:?}", result);
    Ok(())
}

fn read_secret_from_file(fpath: &str) -> eyre::Result<String> {
    let f = std::fs::File::open(fpath)?;
    let mut buf_reader = BufReader::new(f);
    let mut secret = String::new();
    buf_reader.read_line(&mut secret)?;
    Ok(secret.trim().to_string())
}
