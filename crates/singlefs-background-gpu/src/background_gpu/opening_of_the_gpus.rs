//! 开卡：进程级的 Vulkan 实例、挂载那一刻开参与的卡、切换时在另一条线程上开新点到的卡、开不成的重开与连着开不成的次数、
//! 同一张卡上前一次开卡还没回来时不起下一次（进程级的那一张表）、量代价与枚举本机适配器。

use super::*;

impl VulkanInstanceOfThisProcess {
    /// 建这个进程的那一个实例：调用方一个进程只建这一次、留着它，之后每次开卡都交它。
    /// 驱动清单定在这一刻（[`VulkanInstanceOfThisProcess`]）：此刻驱动不在也交回实例、不报错，这个实例枚举一直是空的，之后修好驱动它也看不到。
    #[must_use]
    pub fn created_for_this_process() -> Self {
        Self::with_the_backends(wgpu::Backends::VULKAN)
    }

    /// 一个后端都不开的实例：不装 Vulkan 加载器、不碰 GPU，枚举交回空列表。只给不要卡的单测走开卡那一条路。
    #[cfg(test)]
    pub(super) fn without_any_backend() -> Self {
        Self::with_the_backends(wgpu::Backends::empty())
    }

    fn with_the_backends(backends: wgpu::Backends) -> Self {
        Self {
            instance: wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends,
                ..wgpu::InstanceDescriptor::new_without_display_handle()
            }),
        }
    }

    /// 这个实例看得到的 Vulkan 适配器，按枚举的次序。枚举本身不受 `wait_limit` 管：wgpu 在造 future 那一步同步做完枚举，期限从它做完才起算，
    /// 驱动在枚举里挂住时这里跟着挂住、不交回。
    ///
    /// # Errors
    /// 枚举做完之后的等超过 `wait_limit`：上限小到头一次看就到期（上限是 0 同样），或到期限 future 还没好。
    pub fn vulkan_adapters_enumerated_within(
        &self,
        wait_limit: Duration,
    ) -> Result<Vec<wgpu::Adapter>, GpuFailureCause> {
        wait_for_the_wgpu_future(
            self.instance.enumerate_adapters(wgpu::Backends::VULKAN),
            wait_limit,
            &SteadyClockOfThisProcess::starting_now(),
        )
    }
}

/// 开卡从哪里取这个进程的 Vulkan 实例：产品交 [`VulkanInstanceOfThisProcess`] 本身；不要卡的单测换上记下取了几次的替身，
/// 钉住每次开卡都取交进来的那一份、不另建。
pub trait VulkanInstanceHandedToTheOpening {
    /// 这一次开卡用的实例（交回的克隆指的是同一个实例）。
    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess;
}

impl VulkanInstanceHandedToTheOpening for VulkanInstanceOfThisProcess {
    fn vulkan_instance_for_this_opening(&self) -> VulkanInstanceOfThisProcess {
        self.clone()
    }
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间（同
/// [`open_background_gpu_with_the_memory_leaked_in_this_process`]，进程里还一张卡都没泄漏过、没有一次开卡挂住；实例是调用方留着的 `vulkan_instance`）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的五种。
pub fn open_background_gpu(
    selection: &BackgroundGpuSelection,
    vulkan_instance: &VulkanInstanceOfThisProcess,
) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
    open_background_gpu_with_the_memory_leaked_in_this_process(
        selection,
        &GpuMemoryLeakedInThisProcess::with_nothing_leaked(),
        &OpeningsNotReturnedInThisProcess::with_none_not_returned(),
        vulkan_instance,
        |_gpus_to_open| {},
    )
}

