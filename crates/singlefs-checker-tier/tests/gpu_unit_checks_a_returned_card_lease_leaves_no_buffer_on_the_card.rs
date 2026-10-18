//! checker 档模块：gpu_unit_checks
//! 卡池的借据丢掉时先等设备做完、再销账（第 10z 批第六块，用户 2026-10-08 定第 10z 批代码轮岔路 G2「改用乙」）：在本机的卡上借一笔、
//! 建两张 1 GiB 的缓冲，提交一次清零连一段空转约 250 毫秒的内核（活在飞），紧接着丢掉缓冲与借据，之后读一次 `nvidia-smi -q -d MEMORY,PIDS`，
//! 这个进程在那张卡上占着的已经不含那笔缓冲（第 10z 批第八块，第 10z 批第六块代码轮攻方 F-H4：提交之后先读一次显存再丢，丢的时候活早做完了，
//! 照 E174「已做完」档改回只 poll 一次也读不到那笔缓冲，分不出等没等；照 E174「在飞·中」改成活在飞时丢，不等的那一臂第一份读数含整笔）。
//! 读法照 E174（借据丢掉时等待与显存回还）跑前登记 `research/prompts/e174-preregistration.md` 第一节：建缓冲之前读三份取中位数 U_pre，
//! τ 是那三份的极差加 2 MiB；缓冲建好、提交之前读一份，比 U_pre 多出的不少于 S − τ（阳性对照：读数分得出这笔缓冲）；借据丢掉之后读一份，
//! 比 U_pre 多出的不超过 τ。要 `gpu` 特性；本机一张 NVIDIA 独显适配器都没有时打一行 `NOT_RUN_NO_GPU_ADAPTER` 报「本次未跑」、返回成功
//! （`gpu_unit_checks::gpu_test_start`）。
#![cfg(feature = "gpu")]

use singlefs_checker_tier::gpu_unit_checks::gpu_test_start::gpu_card_pool_or_report_not_run;
use singlefs_harness::gpu_memory_ledger::{
    card_memory_readings_of_one_nvidia_smi_report, OccupiedByThisProcess,
    NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS,
};

const TEST_NAME: &str =
    "a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading";
/// 一张缓冲 1 GiB（E174 第五节：S 大于 1 GiB 的建 ⌈S / 1 GiB⌉ 张、每张 1 GiB）。
const MEBIBYTES_OF_ONE_BUFFER: u64 = 1024;
const BUFFERS: u64 = 2;
/// 两份各自取整到 MiB 的读数，相减最多差出不到 2 MiB（E174 第六节 τ 的来由）。
const ROUNDING_OF_TWO_READINGS_MEBIBYTES: u64 = 2;
const BYTES_IN_A_MEBIBYTE: u64 = 1024 * 1024;
/// 空转内核至少跑多久（E174 修订 1「在飞·中」的空转目标 250 毫秒：比读一次显存的约 0.05 秒长得多，丢借据时活还在飞）。
const SPIN_AT_LEAST: std::time::Duration = std::time::Duration::from_millis(250);
/// 标定空转内核时头一次试的循环次数（E174 装置同一个数）与最多试几次。
const FIRST_GUESS_OF_SPIN_ITERATIONS: u32 = 2_000_000;
const SPIN_CALIBRATION_ATTEMPTS: u32 = 8;
/// 标定空转内核时等一次提交的上限：这是测试自己等自己那次提交，不是借据归还前等设备空闲的那个量
/// （`gpu_unit_checks::LONGEST_WAIT_FOR_THE_DEVICE_BEFORE_RETURNING_A_LEASE_OR_A_PLACE_IN_THE_MEMORY_QUEUE`），两个数各自定。
const SPIN_CALIBRATION_SUBMISSION_WAIT: std::time::Duration = std::time::Duration::from_secs(30);
/// 空转内核绑的那一截（第一张缓冲的头 256 字节）。
const SPIN_KERNEL_BINDING_BYTES: u64 = 256;
/// 提交到丢借据之间最多占空转时长的几成，活才算在飞（E174 登记第九节 K9 的 0.8）。
const IN_FLIGHT_SHARE_PERCENT: u128 = 80;

