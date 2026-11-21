//! 真卡主机与校准、背景负载、卡租借通道、子进程的真卡与模型角色。

use super::*;

// ===== 以下是真卡的那一份 CardHost（挂 wgpu，单测与 `checks` 不碰） =====

// ============================== 真卡：wgpu 30.0.1，Vulkan ==============================

/// 空转内核：单线程做 `iteration_count` 次相依的整数运算，结果写进第一张缓冲的第一个字，编译器消不掉。
/// `iteration_count` 是管线常量，标定时按目标时长调。另一个入口只写一个字（登记借法 P2 的小内核）。
const WORK_SHADER_SOURCE: &str = r#"
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

@compute @workgroup_size(1)
fn write_one_word() {
    output_words[0] = 1u;
}
"#;

const NVIDIA_VENDOR_IDENTIFIER: u32 = 0x10DE;
const CALIBRATION_BUFFER_BYTES: u64 = MEBIBYTE;
const FIRST_CALIBRATION_GUESS_ITERATIONS: u32 = 2_000_000;
const LEASE_PIECE_USAGE: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE.union(wgpu::BufferUsages::COPY_DST);

fn wait_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(WAIT_TIMEOUT_SECONDS)
}

fn build_work_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    entry_point: &str,
    iteration_count: u32,
) -> wgpu::ComputePipeline {
    let constants = [("iteration_count", f64::from(iteration_count))];
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("e174-work-pipeline"),
        layout: None,
        module,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions { constants: &constants, zero_initialize_workgroup_memory: true },
        cache: None,
    })
}

/// 清每张缓冲，再在第一张的头 256 字节上派一次内核，一次提交。
fn submit_clear_and_kernel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pieces: &[wgpu::Buffer],
    pipeline: &wgpu::ComputePipeline,
) -> wgpu::SubmissionIndex {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("e174-encoder") });
    for piece in pieces {
        encoder.clear_buffer(piece, 0, None);
    }
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("e174-bind-group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &pieces[0],
                offset: 0,
                size: std::num::NonZeroU64::new(SPIN_KERNEL_BINDING_BYTES),
            }),
        }],
    });
    {
        let mut compute_pass =
            encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("e174-pass"), timestamp_writes: None });
        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(1, 1, 1);
    }
    queue.submit(Some(encoder.finish()))
}

fn create_piece(device: &wgpu::Device, bytes: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("e174-lease-piece"),
        size: bytes,
        usage: LEASE_PIECE_USAGE,
        mapped_at_creation: false,
    })
}

struct CalibrationResult {
    iteration_count: u32,
    pub(super) work_microseconds: u64,
    is_in_band: bool,
}

/// 登记第五节「活的标定」：用一张 1 MiB 的缓冲派空转内核，按目标时长调循环次数，同一条管线连量 3 次都在
/// 目标的 0.8–1.25 倍之间为止；记这 3 次的中位数为这个进程的 W。试了 `CALIBRATION_ATTEMPT_LIMIT` 次还没调到记不在带内。
fn calibrate_spin_kernel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    module: &wgpu::ShaderModule,
    target_microseconds: u64,
) -> CalibrationResult {
    let calibration_pieces = vec![create_piece(device, CALIBRATION_BUFFER_BYTES)];
    let lower = target_microseconds * CALIBRATION_BAND_LOWER_PERCENT / 100;
    let upper = target_microseconds * CALIBRATION_BAND_UPPER_PERCENT / 100;
    let mut iteration_count = FIRST_CALIBRATION_GUESS_ITERATIONS;
    let mut result = CalibrationResult { iteration_count, work_microseconds: 0, is_in_band: false };
    for _attempt in 0..CALIBRATION_ATTEMPT_LIMIT {
        let pipeline = build_work_pipeline(device, module, "spin", iteration_count);
        let mut durations = Vec::with_capacity(CALIBRATION_CONSECUTIVE_HITS);
        for _measurement in 0..CALIBRATION_CONSECUTIVE_HITS {
            let started = std::time::Instant::now();
            let submission = submit_clear_and_kernel(device, queue, &calibration_pieces, &pipeline);
            device
                .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(wait_timeout()) })
                .expect("标定时 Wait 失败");
            durations.push(u64::try_from(started.elapsed().as_micros()).expect("时长装得进 u64"));
        }
        let median = median_of(&durations);
        result = CalibrationResult {
            iteration_count,
            work_microseconds: median,
            is_in_band: durations.iter().all(|duration| (lower..=upper).contains(duration)),
        };
        if result.is_in_band {
            break;
        }
        let scaled = u128::from(iteration_count) * u128::from(target_microseconds) / u128::from(median.max(1));
        iteration_count = u32::try_from(scaled.clamp(1, u128::from(u32::MAX))).expect("夹在 u32 范围内");
    }
    result
}

