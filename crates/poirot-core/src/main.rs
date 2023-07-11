use ethers::prelude::k256::elliptic_curve::rand_core::block;
use poirot_core::{parser::Parser, TracingClient};

use poirot_core::{abi::load_all_jsonabis, action::ActionType};

use std::{env, error::Error, path::Path};

// reth types
use reth_rpc_types::trace::parity::TraceType;
use std::collections::HashSet;

use reth_primitives::{BlockId, BlockNumberOrTag};
use tracing::Subscriber;
use tracing_subscriber::{
    filter::Directive, prelude::*, registry::LookupSpan, EnvFilter, Layer, Registry,
};
// reth types
use reth_primitives::{BlockNumHash, H256};
use reth_rpc_types::trace::geth::GethDebugTracingOptions;
// alloy
use alloy_json_abi::*;
use std::str::FromStr;

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .try_init();

    // Create the runtime
    let runtime = tokio_runtime().expect("Failed to create runtime");

    let current_dir = env::current_dir().expect("Failed to get current directory");
    let abi_dir = current_dir.parent().expect("Failed to get parent directory").join("abi");

    /*match load_all_jsonabis("abi") {
        Ok(abis) => {
            for abi in abis {
                println!("Successfully loaded ABI");
            }
        }
        Err(e) => eprintln!("Failed to load ABIs: {}", e)
    } */

    // Use the runtime to execute the async function
    match runtime.block_on(run(runtime.handle().clone())) {
        Ok(()) => println!("Success!"),
        Err(e) => {
            eprintln!("Error: {:?}", e);

            let mut source: Option<&dyn Error> = e.source();
            while let Some(err) = source {
                eprintln!("Caused by: {:?}", err);
                source = err.source();
            }
        }
    }
}

pub fn tokio_runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // increase stack size, mostly for RPC calls that use the evm: <https://github.com/paradigmxyz/reth/issues/3056> and  <https://github.com/bluealloy/revm/issues/305>
        .thread_stack_size(8 * 1024 * 1024)
        .build()
}

async fn run(handle: tokio::runtime::Handle) -> Result<(), Box<dyn Error>> {
    // Read environment variables
    let db_path = match env::var("DB_PATH") {
        Ok(path) => path,
        Err(_) => return Err(Box::new(std::env::VarError::NotPresent)),
    };

    let db_path = Path::new(&db_path);

    // Initialize TracingClient
    let tracer = TracingClient::new(db_path, handle);

    // Test
    let parity_trace =
        tracer.reth_trace.trace_block(BlockId::Number(BlockNumberOrTag::Latest)).await?;

    let parser = Parser::new(parity_trace.unwrap());

    parser.store.insert(reth_primitives::H160::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f").unwrap(), std::path::PathBuf::from("../abi/uni.json"));

    for i in parser.parse() {
        match i {
            Ok(val) => println!("{val:#?}"),
            Err(_) => continue,
        }
    }

    /*

    let parity_trace =
        tracer.reth_trace.trace_block(BlockId::Number(BlockNumberOrTag::Latest)).await?;

    let parser = Parser::new(parity_trace.unwrap());

    for i in parser.parse() {
        match i.ty {
            // ActionType::Transfer(_) => println!("{i:#?}"),
            ActionType::Swap(_) => println!("{i:#?}"),
            _ => continue,
        }
    }

    // Print traces
    */

    Ok(())
}

// async fn test(tracer: &TracingClient) -> Result<(), Box<dyn std::error::Error>> {
//     let tx_hash_str = "0x6f4c57c271b9054dbe31833d20138b15838e1482384c0cd6b1be44db54805bce";
//     let tx_hash = H256::from_str(tx_hash_str)?;

//     let trace_types: HashSet<TraceType> =
//         vec![TraceType::Trace, TraceType::VmTrace, TraceType::StateDiff].into_iter().collect();

//     let trace_results = tracer.reth_trace.replay_transaction(tx_hash, trace_types).await?;

//     // Print traces
//     trace_results
//         .trace
//         .as_ref()
//         .map(|trace| {
//             trace.iter().for_each(|t| println!("{:#?}", t));
//         })
//         .unwrap_or_else(|| println!("No regular trace found for transaction."));

//     trace_results
//         .vm_trace
//         .as_ref()
//         .map(|vm_trace| {
//             println!("{:#?}", vm_trace);
//         })
//         .unwrap_or_else(|| println!("No VM trace found for transaction."));

//     trace_results
//         .state_diff
//         .as_ref()
//         .map(|state_diff| {
//             println!("{:#?}", state_diff);
//         })
//         .unwrap_or_else(|| println!("No state diff found for transaction."));

//     Ok(())
// }
