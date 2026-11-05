# m4-gpugc-code-r1 正推腿（Sonnet）报告：七·3 GPU 垃圾回收代码对 D26 已定项 13 与七·3 验收

<!-- doc-lint:not-numbers T1 T2 T3 T4 T5 T6 -->

日期：2026-10-30。腿：three-way-forward（Sonnet）。草稿目录 `~/.local/share/singlefs/work/m4-gpugc-code-r1-sonnet/`。

## 0　读法与前提

- 代码行号写仓根相对路径（`crates/…`）。我用 `cmp` 对过冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r1/repo/` 与此刻主工作区：`crates/singlefs-core/src`、`crates/singlefs-background-gpu/src` 与两个包的 `tests/` 下 482 个 `.rs`/`.wgsl` 文件只有一份不同（`crates/singlefs-harness/tests/writable_mount_rehearsal_takes_the_devices_rewritten_by_the_acquisition_from_the_check_of_every_device.rs`，本报告不引）；`.claude/kb/decisions/26-后台整理与放置回收.md`、`.claude/kb/mount-parameters.md`、`.claude/kb/milestone/04-fourth-txn.md` 三份逐字节相同；`crates/mutations.tsv` 前 6238 行相同（第 6239 行起别的会话追加的，本报告引的行都在 6239 之前）。所以仓根相对路径的行号与冻结副本的行号是同一个。
- 引 kb：D26 已定项 13 八条在 `.claude/kb/decisions/26-后台整理与放置回收.md` 第 245–252 行（一条一行：第 1 条 245、第 2 条 246、第 3 条 247、第 4 条 248、第 5 条 249、第 6 条 250、第 7 条 251、第 8 条 252）。七·3 验收在 `.claude/kb/milestone/04-fourth-txn.md:145`。挂载参数表在 `.claude/kb/mount-parameters.md` 第 22–26 行。
- 补丁块写补丁目录名（`m4-gpugc-<名>`），不写 diff 文件的行号。
- 本机能做的我做了：读码、`grep -nF`、`cmp`、一张 257 行变异行的机械表（`draft/rows.tsv`，锚点在冻结副本里都恰好命中一次、点名的用例名都在 `crates/` 里找得到）。能跑的变异行在量法拷贝里跑了几条，结果在 T6。真卡用例一条没跑。
- 标记：「读码推」是只读代码推的、没跑；「跑过」带命令与输出。
- 节序：T1、T2、T3、T5、T6、T4（T4.3 要等一条自拟变异的结果，最后写入），不影响各节自足。

## T1　越位：代码管的比条款多还是少

### T1.1　挂载与切换每一支拒（同一套校验，挂载与切换共用）

挂载入口 `crates/singlefs-core/src/mount.rs:8986` 调 `admit_background_compute_settings_with_the_gpu_compute_and_the_gc_cpu_compensation_threshold`；切换入口 `crates/singlefs-core/src/background_compute.rs:1301` 调同一个函数。所以挂载与切换的每一支拒一模一样（我没有找到只在一边判的拒）。第 4 条原文（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）：「个数与参与的卡数不等、某一个数超过那张卡的显存总量、GPU 不在、驱动不对、点名的卡不存在，或 `background_gc_cpu_compensation_threshold` 超过 100（选 `cpu` 时也判），就拒绝挂载（或拒绝切换）并报错」。

| # | 代码这一支（文件:行） | 对第 4 条哪一分句 | 判 |
|---|---|---|---|
| 1 | 阈值大于 100 拒：`crates/singlefs-core/src/background_compute.rs:736`（排在最前，选 `cpu` 也过它） | 「`background_gc_cpu_compensation_threshold` 超过 100（选 `cpu` 时也判）」 | 是条款说的；恰好 100 放过（代码用 `>`），条款「0 到 100」含 100，对得上 |
| 2 | 选 `gpu` 而没递实现：`crates/singlefs-core/src/background_compute.rs:746`（交回 `GpuNotPresent`） | 「GPU 不在」；实现要由调用方递进来是第 8 条后的射程段（第 254 行）「挂载时由调用方把它的实现递进去」（`.claude/kb/decisions/26-后台整理与放置回收.md:254`） | 是条款说的（读作 GPU 不在） |
| 3 | 参与的卡一张都没有：`crates/singlefs-core/src/background_compute.rs:859` | 「GPU 不在」 | 是条款说的 |
| 4 | 点名的卡不在卡清单：`crates/singlefs-core/src/background_compute.rs:927-936`（`NamedGpuDeviceNotInTheInventory`） | 「点名的卡不存在」 | 是条款说的 |
| 5 | 上限个数不等于参与的卡数：`crates/singlefs-core/src/background_compute.rs:863` | 「个数与参与的卡数不等」，第 3 条「不补齐」（`.claude/kb/decisions/26-后台整理与放置回收.md:247`） | 是条款说的 |
| 6 | 某个上限大于那张卡的显存总量：`crates/singlefs-core/src/background_compute.rs:874`（严格大于，上限恰好等于总量放过） | 「某一个数超过那张卡的显存总量」 | 是条款说的；恰好等于不拒，对得上「超过」。注意「显存总量」条款没定义，代码取 Vulkan 最大那块设备本地堆（`crates/singlefs-background-gpu/src/background_gpu.rs:342-367`，条款没写） |
| 7 | 参与的卡里有挂载那一刻没开成的：`crates/singlefs-core/src/background_compute.rs:888-900`（`ParticipatingGpuDeviceNotOpenedAtTheMount`） | 最近的是「驱动不对」与第 4 条「挂载那一刻量代价的提交超时不重交，算那张卡没开成」，但条款只说「算那张卡没开成」，没写没开成就拒整个挂载 | 条款没写明；`.claude/kb/mount-parameters.md:23` 写了「参与的卡挂载那一刻没开成……拒绝挂载或切换」，来自第 45 批实现员的读法 |
| 8 | 点名列表是空的：`crates/singlefs-core/src/background_compute.rs:915` | 无 | 条款没写而代码拒了；`.claude/kb/mount-parameters.md:23` 自己写「条款没写，代码第一版拒」 |
| 9 | 同一张卡点了两次：`crates/singlefs-core/src/background_compute.rs:920` | 无（第 3 条「按次序一张卡一个」没说重复） | 条款没写而代码拒了；同上 |
| 10 | 选 `gpu` 而等 GPU 的上限为 0：`crates/singlefs-core/src/background_compute.rs:885` | 无（第 4 条只写了「一批等 GPU 超过挂载参数 `background_gpu_wait_limit`」，没写取值范围） | 条款没写而代码拒了；`.claude/kb/mount-parameters.md:25` 记「大于 0」 |
| 11 | 独显缺 PCI 总线地址、读不到设备本地堆、两张报同一个总线地址：`crates/singlefs-background-gpu/src/adapter_inventory.rs:105`（另两支：读不到设备本地堆在 :112，同一个总线地址出现两次在 :120），整个挂载拒（`BackgroundGpuOpenRefused::DiscreteGpuListNotSupported`，`crates/singlefs-background-gpu/src/background_gpu.rs:298-299`） | 无 | 条款没写而代码拒了。这三支只看独显，且在筛「点名」之前判：本机有一张缺地址的独显，即使调用方只点了别的好卡，也整个挂载拒。比条款多拒的是未被点名的卡 |
| 12 | 枚举适配器等超过等 GPU 的上限：`crates/singlefs-background-gpu/src/background_gpu.rs:290-295`（`AdapterEnumerationFailed`） | 「GPU 不在、驱动不对」 | 是条款说的（读作驱动不对）；这一处用的是 `wait_limit`，不重交（第 4 条「挂载那一刻量代价的提交超时不重交」） |
| 13 | 参与的卡开卡失败：`crates/singlefs-background-gpu/src/background_gpu.rs:145-151`（`GpuUnusable`） | 「驱动不对」 | 是条款说的；给了点名列表而没点名的卡开不了则跳过不拒（`crates/singlefs-background-gpu/src/background_gpu.rs:146`），条款没写这条跳过（第 45 批的读法） |

条款写了而代码放过的：

- A　「GPU 不在、驱动不对」在切换时读的是挂载那一刻的卡清单快照：`crates/singlefs-background-gpu/src/background_gpu.rs:1610-1613`（`devices_with_total_memory` 回 `self.discrete_gpus_after_the_opening`）。挂载之后掉了的卡、驱动重置了的卡，切换时校验不到，照收；到第一批算时才以 `GpuFailed` 之类出错停下。条款写的是「挂载或切换那一刻」拒。读码推，没造世界跑；`crates/singlefs-core/src/background_compute.rs:64-76` 的 `GpuInventory` 本身只定义了一次读，也没有「现枚举一次」的方法。
- B　选 `cpu` 时除阈值外不判卡与上限：`crates/singlefs-core/src/background_compute.rs:846-848`。第 4 条括注「（选 `cpu` 时也判）」紧挨着阈值那一句，字面读法是只有阈值选 `cpu` 也判，所以代码取窄读，与 `.claude/kb/mount-parameters.md:23-25` 一致；但第 4 条主句「`background_gpu_memory_limit` 给了而个数与参与的卡数不等……就拒绝挂载」没写「选 `gpu` 时」，宽读的话 `cpu` 模式下给错上限也该拒。条款歧义，代码取窄读。

### T1.2　切换在哪一行取生效；停着时的解除是不是当场

- 请求只记成待生效：`crates/singlefs-core/src/background_compute.rs:1306`（`self.requested_from_the_next_batch = Some(admitted);`）。同一批开头之前请求几次，最后一次盖前面的（注释 `crates/singlefs-core/src/background_compute.rs:1281`）。
- 换上：`crates/singlefs-core/src/background_compute.rs:1333`（`begin_a_batch` 里 `requested_from_the_next_batch.take()`），交给实现在 :1335。`begin_a_batch` 在一批整理的第一件事调：`crates/singlefs-core/src/mounted_session.rs:3644`。批大小、预算、节奏（前台 fsync、写带宽）的判在它前面（`crates/singlefs-core/src/mounted_session.rs:3279-3332`），与设置无关。
- 一批之内不换：钩子（只供测试）在 `crates/singlefs-core/src/mounted_session.rs:3657`，在 `begin_a_batch` 之后；`end_the_batch` 断言生效的设置没变：`crates/singlefs-core/src/background_compute.rs:1383-1386`。结论：切换在批的开头取生效，不在中间；符合第 1 条「从下一批开始生效；一批之内不换」（`.claude/kb/decisions/26-后台整理与放置回收.md:245`）。
- 停着时解除是当场：`crates/singlefs-core/src/background_compute.rs:1307`（`self.stopped_after_a_compute_error.clear();`）在记待生效之后立刻做，不等下一批；校验不过的切换在 :1301-1305 的 `?` 就返回，不清停着。对第 4 条「当场解除停下，从下一批起用新设置」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）：对得上。
- 但有一处不对称（读码推，没造世界跑）：挂载时一张卡都用不上的设置在 `of_a_mount` 里当场记成停着（`crates/singlefs-core/src/background_compute.rs:1124-1129`），所以一趟开头的那一判（`crates/singlefs-core/src/mounted_session.rs:3207`）在任何写之前拦住；切换到这样的设置，停着要到下一批的 `begin_a_batch` 才记（`crates/singlefs-core/src/background_compute.rs:1336-1337`），而这之前 `crates/singlefs-core/src/mounted_session.rs:3207` 看到的是「没停着」，之后 :3230 起的删除续做可以先发写。用例注释和 :3205-3206 的注释都写「停着……一个写都不发（删除续做也不走）」，切换到用不上的设置这一条路径上这句不成立。后果：只多发几步删除续做，是用户本来就该推的活；「该跑了」也会在 `crates/singlefs-core/src/mounted_session.rs:2395` 被取走而不放回（停着那一支 `NotStarted` 是在 :2388 取走之前返回的，所以停着时「该跑了」留着；这条路径上丢）。

### T1.3　卡的分类字段、后端

- 按什么字段：wgpu 适配器信息的 `device_type`，一一对到 `AdapterDeviceKind`（`crates/singlefs-background-gpu/src/adapter_inventory.rs:29-37`），只有 `DiscreteGpu` 算（:44）；集显、虚拟显卡、软件渲染（`wgpu::DeviceType::Cpu`，:35）、`Other` 都不进「全部」。对第 2 条「集成显卡与软件 Vulkan 适配器不算」（`.claude/kb/decisions/26-后台整理与放置回收.md:246`）：对得上，且比条款严一点（虚拟显卡、认不出类型的也不算，条款没提）。
- Vulkan 以外的后端混不进来：实例与枚举都只开 `wgpu::Backends::VULKAN`（`crates/singlefs-background-gpu/src/background_gpu.rs:287` 与 :291）；wgpu 的 Cargo 特性也只开 `vulkan`（`crates/singlefs-background-gpu/Cargo.toml` 的 wgpu 一行）。条款射程写的是「集成显卡与软件 Vulkan 适配器」，没写别的后端，代码只认 Vulkan。
- 一个已知缝：一台机器上 Vulkan 把独显报成 `IntegratedGpu`（如部分笔记本混合显卡）时会被当集显排除，条款里的「独立显卡」没有第二个判据。读码推，没有造样本。

### T1.4　补跑只管 GC，与「那个后台任务停下」分不分得开

- 今天 `BackgroundTask` 只有一个成员（`crates/singlefs-core/src/background_compute.rs:41-44`），「那个后台任务停下」与「GC 补跑」在语义上同一个对象，运行时分不开也不用分。
- 补跑的 `match task` 是穷尽的（`crates/singlefs-core/src/background_compute.rs:1237-1246`，一臂 `CompactionRelocation`），以后加一种后台任务，编译器逼着在这里补一臂，可以写「照旧停着」。这一处对第 4 条「补跑只管 GC，打包搬迁停着时不补跑」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）有编译期的守着。
- 停着任务表 `BACKGROUND_TASKS_WITH_A_GPU_PATH: [BackgroundTask; 1]`（`crates/singlefs-core/src/background_compute.rs:1035`）没有这种守着：数组长度是写死的字面量、不由枚举成员数决定，加成员编译器不报；全仓 `grep -rn BACKGROUND_TASKS_WITH_A_GPU_PATH crates` 只有 :1035 与 :1046 两处，且是私有常量，测试读不到（主 agent 记下的第 ③ 处，判在 T5.7）。

## T2　内部矛盾：核心层与 GPU 包各自的约定对不对得上

### T2.1　trait 三段方法两侧对照（入出类型、错误成员）

| 方法 | 核心层 trait（文件:行） | GPU 包实现（文件:行） | 入出类型是否一致 |
|---|---|---|---|
| 甲 `verify_unit_checksums` | `crates/singlefs-core/src/background_compute.rs:274-277`：入 `&[UnitChecksumToVerifyOnTheGpu]`，出 `Result<Vec<GpuEntryDecision<UnitChecksumOutcome>>, BackgroundGpuComputeFailed>` | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:423`：先换成包内的 `UnitChecksumToVerify`，经 `unit_checksums_by_both_kernels` 取两份核各一份结果，`reconciled_unit_checksums` 比对，再换回核心层的类型 | 一致；条数一对一由核心层断言（`crates/singlefs-core/src/background_compute.rs:469-473`） |
| 乙 `classify_region_records` | `crates/singlefs-core/src/background_compute.rs:284-289`：入 `region`、`&[RegionRecordToClassifyOnTheGpu]`、`&[CentralMappingEntry]`，出 `Vec<GpuEntryDecision<AllocationInRegion>>` | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:451` | 一致 |
| 丙 `count_live_slots_per_cluster_segment` | `crates/singlefs-core/src/background_compute.rs:295-300`：入分配记录、`UnitAreaOfOneDevice`、`&[ClusterSegmentEntryKey]`，出 `Vec<GpuEntryDecision<u64>>` | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:483` | 一致 |
| 换设置 | `crates/singlefs-core/src/background_compute.rs:306` | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:508`（转给 `BackgroundGpu::use_the_settings_from_the_next_batch`，`crates/singlefs-background-gpu/src/background_gpu.rs:1138`；三段计数不动） | 一致 |
| 一张都用不上 | `crates/singlefs-core/src/background_compute.rs:313`（默认交回 `None`） | `crates/singlefs-background-gpu/src/background_gpu_compute.rs:513` | 一致 |

错误成员（核心层 `BackgroundGpuComputeFailed`，`crates/singlefs-core/src/background_compute.rs:173-201`）与 GPU 包产出的对应：

| 核心层成员（行） | GPU 包在哪映射出（`crates/singlefs-background-gpu/src/background_gpu_compute.rs`） | 产品包真的会产出吗 |
|---|---|---|
| `GpuFailed`（:175） | `core_failure_of_the_gpu`（:214）的非等待类各臂（:224 之后、:265 之前） | 会 |
| `WaitForTheGpuExceededTheLimit`（:180） | 同函数的两臂（:224 与其后一臂）：「这一批的提交等超时」与「前一次超时的提交又没等到」 | 会，但只出在没经重交包装的两处：交活之前先等前一次超时提交的那一等（`crates/singlefs-background-gpu/src/background_gpu.rs:1273`，乙段 :1386、丙段 :1509 同形），与换设置时量代价的提交（`crates/singlefs-background-gpu/src/background_gpu.rs:1171`）。别处超时都被重交包装成下一行 |
| `GpuUnusableAfterResubmissions`（:190） | 同函数 `UnusableAfterResubmissions` 臂 | 会 |
| `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`（:198） | `core_failure_of_every_participating_gpu_left_out` | 会 |
| `InputNotSupportedOnTheGpu::EntryLargerThanAChunkOnTheDevice`（:245） | `core_failure_of_the_smallest_piece_not_held`，且 `input_bytes` 字段填的是「最小一片合占的字节数」，不是核心层成员注释说的「`input_bytes` 是它自己的输入字节数」（`crates/singlefs-core/src/background_compute.rs:243-244`） | 会；字段含义两侧不一致 |
| `InputNotSupportedOnTheGpu::MappingKeyLongerThanTheKernelsHold`（:256） | `core_failure_of_the_region_record_batch` | 会 |
| `InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice`（:251） | 无 | 不会：全仓 `grep -rn TablesLargerThanABindingOnTheDevice crates` 只有定义（`crates/singlefs-core/src/background_compute.rs:251`）与一处测试里手造的值（`crates/singlefs-harness/tests/compaction_batches_on_the_gpu_path_through_the_background_gpu_compute.rs:849`）。核心层留着一个产品包不产的成员，没有用例钉它的映射 |

核心层对三种做同一件事（`crates/singlefs-core/src/background_compute.rs:170`：「这一批放弃、意图留在盘上、那个后台任务停下并报这个错，不换回 CPU」）；实现上 `advance_compaction_by_one_publish` 对所有 `BackgroundComputeError` 一视同仁（`crates/singlefs-core/src/mounted_session.rs:3699-3711`）。第 4 条原文「挂着的时候 GPU 出错（含重交 3 次都超时），正在算的那一批放弃（意图留在盘上，目标态单调，已定项 2），那个后台任务停下并报警」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）：对得上。

### T2.2　切片之后的合并，各段的不变量

- 甲（校验和，`crates/singlefs-background-gpu/src/checksum_kernels.rs:397-470`）：先按单元切片（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:186`），每片两份核各算一次，按单元接回：查表那一份用「前一片的寄存器喂后一片那么多个零字节再异或后一片的值」（零字节算子矩阵，:271），逐位移那一份用乘 x 的幂（:350）；两份各用各的合法，与第 8 条「不共用核函数」同向。不变量：一个单元的片按字节次序首尾相接且不漏（:427 与 :434 的两句断言）、整单元只一片就直接用核交回的、合出来的值在合成之后才与期望值比，不比每片。合成公式本身有 lib 用例（`crates/singlefs-background-gpu/src/checksum_kernels.rs:605`、:652）在 CPU 上逐位对过。我另核了一个前提：CRC-32C 的两片合成式 `crc(A‖B)=shift(crc(A),|B|)⊕crc(B)` 在初值与末尾取反都带上时成立（与 zlib 的 crc32_combine 同形），代码注释「初值与末了的异或互相抵掉」是对的。
- 乙（区域扫描，`crates/singlefs-background-gpu/src/region_record_kernels.rs`）：两层切。第一层映射按落点区间切（:687，同一个落点上的全部条目落同一片）；第二层单元比头一片（`UNIT_HEAD_PIECE_INPUT_BYTES = 628`，:92）长的才切，头一片之后的片只算 CRC 寄存器、一片接前一片读回的寄存器（:922-965）。628 的依据：码 2 索引节点最宽的头 `86 + 2×255` 再加 29 字节预留位 = 625，补到整字 628；我对了 `crates/singlefs-core/src/unit.rs:526-533` 与 :557 的头宽式子和预留位 29（`crates/singlefs-core/src/unit.rs:126`），一致。跨映射片的合并：`walking_classification_over_the_mapping_slices`（:834）与 `searching_classification_over_the_mapping_slices`（:857）两份各写各的，规则相同：取本片（落点所在片）的分类；本片是已死而别的片是「key 与位置不符」就改判后者。不变量：各片的落点区间首尾相接（`expect("各片的落点区间首尾相接、罩住全部落点")`，:1033）、一片之下的几段按记录次序首尾相接（:996 的断言）。lib 里有 CPU 模拟的对拍用例：映射切片（:2073）、单元切片（:2310）。真核（WGSL）的行为只有真卡用例罩着，我没跑。
- 丙（选段打分，`crates/singlefs-background-gpu/src/background_gpu.rs:1478-1566`）：记录表按记录切片，各片数出的活槽数逐段相加（`live_slots_summed_over_the_record_slices`），两份核各加各的。
- 切片之后一个条目跨两片时两份核的比对是不是仍按条目：是。三段都是先在各自的合并函数里把片合成「每条目两份结果」，才进 `reconcile_the_two_kernel_outcomes`（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:98`，键是条目键，不是片号）。证据：甲 `crates/singlefs-background-gpu/src/checksum_kernels.rs:216-238`、乙 `crates/singlefs-background-gpu/src/background_gpu_compute.rs:468`（交进 `reconciled_region_record_classifications` 的是整条记录的两份分类）。

### T2.3　两份核比对与第 8 条三个计数的落点

- 条款：「同一个条目连续 2 轮（N = 2）都不确定，交 CPU 那一份实现单独判一次……连续不确定的计数只放在内存里，重挂清零。每一批报确认、不确定、交 CPU 判的条目各多少」（`.claude/kb/decisions/26-后台整理与放置回收.md:252`，下面三问都对这一条）。
- 连续不确定计数住在 GPU 包实现里，三段各一份 `ConsecutiveUncertaintyCounts`（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:132` 起的结构、:146 建时全空），`HashMap` 只在内存（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:47`）。重挂清零：挂载每次 `open_background_gpu_compute` 新建一份实现（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:185`）。切换不清（:508 的注释与用例 `crates/singlefs-background-gpu/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:991`）。对得上。
- 三个计数的落点：GPU 包的 `ReconciledBatch` 也有三个数（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:37`），但产品路径只取 `decisions`、丢掉它们；运行时看到的三个数是核心层自己从判定里数的：`GpuEntryCounts::note_the_decisions`（`crates/singlefs-core/src/background_compute.rs:407-417`）→ `decisions_of_one_segment`（:461）→ `GpuWorkOfTheBatch`（:422）→ 每批记账 `BackgroundComputeOfOneBatch.path_taken`（:944-948、:985-992）→ `CompactionRun.background_compute_of_each_batch`（`crates/singlefs-core/src/mounted_session.rs:3174`）。两边各数一份、产品只读核心层那一份；GPU 包那一份是死数（只有它自己的 lib 用例读）。
- 一处条款没写的口径：计数的「轮」是「这个条目出现在一批里并且不确定」，没出现在某一批里的条目计数原样留着（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:7`：「没出现在这一批里的条目，计数原样留着」）。所以「连续 2 轮」实际是「该条目连续 2 次出现并都不确定」，中间隔了几批没扫到它不打断。条款只说「连续 2 轮」，没定义轮。键是 `(盘, 槽, 跨度)`（`crates/singlefs-core/src/background_compute.rs:99-103`），同一个键在两次整理意图之间可能指不同的记录（槽释放后重新分配），留着的 1 轮会算到新记录头上：新记录第一次不确定就被直接交 CPU 判。方向是安全的（多交 CPU），代价是少了一轮 GPU 复核。读码推，没造样本。
- 一处跨段不一致（读码推）：出错放弃的是整批，但三段是顺序交的（乙在先、甲在后，`crates/singlefs-core/src/region_scan.rs:581-651`；丙在选段处，`crates/singlefs-core/src/mounted_session.rs:3792` 起）。乙段先成功、甲段才出错时，乙段的连续不确定计数已经推进了，这一批却放弃。GPU 包文档写的「出错时这一批的判定一条都不交、计数不动」（`crates/singlefs-core/src/background_compute.rs:268`）只管单个方法，不管批内多段。后果同上：下一批同一个条目少一轮复核，安全方向。

