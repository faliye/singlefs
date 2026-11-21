//! 一张开着的卡（[`GpuInUse`]）：起设备、编六条管线、交一段两份核、等提交与回读，wgpu 报的错与等的时候 panic 怎么收。

use super::*;

/// 在一张卡上起设备、编六条管线，不量代价（量代价在 [`started_gpu_with_its_cost_measured`]）。账从这个进程里之前泄漏在它上面的
/// `bytes_leaked_by_earlier_mounts` 起。
///
/// # Errors
/// 起不了设备、编不了管线。
pub(super) fn started_gpu_in_use(
    adapter: &wgpu::Adapter,
    discrete_gpu: DiscreteGpuOnThisHost,
    memory_limit: Option<GpuMemoryInBytes>,
    wait_limit: Duration,
    bytes_leaked_by_earlier_mounts: GpuMemoryInBytes,
) -> Result<GpuInUse, GpuFailureCause> {
    let adapter_limits = adapter.limits();
    let (device, queue) = wait_for_the_wgpu_future(
        adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("singlefs-background-gpu"),
            required_features: wgpu::Features::empty(),
            // 一段的输入与描述各绑成一张存储缓冲：按这张卡实际支持的上限要，默认的 128 MiB 绑定上限会让一批切得太碎。
            required_limits: adapter_limits.clone(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Performance,
            trace: wgpu::Trace::Off,
        }),
        wait_limit,
        &SteadyClockOfThisProcess::starting_now(),
    )?
    .map_err(|error| GpuFailureCause::DeviceOrPipelineDidNotStart {
        reason: format!("起不了设备：{error}"),
    })?;
    let fault_reported_by_wgpu: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let fault_record_for_uncaptured_errors = Arc::clone(&fault_reported_by_wgpu);
    device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
        record_the_fault_reported_by_wgpu(
            &fault_record_for_uncaptured_errors,
            format!("wgpu 报了没接住的错：{error}"),
        );
    }));
    let fault_record_for_the_loss = Arc::clone(&fault_reported_by_wgpu);
    device.set_device_lost_callback(move |reason, message| {
        record_the_fault_reported_by_wgpu(
            &fault_record_for_the_loss,
            format!("设备丢了（{reason:?}）：{message}"),
        );
    });
    let table_driven_pipeline = compute_pipeline_of_the_kernel(
        &device,
        TABLE_DRIVEN_KERNEL_SOURCE,
        TABLE_DRIVEN_KERNEL_ENTRY_POINT,
    );
    let bitwise_pipeline =
        compute_pipeline_of_the_kernel(&device, BITWISE_KERNEL_SOURCE, BITWISE_KERNEL_ENTRY_POINT);
    let walking_pipeline =
        compute_pipeline_of_the_kernel(&device, WALKING_KERNEL_SOURCE, WALKING_KERNEL_ENTRY_POINT);
    let searching_pipeline = compute_pipeline_of_the_kernel(
        &device,
        SEARCHING_KERNEL_SOURCE,
        SEARCHING_KERNEL_ENTRY_POINT,
    );
    let live_slot_walking_pipeline = compute_pipeline_of_the_kernel(
        &device,
        WALKING_EVERY_RECORD_KERNEL_SOURCE,
        WALKING_EVERY_RECORD_KERNEL_ENTRY_POINT,
    );
    let live_slot_searching_pipeline = compute_pipeline_of_the_kernel(
        &device,
        SEARCHING_SORTED_RECORDS_KERNEL_SOURCE,
        SEARCHING_SORTED_RECORDS_KERNEL_ENTRY_POINT,
    );
    if let Some(reason) = fault_taken_from_the_record(&fault_reported_by_wgpu) {
        return Err(GpuFailureCause::DeviceOrPipelineDidNotStart { reason });
    }
    let largest_entry_count_per_dispatch = EntryCountOfAChunk(
        u64::from(adapter_limits.max_compute_workgroups_per_dimension)
            * u64::from(
                INVOCATIONS_PER_WORKGROUP
                    .min(REGION_RECORD_INVOCATIONS_PER_WORKGROUP)
                    .min(LIVE_SLOT_INVOCATIONS_PER_WORKGROUP),
            ),
    );
    let budget = GpuBudgetUnderTheSettings::with_the_bytes_leaked_by_earlier_mounts(
        GpuMemoryInBytes(
            adapter_limits
                .max_storage_buffer_binding_size
                .min(adapter_limits.max_buffer_size)
                .min(LARGEST_STORAGE_BINDING_THE_KERNELS_ADDRESS_IN_BYTES.0),
        ),
        largest_entry_count_per_dispatch,
        memory_limit,
        wait_limit,
        bytes_leaked_by_earlier_mounts,
    );
    Ok(GpuInUse {
        discrete_gpu,
        device,
        queue,
        table_driven_pipeline,
        bitwise_pipeline,
        walking_pipeline,
        searching_pipeline,
        live_slot_walking_pipeline,
        live_slot_searching_pipeline,
        budget,
        fault_reported_by_wgpu,
        panic_in_a_wait_of_the_device: OnceLock::new(),
        measured_cost: None,
    })
}

