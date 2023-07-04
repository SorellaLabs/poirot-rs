use poirot_core::TracingClient;
use std::{env, error::Error, path::Path};

// reth types
use reth_primitives::BlockId;
use reth_rpc_types::trace::geth::GethDebugTracingOptions;

fn main() {
    // Create the runtime
    let runtime = tokio_runtime().expect("Failed to create runtime");

    // Use the runtime to execute the async function
    match runtime.block_on(run(runtime.handle().clone())) {
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

pub fn tokio_runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // increase stack size, mostly for RPC calls that use the evm: <https://github.com/paradigmxyz/reth/issues/3056> and  <https://github.com/bluealloy/revm/issues/305>
        .thread_stack_size(16 * 1024 * 1024)
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

    let tx_hash =
        "0x742940f6bd10a5014055eb6f940ec894b3e164b985e02655fd04ce072ba6b854".parse().unwrap();

    let tracing_opt = GethDebugTracingOptions::default();

    let tx_trace = tracer.reth_debug.debug_trace_transaction(tx_hash, tracing_opt.clone()).await?;

    // Print traces
    println!("{:?}", tx_trace);

    // Trace this mev block:
    let block_number = BlockId::from(17600791);

    // This throws InternalTracingError
    let block_parity_trace = tracer.reth_trace.trace_block(block_number).await?;

    // Print traces
    if let Some(block_trace) = block_parity_trace {
        for trace in block_trace {
            println!("{:?}", trace);
        }
    }

    let tracing_opt = GethDebugTracingOptions::default();

    // This throws InternalTracingError
    let block_trace = tracer.reth_debug.debug_trace_block(block_number, tracing_opt).await?;

    for trace in block_trace {
        println!("{:?}", trace);
    }

    // This works fine
    /*let block = match tracer.reth_api.block_transaction_count_by_number(17600791.into()).await {
            Ok(block) => block,
            Err(e) => {
                eprintln!("Failed to get block transaction count: {:?}", e);
                return Err(Box::new(e))
            }
        };

        println!("Block: {:?}", block.unwrap());

        // This works fine
        let tx_hash =
            "0xec98e974ac4bdf912236ba566bf171419e814086d2d3fb8b5e62b6e0acb5b591".parse().unwrap();

        let tx_trace = match tracer.reth_debug.raw_transaction(tx_hash).await {
            Ok(block_traces) => block_traces,
            Err(e) => {
                eprintln!("Failed to trace block: {:?}", e);
                return Err(Box::new(e))
            }
        };

        // Print traces
        println!("{:?}", tx_trace);
    */

    Ok(())
}

//TODO build trace decoder for Univ3 swaps, maybe use alloys-rs decoder have to see compat with
// reth