### T2.4　`failure_under_the_settings_handed_in` 挂载与切换读到的是不是同一个状态

- 同一个函数、同一份状态。挂载：`of_a_mount` 里 `hand_the_gpu_settings_to_the_implementation` 然后 `tasks_stopped_when_no_gpu_is_usable_under_the_settings_handed_in`（`crates/singlefs-core/src/background_compute.rs:1125-1129`）；切换生效：`begin_a_batch` 里同样两步（:1335-1337）；两处经同一个函数（:1041-1057）调 trait 方法（:699），GPU 包答的是 `batch_not_split_over_the_gpus()` 读出的 `self.participating` 与 `left_out_under_their_memory_limit`（`crates/singlefs-background-gpu/src/background_gpu.rs:1193-1210`），这些字段在 `use_the_settings_from_the_next_batch` 的末尾一次写完（:1181-1184）。所以没有错开一批：交设置与读失败在同一个 `begin_a_batch` 里、同一批开算之前。
- 一个缝：交设置（:1335 → `lock()` :683）与读失败（:1337 → `lock()` :698）是两次各自取锁，之间没有持锁。实现把手可以几处共用（`crates/singlefs-core/src/background_compute.rs:347-348`「几处共用同一份」），两个会话共用一个把手、各自在这两次取锁之间插入对方的交设置，就会读到对方那一组的状态。单会话（一个把手只给一个会话）里不会发生。条款没写把手能不能共用。读码推，没造两个会话。

