//! 扫描里的校验和核对那两份 WGSL 核在真显卡上与 CPU 参照（核心层的 `crc32_castagnoli`）对拍、两份核彼此对拍
//! （D26（后台整理与放置回收） 已定项 13「GPU 路径与 CPU 路径是两份实现……两份实现在测试里对拍」）。
//! 只开服务经 `GPU_CARDS_BY_PRIORITY` 交给这个进程、属于这一台的那几张卡（`common_discrete_gpus_of_this_host`）：服务没交这一台的卡时每条打一行
//! `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>` 报「本次未跑」、返回成功；交了卡却对不上本机的独显、或开不了判红。
//! 源头没声明张数，服务分一张；整包一起跑时服务照别的目标的声明多分，这里照样全用交来的（点名一张的那几条取头一张）。
//! 只在开了特性 `tests-on-a-real-gpu` 时编；用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`。
//! 开卡时每张参与的卡的显存上限给 1 GiB（七·3 验收，用户 2026-11-01 定；`common_discrete_gpus_of_this_host`），专门测小上限切片的几条照用更小的。
//! 一张卡按卡的顺序填满那一条在这里（`common_checksum_batches_over_the_handed_over_gpus`，用户 2026-11-02 定「按卡的顺序填满」，测饱和）；
//! 两张、三张的各在自己的测试目标里（`checksum_batches_over_two_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`、
//! `checksum_batches_over_three_real_gpus_give_each_gpu_work_and_agree_with_the_cpu_reference`，源头声明张数）。
//!
//! 长度取 0、1、7、8、9、4095、4096（空、单字节、不满一字、正好一字、一字多一字节、一块少一字节、正好一块），每种长度又各放在
//! 起点模 4 余 0、1、2、3 的位置上（前面垫 0 到 3 字节的单元），期望值一半给对的、一半给错的，判定两边都走到。
mod common_checksum_batches_over_the_handed_over_gpus;
mod common_discrete_gpus_of_this_host;

use common_checksum_batches_over_the_handed_over_gpus::{
    cpu_reference_outcomes,
    every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus, unit_bytes_of,
    units_to_verify,
};
use common_discrete_gpus_of_this_host::{
    background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run,
    handed_over_gpus_of_the_declared_count_or_report_not_run, handed_over_gpus_or_report_not_run,
    opened_background_gpu, selection_over_the_handed_over_gpus,
};
use singlefs_background_gpu::adapter_inventory::gpu_device_identifier_of_the_pci_bus_address;
use singlefs_background_gpu::checksum_kernels::{ChecksumKernelOutcome, CHECKSUM_CHUNK_FOOTPRINT};
use singlefs_background_gpu::two_kernel_reconciliation::{
    ConsecutiveUncertaintyCounts, EntryDecision,
};
use singlefs_core::background_compute::{GpuInventory, GpuMemoryInBytes};

const UNIT_LENGTHS_UNDER_TEST: [usize; 7] = [0, 1, 7, 8, 9, 4095, 4096];
const STARTING_OFFSET_RESIDUES: usize = 4;
/// 等 GPU 的上限：这几批在健康的卡上远用不了这么久。
const WAIT_LIMIT_OF_THE_GPU_TESTS: std::time::Duration = std::time::Duration::from_secs(60);

/// 这批要核的单元字节与期望值：每种起点余数先垫一个那么长的单元，再放每种长度；期望值逢单数位置故意给错。
fn unit_bytes_and_expected_checksums() -> Vec<(Vec<u8>, u32)> {
    let mut units = Vec::new();
    for starting_offset_residue in 0..STARTING_OFFSET_RESIDUES {
        units.push((vec![0x5A; starting_offset_residue], 0));
        for unit_length in UNIT_LENGTHS_UNDER_TEST {
            let unit_bytes = unit_bytes_of(unit_length, starting_offset_residue);
            let correct = singlefs_core::checksum::crc32_castagnoli(&unit_bytes);
            let expected_crc32c = if units.len() % 2 == 0 {
                correct
            } else {
                correct ^ 1
            };
            units.push((unit_bytes, expected_crc32c));
        }
    }
    units
}