struct WgpuCardHost {
    pub(super) card: Card,
    /// 服务给的那一张的总线号（去掉域，`bus_of_the_handed_card_for` 取的）。
    bus: String,
    started: std::time::Instant,
    /// 实例要活到进程结束，设备才不会在用着的时候被收走。
    _instance: wgpu::Instance,
    adapter_description: String,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader_module: wgpu::ShaderModule,
    spin_pipeline: wgpu::ComputePipeline,
    small_pipeline: wgpu::ComputePipeline,
    pub(super) pieces: Vec<wgpu::Buffer>,
    coexisting: Vec<wgpu::Buffer>,
    submissions: Vec<wgpu::SubmissionIndex>,
}

/// 登记第五节：后端只用 Vulkan，`required_limits` 取 `adapter.limits()`，`memory_hints` 取 `Performance`。
/// 开的是服务给的那一张：`bus` 由 `bus_of_the_handed_card_for` 取，Vulkan 适配器里总线号对上它的那张 NVIDIA 独显。
fn open_wgpu_card(card: Card, bus: &str) -> Result<WgpuCardHost, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.enumerate_adapters(wgpu::Backends::VULKAN))
        .into_iter()
        .find(|adapter| {
            let information = adapter.get_info();
            information.vendor == NVIDIA_VENDOR_IDENTIFIER
                && information.device_type == wgpu::DeviceType::DiscreteGpu
                && bus_without_domain(&information.device_pci_bus_id) == bus
        })
        .ok_or_else(|| format!("Vulkan 适配器里没有总线 {bus} 那张 NVIDIA 独显（服务给的那一张）"))?;
    let information = adapter.get_info();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("e174-lease-drop-wait-memory-return"),
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
    }))
    .map_err(|error| format!("request_device 失败：{error:?}"))?;
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("e174-work-shader"),
        source: wgpu::ShaderSource::Wgsl(WORK_SHADER_SOURCE.into()),
    });
    let spin_pipeline = build_work_pipeline(&device, &shader_module, "spin", FIRST_CALIBRATION_GUESS_ITERATIONS);
    let small_pipeline = build_work_pipeline(&device, &shader_module, "write_one_word", 1);
    Ok(WgpuCardHost {
        card,
        bus: bus.to_string(),
        started: std::time::Instant::now(),
        _instance: instance,
        adapter_description: format!(
            "name={:?} vendor=0x{:X} device=0x{:X} driver={:?} driver_info={:?} pci_bus_id={:?}",
            information.name, information.vendor, information.device, information.driver, information.driver_info, information.device_pci_bus_id
        ),
        device,
        queue,
        shader_module,
        spin_pipeline,
        small_pipeline,
        pieces: Vec::new(),
        coexisting: Vec::new(),
        submissions: Vec::new(),
    })
}