/// 空转内核：单线程做 `iteration_count` 次相依的整数运算，结果写进第一张缓冲的第一个字，编译器消不掉（照 E174 装置的那一个）。
const SPIN_SHADER_SOURCE: &str = r#"
override iteration_count: u32 = 1u;

@group(0) @binding(0) var<storage, read_write> output_words: array<u32>;

@compute @workgroup_size(1)
fn spin() {
    var state: u32 = 12345u;
    for (var step: u32 = 0u; step < iteration_count; step = step + 1u) {
        state = state * 1664525u + 1013904223u;
        state = state ^ (state >> 11u);
    }
    output_words[0] = state;
}
"#;

/// 这个进程此刻在总线号 `bus`（去掉域的写法）那张卡上占着的（MiB；报告里没列这个进程是 0）：跑一次 `nvidia-smi -q -d MEMORY,PIDS`。
fn this_process_occupancy_on_the_card_mebibytes(bus: &str) -> u64 {
    let output = std::process::Command::new("nvidia-smi")
        .args(NVIDIA_SMI_MEMORY_AND_PROCESSES_REPORT_ARGUMENTS)
        .output()
        .expect("借到了卡的机器上起得了 nvidia-smi");
    assert!(output.status.success(), "nvidia-smi 退了 {}", output.status);
    let readings = card_memory_readings_of_one_nvidia_smi_report(
        &String::from_utf8_lossy(&output.stdout),
        std::process::id(),
        &[bus.to_string()],
        &|_bus| false,
    )
    .expect("报告里有借到的那张卡");
    match readings[0].occupied_by_this_process {
        OccupiedByThisProcess::OnlyThroughTheBooks { mebibytes } => mebibytes,
        OccupiedByThisProcess::NotListed => 0,
        OccupiedByThisProcess::AlsoOutsideTheBooks { mebibytes } => {
            panic!("这条用例不起卡池之外的设备，读到分不出的 {mebibytes} MiB")
        }
    }
}

/// 循环 `iteration_count` 次的空转管线。
fn spin_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    iteration_count: u32,
) -> wgpu::ComputePipeline {
    let constants = [("iteration_count", f64::from(iteration_count))];
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(TEST_NAME),
        layout: None,
        module,
        entry_point: Some("spin"),
        compilation_options: wgpu::PipelineCompilationOptions {
            constants: &constants,
            zero_initialize_workgroup_memory: true,
        },
        cache: None,
    })
}

/// 一次提交：清每张缓冲，再在第一张的头 [`SPIN_KERNEL_BINDING_BYTES`] 上派一次空转内核。
fn submit_clearing_and_spinning(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffers: &[wgpu::Buffer],
    pipeline: &wgpu::ComputePipeline,
) -> wgpu::SubmissionIndex {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some(TEST_NAME),
    });
    for buffer in buffers {
        encoder.clear_buffer(buffer, 0, None);
    }
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(TEST_NAME),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffers[0],
                offset: 0,
                size: std::num::NonZeroU64::new(SPIN_KERNEL_BINDING_BYTES),
            }),
        }],
    });
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some(TEST_NAME),
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    queue.submit([encoder.finish()])
}

/// 标定一条至少空转 [`SPIN_AT_LEAST`] 的管线（在一张 1 MiB 的缓冲上派、等完量挂钟，不够就照比例加循环次数再试）：交管线与量到的时长。
/// 标定用的缓冲等完就丢了，不留在之后的读数里。
fn spin_pipeline_running_at_least_the_target(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (wgpu::ComputePipeline, std::time::Duration) {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(TEST_NAME),
        source: wgpu::ShaderSource::Wgsl(SPIN_SHADER_SOURCE.into()),
    });
    let calibration_buffers = [device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(TEST_NAME),
        size: BYTES_IN_A_MEBIBYTE,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })];
    let mut iteration_count = FIRST_GUESS_OF_SPIN_ITERATIONS;
    for _attempt in 0..SPIN_CALIBRATION_ATTEMPTS {
        let pipeline = spin_pipeline(device, &module, iteration_count);
        let started = std::time::Instant::now();
        let submission =
            submit_clearing_and_spinning(device, queue, &calibration_buffers, &pipeline);
        device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(SPIN_CALIBRATION_SUBMISSION_WAIT),
            })
            .expect("标定空转内核时等得到");
        let spun = started.elapsed();
        if spun >= SPIN_AT_LEAST {
            return (pipeline, spun);
        }
        // 照比例加到目标的一倍半（再短也至少翻一倍），下一次多半就够
        let scaled = u128::from(iteration_count) * SPIN_AT_LEAST.as_micros() * 3
            / (2 * spun.as_micros().max(1));
        iteration_count =
            u32::try_from(scaled.max(u128::from(iteration_count) * 2)).unwrap_or(u32::MAX);
    }
    panic!("试了 {SPIN_CALIBRATION_ATTEMPTS} 次标定不出至少空转 {SPIN_AT_LEAST:?} 的内核（最后一次 {iteration_count} 次循环）")
}

