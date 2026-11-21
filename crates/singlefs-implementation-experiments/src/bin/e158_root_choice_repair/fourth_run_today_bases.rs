//! 第 4 次跑今天那一臂的底：编码、解码、存取与造底。

use super::*;

// ---------------------------------------------------------------------------
// 12.7 在今天那一臂上造的起始镜像（登记「读法写死」表「被测那次挂载」一行：被测之前的镜像一律在今天那一臂上造）。
//      起始镜像里有一段要靠今天那一臂的代码走的（L5、PC-22、PC-554 的 A；L1(b) 那次可写挂载的写序列；H1f 第二次取号），
//      由今天那一份副本跑 `r4-bases` 写进目录（环境变量 `SINGLEFS_E158_TODAY_BASES`），各臂读回来；各臂自己按同一串步骤记账，
//      账里的根与读回来的镜像上的自证根逐条核（`r4_today_base_check` 行）。
// ---------------------------------------------------------------------------

/// 一份在今天那一臂上造的起始镜像：镜像，今天那一臂那几次调用写下、装置的账要补的根（与它们沿用的内容号），
/// 与 L5 要的「今天那一臂在孪生镜像上做那次挂载时每一片被读的次数」。
pub(super) struct TodayBase {
    pub(super) image: MemoryPool,
    appended_roots: Vec<(TimelineRoot, u64)>,
    pub(super) page_reads: Vec<u64>,
}

const TODAY_BASE_MAGIC: &[u8; 8] = b"E158R4B1";

fn today_bases_directory() -> Result<std::path::PathBuf, String> {
    env::var("SINGLEFS_E158_TODAY_BASES")
        .map(std::path::PathBuf::from)
        .map_err(|_| {
            "环境变量 SINGLEFS_E158_TODAY_BASES 没给（先在今天那一份副本上跑 r4-bases）".to_string()
        })
}

fn encode_today_base(base: &TodayBase) -> Vec<u8> {
    let mut bytes = TODAY_BASE_MAGIC.to_vec();
    bytes.extend_from_slice(
        &u32::try_from(base.image.devices.len())
            .expect("两块盘")
            .to_le_bytes(),
    );
    for (identity, device) in &base.image.devices {
        let sectors =
            device.written_sectors_in(DeviceOffsetInBytes(0), base.image.device_size_in_bytes);
        bytes.extend_from_slice(&identity.0.to_le_bytes());
        bytes.extend_from_slice(&(sectors.len() as u64).to_le_bytes());
        for sector in sectors {
            bytes.extend_from_slice(&sector.to_le_bytes());
            bytes.extend_from_slice(
                &device.read(DeviceOffsetInBytes(sector * SECTOR_BYTES_LOCAL), 512),
            );
        }
    }
    bytes.extend_from_slice(&(base.appended_roots.len() as u64).to_le_bytes());
    for ((instance, txg), content_number) in &base.appended_roots {
        bytes.extend_from_slice(&instance.to_le_bytes());
        bytes.extend_from_slice(&txg.to_le_bytes());
        bytes.extend_from_slice(&content_number.to_le_bytes());
    }
    bytes.extend_from_slice(&(base.page_reads.len() as u64).to_le_bytes());
    for reads in &base.page_reads {
        bytes.extend_from_slice(&reads.to_le_bytes());
    }
    bytes
}

/// 按字节流顺序取定宽的整数（`decode_today_base` 用）。
struct ByteCursor<'bytes> {
    bytes: &'bytes [u8],
    position: usize,
}

impl ByteCursor<'_> {
    pub(super) fn take(&mut self, length: usize) -> Result<&[u8], String> {
        let end = self.position + length;
        let taken = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| format!("起始镜像文件在第 {} 字节截断", self.position))?;
        self.position = end;
        Ok(taken)
    }

    fn u32_value(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 字节"),
        ))
    }

    fn u64_value(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().expect("8 字节"),
        ))
    }
}

fn decode_today_base(bytes: &[u8]) -> Result<TodayBase, String> {
    let mut cursor = ByteCursor { bytes, position: 0 };
    if cursor.take(8)? != TODAY_BASE_MAGIC {
        return Err("起始镜像文件的魔数不对".to_string());
    }
    let device_count = cursor.u32_value()?;
    let mut devices = BTreeMap::new();
    for _ in 0..device_count {
        let identity = DeviceIdentity(cursor.u32_value()?);
        let sector_count = cursor.u64_value()?;
        let mut device = SparseDevice::default();
        for _ in 0..sector_count {
            let sector = cursor.u64_value()?;
            let content = cursor.take(512)?.to_vec();
            device.write(DeviceOffsetInBytes(sector * SECTOR_BYTES_LOCAL), &content);
        }
        devices.insert(identity, device);
    }
    let appended_count = cursor.u64_value()?;
    let mut appended_roots = Vec::new();
    for _ in 0..appended_count {
        let instance = cursor.u32_value()?;
        let txg = cursor.u64_value()?;
        let content_number = cursor.u64_value()?;
        appended_roots.push(((instance, txg), content_number));
    }
    let page_count = cursor.u64_value()?;
    let mut page_reads = Vec::new();
    for _ in 0..page_count {
        page_reads.push(cursor.u64_value()?);
    }
    Ok(TodayBase {
        image: MemoryPool {
            devices,
            device_size_in_bytes: IMAGE_BYTES,
        },
        appended_roots,
        page_reads,
    })
}

