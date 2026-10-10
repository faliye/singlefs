
// ====================== 本轮正推腿（m3-batch10z-vw-code-r1，sonnet 腿）K1-4：报错那一步与要出路那一步碰了什么 ======================
// 量法副本（不是产品代码）：unlink.rs 的 DeletionSwitches 手写 Clone 带计数（PoolAllocator 每克隆一次就克隆它一次）；
// mounted_session.rs 里 VW_SKIP_REPORT 置着时 wide_protection_at_the_ceiling 一进来就交 NotWhatStopsTheAdmission（量「报错那一步」的差）。

struct VwPlan {
    reads: Cell<u64>,
    writes: Cell<u64>,
    zeroes: Cell<u64>,
    barriers: Cell<u64>,
    probes: Cell<u64>,
    sizes: Cell<u64>,
    fail_read_at: Cell<Option<u64>>,
    fired: Cell<u64>,
    pin_at: Cell<Option<u64>>,
    fail_write_after: Cell<i64>,
    fail_all_from: Cell<Option<u64>>,
    pinned: std::cell::RefCell<Vec<(DeviceIdentity, u64, u64)>>,
}

impl VwPlan {
    fn new() -> Rc<Self> {
        Rc::new(VwPlan {
            reads: Cell::new(0),
            writes: Cell::new(0),
            zeroes: Cell::new(0),
            barriers: Cell::new(0),
            probes: Cell::new(0),
            sizes: Cell::new(0),
            fail_read_at: Cell::new(None),
            fired: Cell::new(0),
            pin_at: Cell::new(None),
            fail_write_after: Cell::new(-1),
            fail_all_from: Cell::new(None),
            pinned: std::cell::RefCell::new(Vec::new()),
        })
    }
    fn reset(&self) {
        for cell in [&self.reads, &self.writes, &self.zeroes, &self.barriers, &self.probes, &self.sizes, &self.fired] {
            cell.set(0);
        }
    }
    fn counts(&self) -> [u64; 6] {
        [self.reads.get(), self.writes.get(), self.zeroes.get(), self.barriers.get(), self.probes.get(), self.sizes.get()]
    }
}

struct VwDev {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    plan: Rc<VwPlan>,
}

impl BlockDevice for VwDev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let n = self.plan.reads.get() + 1;
        self.plan.reads.set(n);
        if self.plan.pin_at.get() == Some(n) {
            self.plan.pinned.borrow_mut().push((self.identity, offset.0, offset.0 + buffer.len() as u64));
        }
        let end = offset.0 + buffer.len() as u64;
        if self.plan.pinned.borrow().iter().any(|(device, start, stop)| *device == self.identity && offset.0 < *stop && end > *start) {
            self.plan.fired.set(self.plan.fired.get() + 1);
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("vw 钉住读不出")));
        }
        if self.plan.fail_all_from.get().is_some_and(|from| n >= from) {
            self.plan.fired.set(self.plan.fired.get() + 1);
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("vw 从此全部读不出")));
        }
        if self.plan.fail_read_at.get() == Some(n) {
            self.plan.fired.set(self.plan.fired.get() + 1);
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("vw 注入的读错")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.plan.writes.set(self.plan.writes.get() + 1);
        let left = self.plan.fail_write_after.get();
        if left == 0 {
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("vw 注入的写错")));
        }
        if left > 0 {
            self.plan.fail_write_after.set(left - 1);
        }
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.plan.zeroes.set(self.plan.zeroes.get() + 1);
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.plan.barriers.set(self.plan.barriers.get() + 1);
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.plan.probes.set(self.plan.probes.get() + 1);
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.plan.sizes.set(self.plan.sizes.get() + 1);
        self.inner.size_in_bytes()
    }
}

