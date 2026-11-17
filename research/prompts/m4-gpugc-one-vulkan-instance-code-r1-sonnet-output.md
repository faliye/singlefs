# m4-gpugc-one-vulkan-instance-code-r1：云端正推腿（Sonnet）报告（2026-11-09）

范围：T1–T4 与检查单八格，T1、T4 主攻。立场：核「代码今天做的是不是改法声称的那件事」。方案按 `crates/` 今天的实现来谈。
被判代码：冻结副本 `~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo/`（下文 `crates/...:行号` 都相对它）。
wgpu 源码：`~/.cargo/registry/src/*/` 下的 `wgpu-30.0.1`、`wgpu-core-30.0.1`、`wgpu-hal-30.0.1`、`wgpu-types-30.0.1`（`Cargo.lock` 锁 `wgpu` 30.0.1）。
「量的」= 这一趟我用只读命令读到；「推的」= 读码推出来、没跑过；「转述」= 背景材料或别人报告里的话、我没复核。
我没跑 cargo、没交任何任务。引的每一句代码都在冻结副本里 `grep -nF` 过（见末尾 cite-check 一节）。

## 先说结论（每格一句，细节在各节）

| 格 | 回 |
|---|---|
| T1 产品路径只有一份实例 | 成立。两处要写明：测试进程只做到「一条用例一个实例」（T1.5）；失败一次的实例会一直失败（T1.6） |
| T2 拒绝分得清 | 成立。四臂各交回的成员见 T2.1；核心层的两个成员在这条路径上走不到，代码没写走不到的理由（检查单·走不到） |
| T3 把手寿命与没钉住的地方 | 克隆指同一实例成立；丢掉实例已开的卡照用；枚举超时与枚举为空开卡都是拒、不挂住；`self.clone()` 能在任何机器上钉住，办法见 T3.3，待主 agent 跑 |
| T4 不开特性不碰 GPU | 成立（`Backends::empty()` 不进 Vulkan 加载器，读 wgpu 源码；`#[cfg(test)]` 构造没漏出去） |
| 检查单八格 | 见末尾「判定一览」 |

## T1　过度外推与越位：实例真的只有一份吗

### T1.1　全仓建 wgpu 实例的点（量的）

```
$ grep -rnE 'wgpu::Instance::new|Instance::new\(|InstanceDescriptor' crates --include=*.rs   （冻结副本内，排掉 e202 的零命中）
crates/singlefs-verification/src/gpu_unit_checks.rs:402:        wgpu::Instance::new(wgpu::InstanceDescriptor {
crates/singlefs-verification/src/gpu_unit_checks.rs:404:            ..wgpu::InstanceDescriptor::new_without_display_handle()
crates/singlefs-background-gpu/src/background_gpu.rs:770:            instance: wgpu::Instance::new(wgpu::InstanceDescriptor {
crates/singlefs-background-gpu/src/background_gpu.rs:772:                ..wgpu::InstanceDescriptor::new_without_display_handle()
```

- 产品 GPU 包里只剩一处：`crates/singlefs-background-gpu/src/background_gpu.rs:768-775`（`with_the_backends`），只有两个入口走到它：`created_for_this_process()`（:758-760）和 `#[cfg(test)] fn without_any_backend()`（:763-766）。e202 里 `Instance::new` 零命中。
- 验证包 `crates/singlefs-verification/src/gpu_unit_checks.rs:396-397` 另有一个 `static VULKAN_INSTANCE: std::sync::OnceLock<wgpu::Instance>`，是验证包自己的进程级单例，这一批没动。它与产品 GPU 包没有依赖：`git grep -n 'singlefs-background-gpu\|singlefs_background_gpu' -- crates ':!crates/singlefs-background-gpu' ':!crates/mutations.tsv'` 只命中 `crates/singlefs-core/src/background_compute.rs:5`（文档注释）；产品包 `Cargo.toml` 里没有 `singlefs-verification`。所以同一进程里不会同时有验证包那一个与产品包这一个（依据是依赖图，没起进程数过，推的）。

### T1.2　`Clone` 之后是不是真指同一个实例（读 wgpu 30.0.1 源码，量的）

- `wgpu-30.0.1/src/api/instance.rs:31-32`：`#[derive(Debug, Clone)]` 后接 `pub struct Instance { inner: dispatch::DispatchInstance }`。
- `wgpu-30.0.1/src/dispatch.rs:998`：`dispatch_types! {ref type DispatchInstance: InstanceInterface = ContextWgpuCore, ContextWebGpu, DynContext}`；宏体 `dispatch.rs:723-732` 生成 `#[derive(... Clone)] pub enum $name { Core(Arc<$core_type>), ... }`。所以 `Instance::clone` 克隆的是 `Arc<ContextWgpuCore>`。
- `wgpu-30.0.1/src/backend/wgpu_core.rs:43-44`：`#[derive(Clone)]` 后接 `pub struct ContextWgpuCore(Arc<wgc::global::Global>);`。再一层 `Arc<Global>`。
- 真正的 Vulkan 实例住在 `Global.instance.instance_per_backend`（`wgpu-core-30.0.1/src/instance.rs:132-137` 的 `instance_per_backend.push`），只在 `Global::new`（`wgpu-core-30.0.1/src/global.rs:34-46`）里建一次，入口 `wgpu-30.0.1/src/backend/wgpu_core.rs:781-785`。
- 结论：`VulkanInstanceOfThisProcess`（`crates/singlefs-background-gpu/src/background_gpu.rs:750-753`，`#[derive(Clone, Debug)]`）的 `clone` 只加两层 Arc 引用计数，不再建 Vulkan 实例。成立（读源码，没起进程数 Vulkan 实例）。
- 适配器、设备、队列也各抓一份 `ContextWgpuCore` 的克隆：`wgpu_core.rs:460-463`（`CoreAdapter`）、`:466-471`（`CoreDevice`）、`:562-566`（`CoreQueue`）；枚举时 `context: self.clone()` 在 `:921-924`。T3.1 用它。

### T1.3　产品路径上每条开卡路径各拿到的是哪一份实例

核心层与别的包没有调开卡入口的地方：`git grep -n 'open_background_gpu' -- . ':!crates/singlefs-background-gpu' ':!research' ':!records' ':!crates/mutations.tsv' ':!.claude/kb'` 零输出（量的）。产品包内三个入口：

