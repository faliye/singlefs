# 增补 3 第 1 件（随机历史生成器）实现员报告

时刻按 UTC 记（东京 = UTC+9）。开工 13:53Z。

## 〇、结论与停在哪

- **后续（第十二节）**：主 agent 定甲之后补丁已落进主工作区（清单第 1 条指到收口表第 43 行），两处生成器前提写了出处；`check.sh` 绿（15:00:23Z–15:01:17Z）；第 129、130 行按门禁 59 号的做法在打了补丁的工作区副本上复跑都红（第 130 行只有种子 54、69 判得出，余量薄）；整表锚点预扫 125 条都恰好命中一次。下面第〇到十一节是定甲之前写的，快档「红」、「阻塞」那几句以第十二节为准。
- 生成器、执行器、「已知红」清单、收缩、快档 / 大档 / 收缩一个种子的用例都落了地；随机源手写 SplitMix64，同一个种子逐项、逐字节复现。
- **停下交主 agent（阻塞）**：随机历史在今天的代码上撞到一类清单外的失败（新发现，第四节）：回退之后把 F 抬进「回退目标根与新实例第一次发布之间」那段空档，抬 F 那一步之后 checker 判 I-3.1（已分配统计对得上） 红。快档种子 [0, 96) 里种子 80 撞到它，所以**快档在今天的代码上是红的**，`check.sh` 因此红在 cargo test（第七节原样）。
  它登不登进「已知红」清单、指到收口表哪一行（或先修），是主 agent 的判断，我没自己定；按定义「你要证明的那条在基线红集里就停下交回」停在这里。
  备好了一份补丁（清单加第 1 条 + 一条钉住它的复现用例），在副本上量过：打上之后快档绿，第 41、121 行两条变异各自在写死的种子数之内判红（第五节）。补丁路径与内容见第九节。
- 起始清单只登记了第 ② 行（根环转圈之后 I-3.1 红）；第 38、39、40 行核过代码，不登记（第三节）。
- 快档在主工作区上 25–27 秒（debug、16 线程、机器轻载时；第六节）；变异下 21–26 秒（不收缩，只报种子）。

## 一、这一轮写过的文件

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-harness/src/history.rs`（新建） | 生成器、执行器、panic 捕获、「已知红」清单、收缩、并行跑一批种子、计数与报告 |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`（新建） | 快档、大档（`#[ignore]`）、收缩一个种子（`#[ignore]`）、已知红第 0 条的复现、同种子逐字节复现 |
| `crates/singlefs-harness/src/lib.rs` | `pub mod history;`；`SharedStream::operation_count`（不拷整条流数步数） |
| `crates/singlefs-harness/src/crash.rs` | `SparseDevice::read_into`（先整段写 0、再按区间只拷写过的扇区），`read` 与 `SparseBlockDevice::read_at` 改走它：可写挂载与冷启动要把 768 MiB 的 journal 环逐条读一遍，debug 下一次挂载从 0.48 秒降到 0.12 秒（草稿 `timing/` 里量的） |
| `crates/mutations.tsv` | 末尾追加第 129、130 行（变异名见第五节）；别的行没碰 |

`git diff --stat -- crates litmus` 原样（新建的两个文件没进 git，不在 diff 里）：

```
 crates/mutations.tsv                 |  2 ++
 crates/singlefs-harness/src/crash.rs | 25 ++++++++++++++++---------
 crates/singlefs-harness/src/lib.rs   |  6 ++++++
 3 files changed, 24 insertions(+), 9 deletions(-)
```

`git status --short -- crates litmus`：`M crates/mutations.tsv`、`M crates/singlefs-harness/src/crash.rs`、`M crates/singlefs-harness/src/lib.rs`、`?? crates/singlefs-harness/src/history.rs`、`?? crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`。

没碰 `crates/singlefs-core`、`crates/singlefs-checker`、`litmus/`。

## 二、生成器与执行器：做了什么、放在哪、第 3–5 件怎么接

放在 `crates/singlefs-harness/src/history.rs`（库模块，不放测试文件里）：第 3–5 件的用例文件 `use singlefs_harness::history::…` 就能拿到同一个生成器与执行器，门禁阶段与以后的二进制也能用。

- **随机源**：`SeededRandomSource`（第 62 行），SplitMix64，不加依赖；单测钉住参考序列（种子 0 的前三个数）。
- **生成**：`generate_history(seed, 步数)`（第 390 行）先按种子定起点（只做过 mkfs / 第一个文件之后，各半），再逐步抽操作。一步操作只带「做什么 + 选择子」（`HistoryOperation`，第 190 行）：内容长度（0、[1, 载荷容量 − 2]、载荷容量 − 1、载荷容量、载荷容量 + 1、数据单元整宽 32768）与填充种子；回退目标（根环里按新到旧第几条 / 根环之外）；抬 F 的目标（现行 F 之上第几格，取模之后最大到现行 txg + 2，上限之上的目标一定取得到）。落到哪条根、F 抬到多少，执行时按那一刻的盘面现解——删掉前面几步之后后面的操作照样有意义，收缩靠这一条。各类操作的比重按生成时对会话的乐观估计调（`operation_weights`，第 266 行），估计错了执行时记「前提不满足」、入口不调。同一个种子、步数只截尾，前面几步不变。
- **操作只调今天的公开入口**：`transaction::publish_first_file` / `publish_overwrite` / `publish_without_units`、关掉会话再 `mount::mount_writable`、关掉会话再 `mount::mount_rollback`（影子账开着）、`mount::raise_rollback_floor`（影子账开着）、关掉会话再冷启动 `recovery::recover`（看 journal）。起点「第一个文件之后」走 `acquire_instance` → `warm_up` → `publish_first_file`（与 `tests/common` 的 `build_pool` 同一条路）。
- **执行**：`execute_history_observing(历史, 录制流, 观察者)`（第 1495 行）。盘是两块 4 GiB 稀疏内存盘外面包录制器（`HistoryDevice`，第 47 行）。起点之后与每一步之后对镜像跑 `singlefs_checker::walk::check_pool_image`；这一步录制流一步都没多（一个写都没发）时镜像逐字节不变，沿用上一次结论、不重跑，单独计数（第 1540 行）。整段历史包在 `catch_unwind` 里，panic 钩子只在本线程捕获时记位置与消息、不打印，别的线程照旧（第 651、672 行）。第一次失败就停。
- **失败的分类**：panic 或 checker 违例 → 先对「已知红」清单（`KNOWN_RED_FORMS`，第 746 行），对上就记「已知红」、这段历史到此为止；对不上是新发现，按签名归类（panic 按位置，违例按判红的不变量集合）。
- **收缩**：`shrink_operations`（第 1741 行）按块删（块长从一半起减半到 1），删不动再把每一步往简单的方向换（内容按 0 字节 < 1 字节 < 别的排，只换成排在前面的；回退目标与抬 F 目标的选择子只换成比现在小的，至多试 32 个，执行时取模，大选择子就收成等价的小数）；删法按窗口并行试、取块号最小的还失败的那个，与一块一块按次序试逐项相同（单测钉住：窗口 1 与窗口 7 收到同一个结果）。签名不变的删法才留下。
- **一批种子**：`run_history_campaign(第一个种子, 种子数, 步数, 线程数, 收不收缩)`（第 1985 行）。每段历史各用各的盘与录制流，分给多个线程跑，跑完按种子排好再汇总，结论与线程数无关。快档不收缩（只报种子与失败在哪一步：debug 下收缩一类量到过两分多钟）；大档每类都收缩；另有「收缩一个种子」的 `#[ignore]` 用例。

第 3–5 件怎么接（都不改生成器）：
- **崩溃注入**：`execute_history_observing` 的录制流由调用方给，给一条 `SharedStream::retaining_contents()` 就留下每次写的字节；截断点从流里抽，拿 `MemoryPool::apply` 施加前缀、或拿 `crash.rs` 的 `CrashImage` 按段取持久子集，恢复之后跑 checker。
- **故障注入**：今天盘只有一种（`HistoryDevice = RecordingBlockDevice<SparseBlockDevice>`），按编码纪律「只有一个实现的 trait 不抽」没做成泛型；第 4 件在录制器与内存盘之间加一层故障包装时，把 `HistoryDevice` 与 `HistoryPool::start`、`HistoryPool::image` 三处换成泛型（那时有两个真实现）。
- **坏盘输入**：观察者每一步拿到那一刻的整份镜像（`StepObservation::image`），就是一份合法镜像；拷一份去翻位、清扇区、截尾，再喂给恢复、挂载与 checker。

## 三、「已知红」清单

**登记了 1 条**（`history.rs` 第 746 行），指到增补 2 收口表第 ② 行（`.claude/kb/milestone/02-second-txn.md` 第 311 行）：

> 根环转过一圈之后（按 checker 的读法，盘上最新根的 txg ≥ R × S = 24）I-3.1（已分配统计对得上） 红、记账的已分配大于遍历全部有效根得到的、别的不变量都不红、没有 panic；不限哪一步操作之后（转圈可以跨几次挂载）