fn compute_pipeline_of_the_kernel(
    device: &wgpu::Device,
    kernel_source: &str,
    entry_point: &str,
) -> wgpu::ComputePipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(entry_point),
        source: wgpu::ShaderSource::Wgsl(kernel_source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(entry_point),
        layout: None,
        module: &module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn record_the_fault_reported_by_wgpu(record: &Mutex<Option<String>>, reason: String) {
    let mut recorded = record
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // 先记下的那一个原因留着：后面的多半是它的连锁反应
    if recorded.is_none() {
        *recorded = Some(reason);
    }
}

fn fault_taken_from_the_record(record: &Mutex<Option<String>>) -> Option<String> {
    record
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
}

fn bytes_read_back_from(readback: &wgpu::Buffer) -> Result<Vec<u8>, GpuFailureCause> {
    let read_back = {
        let mapped = readback.slice(..).get_mapped_range().map_err(|error| {
            GpuFailureCause::ReadbackMappingFailed {
                reason: format!("{error}"),
            }
        })?;
        mapped.to_vec()
    };
    readback.unmap();
    Ok(read_back)
}

/// 主机上要装进一个缓冲的内容有多长（对齐前）。
pub(super) fn bytes_in_a_buffer(bytes: &[u8]) -> InputLengthOnTheHostInBytes {
    InputLengthOnTheHostInBytes::of_this_many_bytes(bytes.len())
}

/// 装着 `contents` 建的缓冲在卡上占的字节数：同 [`buffer_with_contents_created_within_the_scope_of_the_caller`] 的建法，按整字向上补、至少一个整字。
fn bytes_on_the_gpu_of_a_buffer_with(contents: &[u8]) -> GpuMemoryInBytes {
    input_buffer_bytes(bytes_in_a_buffer(contents))
}

/// 看一次这张卡上的这次提交在 GPU 上做没做完，最多等 `time_left`（wgpu 自己带限时的 `poll`）。`poll` 里的 panic 接住、记进
/// `panic_in_a_wait_of_the_device`，之后不再调它（[`completion_check_of_one_poll_with_its_panic_caught`]）。本包里进 wgpu 等的只有这一处
/// （等一批的提交、冲掉待写的空提交、交活与重交之前等前一次还在跑的提交都走它）；回读映射的回调由这一处的 `poll` 推着送到，等映射只收通道、不进 wgpu。
fn completion_check_of_the_submission(
    device: &wgpu::Device,
    panic_in_a_wait_of_the_device: &OnceLock<String>,
    submission: &wgpu::SubmissionIndex,
    time_left: Duration,
) -> CompletionCheck<()> {
    completion_check_of_one_poll_with_its_panic_caught(panic_in_a_wait_of_the_device, || {
        device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission.clone()),
            timeout: Some(time_left),
        })
    })
}

/// 照一次 `poll_once`（wgpu 的 `Device::poll`）的结局判这次提交做没做完。wgpu 30.0.1 在等待报设备丢了时在 `Device::poll` 里 panic
/// （`wgpu-30.0.1/src/backend/wgpu_core.rs` 的 `handle_error_fatal(err, "Device::poll")`）：这里接住，算卡当场报出错
/// （[`GpuFailureCause::UncapturedErrorOrDeviceLost`]，原因带 panic 的原话），走一批出错那一路（任务停下、报警、照 D26（后台整理与放置回收） 已定项 13 第 4 条读一次 PCI 状态），
/// 不让 panic 穿出去（后台那一趟持着会话锁）。panic 的原话记进 `panic_in_a_wait_of_the_device`；记过的不再调 `poll_once`，当场照记下的报。
/// 那一次提交照「没等到它做完」记在账上（`SubmissionOfTheChunk::after_waiting_for_its_completion`），这张卡交活之前的那一等因此每次都走到这里、当场报出错，
/// 不再交活；关它时它有提交还在跑，整张泄漏、不析构（`gpus_of_this_host::gpus_dropped_leaking_those_still_running`）。
pub(super) fn completion_check_of_one_poll_with_its_panic_caught(
    panic_in_a_wait_of_the_device: &OnceLock<String>,
    poll_once: impl FnOnce() -> Result<wgpu::PollStatus, wgpu::PollError>,
) -> CompletionCheck<()> {
    if let Some(said_by_the_panic) = panic_in_a_wait_of_the_device.get() {
        return CompletionCheck::Failed(device_lost_by_a_panic_in_the_wait(said_by_the_panic));
    }
    // AssertUnwindSafe：`poll_once` 只借着这张卡的设备与一个提交号，本包自己的状态（账、记下的错、PCI 读数）一样都不在它里面改，panic 时改了一半的
    // 只可能是 wgpu 里这张卡的设备那一份（设备丢了的标记、在跑的提交与回读映射的登记、映射回调送没送到）。接住之后不再碰它：这一次提交照没等到它做完
    // 留在账上，之后这张卡的每一次等都在上面那一步当场交回、不再调 wgpu；交活与量代价之前都先等这次提交，所以也不再在它上面建缓冲、交提交；
    // 卸载与切换关卡时它有提交还在跑，整张泄漏、不析构（`card_leaked_when_dropped`）。
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(poll_once)) {
        Ok(Ok(_poll_status)) => CompletionCheck::Completed(()),
        Ok(Err(wgpu::PollError::Timeout)) => CompletionCheck::NotYetComplete,
        Ok(Err(wrong_index @ wgpu::PollError::WrongSubmissionIndex(..))) => {
            CompletionCheck::Failed(GpuFailureCause::WaitForTheSubmissionFailed {
                reason: format!("{wrong_index}"),
            })
        }
        Err(payload) => {
            let said_by_the_panic = payload
                .downcast_ref::<&str>()
                .map(|said| (*said).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "（panic 带的不是字符串）".to_string());
            let recorded = panic_in_a_wait_of_the_device.get_or_init(|| said_by_the_panic);
            CompletionCheck::Failed(device_lost_by_a_panic_in_the_wait(recorded))
        }
    }
}

