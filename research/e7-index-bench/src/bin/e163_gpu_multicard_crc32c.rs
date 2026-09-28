//! E163（GPU 多卡算单元校验和）：单机多卡、双机多卡能不能把一批 32768 字节单元的
//! CRC-32C 切开算、合并后与 CPU 参照逐个相同；比对本身会不会红。
//!
// admission: always 每一次跑都是新的适配器枚举、新的显存分配与新的计时，上一次的结论不替这一次作保
// run-condition: command cargo
//!
//! 跑前登记：`research/prompts/e163-preregistration.md`。岔路单：`research/prompts/m2-gpu-multicard-r1-forks.md`。
//!
//! 独立手写模型，不与 `crates/` 共用代码（`.claude/rules/implementation-first.md` 第 4 条）：
//! 臂 C（CPU 参照）与臂 G（着色器）各自实现 CRC-32C，互不调用、不共用表。
//! 参数照 D18（块里携带什么信息） 已定项 17：多项式 0x1EDC6F41（反射 0x82F63B78）、
//! 输入输出反射、初值 0xFFFFFFFF、输出取反——两份实现都在本文件里，逐处标注行号供执行员核对。

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Instant;

// ============================== D18 已定项 17：CRC-32C 参数（A 类断言的出处） ==============================

/// 反射多项式 0x1EDC6F41 的反射形式；臂 C（下面的 `crc32c_reference`）与臂 G（WGSL 着色器源码
/// 里的同名常量）各自持有一份，不共用，用来在健康输入上互证。
const CASTAGNOLI_REFLECTED_POLYNOMIAL: u32 = 0x82F6_3B78;
/// 初值 0xFFFFFFFF。
const CASTAGNOLI_INITIAL_VALUE: u32 = 0xFFFF_FFFF;

/// 臂 C：按位、反射输入输出、初值 0xFFFFFFFF、输出取反的 CRC-32C 参照实现。
/// 不查表、不与臂 G（WGSL 着色器）共用任何代码或数据。
fn crc32c_reference(bytes: &[u8]) -> u32 {
    let mut register: u32 = CASTAGNOLI_INITIAL_VALUE;
    for &current_byte in bytes {
        register ^= u32::from(current_byte);
        for _ in 0..8 {
            let low_bit_is_set = register & 1 == 1;
            register = if low_bit_is_set {
                (register >> 1) ^ CASTAGNOLI_REFLECTED_POLYNOMIAL
            } else {
                register >> 1
            };
        }
    }
    register ^ CASTAGNOLI_INITIAL_VALUE
}

/// 跑前登记第七节 B 类锚点：八个输入与它们各自的 CRC-32C，命令核过（登记第十三节）。
const EXPECTED_ANCHOR_VALUES: [u32; 8] = [
    0xE306_9283, // ASCII "123456789"
    0x0000_0000, // 空串
    0x8A91_36AA, // 32 × 0x00
    0x62A8_AB43, // 32 × 0xFF
    0x46DD_794E, // 0x00..0x1F 升序
    0x113F_DB5C, // 0x1F..0x00 降序
    0xBC43_BAAD, // 32768 × 0x00
    0x434C_2368, // 32768 × 0xFF
];

const ANCHOR_VECTOR_NAMES: [&str; 8] = [
    "ascii_123456789",
    "empty",
    "thirty_two_zero_bytes",
    "thirty_two_ff_bytes",
    "thirty_two_ascending",
    "thirty_two_descending",
    "unit_length_zero_bytes",
    "unit_length_ff_bytes",
];

/// 跑前登记第七节 B 类的八个锚点向量，次序与 `EXPECTED_ANCHOR_VALUES`、`ANCHOR_VECTOR_NAMES` 一致。
fn anchor_vectors() -> [Vec<u8>; 8] {
    [
        b"123456789".to_vec(),
        Vec::new(),
        vec![0x00u8; 32],
        vec![0xFFu8; 32],
        (0u8..32).collect(),
        (0u8..32).rev().collect(),
        vec![0x00u8; UNIT_BYTE_LENGTH],
        vec![0xFFu8; UNIT_BYTE_LENGTH],
    ]
}

/// 逐个核对臂 C（或任意「按同一份参照实现算出来的」序列）与登记的八个锚点是否一致。
fn anchor_mismatches(computed: &[u32; 8]) -> Vec<usize> {
    (0..8)
        .filter(|&index| computed[index] != EXPECTED_ANCHOR_VALUES[index])
        .collect()
}

// ============================== 批的几何与生成 ==============================

/// 单元大小（登记「读法写死」：对整 32768 字节求 CRC-32C）。
const UNIT_BYTE_LENGTH: usize = 32_768;
/// 每个单元占的 u32 字数：输入缓冲区按 `array<u32>` 传给着色器（登记第五节「臂 G」）。
const WORDS_PER_UNIT_SLOT: usize = UNIT_BYTE_LENGTH / 4;

const SPLITMIX64_INCREMENT: u64 = 0x9E37_79B9_7F4A_7C15;
const SPLITMIX64_FIRST_MULTIPLIER: u64 = 0xBF58_476D_1CE4_E5B9;
const SPLITMIX64_SECOND_MULTIPLIER: u64 = 0x94D0_49BB_1331_11EB;

fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(SPLITMIX64_INCREMENT);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(SPLITMIX64_FIRST_MULTIPLIER);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(SPLITMIX64_SECOND_MULTIPLIER);
    mixed ^ (mixed >> 31)
}

/// 单元 i 的内容 = splitmix64 以 `种子 XOR i·0x9E3779B97F4A7C15` 为初态的输出按小端铺满
/// 32768 字节（登记第五节「批」）。
fn generate_unit_bytes(seed: u64, global_unit_identifier: u64) -> Vec<u8> {
    let mut state = seed ^ global_unit_identifier.wrapping_mul(SPLITMIX64_INCREMENT);
    let mut bytes = Vec::with_capacity(UNIT_BYTE_LENGTH);
    while bytes.len() < UNIT_BYTE_LENGTH {
        bytes.extend_from_slice(&splitmix64_next(&mut state).to_le_bytes());
    }
    bytes.truncate(UNIT_BYTE_LENGTH);
    bytes
}

