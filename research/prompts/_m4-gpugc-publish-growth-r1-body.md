# m4-gpugc-publish-growth-r1：核心层每次发布的代价随版本涨，定位、外推与改法（2026-11-05）

## 一、题面

E202（GPU与CPU两种模式GC对照组）的装置造起步池（`crates/singlefs-background-gpu/src/bin/e202_gpu_cpu_gc_comparison.rs` 的 `construct_the_starting_image`）三档都建不出来。两位调查员（`/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/result.md`、`/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator-2/result.md`）读码装探针、主 agent 交了三条探针任务，给出定位与改法。用户 2026-11-05 看了弹窗之后定「重推 ，确认推的没有问题」。这一轮要判的是：调查员结论里**推的**那几样站不站得住，量的那几样口径对不对。

## 二、实现今天的样子（主 agent 的观测，2026-11-05）

- `crates/singlefs-core/src/transaction.rs:3205`–`:3210`：`pub struct PublishedUnit { pub slot: SlotNumber, pub identity: TransactionUnit, pub bytes: Vec<u8> }`，每个单元带整份字节。
- `crates/singlefs-core/src/mounted_session.rs:3064`–`:3067`：`published_in_this_change.versions_published.push(published.clone())`。
- `crates/singlefs-core/src/allocator.rs:2202`：`record_for` 的文档原句「某块盘上某个槽的分配记录；没分配过就 `None`。释放判定路径拿它核「映射查出来的落点真的在册、跨度对得上」。」
- 这四份核心层文件（`transaction.rs`、`mounted_session.rs`、`allocator.rs`、`allocation_record_tree.rs`）此刻与 HEAD 相同（`git diff --stat` 输出为空）。
- 量的读数（2288MiB 档，产物与日志都在仓外草稿目录，路径见下）：
  - 任务 7496，`research/results/e202-gpu-cpu-gc-comparison-2026-11-05/construction-timing-2288MiB-each-device-copies.out`：P=1024、2048、4096 时 `construction_seconds=27.324`、`184.946`、`950.446`；`phase=interleaved_half` 每步挂钟末八分之一均值 59.696、358.455、508.159 ms。
  - 任务 7555（共用块存法）P=4096 `construction_seconds=646.824`、常驻 `resident_bytes=2071564288`、块字节 `chunk_bytes_stored=586481664`；任务 7589（8GiB 档的盘）P=2048 `construction_seconds=149.637`。
  - 探针任务 7603（P=512、1024）、7622（P=2048）、7623（阳性对照），日志在 `/home/fy5090/.local/share/singlefs/work/m4-gpugc-construction-growth-investigator/probe-runs/task-<号>.log`，探针源码在同目录的 `probe-copy/`（相对主工作区改了 `crates/singlefs-core/src/` 下五份、加了 `investigation_probe.rs`，和 E202 装置一份）。
- 第二位调查员的结论（当线索，不当依据）：深拷单元那一组在 P=2048 末两个交错写窗口里占发布挂钟约一半，随单元字节线性涨；扫分配记录那一组占 26–28%，随分配记录数 R 线性涨；约 15% 没装探针、也在涨；改法甲档 `PublishedUnit.bytes` 换 `Arc<[u8]>`，乙档 `Arc<TransactionOutput>` 加持久结构，扫记录那一组换按（设备，槽）有序的结构、分配器整份拷贝换回滚日志；不碰盘上格式（引 `.claude/kb/decisions/03-空间分配.md:19`、`.claude/kb/decisions/08-核心索引结构.md:467`、`.claude/kb/decisions/16-发布语义.md:27`）。

## 三、格

### Q1　定位与外推

1. 读三份探针日志与探针源码，核调查员的四格（深拷、扫记录、内存盘、装置）口径：探针包住的是不是它说的那几行；阳性对照（`INVESTIGATION_EXTRA_UNITS_CLONE=1`）能不能证明探针抓得到一份整版深拷；四格之和与发布挂钟差的那 15% 落在哪。
2. 「每次发布进这几处的次数不涨，涨的是每次的代价」与「总挂钟高于 P² 是缺页造成」两句：前一句读数撑不撑，后一句是推的，能不能找到能分辨它的读数。
3. 外推「整档 P=36608 要几十小时」：用哪条拟合、在哪几个 P 上取的点、端点外推多远；给出区间并写明是推的。

要交的：每一问一个结论（成立 / 不成立 / 推的），不成立的给产物原行或源码行。

### Q2　改法

1. 甲档（`Arc<[u8]>`）：照 `crates/` 今天的实现，能不能只改类型而不改语义（`PartialEq`、录制流、写盘的字节、checker 读的东西都不变）；「系数小约三个量级」是怎么算的、对不对；改完每次发布还剩哪些随单元数涨的项。
2. 乙档（持久结构）：要动哪些公开类型、哪些调用方；是不是只有它能让每次发布不随单元数涨。
3. 扫分配记录那一组：换有序结构、碰过的叶作为分配释放的副产品记下、分配器拷贝换回滚日志，三样各自能不能做、碰不碰 `.claude/kb/decisions/03-空间分配.md` 已定项 7 的盘上编码、碰不碰 `.claude/kb/decisions/16-发布语义.md` 已定项 17「报错之后这次发布可能已成也可能未成」那一条对内存态的要求。
4. 常驻内存比内存盘块字节多出的那一截（P=4096 时 2071564288 对 586481664）：是不是同一个根（版本里带整份字节、挂载交回值 `MountOutput` 留着整份版本），改法管不管得到。

要交的：每一档、每一样一个结论（做得到 / 做不到 / 推的）与改到的文件；做不到的写为什么。

### Q3　归类

`.claude/rules/fs-design.md`「记账是事务的副产品，不是事后的遍历」那张表第一行「运行时决策路径（分配、ENOSPC 准入、defer 窗口、生命周期判定）｜不许，且代价不许随盘容量增长」：扫分配记录那一组与深拷那一组各落不落在这一行；依据只引条文原句与代码行。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | Q1–Q3 | 从探针日志、探针源码与 `crates/` 今天的实现推到结论，逐格给依据 |
| 攻方腿（Opus） | Q1–Q3 | 找调查员结论里站不住的：探针口径错位、阳性对照证明不了的、外推越位、改法漏掉的随 U 或 R 涨的项、改法会改语义或碰格式与已定分项的地方 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要造模型核的，模型放 `research/prompts/m4-gpugc-publish-growth-r1-<模型>-model/`，草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。
- 腿不交任务：要跑的命令写成 `task-server/task submit -- …` 的整条，写进报告「要主 agent 跑的命令」一节。
- 报告写进 `research/prompts/m4-gpugc-publish-growth-r1-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
