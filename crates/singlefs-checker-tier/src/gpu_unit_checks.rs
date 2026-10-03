//! 崩溃放量的 GPU 核对（里程碑二「增补 4」第四项；用户 2026-09-28 定：数据从 KV 按批读出来交给 GPU 核对，写回，门禁一次读违例）。
//! 这一版 GPU 接单元级那一截：一批单元的头校验和与载荷 CRC 在 GPU 上算（CRC-32C，参数与 D18（块里携带什么信息） 已定项 17 相同），
//! 同一批在 CPU 上再算一遍逐个对拍，对不上就判红（`GpuCpuDisagreement`）；走树、恢复、记录核对器照旧在 CPU 上。
//! 着色器与多卡枚举取自 E163（GPU多卡算单元校验和） 的装置（那边两卡 2560/2560 与 CPU 逐个相同）。
//! 分工按显卡空闲显存（`nvidia-smi` 的 memory.free，按 PCI 总线号对上 wgpu 的适配器），空闲显存低于门槛的卡不用。

use singlefs_checker::{
    crc32_castagnoli_table, unit_checksum_layout, UNIT_HEADER_CHECKSUM_BYTE_OFFSET,
};

/// 空闲显存低于这个数（MiB）的卡不用：本机有卡被别的服务占着时只剩几百 MiB，起上下文都难（E163 M2 那张卡 141 MiB 起不来）。
pub const MINIMUM_FREE_MEMORY_MEBIBYTES: u64 = 2048;
/// 一个 GPU 槽装一段字节：最大 32 KiB（数据单元的长度），短的按长度截断。
const SLOT_BYTES: usize = 32_768;
const WORKGROUP_SIZE: usize = 64;
const RESULT_ENTRY_BYTES: usize = 8;
/// NVIDIA 的厂商号（Vulkan 适配器与 PCI 设备报的都是它）：崩溃放量只用 NVIDIA 独显，[`usable_gpu_cards`] 按它筛卡，
/// 测试取卡的判定（[`gpu_test_start`]）按它认 Vulkan 适配器与 PCI 上的显示控制器。
pub const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

const CRC32C_SHADER_SOURCE: &str = r#"
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

/// 没给额度的卡，崩溃放量最多用它空闲显存的几分之几（分子、分母）：剩下的留给卡上别的进程。
const SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA: (u64, u64) = (3, 4);

/// 一张起好上下文的卡。
pub struct GpuCard {
    pub name: String,
    pub pci_bus_identifier: String,
    /// `nvidia-smi` 给这张卡的序号（配置按它点名）。
    pub index: u32,
    /// 起上下文那一刻 `nvidia-smi` 报的空闲显存（MiB）。
    pub free_memory_mebibytes: u64,
    /// 配置给这张卡的显存额度（MiB）：崩溃放量在这张卡上占的显存不超过它；没给额度的是 `None`。
    pub memory_quota_mebibytes: Option<u64>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

/// `nvidia-smi` 报的一张卡：序号与空闲显存 MiB。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReportedCard {
    index: u32,
    free_memory_mebibytes: u64,
}

/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 序号与空闲显存。起不来 `nvidia-smi` 就是空表。
fn reported_cards_by_pci_bus() -> std::collections::BTreeMap<String, ReportedCard> {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=pci.bus_id,index,memory.free",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return std::collections::BTreeMap::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split(',');
            let bus = fields.next()?;
            let index = fields.next()?.trim().parse().ok()?;
            let free_memory_mebibytes = fields.next()?.trim().parse().ok()?;
            Some((
                normalized_bus(bus),
                ReportedCard {
                    index,
                    free_memory_mebibytes,
                },
            ))
        })
        .collect()
}

