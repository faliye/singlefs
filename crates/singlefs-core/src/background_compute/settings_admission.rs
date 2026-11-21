//! 后台计算设置与它的校验：照卡清单校验出参与的卡与上限（[`AdmittedGpuSettings`]），校验过的整组设置
//! （[`AdmittedBackgroundComputeSettings`]）与 GC 的 CPU 补偿阈值的读数。

use super::*;

/// 一张参与的卡：它的标识、显存总量（卡清单答的），与给它的显存上限（`None` 是不设上限，第 3 条）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParticipatingGpuDevice {
    pub device: GpuDeviceIdentifier,
    pub total_memory: GpuMemoryInBytes,
    pub memory_limit: Option<GpuMemoryInBytes>,
}

/// 校验过、选 `gpu` 的一组设置交给实现的样子（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]）：参与的卡按上限对应的次序
/// （没点名是卡清单的次序，点了名是点名的次序）、各自的显存上限，与等 GPU 的上限。只由 [`admit_gpu_settings_against_the_inventory`] 造出：
/// 至少一张卡、同一张卡只出现一次、每张都在校验时的卡清单里、上限不超过那张卡的显存总量、等的上限不是 0。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedGpuSettings {
    participating_devices: Vec<ParticipatingGpuDevice>,
    wait_limit: Duration,
}

impl AdmittedGpuSettings {
    #[must_use]
    pub fn participating_devices(&self) -> &[ParticipatingGpuDevice] {
        &self.participating_devices
    }

    #[must_use]
    pub fn wait_limit(&self) -> Duration {
        self.wait_limit
    }
}

/// 等 GPU 交回一批结果的上限的默认：10 秒。默认值是推的，待量：E185（GC一批搬迁的挂钟拆分）
/// 读数里，一批 CPU 18–338 ms（第 14 行）、GPU 往返让前台多等 5.50–14.76 ms（第 11 行），取比最慢的一批大一个半数量级的数：
/// 卡死的驱动等得出来，慢而活着的卡不被当成出错。
pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);

/// 一组后台计算的挂载参数（第 1–3 条，加等 GPU 结果的上限）。每次挂载各给各的；挂着的时候整组换（[`BackgroundComputeOfThisMount::request_switch_from_the_next_batch`]）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BackgroundComputeSettings {
    pub background_compute: BackgroundComputeBackend,
    /// `None` 是不给：用本机全部显卡（第 2 条）。
    pub background_gpu_devices: Option<Vec<GpuDeviceIdentifier>>,
    /// `None` 是不给：不设上限（第 3 条）。给了就按次序与参与的卡一张一个，不补齐。
    pub background_gpu_memory_limit: Option<Vec<GpuMemoryInBytes>>,
    /// 一批交给 GPU 之后至多等它多久（过了这个上限还没交回，照第 4 条「挂着的时候 GPU 出错」办，
    /// 这一批放弃、那个后台任务停下并报错）。只在选 `gpu` 时起作用；选 `cpu` 时照收、不用。不许是 0
    /// （[`BackgroundComputeSettingsRefused::GpuWaitLimitOfZero`]）。默认 [`BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT`]。
    pub background_gpu_wait_limit: Duration,
}

impl BackgroundComputeSettings {
    /// 产品默认：`cpu`，卡与上限都不给（第 1 条「默认 `cpu`」），等 GPU 的上限取默认。现有的可写挂载入口都等价于挂载时给这一组。
    pub const PRODUCT: BackgroundComputeSettings = BackgroundComputeSettings {
        background_compute: BackgroundComputeBackend::Cpu,
        background_gpu_devices: None,
        background_gpu_memory_limit: None,
        background_gpu_wait_limit: BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT,
    };
}

/// 挂载参数 `background_gc_cpu_compensation_threshold`（有默认值、可调）：GC 因一批计算出错停着时，池子里有一块盘的已分配字节占它的容量的百分比严格大于它
/// （第 4 条：任一块盘超过就补跑），GC 照常起跑、这一批在 CPU 上补跑。
/// 与 [`BackgroundComputeSettings`] 同属一组挂载参数：挂载与挂着切换时一起给、一起校验、从下一批起一起生效（第 1、4 条）。
/// 取值 0 到 100（含两端，整百分点）：100 是停着就不补跑，0 是池子里有一个字节已分配就补跑；超过 100 挂载或切换那一刻拒
/// （[`BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity`]），选 `cpu` 时也判。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundGcCpuCompensationThreshold {
    pub allocated_percent_of_the_capacity: u32,
}

