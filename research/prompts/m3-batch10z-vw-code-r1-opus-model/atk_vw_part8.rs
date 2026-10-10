
// ---------- K1-4：要出路那一次读盘出错（一次性的读错、从此读不出的一段） ----------

/// 几块盘共用的读故障计划：第 `fail_at` 次读（整池数，从 1 数）报错一次；第 `pin_at` 次读读的那一段从这一次起一直读不出（写也不撤）。
struct AtkReadPlan {
    reads: Cell<u64>,
    fail_at: Cell<Option<u64>>,
    pin_at: Cell<Option<u64>>,
    ranges: std::cell::RefCell<Vec<(DeviceIdentity, u64, u64)>>,
    fired: Cell<u64>,
}

impl AtkReadPlan {
    fn new(fail_at: Option<u64>, pin_at: Option<u64>) -> Rc<Self> {
        Rc::new(AtkReadPlan { reads: Cell::new(0), fail_at: Cell::new(fail_at), pin_at: Cell::new(pin_at), ranges: std::cell::RefCell::new(Vec::new()), fired: Cell::new(0) })
    }
}

struct AtkFaultDev {
    identity: DeviceIdentity,
    inner: SparseBlockDevice,
    plan: Rc<AtkReadPlan>,
}

impl BlockDevice for AtkFaultDev {
    fn read_at(&self, offset: DeviceOffsetInBytes, buffer: &mut [u8]) -> Result<(), BlockDeviceError> {
        let read = self.plan.reads.get() + 1;
        self.plan.reads.set(read);
        let length = u64::try_from(buffer.len()).expect("读长");
        if self.plan.pin_at.get() == Some(read) {
            self.plan.ranges.borrow_mut().push((self.identity, offset.0, offset.0 + length));
        }
        let in_range = self.plan.ranges.borrow().iter().any(|(device, start, end)| *device == self.identity && offset.0 < *end && offset.0 + length > *start);
        if self.plan.fail_at.get() == Some(read) || in_range {
            self.plan.fired.set(self.plan.fired.get() + 1);
            return Err(BlockDeviceError::InputOutput(std::io::Error::other("攻方注入的读错")));
        }
        self.inner.read_at(offset, buffer)
    }
    fn write_at(&mut self, offset: DeviceOffsetInBytes, bytes: &[u8], durability: WriteDurability) -> Result<(), BlockDeviceError> {
        self.inner.write_at(offset, bytes, durability)
    }
    fn write_zeroes_at(&mut self, offset: DeviceOffsetInBytes, length: u64) -> Result<(), BlockDeviceError> {
        self.inner.write_zeroes_at(offset, length)
    }
    fn barrier(&mut self) -> Result<(), BlockDeviceError> {
        self.inner.barrier()
    }
    fn probe_physical_block_size(&self) -> PhysicalBlockSizeInBytes {
        self.inner.probe_physical_block_size()
    }
    fn size_in_bytes(&self) -> u64 {
        self.inner.size_in_bytes()
    }
}

impl DeviceOverASparseImage for AtkFaultDev {
    fn sparse_image(&self) -> &SparseDevice {
        &self.inner.image
    }
}

impl AtkSnap {
    fn pool_with(&self, plan: &Rc<AtkReadPlan>) -> PoolUnderTest<AtkFaultDev> {
        PoolUnderTest {
            parameters: self.parameters.clone(),
            device_bytes: self.device_bytes,
            devices: plain_devices_on(&self.image).into_iter().map(|(identity, inner)| (identity, AtkFaultDev { identity, inner, plan: plan.clone() })).collect(),
            session: self.session.as_ref().map(copy_of_the_session),
            write_time_seconds: self.write_time_seconds,
            content_of_the_current_version: self.content_now.clone(),
            content_of_each_root: self.contents.clone(),
        }
    }
}

/// K1-4：K1 几格报卡住之后要出路，要出路那一次第 n 次读报错一次（n 扫过这一次的每一次读）或第 n 次读读的那一段从此读不出：
/// 要出路的结局、会话与盘动没动；从此读不出的那一种，照出路二在同一组故障上真走一遍比列不列推环。
#[test]
#[ignore = "攻方原型，手动跑"]
fn atk_k1_read_faults_while_asking_for_the_ways_out() {
    let unit_bytes = singlefs_core::unit::data_unit_payload_capacity();
    let step: u64 = atk_env_or("ATK_STEP_N", 1);
    let cells: Vec<(u64, usize, usize)> = std::env::var("ATK_CELLS")
        .unwrap_or_else(|_| "4:16:13,4:16:20,2:16:17,8:11:40".to_string())
        .split(',')
        .map(|cell| {
            let parts: Vec<usize> = cell.split(':').map(|text| text.parse().expect("数")).collect();
            (u64::try_from(parts[0]).expect("数"), parts[1], parts[2])
        })
        .collect();
    for (pre, fill, grow) in cells {
        let (_base, crashed, contents, write_time_seconds) = atk_k1_record_on(384, pre, fill, 5).expect("录得下");
        let (mut pool, _) = atk_mounted_pool_from(&parameters_of_the_384_slot_small_devices(), device_bytes_of_the_384_slot_small_devices(), &crashed, &contents, write_time_seconds).expect("挂");
        let size_now = pool.content_of_the_current_version.len().div_ceil(unit_bytes);
        let change = atk_seq_of(&mut pool, size_now + grow);
        let outcome = atk_apply(&mut pool, &change);
        let Some(stuck) = atk_stuck_of(&outcome) else {
            println!("ATK k1askrd pre={pre} fill={fill} g={grow} first={}", atk_kind(&outcome));
            continue;
        };
        let after = AtkSnap::of(&pool);
        let dry_plan = AtkReadPlan::new(None, None);
        let dry = after.pool_with(&dry_plan);
        let base = atk_ask_direct(&dry, stuck);
        let total_reads = dry_plan.reads.get();
        let base_debug = format!("{:?}", base.result);
        let mut transient: BTreeMap<String, (usize, u64)> = BTreeMap::new();
        let mut pinned: BTreeMap<String, (usize, u64)> = BTreeMap::new();
        for n in (1..=total_reads).step_by(usize::try_from(step).expect("步长")) {
            let plan = AtkReadPlan::new(Some(n), None);
            let faulted = after.pool_with(&plan);
            let ask = atk_ask_direct(&faulted, stuck);
            let key = format!("{} same_as_without_fault={} untouched={}", ask.head(), format!("{:?}", ask.result) == base_debug, ask.untouched);
            transient.entry(key).or_insert((0, n)).0 += 1;
            let plan = AtkReadPlan::new(None, Some(n));
            let mut faulted = after.pool_with(&plan);
            let ask = atk_ask_direct(&faulted, stuck);
            let head = ask.head();
            let (line, ok) = atk_compare(&ask, &mut faulted, &change);
            let follow: String = line.split(" push=").nth(1).unwrap_or("").split(" agree").next().unwrap_or("").chars().take(70).collect();
            let key = format!("{head} untouched={} | follow: {follow} ok={ok:?}", ask.untouched);
            pinned.entry(key).or_insert((0, n)).0 += 1;
        }
        println!("ATK k1askrd pre={pre} fill={fill} g={grow} base={} reads_of_the_ask={total_reads} step={step}", base.head());
        for (key, (count, first)) in &transient {
            println!("ATK k1askrd pre={pre} fill={fill} g={grow} transient {count:5} × {key} (first n={first})");
        }
        for (key, (count, first)) in &pinned {
            println!("ATK k1askrd pre={pre} fill={fill} g={grow} pinned    {count:5} × {key} (first n={first})");
        }
    }
}