#[test]
fn the_table_driven_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset() {
    let Some(mut background_gpu) =
        background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
            "the_table_driven_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset",
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
        return;
    };
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("健康的卡上一批交得成");
    let table_driven: Vec<ChecksumKernelOutcome> = outcomes
        .iter()
        .map(|(table_driven, _bitwise)| *table_driven)
        .collect();
    assert_eq!(
        table_driven,
        cpu_reference_outcomes(&unit_bytes_and_expected),
        "查表那一份核与 CPU 参照逐条相同（算出的值与判定）"
    );
}

#[test]
fn the_bitwise_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset() {
    let Some(mut background_gpu) =
        background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
            "the_bitwise_kernel_matches_the_cpu_reference_on_every_length_and_starting_offset",
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
        return;
    };
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("健康的卡上一批交得成");
    let bitwise: Vec<ChecksumKernelOutcome> = outcomes
        .iter()
        .map(|(_table_driven, bitwise)| *bitwise)
        .collect();
    assert_eq!(
        bitwise,
        cpu_reference_outcomes(&unit_bytes_and_expected),
        "逐位移那一份核与 CPU 参照逐条相同（算出的值与判定）"
    );
}

#[test]
fn the_two_kernels_confirm_every_entry_on_a_healthy_gpu() {
    let Some(mut background_gpu) =
        background_gpu_on_the_handed_over_gpus_under_one_gibibyte_each_or_report_not_run(
            "the_two_kernels_confirm_every_entry_on_a_healthy_gpu",
            WAIT_LIMIT_OF_THE_GPU_TESTS,
        )
    else {
        return;
    };
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let mut uncertainty = ConsecutiveUncertaintyCounts::with_no_entry_uncertain();
    let reconciled = background_gpu
        .verify_unit_checksums(&units_to_verify(&unit_bytes_and_expected), &mut uncertainty)
        .expect("健康的卡上一批交得成");
    assert_eq!(
        (
            reconciled.confirmed_entries,
            reconciled.uncertain_entries,
            reconciled.entries_judged_by_the_cpu_reference
        ),
        (unit_bytes_and_expected.len(), 0, 0),
        "两份核彼此逐条相同：全部确认"
    );
    let confirmed: Vec<ChecksumKernelOutcome> = reconciled
        .decisions
        .iter()
        .map(|decision| match decision {
            EntryDecision::ConfirmedByBothKernels(outcome) => *outcome,
            EntryDecision::UncertainThisRound | EntryDecision::JudgedByTheCpuReference(_) => {
                panic!("健康的卡上不该有不确定的条目：{decision:?}")
            }
        })
        .collect();
    assert_eq!(confirmed, cpu_reference_outcomes(&unit_bytes_and_expected));
}

#[test]
fn the_memory_limit_smaller_than_the_batch_cuts_it_into_chunks_with_the_same_outcomes() {
    let test_name =
        "the_memory_limit_smaller_than_the_batch_cuts_it_into_chunks_with_the_same_outcomes";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    // 8 KiB：一块 4096 字节的单元连同描述与结果装得下，整批（约 33 KiB 的输入）装不下，要切成好几段
    let small_memory_limit = GpuMemoryInBytes(8 * 1024);
    let mut limited = opened_background_gpu(&selection_over_the_handed_over_gpus(
        &handed_over_gpus,
        small_memory_limit,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    ));
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    let outcomes = limited
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("每个单元单独装得进 8 KiB");
    let reference = cpu_reference_outcomes(&unit_bytes_and_expected);
    assert_eq!(
        outcomes,
        reference
            .iter()
            .map(|outcome| (*outcome, *outcome))
            .collect::<Vec<_>>(),
        "切成几段交，两份核的结果照样逐条与 CPU 参照相同"
    );
}