/// [`BackgroundGcCpuCompensationThreshold`] 的默认：85%。
pub const BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT:
    BackgroundGcCpuCompensationThreshold = BackgroundGcCpuCompensationThreshold {
    allocated_percent_of_the_capacity: 85,
};

/// 百分比的满值：已分配不超过总容量，阈值超过它就永远越不过。
const WHOLE_CAPACITY_IN_PERCENT: u32 = 100;

/// 一块盘此刻的已分配字节与容量（补跑那一判读的两个量）：分配器给这块盘增量维护的两个计数——占着的槽数
/// （`allocator::DeviceFreeMap::allocated_slots`，含已释放、还在 defer 窗口里的，同空间准入式子里的「已分配」）与单元区槽数
/// （`allocator::DeviceFreeMap::unit_area_slots`，同式子里的「容量」）——各换成字节。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AllocatedBytesAndCapacityOfADevice {
    pub device: DeviceIdentity,
    pub allocated_bytes: u64,
    pub capacity_bytes: u64,
}

impl AllocatedBytesAndCapacityOfADevice {
    /// 池子里已分配占容量最满的那一块盘（任一块盘超过就补跑：最满的那一块超过阈值，就是有一块超过；
    /// 每块都不超过，最满的那一块也不超过）。逐盘比 `已分配 ÷ 容量`（交叉相乘在 u128 里，不取整），一样满的取分配器里排在前面的那一块；
    /// 容量是 0 的盘不参加比（它装不下一个槽，占比无从谈起），全是 0 的取头一块。不遍历槽，代价随盘数、不随盘容量
    /// （`.claude/rules/fs-design.md`「记账是事务的副产品」：起不起跑是运行时决策路径）。
    ///
    /// # Panics
    /// 分配器里一块盘都没有：可写挂载的池至少有一块盘。
    #[must_use]
    pub fn of_the_fullest_device_of_the_allocator(allocator: &PoolAllocator) -> Self {
        let of_the_device = |device_map: &DeviceFreeMap| AllocatedBytesAndCapacityOfADevice {
            device: device_map.device,
            allocated_bytes: BytesOnOneDevice::of_slots(device_map.allocated_slots()).0,
            capacity_bytes: BytesOnOneDevice::of_slots(device_map.unit_area_slots()).0,
        };
        let first_device = of_the_device(
            allocator
                .devices
                .first()
                .expect("可写挂载的池至少有一块盘：分配器照盘表一块盘一张空闲图"),
        );
        allocator
            .devices
            .iter()
            .map(of_the_device)
            .filter(|device| device.capacity_bytes != 0)
            .fold(
                None,
                |fullest_so_far: Option<Self>, device| match fullest_so_far {
                    Some(fullest) if !device.is_fuller_than(&fullest) => Some(fullest),
                    Some(_) | None => Some(device),
                },
            )
            .unwrap_or(first_device)
    }

    /// 这一块盘的已分配占容量严格大于 `other` 的：`已分配 × 对方容量 > 对方已分配 × 容量`。两块盘的容量都不是 0 时才有意义。
    fn is_fuller_than(&self, other: &Self) -> bool {
        u128::from(self.allocated_bytes) * u128::from(other.capacity_bytes)
            > u128::from(other.allocated_bytes) * u128::from(self.capacity_bytes)
    }
}

/// 补跑那一判的读数：池子此刻最满的那一块盘（哪块盘、它的已分配字节与容量），与那一刻生效的阈值。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GcCpuCompensationReading {
    pub fullest_device: AllocatedBytesAndCapacityOfADevice,
    pub threshold: BackgroundGcCpuCompensationThreshold,
}