pub(super) fn run_nvidia_smi(arguments: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("nvidia-smi")
        .args(arguments)
        .output()
        .map_err(|error| format!("起不了 nvidia-smi：{error}"))?;
    if !output.status.success() {
        return Err(format!("nvidia-smi 退 {:?}：{}", output.status.code(), String::from_utf8_lossy(&output.stderr)));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

impl CardHost for WgpuCardHost {
    fn card(&self) -> Card {
        self.card
    }

    fn bus_of_the_card(&self) -> String {
        self.bus.clone()
    }

    fn process_identifier(&self) -> u32 {
        std::process::id()
    }

    fn now_microseconds(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_micros()).expect("时长装得进 u64")
    }

    fn read_report_text(&mut self) -> Result<String, String> {
        run_nvidia_smi(&PROCESS_REPORT_ARGUMENTS)
    }

    fn read_compute_applications_text(&mut self) -> Result<String, String> {
        run_nvidia_smi(&COMPUTE_APPLICATIONS_ARGUMENTS)
    }

    fn read_allocator_figures(&mut self) -> Option<AllocatorFigures> {
        self.device.generate_allocator_report().map(|report| AllocatorFigures {
            reserved_bytes: report.total_reserved_bytes,
            allocated_bytes: report.total_allocated_bytes,
        })
    }

    fn create_lease_pieces(&mut self, piece_byte_lengths: &[u64]) {
        for bytes in piece_byte_lengths {
            self.pieces.push(create_piece(&self.device, *bytes));
        }
    }

    fn submit_lease_work(&mut self, kind: WorkKind) -> SubmissionToken {
        let pipeline = match kind {
            WorkKind::Spin => &self.spin_pipeline,
            WorkKind::SmallKernel => &self.small_pipeline,
        };
        let submission = submit_clear_and_kernel(&self.device, &self.queue, &self.pieces, pipeline);
        self.submissions.push(submission);
        SubmissionToken(u64::try_from(self.submissions.len() - 1).expect("提交号装得进 u64"))
    }

    fn poll_device(&mut self, mode: PollMode) -> Result<(), String> {
        let poll_type = match mode {
            PollMode::Poll => wgpu::PollType::Poll,
            PollMode::WaitLatest => wgpu::PollType::Wait { submission_index: None, timeout: Some(wait_timeout()) },
            PollMode::WaitOwnSubmission(token) => wgpu::PollType::Wait {
                submission_index: Some(self.submissions[usize::try_from(token.0).expect("提交号装得进 usize")].clone()),
                timeout: Some(wait_timeout()),
            },
        };
        self.device.poll(poll_type).map(|_status| ()).map_err(|error| format!("{error:?}"))
    }

    fn drop_lease_pieces(&mut self) {
        self.pieces.clear();
    }

    fn create_coexisting_buffer(&mut self, bytes: u64) {
        let buffer = create_piece(&self.device, bytes);
        let coexisting_pieces = [buffer];
        let submission = submit_clear_and_kernel(&self.device, &self.queue, &coexisting_pieces, &self.small_pipeline);
        self.device
            .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(wait_timeout()) })
            .expect("并存的长借据那张缓冲清完时 Wait 失败");
        self.coexisting.extend(coexisting_pieces);
    }
}

// ============================== 第二段：同一设备上另一条线程接着提交 ==============================

/// 登记第五节第二段：线程 B 在同一个设备上一次接一次提交空转 400 ms 的活；起好之后才放行读数，B 不再建缓冲。
struct BackgroundLoad {
    should_stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<u64>>,
}

fn start_background_load(host: &WgpuCardHost, iteration_count: u32) -> BackgroundLoad {
    let device = host.device.clone();
    let queue = host.queue.clone();
    let module = host.shader_module.clone();
    let should_stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let thread_should_stop = std::sync::Arc::clone(&should_stop);
    let (ready_sender, ready_receiver) = std::sync::mpsc::channel::<()>();
    let handle = std::thread::spawn(move || {
        let pipeline = build_work_pipeline(&device, &module, "spin", iteration_count);
        let background_pieces = vec![create_piece(&device, CALIBRATION_BUFFER_BYTES)];
        ready_sender.send(()).expect("主线程在等 B 起好");
        let mut submissions_done: u64 = 0;
        while !thread_should_stop.load(std::sync::atomic::Ordering::Relaxed) {
            let submission = submit_clear_and_kernel(&device, &queue, &background_pieces, &pipeline);
            device
                .poll(wgpu::PollType::Wait { submission_index: Some(submission), timeout: Some(wait_timeout()) })
                .expect("线程 B 的 Wait 失败");
            submissions_done += 1;
        }
        submissions_done
    });
    ready_receiver.recv().expect("线程 B 起好之前就退了");
    BackgroundLoad { should_stop, handle: Some(handle) }
}

impl BackgroundLoad {
    fn stop_and_count_submissions(&mut self) -> u64 {
        self.should_stop.store(true, std::sync::atomic::Ordering::Relaxed);
        self.handle.take().map_or(0, |handle| handle.join().expect("线程 B 没有 panic"))
    }
}

// ============================== 服务给的卡 ==============================
//
// 卡由统一任务服务分（`task-server/task_server.py` 文件头 ②「GPU」「按需领卡」）：起跑就交卡的，卡串在环境变量 `GPU_CARDS_BY_PRIORITY`；
// 按需领卡的，环境里只有领卡通道（`CARD_LEASE_CHANNEL_VARIABLES` 那五个），子进程到开设备前一刻经领卡客户端 `task-server/task lease take` 领一回、
// 退出时还（写法照 `crates/singlefs-verification/src/gpu_card_configuration.rs` 的 `CardLeaseChannelOfTheCaller`；这份装置不依赖 `crates/` 下的包，这里另写一份）。
// 卡串是空白隔开的若干项，每项 `<HOSTS 里的地址>:<nvidia-smi 序号>:<额度>`；这一台是 `HOSTS` 里哪一项只由多机配置判法
// `scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses` 打的 `THIS_HOST_ADDRESS` 认。