/// 往一份「上传副本」字节里翻一个位，不touch 磁盘上的输入文件（M3 的 F1 注入，登记第五节「跑 F1、F2」）。
fn apply_bit_flip(bytes: &mut [u8], byte_offset_within_unit: usize, bit_index_within_byte: u32) {
    bytes[byte_offset_within_unit] ^= 1u8 << bit_index_within_byte;
}

// ============================== 输入批文件格式 ==============================

const BATCH_FILE_MAGIC: &[u8; 8] = b"E163IN01";

fn write_batch_file(output_path: &PathBuf, unit_count: u64, seed: u64) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    file.write_all(BATCH_FILE_MAGIC)?;
    file.write_all(&unit_count.to_le_bytes())?;
    file.write_all(&(UNIT_BYTE_LENGTH as u64).to_le_bytes())?;
    file.write_all(&seed.to_le_bytes())?;
    for global_unit_identifier in 0..unit_count {
        let unit_bytes = generate_unit_bytes(seed, global_unit_identifier);
        file.write_all(&unit_bytes)?;
    }
    file.flush()
}

struct BatchFileHeader {
    unit_count: u64,
    unit_byte_length: u64,
    seed: u64,
    data_start_offset: u64,
}

fn read_batch_header(input_path: &PathBuf) -> std::io::Result<BatchFileHeader> {
    let mut file = File::open(input_path)?;
    let mut magic = [0u8; 8];
    file.read_exact(&mut magic)?;
    assert_eq!(&magic, BATCH_FILE_MAGIC, "输入文件头魔数不对：{input_path:?}");
    let mut eight_bytes = [0u8; 8];
    file.read_exact(&mut eight_bytes)?;
    let unit_count = u64::from_le_bytes(eight_bytes);
    file.read_exact(&mut eight_bytes)?;
    let unit_byte_length = u64::from_le_bytes(eight_bytes);
    file.read_exact(&mut eight_bytes)?;
    let seed = u64::from_le_bytes(eight_bytes);
    Ok(BatchFileHeader { unit_count, unit_byte_length, seed, data_start_offset: 8 + 8 + 8 + 8 })
}

/// 从批文件里读出 `[start, start+count)` 这一段单元，拼成 `count * UNIT_BYTE_LENGTH` 字节的连续缓冲区。
fn read_unit_range(input_path: &PathBuf, header: &BatchFileHeader, start: u64, count: u64) -> std::io::Result<Vec<u8>> {
    assert_eq!(header.unit_byte_length as usize, UNIT_BYTE_LENGTH, "批文件的单元字节数与本装置写死的不一致");
    assert_eq!(UNIT_BYTE_LENGTH, WORDS_PER_UNIT_SLOT * 4, "字节数与字数的换算与 WGSL 着色器里那份写死的常量必须一致");
    assert!(start + count <= header.unit_count, "读的区间超出批文件的单元数");
    let mut file = File::open(input_path)?;
    file.seek(SeekFrom::Start(header.data_start_offset + start * UNIT_BYTE_LENGTH as u64))?;
    let mut bytes = vec![0u8; (count as usize) * UNIT_BYTE_LENGTH];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

// ============================== 「卡」的定义与适配器枚举 ==============================

/// 「卡」= wgpu 在 Vulkan 后端枚举出的、`vendor == 0x10DE` 且 `device_type == DiscreteGpu`
/// 的适配器；软件适配器（llvmpipe 一类）与 CPU 类型不算卡（登记「读法写死」）。
const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;

fn new_vulkan_instance() -> wgpu::Instance {
    wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::VULKAN, ..wgpu::InstanceDescriptor::new_without_display_handle() })
}

fn nvidia_discrete_adapters(instance: &wgpu::Instance) -> Vec<wgpu::Adapter> {
    pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
        .into_iter()
        .filter(|adapter| {
            let adapter_information = adapter.get_info();
            adapter_information.vendor == NVIDIA_VENDOR_IDENTIFIER && adapter_information.device_type == wgpu::DeviceType::DiscreteGpu
        })
        .collect()
}

fn adapter_information_fields(adapter_information: &wgpu::AdapterInfo) -> String {
    format!(
        "name={:?} vendor=0x{:X} device=0x{:X} device_type={:?} driver={:?} driver_info={:?} backend={:?}",
        adapter_information.name,
        adapter_information.vendor,
        adapter_information.device,
        adapter_information.device_type,
        adapter_information.driver,
        adapter_information.driver_info,
        adapter_information.backend
    )
}

// ============================== 臂 G：GPU 上下文与派发 ==============================

/// WGSL 计算着色器：每个调用算一个单元，字节从 `array<u32>` 里按
/// `(word >> 8*(k%4)) & 0xFF` 取（登记第五节「臂 G」），CRC-32C 参数与臂 C 逐一对得上：
/// 反射多项式 0x82F63B78、初值 0xFFFFFFFF、输出取反——这四样与本文件顶部 `crc32c_reference`
/// 用的是同一组常量值，两处各自持有一份（A 类断言，执行员贴这两处源码行核对）。
const COMPUTE_SHADER_SOURCE: &str = r#"
struct Params {
    unit_count: u32,
    reserved0: u32,
    reserved1: u32,
    reserved2: u32,
};

struct ResultEntry {
    crc: u32,
    local_offset: u32,
};

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> unit_words: array<u32>;
@group(0) @binding(2) var<storage, read> unit_lengths: array<u32>;
@group(0) @binding(3) var<storage, read_write> results: array<ResultEntry>;

