# m4-gpugc-one-vulkan-instance-code-r1：里程碑四七节 GPU 垃圾回收，一个进程一个 Vulkan 实例，代码轮第一轮正文（2026-11-08）

<!-- doc-lint:not-numbers T1 T2 T3 T4 -->

## 一、题面

产品 GPU 包的真卡测试在第二台（NVIDIA 595.99.02，3 张卡）上开卡被拒。调查（任务 9625、9626、9650，报告 `~/.local/share/singlefs/work/m4-gpugc-real-card-inventory-investigation-r2/`）量到：同一进程里 Vulkan 实例建到大约第 12 个之后，新实例枚举不到独显。第 98 批（补丁目录 `~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-writer-r2/patch/`，实现员报告是同目录的 `result.md`）的改法：实例由调用方持有、显式传下去，不用全局状态与单例（`.claude/singlefs-ai-sop/rules/code-discipline.md` 第 257 行「不用全局可变状态；不用单例」）；另把「点名的卡不在这一次枚举的清单里」从「GPU 不在」分出来，成为新的拒绝成员。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-gpugc-one-vulkan-instance-code-r1-diff.md`（三份产品代码文件的块，原样抄录）；测试代码的块只在补丁目录的 `crates.patch` 里。

## 二、实现今天的样子（主 agent 的观测，2026-11-08）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-one-vulkan-instance-code-r1/repo/`：`git archive` 取自提交 `8ed2ea484` 的 `crates`、`Cargo.toml`、`Cargo.lock`、`.claude/kb`、`.claude/rules`。三份被判文件与主工作区逐字节相同（`cmp`，量的），它们的 sha256 在 `research/prompts/m4-gpugc-one-vulkan-instance-code-r1-snapshot/crates-sha256.txt`。把补丁打到 `baseline/` 上，三份也与主工作区逐字节相同（`cmp`，量的）。
- 被判的产品代码：`crates/singlefs-background-gpu/src/` 的 `background_gpu.rs`、`background_gpu_compute.rs`、`bin/e202_gpu_cpu_gc_comparison.rs`。
- 改法的落点（实现员报告「结论」一节，腿以冻结副本为准，报告只当线索）：
  - `VulkanInstanceOfThisProcess`（`Clone`，克隆指同一个实例）；构造 `created_for_this_process()`、只给单测的 `without_any_backend()`；枚举只经 `vulkan_adapters_enumerated_within(wait_limit)`。
  - trait `VulkanInstanceHandedToTheOpening`：产品实现交回 `self.clone()`，另一个实现是单测替身。
  - 开卡入口 `open_background_gpu`、`open_background_gpu_with_the_memory_leaked_in_this_process`、`open_background_gpu_compute` 都多一个实例参数。
  - 新拒绝成员 `BackgroundGpuOpenRefused::NamedGpuDeviceNotOnTheListOfThisEnumeration`、结构体 `AdapterSeenInTheEnumeration`、纯函数 `discrete_gpus_and_participating_gpus_admitted_from_the_enumeration`。
  - e202 的 `main` 只给要卡的子命令建一份实例，经 `GpusOfTheCommand::EnumeratedOn`、`OpeningOfTheCommand.vulkan_instance` 与两份 RunSetting 的 `vulkan_instance: Option<&…>` 传到开卡。
- 核心层没有调开卡入口的地方：实现员报告写 `grep -rln 'singlefs_background_gpu\|singlefs-background-gpu' crates/ --include=*.rs --include=Cargo.toml` 去掉本包只剩 `crates/singlefs-core/src/background_compute.rs` 第 5 行的文档注释（报告的量，腿现查）。
- 验证段的数（量的）：
  - 任务 9810 在第二台（3 张卡）跑六个真卡目标全绿，含 `opening_the_handed_over_gpus_again_and_again_in_one_process_succeeds_every_round_on_one_vulkan_instance ... ok`。
  - 合入后验证 9755 的两个包测试全绿，9 个测试二进制共 464 条，clippy、build 过；它停在验证包带 gpu 特性那一趟，那一趟没有结局，原因与去向在调度记录 `records/2026-10-19-里程碑四GPU垃圾回收开工.md` 末尾两行。