#[test]
fn a_card_lease_returned_after_waiting_for_the_device_leaves_its_buffers_off_the_next_reading() {
    let Some(pool) = gpu_card_pool_or_report_not_run(TEST_NAME) else {
        return;
    };
    let size_mebibytes = MEBIBYTES_OF_ONE_BUFFER * BUFFERS;
    let Some(lease) = pool.lease_card_or_report_not_run(TEST_NAME, size_mebibytes) else {
        return;
    };
    let index = lease.card().index;
    let lease_identifier = lease.lease_identifier();
    let bus = lease.card().pci_bus_identifier.clone();
    let device = lease.card().device();
    let queue = lease.card().queue();
    let (pipeline, spin_duration) = spin_pipeline_running_at_least_the_target(device, queue);
    let mut readings_before: Vec<u64> = (0..3)
        .map(|_| this_process_occupancy_on_the_card_mebibytes(&bus))
        .collect();
    readings_before.sort_unstable();
    let occupied_before = readings_before[1];
    let tolerance = readings_before[2] - readings_before[0] + ROUNDING_OF_TWO_READINGS_MEBIBYTES;
    let buffers: Vec<wgpu::Buffer> = (0..BUFFERS)
        .map(|_| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(TEST_NAME),
                size: MEBIBYTES_OF_ONE_BUFFER * BYTES_IN_A_MEBIBYTE,
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        })
        .collect();
    let occupied_while_the_buffers_live = this_process_occupancy_on_the_card_mebibytes(&bus);
    assert!(
        occupied_while_the_buffers_live + tolerance >= occupied_before + size_mebibytes,
        "阳性对照：缓冲建好、提交之前读数里有它（建之前 {occupied_before}、建好 {occupied_while_the_buffers_live}、S {size_mebibytes}、τ {tolerance}，MiB）"
    );
    let submitted = std::time::Instant::now();
    let _in_flight = submit_clearing_and_spinning(device, queue, &buffers, &pipeline);
    drop(buffers);
    let submitted_to_the_lease_dropped = submitted.elapsed();
    drop(lease);
    assert!(
        submitted_to_the_lease_dropped.as_micros() * 100
            < spin_duration.as_micros() * IN_FLIGHT_SHARE_PERCENT,
        "丢借据时活还在飞（提交到丢借据 {submitted_to_the_lease_dropped:?}，空转 {spin_duration:?}）：不在飞的话分不出等没等"
    );
    assert!(
        !pool.is_the_lease_on_the_books(index, lease_identifier),
        "借据丢掉、等完设备之后销了账"
    );
    let occupied_after_the_lease_was_returned = this_process_occupancy_on_the_card_mebibytes(&bus);
    println!(
        "GPU_CARD_LEASE_RETURNED index={index} occupied_before={occupied_before} occupied_while_the_buffers_live={occupied_while_the_buffers_live} \
         occupied_after_the_lease_was_returned={occupied_after_the_lease_was_returned} tolerance={tolerance} \
         spin_microseconds={} submitted_to_the_lease_dropped_microseconds={}",
        spin_duration.as_micros(),
        submitted_to_the_lease_dropped.as_micros()
    );
    assert!(
        occupied_after_the_lease_was_returned <= occupied_before + tolerance,
        "借据丢掉、等完设备之后下一次读数里那笔缓冲已经不在（建之前 {occupied_before}、之后 {occupied_after_the_lease_was_returned}、τ {tolerance}，MiB）"
    );
}