impl DeviceOverASparseImage for VwDev {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

impl AtkSnap {
    fn pool_vw(&self, plan: &Rc<VwPlan>) -> PoolUnderTest<VwDev> {
        PoolUnderTest {
            parameters: self.parameters.clone(),
            device_bytes: self.device_bytes,
            devices: plain_devices_on(&self.image)
                .into_iter()
                .map(|(identity, inner)| (identity, VwDev { identity, inner, plan: plan.clone() }))
                .collect(),
            session: self.session.as_ref().map(copy_of_the_session),
            write_time_seconds: self.write_time_seconds,
            content_of_the_current_version: self.content_now.clone(),
            content_of_each_root: self.contents.clone(),
        }
    }
}

fn vw_publish<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, write: &AtkWrite) -> Result<AtkPublished, UserChangeRefused> {
    let file = AtkFirstFile { content: &write.content, write_time_seconds: write.write_time_seconds };
    let change = if write.overwrite { AtkUserChange::Overwrite(file) } else { AtkUserChange::SequentialWrite(file) };
    pool.publish(change, Some(&write.content))
}

fn vw_outcome_head(outcome: &Result<AtkPublished, UserChangeRefused>) -> String {
    atk_kind(outcome)
}

/// 报错那一步（量差）：同一份快照上同一个写发两次，一次照常（交「被宽护卡住」），一次 VW_SKIP_REPORT 置着（那一步交 NotWhat）；
/// 两次的盘调用计数（读、写、写零、屏障、探物理块、问大小）与分配器克隆数应逐项相同 = 那一步一项都没碰。
/// 要出路那一步：单独量计数；会话、分配器、盘不变；读错注入扫过它的每一次读。
#[test]
#[ignore = "正推腿 K1-4，手动跑"]
fn vw_k14_report_step_touches_nothing_and_ways_out_changes_nothing() {
    let cells = atk_cells_from_env();
    let mut cells_stuck = 0;
    for (pre, fill, grow) in cells {
        let (snap, units) = atk_k1_cell(pre, fill, grow);
        // 一、报错那一步：量差
        let mut counts_of_run = Vec::new();
        for skip in [false, true] {
            let plan = VwPlan::new();
            let mut pool = snap.pool_vw(&plan);
            let write = atk_write_of(&mut pool, units);
            plan.reset();
            singlefs_core::mounted_session::VW_SKIP_REPORT.store(skip, std::sync::atomic::Ordering::SeqCst);
            let clones_before = singlefs_core::unlink::ALLOCATOR_CLONES.load(std::sync::atomic::Ordering::SeqCst);
            let outcome = vw_publish(&mut pool, &write);
            let clones = singlefs_core::unlink::ALLOCATOR_CLONES.load(std::sync::atomic::Ordering::SeqCst) - clones_before;
            singlefs_core::mounted_session::VW_SKIP_REPORT.store(false, std::sync::atomic::Ordering::SeqCst);
            counts_of_run.push((plan.counts(), clones, vw_outcome_head(&outcome), format!("{:?}", pool.session().allocator) , pool.image() == snap.pool().image(), format!("{:?}", pool.session().current.root())));
        }
        let (c0, c1) = (&counts_of_run[0], &counts_of_run[1]);
        let stuck_run = c0.2 == "stuck";
        if stuck_run { cells_stuck += 1; }
        println!(
            "VW k14a pre={pre} fill={fill} g={grow} report={} forced_notwhat={} counts[r,w,z,b,probe,size]={:?} vs {:?} allocator_clones={} vs {} same_counts={} same_clones={} same_allocator_after={} same_current_after={}",
            c0.2, c1.2, c0.0, c1.0, c0.1, c1.1, c0.0 == c1.0, c0.1 == c1.1, c0.3 == c1.3, c0.5 == c1.5
        );
        if !stuck_run { continue; }
        // 二、要出路那一步
        let plan = VwPlan::new();
        let mut pool = snap.pool_vw(&plan);
        let write = atk_write_of(&mut pool, units);
        let outcome = vw_publish(&mut pool, &write);
        let stuck = atk_stuck_of(&outcome).expect("stuck");
        let allocator_before = format!("{:?}", pool.session().allocator);
        let image_before = pool.image();
        let current_before = format!("{:?}", pool.session().current);
        plan.reset();
        let clones_before = singlefs_core::unlink::ALLOCATOR_CLONES.load(std::sync::atomic::Ordering::SeqCst);
        let ways = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
        let clones = singlefs_core::unlink::ALLOCATOR_CLONES.load(std::sync::atomic::Ordering::SeqCst) - clones_before;
        let counts = plan.counts();
        let unchanged = allocator_before == format!("{:?}", pool.session().allocator) && image_before == pool.image() && current_before == format!("{:?}", pool.session().current);
        let total_reads = counts[0];
        println!(
            "VW k14b pre={pre} fill={fill} g={grow} ways_out_counts[r,w,z,b,probe,size]={counts:?} allocator_clones={clones} head={} unchanged={unchanged}",
            atk_rehearsal_head_from(&ways)
        );
        // 三、读错注入：第 n 次读报错一次，n 扫过要出路那一步的每一次读
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut moved = 0usize;
        for n in 1..=total_reads {
            plan.reset();
            plan.fail_read_at.set(Some(n));
            let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
            plan.fail_read_at.set(None);
            let same = allocator_before == format!("{:?}", pool.session().allocator) && image_before == pool.image() && current_before == format!("{:?}", pool.session().current);
            if !same { moved += 1; }
            *tally.entry(format!("{} ways={}", atk_rehearsal_head_from(&w), w.ways_out.len())).or_insert(0) += 1;
        }
        println!("VW k14c pre={pre} fill={fill} g={grow} reads_in_ways_out={total_reads} session_or_disk_changed={moved} tally={tally:?}");
    }
    println!("VW k14 done cells_stuck={cells_stuck}");
}