- 用户已定的（不攻）：测试里不用 `static`、不用全局计数；用户 2026-11-07 选「多卡那几条拆成单交」，门禁样本与 `.claude/skills/crash-test/commit-stages.md` 第 2d 步已照办。
- 测试进程里只做到「一条用例一个实例」：实现员报告「停下交主 agent」第 2 条，checksum 目标 8 条、live_slot 6、region_record 6、compaction 7（另有子进程 1 个）。第二台上的界大约是 12（量的，调查报告），每个目标各自一个进程，都小于这个界。这里只做到这一步的依据是用户定的不用单例，不是证明了界不会变。
- 变异行：`crates/mutations.tsv` 第 7016–7019 行加一条换过锚点的行。「产品交给开卡的实例每次另建一个」那一行只在第二台才红，违反「所有任务都要多机可用」，主 agent 打补丁前从 `mutations-append.tsv` 里删了（备份 `mutations-append.tsv.before-drop`），所以 `vulkan_instance_for_this_opening` 产品实现的 `self.clone()` 没有变异行钉着。
- 压着的条款：
  - `.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13（第 241 行起到下一个 `####` 为止），重点是定案第 2 条「用哪几张卡」与第 4 条「配错时拒；GPU 用不上时重交、报警、停下，GC 按占用在 CPU 上补跑」。
  - `.claude/kb/milestone/04-fourth-txn.md` 第 140 行「## 七　GPU 垃圾回收」一节的七·3（第 151 行）。
  - `crates/singlefs-background-gpu/Cargo.toml` 第 27 行：「不开时 `cargo test` 整包与整工作区都不碰 GPU」。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　过度外推与越位：实例真的只有一份吗
产品路径上每条开卡路径各拿到的是哪一份实例；`git grep` 全包的 `wgpu::Instance::new`、`Instance::new(`；`Clone` 之后是不是真指同一个实例（读 `Cargo.lock` 锁的 wgpu 30.0.1 源码，不靠报告）；e202 每个子命令建几份，sensitivity 交替跑与并发跑是不是共用一份。「界大约是 12、用例数小于它」这一句外推到了哪里：同一进程里失败重开、换设置重开、驱动全藏起来那一次，实例数各是多少。

### T2　内部矛盾：拒绝分不分得清
纯函数 `discrete_gpus_and_participating_gpus_admitted_from_the_enumeration` 的每一臂：点名的卡不在清单、独显没报总线地址、没有独显又没点名、别的核心层拒绝，各交回哪个成员；与 D26（后台整理与放置回收） 已定项 13 第 4 条「GPU 不在」那一格是不是同一件事，会不会把该报「GPU 不在」的报成新成员或反过来；调用方（`background_gpu_compute.rs`、e202、挂载与切换）有没有哪一处还按旧成员 `match`、把新成员归错；`# Errors` 文档与成员是否一一对应。

### T3　哪一条最脆弱：把手的寿命与没钉住的地方
- 实例的寿命：`AdaptersEnumeratedByTheInstance` 持一份克隆，`BackgroundGpu` 与泄漏的那一份（`GpuMemoryLeakedInThisProcess`）分别撑到什么时候；调用方丢掉实例之后已开的卡还能不能用；丢掉再新建会不会回到老问题。
- `self.clone()` 没有变异行钉着：有没有一条不靠第二台驱动、任何机器都能跑的办法钉住「产品实现不每次另建」，写出变异行与断言。
- 枚举超时、枚举为空这两种情况下，开卡是拒还是挂住，拒的是哪个成员。