## T3　出错与资源边界

### T3.1　重交 3 次

条款（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）：「挂着的时候一批等 GPU 超过挂载参数 `background_gpu_wait_limit`，放弃的那一批在同一张卡上重交，至多 3 次，都超时才报警、判那张卡用不了」。

| 问 | 代码 | 判 |
|---|---|---|
| 重交几次 | `crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:117`（`RESUBMISSIONS_AFTER_A_TIMEOUT: u32 = 3`），循环 :152（`for _resubmission in 0..RESUBMISSIONS_AFTER_A_TIMEOUT`），共 1 + 3 = 4 次提交 | 是条款说的 |
| 每次等的上限是不是同一个 `wait_limit` | 是同一个数，但每一次等都开一个新的整窗：提交后等做完与等两份回读共用一个期限（`crates/singlefs-background-gpu/src/background_gpu.rs:716`）；重交前等前一次超时提交的那一等另开一个同长的期限（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:135`；交活前先等那一等 :115 与 :156 也各开一个同长的期限） | 条款没写「等前一次」要不要另算一窗。一个片的最坏等待是 1 + 3 × 2 = 7 个窗（重交前等前一次满窗、再交一次又满窗）；记录里第 49 批交回写的是「最坏约 4–7 个等的上限」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:547`），对得上。默认 10 秒（`crates/singlefs-core/src/background_compute.rs:481`）下一个片最坏 70 秒才报警；挂载参数表也记着默认是推的、待量（`.claude/kb/mount-parameters.md:25`） |
| 重交的是「批」还是「片」 | 包装 `outcome_resubmitted_after_timeouts` 套在每一个片的提交上（`crates/singlefs-background-gpu/src/background_gpu.rs:1302`、:1414、:1535），不套在整批上；重交的是同一个片、同一份输入、同一张卡（闭包里 `&mut *gpu_in_use`），前面已做完的片的结果留着 | 主 agent 记下的 ⑤ 的判：「同一张卡」对得上；「同一批」按片读才对得上，条款只写了「批」，没写批被切成几片时重交哪一个。条款没说，代码取最小单位，不重做已完成的片。方向无害 |
| 先前超时的提交回来了怎么算 | 重交前先在一个窗里等它：做完就放掉它占的账再交（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:385-406`），没做完就算这一次重交失败、不交（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:153` 的 `and_then`；失败原因归到 :156-157，不是等超过上限的原因当场交回） | 条款没写；代码这样算是为了不越显存上限（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:12-15` 的文件头） |
| 另一种先前超时的提交：下一批交活前的那一等 | `crates/singlefs-background-gpu/src/background_gpu.rs:1273`（甲）、:1386（乙）、:1509（丙）调 `room_once_the_submission_running_past_its_wait_completed`，这一等在重交包装之外：一个窗等不到就当场交回出错（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:461-474` 把它命名成 `EarlierTimedOutSubmissionNotCompleteWithinTheWaitLimit`），映射成 `WaitForTheGpuExceededTheLimit`（`crates/singlefs-background-gpu/src/background_gpu_compute.rs:224` 起的两臂），后台任务停下 | 条款写的是「都超时才报警」；这一支一个窗就报警，没有 3 次。它只出在一张卡已经判过用不了、又被切换点回来的第一批上（任务停着时批不开，除非补跑在 CPU 上，不进 GPU）。条款没写这一支 |
| 一张卡用不了之后这一批去哪 | 整批放弃：各卡的切法在交任何片之前先全算完（`crates/singlefs-background-gpu/src/background_gpu.rs:1264-1291`），一张卡出错整批返回（:1309），不把它的活挪给别的卡；核心层把这一批放弃、后台任务停下（`crates/singlefs-core/src/mounted_session.rs:3699-3711`） | 是条款说的（「正在算的那一批放弃」，:248） |

### T3.2　显存账

- 账的单位是一个片（一段）：`ChunkFootprint::bytes_held_by_a_chunk` = 输入缓冲（按 4 字节向上取整，最少 4）+ 每条的固定占用 + 每片的固定占用（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:51-55`）。
- 实际记账的是真建的缓冲：甲段 `参数 16 + 输入 + 描述 + 4 × 结果字节`（`crates/singlefs-background-gpu/src/background_gpu.rs:788-792`），其中 4 倍结果 = 两份核各一个结果缓冲 + 各一个回读暂存。记账前还断言真建的字节数与切批按的 footprint 逐字节相等（:793-806）。乙段 :914-922、丙段 :1031-1039 同形。记账写进 `GpuMemoryAccount::hold`，超上限直接 `assert!`（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:327-338`）。
- 条款（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）说「攒批、缓冲、暂存区一起算在里面」。对号：缓冲 = 参数、输入、描述、结果；暂存区 = 回读缓冲（那 4 倍里的两份）；「攒批」只能对到一个片里的输入。
- 条款没落地的一句（打中，见末尾「打中的」第 2 条）：同一行写着「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」。代码里没有任何跨批攒的东西：`grep -rn 攒` 在 GPU 包与 `background_compute.rs`、`mounted_session.rs` 的 GPU 路径里零命中（命中的 `crates/singlefs-core/src/mounted_session.rs:180` 等是发布层的攒批会话，不相干）；每一批整理各自按三段交一遍。
- 账之外的占用：账只加应用层建的缓冲字节；wgpu/驱动的对齐与子分配开销、六条管线与着色器的常驻占用、绑定组都不在账里。所以条款「任何时刻都不许比它多」在账内成立，真卡上实际显存占用可能略多于账。读码推，没量（我不起 GPU）。
- 切片之后每片峰值是否仍不超上限：切批 `chunks_that_fit_on_the_gpu` 的 `fits` 把「显存上限、输入缓冲和按条缓冲各自装进绑定上限、条数不超一次派发上限」四件一起判（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:98-107`），上限用的是「上限扣掉此刻还占着的」（:70-78 的 `with_the_room_left_after`）；账里一次只会有一个片在跑：交完一片就等它做完再放账，等不到的记成卡上还在跑的一次（:356-378），之后这张卡不交新片（:327-331 的第一句断言）。所以账内峰值不超上限由「切批 + 断言」两层守着；断言红是 panic，不是返回错误：panic 之后实现把手的互斥锁中毒，同一个把手上之后每次 `lock()` 再 panic（`crates/singlefs-core/src/background_compute.rs:374-378`）。
- 真卡用例「4156 字节超过了上限 500 字节」那一次红：修的是产品行为和用例两处，不是单改用例，也不是改账。过程：
  - 4156 = 量代价那一段 `COST_PROBE_CHUNK`（一个 4096 字节的单元，`crates/singlefs-background-gpu/src/background_gpu.rs:100-103`）的合占：4096 + 12 + 4×8 + 16（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:541` 第 48 批交回那一行的算法）。开卡时没经切批直接记账，所以上限 500 的卡一建就撞 :327 的断言（`records/2026-10-19-里程碑四GPU垃圾回收开工.md:536` 记的那次红）。
  - 第 48 批（补丁块 `m4-gpugc-gpu-reds-investigator`）：产品改成开卡前先判量代价那一段装不装得下上限，装不下拒挂载（该补丁里新加的 `CostProbeLargerThanTheMemoryLimit`），并把真卡用例改成「开卡不设上限、再用切换换成小上限」，另加一条断言「500 字节装不下那一段 4156 字节：拒挂载」。账（`hold`）与断言本身没动。
  - 第 49 批（补丁块 `m4-gpugc-adaptive-split-writer`）：用户 2026-10-28 定「装不下应该自适应的分片」，产品改成量代价那一段也按上限切片、代价取各片之和（`crates/singlefs-background-gpu/src/background_gpu.rs:597-630`），第 48 批的拒挂载成员删了；真卡用例改回开卡就给小上限，上限不再写死 500，而是现算 `two_segments`（`crates/singlefs-background-gpu/tests/live_slot_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:304`），并断言它不小于参与的卡至少要装得下的那一片。
  - 结论：现在的用例不再含 500 字节这个数。红的根因（量代价那一段没切片）是改产品修的；用例两次改了输入。条款支持这个方向：`.claude/kb/decisions/26-后台整理与放置回收.md:247` 写「挂载那一刻量代价（第 2 条）的那一段同样切片，代价取各片之和」。

