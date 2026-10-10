
// ---------- K1-2：预演里读盘出错（一次性的读错、盘上永久读不出的几段） ----------

/// 几块盘共用的读故障计划：第 `fail_at` 次读（整池数，从 1 数）报错一次；`active` 时读到 `ranges` 里任何一段都报错（写也不撤）。
struct AtkReadPlan {
    reads: Cell<u64>,
    fail_at: Cell<Option<u64>>,
    ranges: std::cell::RefCell<Vec<(DeviceIdentity, u64, u64)>>,
    active: Cell<bool>,
    fired: Cell<u64>,
    /// 第 `pin_at` 次读（从 1 数）读的那一段从这一次起一直读不出（写也不撤）。
    pin_at: Cell<Option<u64>>,
}

impl AtkReadPlan {
    fn new(fail_at: Option<u64>) -> Rc<Self> {
        Rc::new(AtkReadPlan {
            reads: Cell::new(0),
            fail_at: Cell::new(fail_at),
            ranges: std::cell::RefCell::new(Vec::new()),
            active: Cell::new(false),
            fired: Cell::new(0),
            pin_at: Cell::new(None),
        })
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
            self.plan.active.set(true);
        }
        let in_range = self.plan.active.get()
            && self.plan.ranges.borrow().iter().any(|(device, start, end)| {
                *device == self.identity && offset.0 < *end && offset.0 + length > *start
            });
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
            devices: plain_devices_on(&self.image)
                .into_iter()
                .map(|(identity, inner)| (identity, AtkFaultDev { identity, inner, plan: plan.clone() }))
                .collect(),
            session: self.session.as_ref().map(copy_of_the_session),
            write_time_seconds: self.write_time_seconds,
            content_of_the_current_version: self.content_now.clone(),
            content_of_each_root: self.contents.clone(),
        }
    }
}

/// 预演结局的短名，带拒绝的成员名（PushRefused 里是哪一种、ChangeStillRefused 里的拒绝是哪一种）。
fn atk_rehearsal_head(stuck: &StuckBehindTheWideProtection) -> String {
    let head = |text: String| text.chars().take_while(|c| *c != '(' && *c != ' ' && *c != '{').collect::<String>();
    let atk_w = atk_ways(stuck);
    match &atk_w.pushing_the_root_ring {
        RootRingPushRehearsal::ChangeAdmittedAfterThePush { .. } => "adm".to_string(),
        RootRingPushRehearsal::ChangeStillRefusedAfterThePush { refusal, .. } => {
            let inner = match refusal.as_ref() {
                UserChangeRefused::Publish { cause, .. } => format!("Publish:{}", head(format!("{cause:?}"))),
                UserChangeRefused::FloorRaiseFailedWhilePushingForSpace(failed) => {
                    format!("FloorRaiseFailed:{}", head(format!("{:?}", failed.cause)))
                }
                other => head(format!("{other:?}")),
            };
            format!("still[{inner}]")
        }
        RootRingPushRehearsal::PushRefused(refusal) => {
            let inner = match refusal.as_ref() {
                RootRingPushRefused::DevicesHandedIn(error) => format!("DevicesHandedIn:{}", head(format!("{error:?}"))),
                RootRingPushRefused::RefusedByTheRehearsalBeforeAnyWrite { cause, .. } => {
                    format!("RefusedByTheRehearsal:{}", head(format!("{cause:?}")))
                }
                RootRingPushRefused::PublishFailedAfterTheRehearsal(failed) => {
                    format!("PublishFailedAfterTheRehearsal:{}", head(format!("{:?}", failed.cause)))
                }
                other => head(format!("{other:?}")),
            };
            format!("pushref[{inner}]")
        }
    }
}

/// 照出路二走（推环、再发同一个改动一次），任何盘。
fn atk_follow_on<Device: DeviceOverASparseImage>(pool: &mut PoolUnderTest<Device>, write: &AtkWrite) -> String {
    match push_the_root_ring_of(pool) {
        Ok(pushed) => format!("pushed({}) retry={}", pushed.versions_published.len(), atk_kind(&atk_apply(pool, write))),
        Err(refusal) => format!("push_refused({})", format!("{refusal:?}").chars().take(80).collect::<String>()),
    }
}