/// 挂载那一刻：枚举适配器、判卡清单与校验、只在参与的卡上起设备与各段两份核、量一次提交的主机 CPU 时间；参与的卡照 `selection`
/// （之后切换时核心层另交，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。等 wgpu 的 future 都在 `selection.background_gpu_wait_limit` 之内；
/// 枚举适配器本身不在其内：wgpu 在造 future 那一步同步做完枚举，驱动在枚举里挂住时开卡跟着挂住、不拒（[`VulkanInstanceOfThisProcess::vulkan_adapters_enumerated_within`]）。
/// 上限给 0 在枚举之前就拒成 [`BackgroundGpuOpenRefused::SettingsRefused`] 的 `GpuWaitLimitOfZero`，不看选了哪几张卡。
/// 给了 `background_gpu_devices` 时只判点名的卡（没点名的独显交不成一张卡就不收、报出来）。
/// 每张卡开卡（起设备、编各段两份核的管线、量代价）至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]（五分钟），到点当那张卡没开成
/// （`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`，报等超过上限、带这个上限），挂载等开卡成功或到点才往下走。
/// 一张参与的卡开不成就隔 1 分钟重开、至多重开 3 次（[`ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT`]，这条线程上睡着等），都不成才拒绝挂载。
/// 枚举完、判过卡清单与校验，逐张开卡之前把要开几张卡（参与的卡数）交给 `report_the_gpus_to_open`（进程开卡器照它把外层的期限延到枚举期限加
/// 卡数乘一张卡连同重开的最长等待）；枚举不成、卡清单或校验拒了的不报。
/// 量代价出错时账上记着还在跑的提交的那张卡泄漏、不析构，这一次占着的字节当场记进 `memory_leaked_in_this_process`
/// （`failure_of_the_measurement_dropping_or_leaking_the_card`）。
/// 没参与的卡不开、不量代价（第 2 条「挂载时只开参与的卡」）：卡清单照列它，之后的切换点到它时在会话锁外开（[`OpeningOfAGpuAtASwitch`]，
/// 同一份适配器与泄漏账）。参与的卡重开到上限都没开成就拒绝挂载（[`gpus_opened_at_the_mount`]），拒绝里带判它开不成那一刻读的本机 PCI 状态
/// （[`PciStateReaderOfTheGpus::of_this_host`]；挂着之后判卡用不了时也照它读）。参与的卡照开卡那一组给它的显存上限
/// （先扣掉这个进程里之前几次挂载泄漏在它上面的字节，`memory_leaked_in_this_process`）把量代价那一段（[`COST_PROBE_CHUNK`]）切成几片逐片量、代价取各片之和；
/// 显存上限连最小的一片都装不下的卡照开、不量代价、不参与、报出来（[`BackgroundGpuOverTheCards::gpus_left_out_under_their_memory_limit`]），挂载不因此拒绝；
/// 参与的卡因此一张都不剩时照样交回，三段每一批都报「参与的卡一张都不剩」（[`BatchNotSplitOverTheGpus::EveryParticipatingGpuLeftOut`]），
/// 核心层挂载那一刻就读到、当场报警停着。丢掉它时泄漏的卡记进 `memory_leaked_in_this_process`。
/// 枚举与之后现查本机在的独显都用 `vulkan_instance_handed_over` 交来的那一个实例（调用方在进程里留着的那一份，[`VulkanInstanceOfThisProcess`]），不另建。
/// 一张卡上前一次开卡到点之后还没回来就不再起下一次、没到点还在开的就排队等它开完，那一张表是 `openings_not_returned_in_this_process`（调用方在进程里留着的那一份，挂载与之后的切换开卡共用）。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的五种。
pub fn open_background_gpu_with_the_memory_leaked_in_this_process(
    selection: &BackgroundGpuSelection,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: &OpeningsNotReturnedInThisProcess,
    vulkan_instance_handed_over: &impl VulkanInstanceHandedToTheOpening,
    report_the_gpus_to_open: impl FnOnce(usize),
) -> Result<BackgroundGpu, BackgroundGpuOpenRefused> {
    // 上限 0 是配错，出路是改参数；枚举那一步头一次看就算到期，要先判，不然报成枚举没成。
    if selection.background_gpu_wait_limit.is_zero() {
        return Err(BackgroundGpuOpenRefused::SettingsRefused(
            BackgroundComputeSettingsRefused::GpuWaitLimitOfZero,
        ));
    }
    let vulkan_instance = vulkan_instance_handed_over.vulkan_instance_for_this_opening();
    let adapters = vulkan_instance
        .vulkan_adapters_enumerated_within(selection.background_gpu_wait_limit)
        .map_err(|cause| BackgroundGpuOpenRefused::AdapterEnumerationFailed { cause })?;
    let descriptions: Vec<EnumeratedAdapterDescription> =
        adapters.iter().map(description_of_the_adapter).collect();
    let (discrete_gpus, participating) =
        discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
            selection,
            &descriptions,
        )?;
    report_the_gpus_to_open(participating.len());
    let mut open_a_gpu = opening_of_a_gpu_in_use(
        adapters,
        memory_leaked_in_this_process.clone(),
        openings_not_returned_in_this_process.clone(),
    );
    let pci_state_reader = PciStateReaderOfTheGpus::of_this_host();
    let gpus_opened = gpus_opened_at_the_mount(
        discrete_gpus.listed,
        &participating,
        bytes_of_the_smallest_piece_every_participating_gpu_holds(),
        memory_leaked_in_this_process,
        ReopeningsAndPciStateReadingOfAGpuNotOpened {
            reopenings: ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
            pci_state_reader: &pci_state_reader,
        },
        std::thread::sleep,
        |discrete_gpu, memory_limit_at_the_opening, cost_at_the_opening| {
            open_a_gpu(
                discrete_gpu,
                memory_limit_at_the_opening,
                selection.background_gpu_wait_limit,
                cost_at_the_opening,
            )
        },
    )?;
    Ok(BackgroundGpuOverTheCards::of_the_cards_opened_at_the_mount(
        gpus_opened.opened,
        gpus_opened.discrete_gpus,
        discrete_gpus.unnamed_left_off_the_list,
        gpus_opened.participating,
        gpus_opened.left_out_under_their_memory_limit,
        memory_leaked_in_this_process.clone(),
        Box::new(AdaptersEnumeratedByTheInstance {
            vulkan_instance,
            wait_limit: selection.background_gpu_wait_limit,
        }),
        Box::new(open_a_gpu),
        ReopeningsOfAGpuNotOpened::OF_THE_PRODUCT,
        pci_state_reader,
    ))
}