### T3.3　连一片都装不下 与 参与的卡一张都不剩

- 「最小的一片」的口径：三段各有最小片，取最大的一个：校验和 64、选段打分 104、区域扫描 972（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:396` 起的函数，用例钉三个数，972 在 :1194）。上限比 972 小的卡一律「不用」，哪怕它装得下校验和那一段。条款（:247）写「连最小的一片都装不下，这张卡就不用」，没说三段取哪个；取最大是保守读法（任何一段都要能交，否则这张卡交不了一整批）。
- 区域扫描那一段的 972 是与输入无关的下限；一个落点上条目多的映射，实际最小片更大，那种批装不下时不是「不用这张卡」，而是这一批出错停下（`GpuCannotHoldTheSmallestPiece` → `InputNotSupportedOnTheGpu`，`crates/singlefs-background-gpu/src/background_gpu_compute.rs:332` 起的函数）。条款 :247 写「挂着时单条装不下也切」，没写切不动时怎么办。
- 成员：参与的卡一张都不剩 = `EveryParticipatingGpuLeftOutUnderItsMemoryLimit`（`crates/singlefs-core/src/background_compute.rs:198`），挂载与切换都不拒，后台任务当场记成停着（:1031 的 `GpuUnusableUnderTheSettingsHandedIn`）。是条款说的（:247「参与的卡因此一张都不剩时挂载照样不拒，后台任务照第 4 条进出错停着」）。

### T3.4　卸载时的泄漏边界

- 丢 `BackgroundGpu`：逐张卡丢，记着「等超过上限、没做完的提交」的整张卡 `std::mem::forget`（`crates/singlefs-background-gpu/src/gpus_of_this_host.rs:556`），别的照常丢（:549 的函数；调用在 `crates/singlefs-background-gpu/src/background_gpu.rs:1593-1604`）。泄漏的判据是账上的 `has_a_submission_running_past_its_wait()`（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:307-309`），不是「还有没做完的提交」的真实情形：等提交时出任何错（不只超时）账上也记成可能在跑（:245-255）。主 agent 记下的 ④ 的判：边界就是「账上记着一次没等到做完的提交」的卡，不多不少；等得到的卡不泄漏。
- 泄漏的是：整张卡的设备、队列、六条管线和 wgpu 为那次提交留着的缓冲（永不析构，进程活着就在）。主机内存与显存都没有上限：显存是那一片的缓冲，有上限时 ≤ 上限，不给上限时只受绑定上限 2 GiB 一个缓冲的限制（`crates/singlefs-background-gpu/src/background_gpu.rs:96`）。
- 泄漏之后进程里还能再挂：包里没有进程级的静态（`grep -n "static \|OnceLock\|thread_local" crates/singlefs-background-gpu/src/*.rs` 只有两处无关的 `&'static str`）；`open_background_gpu` 新建 `wgpu::Instance` 与设备（`crates/singlefs-background-gpu/src/background_gpu.rs:286-324`）。但新一次挂载的显存账从 0 起，不含上一次泄漏的那片，所以同一进程里「卸载 → 再挂载」之后，同一张卡上两次挂载合占的显存可以超过用户给的上限（第 3 条「任何时刻都不许比它多」）。条款没写泄漏之后的账。读码推，没造样本。
- 泄漏的数量运行时看不到：`let _gpus_leaked = gpus_dropped_leaking_those_still_running(…)`（`crates/singlefs-background-gpu/src/background_gpu.rs:1596`）把泄漏了几张丢掉了，只有 lib 用例读这个返回值（五条硬要求第 4 条「可观测」）。

## T5　五条硬要求：逐条分支「强制进入的开关」与「可观测」

五条硬要求原文在 `.claude/rules/fs-design.md` 第 118–128 行（第 2 条「每条分支必须能被测试强制进入」，第 4 条「分支必须可观测」，第 5 条「分支变量不许在操作中途改变」）。「可观测」一列指运行时（产品代码路径）能读到的量，不是测试里另算的。

| # | 分支 | 强制进入的开关（文件:行） | 可观测（从哪个量看出） |
|---|---|---|---|
| 1 | cpu / gpu | 挂载给的设置 `BackgroundComputeSettings.background_compute`（`crates/singlefs-core/src/background_compute.rs:486`），挂载入口 `crates/singlefs-core/src/mount.rs:8977`，切换入口 `crates/singlefs-core/src/mounted_session.rs:2232` | 每批的 `BackgroundComputeOfOneBatch.requested_backend` 与 `path_taken`（`crates/singlefs-core/src/background_compute.rs:985-992`），经 `CompactionRun.background_compute_of_each_batch`（`crates/singlefs-core/src/mounted_session.rs:3188`）交出 |
| 2 | 一批算着出错、任务停下 | 只供测试的 `report_a_compute_error_in_the_batch_under_way_for_tests`（`crates/singlefs-core/src/background_compute.rs:1163`）经钩子（`crates/singlefs-core/src/mounted_session.rs:2193`）；或递一个交回 `Err` 的 GPU 实现 | `stopped_after_a_compute_error()`（`crates/singlefs-core/src/background_compute.rs:1149`）、批记账里的 `compute_error_that_abandoned_the_batch`（:991）、一趟的停因 `BatchAbandonedAfterABackgroundComputeError`（`crates/singlefs-core/src/mounted_session.rs:3707`） |
| 3 | 停着而超过阈值 → CPU 补跑 | 没有专用开关，靠输入进：挂载给阈值 0%（任何已分配字节都越线，`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:334` 那条用例）或在分配器里把某块盘标成已分配（同文件 :828 的辅助函数） | 批记账 `ReasonTheBatchRanOnTheCpu::CompensatingWhileStoppedAfterAComputeError(读数)`（`crates/singlefs-core/src/background_compute.rs:957`）、停着任务上的 `batches_compensated_on_the_cpu`（:1077）、`MountedSession::gc_cpu_compensation_reading()`（`crates/singlefs-core/src/mounted_session.rs:2217`） |
| 4 | 停着而不超阈值 → 照旧停着 | 阈值 100%（永远越不过）靠输入进；「补跑途中降到不超过」只有只供测试的开关 `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`（`crates/singlefs-core/src/background_compute.rs:1142`）能进 | 一趟的停因 `BackgroundTaskStoppedAfterAnEarlierComputeError`（`crates/singlefs-core/src/mounted_session.rs:3214`）、后台入口的 `NotStarted(BackgroundTaskStoppedAfterAComputeError)`（:2389-2391） |
| 5 | 照设置走（没停着） | 默认 | 同 1：`path_taken` 是 `Cpu(CpuRequestedByTheSettings)` 或 `Gpu(…)` |
| 6 | 切片 / 不切 | 靠输入进：显存上限（挂载设置，`crates/singlefs-core/src/background_compute.rs:490`）调小；没有绕过判据直接走某一支的开关 | 没有。批记账 `GpuWorkOfTheBatch` 只数条目（`crates/singlefs-core/src/background_compute.rs:422-429`），不数提交了几个片、切没切；`BackgroundGpu::participating_gpus()` 只报上限与代价（`crates/singlefs-background-gpu/src/background_gpu.rs:1113`），不报片数。运行时看不出这一批是切着交的还是整批交的 |
| 7 | 重交第 1–3 次 | 产品路径上没有强制超时的开关；等的上限调到极小（挂载设置，`crates/singlefs-core/src/background_compute.rs:494`）靠输入进，不确定性大。lib 层 `outcome_resubmitted_after_timeouts` 吃闭包，用例换假提交与假钟（`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs:142-146`、用例替身 :344-363） | 重交后成功：没有（成功就是成功，批记账里没有「重交过几次」）。重交用尽：`GpuUnusableAfterResubmissions { resubmissions, … }`（`crates/singlefs-core/src/background_compute.rs:190-195`） |
| 8 | 两份核相同 / 不同 / 交 CPU 判 | 产品核没有开关能让某一份核故意算错；GPU 包对拍用例用 CPU 替身核并往里注入分歧（`crates/singlefs-background-gpu/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:537` 的 `inject_disagreements`）。真核只靠变异行罩 | 每批每段的三个计数 `GpuEntryCounts`（`crates/singlefs-core/src/background_compute.rs:399-403`）经 `GpuWorkOfTheBatch` 交出 |
| 9 | 参与的卡 | 靠设置里的卡列表进 | `ParticipatingGpuReport`（`crates/singlefs-background-gpu/src/background_gpu.rs:224-229`）与 `gpus_left_out_under_their_memory_limit()`（:1104）；这两个读的是包，不在批记账里。批记账里没有「这一批用了哪几张卡」 |

分支变量一批之内会不会变：

- 后台计算设置：`end_the_batch` 断言一批结束时生效的设置与开头读的相同（`crates/singlefs-core/src/background_compute.rs:1383-1386`），换上只在 `begin_a_batch`（:1333）。一批里调的钩子（`crates/singlefs-core/src/mounted_session.rs:3657`）只能请求，请求只记成待生效（:1306）。守住了。
- 补跑阈值与读数：读数在 `begin_a_batch` 里定一次、带进批记账（`crates/singlefs-core/src/background_compute.rs:1339-1361`），一批里不再读。守住了。
- 交给 GPU 包的设置：包里的参与的卡、显存上限、等的上限只在两处被改：`of_a_mount`（:1125）与 `begin_a_batch`（:1335），都在批开头；一批里 `lock()` 的守卫一直持到整批算完（`crates/singlefs-core/src/mounted_session.rs:3676`）。有一个窗：`begin_a_batch` 交完设置就放锁（:683 的临时借出），到 :3676 重新借之间钩子在跑；如果调用方自己留着把手的另一份克隆（`of_a_shared_implementation`，`crates/singlefs-core/src/background_compute.rs:366`）并在这个窗里调 `use_the_settings_from_the_next_batch`，这一批用的设置就不是批开头定的。产品路径不会这样做，只有持把手的别的线程能做到。读码推。
- 分配器的占用在批里当然在变（发布在改它）；补跑那一判只在批开头读一次，不受影响。

`BACKGROUND_TASKS_WITH_A_GPU_PATH` 写死长度 1（`crates/singlefs-core/src/background_compute.rs:1035`）时，加一个带 GPU 路径的任务而没改它的世界里哪条测试会红：我没找到会红的。依据：这个常量是私有的，全仓只有 :1035 与 :1046 两处引用；所有用例只枚举 `CompactionRelocation` 一个成员（`grep -rn "BackgroundTask::" crates/singlefs-harness crates/singlefs-background-gpu` 全是这一个成员）；停着那一支的用例看的是停着表里恰好有这一个任务（例如 `crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:274`），新任务不在表里它们照样绿。能让它红的会是「对每个成员断言挂载时一张卡都用不上就停着」的用例，现在没有；`BackgroundTask` 是只有一个成员的封闭枚举，没法在用例里枚举成员。

## T6　测试与变异判的是不是那件事

### T6.0　背景材料里的行号对不上

背景材料说「第 51 批改过输入的乙段切片三行 6131、6137、6138」。冻结副本与此刻主工作区的 `crates/mutations.tsv` 前 6238 行相同，这三行是 C773 的三行（第 6131 行名字以「C773 根槽读者只核头一个扇区」起头，第 6137、6138 行是「C773 写者」两行），不是乙段切片。乙段单元切片的三行在第 6121、6122、6123 行（名字以「七·3 乙段单元切片」起头，点名 `a_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications`）。第 51 批调查员交回里写的 6137、6138、6139 是那一批合入之前的编号（`~/.local/share/singlefs/work/m4-gpugc-slice-mutations-investigator/patch/result.md` 结论 1），合入后整体前移 16 行。下面的 T6.2 按真实行号写。

