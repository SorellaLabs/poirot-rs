use ethers::{
    providers::{Http, Middleware, Provider},
    types::{BlockNumber, TraceType},
};

use crate::TraceClient;



pub fn into_addr(url: &str, port: &str) -> String {
    format!("{}:{}", url, port)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let block_number = BlockNumber::try_from(17565965)?;

    let tracer: TraceClient = TraceClient::new("http://localhost:8545".to_string());



    //let block = provider.get_block(block_number).await?;

    //println!("Block: {:?}", block);
    //let traces = provider.trace_block(block_number).await?;

    /*print!("Tracing block");
    for trace in traces {
        println!("{:#?}", trace);
    }
    */
    let trace_type: Vec<TraceType> =
        vec![TraceType::Trace, TraceType::VmTrace, TraceType::StateDiff];
    let trace_state_diff =
        provider.trace_replay_block_transactions(block_number, trace_type).await?;

    for trace in trace_state_diff {
        println!("{:#?}", trace);
    }

    Ok(())
}

//TODO build trace decoder for Univ3 swaps using reth & / or heimdall +
