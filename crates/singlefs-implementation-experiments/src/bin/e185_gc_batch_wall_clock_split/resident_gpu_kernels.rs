use super::*;
use singlefs_verification::gpu_unit_checks::GpuCard;

/// 核一：照 `crates/singlefs-verification/src/gpu_unit_checks.rs` 那份逐位算法另抄一份（那份源码是私有常量，装置读不到；行为由停机 J-S1
/// 与 crates 的 `GpuCard::crc32c_of_ranges` 逐条对拍）。每个单元一条线程、工作组 64。
const KERNEL_ONE_SOURCE: &str = r#"
struct Params { count: u32, reserved0: u32, reserved1: u32, reserved2: u32, };
struct ResultEntry { crc: u32, local_offset: u32, };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> slot_words: array<u32>;
@group(0) @binding(2) var<storage, read> slot_lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> results: array<ResultEntry>;
const CRC32C_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const CRC32C_INITIAL_VALUE: u32 = 0xFFFFFFFFu;
const WORDS_PER_SLOT: u32 = 8192u;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local_offset = global_id.x;
    if (local_offset >= params.count) { return; }
    let length_in_bytes = slot_lengths[local_offset];
    let base_word = local_offset * WORDS_PER_SLOT;
    var crc_register: u32 = CRC32C_INITIAL_VALUE;
    var byte_index: u32 = 0u;
    loop {
        if (byte_index >= length_in_bytes) { break; }
        let word = slot_words[base_word + byte_index / 4u];
        let current_byte = (word >> (8u * (byte_index % 4u))) & 0xFFu;
        crc_register = crc_register ^ current_byte;
        var bit_index: u32 = 0u;
        loop {
            if (bit_index >= 8u) { break; }
            if ((crc_register & 1u) == 1u) {
                crc_register = (crc_register >> 1u) ^ CRC32C_REFLECTED_POLYNOMIAL;
            } else {
                crc_register = crc_register >> 1u;
            }
            bit_index = bit_index + 1u;
        }
        byte_index = byte_index + 1u;
    }
    results[local_offset].crc = crc_register ^ CRC32C_INITIAL_VALUE;
    results[local_offset].local_offset = local_offset;
}
"#;

/// 核二：装置新写，按字节查 256 项表，表在工作组共享内存里（工作组里 64 条线程每条算表里 4 项，栅栏之后再算），不共用核一的核函数
/// （D26 已定项 13 第 8 条）。每个单元一条线程、工作组 64。
const KERNEL_TWO_SOURCE: &str = r#"
struct DispatchParameters { unit_count: u32, unused_first: u32, unused_second: u32, unused_third: u32, };
struct UnitResult { checksum: u32, unit_index: u32, };
@group(0) @binding(0) var<uniform> dispatch_parameters: DispatchParameters;
@group(0) @binding(1) var<storage, read> unit_words: array<u32>;
@group(0) @binding(2) var<storage, read> unit_lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> unit_results: array<UnitResult>;
const REVERSED_CASTAGNOLI_POLYNOMIAL: u32 = 0x82F63B78u;
const WORDS_PER_UNIT_SLOT: u32 = 8192u;
const TABLE_ENTRIES_PER_LANE: u32 = 4u;
var<workgroup> byte_table: array<u32, 256>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>, @builtin(local_invocation_index) lane: u32) {
    for (var entry_of_the_lane: u32 = 0u; entry_of_the_lane < TABLE_ENTRIES_PER_LANE; entry_of_the_lane = entry_of_the_lane + 1u) {
        let table_index = lane * TABLE_ENTRIES_PER_LANE + entry_of_the_lane;
        var table_value: u32 = table_index;
        for (var shift_round: u32 = 0u; shift_round < 8u; shift_round = shift_round + 1u) {
            table_value = select(table_value >> 1u, (table_value >> 1u) ^ REVERSED_CASTAGNOLI_POLYNOMIAL, (table_value & 1u) == 1u);
        }
        byte_table[table_index] = table_value;
    }
    workgroupBarrier();
    let unit_index = invocation.x;
    if (unit_index >= dispatch_parameters.unit_count) { return; }
    let unit_length = unit_lengths[unit_index];
    let first_word = unit_index * WORDS_PER_UNIT_SLOT;
    var running_checksum: u32 = 0xFFFFFFFFu;
    for (var position: u32 = 0u; position < unit_length; position = position + 1u) {
        let packed = unit_words[first_word + position / 4u];
        let unit_byte = (packed >> (8u * (position % 4u))) & 0xFFu;
        running_checksum = byte_table[(running_checksum ^ unit_byte) & 0xFFu] ^ (running_checksum >> 8u);
    }
    unit_results[unit_index].checksum = running_checksum ^ 0xFFFFFFFFu;
    unit_results[unit_index].unit_index = unit_index;
}
"#;