| 入口 | 实例从哪来 | 位置 |
|---|---|---|
| `open_background_gpu(selection, vulkan_instance)` | 参数 `&VulkanInstanceOfThisProcess`，原样交下一个 | `crates/singlefs-background-gpu/src/background_gpu.rs:811-818` |
| `open_background_gpu_with_the_memory_leaked_in_this_process(selection, leaked, vulkan_instance_handed_over)` | 参数 `&impl VulkanInstanceHandedToTheOpening`；函数内 `vulkan_instance_handed_over.vulkan_instance_for_this_opening()` 取一份克隆，枚举走 `vulkan_adapters_enumerated_within`，克隆再交 `AdaptersEnumeratedByTheInstance` | `background_gpu.rs:837-847`、`:884-889` |
| `open_background_gpu_compute(selection, leaked, vulkan_instance)` | 参数 `&VulkanInstanceOfThisProcess`，原样交上一个 | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:201-210` |
| 产品对 trait 的实现 | `self.clone()` | `background_gpu.rs:800-803`（`self.clone()` 在 :802） |

这三个函数体里没有新建实例；开卡时的枚举与挂着切换现查（`AdaptersEnumeratedByTheInstance`，`background_gpu.rs:942-946`，字段 `vulkan_instance`）用的是同一份实例的克隆。成立。

### T1.4　e202 每个子命令建几份（量的）

`main` 在 `crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs:8450-8488`，一个进程只走一个 `match` 分支：

| 子命令 | 建几份 | 行 |
|---|---|---|
| `geometry-check` | 0。`geometry_check_command(&product)`；内部 `LockstepRunSetting.vulkan_instance: None`（diff 里 `geometry_check_of_a_tier` 一处），只跑 `LOCKSTEP_ARMS[0]`，那是 `ComputeOfTheArm::Cpu`（:2682-2686） | :8453 |
| `lockstep` | 1：在实参位置 `&VulkanInstanceOfThisProcess::created_for_this_process()` | :8454-8457 |
| `concurrent` | 1 | :8458-8461 |
| `sensitivity` | 1：交给 `sensitivity_command`（:7261），它把同一个 `vulkan_instance` 先交 `lockstep_command`、再交 `concurrent_command`，交替跑与并发跑共用一份 | :8462-8469 |
| `judge-trend` | 0 | :8470-8473 |
| `stage-three-cost` | 1 | :8474-8478 |
| `construction-timing` | 0：`GpusOfTheCommand::NotNeeded` | :8479-8481、:7464 |

- 实参位置的临时值活过整条调用（推的）；`std::process::exit` 在 :8487。进程内实例数：要卡的四个子命令各 1，不要卡的三个各 0。
- `opening_of_the_command`（:5587）里 `GpusOfTheCommand::EnumeratedOn(vulkan_instance)`（:5625）把 `vulkan_instance.clone()` 存进 `OpeningOfTheCommand.vulkan_instance`（:5576，`Option<VulkanInstanceOfThisProcess>`）；`LockstepRunSetting`、`ConcurrentRunSetting` 与 `stage_three_*` 的开卡都读 `opening.vulkan_instance.as_ref()`。
- GPU 臂开卡经 `vulkan_instance_for_the_gpu_arm`（:4230-4235）：`None` 时回 `Err(String)`、不静默新建。`open_the_gpu_of_the_arm` 里先判 `arm.compute.uses_the_gpu()`，CPU 臂在取实例之前返回（:4244-4250 一带，diff 里该函数）。
- 结论：sensitivity 的交替跑与并发跑共用一份，成立。

### T1.5　「界大约是 12、用例数小于它」这句外推到了哪里

- 界本身是背景材料转述的第二台读数（NVIDIA 595.99.02，调查任务 9625、9626、9650），我没复核，也没有本机的读数。读它的范围只到「第二台这一个驱动」。
- 代码注释把它写进了 `crates/singlefs-background-gpu/src/background_gpu.rs:748-749`：「第二台上同一个进程里新建到第 12 个实例起，…枚举只剩软件 Vulkan、三张独显一张都不报（前面的实例都已丢掉）」。措辞限在第二台、用「不一定」，没外推到别的驱动。这一句数的是**累计新建数**（括注「前面的实例都已丢掉」），不是同时存活数。
- 这一批「一个进程一个实例」是机制上不再累计（T1.3、T1.4），不靠「小于界」。「用例数小于界」只出现在测试进程这一层。我数了真卡目标里真正建实例的点（量的）：

```
checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs：#[test] 8 条，handed_over_gpus_or_report_not_run( 3 处，created_for_this_process() 0 处
compaction_on_a_real_gpu_through_the_background_gpu_compute_ends_as_on_the_cpu.rs：#[test] 7 条，handed_over 4 处，created_for_this_process() 1 处（:349）
live_slot_kernels_..._reference.rs：6 条，handed_over 3 处，created_for 0
region_record_kernels_..._reference.rs：6 条，handed_over 3 处，created_for 0
checksum_batches_over_two_real_gpus_...：2 条，handed_over 1 处
checksum_batches_over_three_real_gpus_...：1 条，经 common_checksum_batches_over_the_handed_over_gpus/mod.rs:90 的助手取
```

  所以一个目标进程里真正建的实例数最多 5（compaction：4+1），远小于背景材料写的「每个目标 8、6、6、7 条用例」；**实例个数取决于有几条用例走 `handed_over_gpus_or_report_not_run`/`created_for_this_process`，不是用例条数。**
- 外推点：(a) 界是第二台读数，别的驱动的界未知，也没有证明它不随驱动版本变（推的：背景材料第二节自己写了「不是证明了界不会变」）；(b) 一个目标里再加 8 条走交卡路径的用例就够到界，没有机制拦；(c) `created_for_this_process()`（`background_gpu.rs:758`）是普通 `pub fn`，名字写「for this process」，但调用方连调几次都建出几个，「一个进程一个」只靠调用方遵守——这是用户定的「不用单例」的直接后果，不是疏漏，但**产品级没有任何东西拦第二次调用**。该风险没有钉：全仓 `grep -rn 'created_for_this_process' crates` 的调用点共 8 处（e202 main 4 处、e202 单测 1 处、`common_discrete_gpus_of_this_host/mod.rs:520`、`compaction_...:349` 与 `background_gpu.rs` 的定义），没有一处同一进程两次。

### T1.6　失败重开、换设置重开、驱动全藏起来：实例数各是多少

| 场景 | 新增实例数 | 依据 |
|---|---|---|
| 开卡被拒（`Err`）之后调用方再调开卡入口 | 0，用的还是调用方手里那一份 | 三个入口都只取参数（T1.3） |
| 换设置重开（e202 的 lockstep 各臂与重复、sensitivity 的交替后并发） | 0 | `open_the_gpu_of_the_arm` 每次取 `setting.vulkan_instance`（diff 里 `run_lockstep_once`、`run_concurrent_once` 两处） |
| 驱动全藏起来那一次（加载器或驱动缺） | 0；但这一份实例**永久空**，见下 | `wgpu-core-30.0.1/src/instance.rs:132-145` |
| 调用方丢掉实例再 `created_for_this_process()` | +1，回到旧的累计计数 | `background_gpu.rs:758-760` |

「驱动全藏起来」一条的读法（读源码，量的）：`wgpu-core-30.0.1/src/instance.rs:132-145`，`A::Instance::init(&hal_desc)` 返回 `Err` 时只 `log::debug!`，不 push 进 `instance_per_backend`；之后每次 `enumerate_adapters`（`instance.rs:436-445`）只遍历 `instance_per_backend`。所以一个在加载器缺或驱动暂时不可用的那一刻建出的 `VulkanInstanceOfThisProcess`，之后每次枚举都是空，开卡每次拒「点名的卡不在清单里」（点名时）或「GPU 不在」（没点名时）。旧代码每次开卡现建，驱动恢复后下一次开卡就能成。新代码里要恢复只有丢掉重建一条路，而重建又走回累计计数。这是**行为变化**，不是缺陷：调用方是否重建由它决定；但代码和文档注释没有写这一点（`background_gpu.rs:742-749` 只写了「只建一个、反复枚举每次三张都在」这条好的一面）。推的，没跑过；要核可以让主 agent 在无 Vulkan 加载器的环境下起「建实例 → 装上加载器 → 再枚举」，我没有这样的环境说明，所以写「推的」。

长期持有的实例上每次枚举都重新调 `vkEnumeratePhysicalDevices`（`wgpu-hal-30.0.1/src/vulkan/instance.rs:946`，量的），所以驱动侧热插拔后重新枚举是能反映的；`AdaptersEnumeratedByTheInstance` 的「挂着切换现查」依赖的也是这一点，旧代码同样依赖。

## T2　内部矛盾：拒绝分不分得清

被判函数：`crates/singlefs-background-gpu/src/background_gpu.rs:899-939`（`discrete_gpus_and_participating_gpus_admitted_from_the_enumeration`）。它先调 `discrete_gpus_of_the_enumeration_under_the_named`（`crates/singlefs-background-gpu/src/adapter_inventory.rs:126-207`）交成卡清单，再调 `participating_gpus_admitted_by_the_mount_checks`（`adapter_inventory.rs:280-333`，里头调核心层 `admit_background_compute_settings`，`crates/singlefs-core/src/background_compute.rs` 的 `participating_gpu_devices`）。三种可能的返回：`DiscreteGpuListNotSupported`（:907）、新成员（:910-921）、其余九个核心层成员里除 `NamedGpuDeviceNotInTheInventory` 的八个原样进 `SettingsRefused`（:922-937）。

### T2.1　各臂各交回哪个成员（读码推出，下表每行的判法行都对着代码；入口行是我对着 `adapter_inventory.rs:126-207` 与核心层读出来的）

| 输入 | 点名 | 交回 | 依据 |
|---|---|---|---|
| 点名的卡不在这一次枚举交成的清单里（枚举里没有这个地址，或有这个地址而它不是独显） | `Some(非空)` | `NamedGpuDeviceNotOnTheListOfThisEnumeration{device, adapters_in_the_enumeration, unnamed_left_off_the_list}` | `adapter_inventory.rs:139-146` 只收有地址且被点名的；核心层 `participating_gpu_devices` 找不到返 `NamedGpuDeviceNotInTheInventory`（`background_compute.rs` 的 `let Some(device_on_this_host) = ... else`）；本函数 :910 接住 |
| 独显没报总线地址（别的点名的卡在） | `Some` | 成功。没报地址的那张进 `unnamed_left_off_the_list`，不拒 | `adapter_inventory.rs:159-167`：地址空串的不会被点到（:142 判 `!is_empty()`），落到 `NotNamed` 臂记下 |
| 独显没报总线地址，点名的卡又不在 | `Some` | 新成员，且 `unnamed_left_off_the_list` 里带 `DiscreteGpuWithoutAPciBusAddressIsNotSupported{name}` | 单测 `naming_a_card_while_the_discrete_gpu_reports_no_bus_address_is_refused_listing_that_gpu_left_off`（`background_gpu.rs:4702`） |
| 独显没报总线地址，没点名 | `None` | `DiscreteGpuListNotSupported(DiscreteGpuWithoutAPciBusAddressIsNotSupported)` | `adapter_inventory.rs:139-140` 没点名一律 `Participates` → :152 `Err(refused)` → :907 |
| 点名的卡在、读不到设备本地堆，或同一地址报了两次 | `Some` | `DiscreteGpuListNotSupported` | `adapter_inventory.rs:168-176、:180-193` `Participates` 臂 |
| 没有独显、没点名 | `None` | `SettingsRefused(GpuNotPresent)` | `background_compute.rs` 的 `if participating_devices.is_empty()`；单测 `no_discrete_gpu_without_named_cards_is_refused_as_gpu_not_present_through_the_settings`（:4744） |
| 一张独显都看不到（枚举为空）且点了名 | `Some(非空)` | 新成员，`adapters_in_the_enumeration` 为空 | 核心层先判点名；单测 `every_opening_in_one_process_takes_...`（:4629）钉的正是这个 |
| 点了空列表 / 同一张卡点两次 / 上限个数不对 / 上限超显存 / 等的上限为 0 | 各样 | `SettingsRefused(对应核心成员)` | :922-937 的 `refused @ (...)` |
| 等 wgpu 枚举超过等的上限 | 任意 | `AdapterEnumerationFailed{cause}` | :842-845；但在原生 wgpu 后端这一条走不到，见 T3.4 |
| 点名的卡枚举到了而开卡中途失败 | 任意 | `GpuUnusable`（不经本函数） | `crates/singlefs-background-gpu/src/gpus_of_this_host.rs:856` |

- 编译器保证不漏：`BackgroundComputeSettingsRefused` 九个成员（`crates/singlefs-core/src/background_compute.rs:739-768` 数得 9 个），:910 一个加 :923-935 八个，合 9，且没有 `_ =>`。以后核心层加成员，这个 `match` 会编不过——这是好的。
- 有一个核心成员在这条路径上交回的可能性是零，却照样列在 :922-937 的 `|` 里：`ParticipatingGpuDeviceNotOpenedAtTheMount`（卡清单是 `DiscreteGpuInventorySnapshot`，`adapter_inventory.rs:235-242` 只实现 `devices_with_total_memory`，`devices_not_opened_at_the_mount` 取核心层 trait 默认交空，`background_compute.rs:88-90`）与 `GcCpuCompensationThresholdAboveTheWholeCapacity`（只由外层 `admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold` 交，`admit_background_compute_settings` 的文档写「除阈值越界之外的八种」）。这里的写法是「列出来转成 `SettingsRefused`」，不是 `unreachable!`，所以真交回时不 panic，是保守的。没写「走不到」的注释，见检查单·走不到。

### T2.2　与 D26（后台整理与放置回收）已定项 13 第 4 条「GPU 不在」那一格是不是同一件事

条款原文（`.claude/kb/decisions/26-后台整理与放置回收.md:248`，`grep -nF` 核过，条款是长段，下面整行里要用的只引那一句）：

> 挂载或切换那一刻，`background_gpu_memory_limit` 给了而个数与参与的卡数不等、某一个数超过那张卡的显存总量、GPU 不在、驱动不对、点名的卡不存在，或 `background_gc_cpu_compensation_threshold` 超过 100（选 `cpu` 时也判），就拒绝挂载（或拒绝切换）并报错。

同文件 :246 第 2 条：「点名它们照「点名的卡不存在」拒（第 4 条）」。

- 条款把「GPU 不在」和「点名的卡不存在」写成并列的两种触发，都拒挂载。核心层成员里分别对应 `GpuNotPresent`（`background_compute.rs:741`）与 `NamedGpuDeviceNotInTheInventory`（`:743`），文档串一个写「GPU 不在」一个写「点名的卡不存在」。
- 所以新成员对应的是条款的**「点名的卡不存在」**，不是「GPU 不在」。这一批之前这种情形走 `SettingsRefused(NamedGpuDeviceNotInTheInventory{..})`，没走 `GpuNotPresent`。背景材料与改法说的「从『GPU 不在』分出来」按代码读是**不准的说法**：真正被分出来的是 `SettingsRefused` 里的 `NamedGpuDeviceNotInTheInventory`，`GpuNotPresent` 仍留在 `SettingsRefused`，没动。（代码读出来的；改法文字是转述，我没有看到单独的改法声称原文，只看到背景材料第一节这一句。）
- 分界的实际判据是**有没有点名**：点名了又没找到 → 新成员（哪怕整个枚举为空，即「一张卡都看不到」，用户眼里就是 GPU 不在）；没点名又没有独显 → `GpuNotPresent`。这个判据来自核心层的 `participating_gpu_devices`（先判点名后判空，`background_compute.rs`），**这一批前后一致**，不是新引入的。两个成员都是条款要求的「拒绝挂载并报错」，拒的结局相同；差别只在报错标签与带的诊断。
- 灰区一处：驱动全藏起来 + 点了名 → 新成员 + `adapters_in_the_enumeration: []`。区分「加载器没装 / 驱动被藏」与「驱动报了零个设备」靠不了这个成员，只能看日志（`wgpu-core-30.0.1/src/instance.rs:138-144` 在 `debug` 级，`wgpu-hal-30.0.1/src/vulkan/instance.rs:946-952` 在 `error` 级）。不算内部矛盾，算诊断有限。
- 不会把该报「GPU 不在」的报成新成员**以外**的东西：没点名时新成员不可能出现（:910 需要核心层交 `NamedGpuDeviceNotInTheInventory`，而这个只在点了名时由 `participating_gpu_devices` 交）。反过来，点了名时 `GpuNotPresent` 也不可能出现（`participating_devices` 非空，名字逐个找，找不到先返回）。成立。

### T2.3　调用方有没有哪一处还按旧成员 `match`、把新成员归错（量的）

- 产品代码里对 `BackgroundGpuOpenRefused` 取值的地方：`background_gpu_compute.rs:205-210` 用 `?` 原样上抛；e202 的 :4256、:4269、:7850、:7950 都是 `format!("开后台 GPU：{refused:?}")`，只做 `Debug` 格式化，不 `match`。核心层没有调用点（T1.3）。没有一处按 `SettingsRefused(NamedGpuDeviceNotInTheInventory{..})` 取。
- 测试里 `tests/compaction_on_a_real_gpu_...:363` 取 `SettingsRefused(GpuNotPresent)`，仍对（该用例的设置没点名）。其余点名卡不存在的用例（`compaction_...:253`、`background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:772`、`background_gpu.rs:3647`）走的是核心层 `admit_*` 入口，返回的是 `BackgroundComputeSettingsRefused` 本身，不经 `BackgroundGpuOpenRefused`，不受影响。
- 同一条条款的「点名的卡不存在」于是在两层各有名字：开卡层 `BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration`，挂载/切换层核心 `BackgroundComputeSettingsRefused::NamedGpuDeviceNotInTheInventory`。这是分层，不是矛盾；但今后要写「点名的卡不存在」的统一处理，两个都要认。

### T2.4　`# Errors` 文档与成员是否一一对应

