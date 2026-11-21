use super::*;

// ============================================================================================
// E33：H2 的容量格与记录格（登记 X2–X10、5.2）。文件后端稀疏镜像，一格一轮一个进程、一片结果文件。
// ============================================================================================

/// 数据单元的载荷容量：D18（块里携带什么信息） 已定项 11 登记表码 1 那一行「明文头 105 / 含预留位 134」，单元 32768
/// ⇒ 32768 − 134 = 32634；记录格顺序写 n 个单元的内容长度按它算。回比 `singlefs_core::unit::data_unit_payload_capacity`（`h2_payload_constant` 行）。
const DATA_UNIT_PAYLOAD_BYTES: u64 = 32634;
pub(super) const TEBIBYTE: u64 = 1 << 40;
/// 容量阶梯（登记 5.2、X3）。
pub(super) const CAPACITY_LADDER_DEVICE_BYTES: [u64; 5] = [
    4 * GIBIBYTE,
    64 * GIBIBYTE,
    TEBIBYTE,
    4 * TEBIBYTE,
    16 * TEBIBYTE - MEBIBYTE,
];
/// 记录格：C = 4 GiB 上首个文件之后顺序写的单元数（登记 5.2）。
pub(super) const RECORD_CELL_SEQUENTIALLY_WRITTEN_UNIT_COUNTS: [u64; 3] = [0, 1024, 10240];
/// 几何敏感性（登记 8.2）：记录格 n = 10240 那一状态在这三档容量上再求一次 b。
pub(super) const OCCUPANCY_SENSITIVITY_DEVICE_BYTES: [u64; 3] =
    [4 * GIBIBYTE, 64 * GIBIBYTE, TEBIBYTE];
pub(super) const OCCUPANCY_SENSITIVITY_UNITS: u64 = 10240;
/// 每格 5 个进程（5 轮），每个进程里 X4、X5 各 5 次（登记 X4、X5、5.2）。
pub(super) const CAPACITY_CELL_ROUNDS: u64 = 5;
pub(super) const MEASURED_CALLS_PER_ROUND: u64 = 5;
/// PC-平（登记 5.3）：量窗里分配并丢掉一份 1 MiB。
const FLAT_CONTROL_BYTES: usize = 1 << 20;
/// X5：覆盖写第一个文件第 0 个数据单元以内的 4096 字节。
const MEASURED_OVERWRITE_BYTES: usize = 4096;

/// 一个格：盘容量与首个文件之后顺序写的单元数（容量格是 0）。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct CapacityCell {
    pub(super) device_bytes: u64,
    pub(super) sequentially_written_units: u64,
}

impl CapacityCell {
    pub(super) fn name(self) -> String {
        format!(
            "h2-device-{}-mebibytes-units-{}",
            self.device_bytes / MEBIBYTE,
            self.sequentially_written_units
        )
    }
}

/// 全部格，次序固定：容量阶梯（n = 0），记录格里 n > 0 的两格，几何敏感性里还没出现过的两格。同一格只出现一次（4 GiB、n = 0 既是容量格也是记录格）。
pub(super) fn capacity_cells() -> Vec<CapacityCell> {
    let ladder = CAPACITY_LADDER_DEVICE_BYTES.map(|device_bytes| CapacityCell {
        device_bytes,
        sequentially_written_units: 0,
    });
    let records = RECORD_CELL_SEQUENTIALLY_WRITTEN_UNIT_COUNTS.map(|units| CapacityCell {
        device_bytes: 4 * GIBIBYTE,
        sequentially_written_units: units,
    });
    let occupancy = OCCUPANCY_SENSITIVITY_DEVICE_BYTES.map(|device_bytes| CapacityCell {
        device_bytes,
        sequentially_written_units: OCCUPANCY_SENSITIVITY_UNITS,
    });
    let mut cells: Vec<CapacityCell> = Vec::new();
    for cell in ladder.into_iter().chain(records).chain(occupancy) {
        if !cells.contains(&cell) {
            cells.push(cell);
        }
    }
    cells
}

/// 全部片，轮在外层（第 1 轮的每一格、第 2 轮的每一格……）：挂钟随时间的漂移摊到每一格上，不集中在某一格。
pub(super) fn capacity_shards() -> Vec<(String, CapacityCell, u64)> {
    (1..=CAPACITY_CELL_ROUNDS)
        .flat_map(|round| {
            capacity_cells()
                .into_iter()
                .map(move |cell| (format!("{}-round-{round}", cell.name()), cell, round))
        })
        .collect()
}