/// 用户 2026-10-28 定「装不下应该自适应的分片」：上限 1 KiB 时 4095、4096 字节的单元单独成一段都装不下，切成几片逐片交、两份核各片的值
/// 各按自己的合法合成整单元的值，照样逐条与 CPU 参照（对整单元一次算）相同；开卡时量代价那一段同样切成几片量。
#[test]
fn the_memory_limit_smaller_than_a_unit_cuts_it_into_pieces_with_the_same_outcomes() {
    let test_name =
        "the_memory_limit_smaller_than_a_unit_cuts_it_into_pieces_with_the_same_outcomes";
    let Some(handed_over_gpus) =
        handed_over_gpus_or_report_not_run(test_name, WAIT_LIMIT_OF_THE_GPU_TESTS)
    else {
        return;
    };
    let smaller_than_a_unit = GpuMemoryInBytes(1024);
    // 上限比量代价那一段小：切开量，不拒挂载
    let mut limited = opened_background_gpu(&selection_over_the_handed_over_gpus(
        &handed_over_gpus,
        smaller_than_a_unit,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    ));
    assert_eq!(
        limited.participating_gpus().len(),
        handed_over_gpus.len(),
        "1 KiB 装得下最小的一片：每张交来的卡都参与"
    );
    let unit_bytes_and_expected = unit_bytes_and_expected_checksums();
    assert!(
        unit_bytes_and_expected
            .iter()
            .any(|(unit_bytes, _expected)| unit_bytes.len() > 1024),
        "这一批里有单元比上限长"
    );
    let outcomes = limited
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("切成几片交得成");
    assert_eq!(
        outcomes,
        cpu_reference_outcomes(&unit_bytes_and_expected)
            .iter()
            .map(|outcome| (*outcome, *outcome))
            .collect::<Vec<_>>(),
        "切成几片交、各片合成整单元的值，两份核照样逐条与 CPU 参照相同"
    );
}

/// 空闲显存不够那一条一批的单元数与每个单元的字节数：合起来 512 MiB。一段装下整批时这一段的输入缓冲（512 MiB）比 wgpu-hal 30.0.1 在
/// `MemoryHints::Performance` 下一块子分配块的上限（256 MiB，`AllocationSizes::from_memory_hints`）大，gpu-allocator 0.28.0 给它单独向驱动申请正好这么大一块
/// （`requires_personal_block`），不会落进产品设备开卡时已经留着的块里（gpu-allocator 至少留一块空的子分配块不放）；推的，读这两个包的源码。
const UNIT_COUNT_OF_THE_BATCH_SHORT_OF_FREE_MEMORY: usize = 8192;
const UNIT_BYTES_OF_THE_BATCH_SHORT_OF_FREE_MEMORY: usize = 64 << 10;
/// 占完之后这张卡的空闲显存要落在这一截里：上界比整批那一段的输入缓冲（512 MiB）小，一段装下整批建不成；下界比一块子分配块的上限（256 MiB）
/// 多 32 MiB，对半切过的一段（输入缓冲不到 256 MiB）建得成，重切之后交得完。
const FREE_BYTES_LEFT_ON_THE_CARD: std::ops::Range<u64> = (288 << 20)..(448 << 20);
/// 占显存的那个设备的子分配块定在 4 MiB（`MemoryHints::Manual`）：比它大的缓冲各自单独向驱动申请正好那么多，占的量照每次建的字节走。
const SUBALLOCATED_BLOCK_BYTES_OF_THE_HOLDING_DEVICE: u64 = 4 << 20;