- 判定只用 checker 自己的读法读盘（`singlefs_checker::image::chosen_superblocks` / `valid_roots`），不经实现的择根；「记账多于遍历」从 I-3.1 的违例文字里读两个数（`allocated_and_walked_bytes`，第 711 行），文字对不上就不认、算新发现。
- 形态比收口表那句「一次挂载转过一整圈根环」宽：攻方腿的日志（`research/prompts/m2-wave2-code-r1-opus-model/overlap-pinpoint.log` 第 22 行）在「每次挂载覆盖写 2 次」、第 6 次挂载就红，转圈跨了几次挂载；随机历史里也是跨挂载转圈的居多。宽到什么程度该不该收窄，见第九节问题 2。
- 复现钉在 `turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form`：第一个文件之后可写挂载一次，同一次挂载里连着覆盖写，**红在 txg 26 那一步，不是 txg 24**：txg 24、25、26 依次盖掉第 0 代根与两条暖机根，盖掉最后一条（txg 2）之后再没有有效根引用 mkfs 的第 0 版树表单元（1 槽，txg 3 释放、没回收），记账比遍历多 16384 字节。用例钉住这三个数（第 21 步、txg 26、差 16384）；这一条修好之后它会红，那时删清单这一条、用例改成「跑完」。

**核过、不登记的三行**：

| 行 | 核了什么 | 为什么不登记 |
|---|---|---|
| 第 38 行（第 370 次可写挂载越界 panic） | `crates/singlefs-core/src/mount.rs` 第 888 行 `if rows_in_version + rows_to_write + 1 > records_per_page {` 在取号（第 993 行 `let instance = acquire_expected_instance(…)`）之前，返回 `MountError::InstanceTableRowsExceedOnePageSecondPageUnsupported`（第 109 行） | 今天是 `Err`，第 1 件里算合法结局，不是失败；而且随机历史走不到：快档一段历史里最多挂载成功 9 次（第六节计数），根环转圈的已知红先到 |
| 第 39 行（812 条上限附近误拒） | 发布路径的准入 `transaction.rs` 第 1326 行 `if records_after_this_publish > allocation_node_capacity {` 返回 `Err` | 拒绝是 `Err`，第 1 件只判 panic 与违例；「该不该拒」是第 2 件模型的事。快档与大档都没走到这面墙（计数里没有 `AllocationRecordsExceedOneNode`） |
| 第 40 行（超级块槽写失败之后分配器照样退回） | 要一次写报错才走得到：`crash.rs` 第 115–126 行 `SparseBlockDevice` 的写与屏障恒 `Ok(())` | 第 1 件不注入故障，走不到；归第 4 件 |

推翻条件：快档或大档里出现「I-3.1 红、记账多于遍历、别的不红、根环已转圈」而机理不是「被盖掉的根独占的、已释放没回收的槽」——那说明这条形态宽得盖住了别的问题。

## 四、新发现（清单外，1 类；只报告，没改 `singlefs-core` / `singlefs-checker`）

**签名**：`CheckerViolations { invariants: ["I-3.1"] }`，发生在 `RaiseRollbackFloor` 那一步之后，根环没转圈，记账的已分配大于遍历全部有效根得到的。

**快档里的种子**：[0, 96) 里只有种子 80（第 21 步）。大档（release、种子 [0, 3000)、每段 40 步，打了第九节补丁的副本上跑，它把这一类记成「已知红第 1 条」）：30 段以它收尾，前几个种子 80、174、254、292、435、556、616、659（完整输出见第八节路径）；这 30 段每一段在失败之前都至少成功回退过一次（草稿里的一次性用例逐段数过，没有例外）。

**最短复现**（`shrink_one_failing_seed_from_the_environment` 对种子 80 收出来 11 步，起点「第一个文件之后」；回退目标与抬 F 目标的选择子收缩时已换成等价的小数：第 3 步根环新到旧第 0 条、第 4 步第 3 条、第 10 步 F 之上第 8 格；第九节补丁里那条复现用例逐项照它写）：

| 步 | 操作 | 结局（txg） |
|---|---|---|
| 0 | 可写挂载 | 实例 2，写行 4、暖机 5 |
| 1–2 | 覆盖写两次（0 字节） | 6、7 |
| 3 | 回退到根环最新那条 (2, 7) | 实例 3，8–10 |
| 4 | 再回退到 (2, 7)（新到旧第 3 条） | 实例 4，11–13；实例 3 的三条根被抛弃 |
| 5 | 覆盖写 | 14 |
| 6 | 可写挂载 | 实例 5，15、16 |
| 7–9 | 覆盖写三次 | 17–19 |
| 10 | 抬 F 到 8 | 空发布 20–22，回收 26 个落点 → I-3.1 红 |

违例原文：`I-3.1：盘 0：记账的已分配 Some(1441792)，遍历全部有效根得到 1343488`，差 98304 = 6 × 16384。

**机理**（草稿里逐步读根环核过，`explore/` 里的一次性用例，读法用 checker 的 `valid_roots`）：F = 8 那个 txg 上的根属于被抛弃的实例 3；(2, 7) 的 txg 7 < 8，出了 checker 取并集的候选集；它的实例表（2 槽）与四个固定点单元（各 1 槽）在第二次回退之后的写行（txg 11）才被换下，释放代 11 > F = 8，回收门槛 max(F, 环里最旧有效根) = 8 回收不到它们，记账照样算已分配，而候选集里已经没有一条根引用它们。上限算出来是 min(每块盘最新有效根, 第 4 新的非空有效根) = 14，F = 8 在上限之内，是合法的抬 F。

它像是收口表第 ② 行那一族（「checker 候选集下界用哪个 F」「I-3.1 读法甲与分配器的『占着』分道」），但触发不是根环转圈，而是 F 落进回退留下的空档：被 F 挡出候选集的根，它独占的槽的释放代可以远高于它自己的 txg（中间隔着一整条被抛弃的时间线）。归不归第 ② 行、要不要先修，交主 agent。

空档核过：同一段复现只换抬 F 的目标，F = 7 跑完、F = 8、9、10 红（已知红第 1 条，即这一类）、F = 11、12 跑完——红的正好是被抛弃的实例 3 那三个 txg；F = 11 时回收从 26 个落点变成 31 个，多出的 5 个与 (2, 7) 那 6 槽的落点数（实例表 1 个落点跨 2 槽、四个固定点单元各 1 个）对得上（没逐个核槽号）。

推翻条件：在一段没有成功回退过的历史上撞到同一个签名；或者 F 落在空档之外（上面那段里 F = 7、11）也红。

## 五、判别力：第 41、121 行两条变异，与追加的第 129、130 行

`crates/mutations.tsv` 末尾追加两行（原文与替换文逐字照第 41、121 行，点名的测试都是快档那条 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，cargo test 参数 `-p singlefs-harness --test second_transaction_supplement_three_random_history -- random_histories_fast_tier`）：

- 第 129 行，变异名：`增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）`
- 第 130 行，变异名：`增补 3 第 1 件：随机历史快档判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）`

追加之后原文各在 `crates/singlefs-core/src/allocator.rs` 里命中 1 次（草稿里用门禁 59 号同一个读法核过）；两行的草稿在 `/tmp/claude-1000/m2-supp3-item1/mutations-append.tsv`，与仓里末两行逐字节相同。

**在打了第九节补丁的副本上**（基线：快档绿、新发现 0，26.2 秒；日志 `proposal/round4-whole-binary.log`）：

| 变异 | 快档 | 第一个判红的种子 | 判红的签名（种子） | 用时 |
|---|---|---|---|---|
| 第 41 行（复用时追加记录而不改写） | 红 | 3 | `[I-1.1, I-3.9, I-5.4]`（3、7、15、20、26、27、38、46、51、54、56、57、62、66、68、69、94）；panic `crates/singlefs-core/src/allocator.rs:529`（71） | 20.3 秒 |
| 第 121 行（复用时新记录罩住的已回收记录不删） | 红 | 54 | `[I-5.4]`（54、69） | 25.0 秒 |

日志：`proposal-mutant-41/round4-fast-tier.log`、`proposal-mutant-121/round4-fast-tier.log`。两条都在写死的种子数（96）之内判红。第 121 行那条只有两个种子判得出，余量薄：快档种子数或生成器比重再改，要重量一遍它还红不红。

**在主工作区同一份代码上（不打补丁）**：快档的基线红集就是快档自己（种子 80 的新发现），门禁 59 号只看「点名的测试 FAILED」，第 129、130 行在今天的代码上**恒红、什么都没证明**，要等第九节的问题定了、快档在基线上转绿才有效。看签名的话，变异仍分得出来：第 41 行多出 `[I-1.1, I-3.9, I-5.4]`（第一个种子 3）与 panic（种子 71）两类，第 121 行多出 `[I-5.4]`（54、69）一类，基线只有 `[I-3.1]`（80）一类（日志 `mutant-41/round4-fast-tier.log`、`mutant-121/round4-fast-tier.log`）。

## 六、快档：规模、耗时、计数

- 写死：种子 [0, 96)、每段 30 步（测试文件第 16–18 行 `FAST_TIER_FIRST_SEED` / `FAST_TIER_SEEDS` / `FAST_TIER_OPERATIONS_PER_HISTORY`），线程数取 min(本机核数, 16)，结论与线程数无关。
- 耗时：`check.sh` 里这条 25.7 秒（14:44Z 那次，下面原样）与 25.4 秒（14:53Z 最终代码那次，计数逐行相同）；两次跑时机器上都没有别的 cargo、门禁、性能测量进程。debug 下 checker 一次约 135 毫秒、是大头（草稿 `timing/` 里量的：一次可写挂载 0.12 秒、一次覆盖写 5 毫秒、一次 checker 136 毫秒）；一个写都没发的步不重跑 checker（1421 步）。机器满载时会慢，目标一分钟以内留了一倍多余量。
- 各条路径都跑到了：快档末尾有一组断言（测试文件 `assert_every_path_was_exercised`）：七类操作各至少一次 Ok；见过 `RollbackFloorAboveCeiling`、`RollbackTargetNotInRing`、`RollbackToVersionWithoutFileUnsupported`、`ContentExceedsDataUnit`、`FirstFileVersionNotRightAfterTheSecondWarmUp` 与 `RollbackTargetNotACandidate`；抬 F 回收到过落点；复用改写过已释放的记录、其中有跨度变了的、也删过被罩住的；六种内容长度都进过入口；冷启动读回过文件；checker 判绿过 I-3.1 与 I-5.4；至少一段历史转过根环。薄的几项：`RollbackTargetNotACandidate（txg 低于生效的回退下界 F）` 1 次、已释放记录被删 4 条、零单元发布 Ok 7 次。

