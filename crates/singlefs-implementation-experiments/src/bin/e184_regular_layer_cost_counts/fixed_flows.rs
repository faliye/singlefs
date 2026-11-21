use super::*;

// ───────────────────────────── 固定流：覆盖写链三档与冒烟层四条 ─────────────────────────────

/// 覆盖写链每次覆盖写的数据单元数（登记 5.1：1、4、16 三档；4 是主取样，1 与 16 是方向相反的取样点）。
const OVERWRITE_CHAIN_UNITS_PER_OVERWRITE: [usize; 3] = [1, 4, 16];

/// 一条手写流在内存里摆好的样子（基镜像是建好的池，不是重放 mkfs 的写）。
pub(super) struct PreparedFlow {
    pub(super) family: &'static str,
    pub(super) tier: &'static str,
    pub(super) label: String,
    pub(super) ending: String,
    pub(super) base: MemoryPool,
    pub(super) writes: Vec<RetainedWrite>,
    pub(super) segments: Vec<Vec<usize>>,
    pub(super) judged_root_index: usize,
    pub(super) versions_with_a_file: Vec<PublishedVersion>,
    pub(super) versions_without_a_file: Vec<PublishedVersionWithoutAFile>,
}

impl PreparedFlow {
    /// 转储：基镜像存成「池里写过的每个扇区各一次整扇区写」（`base_writes_of_the_pool`），叠回同样大小的空盘得到同一个池。
    /// 手写流不记 mkfs 占了录制流开头的几步（记 0）。
    pub(super) fn dump(&self) -> FlowDump {
        FlowDump {
            family: self.family.to_string(),
            label: self.label.clone(),
            tier: self.tier.to_string(),
            seed: None,
            ending: self.ending.clone(),
            device_bytes: self.base.device_size_in_bytes,
            device_identities: self
                .base
                .devices
                .keys()
                .map(|identity| identity.0)
                .collect(),
            operations_of_the_make_filesystem: 0,
            base_writes: write_dump::base_writes_of_the_pool(&self.base),
            writes: self.writes.clone(),
            segments: self.segments.clone(),
            judged_root_index: Some(self.judged_root_index),
            versions_with_a_file: self.versions_with_a_file.clone(),
            versions_without_a_file: self.versions_without_a_file.clone(),
            compaction_operations: None,
        }
    }
}

fn smoke_flow(
    label: &str,
    base: MemoryPool,
    writes: Vec<RetainedWrite>,
    segments: Vec<Vec<usize>>,
    judged_root_index: usize,
    versions: Vec<PublishedVersion>,
) -> PreparedFlow {
    PreparedFlow {
        family: "smoke",
        tier: "baseline",
        label: format!("smoke-{label}"),
        ending: "NotAHistory".to_string(),
        base,
        writes,
        segments,
        judged_root_index,
        versions_with_a_file: versions,
        versions_without_a_file: Vec::new(),
    }
}

/// `crash-case:new-pool-file-creation-stream`：被判的版本就是被判的那条根槽写出的那一代，内容是第一个文件（测试文件 `single_version` 的做法）。
pub(super) fn prepared_new_pool_stream() -> PreparedFlow {
    let prepared = new_pool_file_creation_stream::prepare_through_the_first_file("e184-new-pool");
    let (instance, checkpoint_txg) = root_identity_written_by(
        prepared.writes[prepared.root_index]
            .bytes()
            .expect("根槽写是普通写，带着字节"),
    );
    let versions = vec![PublishedVersion {
        instance,
        checkpoint_txg,
        content: experiment_pool::file_content(),
    }];
    smoke_flow(
        "new_pool_file_creation_stream",
        prepared.base,
        prepared.writes,
        prepared.segments,
        prepared.root_index,
        versions,
    )
}

/// `crash-case:overwrite-fixed-script-stream`：到 E 再正常卸载那一支。
fn prepared_fixed_script_stream() -> PreparedFlow {
    let prepared = fixed_script_stream::prepare_through_the_unmount("e184-fixed-script");
    smoke_flow(
        "overwrite_fixed_script_stream",
        prepared.base,
        prepared.writes,
        prepared.segments,
        prepared.judged_root_index,
        prepared.versions,
    )
}