/// 本进程此刻在这张卡最大的那一块设备本地堆上还能建多少字节：`VK_EXT_memory_budget` 的 heapBudget 减 heapUsage（驱动给的估计，
/// 算进本进程里各个设备在这张卡上占的，也扣掉别的进程占的）。不是 Vulkan 后端、实例或物理设备不到 Vulkan 1.1、这张卡不支持这个扩展，交回 `None`。
#[allow(
    unsafe_code,
    reason = "wgpu 不报空闲显存，只能经 hal 层拿 Vulkan 实例与物理设备句柄去读 VK_EXT_memory_budget；wgpu 与 ash 都没有安全的封装"
)]
fn free_device_local_bytes_of_the_card(adapter: &wgpu::Adapter) -> Option<u64> {
    // SAFETY: `as_hal` 只借出 hal 层的适配器，这里不销毁也不改它指向的任何 Vulkan 对象；守卫在这个函数里丢掉。
    let hal_adapter = unsafe { adapter.as_hal::<wgpu::hal::api::Vulkan>() }?;
    let shared_instance = hal_adapter.shared_instance();
    let capabilities = hal_adapter.physical_device_capabilities();
    if shared_instance.instance_api_version() < ash::vk::API_VERSION_1_1
        || capabilities.properties().api_version < ash::vk::API_VERSION_1_1
        || !capabilities.supports_extension(ash::ext::memory_budget::NAME)
    {
        return None;
    }
    let mut budget = ash::vk::PhysicalDeviceMemoryBudgetPropertiesEXT::default();
    let mut memory_properties =
        ash::vk::PhysicalDeviceMemoryProperties2::default().push_next(&mut budget);
    // SAFETY: 物理设备句柄取自同一个 hal 适配器，它就是这个实例枚举出来的；上面查过实例与物理设备都到 Vulkan 1.1、物理设备支持
    // `VK_EXT_memory_budget`，`vkGetPhysicalDeviceMemoryProperties2` 只往 `memory_properties` 与挂在它后面的 `budget` 里写。
    unsafe {
        shared_instance
            .raw_instance()
            .get_physical_device_memory_properties2(
                hal_adapter.raw_physical_device(),
                &mut memory_properties,
            );
    }
    let heap_count = usize::try_from(memory_properties.memory_properties.memory_heap_count)
        .expect("内存堆至多 16 块，装得进 usize");
    let memory_heaps = memory_properties.memory_properties.memory_heaps;
    let largest_device_local_heap = memory_heaps
        .iter()
        .take(heap_count)
        .enumerate()
        .filter(|(_heap_index, memory_heap)| {
            memory_heap
                .flags
                .contains(ash::vk::MemoryHeapFlags::DEVICE_LOCAL)
        })
        .max_by_key(|(_heap_index, memory_heap)| memory_heap.size)
        .map(|(heap_index, _memory_heap)| heap_index)?;
    Some(
        budget.heap_budget[largest_device_local_heap]
            .saturating_sub(budget.heap_usage[largest_device_local_heap]),
    )
}

/// 本机总线地址 `pci_bus_address` 那张卡的 Vulkan 适配器（另开一个 wgpu 实例枚举）；找不到交回 `None`。
fn vulkan_adapter_of_the_card(pci_bus_address: &str) -> Option<wgpu::Adapter> {
    use singlefs_background_gpu::background_gpu::wait_for_the_wgpu_future;
    use singlefs_background_gpu::waiting_for_the_gpu::SteadyClockOfThisProcess;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    wait_for_the_wgpu_future(
        instance.enumerate_adapters(wgpu::Backends::VULKAN),
        WAIT_LIMIT_OF_THE_GPU_TESTS,
        &SteadyClockOfThisProcess::starting_now(),
    )
    .expect("枚举适配器在等的上限之内")
    .into_iter()
    .find(|adapter| adapter.get_info().device_pci_bus_id == pci_bus_address)
}

/// 另一个设备把一张卡的空闲显存占进一截里之后交回的：占着的设备与缓冲（随它们活着）、占了多少字节、占完回读的空闲量。
struct CardHeldDownToItsFreeMemory {
    _holding_device: wgpu::Device,
    held_buffers: Vec<wgpu::Buffer>,
    bytes_held: u64,
    free_bytes_after_holding: u64,
}

