# 根环实现第 10y 批：代码轮第一轮正文（2026-10-07）

<!-- doc-lint:not-numbers K1 K2 K3 K4 G1 -->

## 一、题面

这一轮攻第 10y 批：它修第 10x 批代码轮判决第三节那几条。判决是 `research/prompts/m3-batch10x-code-r1-main-verification.md`。另攻删除线第 33 批 T 与根环线 T1 甲的交界。

GPU 用在崩溃放量与对拍上的总口径在 `.claude/kb/decisions/24-后台重活能不能卸给GPU.md`（D24（后台重活能不能卸给 GPU））。T1 甲在 `.claude/kb/decisions/16-发布语义.md` 已定项 1 第 37 行，C694 在 `.claude/kb/checks-owed.md`。

用户原话：
- 「全局的任务仅仅管会不会对指定的电脑造成cpu或者gpu的oom」；
- 「不压并发」；
- 2026-10-07 定 G1 二：不做跨进程账，每次借现读空闲、只记本进程借的，两个进程的竞态靠运行时退 245、报本次未跑兜底（岔路单 `research/prompts/m3-batch10t-code-r1-forks.md`）。

改动全文：`research/prompts/_m3-batch10y-code-r1-diff.md`。实现员报告：`~/.local/share/singlefs/work/m3-r1/batch10y/patch/report.md`。

| # | 改动 | 落点 |
|---|---|---|
| A | K1-1：每借一次只跑一次 `nvidia-smi -q -d MEMORY,PIDS`，空闲与本进程占用出自同一份报告；认出本进程没走卡池的设备；已销账还没还的单独记上界；「空着时能借的」上界改用 Total − Reserved | `crates/singlefs-harness/src/gpu_memory_ledger.rs`、`crates/singlefs-checker-tier/src/gpu_unit_checks.rs` |
| B | K1-3：起设备时显存不够分开报本次未跑、CRC 管线接住、每个设备装回调接另起线程上的 Out of Memory、只排计划那一趟包进去、设备丢了下一次借重起、不再缓存 Err | `gpu_unit_checks.rs`、`crates/singlefs-checker-tier/src/crash_amplification.rs` |
| C | K2-2：每张卡此刻都放不下一个状态而空着时放得下，报本次未跑；`NothingToPlace` 拆两种 | `crash_amplification.rs`、`gpu_memory_ledger.rs` |
| D | K3：判完再核借据、扫源码认 `cfg_attr`、名单外 tests/ 也扫、补变异行 | 两份测试、`crates/mutations.tsv` |

## 二、实现今天的样子（主 agent 的观测，2026-10-07）

- 冻结副本 `~/.local/share/singlefs/work/m3-batch10y-code-r1/tree/`，取在 10y 合入之后（底含删除线 U、T）；被判文件 sha256 在 `research/prompts/m3-batch10y-code-r1-snapshot/crates-sha256.txt`。
- 合入验证与变异证红：见调度记录「10y 合后验证」那一行。
- 已知没做、不在这一轮被判范围的（腿报出来不算打中）：
  - `lease_card_waiting_for_room` 同进程兄弟借着时 panic：调用都在对拍文件，转删除线下一批；
  - 单子外的 9 处过时注释；
  - 驱动固定开销 2560 MiB 没上卡量。
- 实现员自定、零轮的：「带进程那一段」的 `nvidia-smi -q` 格式照二进制模板写，主 agent 上卡核。
- 用户已定、报出来不算打中的：两个进程同时读到同一份空闲的竞态，由运行时兜底。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### K1　判决第三节各条做到没有
1. K1-1 四形在新代码上还在不在？一次 `nvidia-smi -q` 解析出来的空闲与本进程占用，与 `--query-*` 两种读法取的是不是同一个量？解析认不得的输出时，是报错、本次未跑，还是悄悄当 0？
2. K1-3 那几处 Out of Memory 都接住了吗？回调接住之后，这一块的结局是本次未跑、判完的块照写进库吗？设备丢了重起时，会不会把丢之前借的账算错？
3. K2-2 拆出的两种结局，各自的出口对不对？

### K2　只防 OOM、不压并发两头（新代码引进来的）
1. 新加的上界、重起、回调，有没有一形让账给的量大于此刻真能用的？
2. 有没有一形，卡放得下却报本次未跑、判红，或者整趟弃卡？

### K3　测试判别力
扫源码、判完再核、新变异行，能不能拦住这一批每一处被改回去？有没有一形「改回去」照绿？

### K4　删除线第 33 批 T 与 T1 甲的交界
T 改了 `crates/singlefs-core/src/mount.rs`：被抛弃根树表读不出时，从「只计数不拒」改成「现行账里有那条根之后的记录罩住它的树表槽就计数，没有就拒可写」，这是 C694 的 B。T 还改了 `crates/singlefs-core/src/allocator.rs`：隔离账加「水位之上没施加的 journal 记录点名的落点」，这是 C694 的 C。这一格攻两个问题：
1. 与 T1 甲（重建分配器两遍里只有一遍自证过的槽，抬 F 那一串拿记着的根顶上）放在一起，有没有一形让被抛弃根的落点掉出影子账（I-7.4）？
2. 有没有一形，本该放行的可写挂载被拒？

## 四、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
