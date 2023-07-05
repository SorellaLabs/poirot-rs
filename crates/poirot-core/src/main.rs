use poirot_core::TracingClient;
use std::{env, error::Error, path::Path};
use tracing_subscriber::EnvFilter;

// reth types
use reth_rpc_types::trace::geth::GethDebugTracingOptions;

fn main() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .try_init();

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

    let tx_hash =
        "0x742940f6bd10a5014055eb6f940ec894b3e164b985e02655fd04ce072ba6b854".parse().unwrap();

    let tracing_opt = GethDebugTracingOptions::default();

    let tx_trace = tracer.reth_debug.debug_trace_transaction(tx_hash, tracing_opt.clone()).await?;

    // Print traces
    println!("{:#?}", tx_trace);

    let parity_trace = tracer.reth_trace.trace_transaction(tx_hash).await?;

    // Print traces
    println!("{:#?}", parity_trace);

    Ok(())
}
