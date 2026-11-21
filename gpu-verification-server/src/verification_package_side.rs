//! 验证包那一侧的真的那一份（[`VerificationPackageSide`]）：连一份判定库（真库 `LibraryKeySpace::TheOneLibrary`，或登记的临时库里的一段名字空间
//! `LibraryKeySpace::DisposableLibrary`）开一份
//! `VerdictStore`，待判行的扫与重读、读回装判器的材料、在库上现算判到了哪一步、把结论字编成结论帧，一律调验证包交出的那几个函数
//! （`VerdictStore::every_gpu_pending_row`、`read_gpu_pending_rows`；`crash_amplification::material_of_a_gpu_pending_stream`、
//! `judging_of_a_gpu_pending_block_in_the_library`、`conclusion_frame_of_a_gpu_pending_block`），服务不另写一份。
//! 读回的材料按（录下的流程的摘要，判表摘要）留着，同一条流的几行共用一份。
//! 核对待判行那一族同样只调验证包：`VerdictStore::every_facts_verifier_pending_row`、`read_facts_verifier_pending_rows`；
//! `crash_amplification::facts_verifier_pending_block_is_verified_in_the_library`、`material_of_a_facts_verifier_pending_block`、
//! `facts_verifier_frame_of_a_pending_block`。核对材料不留：编一块的帧时照这一块的行重读一份（验证包的帧只收照同一行读回的材料）。
//! 判表待判行那一族同样只调验证包：`VerdictStore::every_packed_tables_pending_row`、`read_packed_tables_pending_row`、
//! `packed_tables_verdict_words_rows_for_one_frame`；`crash_amplification::packed_tables_verdict_words_are_in_the_library`、
//! `read_packed_judge_tables_for_a_pending_row`、`kernel_sources_of_a_packed_tables_pending_row`。这一族的流照行现读（一张判表一条流、只判一回），内核源码照行里的旋钮栏取
//! （`crate::packed_tables_family` 文件头「内核源码」）。服务主循环里的材料是 [`MaterialsOfAServedStream`]：镜像判定那一族的那一臂与这里留着的是同一份。
//! 键空间：验证包交回的帧里的键是接上这一份键空间之后的整把键；待判池自己编的键（删待判行、强制现判凭据）调验证包带 [`VerificationPackageSide::key_space`] 这份键空间的那一把。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use prune_server::client::RouteAddress;
use singlefs_verification::crash_amplification::{
    conclusion_frame_of_a_gpu_pending_block, facts_verifier_frame_of_a_pending_block,
    facts_verifier_pending_block_is_verified_in_the_library,
    judging_of_a_gpu_pending_block_in_the_library, kernel_sources_of_a_packed_tables_pending_row,
    material_of_a_facts_verifier_pending_block, material_of_a_gpu_pending_stream,
    packed_tables_verdict_words_are_in_the_library, read_packed_judge_tables_for_a_pending_row,
    CardThatJudgedAPendingBlock, GpuPendingBlockInTheLibrary, GpuPendingStreamMaterial,
};
use singlefs_verification::sha256::DIGEST_BYTES;
use singlefs_verification::verdict_store::{
    FactsVerifierPendingRow, FactsVerifierRowsForOneFrame, GpuPendingRow, JudgingDevice,
    LibraryKeySpace, PackedTablesPendingRow, PruneServerFailureHandling, PruneServerStorage,
    VerdictStore, VerdictStoreError,
};

use crate::facts_verifier_family::{
    FactsVerifierBlockName, FactsVerifierMaterial, FactsVerifierPendingRowOfTheLibrary,
};
use crate::gpu_judges_on_a_card::{
    GpuStreamMaterials, JudgeMaterialsOnACard, MaterialsOfAServedStream,
};
use crate::library_pending_pool::{
    ConclusionFrame, PendingBlockName, PendingRowOfTheLibrary, StreamOfTheRow,
    VerificationPackageSide,
};
use crate::packed_tables_family::{
    PackedTablesFrame, PackedTablesName, PackedTablesPendingRowOfTheLibrary,
};
use crate::pending_pool::StateVerdict;
use crate::stream_identity::{StreamFacts, StreamKey};

/// 读回过的一条流：材料与服务主循环登记它要的键、事实。
struct MaterialOfAStream {
    material: Arc<GpuPendingStreamMaterial>,
    stream: StreamKey,
    facts: StreamFacts,
}

/// 键空间前缀对应的键空间：空的是真库（键原样），不空的是登记的临时库里的一段名字空间。
#[must_use]
pub fn key_space_of_the_prefix(key_space_prefix: &[u8]) -> LibraryKeySpace {
    if key_space_prefix.is_empty() {
        LibraryKeySpace::TheOneLibrary
    } else {
        LibraryKeySpace::DisposableLibrary(key_space_prefix.to_vec())
    }
}

/// 连着一份判定库（真库，或登记的临时库）的验证包那一侧。
pub struct VerificationPackageOverALibrary {
    store: VerdictStore,
    key_space: LibraryKeySpace,
    materials: BTreeMap<([u8; DIGEST_BYTES], [u8; DIGEST_BYTES]), MaterialOfAStream>,
}