/// 服务交卡的环境变量。
const VARIABLE_OF_THE_CARDS_HANDED_OVER: &str = "GPU_CARDS_BY_PRIORITY";
/// 服务交给按需领卡那一次尝试的领卡通道：通道目录、凭据、尝试标签、任务号、第几次尝试（`task-server/task_server_lib.py` 文件头「GPU」的领卡通道），五个都在、都不空才算有通道。
pub(super) const CARD_LEASE_CHANNEL_VARIABLES: [&str; 5] =
    ["TASK_SERVER_CARD_LEASE_DIRECTORY", "TASK_SERVER_CARD_LEASE_TOKEN", "TASK_SERVER_ATTEMPT_LABEL", "TASK_SERVER_TASK", "TASK_SERVER_ATTEMPT"];
/// 服务每次尝试交给命令的「提交方那一份仓的根」：领卡客户端取它下面的 `CARD_LEASE_CLIENT_IN_THE_REPOSITORY`。
const SOURCE_REPOSITORY_VARIABLE: &str = "TASK_SERVER_SOURCE_REPOSITORY";
/// 领卡客户端在仓里的路径（从仓根起）。
const CARD_LEASE_CLIENT_IN_THE_REPOSITORY: &str = "task-server/task";
/// 还卡子进程跑的 shell：标准输入读到收尾（领卡的进程退出，写端随之关掉）再起客户端 `lease return <卡串>`（`$0` 是客户端、`$1` 是卡串）。
const RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS: &str = r#"while read -r line_never_written; do :; done; exec "$0" lease return "$1""#;
/// 认服务给的卡是哪一型、在哪条总线：`nvidia-smi` 报每张卡的序号、总线号与 FB Total（MiB）。
const HANDED_CARD_QUERY_ARGUMENTS: [&str; 2] = ["--query-gpu=index,pci.bus_id,memory.total", "--format=csv,noheader,nounits"];

