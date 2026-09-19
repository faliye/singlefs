# 层 0 崩溃点重放改成多线程（增补 2 收口表第 41 行）——实现员报告

时刻都是 UTC。仓副本：`/tmp/claude-1000/m2-layer0-parallel/copy/`（自己的 target）；主工作区开工时的快照：`…/original/`（`crates/`、`.claude/gate.d/`）；变异副本：`…/mutant/`；日志：`…/logs/`；草稿脚本：`…/draft/`。主工作区 `crates/`、`target/` 一个字没动。

**先说一件出错的事**：16:13 我误把一条占位文字「(not yet — continuing work; ignore)」经 SubagentHandback 交回了（本该只在最后调一次）。活没停，这份报告就是完整交付；`progress.md` 里记了这件事。

## 一、结论

1. `crash.rs` 里 `enumerate_layer0_*` 那一族全部改走一个按状态序号区间切片、`std::thread::scope` 多线程跑的本体，没加依赖；原有五个枚举函数签名不变，调用点一个没改。线程数环境变量名 **`SINGLEFS_LAYER0_THREADS`**，没设取 `available_parallelism`。
2. 合并是确定的：计数按片的次序相加、第一处违例取序号最小的那一处、观察者在调用线程上按序号次序调用（不要求 `Send`）。
   - 第一条流：线程数 1 与 32 各跑两次（外加一次负载 100 上下时的 32 线程），`LAYER0` / `CHECKER` / `CHECKER_FIRST` 三行彼此逐字相同，也与单进程对照 `research/prompts/m2-wave2-crash-verifier/54-supplement-first-txn-checker-line.log` 的全量那三行逐字相同。
   - 第二条流：线程数 32 跑了两次（负载 100 上下一次、机器空闲时一次），`LAYER0B` 行去掉行首 `LAYER0B ` 之后，与 `54-layer0-replay.log` 第二条 ✓ 行冒号之后的 879 个字符逐字相同。
3. 用时（机器空闲时重测，详见第五节）：第一条流 1 线程 202.45 秒 → 32 线程 7.35 秒；第二条流 32 线程 492.23 秒（单进程对照两条流合计 3 小时 50 分 49 秒，其中第二条流约 3 小时 47 分）。补过的 54 号整道在副本上 8 分 24 秒。
4. 进度：每跑完一片打一行 `LAYER0_PROGRESS`，`--nocapture` 下边跑边出；全量两条流各 512 行。
5. 判别力：丢掉一片、相邻两片重叠，已有的「展开的段按闭式数」断言都红（还有另外 4 到 5 条也红）；`crates/mutations.tsv` 追加 8 行（在补丁里）。
6. 门禁 54 号的改法写成补丁：线程数传进去、成功行报实际线程数、「没显式设成 1 却只用了 1 个线程」判红带下一步、进度行边跑边转出来不删，另把第一条流的 `CHECKER` 行报出来。
7. 补丁两份，19:38 对主工作区的内容 `patch -p1 --dry-run` 都干净（第二节、第十节）；那时主工作区这四个文件与开工快照逐字节相同。
8. 副本上 `check.sh` 绿（第九节）。

**什么现象会推翻**：同一份代码上换线程数、换切法跑出的 `LAYER0` / `LAYER0B` / `CHECKER` 行有任何一个字不同；或者某次全量的 `LAYER0_PARALLEL_FINISHED` 行 `worker_threads=1` 而没显式设成 1；或者主工作区在我快照之后改了 `crash.rs`、测试文件、`mutations.tsv`、54 号，补丁打不上或打上之后编不过。
## 二、这一轮写过的文件

主工作区一个字没写。全在 `/tmp/claude-1000/m2-layer0-parallel/` 下：

- 副本里改的（补丁的来源）：`copy/crates/singlefs-harness/src/crash.rs`、`copy/crates/singlefs-harness/tests/second_transaction_step_zero_layer0.rs`、`copy/crates/mutations.tsv`（末尾追加 8 行，变异名见第六节）、`copy/.claude/gate.d/54-layer0-replay.sh`。
- 补丁：`crates-layer0-parallel.patch`（`crates/` 三个文件）、`gate54-layer0-parallel.patch`（54 号），都是 `-p1`、`a/` `b/` 前缀，由 `draft/make-patches.sh` 拿 `original/`（开工 15:51 从主工作区拷的快照）对 `copy/` 生成。
- 草稿：`draft/`（`run-layer0.sh`、`run-mutations.sh`、`mutations-local.tsv`、`summarize-mutation.py`、`gate54-functions-check.sh`、`make-patches.sh`、报告分段）；变异副本 `mutant/`；59 号复跑用的临时根 `gate59-root/` 与 target `gate59-target/`；日志 `logs/`；`progress.md`；本报告 `report.md`。