/// 在这一次枚举到的适配器（`adapters`，按枚举的次序）里的一张独显上开卡：起设备、编各段两份核的管线（[`started_gpu_in_use`]），
/// 再照开卡那一步（[`opening_of_a_card_within_the_limit`]）量代价、出错时处置这张卡、整个开卡至多等 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]。
/// 挂载那一刻开参与的卡与切换生效那一刻开挂载时没开的卡都走它。
fn opening_of_a_gpu_in_use(
    adapters: Vec<wgpu::Adapter>,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<GpuInUse, GpuFailureCause>
       + Send
       + 'static {
    opening_of_a_card_within_the_limit(
        WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT,
        move |discrete_gpu: &DiscreteGpuOnThisHost,
              memory_limit_at_the_opening,
              wait_limit,
              bytes_leaked_by_earlier_mounts| {
            let adapter = adapters[discrete_gpu.position_in_the_enumeration].clone();
            let discrete_gpu_to_start = discrete_gpu.clone();
            move || {
                started_gpu_in_use(
                    &adapter,
                    discrete_gpu_to_start,
                    memory_limit_at_the_opening,
                    wait_limit,
                    bytes_leaked_by_earlier_mounts,
                )
            }
        },
        memory_leaked_in_this_process,
        openings_not_returned_in_this_process,
    )
}

/// 开一张卡那一步（挂载那一刻开参与的卡、切换生效那一刻开挂载时没开的卡，对任何一种卡都一样；`Card` 产品是 [`GpuInUse`]，测试换上替身卡）：
/// `start_of_the_card` 照这张卡、给它的显存上限、等 GPU 的上限与这个进程里之前几次挂载泄漏在它上面的字节（从 `memory_leaked_in_this_process` 现读）
/// 交回起这张卡的那一件；那一件连同量代价（[`started_gpu_with_its_cost_measured`]，量不量照 `CostAtTheOpening`；出错时有提交还在跑的那张卡泄漏、
/// 这一次占着的字节记进 `memory_leaked_in_this_process`）在另一条线程上做，整个至多等 `opening_limit`（产品是 [`WAIT_LIMIT_OF_OPENING_A_GPU_AT_THE_MOUNT`]；
/// 到点当那张卡没开成，`waiting_for_the_gpu::outcome_of_opening_a_gpu_within_the_limit`）。处置量代价出错的卡在那条线程上：
/// 到点之后那条线程跑完时交回的只是那个错，没有要析构的卡。
/// 同一张卡上前一次开卡到点之后还没回来（开它的那一方到点不等了，那条线程还挂在驱动里），就不再起下一次：这一次当场算没开成
/// （[`opening_not_started_while_an_earlier_one_has_not_returned`]），照重开的间隔往下数（挂载与切换同一个规矩）。前一次没到点、还在开的（别的挂载正在开这张卡），
/// 这一次在自己的期限之内排队等它开完再开，等的时间算进这一次的期限，等满了照这一次到点办（报等超过上限、带 `opening_limit`）。
/// 那一张表是 `openings_not_returned_in_this_process`（进程开卡器持有、各次挂载与切换共用：前一次挂载里挂住的那张卡，下一次挂载同样不再起），
/// 同一张卡上同时在开的线程整个进程至多一条。
pub(super) fn opening_of_a_card_within_the_limit<Card, StartTheCard>(
    opening_limit: Duration,
    mut start_of_the_card: impl FnMut(
            &DiscreteGpuOnThisHost,
            Option<GpuMemoryInBytes>,
            Duration,
            GpuMemoryInBytes,
        ) -> StartTheCard
        + Send
        + 'static,
    memory_leaked_in_this_process: GpuMemoryLeakedInThisProcess,
    openings_not_returned_in_this_process: OpeningsNotReturnedInThisProcess,
) -> impl FnMut(
    &DiscreteGpuOnThisHost,
    Option<GpuMemoryInBytes>,
    Duration,
    CostAtTheOpening,
) -> Result<Card, GpuFailureCause>
       + Send
       + 'static
where
    Card: CardOfTheBackgroundGpu + Send + 'static,
    StartTheCard: FnOnce() -> Result<Card, GpuFailureCause> + Send + 'static,
{
    move |discrete_gpu, memory_limit_at_the_opening, wait_limit, cost_at_the_opening| {
        let opening_began_at = Instant::now();
        let opening_under_way =
            match OpeningUnderWayOnTheDevice::noted_once_no_earlier_one_is_under_way(
                &openings_not_returned_in_this_process,
                &discrete_gpu.identifier,
                opening_began_at,
                opening_limit,
            ) {
                Ok(opening_under_way) => opening_under_way,
                Err(EarlierOpeningOfTheDeviceInTheWay::NotReturnedPastTheLimitOfItsCaller) => {
                    return Err(opening_not_started_while_an_earlier_one_has_not_returned());
                }
                Err(EarlierOpeningOfTheDeviceInTheWay::StillUnderWayAtTheLimitOfThisOpening) => {
                    return Err(GpuFailureCause::WaitForTheGpuExceededTheLimit {
                        wait_limit: opening_limit,
                    });
                }
            };
        let opening_number = opening_under_way.opening_number;
        let bytes_leaked_by_earlier_mounts =
            memory_leaked_in_this_process.bytes_leaked_on(&discrete_gpu.identifier);
        let start_the_card = start_of_the_card(
            discrete_gpu,
            memory_limit_at_the_opening,
            wait_limit,
            bytes_leaked_by_earlier_mounts,
        );
        let memory_leaked_noted_by_the_opening = memory_leaked_in_this_process.clone();
        let outcome = outcome_of_opening_a_gpu_within_what_is_left_of_the_limit(
            opening_limit,
            opening_limit.saturating_sub(opening_began_at.elapsed()),
            move || {
                let outcome_of_the_start = started_gpu_with_its_cost_measured(
                    start_the_card,
                    cost_at_the_opening,
                    &memory_leaked_noted_by_the_opening,
                );
                // 交回结局之前先从表里划掉：等它的那一头收到结局时，表里已没有这一次。
                drop(opening_under_way);
                outcome_of_the_start
            },
        );
        // 收到了结局的，这一次在表里已经划掉、下面这一步什么都不动；到点没收到的，那条线程还挂着，记成没回来。
        openings_not_returned_in_this_process
            .note_the_opening_not_returned_past_the_limit_of_its_caller(opening_number);
        outcome
    }
}