- `BackgroundGpuOpenRefused`（`background_gpu.rs:170-189`）五个成员：`SettingsRefused`、`NamedGpuDeviceNotOnTheListOfThisEnumeration`、`DiscreteGpuListNotSupported`、`AdapterEnumerationFailed`、`GpuUnusable`。
- `open_background_gpu`（:811）与 `open_background_gpu_with_the_memory_leaked_in_this_process`（:837）的 `# Errors` 写「五种」，对得上。`open_background_gpu_compute`（`background_gpu_compute.rs:201`）的 `# Errors` 逐个列了五种，对得上。内部函数 :897-898 列三种，对得上函数体里三个出口。成立。

## T3　哪一条最脆弱：把手的寿命与没钉住的地方

### T3.1　实例的寿命（读码，量的；没跑进程）

| 持有者 | 持有什么 | 撑到什么时候 | 位置 |
|---|---|---|---|
| 调用方 | `VulkanInstanceOfThisProcess`（Arc 两层） | 调用方自己丢 | `background_gpu.rs:750-753` |
| `BackgroundGpu`（`pub type BackgroundGpu = BackgroundGpuOverTheCards<GpuInUse>`，:402） | `discrete_gpus_present_now: Box<dyn DiscreteGpusPresentNow + Send>`（:398）里装着 `AdaptersEnumeratedByTheInstance{vulkan_instance 的一份克隆}`（:942-946，开卡处 :884-889） | 这一次挂载的把手被丢掉 | |
| 每张开着的卡里的 wgpu 适配器、设备、队列 | 各自一份 `ContextWgpuCore`（`Arc<Global>`） | 这些对象被丢掉 | `wgpu-30.0.1/src/backend/wgpu_core.rs:460-463、:466-471、:562-566` |
| `GpuMemoryLeakedInThisProcess`（泄漏的那一份） | 只有 `Arc<Mutex<Vec<(GpuDeviceIdentifier, u64)>>>`——每张卡上泄漏了多少字节的账，**不持有任何 wgpu 对象**（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:737-739`） | 调用方持有多久算多久；与实例寿命无关 | |
| 真正泄漏的卡（等超时没做完的提交） | `std::mem::forget(gpu)`（`gpus_of_this_host.rs:921`）：整张 `GpuInUse` 连设备与队列不析构 | 进程结束 | |

- 「调用方丢掉实例之后已开的卡还能不能用」：能。设备、适配器、队列各自抓着 `Arc<Global>` 的克隆，`Global`（连同 Vulkan 的 `VkInstance`）要所有克隆都丢了才释放。`BackgroundGpu` 里还多一份。成立（读源码）。
- 因此 `BackgroundGpuOverTheCards` 字段注释（`background_gpu.rs:396-397`）写的「排在卡后面，卡都丢了才丢它」在内存安全上已不是必须：靠 Arc，次序无关；它写的是「尽量保持旧次序」，不算错。
- 泄漏的卡把它那份 `Arc<Global>` 一起泄漏，等于这个进程里这一个 `VkInstance` 永不释放。旧代码里每次开卡一个实例、泄漏一张卡就泄漏那一次的整个实例（累计更快撞界）；新代码只泄漏一份，不比旧的差。
- 「丢掉再新建会不会回到老问题」：会。累计新建数照加（T1.6 末行）。调用方想在两次挂载之间换新实例（比如想从「驱动恢复」里恢复，T1.6）就走回计数。要不要提供「重新枚举而不重新建」的办法，是设计问题，不在这一批；我把它记作风险，不当缺陷。

### T3.2　`self.clone()` 没有变异行：现状（量的）

- 变异表 `crates/mutations.tsv` 第 7016-7019 行是这一批的四行（我用只读脚本解析：每行 6 列，每行的「原文」在冻结副本与主工作区里各恰好命中 1 次）。第 7016 行把开卡函数里的 `vulkan_instance_handed_over.vulkan_instance_for_this_opening()` 换成另建一个，由单测 `every_opening_in_one_process_takes_the_vulkan_instance_handed_over_instead_of_creating_one` 的「替身记下取了 3 次」钉住。它钉的是**开卡函数取的是 trait**，不钉**产品对 trait 的实现**（`background_gpu.rs:800-803`）。
- 所以现状：把 :802 的 `self.clone()` 换成另建一个，没有任何测试会红，除非在第二台上跑真卡的 30 轮用例（它在第 12 个实例起才红）。这与背景材料第二节一致。

### T3.3　不靠第二台驱动、任何机器都能钉住的办法（我写了，没跑）

- 原理（读 wgpu 30.0.1 源码，量的）：`wgpu::Instance` 的相等比较是把比较转到内部（`wgpu-30.0.1/src/api/instance.rs:38` `impl_eq_ord_hash_proxy!(Instance => .inner)`），内部的 `DispatchInstance` 枚举 derive 了 `PartialEq`（`dispatch.rs:723-727` 宏体），`Arc<ContextWgpuCore>` 的比较落到 `ContextWgpuCore` 的相等——它是 `impl_eq_ord_hash_arc_address!(ContextWgpuCore => .0)`（`backend/wgpu_core.rs:750`；宏 `cmp.rs:71-80`），比的是 `Arc::as_ptr`，即 `Arc<Global>` 的地址。所以两个 `wgpu::Instance` 相等 ⇔ 指同一个 `Global`；`Clone` 之后相等、各自新建不相等。
- 因此单测里用不开后端的实例（`without_any_backend()`，`Backends::empty()`，不进 Vulkan 加载器，T4.1）就能比：`held.vulkan_instance_for_this_opening().instance == held.instance`。`mod tests` 是 `background_gpu.rs` 的子模块，读得到私有字段 `instance`。
- 测试补丁与变异行（草稿目录，写成补丁，没打到仓里）：
  - `~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-sonnet/proposed/the-product-takes-the-held-instance.patch`（sha256 f21ac1ac90ee37d2d0e571e23bc25b95bb4e93f1aa316eddb5d337d2edfd5abf）：在 `background_gpu.rs` 的 `mod tests` 末尾加 `the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one`，两句断言：手里的与取出来的相等；另建的与手里的不相等（对照，证明比较分得出）。
  - `.../proposed/the-product-takes-the-held-instance.mutation-row.tsv`（sha256 55488f2bf829421c9292e5f68797148b9148845091a5ea73a5ae78f48ec0e5fc）：一行 6 列，把 :801-803 换成 `Self::with_the_backends(wgpu::Backends::empty())`（每次另建一个、不碰 Vulkan，任何机器都能编能跑），原文在冻结副本里恰好命中 1 次（我数了）。
  - `.../proposed/run-held-instance-test.sh`（sha256 e9fe725c112d322cdc2e0732a47f4a964ab07853769684165f96356330418bd0）：在冻结副本的拷贝上打补丁、跑新用例（该绿）、打变异、再跑（该红）；`bash -n` 过。
- 这一对要在副本里打上、编、跑绿，再打变异、跑红，才算钉住。命令在「要主 agent 跑的命令」一节。**现在是推的：没编过，也没红过**。

### T3.4　枚举超时、枚举为空，开卡是拒还是挂住，拒的是哪个成员

- 枚举为空：不挂住，开卡函数返回 `Err`。点了名 → 新成员（`adapters_in_the_enumeration: []`）；没点名 → `SettingsRefused(GpuNotPresent)`（T2.1）。读码，钉在单测 `every_opening_in_one_process_takes_...`（:4629）与 `no_discrete_gpu_without_named_cards_...`（:4744）。
- 枚举超时：代码上拒成 `AdapterEnumerationFailed{cause}`（:842-845）。**但在 wgpu 30.0.1 的原生后端，这条超时走不到**：`Instance::enumerate_adapters` 在 `wait_for_the_wgpu_future(<future>, …)` 的实参位置**已经同步跑完了** `vkEnumeratePhysicalDevices`（`wgpu-30.0.1/src/backend/wgpu_core.rs:917-928` 先 `.collect()` 出 `Vec<DispatchAdapter>`，最后 `Box::pin(ready(adapters))`），交进 `wait_for_the_wgpu_future`（`background_gpu.rs:2602-2620`）的是一个**立即就绪**的 future，第一次轮询就 `Ready`；计时（`GpuWaitDeadline::starting_now`）在 future 已经构造好之后才开始。所以 `wait_limit` 管不到一个卡死的枚举：驱动在 `vkEnumeratePhysicalDevices` 里挂住，调用线程就挂住。这是旧代码就有的（旧代码是同一个调用形态），这一批没引入，也没修。推的（读码），没起进程让它真卡死。
- 在「长期持有的实例」下多了一点：挂着切换的现查（`identifiers_of_the_discrete_gpus_present_now`，:947 起）与另一个线程的开卡共用同一个 `Global`，`enumerate_adapters` 会往 `hub.adapters` 注册适配器 id（`wgpu-core-30.0.1/src/instance.rs:1142-1154`），适配器 drop 时移除（`:1249-1254`）。每次枚举的临时 `Vec<wgpu::Adapter>` 在 `identifiers_of_...` 函数末尾就丢，不累积。读码，没压过。

## T4　不开特性不碰 GPU

### T4.1　`Backends::empty()` 在 wgpu 30.0.1 上真不装 Vulkan 加载器吗（读源码，量的）

- 唯一装加载器的地方：`wgpu-hal-30.0.1/src/vulkan/instance.rs:581-583`（`ash::Entry::load()`，在 `init_with_callback` 开头）。`grep -rn 'Entry::load\|libvulkan' wgpu-hal-30.0.1/src --include=*.rs` 除这两行外只命中 `swapchain/native.rs` 里讲 Android 的注释。
- 调用它的路径：`wgpu-30.0.1/src/api/instance.rs:63-` `Instance::new` → `ContextWgpuCore::new`（`backend/wgpu_core.rs:781-785`）→ `Global::new`（`wgpu-core-30.0.1/src/global.rs:34-44`）→ `wgpu-core-30.0.1/src/instance.rs:66-95` 的 `Instance::new` → `try_add_hal`（:98-）。`try_add_hal` 在 :106-111 先 `self.supported_backends |= A::VARIANT.into();`，随后 `if !instance_desc.backends.contains(A::VARIANT.into()) { ...; return; }`——**在 `A::Instance::init`（:132）之前就返回**。`Backends::empty()` 不含 Vulkan ⇒ 不进 `init` ⇒ 不进 `Entry::load`。
- 另一个会 panic 的口子：`Instance::new` 在 `Self::enabled_backend_features().is_empty()` 时 panic（`api/instance.rs:63-70`），只看编译时启的特性，与请求的 `backends` 无关；本包 `wgpu = { ..., features = ["vulkan", "wgsl", "std"] }`（`crates/singlefs-background-gpu/Cargo.toml:11`）启了 vulkan，不 panic。
- 枚举也不进驱动：`wgpu-core-30.0.1/src/instance.rs:436-445` 对 `instance_per_backend` 过滤，空实例的列表是空的，`vulkan_adapters_enumerated_within` 的 `enumerate_adapters(Backends::VULKAN)` 返回空 `Vec`。
- 成立。限定：这是读源码；没起进程在无 Vulkan 的机器上观察。

### T4.2　三处要不要建真实例

| 位置 | 建不建真实例 | 依据 |
|---|---|---|
| 单测 `every_opening_in_one_process_takes_...`（`background_gpu.rs:4629`） | 不建真 Vulkan 实例：唯一的实例是 `VulkanInstanceOfThisProcess::without_any_backend()`（:4632），`Backends::empty()` | T4.1；之后开卡的枚举走 `instance_per_backend` 为空的 `Global`，不碰驱动 |
| e202 的 CPU 臂 | 不建、不用：`open_the_gpu_of_the_arm` 先判 `arm.compute.uses_the_gpu()` 为假就返回（diff 里该函数，冻结副本 :4244 一带），在取实例之前。但**所在子命令在 `main` 里已建了实例**（`lockstep`/`concurrent`/`sensitivity`/`stage-three-cost`，:8454-8478）——这四条子命令的 CPU 臂也在一个已加载 Vulkan 加载器的进程里跑。这四个子命令本来就包含 GPU 臂，不算越位 | T1.4 |
| `geometry-check` | 不建（`main` 臂 :8453 不带实例；内部 `vulkan_instance: None`，只跑 CPU 臂） | T1.4 |
| `construction-timing` | 不建（`GpusOfTheCommand::NotNeeded`，:7464；main :8479-8481） | T1.4 |
| e202 自带的真卡单测 `the_gpu_arm_on_a_real_card_counts_...`（:11304-11307） | 建真实例，但带 `#[cfg(feature = "tests-on-a-real-gpu")]`（:11303），不开特性不编 | 量的 |