/// 工作组大小（两核都是 64）。
const WORKGROUP_SIZE: u64 = 64;

/// 一张卡上常驻的两核与缓冲：管线、缓冲（按这一段最大一档开）、绑定组都在计时窗口外建一次，每次提交只做 ①–⑦（登记第一节「常驻写法」）。
/// 共用形态两核读核一那一块输入与长度表；分开形态各读各的。
pub(super) struct ResidentKernels {
    device: wgpu::Device,
    queue: wgpu::Queue,
    kernel_one: wgpu::ComputePipeline,
    kernel_two: wgpu::ComputePipeline,
    parameters: wgpu::Buffer,
    input_of_kernel_one: wgpu::Buffer,
    input_of_kernel_two: wgpu::Buffer,
    lengths_of_kernel_one: wgpu::Buffer,
    lengths_of_kernel_two: wgpu::Buffer,
    results_of_kernel_one: wgpu::Buffer,
    results_of_kernel_two: wgpu::Buffer,
    readback: wgpu::Buffer,
    kernel_one_bindings: wgpu::BindGroup,
    kernel_two_shared_bindings: wgpu::BindGroup,
    kernel_two_split_bindings: wgpu::BindGroup,
    pub(super) capacity_units: u64,
}

/// 回读映射登记之后、等完之前攥着的东西：这一次提交的序号与回读回调送结果的那一头。
pub(super) struct PendingReadback {
    submission: wgpu::SubmissionIndex,
    receiver: std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
    mapped_bytes: u64,
}

fn storage_buffer(device: &wgpu::Device, bytes: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: bytes,
        usage,
        mapped_at_creation: false,
    })
}

fn pipeline_of(device: &wgpu::Device, label: &str, source: &str) -> wgpu::ComputePipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: None,
        module: &module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn bindings_of(
    device: &wgpu::Device,
    pipeline: &wgpu::ComputePipeline,
    buffers: [&wgpu::Buffer; 4],
) -> wgpu::BindGroup {
    let layout = pipeline.get_bind_group_layout(0);
    let entries: Vec<wgpu::BindGroupEntry<'_>> = buffers
        .iter()
        .zip(0_u32..)
        .map(|(buffer, binding)| wgpu::BindGroupEntry {
            binding,
            resource: buffer.as_entire_binding(),
        })
        .collect();
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &layout,
        entries: &entries,
    })
}

