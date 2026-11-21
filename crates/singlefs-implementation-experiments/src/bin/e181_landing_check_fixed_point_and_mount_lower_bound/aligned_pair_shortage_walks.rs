//! 按种子走的随机步：会话池、步的动作、今天能不能再建一个 inode、步的合计与观测。

use super::*;

/// 会话层的池：盘加一次可写挂载之后的会话。拷贝它不拷盘上的扇区。
pub(super) struct SessionPool {
    pub(super) parameters: MakeFilesystemParameters,
    pub(super) devices: Vec<(DeviceIdentity, ForkedDevice)>,
    pub(super) session: MountedSession,
    /// 这个池里用 `NewInodes` 建出来、还没删的 inode 号（`DeleteACreatedInode` 从尾上取）。
    pub(super) created_inodes: Vec<u64>,
    /// 下一次写进记录的时间秒：每次改动加 1，只为让记录不同。
    pub(super) write_time_seconds: u64,
}

impl SessionPool {
    fn of_a_mount(
        parameters: MakeFilesystemParameters,
        mut devices: Vec<(DeviceIdentity, ForkedDevice)>,
        created_inodes: Vec<u64>,
        write_time_seconds: u64,
    ) -> Result<Self, String> {
        let mounted = mount_writable(&parameters, &mut devices)
            .map_err(|error| format!("可写挂载被拒：{error:?}"))?;
        let (_output, session) = MountedSession::of_the_mount(mounted);
        Ok(Self {
            parameters,
            devices,
            session,
            created_inodes,
            write_time_seconds,
        })
    }

    pub(super) fn fork(&mut self) -> SessionPool {
        for (_identity, device) in &mut self.devices {
            drop(device.fork());
        }
        self.fork_of_a_sealed_base()
    }

    /// 底已经封存过（顶层空）时，不需要 `&mut` 的拷贝：几个线程各拷一份。
    pub(super) fn fork_of_a_sealed_base(&self) -> SessionPool {
        SessionPool {
            parameters: self.parameters.clone(),
            devices: self
                .devices
                .iter()
                .map(|(identity, device)| (*identity, device.fork_of_a_sealed_base()))
                .collect(),
            session: MountedSession {
                allocator: self.session.allocator.clone(),
                current: self.session.current.clone(),
                instance: self.session.instance,
                shadow_ledger: self.session.shadow_ledger,
                parameters_and_device_table: self.session.parameters_and_device_table.clone(),
                compaction: self.session.compaction.clone(),
            },
            created_inodes: self.created_inodes.clone(),
            write_time_seconds: self.write_time_seconds,
        }
    }
}

/// 在 [`build_pool_after_the_first_file`] 的盘上可写挂载一次，交回会话层的池。
pub(super) fn session_pool_of(pool: &mut Pool) -> Result<SessionPool, String> {
    let devices = pool
        .devices
        .iter_mut()
        .map(|(identity, device)| (*identity, device.fork()))
        .collect();
    SessionPool::of_a_mount(
        pool.parameters.clone(),
        devices,
        Vec::new(),
        FIXED_WRITE_TIME_SECONDS,
    )
}

/// 种子驱动的随机数（splitmix64）：每个种子一条自己的流，凭种子能单独复现。
pub(super) struct SeededStream {
    pub(super) state: u64,
}

impl SeededStream {
    pub(super) fn of_seed(seed: u64) -> Self {
        Self {
            state: seed
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0x1234_5678_9ABC_DEF1),
        }
    }

    fn next_number(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    pub(super) fn below(&mut self, bound: u64) -> u64 {
        self.next_number() % bound
    }
}

/// 一步走法（登记 5.2 节：{NewInodes 1 / 8 / 64 个、删一个 NewInodes 建的空 inode、正常卸载再可写挂载}）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WalkAction {
    CreateOneInode,
    CreateEightInodes,
    CreateSixtyFourInodes,
    DeleteACreatedInode,
    UnmountAndRemount,
}

pub(super) const WALK_ACTIONS: [WalkAction; 5] = [
    WalkAction::CreateOneInode,
    WalkAction::CreateEightInodes,
    WalkAction::CreateSixtyFourInodes,
    WalkAction::DeleteACreatedInode,
    WalkAction::UnmountAndRemount,
];