fn atk_rehearsal_head_from(w: &singlefs_core::mounted_session::WaysOutOfTheWideProtection) -> String {
    let head = |text: String| text.chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect::<String>();
    match &w.pushing_the_root_ring {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush { .. } => "adm".to_string(),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { refusal, .. } => format!("still[{}]", head(format!("{refusal:?}"))),
        RootRingPushRehearsal::PushRefused(refusal) => format!("pushref[{}]", head(format!("{refusal:?}"))),
    }
}

/// 要出路那一步里第 n 次读读的那一段从此读不出（写也不撤）：要出路交什么，对照「同一组故障上真推环、再发同一个写」的结局。
#[test]
#[ignore = "正推腿 K1-4，手动跑"]
fn vw_k14_persistent_read_fault_during_ways_out_vs_truth() {
    let stride: u64 = std::env::var("VW_STRIDE").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    for (pre, fill, grow) in atk_cells_from_env() {
        let (snap, units) = atk_k1_cell(pre, fill, grow);
        // 先量要出路那一步有多少次读
        let plan = VwPlan::new();
        let mut pool = snap.pool_vw(&plan);
        let write = atk_write_of(&mut pool, units);
        let outcome = vw_publish(&mut pool, &write);
        let stuck = atk_stuck_of(&outcome).expect("stuck");
        plan.reset();
        let _ = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
        let total = plan.reads.get();
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut first_of: BTreeMap<String, u64> = BTreeMap::new();
        let mut n = 1;
        while n <= total {
            let plan = VwPlan::new();
            let mut pool = snap.pool_vw(&plan);
            let write = atk_write_of(&mut pool, units);
            let outcome = vw_publish(&mut pool, &write);
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let allocator_before = format!("{:?}", pool.session().allocator);
            let image_before = pool.image();
            plan.reset();
            plan.pin_at.set(Some(n));
            let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
            let fired_in_ways_out = plan.fired.get();
            let unchanged = allocator_before == format!("{:?}", pool.session().allocator) && image_before == pool.image();
            let reh = atk_rehearsal_head_from(&w);
            let listed = w.ways_out.len() == 2;
            // 真走：同一组故障（钉住的段还在）上推环、再发同一个写
            plan.pin_at.set(None);
            let pushed = push_the_root_ring_of(&mut pool);
            let truth = match &pushed {
                Err(refusal) => format!("pushref[{}]", format!("{refusal:?}").chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect::<String>()),
                Ok(_) => {
                    let again = vw_publish(&mut pool, &write);
                    if again.is_ok() { "adm".to_string() } else { "still".to_string() }
                }
            };
            let class = |text: &str| if text.starts_with("adm") { "adm" } else if text.starts_with("still") { "still" } else if text.starts_with("pushref") { "pushref" } else { "other" }.to_string();
            let agree = class(&reh) == class(&truth) && (listed == (class(&truth) == "adm"));
            let key = format!("fired_in_ways_out={} reh={} truth={} agree={agree} unchanged={unchanged}", fired_in_ways_out > 0, class(&reh), class(&truth));
            first_of.entry(key.clone()).or_insert(n);
            *tally.entry(key).or_insert(0) += 1;
            n += stride;
        }
        println!("VW k14d pre={pre} fill={fill} g={grow} reads_in_ways_out={total} stride={stride}");
        for (key, count) in &tally {
            println!("VW k14d pre={pre} fill={fill} g={grow}   {count:5} × {key} (first n={})", first_of[key]);
        }
    }
}