impl VerificationPackageOverALibrary {
    /// 连 `route` 那一份判定库服务里 `key_space` 那一份库（追加日志放 `journal_path`）。读写不成交回错，不退出进程（服务照它的退出码退）。
    ///
    /// # Errors
    /// 连不上；判定向量表读不成。
    pub fn open(
        route: RouteAddress,
        journal_path: PathBuf,
        key_space: LibraryKeySpace,
    ) -> Result<Self, VerdictStoreError> {
        let storage = PruneServerStorage::connect(
            route,
            journal_path,
            PruneServerFailureHandling::ReturnTheError,
            key_space.clone(),
        )?;
        Ok(Self {
            store: VerdictStore::open_the_prune_server(storage)?,
            key_space,
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

impl VerificationPackageSide for VerificationPackageOverALibrary {
    type Materials = MaterialsOfAServedStream;

    fn key_space(&self) -> LibraryKeySpace {
        self.key_space.clone()
    }

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
    ) -> Result<StreamOfTheRow<MaterialsOfAServedStream>, String> {
        let read_back = self.material_of(row)?;
        Ok(StreamOfTheRow {
            stream: read_back.stream,
            facts: read_back.facts,
            materials: Arc::new(MaterialsOfAServedStream::ImageJudgement(Arc::clone(
                &read_back.material,
            ))),
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

    fn facts_verifier_pending_rows(
        &mut self,
    ) -> Result<Vec<FactsVerifierPendingRowOfTheLibrary>, VerdictStoreError> {
        Ok(self
            .store
            .every_facts_verifier_pending_row()?
            .into_iter()
            .map(
                |(stream, block_number, row)| FactsVerifierPendingRowOfTheLibrary {
                    name: FactsVerifierBlockName {
                        stream,
                        block_number,
                    },
                    row,
                },
            )
            .collect())
    }

    fn facts_verifier_pending_row(
        &mut self,
        name: &FactsVerifierBlockName,
    ) -> Result<Option<FactsVerifierPendingRow>, VerdictStoreError> {
        Ok(self
            .store
            .read_facts_verifier_pending_rows(&[(name.stream, name.block_number)])?
            .pop()
            .flatten())
    }

    fn facts_verifier_block_is_verified(
        &mut self,
        row: &FactsVerifierPendingRow,
    ) -> Result<bool, VerdictStoreError> {
        facts_verifier_pending_block_is_verified_in_the_library(&self.store, row)
    }

    fn facts_verifier_material(
        &mut self,
        row: &FactsVerifierPendingRow,
    ) -> Result<Arc<dyn FactsVerifierMaterial>, String> {
        let material: Arc<dyn FactsVerifierMaterial> = Arc::new(
            material_of_a_facts_verifier_pending_block(&self.store, row)?,
        );
        Ok(material)
    }

    fn facts_verifier_frame(
        &mut self,
        name: &FactsVerifierBlockName,
        row: &FactsVerifierPendingRow,
        bits_of_each_state: &[u32],
    ) -> Result<FactsVerifierRowsForOneFrame, String> {
        let material = material_of_a_facts_verifier_pending_block(&self.store, row)?;
        facts_verifier_frame_of_a_pending_block(
            &self.store,
            &material,
            &name.stream,
            name.block_number,
            row,
            bits_of_each_state,
        )
    }

    fn packed_tables_pending_rows(
        &mut self,
    ) -> Result<Vec<PackedTablesPendingRowOfTheLibrary>, VerdictStoreError> {
        Ok(self
            .store
            .every_packed_tables_pending_row()?
            .into_iter()
            .map(|(key, row)| PackedTablesPendingRowOfTheLibrary {
                name: PackedTablesName::of_the_key(&key),
                row,
            })
            .collect())
    }

    fn packed_tables_pending_row(
        &mut self,
        name: &PackedTablesName,
    ) -> Result<Option<PackedTablesPendingRow>, VerdictStoreError> {
        self.store
            .read_packed_tables_pending_row(&name.library_key())
    }

    fn packed_tables_verdict_words_are_in_the_library(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
    ) -> Result<bool, VerdictStoreError> {
        packed_tables_verdict_words_are_in_the_library(&self.store, &name.library_key(), row)
    }

    fn packed_tables_stream(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
    ) -> Result<StreamOfTheRow<MaterialsOfAServedStream>, String> {
        let key = name.library_key();
        let kernel_sources = kernel_sources_of_a_packed_tables_pending_row(&key, row)?;
        let tables = read_packed_judge_tables_for_a_pending_row(&self.store, &key, row)?;
        let materials = GpuStreamMaterials {
            tables,
            kernel_sources,
        };
        Ok(StreamOfTheRow {
            stream: materials.stream_key(),
            facts: materials.stream_facts(),
            materials: Arc::new(MaterialsOfAServedStream::PackedTables(materials)),
        })
    }

    fn packed_tables_frame(
        &mut self,
        name: &PackedTablesName,
        row: &PackedTablesPendingRow,
        verdicts: &[StateVerdict],
        device: &JudgingDevice,
    ) -> Result<PackedTablesFrame, String> {
        let frame = self
            .store
            .packed_tables_verdict_words_rows_for_one_frame(
                &name.library_key(),
                row,
                verdicts,
                device,
            )
            .map_err(|error| format!("{error:?}"))?;
        Ok(PackedTablesFrame {
            rows_to_put: frame.rows_to_put,
            verdict_words_row_key: frame.verdict_words_row_key,
            verdict_words_row_as_read: frame.verdict_words_row_as_read,
            pending_row_key_to_delete: frame.pending_row_key_to_delete,
            pending_row_as_read: frame.pending_row_as_read,
        })
    }
}