impl GcCpuCompensationReading {
    /// 最满的那一块盘的已分配占它的容量严格大于阈值：`已分配 × 100 > 容量 × 阈值百分比`，乘在 u128 里，不溢出、不取整。
    #[must_use]
    pub fn is_the_allocated_share_above_the_threshold(&self) -> bool {
        u128::from(self.fullest_device.allocated_bytes) * u128::from(WHOLE_CAPACITY_IN_PERCENT)
            > u128::from(self.fullest_device.capacity_bytes)
                * u128::from(self.threshold.allocated_percent_of_the_capacity)
    }
}

/// 一组设置在挂载或切换那一刻被拒（第 4 条：拒，不降级到 cpu）。成员按调用方要改哪一样分：第 4 条点名的四种配错
/// （GPU 不在、点名的卡不存在、上限个数与卡数不等、上限超过显存总量），条款没写、第一版不支持的两种点法（空卡列表、同一张卡点两次），
/// 等 GPU 结果的上限给了 0；GC 在 CPU 上补跑的阈值超过 100%。切换点到的卡开不开得成不在这里判：切换照收，开不成照
/// [`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`] 停 GC、报警。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BackgroundComputeSettingsRefused {
    /// 选了 `gpu` 而 GPU 不在：没有卡清单，或没点名卡而清单里一张卡都没有。改选 `cpu`，或装上 GPU。
    GpuNotPresent,
    /// `background_gpu_devices` 点名的这张卡不在卡清单里（点名的卡不存在）。
    NamedGpuDeviceNotInTheInventory { device: GpuDeviceIdentifier },
    /// `background_gpu_memory_limit` 给了而个数与参与的卡数不等（给了卡列表对它的列表、没给对本机全部显卡）。
    MemoryLimitCountDifferentFromTheParticipatingDevices {
        memory_limits_given: usize,
        participating_devices: usize,
    },
    /// 给这张卡的上限超过它的显存总量。
    MemoryLimitAboveTheTotalMemoryOfTheDevice {
        device: GpuDeviceIdentifier,
        memory_limit: GpuMemoryInBytes,
        total_memory: GpuMemoryInBytes,
    },
    /// `background_gpu_devices` 给了而一张卡都没点：第 2 条只写了「不给就用本机全部显卡」，给了空列表算什么条款没写，第一版不支持。
    EmptyGpuDeviceListWhoseMeaningIsUndecided,
    /// 同一张卡在 `background_gpu_devices` 里点了不止一次：第 3 条按次序一张卡一个上限，同一张卡两个上限怎么算条款没写，第一版不支持。
    GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided { device: GpuDeviceIdentifier },
    /// 选了 `gpu` 而 `background_gpu_wait_limit` 是 0：一批交出去就算超时，一批都算不成。改成大于 0 的上限。
    GpuWaitLimitOfZero,
    /// `background_gc_cpu_compensation_threshold` 超过 100%：已分配不会超过总容量，这个阈值永远越不过。改成 0 到 100 之间。
    GcCpuCompensationThresholdAboveTheWholeCapacity {
        threshold: BackgroundGcCpuCompensationThreshold,
    },
}

/// 校验过的一组设置，连同它校验时用的那一份 GPU 实现与 GC 在 CPU 上补跑的阈值：只由
/// [`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`] 造出，
/// 产品默认另有 [`Self::PRODUCT`]。选 `gpu` 的那一组恒带着实现与交给实现的那一组（没给实现那一刻就拒了），一批照它算。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmittedBackgroundComputeSettings {
    pub(super) settings: BackgroundComputeSettings,
    pub(super) gpu_compute: Option<BackgroundGpuComputeHandle>,
    /// 选 `gpu` 时恒有：照实现答的卡清单校验出的参与的卡、上限与等的上限；选 `cpu` 时恒无。
    gpu_settings: Option<AdmittedGpuSettings>,
    /// 恒不超过 100%（校验那一刻判过）。
    pub(super) gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
}

impl AdmittedBackgroundComputeSettings {
    /// [`BackgroundComputeSettings::PRODUCT`]：选 `cpu` 照收、不校验，不带实现；补跑的阈值取默认。
    pub const PRODUCT: AdmittedBackgroundComputeSettings = AdmittedBackgroundComputeSettings {
        settings: BackgroundComputeSettings::PRODUCT,
        gpu_compute: None,
        gpu_settings: None,
        gc_cpu_compensation_threshold: BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    };