impl OpeningsNotReturnedInThisProcess {
    /// 这个进程里还没有一次开卡在开、没有一次挂住。
    #[must_use]
    pub fn with_none_not_returned() -> Self {
        Self {
            openings_on_the_devices: Arc::new((
                Mutex::new(OpeningsUnderWayInThisProcess {
                    openings: Vec::new(),
                    openings_numbered_so_far: 0,
                    openings_waiting_for_an_earlier_one: 0,
                }),
                Condvar::new(),
            )),
        }
    }

    /// 两份把手是不是同一张表（进程开卡器交给各次开卡的是同一张）。
    #[must_use]
    pub fn is_the_same_table_as(&self, other: &Self) -> bool {
        Arc::ptr_eq(
            &self.openings_on_the_devices,
            &other.openings_on_the_devices,
        )
    }

    /// 此刻有一次开卡到点之后还没回来的卡（还在期限之内开着的不列）。
    #[must_use]
    pub fn devices_with_an_opening_not_returned(&self) -> Vec<GpuDeviceIdentifier> {
        self.openings_locked()
            .openings
            .iter()
            .filter(|opening| {
                opening.state == StateOfAnOpeningUnderWay::NotReturnedPastTheLimitOfItsCaller
            })
            .map(|opening| opening.device.clone())
            .collect()
    }

    /// 此刻有几次开卡在排队等同一张卡上前一次开完（开它的那一方还在期限之内）。
    #[must_use]
    pub fn openings_waiting_for_an_earlier_one(&self) -> usize {
        self.openings_locked().openings_waiting_for_an_earlier_one
    }

    fn openings_locked(&self) -> std::sync::MutexGuard<'_, OpeningsUnderWayInThisProcess> {
        self.openings_on_the_devices
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// 第 `opening_number` 次开卡的那一方到点不等了：它还在表里（那条线程还没回来）就记成没回来，叫醒排队等这张卡的；已经回来、划掉了的不动。
    fn note_the_opening_not_returned_past_the_limit_of_its_caller(&self, opening_number: u64) {
        let mut openings = self.openings_locked();
        if let Some(opening) = openings
            .openings
            .iter_mut()
            .find(|opening| opening.opening_number == opening_number)
        {
            opening.state = StateOfAnOpeningUnderWay::NotReturnedPastTheLimitOfItsCaller;
            self.openings_on_the_devices.1.notify_all();
        }
    }
}

impl OpeningUnderWayOnTheDevice {
    /// 这张卡上没有在开的开卡，就记下这一次、交回它。有一次在开而开它的那一方还在期限之内，就排队等它开完（表里划掉）再记，
    /// 等到这一次自己的期限（`opening_began_at` 起 `opening_limit`）还没等到就不记、交回还在开；前一次已记成没回来的不等、交回没回来。
    fn noted_once_no_earlier_one_is_under_way(
        openings_in_this_process: &OpeningsNotReturnedInThisProcess,
        device: &GpuDeviceIdentifier,
        opening_began_at: Instant,
        opening_limit: Duration,
    ) -> Result<Self, EarlierOpeningOfTheDeviceInTheWay> {
        let (_openings, an_opening_ended_or_was_given_up) =
            &*openings_in_this_process.openings_on_the_devices;
        let mut openings = openings_in_this_process.openings_locked();
        let mut is_counted_as_waiting = false;
        let noted = loop {
            let state_of_the_earlier_one = openings
                .openings
                .iter()
                .find(|opening| opening.device == *device)
                .map(|opening| opening.state);
            match state_of_the_earlier_one {
                None => {
                    openings.openings_numbered_so_far += 1;
                    let opening_number = openings.openings_numbered_so_far;
                    openings.openings.push(OpeningUnderWayInThisProcess {
                        device: device.clone(),
                        opening_number,
                        state: StateOfAnOpeningUnderWay::WithinTheLimitOfItsCaller,
                    });
                    break Ok(opening_number);
                }
                Some(StateOfAnOpeningUnderWay::NotReturnedPastTheLimitOfItsCaller) => {
                    break Err(
                        EarlierOpeningOfTheDeviceInTheWay::NotReturnedPastTheLimitOfItsCaller,
                    );
                }
                Some(StateOfAnOpeningUnderWay::WithinTheLimitOfItsCaller) => {
                    let time_left = opening_limit.saturating_sub(opening_began_at.elapsed());
                    if time_left.is_zero() {
                        break Err(
                            EarlierOpeningOfTheDeviceInTheWay::StillUnderWayAtTheLimitOfThisOpening,
                        );
                    }
                    if !is_counted_as_waiting {
                        openings.openings_waiting_for_an_earlier_one += 1;
                        is_counted_as_waiting = true;
                    }
                    openings = an_opening_ended_or_was_given_up
                        .wait_timeout(openings, time_left)
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .0;
                }
            }
        };
        if is_counted_as_waiting {
            openings.openings_waiting_for_an_earlier_one -= 1;
        }
        noted.map(|opening_number| Self {
            openings_in_this_process: openings_in_this_process.clone(),
            opening_number,
        })
    }
}