/// `crash-case:multi-record-publish-stream`。
fn prepared_multi_record_stream() -> PreparedFlow {
    let (base, writes, segments, judged_root_index, versions) =
        multi_record_publish_stream::prepare_through_the_third_publish("e184-multi-record");
    smoke_flow(
        "multi_record_publish_stream",
        base,
        writes,
        segments,
        judged_root_index,
        versions,
    )
}

/// `crash-case:snapshot-creation-stream`：建快照流照 [`snapshot_creation_flow::prepare_the_snapshot_creation_flow`] 摆。
pub(super) fn prepared_snapshot_creation_stream() -> PreparedFlow {
    let prepared =
        snapshot_creation_flow::prepare_the_snapshot_creation_flow("e184-snapshot-creation");
    PreparedFlow {
        versions_without_a_file: prepared.versions_without_a_file,
        ..smoke_flow(
            "snapshot_creation_stream",
            prepared.base,
            prepared.writes,
            prepared.segments,
            prepared.judged_root_index,
            prepared.versions_with_a_file,
        )
    }
}

/// 覆盖写：把现行那一版的内容换成 `content`，`units` 是这份内容占几个数据单元。
/// 一个数据单元走 `publish_overwrite`（与验证包测试搭建 `common_crash_points::overwrite_step` 同一个发布调用，只是不设崩溃点）；
/// 多个数据单元走 [`multi_record_steps::sequential_write`]（`publish_sequential_write`，录崩溃点那一路的 `sequential_write_step` 调的也是它）：
/// `publish_overwrite` 的契约是一个数据单元（`crates/singlefs-core/src/transaction.rs` 的 `publish_overwrite` 文档注释「契约是一个数据单元」，
/// 装不进报 `ContentExceedsDataUnit`），多个数据单元的整份内容替换在 crates 里只有顺序写这一个入口（登记第十二节修订 6）。
fn overwrite_the_first_file(
    pool: &mut experiment_pool::BuiltPool,
    content: &[u8],
    units: usize,
) -> TransactionOutput {
    if units == 1 {
        let publish_parameters = experiment_pool::parameters();
        let devices = pool.devices.as_mut().expect("镜像还开着");
        let mut writer = PoolWriter::new(&publish_parameters, devices.as_mut_slice());
        publish_overwrite(
            &mut writer,
            &mut pool.allocator,
            &pool.output,
            FirstFile {
                content,
                write_time_seconds: experiment_pool::FIXED_WRITE_TIME_SECONDS + 60,
            },
            InstanceGeneration(1),
        )
        .expect("覆盖写")
    } else {
        multi_record_steps::sequential_write(pool, content)
    }
}

/// mkfs 之后录制流里的写数（屏障不算）。
fn writes_after_the_make_filesystem(pool: &experiment_pool::BuiltPool) -> usize {
    pool.retained_operations()[pool.mkfs_operation_count..]
        .iter()
        .filter(|operation| operation.operation.kind != RecordedOperationKind::Barrier)
        .count()
}

/// 臂 F：标准前缀（取号、暖机、第一个文件 A）之后对第一个文件逐次覆盖写，第 k 次的内容是 `content_needing(units, k)`，写数到 1000 为止。
/// 一个数据单元的一档走 `publish_overwrite`，多个数据单元的两档走顺序写（原因见 [`overwrite_the_first_file`]）。交回流与覆盖写的次数。
fn prepared_overwrite_chain(units: usize) -> (PreparedFlow, usize) {
    let mut pool = experiment_pool::build_pool(&format!(
        "e184-overwrite_chain-{units:02}_units_per_overwrite"
    ));
    let first = pool.output.clone();
    let mut versions = vec![PublishedVersion {
        instance: first.root.instance,
        checkpoint_txg: first.root.checkpoint_txg,
        content: experiment_pool::file_content(),
    }];
    let mut overwrites = 0usize;
    while writes_after_the_make_filesystem(&pool) < WRITES_OF_A_THOUSAND_WRITE_STREAM {
        overwrites += 1;
        let content = multi_record_steps::content_needing(units, overwrites);
        let output = overwrite_the_first_file(&mut pool, &content, units);
        versions.push(PublishedVersion {
            instance: output.root.instance,
            checkpoint_txg: output.root.checkpoint_txg,
            content,
        });
        pool.output = output;
    }
    let base = pool.memory_pool_after_mkfs();
    let (writes, segments) = writes_and_segments(
        &pool.retained_operations()[pool.mkfs_operation_count..],
        &experiment_pool::geometry(),
    );
    let judged_root_index = writes
        .iter()
        .rposition(|write| write.kind == StepKind::RootRecordFua)
        .expect("至少一条根槽写");
    let flow = PreparedFlow {
        family: "overwrite_chain",
        tier: "candidate",
        label: format!("overwrite_chain-{units:02}_units_per_overwrite"),
        ending: format!("NotAHistory:overwrites={overwrites}"),
        base,
        writes,
        segments,
        judged_root_index,
        versions_with_a_file: versions,
        versions_without_a_file: Vec::new(),
    };
    (flow, overwrites)
}