    #[must_use]
    pub fn settings(&self) -> &BackgroundComputeSettings {
        &self.settings
    }

    /// GC 停着时在 CPU 上补跑的阈值（挂载参数 `background_gc_cpu_compensation_threshold`）。
    #[must_use]
    pub fn gc_cpu_compensation_threshold(&self) -> BackgroundGcCpuCompensationThreshold {
        self.gc_cpu_compensation_threshold
    }

    /// 选 `gpu` 时交给实现的那一组（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]）；选 `cpu` 时 `None`。
    #[must_use]
    pub fn gpu_settings(&self) -> Option<&AdmittedGpuSettings> {
        self.gpu_settings.as_ref()
    }

    /// 把这一组交给它带着的实现：选 `gpu` 时交这一组（[`BackgroundGpuCompute::use_the_settings_from_the_next_batch`]），从下一批起照它跑
    /// （点到没开的卡时实现开完才换上）；选 `cpu` 而带着实现时告诉它一张卡都不参与（[`BackgroundGpuCompute::use_no_gpu_from_the_next_batch`]，
    /// 第 2 条「从 gpu 切到 cpu……关掉全部开着的卡」）；选 `cpu` 而没带实现时什么都不做。
    ///
    /// # Panics
    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
    pub(super) fn hand_the_settings_to_the_implementation(
        &self,
    ) -> Result<(), GpuImplementationPanickedWhileLent> {
        match (&self.gpu_settings, &self.gpu_compute) {
            (Some(gpu_settings), gpu_compute) => {
                gpu_compute
                    .as_ref()
                    .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
                    .lock()?
                    .use_the_settings_from_the_next_batch(gpu_settings);
            }
            (None, Some(gpu_compute)) => gpu_compute.lock()?.use_no_gpu_from_the_next_batch(),
            (None, None) => {}
        }
        Ok(())
    }

    /// 选 `gpu` 时问它带着的实现最近交给它的那一组此刻怎样了
    /// （[`BackgroundGpuCompute::put_the_gpus_opened_since_the_settings_were_handed_in_in_effect`]，开完的卡在这一刻换上）；
    /// 选 `cpu` 时 `None`（实现这时不算活）。
    ///
    /// # Panics
    /// 选 `gpu` 而没带实现：选 `gpu` 的那一组在校验那一刻就带上实现。
    pub(super) fn latest_gpu_settings_handed_in_asked_of_the_implementation(
        &self,
    ) -> Option<Result<LatestGpuSettingsHandedIn, GpuImplementationPanickedWhileLent>> {
        self.gpu_settings.as_ref()?;
        Some(
            self.gpu_compute
                .as_ref()
                .expect("选 gpu 的那一组在校验那一刻就带上实现：没给实现照 GPU 不在拒了")
                .lock()
                .map(|mut implementation| {
                    implementation.put_the_gpus_opened_since_the_settings_were_handed_in_in_effect()
                }),
        )
    }

    /// 这一组带着的 GPU 实现：选 `gpu` 时恒有；选 `cpu` 时是挂载或切换时递进来的那一份（以后切到 `gpu` 没另给实现时用它），没给是 `None`。
    #[must_use]
    pub fn gpu_compute(&self) -> Option<&BackgroundGpuComputeHandle> {
        self.gpu_compute.as_ref()
    }
}

/// 同 [`admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`]，GC 在 CPU 上补跑的阈值取默认
/// [`BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT`]（不给就是默认）。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
pub fn admit_background_compute_settings_with_the_gpu_compute(
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
) -> Result<AdmittedBackgroundComputeSettings, BackgroundComputeSettingsRefused> {
    admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
        settings,
        gpu_compute,
        BACKGROUND_GC_CPU_COMPENSATION_THRESHOLD_BY_DEFAULT,
    )
}

