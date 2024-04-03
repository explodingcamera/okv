use crate::{tx::RedbTransaction, RedbColumn};
use okv_core::async_fallback;
use okv_core::backend::DBColumn;

async_fallback!(RedbColumn);

impl<'a> okv_core::backend_async::DBColumnAsync for RedbTransaction<'a>
where
    RedbTransaction<'a>: okv_core::backend::DBColumn,
{
    okv_core::async_fallback_impl!();
}