impl WalkAction {
    pub(super) fn name(self) -> &'static str {
        match self {
            WalkAction::CreateOneInode => "create_one_inode",
            WalkAction::CreateEightInodes => "create_eight_inodes",
            WalkAction::CreateSixtyFourInodes => "create_sixty_four_inodes",
            WalkAction::DeleteACreatedInode => "delete_a_created_inode",
            WalkAction::UnmountAndRemount => "unmount_and_remount",
        }
    }
}

/// 走法的结局：成员名（成了、或被拒的成员名）。
pub(super) fn variant_name_of(debug_text: &str) -> String {
    debug_text
        .split(['(', ' ', '{'])
        .next()
        .unwrap_or("")
        .to_string()
}

pub(super) fn apply_walk_action(
    state: &mut SessionPool,
    action: WalkAction,
) -> Result<String, String> {
    state.write_time_seconds += 1;
    let change = match action {
        WalkAction::CreateOneInode => Some(UserChange::NewInodes {
            count: 1,
            write_time_seconds: state.write_time_seconds,
        }),
        WalkAction::CreateEightInodes => Some(UserChange::NewInodes {
            count: 8,
            write_time_seconds: state.write_time_seconds,
        }),
        WalkAction::CreateSixtyFourInodes => Some(UserChange::NewInodes {
            count: 64,
            write_time_seconds: state.write_time_seconds,
        }),
        WalkAction::DeleteACreatedInode => {
            state.created_inodes.last().map(|inode| UserChange::Unlink {
                inode: InodeNumber(*inode),
                write_time_seconds: state.write_time_seconds,
            })
        }
        WalkAction::UnmountAndRemount => None,
    };
    match (action, change) {
        (WalkAction::UnmountAndRemount, _) => {
            unmount(
                &state.parameters,
                &mut state.devices,
                &mut state.session.allocator,
                &mut state.session.current,
                state.session.shadow_ledger,
            )
            .map_err(|error| format!("卸载被拒：{error:?}"))?;
            let devices = state
                .devices
                .iter_mut()
                .map(|(identity, device)| (*identity, device.fork()))
                .collect();
            let remounted = SessionPool::of_a_mount(
                state.parameters.clone(),
                devices,
                std::mem::take(&mut state.created_inodes),
                state.write_time_seconds,
            )?;
            *state = remounted;
            Ok("remounted".to_string())
        }
        (_, None) => Ok("nothing_to_delete".to_string()),
        (_, Some(change)) => {
            let first_new_inode = state
                .session
                .current
                .file_version()
                .map(TransactionOutput::inode_number_watermark);
            let published = state
                .session
                .publish_user_change(&mut state.devices, change);
            match published {
                Ok(_published) => {
                    match (action, first_new_inode) {
                        (WalkAction::CreateOneInode, Some(first)) => {
                            state.created_inodes.push(first)
                        }
                        (WalkAction::CreateEightInodes, Some(first)) => {
                            state.created_inodes.extend(first..first + 8);
                        }
                        (WalkAction::CreateSixtyFourInodes, Some(first)) => {
                            state.created_inodes.extend(first..first + 64);
                        }
                        (WalkAction::DeleteACreatedInode, _) => {
                            state.created_inodes.pop();
                        }
                        (
                            WalkAction::CreateOneInode
                            | WalkAction::CreateEightInodes
                            | WalkAction::CreateSixtyFourInodes
                            | WalkAction::UnmountAndRemount,
                            None,
                        )
                        | (WalkAction::UnmountAndRemount, Some(_)) => {}
                    }
                    Ok("published".to_string())
                }
                Err(refusal) => Ok(format!(
                    "refused_{}",
                    variant_name_of(&format!("{refusal:?}"))
                )),
            }
        }
    }
}

