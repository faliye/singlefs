# m4-gpu-wait-outside-the-lock-code-r2：后台整理一批等 GPU 拆到会话锁外、前台等锁让路，代码轮第二轮正文（2026-11-20）

## 一、题面

攻代码轮第一轮判决的修补：判决 `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md`「三、改法」七条，与合入之后修的两行变异锚点。另派一条辩方腿复核第一轮的判决。

- 条款：`.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 4、5、6 条（第 6 条已照用户 2026-11-19 弹窗定的「取了输入没让也清零 (Recommended)」写回）；`.claude/kb/decisions/24-后台重活能不能卸给GPU.md` 已定项 2 的射程；设计第三轮的判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md`「三、改法」十四条。
- 修补的补丁（拆分之前写的、以那时的主工作区为基线）：`~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-fix/patch/`（`crates.patch` 改 7 份文件，`mutations-append.tsv` 追加 16 行、`mutations-replacements.tsv` 换 5 行），实现员报告 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-fix/result.md`；锚点修补 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-anchor-fix/patch/`（换 `crates/mutations.tsv` 两行：「七·3 设置交给实现：切换生效的那一批开头交给实现的是换上之前的那一组」「七·3 GPU 代码轮一：一批三段怎么交的不记进那一批的记账」），报告同目录 `result.md`。两份补丁照用户指令不经验证直接合进了主工作区，之后 crates 代码拆分把这几份文件拆成了目录；拆分没改语义，拆分本身另有一轮代码轮，这一轮不攻拆分。
- 修补在拆后的树里落在哪（主 agent 现查的行号）：
  - 改法第 1 条（回锁出错那一支过回退门）：`crates/singlefs-core/src/mounted_session/compaction_batch.rs:633` `judge_the_computed_compaction_batch_at_the_relock`、`:755` `is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate`；`crates/singlefs-core/src/mounted_session/compaction_publish.rs:285` `rollback_gate_at_the_relock_of`。
  - 改法第 2 条（回锁有冻结就丢批）：`crates/singlefs-core/src/mounted_session/compaction_batch.rs` 与 `crates/singlefs-core/src/mounted_session/background_compaction_out_of_the_lock.rs:512` `publish_the_batch_computed_out_of_the_lock`；字段 `txg_of_the_frozen_publish_at_the_input` 在树里 0 处。
  - 改法第 3 条（GPU 包建缓冲那一等读停标记）：`crates/singlefs-background-gpu/src/background_gpu/buffers_of_a_chunk_in_one_out_of_memory_scope.rs:26`、`:50`，`crates/singlefs-background-gpu/src/background_gpu/waiting_for_wgpu_futures_and_thread_clocks.rs:82` `wait_for_the_wgpu_future_before_the_deadline`，`crates/singlefs-background-gpu/src/background_gpu.rs:872`。
  - 改法第 4 条（前台 fsync 先 `try_lock`）：`crates/singlefs-mount-runtime/src/lib.rs:671` `lock_the_session_for_a_foreground_fsync`、`:700` `register_a_foreground_fsync_waiting_for_the_session_lock`、`:1519` `publish_pending_now`。
  - 改法第 7 条（取了输入没让也清零）：`crates/singlefs-core/src/compaction_plan.rs:767` `break_the_yields_in_a_row_by_a_batch_not_yielding`，调它的在 `crates/singlefs-core/src/mounted_session/compaction_run.rs:465` `judge_before_the_next_compaction_batch`。
  - 改法第 5、6 条的测试：`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/`（拆成目录，`main.rs` 加六份子文件）、`crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs`、`crates/singlefs-mount-runtime/tests/mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock.rs`；变异行在 `crates/mutations.tsv`（这一批的行名单在开工快照目录的两份 tsv）。
- 合入后验证量到的（主 agent 的观测）：写段 28 条命令与锚点 3 条都出了结局、退 0，候选变异证红都抓到；拆后的树上合入后验证另交一趟点 16 个测试目标（结局写进判决「一、腿与核查」）。用到 GPU 的目标与崩溃枚举用例等判定键只算输入数据那一件做完再验，这一轮不判它们的读数。
- 方案按 `crates/` 今天的实现来谈；读码引主工作区今天的路径与行号。

## 二、要攻的地方

