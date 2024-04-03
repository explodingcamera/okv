use crate::{
    tx::RocksDBTransaction, RocksDbColumn, RocksDbOptimisticColumn, RocksDbPessimisticColumn,
};
use okv_core::backend::DBColumn;
use okv_core::{async_fallback, async_fallback_impl, backend_async::DBColumnAsync};

async_fallback!(RocksDbOptimisticColumn);
async_fallback!(RocksDbPessimisticColumn);
async_fallback!(RocksDbColumn);

impl<'a, DB> DBColumnAsync for RocksDBTransaction<'a, DB>
where
    RocksDBTransaction<'a, DB>: okv_core::backend::DBColumn,
{
    async_fallback_impl!();
}