### T4.3　`#[cfg(test)]` 的构造有没有漏到非测试代码

- `without_any_backend` 的定义前有 `#[cfg(test)]`（`background_gpu.rs:763-764`），私有（无 `pub`）。`grep -rn without_any_backend crates` 的引用只有定义（:764）和测试里的一处（:4632）。
- `with_the_backends`（:768）不带 `cfg(test)`，也是私有，被 `created_for_this_process`（产品）与 `without_any_backend`（测试）调；非测试代码里没法传 `Backends::empty()` 进它——因为唯一的非测试调用点 :759 写死 `wgpu::Backends::VULKAN`。成立。
- 公开 trait `VulkanInstanceHandedToTheOpening`（:795）可在包外实现，但返回类型 `VulkanInstanceOfThisProcess` 在包外只能通过 `created_for_this_process()`（真 Vulkan）造出，包外的测试替身没法造「不开后端的实例」。这是限制，不是漏：包外用例想用替身只能带一个真 Vulkan 实例。

## 检查单八格

### 走不到的理由二次查实：成立

- 新加的产品代码里没有 `.expect(`、`unreachable!`、`panic!`、`_ =>`：对改动全文 `grep -nE '^\+.*(\.expect\(|unreachable!|panic!|_ =>)'` 只命中两行（e202 单测里的 `.expect("CPU 臂挂载得成")`、`.expect("枚举得到适配器")`）。**这两行都不是新加的逻辑**：对应的 `-` 行就在同一块 diff 里（`grep -nE '^-.*(\.expect\(|...)'` 命中 `-                .expect("CPU 臂挂载得成");` 与 `-        let discrete_gpus = discrete_gpus_seen_by_this_process().expect("枚举得到适配器");`），是 rustfmt 折行后重新出现在 `+` 里。所以背景材料写的「新加 `expect` 依赖的不变量」实际上是原有两处的重排，没有新不变量。
- 第二个 `expect` 的前提「枚举得到适配器」靠枚举不超时；T3.4 读出原生后端枚举超时走不到，因此该断言只会在真没有独显时失败；且它在 `#[cfg(feature = "tests-on-a-real-gpu")]` 里（e202 :11303），不开特性不编。
- 没写成注释的「走不到」：在三份产品文件的 `+` 行里 `grep -nE '走不到|不会到|不可能|永远不|一定不|不会为'` 零命中。唯一接近的是 :922-937 里把两个本路径交不回的核心成员（T2.1 末）无注释地列进 `SettingsRefused`。那是「列出来而不是 `unreachable!`」，真交回时不 panic，是保守写法；但没有一行注释说它们为什么走不到。**建议**（不是缺陷）：在 :933 旁边写一句「本路径的卡清单恒无开卡没成的卡、阈值不在这里校验」。
- 在任何落盘动作之前返回：成立。开卡路径整条不写盘：`grep -nE 'std::fs|File::|OpenOptions|fs::write|\.write_all\(' background_gpu.rs background_gpu_compute.rs gpus_of_this_host.rs adapter_inventory.rs` 只命中 `background_gpu.rs:4445` 一处读 `/proc/self/status`（测试里）；新函数 :899-939 在开卡（`gpus_opened_at_the_mount`）之前返回，连设备都还没起。