/// 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡。
fn normalized_bus(text: &str) -> String {
    let lowered = text.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

/// 配置点名的卡，按配置的次序（优先级）：`(nvidia-smi 序号, 显存额度 MiB)`。点名而用不了的卡（本机没有这个序号、
/// 空闲显存低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]、上下文起不来）打一行说明，不悄悄换成别的卡；没点名的卡一张不用。
#[must_use]
pub fn gpu_cards_by_priority(configured: &[(u32, u64)]) -> Vec<GpuCard> {
    let mut usable = usable_gpu_cards();
    let mut cards = Vec::with_capacity(configured.len());
    for (index, quota) in configured {
        match usable.iter().position(|card| card.index == *index) {
            Some(position) => {
                let mut card = usable.remove(position);
                card.memory_quota_mebibytes = Some(*quota);
                cards.push(card);
            }
            None => eprintln!(
                "GPU_CARD_NOT_USABLE index={index}：配置点名了这张卡，本机却用不了它（没有这个序号、空闲显存低于 {MINIMUM_FREE_MEMORY_MEBIBYTES} MiB、或上下文起不来）"
            ),
        }
    }
    cards
}

/// 本机能用的卡：Vulkan 下的 NVIDIA 独显，空闲显存不低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]，上下文起得来；按空闲显存从大到小排。
#[must_use]
pub fn usable_gpu_cards() -> Vec<GpuCard> {
    enable_the_driver_shader_disk_cache();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let reported_by_bus = reported_cards_by_pci_bus();
    let mut cards: Vec<GpuCard> =
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
            .into_iter()
            .filter_map(|adapter| {
                let information = adapter.get_info();
                if information.vendor != NVIDIA_VENDOR_IDENTIFIER
                    || information.device_type != wgpu::DeviceType::DiscreteGpu
                {
                    return None;
                }
                let reported =
                    *reported_by_bus.get(&normalized_bus(&information.device_pci_bus_id))?;
                let free = reported.free_memory_mebibytes;
                if free < MINIMUM_FREE_MEMORY_MEBIBYTES {
                    return None;
                }
                let (device, queue) =
                    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                        label: Some("singlefs-gpu-unit-checks"),
                        required_features: wgpu::Features::empty(),
                        // 崩溃放量的判器把整条流的版本字节与每个状态的草稿区各绑成一张存储缓冲，默认的 128 MiB 绑定上限装不下大流：
                        // 按这张卡实际支持的上限要（NVIDIA Vulkan 上是 2 GiB − 1）。
                        required_limits: adapter.limits(),
                        experimental_features: wgpu::ExperimentalFeatures::disabled(),
                        memory_hints: wgpu::MemoryHints::Performance,
                        trace: wgpu::Trace::Off,
                    }))
                    .ok()?;
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("crc32c"),
                    source: wgpu::ShaderSource::Wgsl(CRC32C_SHADER_SOURCE.into()),
                });
                let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("crc32c"),
                    layout: None,
                    module: &module,
                    entry_point: Some("main"),
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                    cache: None,
                });
                Some(GpuCard {
                    name: information.name,
                    pci_bus_identifier: normalized_bus(&information.device_pci_bus_id),
                    index: reported.index,
                    free_memory_mebibytes: free,
                    memory_quota_mebibytes: None,
                    device,
                    queue,
                    pipeline,
                })
            })
            .collect();
    cards.sort_by_key(|card| std::cmp::Reverse(card.free_memory_mebibytes));
    cards
}

/// 驱动把编好的 Vulkan 管线存到盘上的目录：跨轮复用的缓存放 `GATE_CROSS_RUN_TMPDIR` 下；没设就放家目录 `~/.cache/singlefs`
/// （`/tmp` 开机会清空，冷编要十几分钟的内核每次重启都得重编）；连 `HOME` 都没有才退到 `TMPDIR`、`/tmp`
/// （`command-safety.md`「测试镜像一律放临时目录」：有意跨轮复用的缓存不算镜像）。
#[must_use]
pub fn driver_shader_disk_cache_directory() -> std::path::PathBuf {
    let base = std::env::var_os("GATE_CROSS_RUN_TMPDIR")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".cache/singlefs"))
        })
        .or_else(|| std::env::var_os("TMPDIR").map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    base.join("singlefs-gpu-shader-cache")
}

/// NVIDIA 驱动的盘上着色器缓存默认不罩 Vulkan 管线（崩溃放量的判器内核冷编要几分钟，每个进程都重编）；
/// 起 Vulkan 实例之前把它开起来、指到 [`driver_shader_disk_cache_directory`]，同一版内核第二个进程起就直接装（实测 57 秒变 0.5 秒）。
/// 用户已经设了这几个变量的照用户的；`SKIP_CLEANUP` 让驱动不按默认的容量上限清掉大内核的那一份。
fn enable_the_driver_shader_disk_cache() {
    if std::env::var_os("__GL_SHADER_DISK_CACHE").is_none() {
        std::env::set_var("__GL_SHADER_DISK_CACHE", "1");
    }
    if std::env::var_os("__GL_SHADER_DISK_CACHE_PATH").is_none() {
        let directory = driver_shader_disk_cache_directory();
        let _ = std::fs::create_dir_all(&directory);
        std::env::set_var("__GL_SHADER_DISK_CACHE_PATH", &directory);
    }
    if std::env::var_os("__GL_SHADER_DISK_CACHE_SKIP_CLEANUP").is_none() {
        std::env::set_var("__GL_SHADER_DISK_CACHE_SKIP_CLEANUP", "1");
    }
    // 驱动按可执行文件分缓存目录；崩溃放量七个分项各是一个测试二进制，同一版内核每个都要冷编一遍（release 十几分钟）。
    // 给一个共用的应用名，认得这个变量的驱动把它们放进同一份缓存，只编一次；不认得的照旧按可执行文件分。
    if std::env::var_os("__GL_SHADER_DISK_CACHE_APP_NAME").is_none() {
        std::env::set_var("__GL_SHADER_DISK_CACHE_APP_NAME", "singlefs-crash-judge");
    }
}

