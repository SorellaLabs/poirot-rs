use ethers::{
    providers::{Http, Middleware, Provider},
    types::{BlockNumber, TraceType},
};
use std::{convert::TryInto, error::Error, env};


pub fn into_addr(url: &str, port: &str) -> String {
    format!("{}:{}", url, port)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // The HTTP URL for the Ethereum node you want to connect to
    let url = into_addr(
        &env::var("RETH_URL").expect("RETH_URL not found in .env"),
        &env::var("RETH_PORT").expect("RETH_PORT not found in .env"),
    );
    let provider = Provider::<Http>::try_from(&url).unwrap();

    println!("Connected to Ethereum node: {}", url);
    let block_number = BlockNumber::try_from(17565965_u64)?;
    let block_number1 = BlockNumber::try_from(1_u64)?;


    let traces = provider.trace_block(block_number1).await?;
    print!("Tracing block");
    for trace in traces {
        println!("{:#?}", trace);
    }

    let traces = provider.trace_block(block_number).await?;
    print!("Tracing block");
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