补丁对主工作区的改动量（`git apply --stat`，在主工作区跑）：

```
 crates/mutations.tsv                               |    8 
 crates/singlefs-harness/src/crash.rs               |  795 +++++++++++++++++++-
 .../tests/second_transaction_step_zero_layer0.rs   |   71 ++
 3 files changed, 829 insertions(+), 45 deletions(-)
 .claude/gate.d/54-layer0-replay.sh |   63 +++++++++++++++++++++++++++++++++---
 1 file changed, 58 insertions(+), 5 deletions(-)
```

主工作区 `git diff --stat -- crates litmus` 原样（都是别的会话的未提交改动，我没碰）：

```
 crates/mutations.tsv                 |  2 ++
 crates/singlefs-harness/src/crash.rs | 25 ++++++++++++++++---------
 crates/singlefs-harness/src/lib.rs   |  6 ++++++
 3 files changed, 24 insertions(+), 9 deletions(-)
```

补丁是对主工作区今天的内容做的：快照里的 `crash.rs` 已经带着上面那 25 行未提交改动（`SparseDevice::read_into`），`mutations.tsv` 已经带着那 2 行；`history.rs` 与随机历史用例没进补丁。19:38 主工作区这四个文件与快照逐字节相同（`cmp`），两份补丁 `patch -p1 --dry-run` 都干净（`checking file …` 各行、退出码 0）。
## 三、怎么并行的（副本 `crates/singlefs-harness/src/crash.rs`，行号是副本里的）

- 线程数：环境变量 `SINGLEFS_LAYER0_THREADS`（第 887 行常量 `LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`），十进制正整数；没设取 `std::thread::available_parallelism`；那个报错时只用 1 个线程并把来源报成 `available_parallelism_unknown`。设了却是 0、空串、非数字、非 UTF-8 一律 panic（不悄悄退回单线程）。判定在第 955 行 `from_environment_value`，环境变量与核数都由调用方给，单测不改进程环境变量。不加依赖。
- 切片：状态按单线程时的枚举次序编号（第 989 行 `Layer0StatePlan`：每段展开出来的序号区间首尾相接，最后一个序号是全部持久那一个；第 1026 行 `persisted_writes_of_state` 由序号直接算出持久集合）。第 1056 行 `state_slices` 把 [0, 状态数) 切成首尾相接的区间：默认片数取 max(64, 16 × 线程数)、每片至少 16 个状态；用例可以固定每片几个状态（`Layer0SliceLength::StatesPerSlice`）。实测切法：第一条流 32 线程 512 片 × 513 个、1 线程 64 片 × 4097 个；第二条流 32 线程 512 片 × 4111 个。
- 跑：第 1224 行 `enumerate_layer0_in_state_slices` 在 `std::thread::scope` 里起 min(线程数, 片数) 个工作线程，按片号从小到大用一个原子计数领片（先跑完的接着领，越往后的状态越贵也不会有线程闲着）。每个线程每个状态自己建 `CrashImage`、自己记这一片的 `Layer0Tally`；基线 `MemoryPool`、写表、版本表只读共用（`&`，本来就是 `Sync`）。`SharedStream`（`Rc<RefCell<…>>`）只在用例的 `prepare` 里、调用线程上录流时用，枚举拿到的是录完拆好的写表，一个 `Rc` 都不过线程。`expand` 只在调用线程上逐段问一次（建计划时），所以原签名 `&dyn Fn` 不用加 `Sync`，调用点一个都没改。
- 合并（确定的）：工作线程跑完一片经 `mpsc` 交回调用线程；调用线程把早到的片先搁着，按片号从小到大一片片并（第 568 行 `absorb_following_slice`，按字段拆开写全，新加字段不并就编译不过）：计数逐项相加；`first_violation`、`first_ignored_violation` 只在前面各片都没有时取这一片的；`checker_first_violation` 每条不变量各取最早有的那一片。片按序号递增，所以「第一处」就是序号最小的那一处，与线程数、切法无关。收完之后断言每一片都并进来了（第 1332 行）。
- 观察者（`enumerate_layer0_selecting_versions_observing_each_state`）：签名不变，仍是 `&mut dyn FnMut`，**不要求 `Send`**。有观察者时工作线程把每个状态的持久集合与看 journal 那一遍恢复的报告随片带回，调用线程在并这一片时按序号从小到大逐个调观察者——调用次序与单线程时逐个相同（用例里「最后一次赋值为准」的 `records_with_a_reused_named_unit = mismatched` 这类写法照旧成立）。代价：观察者自己的活（改坏 tail 那条用例里每个状态再跑一遍恢复）仍在调用线程上串行；没有观察者时报告不带回，全量流不留两百万份报告。
- 出错：工作线程 panic 时立一面旗子（第 1085 行 `RaiseFlagWhenPanicking`），别的线程不再领新片；观察者 panic 时接收端随之丢掉，工作线程下一次交片发不出去就退。`scope` 照常把 panic 传出来。
- 进度：调用线程每收到一片打一行 `LAYER0_PROGRESS slice=…/… states=[起,止) segments=首段..=末段 finished_slices=…/… finished_states=…/… elapsed_seconds=…`（最后全部持久那个状态的段号写成 `all_persisted`），开跑、跑完各一行 `LAYER0_PARALLEL_START` / `LAYER0_PARALLEL_FINISHED`（状态数、片数、实际起的工作线程数、给定的线程数、线程数从哪来、耗时）。都是 `println!`，`--nocapture` 下边跑边出；`LAYER0` / `LAYER0B` / `CHECKER` 行一个字没动。
- 公开接口：原来五个枚举函数签名不变（`enumerate_layer0`、`enumerate_layer0_selecting`、`enumerate_layer0_selecting_versions`、`enumerate_layer0_versions`、带观察者的那个），都走并行本体、线程数取环境变量；新增 `enumerate_layer0_in_state_slices`（显式给切法与可选观察者，用例对拍用）、`Layer0Parallelism`、`Layer0SliceLength`、`Layer0WorkerThreadsSource`、`Layer0StateObserver`、常量 `LAYER0_WORKER_THREADS_ENVIRONMENT_VARIABLE`。
## 四、逐字比对（验收第 2 条）

