//! 验证包那一侧的真的那一份（[`VerificationPackageSide`]）：连全仓那一份判定库（`LibraryKeySpace::TheOneLibrary`）开一份
//! `VerdictStore`，待判行的扫与重读、读回装判器的材料、在库上现算判到了哪一步、把结论字编成结论帧，一律调验证包交出的那几个函数
//! （`VerdictStore::every_gpu_pending_row`、`read_gpu_pending_rows`；`crash_amplification::material_of_a_gpu_pending_stream`、
//! `judging_of_a_gpu_pending_block_in_the_library`、`conclusion_frame_of_a_gpu_pending_block`），服务不另写一份。
//! 读回的材料按（录下的流程的摘要，判表摘要）留着，同一条流的几行共用一份。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use prune_server::client::RouteAddress;
use singlefs_verification::crash_amplification::{
    conclusion_frame_of_a_gpu_pending_block, judging_of_a_gpu_pending_block_in_the_library,
    material_of_a_gpu_pending_stream, CardThatJudgedAPendingBlock, GpuPendingBlockInTheLibrary,
    GpuPendingStreamMaterial,
};
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    GpuPendingRow, LibraryKeySpace, PruneServerFailureHandling, PruneServerStorage, VerdictStore,
    VerdictStoreError,
};

use crate::gpu_judges_on_a_card::JudgeMaterialsOnACard;
use crate::library_pending_pool::{
    ConclusionFrame, PendingBlockName, PendingRowOfTheLibrary, StreamOfTheRow,
    VerificationPackageSide,
};
use crate::pending_pool::StateVerdict;
use crate::stream_identity::{StreamFacts, StreamKey};

/// 读回过的一条流：材料与服务主循环登记它要的键、事实。
struct MaterialOfAStream {
    material: Arc<GpuPendingStreamMaterial>,
    stream: StreamKey,
    facts: StreamFacts,
}

/// 连着全仓那一份判定库的验证包那一侧。
pub struct VerificationPackageOverTheOneLibrary {
    store: VerdictStore,
    materials: BTreeMap<([u8; DIGEST_BYTES], [u8; DIGEST_BYTES]), MaterialOfAStream>,
}

impl VerificationPackageOverTheOneLibrary {
    /// 连 `route` 那一份判定库服务里全仓那一份库（追加日志放 `journal_path`）。读写不成交回错，不退出进程（服务照它的退出码退）。
    ///
    /// # Errors
    /// 连不上；库的布局不是这一版、判定向量表读不成。
    pub fn open(route: RouteAddress, journal_path: PathBuf) -> Result<Self, VerdictStoreError> {
        let storage = PruneServerStorage::connect(
            route,
            journal_path,
            PruneServerFailureHandling::ReturnTheError,
            LibraryKeySpace::TheOneLibrary,
        )?;
        Ok(Self {
            store: VerdictStore::open_the_prune_server(storage)?,
            materials: BTreeMap::new(),
        })
    }

    /// 这一行那条流的材料：留着就交回留着的，没留着照验证包读回、留下。
    fn material_of(&mut self, row: &GpuPendingRow) -> Result<&MaterialOfAStream, String> {
        let key = (row.recorded_flow_digest, row.packed_judge_tables_digest);
        if !self.materials.contains_key(&key) {
            let material = material_of_a_gpu_pending_stream(&self.store, row)?;
            let stream = material.stream_key();
            let facts = material.stream_facts();
            self.materials.insert(
                key,
                MaterialOfAStream {
                    material: Arc::new(material),
                    stream,
                    facts,
                },
            );
        }
        Ok(&self.materials[&key])
    }
}

impl VerificationPackageSide for VerificationPackageOverTheOneLibrary {
    type Materials = GpuPendingStreamMaterial;

    fn pending_rows(&mut self) -> Result<Vec<PendingRowOfTheLibrary>, VerdictStoreError> {
        Ok(self
            .store
            .every_gpu_pending_row()?
            .into_iter()
            .map(|(stream, block_number, row)| PendingRowOfTheLibrary {
                name: PendingBlockName {
                    stream,
                    block_number,
                },
                row,
            })
            .collect())
    }

    fn pending_row(
        &mut self,
        name: &PendingBlockName,
    ) -> Result<Option<GpuPendingRow>, VerdictStoreError> {
        Ok(self
            .store
            .read_gpu_pending_rows(&[(name.stream, name.block_number)])?
            .pop()
            .flatten())
    }

    fn stream_of_the_row(
        &mut self,
        row: &GpuPendingRow,
    ) -> Result<StreamOfTheRow<GpuPendingStreamMaterial>, String> {
        let read_back = self.material_of(row)?;
        Ok(StreamOfTheRow {
            stream: read_back.stream,
            facts: read_back.facts,
            materials: Arc::clone(&read_back.material),
        })
    }

    fn judging_in_the_library(
        &mut self,
        row: &GpuPendingRow,
    ) -> Result<GpuPendingBlockInTheLibrary, VerdictStoreError> {
        judging_of_a_gpu_pending_block_in_the_library(&self.store, row)
    }

    fn conclusion_frame(
        &mut self,
        row: &GpuPendingRow,
        verdicts: Vec<StateVerdict>,
        card: &CardThatJudgedAPendingBlock,
    ) -> Result<ConclusionFrame, String> {
        let material = Arc::clone(&self.material_of(row)?.material);
        let frame = conclusion_frame_of_a_gpu_pending_block(
            &mut self.store,
            &material,
            row,
            verdicts,
            card,
        )?;
        Ok(ConclusionFrame {
            rows_to_put: frame.rows.rows_to_put,
            keys_to_delete: frame.rows.keys_to_delete,
            guard_key: frame.rows.guard_key,
            guard_row_as_read: frame.rows.guard_row_as_read,
            conclusion_digest: frame.conclusion_digest,
        })
    }
}