const CRC32C_REFLECTED_POLYNOMIAL: u32 = 0x82F63B78u;
const CRC32C_INITIAL_VALUE: u32 = 0xFFFFFFFFu;
const WORDS_PER_UNIT_SLOT: u32 = 8192u; // 32768 字节 / 4

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let local_offset = global_id.x;
    if (local_offset >= params.unit_count) {
        return;
    }
    let length_in_bytes = unit_lengths[local_offset];
    let base_word = local_offset * WORDS_PER_UNIT_SLOT;
    var crc_register: u32 = CRC32C_INITIAL_VALUE;
    var byte_index: u32 = 0u;
    loop {
        if (byte_index >= length_in_bytes) {
            break;
        }
        let word_index = base_word + byte_index / 4u;
        let word = unit_words[word_index];
        let shift = 8u * (byte_index % 4u);
        let current_byte = (word >> shift) & 0xFFu;
        crc_register = crc_register ^ current_byte;
        var bit_index: u32 = 0u;
        loop {
            if (bit_index >= 8u) {
                break;
            }
            let low_bit_is_set = (crc_register & 1u) == 1u;
            if (low_bit_is_set) {
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

const RESULT_ENTRY_BYTE_LENGTH: usize = 8; // crc: u32 + local_offset: u32
const WORKGROUP_SIZE: u32 = 64;
const SENTINEL_WORD: u32 = 0xFFFF_FFFF;

struct GpuContext {
    adapter_information: wgpu::AdapterInfo,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

/// 起得来上下文＝`request_device` 成功且第一块缓冲区分配成功（登记「读法写死」）。
/// 失败时把 wgpu 原样错误报回去，不换配置重试（登记第五节「对面那条臂」）。
fn build_gpu_context(adapter: wgpu::Adapter) -> Result<GpuContext, String> {
    let adapter_information = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("e163-gpu-multicard-crc32c"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| format!("request_device 失败：{error:?}"))?;

    // 「第一块缓冲区分配成功」：起一个很小的 storage 缓冲区探一次，探不到就是这张卡起不来上下文。
    let probe_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("e163-probe-buffer"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });
    drop(probe_buffer);

    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("e163-crc32c-shader"),
        source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER_SOURCE.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("e163-crc32c-pipeline"),
        layout: None,
        module: &shader_module,
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    });
    Ok(GpuContext { adapter_information, device, queue, pipeline })
}

fn u32_slice_to_le_bytes(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// 派发一批单元给 GPU 算 CRC-32C。`unit_slots` 是 `count * UNIT_BYTE_LENGTH` 字节的连续缓冲区
/// （每个单元占一个满槽，不足的用 `lengths[i]` 截断，供 `anchors` 短输入复用同一条路径）；
/// 返回长度为 `count` 的 `(crc, local_offset)`，`local_offset` 取自着色器里的
/// `global_invocation_id.x`（登记第五节「臂 G」）。
///
/// 派发前结果缓冲区整片填哨兵 `0xFFFFFFFF`（登记同节），调用方用 `assert_full_coverage_no_sentinel`
/// 核「没有哨兵残留、`local_offset` 等于数组下标」这两条钉死的断言（登记第七节「另三条」）。
fn run_gpu_crc(context: &GpuContext, unit_slots: &[u8], lengths: &[u32]) -> Vec<(u32, u32)> {
    let count = lengths.len();
    assert_eq!(unit_slots.len(), count * UNIT_BYTE_LENGTH, "输入槽长度与单元数不匹配");
    let device = &context.device;
    let queue = &context.queue;

    let dispatch_parameters = [count as u32, 0u32, 0u32, 0u32];
    let parameter_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("e163-params"),
            contents: &u32_slice_to_le_bytes(&dispatch_parameters),
            usage: wgpu::BufferUsages::UNIFORM,
        },
    );
    let input_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor { label: Some("e163-input"), contents: unit_slots, usage: wgpu::BufferUsages::STORAGE },
    );
    let lengths_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor { label: Some("e163-lengths"), contents: &u32_slice_to_le_bytes(lengths), usage: wgpu::BufferUsages::STORAGE },
    );
    let sentinel_bytes = vec![0xFFu8; count * RESULT_ENTRY_BYTE_LENGTH];
    let result_buffer = wgpu::util::DeviceExt::create_buffer_init(
        device,
        &wgpu::util::BufferInitDescriptor {
            label: Some("e163-results"),
            contents: &sentinel_bytes,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        },
    );
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("e163-staging"),
        size: (count * RESULT_ENTRY_BYTE_LENGTH) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = context.pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("e163-bind-group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: parameter_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: input_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: lengths_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: result_buffer.as_entire_binding() },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("e163-encoder") });
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("e163-pass"), timestamp_writes: None });
        compute_pass.set_pipeline(&context.pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        let workgroup_count = count.div_ceil(WORKGROUP_SIZE as usize) as u32;
        compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
    }
    encoder.copy_buffer_to_buffer(&result_buffer, 0, &staging_buffer, 0, (count * RESULT_ENTRY_BYTE_LENGTH) as u64);
    queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).expect("回读结果的 map_async 回调发送失败");
    });
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).expect("device.poll 失败");
    receiver.recv().expect("map_async 回调没有送达").expect("map_async 报告失败");

    let mapped = slice.get_mapped_range().expect("读回结果缓冲区失败");
    let mut results = Vec::with_capacity(count);
    for chunk in mapped.chunks_exact(RESULT_ENTRY_BYTE_LENGTH) {
        let crc = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        let local_offset = u32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]);
        results.push((crc, local_offset));
    }
    drop(mapped);
    staging_buffer.unmap();
    results
}

/// 登记第七节「另三条」：`gid` = 片内偏移（没有重排）、没有哨兵残留。
/// 违反就是作废 2（装置或这一次派发本身错了），panic 让调用方（run-shard / 单测）都能看见。
fn assert_full_coverage_no_sentinel(results: &[(u32, u32)]) {
    for (index, &(crc, local_offset)) in results.iter().enumerate() {
        assert_eq!(local_offset as usize, index, "第 {index} 条结果的 local_offset 不等于数组下标，gid 断言不过");
        assert!(!(crc == SENTINEL_WORD && local_offset == SENTINEL_WORD), "第 {index} 条结果仍是哨兵值，覆盖数不足");
    }
}

// ============================== 一片（shard）跑出来的结果文件 ==============================

/// 一条注入的翻位：全局单元号、单元内字节偏移、字节内位号（登记第五节「跑 F1、F2」）。
#[derive(Clone, Copy)]
struct BitFlipInjection {
    global_unit_identifier: u64,
    byte_offset_within_unit: usize,
    bit_index_within_byte: u32,
}

struct ShardResultRecord {
    global_unit_identifier: u64,
    crc: u32,
}