/// 调用方给这个进程的领卡通道：五个变量的名字与值，原样交给领卡客户端。
#[derive(Debug, PartialEq, Eq)]
pub(super) struct CardLeaseChannel {
    variables: Vec<(&'static str, String)>,
}

/// 照 `value_of`（按名字取一个环境变量的值）认通道：五个都给了、都不空才是 `Some`。
pub(super) fn card_lease_channel_of(value_of: &dyn Fn(&str) -> Option<String>) -> Option<CardLeaseChannel> {
    let mut variables = Vec::new();
    for name in CARD_LEASE_CHANNEL_VARIABLES {
        let value = value_of(name).filter(|value| !value.is_empty())?;
        variables.push((name, value));
    }
    Some(CardLeaseChannel { variables })
}

/// 这个进程的卡串从哪来：交卡变量有值（不是空白）照它、不领；没有而有领卡通道，照 `lease` 领（真要卡的这一刻才调）；两样都没有是 `None`。
pub(super) fn cards_handed_over_or_leased(
    handover_value: Option<&str>,
    channel: Option<&CardLeaseChannel>,
    lease: &dyn Fn(&CardLeaseChannel) -> Result<String, String>,
) -> Result<Option<String>, String> {
    match (handover_value.filter(|value| !value.trim().is_empty()), channel) {
        (Some(handover_text), _) => Ok(Some(handover_text.to_string())),
        (None, Some(channel)) => lease(channel).map(Some),
        (None, None) => Ok(None),
    }
}

/// 领卡客户端在哪：服务交的仓根（`SOURCE_REPOSITORY_VARIABLE` 的值）给了就是它下面那一份；没给的从 `start_directory` 往上逐层找。
fn card_lease_client_from(source_repository: Option<std::path::PathBuf>, start_directory: &std::path::Path) -> Result<std::path::PathBuf, String> {
    if let Some(repository) = source_repository {
        let client = repository.join(CARD_LEASE_CLIENT_IN_THE_REPOSITORY);
        return if client.is_file() { Ok(client) } else { Err(format!("{SOURCE_REPOSITORY_VARIABLE} 点的仓根下没有领卡客户端：{} 不是文件", client.display())) };
    }
    start_directory
        .ancestors()
        .map(|directory| directory.join(CARD_LEASE_CLIENT_IN_THE_REPOSITORY))
        .find(|client| client.is_file())
        .ok_or_else(|| format!("从 {} 往上找不到领卡客户端 {CARD_LEASE_CLIENT_IN_THE_REPOSITORY}", start_directory.display()))
}

/// 领到的卡串，连同等着这个进程退出就还卡的那个子进程（它的标准输入写端只在这个进程手里，进程退出写端就关）。
#[derive(Debug)]
pub(super) struct CardsLeasedUntilThisProcessExits {
    pub(super) cards: String,
    /// 平常只为握着它标准输入的写端（进程退出才关），不读；单测经 `return_the_cards_now` 当场还。
    _return_waiting_for_the_exit: std::process::Child,
}

impl CardsLeasedUntilThisProcessExits {
    /// 不等进程退出，当场还：关掉还卡子进程的标准输入、等它起的客户端退出；交回客户端退没退 0（单测用）。
    #[cfg(test)]
    pub(super) fn return_the_cards_now(mut self) -> bool {
        drop(self._return_waiting_for_the_exit.stdin.take());
        self._return_waiting_for_the_exit.wait().is_ok_and(|status| status.success())
    }
}

/// 经 `client` 领一回卡：`lease take` 的标准输出是卡串（客户端在等卡时往标准错误打的行照进这个进程的标准错误）；领到之后起还卡的子进程。
/// 客户端退非 0、退 0 而没交卡串、起不了还卡的子进程都交 `Err`（不重试；后两种领到的当场还）。
pub(super) fn lease_cards_through_the_client(client: &std::path::Path, channel: &CardLeaseChannel) -> Result<CardsLeasedUntilThisProcessExits, String> {
    let variables = || channel.variables.iter().map(|(name, value)| (*name, value.as_str()));
    let taken = std::process::Command::new(client)
        .args(["lease", "take"])
        .envs(variables())
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .output()
        .map_err(|error| format!("起不了领卡客户端 {}：{error}", client.display()))?;
    if !taken.status.success() {
        return Err(format!("按需领卡没成：{} lease take 退 {:?}（原因在它往标准错误打的那一行）", client.display(), taken.status.code()));
    }
    let cards = String::from_utf8_lossy(&taken.stdout).trim().to_string();
    let return_the_cards_at_once = || {
        // 领到了而往下走不通：当场还，结局不改这一句错（尝试结束时服务照样收回）
        let _returned = std::process::Command::new(client).args(["lease", "return"]).envs(variables()).stdin(std::process::Stdio::null()).status();
    };
    if cards.is_empty() {
        return_the_cards_at_once();
        return Err(format!("按需领卡没成：{} lease take 退 0 而没交卡串", client.display()));
    }
    match std::process::Command::new("sh")
        .arg("-c")
        .arg(RETURN_THE_CARDS_WHEN_THE_STANDARD_INPUT_ENDS)
        .arg(client)
        .arg(&cards)
        .envs(variables())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
    {
        Ok(return_waiting_for_the_exit) => Ok(CardsLeasedUntilThisProcessExits { cards, _return_waiting_for_the_exit: return_waiting_for_the_exit }),
        Err(error) => {
            return_the_cards_at_once();
            Err(format!("按需领卡领到了 {cards}，起不了等这个进程退出就还卡的子进程（sh）：{error}；领到的已当场还掉"))
        }
    }
}

/// 这个进程按需领卡那一回的结局：进程里只领一回，之后每次交同一份；被拒的那一回也记下，不重试。
static CARDS_LEASED_BY_THIS_PROCESS: std::sync::OnceLock<Result<CardsLeasedUntilThisProcessExits, String>> = std::sync::OnceLock::new();

/// 这个进程按需领到的卡串：头一回问的时候才领（客户端在等卡时这里照等），进程退出时还。
fn cards_leased_by_this_process(channel: &CardLeaseChannel) -> Result<&'static str, String> {
    CARDS_LEASED_BY_THIS_PROCESS
        .get_or_init(|| {
            let current_directory = std::env::current_dir().map_err(|error| format!("取不到当前目录（领卡客户端从这里往上找）：{error}"))?;
            let client = card_lease_client_from(std::env::var_os(SOURCE_REPOSITORY_VARIABLE).map(std::path::PathBuf::from), &current_directory)?;
            let leased = lease_cards_through_the_client(&client, channel)?;
            eprintln!("GPU_CARDS_LEASED_ON_DEMAND cards={}", leased.cards);
            Ok(leased)
        })
        .as_ref()
        .map(|leased| leased.cards.as_str())
        .map_err(Clone::clone)
}

