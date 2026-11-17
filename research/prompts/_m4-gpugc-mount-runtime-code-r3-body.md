# m4-gpugc-mount-runtime-code-r3：里程碑四七节 GPU 垃圾回收，运行库第二轮修补与 accept 抬 F，代码轮第三轮正文（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

判决 `research/prompts/m4-gpugc-mount-runtime-code-r2-main-verification.md` 第三节八条（第 113 批）与用户 2026-11-09 弹窗定的「accept 自己抬 F (Recommended)」（第 115 批）已合入。

这是这一题的第三轮，也是最后一轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第三轮分工：「已定」项撑不撑得住、未定项清单的完整性、报出的数字能不能核。这一轮只攻前两轮站住的形态与第二轮的修补；这一轮新冒出来的零轮形态不再开轮，写进判决的交用户表。不重复已判的格（满盘 accept、插队、反向饿线程、满盘加 fsync、低于底额让路不再攻，除非修补改变了它们）。

改动全文：`research/prompts/_m4-gpugc-mount-runtime-code-r3-diff.md`（相对第二轮冻结副本，含别的会话同时段的改动，那些当上下文读、不攻）。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r3/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-mount-runtime-code-r3-snapshot/crates-sha256.txt`。
- 被判的产品代码：`crates/singlefs-mount-runtime/src/lib.rs`；`crates/singlefs-core/src/` 的 `compaction_plan.rs`、`mounted_session.rs`、`transaction.rs`；`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs`。
- 量到的（合入之后，主工作区）：任务 11441「✓ 测试全绿（23 个测试二进制，共 600 条测试；跑了 23 项（4 个包），--item 点名的；按条复用：现判 599、复用 250、拒算 1）」「✓ fmt 过」；clippy 四个包任务 11523 退 0；静态核任务 11520 退 0；拒算任务 11532「拒算 12 条、登记表 12 行」。证红在副本上：任务 11339 抓到 19 条、11400 抓到 1 条、11405 抓到 10 条、11535 抓到 8 条。
- 没有结局的（写正文时，不当过）：崩溃流两项缩域任务 11533；门禁 code-source-discipline 五格任务 11530；任务 11519。
- 第二轮之后量到的读数（输出 `~/.local/share/singlefs/work/m4-gpugc-queue-walk-on-a-really-full-pool/`，探针 `probe/queue_walk_on_a_really_full_pool.rs`；任务 11290、11298，都是主工作区合入之前的代码，只写 `accept` 自己抬 F 之前的行为）：
  - 三种填法写满，队列走不动的状态走不到：`refused_walks=0`，续做 137 次 `WalkedToEmpty`。门槛扫到 128 到 256 个槽，填到被拒时还有 3400 多个空槽。这个「走不到」只在这几条历史上量过，320 MiB、240 段。
  - `arm=clean_unlink_only_accept base=DescendingSizes size=Largest files=1 small_went_through_at_pass=None`，`deferred=2330`：这是第 115 批要修的那一格。
- 第 115 批用例量到的一处（任务 11320）：`DescendingSizes` 上只删 1 个一单元或 32 单元的文件，`accept` 推抬 F 之后 `stop: FloorAlreadyAtTheCeiling { floor: CheckpointTxg(47), ceiling: CheckpointTxg(47) }`、`available: AvailableBytesOnOneDevice(-1017100)`，主 agent 判是 D16 保留 4 个状态的结果、不是缺陷，用例改成钉拒在上限、再删 3 个之后写得进（`crates/singlefs-mount-runtime/tests/mount_runtime_accept_raises_the_floor_for_space.rs` 的 `one_unlink_stops_at_the_floor_ceiling_and_four_go_through`）。这是主 agent 的判断，被攻过零轮。
- 实现员留给这一轮的（报告原话是线索，`~/.local/share/singlefs/work/m4-gpugc-accept-raises-the-floor-writer/result.md`、`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-r2-fixes-writer/result.md`）：`publish_the_pending_changes` 改成只在冻结的是这一批自己时才不放回待发；推的空发布占根环、不进 `publishes_since_the_last_user_change`，这几格根槽不在回退门的账上；`AcceptedButTheTriggeredPublishRefused` 带推成的串这一支没有专门的用例；真线程上重发成了之后队列至多等一个 `compaction_period` 才走；回退与抬 F 那一串冻结的种类是 `NeitherOfTheCountedKinds`。
- 已知红，不归这一批：可写挂载回退相关 15 条拒在 `NewerStateStillUnreadableAfterOneReread`（C971，别的会话在修）。
- 压着的条款：`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 1、7、10、17；`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 1、3、13。第二轮判决要写回的 D26 射程还没写回（kb 写回排在这一轮判决之后）。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　「已定」项撑不撑得住
`accept` 抬 F 之后，这些已定的说法还成不成立：D16 已定项 1 保留 4 个状态（抬 F 的空发布占根环、不进回退门的账；连续 `accept` 被拒、每次都推抬 F，根环与状态数怎么走，能不能把门的计数绕过去）；D26 已定项 13 第 5 条破例与「挂载不给新预算」（`accept` 推的串会不会让破例、备忘、预算判错）；D16 已定项 7、17（`accept` 自己写盘之后，写失败、冻结、待发放回那几条路上 fsync 还是不是只在持久之后答成功）。第二轮的「冻结时 fsync 拒」「队列不空算起跑」「重发成了照种类记账」各有没有被 `accept` 抬 F 破坏。主 agent 那条「只删 1 个文件写不进是 D16 的结果、不是缺陷」站不站得住：`available` 是负的，真是回退窗口占的吗，有没有一个不违反 D16 的历史里本可以写进去却拒了。

### T2　未定项清单的完整性
前三轮之后，这一题还有什么该定而没写进任何清单：`accept` 持锁时长（前台自己发空发布）与「一批之内 GPU 持锁」放一起的最坏；`accept` 抬 F 在崩溃循环里的边界；前台 unlink 整段持锁（判决登优化项）在抬 F 之后变没变；`mark_allocated` 摆满才走得到的那一格（只在 320 MiB 上判走不到）外推到大盘站不站得住；重发不退避。逐条答：该进哪张表（D26 射程、欠账、里程碑优化项），还是这一批就要修。

### T3　报出的数字能不能核
判决与本正文里报的数：`refused_walks=0` 与门槛 128 到 256（探针的 `leave_this_many_unblocked_slots` 摆法与真盘同不同口径）；`one_unlink_stops_at_the_floor_ceiling_and_four_go_through`「删够 4 个」这个数是不是恰好，3 个行不行、5 个是不是也可能不行（换填法、换文件大小）；余量取 S 那一组读数（`margin_crash_loop`）带不带最坏的活意图；证红合计。写得出推翻条件的给探针。

### 检查单：走不到的理由二次查实
两批实现员报告「走不到的理由」都写「无新增」（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-r2-fixes-writer/result.md`、`~/.local/share/singlefs/work/m4-gpugc-accept-raises-the-floor-writer/result.md`，`awk` 扫 `crates.patch` 里 `src/` 下的加行 0 处）。腿现查：这两批新代码里写成注释的「走不到」与 `expect`。

