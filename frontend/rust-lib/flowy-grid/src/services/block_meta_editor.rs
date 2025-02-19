use bytes::Bytes;
use flowy_error::{FlowyError, FlowyResult};
use flowy_grid_data_model::entities::{CellMeta, RowMeta, RowMetaChangeset, RowOrder};
use flowy_revision::{RevisionCloudService, RevisionCompactor, RevisionManager, RevisionObjectBuilder};
use flowy_sync::client_grid::{GridBlockMetaChange, GridBlockMetaPad};
use flowy_sync::entities::revision::Revision;
use flowy_sync::util::make_delta_from_revisions;
use lib_infra::future::FutureResult;
use lib_ot::core::PlainTextAttributes;

use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ClientGridBlockMetaEditor {
    user_id: String,
    pub block_id: String,
    pad: Arc<RwLock<GridBlockMetaPad>>,
    rev_manager: Arc<RevisionManager>,
}

impl ClientGridBlockMetaEditor {
    pub async fn new(
        user_id: &str,
        token: &str,
        block_id: &str,
        mut rev_manager: RevisionManager,
    ) -> FlowyResult<Self> {
        let cloud = Arc::new(GridBlockMetaRevisionCloudService {
            token: token.to_owned(),
        });
        let block_meta_pad = rev_manager.load::<GridBlockMetaPadBuilder>(Some(cloud)).await?;
        let pad = Arc::new(RwLock::new(block_meta_pad));
        let rev_manager = Arc::new(rev_manager);
        let user_id = user_id.to_owned();
        let block_id = block_id.to_owned();
        Ok(Self {
            user_id,
            block_id,
            pad,
            rev_manager,
        })
    }

    pub(crate) async fn create_row(&self, row: RowMeta, start_row_id: Option<String>) -> FlowyResult<i32> {
        let mut row_count = 0;
        let _ = self
            .modify(|pad| {
                let change = pad.add_row_meta(row, start_row_id)?;
                row_count = pad.number_of_rows();
                Ok(change)
            })
            .await?;

        Ok(row_count)
    }

    pub async fn delete_rows(&self, ids: Vec<String>) -> FlowyResult<i32> {
        let mut row_count = 0;
        let _ = self
            .modify(|pad| {
                let changeset = pad.delete_rows(&ids)?;
                row_count = pad.number_of_rows();
                Ok(changeset)
            })
            .await?;
        Ok(row_count)
    }

    pub async fn update_row(&self, changeset: RowMetaChangeset) -> FlowyResult<()> {
        let _ = self.modify(|pad| Ok(pad.update_row(changeset)?)).await?;
        Ok(())
    }

    pub async fn get_row_metas(&self, row_ids: Option<Vec<String>>) -> FlowyResult<Vec<Arc<RowMeta>>> {
        let row_metas = self.pad.read().await.get_row_metas(&row_ids)?;
        Ok(row_metas)
    }

    pub async fn get_cell_metas(&self, field_id: &str, row_ids: &Option<Vec<String>>) -> FlowyResult<Vec<CellMeta>> {
        let cell_metas = self.pad.read().await.get_cell_metas(field_id, row_ids)?;
        Ok(cell_metas)
    }

    pub async fn get_row_orders(&self, row_ids: &Option<Vec<String>>) -> FlowyResult<Vec<RowOrder>> {
        let row_orders = self
            .pad
            .read()
            .await
            .get_row_metas(row_ids)?
            .iter()
            .map(RowOrder::from)
            .collect::<Vec<RowOrder>>();
        Ok(row_orders)
    }

    async fn modify<F>(&self, f: F) -> FlowyResult<()>
    where
        F: for<'a> FnOnce(&'a mut GridBlockMetaPad) -> FlowyResult<Option<GridBlockMetaChange>>,
    {
        let mut write_guard = self.pad.write().await;
        match f(&mut *write_guard)? {
            None => {}
            Some(change) => {
                let _ = self.apply_change(change).await?;
            }
        }
        Ok(())
    }

    async fn apply_change(&self, change: GridBlockMetaChange) -> FlowyResult<()> {
        let GridBlockMetaChange { delta, md5 } = change;
        let user_id = self.user_id.clone();
        let (base_rev_id, rev_id) = self.rev_manager.next_rev_id_pair();
        let delta_data = delta.to_delta_bytes();
        let revision = Revision::new(
            &self.rev_manager.object_id,
            base_rev_id,
            rev_id,
            delta_data,
            &user_id,
            md5,
        );
        let _ = self
            .rev_manager
            .add_local_revision(&revision, Box::new(GridBlockMetaRevisionCompactor()))
            .await?;
        Ok(())
    }
}

struct GridBlockMetaRevisionCloudService {
    #[allow(dead_code)]
    token: String,
}

impl RevisionCloudService for GridBlockMetaRevisionCloudService {
    #[tracing::instrument(level = "trace", skip(self))]
    fn fetch_object(&self, _user_id: &str, _object_id: &str) -> FutureResult<Vec<Revision>, FlowyError> {
        FutureResult::new(async move { Ok(vec![]) })
    }
}

struct GridBlockMetaPadBuilder();
impl RevisionObjectBuilder for GridBlockMetaPadBuilder {
    type Output = GridBlockMetaPad;

    fn build_object(object_id: &str, revisions: Vec<Revision>) -> FlowyResult<Self::Output> {
        let pad = GridBlockMetaPad::from_revisions(object_id, revisions)?;
        Ok(pad)
    }
}

struct GridBlockMetaRevisionCompactor();
impl RevisionCompactor for GridBlockMetaRevisionCompactor {
    fn bytes_from_revisions(&self, revisions: Vec<Revision>) -> FlowyResult<Bytes> {
        let delta = make_delta_from_revisions::<PlainTextAttributes>(revisions)?;
        Ok(delta.to_delta_bytes())
    }
}