/// 等提交时 `Device::poll` panic 了报的原因：卡当场报出错的那一种，带 panic 的原话。
fn device_lost_by_a_panic_in_the_wait(said_by_the_panic: &str) -> GpuFailureCause {
    GpuFailureCause::UncapturedErrorOrDeviceLost {
        reason: format!("等提交时 wgpu 的 Device::poll panic 了：{said_by_the_panic}"),
    }
}

impl GpuInUse {
    /// 量一次提交的主机 CPU 时间（第 2 条；挂载那一刻，或换设置让开卡时没量的卡参与时）：这条线程的 CPU 钟（不是挂钟）。
    /// 账上不记着还在跑的提交（挂载那一刻刚开的卡没有；换设置时外壳先等它做完，[`BackgroundGpuOverTheCards::use_the_settings_from_the_next_batch`]）。
    /// 交的那一段是 [`COST_PROBE_CHUNK`]，照这张卡此刻剩下的界（上限扣掉已占的，含这个进程里之前泄漏在它上面的）切成几片、几段
    /// （装得下就一段一片），每一次是各段各交一遍：建缓冲、编码两份核、交、等、回读、解码，与挂着时交一段走同一条路
    /// （[`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]）；先空跑一遍，再量 [`COST_MEASUREMENT_SUBMISSIONS`] 遍，
    /// 每遍取各段之和，取中位数（[`measured_cost_summed_over_the_pieces`]）。
    ///
    /// # Errors
    /// 交出错照原样交回；最小的一片装不进这张卡的绑定上限报成起不了（参与的卡显存上限装得下最小的一片，这里只剩绑定上限一样）。
    fn measured_cost_of_one_submission(
        &mut self,
    ) -> Result<HostCpuNanosecondsPerSubmission, GpuFailureCause> {
        let probe_unit_bytes = vec![
            0_u8;
            usize::try_from(COST_PROBE_CHUNK.input_bytes.0)
                .expect("量代价那一条 4096 字节，装得进 usize")
        ];
        let probe = [UnitChecksumToVerify {
            entry_key: (),
            unit_bytes: probe_unit_bytes.as_slice(),
            expected_crc32c: 0,
        }];
        let chunks_of_pieces = COST_PROBE_CHUNK
            .pieces_under_the_bounds(self.budget.room_left_now())
            .map_err(
                |smallest_piece| GpuFailureCause::DeviceOrPipelineDidNotStart {
                    reason: format!(
                        "量代价那一段最小的一片 {} 字节装不进这张卡的绑定上限",
                        smallest_piece.smallest_piece_bytes.0
                    ),
                },
            )?;
        measured_cost_summed_over_the_pieces(
            &chunks_of_pieces,
            COST_MEASUREMENT_SUBMISSIONS,
            |pieces| {
                let piece_units = units_of_the_pieces(&probe, pieces);
                let cpu_time_before = host_cpu_time_of_this_thread();
                self.both_kernel_outcomes_of_a_checksum_chunk(&piece_units)?;
                Ok(host_cpu_time_of_this_thread().saturating_sub(cpu_time_before))
            },
        )
    }

    /// 这张卡上建一份装着 `contents` 的缓冲（[`buffer_with_contents_created_within_the_scope_of_the_caller`]）：只在建一段缓冲的那个显存不足的错误范围里调。
    fn buffer_with_contents(&self, contents: &[u8], usage: wgpu::BufferUsages) -> wgpu::Buffer {
        buffer_with_contents_created_within_the_scope_of_the_caller(&self.device, contents, usage)
    }