/// 一片的结果文件：文本格式，头部若干 `key=value` 行，一个 `# results` 分隔行，随后每行
/// 一条 `<global_unit_identifier> <crc_hex>`（登记第五节「计时」「传输」，第九节变异 V9 的截断检测在
/// merge 阶段按行数与 sha256 一起判）。
fn write_shard_result_file(
    output_path: &PathBuf,
    adapter_information: &wgpu::AdapterInfo,
    shard_start: u64,
    shard_count: u64,
    build_device_milliseconds: f64,
    transfer_compute_readback_milliseconds: f64,
    total_milliseconds: f64,
    injected_flips: &[BitFlipInjection],
    injected_drops: &[u64],
    sentinel_and_gid_ok: bool,
    anchors_cpu_match: bool,
    anchors_gpu_match: bool,
    records: &[ShardResultRecord],
) -> std::io::Result<()> {
    let mut file = File::create(output_path)?;
    writeln!(file, "name={:?}", adapter_information.name)?;
    writeln!(file, "vendor=0x{:X}", adapter_information.vendor)?;
    writeln!(file, "device=0x{:X}", adapter_information.device)?;
    writeln!(file, "device_type={:?}", adapter_information.device_type)?;
    writeln!(file, "driver={:?}", adapter_information.driver)?;
    writeln!(file, "driver_info={:?}", adapter_information.driver_info)?;
    writeln!(file, "backend={:?}", adapter_information.backend)?;
    writeln!(file, "start={shard_start}")?;
    writeln!(file, "count={shard_count}")?;
    writeln!(file, "build_device_ms={build_device_milliseconds:.3}")?;
    writeln!(file, "transfer_compute_readback_ms={transfer_compute_readback_milliseconds:.3}")?;
    writeln!(file, "total_ms={total_milliseconds:.3}")?;
    if injected_flips.is_empty() {
        writeln!(file, "injected_flip=none")?;
    } else {
        let flips: Vec<String> = injected_flips
            .iter()
            .map(|flip| format!("{}:{}:{}", flip.global_unit_identifier, flip.byte_offset_within_unit, flip.bit_index_within_byte))
            .collect();
        writeln!(file, "injected_flip={}", flips.join(","))?;
    }
    if injected_drops.is_empty() {
        writeln!(file, "injected_drop=none")?;
    } else {
        let drops: Vec<String> = injected_drops.iter().map(u64::to_string).collect();
        writeln!(file, "injected_drop={}", drops.join(","))?;
    }
    writeln!(file, "sentinel_and_gid_ok={sentinel_and_gid_ok}")?;
    writeln!(file, "anchors_cpu_match={anchors_cpu_match}")?;
    writeln!(file, "anchors_gpu_match={anchors_gpu_match}")?;
    writeln!(file, "record_count={}", records.len())?;
    writeln!(file, "# results")?;
    for record in records {
        writeln!(file, "{} {:08X}", record.global_unit_identifier, record.crc)?;
    }
    file.flush()
}

fn read_shard_result_file(path: &PathBuf) -> std::io::Result<Vec<ShardResultRecord>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut records = Vec::new();
    let mut past_header = false;
    for line in reader.lines() {
        let line = line?;
        if !past_header {
            if line == "# results" {
                past_header = true;
            }
            continue;
        }
        let mut parts = line.split_whitespace();
        let global_unit_identifier: u64 = parts.next().expect("结果行缺全局单元号").parse().expect("全局单元号不是数字");
        let crc_hex = parts.next().expect("结果行缺 CRC 十六进制值");
        let crc = u32::from_str_radix(crc_hex, 16).expect("CRC 不是合法的十六进制");
        records.push(ShardResultRecord { global_unit_identifier, crc });
    }
    Ok(records)
}

// ============================== 合并比对（arm C 对 arm G，登记第六节 Q2/Q5/Q8/Q9） ==============================

#[derive(Default, Debug)]
struct MergeReport {
    matched_count: usize,
    mismatched_global_unit_identifiers: Vec<u64>,
    missing_global_unit_identifiers: Vec<u64>,
    duplicated_global_unit_identifiers: Vec<u64>,
}

/// 纯函数，不碰 GPU、不碰文件：拿臂 C 算好的参照值（下标即全局单元号）与几片臂 G 的结果，
/// 逐个全局单元号比对。这是 V1（合并比对恒判相同）与 V2（不按 0..N 查缺）两条变异要抓的函数。
fn compare_reference_and_shards(cpu_reference: &[u32], shard_records: &[&[ShardResultRecord]]) -> MergeReport {
    let total_units = cpu_reference.len() as u64;
    let mut seen_crc_by_unit: std::collections::HashMap<u64, Vec<u32>> = std::collections::HashMap::new();
    for shard in shard_records {
        for record in *shard {
            seen_crc_by_unit.entry(record.global_unit_identifier).or_default().push(record.crc);
        }
    }
    let mut report = MergeReport::default();
    for global_unit_identifier in 0..total_units {
        match seen_crc_by_unit.get(&global_unit_identifier) {
            None => report.missing_global_unit_identifiers.push(global_unit_identifier),
            Some(values) if values.len() > 1 => report.duplicated_global_unit_identifiers.push(global_unit_identifier),
            Some(values) => {
                if values[0] == cpu_reference[global_unit_identifier as usize] {
                    report.matched_count += 1;
                } else {
                    report.mismatched_global_unit_identifiers.push(global_unit_identifier);
                }
            }
        }
    }
    report
}

/// 臂 C：对批文件里全部 `total_units` 个单元逐个求 `crc32c_reference`，下标即全局单元号。
fn compute_cpu_reference_for_all_units(input_path: &PathBuf, header: &BatchFileHeader, total_units: u64) -> Vec<u32> {
    let all_bytes = read_unit_range(input_path, header, 0, total_units).expect("读取整批输入失败");
    (0..total_units as usize)
        .map(|unit_index| {
            let start = unit_index * UNIT_BYTE_LENGTH;
            crc32c_reference(&all_bytes[start..start + UNIT_BYTE_LENGTH])
        })
        .collect()
}

// ============================== 命令行 ==============================

fn parse_named_argument(arguments: &[String], name: &str) -> Option<String> {
    arguments.iter().position(|argument| argument == name).map(|index| arguments[index + 1].clone())
}

