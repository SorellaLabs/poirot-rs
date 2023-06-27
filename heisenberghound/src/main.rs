use ethers::{
    providers::{Http, Middleware, Provider},
    types::{BlockNumber, TraceType},
};
use std::{convert::TryInto, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // The HTTP URL for the Ethereum node you want to connect to
    let provider = Provider::<Http>::try_from("http://localhost:8545")?;

    let block_number = BlockNumber::try_from(17565965_u64)?;

    let traces = provider.trace_block(block_number).await?;

    for trace in traces {
        println!("{:#?}", trace);
    }

    let trace_type: Vec<TraceType> =
        vec![TraceType::Trace, TraceType::VmTrace, TraceType::StateDiff];
    let trace_state_diff =
        provider.trace_replay_block_transactions(block_number, trace_type).await?;

    for trace in trace_state_diff {
        println!("{:#?}", trace);
    }

    Ok(())
}