/// F10 的盘面谓词 G10 的读数（登记 5.2 节）：下一次 `NewInodes` 1 个那次发布要落的提交内生块——inode 叶容器（两槽）与 inode 树根（一槽）
/// 是普通分配那一半，加固定点按 ckpt_cost 的最坏数顶（`admission::units_of_a_publish_to_land`）——对照每块盘此刻没挡的槽与对齐的空槽对。
pub(super) const NEW_INODE_ORDINARY_COMMIT_GENERATED_SLOTS: u64 = 3;
/// 同一次发布要落的两槽提交内生块个数：一片 inode 叶容器。
const NEW_INODE_TWO_SLOT_COMMIT_GENERATED_UNITS: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RoomReading {
    /// 每块盘对齐的空槽对数：起点偶数、两槽都没挡，聚簇段内外都算（空集合作段表，登记「读法写死」）。
    pub(super) aligned_free_pairs: Vec<u64>,
    /// 每块盘没挡的槽数。
    pub(super) unblocked_slots: Vec<u64>,
    pub(super) needed_slots: u64,
    pub(super) needed_pairs: u64,
}

impl RoomReading {
    pub(super) fn of(state: &SessionPool) -> Self {
        let allocator = &state.session.allocator;
        let no_cluster_segments: BTreeSet<SlotNumber> = BTreeSet::new();
        let version = state
            .session
            .current
            .file_version()
            .expect("会话的现行那一版带文件");
        Self {
            aligned_free_pairs: allocator
                .devices
                .iter()
                .map(|device_map| {
                    device_map
                        .unblocked_slot_pairs_outside_the_cluster_segments(&no_cluster_segments)
                        .0
                })
                .collect(),
            unblocked_slots: allocator
                .devices
                .iter()
                .map(|device_map| device_map.unblocked_slots())
                .collect(),
            needed_slots: NEW_INODE_ORDINARY_COMMIT_GENERATED_SLOTS
                + core_checkpoint_cost(
                    version,
                    allocator,
                    CodeTwoTreeNodeCapacities::FromTheNodeFormat,
                ),
            needed_pairs: NEW_INODE_TWO_SLOT_COMMIT_GENERATED_UNITS,
        }
    }

    /// 落得下那一判 ②（只数槽）放行：每块盘没挡的槽都够。
    pub(super) fn slots_are_enough_on_every_device(&self) -> bool {
        self.unblocked_slots
            .iter()
            .all(|slots| *slots >= self.needed_slots)
    }

    /// 至少一块盘的对齐空槽对不够。
    pub(super) fn aligned_pairs_are_short_on_some_device(&self) -> bool {
        self.aligned_free_pairs
            .iter()
            .any(|pairs| *pairs < self.needed_pairs)
    }

    /// G10 = 没挡的槽够 ∧ 对齐的空槽对不够。两个量各报各的，这里是登记写死的那一个合取。
    pub(super) fn holds(&self) -> bool {
        self.slots_are_enough_on_every_device() && self.aligned_pairs_are_short_on_some_device()
    }
}

