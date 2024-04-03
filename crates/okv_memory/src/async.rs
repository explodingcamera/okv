use crate::MemDBColumn;
use okv_core::async_fallback;
use okv_core::backend::DBColumn;

async_fallback!(MemDBColumn);