impl ResidentKernels {
    /// 在这张卡上编两核、按 `capacity_units` 个 32 KiB 槽开两份输入、两份长度表、两份结果与一块回读缓冲，建三个绑定组（计时窗口外）。
    pub(super) fn start(card: &GpuCard, capacity_units: u64) -> ResidentKernels {
        let device = card.device().clone();
        let queue = card.queue().clone();
        let kernel_one = pipeline_of(&device, "e185_kernel_one", KERNEL_ONE_SOURCE);
        let kernel_two = pipeline_of(&device, "e185_kernel_two", KERNEL_TWO_SOURCE);
        let storage_in = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        let storage_out = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC;
        let input_bytes = capacity_units * LOCAL_GPU_SLOT_BYTES;
        let length_bytes = capacity_units * 4;
        let result_bytes = capacity_units * LOCAL_GPU_RESULT_ENTRY_BYTES;
        let parameters = storage_buffer(
            &device,
            LOCAL_GPU_PARAMETER_BYTES,
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let input_of_kernel_one = storage_buffer(&device, input_bytes, storage_in);
        let input_of_kernel_two = storage_buffer(&device, input_bytes, storage_in);
        let lengths_of_kernel_one = storage_buffer(&device, length_bytes, storage_in);
        let lengths_of_kernel_two = storage_buffer(&device, length_bytes, storage_in);
        let results_of_kernel_one = storage_buffer(&device, result_bytes, storage_out);
        let results_of_kernel_two = storage_buffer(&device, result_bytes, storage_out);
        let readback = storage_buffer(
            &device,
            2 * result_bytes,
            wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        );
        let kernel_one_bindings = bindings_of(
            &device,
            &kernel_one,
            [
                &parameters,
                &input_of_kernel_one,
                &lengths_of_kernel_one,
                &results_of_kernel_one,
            ],
        );
        let kernel_two_shared_bindings = bindings_of(
            &device,
            &kernel_two,
            [
                &parameters,
                &input_of_kernel_one,
                &lengths_of_kernel_one,
                &results_of_kernel_two,
            ],
        );
        let kernel_two_split_bindings = bindings_of(
            &device,
            &kernel_two,
            [
                &parameters,
                &input_of_kernel_two,
                &lengths_of_kernel_two,
                &results_of_kernel_two,
            ],
        );
        ResidentKernels {
            device,
            queue,
            kernel_one,
            kernel_two,
            parameters,
            input_of_kernel_one,
            input_of_kernel_two,
            lengths_of_kernel_one,
            lengths_of_kernel_two,
            results_of_kernel_one,
            results_of_kernel_two,
            readback,
            kernel_one_bindings,
            kernel_two_shared_bindings,
            kernel_two_split_bindings,
            capacity_units,
        }
    }

    /// ① 写输入：单元字节（[`write_the_unit_bytes`]，交回写进卡的单元字节）、长度表（分开形态各写一份）、派活参数。
    /// `kernel_two_patches` 是分开形态里核二那一份输入在单元字节写完之后再改的几段（偏移、字节；第二段 P5 用，第一段不给）。
    pub(super) fn write_the_inputs(
        &self,
        form: InputForm,
        unit_bytes: &[u8],
        lengths: &[u32],
        kernel_two_patches: &[(u64, Vec<u8>)],
    ) -> u64 {
        let units = u64::try_from(lengths.len()).expect("单元数");
        assert!(
            units <= self.capacity_units,
            "一次提交 {units} 个单元，超过常驻缓冲的 {}",
            self.capacity_units
        );
        let written = write_the_unit_bytes(form, unit_bytes, |buffer, bytes| {
            let target = match buffer {
                InputBufferOfTheKernel::KernelOne => &self.input_of_kernel_one,
                InputBufferOfTheKernel::KernelTwo => &self.input_of_kernel_two,
            };
            self.queue.write_buffer(target, 0, bytes);
        });
        for (offset, bytes) in kernel_two_patches {
            self.queue
                .write_buffer(&self.input_of_kernel_two, *offset, bytes);
        }
        let length_bytes: Vec<u8> = lengths
            .iter()
            .flat_map(|length| length.to_le_bytes())
            .collect();
        self.queue
            .write_buffer(&self.lengths_of_kernel_one, 0, &length_bytes);
        if form == InputForm::Split {
            self.queue
                .write_buffer(&self.lengths_of_kernel_two, 0, &length_bytes);
        }
        let count = u32::try_from(units).expect("单元数装得进 u32");
        let parameter_bytes: Vec<u8> = [count, 0, 0, 0]
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect();
        self.queue
            .write_buffer(&self.parameters, 0, &parameter_bytes);
        written
    }

    /// ② 编一个命令缓冲：核一、核二各派一次（每个单元一条线程；`kernels` 是只派核一时核二不派），两份结果拷进同一块回读缓冲
    /// （前 n 条核一、后 n 条核二）；③ 提交一次。
    pub(super) fn encode_and_submit(
        &self,
        form: InputForm,
        units: u64,
        kernels: KernelsToDispatch,
    ) -> wgpu::SubmissionIndex {
        let workgroups = u32::try_from(units.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32");
        let result_bytes = units * LOCAL_GPU_RESULT_ENTRY_BYTES;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.kernel_one);
            pass.set_bind_group(0, &self.kernel_one_bindings, &[]);
            pass.dispatch_workgroups(workgroups, 1, 1);
            match kernels {
                KernelsToDispatch::Both => {
                    pass.set_pipeline(&self.kernel_two);
                    let kernel_two_bindings = match form {
                        InputForm::Shared => &self.kernel_two_shared_bindings,
                        InputForm::Split => &self.kernel_two_split_bindings,
                    };
                    pass.set_bind_group(0, kernel_two_bindings, &[]);
                    pass.dispatch_workgroups(workgroups, 1, 1);
                }
                KernelsToDispatch::KernelOneOnly => {}
            }
        }
        encoder.copy_buffer_to_buffer(
            &self.results_of_kernel_one,
            0,
            &self.readback,
            0,
            result_bytes,
        );
        encoder.copy_buffer_to_buffer(
            &self.results_of_kernel_two,
            0,
            &self.readback,
            result_bytes,
            result_bytes,
        );
        self.queue.submit(std::iter::once(encoder.finish()))
    }