跑法：`draft/run-layer0.sh <线程数> <测试二进制> <用例名> <日志>`，在副本里 `SINGLEFS_LAYER0_THREADS=<线程数> nice -n 19 cargo test --offline --release -p singlefs-harness --test <二进制> -- --include-ignored --exact <全量用例> --nocapture`，日志首尾记 UTC 时刻、`uptime`、挂钟秒数、退出码。

第一条流（`first_transaction_step_seven_layer0::layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations`）：

| 日志（`logs/`） | 线程数 | 切法 | `LAYER0` 行 | `CHECKER` 行 | `CHECKER_FIRST` 行 |
|---|---|---|---|---|---|
| `first-stream-threads32-loaded.log` | 32 | 512 片 × 513 | 同 | 同 | 同 |
| `timing-first-stream-threads1.log` | 1 | 64 片 × 4097 | 同 | 同 | 同 |
| `timing-first-stream-threads32.log` | 32 | 512 片 × 513 | 同 | 同 | 同 |
| `timing-first-stream-threads1-second-try.log` | 1 | 64 片 × 4097 | 同 | 同 | 同 |
| `timing-first-stream-threads32-second-try.log` | 32 | 512 片 × 513 | 同 | 同 | 同 |
| 对照 `research/prompts/m2-wave2-crash-verifier/54-supplement-first-txn-checker-line.log`（单进程） | 1（并行之前的代码） | 不切 | 同 | 同（全量那一行，不是 22 个状态那一行） | 同 |

「同」是 bash 字符串逐字相等（`[[ "$a" == "$b" ]]`）；三行的 sha256 前 16 位：`LAYER0` b77948b460c752a4、`CHECKER` 107a95c7d9c0c295、`CHECKER_FIRST` 1049609028f8b4d9。原样：

```
LAYER0 states=262165 closed_form=262165 violations=0 root_persisted_states=4 no_file=262158 file_read=7 failed=0 verification_ran=6 verification_failed=0 journal_differing=3 exhaustive=true
CHECKER_FIRST {}
```

第二条流（`second_transaction_step_zero_layer0::full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`）：`LAYER0B` 行去掉行首 `LAYER0B `，与 `research/prompts/m2-wave2-crash-verifier/54-layer0-replay.log` 里「✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle）：」之后的文字逐字相等，879 个字符，两边 `printf '%s' | sha256sum` 都是 `2bde7d65a2c283e72364ac1b9fbdb9fae90b2b8715562d975698f42fbdd66c3d`。比过的日志：`second-stream-threads32-loaded.log`（32 线程、负载 74→78，最高 130 上下）、`timing-second-stream-threads32.log`（32 线程），机器空闲时的第二次见第五节。

对照日志跑的是 HEAD 502ba80 加当时工作区的改动；今天主工作区的 crates 已经是 cae5092 加未提交改动，两份对照读数与今天代码的并行读数逐字相同，说明中间的代码变化没动这两条流的读数，也说明并行没动。
## 五、用时（验收第 5 条）