主工作区 `check.sh` 里快档打出的计数（原样，只删了每条不变量判绿几次的 29 行，那 29 行在草稿 `check-sh.log` 第 331–404 行之间）：

```
── 随机历史快档 ──
种子 [0, 96)，每段 30 步
历史 96 段：跑完 45、以已知红收尾 {0: 50}、新发现 1；根环转过一圈的 62 段；最高 txg 37
  操作 PublishFirstFile：Ok 24、Err 62、前提不满足没调 56
  操作 PublishOverwrite：Ok 655、Err 104、前提不满足没调 456
  操作 PublishWithoutUnits：Ok 7、Err 0、前提不满足没调 64
  操作 CloseAndMountWritable：Ok 321、Err 92、前提不满足没调 0
  操作 CloseAndMountRollback：Ok 112、Err 192、前提不满足没调 0
  操作 RaiseRollbackFloor：Ok 42、Err 156、前提不满足没调 142
  操作 ColdStartRecover：Ok 97、Err 0、前提不满足没调 0
  内容长度 0 字节：入口被调 87 次
  内容长度 [1, 载荷容量 − 2]：入口被调 502 次
  内容长度 数据单元整宽 32768（装不下）：入口被调 50 次
  内容长度 载荷容量：入口被调 70 次
  内容长度 载荷容量 + 1（装不下）：入口被调 66 次
  内容长度 载荷容量 − 1：入口被调 70 次
  Err 成员 MountError::InstanceRowsOnVersionWithoutFileUnsupported：92 次
  Err 成员 MountError::RaiseNeedsRewrittenInstanceTableUnitInCurrentVersion：34 次
  Err 成员 MountError::RollbackFloorAboveCeiling：122 次
  Err 成员 MountError::RollbackTargetNotACandidate（txg 低于生效的回退下界 F）：1 次
  Err 成员 MountError::RollbackTargetNotACandidate（实例表里那个实例的行 T 更小：这是被抛弃时间线的根）：5 次
  Err 成员 MountError::RollbackTargetNotInRing：61 次
  Err 成员 MountError::RollbackToVersionWithoutFileUnsupported：125 次
  Err 成员 PublishError::ContentExceedsDataUnit：108 次
  Err 成员 PublishError::FirstFileVersionNotRightAfterTheSecondWarmUp：58 次
  前提不满足（没有可写会话）：660 次
  前提不满足（现行版本带文件）：34 次
  前提不满足（现行版本树表 0 条）：24 次
  冷启动结局 FileRead：78 次
  冷启动结局 NoFile：19 次
  一次挂载里最多写出 23 条根；一段历史里最多挂载成功 9 次、最多试挂载 12 次
  抬 F 回收到落点 31 次、共回收 852 个落点；复用已释放的记录 234 条（其中跨度变了 52 条）；已释放的记录被删 4 条
  checker 跑了 1257 次；一个写都没发、沿用上一次结论的 1421 步
已知红第 0 条（增补 2 收口表第 ② 行（一次挂载转过一整圈根环时 checker 在合法状态上判 I-3.1 红））：50 段；前几个种子 [(0, Operation(22)), (1, Operation(24)), (2, Operation(27)), (4, Operation(25)), (11, Operation(21)), (12, Operation(18)), (13, Operation(24)), (16, Operation(18))]
新发现 CheckerViolations { invariants: ["I-3.1"] }：第一个种子 80（同签名的种子 [80]），在 Operation(21)（Some(RaiseRollbackFloor)）之后
  I-3.1：盘 0：记账的已分配 Some(1441792)，遍历全部有效根得到 1343488
  没收缩（这一档只报种子）：SINGLEFS_RANDOM_HISTORY_SHRINK_SEED=80 跑「收缩一个种子」那条 #[ignore] 用例，或跑大档
用时 25.7 秒
```

## 七、check.sh 与登记给我的门禁阶段

`nice -n 19 bash .claude/scripts/check.sh`（主工作区，最终代码，14:53:37Z 起、14:54:20Z 完；格式、clippy、构建都过，红在 cargo test 的快档，原因是第四节的新发现；14:44Z 那一次结果相同）末尾原样：

```

failures:
    random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation

test result: FAILED. 2 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 25.39s

error: test failed, to rerun pass `-p singlefs-harness --test second_transaction_supplement_three_random_history`
  ✗ 单测失败
     → 怎么办： 上面列出了失败的用例名。单跑一个看细节：
                cargo test --all <用例名> -- --nocapture
                改代码还是改断言，先想清楚是哪一种——直接改断言等于把测试关掉。
exit 1
```

cargo test 在第一个红的测试二进制上停，后面的没跑；另跑了一次 `nice -n 19 cargo test --all --no-fail-fast`（草稿 `cargo-test-no-fail-fast.log`；在收缩的「只往简单的方向换」那处改动之前，其余代码与最终相同）看其余的：35 个 `test result: ok`，唯一红的是这一个，末尾原样：

```
error: 1 target failed:
    `-p singlefs-harness --test second_transaction_supplement_three_random_history`
exit 101
```

登记给实现员的阶段（`awk … stage-owners.tsv` 列出的只有一个）：`nice -n 19 bash .claude/gate.d/53-format-const-placeholders.sh`，末行原样与退出码：

```
  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））
exit 0
```

打了第九节补丁的副本上整个测试二进制（快档、两条已知红复现、同种子逐字节复现）：4 条通过、2 条 `#[ignore]`，快档 26.2 秒（`proposal/round4-whole-binary.log`）；最终代码再打补丁重跑一次，快档 25.9 秒、照样通过（`proposal/final-whole-binary.log`）。那个副本上 clippy（与 check.sh 同一组 -D）与 rustfmt --check 都过。第十一节的变异证明用的是主工作区的代码（没打补丁）。

## 八、大档与草稿产物

大档（`random_histories_large_tier_seeds_and_length_from_the_environment`，`#[ignore]`；`SINGLEFS_RANDOM_HISTORY_SEEDS` / `_OPERATIONS` / `_FIRST_SEED` / `_THREADS`，默认 1000 段 × 60 步、全部核；每类新发现都收缩）。跑过的几次，都在草稿副本上、release、16 线程：

| 什么代码 | 规模 | 结果 | 用时 | 日志 |
|---|---|---|---|---|
| 最终代码 + 第九节补丁 | 种子 [0, 3000) × 40 步 | 跑完 1041、已知红第 0 条 1929、第 1 条 30、新发现 0；一次挂载里最多 29 条根；最多挂载成功 10 次；抬 F 回收到落点 1315 次；复用已释放记录 10102 条（跨度变了 2750）；checker 41553 次 | 105.8 秒 | `proposal/round4-large-tier-3000x40.log` |
| 生成器改比重之前 + 同一补丁 | [0, 3000) × 40 | 新发现 0，第 1 条 31 段 | 135.3 秒 | `proposal/large-tier-3000x40.log` |
| 生成器改比重之前、不打补丁 | [0, 1000) × 40 | 新发现只有第四节那一类（13 段） | 42.7 秒 | `explore/run-1000x40.log` |
| 第一版生成器、第 41 / 121 行变异 | [0, 200) × 30 | 两条都红（第 41 行：`[I-1.1, I-3.9, I-5.4]` 首个种子 0、panic `allocator.rs:529` 首个种子 2；第 121 行：`[I-5.4]` 首个种子 15） | 46.7 / 47.8 秒 | `mutant-41/run-200x30.log`、`mutant-121/run-200x30.log` |

大档没有 panic、没有第四节之外的新发现；根环转圈的已知红收尾了大约三分之二的历史，比 24 个 txg 更深的状态几乎走不到——这一条修好之前，随机历史罩不到「根环转过之后」的那些路径（例如环里最旧有效根接管回收门槛）。

草稿都在 `/tmp/claude-1000/m2-supp3-item1/`，没入库：定义里我不写 `research/`。主 agent 要留的话，要拷的是：`report.md`、`provisional-known-red-second-form.diff`、`mutations-append.tsv`、`check-sh.log`、`cargo-test-no-fail-fast.log`、`proof/proofs.log`、上表几份日志、`progress.md`。门禁 69 号在 `crates/mutations.tsv` 改了之后要 `research/results/` 里有一份不比它旧的产物，这一条我没法满足，交回。

## 九、停下交主 agent 的设计问题

**问题 1（阻塞）：第四节的新发现怎么处置。** 今天快档在主工作区上红、`check.sh` 红、第 129、130 行在门禁 59 号里恒红不证明任何东西，都卡在这一条。两条路：