    /// ④ 回读映射登记（前 2n 条）。
    pub(super) fn register_the_readback(
        &self,
        submission: wgpu::SubmissionIndex,
        units: u64,
    ) -> PendingReadback {
        let mapped_bytes = 2 * units * LOCAL_GPU_RESULT_ENTRY_BYTES;
        let (sender, receiver) = std::sync::mpsc::channel();
        self.readback
            .slice(..mapped_bytes)
            .map_async(wgpu::MapMode::Read, move |result| {
                // 送不出去只有一种：收的那一头已经丢了，这次的结果没人要
                let _nobody_waits_any_more = sender.send(result);
            });
        PendingReadback {
            submission,
            receiver,
            mapped_bytes,
        }
    }

    /// ⑤ 等（登记第一节「等法」）：阻塞等只等这一次提交；睡眠等循环 poll、没到就睡。交回 poll 了几次（阻塞等交 0）。
    ///
    /// # Panics
    /// 等不成（设备丢了）或回读映射不成：这一段停下，不带着错的结果往下走。
    pub(super) fn wait(&self, pending: &PendingReadback, method: WaitMethod) -> u64 {
        let polls = match method.sleep_interval() {
            None => {
                let polled = self.device.poll(wgpu::PollType::Wait {
                    submission_index: Some(pending.submission.clone()),
                    timeout: None,
                });
                assert!(polled.is_ok(), "阻塞等没等成：{polled:?}");
                0
            }
            Some(interval) => wait_by_sleeping(
                || {
                    let polled = self.device.poll(wgpu::PollType::Poll);
                    assert!(polled.is_ok(), "睡眠等里 poll 不成：{polled:?}");
                    // 回调送到了（映射成不成留给 ⑥ 取映射时判）：收到什么都算到了，只有「还空着」算没到
                    !matches!(
                        pending.receiver.try_recv(),
                        Err(std::sync::mpsc::TryRecvError::Empty)
                    )
                },
                std::thread::sleep,
                interval,
            ),
        };
        polls
    }

