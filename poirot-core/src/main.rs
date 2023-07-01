use std::{env, error::Error, path::Path};

use poirot_rs::TracingClient;

// reth types
use reth_primitives::BlockId;
use reth_rpc_api::EthApiServer;
use reth_rpc_types::trace::geth::GethDebugTracingOptions;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => println!("Success!"),
        Err(e) => {
            eprintln!("Error: {:?}", e);

            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("Caused by: {:?}", err);
                source = err.source();
            }
        }
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    // Read environment variables
    let db_path = match env::var("DB_PATH") {
        Ok(path) => path,
        Err(_) => return Err(Box::new(std::env::VarError::NotPresent)),
    };
    let db_path = Path::new(&db_path);

    // Get a handle to the current runtime.
    let handle = tokio::runtime::Handle::current();

    // Initialize TracingClient
    let tracer = TracingClient::new(db_path, handle.clone());

    // Trace this mev block:
    let block_number = BlockId::from(17600791);

    let block = match tracer.reth_api.block_transaction_count_by_number(17600791.into()).await {
        Ok(block) => block,
        Err(e) => {
            eprintln!("Failed to get block transaction count: {:?}", e);
            return Err(Box::new(e))
        }
    };

    println!("Block: {:?}", block.unwrap());

    let tracing_opt = GethDebugTracingOptions::default();
    let block_traces1 = match tracer.reth_debug.debug_trace_block(block_number, tracing_opt).await {
        Ok(block_traces) => block_traces,
        Err(e) => {
            eprintln!("Failed to trace block: {:?}", e);
            return Err(Box::new(e))
        }
    };

    // Print traces
    for trace in block_traces1 {
        println!("{:?}", trace);
    }

    /*let block_traces = tracer.reth_trace.trace_block(block_number).await?;

    // Print traces
    for trace in block_traces {
        println!("{:?}", trace);
    } */

    Ok(())
}

//TODO build trace decoder for Univ3 swaps using reth & / or heimdall +