/// 在状态的拷贝上直接走发布路径发一次 `NewInodes` 1 个（今天那一臂 Q2）：落得下那一判与固定点各拒不拒。
pub(super) fn probe_one_new_inode_today(state: &mut SessionPool) -> RefusalOrPublished {
    let mut copy = state.fork();
    let previous = copy
        .session
        .current
        .file_version()
        .expect("现行那一版带文件")
        .clone();
    let mut writer = PoolWriter::new(&copy.parameters, &mut copy.devices);
    match publish_new_inodes(
        &mut writer,
        &mut copy.session.allocator,
        &previous,
        1,
        FIXED_WRITE_TIME_SECONDS,
        copy.session.instance,
    ) {
        Ok(_output) => RefusalOrPublished::Published,
        Err(error) => RefusalOrPublished::Refused(RefusalClass::of(&error)),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RefusalOrPublished {
    Published,
    Refused(RefusalClass),
}

impl RefusalOrPublished {
    pub(super) fn name(&self) -> &'static str {
        match self {
            RefusalOrPublished::Published => "published",
            RefusalOrPublished::Refused(refusal) => refusal.name(),
        }
    }
}

/// 在状态的拷贝上经会话发一次 `NewInodes` 1 个：会话推抬 F 之后用户看到的结局。
fn session_outcome_of_one_new_inode(state: &mut SessionPool) -> String {
    let mut copy = state.fork();
    let write_time_seconds = copy.write_time_seconds + 1;
    let published = copy.session.publish_user_change(
        &mut copy.devices,
        UserChange::NewInodes {
            count: 1,
            write_time_seconds,
        },
    );
    match published {
        Ok(done) => format!("published_after_{}_floor_raises", done.floor_raises.len()),
        Err(refusal) => format!("refused_{}", variant_name_of(&format!("{refusal:?}"))),
    }
}

/// 一个种子走完 300 步的账。
#[derive(Clone, Debug, Default)]
pub(super) struct WalkTally {
    pub(super) walks: u64,
    pub(super) states: u64,
    pub(super) slots_enough_and_aligned_pairs_short_states: u64,
    pub(super) first_slots_enough_and_aligned_pairs_short_step: Option<u64>,
    pub(super) aligned_pairs_peak: u64,
    pub(super) aligned_pairs_positive_states: u64,
    aligned_pairs_final_minimum: Option<u64>,
    pub(super) unblocked_slots_peak: u64,
    unblocked_slots_final_minimum: Option<u64>,
    pub(super) slots_enough_states: u64,
    pub(super) pairs_short_states: u64,
    pub(super) action_refusals: std::collections::BTreeMap<String, u64>,
    pub(super) slots_enough_and_aligned_pairs_short_today_outcomes:
        std::collections::BTreeMap<&'static str, u64>,
    pub(super) slots_enough_and_aligned_pairs_short_session_outcomes:
        std::collections::BTreeMap<String, u64>,
    pub(super) model_checks: u64,
    pub(super) placement_refusals_with_slots_enough_and_aligned_pairs_short_false: u64,
    pub(super) slots_enough_and_aligned_pairs_short_probes_that_published: u64,
}

impl WalkTally {
    fn merge(&mut self, other: &WalkTally) {
        self.walks += other.walks;
        self.states += other.states;
        self.slots_enough_and_aligned_pairs_short_states +=
            other.slots_enough_and_aligned_pairs_short_states;
        self.first_slots_enough_and_aligned_pairs_short_step = match (
            self.first_slots_enough_and_aligned_pairs_short_step,
            other.first_slots_enough_and_aligned_pairs_short_step,
        ) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (Some(only), None) | (None, Some(only)) => Some(only),
            (None, None) => None,
        };
        self.aligned_pairs_peak = self.aligned_pairs_peak.max(other.aligned_pairs_peak);
        self.aligned_pairs_positive_states += other.aligned_pairs_positive_states;
        self.aligned_pairs_final_minimum = minimum_of_unsigned_options(
            self.aligned_pairs_final_minimum,
            other.aligned_pairs_final_minimum,
        );
        self.unblocked_slots_peak = self.unblocked_slots_peak.max(other.unblocked_slots_peak);
        self.unblocked_slots_final_minimum = minimum_of_unsigned_options(
            self.unblocked_slots_final_minimum,
            other.unblocked_slots_final_minimum,
        );
        self.slots_enough_states += other.slots_enough_states;
        self.pairs_short_states += other.pairs_short_states;
        for (action, count) in &other.action_refusals {
            *self.action_refusals.entry(action.clone()).or_default() += count;
        }
        for (outcome, count) in &other.slots_enough_and_aligned_pairs_short_today_outcomes {
            *self
                .slots_enough_and_aligned_pairs_short_today_outcomes
                .entry(outcome)
                .or_default() += count;
        }
        for (outcome, count) in &other.slots_enough_and_aligned_pairs_short_session_outcomes {
            *self
                .slots_enough_and_aligned_pairs_short_session_outcomes
                .entry(outcome.clone())
                .or_default() += count;
        }
        self.model_checks += other.model_checks;
        self.placement_refusals_with_slots_enough_and_aligned_pairs_short_false +=
            other.placement_refusals_with_slots_enough_and_aligned_pairs_short_false;
        self.slots_enough_and_aligned_pairs_short_probes_that_published +=
            other.slots_enough_and_aligned_pairs_short_probes_that_published;
    }
}

fn minimum_of_unsigned_options(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(only), None) | (None, Some(only)) => Some(only),
        (None, None) => None,
    }
}