1. 七条修补本身：每条改的是不是判决那一条说的，改了之后有没有新的错（例：第 1 条门关不发之后，意图与 `R` 排他留到门开，停着时那一趟开头那一判真的会在门开时补发吗；第 2 条把冻结判据放宽到「回锁时有冻结」之后，取输入前就冻着的那一批在取输入那一刻就该拒还是到回锁才丢；第 3 条拆出期限那一半之后，期限照这张卡的账算对不对，停标记读的是不是同一份；第 4 条 fsync 改成先 `try_lock` 之后，取不到再登记那一刻到真去等锁之间后台有没有漏让；第 7 条清零放在写带宽与回退门两判之前，显式入口与可写挂载续做不问调用方时只靠一批发成清零，两种口径会不会让计数在某条路上永远不清或清得太早）。
2. 第一轮只抽查过、留给这一轮的三处：
   - 停着时开头单发的删意图发成之后，清不清让路的两个计数（照第 6 条新口径「一批发成两个都清零」，删意图也是一次整理发布）。
   - 签发那一支的空洞数、要搬数取现行那一版：补的用例 `an_issue_at_the_relock_takes_the_quantities_of_the_intent_from_the_current_version`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs`）是不是真让第一轮那四条候选变异红。
   - 签发重判资格里「开过的段」「钉住与隔离」两个合取项有没有用例罩到：改掉其中一项，现有用例会不会照样绿。
3. 合入之后修的两行变异：原文在今天的源码里是不是各命中 1 次、替换文改回去点名的测试会不会红。
4. 测试测的是不是条款而不是实现：新用例与改写的用例（`a_compaction_publish_of_the_explicit_entry_clears_the_yields_in_a_row` 替掉原来那条）断言的是条款第 5、6 条说的行为，还是这一版实现碰巧的计数。

## 三、检查单

### 检查单：走不到的理由二次查实

修补的实现员报告「走不到的理由」一节原文：「无。产品代码 `crates/<包>/src/` 里这一批没有新加 `todo!`、`unreachable!`、`assert!`，也没有以 Undecided、Unsupported 或 OutsideTheFirstVersion 收尾的错误成员（GPU 包里新加的 `expect` 在 `#[cfg(test)]` 模块的替身里）。」锚点修补只改 `crates/mutations.tsv`。腿核这句在拆后的树里还成立：修补碰的几处函数里有没有新加的走不到的断言。

### 检查单：类型与名字

新名字：`RollbackGateAtTheRelock` 的新用法、`is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate`、`BatchDroppedWhileAPublishIsFrozenAtTheRelock`、`wait_for_the_wgpu_future_before_the_deadline`、`lock_the_session_for_a_foreground_fsync`、`register_a_foreground_fsync_waiting_for_the_session_lock`、观测 `foreground_fsyncs_registered_as_waiting_for_the_session_lock`、`break_the_yields_in_a_row_by_a_batch_not_yielding`。腿核：同一个量不同语义有没有共用裸整数、有没有缩写与 `_ =>`、`expect` 写没写清依赖哪条不变量。

### 检查单：不兼容旧代码与旧格式

修补不改盘上格式、不加版本号与迁移路径；第 2 条删了取输入时记的冻结 txg 字段，没留旧判据的分支。腿核有没有为旧路径留的分支。

### 检查单：不新建判定库

修补只加单元与集成测试，不碰崩溃枚举用例与判定存放。腿核测试有没有另开判定存放处。

### 检查单：所有任务多机可用

新用例不读本机独有状态、不写死主机名与地址。腿核。

### 检查单：测试优先多线程

运行库两份测试 `#[test]` 12/12、10/10 都带不切片并行的理由注释（改法第 6 条，计数命令在修补报告第五节）；验证包新用例各自一个池、一个场景。腿核该并行而没并行的、理由站不站得住。

### 检查单：带 --item 的命令与项

修补报告第五节的命令都带范围声明与点名的测试函数；拆后的树上合入后验证点的 16 个测试目标由 `merge-and-verify.sh --list-items` 列出。腿核修补新加的变异行点名的测试在拆后的树里还筛得到（目标 `compaction_batch_computed_out_of_the_session_lock` 拆成了目录，`--test` 名不变）。

### 检查单：新崩溃流多罩了什么

本轮改动没有这一类：修补没有新加崩溃点重放流或崩溃枚举用例，查过的范围是两份补丁的 `crates.patch` 与追加、替换的变异行。

### 检查单：放的位置

新用例住在被测那个包自己的 `tests/`（运行库在 `crates/singlefs-mount-runtime/tests/`，跨包的在验证包 `crates/singlefs-verification/tests/`）；GPU 包的替身在它 `src/` 的 `#[cfg(test)]` 模块里。没有新开包、没有跨包 `#[path]` 与 `include!`。腿核。

## 四、分工

| 腿 | 立场 |
|---|---|
| 云端辩方（Sonnet） | 复核第一轮判决：采的七条够不够得着条款、「不采」的四条（第 8 到 11 条）判得对不对、交用户那一条的两个候选是不是同样被打中；再逐条核修补做的是不是判决说的，九格逐格答成立 / 不成立 / 推不出 |
| 云端攻方（Opus） | 攻上一轮的修补与第二节第 2 条那三处；第一轮已攻过而没打中的角度不重复：锁外与维护线程发布交错、CPU 路径卸载等整批、锁里旧入口仍是 pub、取输入先排空后问让路。造修补之后才写得出的错误历史；测试改回去会不会照样绿；由用户决定的动作放开扫 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查。
- 引代码写主工作区路径与行号，读的时候核一下那一行还在。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节；读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
