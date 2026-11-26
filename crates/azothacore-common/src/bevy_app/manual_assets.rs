use std::{
    collections::BTreeMap,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    path::{Path, PathBuf},
};

use bevy::{ecs::system::SystemParam, prelude::*};
use tokio::task::{JoinError, JoinHandle};
use tracing::error;

use crate::{
    AzResult,
    az_error,
    bevy_app::{TokioRuntime, az_startup_succeeded},
};

// TODO: with the worldserver vmap work, order the systems that read models after
// `update_finish_load()`. Until then a finished async load shows up one frame late.
#[derive(SystemSet)]
pub enum ManualAssetSet<A> {
    UpdateFinishLoad(PhantomData<A>),
}

impl<A> ManualAssetSet<A> {
    pub fn update_finish_load() -> Self {
        Self::UpdateFinishLoad(PhantomData::<A>)
    }
}

impl<A> Clone for ManualAssetSet<A> {
    fn clone(&self) -> Self {
        match self {
            Self::UpdateFinishLoad(p) => Self::UpdateFinishLoad(*p),
        }
    }
}

impl<A> PartialEq for ManualAssetSet<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::UpdateFinishLoad(l0), Self::UpdateFinishLoad(r0)) => l0 == r0,
        }
    }
}

impl<A> Debug for ManualAssetSet<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ctype = std::any::type_name::<A>();
        match self {
            Self::UpdateFinishLoad(_) => f.debug_tuple("UpdateFinishLoad").field(&ctype).finish(),
        }
    }
}

impl<A> Hash for ManualAssetSet<A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl<A> Eq for ManualAssetSet<A> {}

pub fn manual_asset_plugin<A>(app: &mut App)
where
    A: ManualAsset,
{
    app.init_resource::<ManualAssetIds<A>>().add_systems(
        Update,
        finish_manual_loads::<A>
            .run_if(az_startup_succeeded())
            .in_set(ManualAssetSet::<A>::update_finish_load()),
    );
}

pub fn finish_manual_loads<A: ManualAsset>(mut assets_handler: ManualAssetsHandler<A>) {
    let done: Vec<_> = assets_handler
        .manual_asset_ids
        .pending
        .iter()
        .filter(|(_, e)| matches!(e, (_, t) if t.is_finished()))
        .map(|(p, _)| p.clone())
        .collect();

    for path in done {
        let Some((h, task)) = assets_handler.manual_asset_ids.pending.remove(&path) else {
            continue;
        };
        if let Err(e) = assets_handler.finish_pending(&path, h, assets_handler.rt.block_on(task)) {
            error!(target:"assets", cause=?e, ?path, "error loading asset");
        }
    }
}

#[derive(Resource)]
struct ManualAssetIds<A: Asset> {
    loaded:  BTreeMap<PathBuf, AssetId<A>>,
    /// Pending with strong handle here to stop Bevy from freeing
    /// the reserved slot before we insert into it.
    pending: BTreeMap<PathBuf, (Handle<A>, JoinHandle<AzResult<A>>)>,
}

impl<A: Asset> Default for ManualAssetIds<A> {
    fn default() -> Self {
        Self {
            loaded:  BTreeMap::new(),
            pending: BTreeMap::new(),
        }
    }
}

#[derive(SystemParam)]
pub struct ManualAssetsHandler<'w, A>
where
    A: ManualAsset,
{
    assets:           ResMut<'w, Assets<A>>,
    manual_asset_ids: ResMut<'w, ManualAssetIds<A>>,
    rt:               Res<'w, TokioRuntime>,
}

pub trait ManualAsset: Asset {
    fn load_asset(path: &Path) -> AzResult<Self>
    where
        Self: std::marker::Sized;
}

impl<'w, A> ManualAssetsHandler<'w, A>
where
    A: ManualAsset,
{
    pub fn get(&self, h: impl Into<AssetId<A>>) -> Option<&A> {
        self.assets.get(h)
    }

    pub fn load<P: AsRef<Path>>(&mut self, path: P, load_async: bool) -> AzResult<Handle<A>> {
        let path = path.as_ref().to_path_buf();

        if let Some((h, task)) = self.manual_asset_ids.pending.get_mut(&path) {
            if load_async && !task.is_finished() {
                // Returns immediately with the handle if the task is still running
                return Ok(h.clone());
            }
            let join_res = self.rt.block_on(task);
            let h = h.clone();
            return self.finish_pending(path, h, join_res);
        }

        if let Some(id) = self.manual_asset_ids.loaded.get_mut(&path) {
            if let Some(h) = self.assets.get_strong_handle(*id) {
                return Ok(h);
            }
            self.manual_asset_ids.pending.remove(&path);
            self.manual_asset_ids.loaded.remove(&path);
        }
        if load_async {
            let h = self.assets.reserve_handle();
            let path_cloned = path.clone();
            let task = self.rt.spawn_blocking(move || A::load_asset(&path_cloned));
            self.manual_asset_ids.pending.insert(path.clone(), (h.clone(), task));
            return Ok(h);
        }

        let res = A::load_asset(&path)?;
        let h = self.assets.add(res);
        self.manual_asset_ids.loaded.insert(path.clone(), h.id());
        Ok(h)
    }

    fn finish_pending<P: AsRef<Path>>(&mut self, path: P, h: Handle<A>, join_res: Result<AzResult<A>, JoinError>) -> AzResult<Handle<A>> {
        match join_res.map_err(|e| az_error!(e)).and_then(|r| r.map_err(|e| az_error!(e))) {
            Ok(new_asset) => {
                let id = h.id();
                self.manual_asset_ids.pending.remove(path.as_ref());
                self.manual_asset_ids.loaded.insert(path.as_ref().to_path_buf(), id);
                let _ = self.assets.insert(id, new_asset);
                Ok(h)
            },
            Err(e) => {
                self.manual_asset_ids.pending.remove(path.as_ref());
                self.manual_asset_ids.loaded.remove(path.as_ref());
                Err(e)
            },
        }
    }
}