### 类型与名字：成立

- 新名字十三个都在冻结副本里定义恰好 1 次（`grep -rnE "(fn|struct|enum|trait) <名>\b"`：`vulkan_instance_for_this_opening` 命中 3 处，是 trait 声明、产品实现、测试替身实现三处同一个方法，不是重名）。`NamedGpuDeviceNotOnTheListOfThisEnumeration` 既是成员又是结构体名，一个在枚举成员作用域、一个在模块作用域，能编（`background_gpu.rs:176、:194`）。
- 空泛词：拿 `.claude/naming-vague-words`（45 个词）逐个拆词对，十三个名字没有一个整个名字都由空泛词拼成（只有 `process` 一个词命中，其余是说出内容的词）。这是我用只读脚本做的近似，判据以门禁 `code-source-discipline` 的 vague-names 格为准，我没跑。
- 序数、位置代称、缩写：新名字里没有。新文档串里有一处措辞不准：e202 `vulkan_instance_for_the_gpu_arm` 的文档写「（这条命令枚举独显时建的）」（diff 里该函数），实际实例是 `main` 在 :8454-8478 建的，枚举只是用它；`OpeningOfTheCommand` 的文档写的是「枚举它们的那一份进程级 Vulkan 实例」，措辞正确。后者是对的，前者小误。