fn parse_repeated_named_argument(arguments: &[String], name: &str) -> Vec<String> {
    arguments
        .iter()
        .zip(arguments.iter().skip(1))
        .filter(|(flag, _)| *flag == name)
        .map(|(_, value)| value.clone())
        .collect()
}

fn parse_bit_flip_specification(specification: &str) -> BitFlipInjection {
    let mut parts = specification.split(':');
    let global_unit_identifier: u64 = parts.next().expect("翻位注入缺全局单元号").parse().expect("全局单元号不是数字");
    let byte_offset_within_unit: usize = parts.next().expect("翻位注入缺字节偏移").parse().expect("字节偏移不是数字");
    let bit_index_within_byte: u32 = parts.next().expect("翻位注入缺位号").parse().expect("位号不是数字");
    BitFlipInjection { global_unit_identifier, byte_offset_within_unit, bit_index_within_byte }
}

/// `probe`：只枚举、不建设备（登记第六节 Q4：另一台上「--probe」的退出码、stderr、列出的 NVIDIA 卡数）。
fn command_probe() {
    let instance = new_vulkan_instance();
    let adapters = nvidia_discrete_adapters(&instance);
    for adapter in &adapters {
        println!("PROBE {}", adapter_information_fields(&adapter.get_info()));
    }
    println!("PROBE_COUNT={}", adapters.len());
}

/// `anchors --adapter-index N`：算臂 C 与第 N 张 NVIDIA 独显卡（枚举序）的第七节 B 类八锚点，
/// 都与登记的 `EXPECTED_ANCHOR_VALUES` 比对（PC-A，登记第五节「阳性对照」）。
fn command_anchors(arguments: &[String]) {
    let adapter_index: usize = parse_named_argument(arguments, "--adapter-index").expect("anchors 需要 --adapter-index").parse().expect("adapter-index 不是数字");

    let vectors = anchor_vectors();
    let cpu_values: [u32; 8] = std::array::from_fn(|index| crc32c_reference(&vectors[index]));
    let cpu_mismatches = anchor_mismatches(&cpu_values);
    for index in 0..8 {
        println!("ANCHOR_CPU name={} expected={:08X} got={:08X}", ANCHOR_VECTOR_NAMES[index], EXPECTED_ANCHOR_VALUES[index], cpu_values[index]);
    }
    println!("ANCHOR_CPU_MATCH={}", cpu_mismatches.is_empty());

    let instance = new_vulkan_instance();
    let adapters = nvidia_discrete_adapters(&instance);
    let adapter = adapters.into_iter().nth(adapter_index).unwrap_or_else(|| panic!("枚举不到第 {adapter_index} 张 NVIDIA 独显卡"));
    let context = build_gpu_context(adapter).unwrap_or_else(|error| panic!("第 {adapter_index} 张卡建不起上下文：{error}"));
    println!("ANCHOR_ADAPTER {}", adapter_information_fields(&context.adapter_information));

    let lengths: [u32; 8] = std::array::from_fn(|index| vectors[index].len() as u32);
    let mut unit_slots = vec![0u8; 8 * UNIT_BYTE_LENGTH];
    for (index, vector) in vectors.iter().enumerate() {
        unit_slots[index * UNIT_BYTE_LENGTH..index * UNIT_BYTE_LENGTH + vector.len()].copy_from_slice(vector);
    }
    let gpu_results = run_gpu_crc(&context, &unit_slots, &lengths);
    assert_full_coverage_no_sentinel(&gpu_results);
    let gpu_values: [u32; 8] = std::array::from_fn(|index| gpu_results[index].0);
    let gpu_mismatches = anchor_mismatches(&gpu_values);
    for index in 0..8 {
        println!("ANCHOR_GPU name={} expected={:08X} got={:08X}", ANCHOR_VECTOR_NAMES[index], EXPECTED_ANCHOR_VALUES[index], gpu_values[index]);
    }
    println!("ANCHOR_GPU_MATCH={}", gpu_mismatches.is_empty());
    if !cpu_mismatches.is_empty() || !gpu_mismatches.is_empty() {
        std::process::exit(1);
    }
}

fn command_generate_input(arguments: &[String]) {
    let output_path = PathBuf::from(parse_named_argument(arguments, "--output").expect("gen-input 需要 --output"));
    let units: u64 = parse_named_argument(arguments, "--units").expect("gen-input 需要 --units").parse().expect("units 不是数字");
    let seed_text = parse_named_argument(arguments, "--seed").expect("gen-input 需要 --seed");
    let seed_text = seed_text.strip_prefix("0x").unwrap_or(&seed_text);
    let seed = u64::from_str_radix(seed_text, 16).expect("seed 不是合法的十六进制");
    write_batch_file(&output_path, units, seed).expect("写批文件失败");
    println!("GEN_INPUT path={output_path:?} units={units} seed=0x{seed:016X} bytes={}", units as usize * UNIT_BYTE_LENGTH + 32);
}