impl Drop for OpeningUnderWayOnTheDevice {
    fn drop(&mut self) {
        let mut openings = self.openings_in_this_process.openings_locked();
        openings
            .openings
            .retain(|opening| opening.opening_number != self.opening_number);
        self.openings_in_this_process
            .openings_on_the_devices
            .1
            .notify_all();
    }
}

/// 同一张卡上前一次开卡到点没回来、这一次不起时报的原因（算这一次没开成，照重开的间隔往下数；不记进连着开不成的次数）。
pub(super) fn opening_not_started_while_an_earlier_one_has_not_returned() -> GpuFailureCause {
    GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned
}

/// 开一次卡（`open`），开卡里 panic 了就接住、算这一次没开成（[`GpuFailureCause::OpeningPanicked`]，带着 panic 的原话），不往上传：
/// 挂载那一刻与切换开卡线程都照重开的规矩往下走，不在两批之间持着会话锁时再 panic（挂载与切换共用的开一张卡那一步
/// `gpus_of_this_host::gpu_opened_under_its_memory_limit` 包着它）。
pub(crate) fn opening_with_its_panic_caught<Opened>(
    open: impl FnOnce() -> Result<Opened, GpuFailureCause>,
) -> Result<Opened, GpuFailureCause> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(open)).unwrap_or_else(|payload| {
        let said_by_the_panic = payload
            .downcast_ref::<&str>()
            .map(|said| (*said).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "（panic 带的不是字符串）".to_string());
        Err(GpuFailureCause::OpeningPanicked { said_by_the_panic })
    })
}

/// 切换开卡叫停之后（卸载、切到 `cpu`）不再开的那几次报的原因：这一次开卡的结局没人收，原因不报到哪里。
fn opening_called_off() -> GpuFailureCause {
    GpuFailureCause::OpeningCalledOff
}

impl FailedOpeningsOfEachGpuInThisMount {
    /// 这张卡这一次挂载里到此刻连着开不成了几次。
    pub(super) fn failed_openings_of(&self, device: &GpuDeviceIdentifier) -> u32 {
        self.failed_openings_of_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|(counted_device, _failed_openings)| counted_device == device)
            .map_or(0, |(_device, failed_openings)| *failed_openings)
    }

    /// 这张卡这一次挂载里还能开几次：一张卡至多开 [`ReopeningsOfAGpuNotOpened::openings_at_most`] 次，扣掉它此刻连着开不成的；0 是试满了。
    /// 交设置那一刻判试满（[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）与切换开卡线程开一张卡之前算这一次至多开几次
    /// （[`gpus_opened_at_a_switch`]）都读它。
    pub(super) fn openings_left_in_this_mount_of(
        &self,
        device: &GpuDeviceIdentifier,
        reopenings: ReopeningsOfAGpuNotOpened,
    ) -> u32 {
        reopenings
            .openings_at_most()
            .saturating_sub(self.failed_openings_of(device))
    }

    /// 记下这张卡又开不成一次。
    pub(super) fn note_a_failed_opening(&self, device: &GpuDeviceIdentifier) {
        let mut failed_openings_of_each_device = self
            .failed_openings_of_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match failed_openings_of_each_device
            .iter_mut()
            .find(|(counted_device, _failed_openings)| counted_device == device)
        {
            Some((_device, failed_openings)) => *failed_openings += 1,
            None => failed_openings_of_each_device.push((device.clone(), 1)),
        }
    }

    /// 这张卡开成了：它连着开不成的次数清零，别的卡的不动。
    fn forget_the_failed_openings_of(&self, device: &GpuDeviceIdentifier) {
        self.failed_openings_of_each_device
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|(counted_device, _failed_openings)| counted_device != device);
    }

    /// 照切换开卡线程开一次的结局记这张卡：开成清零；开不成照 [`opening_counts_toward_the_failed_openings`] 记一次或不记。
    fn note_the_outcome_of_an_opening<Opened>(
        &self,
        device: &GpuDeviceIdentifier,
        outcome: &Result<Opened, GpuFailureCause>,
    ) {
        match outcome {
            Ok(_opened) => self.forget_the_failed_openings_of(device),
            Err(cause) => {
                if opening_counts_toward_the_failed_openings(cause) {
                    self.note_a_failed_opening(device);
                }
            }
        }
    }
}

/// 这一次开不成算不算进一张卡连着开不成的次数：叫停的那一次没开，不算；前一次开卡到点没回来、这一次没起的，不算（出路是重启进程，
/// 不是卸载重挂：不把这张卡判成试满，之后的切换照常为它起开卡，照旧报没回来）；调用方在停而放弃的那一等不算（开卡那几等不读停标记，走不到；
/// 算的话一次没开成的数里混进了没开完的那一次）；别的都算。
fn opening_counts_toward_the_failed_openings(cause: &GpuFailureCause) -> bool {
    match cause {
        GpuFailureCause::OpeningCalledOff
        | GpuFailureCause::EarlierOpeningOfTheDeviceNotReturned => false,
        GpuFailureCause::GaveUpWaitingForTheCallerIsStopping => false,
        GpuFailureCause::DeviceOrPipelineDidNotStart { .. }
        | GpuFailureCause::OpeningThreadNotStarted { .. }
        | GpuFailureCause::OpeningPanicked { .. }
        | GpuFailureCause::OpeningsInThisMountUsedUp { .. }
        | GpuFailureCause::FlushOfThePendingWritesNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::UncapturedErrorOrDeviceLost { .. }
        | GpuFailureCause::WaitForTheSubmissionFailed { .. }
        | GpuFailureCause::WaitForTheGpuExceededTheLimit { .. }
        | GpuFailureCause::EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit { .. }
        | GpuFailureCause::ReadbackMappingFailed { .. }
        | GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord(..)
        | GpuFailureCause::OutOfMemoryCreatingTheBuffers { .. }
        | GpuFailureCause::UnusableAfterResubmissions { .. }
        | GpuFailureCause::NotOpenedAfterTheReopenings { .. } => true,
    }
}