挂钟取 `run-layer0.sh` 里 cargo 前后 `date +%s%N` 之差（含 cargo 启动，release 已编好）；负载取日志首尾的 `uptime`（1 / 5 / 15 分钟均值），另有每 20 秒一次的 `/proc/loadavg` 采样 `logs/timing-load-samples.log`。本机 32 核。

| 流 | 线程数 | 日志（`logs/`） | 挂钟 | 开跑时负载 | 跑完时负载 | 期间 |
|---|---|---|---|---|---|---|
| 第一条（262165 个状态） | 1 | `timing-first-stream-threads1.log` | 226.06 秒 | 3.21, 0.86, 0.31 | 17.51, 9.22, 3.80 | 别的会话同时在跑（19:01:24 看到 `/tmp/claude-1000/m2-supp3-item1-r1-opus/target-C` 下的随机历史快档占 1608% CPU），负载涨到 17，这一次不算数 |
| 第一条 | 1 | `timing-first-stream-threads1-second-try.log` | **202.45 秒** | 18.99, 25.69, 15.70 | 1.60, 13.43, 12.79 | 开跑时的 18.99 是我上一趟 32 线程留下的余数，采样 1 分钟均值一路降到 1.65，只有这一个线程 |
| 第一条 | 32 | `timing-first-stream-threads32.log` | 11.95 秒 | 17.51, 9.22, 3.80 | 24.56, 11.18, 4.53 | 开跑时 1 分钟负载 17.51 不是我的进程（上一趟是单线程），没核是谁，这一次不算数 |
| 第一条 | 32 | `timing-first-stream-threads32-second-try.log` | **7.35 秒** | 1.24, 12.77, 12.58 | 3.71, 13.09, 12.69 | 空闲 |
| 第二条（2104413 个状态） | 32 | `timing-second-stream-threads32.log` | 503.08 秒 | 24.56, 11.18, 4.53 | 28.83, 27.94, 16.13 | 采样最高 33.53（其中 32 是自己） |
| 第二条 | 32 | `timing-second-stream-threads32-second-try.log` | **492.23 秒** | 3.14, 12.66, 12.55 | 27.13, 27.34, 20.31 | 采样 5.45 到 32.08，32 是自己 |
| 第二条 | 32 | `second-stream-threads32-loaded.log` | 1874.47 秒 | 73.83, 65.45, 48.02 | 77.91, 100.47, 103.97 | 负载 100 上下（另一条腿在跑变异），只用来比对读数 |

读法：第一条流 202.45 → 7.35 秒，约 27.5 倍（单进程对照：同日单独跑整个用例文件 203.84 秒）。第二条流 32 线程 492.23 秒（8 分 12 秒）；单进程对照两条流合计 3 小时 50 分 49 秒，减去第一条流约 3 分钟，第二条流约 3 小时 47 分（约 13600 秒），约 27.7 倍。两条流 32 线程合计约 8 分 20 秒。第二条流最后 32 片（480 → 512）用了约 40 秒（451.5 → 491.9 秒）：越往后的状态越贵、又是最后才领，尾巴上有线程闲着；要再压可以倒着发片（序号大的先领），合并次序不受影响，这一轮没做。
## 六、判别力：每条新测试怎么证明会红

做法：`mutant/` 是 `copy/` 的另一份副本（`rsync -a --exclude target --exclude .git`），自己的 target；`draft/run-mutations.sh` 逐条改坏 `copy/` 里的 `crash.rs` 一处，跑四个测试二进制（`--lib`、`first_transaction_step_seven_layer0`、`second_transaction_step_zero_layer0`、`second_transaction_step_three_formatted_pool_layer0`，debug，`--no-fail-fast`），记红了哪些；改完从 `copy/` 拷回原件再 `touch`，`cmp` 过。基线（不改动）四个二进制全绿，基线红集为空（`logs/mutation-baseline.log`：23 / 5 / 3 / 7 passed）。被测代码里没有 `debug_assert`。日志：`logs/mutation-<名>.log`，摘要用 `draft/summarize-mutation.py` 取。