/// 在 `adapter` 那张卡上另起一个设备（子分配块 [`SUBALLOCATED_BLOCK_BYTES_OF_THE_HOLDING_DEVICE`]），逐块建存储缓冲占显存：每块建之前回读这张卡此刻的空闲量
/// （[`free_device_local_bytes_of_the_card`]），这一块建「此刻空闲的减去 `free_bytes_wanted` 的中点」那么多；空闲量落进 `free_bytes_wanted` 才交回。
/// 一块建不成（显存不足）就把能建的最大一块减半再试。每一轮要么建成一块（占的字节至少涨一块子分配块那么多，合起来到 `total_memory_bytes` 就停），
/// 要么把能建的最大一块减半（减到一块子分配块以下就停）：有界。
///
/// # Errors
/// 占不进那一截，交回一句原因，写明占了多少、空闲还剩多少：这张卡不报空闲量、起不了设备、占过了头、建不成比一块子分配块大的缓冲，
/// 或合起来到了显存总量还没占够（驱动把建不成的挪去别的内存，或空闲量不随占的走）。
fn card_held_down_to_its_free_memory_within(
    adapter: &wgpu::Adapter,
    total_memory_bytes: u64,
    free_bytes_wanted: &std::ops::Range<u64>,
) -> Result<CardHeldDownToItsFreeMemory, String> {
    use singlefs_background_gpu::background_gpu::wait_for_the_wgpu_future;
    use singlefs_background_gpu::waiting_for_the_gpu::SteadyClockOfThisProcess;
    let free_bytes_before_holding = free_device_local_bytes_of_the_card(adapter).ok_or_else(|| {
        "这张卡不报此刻的空闲显存（不是 Vulkan 后端、不到 Vulkan 1.1，或不支持 VK_EXT_memory_budget）".to_string()
    })?;
    let (holding_device, _queue) = wait_for_the_wgpu_future(
        adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("holding the free memory"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            experimental_features: wgpu::ExperimentalFeatures::disabled(),
            memory_hints: wgpu::MemoryHints::Manual {
                suballocated_device_memory_block_size:
                    SUBALLOCATED_BLOCK_BYTES_OF_THE_HOLDING_DEVICE
                        ..SUBALLOCATED_BLOCK_BYTES_OF_THE_HOLDING_DEVICE,
            },
            trace: wgpu::Trace::Off,
        }),
        WAIT_LIMIT_OF_THE_GPU_TESTS,
        &SteadyClockOfThisProcess::starting_now(),
    )
    .expect("起设备在等的上限之内")
    .map_err(|refused| format!("起不了占显存的那个设备：{refused}"))?;
    let free_bytes_aimed_at =
        free_bytes_wanted.start + (free_bytes_wanted.end - free_bytes_wanted.start) / 2;
    let mut held_buffers = Vec::new();
    let mut bytes_held: u64 = 0;
    let mut largest_buffer_bytes_to_try = adapter.limits().max_buffer_size.min(1 << 30);
    loop {
        let free_bytes_now = free_device_local_bytes_of_the_card(adapter)
            .ok_or_else(|| "占的途中这张卡不再报空闲显存".to_string())?;
        if free_bytes_wanted.contains(&free_bytes_now) {
            return Ok(CardHeldDownToItsFreeMemory {
                _holding_device: holding_device,
                held_buffers,
                bytes_held,
                free_bytes_after_holding: free_bytes_now,
            });
        }
        let progress = format!(
            "占之前空闲 {free_bytes_before_holding} 字节，占了 {bytes_held} 字节（{} 块），此刻空闲 {free_bytes_now} 字节，要的是 {}..{} 字节",
            held_buffers.len(),
            free_bytes_wanted.start,
            free_bytes_wanted.end
        );
        if free_bytes_now < free_bytes_wanted.start {
            return Err(format!("占过了头：{progress}"));
        }
        let buffer_bytes =
            (free_bytes_now - free_bytes_aimed_at).min(largest_buffer_bytes_to_try) / 4 * 4;
        if buffer_bytes <= SUBALLOCATED_BLOCK_BYTES_OF_THE_HOLDING_DEVICE {
            return Err(format!(
                "建不成比一块子分配块（{SUBALLOCATED_BLOCK_BYTES_OF_THE_HOLDING_DEVICE} 字节）大的缓冲：{progress}"
            ));
        }
        if bytes_held + buffer_bytes > total_memory_bytes {
            return Err(format!(
                "合起来到了显存总量 {total_memory_bytes} 字节还没占够（驱动把建不成的挪去别的内存，或空闲量不随占的走）：{progress}"
            ));
        }
        let out_of_memory_scope = holding_device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        let buffer = holding_device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: buffer_bytes,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let out_of_memory = wait_for_the_wgpu_future(
            out_of_memory_scope.pop(),
            WAIT_LIMIT_OF_THE_GPU_TESTS,
            &SteadyClockOfThisProcess::starting_now(),
        )
        .expect("交回错误范围在等的上限之内");
        if out_of_memory.is_some() {
            drop(buffer);
            largest_buffer_bytes_to_try = buffer_bytes / 2;
        } else {
            bytes_held += buffer_bytes;
            held_buffers.push(buffer);
        }
    }
}