impl GpuCard {
    /// 崩溃放量在这张卡上最多占多少显存（MiB）：给了额度的取额度与空闲显存里小的那个，
    /// 没给额度的取空闲显存的 [`SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA`]。
    #[must_use]
    pub fn memory_budget_mebibytes(&self) -> u64 {
        match self.memory_quota_mebibytes {
            Some(quota) => quota.min(self.free_memory_mebibytes),
            None => {
                let (numerator, denominator) = SHARE_OF_FREE_MEMORY_WITHOUT_A_QUOTA;
                self.free_memory_mebibytes / denominator * numerator
            }
        }
    }

    /// 这张卡的设备（崩溃放量第 ② 段的核对内核在同一张卡上另起自己的管线）。
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// 这张卡的队列。
    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// 一批字节段（每段不超过 32 KiB）在这张卡上算 CRC-32C，按次序交回。
    ///
    /// # Panics
    /// 有一段超过 32 KiB；回读的结果序号对不上（着色器与派发错了，不许带着错结果往下走）。
    #[must_use]
    pub fn crc32c_of_ranges(&self, ranges: &[&[u8]]) -> Vec<u32> {
        if ranges.is_empty() {
            return Vec::new();
        }
        let count = ranges.len();
        let mut slots = vec![0u8; count * SLOT_BYTES];
        let mut lengths = Vec::with_capacity(count);
        for (index, range) in ranges.iter().enumerate() {
            assert!(
                range.len() <= SLOT_BYTES,
                "一段 {} 字节，超过一个槽的 {SLOT_BYTES}",
                range.len()
            );
            slots[index * SLOT_BYTES..index * SLOT_BYTES + range.len()].copy_from_slice(range);
            lengths.push(u32::try_from(range.len()).expect("不超过 32 KiB"));
        }
        let words_to_bytes = |values: &[u32]| {
            values
                .iter()
                .flat_map(|value| value.to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let count_u32 = u32::try_from(count).expect("一批的段数装得进 u32");
        let device = &self.device;
        let parameters = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&[count_u32, 0, 0, 0]),
                usage: wgpu::BufferUsages::UNIFORM,
            },
        );
        let input = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &slots,
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let length_buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &words_to_bytes(&lengths),
                usage: wgpu::BufferUsages::STORAGE,
            },
        );
        let result_bytes = u64::try_from(count * RESULT_ENTRY_BYTES).expect("装得进 u64");
        let results = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &vec![0xFFu8; count * RESULT_ENTRY_BYTES],
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            },
        );
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: result_bytes,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = self.pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: parameters.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: length_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: results.as_entire_binding(),
                },
            ],
        });
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups(
                u32::try_from(count.div_ceil(WORKGROUP_SIZE)).expect("工作组数装得进 u32"),
                1,
                1,
            );
        }
        encoder.copy_buffer_to_buffer(&results, 0, &staging, 0, result_bytes);
        self.queue.submit(std::iter::once(encoder.finish()));
        let slice = staging.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).expect("回调送得出去")
        });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: None,
            })
            .expect("等 GPU 算完");
        receiver.recv().expect("回调送达").expect("回读映射成功");
        let mapped = slice.get_mapped_range().expect("读得到回读缓冲区");
        let mut crcs = Vec::with_capacity(count);
        let (entries, remainder) = mapped.as_chunks::<RESULT_ENTRY_BYTES>();
        assert!(
            remainder.is_empty(),
            "回读缓冲区的长度是每条结果 8 字节的整数倍"
        );
        for (index, chunk) in entries.iter().enumerate() {
            let crc = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let local_offset = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
            assert_eq!(
                usize::try_from(local_offset).expect("装得进 usize"),
                index,
                "GPU 回来的第 {index} 条结果序号不对"
            );
            crcs.push(crc);
        }
        drop(mapped);
        staging.unmap();
        crcs
    }
}

/// 一个单元两道校验和的核对结果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitChecksumVerdict {
    BothHold,
    /// 类标签不认得、长度不对：不是这一截能判的，交回 CPU 那一整套去判。
    NotAUnitThisCheckReads,
    HeaderChecksumMismatch,
    PayloadCrcMismatch,
    /// GPU 与 CPU 算出的 CRC 不同：GPU 这一截本身错了，整批不作数。
    GpuCpuDisagreement,
}