| 改坏哪一行（副本 crash.rs） | 红了哪些测试 → 哪条断言 |
|---|---|
| M1 第 1069 行 `(0..state_count.div_ceil(states_per_slice))` → `(1..…)`：丢掉第一片 | 7 条：`state_slices_cover_every_state_exactly_once_in_ordinal_order`（「最后一片止于状态数」）；`layer0_partial_enumeration_…`（first_transaction_step_seven_layer0.rs:219 的 22 个状态逐项计数）；`every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_…`（second_transaction_step_three_formatted_pool_layer0.rs:148「展开的段按闭式数」）；`every_crash_state_outside_the_two_unit_segments_…`（second_transaction_step_zero_layer0.rs:447「展开的段按闭式数」）；`stale_tail_…`（:1141 状态数）；`residual_record_…`（:970 状态数）；`one_state_slices_on_eight_threads_…`（:519「平时跑的那 108 个状态」） |
| M2 第 1073 行 `.saturating_add(states_per_slice)` 之后再 `.saturating_add(1)`：相邻两片重叠 1 个状态 | 6 条：`state_slices_cover_…`（「片首接上一片的尾」）；`layer0_partial_enumeration_…`（:219）；formatted pool 快用例（:148「展开的段按闭式数」）；two_unit_segments 快用例（:447「展开的段按闭式数」）；`residual_record_…`（:970）；`one_state_slices_…`（:526 观察者次序） |
| M3 第 598 行 `if self.first_violation.is_none()` → `if first_violation.is_some()`：并片时第一处违例取后面那一片的 | 2 条：`absorbing_a_following_slice_…`（crash.rs 单测，第一处违例那条 assert_eq）；`one_state_slices_on_eight_threads_…`（:530「计数与每一处「第一处」都逐项相同」） |
| M4 第 616 行 `checker_first_violation.entry(invariant).or_insert(detail)` → `insert(invariant, detail)`：checker 第一处取后面那一片的 | 1 条：`absorbing_a_following_slice_…`（「每条不变量的第一处各取最早有的那一片」）。集成用例没红：到 E 的快档 108 个状态上 checker 零违例，这一格只有单测盯着 |
| M5 第 1310 行 `waiting_for_earlier_slices.remove(&next_slice_to_merge)` → `pop_first()`：谁先到先并、先调观察者 | 1 条：`one_state_slices_on_eight_threads_…`（:526「观察者按序号次序看到同一串持久集合」）。靠 8 个线程抢 108 片时到达次序乱，理论上可能碰巧有序而不红，所以没进变异表 |
| M6 第 1035 行 `let subset_mask = ordinal - …start` → 取反：段内子集算错 | 3 条：`the_state_plan_hands_out_…`（「第 k 个状态就是逐段走到的第 k 个」）；`layer0_partial_enumeration_…`（:219）；`residual_record_…`（:971 走到残留记录的状态数） |
| M7 第 959 行 `match environment_value {` → `match environment_value.and(Err::<String, _>(NotPresent)) {`：不读环境变量 | 2 条：`worker_threads_come_from_the_environment_variable_…`（「环境变量设了就不该再问 available_parallelism」那个 panic）；`zero_worker_threads_…` |
| M8 第 961 行 `text.parse::<NonZeroUsize>()` 之后 `.or(Ok(NonZeroUsize::MIN))`：设成 0 悄悄退回 1 个线程 | 1 条：`zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back` |

M8 跑的时候那条测试还写成 `#[should_panic]`；门禁 59 号的判红正则是 `^test … \.\.\. FAILED$`，`should_panic` 的测试输出行带「- should panic」认不出（本地复跑 59 号那 8 行时它报「没跑到」，`logs/gate59-new-rows.log`）。已改成 `catch_unwind` 加断言的普通测试；在这一版上重跑：基线四个二进制照旧全绿（`logs/mutation-baseline-final-code.log`），M8 红在 `zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back`（crash.rs 第 1585 行的 `expect_err`「设成 0 要停下，不许退回 1 个线程接着跑」，`logs/mutation-M8_zero_threads_fall_back_to_one.log`）；59 号判法复跑 8 行全红在点名的测试上（第九节）。

### `crates/mutations.tsv` 末尾追加的 8 行（补丁里，主工作区的表没动）

变异名（第一段）：
1. `增补 2 第 41 行 层 0 并行：丢掉第一片（状态数等于闭式的断言要红）` → 点名 `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims`
2. `增补 2 第 41 行 层 0 并行：相邻两片重叠（状态数等于闭式的断言要红）` → 同上
3. `增补 2 第 41 行 层 0 并行：相邻两片重叠（切片单测）` → `state_slices_cover_every_state_exactly_once_in_ordinal_order`
4. `增补 2 第 41 行 层 0 并行：按序号取状态时段内子集掩码取反` → `the_state_plan_hands_out_the_same_persisted_sets_in_the_same_order_as_walking_segment_by_segment`
5. `增补 2 第 41 行 层 0 并行：并片时第一处违例取后面那一片的` → `one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread`
6. `增补 2 第 41 行 层 0 并行：并片时 checker 每条不变量的第一处违例取后面那一片的` → `absorbing_a_following_slice_adds_counts_and_keeps_the_earlier_first_violation`
7. `增补 2 第 41 行 层 0 并行：不读 SINGLEFS_LAYER0_THREADS` → `worker_threads_come_from_the_environment_variable_before_available_parallelism`
8. `增补 2 第 41 行 层 0 并行：SINGLEFS_LAYER0_THREADS=0 悄悄退回 1 个线程` → `zero_worker_threads_in_the_environment_variable_stops_instead_of_falling_back`

