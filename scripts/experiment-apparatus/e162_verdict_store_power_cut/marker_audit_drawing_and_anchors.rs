//! 标记的审计、按桶抽点与核对项的生成，以及产物行、角色参数与锚点行。

use super::*;

// ============================== 标记核对（V22）与分桶（补 5.2、补 7 B2、B5） ==============================

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarkerScan {
    /// （条目下标，标记），按日志次序。
    pub(super) markers: Vec<(usize, Marker)>,
    checksum_failures: u64,
    malformed: u64,
    misplaced: u64,
}

pub(super) fn scan_markers(log: &LogBytes, device_log: &DeviceLog) -> Result<MarkerScan, String> {
    let partition_start = marker_partition_start_sector();
    let partition_end = partition_start + marker_partition_sectors();
    let mut scan = MarkerScan { markers: Vec::new(), checksum_failures: 0, malformed: 0, misplaced: 0 };
    let mut sector = [0u8; SECTOR_BYTES];
    for (entry_index, entry) in device_log.entries.iter().enumerate() {
        let LogEntry::Write { first_sector, sector_count, data_offset_in_log, .. } = *entry else { continue };
        for disk_sector in first_sector.max(partition_start)..(first_sector + sector_count).min(partition_end) {
            log.read_exact_at(&mut sector, data_offset_in_log + (disk_sector - first_sector) * sector_bytes())?;
            match decode_marker(&sector) {
                MarkerDecoding::Valid(marker) => {
                    if marker.sequence != disk_sector - partition_start {
                        scan.misplaced += 1;
                    }
                    scan.markers.push((entry_index, marker));
                }
                MarkerDecoding::ChecksumMismatch => scan.checksum_failures += 1,
                MarkerDecoding::NotAMarker | MarkerDecoding::Malformed(_) => scan.malformed += 1,
            }
        }
    }
    Ok(scan)
}

/// 标记在日志上的位置：S、E 与每块的 B_j、C_j 的条目下标。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CommitWindows {
    pub(super) start_entry: usize,
    pub(super) end_entry: usize,
    pub(super) begin_entries: Vec<usize>,
    pub(super) confirm_entries: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MarkerAudit {
    pub(super) problems: Vec<String>,
    pub(super) confirm_count: u64,
    pub(super) windows: Option<CommitWindows>,
}

/// V22：c 在日志里从 0 起逐个加 1；次序是 S、（B_j、C_j）…、E；C 标记个数与来宾报的已确认块数相等；CRC 都对。
pub(super) fn audit_markers(scan: &MarkerScan, arm_index: u32, guest_confirmed_blocks: u64) -> MarkerAudit {
    let mut problems = Vec::new();
    if scan.checksum_failures > 0 {
        problems.push(format!("checksum_failures={}", scan.checksum_failures));
    }
    if scan.malformed > 0 {
        problems.push(format!("malformed={}", scan.malformed));
    }
    if scan.misplaced > 0 {
        problems.push(format!("misplaced={}", scan.misplaced));
    }
    let mut start_entry = None;
    let mut end_entry = None;
    let mut begin_entries = Vec::new();
    let mut confirm_entries = Vec::new();
    for (position, (entry_index, marker)) in scan.markers.iter().enumerate() {
        if marker.sequence != u64::try_from(position).expect("装得进 u64") {
            problems.push(format!("sequence_gap_at={position}"));
            break;
        }
        if marker.arm_index != arm_index {
            problems.push(format!("arm_mismatch_at={position}"));
            break;
        }
        let expected_block = u64::try_from(confirm_entries.len()).expect("装得进 u64");
        let in_order = match marker.kind {
            MarkerKind::Start => position == 0,
            MarkerKind::Begin => {
                start_entry.is_some() && end_entry.is_none() && begin_entries.len() == confirm_entries.len() && marker.block_index == Some(expected_block)
            }
            MarkerKind::Confirm => {
                begin_entries.len() == confirm_entries.len() + 1 && marker.block_index == Some(expected_block)
            }
            MarkerKind::End => start_entry.is_some() && end_entry.is_none() && begin_entries.len() == confirm_entries.len(),
        };
        if !in_order {
            problems.push(format!("out_of_order_at={position}:{:?}", marker.kind));
            break;
        }
        match marker.kind {
            MarkerKind::Start => start_entry = Some(*entry_index),
            MarkerKind::Begin => begin_entries.push(*entry_index),
            MarkerKind::Confirm => confirm_entries.push(*entry_index),
            MarkerKind::End => end_entry = Some(*entry_index),
        }
    }
    let confirm_count = u64::try_from(confirm_entries.len()).expect("装得进 u64");
    if start_entry.is_none() {
        problems.push("missing_start".to_string());
    }
    if end_entry.is_none() {
        problems.push("missing_end".to_string());
    }
    if confirm_count != guest_confirmed_blocks {
        problems.push(format!("confirm_markers={confirm_count}_guest_confirmed={guest_confirmed_blocks}"));
    }
    let windows = match (start_entry, end_entry, problems.is_empty()) {
        (Some(start), Some(end), true) => Some(CommitWindows { start_entry: start, end_entry: end, begin_entries, confirm_entries }),
        (Some(_) | None, Some(_) | None, true | false) => None,
    };
    MarkerAudit { problems, confirm_count, windows }
}