/// 卡串里给这一台的那几张的 `nvidia-smi` 序号（照卡串的次序）：每项从右边切两刀（地址可以是 IPv6、自带冒号），三段都不许空、序号是数；
/// 只留地址与 `this_host` 逐字相同的；一张都没有、读不成交 `Err`。
pub(super) fn handed_indices_of_this_host(cards: &str, this_host: &str) -> Result<Vec<u32>, String> {
    let mut indices = Vec::new();
    for entry in cards.split_whitespace() {
        let without_three_fields = || format!("卡串里的「{entry}」从右边切不出地址、序号、额度三段");
        let (address_and_index, quota) = entry.rsplit_once(':').ok_or_else(without_three_fields)?;
        let (host_address, index_text) = address_and_index.rsplit_once(':').ok_or_else(without_three_fields)?;
        if host_address.is_empty() || index_text.is_empty() || quota.is_empty() {
            return Err(without_three_fields());
        }
        let index: u32 = index_text.parse().map_err(|_| format!("卡串里的「{entry}」序号不是数"))?;
        if host_address == this_host {
            indices.push(index);
        }
    }
    if indices.is_empty() {
        return Err(format!("服务给的卡（{cards}）里没有这一台（HOSTS 里的 {this_host}）的"));
    }
    Ok(indices)
}

/// bash `printf %q` 打的一个地址还原成原样：只认反斜杠转义与空串的 `''`，别的引号写法认不出就报错。
fn value_printed_by_printf_quoted(quoted: &str) -> Result<String, String> {
    if quoted == "''" {
        return Ok(String::new());
    }
    if quoted.starts_with('\'') || quoted.starts_with("$'") {
        return Err(format!("「{quoted}」的引号写法不是判法给地址打的那一种"));
    }
    let mut value = String::with_capacity(quoted.len());
    let mut characters = quoted.chars();
    while let Some(character) = characters.next() {
        if character == '\\' {
            value.push(characters.next().ok_or_else(|| format!("「{quoted}」结尾多了一个反斜杠"))?);
        } else {
            value.push(character);
        }
    }
    Ok(value)
}

/// 多机配置判法 `--emit-addresses` 打的那几行（`KEY=<printf %q 的值>`）里这一台的地址（`THIS_HOST_ADDRESS` 那一行）。
pub(super) fn this_host_address_emitted(printed: &str) -> Result<String, String> {
    let quoted = printed.lines().find_map(|line| line.strip_prefix("THIS_HOST_ADDRESS=")).ok_or_else(|| format!("多机配置判法没打 THIS_HOST_ADDRESS 那一行（打的是「{printed}」）"))?;
    let address = value_printed_by_printf_quoted(quoted)?;
    if address.is_empty() {
        return Err("多机配置判法打的 THIS_HOST_ADDRESS 是空的".to_string());
    }
    Ok(address)
}

/// 起多机配置判法 `bash <仓根>/scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses`（仓根是清单目录往上两层），读这一台的地址。
fn this_host_address_by_the_configuration_check() -> Result<String, String> {
    let manifest_directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_directory.ancestors().nth(2).ok_or("清单目录在 <仓根>/scripts/experiment-apparatus 下")?;
    let script = root.join("scripts/crash/crash-point-replay-shard-configuration-check.sh");
    let output = std::process::Command::new("bash")
        .arg(&script)
        .arg("--emit-addresses")
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|error| format!("起不了多机配置判法 bash {} --emit-addresses：{error}（这一台是 HOSTS 里哪一项只由它认）", script.display()))?;
    let printed = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(format!("多机配置判法认不出这一台（退 {:?}）：{}{}", output.status.code(), printed.trim(), String::from_utf8_lossy(&output.stderr).trim()));
    }
    this_host_address_emitted(&printed)
}