- 甲：判它不挡增补 3，进增补 2 收口表立一行（像是第 ② 行那一族，归不归第 ② 行由你判），再把它登进「已知红」清单。补丁已备好：`/tmp/claude-1000/m2-supp3-item1/provisional-known-red-second-form.diff`（sha256 `03d4ca63386c5c85d43f78443a62f7a89a7d95d37eed816769d7df345a132605`，119 行，对今天主工作区的两个文件 `patch -p1 --dry-run` 过）。它加的是：`history.rs` 里一个判定函数（抬 F 那一步之后、根环没转圈、只有 I-3.1 红且记账多于遍历）、`KNOWN_RED_FORMS` 第 1 条（`closeout_table_row` 现在写的是「增补 2 收口表第 ？ 行（待主 agent 立行…）」，立了行要把行号填进去）、测试文件里一条复现用例 `raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form`（第四节那 11 步，钉住 F = 8、回收 26、差 6 × 16384）。打上之后的数据在第五、七、八节。
- 乙：判它挡着，先修（改 `singlefs-core` 的回收门槛或 checker 的候选集下界，要走三方），修好之前快档红着。

判定函数只看「抬 F 之后、没转圈、I-3.1 记账多于遍历」，没看「F 是否落在回退的空档里」：从 `FailureObservation` 里拿不到回退历史。要收窄就得给观察加字段，你定了形态我再改。

**问题 2：已知红第 0 条的形态宽度。** 我登记的是「根环转过一圈之后（盘上最新根 txg ≥ 24）、只有 I-3.1 红且记账多于遍历、任一操作之后」，比收口表第 ② 行那句「一次挂载转过一整圈根环」宽（证据见第三节）。宽了会把「转圈之后才出现的别的 I-3.1 多算」一并当已知红吞掉；要收窄（例如只认「最后一条引用 mkfs 第 0 版树表的根被盖」那一步），你定。

**问题 3：两处我按文档注释取的读法，没有条款明写，列出来请你看一眼。**
- `publish_without_units` 只在现行版本树表 0 条时调（它的文档注释与 D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）；带文件的版本上记「前提不满足」，不调。
- `raise_rollback_floor` 的目标只取「现行 F 及以上」，不往下抬（名字是「抬」，文档说「抬回退下界 F 到 new_floor」）；往下的目标这个入口会不会接、接了算不算合法，没有条款，没试。F 回落本身在收口表第 ② 行里是打回重议的。

**问题 4（不阻塞，性能）**：快档的时间大半在 debug 下的 checker（CRC 逐字节查表，一次 136 毫秒）。根 `Cargo.toml` 给 `singlefs-checker` / `singlefs-core` 开 `[profile.dev.package.…] opt-level` 能把快档与所有 debug 用例一起提速，根 `Cargo.toml` 不在我的写范围，没动。

## 十、没做什么

- 没走三方对抗（第 2 步）、没跑 checker 那一步的层 0 / QEMU / herd7 / crates 变异表整表复跑（归 `crash-verifier`）；没提交。门禁 56 号要这次改动里的 `crates/singlefs-harness/src/history.rs`、`crash.rs`、`lib.rs` 在同一次改动带来的 `research/prompts/*-main-verification.md` 里被点名，这一步归主 agent。
- 没做第 2 件（理想模型、对拍、`# gate-covers: 模型对拍`）：冷启动读回的内容对不对、`Err` 该不该出现（第 39 行的误拒）都没判。
- 没新加层 0 流、没做崩溃点重放；随机历史不枚举崩溃状态。
- 没改 `crates/singlefs-core`、`crates/singlefs-checker`；第四节的新发现没修。
- 没自己把第四节的形态登进「已知红」清单（问题 1）；补丁只在草稿副本上打过。
- 快档不收缩（只报种子）；收缩靠 `shrink_one_failing_seed_from_the_environment` 或大档。大档里收缩在 release 下每类几秒到十几秒；debug 下没量完整的（第一版在变异下顺序收缩三类用了 398 秒，改成窗口并行、快档不收缩之后没再量 debug 的收缩）。
- 故障注入、坏盘输入没做；`HistoryDevice` 没做成泛型（第二节）。
- `FailureObservation` 里没有回退历史、F 的值，已知红判定因此只能按「哪一步之后、转没转圈、哪条不变量、记账方向」分；第 1 条要不要按「F 落在空档里」收窄，等问题 1。
- 生成器比重是按一次次跑出来的计数手调的，没有论证「够」：零单元发布 Ok 7 次、`RollbackTargetNotACandidate（txg 低于生效的回退下界 F）` 1 次、已释放记录被删 4 条是薄的；第 121 行的变异在快档里只有 2 个种子判得出。
- 在草稿副本里改坏再还原都是从 `proof/pristine/` 拷回并 `touch`；主工作区里没改坏过任何东西。
- 本机 `ps` 看到的：开工时另一会话的 `gate.sh --staged` 在跑（草稿 `progress-ps-start.txt`）；跑 `check.sh` 时没有别的 cargo / 门禁 / 性能测量进程；中途没等过锁。另一会话改 naming-lint 期间（主 agent 两条消息之间，第一条在 14:12Z 之前、第二条约 14:23Z 收到，时刻是我事后看钟估的）我按消息没碰那五个文件，`crates/mutations.tsv` 两行在它提交（`00c9d4f`）之后才追加。

## 十一、每条新测试的「改坏哪一行 → 哪条断言红」

做法：草稿副本 `proof/repo`（自己的 target），先跑不改动的基线，再逐条改坏、跑那条测试所在的整个测试二进制、从 `proof/pristine/` 拷回并 `touch`。最后一轮在最终代码上全部重跑（`history.rs` sha256 `c6937bbf…`、测试文件 `f6374d39…`），日志 `proof/proofs.log`「第四轮」。行号是主工作区今天的行号。

**基线红集**：lib 测试二进制全绿；`second_transaction_supplement_three_random_history` 里快档红（第四节的新发现），其余绿。下面「同时红」不算基线红集里的快档。

| 新测试 | 改坏哪一行（原文 → 替换） | 哪条断言红 | 同时红 |
|---|---|---|---|
| `seeded_random_source_reproduces_the_splitmix64_reference_sequence` | `history.rs` 第 75 行混合常数 `0xbf58_476d_1ce4_e5b9` → `…e5b8` | 第一个 `assert_eq!(source.next_word(), 0xe220_a839_7b1d_cdaf)`：left 14692572579270306168 / right 16294208416658607535 | 无 |
| `the_same_seed_generates_the_same_history_and_another_seed_does_not` | 第 391 行 `SeededRandomSource::from_seed(seed.0)` → `from_seed(0)` | `assert_ne!` 「只比操作序列…」 | 无 |
| `shrinking_keeps_only_the_steps_the_failure_needs` | 第 1778 行 `verdicts.iter().position(` → `rposition(` | 「按窗口并行试与一块一块按次序试，收到同一个结果」：left `[RaiseRollbackFloor(0), ColdStartRecover]` / right `[PublishWithoutUnits, ColdStartRecover]` | 无 |
| `allocated_and_walked_bytes_are_read_from_the_checker_detail` | 第 714 行锚 `"遍历全部有效根得到 "` → `"遍历全部有效根得 "` | 第一个 `assert_eq!`：left None / right Some((3031040, 2850816)) | 无 |
| `turning_the_root_ring_with_overwrites_in_one_mount_ends_in_the_first_known_red_form` | 第 737 行 `allocated > walked` → `allocated < walked` | `let … else { panic!("要以已知红收尾…") }`：收尾成了 `NewFinding` | 无 |
| 同上 | 第 1662 行 `newest >= slot_count` → `newest > slot_count + 2` | 同上 | 无 |
| `the_same_seed_runs_to_the_same_outcomes_and_the_same_bytes_twice` | 第 1438 行写入时间加上系统时钟的纳秒 | 「每一步之后的镜像逐字节相同」（结局与计数照样相等，只有字节不同） | 无 |
| 快档 `random_histories_fast_tier_…` | `crates/mutations.tsv` 第 41、121 行两条 | 「「已知红」清单外的失败」，种子见第五节 | 在主工作区上它本身在基线红集里；证明是在打了补丁的副本上做的 |
| 快档的「路径跑到了」那组断言 | 第 284 行 `(HistoryOperationKind::RaiseRollbackFloor, 16)` → `0`（打了补丁的副本上） | 「RaiseRollbackFloor 一次 Ok 都没有」 | 无（两条已知红复现与同种子用例照绿） |
| 大档（`#[ignore]`） | 第 41、121 行变异，release，种子 [0, 200) × 30 步（第一版生成器） | 「「已知红」清单外的失败」 | — |
| 改过的 `crash.rs`（`read_into`，第 55 行） | 区间终点少一个扇区 | 已有用例 `sparse_device_reads_zero_for_holes_and_flip_byte_only_touches_one_sector`（`crash.rs` 第 951 行）在第 955 行：left 全 0 / right 全 7 | 无 |

两条单测第一版改坏之后**没红**，改了测试再证：
- `the_same_seed_generates…` 第一版拿整个 `GeneratedHistory` 比 `assert_ne!`，历史里带着种子，种子不同就恒不等，「生成不看种子」照绿；改成只比操作序列。
- `shrinking_keeps_only_the_steps_the_failure_needs` 第一版的人造判据（有抬 F 与冷启动就算失败）下，取块号最大的还失败的那个与取最小的收到同一个结果，`rposition` 照绿；改成「冷启动之前有抬 F 或零单元发布」，两条路收到不同的结果。

`shrink_one_failing_seed_from_the_environment`（`#[ignore]`）是工具不是判据：对种子 80（release）收出第四节那 11 步；对撞不到新发现的种子 panic「没有撞到新发现」。