/// `run-shard`：一片的完整流程——PC-A 锚点、读输入区间、按需翻位（只改上传副本）、GPU 派发、
/// 覆盖断言、按需丢结果行、写结果文件（登记第五节「跑 R1」「跑 R2」「跑 F1、F2」）。
fn command_run_shard(arguments: &[String]) {
    let input_path = PathBuf::from(parse_named_argument(arguments, "--input").expect("run-shard 需要 --input"));
    let start: u64 = parse_named_argument(arguments, "--start").expect("run-shard 需要 --start").parse().expect("start 不是数字");
    let count: u64 = parse_named_argument(arguments, "--count").expect("run-shard 需要 --count").parse().expect("count 不是数字");
    let adapter_index: usize = parse_named_argument(arguments, "--adapter-index").expect("run-shard 需要 --adapter-index").parse().expect("adapter-index 不是数字");
    let output_path = PathBuf::from(parse_named_argument(arguments, "--output").expect("run-shard 需要 --output"));
    let flips: Vec<BitFlipInjection> = parse_repeated_named_argument(arguments, "--inject-flip").iter().map(|specification| parse_bit_flip_specification(specification)).collect();
    let drops: Vec<u64> = parse_repeated_named_argument(arguments, "--inject-drop").iter().map(|text| text.parse().expect("drop 的全局单元号不是数字")).collect();

    let total_start = Instant::now();

    let instance = new_vulkan_instance();
    let adapters = nvidia_discrete_adapters(&instance);
    let adapter_count = adapters.len();
    let adapter = adapters
        .into_iter()
        .nth(adapter_index)
        .unwrap_or_else(|| panic!("枚举到 {adapter_count} 张 NVIDIA 独显卡，第 {adapter_index} 张不存在"));

    let build_device_start = Instant::now();
    let context = build_gpu_context(adapter).unwrap_or_else(|error| panic!("第 {adapter_index} 张卡建不起上下文：{error}"));
    let build_device_milliseconds = build_device_start.elapsed().as_secs_f64() * 1000.0;
    eprintln!("RUN_SHARD_ADAPTER index={adapter_index} {}", adapter_information_fields(&context.adapter_information));

    // PC-A：这一次跑开头，臂 C 与这张卡都算第七节 B 类八锚点。
    let vectors = anchor_vectors();
    let cpu_anchor_values: [u32; 8] = std::array::from_fn(|index| crc32c_reference(&vectors[index]));
    let anchors_cpu_match = anchor_mismatches(&cpu_anchor_values).is_empty();
    let anchor_lengths: [u32; 8] = std::array::from_fn(|index| vectors[index].len() as u32);
    let mut anchor_slots = vec![0u8; 8 * UNIT_BYTE_LENGTH];
    for (index, vector) in vectors.iter().enumerate() {
        anchor_slots[index * UNIT_BYTE_LENGTH..index * UNIT_BYTE_LENGTH + vector.len()].copy_from_slice(vector);
    }
    let anchor_gpu_results = run_gpu_crc(&context, &anchor_slots, &anchor_lengths);
    assert_full_coverage_no_sentinel(&anchor_gpu_results);
    let gpu_anchor_values: [u32; 8] = std::array::from_fn(|index| anchor_gpu_results[index].0);
    let anchors_gpu_match = anchor_mismatches(&gpu_anchor_values).is_empty();

    let header = read_batch_header(&input_path).expect("读批文件头失败");
    eprintln!("RUN_SHARD_INPUT_HEADER seed=0x{:016X} unit_count={}", header.seed, header.unit_count);
    let mut unit_slots = read_unit_range(&input_path, &header, start, count).expect("读输入区间失败");
    for flip in &flips {
        assert!(flip.global_unit_identifier >= start && flip.global_unit_identifier < start + count, "翻位注入的单元号不在这一片范围内");
        let local_index = (flip.global_unit_identifier - start) as usize;
        let slot_start = local_index * UNIT_BYTE_LENGTH;
        apply_bit_flip(&mut unit_slots[slot_start..slot_start + UNIT_BYTE_LENGTH], flip.byte_offset_within_unit, flip.bit_index_within_byte);
    }
    let lengths = vec![UNIT_BYTE_LENGTH as u32; count as usize];

    let transfer_compute_readback_start = Instant::now();
    let gpu_results = run_gpu_crc(&context, &unit_slots, &lengths);
    let transfer_compute_readback_milliseconds = transfer_compute_readback_start.elapsed().as_secs_f64() * 1000.0;

    let mut sentinel_and_gid_ok = true;
    for (index, &(crc, local_offset)) in gpu_results.iter().enumerate() {
        if local_offset as usize != index || (crc == SENTINEL_WORD && local_offset == SENTINEL_WORD) {
            sentinel_and_gid_ok = false;
        }
    }
    assert!(sentinel_and_gid_ok, "这一片的覆盖 / gid 断言不过（作废 2）");

    let records: Vec<ShardResultRecord> = gpu_results
        .iter()
        .enumerate()
        .map(|(local_offset, &(crc, _))| ShardResultRecord { global_unit_identifier: start + local_offset as u64, crc })
        .filter(|record| !drops.contains(&record.global_unit_identifier))
        .collect();

    let total_milliseconds = total_start.elapsed().as_secs_f64() * 1000.0;
    write_shard_result_file(
        &output_path,
        &context.adapter_information,
        start,
        count,
        build_device_milliseconds,
        transfer_compute_readback_milliseconds,
        total_milliseconds,
        &flips,
        &drops,
        sentinel_and_gid_ok,
        anchors_cpu_match,
        anchors_gpu_match,
        &records,
    )
    .expect("写结果文件失败");
    println!(
        "RUN_SHARD start={start} count={count} adapter_index={adapter_index} build_device_ms={build_device_milliseconds:.3} \
transfer_compute_readback_ms={transfer_compute_readback_milliseconds:.3} total_ms={total_milliseconds:.3} \
anchors_cpu_match={anchors_cpu_match} anchors_gpu_match={anchors_gpu_match} sentinel_and_gid_ok={sentinel_and_gid_ok} \
record_count={} output={output_path:?}",
        records.len()
    );
}