pub(super) fn capacity_shard_by_name(name: &str) -> Option<(CapacityCell, u64)> {
    capacity_shards()
        .into_iter()
        .find(|(shard, _, _)| shard == name)
        .map(|(_, cell, round)| (cell, round))
}

/// 跑的次序（登记 r2 X23）：一轮之内盘容量从大到小、同容量里顺序写的单元数从大到小（16 TiB − 1 MiB 那一格最先）；轮在外层。
/// 服务撞上限整片乘 1.5 重排，大的先跑，抬够之后同一次尝试里余下的格吃同一个上限。合并读片仍照 [`capacity_shards`] 的次序。
pub(super) fn capacity_shards_in_running_order() -> Vec<(String, CapacityCell, u64)> {
    let mut cells = capacity_cells();
    cells.sort_by(|left, right| right.cmp(left));
    (1..=CAPACITY_CELL_ROUNDS)
        .flat_map(|round| {
            cells
                .clone()
                .into_iter()
                .map(move |cell| (format!("{}-round-{round}", cell.name()), cell, round))
        })
        .collect()
}

/// 跑这一片的那一台（登记 r2 X24）：只认这两个写法，不写主机名与地址。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShardHost {
    ThisMachine,
    SecondMachine,
}

impl ShardHost {
    pub(super) fn parse(text: &str) -> Option<ShardHost> {
        match text {
            "本机" => Some(ShardHost::ThisMachine),
            "第二台" => Some(ShardHost::SecondMachine),
            _ => None,
        }
    }

    pub(super) fn text(self) -> &'static str {
        match self {
            ShardHost::ThisMachine => "本机",
            ShardHost::SecondMachine => "第二台",
        }
    }
}

/// 跑法脚本交给一片的四样：编出这个进程的程序文件的 sha256，与登记 r2 E42 加的三样（X24、X26）——台、服务的哪一片哪一次尝试
/// （`<片号>/<片数>:<第几次>`）、镜像目录所在的文件系统类型。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CapacityShardContext {
    pub(super) binary_sha256: String,
    pub(super) host: ShardHost,
    pub(super) service_attempt: String,
    pub(super) image_filesystem: String,
}

impl CapacityShardContext {
    /// 三个值都要有、不带空白（产物行按空白切字段）；服务的尝试写成 `<片号>/<片数>:<第几次>`、三段都是数。
    pub(super) fn parse(
        binary_sha256: &str,
        host: &str,
        service_attempt: &str,
        image_filesystem: &str,
    ) -> Option<Self> {
        let host = ShardHost::parse(host)?;
        let (shard_of_shards, attempt) = service_attempt.split_once(':')?;
        let (shard_number, shard_count) = shard_of_shards.split_once('/')?;
        let all_numbers = [shard_number, shard_count, attempt]
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()));
        let filesystem_is_one_word =
            !image_filesystem.is_empty() && !image_filesystem.chars().any(char::is_whitespace);
        (all_numbers && filesystem_is_one_word).then(|| CapacityShardContext {
            binary_sha256: binary_sha256.to_string(),
            host,
            service_attempt: service_attempt.to_string(),
            image_filesystem: image_filesystem.to_string(),
        })
    }
}

/// X5 第 `call_index` 次覆盖写的内容：4096 字节，随轮次与序号变（同 E156 `overwrite_content` 的造法；首个文件是 0 号）。
fn measured_overwrite_content(round: u64, call_index: u64) -> Vec<u8> {
    let salt = usize::try_from(1 + round * MEASURED_CALLS_PER_ROUND + call_index).expect("序号小");
    (0..MEASURED_OVERWRITE_BYTES)
        .map(|index| u8::try_from((index + salt * 7 + 3) % 251).expect("小于 256"))
        .collect()
}

/// 记录格顺序写的内容：`units` 个单元的载荷，随字节序号变。
fn sequential_write_content(units: u64) -> Vec<u8> {
    let length =
        usize::try_from(units * DATA_UNIT_PAYLOAD_BYTES).expect("至多 10240 个单元，装得进 usize");
    (0..length)
        .map(|index| u8::try_from((index / 4096 + index) % 251).expect("小于 256"))
        .collect()
}

/// 一个量窗（登记 X6、X7）：被测进程里 `Instant` 包住那一调；开始时把峰值重置成此刻的在用字节 B₀，结束读峰值 P，报 P − B₀、B₀ 与结束时的在用字节。
/// 在用字节与峰值读的是发起量窗的这条线程自己的那一份（[`CountingAllocator`] 的线程本地记账），别的线程在窗里分、放的不进读数；
/// 被测的那一调自己不起线程（登记第十四节：各片里的装置是单线程；`singlefs-core` 与 `singlefs-format` 的源码里没有起线程的调用），所以单线程进程里它与全进程的读数相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MeasuredWindow {
    pub(super) nanoseconds: u64,
    pub(super) peak_over_baseline_bytes: u64,
    pub(super) baseline_bytes: u64,
    pub(super) in_use_after_bytes: u64,
}