    /// 单测用的一次长活：只派核一 `passes` 遍（不拷回读缓冲），当「同一个设备上另交的一次长的活」（第九节 V99 那条带卡单测）。
    #[cfg(test)]
    pub(super) fn submit_a_long_job(&self, units: u64, passes: u32) -> wgpu::SubmissionIndex {
        let workgroups = u32::try_from(units.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.kernel_one);
            pass.set_bind_group(0, &self.kernel_one_bindings, &[]);
            for _pass in 0..passes {
                pass.dispatch_workgroups(workgroups, 1, 1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()))
    }

    /// 单测用：阻塞到这一次提交做完。
    #[cfg(test)]
    pub(super) fn block_until_done(&self, submission: wgpu::SubmissionIndex) {
        let polled = self.device.poll(wgpu::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        });
        assert!(polled.is_ok(), "等长活没等成：{polled:?}");
    }

    /// ⑥ 在映射着的回读字节上做 `read`（比对在映射上做，不先拷一份）；⑦ 解映射。阻塞等之后回调已经送到，先收它。
    ///
    /// # Panics
    /// 回读映射不成。
    pub(super) fn read_then_unmap<Answer>(
        &self,
        pending: &PendingReadback,
        read: impl FnOnce(&[u8]) -> Answer,
    ) -> Answer {
        let _mapping_result_already_taken_by_a_sleeping_wait = pending.receiver.try_recv();
        let answer = {
            let mapped = self
                .readback
                .slice(..pending.mapped_bytes)
                .get_mapped_range()
                .expect("回读缓冲映射着");
            read(&mapped)
        };
        self.readback.unmap();
        answer
    }
}

/// 常驻写法的一次提交（[`SubmissionSteps`] 的真卡那一份）：`unit_bytes` 是 n 个 32 KiB 槽连着放的字节，`lengths` 每单元一个长度，
/// `expected` 是计时窗口外 CPU 算好的期望值；`kernel_two_flip` 是 P3 在读回之后、比对之前把核二哪一条异或 1。
pub(super) struct ResidentSubmission<'kernels> {
    pub(super) kernels: &'kernels ResidentKernels,
    pub(super) form: InputForm,
    pub(super) wait_method: WaitMethod,
    pub(super) unit_bytes: &'kernels [u8],
    pub(super) lengths: &'kernels [u32],
    pub(super) expected: &'kernels [u32],
    pub(super) kernel_two_flip: Option<usize>,
    pub(super) submission: Option<wgpu::SubmissionIndex>,
    pub(super) pending: Option<PendingReadback>,
}

impl ResidentSubmission<'_> {
    fn units(&self) -> u64 {
        u64::try_from(self.lengths.len()).expect("单元数")
    }
}

impl SubmissionSteps for ResidentSubmission<'_> {
    fn prepare(&mut self) -> u64 {
        self.kernels
            .write_the_inputs(self.form, self.unit_bytes, self.lengths, &[])
    }

    fn encode_and_submit(&mut self) {
        self.submission = Some(self.kernels.encode_and_submit(
            self.form,
            self.units(),
            KernelsToDispatch::Both,
        ));
    }

    fn register_the_readback(&mut self) {
        let submission = self.submission.take().expect("③ 提交在 ④ 之前");
        self.pending = Some(self.kernels.register_the_readback(submission, self.units()));
    }

    fn wait(&mut self) -> Option<u64> {
        let pending = self.pending.as_ref().expect("④ 登记在 ⑤ 之前");
        Some(self.kernels.wait(pending, self.wait_method))
    }

    fn read_and_compare(&mut self) -> SubmissionComparison {
        let pending = self.pending.take().expect("④ 登记在 ⑥ 之前");
        let (expected, kernel_two_flip) = (self.expected, self.kernel_two_flip);
        self.kernels.read_then_unmap(&pending, |readback| {
            compare_the_readback(readback, expected, kernel_two_flip)
        })
    }

    fn unmap(&mut self) {
        // ⑦ 在 read_then_unmap 里随 ⑥ 做完（映射的视图要先丢掉才能解映射）；计时区间照样罩着它
    }
}