/// 空闲显存不够那一条的一批：[`UNIT_COUNT_OF_THE_BATCH_SHORT_OF_FREE_MEMORY`] 个 [`UNIT_BYTES_OF_THE_BATCH_SHORT_OF_FREE_MEMORY`] 字节的单元，期望值照 CPU 参照算对。
/// 各单元是同一段伪随机字节、开头八个字节换成单元的序号（每个单元不同；只现算一段伪随机字节，debug 编的测试里造 512 MiB 不耗在逐字节的生成上）。
fn units_of_the_batch_short_of_free_memory() -> Vec<(Vec<u8>, u32)> {
    let bytes_shared_by_every_unit = unit_bytes_of(UNIT_BYTES_OF_THE_BATCH_SHORT_OF_FREE_MEMORY, 0);
    (0..UNIT_COUNT_OF_THE_BATCH_SHORT_OF_FREE_MEMORY)
        .map(|unit| {
            let mut unit_bytes = bytes_shared_by_every_unit.clone();
            unit_bytes[..8].copy_from_slice(&u64::try_from(unit).expect("单元序号").to_le_bytes());
            let expected_crc32c = singlefs_core::checksum::crc32_castagnoli(&unit_bytes);
            (unit_bytes, expected_crc32c)
        })
        .collect()
}