    /// 在只接显存不足的错误范围里建这一段的缓冲（`create_the_buffers`）与两份核各自的回读暂存
    /// （[`buffers_and_both_readbacks_created_within_one_out_of_memory_scope`]），这一段的账（`bytes_held_by_this_chunk`）调用方已经记下：
    /// 建得成交回缓冲与回读暂存；建不成（显存不足，第 3 条「卡上此刻空闲的不到这个数」，或等 wgpu 交回那个错误范围超过上限、调用方在停而放弃了那一等）就一次都不交：
    /// 建不成之前已经建好、解开映射的那几份缓冲连同暂存挂在队列的待写里，要等冲掉待写的那一次提交做完才放，所以先交一次空提交冲掉它们、
    /// 在等的上限之内等它做完，做完才放这一段的账、交回建不成的那个错；等不到（等满上限，或调用方在停而放弃了那一等）就把这一段的账记成还在跑的那次提交、
    /// 交回等的那一次的错（`GpuMemoryAccount::settle_the_chunk_whose_buffers_were_not_created`）。
    fn buffers_created_within_the_free_memory<Buffers: AsRef<[wgpu::Buffer]>>(
        &mut self,
        bytes_held_by_this_chunk: GpuMemoryInBytes,
        outcome_bytes: GpuMemoryInBytes,
        create_the_buffers: impl FnOnce(&Self) -> Buffers,
    ) -> Result<BuffersOfAChunk<Buffers, wgpu::Buffer>, GpuFailureCause> {
        let gpu_in_use: &Self = self;
        let created = buffers_and_both_readbacks_created_within_one_out_of_memory_scope(
            &gpu_in_use.device,
            &gpu_in_use.budget,
            bytes_held_by_this_chunk,
            outcome_bytes,
            || create_the_buffers(gpu_in_use),
        );
        let failure_creating_the_buffers = match created {
            Ok(buffers_of_the_chunk) => return Ok(buffers_of_the_chunk),
            Err(failure_creating_the_buffers) => failure_creating_the_buffers,
        };
        let flushing_submission = self.queue.submit(std::iter::empty());
        let clock = SteadyClockOfThisProcess::starting_now();
        let deadline = self
            .budget
            .deadline_of_a_wait_of_the_batch_starting_now(&clock);
        let device = &self.device;
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        let waited_for_the_flush =
            wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
                completion_check_of_the_submission(
                    device,
                    panic_in_a_wait_of_the_device,
                    &flushing_submission,
                    time_left,
                )
            });
        Err(self
            .budget
            .memory_account_mut()
            .settle_the_chunk_whose_buffers_were_not_created(
                bytes_held_by_this_chunk,
                failure_creating_the_buffers,
                flushing_submission,
                waited_for_the_flush,
            ))
    }

    /// 两份核在同一次提交里各算一遍、各自的结果缓冲拷进各自的回读暂存（`readbacks`，按派发的次序，与别的缓冲在同一个错误范围里建好的），
    /// 等做完、等两份映射好，交回两份读回的字节（按派发的次序），
    /// 连同这次提交等完之后在卡上还占不占显存（没等到它做完的，调用方照它把这一段的账留着）。
    /// 一次提交里的几处等共用一个期限，合起来不超过此刻设置给这张卡的等的上限；期限从这张卡的账上取，调用方在停时这几处等放弃
    /// （[`GpuBudgetUnderTheSettings::deadline_of_a_wait_of_the_batch_starting_now`]），放弃在等提交那一处的，这次提交照没等到它做完留在账上。
    fn dispatch_both_kernels_and_read_back(
        &self,
        kernel_dispatches: [KernelDispatch<'_>; 2],
        readbacks: &[wgpu::Buffer; 2],
        workgroups: u32,
        outcome_bytes: GpuMemoryInBytes,
    ) -> (
        Result<[Vec<u8>; 2], GpuFailureCause>,
        SubmissionOfTheChunk<wgpu::SubmissionIndex>,
    ) {
        let device = &self.device;
        let bind_groups: Vec<wgpu::BindGroup> = kernel_dispatches
            .iter()
            .map(|kernel_dispatch| {
                let entries: Vec<wgpu::BindGroupEntry<'_>> = kernel_dispatch
                    .bound_buffers
                    .iter()
                    .map(|(binding, buffer)| wgpu::BindGroupEntry {
                        binding: *binding,
                        resource: buffer.as_entire_binding(),
                    })
                    .collect();
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &kernel_dispatch.pipeline.get_bind_group_layout(0),
                    entries: &entries,
                })
            })
            .collect();
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            for (kernel_dispatch, bind_group) in kernel_dispatches.iter().zip(&bind_groups) {
                pass.set_pipeline(kernel_dispatch.pipeline);
                pass.set_bind_group(0, bind_group, &[]);
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
        }
        for (kernel_dispatch, readback) in kernel_dispatches.iter().zip(readbacks) {
            encoder.copy_buffer_to_buffer(
                kernel_dispatch.outcome_buffer,
                0,
                readback,
                0,
                outcome_bytes.0,
            );
        }
        let submission = self.queue.submit(std::iter::once(encoder.finish()));
        let (mapping_sender, mapping_receiver) = std::sync::mpsc::channel();
        for readback in readbacks {
            let sender_for_this_readback = mapping_sender.clone();
            readback
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |mapping_outcome| {
                    // 送不出去只有一种：收的那一头已经丢了（这一批已经按出错交回），这次映射的结局没人要
                    let _nobody_waits_for_this_mapping_any_more =
                        sender_for_this_readback.send(mapping_outcome);
                });
        }
        drop(mapping_sender);
        let clock = SteadyClockOfThisProcess::starting_now();
        let deadline = self
            .budget
            .deadline_of_a_wait_of_the_batch_starting_now(&clock);
        let polled = wait_for_completion_before_the_deadline(&deadline, &clock, |time_left| {
            completion_check_of_the_submission(
                device,
                &self.panic_in_a_wait_of_the_device,
                &submission,
                time_left,
            )
        });
        let submission_of_the_chunk =
            SubmissionOfTheChunk::after_waiting_for_its_completion(submission, &polled);
        let read_back = self.both_readbacks_once_the_submission_completed(
            polled,
            readbacks,
            &mapping_receiver,
            &deadline,
            &clock,
        );
        (read_back, submission_of_the_chunk)
    }

    /// 等完提交之后：先看 wgpu 的回调记没记下错，再看提交等没等到，再在同一个期限之内等两份回读映射好、读出来。
    fn both_readbacks_once_the_submission_completed(
        &self,
        polled: Result<(), GpuFailureCause>,
        readbacks: &[wgpu::Buffer; 2],
        mapping_receiver: &std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
        deadline: &GpuWaitDeadline,
        clock: &SteadyClockOfThisProcess,
    ) -> Result<[Vec<u8>; 2], GpuFailureCause> {
        if let Some(reason) = fault_taken_from_the_record(&self.fault_reported_by_wgpu) {
            return Err(GpuFailureCause::UncapturedErrorOrDeviceLost { reason });
        }
        polled?;
        for _readback in readbacks {
            wait_for_completion_before_the_deadline(deadline, clock, |time_left| {
                match mapping_receiver.recv_timeout(time_left) {
                    Ok(Ok(())) => CompletionCheck::Completed(()),
                    Ok(Err(error)) => {
                        CompletionCheck::Failed(GpuFailureCause::ReadbackMappingFailed {
                            reason: format!("{error}"),
                        })
                    }
                    Err(RecvTimeoutError::Timeout) => CompletionCheck::NotYetComplete,
                    Err(RecvTimeoutError::Disconnected) => {
                        CompletionCheck::Failed(GpuFailureCause::ReadbackMappingFailed {
                            reason: "等完了提交，映射的回调还没送到就都丢了".to_string(),
                        })
                    }
                }
            })?;
        }
        let [first_readback, second_readback] = readbacks;
        Ok([
            bytes_read_back_from(first_readback)?,
            bytes_read_back_from(second_readback)?,
        ])
    }
}