/// 切换开卡那条线程上做的事：按给的次序逐张开 `discrete_gpus_to_open`（各带这一组给它的显存上限），开一张同挂载那一刻开卡那一步
/// （[`gpu_opened_under_its_memory_limit`]，开一次交 `opening_of_a_gpu_at_a_switch`、等 GPU 的上限是这一组的 `wait_limit`；开卡里 panic 了在那一步就接住、
/// 算这一次没开成），开不成就照 `reopenings` 的间隔重开、每次重开之前在这条线程上睡一个间隔（[`gpu_opened_with_the_reopenings`]）。
/// 一张卡这一次至多开它在这一次挂载里剩下的次数（[`FailedOpeningsOfEachGpuInThisMount::openings_left_in_this_mount_of`]，与交设置那一刻判试满是同一个判据）；
/// 开到它这一刻已一次不剩（交设置那一刻还剩，之后前一条线程叫停时正在开的那一次开完、记进了次数）就不开，当场记成它开不成（[`GpuFailureCause::OpeningsInThisMountUsedUp`]，
/// 排在起线程之前的原因后面），后面的不再开。每开不成一次记进 `failed_openings_in_this_mount`、开成一次把它清零（不记的几种照 [`opening_counts_toward_the_failed_openings`]）。
/// 有一张重开到上限都没开成就停在它，后面的不再开。
/// 装不下最小的一片的照开、不量代价（换上那一组时照这张卡的账与开着的卡同一判，再判一次）。`causes_of_each_start` 是起这条线程之前起不了线程的那几次
/// （各算头一张卡一次没开成、已记进它连着开不成的次数）：原因排在头一张卡的原因前面。
/// 每一次开卡之前看 `called_off`：置上了就不再开（卸载、切到 `cpu`），也不再睡重开的间隔。
/// 判一张卡开不成（重开到上限都没开成、或开到它这一刻已试满）那一刻照 `pci_state_reader` 读一次它的 PCI 状态，带进开不成的那一份；
/// 每一次没开成而还要重开时不读，`called_off` 置上了（结局没人收）也不读（[`pci_state_of_the_gpu_not_opened_unless_called_off`]）。
#[allow(
    clippy::too_many_arguments,
    reason = "参数各是一样东西：开一张卡那一步、要开的卡、等的上限、重开的规矩、起线程之前的原因、叫停、泄漏账、连着开不成的次数与读 PCI 状态的那一份"
)]
pub(super) fn gpus_opened_at_a_switch<Card>(
    opening_of_a_gpu_at_a_switch: &mut OpeningOfAGpuAtASwitch<Card>,
    discrete_gpus_to_open: Vec<(DiscreteGpuOnThisHost, Option<GpuMemoryInBytes>)>,
    wait_limit: Duration,
    reopenings: ReopeningsOfAGpuNotOpened,
    causes_of_each_start: Vec<GpuFailureCause>,
    called_off: &AtomicBool,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
    failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
    pci_state_reader: &PciStateReaderOfTheGpus,
) -> OutcomeOfTheOpeningAtASwitch<Card> {
    let smallest_piece_bytes = bytes_of_the_smallest_piece_every_participating_gpu_holds();
    let mut opened = Vec::with_capacity(discrete_gpus_to_open.len());
    let mut causes_before_the_thread = causes_of_each_start;
    for (discrete_gpu, memory_limit) in discrete_gpus_to_open {
        let causes_before_this_card = std::mem::take(&mut causes_before_the_thread);
        let openings_left_in_this_mount = failed_openings_in_this_mount
            .openings_left_in_this_mount_of(&discrete_gpu.identifier, reopenings);
        // 交设置那一刻没试满、这一刻试满了：前一条切换开卡线程叫停时正在开的那一次（叫停只拦之后的几次）在这条线程起来之后才开完、记进了次数。
        // 照交设置那一刻同一个判据当场记成开不成，不再开。
        if openings_left_in_this_mount == 0 {
            let mut causes_of_each_opening = causes_before_this_card;
            causes_of_each_opening.push(openings_in_this_mount_used_up(
                failed_openings_in_this_mount,
                &discrete_gpu.identifier,
            ));
            return OutcomeOfTheOpeningAtASwitch {
                opened,
                not_opened: Some(GpuNotOpenedAfterTheReopenings {
                    pci_state_when_judged_unusable:
                        pci_state_of_the_gpu_not_opened_unless_called_off(
                            pci_state_reader,
                            called_off,
                            &discrete_gpu.identifier,
                        ),
                    device: discrete_gpu.identifier,
                    causes_of_each_opening,
                }),
            };
        }
        let reopenings_of_this_card = ReopeningsOfAGpuNotOpened {
            interval: reopenings.interval,
            reopenings_at_most: openings_left_in_this_mount.saturating_sub(1),
        };
        let opened_with_the_reopenings = gpu_opened_with_the_reopenings(
            reopenings_of_this_card,
            |interval| {
                if !called_off.load(Ordering::SeqCst) {
                    std::thread::sleep(interval);
                }
            },
            || {
                let opened_once = if called_off.load(Ordering::SeqCst) {
                    Err(opening_called_off())
                } else {
                    gpu_opened_under_its_memory_limit(
                        &discrete_gpu,
                        memory_limit,
                        smallest_piece_bytes,
                        memory_leaked_in_this_process,
                        |gpu_to_open, memory_limit_at_the_opening, cost_at_the_opening| {
                            opening_of_a_gpu_at_a_switch(
                                gpu_to_open,
                                memory_limit_at_the_opening,
                                wait_limit,
                                cost_at_the_opening,
                            )
                        },
                    )
                };
                failed_openings_in_this_mount
                    .note_the_outcome_of_an_opening(&discrete_gpu.identifier, &opened_once);
                opened_once
            },
        );
        match opened_with_the_reopenings {
            Ok((card, _left_out_judged_again_when_the_settings_are_put_in_effect)) => {
                opened.push(card);
            }
            Err(causes_in_the_thread) => {
                let mut causes_of_each_opening = causes_before_this_card;
                causes_of_each_opening.extend(causes_in_the_thread);
                return OutcomeOfTheOpeningAtASwitch {
                    opened,
                    not_opened: Some(GpuNotOpenedAfterTheReopenings {
                        pci_state_when_judged_unusable:
                            pci_state_of_the_gpu_not_opened_unless_called_off(
                                pci_state_reader,
                                called_off,
                                &discrete_gpu.identifier,
                            ),
                        device: discrete_gpu.identifier,
                        causes_of_each_opening,
                    }),
                };
            }
        }
    }
    OutcomeOfTheOpeningAtASwitch {
        opened,
        not_opened: None,
    }
}