/// 判决 m4-gpugc-code-r1 第三节第 5 条（D26（后台整理与放置回收） 已定项 13 第 3 条「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」）在真卡上：
/// 点名服务交来的头一张卡、上限照一段装下整批（[`units_of_the_batch_short_of_free_memory`]，512 MiB 多）给，开卡之后另一个设备逐块占显存、每块后回读这张卡
/// 此刻的空闲量，占到空闲的落进 [`FREE_BYTES_LEFT_ON_THE_CARD`]（比这一段的输入缓冲小）才往下走，再交这一批——照上限一段装得下而建缓冲显存不足，
/// 按更小的片重切重交，照常交回、两份核逐条与 CPU 参照相同，重切了至少一次。
/// 占不进那一截（这张卡不报空闲量、驱动把建不成的挪去别的内存、占过了头）、或这张卡一个绑定装不下一段的输入缓冲时，打 `NOT_RUN_NO_GPU_ADAPTER` 那一行
/// 写明占了多少、空闲剩多少，交回成功（这一条没跑到，不算判过）。
/// 这一批比一块子分配块大（[`UNIT_COUNT_OF_THE_BATCH_SHORT_OF_FREE_MEMORY`] 的说明）：一段只有几百 KiB 时产品设备从开卡时留着的子分配块里就建得成，
/// 卡上真没空闲的也不重切。
#[test]
fn the_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu() {
    let test_name =
        "the_batch_on_a_card_with_less_free_memory_than_its_limit_is_recut_and_agrees_with_the_cpu";
    let Some(handed_over_gpus) = handed_over_gpus_of_the_declared_count_or_report_not_run(
        test_name,
        1,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    ) else {
        return;
    };
    let pci_bus_address = handed_over_gpus[0].pci_bus_address.clone();
    let card = gpu_device_identifier_of_the_pci_bus_address(&pci_bus_address);
    let unit_bytes_and_expected = units_of_the_batch_short_of_free_memory();
    let input_bytes_of_the_batch = u64::try_from(
        UNIT_COUNT_OF_THE_BATCH_SHORT_OF_FREE_MEMORY * UNIT_BYTES_OF_THE_BATCH_SHORT_OF_FREE_MEMORY,
    )
    .expect("512 MiB 装得进 u64");
    let memory_limit_holding_the_batch_in_one_chunk =
        GpuMemoryInBytes(CHECKSUM_CHUNK_FOOTPRINT.bytes_held_by_a_chunk(
            input_bytes_of_the_batch,
            u64::try_from(UNIT_COUNT_OF_THE_BATCH_SHORT_OF_FREE_MEMORY).expect("单元数装得进 u64"),
        ));
    let adapter =
        vulkan_adapter_of_the_card(&pci_bus_address).expect("刚枚举到的那张卡再枚举一次照样在");
    let largest_binding_of_the_card = adapter
        .limits()
        .max_storage_buffer_binding_size
        .min(adapter.limits().max_buffer_size);
    if largest_binding_of_the_card < input_bytes_of_the_batch {
        println!(
            "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=这张卡一个绑定至多 {largest_binding_of_the_card} 字节，装不下一段的输入缓冲 {input_bytes_of_the_batch} 字节"
        );
        return;
    }
    let mut background_gpu = opened_background_gpu(&selection_over_the_handed_over_gpus(
        &handed_over_gpus,
        memory_limit_holding_the_batch_in_one_chunk,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    ));
    let (_card, total_memory) = GpuInventory::devices_with_total_memory(&background_gpu)
        .into_iter()
        .find(|(identifier, _total_memory)| *identifier == card)
        .expect("点名的卡在卡清单里");
    let card_held_down = match card_held_down_to_its_free_memory_within(
        &adapter,
        total_memory.0,
        &FREE_BYTES_LEFT_ON_THE_CARD,
    ) {
        Ok(card_held_down) => card_held_down,
        Err(why_not_held_down) => {
            println!(
                "NOT_RUN_NO_GPU_ADAPTER test={test_name} reason=另一个设备没把这张卡的空闲显存占进 {}..{} 字节：{why_not_held_down}",
                FREE_BYTES_LEFT_ON_THE_CARD.start, FREE_BYTES_LEFT_ON_THE_CARD.end
            );
            return;
        }
    };
    println!(
        "CARD_HELD_DOWN_TO_ITS_FREE_MEMORY test={test_name} bytes_held={} buffers={} free_bytes_after_holding={} input_bytes_of_the_batch={input_bytes_of_the_batch}",
        card_held_down.bytes_held,
        card_held_down.held_buffers.len(),
        card_held_down.free_bytes_after_holding
    );
    let outcomes = background_gpu
        .both_kernel_outcomes_of_the_batch(&units_to_verify(&unit_bytes_and_expected))
        .expect("空闲的比上限少：按更小的片重切重交，不报出错");
    let submissions = background_gpu
        .take_the_submissions_of_the_batch()
        .unit_checksum_verification;
    println!(
        "RECUT_AFTER_RUNNING_OUT_OF_MEMORY test={test_name} recuts={} submissions={} pieces={}",
        submissions.recuts_after_running_out_of_memory,
        submissions.submissions_completed,
        submissions.pieces_submitted
    );
    drop(card_held_down);
    assert_eq!(
        outcomes,
        cpu_reference_outcomes(&unit_bytes_and_expected)
            .iter()
            .map(|outcome| (*outcome, *outcome))
            .collect::<Vec<_>>(),
        "重切重交之后两份核逐条与 CPU 参照相同"
    );
    assert!(
        submissions.recuts_after_running_out_of_memory >= 1,
        "另一个设备把空闲的占到 {}..{} 字节，比一段装下整批的输入缓冲 {input_bytes_of_the_batch} 字节少：这一段建不成、至少重切一次（{submissions:?}）",
        FREE_BYTES_LEFT_ON_THE_CARD.start,
        FREE_BYTES_LEFT_ON_THE_CARD.end
    );
}

/// 一张卡按卡的顺序填满（`common_checksum_batches_over_the_handed_over_gpus`）：点名服务交来的头一张，每次只交得下几十个单元，交两次的量，
/// 都交给这一张、交两次。两张、三张的各在自己的测试目标里（文件头）。
#[test]
fn the_batch_over_one_named_real_gpu_gives_it_work_and_it_submits() {
    every_handed_over_gpu_of_a_batch_is_given_work_and_submits_on_the_real_gpus(
        "the_batch_over_one_named_real_gpu_gives_it_work_and_it_submits",
        1,
        WAIT_LIMIT_OF_THE_GPU_TESTS,
    );
}
