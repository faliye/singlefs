# m4-gpugc-card-link-check-code-r1：判卡用不了那一刻查硬件与链路，代码轮第一轮正文（2026-11-13）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

被判的是一批已合进主工作区的改动（两份补丁：实现，加它的编码纪律修补）：判一张显卡用不了、要报警的那一刻，照卡的 PCI 地址读一次本机 sysfs 的 PCI 状态，读数带进报警与挂载的报错，行为不变。

压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 13 第 4 条里从「判一张卡用不了、要报警的那一刻，查一次这张卡的硬件与链路」起、到「这张卡再一次被判用不了时才再读一次。」止的那一段。第 4 条别的句子不归这一轮。

用户定的三件（原话在决策变更史 D26（后台整理与放置回收） 节 2026-11-13 块「已定项 13 第 4 条补报警那一刻查卡的硬件与链路」那一条的用户原话行，这一轮不抄变更史）：判卡用不了、报警那一刻查；读数写进报警与观测、行为不变；只有卡不在总线上才当判据。主 agent 另补的三处（同一块里「查硬件与链路的时刻由两种补成三种」那一条）：第三种时刻（卡当场报出错）、挂载拒绝时读数进挂载报错、停着之后报警带同一份不重读。

用户定这一批只走一轮：这一轮打中的改完就收，不开第二、三轮；判决里写明只攻过一轮。

方案按 `crates/` 今天的实现来谈。这一批还没合进主工作区（合入后验证在跑）：被判的代码读冻结副本 `~/.local/share/singlefs/work/m4-gpugc-card-link-check-code-r1/repo/`（主工作区加两份补丁，只带 `crates/`），不读主工作区的 `crates/`；改动全文在 `research/prompts/_m4-gpugc-card-link-check-code-r1-diff.md`。被判的产品代码六份：`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs`、`crates/singlefs-background-gpu/src/background_gpu.rs`、`crates/singlefs-background-gpu/src/background_gpu_compute.rs`、`crates/singlefs-background-gpu/src/gpus_of_this_host.rs`、`crates/singlefs-background-gpu/src/lib.rs`。

## 二、实现今天的样子（主 agent 的观测，2026-11-13）

读过的路径与看到的事实（行号以开工快照为准，腿引行时现查）：

- `crates/singlefs-core/src/background_compute.rs`：读数的数据类型 `PciStateOfTheGpuReadWhenJudgedUnusable`（三个成员：`NotReadForThisError`、`PciBusAddressNotADirectoryName`、`Read`，后者装箱）、`PciStateOfTheGpu`（照哪个地址、读的哪个目录、九样读数）、单个文件的三态 `PciFileReading`、目录的三态 `PciDeviceDirectoryReading`、使能位 `PciDeviceEnable`；判据纯函数 `hardware_judgement_of_the_pci_state`；三个出错成员 `BackgroundGpuComputeFailed::GpuFailed`、`GpuNotOpenedAfterTheReopenings`、`GpuUnusableAfterResubmissions` 各带一个装箱的读数字段；报警 `AlarmOfABackgroundTaskStoppedAfterAComputeError` 的读数由同名方法从 `error_that_stopped_the_task` 现取。核心层不读文件、不读钟。
- `crates/singlefs-background-gpu/src/pci_state_of_the_gpu.rs`（新模块）：`PciStateReaderOfTheGpus` 持 PCI 设备目录的根（产品取 `/sys/bus/pci/devices`，测试换成自己的目录），带读次数计数；`pci_state_of_the_gpu` 读目录与九份文件；`whether_the_cause_judges_the_gpu_unusable_in_a_batch` 判一批里的出错是不是该读的时刻。
- `crates/singlefs-background-gpu/src/background_gpu.rs`：一批在卡上出错的两处（`outcomes_of_a_batch_over_the_cards` 的出错臂、`plans_of_a_batch` 的出错映射）按出错原因决定读不读；挂着切换时开卡重开用完、试满当场答开不成、起不了线程而次数用完，各读一次；`BackgroundGpuOpenRefused::GpuUnusable` 带读数字段。
- `crates/singlefs-background-gpu/src/gpus_of_this_host.rs`：`gpus_opened_at_the_mount` 重开用完拒绝挂载时读一次。
- `crates/singlefs-background-gpu/src/background_gpu_compute.rs`：`core_failure_of_the_gpu` 多一个读数参数，照原样放进核心层的三个成员。
- 挂载报错：开卡器把 `BackgroundGpuOpenRefused` 的 Debug 文本写进运行库的 `BackgroundGpuNotOpened::RefusedByTheOpening { description }`，`crates/singlefs-mount-runtime/src/lib.rs` 没改。

实现员没写进条款、自己选的十二处在它的报告 `~/.local/share/singlefs/work/m4-gpugc-card-link-check-writer/result.md`「条款没写的岔路」一节（腿读不到那份报告，要紧的八处抄在第三节 T2）。

## 三、格

### T1　代码做的是不是条款说的

逐句对：条款那一段的每一句（读哪九样、三种时刻、平常的批与每次重交之前不查、读数写进报警与观测而行为不变、不读 Xid 与 `nvidia-smi`、读不到的报「读不到」不当 0 不当正常、只有卡不在总线上才判硬件坏、挂载拒绝时读数进挂载报错、停着之后每次报警带同一份不重读、再一次被判用不了才再读）在代码里各落在哪一行、落得对不对。有一句没落、落得比条款宽或窄，就是打中。