/// 断电点 point 上：已确认 = C 标记下标 < point 的块数；已开始 = B 标记下标 < point 的块数。
pub(super) fn block_classes_at(windows: &CommitWindows, point: usize) -> (u64, u64) {
    let confirmed_end = windows.confirm_entries.iter().filter(|&&entry| entry < point).count();
    let started_end = windows.begin_entries.iter().filter(|&&entry| entry < point).count();
    (u64::try_from(confirmed_end).expect("装得进 u64"), u64::try_from(started_end).expect("装得进 u64"))
}

fn sorted_percentile(sorted: &[u64], numerator: usize, denominator: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = (sorted.len() * numerator).div_ceil(denominator).max(1);
    sorted[rank - 1]
}

pub(super) fn median_of(values: &[u64]) -> u64 {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted_percentile(&sorted, 1, 2)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BucketPools {
    /// 桶 A：普通窗的 (B_j, C_j] 里全部条目边界。
    pub(super) normal_commit_points: Vec<usize>,
    /// 桶 B：长窗的 (B_j, C_j]。
    pub(super) long_commit_points: Vec<usize>,
    /// 桶 C：(S, B_0]、(C_j, B_{j+1}]、(C_末, E]。
    pub(super) between_commit_points: Vec<usize>,
    pub(super) median_window_entries: u64,
    pub(super) normal_blocks: Vec<u64>,
    pub(super) long_blocks: Vec<u64>,
}

pub(super) fn bucket_pools(windows: &CommitWindows) -> BucketPools {
    let window_lengths: Vec<u64> = windows
        .begin_entries
        .iter()
        .zip(&windows.confirm_entries)
        .map(|(begin, confirm)| u64::try_from(confirm - begin).expect("装得进 u64"))
        .collect();
    let median_window_entries = median_of(&window_lengths);
    let mut pools = BucketPools {
        normal_commit_points: Vec::new(),
        long_commit_points: Vec::new(),
        between_commit_points: Vec::new(),
        median_window_entries,
        normal_blocks: Vec::new(),
        long_blocks: Vec::new(),
    };
    for (block_position, (&begin, &confirm)) in windows.begin_entries.iter().zip(&windows.confirm_entries).enumerate() {
        let block_index = u64::try_from(block_position).expect("装得进 u64");
        if window_lengths[block_position] > LONG_WINDOW_MEDIAN_FACTOR * median_window_entries {
            pools.long_blocks.push(block_index);
            pools.long_commit_points.extend(begin + 1..=confirm);
        } else {
            pools.normal_blocks.push(block_index);
            pools.normal_commit_points.extend(begin + 1..=confirm);
        }
    }
    let mut previous_boundary = windows.start_entry;
    for (&begin, &confirm) in windows.begin_entries.iter().zip(&windows.confirm_entries) {
        pools.between_commit_points.extend(previous_boundary + 1..=begin);
        previous_boundary = confirm;
    }
    pools.between_commit_points.extend(previous_boundary + 1..=windows.end_entry);
    pools
}

fn bucket_generator(seed: u64, arm_index: u64, bucket: u64) -> SplitMix64 {
    SplitMix64::new(seed ^ ((arm_index << 8) | bucket).wrapping_mul(MIXING_MULTIPLIER))
}

/// `池[输出 mod 池长]`、跳过重复；池不比要的多就全取。
fn draw_distinct(generator: &mut SplitMix64, pool: &[usize], wanted: usize) -> Vec<usize> {
    if pool.len() <= wanted {
        return pool.to_vec();
    }
    let pool_length = u64::try_from(pool.len()).expect("装得进 u64");
    let mut drawn = Vec::with_capacity(wanted);
    let mut seen = HashSet::new();
    while drawn.len() < wanted {
        let candidate = pool[usize::try_from(generator.next_output() % pool_length).expect("小于池长")];
        if seen.insert(candidate) {
            drawn.push(candidate);
        }
    }
    drawn
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StratifiedDraw {
    bucket_normal: Vec<usize>,
    bucket_long: Vec<usize>,
    bucket_between: Vec<usize>,
}

/// 补 7 B2：先取桶 B、再取桶 C，桶 A 取 total 减去 B、C 两桶实取的数。
fn stratified_draw(arm_index: u64, pools: &BucketPools, long_quota: usize, between_quota: usize, total: usize) -> StratifiedDraw {
    let mut long_generator = bucket_generator(SEED_CUT, arm_index, BUCKET_LONG_COMMITS);
    let bucket_long = draw_distinct(&mut long_generator, &pools.long_commit_points, long_quota);
    let mut between_generator = bucket_generator(SEED_CUT, arm_index, BUCKET_BETWEEN_COMMITS);
    let bucket_between = draw_distinct(&mut between_generator, &pools.between_commit_points, between_quota);
    let mut normal_generator = bucket_generator(SEED_CUT, arm_index, BUCKET_NORMAL_COMMITS);
    let bucket_normal =
        draw_distinct(&mut normal_generator, &pools.normal_commit_points, total - bucket_long.len() - bucket_between.len());
    StratifiedDraw { bucket_normal, bucket_long, bucket_between }
}

/// 补 7 B5：长窗挑 2 个（不足就全挑），普通窗补足 8 个；下标是各自按 j 升序排好之后的位置。
fn picked_commit_positions(arm_index: u64, normal_count: usize, long_count: usize) -> (Vec<usize>, Vec<usize>) {
    let mut normal_generator = bucket_generator(SEED_PICK, arm_index, BUCKET_PICK_NORMAL);
    let mut long_generator = bucket_generator(SEED_PICK, arm_index, BUCKET_PICK_LONG);
    let long_positions = draw_distinct(&mut long_generator, &(0..long_count).collect::<Vec<usize>>(), PICKED_LONG_COMMIT_TARGET);
    let normal_positions =
        draw_distinct(&mut normal_generator, &(0..normal_count).collect::<Vec<usize>>(), PICKED_COMMIT_TOTAL - long_positions.len());
    (normal_positions, long_positions)
}

/// 一块的 (B_j, C_j] 里逐个条目边界；多于 64 个时取包含两端的 64 个等距点（第 s 个取 floor(s × (n − 1) ÷ 63)）。
pub(super) fn commit_window_boundaries(begin_entry: usize, confirm_entry: usize) -> Vec<usize> {
    let all: Vec<usize> = (begin_entry + 1..=confirm_entry).collect();
    if all.len() <= COMMIT_BOUNDARY_LIMIT {
        return all;
    }
    let last = all.len() - 1;
    (0..COMMIT_BOUNDARY_LIMIT).map(|step| all[step * last / (COMMIT_BOUNDARY_LIMIT - 1)]).collect()
}

/// PC4-L、PC4-NB、PC4-FD 的 20 点：(S, E] 全部边界里单桶取（补 7 B2c 那种取法）。
fn control_points(arm_index: u64, windows: &CommitWindows) -> Vec<usize> {
    let pools = BucketPools {
        normal_commit_points: (windows.start_entry + 1..=windows.end_entry).collect(),
        long_commit_points: Vec::new(),
        between_commit_points: Vec::new(),
        median_window_entries: 0,
        normal_blocks: Vec::new(),
        long_blocks: Vec::new(),
    };
    let mut points = stratified_draw(arm_index, &pools, 0, 0, CONTROL_POINT_TOTAL).bucket_normal;
    points.sort_unstable();
    points
}

// ============================== 验盘计划 ==============================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PointGroup {
    /// C-strat 桶 A「普通提交里」。
    StratifiedNormalCommits,
    /// C-strat 桶 B「长窗提交里」。
    StratifiedLongCommits,
    /// C-strat 桶 C「两次提交之间」。
    StratifiedBetweenCommits,
    /// C-commit：挑出的第 picked 块（先普通后长窗）窗口里第 boundary 个边界。
    CommitSweep { picked: u64, boundary: u64, block_index: u64 },
    /// PC4-L、PC4-NB、PC4-FD 的单桶 20 点。
    ControlUniform,
    /// PC4-S、PC4-P、PC4-O：「全部条目」重放出的盘面。
    FullReplayDamage,
}

impl PointGroup {
    pub(super) fn labels(self) -> (&'static str, &'static str) {
        match self {
            PointGroup::StratifiedNormalCommits => ("C-strat", "A"),
            PointGroup::StratifiedLongCommits => ("C-strat", "B"),
            PointGroup::StratifiedBetweenCommits => ("C-strat", "C"),
            PointGroup::CommitSweep { .. } => ("C-commit", "-"),
            PointGroup::ControlUniform => ("control", "-"),
            PointGroup::FullReplayDamage => ("full-replay", "-"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VerifyItem {
    pub(super) item_number: usize,
    pub(super) group: PointGroup,
    pub(super) point: usize,
    pub(super) variant: ReplayVariant,
    pub(super) subset_index: u64,
    pub(super) confirmed_end: u64,
    pub(super) started_end: u64,
    pub(super) damage: LibraryDamage,
}

impl VerifyItem {
    pub(super) fn plan_line(&self) -> String {
        let (group_label, bucket_label) = self.group.labels();
        let (picked, boundary, block) = match self.group {
            PointGroup::CommitSweep { picked, boundary, block_index } => (picked.to_string(), boundary.to_string(), block_index.to_string()),
            PointGroup::StratifiedNormalCommits
            | PointGroup::StratifiedLongCommits
            | PointGroup::StratifiedBetweenCommits
            | PointGroup::ControlUniform
            | PointGroup::FullReplayDamage => ("-".to_string(), "-".to_string(), "-".to_string()),
        };
        format!(
            "item number={} group={group_label} bucket={bucket_label} picked={picked} boundary={boundary} block={block} point={} variant={} subset={} confirmed_end={} started_end={} damage={}",
            self.item_number,
            self.point,
            self.variant.label(),
            self.subset_index,
            self.confirmed_end,
            self.started_end,
            self.damage.label()
        )
    }
}

/// 一次写盘开机之后要验的全部「断电点 × 取法」（补 5.2 断电点表；对照 20 点；PC4-S/P/O）。
/// full_replay_point 是这次开机的条目总数：PC4-S/P/O 在「全部条目」重放出的盘面上做。
pub(super) fn verification_items(arm: PowerCutArm, windows: &CommitWindows, pools: &BucketPools, full_replay_point: usize) -> Vec<VerifyItem> {
    let arm_index = u64::from(arm.index());
    let mut items = Vec::new();
    let push = |items: &mut Vec<VerifyItem>, group: PointGroup, point: usize, variant: ReplayVariant, subset_index: u64, damage: LibraryDamage| {
        let (confirmed_end, started_end) = block_classes_at(windows, point);
        let item_number = items.len();
        items.push(VerifyItem { item_number, group, point, variant, subset_index, confirmed_end, started_end, damage });
    };
    match arm.purpose() {
        ArmPurpose::Main | ArmPurpose::SmallBlockSamplingPoint => {
            let (total, long_quota, between_quota, variants): (usize, usize, usize, &[ReplayVariant]) = match arm.purpose() {
                ArmPurpose::SmallBlockSamplingPoint => (
                    SMALL_BLOCK_POINT_TOTAL,
                    SMALL_BLOCK_LONG_COMMIT_QUOTA,
                    SMALL_BLOCK_BETWEEN_COMMITS_QUOTA,
                    &[ReplayVariant::DropUnflushed, ReplayVariant::RandomSubset],
                ),
                ArmPurpose::Main
                | ArmPurpose::LostPageCacheControl
                | ArmPurpose::LostWriteCacheControl
                | ArmPurpose::MissingDirectorySyncControl
                | ArmPurpose::WriteCacheModelControl => {
                    (STRATIFIED_POINT_TOTAL, LONG_COMMIT_BUCKET_QUOTA, BETWEEN_COMMITS_BUCKET_QUOTA, &ALL_REPLAY_VARIANTS)
                }
            };
            let draw = stratified_draw(arm_index, pools, long_quota, between_quota, total);
            let mut stratified: Vec<(usize, PointGroup)> = draw
                .bucket_normal
                .iter()
                .map(|&point| (point, PointGroup::StratifiedNormalCommits))
                .chain(draw.bucket_long.iter().map(|&point| (point, PointGroup::StratifiedLongCommits)))
                .chain(draw.bucket_between.iter().map(|&point| (point, PointGroup::StratifiedBetweenCommits)))
                .collect();
            stratified.sort_unstable_by_key(|(point, _group)| *point);
            for (subset_index, (point, group)) in stratified.into_iter().enumerate() {
                for &variant in variants {
                    push(&mut items, group, point, variant, u64::try_from(subset_index).expect("装得进 u64"), LibraryDamage::Untouched);
                }
            }
            if arm.purpose() == ArmPurpose::Main {
                let (normal_positions, long_positions) = picked_commit_positions(arm_index, pools.normal_blocks.len(), pools.long_blocks.len());
                let picked_blocks: Vec<u64> = normal_positions
                    .iter()
                    .map(|&position| pools.normal_blocks[position])
                    .chain(long_positions.iter().map(|&position| pools.long_blocks[position]))
                    .collect();
                for (picked, block_index) in picked_blocks.into_iter().enumerate() {
                    let block_position = usize::try_from(block_index).expect("装得进 usize");
                    let boundaries = commit_window_boundaries(windows.begin_entries[block_position], windows.confirm_entries[block_position]);
                    for (boundary, point) in boundaries.into_iter().enumerate() {
                        let picked_number = u64::try_from(picked).expect("装得进 u64");
                        let boundary_number = u64::try_from(boundary).expect("装得进 u64");
                        let subset_index = COMMIT_SUBSET_INDEX_BASE + COMMIT_SUBSET_INDEX_STRIDE * picked_number + boundary_number;
                        let group = PointGroup::CommitSweep { picked: picked_number, boundary: boundary_number, block_index };
                        for variant in [ReplayVariant::DropUnflushed, ReplayVariant::RandomSubset] {
                            push(&mut items, group, point, variant, subset_index, LibraryDamage::Untouched);
                        }
                    }
                }
                assert!(full_replay_point > windows.end_entry, "全部条目的断电点在 E 标记之后");
                for damage in [LibraryDamage::SilentCorruption, LibraryDamage::PhantomKey, LibraryDamage::OpenBreaking] {
                    push(&mut items, PointGroup::FullReplayDamage, full_replay_point, ReplayVariant::KeepUnflushed, 0, damage);
                }
            }
        }
        ArmPurpose::LostPageCacheControl | ArmPurpose::LostWriteCacheControl | ArmPurpose::MissingDirectorySyncControl => {
            let variant = match arm.purpose() {
                ArmPurpose::LostPageCacheControl => ReplayVariant::KeepUnflushed,
                ArmPurpose::LostWriteCacheControl
                | ArmPurpose::MissingDirectorySyncControl
                | ArmPurpose::Main
                | ArmPurpose::SmallBlockSamplingPoint
                | ArmPurpose::WriteCacheModelControl => ReplayVariant::DropUnflushed,
            };
            for (subset_index, point) in control_points(arm_index, windows).into_iter().enumerate() {
                push(&mut items, PointGroup::ControlUniform, point, variant, u64::try_from(subset_index).expect("装得进 u64"), LibraryDamage::Untouched);
            }
        }
        ArmPurpose::WriteCacheModelControl => {}
    }
    items
}

// ============================== 产物行与参数 ==============================

/// 行打到哪：标准输出（给了文件就同时写进那份文件），或全扔掉（来宾开头的锚点自检只要对不上的个数）。
enum LineDestination {
    StandardOutput { copy_to_file: Option<File> },
    Discarded,
}

/// 产物行：每行 `E7RESULT ` 起头，收尾 `name=done emitted=N`。
pub(super) struct ProductLines {
    emitter: Emitter,
    destination: LineDestination,
}

impl ProductLines {
    pub(super) fn to_standard_output() -> ProductLines {
        ProductLines { emitter: Emitter::new(), destination: LineDestination::StandardOutput { copy_to_file: None } }
    }

    pub(super) fn discarding() -> ProductLines {
        ProductLines { emitter: Emitter::new(), destination: LineDestination::Discarded }
    }

    pub(super) fn with_file_copy(path: &Path) -> Result<ProductLines, String> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("建不出产物文件 {}（已存在就换一个新文件名）：{error}", path.display()))?;
        Ok(ProductLines { emitter: Emitter::new(), destination: LineDestination::StandardOutput { copy_to_file: Some(file) } })
    }

    pub(super) fn line(&mut self, body: impl AsRef<str>) {
        let text = self.emitter.emit_raw(body.as_ref());
        self.write_through(&text);
    }

    pub(super) fn finish(&mut self) {
        let text = self.emitter.finish();
        self.write_through(&text);
    }

    fn write_through(&mut self, text: &str) {
        match &mut self.destination {
            LineDestination::Discarded => {}
            LineDestination::StandardOutput { copy_to_file } => {
                let mut standard_output = std::io::stdout().lock();
                writeln!(standard_output, "{text}").expect("写得进标准输出");
                standard_output.flush().expect("刷得出标准输出");
                if let Some(file) = copy_to_file {
                    writeln!(file, "{text}").expect("写得进产物文件");
                }
            }
        }
    }
}

pub(super) fn field_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    line.split(' ').find_map(|field| field.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
}

pub(super) fn field_number(line: &str, key: &str) -> Option<u64> {
    field_value(line, key).and_then(|text| text.parse().ok())
}

/// 行里的自由文本（错误信息、dmesg）：空白换成下划线、截到 240 个字符，免得撑破「空格分字段」的产物行。
pub(super) fn sanitized(text: &str) -> String {
    let flattened: String = text.chars().map(|character| if character.is_whitespace() { '_' } else { character }).collect();
    let shortened: String = flattened.chars().take(240).collect();
    if shortened.is_empty() {
        "-".to_string()
    } else {
        shortened
    }
}

/// `--键 值` 与 `--开关`；不以 `--` 起头又没被当成值的，算位置参数（来宾里 vm-bench 塞进来的 /dev/vda）。
pub(super) struct RoleArguments {
    values: BTreeMap<String, String>,
}

impl RoleArguments {
    pub(super) fn parse(arguments: &[String]) -> RoleArguments {
        let mut values = BTreeMap::new();
        let mut position = 0usize;
        while position < arguments.len() {
            let token = &arguments[position];
            if let Some(key) = token.strip_prefix("--") {
                let next_is_value = arguments.get(position + 1).is_some_and(|next| !next.starts_with("--"));
                if next_is_value {
                    values.insert(key.to_string(), arguments[position + 1].clone());
                    position += 2;
                } else {
                    values.insert(key.to_string(), "true".to_string());
                    position += 1;
                }
            } else {
                position += 1;
            }
        }
        RoleArguments { values }
    }

    pub(super) fn text(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    pub(super) fn number(&self, key: &str) -> Option<u64> {
        self.text(key).and_then(|text| text.parse().ok())
    }

    pub(super) fn flag(&self, key: &str) -> bool {
        self.text(key) == Some("true")
    }
}

// ============================== 锚点自检（登记 7.2 A1–A6、补 7 B 系列） ==============================

pub(super) fn synthetic_anchor_log_entries() -> Vec<(u64, u64, u64, Vec<u8>)> {
    vec![
        (0, 1, 0, vec![0x11; 512]),
        (8, 2, 0, vec![0x22; 1024]),
        (0, 0, LOG_FLUSH_FLAG, Vec::new()),
        (0, 1, 0, vec![0x33; 512]),
        (16, 1, 0, vec![0x44; 512]),
        (4, 4, 0, vec![0x55; 2048]),
    ]
}

const ANCHOR_IMAGE_SECTORS: usize = 24;
const ANCHOR_REPLAY_ARM_INDEX: u64 = 0;
const ANCHOR_REPLAY_SUBSET_INDEX: u64 = 15;

/// B8：合成日志在断电点 point 上按 variant 重放出的 24 扇区镜像的 sha256。
pub(super) fn synthetic_anchor_image_digest(log_bytes: Vec<u8>, point: usize, variant: ReplayVariant) -> Result<String, String> {
    let log = LogBytes::InMemory(log_bytes);
    let device_log = parse_device_log(&log, DeclaredCountHandling::StopAtDeclared).map_err(|error| error.to_string())?;
    let selection = select_replayed_writes(&device_log.entries, point, variant, ANCHOR_REPLAY_ARM_INDEX, ANCHOR_REPLAY_SUBSET_INDEX);
    let mut image = ReplayImage::InMemory(vec![0u8; ANCHOR_IMAGE_SECTORS * SECTOR_BYTES]);
    apply_replayed_writes(&log, &selection.writes, &mut image)?;
    match image {
        ReplayImage::InMemory(bytes) => Ok(sha256_hexadecimal(&bytes)),
        ReplayImage::OnDisk(_) => Err("B8 的镜像在内存里".to_string()),
    }
}

pub(super) fn bit_string(bits: &[bool]) -> String {
    bits.iter().map(|&bit| if bit { '1' } else { '0' }).collect()
}

fn join_usize(values: &[usize]) -> String {
    values.iter().map(usize::to_string).collect::<Vec<String>>().join(",")
}

/// 跑全部锚点，每个一行；返回对不上的个数。
pub(super) fn run_anchors(output: &mut ProductLines) -> usize {
    let mut mismatches = 0usize;
    let mut record = |output: &mut ProductLines, name: &str, got: String, expected: &str| {
        let matches = got == expected;
        if !matches {
            mismatches += 1;
        }
        output.line(format!("name=anchor id={name} value={got} matches_registration={matches}"));
    };
    let mut zero_generator = SplitMix64::new(0);
    let first_three: Vec<String> = (0..3).map(|_draw| format!("{:#018x}", zero_generator.next_output())).collect();
    record(output, "A1", first_three.join(","), "0xe220a8397b1dcdaf,0x6e789e6aa1b965f4,0x06c45d188009454f");
    record(output, "A2", format!("{:#010x}", crc32c(&[b"123456789"])), "0xe3069283");
    for (block_index, expected) in [
        (0u64, "76020f1b59444d587a6607195c0f8fed/8351809/0x08eefd5c"),
        (12_345, "b3c1f0df0c77c9e98c9df5b21d0395bf/8321879/0x565d4801"),
    ] {
        let value = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        record(output, &format!("A3_j{block_index}"), format!("{}/{byte_sum}/{:#010x}", hexadecimal(&value[..16]), crc32c(&[&value])), expected);
    }
    let small = block_value(ValuePattern::Random, SEED_THROUGHPUT_BLOCKS, 0, SMALL_STATES_PER_BLOCK);
    let small_sum: u64 = small.iter().map(|&byte| u64::from(byte)).sum();
    record(output, "A3b", format!("{}/{small_sum}", hexadecimal(&small[..16])), "76020f1b59444d587a6607195c0f8fed/131533");
    for (block_index, expected) in [(0u64, "267/32629"), (12_345, "245/28761")] {
        let value = block_value(ValuePattern::Sparse, SEED_THROUGHPUT_BLOCKS, block_index, PRIMARY_STATES_PER_BLOCK);
        let nonzero = value.iter().filter(|&&byte| byte != 0).count();
        let byte_sum: u64 = value.iter().map(|&byte| u64::from(byte)).sum();
        record(output, &format!("A4_j{block_index}"), format!("{nonzero}/{byte_sum}"), expected);
    }
    let fingerprint = input_fingerprint();
    record(output, "A5", hexadecimal(&fingerprint), "7299025377b7594e92ceeb8a1c3cbeac");
    for (block_index, expected) in [
        (0u64, "000000000000000000000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (1, "000000000000000100000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (63, "000000070000000700000000000000007299025377b7594e92ceeb8a1c3cbeac"),
        (64, "000000000000000000000000000100007299025377b7594e92ceeb8a1c3cbeac"),
        (12_345, "00000007000000010000000000c000007299025377b7594e92ceeb8a1c3cbeac"),
        (99_999, "000000030000000700000000061a00007299025377b7594e92ceeb8a1c3cbeac"),
    ] {
        let key = interleaved_block_key(block_index, PRIMARY_STATES_PER_BLOCK, &fingerprint);
        record(output, &format!("A6_j{block_index}"), hexadecimal(&key), expected);
    }
    let small_key = interleaved_block_key(12_345, SMALL_STATES_PER_BLOCK, &fingerprint);
    record(output, "A6b", hexadecimal(&small_key), "000000070000000100000000000300007299025377b7594e92ceeb8a1c3cbeac");
    let arm_names: Vec<String> = ALL_POWER_CUT_ARMS.iter().map(|arm| format!("{}={}", arm.index(), arm.label())).collect();
    record(
        output,
        "B1",
        arm_names.join(","),
        "0=R1,1=R0,2=K,3=F,4=R1-None,5=R0-None,6=K-nosync,7=F-nofsync,8=F-nodirsync,9=PC4-C,10=R1-NB,11=R0-NB,12=K-NB,13=F-NB,14=R1-bs10,15=R0-bs10,16=K-bs10,17=F-bs10",
    );
    let anchor_pools = |long_pool: Vec<usize>| BucketPools {
        normal_commit_points: (1_000..41_000).collect(),
        long_commit_points: long_pool,
        between_commit_points: (60_000..66_000).collect(),
        median_window_entries: 0,
        normal_blocks: Vec::new(),
        long_blocks: Vec::new(),
    };
    let draw = stratified_draw(0, &anchor_pools((50_000..50_500).collect()), LONG_COMMIT_BUCKET_QUOTA, BETWEEN_COMMITS_BUCKET_QUOTA, STRATIFIED_POINT_TOTAL);
    let draw_sum: usize = draw.bucket_normal.iter().chain(&draw.bucket_long).chain(&draw.bucket_between).sum();
    record(
        output,
        "B2",
        format!(
            "{}/{}/{};{};{};{};{draw_sum}",
            draw.bucket_normal.len(),
            draw.bucket_long.len(),
            draw.bucket_between.len(),
            join_usize(&draw.bucket_normal[..5]),
            join_usize(&draw.bucket_long[..5]),
            join_usize(&draw.bucket_between[..5])
        ),
        "100/60/40;6895,5605,40371,9726,18098;50396,50274,50461,50044,50216;64169,60139,63547,63950,62245;7787893",
    );
    let short_draw = stratified_draw(3, &anchor_pools((50_000..50_030).collect()), LONG_COMMIT_BUCKET_QUOTA, BETWEEN_COMMITS_BUCKET_QUOTA, STRATIFIED_POINT_TOTAL);
    record(
        output,
        "B2b",
        format!("{}/{}/{};{}", short_draw.bucket_normal.len(), short_draw.bucket_long.len(), short_draw.bucket_between.len(), join_usize(&short_draw.bucket_normal[..3])),
        "130/30/40;26612,4589,2232",
    );
    let control_windows = CommitWindows { start_entry: 999, end_entry: 40_999, begin_entries: Vec::new(), confirm_entries: Vec::new() };
    record(
        output,
        "B2c",
        join_usize(&control_points(3, &control_windows)),
        "2232,2241,2612,4589,9184,14497,14598,15089,16593,18900,21649,26612,27155,28849,31573,32287,32340,34273,37366,38401",
    );
    let subset_anchor_sectors = [8u64, 1, 16, 8, 1, 1, 128, 8, 2, 1];
    let first_subset = unflushed_subset(0, 0, &subset_anchor_sectors);
    record(
        output,
        "B3",
        format!("{}/{:?}", bit_string(&first_subset.kept), first_subset.torn.map(|torn| (torn.position_in_unflushed, torn.sectors_kept))),
        "1000000100/Some((7, 1))",
    );
    let second_subset = unflushed_subset(2, 7, &subset_anchor_sectors);
    record(
        output,
        "B3b",
        format!("{}/{:?}", bit_string(&second_subset.kept), second_subset.torn.map(|torn| (torn.position_in_unflushed, torn.sectors_kept))),
        "0110000011/Some((2, 5))",
    );
    let write_cache_model_subset = unflushed_subset(u64::from(PowerCutArm::WriteCacheModel.index()), 0, &[1u64; 100]);
    let write_cache_model_kept = write_cache_model_subset.kept.iter().filter(|&&is_kept| is_kept).count();
    record(output, "B4", format!("{write_cache_model_kept}/{}", 100 - write_cache_model_kept), "53/47");
    for (arm_index, expected) in [
        (0u64, "1380,2481,2596,4737,5356,3368/86,69"),
        (1, "4969,4100,4435,810,3663,762/7,31"),
        (2, "5473,3722,537,2873,1713,5287/78,42"),
        (3, "4929,1614,967,5654,1819,4038/57,29"),
    ] {
        let (normal_positions, long_positions) = picked_commit_positions(arm_index, 5_900, 100);
        record(output, &format!("B5_a{arm_index}"), format!("{}/{}", join_usize(&normal_positions), join_usize(&long_positions)), expected);
    }
    let (normal_positions_without_long, long_positions_when_none) = picked_commit_positions(0, 6_000, 0);
    record(
        output,
        "B5b",
        format!("{}/{}", join_usize(&normal_positions_without_long), join_usize(&long_positions_when_none)),
        "5280,2981,296,1337,4056,4668,2830,1561/",
    );
    let marker_anchor = encode_marker(&Marker { kind: MarkerKind::Confirm, sequence: 5, block_index: Some(2), arm_index: 0 });
    record(
        output,
        "B7",
        format!("{}/{}", hexadecimal(&marker_anchor[..32]), sha256_hexadecimal(&marker_anchor)),
        "53464d4b43000000050000000000000002000000000000000000000070e31542/845bcdb2f8e741bbb81e81f3f40032f0e4bd948f1032c56d171d388abea214ee",
    );
    let synthetic_anchor_log = synthetic_log_bytes(&synthetic_anchor_log_entries(), 6);
    record(output, "B8_log", format!("{}/{}", synthetic_anchor_log.len(), sha256_hexadecimal(&synthetic_anchor_log)), "8192/078793918714a9ff52eb78e50f4871973e9073eaba9c4fcaf61b53463c098897");
    for (point, variant, expected) in [
        (6usize, ReplayVariant::DropUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (6, ReplayVariant::KeepUnflushed, "6e6d2a487aa2864b75b62637bd06aa2983e5fd5a5a8344debcdee784e774e510"),
        (6, ReplayVariant::RandomSubset, "df9020850109e36d3b6d34da043be5a0fb016ffd4004372c52dac115009c1d65"),
        (6, ReplayVariant::RandomSubsetWithTear, "6467f85ca3d8eefcb7eac7ea3cd4978d4cbb92d6b66b842b68a19adb6d46a763"),
        (5, ReplayVariant::DropUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (5, ReplayVariant::KeepUnflushed, "4d2a669e7156f1c1ec6cc051809c0a67c191b9b5c65df784ad0c717eec5a4608"),
        (5, ReplayVariant::RandomSubset, "0783c6aa6d776285d9fd1100701a4d57a6256e54332bafea82a53af6b1bacb55"),
        (5, ReplayVariant::RandomSubsetWithTear, "0783c6aa6d776285d9fd1100701a4d57a6256e54332bafea82a53af6b1bacb55"),
        (3, ReplayVariant::DropUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (3, ReplayVariant::KeepUnflushed, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (3, ReplayVariant::RandomSubset, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
        (3, ReplayVariant::RandomSubsetWithTear, "35a90229de2b1bd3674ec949ccbfc9676cd6718869f40225cc3628b9670807a0"),
    ] {
        let digest = synthetic_anchor_image_digest(synthetic_anchor_log.clone(), point, variant).unwrap_or_else(|error| format!("error:{}", sanitized(&error)));
        record(output, &format!("B8_k{point}_{}", variant.label()), digest, expected);
    }
    let mut stale_entries = synthetic_anchor_log_entries();
    stale_entries.push((20, 1, 0, vec![0x66; 512]));
    let stale_log = synthetic_log_bytes(&stale_entries, 6);
    let stale_parsed = parse_device_log(&LogBytes::InMemory(stale_log.clone()), DeclaredCountHandling::StopAtDeclared)
        .map(|device_log| device_log.entries.len())
        .unwrap_or(0);
    let stale_digest = synthetic_anchor_image_digest(stale_log.clone(), stale_parsed, ReplayVariant::KeepUnflushed).unwrap_or_else(|error| format!("error:{}", sanitized(&error)));
    record(
        output,
        "B8b",
        format!("{}/{}/{stale_parsed}/{stale_digest}", stale_log.len(), sha256_hexadecimal(&stale_log)),
        "9216/9956234bb62be39429e2086b19e37c511266fa9e56c16d3429579ba9a9803bd1/6/6e6d2a487aa2864b75b62637bd06aa2983e5fd5a5a8344debcdee784e774e510",
    );
    let synthetic_model_log = LogBytes::InMemory(synthetic_write_cache_model_log());
    let synthetic_model = parse_device_log(&synthetic_model_log, DeclaredCountHandling::StopAtDeclared)
        .map_err(|error| error.to_string())
        .and_then(|device_log| judge_write_cache_model(&synthetic_model_log, &device_log));
    let synthetic_model_text = match synthetic_model {
        Ok(judgement) => {
            let lost: Vec<String> = judgement
                .lost_by_variant
                .iter()
                .map(|(variant, first_lost, second_lost)| format!("{}:{first_lost},{second_lost}", variant.label()))
                .collect();
            format!(
                "{}/{}/{}/{}/{}",
                judgement.point,
                judgement.entries_between_batches,
                judgement.flushes_between_batches,
                lost.join(";"),
                judgement.registered_counts_hold
            )
        }
        Err(error) => format!("error:{}", sanitized(&error)),
    };
    record(output, "PC4-C_synthetic", synthetic_model_text, "201/1/1/U-drop:0,100;U-keep:0,0;U-rand:0,47;U-tear:0,47/true");
    let control_differences: Vec<String> = ALL_POWER_CUT_ARMS
        .iter()
        .filter_map(|arm| {
            let settings = arm.write_settings()?;
            let correct = arm.correct_form().write_settings()?;
            let differing = settings.differing_setting_names(correct);
            let registered: Vec<&str> = arm.registered_control_difference().into_iter().collect();
            (differing != registered).then(|| format!("{}:{}", arm.label(), differing.join("+")))
        })
        .collect();
    record(output, "M26_control_forms", if control_differences.is_empty() { "exactly_registered".to_string() } else { control_differences.join(",") }, "exactly_registered");
    let marker_capacity_needed = 2 * ROCKSDB_REDO_BLOCK_LIMIT + 2;
    record(output, "B9b", format!("{}/{}", 2 * MAIN_BLOCK_COUNT + 2, marker_partition_sectors()), "12002/131072");
    record(
        output,
        "B9c",
        format!(
            "{FIRST_PARTITION_START_SECTOR}/{}/{}/{}/{}",
            first_partition_sectors(),
            marker_partition_start_sector(),
            marker_partition_sectors(),
            (marker_partition_start_sector() + marker_partition_sectors()) * sector_bytes() / MEBIBYTE
        ),
        "2048/6291456/6293504/131072/3137",
    );
    record(
        output,
        "B9d",
        format!("{}", marker_capacity_needed <= marker_partition_sectors() && (marker_partition_start_sector() + marker_partition_sectors()) * sector_bytes() <= DISK_MEBIBYTES * MEBIBYTE),
        "true",
    );
    mismatches
}