### T6.1　机械表（跑过）

命令（在冻结副本上读，不改它）：`python3 -I ~/.local/share/singlefs/work/m4-gpugc-code-r1-sonnet/draft/rows.py > draft/rows.tsv`。脚本取 `crates/mutations.tsv` 里名字以「七·3」起头的行加第 6131、6137、6138 行，逐行查锚点在文件里命中几次与行号，输出（原样）：stderr 一行 `257`，`rows.tsv` 257 行；第 5 列（锚点命中次数）257 行全是 1。点名的用例名我另用正则在 `crates/` 全部 `.rs` 里找了 `fn <名>`，257 行都找得到（0 缺）。`sha256`：`rows.py` e5bde41b…、`rows.tsv` 806e0872…（草稿目录里）。

按点名的用例类型分（`rows.tsv` 第 8 列）：GPU 包 `--lib` 112 行、`singlefs-harness` 91 行、真卡用例（特性 `tests-on-a-real-gpu`）29 行、GPU 包的集成测试（CPU 替身）15 行、核心层 `--lib` 10 行。

### T6.2　逐条列：改的哪一行、点名的用例红在哪条断言、为什么那条断言判的是这一处

257 行我没有逐条再跑一遍（真卡 29 行不能跑，其余每条要编一次库，排内存队列）。我在量法拷贝里把下面 7 行按它们点名的参数原样跑了一遍，红的位置是跑出来的原样（输出在 `draft/mut-<行号>.txt`）；命令形态：`python3 -I draft/mut.py <行号>`，脚本把锚点换成替换文、起 `cargo test --offline <该行的参数>`（经 `run-with-memory-cap.sh 16G`），跑完还原。

| 行 | 改的哪一行（文件:行；原 → 改） | 点名的用例 | 红在哪条断言（跑出来的） | 为什么那条断言判的是这一处 |
|---|---|---|---|---|
| 5597 | `crates/singlefs-core/src/background_compute.rs:1334`：`self.in_effect = requested;` → `drop(requested);` | `a_batch_under_gpu_settings_records_the_gpu_request_and_runs_its_segments_on_the_gpu_compute` | `crates/singlefs-harness/tests/background_compute_settings_and_switch.rs:561` 的 `assert_eq!`：左 `(CompactionRelocation, Cpu)`、右 `(CompactionRelocation, Gpu)` | 用例先在 `cpu` 的会话上切到 `gpu`（待生效），再跑一趟；换上这一步被丢了，每批的请求后端还是 `cpu`。断言看的就是批记账里的请求后端，判的正是换上这一处 |
| 5636 | `crates/singlefs-core/src/background_compute.rs:1182`：`if self.requested_from_the_next_batch.is_some() {` → `if false && …` | `a_switch_pending_before_a_compute_error_lifts_the_stop_at_the_moment_of_the_stop` | `crates/singlefs-harness/tests/compaction_off_the_calling_thread_and_admission.rs:1131`：一趟的停因应是解除、实得 `BatchAbandonedAfterABackgroundComputeError { task_after_the_error: LeftStopped }` | 断言看的是出错那一刻 `stop_the_task_after_a_compute_error` 的返回值成员（`StopLiftedByTheSwitchRequestedBeforeTheError` 还是 `LeftStopped`），判的就是第 4 条「出错之前已经收到、还没生效的切换请求，在停下那一刻也算数」 |
| 6056 | `crates/singlefs-core/src/background_compute.rs:736`：`> WHOLE_CAPACITY_IN_PERCENT` → `>=` | `a_gc_cpu_compensation_threshold_above_one_hundred_percent_is_refused_at_the_mount_and_at_a_switch` | `crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:695`（`.expect("100% 照收")`）：实得 `GcCpuCompensationThresholdAboveTheWholeCapacity { … 100 }` | 用例同时给 101（应拒）与 100（应收），100 那一边正好卡住 `>` 与 `>=` 的差；条款「0 到 100」含两端 |
| 6101 | `crates/singlefs-core/src/background_compute.rs:566`：`if !device.is_fuller_than(&fullest)` → `if device.is_fuller_than(&fullest)`（最满挑成最空） | `a_compaction_stopped_after_a_gpu_failure_runs_on_the_cpu_when_one_device_is_above_the_threshold_though_the_pool_as_a_whole_is_not` | `crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:909` 的 `assert_eq!`：读数里最满的盘实得 `DeviceIdentity(1)`（2 572 288 字节），应是 `DeviceIdentity(0)`（104 202 240 字节） | 两块盘占用悬殊，读数里的盘号与字节数是直接比的，判的是挑最满那一步 |
| 6104 | `crates/singlefs-core/src/mounted_session.rs:3207`：`if compaction_stays_stopped_pausing_the_live_intent(…) {` → `if false && …` | `a_run_left_stopped_at_its_start_reports_the_stop_before_judging_the_batch_size_or_continuing_the_deletion_queue` | `crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1153`：停因实得 `BatchSizeNotDecidedByTheClauses`，应是 `BackgroundTaskStoppedAfterAnEarlierComputeError` | 红在「停着先于批大小那一判」，不是在「先于删除续做」：这条用例的名字写了后一半，用例里没有造非空的删除队列（`grep -n -i "unlink\|delete\|deletion\|queue" crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs` 整个文件只命中用例名那一行 :1112，没有一处造出非空的删除队列）。删除续做那一半没有断言罩着 |
| 6063 | `crates/singlefs-core/src/mounted_session.rs:3207`：`if compaction_stays…` → `if !compaction_stays…`（名字：「一趟开头停着那一判反过来（停着且超过阈值时一个写都不发）」） | `a_compaction_stopped_after_a_gpu_failure_moves_its_intent_on_the_cpu_while_the_pool_is_above_the_threshold` | `crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:240`：这是用例的装置函数里的前置断言（「在 GPU 上只发了签发那一次，意图在强制选的那一段上」），实得 `Stopped(BackgroundTaskStoppedAfterAnEarlierComputeError)` | 红了，但红在没停着的装置阶段（反过来之后没停着的会话被拦住了），没走到用例自己关于「停着且超过阈值」的断言。这一行的名字写的是「停着且超过阈值」，实际判的是「没停着时放行」。一个只改「停着且超过阈值」（例如把 `OnTheCpuToCompensateWhileStopped` 也当停着）的写法这一行盯不盯得住，我没跑（待核） |
| 6181 | `crates/singlefs-core/src/background_compute.rs:1336`：停着表在切换生效那一批开头不记（`let _stops_dropped_at_the_switch = …`） | `a_switch_to_settings_whose_participating_gpus_are_all_left_out_stops_the_compaction_at_the_next_batch_as_at_the_mount` | `crates/singlefs-background-gpu/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs:1312`：消息「切换之前那一批走 GPU；切换生效的那一批开头停着、不开」，实得那一批照开、算着在乙段交回 `EveryParticipatingGpuLeftOutUnderItsMemoryLimit` 才停 | 判的正是「切换生效那一批开头当场停着」与「挂载那一刻一样」 |

模块级与批注：6101 这一行罩的是「最满的盘比最空的」，没罩「取最满」的别的坏写法，见 T6.3。

### T6.3　换一种同样坏的写法，红不红（跑过）

- 自拟变异 A：`crates/singlefs-core/src/background_compute.rs:566` 的 `Some(fullest) if !device.is_fuller_than(&fullest) => Some(fullest),` 改成 `Some(fullest) => Some(fullest),`（永远取分配器里的第一块盘，不比占用）。命令：`python3 -I draft/mut2.py fullest-always-first crates/singlefs-core/src/background_compute.rs '<原>' '<改>' '-p singlefs-harness --test compaction_on_the_cpu_while_stopped_after_a_gpu_failure'`，输出（`draft/mut2-fullest-always-first.txt`）：`test result: ok. 9 passed; 0 failed`，`RC 0`。该测试目标里 9 条全绿。原因：全仓只有这一个测试文件读「最满的盘」（`grep -rln "of_the_fullest_device_of_the_allocator\|gc_cpu_compensation_reading\|CompensatingWhileStopped\|fullest_device" crates` 只命中 `background_compute.rs`、`mounted_session.rs` 与这一个测试文件），文件里没有 `DeviceIdentity(1)`（`grep -n "DeviceIdentity(1)"` 只命中一处注释行），所有用例里最满的都是第一块盘，或两块盘同样满。核心层 `background_compute.rs` 里没有 `#[cfg(test)]` 模块，`of_the_fullest_device_of_the_allocator` 没有单测。这是 `.claude/rules/mutation-sampling.md` 第三类（取样点不敏感）：6100、6101 两行的取样点上「取第一块盘」与「取最满」同值。需要补一个最满的盘是第二块盘的取样点。
- 自拟变异 B（删除续做那一支整个去掉）的结果见 T4.3（`mounted_session.rs:3238` 的 `if !queue_is_empty {` → `if false && …`）。

### T6.4　乙段单元切片三行（真卡才罩得住）

| 行 | 改的哪一行 | 红在哪条断言（读码推，真卡我没跑） | 为什么判这一处 |
|---|---|---|---|
| 6121 | `crates/singlefs-background-gpu/src/region_record_kernel_walking_the_mapping.wgsl:216`：`register_by_table_after_zero_bytes(register_over_this_piece, unit.bytes_after_this_piece) ^ unit.register_of_the_bytes_after;` 去掉 `^ unit.register_of_the_bytes_after` | `a_memory_limit_smaller_than_a_unit_cuts_each_unit_into_pieces_with_the_same_classifications` | `crates/singlefs-background-gpu/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:773` 的 `assert_eq!(walking, reference, …)`（逐条走映射那一份与 CPU 参照逐条相同） | 头一片之后的寄存器不是 0 时，不异或它，载荷 CRC 算错，可搬的码 1 单元被判成载荷校验和不对，分类与参照不同 |
| 6122 | `crates/singlefs-background-gpu/src/region_record_kernel_walking_the_mapping.wgsl:427`：头一片之后的一片从 `descriptor.register_of_the_walk`（带进来的寄存器）起 → 从 `0u` 起 | 同上 `assert_eq!(walking, …)` | 头一片之后的第二片起必须接着前一片的寄存器；每片都从 0 起，第二片之后寄存器错 |
| 6123 | `crates/singlefs-background-gpu/src/region_record_kernel_searching_sorted_indexes.wgsl:226`：`register_after_zero_bytes(register_through_this_piece, unit.length_after_this_piece) ^ …` → `register_through_this_piece ^ …`（不喂之后那么多个 0 字节） | 同文件 :777 的 `assert_eq!(searching, reference, …)` | 同上，改的是按索引查的那一份 |

红不红取决于输入：用例里先有一句断言「有一个可搬的单元在头一片之后还有不是 0 的字节」（`crates/singlefs-background-gpu/tests/region_record_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:761`），码 1 单元载荷用两万字节各不相同的那一种（`PayloadOfTheDataUnits::RunningPastTheHeadPiece`，:117）。第 51 批前的输入（载荷 100 字节，`WithinTheHeadPiece`，:116）下，这三行里 6121、6122 与原式逐位相同（头一片之后全是 0 字节、寄存器恒 0），所以当时不红：这是第三类。现在别的五条真卡用例仍用 `WithinTheHeadPiece`（:515、:543、:571、:627、:660），但它们不切单元（上限不设或只切映射），不受这一类影响。没有 GPU 的机器上这条用例打 `NOT_RUN_NO_GPU_ADAPTER` 后返回成功（`crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:36-39` 的同款装置函数），这三行在那种机器上不会红。