### 检查单：类型与名字
新名字：`AnswerOfTheCallerBeforeTheNextBatch`（成员 `NoForegroundFsyncWaiting`、`AForegroundFsyncIsWaiting`、`CallerIsStoppingItsThread`）、`answer_before_the_next_batch`、`YieldedToTheCallerStoppingItsThread`、`KindOfTheFrozenPublish`、`DeletionQueueNotEmpty`、`OnlyTheDeletionQueue`、`DeletionQueueWalkedWithoutACompactionPublish`、`publish_the_next_step_of_the_deletion_queue`、`RefusedAfterRaisingTheFloorForSpace`、`floor_raises_pushed_by_this_accept`、`floor_raises_pushed_for_space`、`deletion_steps_published`、`deletion_continuation_publishes`、`LimitGivenBackIfTheMakingPanics`。第二轮打中的两处（枚举名、停因）修到了没有。

### 检查单：不兼容旧代码与旧格式
主 agent 读补丁文件清单，这两批没有这一类；`crates/` 下 `grep` `ForegroundFsync` 不再命中枚举名（命中的是 `AForegroundFsyncIsWaiting` 这类成员与停因）。腿现查。

### 检查单：不新建判定库
这两批没有这一类，查过两批补丁文件清单。腿现查。

### 检查单：所有任务多机可用
新用例都在内存盘上现造，不读活树（拒算任务 11532 没有新增）。真写满那一族用例内存峰值 15.11 GiB（任务 11320 第 5 次尝试），两台都能跑，但排队久；腿判要不要这一批收。

### 检查单：测试优先多线程
`mount_runtime_accept_raises_the_floor_for_space` 14 条臂并行，各持一份盘；`crash_enumeration_frozen_publish_resent_stream` 用 `enumerate_crash_states_with_every_kind_of_observer`（切片并行与工作线程在它里面）。腿核。

### 检查单：带 --item 的命令与项
崩溃流两项 `frozen-publish-resent-after-a-failed-journal-write`、`frozen-publish-resent-after-a-failed-last-barrier` 登记在 `.claude/gate.d/stage-inputs.tsv` 与 `.claude/gate.d/crash-point-replay-items.tsv` 末尾；实现员报告的命令点了它们。形式化项由 `.claude/gate.d/formal-verification.sh --list-items` 现算（第 113 批改了 `transaction.rs`），主 agent 交；腿读登记，不跑。

### 检查单：新崩溃流多罩了什么
`crates/singlefs-verification/tests/crash_enumeration_frozen_publish_resent_stream.rs` 两条流：第二块盘日志记录写报错之后重发；最后一道屏障报错（根已写）之后重发。文件头写了比已有流多罩失败那一遍与重发那一遍同段的状态。腿核流里真有那几步、用甲二展开（单元写全不落或全落）丢不丢状态。第 115 批判不加流：`accept` 推抬 F 写的段序列与 `publish_user_change` 同一个函数，小用例钉前缀；腿判这一判够不够，`accept` 抬 F 之后待发只在内存里，这个状态崩溃之后盘上有没有已有流没罩到的。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端正推（Sonnet） | T1–T3 与检查单八格，T3 主攻 | 核「代码今天做的是不是判决与用户定案说的」：每条定值、每个门落到文件:行；报出的数逐个能不能核；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T3 与检查单八格，T1、T2 主攻 | 在冻结副本的拷贝上造反例，用户动作放开扫，探针能复跑；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑（服务撞内存上限会重排同一条命令，探针的内存与时间写进报告）；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