/// 给这一台的卡里（`handed_indices`，照卡串的次序）头一张与 `card` 同型号的那张的总线号（去掉域）：型号照 `report`
/// （`HANDED_CARD_QUERY_ARGUMENTS` 那一种 `nvidia-smi` 输出，每行「序号, 总线号, FB Total」）的 FB Total 等于登记第七节 B1 的数认；
/// 给的卡 `nvidia-smi` 没报、一张都不是这一型交 `Err`（不退回别的卡）。
pub(super) fn bus_for_the_cell_among(card: Card, handed_indices: &[u32], report: &str) -> Result<String, String> {
    let mut reported: BTreeMap<u32, (String, String)> = BTreeMap::new();
    for line in report.lines().filter(|line| !line.trim().is_empty()) {
        let fields: Vec<&str> = line.split(',').map(str::trim).collect();
        let [index_text, bus, total] = fields.as_slice() else {
            return Err(format!("nvidia-smi 的一行「{line}」不是「序号, 总线号, FB Total」三段"));
        };
        let index: u32 = index_text.parse().map_err(|_| format!("nvidia-smi 的一行「{line}」序号不是数"))?;
        reported.insert(index, (bus_without_domain(bus), (*total).to_string()));
    }
    let wanted_total = card.total_mebibytes().to_string();
    let mut seen = Vec::new();
    for index in handed_indices {
        let (bus, total) = reported.get(index).ok_or_else(|| format!("服务给的卡 nvidia-smi 序号 {index}，nvidia-smi 没报这个序号"))?;
        if *total == wanted_total {
            return Ok(bus.clone());
        }
        seen.push(format!("序号 {index} 总线 {bus} FB Total {total} MiB"));
    }
    Err(format!("服务给这一台的卡（{}）里没有 {} 那一型（FB Total {wanted_total} MiB）", seen.join("；"), card.label()))
}

/// 子进程开哪一张卡：服务给这个进程的卡串（环境里有照它，没有而有领卡通道就在这一刻领）里这一台的那几张，取头一张与这一格同型号的（`bus_for_the_cell_among`）。
/// 两样都没有、领卡没成、卡串里没有这一台的、没有这一型都交 `Err`（子进程退非 0），不自己枚举、不按写死的总线号开。
fn bus_of_the_handed_card_for(card: Card) -> Result<String, String> {
    let handover_value = std::env::var(VARIABLE_OF_THE_CARDS_HANDED_OVER).ok();
    let channel = card_lease_channel_of(&|name| std::env::var(name).ok());
    let cards = cards_handed_over_or_leased(handover_value.as_deref(), channel.as_ref(), &|channel| cards_leased_by_this_process(channel).map(str::to_string))?
        .ok_or_else(|| format!("只开服务给的卡：环境里没有 {VARIABLE_OF_THE_CARDS_HANDED_OVER}，也没有领卡通道（{}）；整条经 task-server/task submit 交", CARD_LEASE_CHANNEL_VARIABLES.join("、")))?;
    let handed_indices = handed_indices_of_this_host(&cards, &this_host_address_by_the_configuration_check()?)?;
    bus_for_the_cell_among(card, &handed_indices, &run_nvidia_smi(&HANDED_CARD_QUERY_ARGUMENTS)?)
}

// ============================== 子进程入口 ==============================

/// 真卡上的一个装置进程：开设备前一刻才取卡（`bus_of_the_handed_card_for`），之后的步骤与次序照登记第五节。
fn run_wgpu_child(job: &ChildJob, raw_output_path: Option<&str>) -> Result<(), String> {
    let bus = bus_of_the_handed_card_for(job.cell.card)?;
    let mut host = open_wgpu_card(job.cell.card, &bus)?;
    let calibration = calibrate_spin_kernel(&host.device, &host.queue, &host.shader_module, job.cell.liveness.work_target_microseconds());
    host.spin_pipeline = build_work_pipeline(&host.device, &host.shader_module, "spin", calibration.iteration_count);
    let mut background = if job.cell.has_background_load() {
        let background_calibration =
            calibrate_spin_kernel(&host.device, &host.queue, &host.shader_module, BACKGROUND_WORK_TARGET_MICROSECONDS);
        Some(start_background_load(&host, background_calibration.iteration_count))
    } else {
        None
    };
    let mut raw_log = RawLog::default();
    let outcome = run_child_procedure(&mut host, job, calibration.work_microseconds, calibration.is_in_band, &mut raw_log);
    let background_submissions = background.as_mut().map(BackgroundLoad::stop_and_count_submissions);
    let extra = format!(
        "background_submissions={} adapter={}",
        background_submissions.map_or_else(|| "none".to_string(), |count| count.to_string()),
        without_whitespace(&host.adapter_description)
    );
    write_child_result(job, &outcome, (calibration.work_microseconds, calibration.is_in_band), &extra, &raw_log, raw_output_path)
}

// ===== 以上是真卡的那一份 CardHost =====