## 十二、打补丁之后（2026-09-18 主 agent 定甲之后，约 14:57Z 收到）

**改了什么**（都用 Edit 落进主工作区，没用 `patch` 命令）：
- `crates/singlefs-harness/src/history.rs`：第九节那份补丁的判定函数 `raise_after_rollback_leaves_allocated_statistic_above_walked` 与 `KNOWN_RED_FORMS` 第 1 条，`closeout_table_row` 填成「增补 2 收口表第 43 行」；清单文档注释加一句「第 0 条的宽度（转圈跨几次挂载也算）2026-09-18 主 agent 定案保留，记在收口表第 ② 行」，第 0 条本身不动。
- 同一文件，两处生成器前提写出处（没另开讨论）：
  - `FloorTargetChoice::steps_above_current_floor` 的注释：目标只取现行 F 及以上，出处 `crates/singlefs-core/src/mount.rs` 第 590 行（`raise_rollback_floor` 的文档注释「抬回退下界 F 到 `new_floor`（D16（发布语义） 已定项 1）」）、`.claude/kb/decisions/16-发布语义.md` 第 107 行（已定项 1）；并写明以后要测「前提之外调它应当返回 `Err`，不许 panic」。
  - `MissingPrecondition::CurrentVersionWithFile` 的注释：`publish_without_units` 只在树表 0 条的一版上调，出处 `crates/singlefs-core/src/transaction.rs` 第 506 行（「零单元发布（D16（发布语义） 已定项 9「树表 0 条 ⇒ 零单元」）」）、`.claude/kb/decisions/16-发布语义.md` 第 192 行（已定项 9）；同样写明以后要测的那一句。
  - 两处执行时的判定（`apply_publish_without_units`、`apply_raise_rollback_floor`）旁各加一行注释指到上面两段。
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`：补丁里那条复现用例 `raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form`（注释里的行号改成第 43 行），与它要的 `use`。
- 没动 `opt-level`、kb、别的文件。`crates/mutations.tsv` 这一步没改（第 129、130 行是之前追加的）。

**打完之后的 sha256**：

```
5e715bbe3b8d4a0b2a3e73d1485a35171c4f97e96b2adf889fbdb5f50b0d0d46  crates/singlefs-harness/src/history.rs
4fa5f0f5161bcdbd7314395b9caffb9e87e2651c90ddabe3fb708ab5a979f4db  crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
f2612b1c413c3b46fc28a6b19a47433b89657da45ebf5d0444e9790367d7cbc6  crates/singlefs-harness/src/crash.rs
133ce0338e86df7e74367c76ce51cccef0bd7a418ed846d6b844f2a08374e088  crates/singlefs-harness/src/lib.rs
93d12d4643208a21b25bbf1ea49642dacf8e87c27c28416ae11e16e8d1d04780  crates/mutations.tsv
```

`git diff --stat -- crates litmus` 原样（新建的两个文件没进 git，不在 diff 里；与打补丁之前相同）：

```
 crates/mutations.tsv                 |  2 ++
 crates/singlefs-harness/src/crash.rs | 25 ++++++++++++++++---------
 crates/singlefs-harness/src/lib.rs   |  6 ++++++
 3 files changed, 24 insertions(+), 9 deletions(-)
```

**`nice -n 19 bash .claude/scripts/check.sh`**（主工作区，15:00:23Z 起、15:01:17Z 完；开跑前 `ps` 没有别的 cargo / 门禁 / 性能测量进程）：绿。格式、clippy、构建、单测四段都过；快档 25.7 秒，历史 96 段：跑完 45、已知红第 0 条 50 段、第 1 条 1 段（种子 80）、新发现 0。末尾原样：

```
   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit 0
```

**第 129、130 行按门禁 59 号的做法复跑**（草稿 `gate59-rows.py`：照抄 59 号的锚点预扫与复跑逻辑，把打了补丁的主工作区拷进临时目录、target 用草稿里自己的 `gate59-target`，只复跑这两行；15:01:17Z 起、15:02:04Z 完）。输出原样：

```
  ✓ 锚点预扫：125 条变异，原文都恰好命中一次
  ✓ 第 129 行 增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了
  ✓ 第 130 行 增补 3 第 1 件：随机历史快档判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了