### 不兼容旧代码与旧格式：成立

- 对改动全文 `grep -nE '^\+.*(legacy|旧|兼容|迁移|migrat|原先|此前|起初|最初|后来|改成|换成了|以前)'` 零命中；`grep -nE '^\+.*(version|VERSION|layout|LAYOUT|compat|legacy|migrate|schema)'` 零命中；`grep -nE '^\+.*(20[0-9][0-9]-[0-9][0-9]|第 [0-9]+ 批|批 ?[0-9]|singlefs-[0-9][0-9]|三方|判决)'` 零命中（日期、批号、会话名、判决名都没进代码注释）。
- 这一批不碰盘上格式：改动文件只有产品 GPU 包的三份源码与它的测试（`crates.patch` 的 `diff --git` 行共 9 个文件，其中 `crates/singlefs-core` 与 `crates/singlefs-verification` 一个都没有）。
- 没验证的一项：注释里是否有门禁 `comment-history` 格认的历史写法，我只扫了常见词，没跑 `scripts/gate/comment-history-check.py`。`.claude/gate.d/comment-history-lag.tsv` 登记 `background_gpu_compute.rs` 8 行、`e202_gpu_cpu_gc_comparison.rs` 6 行；`background_gpu.rs` 不在表里（现 0 命中）。**该格数字待主 agent 跑**（命令见下）。

### 不新建判定库：成立

