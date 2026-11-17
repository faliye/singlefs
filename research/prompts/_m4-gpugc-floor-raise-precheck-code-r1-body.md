# m4-gpugc-floor-raise-precheck-code-r1：里程碑四七节 GPU 垃圾回收，抬 F 之前先算，代码轮第一轮正文（2026-11-10）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

判决 `research/prompts/m4-gpugc-mount-runtime-code-r3-main-verification.md` 第三节第 1 条，用户 2026-11-09 弹窗定「改：推之前先算」：可用加 defer 里全部字节仍小于需求就不推，报空间不够；小写照推。主 agent 在这个射程内定：两种短缺都判，放得出的量算上界、defer 加隔离，只拒怎么推都不够的写，保住被宽护卡住那条出路。第 119 批实现，补丁目录 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-writer-r3/patch/`，实现员报告同目录 `../result.md`，已合入主工作区。

这是这一题的第一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第一轮分工：过度外推与越位、内部矛盾与循环依赖、哪一条最脆弱。

改动全文：`research/prompts/_m4-gpugc-floor-raise-precheck-code-r1-diff.md`（第 119 批补丁里 `src/` 下、验证包之外的段）。

## 二、实现今天的样子（主 agent 的观测，2026-11-10）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r1/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-floor-raise-precheck-code-r1-snapshot/crates-sha256.txt`。
- 被判的产品代码：`crates/singlefs-core/src/mounted_session.rs`（判法在 `push_one_floor_raise_for_a_refusal_for_space` 开头，3 处调用共用；新成员 `UserChangeRefused::NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`）；`crates/singlefs-core/src/compaction_plan.rs`、`crates/singlefs-mount-runtime/src/lib.rs`（只改文档）。
- 量到的（实现员副本上，合入之前）：clippy 三个包任务 11699 退 0；核心层判法模块 10 条任务 11702 绿；验证包 18 个目标 1215 条任务 11701 绿。证红：核心层 10 条任务 11738 抓到 10 条；验证包 4 条任务 11739 抓到 4 条；运行库 3 条任务 11776 抓到 3 条（现判 2、复用 1）。
- 没有结局的（写正文时，不当过）：运行库 `mount_runtime_accept_raises_the_floor_for_space` 那一族（真写满用例）在副本上的那一趟撤了（任务 11700，证红要往同一份副本打变异）；合入后验证任务 11805 写正文时还在排队。
- 实现员留给这一轮的（报告原话是线索）：真推过一串、第二轮才判到放不下的那一支没有整池用例，只在核心层单测钉了「带着串」；上界恰好等于需求只钉在纯判法上；`crash_enumeration_last_fitting_…` 严格写着 `NoSpaceAfterRaisingTheFloor`，可能变红。
- 已知红，不归这一批：可写挂载回退相关拒在 `NewerStateStillUnreadableAfterOneReread`（C971，别的会话在修）。
- 压着的条款：`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 1（「抬 F 的上限」「准入」两行）、已定项 10；`.claude/kb/decisions/28-挂载期承诺量.md` D28（挂载期承诺量） 已定项 1。用户 2026-10-08 定的被宽护卡住那条出路（`wide_protection_at_the_ceiling`）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　过度外推与越位
「上界」真是上界吗：抬 F 一串放得出的量，是不是只有 defer 加隔离加此刻可用（扣住的槽、被抛弃根独占、挂载期承诺量、环窗口随抬 F 变不变）；槽对那一判的上界（段外槽对加每个放得开的槽各补一对）是不是该这样算。判法有没有越位到不该拦的拒绝（`PlacementRefused`、提交内生块那一判照旧推）。上界算小了会误拒一个推了本来写得进的写：找这样的历史。

### T2　内部矛盾与循环依赖
判法读的分配器与后面推串用的是不是同一份；第二轮才判到放不下时带回的串、被拒、已发成的版是否与 `publish_user_change`、`accept`、运行库三层的记账一致（预算、备忘、冻结种类、上一条完成的意图）；D16 已定项 10 定案「过不了就在任何读写之前拒」与 `accept` 抬 F、与这一判的关系。

### T3　哪一条最脆弱
上界恰好等于需求照推那一格；多块盘一块短一块够；短缺种类混着来；宽护卡住的那条出路（`crates/singlefs-verification/tests/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table.rs`）有没有被这一判吃掉。

### 检查单：走不到的理由二次查实
实现员报告「走不到的理由」原文：产品代码没有新加 `todo!`、`unreachable!`、`assert!`，新错误成员不是条款没定的那一种。两处 `expect`（都在 `mounted_session.rs`）：`SlotsAFloorRaiseCouldRelease::of_the_device` 的 `checked_sub … expect`（没挡的槽数不大于空闲槽数）；`devices_short_even_with_everything_a_floor_raise_could_release` 的 `find … expect`（拒绝里的盘来自同一个分配器，调用点只有 `push_one_floor_raise_for_a_refusal_for_space`）。腿逐处追调用点判。

### 检查单：类型与名字
新名字：`NoSpaceEvenWithEverythingAFloorRaiseCouldRelease`、`SlotsAFloorRaiseCouldRelease`、`devices_short_even_with_everything_a_floor_raise_could_release`（腿现查全表）。字节与槽两种单位有没有共用裸整数；有没有 `_ =>`。

### 检查单：不兼容旧代码与旧格式
本轮改动没有这一类（主 agent 读补丁文件清单）；腿现查。

### 检查单：不新建判定库
本轮改动没有这一类：查过补丁文件清单。腿现查。

### 检查单：所有任务多机可用
新用例都在内存盘上现造；真写满那一族内存峰值 15 GiB 量级（第三轮判决已登优化项）。腿核新用例有没有读本机独有状态。

### 检查单：测试优先多线程
核心层判法用例是纯函数小用例；运行库两条崩溃循环用例单线程跑一条历史，理由写没写（腿核）。

### 检查单：带 --item 的命令与项
实现员报告「要主 agent 跑的命令」第 9、10 条：崩溃点重放 `--item truncate-to-zero-on-a-nearly-full-pool --item last-fitting-unit-then-refusal-and-unmount-on-a-nearly-full-pool --item floor-raise-pushed-by-the-session`；crates-mutation-replay `--item` 5 行。腿核项名在 `.claude/gate.d/crash-point-replay-items.tsv` 与 `crates/mutations.tsv` 里存在（读登记，不跑）。

### 检查单：新崩溃流多罩了什么
本轮改动没有新崩溃流（实现员报告）。腿判：这一判让写在推串之前就拒，盘上少了几串空发布；已有的「会话推串」与贴边两条流（上面三项）罩不罩这个少写的状态，要不要加小用例钉「被判放不下时录制流一个写都没有」。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T3 与检查单八格，T1 主攻 | 核「代码今天做的是不是判决与用户定案说的」：上界每一项落到文件:行；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T3 与检查单八格，T1、T3 主攻 | 在冻结副本的拷贝上造反例：误拒一个推了本来写得进的写、宽护那条出路被吃掉、多盘与混合短缺；用户动作放开扫，探针能复跑；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑，探针的内存与时间写进报告（真写满的盘一份约 0.7 GiB，避免一次并行持十几份）；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