exit 0
```

- **整表锚点预扫**：125 条变异（`crates/mutations.tsv` 130 行里去掉 5 行注释），原文都恰好命中一次。
- **第 129 行**：快档判红，余量足。第一个判红的种子是 3；96 段里 18 段是新发现：`[I-1.1, I-3.9, I-5.4]` 17 段（种子 3、7、15、20、26、27、38、46、51、54、56、57、62、66、68、69、94），`crates/singlefs-core/src/allocator.rs:529` 的 panic 1 段（种子 71）。19.9 秒（日志 `gate59-row-129.log`）。
- **第 130 行**：快档判红，但**余量薄**。只有两个种子判得出：54、69，都是 `[I-5.4]`。24.5 秒（日志 `gate59-row-130.log`）。快档的种子数、生成器比重、每段步数改了之后，要先重量它还红不红。
- 两行里「已知红第 1 条」都照样只收种子 80，没把变异的红吞掉。

**新复现用例会不会红**（草稿副本 `proof/repo`，打了补丁的代码，日志 `proof/proofs.log`「第五轮」；基线：整个测试二进制 4 条通过）：

| 改坏哪一行 | 结果 |
|---|---|
| `history.rs` 清单第 1 条的判定 `observation.operation_kind == Some(HistoryOperationKind::RaiseRollbackFloor)` → `PublishOverwrite` | `raising_the_floor_into_the_gap_left_by_a_rollback_ends_in_the_second_known_red_form` 红在「要以已知红收尾」（收尾成了 `NewFinding`）；同时红：快档（种子 80 成了新发现） |
| 同一个判定去掉 `&& !observation.root_ring_has_turned()` | **不红**，整个测试二进制 4 条照过。是等价变异：清单按次序对，根环转过圈、只有 I-3.1 记账多于遍历的，先被第 0 条接住，走不到第 1 条；这个合取项今天只是写明意图。要删它还是留着，你定（我没动） |

**没做**：三方代码轮、`crash-verifier`（crash.rs 改了要跑的门禁 54 号、59 号整表复跑）归主 agent 派；第十一节那几条测试的代码这一步没动，没重跑它们的变异证明。

## 十三、代码三方第一轮之后（判决 `research/prompts/m2-supp3-item1-code-r1-main-verification.md` 第二节第 1–5、7 条；19:47Z 收到）

### 13.0 结论

- 第 1、3、4、5、7 条做了，各自有会红的用例；变异表第 130 行改点名专门取样点，末尾追加第 139–141 行。
- **第 2 条停下交回**：在单一镜像上判不了（13.2 节），缺的输入写在那里。第 0 条保持原样，没放宽；只加了一个收窄：执行器判出失败的，一概不算已知红。
- `check.sh` 绿（20:16:00Z–20:17:03Z，13.6 节）。第 129、130、139、140、141 行按门禁 59 号的做法复跑都红，整表锚点预扫 136 条都只命中一次（13.7 节）。
- 没改 `singlefs-core`、`singlefs-checker`。

### 13.1 第 1 条：已知红第 1 条收窄（P1）

- `history.rs` 第 930 行 `raise_after_rollback_leaves_allocated_statistic_above_walked` 加了一个合取：`observation.raised_floor_lands_only_on_abandoned_roots == Some(true)`。
- 这个值由第 2030 行 `raised_floor_lands_only_on_abandoned_roots` 算，读抬 F 之前的镜像：F 那个 txg 上的根按最新根的实例表判，全属于被抛弃的实例才为真；没有根、读不出都给 None。
- 执行器只在「抬 F 那一步、Ok、checker 判红」时算它（第 1886–1914 行）。
- 读法用的是实现的 `recovery::readable_roots` / `choose_root` / `instance_table_of_root`，与攻方 P1 相同。checker 解实例表的那一段不对外，没另写一份解析。这一点写在函数的文档注释里，也是下一轮能攻的一格。
- 用例 `an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding`（测试文件第 317 行）：
  - 历史照攻方的 `opus_z2_history`：可写挂载、覆盖写三次、抬 F 到 3，没有回退。今天的代码上这段跑完。
  - 抬 F 之前那份镜像上判定为 `Some(false)`。
  - 配上攻方在 A1 下量到的违例文字，分类是新发现。
  - 另一侧由第 43 行的复现用例钉着：种子 80 那一段必须分类成第 1 条，要 `Some(true)`。
- 变异表第 139 行：去掉那个合取，这条用例红（13.7 节）。
- release 下 A1（回收门槛 ≤ 写成 <）的快档，草稿 `r2/a1/fast-release.log`：第 1 条只剩种子 80 那 1 段，新发现 17（`I-3.1`，第一个种子 7，抬 F 之后）；攻方报告里改之前是第 1 条 18 段、新发现 0。基线快档计数与改之前逐字相同。

### 13.2 第 2 条：第 0 条按多算的量收窄——在镜像上判不了，停下

要判的是：记账多出来的那些槽，都是已释放、没回收、只被已经转出根环的根引用过。缺的输入有两样：

1. **每条已释放记录现在是「defer（仍占着）」还是「已回收、还没复用」。**
   - 盘上两种都写成已释放。`crates/singlefs-core/src/allocator.rs` 里 `PoolAllocator` 的字段 `reclaimed` 注释写着「已回收、还没被复用的落点（盘上那条记录仍写着已释放；复用时那条记录被改写）。只住内存。」
   - 记账树只给每块盘 defer 的总字节数（`records.rs` 的 `STATISTIC_DEFER_QUEUE_BYTES`），不给是哪几条。
   - 这件事只在写这一版的那个进程的分配器里（`DeviceFreeMap` 的位图，公开的只有 `is_free`）。
2. **一条被 F 挡出候选集、还在环里的根独占的那几条已释放记录，算不算多算**，取决于第 1 样。镜像上只看得出它们被谁引用，看不出它们回收了没有。

草稿量了会差多少：`r2/probe` 里一条一次性用例，只在副本，没入库。

- 做法：对每段以第 0 条收尾的历史重建失败那一刻的镜像，按实现的读法（`allocation_records_under_root`，树表 0 条的根取它指着的 mkfs 两个单元）算三个量：
  - 盘 0 的多算槽数 O（取自 I-3.1 的文字）；
  - U：已释放、环里没有一条根引用；
  - N：已释放、只被环里的非候选根引用。
- 按三类数：

| 取样 | 第 0 条收尾 | N 空、O ≤ U（判得了，是第 0 条） | N 非空、O ≤ U（判不了） | O > U（判得了，不是第 0 条） |
|---|---|---|---|---|
| 快档种子 [0, 96) × 30 | 50 | 44 | 4（种子 20、66、68、94） | 2（种子 61、81） |
| 种子 [0, 1000) × 40 | 624 | 506 | 91 | 27 |

- 最后一列是**今天的代码上**就有的：种子 61、81 都是 O = 13、U = 11、N = 2。多出来的正好是只被还在环里的被抛弃根引用的 2 槽，不是转圈留下的。按判决的收窄写法照做，快档在基线上就红。
- 那 2 段是什么、要不要进收口表，交你定；我没核它们的机理（只数了槽）。日志 `r2/probe/form0-96x30.log`、`form0-1000x40.log`。
- 第 2 类在镜像上判不了。要判的话，执行器得把写这一版的进程的分配器状态交给分类（例如每个已释放槽的 `is_free`）。那是实现自己的内存状态，拿它判实现的输出合不合适，交你定。

### 13.3 第 3 条：分配代

- `history.rs` 第 686 行 `records_changed_with_a_generation_outside_the_publish_txgs`：这一次发布之后有、之前没有一模一样的一条的记录，代要等于这次发布的 txg。覆盖新分配、复用改写、这次释放（释放代）三种。
- 第一个文件与覆盖写，拿分配器发布前后的记录比。
- 抬 F 按 `RaisedFloor::publishes` 逐次比盘上的记录；半路报错时，按那几次的 txg 区间比。
- 挂载的写行与暖机不比：挂载交回的东西里没有挂载之前那一版的记录。
- 判出来记成执行器判的失败 `HarnessJudgement::AllocationGenerationIsNotThePublishTxg`，签名是 `HarnessJudgement { judgement: "改写或新增的分配记录的代不是这次发布的 txg" }`。
- 单测 `record_rewritten_by_a_publish_that_keeps_its_old_generation_is_reported`。
- 变异表第 140 行（原文同 N2），点名快档：
  - 门禁 59 号的做法下快档红，新发现 17 段、第一个种子 3。
  - release 下 `r2/n2/fast-release.log` 同样 17 段。

### 13.4 第 4 条：冷启动读回报错（P2）

- `AppliedEffect::Recovered` 加了 `read_back: ColdStartReadBack`（`NoFile` / `FileRead` / `Failed`）。
- 第 733 行 `harness_judgement_of_outcome`：`Failed` 记成 `HarnessJudgement::ColdStartRecoveryFailedOnCheckerGreenImage`。冷启动不写盘，这一步之前那份镜像 checker 判过绿，判红的话历史已经停了。
- 单测 `failed_cold_start_read_back_is_a_harness_judgement_and_the_other_read_backs_are_not`。
- 变异表第 141 行（B6，`unit.rs` 补齐字节从最后一个载荷字节算起），点名快档：门禁 59 号的做法下红，新发现 45 段、第一个种子 0。
- 执行器判的失败一概不进已知红：`only_allocated_statistic_above_walked`（第 913 行）加了 `harness_judgement.is_none()`。同一步 checker 也红时，签名取执行器判的那一种。单测 `harness_judgement_is_never_classified_as_a_known_red_form`。

### 13.5 第 5、7 条：内容长度计数；第 121 行那一类的专门取样点

- **第 5 条**：
  - `HistoryTally::content_lengths_published` 数每种长度发成几次，报告行改成「入口被调 N 次、发成 M 次」。
  - 快档的路径断言加了一组：装得下的四种（0、[1, 载荷容量 − 2]、载荷容量 − 1、载荷容量）各至少发成一次。
  - B1（`transaction.rs` 的 `>` 写成 `>=`）下快档红在「装得下的内容长度「载荷容量」一次都没发成」。debug 与 release 各跑过一次，见 13.8 节。
- **第 7 条**：
  - `GenerationWeights`（第 276 行）把比重做成一组值：`BROAD` 就是原来那组，起点写成 1 : 1，与第一版的「对 2 取余」逐项相同。基线快档的计数与改之前逐字相同，都是 `{0: 50, 1: 1}`、新发现 0。
  - `REUSE_AFTER_RAISING_THE_FLOOR`（第 322 行）的比重：起点 1 : 9 偏向第一个文件之后；带文件时覆盖写 55、抬 F 22、可写挂载 18；少回退与冷启动。
  - 新用例 `reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms`（测试文件第 171 行），种子 [0, 48)、每段 30 步：
    - 先断言清单外的失败一条都没有；
    - 再断言这一路跑到了：罩住别的已释放记录的起点 ≥ 1、复用时跨度变了 ≥ 1、抬 F 回收到落点 ≥ 1。
    - 「罩住」这个数是新加的 `RecordReuse::released_records_covered`，不看罩住的那条删没删，变异下照样成立。所以门禁 59 号看到的红只会是分类判出来的：第 130 行复跑时红在第 186 行那条「清单外的失败」断言。
  - 变异表第 130 行（我自己那一行）的 cargo 参数与点名的测试改成这条。
  - 大档加了两个环境变量 `SINGLEFS_RANDOM_HISTORY_WEIGHTS`（broad / reuse）、`SINGLEFS_RANDOM_HISTORY_SHRINK`（every / none），并把每个新发现的种子打出来，量判出率用。
- **判出率**：release，草稿 `r2/m121/reuse-6000x30.log` 与 `r2/base/reuse-6000x30.log`。
  - 第 121 行的变异下，偏向复用的比重在种子 [0, 6000) × 30 步上 799 个种子判出，占 13.3%，签名 `[I-5.4]` 或 `[I-3.1, I-5.4]`。
  - 按窗口切（一窗 = 连续一段种子，数这一段里判出了几个种子）：

    | 一窗多少个种子 | 窗数 | 一窗至少判出几个 | 判出 0 个的窗 |
    |---|---|---|---|
    | 8 | 750 | 0 | 237 |
    | 12 | 500 | 0 | 83 |
    | 16 | 375 | 0 | 40 |
    | 24 | 250 | 0 | 7 |
    | 32 | 187 | 0 | 2 |
    | **48（用例用这个）** | 125 | **2** | 0 |
    | 64 | 93 | 2 | 0 |
    | 96 | 62 | 5 | 0 |

  - 门禁那一窗 [0, 48) 判出 7 个：种子 8、11、12、20、21、35、46。门禁 59 号的做法下复跑，新发现正是这 7 个。
  - 同比重在基线上 6000 × 30：新发现 0；罩住别的已释放记录的起点 2456 次，快档那组比重下被删只有 4 次。
- **耗时**：`check.sh` 里这条 25.9 秒，快档 33.9 秒，两条在同一个测试二进制里并行跑，那个二进制 33.91 秒跑完。`check.sh` 整个 63 秒（20:16:00Z–20:17:03Z）。只看快档的话它比上一轮的 25.7 秒慢，是两条用例各开 16 个线程抢核。

### 13.6 check.sh

`nice -n 19 bash .claude/scripts/check.sh`，主工作区，20:16:00Z 起、20:17:03Z 完。开跑前 `ps` 没有别的 cargo、门禁、性能测量进程（草稿 `round2-final.ps` 是空的）。末尾原样：

```
   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit 0
```

偏向复用那条用例打出来的计数：草稿 `check-sh-round2.log` 第 350–411 行；快档打出来的计数：第 414–477 行。

### 13.7 门禁 59 号的做法复跑第 129、130、139、140、141 行，与整表锚点预扫

草稿 `gate59-rows.py`：照抄 59 号的预扫与复跑逻辑，把主工作区拷进临时目录、target 用草稿自己的。20:17:03Z 起、20:18:22Z 完。输出原样：

```
  ✓ 锚点预扫：136 条变异，原文都恰好命中一次
  ✓ 第 129 行 增补 3 第 1 件：随机历史快档判出「复用时追加记录而不改写」（第 41 行同一处）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了
  ✓ 第 130 行 增补 3 第 1 件：随机历史偏向抬 F 之后复用的取样点判出「复用时新记录罩住的已回收记录不删」（第 121 行同一处）：reuse_heavy_random_histories_cover_released_records_and_end_only_in_known_red_forms 红了
  ✓ 第 139 行 增补 3 第 1 件（代码三方第一轮第 1 条）：已知红第 1 条不看 F 是否落在回退留下的空档里：an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding 红了
  ✓ 第 140 行 增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了
  ✓ 第 141 行 增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了
