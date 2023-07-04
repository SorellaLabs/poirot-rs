use eyre::Context;
use reth_beacon_consensus::BeaconConsensus;
use reth_blockchain_tree::{
    externals::TreeExternals, BlockchainTree, BlockchainTreeConfig, ShareableBlockchainTree,
};

// Reth
use reth_db::{
    database::{Database, DatabaseGAT},
    mdbx::{Env, WriteMap},
    tables,
    transaction::DbTx,
    DatabaseError,
};
use reth_interfaces::db::LogLevel;

use reth_network_api::test_utils::NoopNetwork;
use reth_primitives::MAINNET;
use reth_provider::{providers::BlockchainProvider, ProviderFactory};
use reth_revm::Factory;
use reth_rpc::{
    eth::{
        cache::{EthStateCache, EthStateCacheConfig},
        gas_oracle::{GasPriceOracle, GasPriceOracleConfig},
    },
    DebugApi, EthApi, EthFilter, TraceApi, TracingCallGuard,
};
use reth_tasks::TaskManager;
use reth_transaction_pool::{EthTransactionValidator, GasCostOrdering, Pool, PooledTransaction};
// Std
use std::{fmt::Debug, path::Path, sync::Arc};
use tokio::runtime::Handle;

pub type Provider = BlockchainProvider<
    Arc<Env<WriteMap>>,
    ShareableBlockchainTree<Arc<Env<WriteMap>>, Arc<BeaconConsensus>, Factory>,
>;

pub type RethTxPool =
    Pool<EthTransactionValidator<Provider, PooledTransaction>, GasCostOrdering<PooledTransaction>>;

pub type RethApi = EthApi<Provider, RethTxPool, NoopNetwork>;
pub type RethFilter = EthFilter<Provider, RethTxPool>;
pub type RethTrace = TraceApi<Provider, RethApi>;
pub type RethDebug = DebugApi<Provider, RethApi>;

pub struct TracingClient {
    pub reth_api: EthApi<Provider, RethTxPool, NoopNetwork>,
    pub reth_trace: TraceApi<Provider, RethApi>,
    pub reth_filter: EthFilter<Provider, RethTxPool>,
    pub reth_debug: DebugApi<Provider, RethApi>,
}

impl TracingClient {
    pub fn new(db_path: &Path, handle: Handle) -> Self {
        let task_manager = TaskManager::new(handle);
        let task_executor = task_manager.executor();

        let chain = MAINNET.clone();
        let db = Arc::new(init_db(db_path).unwrap());

        let tree_externals = TreeExternals::new(
            db.clone(),
            Arc::new(BeaconConsensus::new(Arc::clone(&chain))),
            Factory::new(chain.clone()),
            Arc::clone(&chain),
        );

        let tree_config = BlockchainTreeConfig::default();
        let (canon_state_notification_sender, _receiver) =
            tokio::sync::broadcast::channel(tree_config.max_reorg_depth() as usize * 2);

        let blockchain_tree = ShareableBlockchainTree::new(
            BlockchainTree::new(tree_externals, canon_state_notification_sender, tree_config)
                .unwrap(),
        );

        let provider = BlockchainProvider::new(
            ProviderFactory::new(Arc::clone(&db), Arc::clone(&chain)),
            blockchain_tree,
        )
        .unwrap();

        let state_cache = EthStateCache::spawn(provider.clone(), EthStateCacheConfig::default());
        let tx_pool = reth_transaction_pool::Pool::eth_pool(
            EthTransactionValidator::new(provider.clone(), chain, task_executor.clone(), 1),
            Default::default(),
        );

        let reth_api = EthApi::new(
            provider.clone(),
            tx_pool.clone(),
            NoopNetwork,
            state_cache.clone(),
            GasPriceOracle::new(
                provider.clone(),
                GasPriceOracleConfig::default(),
                state_cache.clone(),
            ),
        );

        let max_tracing_requests = 10;
        let tracing_call_guard = TracingCallGuard::new(max_tracing_requests);

        let reth_trace = TraceApi::new(
            provider.clone(),
            reth_api.clone(),
            state_cache.clone(),
            Box::new(task_executor.clone()),
            tracing_call_guard.clone(),
        );

        let reth_debug = DebugApi::new(
            provider.clone(),
            reth_api.clone(),
            Box::new(task_executor.clone()),
            tracing_call_guard,
        );

        let max_logs_per_response = 1000;
        let reth_filter = EthFilter::new(
            provider,
            tx_pool,
            state_cache,
            max_logs_per_response,
            Box::new(task_executor),
        );

        Self { reth_api, reth_filter, reth_trace, reth_debug }
    }
}

/// re-implementation of 'view()'
/// allows for a function to be passed in through a RO libmdbx transaction
/// /reth/crates/storage/db/src/abstraction/database.rs
pub fn view<F, T>(db: &Env<WriteMap>, f: F) -> Result<T, DatabaseError>
where
    F: FnOnce(&<Env<WriteMap> as DatabaseGAT<'_>>::TX) -> T,
{
    let tx = db.tx()?;
    let res = f(&tx);
    tx.commit()?;

    Ok(res)
}

/// Opens up an existing database at the specified path.
pub fn init_db<P: AsRef<Path> + Debug>(path: P) -> eyre::Result<Env<WriteMap>> {
    std::fs::create_dir_all(path.as_ref())?;
    println!("{:?}", path);
    let db = reth_db::mdbx::Env::<reth_db::mdbx::WriteMap>::open(
        path.as_ref(),
        reth_db::mdbx::EnvKind::RO,
        Some(LogLevel::Extra),
    )?;

    view(&db, |tx| {
        for table in tables::Tables::ALL.iter().map(|table| table.name()) {
            tx.inner.open_db(Some(table)).wrap_err("Could not open db.").unwrap();
        }
    })?;

    Ok(db)
}