### T6.5　替身罩不到、只有真卡用例罩着的行为（单列）

`rows.tsv` 里点名真卡用例的 29 行（命令里带 `--features tests-on-a-real-gpu`）：

- 十九行改的是 WGSL 核本身：校验和两份核 5670–5673；乙段两份核 5685–5689；丙段两份核 5799–5804、5991；乙段切片 6121–6123。CPU 替身（`crates/singlefs-background-gpu/tests/background_gpu_compute_agrees_with_the_cpu_backed_test_double.rs`）和 lib 的 CPU 模拟不执行 WGSL，罩不到。
- 十行改的是 GPU 包外壳（`BackgroundGpu` 本身，替身整个换掉了它）：5805、5812（丙段给核绑缓冲）、5813（丙段分段接回）、5869、5870（换设置时参与的卡、卡清单）、6016–6018（三段按剩下的显存切片）、6019（开卡时判最小一片）、6108（一张都不剩时报的被晾下的卡）。
- 我没有找到变异行的外壳行为（读码推）：外壳里三处套重交包装（`crates/singlefs-background-gpu/src/background_gpu.rs:1302`、:1414、:1535）、三处交活前的先等（:1273、:1386、:1509）、结账（`settle_the_chunk_after_its_submission`，:848、:970、:1081）、`Drop` 里判泄漏的闭包（:1597-1602）。`rows.tsv` 里 `background_gpu.rs` 的 11 行（5668、5669、5805、5812、5869、5870、6016–6019、6108）锚点没有一个落在这些行上。重交包装自己、账的函数、`gpus_dropped_leaking_those_still_running` 各有 lib 变异行（5996–5999、5745–5751、6014），但「外壳三处都套了包装、都在交活前先等、`Drop` 传的判据是账上的那一位」这件接线没有行罩；真卡用例里也没有一条造超时（没有强制超时的开关，见 T5.7）。把某一段外壳的 `outcome_resubmitted_after_timeouts(…)` 换成直接调用，我预计不会有测试红。这一条我没跑（排内存队列），按「没核到」记。
- 时间相关的真卡行为只有真卡罩着：`wait_for_the_wgpu_future`（5669 是 lib）与 `device.poll(…Wait{timeout})` 的超时分支（`crates/singlefs-background-gpu/src/background_gpu.rs:533-538`）没有行；5991（核不终止要靠等的上限截出错）是真卡行。

### T6.6　别的切片用例有没有落进取样点不敏感

- 甲段切片：真卡用例的单元字节是按长度和起点余数现算的伪随机字节、每个单元不同（`crates/singlefs-background-gpu/tests/checksum_kernels_on_a_real_gpu_agree_with_the_cpu_reference.rs:50`）；lib 里合成式有两条（`crates/singlefs-background-gpu/src/checksum_kernels.rs:605`、:652）。不落进这一类。
- 乙段映射切片：真卡用例用 100 字节载荷是合适的（只切映射、不切单元）。但「key 在别的片里改判」那条合并规则（`crates/singlefs-background-gpu/src/region_record_kernels.rs:834`、:857）要求一条已死的记录而它的 key 在另一片映射里；lib 有变异行 6006、6007 罩（点名 `a_mapping_cut_into_slices_under_a_small_memory_limit_classifies_every_record_as_the_whole_mapping`，lib 的 CPU 模拟）。真卡那一边是否真造出了「key 在另一片」，取决于卡上切片点落在哪，我没法在没卡的机器上核（核不动）。
- 丙段切片：6004（各片相加）罩 lib；真卡 6018（外壳是否切）罩外壳。没看到零载荷类的问题。
- 补跑按盘：见 T6.3，最满的盘永远是第一块，落进这一类。

## T4　CPU 一侧：补跑、挪线程与两处快写法

### T4.1　补跑读数取的是哪一刻的数、含什么

- 哪一刻：此刻分配器内存里的计数，不是上一次发布的版本里的数。一趟开头的那一判（`crates/singlefs-core/src/mounted_session.rs:3104`，后台入口起跑前那一判 :2388 走同一个函数）与每一批开头（`crates/singlefs-core/src/mounted_session.rs:3646`，进 `begin_a_batch` 之前现取）各读一次，读的都是 `of_the_fullest_device_of_the_allocator(allocator)`（`crates/singlefs-core/src/background_compute.rs:546-571`），逐盘取 `DeviceFreeMap::allocated_slots()` 与 `unit_area_slots()` 乘 16 KiB；两个计数由分配器在每次标已分配（`crates/singlefs-core/src/allocator.rs:946-948`）和每次回收放回（:848-850）时增量维护，不遍历槽。一趟开头那一读与第一批开头那一读之间可以隔着删除续做的发布（见 T4.3），所以这两读可以不同；批记账里带的是第二次（批开头）那一读。
- 含什么：「已分配」是占着的槽数 = 仍分配的 + 已释放而还在 defer 窗口里的（`crates/singlefs-core/src/allocator.rs:325-327`）；「容量」是单元区槽数（:614-615），系统区在外。比较是 `已分配 × 100 > 容量 × 阈值` 在 u128 里做（`crates/singlefs-core/src/background_compute.rs:590-594`），严格大于，不取整；容量为 0 的盘不参加挑（:562），全是 0 取头一块。条款只说「「已分配 ÷ 容量」」，没说是否含保留、整理暂存：代码里没有单独的「保留」与「整理暂存」计数，整理搬迁目的地是发布时才在分配器里标已分配的，批开头那一读不含这一批将要占的目的地。影子账隔离的槽、被隔离标记的槽算不算已分配，我没核（`allocator.rs` 的隔离路径没细读），记「没核到」。
- 取哪一块：最满的一块（逐盘交叉相乘比占比，一样满取前面那块），用户 2026-10-28 弹窗定「任一块盘超过就补跑」（`crates/singlefs-core/src/background_compute.rs:538-541` 的文档）；第 4 条原文「任一块盘「已分配 ÷ 容量」超过挂载参数 `background_gc_cpu_compensation_threshold`」（`.claude/kb/decisions/26-后台整理与放置回收.md:248`）：最满那块超过 ⇔ 有一块超过，等价。主 agent 记下的 ⑥ 的判：对得上。注意 T6.3：测试里最满的永远是第一块盘。

### T4.2　只供测试的开关在发布构建里够不够得着

- 够得着，三个：
  - `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests`（`crates/singlefs-core/src/background_compute.rs:1142`，`pub`）与 `report_a_compute_error_in_the_batch_under_way_for_tests`（:1163，`pub`）：要一个 `&mut BackgroundComputeOfThisMount`。`MountedSession::background_compute()` 只交 `&`（`crates/singlefs-core/src/mounted_session.rs:2209`）；唯一把 `&mut` 交到调用方手里的是 `run_compaction_calling_a_test_only_hook_in_the_middle_of_each_batch`（`crates/singlefs-core/src/mounted_session.rs:2193`，`pub`，无 `cfg`、无特性），它把 `&mut` 递给调用方给的闭包。所以发布构建里任何依赖本库的代码都能经这个钩子调到这两个开关。
  - `crc32_castagnoli_on_route`（`crates/singlefs-core/src/checksum.rs:130`，`pub`）：只选路，两条路对同一段字节交回同一个数，不改语义。
- 对照：同一个包里别的只供测试的开关是特性门控的，默认关：`lookup-work-counters`（`crates/singlefs-core/Cargo.toml:34`）与 `test-only-publish-thresholds`（:37，用在 `crates/singlefs-core/src/system_configuration.rs:471`）。七·3 这一组没有门控，与前例不一致。
- 后果：第一个开关一旦调用，这次挂载后面每一批、停着的任务都当占用不超过阈值判，永远不补跑，且切换不重置它（`gc_cpu_compensation_judgement` 只在 `of_a_mount` 赋初值 :1135 与这个开关里赋值 :1143）；没停着的任务不受影响（`crates/singlefs-core/src/background_compute.rs:1227-1229` 先判没停着就走设置）。第二个开关让下一批注入一个出错、任务停下。后果限于调用方自己的会话，没有跨边界；这是 C921 的射程，我判它「够得着、后果小」：库的使用方本来就能自己不调补跑。主 agent 记下的 ① 的判：成立。

### T4.3　第 3206 行与第 3230 行两支的先后，停着时第二支进不进得去

- 先后（`crates/singlefs-core/src/mounted_session.rs`）：:3207 停着的那一判（返回即「一个写都不发」）→ :3218 批大小那一判 → :3230 起删除续做（`if !queue_is_empty {`，:3238）。停着而不超阈值时 :3207 就返回，第二支进不去，是写法决定的。
- 停着而超过阈值（补跑）时 :3207 放行，第二支进得去（只要删除队列非空）。这时删除续做发的写不算「一个写都不发」的违约：该判的语义就是「超过阈值就照常推、每一批补跑」（:3205-3206 注释）。
- 主 agent 记下的 ② 要的是「造不出进得去的输入，是不是死代码」。判：不是死代码，输入造得出，两条独立的路：
  1. 挂载交回时队列非空：挂载的删除续做推满抬 F 仍放不下时，挂载照样做成，`current` 是最后落盘的那一版，队列留着（`crates/singlefs-core/src/mount.rs:7724` 的文档，`StillShortAfterTheFloorRaises`，:7833 一带；这时挂载那一次的整理续做 :7659 返回 `None` 不跑）。会话拿到这样的 `current` 后第一次调整理入口就走 :3238。
  2. 会话里一次用户改动的走空失败：`publish_the_change_raising_the_floor_for_space` 在发成改动之后走空，走空半路被拒（不是空间不够那种）时 `*current` 已是走到的那一版再 `walk?` 返回错误（`crates/singlefs-core/src/mounted_session.rs:2616` 与 :2621），队列留着。主 agent 说的「会话里删除一次走完」是成功路径。
  3. 再叠停着且超阈值：路 1 的挂载里同时给 `gpu` 设置和一个连最小一片都装不下的上限（参与的卡一张都不剩，挂载不拒，`of_a_mount` 当场记停着，`crates/singlefs-core/src/background_compute.rs:1124-1129`），阈值给 0%：挂载交回时就是「停着 + 超阈值 + 队列非空」，下一次 `run_compaction` 或后台入口进 :3207（放行）→ :3238（走空）→ 批开头补跑。读码推；我没造这个世界跑（需要一个让抬 F 推满仍放不下的池子，harness 里有这类池子的用例：`crates/singlefs-harness/tests/admission_raises_the_floor_before_refusing.rs`、`a_floor_raise_refused_for_space_counts_as_short_of_space.rs`，没把它们和 `gpu` 设置接起来）。