/// `merge`：臂 C 对整批（`--total-units` 个）现算参照，与各片结果比对，逐量打印
/// 登记第六节 Q2/Q3/Q5/Q7/Q8/Q9 需要的计数（作废 2 的另两条断言：2560 个值两两不同、
/// 抽三单元与臂 C 同值，在这里一并做）。
fn command_merge(arguments: &[String]) {
    let original_input_path = PathBuf::from(parse_named_argument(arguments, "--original-input").expect("merge 需要 --original-input"));
    let total_units: u64 = parse_named_argument(arguments, "--total-units").expect("merge 需要 --total-units").parse().expect("total-units 不是数字");
    let shard_paths: Vec<PathBuf> = parse_repeated_named_argument(arguments, "--shard").iter().map(PathBuf::from).collect();
    assert!(!shard_paths.is_empty(), "merge 至少要给一个 --shard");

    let header = read_batch_header(&original_input_path).expect("读批文件头失败");
    println!("MERGE_INPUT_HEADER seed=0x{:016X} unit_count={}", header.seed, header.unit_count);
    assert!(header.unit_count >= total_units, "批文件的单元数比 --total-units 还少");
    let cpu_reference = compute_cpu_reference_for_all_units(&original_input_path, &header, total_units);

    // 作废 2：2560 个值两两不同（真碰撞概率约 7.6e-4，登记第七节「另三条」）。
    let mut sorted_reference = cpu_reference.clone();
    sorted_reference.sort_unstable();
    let distinct_count = sorted_reference.windows(2).filter(|pair| pair[0] != pair[1]).count() + usize::from(!sorted_reference.is_empty());
    let all_distinct = distinct_count == cpu_reference.len();

    let shard_records: Vec<Vec<ShardResultRecord>> = shard_paths.iter().map(|path| read_shard_result_file(path).unwrap_or_else(|error| panic!("读结果文件 {path:?} 失败：{error}"))).collect();
    let shard_record_slices: Vec<&[ShardResultRecord]> = shard_records.iter().map(Vec::as_slice).collect();
    let report = compare_reference_and_shards(&cpu_reference, &shard_record_slices);

    println!("MERGE total_units={total_units} shards={}", shard_paths.len());
    println!("MERGE matched_count={}", report.matched_count);
    println!("MERGE mismatched_count={} mismatched_global_unit_identifiers={:?}", report.mismatched_global_unit_identifiers.len(), report.mismatched_global_unit_identifiers);
    println!("MERGE missing_count={} missing_global_unit_identifiers={:?}", report.missing_global_unit_identifiers.len(), report.missing_global_unit_identifiers);
    println!("MERGE duplicated_count={} duplicated_global_unit_identifiers={:?}", report.duplicated_global_unit_identifiers.len(), report.duplicated_global_unit_identifiers);
    println!("MERGE all_reference_values_distinct={all_distinct}");
    // 登记第七节「另三条」：抽算单元 0、1279、2559（N = 2560 时的首、中、末）。
    let middle_sample_index = if total_units > 1279 { 1279 } else { total_units / 2 };
    let sample_indices: [u64; 3] = [0, middle_sample_index, total_units - 1];
    for &sample_index in &sample_indices {
        let recomputed_bytes = read_unit_range(&original_input_path, &header, sample_index, 1).expect("抽算失败");
        let recomputed = crc32c_reference(&recomputed_bytes);
        println!("MERGE sample_unit={sample_index} recomputed={recomputed:08X} cpu_reference={:08X} equal={}", cpu_reference[sample_index as usize], recomputed == cpu_reference[sample_index as usize]);
    }
}

/// 开跑前判准入与运行条件（`.claude/singlefs-ai-sop/rules/preflight-discipline.md`「开头先判」Rust 那一行）。
/// `run-shard` 要在另一台上跑（那边只有这个二进制，没有源文件与规范副本），不判；别的子命令都在本机、先判。
fn preflight() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.first().is_some_and(|subcommand| subcommand == "run-shard") {
        return;
    }
    let manifest_directory = env!("CARGO_MANIFEST_DIR");
    let source = format!("{manifest_directory}/src/bin/e163_gpu_multicard_crc32c.rs");
    let script = format!("{manifest_directory}/../../.claude/singlefs-ai-sop/scripts/preflight.py");
    let mut command = std::process::Command::new("python3");
    command.arg(&script).arg("check").arg(&source);
    if arguments.iter().any(|argument| argument == "--force") {
        command.arg("--force");
    }
    command.arg("--");
    command.args(arguments.iter().filter(|argument| *argument != "--force"));
    let output = command.output().unwrap_or_else(|error| {
        eprintln!("  ✗ 起不了 python3 判准入：{error}");
        eprintln!("  → 怎么办：装上 python3，或在有 python3 的机器上跑");
        std::process::exit(78)
    });
    if !output.status.success() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        print!("{}", String::from_utf8_lossy(&output.stdout));
        std::process::exit(output.status.code().unwrap_or(1));
    }
}

fn main() {
    preflight();
    let arguments: Vec<String> = env::args().collect();
    let subcommand = arguments.get(1).map(String::as_str).unwrap_or("");
    let rest = &arguments[1.min(arguments.len())..];
    match subcommand {
        "probe" => command_probe(),
        "anchors" => command_anchors(rest),
        "gen-input" => command_generate_input(rest),
        "run-shard" => command_run_shard(rest),
        "merge" => command_merge(rest),
        other => {
            eprintln!("认不出的子命令「{other}」，要 probe / anchors / gen-input / run-shard / merge 之一");
            std::process::exit(2);
        }
    }
}

// ============================== 单测 ==============================
//
// 前六条不碰 GPU，`cargo test --release --bin e163-gpu-multicard-crc32c` 就能跑；
// 后三条要一张起得来的 NVIDIA 独显卡，本机与另一台都有，跑不到卡时 panic 说明环境不满足，
// 不静默跳过（跳过等于把断言关掉，见 mutation-sampling.md 第三类）。

#[cfg(test)]
mod tests {
    use super::*;

    /// `cargo test` 默认并行跑各条测试；三条要 GPU 的测试都对同一张（枚举序第 0 张）物理卡
    /// 各建一份 `wgpu::Instance` / `Device` 并派发计算，三个线程同时撞上时会卡死在某个
    /// GPU 侧的阻塞调用里（`request_device` 或 `device.poll(Wait)`），既不报错也不超时——
    /// 2026-09-27 实测：同一套代码单独跑全套测试 0.58 秒通过，另一次三线程撞上后挂起 39
    /// 分钟没有任何输出。这把锁把「碰不碰同一张卡」的测试序列化，让默认并行的 `cargo test`
    /// 也不再互相踩踏。
    static GPU_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn first_nvidia_discrete_context() -> GpuContext {
        let instance = new_vulkan_instance();
        let adapters = nvidia_discrete_adapters(&instance);
        assert!(!adapters.is_empty(), "这台机器上枚举不到任何一张 NVIDIA 独显卡，测不了 GPU 路径");
        let adapter = adapters.into_iter().next().expect("刚判过非空");
        build_gpu_context(adapter).expect("建 GPU 上下文失败")
    }

    #[test]
    fn crc32c_reference_matches_prereg_anchor_vectors() {
        let vectors = anchor_vectors();
        for index in 0..8 {
            let computed = crc32c_reference(&vectors[index]);
            assert_eq!(computed, EXPECTED_ANCHOR_VALUES[index], "锚点「{}」算出来的值与登记的不一致", ANCHOR_VECTOR_NAMES[index]);
        }
    }