/// 一批单元在一张卡上核两道校验和，同一批在 CPU 上再算一遍逐个对拍。
#[must_use]
pub fn check_unit_checksums_on_gpu(card: &GpuCard, units: &[&[u8]]) -> Vec<UnitChecksumVerdict> {
    let mut header_copies: Vec<Vec<u8>> = Vec::new();
    let mut plan: Vec<Option<(usize, u32, u32)>> = Vec::with_capacity(units.len());
    for unit in units {
        match unit_checksum_layout(unit).filter(|layout| {
            u64::try_from(unit.len()).ok() == Some(layout.expected_length_in_bytes)
                && layout.header_end <= unit.len()
        }) {
            Some(layout) => {
                let mut covered = unit[..layout.header_end].to_vec();
                let field = UNIT_HEADER_CHECKSUM_BYTE_OFFSET..UNIT_HEADER_CHECKSUM_BYTE_OFFSET + 32;
                let stored_header = u32::from_le_bytes(
                    unit[field.start..field.start + 4]
                        .try_into()
                        .expect("4 字节"),
                );
                covered[field].fill(0);
                let stored_payload = u32::from_le_bytes(
                    unit[layout.payload_crc_offset..layout.payload_crc_offset + 4]
                        .try_into()
                        .expect("4 字节"),
                );
                plan.push(Some((layout.header_end, stored_header, stored_payload)));
                header_copies.push(covered);
            }
            None => plan.push(None),
        }
    }
    let mut ranges: Vec<&[u8]> = Vec::new();
    let mut header_index = 0usize;
    for (unit, entry) in units.iter().zip(&plan) {
        if let Some((header_end, _, _)) = entry {
            ranges.push(&header_copies[header_index]);
            ranges.push(&unit[*header_end..]);
            header_index += 1;
        }
    }
    let gpu = card.crc32c_of_ranges(&ranges);
    let mut verdicts = Vec::with_capacity(units.len());
    let mut next = 0usize;
    for entry in &plan {
        let Some((_, stored_header, stored_payload)) = entry else {
            verdicts.push(UnitChecksumVerdict::NotAUnitThisCheckReads);
            continue;
        };
        let (gpu_header, gpu_payload) = (gpu[next], gpu[next + 1]);
        let (cpu_header, cpu_payload) = (
            crc32_castagnoli_table(ranges[next]),
            crc32_castagnoli_table(ranges[next + 1]),
        );
        next += 2;
        verdicts.push(if gpu_header != cpu_header || gpu_payload != cpu_payload {
            UnitChecksumVerdict::GpuCpuDisagreement
        } else if gpu_header != *stored_header {
            UnitChecksumVerdict::HeaderChecksumMismatch
        } else if gpu_payload != *stored_payload {
            UnitChecksumVerdict::PayloadCrcMismatch
        } else {
            UnitChecksumVerdict::BothHold
        });
    }
    verdicts
}

/// GPU 对拍用例开跑前取卡，只给测试用（`tests/` 下的 GPU 对拍与库内要卡的计时用例都经它）。
/// 本机没有 NVIDIA 卡时打一行 `NOT_RUN_NO_GPU_ADAPTER test=<用例名> reason=<一句>`、用例当场返回成功（门禁按这一行逐条报「本次未跑」，
/// 上游 `test-script-discipline.md`「分快慢两档」：要特定硬件的那几条在没有那种硬件的机器上报「本次未跑」）；有卡而用不上判红
/// （同一篇「用上现有的硬件加速」：有那种硬件而用不上要报错，不悄悄退回）。
/// 「没卡」看两处：Vulkan 枚举里有没有 NVIDIA 独显，PCI 上有没有 NVIDIA 显示控制器——Vulkan 驱动坏了枚举不出来时，PCI 上照样看得见那张卡
/// （C620：GPU 对拍认「没卡」只看 Vulkan 枚举，驱动坏了被当成本次未跑）。看不见 PCI 总线（[`gpu_test_start::PCI_BUS_DIRECTORY`] 整个不在）
/// 也算没卡；总线看得见而设备表读不出照旧判红（代码三方 test-script-align-code-r2 判决 J2 G1）。判定是纯函数 [`gpu_test_start::decide_gpu_test_start`]，
/// 读 Vulkan 与 PCI 在取卡那一层（[`gpu_test_start::usable_cards_or_report_not_run`]）。
pub mod gpu_test_start {
    use super::{usable_gpu_cards, GpuCard, NVIDIA_VENDOR_IDENTIFIER};

    /// PCI 设备 class（24 位：基类、子类、编程接口各一字节）的基类是这个值的是显示控制器；同一块卡的 HDA 音频功能基类是 0x04，不算。
    pub const PCI_DISPLAY_CONTROLLER_BASE_CLASS: u32 = 0x03;
    /// sysfs 里 PCI 总线的目录：它整个不在就是这台机器看不见 PCI 总线（没有 PCI 的微虚机、沙箱，或没挂 sysfs）。
    pub const PCI_BUS_DIRECTORY: &str = "/sys/bus/pci";
    /// [`PCI_BUS_DIRECTORY`] 下列 PCI 设备的子目录：每个设备一个子目录，里面 `vendor` 与 `class` 两个文件各是一行 `0x` 起头的十六进制。
    pub const PCI_DEVICES_SUBDIRECTORY_NAME: &str = "devices";