fn today_base_path(key: &str) -> Result<std::path::PathBuf, String> {
    Ok(today_bases_directory()?.join(format!("{key}.base")))
}

fn save_today_base(key: &str, base: &TodayBase) -> Result<(), String> {
    let path = today_base_path(key)?;
    std::fs::write(&path, encode_today_base(base))
        .map_err(|error| format!("{}: {error}", path.display()))
}

pub(super) fn load_today_base(key: &str) -> Result<TodayBase, String> {
    let path = today_base_path(key)?;
    let bytes = std::fs::read(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    decode_today_base(&bytes)
}

/// 把今天那一臂造的镜像换进这一臂按同一串步骤记的账：补上今天那几次调用写下的根，逐条核账里的根都是读回来的镜像上的自证根。
pub(super) fn adopt_today_base(
    history: &mut ThirdRunHistory,
    key: &str,
    arm: FourthRunArm,
) -> Result<Vec<u64>, String> {
    let base = load_today_base(key)?;
    for (root, content_number) in &base.appended_roots {
        history.push_carried(*root, *content_number);
    }
    let geometry = independent_geometry(&base.image)?;
    let on_the_image = readable_roots_independent(&base.image, &geometry);
    let missing: Vec<String> = history
        .content_number_by_root
        .keys()
        .filter(|root| !on_the_image.contains(root))
        .map(|root| key_text(Some(*root)))
        .collect();
    emit_result(&format!(
        "name=r4_today_base_check arm={} key={key} base_fingerprint={:016x} appended_roots={} ledger_roots_missing_on_the_image={} note=\"根环转过的旧根不在镜像上是正常的，只报数\"",
        arm.name,
        image_fingerprint(&base.image),
        base.appended_roots.len(),
        missing.len()
    ));
    history.pool = base.image;
    history.session = None;
    Ok(base.page_reads)
}

// ---------------------------------------------------------------------------
// 12.8 第三段：H1f（前缀中间夹一次重挂载，前缀镜像在今天那一臂上造）、H1g（a 次挂载崩在取号之后，在这一臂自己的副本上做）。
// ---------------------------------------------------------------------------

fn remount_prefix_key(
    geometry: &Geometry,
    first_overwrites: u64,
    closing: Closing,
    second_overwrites: u64,
) -> String {
    format!(
        "h1f-s{}-ring{}-n1a{first_overwrites}-c{}-n1b{second_overwrites}",
        geometry.slots_per_region,
        geometry.journal_ring_bytes,
        closing.name()
    )
}

/// H1f 的八个前缀（这一臂按同一串步骤记账，镜像换成今天那一臂造的那一份）。
fn fourth_run_remount_prefixes(
    geometry: &Geometry,
    arm: FourthRunArm,
) -> Vec<Result<FourthRunPrefix, String>> {
    let mut prefixes = Vec::new();
    for first_overwrites in FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            for second_overwrites in FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES {
                prefixes.push(
                    fourth_run_remount_prefix(
                        geometry,
                        first_overwrites,
                        closing,
                        second_overwrites,
                        arm.name,
                    )
                    .and_then(|mut prefix| {
                        let key = remount_prefix_key(
                            geometry,
                            first_overwrites,
                            closing,
                            second_overwrites,
                        );
                        adopt_today_base(&mut prefix.history, &key, arm)?;
                        Ok(prefix)
                    }),
                );
            }
        }
    }
    prefixes
}

pub(super) fn run_fourth_run_segment_three(arm: FourthRunArm) {
    run_fourth_run_remount_families(arm);
    run_fourth_run_acquisition_crash_family(arm);
}

/// H1f（G0）：八个前缀接 H1d 的全部断点、接 H1e 的 m 尾巴。
pub(super) fn run_fourth_run_remount_families(arm: FourthRunArm) {
    for (name, continuation) in [
        ("h1f-cuts", FourthRunContinuation::EveryCut),
        ("h1f-tail", FourthRunContinuation::Tail),
    ] {
        let family = FourthRunFamily {
            name,
            geometry: &GEOMETRY_PRIMARY,
            hidden_count: 1,
            continuation,
            tail_overwrites: &THIRD_RUN_TAIL_OVERWRITES,
            durations: FourthRunDurationPlan::OfTheArm,
            checks_the_clauses_on_the_hidden_root:
                ClauseChecksOnTheHiddenRoot::NotRegisteredForThisFamily,
        };
        run_fourth_run_family(
            &family,
            arm,
            fourth_run_remount_prefixes(&GEOMETRY_PRIMARY, arm),
        );
    }
}