    #[test]
    fn compare_reference_and_shards_reports_injected_mismatch() {
        let cpu_reference = vec![0x1111_1111u32, 0x2222_2222u32, 0x3333_3333u32, 0x4444_4444u32];
        let shard = vec![
            ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 },
            ShardResultRecord { global_unit_identifier: 1, crc: 0xDEAD_BEEF }, // 故意错
            ShardResultRecord { global_unit_identifier: 2, crc: 0x3333_3333 },
            ShardResultRecord { global_unit_identifier: 3, crc: 0x4444_4444 },
        ];
        let report = compare_reference_and_shards(&cpu_reference, &[shard.as_slice()]);
        assert_eq!(report.mismatched_global_unit_identifiers, vec![1], "V1 要抓的场景：合并比对必须报出这一个不一致");
        assert_eq!(report.matched_count, 3);
        assert!(report.missing_global_unit_identifiers.is_empty());
    }

    #[test]
    fn compare_reference_and_shards_reports_missing_unit() {
        let cpu_reference = vec![0x1111_1111u32, 0x2222_2222u32, 0x3333_3333u32];
        let shard = vec![ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 }, ShardResultRecord { global_unit_identifier: 2, crc: 0x3333_3333 }];
        let report = compare_reference_and_shards(&cpu_reference, &[shard.as_slice()]);
        assert_eq!(report.missing_global_unit_identifiers, vec![1], "V2 要抓的场景：合并比对必须按 0..N 查出缺的那个");
        assert_eq!(report.matched_count, 2);
    }

    #[test]
    fn compare_reference_and_shards_reports_duplicate_unit() {
        let cpu_reference = vec![0x1111_1111u32];
        let shard_one = vec![ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 }];
        let shard_two = vec![ShardResultRecord { global_unit_identifier: 0, crc: 0x1111_1111 }];
        let report = compare_reference_and_shards(&cpu_reference, &[shard_one.as_slice(), shard_two.as_slice()]);
        assert_eq!(report.duplicated_global_unit_identifiers, vec![0]);
    }

    #[test]
    fn generate_unit_bytes_is_deterministic_and_full_length() {
        let first = generate_unit_bytes(0xE163_0927_0000_0001, 42);
        let second = generate_unit_bytes(0xE163_0927_0000_0001, 42);
        assert_eq!(first, second, "同样的种子与全局单元号，两次生成必须逐字节相同");
        assert_eq!(first.len(), UNIT_BYTE_LENGTH);
        let different_unit = generate_unit_bytes(0xE163_0927_0000_0001, 43);
        assert_ne!(first, different_unit, "相邻单元号的内容不该恰好相同");
    }

    #[test]
    fn apply_bit_flip_changes_exactly_one_bit() {
        let mut bytes = vec![0u8; 8];
        apply_bit_flip(&mut bytes, 3, 5);
        assert_eq!(bytes[3], 0b0010_0000);
        for (index, &value) in bytes.iter().enumerate() {
            if index != 3 {
                assert_eq!(value, 0, "只该有第 3 个字节被改动");
            }
        }
    }

    #[test]
    fn batch_file_round_trip_matches_generator() {
        let path = std::env::temp_dir().join(format!("e163-batch-round-trip-{}.bin", std::process::id()));
        write_batch_file(&path, 5, 0xABCD_1234_0000_0001).expect("写批文件失败");
        let header = read_batch_header(&path).expect("读头失败");
        assert_eq!(header.unit_count, 5);
        assert_eq!(header.unit_byte_length as usize, UNIT_BYTE_LENGTH);
        let read_back = read_unit_range(&path, &header, 2, 1).expect("读区间失败");
        let expected = generate_unit_bytes(0xABCD_1234_0000_0001, 2);
        assert_eq!(read_back, expected, "从批文件读出的第 2 个单元应当与生成函数直接算出的一致");
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn gpu_matches_cpu_reference_on_anchor_vectors() {
        let _gpu_test_guard = GPU_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let context = first_nvidia_discrete_context();
        let vectors = anchor_vectors();
        let lengths: [u32; 8] = std::array::from_fn(|index| vectors[index].len() as u32);
        let mut unit_slots = vec![0u8; 8 * UNIT_BYTE_LENGTH];
        for (index, vector) in vectors.iter().enumerate() {
            unit_slots[index * UNIT_BYTE_LENGTH..index * UNIT_BYTE_LENGTH + vector.len()].copy_from_slice(vector);
        }
        let gpu_results = run_gpu_crc(&context, &unit_slots, &lengths);
        assert_full_coverage_no_sentinel(&gpu_results);
        for index in 0..8 {
            assert_eq!(gpu_results[index].0, EXPECTED_ANCHOR_VALUES[index], "GPU 在锚点「{}」上应当与登记的期望值一致", ANCHOR_VECTOR_NAMES[index]);
        }
    }

    #[test]
    fn gpu_dispatch_covers_every_unit_no_sentinel_residue() {
        let _gpu_test_guard = GPU_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let context = first_nvidia_discrete_context();
        let count = 200usize;
        let unit_slots = vec![0u8; count * UNIT_BYTE_LENGTH];
        let lengths = vec![UNIT_BYTE_LENGTH as u32; count];
        let results = run_gpu_crc(&context, &unit_slots, &lengths);
        assert_eq!(results.len(), count);
        assert_full_coverage_no_sentinel(&results);
    }

    #[test]
    fn injected_bit_flip_changes_gpu_result_but_not_original_cpu_reference() {
        let _gpu_test_guard = GPU_TEST_MUTEX.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let context = first_nvidia_discrete_context();
        let original = generate_unit_bytes(0xE163_0927_0000_0002u64, 7);
        let cpu_reference_on_original = crc32c_reference(&original);
        let mut uploaded_copy = original.clone();
        apply_bit_flip(&mut uploaded_copy, 4099, 1);
        let results = run_gpu_crc(&context, &uploaded_copy, &[UNIT_BYTE_LENGTH as u32]);
        assert_full_coverage_no_sentinel(&results);
        assert_ne!(results[0].0, cpu_reference_on_original, "V7 要抓的场景：翻位只进了上传副本，GPU 结果必须与未翻位的 CPU 参照不同");
    }
}