### T4　不开特性不碰 GPU
`without_any_backend()` 的 `Backends::empty()` 在 wgpu 30.0.1 上真不装 Vulkan 加载器吗（读源码，报告写的是读出来的）；不要卡的单测 `every_opening_in_one_process_takes_the_vulkan_instance_handed_over_instead_of_creating_one` 与 e202 的 CPU 臂、`geometry-check`、`construction-timing` 子命令各会不会建真实例；`#[cfg(test)]` 的构造有没有漏到非测试代码。

### 检查单：走不到的理由二次查实
实现员报告没有「走不到的理由」一节（这一批交回在这一格立规之前）。主 agent 在改动全文里 `grep` 新加的 `.expect(`、`unreachable!`、`panic!`、`_ =>`，只命中 2 处，都在 e202 的单测里：`.expect("CPU 臂挂载得成")`、`.expect("枚举得到适配器")`。腿另查这一批有没有写成注释或不写的「走不到」，判是不是在任何落盘动作之前返回。

### 检查单：类型与名字
新名字：`VulkanInstanceOfThisProcess`、`VulkanInstanceHandedToTheOpening`、`vulkan_instance_for_this_opening`、`created_for_this_process`、`without_any_backend`、`with_the_backends`、`vulkan_adapters_enumerated_within`、`NamedGpuDeviceNotOnTheListOfThisEnumeration`、`AdapterSeenInTheEnumeration`、`of_the_description`、`discrete_gpus_and_participating_gpus_admitted_from_the_enumeration`、`vulkan_instance_for_the_gpu_arm`，单测常量 `OPENINGS_ON_THE_HANDED_OVER_INSTANCE`。新加 `expect` 依赖的不变量见「走不到的理由二次查实」那两处。

### 检查单：不兼容旧代码与旧格式
主 agent 读改动全文，没看到认旧版本的分支、旧布局号或迁移路径；这一批不碰盘上格式。腿现查。

### 检查单：不新建判定库
主 agent 读改动全文，没看到新的判定存放处。腿现查。

### 检查单：所有任务多机可用
「只在第二台才红」的那行变异已删（第二节变异行那一条）。新测试 `opening_the_handed_over_gpus_again_and_again_…` 用服务交的卡，两台都跑得起。腿查三份产品文件与新测试有没有读本机独有状态、写死地址。

### 检查单：测试优先多线程
本轮改动里真卡测试走 `--test-threads 1`（`.claude/rules/verification.md`「单个 GPU 测试二进制只用分到的卡」）；不要卡的单测每条只开卡 3 次，不值得切片（推的，没量过）。腿判 30 轮那条串行有没有写明为什么。

### 检查单：带 --item 的命令与项
实现员报告「要主 agent 跑的命令」末尾写这一批没有第 1d 步那三条：产品 GPU 包不在崩溃点重放合表与 Verus 登记里，也没有新的崩溃枚举用例。腿核这句话，并核新变异行能被 `crates-mutation-replay --list-items` 列出（读登记，不跑）。

### 检查单：新崩溃流多罩了什么
本轮改动没有这一类：查过 `.claude/gate.d/stage-inputs.tsv` 的 `crash-case:` 行与改动文件清单，这一批没动崩溃枚举用例（腿现查）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T4 与检查单八格，T1、T4 主攻 | 核「代码今天做的是不是改法声称的那件事」：每条开卡路径写文件:行号与实例从哪来；wgpu 源码那两处写路径与行号；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T4 与检查单八格，T2、T3 主攻 | 在冻结副本的拷贝上造反例：同一进程实例多于一份的路径；拒绝归错成员的输入；丢掉实例之后行为不同的时序；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号；引补丁块写 `_m4-gpugc-one-vulkan-instance-code-r1-diff.md` 里的文件名，不写那份文件的行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改。读码推出来、没量过的写「推的，没量过」。
- wgpu 源码在 `~/.cargo/registry/src/` 下，读，不改。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不写进命令）。要上卡的命令写明要卡、写明点名的测试目标，不写卡号。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