exit 0
```

| 行 | 点名的测试红在哪一行 | 判出的种子 |
|---|---|---|
| 129（第 41 行同一处） | 快档第 158 行「清单外的失败」 | `[I-1.1, I-3.9, I-5.4]` 17 个（3、7、15、20、26、27、38、46、51、54、56、57、62、66、68、69、94）；panic `allocator.rs:529`（71） |
| 130（第 121 行同一处） | 偏向复用那条第 186 行「清单外的失败」 | `[I-5.4]` 7 个（8、11、12、20、21、35、46） |
| 139（第 1 条不看空档） | `an_allocated_…` 第 371 行分类断言 | — |
| 140（N2） | 快档第 158 行 | 执行器判的「分配代」17 个，第一个 3 |
| 141（B6） | 快档第 158 行 | 执行器判的「冷启动读回」45 个，第一个 0 |

每行的日志：`gate59-round2-row-<行号>.log`。

### 13.8 每条新测试会不会红

- 做法：草稿副本 `proof/repo`（自己的 target），从 `proof/pristine/` 拷回并 `touch`。日志 `proof/proofs.log` 的「第六轮」，最终 `history.rs` sha256 `ea22c355…`，测试文件 `f57410ec…`。
- 基线：lib 与这个测试二进制都全绿，基线红集为空。

| 新测试 | 改坏哪一行 | 哪条断言红 | 同时红 |
|---|---|---|---|
| `record_rewritten_by_a_publish_that_keeps_its_old_generation_is_reported` | `history.rs` 第 697 行，分配代判定不看下界 | 「复用改写的那一条还写着释放代 5」：left `[]`，right `[… 50178 … generation: 5 …]` | 无 |
| `failed_cold_start_read_back_is_a_harness_judgement_and_the_other_read_backs_are_not` | 第 741 行，读回文件也记成失败 | 第二个 `assert_eq!`：left `Some(…"FileRead")`，right `None` | 无 |
| 同上 | 第 736–740 行，读回报错不记成失败 | 第一个 `assert_eq!`：left `None`，right `Some(…"Failed")` | 无 |
| `harness_judgement_is_never_classified_as_a_known_red_form` | 第 915 行，已知红不排除执行器判的失败 | 「要是新发现」 | 无 |
| `an_allocated_statistic_over_count_after_raising_the_floor_without_a_rollback_is_a_new_finding` | 第 2053 行，被抛弃的判定把 txg = T 的根也算上 | 「txg 3 那条根是实例 1 的有效根…」：left `Some(true)`，right `Some(false)` | 无 |
| 同上 | 第 934 行，去掉空档那个合取（变异表第 139 行） | 第 371 行分类断言 | 无 |
| `reuse_heavy_random_histories_…` | 第 344 行，抬 F 比重 22 → 0 | 第 191 行「新分配的记录一次都没罩住别的已释放记录的起点…」 | 无 |
| 同上 | `allocator.rs` 的第 121 行变异（变异表第 130 行） | 第 186 行「清单外的失败」 | — |
| 快档，内容长度那组断言 | `transaction.rs` B1（`>` 写成 `>=`） | 第 112 行「装得下的内容长度「载荷容量」一次都没发成」 | 无 |
| 快档，分配代 / 冷启动 | N2 / B6（变异表第 140 / 141 行） | 第 158 行「清单外的失败」 | — |

### 13.9 这一轮写过的文件，与 sha256

- `crates/singlefs-harness/src/history.rs`：
  - 新增 `GenerationWeights`、`HarnessJudgement`、`ColdStartReadBack`、`AppliedStep`；
  - 新增函数 `records_changed_with_a_generation_outside_the_publish_txgs`、`harness_judgement_of_outcome`、`raised_floor_lands_only_on_abandoned_roots`、`generate_history_with_weights`；
  - `FailureObservation` 加两个字段；`FailureSignature` 加 `HarnessJudgement` 成员；`RecordReuse` 加 `released_records_covered`；`HistoryTally` 加 `content_lengths_published` 与 `released_records_covered`；
  - `run_history_campaign` 加比重参数，`CampaignReport` 加 `weights`；`classify_failure` 改成公开；
  - 已知红第 1 条收窄；三条单测改了一处命名（去掉打头的 a_，命名门禁认单字母）。
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`：
  - 快档加 P3 那组断言；
  - 新用例 `reuse_heavy_…` 与 `an_allocated_…`；
  - 大档加比重与收缩两个环境变量，并打出每个新发现的种子；
  - 各处调 `run_history_campaign` 补比重参数。
- `crates/mutations.tsv`：
  - 第 130 行（我自己的）改名字、cargo 参数与点名的测试；
  - 末尾追加第 139–141 行，变异名：
    - 「增补 3 第 1 件（代码三方第一轮第 1 条）：已知红第 1 条不看 F 是否落在回退留下的空档里」
    - 「增补 3 第 1 件（代码三方第一轮第 3 条）：随机历史快档判出「复用改写已回收记录时不改分配代」（N2）」
    - 「增补 3 第 1 件（代码三方第一轮第 4 条）：随机历史快档判出「checker 判绿的镜像上冷启动读回报错」（B6：补齐字节从最后一个载荷字节算起）」
  - 别的行没碰，第 131–138 行（层 0 并行化）原样。
- 没碰：`crash.rs`、`lib.rs`（sha256 与开工时相同）、`second_transaction_step_zero_layer0.rs`、`singlefs-core`、`singlefs-checker`、kb。

```
ea22c355736a69a1791ae9d669c8ad1d37adaf8aed50fea57dbe3e8d7452f56b  crates/singlefs-harness/src/history.rs
f57410ec6068c51d8957aad013050caab8888317a5b7f9bc06a032c8dd236c81  crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
438abd6cdd673fcbe1f767f35865216e1205a143da0a10d575b2d1196a41ad0a  crates/singlefs-harness/src/crash.rs
133ce0338e86df7e74367c76ce51cccef0bd7a418ed846d6b844f2a08374e088  crates/singlefs-harness/src/lib.rs
3f1007c539db8c81f15fa66bdec5d2d7f8efcdadf1804ec90dd827b67cf51366  crates/mutations.tsv
```

`git diff --stat -- crates litmus` 原样（`crash.rs`、`lib.rs`、`second_transaction_step_zero_layer0.rs` 那几行是层 0 并行化补丁与我之前两轮的改动，这一轮没动；我这一轮的两个文件没进 git，不在 diff 里）：

```
 crates/mutations.tsv                               |  13 +
 crates/singlefs-harness/src/crash.rs               | 820 +++++++++++++++++++--
 crates/singlefs-harness/src/lib.rs                 |   6 +
 .../tests/second_transaction_step_zero_layer0.rs   |  71 +-
 4 files changed, 856 insertions(+), 54 deletions(-)
```

### 13.10 没做什么

- **第 2 条没做**（13.2 节）。第 0 条还是原来的宽度，只多了「执行器判出失败的不算」这一个收窄。
- **挂载的写行与暖机不比分配代**。挂载之后第一次覆盖写才比得到那次挂载复用的记录；挂载自己那几次发布里复用改写时代没改，这一轮罩不到。
- **P1 的判定用的是实现的读法**，不是 checker 的。
- **没做的量**：`reuse_heavy_…` 这组比重只在 30 步上量了判出率；N2、B6、B1 没在大档上量。
- **归主 agent 派的**：三方下一轮、崩溃验证员（门禁 54 号、59 号整表复跑）。没提交。
- **草稿都在 `/tmp/claude-1000/m2-supp3-item1/`，没入库**：
  - `r2/`：各变异副本的 release 日志与 13.2 节那条一次性用例；
  - `proof/proofs.log` 第六轮；
  - `check-sh-round2.log`、`gate59-round2*.log`、`round2-final.sha256`。

## 十四、代码三方第二轮之后（判决 `research/prompts/m2-supp3-item1-code-r2-main-verification.md` 第二节第 1、2 条；2026-09-19 00:05Z 续做）

### 14.1 第 1 条：挂载写出的写行与暖机也比分配代（被攻过零轮）

- 照攻方的 `y2-proposal-patch.py` 写，落在 `history.rs`：
  - `mount_publish_allocation_judgement` 读挂载之前的镜像（可写挂载与回退都在调入口之前取 `pool.image()`），取 `MountOutput::effective_root` 那棵分配记录树（`recovery::allocation_records_under_root`）当第一次的上一版。
  - 写行与每一次暖机里带文件的那几版逐次比：改写或新增的记录，代要等于那一次的 txg；不等就是 `HarnessJudgement::AllocationGenerationIsNotThePublishTxg`。
- 比攻方的改法多做两样：
  - `AppliedEffect::Mounted` 带回比过几次（`MountAllocationComparison`：逐次比过几次 / 没有带文件的一版 / 上一版读不出没比）与这次挂载里的复用（`RecordReuse`，逐次相加，并进计数）；
  - 计数加两项：挂载写出的发布比过几次、上一版读不出没比的挂载次数。
- 挂载半路报错时不比（入口没交回写出去的那几次）。
- 快档与取样点里「读不出、没比」都是 0 次；比过 936 次（快档）与 581 次（取样点）。见 `check-sh-round3.log` 第 447、380 行。
- 新用例 `row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked`：
  - 历史是形态 a 在取样点比重下种子 2 收缩出来的 11 步，从 mkfs 起，最后一步是抬 F 回收 18 个落点之后的可写挂载。
  - 今天的代码上跑完；最后那次挂载逐次比了两版、复用改写 14 条。