/// 固定流的种类：覆盖写链按每次覆盖写的数据单元数，冒烟层四条各一个。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FixedFlowKind {
    OverwriteChain { units: usize },
    NewPoolFileCreation,
    FixedScript,
    MultiRecordPublish,
    SnapshotCreation,
}

impl FixedFlowKind {
    /// 覆盖写链三档在前，冒烟层四条在后。
    fn every_fixed_flow_kind() -> Vec<Self> {
        OVERWRITE_CHAIN_UNITS_PER_OVERWRITE
            .iter()
            .map(|units| FixedFlowKind::OverwriteChain { units: *units })
            .chain([
                FixedFlowKind::NewPoolFileCreation,
                FixedFlowKind::FixedScript,
                FixedFlowKind::MultiRecordPublish,
                FixedFlowKind::SnapshotCreation,
            ])
            .collect()
    }

    /// 流名，也是转储文件的主干名。
    pub(super) fn label(self) -> String {
        match self {
            FixedFlowKind::OverwriteChain { units } => {
                format!("overwrite_chain-{units:02}_units_per_overwrite")
            }
            FixedFlowKind::NewPoolFileCreation => "smoke-new_pool_file_creation_stream".to_string(),
            FixedFlowKind::FixedScript => "smoke-overwrite_fixed_script_stream".to_string(),
            FixedFlowKind::MultiRecordPublish => "smoke-multi_record_publish_stream".to_string(),
            FixedFlowKind::SnapshotCreation => "smoke-snapshot_creation_stream".to_string(),
        }
    }

    /// 建出这条流；覆盖写链另交回覆盖写的次数。
    pub(super) fn prepare_the_flow(self) -> (PreparedFlow, Option<usize>) {
        match self {
            FixedFlowKind::OverwriteChain { units } => {
                let (flow, overwrites) = prepared_overwrite_chain(units);
                (flow, Some(overwrites))
            }
            FixedFlowKind::NewPoolFileCreation => (prepared_new_pool_stream(), None),
            FixedFlowKind::FixedScript => (prepared_fixed_script_stream(), None),
            FixedFlowKind::MultiRecordPublish => (prepared_multi_record_stream(), None),
            FixedFlowKind::SnapshotCreation => (prepared_snapshot_creation_stream(), None),
        }
    }
}

/// 登记 5.2「转储往返」在手写流上的形态：直接用建好的池当基镜像排的块键，与写转储、读回、由转储重建基镜像再排的块键逐个相同，
/// 基镜像本身逐扇区相同。整条流态数装不进 u64、或块数上界超过 [`MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP`] 排不了（两份计划要同时摆在内存里）时，只比基镜像。
fn direct_base_control(prepared: &PreparedFlow, dump: &FlowDump) -> Result<ControlReading, String> {
    direct_base_control_under(prepared, dump, MOST_BLOCKS_COMPARED_BY_THE_ROUND_TRIP)
}