### T2　条款没写、实现员自己选的地方站不站得住

1. 「卡当场报出错」只算 `UncapturedErrorOrDeviceLost`；`WaitForTheSubmissionFailed`、`ReadbackMappingFailed` 这类同样让任务停下的不读。
2. 切换时量代价出错（`CostNotMeasuredAtTheSwitch`）不读。
3. 切换开卡开不成在会话锁外的开卡线程上读；那一次结果后来被新的切换盖掉、没被采用时也已经读过一次。
4. 试满（`OpeningsInThisMountUsedUp`）当场答开不成也读，同一组设置每交一次读一次。
5. 挂载报错里的读数是 Debug 文本，不是带类型的成员。
6. 卡的标识当不了一个目录名（不是 UTF-8、空、`.`、`..`、带 `/` 或 NUL）时不读，报 `PciBusAddressNotADirectoryName`。
7. `enable` 按使能计数解析：0 是关的、非 0 是开着的、不是十进制数的报 `UnrecognizedText` 且不判坏。
8. 目录不在时九份文件照样各读一次。

每一处判：与条款的字面和用户定的三件有没有冲突；会不会让「行为不变」不成立；会不会在三种时刻之外多读或在三种时刻里漏读。第 4 处另判：停着之后调用方反复交同一组设置时读的次数有没有上界。

### T3　哪一格会错

- 读的路径：卡标识拼进路径那一处读不读得到 PCI 设备目录之外；`read_dir`、`fs::read` 会不会在某种文件（很大的、读着会阻塞的）上卡住或占很多内存，卡住时持着什么锁。
- 读发生时持不持会话锁、会不会拖住前台或别的卡的批。
- 行为不变：停下、报警次数、CPU 补跑、切换解除停下的判定有没有因为出错类型多了字段而变（出错的相等比较今天带上了读数：有没有哪一处拿两个出错比相等来判「同一个错」，读数不同就判成不同）。
- 报警的读数从 `error_that_stopped_the_task` 现取：有没有一种停下，停下的那个错不带卡而读数本该有。
- 再一次被判用不了才再读：停着时又来一次出错，停下记录换不换成新的错、读数跟不跟着换。

### 检查单：走不到的理由二次查实

两份实现员报告的「走不到的理由」一节原文：产品代码（`crates/<包>/src/` 非测试部分）没有新加 `todo!`、`unreachable!`、`assert!`，也没有新加名字以 Undecided、Unsupported 或 OutsideTheFirstVersion 收尾的错误成员；新加的 `assert!` 都在 `#[cfg(test)]` 模块里。腿在开工快照上对这一批改到的产品文件现搜一遍核这句话。

### 检查单：类型与名字

新加的类型：`PciStateOfTheGpuReadWhenJudgedUnusable`、`PciStateOfTheGpu`、`PciFileReading`、`PciDeviceDirectoryReading`、`PciDeviceEnable`、`PciStateReaderOfTheGpus`，判据函数 `hardware_judgement_of_the_pci_state`，读法 `pci_state_of_the_gpu`、`whether_the_cause_judges_the_gpu_unusable_in_a_batch`，访问器 `GpuDeviceIdentifier::opaque_bytes`，观测 `pci_state_readings_taken`。腿核：同一个量不同语义有没有共用裸整数、有没有缩写（`pci` 已登记在 `.claude/abbreviations`）、有没有 `_ =>`、每处 `expect` 写没写依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

本轮改动没有这一类：不动盘上格式，没有版本号常量与迁移路径。查过的范围：两份补丁改到的全部文件。腿核有没有为旧调用方留的兼容成员或缺省参数。

### 检查单：不新建判定库

本轮改动没有这一类：没有新的判定存放处。查过的范围：两份补丁改到的全部文件。

### 检查单：所有任务多机可用

新测试不读本机真 sysfs：读真目录的测试在临时目录里现造文件；「不在的根」用固定的绝对路径 `/singlefs-background-gpu-tests-pci-device-directories-never-created`。腿核：有没有一条新测试的判定随本机有没有显卡、sysfs 长什么样而变；那条固定路径在两台上是不是都读成目录不在。

### 检查单：测试优先多线程

新测试是纯函数与假目录上的单测，各自建状态，由测试框架并行。腿核有没有共用同一个临时目录或同一个计数器而互相干扰的。

### 检查单：带 --item 的命令与项

本轮没有新的崩溃枚举用例与放量分项；变异行 30 行新加、11 行整行替换，都能被 `crates-mutation-replay.sh --item` 点名（主 agent 在快照上点名跑过，任务号与结局在判决里写）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：没有新加崩溃点重放流或用例。读数只在内存里带，不写盘。

## 四、分工

- 正推腿：T1 逐句对条款；检查单八格逐格回「成立」「不成立」或「推不出」。
- 攻方腿：T2、T3；检查单八格逐格回。攻方可以写小模型或小程序核读路径的行为，命令交主 agent 跑。

## 五、交付要求

- 报告分段落盘，每次工具调用写进文件不超过 150 行，第一段排他新建；回复只写短句。
- 草稿与临时文件放腿自己的目录 `~/.local/share/singlefs/work/<腿名>/`。
- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写 `路径:行号`，照开工快照现查。
- 要跑的都写成 `task-server/task submit -- <命令>` 的整条写进报告「要主 agent 跑的命令」一节，腿自己不交。