- 改动全文 `grep -nE '^\+.*(fs::|File::|OpenOptions|tsv|sqlite|rocksdb|RocksDB|write_all|\.write\(|json|serde|ProductWriter|product\.emit)'` 只命中一行 `product.emit(`，是 e202 原有的 `gpus` 产物行被搬进嵌套 `match`（`enumerated=false reason=...` 那一支原样），格式没变：`enumerated={}` 的取值由 `gpus == GpusOfTheCommand::Enumerated` 改为 `vulkan_instance.is_some()`，对「枚举成功」「不要卡」两种输入结果相同（枚举失败的那支在 `map_err` 里提前 `emit` 并返回，没走到这行）。这是实验产物文本行，不是判定存放处。
- 补丁 `crates.patch` 里 9 个文件无 `*.tsv` / 判定相关新文件；变异表 `crates/mutations.tsv` 是登记表，不是判定库。

### 所有任务多机可用：成立（有一处限定）

- 三份产品文件的改动里没有主机名、地址、绝对路径、`env::var`：`grep -nE '^\+.*(env::var|hostname|/home/|/tmp|127\.0|localhost|HOME|NFS|Command::new)'` 零命中；三份产品文件全文 `grep -nE 'env::var|hostname|/home/|/tmp|127\.0|localhost|\$HOME'` 只命中 `background_gpu.rs:4545`（原有的测试里读一个标记变量）。
- 新真卡用例 `opening_the_handed_over_gpus_again_and_again_...`（`checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:547`）取卡走 `handed_over_gpus_or_report_not_run`（`common_discrete_gpus_of_this_host/mod.rs`），读服务交的 `GPU_CARDS_BY_PRIORITY` 与多机配置判法 `scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-addresses`（同文件头与 :447-448），不读本机独有状态；服务没交卡打 `NOT_RUN_NO_GPU_ADAPTER` 报未跑。
- 限定：用例常量注释写了「第二台上每开一次新建一个实例时从 0 数第 11 轮起就开不成」（:40），数字是第二台的读数，只用来选 30 这个轮数，不参与判定；在没有这个界的机器上 30 轮照绿。

### 测试优先多线程：成立（理由只部分写明）

- 真卡目标带 `--test-threads 1` 的依据是 `.claude/rules/verification.md:57`「单个 GPU 测试二进制只用分到的卡（哪几张卡由服务分），带 `--test-threads 1` 跑」；用例文件头第 6 行写了「用 GPU，经 `task-server/task submit` 交（卡与清场由服务定），带 `--test-threads 1`」。
- 30 轮这一条为何是一条串行循环：常量注释写了「30 轮罩得住这个转折、在一条用例里跑得完」（:40）。没有单独写「为什么不拆成几条并行」：每一轮要在同一个实例上丢掉上一轮的把手再开，拆成并行就是几个实例同时开同一批卡，既不是它要钉的情形，也会超出服务分的卡。**读法是对的，原因没写**。
- 不要卡的单测每条只开卡 3 次（`OPENINGS_ON_THE_HANDED_OVER_INSTANCE`，:4616），不值得切片，同意（推的，没量过）。

### 带 --item 的命令与项：成立（核了那句话；变异行能否被列出是读登记推的）

- 「产品 GPU 包不在崩溃点重放合表里」：`.claude/gate.d/crash-point-replay-items.tsv` 对 `product-gpu|background` 零命中（`grep -c` 输出 0）。「没有新的崩溃枚举用例」：`git diff HEAD --stat -- .claude/gate.d/stage-inputs.tsv` 没有改动行，改动文件里没有 `crates/singlefs-verification`。`stage-inputs.tsv:30` 把 `tests-of:singlefs-background-gpu` 登记为 crash-point-replay 阶段的输入、`crash-point-replay.sh` 里有 product-gpu-real-card-tests 一格，所以包不是与该阶段无关，而是只在那一格里（整格跑，不是 `--item` 项）；「这一批没有 1d 那三条 `--item`」的说法站得住。
- Verus 登记：`.claude/gate.d/verus-pins.tsv`、`verus-trust-list.tsv`、`verus-trait-impls-outside-lag.tsv` 对 `background` 零命中。
- 新变异行能否被 `--list-items` 列出：`crates-mutation-replay.sh --list-items` 是门禁阶段，我不跑。读登记：`crates/mutations.tsv` 第 7016-7019 行各 6 列，每行的「原文」在冻结副本与主工作区里各恰好命中 1 次（用只读脚本数的）；格式与旁边已登记的行一致。**推的：能列出；没看到 `--list-items` 输出**，待主 agent 跑。
- 注意第 7016 行变异后的代码是 `VulkanInstanceOfThisProcess::created_for_this_process()`，不开特性跑 `--lib` 时它会在变异后的二进制里**真起 Vulkan 加载器**（只发生在变异副本里，不在产品里）；单测断言先因取次数不是 3 而红，不影响判红，但「不开特性不碰 GPU」在变异跑里不成立。如要保持纯净，可改成 `with_the_backends(wgpu::Backends::empty())`。读法推的，没跑。

### 新崩溃流多罩了什么：成立

- 这一批没改 `.claude/gate.d/stage-inputs.tsv`（`git diff HEAD --stat` 无输出）；`crash-case:` 行里与 GPU 有关的只有第 68 行 `crash-case:compaction-relocation-on-the-gpu-path-stream`，测试住 `singlefs-verification`（它的测试目标名 `crash_enumeration_compaction_relocation_stream_on_the_gpu_path`），与产品 GPU 包无依赖（T1.1），本批没碰。改动文件清单见上一格。

## 什么现象会推翻各条结论

| 结论 | 推翻它的现象 |
|---|---|
| T1 产品路径只有一份实例 | 全仓出现第二处 `wgpu::Instance::new`（今天只有 `background_gpu.rs:770` 与验证包 `gpu_unit_checks.rs:402`）；或 `open_background_gpu*` 的函数体里出现 `created_for_this_process()`；或 e202 同一个子命令里 `created_for_this_process()` 被调第二次（今天 :8456、:8460、:8468、:8477 各在不同的 `match` 臂里，同一进程只走一臂）；或实测 e202 某子命令的 `vkCreateInstance` 次数大于 1（用加载器调试日志数，推的，没测）。 |
| T1.2 `Clone` 指同一个实例 | `wgpu::Instance` 两个克隆比较不相等（T3.3 的新测试第一句断言会红）；或 wgpu 版本升级后 `Instance` 不再是 `Arc` 内部共享（`Cargo.lock` 的 30.0.1 钉死，升级时重读 `dispatch.rs`、`wgpu_core.rs:43-44`）。 |
| T1.6 失败一次的实例永久空 | 在无 Vulkan 加载器的环境里建实例、装好加载器后同一个实例再枚举得到非空——那 `instance_per_backend` 的读法错了。我没有这样的环境，写「推的」。 |
| T2 拒绝分得清 | 找到一个输入：点了名而卡不在清单里，却没交回 `NamedGpuDeviceNotOnTheListOfThisEnumeration`；或没点名、没有独显却交回了新成员；或核心层加了第十个成员、这个 `match` 仍编过（它没有 `_ =>`，编不过才对）。 |
| T2.2 新成员对应条款的「点名的卡不存在」 | `.claude/kb/decisions/26-后台整理与放置回收.md:248` 的并列清单改成把「GPU 不在」写成含「点名的卡不存在」的上位词；或核心层 `participating_gpu_devices` 改成先判空再判点名（那样「驱动全藏起来 + 点名」会交 `GpuNotPresent`，今天的新成员判据就变了）。 |
| T3.1 丢掉实例已开的卡照用 | 一个用例：开卡、丢调用方手里的实例、再在已开的卡上交一批，panic 或 `GpuFailureCause`。今天读码没有，没跑。 |
| T3.3 相等比较能钉住 `self.clone()` | 变异后新测试仍绿（说明相等比较比的不是 `Arc<Global>` 地址）；或补丁在 `without_any_backend()` 上编不过。 |
| T3.4 枚举超时走不到 | 在原生 wgpu 后端上读到 `enumerate_adapters` 返回一个会 `Pending` 的 future；今天 `wgpu-30.0.1/src/backend/wgpu_core.rs:928` 是 `Box::pin(ready(adapters))`。 |
| T4 不开特性不碰 GPU | 不开特性的 `--lib` 测试里观察到 `libvulkan` 被打开（`wgpu-hal-30.0.1/src/vulkan/instance.rs:581-583` 被执行）；或 `without_any_backend` 出现在非 `#[cfg(test)]` 的调用里。 |