/// [`direct_base_control`]，比块键的块数上界上限由调用方给。
fn direct_base_control_under(
    prepared: &PreparedFlow,
    dump: &FlowDump,
    most_blocks: u64,
) -> Result<ControlReading, String> {
    let read_back = FlowDump::from_bytes(&dump.to_bytes())?;
    let rebuilt_base = read_back.base_pool();
    let bases_equal = rebuilt_base == prepared.base;
    let keys_equal =
        if state_count_to_plan(&prepared.base, dump, Expansion::Full, most_blocks).is_some() {
            let direct = plan_blocks_under(
                &prepared.base,
                dump,
                Expansion::Full,
                default_item(),
                most_blocks,
            )?;
            let through_the_dump = plan_blocks_under(
                &rebuilt_base,
                &read_back,
                Expansion::Full,
                default_item(),
                most_blocks,
            )?;
            Some(direct.blocks == through_the_dump.blocks)
        } else {
            None
        };
    Ok(ControlReading {
        kind: "dump_round_trip_direct_base",
        flow: dump.label.clone(),
        passed: bases_equal && keys_equal != Some(false),
        detail: format!(
            "bases_equal={bases_equal} keys_equal={}",
            keys_equal.map_or_else(|| "not_planned".to_string(), |equal| equal.to_string())
        ),
    })
}

/// `record-fixed-chains`：覆盖写链三档与冒烟层四条各写一份转储；已有写完的转储整份跳过。
pub(super) fn run_record_fixed_chains_command(arguments: &[String]) -> i32 {
    let parsed = match ParsedOptions::parse(arguments, &["--out"], &[]) {
        Ok(parsed) => parsed,
        Err(message) => return usage_error(&message),
    };
    let out_directory = match parsed.single("--out") {
        Ok(out) => PathBuf::from(out),
        Err(message) => return usage_error(&message),
    };
    if let Err(error) = std::fs::create_dir_all(&out_directory) {
        return usage_error(&format!("建 {} 失败：{error}", out_directory.display()));
    }
    let mut output = Output::new();
    let (mut recorded, mut skipped, mut failed, mut controls_failed) =
        (0usize, 0usize, 0usize, 0usize);
    for kind in FixedFlowKind::every_fixed_flow_kind() {
        let label = kind.label();
        let path = out_directory.join(format!("{label}{DUMP_FILE_SUFFIX}"));
        if let Ok(bytes) = std::fs::read(&path) {
            if FlowDump::from_bytes(&bytes).is_ok() {
                skipped += 1;
                output.emit(&format!(
                    "name=chain_skipped label={label} sha256={}",
                    sha256_hexadecimal(&bytes)
                ));
                continue;
            }
        }
        let started = Instant::now();
        let built = catch_unwind(AssertUnwindSafe(
            || -> Result<(PreparedFlow, Option<usize>, FlowDump, ControlReading), String> {
                let (prepared, overwrites) = kind.prepare_the_flow();
                let dump = prepared.dump();
                let control = direct_base_control(&prepared, &dump)?;
                Ok((prepared, overwrites, dump, control))
            },
        ));
        let (prepared, overwrites, dump, control) = match built {
            Err(payload) => {
                failed += 1;
                output.emit(&format!(
                    "name=chain_failed label={label} panic={}",
                    panic_text_of(payload.as_ref())
                ));
                continue;
            }
            Ok(Err(message)) => {
                failed += 1;
                output.emit(&format!("name=chain_failed label={label} reason={message}"));
                continue;
            }
            Ok(Ok(built)) => built,
        };
        let bytes = dump.to_bytes();
        if let Err(message) = write_dump::write_bytes_atomically(&path, &bytes) {
            failed += 1;
            output.emit(&format!("name=chain_failed label={label} reason={message}"));
            continue;
        }
        recorded += 1;
        controls_failed += usize::from(!control.passed);
        output.emit(&format!(
            "name=chain_recorded label={label} family={} overwrites={} writes={} segments={} base_sectors={} sha256={} seconds={:.1}",
            prepared.family,
            overwrites.map_or_else(|| "none".to_string(), |count| count.to_string()),
            dump.writes.len(),
            dump.segments.len(),
            dump.base_writes.len(),
            sha256_hexadecimal(&bytes),
            started.elapsed().as_secs_f64()
        ));
        output.emit(&control.body());
    }
    output.emit_closing_line("chains_done", &format!("recorded={recorded} skipped={skipped} failed={failed} controls_failed={controls_failed}"));
    if failed > 0 {
        EXIT_CODE_STOP_CLAUSE_FAILED
    } else if controls_failed > 0 {
        EXIT_CODE_POSITIVE_CONTROL_FAILED
    } else {
        EXIT_CODE_OK
    }
}

#[cfg(test)]
mod tests;