/// L5 的 n1 与 b（登记 5.3 L5 行：n1 ∈ {1, 2, 3}；b 两种）。
pub(super) const FOURTH_RUN_CELL_22_OVERWRITES: [u64; 3] = [1, 2, 3];

pub(super) fn cell_22_base_key(
    geometry: &Geometry,
    overwrites: u64,
    replay_break: ReplayBreak,
) -> String {
    format!(
        "l5-s{}-ring{}-n1{overwrites}-b{}",
        geometry.slots_per_region,
        geometry.journal_ring_bytes,
        replay_break.name()
    )
}

/// 最新根（择根那条）的实例表每一片的两份落点（`crates/` 的沿链读，`instance_table_page_pointers_as_far_as_readable`；
/// 装置没有自己的沿链读，执行员修订第 8 条）。
pub(super) fn instance_table_page_faults_of_the_newest_root(
    pool: &MemoryPool,
    duration: FaultDuration,
) -> Result<Vec<Vec<ThirdRunFault>>, String> {
    let (_, newest) = landing_root_on(pool)?;
    let devices = devices_from_pool(pool, IMAGE_BYTES);
    let pages =
        singlefs_core::recovery::instance_table_page_pointers_as_far_as_readable(&devices, &newest);
    Ok(pages
        .iter()
        .enumerate()
        .map(|(page, pointer)| {
            pointer
                .locations
                .iter()
                .map(|location| ThirdRunFault {
                    device: location.device,
                    offset: location.slot.0 * SLOT_BYTES_LOCAL,
                    length: SLOT_BYTES_LOCAL,
                    form: UnreadableForm::ReadFails,
                    duration,
                    label: format!(
                        "instance_table_page{page}(device={},slot={})",
                        location.device.0, location.slot.0
                    ),
                })
                .collect()
        })
        .collect())
}

/// L5 的起始镜像（今天那一臂，`r4-bases` 里造）：P(n1, C) → A（`read_fails`、W、不断）→ 关闭 C → 撤故障；
/// 另在它的孪生镜像上做一次不注入的可写挂载，数每一片（两份合计）被读的次数 M（登记 5.3 L5 行）。
fn build_today_cell_22_base(
    geometry: &Geometry,
    overwrites: u64,
    replay_break: ReplayBreak,
) -> Result<TodayBase, String> {
    let parameters = parameters_for(geometry);
    let history =
        fourth_run_plain_prefix(geometry, overwrites, Closing::ProcessExit, "today")?.history;
    let pool_geometry = independent_geometry(&history.pool)?;
    let faults = fourth_run_abandonment_faults(
        &history,
        &pool_geometry,
        UnreadableForm::ReadFails,
        replay_break,
        FaultDuration::WholeCall,
        1,
    )?
    .ok_or("A 的落点造不出")?;
    let tested = run_fourth_run_mount_writable(&history.pool, &parameters, &faults, None, None);
    let mounted = tested
        .call
        .outcome
        .as_ref()
        .map_err(|error| format!("今天那一臂的 A 被拒: {error:?}"))?;
    let carried = history
        .content_number_of(root_pair(&mounted.output.effective_root))
        .unwrap_or(0);
    let numbers = content_numbers_with_the_publishes_of(&history, Some(mounted));
    let appended_roots: Vec<(TimelineRoot, u64)> = numbers
        .iter()
        .filter(|(root, _)| !history.content_number_by_root.contains_key(root))
        .map(|(root, _)| (*root, carried))
        .collect();
    let image = tested.call.pool_after.clone();
    let page_faults = instance_table_page_faults_of_the_newest_root(
        &image,
        FaultDuration::FromTheNthRead(u64::MAX),
    )?;
    let flat: Vec<ThirdRunFault> = page_faults.iter().flatten().cloned().collect();
    let twin = run_fourth_run_mount_writable(&image, &parameters, &flat, None, None);
    let page_reads = page_faults
        .iter()
        .scan(0usize, |start, copies| {
            let reads: u64 = twin.reads_of_the_targets[*start..*start + copies.len()]
                .iter()
                .sum();
            *start += copies.len();
            Some(reads)
        })
        .collect();
    Ok(TodayBase {
        image,
        appended_roots,
        page_reads,
    })
}