/// 每隔几步，在 G10 为假的状态上也发一次今天那一臂，核谓词没漏掉被拒在固定点的状态。
const MODEL_CHECK_EVERY_STEPS: u64 = 25;

pub(super) fn observe_state(tally: &mut WalkTally, state: &mut SessionPool, step: u64) {
    let reading = RoomReading::of(state);
    tally.states += 1;
    let aligned_minimum = reading
        .aligned_free_pairs
        .iter()
        .copied()
        .min()
        .unwrap_or(0);
    let unblocked_minimum = reading.unblocked_slots.iter().copied().min().unwrap_or(0);
    tally.aligned_pairs_peak = tally.aligned_pairs_peak.max(aligned_minimum);
    tally.unblocked_slots_peak = tally.unblocked_slots_peak.max(unblocked_minimum);
    tally.aligned_pairs_positive_states += u64::from(aligned_minimum > 0);
    tally.slots_enough_states += u64::from(reading.slots_are_enough_on_every_device());
    tally.pairs_short_states += u64::from(reading.aligned_pairs_are_short_on_some_device());
    if reading.holds() {
        tally.slots_enough_and_aligned_pairs_short_states += 1;
        tally.first_slots_enough_and_aligned_pairs_short_step = tally
            .first_slots_enough_and_aligned_pairs_short_step
            .or(Some(step));
        let today = probe_one_new_inode_today(state);
        *tally
            .slots_enough_and_aligned_pairs_short_today_outcomes
            .entry(today.name())
            .or_default() += 1;
        tally.slots_enough_and_aligned_pairs_short_probes_that_published +=
            u64::from(today == RefusalOrPublished::Published);
        let session_outcome = session_outcome_of_one_new_inode(state);
        *tally
            .slots_enough_and_aligned_pairs_short_session_outcomes
            .entry(session_outcome)
            .or_default() += 1;
    } else if step.is_multiple_of(MODEL_CHECK_EVERY_STEPS) {
        tally.model_checks += 1;
        let today = probe_one_new_inode_today(state);
        tally.placement_refusals_with_slots_enough_and_aligned_pairs_short_false +=
            u64::from(matches!(
                today,
                RefusalOrPublished::Refused(RefusalClass::FixedPointPlacement { .. })
            ));
    }
}

/// 一个种子的 300 步：每步从五种走法里按种子挑一个，走完观测一次盘面。`state` 是这个种子自己的一份拷贝。
fn walk_one_seed_from(state: &mut SessionPool, seed: u64, steps: u64) -> Result<WalkTally, String> {
    let mut stream = SeededStream::of_seed(seed);
    let mut tally = WalkTally {
        walks: 1,
        ..WalkTally::default()
    };
    for step in 0..steps {
        let action = WALK_ACTIONS[usize::try_from(stream.below(5)).expect("下标")];
        let result = apply_walk_action(state, action)?;
        if result.starts_with("refused_") {
            *tally
                .action_refusals
                .entry(format!("{}:{result}", action.name()))
                .or_default() += 1;
        }
        observe_state(&mut tally, state, step);
    }
    let reading = RoomReading::of(state);
    tally.aligned_pairs_final_minimum = reading.aligned_free_pairs.iter().copied().min();
    tally.unblocked_slots_final_minimum = reading.unblocked_slots.iter().copied().min();
    Ok(tally)
}

/// 走种子的线程数：设了 `E181_THREADS` 照它，没设取每台默认重活线程数（`singlefs_verification::heavy_threads`）。
fn walk_thread_count() -> usize {
    let thread_cap_value =
        std::env::var(singlefs_verification::heavy_threads::THREAD_CAP_ENVIRONMENT_VARIABLE).ok();
    singlefs_verification::heavy_threads::own_variable_or_default_heavy_threads(
        "E181_THREADS",
        std::env::var("E181_THREADS").ok().as_deref(),
        thread_cap_value.as_deref(),
    )
    .get()
}