派发只要一行；验收第 4 条的「丢一片、两片重叠」各给了一行（第 1、2 行，点名的是已有的「展开的段按闭式数」那条断言所在的快用例），其余 6 行钉的是这一轮新写的合并、次序与线程数判定。嫌多可以只留第 1、2 行。
## 七、门禁 54 号的改法（`gate54-layer0-parallel.patch`，由主 agent 打）

改的是 `.claude/gate.d/54-layer0-replay.sh`（不在我的写范围，只在副本里改、出补丁）：
- 线程数传进去：`SINGLEFS_LAYER0_THREADS` 设了就用（来源记成「显式设的」），没设就取 `nproc` 并 `export`（来源记成「没设，取本机核数」）。
- 跑 cargo 改成 `cargo test … 2>&1 | tee 日志 | grep --line-buffered '^LAYER0_PROGRESS ' | sed -u 's/^/    /'`：整段输出照旧进临时日志（红的时候照旧 `tail -40`），进度行边跑边转到本阶段的输出里，不删；退出码取 `PIPESTATUS[0]`。
- 成功行里报实际起了几个工作线程、`SINGLEFS_LAYER0_THREADS` 的值与来源、本机核数；第一条流另加一行 `✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：…`，把全量那条的 `CHECKER` 行报出来（收口表第 41 行记的「54 号第一条流把 CHECKER 行删了」那个缺口）。
- 实际线程数取法：计数行（`LAYER0` / `LAYER0B`）里的 `states=N`，找 `LAYER0_PARALLEL_FINISHED states=N ` 那一行的 `worker_threads=`。
- 判红：没显式把 `SINGLEFS_LAYER0_THREADS` 设成 1、本机多于 1 核、全量那条却只起了 1 个工作线程 → 红，带下一步（去看 `Layer0Parallelism::from_environment` 读没读到变量、`state_slices` 切出来的片够不够分；真要单线程就显式 `SINGLEFS_LAYER0_THREADS=1 bash .claude/gate.d/54-layer0-replay.sh`）。找不到状态数对得上的收尾行也红（全量那条绕开了并行本体）。

核过的：副本里 `gate-lint.sh` 过（「83 个脚本（.sh 与 .py）、266 条拒绝都带了出路」）、`shell-lint.sh` 过（「共 79 个脚本」）。两个判线程数的函数拿合成日志逐情形跑过（`draft/gate54-functions-check.sh`）：实际 32 线程四种设法都过；实际 1 线程时，只有「显式设成 1」过，「没设」「显式设成 32」都红；没有收尾行一律红；本机 1 核、没设、1 线程过。**整道 54 号没在副本上跑过**：见「没做什么」。

**补记：补过的 54 号整道在副本上跑了一次**（`env -u SINGLEFS_LAYER0_THREADS nice -n 19 bash .claude/gate.d/54-layer0-replay.sh`，`logs/gate54-patched-on-copy.log`）：19:29:18 → 19:37:42（8 分 24 秒，两条流合计，含两个测试二进制里的快用例），退出码 0，负载 4.87 → 27.47。本阶段输出里 1147 行 `    LAYER0_PROGRESS …`（全量两条各 512 行，快用例 123 行），边跑边出。三行 ✓ 的开头原样（冒号之后的计数太长，这里截掉）：

```
  ✓ 层 0 崩溃点重放全量跑完（第一个事务那条流、每个状态两遍恢复 + checker + 记录核对器；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=262165 …
  ✓ 第一个事务那条流逐条不变量（评估过的状态数/判违例的状态数）：record_root_without_record=0 record_claimed_state_missing_unit=0 I-1.1=262165/0 …
  ✓ 层 0 崩溃点重放全量跑完（两次发布那条流，多版本 oracle；32 个工作线程，SINGLEFS_LAYER0_THREADS=32（没设，取本机核数），本机 32 核）：states=2104413 …
```

两条计数 ✓ 行冒号之后的文字，与早上 `54-layer0-replay.log` 里对应两行冒号之后的文字逐字相同。判红那一支（只起了 1 个线程）没在整道上跑出来过，只在函数层面拿合成日志核过（上面）。
## 八、其余跑重放的用例：每一条用了几个线程、改没改（验收第 7 条）