/// 切换开卡线程判 `device` 开不成、交回之前读它的 PCI 状态：`called_off` 置上了（卸载、切到 `cpu`；叫停之后余下的几次没开，开不成的结局送回去也没人收）
/// 就不读、交回没读；没叫停（重开确实用完、或开到它这一刻已试满，结局后来被更新的交设置盖掉的也算）照读一次。
fn pci_state_of_the_gpu_not_opened_unless_called_off(
    pci_state_reader: &PciStateReaderOfTheGpus,
    called_off: &AtomicBool,
    device: &GpuDeviceIdentifier,
) -> PciStateOfTheGpuReadWhenJudgedUnusable {
    if called_off.load(Ordering::SeqCst) {
        return PciStateOfTheGpuReadWhenJudgedUnusable::NotReadForThisError;
    }
    pci_state_reader.pci_state_of_the_gpu(device)
}

/// 一张卡在这一次挂载里已试满、不再开时报的原因（带它连着开不成的次数）。
pub(super) fn openings_in_this_mount_used_up(
    failed_openings_in_this_mount: &FailedOpeningsOfEachGpuInThisMount,
    device: &GpuDeviceIdentifier,
) -> GpuFailureCause {
    GpuFailureCause::OpeningsInThisMountUsedUp {
        failed_openings: failed_openings_in_this_mount.failed_openings_of(device),
    }
}

/// 起一张卡（`start_the_card`）、照开卡那一组给它的界量一次提交的代价（`cost_at_the_opening`）。量代价出错时这张卡照
/// [`failure_of_the_measurement_dropping_or_leaking_the_card`] 处置：有提交还在跑就泄漏、这一次占着的字节记进 `memory_leaked_in_this_process`。
///
/// # Errors
/// 起不了这张卡（`start_the_card` 交回的，原样）、量代价出错。
fn started_gpu_with_its_cost_measured<Card: CardOfTheBackgroundGpu>(
    start_the_card: impl FnOnce() -> Result<Card, GpuFailureCause>,
    cost_at_the_opening: CostAtTheOpening,
    memory_leaked_in_this_process: &GpuMemoryLeakedInThisProcess,
) -> Result<Card, GpuFailureCause> {
    let mut card = start_the_card()?;
    match cost_at_the_opening {
        CostAtTheOpening::MeasuredUnderTheMemoryLimit => {
            if let Err(failure_of_the_measurement) = card.measure_the_cost_of_one_submission() {
                return Err(failure_of_the_measurement_dropping_or_leaking_the_card(
                    card,
                    failure_of_the_measurement,
                    memory_leaked_in_this_process,
                ));
            }
        }
        CostAtTheOpening::NotMeasuredForTheGpuIsLeftOut => {}
    }
    Ok(card)
}

