use std::{
    marker::PhantomData,
    ops::{Add, Deref},
    sync::atomic::{AtomicU32, AtomicU64, Ordering},
};

use azothacore_common::{az_error, bevy_app::TokioRuntime, AzContext, AzResult};
use azothacore_database::DbDriver;
use bevy::prelude::Resource;
use sqlx::{query_scalar, Database, FromRow, Pool};
use tracing::info;

#[derive(Resource)]
pub struct IDGenerator<T, A, V>(A, PhantomData<T>, PhantomData<V>);

pub trait IDGeneratorTrait<V>: Resource {
    fn new() -> Self;
    fn set(&self, val: V);
    fn generate(&self) -> AzResult<V>;
    /// The next ID that will be returned if generate was called, but will not advance the internal generator.
    fn next_after_max_used(&self) -> V;
}

pub trait DBIDGenerator<D, V>: IDGeneratorTrait<V>
where
    (V,): for<'r> FromRow<'r, <DbDriver as Database>::Row>,
    V: Send + Unpin + Add<Output = V>,
    D: Resource + Deref<Target = Pool<DbDriver>>,
{
    const DB_SELECT_MAX_ID_QUERY: &str;

    fn new_db_generator(db: &D, rt: &TokioRuntime) -> AzResult<Self>
    where
        Self: Sized,
    {
        let this = Self::new();
        let self_type_name = std::any::type_name::<Self>();
        rt.block_on(async {
            info!(target: "server::loading", "initialising DB ID generator for {self_type_name}");
            this.set(
                query_scalar::<_, V>(Self::DB_SELECT_MAX_ID_QUERY)
                    .fetch_one(&**db)
                    .await
                    .with_context(|| format!("failed to get highest maxguid for {self_type_name}"))?,
            );
            AzResult::Ok(())
        })
        .map(|_| this)
    }
}

impl<T> IDGeneratorTrait<u32> for IDGenerator<T, AtomicU32, u32>
where
    T: Sync + Send + 'static,
{
    fn new() -> Self {
        Self(AtomicU32::new(1), Default::default(), Default::default())
    }

    fn set(&self, val: u32) {
        self.0.store(val, Ordering::SeqCst);
    }

    fn next_after_max_used(&self) -> u32 {
        self.0.load(Ordering::SeqCst)
    }

    fn generate(&self) -> AzResult<u32> {
        let new_guid = self.0.fetch_add(1, Ordering::SeqCst);
        if new_guid == 0 {
            return Err(az_error!("ID generator overflow!! Can't continue."));
        }
        Ok(new_guid)
    }
}

impl<T> IDGeneratorTrait<u64> for IDGenerator<T, AtomicU64, u64>
where
    T: Sync + Send + 'static,
{
    fn new() -> Self {
        Self(AtomicU64::new(1), Default::default(), Default::default())
    }

    fn set(&self, val: u64) {
        self.0.store(val, Ordering::SeqCst);
    }

    fn next_after_max_used(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }

    fn generate(&self) -> AzResult<u64> {
        let new_guid = self.0.fetch_add(1, Ordering::SeqCst);
        if new_guid == 0 {
            return Err(az_error!("ID generator overflow!! Can't continue."));
        }
        Ok(new_guid)
    }
}