/// L1(b) 的崩溃状态（今天那一臂）：P(n1, C) 之后一次可写挂载的写序列上前 j 个写落盘（j 取全部真前缀）。
fn build_today_mount_crash_bases(
    geometry: &Geometry,
    overwrites: u64,
) -> Result<Vec<TodayBase>, String> {
    let parameters = parameters_for(geometry);
    let history =
        fourth_run_plain_prefix(geometry, overwrites, Closing::ProcessExit, "today")?.history;
    let call = run_fourth_run_mount_writable(&history.pool, &parameters, &[], None, None);
    call.call
        .outcome
        .as_ref()
        .map_err(|error| format!("今天那一臂的可写挂载被拒: {error:?}"))?;
    Ok((0..writes_in(&call.call.applied))
        .map(|writes| TodayBase {
            image: image_with_the_first_writes(&history.pool, &call.call.applied, writes),
            appended_roots: Vec::new(),
            page_reads: Vec::new(),
        })
        .collect())
}

/// H1f 的前缀镜像（今天那一臂）。
fn build_today_remount_prefix_base(
    geometry: &Geometry,
    first_overwrites: u64,
    closing: Closing,
    second_overwrites: u64,
) -> Result<TodayBase, String> {
    let prefix = fourth_run_remount_prefix(
        geometry,
        first_overwrites,
        closing,
        second_overwrites,
        "today",
    )?;
    Ok(TodayBase {
        image: prefix.history.pool,
        appended_roots: Vec::new(),
        page_reads: Vec::new(),
    })
}

/// `r4-bases`（只在今天那一份副本上跑）：把各段要的「在今天那一臂上造」的起始镜像写进目录。
pub(super) fn run_fourth_run_today_bases(arm: FourthRunArm) {
    if !arm.is_today() {
        emit_result(&format!(
            "name=r4_today_bases arm={} verdict=stop reason=\"r4-bases 只在今天那一份副本上跑\"",
            arm.name
        ));
        return;
    }
    let mut saved = 0u64;
    let mut failures = Vec::new();
    let mut save = |key: String, base: Result<TodayBase, String>| match base.and_then(|base| {
        save_today_base(&key, &base)?;
        Ok(base)
    }) {
        Ok(base) => {
            saved += 1;
            emit_result(&format!(
                "name=r4_today_base key={key} base_fingerprint={:016x} appended_roots={} page_reads={:?}",
                image_fingerprint(&base.image),
                base.appended_roots.len(),
                base.page_reads
            ));
        }
        Err(error) => failures.push(format!("{key}: {error}")),
    };
    for first_overwrites in FOURTH_RUN_REMOUNT_PREFIX_FIRST_OVERWRITES {
        for closing in [Closing::ProcessExit, Closing::NormalUnmount] {
            for second_overwrites in FOURTH_RUN_REMOUNT_PREFIX_SECOND_OVERWRITES {
                save(
                    remount_prefix_key(
                        &GEOMETRY_PRIMARY,
                        first_overwrites,
                        closing,
                        second_overwrites,
                    ),
                    build_today_remount_prefix_base(
                        &GEOMETRY_PRIMARY,
                        first_overwrites,
                        closing,
                        second_overwrites,
                    ),
                );
            }
        }
    }
    for overwrites in THIRD_RUN_OVERWRITES {
        match build_today_mount_crash_bases(&GEOMETRY_PRIMARY, overwrites) {
            Ok(bases) => {
                for (point, base) in bases.into_iter().enumerate() {
                    save(
                        format!(
                            "l1b-s{}-ring{}-n1{overwrites}-point{point}",
                            GEOMETRY_PRIMARY.slots_per_region, GEOMETRY_PRIMARY.journal_ring_bytes
                        ),
                        Ok(base),
                    );
                }
            }
            Err(error) => save(format!("l1b-n1{overwrites}-mount"), Err(error)),
        }
    }
    for geometry in [
        &GEOMETRY_PRIMARY,
        &GEOMETRY_SMALLER_ROOT_RING,
        &GEOMETRY_LARGER_ROOT_RING,
    ] {
        for overwrites in FOURTH_RUN_CELL_22_OVERWRITES {
            for replay_break in [
                ReplayBreak::EveryRecord,
                ReplayBreak::FirstCopyOfTheFirstNamedUnit,
            ] {
                save(
                    cell_22_base_key(geometry, overwrites, replay_break),
                    build_today_cell_22_base(geometry, overwrites, replay_break),
                );
            }
        }
    }
    for failure in &failures {
        emit_result(&format!("name=r4_today_base_failure detail={failure:?}"));
    }
    emit_result(&format!(
        "name=r4_today_bases arm={} saved={saved} failures={} verdict={}",
        arm.name,
        failures.len(),
        if failures.is_empty() { "pass" } else { "fail" }
    ));
}

#[cfg(test)]
mod fourth_run_tests;
