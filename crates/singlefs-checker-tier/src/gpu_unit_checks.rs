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
const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

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