/// 挂载与挂着切换那一刻的校验（第 4 条）：先判 GC 在 CPU 上补跑的阈值不超过 100%（选 `cpu` 时也判：它与卡、上限不同，停着那一判不看后端），
/// 再判选 `gpu` 而没给实现就是 GPU 不在，拒；给了就照实现答的卡清单走 [`admit_background_compute_settings`]。
/// 选 `cpu` 时卡与上限照收、不校验，给了实现也带着。只读卡清单，不碰盘。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 的八种。
pub fn admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold(
    settings: BackgroundComputeSettings,
    gpu_compute: Option<BackgroundGpuComputeHandle>,
    gc_cpu_compensation_threshold: BackgroundGcCpuCompensationThreshold,
) -> Result<AdmittedBackgroundComputeSettings, BackgroundComputeSettingsRefused> {
    if gc_cpu_compensation_threshold.allocated_percent_of_the_capacity > WHOLE_CAPACITY_IN_PERCENT {
        return Err(
            BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                threshold: gc_cpu_compensation_threshold,
            },
        );
    }
    let gpu_settings = match settings.background_compute {
        BackgroundComputeBackend::Cpu => None,
        BackgroundComputeBackend::Gpu => {
            let Some(gpu_compute) = &gpu_compute else {
                return Err(BackgroundComputeSettingsRefused::GpuNotPresent);
            };
            // 不等实现那把锁：锁外算一批时它可能正被借着，切换不在会话锁里等它，被借着就照把手带的卡清单快照校验
            // （D26（后台整理与放置回收） 已定项 13 第 5 条）。
            let inventory_snapshot = GpuInventoryReadOnce {
                devices_with_total_memory: gpu_compute.inventory_without_waiting(),
            };
            Some(admit_gpu_settings_against_the_inventory(
                &settings,
                &inventory_snapshot,
            )?)
        }
    };
    Ok(AdmittedBackgroundComputeSettings {
        settings,
        gpu_compute,
        gpu_settings,
        gc_cpu_compensation_threshold,
    })
}

/// 第 4 条选 `gpu` 那一支的校验（`settings.background_compute` 不看，照选了 `gpu` 判，判法就是 [`admit_background_compute_settings`] 选 `gpu` 那一支），
/// 过了交回交给实现的那一组：参与的卡按上限对应的次序、各带上限，与等的上限。卡清单只读一次，判与取参与的卡读的是同一份。只读卡清单，不碰盘。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
pub fn admit_gpu_settings_against_the_inventory(
    settings: &BackgroundComputeSettings,
    gpu_inventory: &dyn GpuInventory,
) -> Result<AdmittedGpuSettings, BackgroundComputeSettingsRefused> {
    let inventory_read_once = GpuInventoryReadOnce {
        devices_with_total_memory: gpu_inventory.devices_with_total_memory(),
    };
    admit_background_compute_settings(
        BackgroundComputeSettings {
            background_compute: BackgroundComputeBackend::Gpu,
            background_gpu_devices: settings.background_gpu_devices.clone(),
            background_gpu_memory_limit: settings.background_gpu_memory_limit.clone(),
            background_gpu_wait_limit: settings.background_gpu_wait_limit,
        },
        Some(&inventory_read_once),
    )?;
    let participating_devices = participating_gpu_devices(
        settings.background_gpu_devices.as_deref(),
        &inventory_read_once.devices_with_total_memory,
    )
    .expect("上面的校验照同一份清单判过点名的卡：不空、不重、都在清单里");
    let memory_limits: Vec<Option<GpuMemoryInBytes>> = match &settings.background_gpu_memory_limit {
        None => vec![None; participating_devices.len()],
        Some(memory_limits) => memory_limits.iter().copied().map(Some).collect(),
    };
    assert_eq!(
        memory_limits.len(),
        participating_devices.len(),
        "上面的校验判过上限个数与参与的卡数相等"
    );
    Ok(AdmittedGpuSettings {
        participating_devices: participating_devices
            .into_iter()
            .zip(memory_limits)
            .map(
                |((device, total_memory), memory_limit)| ParticipatingGpuDevice {
                    device,
                    total_memory,
                    memory_limit,
                },
            )
            .collect(),
        wait_limit: settings.background_gpu_wait_limit,
    })
}