- 变异表追加两行，点名这条用例：
  - 第 143 行形态 a：原文同 `y2-patch.py` 那一处 `existing.generation = generation;\n            existing.is_released = false;`。
  - 第 144 行形态 b：原文 `    fn record(&mut self, placement: Placement, generation: CheckpointTxg) {\n`。
  - 「只在挂载里」一行替换写不出线程局部标志，改用调用栈判：`std::backtrace::Backtrace::force_capture()` 的文字里有 `mount::establish_instance` 才变异。写行与暖机都在 `establish_instance` 里，重建分配器在它之前，与攻方的标志同一个作用域。
- **重量**（debug，整个测试二进制，草稿 `r3/ya`、`r3/yb`）：
  - 形态 a：快档 7 段、取样点 12 段判红。
  - 形态 b：快档 72 段、取样点 47 段判红。
  - 与攻方、核查员的数逐个相同。
  - 新用例两种形态下都红在第 416 行「今天的代码上这段跑完…」那条断言。
  - 基线上两条大用例新发现 0。

### 14.2 第 2 条：几何敏感性

- 装置：大档（release，32 线程，不收缩）按种子数、每段步数、比重跑，数新发现的种子数（「红」）；日志 `r3/<副本>/sweep-*.log`，汇总 `r3/sweep.tsv`、`r3/sweep-extra.tsv`。
- 取样点：快档那一组 broad 96 × 30 与取样点那一组 reuse 48 × 30 各自配 20 步、40 步，再加比重互换（快档种子配 reuse、取样点种子配 broad）。
- B1 的判据在快档的路径断言里（装得下的「载荷容量」一次都没发成），表里报「发成 / 入口被调」。
- 第 139 行的变异（去掉「落在空档里」）单独施加时，今天的核心上没有东西可露。这里报 A1（回收门槛差一）与「A1 加第 139 行」两列的新发现：第 139 行把 A1 的失败全吞进第 1 条，就是它在各取样点上的判别力。

**几何敏感性**

| 行 / 变异 | broad 96×30（快档） | reuse 48×30（取样点） | broad 96×20 | reuse 48×20 | broad 96×40 | reuse 48×40 | reuse 96×30（快档种子换比重） | broad 48×30（取样点种子换比重） |
|---|---|---|---|---|---|---|---|---|
| 基线（误红） | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| 129（第 41 行同一处） | 18 | 28 | 12 | 19 | 18 | 28 | 50 | 8 |
| 130（第 121 行同一处） | 2 | 7 | **0** | 1 | 5 | 10 | 14 | **0** |
| 139：A1 / A1 + 第 139 行 | 17 / 0 | 24 / 0 | 12 / 0 | 20 / 0 | 17 / 0 | 24 / 0 | 43 / 0 | 7 / 0 |
| 140（N2） | 18 | 28 | 12 | 19 | 18 | 28 | 50 | 8 |
| 141（B6） | 45 | 4 | 40 | 4 | 46 | 6 | 12 | 22 |
| B1：载荷容量发成 / 入口被调 | 0 / 71 | 0 / 58 | 0 / 53 | 0 / 48 | 0 / 80 | 0 / 61 | 0 / 107 | 0 / 34 |
| 143（形态 a） | 7 | 12 | 3 | 5 | 10 | 17 | 21 | 2 |
| 144（形态 b） | 72 | 47 | 71 | 47 | 72 | 47 | 94 | 35 |

第 130 行另补的几个点（`r3/sweep-extra.tsv`）：reuse 96×20 为 2，reuse 96×40 为 20，broad 192×20 为 0。

- **不红的只有第 130 行的两点**：快档比重、20 步（96、192 个种子都是 0），与取样点种子换成快档比重（broad 48×30）。
  - 快档比重的历史在 20 步内走不到「抬 F 回收一批 1 槽单元、重开、下一个 2 槽数据单元罩住两条相邻的已回收记录」，加种子也不红（192 个仍是 0）。
  - 这一类在快档比重下要 30 步、96 个种子才判得出（2 个：54、69）。
- **补了什么**：变异表第 145 行，同一处变异点名快档。第 121 行那一类从此有两个取样点在门禁里：取样点（reuse，第 130 行，7 个）与快档（broad，第 145 行，2 个）。一边的比重被改走了，另一边照样在门禁 59 号里判红。取样点本身的种子数与步数没改。
- **余量薄的点**：
  - 第 130 行 reuse 48×20 只有 1 个（种子 12）；
  - 第 145 行（快档）只有 2 个；
  - 第 143 行在 broad 48×30 只有 2 个。
  - 这几个取样点的种子数、步数、比重以后要改，先重量。
- B1 在八个点上都判得出，但只有快档带这组路径断言：取样点那条用例在 B1 下不红。

### 14.3 check.sh 与门禁 59 号的做法复跑新行

`nice -n 19 bash .claude/scripts/check.sh`，主工作区，00:22:44Z 起、00:23:49Z 完，开跑前 `ps` 没有别的 cargo、门禁、性能测量进程。绿，末尾原样：

```
   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit 0
```

- 耗时：
  - 取样点那条 25.4 秒、快档 33.8 秒，两条在同一个测试二进制里并行跑，那个二进制 33.77 秒跑完；
  - `check.sh` 整个 65 秒。
- 新行按 59 号的做法复跑，00:23:49Z–00:24:19Z，草稿 `gate59-rows.py`。输出原样：

```
  ✓ 锚点预扫：140 条变异，原文都恰好命中一次
  ✓ 第 143 行 增补 3 第 1 件（代码三方第二轮第 1 条，形态 a）：只在挂载写出的写行与暖机里复用改写已回收记录时不改分配代：row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked 红了
  ✓ 第 144 行 增补 3 第 1 件（代码三方第二轮第 1 条，形态 b）：挂载写出的写行与暖机里每次分配的分配代记成 txg − 1：row_and_warm_up_publishes_that_reuse_reclaimed_records_have_their_allocation_generations_checked 红了
  ✓ 第 145 行 增补 3 第 1 件（代码三方第二轮第 2 条，几何补取样点）：随机历史快档也判出「复用时新记录罩住的已回收记录不删」（第 121、130 行同一处）：random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation 红了
exit 0
```

- 第 143、144 行红在新用例第 416 行。
- 第 145 行快档新发现 2 段（种子 54、69，`[I-5.4]`），红在第 158 行。
- 另一个会话追加的第 142 行在我追加之前就在；我追加前重读过，只加在末尾。

### 14.4 新测试会不会红

- **核心侧**：形态 a、b（14.1）。
- **执行器侧**：`proof/proofs.log`「第七轮重跑」，基线全绿。
  - `history.rs` 挂载那段不数比过几次：新用例红在第 429 行，left `NoPublishWithFile`，right `Compared { publishes: 2 }`。
  - 不数复用：新用例红在第 434 行「这次挂载的发布真的复用改写了…」。
- **作废的一轮**：第七轮第一次跑作废。副本是用 `rsync -a` 覆盖过去的，`transaction.rs` 的 mtime 回到比第六轮 B1 那次编译更早，cargo 沿用了带 B1 的旧产物，基线快档因此红在内容长度那一条。删副本重拷、全部 `touch` 之后重跑，基线绿。前几轮的证明都是在新拷的副本上做的，没有这个问题。

### 14.5 写过的文件与 sha256

- `crates/singlefs-harness/src/history.rs`：
  - 新增 `MountAllocationComparison`、`mount_publish_allocation_judgement`、`RecordReuse::plus`；
  - `settle_mount`、`apply_mount_writable`、`apply_mount_rollback` 交回 `AppliedStep`；
  - `AppliedEffect::Mounted` 加两个字段；计数加两项；分派那一处去掉「挂载不比」的注释。
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`：新用例一条。
- `crates/mutations.tsv`：末尾追加第 143–145 行（上面三个变异名）。
- 没碰：`crash.rs`（它的 sha256 变了，是层 0 并行化实现员改的）、`lib.rs`、`singlefs-core`、`singlefs-checker`、kb。

```
3e2955378e34a93e015e925846826664ef1ab462c399dd82e5bf2af74a89b08f  crates/singlefs-harness/src/history.rs
05660861846fb051509db76c13fb3faf7720e77510dd16764c13459bdbe70358  crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs
77766ab8ad442dcec756e52561c9e8886536bb171cab8dc86041d8f7793fa3ec  crates/singlefs-harness/src/crash.rs
133ce0338e86df7e74367c76ce51cccef0bd7a418ed846d6b844f2a08374e088  crates/singlefs-harness/src/lib.rs
851dfdd35a597dcc2d7a59783ab0e2018518a66b1886442f58e37ed1e311dce8  crates/mutations.tsv
```

### 14.6 没做什么

- 第 1 条被攻过零轮（判决写明第 1 件不开第三轮）。挂载半路报错时不比；回退之后生效根读不出的那一格没造出来（「读不出、没比」在快档与取样点里都是 0 次）。
- 几何敏感性只在 release、一次、八个点（第 130 行十一个点）上量。
- 形态 a、b 的行用调用栈判「在挂载里」：要求带调试符号的构建，dev 与 release 都带。
- 草稿在 `/tmp/claude-1000/m2-supp3-item1/`（`r3/`、`proof/proofs.log`、`check-sh-round3.log`、`gate59-round3*.log`、`round3-final.sha256`），没入库。
- 三方与崩溃验证员归主 agent；没提交。