    /// Vulkan 适配器的种类（与 Vulkan 报的物理设备类型一一对应）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum AdapterDeviceKind {
        DiscreteGpu,
        IntegratedGpu,
        VirtualGpu,
        SoftwareRenderingOnTheCpu,
        OtherOrUnknown,
    }

    /// Vulkan 枚举出来的一个适配器里，判定要看的两样。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct EnumeratedAdapter {
        pub vendor_identifier: u32,
        pub device_kind: AdapterDeviceKind,
    }

    impl EnumeratedAdapter {
        /// 崩溃放量认的那一种卡：NVIDIA 独显（[`usable_gpu_cards`] 只从这一种里起上下文）。
        #[must_use]
        pub fn is_an_nvidia_discrete_gpu(&self) -> bool {
            self.vendor_identifier == NVIDIA_VENDOR_IDENTIFIER
                && self.device_kind == AdapterDeviceKind::DiscreteGpu
        }
    }

    /// PCI 上一个设备的厂商号与 class（24 位）。
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct PciDeviceIdentity {
        pub vendor_identifier: u32,
        pub class_code: u32,
    }

    impl PciDeviceIdentity {
        /// 从 sysfs 里这个设备的 `vendor` 与 `class` 两个文件的原文读出来（各一行 `0x` 起头的十六进制，带不带行尾换行都认）；
        /// 有一个读不出交 `None`。
        #[must_use]
        pub fn from_sysfs_text(vendor_text: &str, class_text: &str) -> Option<Self> {
            let hexadecimal_value = |text: &str| {
                let digits = text.trim().strip_prefix("0x")?;
                u32::from_str_radix(digits, 16).ok()
            };
            Some(Self {
                vendor_identifier: hexadecimal_value(vendor_text)?,
                class_code: hexadecimal_value(class_text)?,
            })
        }

        /// NVIDIA 的显示控制器（显卡本身，不是同一块卡上的音频功能）。
        #[must_use]
        pub fn is_an_nvidia_display_controller(&self) -> bool {
            self.vendor_identifier == NVIDIA_VENDOR_IDENTIFIER
                && self.class_code >> 16 == PCI_DISPLAY_CONTROLLER_BASE_CLASS
        }
    }

    /// 读 PCI 设备表的结果。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum PciNvidiaDisplayControllers {
        /// 每个设备的厂商号与 class 都读出来了：其中 NVIDIA 显示控制器有几个。
        Counted {
            nvidia_display_controller_count: usize,
        },
        /// PCI 总线看得见，而设备表列不出、或某个设备的 `vendor`、`class` 读不出、认不出。
        ListingUnreadable { reason: String },
        /// 看不见 PCI 总线：[`PCI_BUS_DIRECTORY`] 整个不在。这台机器上没有 PCI 设备可数，判定与「数到 0 个」同。
        PciBusNotVisible { reason: String },
    }

    /// 一串 PCI 设备里 NVIDIA 显示控制器的个数。
    #[must_use]
    pub fn count_nvidia_display_controllers(devices: &[PciDeviceIdentity]) -> usize {
        devices
            .iter()
            .filter(|device| device.is_an_nvidia_display_controller())
            .count()
    }

    /// 一条 GPU 对拍用例开跑前的结局。
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum GpuTestStart {
        /// 至少一张卡起好了上下文：照常跑。
        RunOnTheUsableCards,
        /// Vulkan 枚举里没有 NVIDIA 独显，PCI 上也没有 NVIDIA 显示控制器（或看不见 PCI 总线）：这台机器没有这种硬件，报「本次未跑」。
        NotRunNoGpuAdapter { reason: String },
        /// Vulkan 枚举出 NVIDIA 独显，却一张卡都没起来（`nvidia-smi` 读不到、空闲显存低于门槛、上下文起不来）：判红。
        AdapterPresentButNoCardStarted { reason: String },
        /// PCI 上有 NVIDIA 显示控制器，Vulkan 枚举里却没有一张 NVIDIA 独显（用户态 Vulkan 驱动坏了或不在）：判红。
        NvidiaCardOnPciButVulkanEnumeratesNone { reason: String },
        /// Vulkan 枚举里没有 NVIDIA 独显，PCI 总线看得见而设备表读不出：分不清没卡与驱动坏，判红
        /// （代码三方 test-script-align-code-r2 判决 J2 G1 定了 C620 留下的这一支：看不见总线才算没卡）。
        PciBusVisibleButListingUnreadable { reason: String },
    }

    /// `adapters` 是 Vulkan 枚举的结果，`pci` 是读 PCI 设备表的结果，`usable_card_count` 是起好上下文的卡数（[`usable_gpu_cards`] 交回几张）。
    #[must_use]
    pub fn decide_gpu_test_start(
        adapters: &[EnumeratedAdapter],
        pci: &PciNvidiaDisplayControllers,
        usable_card_count: usize,
    ) -> GpuTestStart {
        if usable_card_count > 0 {
            return GpuTestStart::RunOnTheUsableCards;
        }
        let nvidia_discrete_adapter_count = adapters
            .iter()
            .filter(|adapter| adapter.is_an_nvidia_discrete_gpu())
            .count();
        if nvidia_discrete_adapter_count > 0 {
            return GpuTestStart::AdapterPresentButNoCardStarted {
                reason: format!(
                    "Vulkan 枚举出 {nvidia_discrete_adapter_count} 张 NVIDIA 独显，一张都没起来（nvidia-smi 读不到这张卡、空闲显存低于门槛、或上下文起不来）：修好驱动或腾出显存再跑"
                ),
            };
        }
        match pci {
            PciNvidiaDisplayControllers::Counted {
                nvidia_display_controller_count,
            } => {
                if *nvidia_display_controller_count == 0 {
                    GpuTestStart::NotRunNoGpuAdapter {
                        reason: format!(
                            "Vulkan 枚举出 {} 个适配器，没有一张 NVIDIA 独显，PCI 上也没有 NVIDIA 显示控制器：{adapters:?}",
                            adapters.len()
                        ),
                    }
                } else {
                    GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone {
                        reason: format!(
                            "PCI 上有 {nvidia_display_controller_count} 个 NVIDIA 显示控制器，Vulkan 枚举出的 {} 个适配器里却没有一张 NVIDIA 独显（{adapters:?}）。驱动坏了：修驱动，或改配置关 GPU",
                            adapters.len()
                        ),
                    }
                }
            }
            PciNvidiaDisplayControllers::ListingUnreadable { reason } => {
                GpuTestStart::PciBusVisibleButListingUnreadable {
                    reason: format!(
                        "Vulkan 枚举出的 {} 个适配器里没有一张 NVIDIA 独显，PCI 总线看得见而设备表读不出（{reason}）：分不清是没卡还是驱动坏了，不报本次未跑",
                        adapters.len()
                    ),
                }
            }
            PciNvidiaDisplayControllers::PciBusNotVisible { reason } => {
                GpuTestStart::NotRunNoGpuAdapter {
                    reason: format!(
                        "Vulkan 枚举出 {} 个适配器，没有一张 NVIDIA 独显，也看不见 PCI 总线（{reason}）：{adapters:?}",
                        adapters.len()
                    ),
                }
            }
        }
    }

    /// 这条用例报「本次未跑」时打的那一行。
    #[must_use]
    pub fn not_run_line(test_name: &str, reason: &str) -> String {
        format!("NOT_RUN_NO_GPU_ADAPTER test={test_name} reason={reason}")
    }

    fn adapter_device_kind(device_type: wgpu::DeviceType) -> AdapterDeviceKind {
        match device_type {
            wgpu::DeviceType::DiscreteGpu => AdapterDeviceKind::DiscreteGpu,
            wgpu::DeviceType::IntegratedGpu => AdapterDeviceKind::IntegratedGpu,
            wgpu::DeviceType::VirtualGpu => AdapterDeviceKind::VirtualGpu,
            wgpu::DeviceType::Cpu => AdapterDeviceKind::SoftwareRenderingOnTheCpu,
            wgpu::DeviceType::Other => AdapterDeviceKind::OtherOrUnknown,
        }
    }

    /// 本机 Vulkan 枚举出的每个适配器（与 [`usable_gpu_cards`] 同一个后端）。
    #[must_use]
    pub fn enumerate_vulkan_adapters() -> Vec<EnumeratedAdapter> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        });
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
            .into_iter()
            .map(|adapter| {
                let information = adapter.get_info();
                EnumeratedAdapter {
                    vendor_identifier: information.vendor,
                    device_kind: adapter_device_kind(information.device_type),
                }
            })
            .collect()
    }

    /// 读 `bus_directory`（平常是 [`PCI_BUS_DIRECTORY`]；测试给一棵假的）：它整个不在（`NotFound`）交
    /// [`PciNvidiaDisplayControllers::PciBusNotVisible`]；在，就照 [`read_pci_nvidia_display_controllers_under`] 读它下面的
    /// [`PCI_DEVICES_SUBDIRECTORY_NAME`]。总线目录在而设备表列不出、某个设备读不出，与总线目录本身看不了（`NotFound` 之外的错，
    /// 权限这类）一样交 [`PciNvidiaDisplayControllers::ListingUnreadable`]，不当成看不见总线。
    #[must_use]
    pub fn read_pci_nvidia_display_controllers_on_the_bus(
        bus_directory: &std::path::Path,
    ) -> PciNvidiaDisplayControllers {
        match std::fs::metadata(bus_directory) {
            Ok(_) => read_pci_nvidia_display_controllers_under(
                &bus_directory.join(PCI_DEVICES_SUBDIRECTORY_NAME),
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                PciNvidiaDisplayControllers::PciBusNotVisible {
                    reason: format!("{} 不在：{error}", bus_directory.display()),
                }
            }
            Err(error) => PciNvidiaDisplayControllers::ListingUnreadable {
                reason: format!("{} 看不了：{error}", bus_directory.display()),
            },
        }
    }

    /// 读 `devices_directory`（平常是 [`PCI_BUS_DIRECTORY`] 下的 [`PCI_DEVICES_SUBDIRECTORY_NAME`]；测试给一棵假的）下每个设备的
    /// `vendor` 与 `class`，数 NVIDIA 显示控制器；目录或任何一个设备的这两个文件读不出、认不出，交 [`PciNvidiaDisplayControllers::ListingUnreadable`]。
    #[must_use]
    pub fn read_pci_nvidia_display_controllers_under(
        devices_directory: &std::path::Path,
    ) -> PciNvidiaDisplayControllers {
        let unreadable =
            |what: String| PciNvidiaDisplayControllers::ListingUnreadable { reason: what };
        let entries = match std::fs::read_dir(devices_directory) {
            Ok(entries) => entries,
            Err(error) => {
                return unreadable(format!("{} 列不出：{error}", devices_directory.display()))
            }
        };
        let mut devices = Vec::new();
        for entry in entries {
            let device_directory = match entry {
                Ok(entry) => entry.path(),
                Err(error) => {
                    return unreadable(format!(
                        "{} 里一项读不出：{error}",
                        devices_directory.display()
                    ))
                }
            };
            let read = |file_name: &str| {
                std::fs::read_to_string(device_directory.join(file_name)).map_err(|error| {
                    format!(
                        "{} 读不出：{error}",
                        device_directory.join(file_name).display()
                    )
                })
            };
            let (vendor_text, class_text) = match (read("vendor"), read("class")) {
                (Ok(vendor_text), Ok(class_text)) => (vendor_text, class_text),
                (Err(what), Ok(_) | Err(_)) | (Ok(_), Err(what)) => return unreadable(what),
            };
            match PciDeviceIdentity::from_sysfs_text(&vendor_text, &class_text) {
                Some(device) => devices.push(device),
                None => {
                    return unreadable(format!(
                        "{} 的 vendor {:?}、class {:?} 认不出",
                        device_directory.display(),
                        vendor_text.trim(),
                        class_text.trim()
                    ))
                }
            }
        }
        PciNvidiaDisplayControllers::Counted {
            nvidia_display_controller_count: count_nvidia_display_controllers(&devices),
        }
    }

    /// 本机起好的每张卡（按空闲显存从大到小，至少一张）；本机没有 NVIDIA 卡时打 [`not_run_line`] 那一行、交回 `None`（调用的用例当场返回）。
    ///
    /// # Panics
    /// 有卡而用不上：Vulkan 枚举出 NVIDIA 独显而一张都没起来，PCI 上有 NVIDIA 显示控制器而 Vulkan 枚举不出，
    /// 或 Vulkan 里没有、PCI 总线看得见而设备表读不出。
    #[must_use]
    pub fn usable_cards_or_report_not_run(test_name: &str) -> Option<Vec<GpuCard>> {
        let cards = usable_gpu_cards();
        match decide_gpu_test_start(
            &enumerate_vulkan_adapters(),
            &read_pci_nvidia_display_controllers_on_the_bus(std::path::Path::new(
                PCI_BUS_DIRECTORY,
            )),
            cards.len(),
        ) {
            GpuTestStart::RunOnTheUsableCards => Some(cards),
            GpuTestStart::NotRunNoGpuAdapter { reason } => {
                println!("{}", not_run_line(test_name, &reason));
                None
            }
            GpuTestStart::AdapterPresentButNoCardStarted { reason }
            | GpuTestStart::NvidiaCardOnPciButVulkanEnumeratesNone { reason }
            | GpuTestStart::PciBusVisibleButListingUnreadable { reason } => {
                panic!("{test_name}：{reason}")
            }
        }
    }

    /// 空闲显存最大的那张卡；别的同 [`usable_cards_or_report_not_run`]。
    ///
    /// # Panics
    /// 同 [`usable_cards_or_report_not_run`]。
    #[must_use]
    pub fn first_usable_card_or_report_not_run(test_name: &str) -> Option<GpuCard> {
        usable_cards_or_report_not_run(test_name).map(|cards| {
            cards.into_iter().next().expect(
                "判成照跑只在起好的卡数大于 0 时（decide_gpu_test_start 第一句），交回的卡表不会是空的",
            )
        })
    }
}