/// 读过一次的卡清单：判与取参与的卡都读这一份，不读第二次（实现答的清单两次之间可能变）。
struct GpuInventoryReadOnce {
    devices_with_total_memory: Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>,
}

impl GpuInventory for GpuInventoryReadOnce {
    fn devices_with_total_memory(&self) -> Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)> {
        self.devices_with_total_memory.clone()
    }
}

/// 第 4 条的校验：选 `cpu` 时卡、显存上限与等 GPU 的上限都照收、不校验（切到 `gpu` 那一刻再校验）；选 `gpu` 时，依次判 GPU 在不在、
/// 点名的卡在不在清单里、上限个数与参与的卡数等不等、每个上限超没超那张卡的显存总量、等 GPU 的上限是不是 0，有一样不对就拒。
/// 只读卡清单，不碰盘。只校验：交回的是交进来的那一组，不带实现、装不进挂载；挂载与切换经
/// [`admit_background_compute_settings_with_the_gpu_compute`]（卡清单由实现答），产品 GPU 包开包时用这一份。
///
/// # Errors
/// [`BackgroundComputeSettingsRefused`] 除阈值越界之外的七种。
pub fn admit_background_compute_settings(
    settings: BackgroundComputeSettings,
    gpu_inventory: Option<&dyn GpuInventory>,
) -> Result<BackgroundComputeSettings, BackgroundComputeSettingsRefused> {
    match settings.background_compute {
        BackgroundComputeBackend::Cpu => {
            return Ok(settings);
        }
        BackgroundComputeBackend::Gpu => {}
    }
    let Some(gpu_inventory) = gpu_inventory else {
        return Err(BackgroundComputeSettingsRefused::GpuNotPresent);
    };
    let devices_on_this_host = gpu_inventory.devices_with_total_memory();
    let participating_devices = participating_gpu_devices(
        settings.background_gpu_devices.as_deref(),
        &devices_on_this_host,
    )?;
    if participating_devices.is_empty() {
        return Err(BackgroundComputeSettingsRefused::GpuNotPresent);
    }
    if let Some(memory_limits) = &settings.background_gpu_memory_limit {
        if memory_limits.len() != participating_devices.len() {
            return Err(
                BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                    memory_limits_given: memory_limits.len(),
                    participating_devices: participating_devices.len(),
                },
            );
        }
        for ((device, total_memory), memory_limit) in
            participating_devices.iter().zip(memory_limits)
        {
            if memory_limit > total_memory {
                return Err(
                    BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice {
                        device: device.clone(),
                        memory_limit: *memory_limit,
                        total_memory: *total_memory,
                    },
                );
            }
        }
    }
    if settings.background_gpu_wait_limit.is_zero() {
        return Err(BackgroundComputeSettingsRefused::GpuWaitLimitOfZero);
    }
    Ok(settings)
}

/// 参与的卡与各自的显存总量，按上限对应的次序：没点名就是清单里的全部，点了名就是点名的那几张、按点名的次序。
///
/// # Errors
/// 点名的空列表、同一张卡点了两次、点名的卡不在清单里。
fn participating_gpu_devices(
    named_devices: Option<&[GpuDeviceIdentifier]>,
    devices_on_this_host: &[(GpuDeviceIdentifier, GpuMemoryInBytes)],
) -> Result<Vec<(GpuDeviceIdentifier, GpuMemoryInBytes)>, BackgroundComputeSettingsRefused> {
    let Some(named_devices) = named_devices else {
        return Ok(devices_on_this_host.to_vec());
    };
    if named_devices.is_empty() {
        return Err(BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided);
    }
    let mut participating_devices = Vec::with_capacity(named_devices.len());
    for (position_in_the_list, named_device) in named_devices.iter().enumerate() {
        if named_devices[..position_in_the_list].contains(named_device) {
            return Err(
                BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided {
                    device: named_device.clone(),
                },
            );
        }
        let Some(device_on_this_host) = devices_on_this_host
            .iter()
            .find(|(device, _)| device == named_device)
        else {
            return Err(
                BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory {
                    device: named_device.clone(),
                },
            );
        };
        participating_devices.push(device_on_this_host.clone());
    }
    Ok(participating_devices)
}