线程数一律按 `SINGLEFS_LAYER0_THREADS` 没设、本机 `available_parallelism` = 32 取；实际起的工作线程数 = min(32, 片数)，片数 = ⌈状态数 / 每片状态数⌉，默认每片至少 16 个状态。下表「实际」一栏取自 `--test-threads=1 --nocapture` 下每条用例打的 `LAYER0_PARALLEL_START` 行（`logs/per-test-threads.log`）；全量两条取自计时日志。「改没改」指用例源码：走枚举族的用例一行没改，靠 `crash.rs` 自动变成多线程。

| 文件 | 用例 | 状态数 | 实际工作线程（改之前都是 1） | 用例源码改没改 |
|---|---|---|---|---|
| first_transaction_step_seven_layer0.rs | `layer0_enumerates_every_crash_state_of_the_settled_stream_with_zero_violations`（ignored，54 号跑） | 262165 | 32（512 片） | 没改 |
| 同上 | `layer0_partial_enumeration_skipping_the_eighteen_write_segment_matches_the_full_tally_shape` | 22 | 2 | 没改 |
| 同上 | `removing_the_barrier_before_the_root_slot_is_caught_by_the_record_checker` | 25 | 2 | 没改 |
| 同上 | `positive_control_root_persisted_without_each_unit_is_caught_by_the_oracle` | 10（直接调 `evaluate_state`） | 1 | 没改：状态是手摆的，不走枚举 |
| 同上 | `applying_a_record_whose_units_are_missing_is_caught_by_the_record_checker` | 1（直接调 `recover`） | 1 | 没改 |
| second_transaction_step_zero_layer0.rs | `full_enumeration_of_the_fixed_script_stream_is_exhaustive_and_clean`（ignored，54 号跑） | 2104413 | 32（512 片） | 没改 |
| 同上 | `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims` | 108 | 7 | 没改 |
| 同上 | `one_state_slices_on_eight_threads_merge_into_the_same_tally_and_observation_order_as_one_slice_on_one_thread`（新） | 108 × 2 | 1 与 8（显式） | 新加 |
| 同上 | `residual_record_seeded_into_the_base_image_is_applied_in_every_crash_state_whose_chain_reaches_it`（带观察者） | 31 | 2；观察者在调用线程上按序号调 | 没改 |
| 同上 | `stale_tail_with_a_reused_named_unit_in_its_window_recovers_every_crash_state_to_the_same_end_state`（带观察者） | 8 | 1（只有 1 片）；观察者里每个状态再跑一遍恢复，也在调用线程上 | 没改 |
| 同上 | `targeted_controls_on_the_second_publish_go_red_where_they_should` | 几个手摆的状态（直接调 `evaluate_state_for_versions`） | 1 | 没改 |
| second_transaction_step_three_formatted_pool_layer0.rs | `every_crash_state_outside_the_unit_segment_of_the_formatted_pool_mount_recovers_to_the_version_its_root_claims` | 22 | 2 | 没改 |
| 同上 | 另两条（段序列、与第一个事务那条流逐项相同） | 不枚举 | — | 没改 |
| second_transaction_step_three_formatted_pool.rs | `formatted_pool_mount_starting_after_a_leftover_record_is_refused_before_acquiring_an_instance` | 1 个手摆的崩溃状态 | 1 | 没改 |
| second_transaction_supplement_three_random_history.rs（别的会话在改，没碰） | 随机历史快档、大档、收缩 | — | 自己的 `SINGLEFS_RANDOM_HISTORY_THREADS`，快档默认 min(核数, 16) | 没碰 |

`crates/` 里 `enumerate_layer0*` 的调用点只有上面这三个测试文件（`grep -rn 'enumerate_layer0' crates --include=*.rs`）；`src/bin/` 三个二进制不跑崩溃状态枚举（`first_transaction_device_log_check` 只对两份盘镜像跑一次 `recover`）。平时 `cargo test` 里的快用例只有几十到一百来个状态，默认切法下起 1 到 7 个线程；这些用例本来就快，没另外调切法。
## 九、`check.sh` 与登记给我的门禁阶段