/// 同一次枚举（`descriptions`）交成卡清单（给了 `background_gpu_devices` 时只判点名的卡）、照它判挂载那一刻的校验、取参与的卡：判与取读同一份清单。
/// 核心层拒成「点名的卡不在卡清单里」时，拒绝带上这一次枚举到的每个适配器（类型与总线地址）与没点名、没收进清单的那几张
/// （[`BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration`]）；核心层别的拒照原样交回。
///
/// # Errors
/// [`BackgroundGpuOpenRefused`] 的 `DiscreteGpuListNotSupported`、`NamedGpuDeviceNotOnTheListOfThisEnumeration` 与 `SettingsRefused`。
pub(super) fn discrete_gpus_and_participating_gpus_admitted_from_the_enumeration(
    selection: &BackgroundGpuSelection,
    descriptions: &[EnumeratedAdapterDescription],
) -> Result<(DiscreteGpusOfTheEnumeration, Vec<ParticipatingGpu>), BackgroundGpuOpenRefused> {
    let discrete_gpus = discrete_gpus_of_the_enumeration_under_the_named(
        descriptions,
        selection.background_gpu_devices.as_deref(),
    )
    .map_err(BackgroundGpuOpenRefused::DiscreteGpuListNotSupported)?;
    match participating_gpus_admitted_by_the_mount_checks(selection, &discrete_gpus.listed) {
        Ok(participating) => Ok((discrete_gpus, participating)),
        Err(BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory { device }) => {
            Err(BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration(
                NamedGpuDeviceNotOnTheListOfThisEnumeration {
                    device,
                    adapters_in_the_enumeration: descriptions
                        .iter()
                        .map(AdapterSeenInTheEnumeration::of_the_description)
                        .collect(),
                    unnamed_left_off_the_list: discrete_gpus.unnamed_left_off_the_list,
                },
            ))
        }
        Err(
            refused @ (BackgroundComputeSettingsRefused::GpuNotPresent
            | BackgroundComputeSettingsRefused::MemoryLimitCountDifferentFromTheParticipatingDevices {
                ..
            }
            | BackgroundComputeSettingsRefused::MemoryLimitAboveTheTotalMemoryOfTheDevice { .. }
            | BackgroundComputeSettingsRefused::EmptyGpuDeviceListWhoseMeaningIsUndecided
            | BackgroundComputeSettingsRefused::GpuDeviceNamedMoreThanOnceWhoseMeaningIsUndecided {
                ..
            }
            | BackgroundComputeSettingsRefused::GpuWaitLimitOfZero
            | BackgroundComputeSettingsRefused::GcCpuCompensationThresholdAboveTheWholeCapacity {
                ..
            }),
        ) => Err(BackgroundGpuOpenRefused::SettingsRefused(refused)),
    }
}

impl DiscreteGpusPresentNow for AdaptersEnumeratedByTheInstance {
    fn identifiers_of_the_discrete_gpus_present_now(&self) -> Vec<GpuDeviceIdentifier> {
        let Ok(adapters) = self
            .vulkan_instance
            .vulkan_adapters_enumerated_within(self.wait_limit)
        else {
            // 枚举不成（等 wgpu 超过上限）：一张都不算在，交回空。切换拒成哪个成员由核心层按点没点名判：
            // 点了名的拒成 `NamedGpuDeviceNotInTheInventory`，没点名的拒成 `GpuNotPresent`。
            return Vec::new();
        };
        let descriptions: Vec<EnumeratedAdapterDescription> = adapters
            .iter()
            .map(|adapter| {
                let adapter_information = adapter.get_info();
                EnumeratedAdapterDescription {
                    name: adapter_information.name,
                    device_kind: AdapterDeviceKind::of_the_wgpu_device_type(
                        adapter_information.device_type,
                    ),
                    pci_bus_address: adapter_information.device_pci_bus_id,
                    device_local_memory: None,
                }
            })
            .collect();
        identifiers_of_the_discrete_gpus_in_the_enumeration(&descriptions)
    }
}

fn description_of_the_adapter(adapter: &wgpu::Adapter) -> EnumeratedAdapterDescription {
    let adapter_information = adapter.get_info();
    let device_kind = AdapterDeviceKind::of_the_wgpu_device_type(adapter_information.device_type);
    let device_local_memory = if device_kind.counts_as_a_background_gpu() {
        largest_device_local_memory_heap(adapter)
    } else {
        None
    };
    EnumeratedAdapterDescription {
        name: adapter_information.name,
        device_kind,
        pci_bus_address: adapter_information.device_pci_bus_id,
        device_local_memory,
    }
}

/// 显存总量：这张卡 Vulkan 物理设备最大的那一块设备本地内存堆。独显上显存是一整块设备本地堆；另一块小的设备本地堆
/// （主机可见的那一窗，例如 256 MiB）是同一片显存的一部分，加起来会重算，所以取最大的一块、不求和。不是 Vulkan 后端的适配器交回 `None`。
#[allow(
    unsafe_code,
    reason = "wgpu 不报显存，只能经 hal 层拿 Vulkan 实例与物理设备句柄去读内存堆；wgpu 与 ash 都没有安全的封装"
)]
fn largest_device_local_memory_heap(adapter: &wgpu::Adapter) -> Option<GpuMemoryInBytes> {
    // SAFETY: `as_hal` 只借出 hal 层的适配器，这里不销毁也不改它指向的任何 Vulkan 对象；守卫在这个函数里丢掉。
    let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
    let raw_instance = hal_adapter.shared_instance().raw_instance();
    // SAFETY: 物理设备句柄取自同一个 hal 适配器，那个适配器就是这个实例枚举出来的；守卫 `hal_adapter` 到函数末尾才丢，
    // 这一刻还没丢，实例与物理设备都还在；`vkGetPhysicalDeviceMemoryProperties` 只读。
    let memory_properties = unsafe {
        raw_instance.get_physical_device_memory_properties(hal_adapter.raw_physical_device())
    };
    memory_properties
        .memory_heaps_as_slice()
        .iter()
        .filter(|memory_heap| {
            memory_heap
                .flags
                .contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL)
        })
        .map(|memory_heap| memory_heap.size)
        .max()
        .map(GpuMemoryInBytes)
}