/// K1-3：不动 (实例代号, txg) 的状态变化——冻结（落盘途中写失败，现行那一版不换）、卸载再挂（新会话）、被拒之后别的改动发成/推环/抬 F（txg 动）、
/// 被拒之后原样再发一次（没发成、不动）。每一种：对旧的那一份报要出路交什么，对照同一刻真推环、再发同一个写的结局。
#[test]
#[ignore = "正推腿 K1-3，手动跑"]
fn vw_k13_state_changes_with_and_without_moving_instance_and_txg() {
    for (pre, fill, grow) in atk_cells_from_env() {
        let (snap, units) = atk_k1_cell(pre, fill, grow);
        let fresh = || {
            let plan = VwPlan::new();
            let mut pool = snap.pool_vw(&plan);
            let write = atk_write_of(&mut pool, units);
            let outcome = vw_publish(&mut pool, &write);
            (plan, pool, write, outcome)
        };
        let describe = |label: &str, pool: &PoolUnderTest<VwDev>, stuck: &StuckBehindTheWideProtection| {
            match pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck) {
                Ok(w) => format!("VW k13 pre={pre} fill={fill} g={grow} {label}: ways_out=Ok({} ways={})", atk_rehearsal_head_from(&w), w.ways_out.len()),
                Err(e) => format!("VW k13 pre={pre} fill={fill} g={grow} {label}: ways_out=Err({})", format!("{e:?}").chars().take(160).collect::<String>()),
            }
        };
        // 0 基线
        {
            let (_plan, mut pool, write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let line = describe("baseline", &pool, stuck);
            let pushed = push_the_root_ring_of(&mut pool);
            let truth = match pushed { Err(e) => format!("pushref {:?}", e).chars().take(60).collect::<String>(), Ok(_) => if vw_publish(&mut pool, &write).is_ok() { "adm".into() } else { "still".into() } };
            println!("{line} truth={truth}");
        }
        // 1 报之后原样再发一次（没发成、盘不动）
        {
            let (_plan, mut pool, write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let again = vw_publish(&mut pool, &write);
            println!("{} second_report={}", describe("after_same_write_again", &pool, stuck), atk_kind(&again));
        }
        // 2 报之后一个失败的小改动（盘表不对，一个盘都没碰）
        {
            let (_plan, mut pool, _write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let mut one_device: Vec<_> = pool.devices.drain(..1).collect();
            let small = AtkFirstFile { content: &[1u8; 16], write_time_seconds: 1 };
            let mut session = pool.session.take().unwrap();
            let refused = session.publish_user_change(&mut one_device, AtkUserChange::Overwrite(small));
            pool.session = Some(session);
            pool.devices.splice(0..0, one_device);
            println!("{} small_change={}", describe("after_a_change_refused_before_any_write", &pool, stuck), format!("{:?}", refused.err().map(|e| format!("{e:?}").chars().take(40).collect::<String>())));
        }
        // 3 冻结：报之后一个小覆盖写落盘途中失败（第 k 次写报错），现行那一版不换
        for k in [0i64, 3, 9] {
            let (plan, mut pool, write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let txg_before = pool.session().current.root().checkpoint_txg.0;
            plan.fail_write_after.set(k);
            let small = pool.overwrite(OVERWRITE_BYTES);
            plan.fail_write_after.set(-1);
            let txg_after = pool.session().current.root().checkpoint_txg.0;
            let frozen = pool.session().allocator.frozen_publish().is_some();
            let line = describe(&format!("after_a_small_write_failing_at_write_{k}(small={},txg {txg_before}->{txg_after},frozen={frozen})", if small.is_ok() { "ok" } else { "err" }), &pool, stuck);
            let pushed = push_the_root_ring_of(&mut pool);
            let truth = match pushed { Err(e) => format!("pushref {:?}", e).chars().take(60).collect::<String>(), Ok(_) => if vw_publish(&mut pool, &write).is_ok() { "adm".into() } else { "still".into() } };
            println!("{line} truth={truth}");
        }
        // 4 报之后别的改动发成（txg 动）
        {
            let (_plan, mut pool, _write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let small = pool.overwrite(OVERWRITE_BYTES);
            println!("{} small_write={}", describe("after_a_small_write_published", &pool, stuck), if small.is_ok() { "ok" } else { "err" });
        }
        // 5 报之后推环
        {
            let (_plan, mut pool, _write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let pushed = push_the_root_ring_of(&mut pool);
            println!("{} pushed={}", describe("after_the_push", &pool, stuck), pushed.is_ok());
        }
        // 6 卸载再挂：新会话
        {
            let (_plan, pool, _write, outcome) = fresh();
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let image = pool.image();
            let contents = pool.content_of_each_root.clone();
            let t = pool.write_time_seconds;
            let old_instance = pool.session().instance.0;
            let mut again = pool;
            let step = {
                let mut session = again.session.take().unwrap();
                let parameters = again.parameters.clone();
                let outcome = singlefs_core::mount::unmount(&parameters, &mut again.devices, &mut session.allocator, &mut session.current, ShadowLedger::On);
                outcome.is_ok()
            };
            let image_after = again.image();
            let remounted = atk_mounted_pool_from(&parameters_of_the_384_slot_small_devices(), device_bytes_of_the_384_slot_small_devices(), &image_after, &contents, t);
            match remounted {
                Ok((new_pool, _)) => println!("{} unmount_ok={step} instance {old_instance}->{}", describe_on_other(&new_pool, stuck), new_pool.session().instance.0),
                Err(e) => println!("VW k13 pre={pre} fill={fill} g={grow} remount_failed unmount_ok={step} {e:?}"),
            }
            let _ = image;
        }
    }
}

fn describe_on_other<Device: DeviceOverASparseImage>(pool: &PoolUnderTest<Device>, stuck: &StuckBehindTheWideProtection) -> String {
    match pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck) {
        Ok(w) => format!("VW k13 after_unmount_and_remount: ways_out=Ok({} ways={})", atk_rehearsal_head_from(&w), w.ways_out.len()),
        Err(e) => format!("VW k13 after_unmount_and_remount: ways_out=Err({})", format!("{e:?}").chars().take(200).collect::<String>()),
    }
}

/// K1-4：交进来的盘表不是挂载那一份（空、只一块、对调、落后的旧盘面）：要出路交什么，对照同一张盘表上真推环的结局；会话与分配器不动。
#[test]
#[ignore = "正推腿 K1-4，手动跑"]
fn vw_k14_wrong_device_tables_are_judged_like_the_push_entry_judges_them() {
    for (pre, fill, grow) in atk_cells_from_env() {
        let (snap, units) = atk_k1_cell(pre, fill, grow);
        for variant in ["empty", "first_only", "reversed", "older_image"] {
            let plan = VwPlan::new();
            let mut pool = snap.pool_vw(&plan);
            let write = atk_write_of(&mut pool, units);
            let outcome = vw_publish(&mut pool, &write);
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let allocator_before = format!("{:?}", pool.session().allocator);
            let mut table: Vec<(DeviceIdentity, VwDev)> = match variant {
                "empty" => Vec::new(),
                "first_only" => pool.devices.drain(..1).collect(),
                "reversed" => { let mut all: Vec<_> = pool.devices.drain(..).collect(); all.reverse(); all }
                _ => { let plan2 = VwPlan::new(); snap.pool_vw(&plan2).devices }
            };
            let ways = pool.session().ways_out_of_the_wide_protection(&table, stuck);
            let head = match &ways {
                Ok(w) => format!("{}:{}", atk_rehearsal_head_from(w), w.ways_out.len()),
                Err(e) => format!("Err({:?})", e).chars().take(40).collect::<String>(),
            };
            let mut session = pool.session.take().unwrap();
            let real = session.push_the_root_ring_to_the_ring_slot_of_r_dropping_rollback_candidates_below_four_states(&mut table);
            let real_head = match &real { Ok(_) => "pushed".to_string(), Err(e) => format!("{e:?}").chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect::<String>() };
            // 真推有没有动盘表上的盘（older_image 那一形真推会写到旧盘面上）：只报结局，不在意盘面
            let _ = allocator_before;
            println!("VW k14e pre={pre} fill={fill} g={grow} table={variant}: ways_out={head} real_push={real_head}");
        }
    }
}

/// K1-4：要出路那一步里从第 n 次读起所有读都报错（两块盘都是）：不 panic、会话与盘不动，交什么；对照同一故障上真推环的结局。
#[test]
#[ignore = "正推腿 K1-4，手动跑"]
fn vw_k14_every_read_fails_from_the_nth_read_of_the_ways_out() {
    let stride: u64 = std::env::var("VW_STRIDE").ok().and_then(|v| v.parse().ok()).unwrap_or(23);
    for (pre, fill, grow) in atk_cells_from_env() {
        let (snap, units) = atk_k1_cell(pre, fill, grow);
        let plan0 = VwPlan::new();
        let mut pool0 = snap.pool_vw(&plan0);
        let write0 = atk_write_of(&mut pool0, units);
        let outcome0 = vw_publish(&mut pool0, &write0);
        let stuck0 = atk_stuck_of(&outcome0).expect("stuck");
        plan0.reset();
        let _ = pool0.session().ways_out_of_the_wide_protection(&pool0.devices, stuck0).expect("要出路");
        let total = plan0.reads.get();
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut n = 1;
        while n <= total {
            let plan = VwPlan::new();
            let mut pool = snap.pool_vw(&plan);
            let write = atk_write_of(&mut pool, units);
            let outcome = vw_publish(&mut pool, &write);
            let stuck = atk_stuck_of(&outcome).expect("stuck");
            let allocator_before = format!("{:?}", pool.session().allocator);
            let image_before = pool.image();
            plan.reset();
            plan.fail_all_from.set(Some(n));
            let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
            let reh = atk_rehearsal_head_from(&w);
            let unchanged = allocator_before == format!("{:?}", pool.session().allocator) && image_before == pool.image();
            let pushed = push_the_root_ring_of(&mut pool);
            let truth = match &pushed { Err(e) => format!("pushref[{}]", format!("{e:?}").chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect::<String>()), Ok(_) => "pushed".to_string() };
            *tally.entry(format!("ways_out={reh} ({} ways) real_push={truth} unchanged={unchanged}", w.ways_out.len())).or_insert(0) += 1;
            n += stride;
        }
        println!("VW k14f pre={pre} fill={fill} g={grow} reads_in_ways_out={total} stride={stride}");
        for (k, c) in &tally { println!("VW k14f   {c:4} × {k}"); }
    }
}

#[test]
#[ignore = "正推腿 K1-4 调试，手动跑"]
fn vw_k14_debug_one_failing_from() {
    let from: u64 = std::env::var("VW_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(100);
    let (pre, fill, grow) = atk_cells_from_env()[0];
    let (snap, units) = atk_k1_cell(pre, fill, grow);
    let plan = VwPlan::new();
    let mut pool = snap.pool_vw(&plan);
    let write = atk_write_of(&mut pool, units);
    let outcome = vw_publish(&mut pool, &write);
    let stuck = atk_stuck_of(&outcome).expect("stuck");
    plan.reset();
    plan.fail_all_from.set(Some(from));
    let w = pool.session().ways_out_of_the_wide_protection(&pool.devices, stuck).expect("要出路");
    println!("VW dbg from={from} reads_attempted={} fired={} head={} ways={} debug={}", plan.reads.get(), plan.fired.get(), atk_rehearsal_head_from(&w), w.ways_out.len(), format!("{:?}", w.pushing_the_root_ring).chars().take(300).collect::<String>());
}

/// K1-4（对准读序号）：从第 n 次读起所有读都报错。要出路交什么；另一份同样的池上，同一个 n 起故障的真推环（推环入口自己的第 n 次读起）结局。
#[test]
#[ignore = "正推腿 K1-4，手动跑"]
fn vw_k14_every_read_fails_from_the_nth_read_ways_out_against_the_real_push_from_its_own_nth_read() {
    let stride: u64 = std::env::var("VW_STRIDE").ok().and_then(|v| v.parse().ok()).unwrap_or(23);
    for (pre, fill, grow) in atk_cells_from_env() {
        let (snap, units) = atk_k1_cell(pre, fill, grow);
        let mk = || {
            let plan = VwPlan::new();
            let mut pool = snap.pool_vw(&plan);
            let write = atk_write_of(&mut pool, units);
            let outcome = vw_publish(&mut pool, &write);
            (plan, pool, write, outcome)
        };
        // 真推环总共读几次
        let (plan_t, mut pool_t, _w, _o) = mk();
        plan_t.reset();
        let _ = push_the_root_ring_of(&mut pool_t);
        let real_reads = plan_t.reads.get();
        let mut tally: BTreeMap<String, usize> = BTreeMap::new();
        let mut first: BTreeMap<String, u64> = BTreeMap::new();
        let mut n = 1;
        while n <= real_reads {
            // 要出路
            let (plan_a, pool_a, _wa, outcome_a) = mk();
            let stuck_a = atk_stuck_of(&outcome_a).expect("stuck");
            plan_a.reset();
            plan_a.fail_all_from.set(Some(n));
            let w = pool_a.session().ways_out_of_the_wide_protection(&pool_a.devices, stuck_a).expect("要出路");
            let reh = atk_rehearsal_head_from(&w);
            // 真推环：从它自己的第 n 次读起
            let (plan_b, mut pool_b, write_b, _ob) = mk();
            plan_b.reset();
            plan_b.fail_all_from.set(Some(n));
            let pushed = push_the_root_ring_of(&mut pool_b);
            plan_b.fail_all_from.set(None);
            let truth = match &pushed {
                Err(e) => format!("pushref[{}]", format!("{e:?}").chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect::<String>()),
                Ok(_) => { if vw_publish(&mut pool_b, &write_b).is_ok() { "adm".to_string() } else { "still".to_string() } }
            };
            let class = |t: &str| if t.starts_with("adm") { "adm" } else if t.starts_with("still") { "still" } else { "pushref" }.to_string();
            let key = format!("ways_out={} real_push_from_its_nth_read={truth} listed_push={}", reh, w.ways_out.len() == 2);
            first.entry(key.clone()).or_insert(n);
            *tally.entry(key).or_insert(0) += 1;
            n += stride;
        }
        println!("VW k14g pre={pre} fill={fill} g={grow} real_push_reads={real_reads} stride={stride}");
        for (k, c) in &tally { println!("VW k14g   {c:4} × {k} (first n={})", first[k]); }
    }
}