impl CardOfTheBackgroundGpu for GpuInUse {
    type Submission = wgpu::SubmissionIndex;

    fn discrete_gpu(&self) -> &DiscreteGpuOnThisHost {
        &self.discrete_gpu
    }

    fn budget(&self) -> &GpuBudgetUnderTheSettings<wgpu::SubmissionIndex> {
        &self.budget
    }

    fn budget_mut(&mut self) -> &mut GpuBudgetUnderTheSettings<wgpu::SubmissionIndex> {
        &mut self.budget
    }

    fn measured_cost(&self) -> Option<HostCpuNanosecondsPerSubmission> {
        self.measured_cost
    }

    fn measure_the_cost_of_one_submission(&mut self) -> Result<(), GpuFailureCause> {
        self.measured_cost = Some(self.measured_cost_of_one_submission()?);
        Ok(())
    }

    /// 在这张卡上交这一批之前调：前一次等超过上限、还在跑的那次提交先在此刻设置给的等的上限之内等它做完、放掉它的账（第 3 条；等不到交回出错，
    /// 不越过上限去交），交回此刻剩下的界（`GpuBudgetUnderTheSettings`）。
    fn room_once_the_submission_running_past_its_wait_completed(
        &mut self,
    ) -> Result<ChunkBoundsOfAGpu, GpuFailureCause> {
        let clock = SteadyClockOfThisProcess::starting_now();
        let device = &self.device;
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        self.budget
            .room_once_the_submission_running_past_its_wait_completed(
                &clock,
                |submission, time_left| {
                    completion_check_of_the_submission(
                        device,
                        panic_in_a_wait_of_the_device,
                        submission,
                        time_left,
                    )
                },
            )
    }