impl MeasuredWindow {
    pub(super) fn text(&self) -> String {
        format!(
            "nanoseconds={} peak_over_baseline_bytes={} baseline_bytes={} in_use_after_bytes={}",
            self.nanoseconds,
            self.peak_over_baseline_bytes,
            self.baseline_bytes,
            self.in_use_after_bytes
        )
    }
}

fn measure_in_a_window<Output>(
    allocator: &CountingAllocator,
    call: impl FnOnce() -> Output,
) -> (Output, MeasuredWindow) {
    let baseline_bytes = allocator.thread_in_use();
    allocator.reset_thread_peak_to_thread_in_use();
    let started = Instant::now();
    let output = call();
    let nanoseconds = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let window = MeasuredWindow {
        nanoseconds,
        peak_over_baseline_bytes: allocator.thread_peak().saturating_sub(baseline_bytes),
        baseline_bytes,
        in_use_after_bytes: allocator.thread_in_use(),
    };
    (output, window)
}

/// 这个进程建的镜像文件：进程结束（正常返回或 panic 展开）时删掉；删不掉不报（镜像在草稿目录里，跑法脚本起片之前按片名清一遍）。
struct ImageFilesRemovedOnDrop {
    paths: Vec<PathBuf>,
}

impl Drop for ImageFilesRemovedOnDrop {
    fn drop(&mut self) {
        for path in &self.paths {
            let removed = std::fs::remove_file(path);
            std::hint::black_box(removed.is_ok());
        }
    }
}

type FilePool = Vec<(DeviceIdentity, FileBackedBlockDevice)>;

/// 镜像文件名：片名加进程号（同一片被杀之后重起不会撞上一份没删的镜像，`create_image_file_exclusively` 不许接着用旧盘面）。
fn capacity_image_path(image_directory: &Path, shard: &str, device_number: u32) -> PathBuf {
    image_directory.join(format!(
        "e183-{shard}-process-{}-device-{device_number}.img",
        std::process::id()
    ))
}

/// 报错前面加上是哪一步：`<步名>:<原因>`。
pub(super) fn failed_at(step: &'static str) -> impl Fn(String) -> String {
    move |reason| format!("{step}:{reason}")
}