副本上最后一版代码跑的 `nice -n 19 bash .claude/scripts/check.sh`（`logs/check-sh-final.log`，19:27:52 → 19:28:47，退出码 0），末尾原样：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests singlefs_harness

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
```

四道全过（`grep -E '✓|✗'`）：`✓ 格式通过`、`✓ clippy 通过`、`✓ 构建通过`、`✓ 单测通过`。16:20 那一趟（`logs/check-sh.log`）也是绿的，但跑的途中我改了代码（改名、重写一条测试），不算数。

登记给 implementation-writer 的阶段只有 `53-format-const-placeholders.sh`，在副本上跑，末行原样 `  ✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））`，退出码 0。

另跑过、不归我的（都在副本上）：
- `naming-lint.sh`：第一次红 2 处（`a_worker_thread_panicked` 里的「a」算单字母），已改名 `some_worker_thread_panicked`；最后一版上重跑，原样 `  ✓ 命名纪律通过：查了 192 个 .rs 文件、35971 个声明的名字`。
- 门禁 59 号的判法，只拿追加的 8 行跑（`gate59-root/` 里的表只留注释与这 8 行，`logs/gate59-new-rows-final.log`）：末行 `  ✓ crates 变异表复跑：8 条变异各自红在点名的测试上（原文都恰好命中一次）`，退出码 0。整张表 133 行的锚点预扫：每行六段、原文在文件里都恰好命中 1 次（Python 照 59 号的判法扫，零条不合）；整表没真跑。
- 补过的 54 号整道在副本上跑了一次（没设 `SINGLEFS_LAYER0_THREADS`），结果见第七节末尾的补记。
## 十、交主 agent 定的事，与没做什么

### 我自己定了、写明在这里的设计取舍（没有条款压着，派发里说「由你定」或没说）

1. 线程数环境变量叫 `SINGLEFS_LAYER0_THREADS`（与 `SINGLEFS_RANDOM_HISTORY_THREADS` 同形）；设了却不是正整数就 panic，不退回单线程。
2. 观察者不要求 `Send`，在调用线程上按序号次序调用，调用次序与单线程时相同。代价是观察者自己的活串行，且有观察者时每个状态的报告要随片带回（全量流上没有带观察者的用例）。
3. 默认切法：片数 max(64, 16 × 线程数)、每片至少 16 个状态。只影响快慢与进度行多少，不影响读数。
4. 进度行、`LAYER0_PARALLEL_*` 两行由库里的 `println!` 打，每一次枚举都打（平时 `cargo test` 被 libtest 收着，`--nocapture` 才看得见）；没做成可关的开关。
5. 「每一片都并进来了」写成库里的断言；「状态数等于闭式」没在库里断言，留给用例（验收第 4 条要的是已有断言红）。
6. 54 号额外把第一条流的 `CHECKER` 行报出来（收口表第 41 行记的缺口，派发没点名）；不要可以删那两行。
7. `mutations.tsv` 追加 8 行而不是 1 行（第六节末尾写了哪两行对应派发要的那一行）。

没有碰到要停下的条款空白：这件活只改验证装置，不改盘上格式、不落盘，没有写错误成员、没有 `todo!`。

### 打补丁时要知道的

- 补丁基于主工作区 15:51 的内容；19:02 与 19:38 两次 `cmp`，这四个文件（`crash.rs`、`second_transaction_step_zero_layer0.rs`、`mutations.tsv`、54 号）与快照逐字节相同，两次 `patch -p1 --dry-run` 两份都干净。那一轮三方的腿交回之后如果改了 `crash.rs`，要先重跑 `--dry-run`。
- `mutations.tsv` 的补丁只在末尾追加；主工作区那张表末尾别的会话再追加过行的话，补丁的上下文对不上，改成手工把 8 行接到末尾即可（行在 `copy/crates/mutations.tsv` 最后 8 行）。

### 没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异表整表复跑归 `crash-verifier`；没提交，没做任何 git 写操作。
- 54 号脚本、`crates/`、`mutations.tsv` 在主工作区一个字没改，只出补丁；产物没拷进 `research/results/` 或 `research/prompts/`（写范围不给我，日志都在 `/tmp/claude-1000/m2-layer0-parallel/logs/`，要留的由主 agent 拷；门禁 69 号查「装置改了而 research/results/ 里没有不比它旧的产物」，打补丁时这一条会碰到）。
- 整张 `mutations.tsv`（133 行）没真跑，只做了锚点预扫；新 8 行照 59 号的判法跑过。
- 负载 100 上下时量的那几个挂钟只拿来比读数，不算用时；空闲时重测的见第五节。
- 第二条流单线程没重跑（3 小时以上），单线程的对照用的是早上那份日志；第一条流单线程重跑了。
- 尾巴上线程闲着（最后 32 片约 40 秒）没优化。
- 随机历史（`crates/singlefs-harness/src/history.rs` 与它的用例，别的会话在改）没碰，它有自己的线程数环境变量。

## 十一、交付物校验和

```
aa3ec1619998a178505c832e2d8a1172b0c2def563abb5d41562207c0badfe65  crates-layer0-parallel.patch
e006a7326fc582d5a9d27da2a2b1fb818caff7b18a33cf2a5ef394317d109740  gate54-layer0-parallel.patch
```