    /// 重交之前调：前一次等超过上限、还在跑的那次提交在此刻设置给的等的上限之内等它做完、放掉它的账。
    fn wait_for_the_timed_out_submission(&mut self) -> Result<(), GpuFailureCause> {
        let clock = SteadyClockOfThisProcess::starting_now();
        let device = &self.device;
        let panic_in_a_wait_of_the_device = &self.panic_in_a_wait_of_the_device;
        self.budget.wait_for_the_submission_running_past_its_wait(
            &clock,
            |submission, time_left| {
                completion_check_of_the_submission(
                    device,
                    panic_in_a_wait_of_the_device,
                    submission,
                    time_left,
                )
            },
        )
    }

    /// 校验和核对的一段（切批切出来的、装得下这张卡的那几片）交给这张卡：两份核在同一次提交里各算一遍，交回每片两份的结果，按段里的次序。
    /// 本包这一段占的缓冲先记进这张卡的账（超上限断言），那次提交做完放掉；没等到它做完的留着（`GpuMemoryAccount::settle_the_chunk_after_its_submission`）；
    /// 建缓冲显存不足的不交，已建的冲掉、做完了才放（[`GpuInUse::buffers_created_within_the_free_memory`]）。
    fn both_kernel_outcomes_of_a_checksum_chunk(
        &mut self,
        units: &[UnitChecksumToVerify<'_, ()>],
    ) -> Result<Vec<(ChecksumKernelOutcome, ChecksumKernelOutcome)>, GpuFailureCause> {
        assert!(!units.is_empty(), "切批不交空段");
        let packed = packed_checksum_chunk(units);
        let entry_count = packed.entry_count();
        let entry_count_in_words =
            u32::try_from(entry_count).expect("一段的条数不超过一次派发的上限，装得进 u32");
        let entry_count_of_the_chunk = EntryCountOfAChunk(u64::from(entry_count_in_words));
        let outcome_bytes = entry_count_of_the_chunk.bytes_of_the_entries(OUTCOME_BYTES_PER_ENTRY);
        let mut parameter_bytes =
            vec![0_u8; usize::try_from(PARAMETER_BUFFER_BYTES.0).expect("16 字节")];
        parameter_bytes[..4].copy_from_slice(&entry_count_in_words.to_le_bytes());
        let descriptor_bytes = packed.descriptor_bytes();
        let outcome_fill = outcome_buffer_before_the_kernel_writes(entry_count);

        let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(
                PARAMETER_BUFFER_BYTES
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&packed.input_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes)),
            )
            .plus(
                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
            );
        assert_eq!(
            bytes_held_by_this_chunk,
            CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
                InputLengthOnTheHostInBytes::of_this_many_bytes(
                    units
                        .iter()
                        .map(|unit| unit.unit_bytes.len())
                        .sum::<usize>()
                ),
                entry_count_of_the_chunk,
            ),
            "这一段真建的缓冲与切批按的占用一致"
        );
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_this_chunk);
        let BuffersOfAChunk {
            buffers:
                [parameter_buffer, input_buffer, descriptor_buffer, table_driven_outcomes, bitwise_outcomes],
            readbacks,
        } = self.buffers_created_within_the_free_memory(
            bytes_held_by_this_chunk,
            outcome_bytes,
            |gpu| {
                [
                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                    gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                ]
            },
        )?;
        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
            [
                KernelDispatch {
                    pipeline: &self.table_driven_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &table_driven_outcomes),
                    ],
                    outcome_buffer: &table_driven_outcomes,
                },
                KernelDispatch {
                    pipeline: &self.bitwise_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &bitwise_outcomes),
                    ],
                    outcome_buffer: &bitwise_outcomes,
                },
            ],
            &readbacks,
            singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_workgroups_for_entries(
                entry_count_in_words,
                INVOCATIONS_PER_WORKGROUP,
            ),
            outcome_bytes,
        );
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [table_driven_bytes, bitwise_bytes] = read_back?;
        let table_driven = decoded_kernel_outcomes(&table_driven_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let bitwise = decoded_kernel_outcomes(&bitwise_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(table_driven.into_iter().zip(bitwise).collect())
    }

    /// 区域扫描的一段（几片）交给这张卡：两份核在同一次提交里各算一遍。`mapping_tables` 是这一段交的那一片映射：从单元头起的那几片的段交它、
    /// 交回每条两份的分类；单元头一片之后那几片的段是 `None`，交一份空映射、交回每片两份算完的寄存器（`region_record_kernels` 文件头「一个单元切成几片」）。
    /// 按段里的次序。账同 [`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]。
    fn both_kernel_outcomes_of_a_region_record_chunk(
        &mut self,
        pieces: &[RegionRecordPieceForTheKernels<'_>],
        mapping_tables: Option<&MappingTablesForTheKernels>,
    ) -> Result<BothKernelOutcomesOfARegionRecordChunk, GpuFailureCause> {
        assert!(!pieces.is_empty(), "切批不交空段");
        let is_a_chunk_of_pieces_after_the_heads = mapping_tables.is_none();
        assert!(
            pieces.iter().all(|piece| matches!(
                piece,
                RegionRecordPieceForTheKernels::PieceAfterTheHead { .. }
            ) == is_a_chunk_of_pieces_after_the_heads),
            "一段里的片同一种：交映射片的全是从单元头起的，不交的全是头一片之后的"
        );
        let empty_mapping_tables;
        let mapping_tables = match mapping_tables {
            Some(mapping_tables) => mapping_tables,
            None => {
                empty_mapping_tables = MappingTablesForTheKernels::of_the_mapping_entries(&[])
                    .expect("空映射没有长过核装得下的 key");
                &empty_mapping_tables
            }
        };
        let packed = packed_region_record_chunk(pieces);
        let record_count = u32::try_from(packed.record_count())
            .expect("一段的条数不超过一次派发的上限，装得进 u32");
        let record_count_of_the_chunk = EntryCountOfAChunk(u64::from(record_count));
        let outcome_bytes =
            record_count_of_the_chunk.bytes_of_the_entries(REGION_RECORD_OUTCOME_BYTES);
        let parameter_bytes = region_record_parameter_bytes(record_count, mapping_tables);
        let descriptor_bytes = little_endian_bytes_of_the_words(&packed.descriptor_words);
        let entry_bytes = little_endian_bytes_of_the_words(&mapping_tables.entry_words);
        let location_item_bytes =
            little_endian_bytes_of_the_words(&mapping_tables.location_item_words);
        let key_sorted_position_bytes =
            little_endian_bytes_of_the_words(&mapping_tables.key_sorted_entry_positions);
        let outcome_fill =
            region_record_outcome_buffer_before_the_kernel_writes(packed.record_count());

        let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(
                bytes_on_the_gpu_of_a_buffer_with(&parameter_bytes)
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&packed.input_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&descriptor_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&entry_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&location_item_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(
                        &key_sorted_position_bytes,
                    )),
            )
            .plus(
                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
            );
        let total_unit_bytes: usize = pieces
            .iter()
            .map(|piece| piece.bytes().map_or(0, <[u8]>::len))
            .sum();
        assert_eq!(
            bytes_held_by_this_chunk,
            region_record_chunk_footprint(mapping_tables).bytes_held_by_a_chunk(
                InputLengthOnTheHostInBytes::of_this_many_bytes(total_unit_bytes),
                record_count_of_the_chunk,
            ),
            "这一段真建的缓冲与切批按的占用一致"
        );
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_this_chunk);
        let BuffersOfAChunk {
            buffers:
                [parameter_buffer, input_buffer, descriptor_buffer, entry_buffer, location_item_buffer, key_sorted_position_buffer, walking_outcomes, searching_outcomes],
            readbacks,
        } = self.buffers_created_within_the_free_memory(
            bytes_held_by_this_chunk,
            outcome_bytes,
            |gpu| {
                [
                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                    gpu.buffer_with_contents(&packed.input_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&descriptor_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&entry_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&location_item_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(
                        &key_sorted_position_bytes,
                        wgpu::BufferUsages::STORAGE,
                    ),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                ]
            },
        )?;
        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
            [
                KernelDispatch {
                    pipeline: &self.walking_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &entry_buffer),
                        (6, &walking_outcomes),
                    ],
                    outcome_buffer: &walking_outcomes,
                },
                KernelDispatch {
                    pipeline: &self.searching_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &input_buffer),
                        (2, &descriptor_buffer),
                        (3, &entry_buffer),
                        (4, &location_item_buffer),
                        (5, &key_sorted_position_buffer),
                        (6, &searching_outcomes),
                    ],
                    outcome_buffer: &searching_outcomes,
                },
            ],
            &readbacks,
            singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_workgroups_for_entries(
                record_count,
                REGION_RECORD_INVOCATIONS_PER_WORKGROUP,
            ),
            outcome_bytes,
        );
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [walking_bytes, searching_bytes] = read_back?;
        if is_a_chunk_of_pieces_after_the_heads {
            let walking = decoded_registers_after_the_pieces(&walking_bytes)
                .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
            let searching = decoded_registers_after_the_pieces(&searching_bytes)
                .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
            return Ok(
                BothKernelOutcomesOfARegionRecordChunk::RegistersAfterThePieces(
                    walking
                        .into_iter()
                        .zip(searching)
                        .map(
                            |(walking_kernel, searching_kernel)| CrcRegistersOfTheTwoKernels {
                                walking_kernel,
                                searching_kernel,
                            },
                        )
                        .collect(),
                ),
            );
        }
        let walking = decoded_region_record_outcomes(&walking_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let searching = decoded_region_record_outcomes(&searching_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(BothKernelOutcomesOfARegionRecordChunk::Classifications(
            walking.into_iter().zip(searching).collect(),
        ))
    }

    /// 选段打分的一段（切批切出来的、装得下这张卡的那几个段）交给这张卡：两份核在同一次提交里各数一遍，交回每段两份的活槽数，按段里的次序。
    /// 这一片的记录表随每一段交一份。账同 [`CardOfTheBackgroundGpu::both_kernel_outcomes_of_a_checksum_chunk`]。
    fn both_kernel_live_slots_of_a_chunk(
        &mut self,
        segments: &[ClusterSegmentEntryKey],
        record_tables: &AllocationRecordTablesForTheKernels,
        unit_area: UnitAreaOfOneDevice,
    ) -> Result<Vec<(u64, u64)>, GpuFailureCause> {
        assert!(!segments.is_empty(), "切批不交空段");
        let segment_count =
            u32::try_from(segments.len()).expect("一段的段数不超过一次派发的上限，装得进 u32");
        let segment_count_of_the_chunk = EntryCountOfAChunk(u64::from(segment_count));
        let outcome_bytes =
            segment_count_of_the_chunk.bytes_of_the_entries(LIVE_SLOT_OUTCOME_BYTES);
        let parameter_bytes = live_slot_parameter_bytes(segment_count, record_tables, unit_area);
        let segment_bound_bytes = little_endian_bytes_of_the_words(&segment_bound_words(segments));
        let record_bytes = little_endian_bytes_of_the_words(&record_tables.record_words);
        let sorted_position_bytes =
            little_endian_bytes_of_the_words(&record_tables.record_positions_by_device_and_slot);
        let outcome_fill = live_slot_outcome_buffer_before_the_kernel_writes(segments.len());

        let bytes_held_by_this_chunk = COPIES_ON_THE_GPU_OF_A_BUFFER_MAPPED_AT_CREATION
            .bytes_of_the_copies(
                bytes_on_the_gpu_of_a_buffer_with(&parameter_bytes)
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&segment_bound_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&record_bytes))
                    .plus(bytes_on_the_gpu_of_a_buffer_with(&sorted_position_bytes)),
            )
            .plus(
                COPIES_ON_THE_GPU_OF_AN_OUTCOME_OF_BOTH_KERNELS.bytes_of_the_copies(outcome_bytes),
            );
        assert_eq!(
            bytes_held_by_this_chunk,
            live_slot_chunk_footprint(record_tables).bytes_held_by_a_chunk(
                segment_count_of_the_chunk.input_bytes_of_the_entries(SEGMENT_BOUND_BYTES),
                segment_count_of_the_chunk,
            ),
            "这一段真建的缓冲与切批按的占用一致"
        );
        self.budget
            .memory_account_mut()
            .hold(bytes_held_by_this_chunk);
        let BuffersOfAChunk {
            buffers:
                [parameter_buffer, segment_bound_buffer, record_buffer, sorted_position_buffer, walking_outcomes, searching_outcomes],
            readbacks,
        } = self.buffers_created_within_the_free_memory(
            bytes_held_by_this_chunk,
            outcome_bytes,
            |gpu| {
                [
                    gpu.buffer_with_contents(&parameter_bytes, wgpu::BufferUsages::UNIFORM),
                    gpu.buffer_with_contents(&segment_bound_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&record_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&sorted_position_bytes, wgpu::BufferUsages::STORAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                    gpu.buffer_with_contents(&outcome_fill, OUTCOME_BUFFER_USAGE),
                ]
            },
        )?;
        let (read_back, submission_of_the_chunk) = self.dispatch_both_kernels_and_read_back(
            [
                KernelDispatch {
                    pipeline: &self.live_slot_walking_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &segment_bound_buffer),
                        (2, &record_buffer),
                        (4, &walking_outcomes),
                    ],
                    outcome_buffer: &walking_outcomes,
                },
                KernelDispatch {
                    pipeline: &self.live_slot_searching_pipeline,
                    bound_buffers: vec![
                        (0, &parameter_buffer),
                        (1, &segment_bound_buffer),
                        (2, &record_buffer),
                        (3, &sorted_position_buffer),
                        (4, &searching_outcomes),
                    ],
                    outcome_buffer: &searching_outcomes,
                },
            ],
            &readbacks,
            singlefs_replay_prefix::gpu_gc_capacity_proofs::gpu_workgroups_for_entries(
                segment_count,
                LIVE_SLOT_INVOCATIONS_PER_WORKGROUP,
            ),
            outcome_bytes,
        );
        self.budget
            .memory_account_mut()
            .settle_the_chunk_after_its_submission(
                bytes_held_by_this_chunk,
                submission_of_the_chunk,
            );
        let [walking_bytes, searching_bytes] = read_back?;
        let walking = decoded_live_slot_outcomes(&walking_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        let searching = decoded_live_slot_outcomes(&searching_bytes)
            .map_err(GpuFailureCause::KernelLeftAnUnrecognizedVerdictWord)?;
        Ok(walking.into_iter().zip(searching).collect())
    }
}