/// 子进程的结果行写 stdout（父进程等它退出后整份读），原文写 `--raw-output` 给的文件。
fn write_child_result(
    job: &ChildJob,
    outcome: &ChildOutcome,
    calibration: (u64, bool),
    extra: &str,
    raw_log: &RawLog,
    raw_output_path: Option<&str>,
) -> Result<(), String> {
    let (work_microseconds, calibration_is_in_band) = calibration;
    for lease in &outcome.leases {
        println!("{}", lease.to_line());
    }
    println!("{}", child_done_line(job, outcome, work_microseconds, calibration_is_in_band, extra));
    if let Some(path) = raw_output_path {
        std::fs::write(path, raw_log_text(job, raw_log)).map_err(|error| format!("写不了原文 {path}：{error}"))?;
    }
    Ok(())
}

/// 模型卡上的一个子进程：给 `run --simulated` 用，整条父子流水线（续跑、补轮、原文压缩、F1 停机）不起卡也走得通。
fn run_simulated_child(job: &ChildJob, raw_output_path: Option<&str>) -> Result<(), String> {
    let work_microseconds = job.cell.liveness.work_target_microseconds();
    let mut host = SimulatedCardHost::new(job.cell.card, work_microseconds);
    let mut raw_log = RawLog::default();
    let outcome = run_child_procedure(&mut host, job, work_microseconds, true, &mut raw_log);
    let extra = "background_submissions=none adapter=simulated";
    write_child_result(job, &outcome, (work_microseconds, true), extra, &raw_log, raw_output_path)
}

/// `child` 子命令：一个装置进程，只按 `--simulated` 分派（登记修订 2 之一「读法写死」）：真卡的轮从 `run_child_on_the_card` 进，
/// `run --simulated` 的轮从 `run_child_on_the_model` 进；两个入口的单元闭包各是那一种轮的子进程代码键。
pub(super) fn run_child(arguments: &[String]) -> Result<(), String> {
    if arguments.iter().any(|argument| argument == "--simulated") {
        run_child_on_the_model(arguments)
    } else {
        run_child_on_the_card(arguments)
    }
}

/// 真卡上的子进程入口：解析参数，再 `run_wgpu_child`。
// 不拆录判两段：这里量的是持卡期间的挂钟与显存归还，持卡期间的次序本身就是被测对象，没有 GPU 结果与 CPU 参照的比对可以挪出去。
fn run_child_on_the_card(arguments: &[String]) -> Result<(), String> {
    let (job, raw_output_path) = parse_child_arguments(arguments)?;
    run_wgpu_child(&job, raw_output_path.as_deref())
}

/// 模型卡上的子进程入口：解析参数，再 `run_simulated_child`。
fn run_child_on_the_model(arguments: &[String]) -> Result<(), String> {
    let (job, raw_output_path) = parse_child_arguments(arguments)?;
    run_simulated_child(&job, raw_output_path.as_deref())
}

/// 子进程的参数：`ChildJob::child_arguments` 那几个，再加 `--raw-output 文件`；`--simulated` 跳过（`run_child` 已经按它分派了）。
fn parse_child_arguments(arguments: &[String]) -> Result<(ChildJob, Option<String>), String> {
    let mut job_arguments: Vec<String> = Vec::new();
    let mut raw_output_path: Option<String> = None;
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--simulated" {
            index += 1;
            continue;
        }
        let value = arguments.get(index + 1).ok_or_else(|| format!("{} 后面没有值", arguments[index]))?;
        if arguments[index] == "--raw-output" {
            raw_output_path = Some(value.clone());
        } else {
            job_arguments.push(arguments[index].clone());
            job_arguments.push(value.clone());
        }
        index += 2;
    }
    Ok((ChildJob::from_child_arguments(&job_arguments)?, raw_output_path))
}

fn raw_log_text(job: &ChildJob, raw_log: &RawLog) -> String {
    let mut text = format!("######## {}\n", job.key_text());
    for (label, report) in &raw_log.entries {
        text.push_str(&format!("==== {label}\n{report}\n"));
    }
    text
}

pub(super) fn child_done_line(job: &ChildJob, outcome: &ChildOutcome, work_microseconds: u64, calibration_is_in_band: bool, extra: &str) -> String {
    format!(
        "E7RESULT name=child_done {} leases_run={} work_microseconds={work_microseconds} calibration_in_band={calibration_is_in_band} \
         child_void={} {extra}",
        job.key_text(),
        outcome.leases.len(),
        void_reasons_text(&outcome.child_void),
    )
}
