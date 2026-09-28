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

/// 一张起好上下文的卡。
pub struct GpuCard {
    pub name: String,
    pub pci_bus_identifier: String,
    /// 起上下文那一刻 `nvidia-smi` 报的空闲显存（MiB）；按它分工。
    pub free_memory_mebibytes: u64,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

/// `nvidia-smi` 报的每张卡：PCI 总线号（小写、去掉前导的 0 域）→ 空闲显存 MiB。起不来 `nvidia-smi` 就是空表。
fn free_memory_by_pci_bus() -> std::collections::BTreeMap<String, u64> {
    let Ok(output) = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=pci.bus_id,memory.free",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return std::collections::BTreeMap::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let (bus, free) = line.split_once(',')?;
            Some((normalized_bus(bus), free.trim().parse().ok()?))
        })
        .collect()
}

/// 总线号只留 `总线:设备.功能` 那一段、小写：`00000000:01:00.0` 与 `0000:01:00.0` 认成同一张卡。
fn normalized_bus(text: &str) -> String {
    let lowered = text.trim().to_ascii_lowercase();
    let parts: Vec<&str> = lowered.split(':').collect();
    parts[parts.len().saturating_sub(2)..].join(":")
}

/// 本机能用的卡：Vulkan 下的 NVIDIA 独显，空闲显存不低于 [`MINIMUM_FREE_MEMORY_MEBIBYTES`]，上下文起得来；按空闲显存从大到小排。
#[must_use]
pub fn usable_gpu_cards() -> Vec<GpuCard> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let free_by_bus = free_memory_by_pci_bus();
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
                let free = *free_by_bus.get(&normalized_bus(&information.device_pci_bus_id))?;
                if free < MINIMUM_FREE_MEMORY_MEBIBYTES {
                    return None;
                }
                let (device, queue) =
                    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                        label: Some("singlefs-gpu-unit-checks"),
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
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
                    free_memory_mebibytes: free,
                    device,
                    queue,
                    pipeline,
                })
            })
            .collect();
    cards.sort_by_key(|card| std::cmp::Reverse(card.free_memory_mebibytes));
    cards
}

impl GpuCard {
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