/// 代码三方 test-script-align-code-r2 判决 J2 G1：看不见 PCI 总线（总线目录整个不在）算没卡、报本次未跑；
/// 总线看得见而设备表列不出、某个设备读不出照旧判红。判定是纯函数，读 PCI 用假的 sysfs 树，不要显卡。
#[cfg(test)]
mod tests {
    use super::gpu_test_start::{
        decide_gpu_test_start, read_pci_nvidia_display_controllers_on_the_bus, GpuTestStart,
        PciNvidiaDisplayControllers, PCI_DEVICES_SUBDIRECTORY_NAME,
    };

    /// 假的 sysfs 里 PCI 总线那一截的样子。
    enum FakePciBus<'devices> {
        /// `<根>/bus` 在，`<根>/bus/pci` 不在。
        NotThere,
        /// `<根>/bus/pci` 在，它下面没有设备子目录。
        ThereWithoutItsDevicesDirectory,
        /// `<根>/bus/pci/devices/<地址>` 每个设备一个子目录，`vendor` 与 `class` 照写；`None` 的那个文件不建。
        ThereWithDevices(&'devices [(&'devices str, Option<&'devices str>, Option<&'devices str>)]),
    }

    /// 照 `pci_bus` 建一棵假的 sysfs，交回 `(根, 总线目录 <根>/bus/pci)`；用完由调用方删根。
    fn fake_sysfs(
        test_name: &str,
        pci_bus: &FakePciBus<'_>,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "singlefs-gpu-test-start-sysfs-{}-{test_name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        let bus_directory = root.join("bus").join("pci");
        std::fs::create_dir_all(root.join("bus")).expect("临时目录建得出");
        match pci_bus {
            FakePciBus::NotThere => {}
            FakePciBus::ThereWithoutItsDevicesDirectory => {
                std::fs::create_dir_all(&bus_directory).expect("临时目录建得出");
            }
            FakePciBus::ThereWithDevices(devices) => {
                for (address, vendor_text, class_text) in *devices {
                    let device_directory = bus_directory
                        .join(PCI_DEVICES_SUBDIRECTORY_NAME)
                        .join(address);
                    std::fs::create_dir_all(&device_directory).expect("临时目录建得出");
                    if let Some(vendor_text) = vendor_text {
                        std::fs::write(device_directory.join("vendor"), vendor_text)
                            .expect("写得进临时目录");
                    }
                    if let Some(class_text) = class_text {
                        std::fs::write(device_directory.join("class"), class_text)
                            .expect("写得进临时目录");
                    }
                }
            }
        }
        (root, bus_directory)
    }

    #[test]
    fn no_nvidia_adapter_and_an_invisible_pci_bus_is_reported_as_not_run() {
        let start = decide_gpu_test_start(
            &[],
            &PciNvidiaDisplayControllers::PciBusNotVisible {
                reason: "假的：/sys/bus/pci 不在".to_string(),
            },
            0,
        );
        assert!(
            matches!(start, GpuTestStart::NotRunNoGpuAdapter { .. }),
            "Vulkan 里没有 NVIDIA 独显、又看不见 PCI 总线的机器没有这种硬件，要报「本次未跑」；判成了 {start:?}"
        );
    }

    #[test]
    fn a_pci_bus_directory_that_is_not_there_is_read_as_the_bus_not_being_visible() {
        let (root, bus_directory) = fake_sysfs("bus-not-there", &FakePciBus::NotThere);
        let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            matches!(read, PciNvidiaDisplayControllers::PciBusNotVisible { .. }),
            "总线目录整个不在是看不见 PCI 总线，不是设备表读不出：{read:?}"
        );
    }

    #[test]
    fn a_visible_pci_bus_without_its_devices_directory_is_an_unreadable_listing_not_an_invisible_bus(
    ) {
        let (root, bus_directory) = fake_sysfs(
            "bus-without-devices",
            &FakePciBus::ThereWithoutItsDevicesDirectory,
        );
        let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            matches!(read, PciNvidiaDisplayControllers::ListingUnreadable { .. }),
            "总线目录在、设备子目录列不出是读不出，不许当成看不见总线：{read:?}"
        );
    }

    #[test]
    fn a_visible_pci_bus_with_a_device_whose_vendor_cannot_be_read_and_no_nvidia_adapter_is_red() {
        let (root, bus_directory) = fake_sysfs(
            "device-vendor-unreadable",
            &FakePciBus::ThereWithDevices(&[("0000:00:1f.3", None, Some("0x040300\n"))]),
        );
        let read = read_pci_nvidia_display_controllers_on_the_bus(&bus_directory);
        let _ = std::fs::remove_dir_all(&root);
        let start = decide_gpu_test_start(&[], &read, 0);
        assert!(
            matches!(
                start,
                GpuTestStart::PciBusVisibleButListingUnreadable { .. }
            ),
            "总线看得见而某个设备的 vendor 读不出，分不清没卡与驱动坏，照旧判红、不报「本次未跑」；读到 {read:?}，判成了 {start:?}"
        );
    }
}
