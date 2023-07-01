use std::env;
use std::path::Path;
use std::error::Error;

use poirot_rs::TracingClient;

// reth types
use reth_primitives::BlockId;
use reth_rpc_types::trace::geth;

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
    let db_path = env::var("DB_PATH").expect("DB_PATH is not set in env");
    let db_path = Path::new(&db_path);

    // Get a handle to the current runtime.
    let handle = tokio::runtime::Handle::current();

    // Initialize TracingClient
    let tracer = TracingClient::new(&db_path, handle.clone());

    // Trace this mev block:
    let block_number = BlockId::from(17565965);
    let block_traces = tracer.reth_trace.trace_block(block_number).await?;

    // Print traces
    for trace in block_traces {
        println!("{:?}", trace);
    }

    Ok(())
}

//TODO build trace decoder for Univ3 swaps using reth & / or heimdall +