/// 测量那一刻之前的几步：mkfs → 可写挂载 → 首个文件 →（记录格）顺序写 n 个单元 → 正常卸载 → 可写挂载（登记 5.2）。
/// 哪一步没成交回 `Err("<步名>:<原因>")`，后面不走。
fn prepare_the_measured_mount(
    cell: CapacityCell,
    parameters: &MakeFilesystemParameters,
    pool: &mut FilePool,
) -> Result<Mounted, String> {
    guarded(|| {
        make_filesystem(parameters, pool)
            .map(|_| ())
            .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("make_filesystem"))?;
    let mounted = guarded(|| {
        mount_writable(parameters, pool).map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_mount"))?;
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    let first_file = first_file_content();
    let first = guarded(|| {
        let mut writer = PoolWriter::new(parameters, pool.as_mut_slice());
        publish_first_file(
            &mut writer,
            &mut session.allocator,
            session.current.root(),
            FirstFile {
                content: &first_file,
                write_time_seconds: FIXED_WRITE_TIME_SECONDS,
            },
            session.instance,
            session.current.record_bytes(),
        )
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("first_file"))?;
    session.current = PoolVersion::of_a_version_of_trees(first);
    if cell.sequentially_written_units > 0 {
        let content = sequential_write_content(cell.sequentially_written_units);
        guarded(|| {
            session
                .publish_user_change(
                    pool,
                    UserChange::SequentialWrite(FirstFile {
                        content: &content,
                        write_time_seconds: FIXED_WRITE_TIME_SECONDS + 1,
                    }),
                )
                .map(|_| ())
                .map_err(|error| variant_name_of(&format!("{error:?}")))
        })
        .map_err(failed_at("sequential_write"))?;
    }
    guarded(|| {
        unmount(
            parameters,
            pool,
            &mut session.allocator,
            &mut session.current,
            session.shadow_ledger,
        )
        .map(|_| ())
        .map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("unmount"))?;
    guarded(|| {
        mount_writable(parameters, pool).map_err(|error| variant_name_of(&format!("{error:?}")))
    })
    .map_err(failed_at("measured_mount"))
}

/// 一片的结局：做完（片文件照写，没走到测量的那几步也算做完，合并时按「那一格这一轮作废」处置）；停机（片文件不写，退 3）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ShardEnd {
    Finished,
    Stopped { reason: String },
}

/// 一片 H2：PC-堆（前后各读一次进程高水位，PC-高水位）、载荷常量回比（不等 ⇒ 停机 T2）、建两块镜像（建不出记「那一台文件系统拒」，r2 X22 第四类）、
/// 走到测量那一刻、X8、X4 五次、PC-平、X5 五次；片做完（哪一步没成也算做完）都以进程高水位那一行收尾（r2 X22 的 H）。
pub(super) fn run_capacity_cell(
    shard: &str,
    cell: CapacityCell,
    round: u64,
    image_directory: &Path,
    context: &CapacityShardContext,
    emitter: &mut Emitter,
) -> ShardEnd {
    run_capacity_cell_with_a_journal_ring(
        shard,
        cell,
        round,
        JOURNAL_RING_DEFAULT_BYTES,
        image_directory,
        context,
        emitter,
    )
}

/// 同 [`run_capacity_cell`]，环长由调用方给：产物取 768 MiB（登记 5.2），单测在小盘、小环上走同一条路。
/// 片做完时末尾打 `h2_process`（进程收尾时的 VmHWM）：r2 X22 说「片文件做完而 H > T下沿」就是「内存上限之外」，不论走没走到测量，所以每条做完的路都打。
fn run_capacity_cell_with_a_journal_ring(
    shard: &str,
    cell: CapacityCell,
    round: u64,
    journal_ring_bytes: u64,
    image_directory: &Path,
    context: &CapacityShardContext,
    emitter: &mut Emitter,
) -> ShardEnd {
    let end = measure_a_capacity_cell(
        shard,
        cell,
        round,
        journal_ring_bytes,
        image_directory,
        context,
        emitter,
    );
    match end {
        ShardEnd::Finished => emitter.emit(&format!(
            "name=h2_process shard={shard} vm_high_water_mark_kibibytes={}",
            optional_number_text(vm_high_water_mark_kibibytes())
        )),
        ShardEnd::Stopped { .. } => {}
    }
    end
}

/// [`run_capacity_cell_with_a_journal_ring`] 的本体：打到 X5 五次为止，哪一步没成就停在那一步（片仍算做完）。
fn measure_a_capacity_cell(
    shard: &str,
    cell: CapacityCell,
    round: u64,
    journal_ring_bytes: u64,
    image_directory: &Path,
    context: &CapacityShardContext,
    emitter: &mut Emitter,
) -> ShardEnd {
    let unit_area_start = unit_area_first_and_last_slot(cell.device_bytes, journal_ring_bytes).0;
    let slots_of_the_unit_area = unit_area_slots(cell.device_bytes, journal_ring_bytes);
    emitter.emit(&format!(
        "name=h2_config shard={shard} device_bytes={} sequentially_written_units={} round={round} journal_ring_bytes={journal_ring_bytes} slots_per_region={SLOTS_PER_REGION_AT_MAKE_FILESYSTEM} unit_area_slots={slots_of_the_unit_area} binary_sha256={} host={} service_attempt={} image_filesystem={}",
        cell.device_bytes, cell.sequentially_written_units, context.binary_sha256, context.host.text(), context.service_attempt, context.image_filesystem,
    ));
    let implementation_payload = u64::try_from(singlefs_core::unit::data_unit_payload_capacity())
        .expect("载荷容量装得进 u64");
    emitter.emit(&format!(
        "name=h2_payload_constant shard={shard} local={DATA_UNIT_PAYLOAD_BYTES} implementation={implementation_payload} equal={}",
        implementation_payload == DATA_UNIT_PAYLOAD_BYTES
    ));
    if implementation_payload != DATA_UNIT_PAYLOAD_BYTES {
        return ShardEnd::Stopped {
            reason: "t2_payload_constant_differs".to_string(),
        };
    }
    let high_water_mark_before_kibibytes = vm_high_water_mark_kibibytes();
    let heap = heap_positive_control(&COUNTING_ALLOCATOR, HEAP_POSITIVE_CONTROL_SLACK_BYTES);
    let high_water_mark_after_kibibytes = vm_high_water_mark_kibibytes();
    emitter.emit(&format!(
        "name=positive_control_heap shard={shard} peak_over_baseline_bytes={} holds={} high_water_mark_before_kibibytes={} high_water_mark_after_kibibytes={} high_water_mark_control_holds={}",
        heap.peak_over_baseline_bytes,
        heap.holds,
        optional_number_text(high_water_mark_before_kibibytes),
        optional_number_text(high_water_mark_after_kibibytes),
        high_water_mark_before_kibibytes
            .zip(high_water_mark_after_kibibytes)
            .map_or_else(
                || "none".to_string(),
                |(before_kibibytes, after_kibibytes)| {
                    high_water_mark_control_holds(before_kibibytes, after_kibibytes).to_string()
                }
            ),
    ));
    let mut images = ImageFilesRemovedOnDrop { paths: Vec::new() };
    let mut pool: FilePool = Vec::new();
    for device_number in 0..2u32 {
        let path = capacity_image_path(image_directory, shard, device_number);
        match FileBackedBlockDevice::create_image_file_exclusively(
            &path,
            cell.device_bytes,
            PhysicalBlockSizeInBytes(PHYSICAL_BLOCK_BYTES),
        ) {
            Ok(device) => {
                images.paths.push(path);
                pool.push((DeviceIdentity(device_number), device));
            }
            Err(error) => {
                emitter.emit(&format!(
                    "name=h2_steps shard={shard} outcome=filesystem_refused:{}",
                    variant_name_of(&format!("{error:?}"))
                ));
                return ShardEnd::Finished;
            }
        }
    }
    let parameters =
        make_filesystem_parameters(journal_ring_bytes, SLOTS_PER_REGION_AT_MAKE_FILESYSTEM);
    let mounted = match prepare_the_measured_mount(cell, &parameters, &mut pool) {
        Ok(mounted) => mounted,
        Err(failed_step) => {
            emitter.emit(&format!(
                "name=h2_steps shard={shard} outcome=failed_at:{failed_step}"
            ));
            return ShardEnd::Finished;
        }
    };
    emitter.emit(&format!("name=h2_steps shard={shard} outcome=ok"));
    let records = mounted.allocator.records().len();
    let (allocator_copy, copy_window) =
        measure_in_a_window(&COUNTING_ALLOCATOR, || mounted.allocator.clone());
    drop(allocator_copy);
    let copy_bytes = copy_window
        .in_use_after_bytes
        .saturating_sub(copy_window.baseline_bytes);
    emitter.emit(&format!(
        "name=h2_allocator_copy shard={shard} records={records} copy_bytes={copy_bytes} copy_nanoseconds={} bytes_per_slot_per_device_millionths={}",
        copy_window.nanoseconds,
        u128::from(copy_bytes) * 1_000_000 / u128::from(POOL_DEVICE_COUNT * slots_of_the_unit_area),
    ));
    let generation = CheckpointTxg(mounted.current.root().checkpoint_txg.0 + 1);
    for call_index in 0..MEASURED_CALLS_PER_ROUND {
        let mut copy = mounted.allocator.clone();
        let (placement, window) =
            measure_in_a_window(&COUNTING_ALLOCATOR, || copy.allocate_user_data(generation));
        drop(copy);
        emitter.emit(&format!(
            "name=h2_allocate shard={shard} call={call_index} {} slots_from_unit_area_start={}",
            window.text(),
            placement.map_or_else(
                || "refused".to_string(),
                |placed| placed.slot.0.saturating_sub(unit_area_start).to_string()
            ),
        ));
    }
    let ((), flat_window) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
        let buffer = vec![0x5Au8; FLAT_CONTROL_BYTES];
        std::hint::black_box(&buffer);
    });
    emitter.emit(&format!(
        "name=positive_control_flat shard={shard} {}",
        flat_window.text()
    ));
    let (_, mut session) = MountedSession::of_the_mount(mounted);
    for call_index in 0..MEASURED_CALLS_PER_ROUND {
        let content = measured_overwrite_content(round, call_index);
        let write_time_seconds =
            FIXED_WRITE_TIME_SECONDS + 2 + round * MEASURED_CALLS_PER_ROUND + call_index;
        let (published, window) = measure_in_a_window(&COUNTING_ALLOCATOR, || {
            session.publish_user_change(
                &mut pool,
                UserChange::Overwrite(FirstFile {
                    content: &content,
                    write_time_seconds,
                }),
            )
        });
        let (outcome, floor_raises) = match &published {
            Ok(change) => ("ok".to_string(), change.floor_raises.len()),
            Err(refusal) => (
                format!("failed:{}", variant_name_of(&format!("{refusal:?}"))),
                0,
            ),
        };
        drop(published);
        emitter.emit(&format!(
            "name=h2_publish shard={shard} call={call_index} outcome={outcome} floor_raises={floor_raises} {} records_after={}",
            window.text(),
            session.allocator.records().len(),
        ));
    }
    drop(images);
    ShardEnd::Finished
}

#[cfg(test)]
mod tests;