## 要主 agent 跑的命令

我没有写模型（只有一个补丁与一个变异行的提议），所以没有 `tools/rerun.sh`。下面一项是 T3.3 的复核：

1. 补丁用例该绿、变异后该红（不要卡、不碰 Vulkan 加载器）。
   - 命令：
     `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-background-gpu 的 lib 测试里新补丁用例与相邻的一条; 范围=2 条测试各跑一遍，变异后再跑 1 条; 为什么=钉住产品对开卡取实例的实现交回调用方手里的那一个实例，变异后该红' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-sonnet/proposed -- bash /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-sonnet/proposed/run-held-instance-test.sh /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo /home/fy5090/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-sonnet/held-instance-copy`
   - 在哪个目录起：上面的 `--cwd`（草稿目录里的 `proposed`）；脚本自己在 `held-instance-copy` 里建拷贝，拷贝由 `task-server/task clean <号>` 之外还要 `rm -r`（95M，不带 target）。
   - 看哪几行：`task-server/task log <号> --all` 里 `HELD_INSTANCE_GREEN rc=0`、`HELD_INSTANCE_NEIGHBOUR_GREEN rc=0`、`HELD_INSTANCE_MUTATION_APPLIED count=1`、`HELD_INSTANCE_MUTATED_RED rc=` 后面是**非 0** 且日志里有 `test background_gpu::tests::the_product_hands_the_opening_the_instance_it_holds_instead_of_a_new_one ... FAILED`。四行齐了才算钉住；少一行、`count` 不是 1、或变异后仍 `rc=0` 都是没钉住。
   - 判读：绿与红同时成立，才说明新测试既不空转、也抓得到「产品实现每次另建」。
2. （可选，注释历史写法的实数，我只扫了常见词）`python3 scripts/gate/comment-history-check.py --list /home/fy5090/code/singlefs`，看输出里三份文件的命中数是否仍是登记数（`background_gpu_compute.rs` 8、`e202_gpu_cpu_gc_comparison.rs` 6、`background_gpu.rs` 无）。我不确定它要不要范围声明，请主 agent 按 `.claude/hooks/run-scope-guard.sh` 定。
3. 「新变异行能否被 `--list-items` 列出」：我按派发提示不写 `.claude/gate.d/` 下的命令，所以这一格只做了读登记（见检查单·带 --item），待主 agent 决定怎么核。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 | 成立 | 产品路径只在 `background_gpu.rs:770` 建实例，三个开卡入口都取参数，e202 四个要卡的子命令各 1 份、sensitivity 两点共用；`Clone` 是两层 Arc 共享。保留意见：`created_for_this_process()` 谁都能多次调（靠约定）；一份在驱动缺时建出的实例永久空（行为变化，没写文档） |
| T1.5 外推 | 成立（限定） | 界是第二台读数；测试进程里实例个数取决于走交卡路径的用例个数，最多 5，不是用例条数 |
| T2 | 成立 | 新成员对应条款的「点名的卡不存在」，不是「GPU 不在」；分界判据是有没有点名；调用方没有按旧成员 `match` 的；`# Errors` 与五个成员一一对应；核心层两个成员在这条路径上交不回，无注释 |
| T3 | 部分成立 | 丢掉实例已开的卡照用（读源码）；`GpuMemoryLeakedInThisProcess` 不持实例；枚举为空是拒、枚举超时在原生后端走不到；`self.clone()` 今天没钉，我写了任何机器都能跑的补丁与变异行，待主 agent 跑 |
| T4 | 成立 | `Backends::empty()` 在 `wgpu-core-30.0.1/src/instance.rs:108-111` 提前返回，到不了 `Entry::load`；`without_any_backend` 只在 `#[cfg(test)]` |
| 检查单·走不到 | 成立 | 新加代码无 `expect`/`unreachable!`/`panic!`/`_ =>`；背景说的两处 `expect` 是原有行的重排；整条路径不落盘 |
| 检查单·类型与名字 | 成立 | 十三个新名字各定义 1 次，无空泛词拼成；一处文档措辞小误（e202 `vulkan_instance_for_the_gpu_arm`） |
| 检查单·不兼容旧 | 成立 | 新加行里没有旧版本、迁移、日期、批号、会话名；注释历史写法的实数待主 agent 跑 |
| 检查单·不新建判定库 | 成立 | 唯一碰到的写是 e202 原有的 `gpus` 产物行，格式没变 |
| 检查单·多机可用 | 成立 | 三份产品文件的新增行无主机名、地址、本机路径；新真卡用例走服务交的卡与多机配置判法 |
| 检查单·多线程 | 成立（理由部分写明） | `--test-threads 1` 的依据在规则与文件头；30 轮为何是一条串行循环没单独写 |
| 检查单·--item | 成立（`--list-items` 推的） | 合表与 Verus 登记都不含产品包；变异行 6 列、原文各命中 1 次 |
| 检查单·新崩溃流 | 成立 | 没动 `stage-inputs.tsv`，改动文件里没有验证包 |

## 没做什么

- 不判别的腿的格，不替主 agent 采纳或出判决；没往统一任务服务交任务、判决要引的数没复跑（命令在上面「要主 agent 跑的命令」一节）。
- 没起 cargo、没编、没跑任何测试：补丁与变异行是写出来的，只做了 `patch --dry-run`（退出 0）与 `rustfmt --check`（退出 0，对补丁后的 `background_gpu.rs`）。
- 没复核背景材料转述的第二台读数（界约 12、任务 9810 全绿、9755 两个包测试全绿）；没读调查任务的原始日志。
- 没在无 Vulkan 加载器的环境里观察 `Backends::empty()` 与「失败一次永久空」；两条都是读源码。
- 没跑门禁阶段（`.claude/gate.d/` 下的脚本）；`comment-history` 格与 `--list-items` 只做了读登记。
- 草稿目录：`~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1-sonnet/`，里面有 `proposed/`（补丁、变异行、脚本、补丁前后的 `background_gpu.rs`）和 `patchcheck/`（补丁试打用的一份单文件，已无用）。