- 这一支有没有测试罩着：`crates/mutations.tsv` 里没有锚点落在 `if !queue_is_empty {` 上的行（`grep -c "if !queue_is_empty {" crates/mutations.tsv` 输出 0）。名字写了「继续删除队列」的用例（`crates/singlefs-harness/tests/compaction_on_the_cpu_while_stopped_after_a_gpu_failure.rs:1112`）没有造队列（T6.2 第 6104 行）。自拟变异 B（把这一支整个去掉，`if !queue_is_empty {` → `if false && !queue_is_empty {`），没跑出结果：两次起跑（先五个整理相关的测试目标，再缩成 `compaction_and_deletion_interleaved_across_a_rollback`、`compaction_runs_stop_and_discard_after_the_publish` 两个）都在 `run-with-memory-cap.sh` 的内存队列里排满了我给的 560 秒，被 `timeout` 杀，退 124，一行测试都没跑（`draft/mut2-drain-branch-removed.txt`、`draft/mut2-drain-branch-removed-2.txt` 第 `RC` 行），不记任何结果。静态上我只核到：同时调整理入口又碰 unlink 的测试文件只有 `compaction_runs_stop_and_discard_after_the_publish.rs`（:728 的 `publish_unlink_intent`，该用例只发 unlink 意图、不跑整理）与 `batching_across_requests_and_barriers.rs`；有没有用例真让这一支执行，记「没核到」。
- 另一缝（读码推）：切换到「一张卡都用不上」的设置时，停着要到下一批的 `begin_a_batch` 才记（`crates/singlefs-core/src/background_compute.rs:1336`），而 :3207 与 :2388 这两个闸看到的还是「没停着」，所以那一趟的删除续做先发写、`DueAfterAUserChange` 在 :2395 被取走不放回。挂载时同样的设置在 `of_a_mount` 就记了，不会这样。见 T1.2。

### T4.4　挪线程三样保证各由哪一行守着（含别的会话拆成「判起跑」「跑」之后）

| 保证 | 由哪一行守着 | 拆成两段之后 |
|---|---|---|
| 一趟发布预算有上限 | 用户改动发成时清零：`crates/singlefs-core/src/mounted_session.rs:2127`；整理每发成一次加一：:3353；一趟的预算现算：:3262-3273（`budget_rule`），循环头判：:3279；后台入口起跑前也判扣完不起跑：:2430-2438 | 判起跑把「按哪条规则算预算」（`CompactionRunBudget::LeftOfTheBudgetGivenByTheLastUserChange`）交给跑，不交数（:2442），跑里（:3267-3272）按当时的计数现算；攒批会话在两段之间排空待发（一次用户改动，会把计数清零），跑看到的是清零后的计数，不是判起跑时的数。成立 |
| 签下一条意图那一判跨趟留着 | 完成时记：`crates/singlefs-core/src/mounted_session.rs:4228`；签下一条前读：:3934；用户改动发成时清：:2132 | 判起跑与跑之间的排空待发是一次用户改动发成，会清掉它（:2132）。这是用户 2026-10-26 定「每次用户改动发成就清掉」（`records/2026-10-19-里程碑四GPU垃圾回收开工.md` 调查员交回那一行，注释在 :2128-2131）；kb 条款只写「跨趟留着」（`.claude/kb/decisions/26-后台整理与放置回收.md:250`），没写这条清除。条款没写而代码加了，来源是 records，不在 kb。成立，口径比条款窄一点 |
| 整理批不插进用户改动的抬 F 与再判之间 | 抬 F 与再判在 `publish_the_change_raising_the_floor_for_space` 一次调用里，调用方拿着 `&mut self`（`crates/singlefs-core/src/mounted_session.rs:2114` 调它，注释 :2128-2131）；整理的每个入口都是另外的 `&mut self` 方法，`publish_user_change` 里不调整理（整理 runner 的调用点只有三处：:2293、:2466、`crates/singlefs-core/src/mount.rs:7662`） | 排空待发（`crates/singlefs-core/src/mounted_session.rs:5128` 的 `publish_the_pending_changes`）在判起跑之后、跑之前整个做完（含它自己的抬 F 与再判）再进跑，所以仍不会插进去。成立 |
| 「用户改动返回之前一批整理都不发」 | `publish_user_change` 里没有对整理 runner 的调用（上一行三处调用点），用例 `crates/singlefs-harness/tests/compaction_off_the_calling_thread_and_admission.rs:296`、:325 | 不受拆分影响 |
| 「前台 fsync 在等时不发下一批」 | 每批之前问调用方：`crates/singlefs-core/src/mounted_session.rs:3304-3316`（在循环里，`begin_a_batch` 之前） | 判起跑不看 fsync；排空待发与删除续做（:3238）发生在循环之前，不受 fsync 约束，只有循环里的整理批受。条款 :250 写的是「不发下一批」，没写删除续做与待发要不要让路。成立，口径限于整理批 |

### T4.5　快写法的三样对拍各在哪条用例，强制进入另一条分支的开关在哪

| 对拍 | 用例（文件:行） | 强制进入另一条分支的开关 |
|---|---|---|
| 区域扫描按身份查 与 线性找，逐项相同 | `crates/singlefs-harness/tests/region_scan_lookup_by_identity_and_hardware_crc.rs:160`（`identity_index_lookups_match_the_linear_lookups_on_random_mapping_entries_and_records`）；线性参照是同文件里写的（:58、:75），不是产品代码 | 没有：产品只有按身份查一条路，线性参照只活在测试里。「另一条分支」靠测试自带的参照，不是开关 |
| CRC-32C 硬件指令 与 软件（slicing-by-8） | 同文件 :546（硬件路）与 :577（强制软件路），及 :527（路是否随处理器的指令检测） | `crc32_castagnoli_on_route(route, bytes)`（`crates/singlefs-core/src/checksum.rs:130`），`pub`，在 T4.2 |
| CRC-32C 与 checker 的按位实现逐字节相同 | 同文件 :494 的 `assert_the_route_matches_the_checker_bitwise_crc`，被 :546、:577 调 | 同上 |

此外 `crc32_castagnoli` 在 `crates/singlefs-core/src/checksum.rs:3` 的文件头写「走哪条路运行时看得出」：`castagnoli_route_of_this_processor`（:88）。

## 主 agent 记下的八处，逐处判

| 处 | 判 | 依据（节） | 什么现象会推翻 |
|---|---|---|---|
| ① 只供测试的开关在发布构建里够得着 | 成立：三个都 `pub`、无门控；后果限于调用方自己的会话 | T4.2 | 在 `crates/singlefs-core` 里找到包住 `judge_the_gc_cpu_compensation_as_not_above_the_threshold_for_tests` 或钩子入口的 `cfg`/特性（我 `grep -n "cfg(feature" crates/singlefs-core/src/*.rs` 只命中 `region_scan.rs`、`transaction.rs`、`system_configuration.rs`，不含这几个） |
| ② 第 3230 行起那一支是不是死代码 | 不是死代码。停着而不超阈值时进不去（第 3207 行在前、写法决定）；停着而超阈值、或没停着时进得去，输入有两条独立的路（挂载交回时队列非空；会话里一次用户改动走空半路被拒）。没有测试和变异行罩它 | T4.3 | 证明挂载与会话里的走空失败后队列一定为空：`crates/singlefs-core/src/mount.rs:7724` 与 `crates/singlefs-core/src/mounted_session.rs:2616-2621` 两处的读法被推翻 |
| ③ 停着表写死长度 1 | 成立；加任务时编译器不逼着改它，也没有用例会红 | T1.4、T5 末段 | 找到一条在新增任务后会红的用例或编译错误；补跑那臂 `match task`（`crates/singlefs-core/src/background_compute.rs:1237`）与 `route_of_the_task`（:969）是穷尽匹配，加成员会编译错，但这两处不是停着表 |
| ④ 卸载泄漏的边界是不是只在「还记着一次超时提交」的卡 | 是，且不多不少；但进程内不共享账（再挂不扣泄漏的那片），泄漏数不可观测 | T3.4 | 找到进程级的账或把泄漏数交到调用方手里的代码（我 `grep` 无） |
| ⑤ 重交用同一张卡同一批 | 同一张卡：成立。「批」：实际重交的是片（一个片的提交），不是整批；条款没写批切成几片时重交哪个 | T3.1 | 在 `outcome_resubmitted_after_timeouts` 之外找到整批重做的路径 |
| ⑥ 补跑按盘取最满那块 | 与条款（任一块超过）等价，实现对；但测试里最满的永远是第一块盘，「永远取第一块」的坏写法全绿 | T4.1、T6.3 | 加一条最满的是第二块盘的用例后，自拟变异 A 红 |
| ⑦ 拆成「判起跑」「跑」之后三样保证与两条行为 | 仍成立。预算交规则不交数；上一条完成的意图被排空待发清掉（用户定的，kb 没写）；fsync 只约束整理批，排空待发与删除续做不受它约束 | T4.4 | 找到判起跑与跑之间别处会改 `publishes_since_the_last_user_change` 或绕过 `foreground_fsync()` 的路 |
| ⑧ 第 45、46 批的读法 | 972 取三段最大，是保守读法，条款（`.claude/kb/decisions/26-后台整理与放置回收.md:247`）只说「最小的一片」没定哪一段；只切比头一片长的单元（`crates/singlefs-background-gpu/src/region_record_kernels.rs:614`）是实现设计，条款没写；一样满取排前面的盘（`crates/singlefs-core/src/background_compute.rs:566`）条款没写，只影响读数里报哪块盘。另：972 是与输入无关的下限，一个落点上条目多的映射实际最小片更大，那种批装不下时是整批出错停下，不是「不用这张卡」（T3.3） | T3.3、T4.1 | 用户对「最小的一片」另有口径 |

## 打中的

下面是条款（或验收、或代码自己写的承诺）与代码对不上的地方。都是线索，由主 agent 现查坐实；我的把握在每条末尾标。

1. **验收「卡上空闲显存不到上限时照常跑、只用空闲的」没有实现，也没有用例。**
   条款行：`.claude/kb/decisions/26-后台整理与放置回收.md:247`「卡上此刻空闲的不到这个数也照常跑，只用空闲的那些」；验收行：`.claude/kb/milestone/04-fourth-txn.md:145`「卡上空闲显存不到上限时照常跑、只用空闲的」。
   代码行：GPU 包只在开卡时读 Vulkan 设备本地堆的总量（`crates/singlefs-background-gpu/src/background_gpu.rs:348-367`），账（`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:277-340`）只数自己建的缓冲，按用户给的上限放行。`grep -rn -i "空闲显存\|memory_budget\|free_memory\|只用空闲" crates/singlefs-background-gpu crates/singlefs-core/src/background_compute.rs crates/singlefs-harness/tests` 零命中。
   对不上在哪：卡上真空闲小于上限时，代码照上限建缓冲，建不成按 wgpu 出错走第 4 条停下；条款与验收要的是照常跑。没有任何用例罩这一句。把握：高（读码 + grep 零命中）；真卡行为没跑。
2. **「GPU 模式把连续几批的计算攒起来一次交给 GPU」没有实现。**
   条款行：`.claude/kb/decisions/26-后台整理与放置回收.md:247`「GPU 模式把连续几批的计算攒起来一次交给 GPU，攒多少受它管」。
   代码行：GPU 路径每一批各自交三段（`crates/singlefs-core/src/mounted_session.rs:3676-3692`），一批里的条目分给卡、按显存切成片；没有跨批缓存。`grep -rn 攒` 在 GPU 包与 `background_compute.rs` 零命中。账里「攒批」能对到的只有一个片里的输入。
   对不上在哪：条款有攒批的语句，代码没有；`.claude/kb/mount-parameters.md:24` 也把「攒批」写进上限的口径。设计轮的判决还把跨批攒标成被打穿的（`.claude/kb/decisions/26-后台整理与放置回收.md:261` 的依据行写「跨批攒不成」，说的是「GC 留在调用线程里开 GPU」那种设计），是不是该删这一句，由主 agent 看。把握：中（条款这句本身可能该删）。