/// 登记 5.2 节的整段 F10：填满数据那一侧、可写挂载，64 个种子各走 300 步（线程数见 [`walk_thread_count`]；
/// 每个线程各拷一份底，种子按序号取，合并按种子序，所以线程数不改产物）。种子数与步数可由 `E181_ALIGNED_PAIR_SHORTAGE_SEEDS`、`E181_ALIGNED_PAIR_SHORTAGE_STEPS` 往小缩
/// （设得比登记值大的拒，见 [`narrowed_count_or_registered`]），缩过的那一趟在产物里写明（登记第八节：缩过规模的不当整段）。
pub(super) fn run_aligned_pair_shortage_walks(
    geometry: Geometry,
    content: &mut ContentBuffer,
) -> Result<WalkTally, String> {
    let seeds = narrowed_count_or_registered(
        "E181_ALIGNED_PAIR_SHORTAGE_SEEDS",
        std::env::var("E181_ALIGNED_PAIR_SHORTAGE_SEEDS")
            .ok()
            .as_deref(),
        ALIGNED_PAIR_SHORTAGE_SEEDS,
    )?;
    let steps = narrowed_count_or_registered(
        "E181_ALIGNED_PAIR_SHORTAGE_STEPS",
        std::env::var("E181_ALIGNED_PAIR_SHORTAGE_STEPS")
            .ok()
            .as_deref(),
        ALIGNED_PAIR_SHORTAGE_STEPS,
    )?;
    let threads = walk_thread_count();
    let mut pool =
        build_pool_after_the_first_file(geometry, CodeTwoTreeNodeCapacities::FromTheNodeFormat);
    let fill = fill_the_data_side(&mut pool, content, FILL_NO_CAP_ON_THE_UNITS_OF_A_PUBLISH)?;
    let mut base = session_pool_of(&mut pool)?;
    drop(pool);
    let initial = RoomReading::of(&base);
    emit(&format!(
        "name=aligned_pair_shortage_setup geometry={} {} seeds={seeds} steps={steps} seeds_and_steps_are_the_registered_ones={} threads={threads} initial_aligned_free_pairs={:?} initial_unblocked_slots={:?} needed_slots={}",
        geometry.name,
        fill.fields(),
        seeds == ALIGNED_PAIR_SHORTAGE_SEEDS && steps == ALIGNED_PAIR_SHORTAGE_STEPS,
        initial.aligned_free_pairs,
        initial.unblocked_slots,
        initial.needed_slots,
    ));
    drop(base.fork());
    let sealed_base = &base;
    let next_seed = std::sync::atomic::AtomicU64::new(0);
    let finished: std::sync::Mutex<Vec<(u64, Result<WalkTally, String>)>> =
        std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _thread in 0..threads {
            scope.spawn(|| loop {
                let seed = next_seed.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if seed >= seeds {
                    break;
                }
                let mut state = sealed_base.fork_of_a_sealed_base();
                let walked = walk_one_seed_from(&mut state, seed, steps);
                finished.lock().expect("锁").push((seed, walked));
            });
        }
    });
    let mut results = finished.into_inner().expect("锁");
    results.sort_by_key(|(seed, _)| *seed);
    let mut total = WalkTally::default();
    for (seed, walked) in results {
        let tally = walked.map_err(|reason| format!("种子 {seed}：{reason}"))?;
        emit(&format!(
            "name=aligned_pair_shortage_seed geometry={} seed={seed} slots_enough_and_aligned_pairs_short_states={} first_slots_enough_and_aligned_pairs_short_step={} aligned_pairs_peak={} aligned_pairs_positive_states={} aligned_pairs_final={} unblocked_slots_peak={} unblocked_slots_final={}",
            geometry.name,
            tally.slots_enough_and_aligned_pairs_short_states,
            tally.first_slots_enough_and_aligned_pairs_short_step.map_or_else(|| "none".to_string(), |step| step.to_string()),
            tally.aligned_pairs_peak,
            tally.aligned_pairs_positive_states,
            text_of_unsigned(tally.aligned_pairs_final_minimum),
            tally.unblocked_slots_peak,
            text_of_unsigned(tally.unblocked_slots_final_minimum),
        ));
        total.merge(&tally);
    }
    emit_aligned_pair_shortage_summary(geometry.name, seeds, steps, &total);
    Ok(total)
}