/// 今天的写法（真实基线，登记第五节「真实基线」）：`GpuCard::crc32c_of_ranges` 调一次（一份核、每次建五个缓冲与绑定组、阻塞等）。
/// 那一次调用整个算在 ① 里，⑤ 分不出来（`wait` 交 `None`）；只有一份核，比对只比它与期望值。
pub(super) struct TodaySubmission<'card> {
    pub(super) card: &'card GpuCard,
    pub(super) ranges: Vec<&'card [u8]>,
    pub(super) expected: &'card [u32],
    pub(super) results: Vec<u32>,
}

impl SubmissionSteps for TodaySubmission<'_> {
    fn prepare(&mut self) -> u64 {
        self.results = self.card.crc32c_of_ranges(&self.ranges);
        u64::try_from(self.ranges.len()).expect("段数") * LOCAL_GPU_SLOT_BYTES
    }

    fn encode_and_submit(&mut self) {}

    fn register_the_readback(&mut self) {}

    fn wait(&mut self) -> Option<u64> {
        None
    }

    fn read_and_compare(&mut self) -> SubmissionComparison {
        let differing = self
            .results
            .iter()
            .zip(self.expected)
            .filter(|(result, expected)| result != expected)
            .count();
        SubmissionComparison {
            confirmed: u64::try_from(self.results.len()).expect("条数"),
            uncertain_entries: Vec::new(),
            kernel_one_differs_from_the_expected: u64::try_from(differing).expect("条数"),
            kernel_two_differs_from_the_expected: 0,
            are_the_sequence_numbers_holding: self.results.len() == self.expected.len(),
        }
    }

    fn unmap(&mut self) {}
}

/// 一张卡上跑一次不计时的提交（P2 的 9 字节、停机 J-S1 的对拍、第二段的每批与重算）：写入、派 `kernels`、提交、阻塞等、读回，
/// 交回每单元两核的原样结果（没派的那一核是 `None`）与序号核没核过。
pub(super) fn untimed_results(
    kernels: &ResidentKernels,
    form: InputForm,
    unit_bytes: &[u8],
    lengths: &[u32],
    kernel_two_patches: &[(u64, Vec<u8>)],
    dispatched: KernelsToDispatch,
) -> (Vec<KernelResultsOfAUnit>, bool) {
    let units = u64::try_from(lengths.len()).expect("单元数");
    let _written = kernels.write_the_inputs(form, unit_bytes, lengths, kernel_two_patches);
    let submission = kernels.encode_and_submit(form, units, dispatched);
    let pending = kernels.register_the_readback(submission, units);
    let _polls = kernels.wait(&pending, WaitMethod::Blocking);
    kernels.read_then_unmap(&pending, |readback| {
        let count = lengths.len();
        let mut are_the_sequence_numbers_holding = true;
        let results = (0..count)
            .map(|entry| {
                let entry_number = u32::try_from(entry).expect("条数");
                let (kernel_one, kernel_one_sequence) = readback_entry(readback, entry);
                let (kernel_two, kernel_two_sequence) = readback_entry(readback, count + entry);
                are_the_sequence_numbers_holding &= kernel_one_sequence == entry_number;
                let kernel_two = match dispatched {
                    KernelsToDispatch::Both => {
                        are_the_sequence_numbers_holding &= kernel_two_sequence == entry_number;
                        Some(kernel_two)
                    }
                    KernelsToDispatch::KernelOneOnly => None,
                };
                (Some(kernel_one), kernel_two)
            })
            .collect();
        (results, are_the_sequence_numbers_holding)
    })
}

#[cfg(test)]
mod tests;