3. **切换时「GPU 不在、驱动不对」只读挂载那一刻的卡清单快照。**
   条款行：`.claude/kb/decisions/26-后台整理与放置回收.md:248`「挂载或切换那一刻……GPU 不在、驱动不对……就拒绝挂载（或拒绝切换）」。
   代码行：`crates/singlefs-background-gpu/src/background_gpu.rs:1610-1613`。
   对不上在哪：挂载之后掉了的卡或驱动重置的卡，切换时校验不到。把握：中（读码，没造世界）。
4. **切换到一张卡都用不上的设置，停着晚于闸一步。**
   代码自己的承诺：`crates/singlefs-core/src/mounted_session.rs:3205-3206` 注释「停着……一个写都不发（删除续做也不走）」。
   代码行：切换时停着要到 `begin_a_batch` 才记（`crates/singlefs-core/src/background_compute.rs:1336`），挂载时在 `of_a_mount` 就记（:1124-1129）；闸 :3207、:2388 在这之前看到「没停着」。
   对不上在哪：那一趟先走删除续做并发写，后台入口还会取走「该跑了」不放回（`crates/singlefs-core/src/mounted_session.rs:2395`）。条款（:247）只说「从下一批起照新上限重新量代价、分活」，没说这一趟；所以这是代码内部不对称，不是条款违约。把握：中（读码推，没跑）。
5. **产品包不产的核心层成员、含义两侧不一致的字段。**
   `crates/singlefs-core/src/background_compute.rs:251`（`TablesLargerThanABindingOnTheDevice`）全仓只有定义和一处测试里手造的值，GPU 包不产；`crates/singlefs-core/src/background_compute.rs:243-244` 说 `input_bytes` 是「它自己的输入字节数」，GPU 包填的是最小一片合占的字节数（转述；`crates/singlefs-background-gpu/src/background_gpu_compute.rs:332` 起）。把握：高。
6. **挂载参数文档的代码落点过期，漏了三种整个挂载都拒的情形。**
   `.claude/kb/mount-parameters.md:12` 与 :22-26 的落点（`background_compute.rs:445`–`:482`、`mount.rs:8658`、`mounted_session.rs:2241`）：我逐行看了，`background_compute.rs:450/452/454/458/477/482` 现在是别的内容，`mount.rs:8658`、`mounted_session.rs:2241` 也不是入口（入口是 `crates/singlefs-core/src/mount.rs:8977`、`crates/singlefs-core/src/mounted_session.rs:2232` 与 :2247）。同一张表没写独显缺 PCI 总线地址、读不到设备本地堆、两张报同一个总线地址这三种整个挂载拒的情形（`crates/singlefs-background-gpu/src/adapter_inventory.rs:105`、:112、:120），也没写 `background_gpu_devices` 的卡标识是 PCI 总线地址串（`crates/singlefs-background-gpu/src/adapter_inventory.rs:89`）。把握：高。
7. **交活前先等前一次超时提交那一等，一窗即报警，没有 3 次。**
   条款行：`.claude/kb/decisions/26-后台整理与放置回收.md:248`「都超时才报警」。
   代码行：`crates/singlefs-background-gpu/src/background_gpu.rs:1273`（乙 :1386、丙 :1509）在重交包装之外。条款没写这一支，只出在已判用不了的卡被切换点回来的第一批。把握：中。
8. **测试与变异的缺口**（细节在 T6）：补跑「取最满」只测到最满是第一块盘的取样点，自拟变异 A 全绿；第 6104 行的用例名写了「继续删除队列」而没有造队列；第 6063 行红在装置前置断言而不是行名写的那件事；`if !queue_is_empty {` 没有变异行；外壳三处重交包装与 `Drop` 的接线没有变异行。
9. **`hold` 超上限是 `assert!`（panic），不是返回错误。** `crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs:327-338`。panic 之后互斥锁中毒，同一把手上之后每次借出再 panic（`crates/singlefs-core/src/background_compute.rs:374-378`）。条款没写；这是对「账内不超上限」唯一的兜底，它红了会把整个调用线程带走。把握：高（读码）。
10. **背景材料的变异行号对不上**（T6.0）：6131、6137、6138 在冻结副本里是 C773 三行；乙段切片三行是 6121–6123。

## 没核到的

- 真卡与 WGSL：不起 GPU，29 行真卡变异行的「红在哪条断言」、第 6121–6123 行的红位、「4156 字节」那次红的复现，我都只能读码与读补丁推，没跑。
- 257 行变异里我只重跑了 7 行加 2 条自拟变异（T6.2、T6.3、T4.3），其余 250 行只做了机械核（锚点唯一命中、用例名存在、点名用例类型的分类），没有逐条跑「红在哪条断言」。
- 补跑读数里隔离槽（影子账）算不算已分配：没读 `allocator.rs` 的隔离路径。
- 多个会话共用同一个实现把手时的两处缝（T2.4、T5 末段）：读码推，没造两个会话。
- 「停着 + 超阈值 + 队列非空」的完整世界（T4.3 第 3 路）：没造。删除续做那一支（`mounted_session.rs:3238`）有没有用例让它执行：自拟变异 B 两次都没排上内存队列，没跑出结果。
- 乙段映射切片里「key 在另一片」的改判，真卡那边切片点落在哪（T6.6）：没有卡，核不动。
- WGSL 里喂零字节的实现与 Rust 侧合成公式是否一致：我只核了 Rust 侧公式（T2.2），没读 WGSL 的喂零字节段。
- `Vulkan 报独显为集显` 的样本（T1.3）：没有样本。
- `cite-check.py` 把裸行号只核「行号在不在文件里」，不比内容；本报告里的裸行号我另用脚本把每处目标行抄出来人工对过一遍（见末尾「复核」）。
- 补丁块（`_m4-gpugc-code-r1-diff.md`）我没有逐块读；读的是冻结副本的现状，补丁块只读了 `m4-gpugc-gpu-reds-investigator` 与 `m4-gpugc-adaptive-split-writer` 两块（T3.2）与第 51 批的 `m4-gpugc-slice-mutations-investigator` 的 result.md。

## 判定一览

判定三种：一致 / 冲突 / 规则没说（条款没写，代码取了一种读法）。

| 格 | 判定 | 一句话 |
|---|---|---|
| T1 挂载与切换每一支拒 | 一致（7 支）、规则没说而代码拒（空列表、同卡点两次、等的上限为 0、独显缺地址/缺堆/总线地址重复）、冲突（切换时「GPU 不在、驱动不对」只读挂载快照，放过） | 见 T1.1 的 13 行与 A、B 两条 |
| T1 切换在哪行取生效 | 一致 | `crates/singlefs-core/src/background_compute.rs:1333` 在 `begin_a_batch`，批开头；一批之内不换有 :1383 的断言 |
| T1 停着时切换当场解除 | 一致，另有一处代码内部不对称 | 请求时清停着（:1307）；切换到用不上的设置，停着晚到下一批（T1.2、打中 4） |
| T1 卡字段、后端 | 一致（比条款严） | 只认 `DiscreteGpu` 与 Vulkan |
| T1 补跑只管 GC | 一致 | 补跑那臂是穷尽匹配；停着表无此守护（③） |
| T2 trait 三段对照 | 一致，另有一个核心层成员产品包不产、一个字段含义两侧不同 | 见 T2.1 两张表 |
| T2 切片合并不变量 | 一致（读码 + lib 的 CPU 模拟用例；真核没跑） | 甲、乙、丙都先合成每条再比对 |
| T2 两份核比对与第 8 条三个计数 | 一致；「连续 2 轮」的轮条款没定义 | 计数在 GPU 包实现里、重挂清零；运行时的三个数是核心层自己数的 |
| T2 `failure_under_the_settings_handed_in` 同一个状态 | 一致 | 单会话内同一函数同一状态；共用把手两会话有缝（读码推） |
| T3 重交 3 次 | 一致；条款没写：每窗各开、片级重交、交活前先等一窗即报警 | 一个片最坏 7 个窗 |
| T3 显存账 | 冲突：「攒批」跨批没有、「空闲显存不到上限照常跑」没有 | 缓冲、暂存区都在账里；账只数应用层缓冲 |
| T3 4156 那次红 | 一致 | 修了产品（先拒、后改切片）也改了用例输入，没动账 |
| T3 卸载泄漏 | 一致；条款没写：进程内账不共享、泄漏数运行时看不到 | 边界 = 账上记着没做完提交的卡 |
| T4 补跑读数 | 一致 | 此刻分配器的增量计数，含 defer 窗口里已释放的 |
| T4 只供测试的开关 | 规则没说（成立：发布构建里够得着） | 与同包特性门控的前例不一致 |
| T4 第 3206/3230 行两支 | 一致；条款没写 | 停着且不超阈值时第二支进不去；其余进得去；没有变异行，有没有用例让它执行没核到 |
| T4 挪线程三样保证 | 一致；一处条款没写（用户改动清上一条完成的意图） | 拆成两段后仍成立 |
| T4 三样对拍 | 一致 | 三条用例在同一个文件；强制开关只有 CRC 一条 |
| T5 五条硬要求 | 部分冲突 | 切片/不切、重交成功、卡的用法运行时看不出；停着表无测试会红 |
| T6 变异行 | 一致（抽样 7 行都红，其中第 6063 行红在装置前置断言）；冲突（取样点不敏感一处、行名与断言不符两处、接线无行一类） | 自拟变异 A 全绿；第 6104、6063 行；外壳重交接线无行 |
| T6 背景材料行号 | 冲突 | 6131、6137、6138 是 C773；切片三行是 6121–6123 |

## 没做什么

- 没判攻方腿的格，没替主 agent 采纳任何一条或出判决；「打中的」是线索，由主 agent 现查。
- 没起 GPU，没跑特性 `tests-on-a-real-gpu` 的任何用例，没起 `.claude/gate.d/` 下任何阶段，没跑 checker 档测试。
- 没读禁读清单里的任何文件（`research/prompts/m4-gpugc-code-r1-opus-output.md`、`research/prompts/m4-gpugc-code-r1-opus-model/`、`~/.local/share/singlefs/work/m4-gpugc-code-r1-opus/`）；我 `ls` 过 `~/.local/share/singlefs/work/` 的目录名（看到那个目录存在），没进去。
- 没改冻结副本；量法拷贝在 `~/.local/share/singlefs/work/m4-gpugc-code-r1-sonnet/repo`（交回前已删）。
- 没写模型目录（没有独立模型，量法是 `draft/rows.py`、`draft/mut.py`、`draft/mut2.py` 三个脚本，sha256 在 T6.1 与下面）。

## 复核

- 裸行号：`cite-check.py` 对裸行号只核「行号在不在文件里」。我另写一段 `python3 -I` 脚本，把报告里每个 `路径:行号` 指到的那一行抄出来（前 75 字），逐条对过一遍我写的话，改了二十来处（多半是我凭印象写的范围或起点）。
- 脚本 sha256：`draft/mut.py` e106813d…、`draft/mut2.py` d6c81e22…、`draft/rows.py` e5bde41b…、`draft/rows.tsv` 806e0872…。

## 清理

交回前删了自己建的量法拷贝 `~/.local/share/singlefs/work/m4-gpugc-code-r1-sonnet/repo`（删前 44M，删前 `diff -rq` 对冻结副本的 `crates/` 无差异，即每条变异都已还原）与编译目录 `~/.local/share/singlefs/work/m4-gpugc-code-r1-sonnet/target`（3.8G）。草稿目录只剩 `draft/`（216K：三个脚本、`rows.tsv`、每次变异跑的原样输出 `mut-*.txt`、`mut2-*.txt`、一次基线编译 `build-1.txt`）。起过的 cargo 全部经 `run-with-memory-cap.sh 16G`，同一时刻一条，无残留进程。
