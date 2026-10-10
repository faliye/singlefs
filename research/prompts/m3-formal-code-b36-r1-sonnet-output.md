# m3-formal-code-b36-r1 云端正推腿（Sonnet）报告

<!-- doc-lint:not-numbers P1 P2 P3 P4 P5 R1 R2 R3 FL RV B2 -->

轮：m3-formal-code-b36-r1（第一轮），腿：云端正推，格 P1–P5，日期 2026-10-06。立场：从判决 `research/prompts/m3-formal-code-b35-r1-main-verification.md` 第三节第 1 条与 kb 条款正推每处改动做的是不是它们说的，再对着 `crates/` 今天的实现（冻结副本）逐格比。
代码引文的行号是冻结副本 `~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree/` 里的行号，kb 引文的行号是仓里 `.claude/kb/` 文件自己的行号。本报告里「推的」= 读码、没跑；「量的」= 这一趟在副本上跑出来、原样输出在 `research/prompts/m3-formal-code-b36-r1-sonnet-model/results/`。

## 〇、材料与核对

- 冻结副本 sha256：`sha256sum -c ../crates-sha256.txt | grep -vc ': OK$'` 输出 `0`（清单 316 行，同目录）。kb 快照：`sha256sum -c research/prompts/m3-formal-code-b36-r1-snapshot/kb-sha256.txt` 三份全 OK（`invariants.md`、`23-journal的角色与格式.md`、`checks-owed.md`），本报告引的 kb 行就是这三份今天的行。
- 复跑：`S=<草稿目录> bash research/prompts/m3-formal-code-b36-r1-sonnet-model/tools/rerun.sh setup build tests probes-build p1-a p1-b p1-c probes-small probes-r2a probes-r2b r1-off rows wgsl`，每条 cargo 经 `run-with-memory-cap.sh 8G` 与 `capped.sh 16`，同一时刻只起一条。编译日志核被测包：`grep -c 'Compiling singlefs-checker v\|Compiling singlefs-core v\|Compiling singlefs-harness v' build.log` 输出 `3`。
- 开工前 `ps -o pid,args -u "$(id -u)"` 没有 `qemu-system`、`vm-bench.sh`、`e152-file-system-benchmark`、`fio`，也没有别的 cargo；中途几次 `ps` 看到别的会话的 cargo test、clippy 与变异在跑（我的变异那几趟因此慢过 240 秒），没有性能测量进程；我的命令都加 `nice -n 19`，没等到 cargo 的文件锁。
- 全程只跑 harness 档（`-p singlefs-harness`）；checker 档的测试、GPU 对拍、`crates/mutations.tsv` 整表一条没跑。
- 两处要说在前面：① `tools/wgsl_validate.py` 起 naga 小程序（cargo 编出来的二进制）头两趟没经内存包装，发现后改成整条脚本经 `run-with-memory-cap.sh 4G`，`rerun.sh wgsl` 阶段现在是包装过的，报告里的输出取包装过的那一趟；② 一条变异命令超过 240 秒被工具层自动转了后台（不是我起的），它自己的 `finally` 还原了源文件，之后 `sha256sum -c` 不对的行数 `0`。

## 一、P1　R1（CPU）

### 1.1 条款与实现各说什么

条款（正推起点，kb 自己的行号）：
- 「恢复只施加 `(实例代号, checkpoint_txg)` 严格大于所选根的记录」（`.claude/kb/decisions/23-journal的角色与格式.md:357`）
- 「第五条：施加的单位是一次发布」（`.claude/kb/decisions/23-journal的角色与格式.md:363`）
- 「所选根覆盖的最后一条 = 与所选根同实例、同 checkpoint_txg 的记录里 jsn 最大的那条」（`.claude/kb/decisions/23-journal的角色与格式.md:369`）
- 「施加一条记录 = 把所选根的这七个字段换成记录里的」（`.claude/kb/decisions/23-journal的角色与格式.md:452`）

实现（core 的施加）：`replay_core` 逐条判，一次发布的记录一直走到带末条标志那条才整次施加，验不过点名单元就停在那一条之前、施加到之前最后一个发布末条为止；新根取最后施加的那条的新根段：
- 「if records_taken_toward_the_in_flight_limit >= in_flight_limit {」（`crates/singlefs-replay-prefix/src/lib.rs:3825`）
- 「report.verification_failed += 1;」（`crates/singlefs-replay-prefix/src/lib.rs:3849`）
- 「root_source_after_replay = record.new_root_source;」（`crates/singlefs-replay-prefix/src/lib.rs:3932`）
- 「NewRootSource::NewRootSegmentOfTheRecordAtKeyOrderIndex(index_in_key_order) => {」（`crates/singlefs-core/src/recovery.rs:3656`）

实现（checker 的认法）：`version_the_next_mount_applies_first` 不再自己筛记录，调 I-4.1 那一份重放，取第一次施加的发布的末条：
- 「fn version_the_next_mount_applies_first(」（`crates/singlefs-checker/src/walk.rs:5875`）
- 「last_record_of_the_first_publish_applied.get_or_insert(record);」（`crates/singlefs-checker/src/walk.rs:8718`）
- 「last_record_of_the_first_publish_applied,」（`crates/singlefs-checker/src/walk.rs:8705`，`StoppedAtANamedUnitThatDoesNotVerify` 那一支带出的字段）
- 「[_, _, ..] => return ReplayTheNextMountWouldDo::AnchorUndecided,」（`crates/singlefs-checker/src/walk.rs:8640`）
- 「let Some(chain_start_txg_without_anchor) = chosen_root.checkpoint_txg.checked_add(1) else {」（`crates/singlefs-checker/src/walk.rs:8642`）
- 「if records_taken_before_this_one >= in_flight_record_limit {」（`crates/singlefs-checker/src/walk.rs:8658`）

I-3.10 与 I-5.4 是不是同一个认法：只算一次、同一个值喂两处。
- 「let version_the_next_mount_applies_first = version_the_next_mount_applies_first(」（`crates/singlefs-checker/src/walk.rs:11278`），I-5.4 的版本表从它的结果造称呼与树表指针；
- 「version_the_next_mount_applies_first.as_ref(),」（`crates/singlefs-checker/src/walk.rs:11333`），I-3.10 经 `tree_table_pointer_of_the_version_the_next_mount_applies_first` 取同一个值的树表指针。
所选根两处都是 `&roots[newest_index].2`（`walk.rs:11208` I-4.1 一处、`walk.rs:11281` 这一处，同一个 `newest_index`）；在飞上限两处都经 `in_flight_record_limit_of_the_journal_ring(journal_ring_bytes)`（`walk.rs:5886`、`walk.rs:8755`），公式 `journal_ring_bytes / JOURNAL_RECORD_BYTES / JOURNAL_SAFETY_FACTOR` 与 core 的 `journal_in_flight_record_limit`（`crates/singlefs-core/src/system_configuration.rs:101-103`）逐字同式（推的，没造超上限的环）。
旧代码里「根环里已有 (实例, txg + 1) 的根就不认」那道 `root_slot_is_readable` 早退，R1 删了：最新根按 (txg, 实例) 取最大，环里不会有 txg 比它大 1 的同实例根，那道早退恒不触发，删它不改行为（推的，`walk.rs` 里择最新根那处的 `fold` 严格更大才换）。

### 1.2 逐历史对照（量的：harness 档探针，副本，R1 开）

记录改法与探针见 `probes/probe_b36_tail.rs`、`probe_b36_tail_a.rs`。判法：checker 认 B = I-5.4 红且违例说明以「下一次挂载先施加的那一版」开头（B 的一片分配记录叶里两条记录罩住同一个槽，造法同新用例）；core 施加 B = 只读 `recover` 的 `prefix_applied` ≥ 1。

| 历史 | core | checker | 量的结果 |
|---|---|---|---|
| 发布 B 三条记录（计数器 4、5、6，两块盘各一份）、根槽没落；6 份拷贝落的 64 个子集 | 27 个子集施加 B（前缀 3）、37 个不施加（断号、缺末条） | 同 27 / 37 | 无分歧 |
| 同上，B 的首条写成上一版的新根段（stale_first） | 同上 | 同上，取末条那份 | 无分歧 |
| 同上，末条写成上一版的新根段（stale_last） | 施加 B 27 个子集 | 不红（读末条那份新根段、指着 A 的树） | 按设计：施加出来的那一版就是末条那份 |
| 同上，A 的末条（计数器 3）两份都读不出（锚点认不到，链首靠序号 1 与 txg + 1） | 27 / 37 | 27 / 37 | 无分歧 |
| 同上，B 三条记录的 txg 都写成 5（锚点在） | 施加（只看计数器） | 认 | 无分歧（27 / 37） |
| 同上，txg 写 5 且 A 末条不在 | 不施加（链首要 txg = 4） | 不认 | 64 / 64 无分歧 |
| 首条写成实例 2 的记录 | 不施加 | 不认 | 64 / 64 无分歧 |
| 两条带末条标志的记录坐在 A 的 (1, 3) 上（锚点认不定） | 拒挂：`RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided` | 不认 B，I-4.1 报「锚点认哪一条条款没写」 | 一致 |
| B (1,4)、C (1,5) 各一条记录、两个根槽都读不出：B、C 的拷贝 4 × 4、B、C 第一个点名单元哪一份被翻一个字节 3 × 3 = 144 张 | 施加 ≥ 1 条 36 张、不施加 108 张 | 认 B 36 张、不认 108 张 | 不一致行 `[]` |
| 管理员回退 D (2, 9) 之后再发 E (2, 10)，E 只在记录里：E 的拷贝 4 种 | 任一份拷贝在就施加（prefix=1，落 (2,10)）；都不在不施加 | 同，点名「实例 2、txg 10」 | 一致 |

量的原样输出（命令在 `research/prompts/m3-formal-code-b36-r1-sonnet-model/` 下跑，日志在 `results/`）：

```
$ python3 tools/summarize_p1.py results/t-p1-all.log
行数 576
  clean                        {('checker 不红', 'core 不施加 B'): 37, ('checker 不红', 'core 施加 B'): 27}
  overlap                      {('checker 不红', 'core 不施加 B'): 37, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 施加 B'): 27}
  overlap+stale_first          {('checker 不红', 'core 不施加 B'): 37, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 施加 B'): 27}
  overlap+stale_last           {('checker 不红', 'core 不施加 B'): 37, ('checker 不红', 'core 施加 B'): 27}
  overlap+no_anchor            {('checker 不红', 'core 不施加 B'): 37, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 施加 B'): 27}
  overlap+first_is_instance2   {('checker 不红', 'core 不施加 B'): 64}
  overlap+gen                  {('checker 不红', 'core 不施加 B'): 37, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 施加 B'): 27}
  overlap+txg5                 {('checker 不红', 'core 不施加 B'): 37, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 施加 B'): 27}
  overlap+txg5+no_anchor       {('checker 不红', 'core 不施加 B'): 64}
overlap+gen：掩码数 64 I-5.4 点名下一次挂载那一版而红的掩码 27 I-3.10 红的掩码 27 两集合相等 True
overlap 里 I-5.4 红的掩码集与 overlap+gen 里 I-3.10 红的掩码集相等 True
$ python3 tools/summarize_p1b.py results/t-p1bgrid.log
行数 144
{('checker 不认 B', 'core 不施加'): 108, ('checker 认 B', 'core 施加 ≥1 条'): 36}
不一致行: []
  ('', 'vfail=0', 'prefix=1', '认B') 9
  ('', 'vfail=0', 'prefix=2', '认B') 9
  ('所选根之上一次发', 'vfail=0', 'prefix=0', '不认B') 36
  ('重放前缀停在一条', 'vfail=1', 'prefix=0', '不认B') 72
  ('重放前缀停在一条', 'vfail=1', 'prefix=1', '认B') 18
```

对照臂（证这套探针分得出旧认法）：把 `mutations.tsv` 第 3020 行（`第36批 36-1 R1：下一次挂载先施加的那一版改回只看带末条标志的那一条记录`）施加到副本、重编、重跑同一矩阵（`rerun.sh r1-off`，还原后 `sha256sum -c` 不对的行数 `0`）：

```
$ python3 tools/summarize_p1.py results/t-p1-r1-off.log | grep -v 不一致
行数 128
  overlap                      {('checker 不红', 'core 不施加 B'): 16, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 不施加 B'): 21, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 施加 B'): 27}
  overlap+first_is_instance2   {('checker 不红', 'core 不施加 B'): 16, ('checker 红 I-5.4（点名下一次挂载那一版）', 'core 不施加 B'): 48}
```

旧认法在 21 个子集上 checker 红而 core 不施加 B（b35 判决说的断号那一形），在首条写成实例 2 的 64 个子集里 48 个也红；R1 之后这两格都是 0。

`Stopped` 带第一次发布的那一支在什么历史上走到：前缀整次收下第一次发布 B（含它的点名单元两份都验过）、停在后面某次发布 C 的点名单元上。144 张网格里这一形 18 张（`('重放前缀停在一条', 'vfail=1', 'prefix=1', '认B') 18`），例：

```
PROBE-P1B copiesB=11 copiesC=11 namedB=0 namedC=1 checker_takes_B=true i54=V:下一次挂载先施加的那一版（实例 1、txg 4）那棵分配记录 i41=重放前缀停在一条点名单元两份没都验过 rec=prefix=1,vfail=1,eff=(1,4),FAILED
PROBE-P1B copiesB=11 copiesC=11 namedB=1 namedC=0 checker_takes_B=false i54=H: i41=重放前缀停在一条点名单元两份没都验过 rec=prefix=0,vfail=1,eff=(1,3)
```

第二行是停在第一次发布自己的点名单元上：第一次一条都没收下（`prefix=0`），checker 不认（`Stopped` 带 `None`）。点名单元停在第一次发布中间不会有第一次那一版，与 core「验不过的那一条之前最后一个发布末条」一致。

### 1.3 合法历史上多报少判（量的）

- 合法历史 H2（新池新建文件之后顺序写三个数据单元，崩溃状态 102 个）与 H3（同一历史之后重开可写挂载、实例 2 覆盖写，191 个）：`PROBE-H2 states_with_a_violation=0`、`PROBE-H3 states_with_a_violation=0`；H3 第一条记录带旧根的合法变体 281 个状态：`PROBE-H3S states=281 union_of_red_invariants_over_all_states={}`。
- R1 开 / 关逐状态签名：`h2 逐状态签名 R1 开 / 关不同的行数: 0`、`h3 逐状态签名 R1 开 / 关不同的行数: 0`（102 + 191 个状态，每态一行整套判定字符加说明哈希）。R1 没改合法历史上任何一个状态的任何一条判定。
- 少判：违例坏盘（overlap、gen）在 core 施加 B 的 27 / 27 个子集上都红，I-5.4 红的掩码集与 I-3.10 红的掩码集同为 27 个、两集合相等；没有「core 施加而 checker 不认」的行（上面 9 种改法 576 行、144 张网格、回退之后 8 行里 0 行）。
- 没造出来的格（读码，推的）：所选根 txg 是 u64 最大值（两边都是锚点那一步之后才判 `checked_add`：checker `walk.rs:8642`，core `crates/singlefs-replay-prefix/src/lib.rs:3675`，两边都一条不施加）；超在飞上限的环（上限公式逐字同式，见 1.1）。
- 两块盘同一计数器坐着不同实例的记录：R1 之前后都经 `records_the_replay_reads` 的 (实例代号, 计数器) 键，R1 没碰它，没另造。

### 1.4 判定

- 取的第一次发布与 core 真施加的第一次发布：**一致**（断号、点名单元停在第一次或后面某一次、锚点认不定、几个实例并存、回退之后、txg 跳一格，上表 0 分歧；所选根 txg 无后继、超在飞上限只读码）。
- I-3.10 与 I-5.4：**一致**，同一个值（`walk.rs:11278` 算一次、`walk.rs:11333` 与 I-5.4 版本表共用）。
- 合法历史上多报少判：**一致**（0 / 0）。
- kb 两行与实现的措辞：**冲突**（条款没跟上 R1，写回批 b35 判决第三节第 3 条已排），两句并排整句抄：
  - 「与下一次挂载先施加的那一版（记录与最新根同实例、txg 比最新根大 1、带提交标记，根环里没有它的根）」（`.claude/kb/invariants.md:199`，I-5.4 行）
  - 「与最新根同实例、checkpoint_txg = 最新根的 txg + 1、带提交标记、根环里没有它的根的那条记录」（`.claude/kb/invariants.md:152`，I-3.10 行）
  - 「这一版的 txg 比最新根大，根环里没有它的根。」（`crates/singlefs-checker/src/walk.rs:5871`，实现里 doc 的写法：与锚点在时 core 只看计数器、不要求 txg 比所选根大 1 相符，`overlap+txg5` 27 / 27 同向）
  两处 kb 行写的是「那条带提交标记的记录」，实现认的是「重放施加的第一次发布的末条」；合法历史上两者是同一条，差在断号、txg 跳格这类 core 不施加或照施加而 kb 字面不覆盖的盘面。
- 推翻条件：任何一张镜像上 core 的只读恢复（`recover`）施加了第一次发布而 checker 的 I-5.4 / I-3.10 没读它的分配记录树（或相反，core 不施加而 checker 读它）；矩阵里出现一行分歧即推翻 1.4 前两条。合法历史上出现 R1 开 / 关的签名行不同、或 `states_with_a_violation` 非 0，推翻第三条。

## 二、P2　R1（GPU，36-2 的乙：经交接段第 31 个字传下标）

这一格 GPU 一条没跑（没有对拍、没有 GPU 变异）；下面除 naga 校验外全是读码，**推的**。GPU 内核的行号是 `crates/singlefs-checker-tier/src/` 下三份 wgsl 在冻结副本里的行号。

### 2.1 写字的路径先于扫描内核开工

- 派活：「// 三趟按次序：扫描内核读走读内核留在草稿区交接段里的东西，同一个编码器里的派活按提交次序可见；」（`crates/singlefs-checker-tier/src/crash_judge_dispatch.rs:750`），三趟是恢复、走读、扫描，扫描最后。
- 写：走读内核 I-4.1 那一阶段先调重放，再把第一次施加的发布的末条下标写进第 31 个字：「sws(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED, replay.last_record_of_the_first_publish_applied);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4180`）。
- 读：扫描内核入口先看 H_VALID：「if (sw(H_BASE + H_VALID) == 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:3151`），为 0 就什么都不做；读字的函数是「let replay_index = sw(H_BASE + H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED);」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1099`）。

### 2.2 所有能置 H_VALID == 1 的路径都写了它

- H_VALID 只在一处置 1：「if (reached == WALK_REACHED_THE_HAND_OFF && undecidable == 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4602`）之内；入口先清零「sws(H_BASE + H_VALID, 0u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4619`）。
- `walk_and_hand_off` 交回 `WALK_REACHED_THE_HAND_OFF` 只有两处：「return WALK_REACHED_THE_HAND_OFF;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4297`）紧跟在「undecidable = UNDECIDABLE_TOO_MANY_ENTRIES | (18u << 8u);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4296`）之后，`undecidable != 0` 所以不置 1；另一处是循环结束后的「return WALK_REACHED_THE_HAND_OFF;」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4307`）。
- 循环只在 `phase == WALK_PHASE_DONE` 时退出（「if (phase == WALK_PHASE_DONE) {」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3829`）；`WALK_PHASE_DONE` 只在 `WALK_PHASE_REPLAYED_VERSION` 这一阶段的两处赋值（「phase = WALK_PHASE_DONE;」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4183` 是重放不施加时、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4274` 是走完之后），两处都在 4180 那次写之后。别的阶段中途判不了的退出都是 `WALK_REACHED_THE_ROOT_RING`（`undecidable` 非 0），不置 1。
- 第 31 个字没有别的写者：交接段每个字都按具名常量读写（`grep -n 'H_BASE' *.wgsl` 去掉「sws(H_BASE + H_…,」「sw(H_BASE + H_…)」两种写法之后只剩 `crash_judge_checker_common.wgsl` 里常量定义与注释两行），第 31 号常量只有新加的这一个：「const H_REPLAY_INDEX_OF_THE_LAST_RECORD_OF_THE_FIRST_PUBLISH_APPLIED: u32 = 31u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:734`），与它后面的「const H_BELOW_FLOOR_SLOTS: u32 = 235504u;   // 盘、槽 lo/hi」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:736`，235472 + 32）不重叠。

### 2.3 走读内核 I-4.1 那一处行为不变

照 36-2 的 diff 逐处看（`research/prompts/_m3-formal-code-b36-r1-diff.md` 里 `replay_the_next_mount_would_do` 那一段）：每个原来 `return NONE` 的出口改成交回 `nothing_applied`（两个字段都是 NONE）或 `(NONE, 第一次那条)`，最后 `return (last_applied, 第一次那条)`；调用点仍是「if (last_applied == NONE || undecidable != 0u) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4182`）。`last_applied` 的值域、`NONE` 的位置都与改前相同，I-4.1 走读不变。点名单元停的那一支：「return ReplayTheNextMountWouldDo(NONE, last_record_of_the_first_publish_applied);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3764`）：第一个字段仍 NONE（I-4.1 仍报不适用），第二个字段带第一次发布末条，与 CPU 的 `Stopped` 同形。第一次发布末条的取法：「if (last_record_of_the_first_publish_applied == NONE) {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3772`）里在 `ends_its_publish` 内记 `i`，与 CPU `get_or_insert(record)`（`walk.rs:8718`）同处同义。

### 2.4 扫描内核读到的那一行与 CPU 认的是同一条记录

- 两侧的环是同一份：CPU 的 `records_the_replay_reads` 按 (实例代号, 计数器) 建键、按盘次序先到先得；GPU 的「fn collect_the_records_the_replay_reads(fsid_low: vec2<u32>) -> u32 {」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3566`）同键、同序、已有键就不再收，下标 `i` 即该表里的第 i 行。
- 扫描内核把下标换成行地址：「return C_REPLAY_RECORDS + replay_index * C_REPLAY_RECORD_WORDS;」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1103`），与走读内核自己读这张表的写法同式（「walk_base = checker_record_of_row(C_REPLAY_RECORDS + last_applied * C_REPLAY_RECORD_WORDS).base;」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4188`）。表落在草稿区第 253936 字起，之后没有别的内核表与它重叠：「const C_REPLAY_RECORDS: u32 = 253936u;」（`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:750`），下一张表 `C_APPEARANCE_PLACEMENTS` 从 255984 起（512 行 × 4 字），扫描内核对这张表只读（grep `C_REPLAY_RECORDS` 在扫描内核里只有 1103 一处）。
- I-3.10、I-5.4 两处消费者都改读行地址：`judge_allocation_records_disjoint` 与 `judge_allocation_generations_against_unit_births` 里「let next_version = record_row_of_the_version_the_next_mount_applies_first();」（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:978`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1086`），随后用 `checker_record_of_row` 解、取新根段偏移处的树表指针；旧的 `checker_record_at(next_version)` 在扫描内核里没有残留（36-2 diff 里两处都换了）。

### 2.5 naga 校验（量的，不是 GPU）

两个内核照 `crates/singlefs-checker-tier/src/crash_judge_gpu.rs` 的 `checker_walk_kernel_source` / `checker_scan_kernel_source` 拼法拼好（头五个常量取值 14、8、4、12、13 抄自该文件 `VERDICT_WORDS` 等定义），交 naga 30.0.1（仓里 `Cargo.lock` 钉的版本）解析并 `ValidationFlags::all()` 校验，与仓里单测 `parse_and_validate` 同一套调用：

```
walk-unmutated rc=0 OK … functions=252 entry_points=1
scan-unmutated rc=0 OK … functions=231 entry_points=1
mut-4-scan rc=0 OK … functions=231 entry_points=1
mut-5-walk rc=0 OK … functions=252 entry_points=1
mut-6-scan rc=0 OK … functions=231 entry_points=1
mut-7-walk rc=0 OK … functions=252 entry_points=1
total rc 0
VALIDATE-FAIL …/wgsl/walk-broken.wgsl
walk-broken rc=3
PARSE-FAIL …/wgsl/scan-broken.wgsl
scan-broken rc=2
```

后四行是校验路径自己会红的对照：构造器少一个参数、引用不存在的常量，必须判红，判红了。`mut-4..7` 是 36-2 点名的四行 GPU 变异（追加 2、替换 2）施加到拼好的源码后仍能解析与校验：变异不是编译失败的假红（P5 用）。

### 2.6 判定

- 写字路径先于扫描内核、所有能置 H_VALID 的路径都写了它：**一致**（推的）。
- 走读内核 I-4.1 那一处行为：**一致**（推的，diff 逐处看）。
- 扫描内核读到的行与 CPU 认的是同一条记录：**一致**（推的；同一份重放、同键同序，唯一的差在 GPU 在 `undecidable` 时整趟不置 H_VALID、回落 CPU，那与改前相同）。
- 核不动：GPU 实际执行与对拍；36-2 改的对拍用例整份的编译（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`，checker 档，不跑；背景材料第二节引合入验证 fmt、clippy 过，我没有复跑）；各线合入后扫描内核超表达式预算（背景材料第二节，归根环线）。
- 推翻条件：GPU 对拍里任一状态上扫描内核红 / 不红 I-5.4 或 I-3.10 与 CPU 不同（尤其断号那条流、点名单元停在 C 上那条流）；或某条能置 H_VALID 为 1 的路径没经过 4180 那次写（读到上一个状态留在草稿区的陈旧下标）。

## 三、P3　R2（对拍用例「拿掉根槽写」两条流的新造法）

对象：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`（checker 档，不跑）。新造法照 36-2 diff 逐行搬进 harness 档探针 `probes/probe_b36_tail_r2.rs`（只把 checker-tier 的 `RecordedFlow` 换成三元组；造法逻辑——流截到 txg 4 的根槽 FUA 之前、从全落的崩溃镜像起新进程可写挂载、再 `publish_overwrite`——一行没改），旧造法取 b35 探针 `h3_flow`（整段历史录完、再拿掉 txg 4 的根槽写）。

### 3.1 与合法历史逐字节一致吗

合法历史 = 崩在 B 的根槽 FUA 之前（记录三条两块盘都落、根槽没落、之后的系统配置轮换没发生），再重启挂载、覆盖写。新造法的前一半是流的前缀（截在根槽写之前，那一步之前是真发布写出的 77 个写），后一半是 core 的 `mount_writable` 与 `publish_overwrite` 在这张崩溃镜像上真写出的 96 个写；两半之间没有手造的字节。量的（`results/t-r2.log`）：

```
PROBE-R2 writes: old=175 new_before_mount=77 new_new_process=96 overwrite=(inst 2, txg 8)
PROBE-R2 max_transaction_in_the_records_of_B(read_from_record_bytes)=4
PROBE-R2 old_final chosen_root=(2,8) instance_rows=[(1, 4, 0)]
PROBE-R2 new_final chosen_root=(2,8) instance_rows=[(1, 4, 4)]
PROBE-R2 sysconf_writes_in_stream_order old=[(0, 1), (0, 1), (1, 1), (1, 1), (2, 1), (2, 1), (3, 1), (3, 1), (6, 1), (6, 1), (6, 2), (6, 2), (7, 2), (7, 2), (8, 2), (8, 2), (9, 2), (9, 2), (10, 2), (10, 2)]
PROBE-R2 sysconf_writes_in_stream_order new=[(0, 1), (0, 1), (1, 1), (1, 1), (2, 1), (2, 1), (3, 1), (3, 1), (3, 2), (3, 2), (7, 2), (7, 2), (8, 2), (8, 2), (9, 2), (9, 2), (10, 2), (10, 2)] (崩溃之前那一段 8 个)
PROBE-R2 legal_crash_image_before_mount(new) chosen_root=(1,3) instance_rows=[]
PROBE-R2 old_final_vs_new_final 不同的槽合计 11
```

- W：实例 1 那一行（实例、所选根 txg、W）旧造法 `(1, 4, 0)`、新造法 `(1, 4, 4)`；B 三条记录里的最大事务号我直接读记录字节（偏移 78，8 字节，不经 core 挂载）得 `4`。新造法的 W = 4 与之相符，旧造法写的 0 是录制时挂载读到 B 的根、什么都没施加留下的（b35 攻方量的那一处）。条款侧：「可写挂载给上一个实例写行时 W 取它」（`crates/singlefs-core/src/recovery.rs:331`，`maximum_applied_transaction`）。
- 系统配置槽：旧造法流里 B 之后多一对 `(6, 1)`（journal_tail 6、实例 1，即 B 发布那次根槽之后的轮换），新造法里没有，取号那一写是 `(3, 2)`（见证到的 tail 3、实例 2）。崩在根槽 FUA 之前，B 的轮换本来就不该有——旧造法是「根槽没落而系统配置已轮换」（b35 攻方量的第二处），新造法不是。崩溃镜像（挂载之前）里两块盘的系统配置槽 `journal_tail` 是 2 与 3、实例 1：B 发布的那次轮换 `(6, 1)` 不在里面。
- 其余差：新旧末态 16 KiB 槽共 11 个不同（journal 环 4 个：盘 0、1 的槽 1025 与 1026；单元区 2 个：盘 0、1 的槽 50304；固定区 2 个：盘 0、1 的槽 0；根环 3 个：盘 0 槽 64、448，盘 1 槽 256）。我只解释了 W 与系统配置轮换序列这两处；别的 9 个槽没有逐字节解释，只知道它们不让末态的整套 CPU 判定变（3.2）。

### 3.2 末态判定与原造法相同吗

```
PROBE-R2 final_state_red old=[] new=[]
PROBE-R2 final_state_partial old=[] new=[]
PROBE-R2 final_state_verdicts_that_differ=[]
```

末态（所有写全落）新旧两张镜像每条不变量的判定与说明文字逐条相同，差集空。（对拍用例逐状态比 GPU 与 CPU，逐状态的 CPU 判定我没有逐个状态跑，只比了末态；见「没做什么」。）

### 3.3 两条流名字说的那一形还在不在（量的，R1 开）

- 流 1「第一条记录带上一版的根、由记录施加出来的那一版取末条」：探针把 B 第一条记录的新根段七项换成 A 的、重封重链，造崩溃镜像、再起新进程挂载。
  ```
  PROBE-R2F1 crash_image_recover=prefix=3,vfail=0,eff=(1,4) final_recover=prefix=0,vfail=0,eff=(2,8) overwrite_txg=8
  PROBE-R2F1 final_red=[] final_partial=[]
  ```
  崩溃镜像上恢复施加三条、落 (1,4)（取末条那份新根段，第一条旧也不挡）；挂载之后实例 1 的行在（`flow1_final instance_rows=[(1, 4, 4)]`）、所选根是 (2,8)，B 的根槽写在流外（造法保证根环里没有 (1,4) 的根），B 是「由记录施加出来的那一版」；末态零红。
- 流 2「树表指针全零」：新造法里记录是干净的、挂载读到干净的三条，改记录发生在挂载之后（对拍用例在流尾另起一段抄一遍重封；探针在末态镜像上直接改同样三条、同盘 4→5→6 重链）。
  ```
  PROBE-R2F2 final_before_the_appended_segment red=[]
  PROBE-R2F2 final_after_the_appended_segment red=["I-2.1", "I-3.1", "I-4.8", "I-7.4"]
  PROBE-R2F2 I-4.8: 由记录施加出来的那一版（实例 1、txg 4）出发的遍历有单元对不上或读不出
  PROBE-R2F2 same_rewrite_on_the_crash_image_before_the_mount red=["I-4.1"] recover=prefix=3,vfail=0,eff=(1,4),FAILED:UnitUnreadable { slot: SlotNumber(0) }
  ```
  对拍用例断言要红的 {I-2.1、I-4.8、I-7.4} 全在，红在「由记录施加出来的那一版（实例 1、txg 4）」上（另多红一条 I-3.1，是这一版走不完、记账对不上它的单元的连带，用例只断言「包含」）；同一份改动落在挂载之前那张崩溃镜像上，B 是下一次挂载先施加的那一版，红的是 I-4.1（用例里挂载前后各一种形）。

### 3.4 判定

- 新造法与合法历史逐字节：**一致**（W、系统配置轮换两处旧造法不对、新造法对；其余差异是连带）。条件：合法历史取「崩在根槽 FUA 之前」这一种崩溃，新造法的后半是 core 真写的，所以「逐字节一致」是由造法保证、再加 W 与轮换序列两处独立读值印证，不是拿另一份手造的合法镜像逐字节比。
- 末态判定相同：**一致**（只比了末态）。
- 两条流名字说的那一形：**一致**。
- 核不动：GPU 与 CPU 逐状态对拍；R2 改的那个测试文件整份编译（checker 档）。
- 推翻条件：把新造法后半挂到别的崩溃点（根槽之前、记录只落一半）后，实例 1 那一行 W 不等于该点上 core 施加记录的最大事务号；或对拍用例里某个状态上 CPU 判定与旧造法同一状态不同而不是只多出挂载之后的内容；或流 2 全落状态上 I-2.1、I-4.8、I-7.4 任一不红。

## 四、P4　R3（三处说明与 I-5.5 理由）

「三处说明」取 walk.rs 里 R1 重写的三段：函数 `version_the_next_mount_applies_first` 的 doc（`walk.rs:5861-5874`，认法）、`tree_table_pointer_of_the_version_the_next_mount_applies_first` 的 doc（`walk.rs:5842-5844`）、`ReplayTheNextMountWouldDo` 的 `Stopped` 与 `Applied` 两处 doc（`walk.rs:8486-8496`）；I-5.5 理由是常量与它两处注释。

- 认法 doc 与判定：§1 全部对照量过：「这里用同一份重放，那次发布同样不认（此前只看带末条标志的那一条，断号的发布 core 不施加、这里照认）」（`crates/singlefs-checker/src/walk.rs:5869`）= 断号 64 个子集里 37 个不施加、checker 不红；「前缀停在后面某次发布的点名单元上时，停之前整次收下的发布」（`crates/singlefs-checker/src/walk.rs:5870`，下一行接「core 照施加，第一次那一版照认」）= 网格 18 张 `prefix=1,vfail=1` 都认 B。
- `Stopped` doc：「停之前已经整次收下的发布 core 照施加（`crates/singlefs-core/src/recovery.rs` 的 `replay_journal`：前缀停在验不过的那一条之前，」（`crates/singlefs-checker/src/walk.rs:8487`）= core `replay_core` 的「applied 是前缀按最后一个发布末条截断」，`verification_failed += 1` 之后 `break`（`crates/singlefs-replay-prefix/src/lib.rs:3849`），`recover` 落 (1,4)、`prefix=1,vfail=1`（量的）。
- `tree_table_pointer_of_…` 的 doc：「I-3.10 经这里读，I-5.4 按版本那一层读同一个函数认出的同一版。」（`crates/singlefs-checker/src/walk.rs:5844`）= `walk.rs:11278` 算一次、`11333` 与版本表共用（§1.1）。
- I-5.5 理由（`crates/singlefs-checker/src/walk.rs:5190`）：「…认出来的那几版在池里的盘上都没有数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上）：一版都不判」。理由只在「认出来而一版都没判到」时出现：`into_report` 里违例、部分没判、成立都压过不适用（`crates/singlefs-checker/src/image.rs:351-374`），所以「不压过违例、部分没判、成立」成立。理由里三样各对应一处 `continue`：extent 树根全零（「if extent_root.all_zero {」，`crates/singlefs-checker/src/walk.rs:5089`）、一条非零数据指针都没有（「if smallest_data_unit_slot_on_each_device.is_empty() {」，`crates/singlefs-checker/src/walk.rs:5112`）、落点全在池里没有的盘上（逐盘「if let Some(&smallest_slot) = smallest_data_unit_slot_on_each_device.get(device) {」，`crates/singlefs-checker/src/walk.rs:5141`，池里没有的盘不进 `reader.devices()`、一次都不评估）。量的：extent 树根指针写零的镜像报 `I-5.5=N 认出了从 mkfs 水位第一次建树的那一版，但认出来的那几版在池里的盘上都没有数据单元（…）：一版都不判`（`results/t-r3.log`），`mkfs_only` 报另一句（没有认出来的版本），`first_file_clean` 成立。后两样（一条非零指针都没有、落点全在池外）只读码、没造镜像（推的）。
- kb 与实现冲突（写回批已排）：
  - 「认出来而没有数据单元那一形仍落缺省理由」（`.claude/kb/invariants.md:200`，I-5.5 行）
  - 「pub const VERSIONS_BUILDING_TREES_FROM_THE_MKFS_WATERMARK_HAVE_NO_DATA_UNIT: &str = "认出了从 mkfs 水位第一次建树的那一版，但认出来的那几版在池里的盘上都没有数据单元（extent 树根全零、一条非零的数据指针都没有，或数据指针的落点全在池里没有的盘上）：一版都不判";」（`crates/singlefs-checker/src/walk.rs:5190`）
- 判定：三处说明与判定逐句一致（**一致**，量的）；理由措辞与它走到的三种形一致（**一致**，一种量的、两种推的）；kb I-5.5 行「仍落缺省理由」与实现**冲突**（kb 没跟上第 34 批 Q4，R3 之后措辞也变了）。别处残留：`crates/singlefs-checker/src/image.rs:211-212` 的 doc 写 `walk` 的 `version_the_next_mount_applies`，仓里没有这个函数名（`grep -n 'fn version_the_next_mount_applies\b' crates -r` 零命中，真名 `replay_the_next_mount_would_do` 与 `version_the_next_mount_applies_first`），不归这一轮（R1 之前就有）。
- 推翻条件：造出一张镜像，I-5.5 报这句理由而它的实际原因不是上面三样（例如认出来的那版有数据单元、也有判到的盘）；或 `into_report` 的优先次序变了，这句压过了违例。

## 五、P5　变异行（追加 4 行、替换 4 行）

八行都是 `crates/mutations.tsv` 里第 36 批的行：追加 3020、3021（harness 档）、3022、3023（checker 档 GPU）；替换 963（`实审 B2`）、2927（`第35批 35-1 FL`）、2855（`第34批 34-1 GPU 判器 RV`）、2934（`第35批 35-1 GPU 判器 FL`）。

### 5.1 整行与改动全文相同、锚点各恰好命中一次

```
$ python3 tools/select_rows36.py <冻结副本> research/prompts/_m3-formal-code-b36-r1-diff.md
0 m3-formal-36-1b appe 第36批 36-1 R1：下一次挂载先施加的那一版改回只看带末条标志 [3020] same singlefs-checker hits=1 harness
1 m3-formal-36-1b appe 第36批 36-1 R1：重放停在后面一次发布的点名单元上时不认停之 [3021] same singlefs-checker hits=1 harness
2 m3-formal-36-1b repl 实审 B2（A2a 第 25 条对齐，前缀口径）：认下一次挂载先施加 [963] same singlefs-checker hits=1 harness
3 m3-formal-36-1b repl 第35批 35-1 FL：下一次挂载先施加的那一版照旧取先到的那条记 [2927] same singlefs-checker hits=1 harness
4 m3-formal-36-2 appe 第36批 36-2 GPU 判器 R1：扫描内核认下一次挂载先施加的 [3022] same singlefs-checker-tier hits=1 checker-tier
5 m3-formal-36-2 appe 第36批 36-2 GPU 判器 R1：走读内核那一份重放停在后面一 [3023] same singlefs-checker-tier hits=1 checker-tier
6 m3-formal-36-2 repl 第34批 34-1 GPU 判器 RV：I-5.4 不收下一次挂载先 [2855] same singlefs-checker-tier hits=1 checker-tier
7 m3-formal-36-2 repl 第35批 35-1 GPU 判器 FL：下一次挂载先施加的那一版照旧 [2934] same singlefs-checker-tier hits=1 checker-tier
```

整表另扫一遍（`tools/anchors_all.py`）：

```
$ python3 tools/anchors_all.py <冻结副本>
行数 3023 列数不是 6 的行 5
锚点不是恰好命中一次的行: [(1352, '源文件不存在', 'litmus/publish-returns-after-the-rotation-barrier.litmus'), (2919, 0, '第26批 inode 树读不出孩子时 I-9.2 不判（类型段不认得的条目只记 I-2.1 与走读失')]
```

2919 是删除线第 26 批那一行，背景材料第二节已说不归这一轮；1352 是 litmus 行（源文件不是 crates 下的代码，列数也不同），与这一轮无关。R1 重写了 `version_the_next_mount_applies_first`，旧锚点没有一条因此落空（除被替换的两行，它们的新锚点命中 1 次）。

### 5.2 点名的测试跑得到、合入后基线绿（harness 档量的，GPU 档推的）

```
$ bash tools/rerun.sh tests
test a_record_whose_back_chain_does_not_match_is_not_the_version_the_next_mount_applies_first ... ok
test a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first ... ok
test a_stale_first_record_of_the_publish_the_next_mount_applies_first_does_not_hide_two_allocation_records_covering_one_slot ... ok
test a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 157 filtered out; finished in 6.35s
```

两条新用例在 `crates/singlefs-harness/tests/checker_known_bad_images.rs:12367`（「fn a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first() {」）与 `:12487`（「fn a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first(」）。GPU 四行点名的四个用例：新两条（`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里 `the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_…`、`…_that_a_replay_stopped_by_a_named_unit_…`）与 34-1 RV、35-1 FL 两条老的（`:4928`、`:8758`）都在文件里（`grep -c` 命中 2 + 各 1）；命令里的 `--features verdict-store,gpu` 在 `crates/singlefs-checker-tier/Cargo.toml` 的 `[features]` 里有（「gpu = ["dep:wgpu", "dep:pollster"]」，`crates/singlefs-checker-tier/Cargo.toml:29`）。GPU 用例基线没跑（checker 档，推的）。

### 5.3 红在断言上（harness 档 4 行量的）

每行施加到副本、跑点名的用例、还原（还原后 `sha256sum -c` 不对的行数 `0`）：

```
0 101 断言（或自带消息 panic） named_failed= True compiled= True ok crates/singlefs-harness/tests/checker_known_bad_images.rs:12409:5:
1 101 断言（或自带消息 panic） named_failed= True compiled= True ok crates/singlefs-harness/tests/checker_known_bad_images.rs（第二条用例里「只红 I-5.4」那一格）
2 101 断言（或自带消息 panic） named_failed= True compiled= True ok crates/singlefs-harness/tests/checker_known_bad_images.rs:5942:5:
3 101 断言（或自带消息 panic） named_failed= True compiled= True ok crates/singlefs-harness/tests/checker_known_bad_images.rs（「第一条记录写旧了也不许把 B 的两条记录罩住同一个槽藏起来：只红 I-5.4」那一格）
```

四行都是 `exit=101`、点名的用例 `FAILED`、断言 `left == right` 失败（不是越界或 unwrap panic，也不是编译失败：编译日志里被测包有 `Compiling`）。第 0 行红在 `B 断了号、core 不施加：…两处坏法都不红` 那一格，第 1 行红在 `重放停在 C 之前，B 照认：它的两条记录罩住同一个槽只红 I-5.4`，第 2 行红在 `B 那条记录进不了前缀：只该红 I-8.6`，第 3 行红在 `第一条记录写旧了也不许把 B 的两条记录罩住同一个槽藏起来`。逐行原样见 `results/mut-results.jsonl`。
GPU 四行没跑（checker 档）：能说的是它们施加到拼好的两个内核后 naga 仍解析、校验通过（2.5 的 `mut-4..7`），不是编译失败的假红；分辨力是类比 CPU 一侧——第 4 行（扫描内核改回只看带末条标志的那条）对应 CPU 第 0 行、第 5 行对应第 1 行、第 7 行对应第 3 行，CPU 三行都红；对拍的状态枚举里含这些状态（断号的子集在首条两份都不落的那段里，点名单元停在 C 上的状态在流尾那段全落时有）。**推的**。

### 5.4 判定

- 追加 4 行、替换 4 行的行与锚点：**一致**（命中 1 次 × 8；整表只有不归这一轮的 2919 与 litmus 行）。
- 点名的测试跑得到、基线绿：harness 4 行**一致**（量的）；GPU 4 行**一致**到「名字在、feature 在」，基线没跑（推的）。
- 红在断言上：harness 4 行**一致**（量的）；GPU 4 行没验。
- 推翻条件：任一行施加后点名的用例仍绿（变异存活）、或红在 panic 越界 / 编译失败而不是断言；GPU 对拍里去掉 `R_DATA` 之外那一处扫描内核读重放第一次施加那一行后对拍仍绿。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| P1 取的第一次发布与 core 真施加的是同一份 | 一致 | 9 种改法 576 行、网格 144 张、回退之后 8 行，checker 认 B ⇔ core 施加第一次发布，0 分歧；R1 关掉对照臂 21 行分歧（量的） |
| P1 `Stopped` 带第一次那一支 | 一致 | 第一次整次收下、停在后面某次发布的点名单元上：网格 18 张 `prefix=1,vfail=1` 都认 B；停在第一次自己的点名单元上 72 张都不认 |
| P1 锚点认不定、几个实例并存、txg 跳格 | 一致 | 两条带末条标志拒挂与不认；首条写成实例 2 的 64 / 64 不认；txg 写 5 锚点在 27 / 27 同认、锚点不在 64 / 64 同不认 |
| P1 所选根 txg 没有后继、超在飞上限 | 一致（推的） | 只读码：两边都在锚点之后判 `checked_add`；上限公式逐字同式，没造镜像 |
| P1 I-3.10 与 I-5.4 仍是同一个认法 | 一致 | 同一个值，`walk.rs:11278` 算一次、`11333` 与版本表共用；红的掩码集相同（27 = 27，量的） |
| P1 合法历史上多报少判 | 一致 | H2 102 态、H3 191 态、H3S 281 态 0 违例；R1 开 / 关逐状态签名 0 行不同 |
| P1 kb I-5.4、I-3.10 两行的措辞 | 冲突 | kb 写「txg 比最新根大 1、带提交标记的那条记录」，实现认「重放施加的第一次发布的末条」；两句并排见 1.4（写回批已排） |
| P2 写那个字的路径先于扫描内核、所有能置 H_VALID == 1 的路径都写了它 | 一致（推的） | 三趟按次序、字在 `walk_and_hand_off` 的 REPLAYED_VERSION 阶段写、H_VALID 只在该阶段之后置 1 |
| P2 走读内核 I-4.1 那一处行为不变 | 一致（推的） | 原来每个 `return NONE` 出口改交 `(NONE, …)`，调用点判 `last_applied == NONE` 不变 |
| P2 扫描内核读到的行与 CPU 认的是同一条记录 | 一致（推的） | 同一份重放环、同键同序，下标换成行地址，唯一的读者是 `record_row_of_the_version_the_next_mount_applies_first` |
| P2 内核源码能解析与校验 | 通过（量的，naga） | 两个内核与 GPU 四行变异后源码 `rc=0`，故意写坏的两份判红；GPU 实际执行与对拍没跑 |
| P3 新造法与合法历史逐字节一致 | 一致 | 实例 1 行 W 旧 0、新 4（读记录字节得 4）；系统配置轮换序列旧多一对 `(6, 1)`、新没有；造法是真崩溃加真重启 |
| P3 末态判定与原造法相同 | 一致 | 末态整套判定差集空（只比了末态） |
| P3 两条流名字说的那一形还在不在 | 一致 | 流 1 施加三条落 (1,4)、末态零红；流 2 红 {I-2.1、I-3.1、I-4.8、I-7.4}，用例要的三条都在，红在「由记录施加出来的那一版」上 |
| P4 三处说明与判定一致 | 一致 | 认法 doc、`tree_table_pointer_of_…` doc、`Stopped` / `Applied` doc 逐句对 §1 的量过 |
| P4 I-5.5 新理由措辞与判定一致 | 一致 | 三种形各对应一处 `continue`；扩了的一种量的、两种只读码；违例、部分没判、成立都压过它 |
| P4 kb I-5.5 行「仍落缺省理由」 | 冲突 | kb 没跟上第 34 批 Q4 与 R3，两句并排见第四节（写回批已排） |
| P5 锚点各恰好命中一次 | 一致 | 8 行 `hits=1`、整行与改动全文相同；整表只有不归这一轮的 2919 与 litmus 行 |
| P5 点名的测试跑得到、基线绿 | harness 一致、GPU 推的 | harness 4 条 `4 passed`；GPU 4 条名字与 feature 在，没跑 |
| P5 红在断言上 | harness 一致、GPU 没验 | 4 行都 `exit=101`、点名用例 FAILED、`assertion left == right`；GPU 4 行只验了能过 naga |

## 七、没做什么

- 不判别：没出判决、没替主 agent 采纳；攻方腿的立场（找反例）没替；禁读清单里的文件一个没读。
- 重型测试一条没跑：checker 档测试、GPU 对拍、`crates/mutations.tsv` 整表、层 0。GPU 的一切（P2 全部、P3 的 GPU 比较、P5 的 4 行 GPU 变异）都是读码（推的）或 naga 解析校验。
- 没复跑 checker 档测试文件的编译：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs` 里 36-1b、36-2 两处改动（R2 的新造法与四条新用例）是否编过，我只有背景材料第二节转述的合入验证 fmt、clippy 过，没有复核。
- 没造的镜像：所选根 txg 是 u64 最大值、超在飞上限的环、I-5.5「一条非零数据指针都没有」「落点全在池外」两种、两块盘同一计数器坐着不同实例的记录；这些格只读码。
- R2 只比了末态的 CPU 判定，没有逐状态把新旧造法的 CPU 判定各跑一遍；新造法流在对拍用例里被枚举出的全部崩溃状态上 GPU 与 CPU 是否逐位一致，核不动。
- 没量 R1 的耗时：`version_the_next_mount_applies_first` 现在在每张镜像上自己重放一遍（I-4.1 又重放一遍，两遍读同一批点名单元），层 0 每个崩溃状态都判，耗时翻没翻、翻多少没量；不是正确性问题。
- 没碰 kb 与代码；kb 两行（I-3.10、I-5.4 的措辞，I-5.5 的理由）与 `crates/singlefs-checker/src/image.rs:211-212` 里一个不存在的函数名只报告。
- 草稿与仓副本：`~/.local/share/singlefs/work/m3-formal-code-b36-r1-sonnet/`（含 `rerun/`、`work/`、`target/`、`target-opt/`、`target-naga/`、`naga-check/`、`draft/`），不入库，原因：编译目录与仓副本只用来复跑，模型与原样输出已落 `research/prompts/m3-formal-code-b36-r1-sonnet-model/`；交回前由我在报告末尾逐个写全路径并说明不删的理由。

## 八、要主 agent 跑的命令

全是 checker 档（要一张 NVIDIA 独显），子 agent 不跑，提交时由主 agent 带 `SINGLEFS_HEAVY_TESTS=commit` 跑；工作目录仓根，上限与线程数按它的派发定，跑完看 `test result` 行：

1. 两条新对拍流（P2、P3 的 GPU 一半）：`cargo test -p singlefs-checker-tier --features verdict-store,gpu --test crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state -- --exact the_gpu_judge_agrees_with_the_cpu_that_a_publish_whose_first_record_did_not_land_is_not_the_version_the_next_mount_applies_first`，与同样写法的 `…_that_a_replay_stopped_by_a_named_unit_of_a_later_publish_still_takes_the_first_publish_as_the_version_the_next_mount_applies_first`。要红的是 `mismatches` 非空；绿 = 过。
2. 受 R2 影响的两条老流：`the_gpu_judge_agrees_with_the_cpu_that_a_version_applied_only_by_its_records_is_walked_from_its_last_record_when_the_first_carries_the_previous_root` 与 `the_gpu_judge_agrees_with_the_cpu_when_the_records_of_a_version_applied_only_by_them_carry_an_all_zero_tree_table_pointer`。
3. GPU 四行变异（`crates/mutations.tsv` 3022、3023、2855、2934）逐行施加到 `crates/singlefs-checker-tier/src/` 的 wgsl 后跑各自点名的用例，要红在对拍断言上。

## 九、交回前清理（`.claude/singlefs-ai-sop/rules/session-wrapup.md` 第 5 条）

交回之前由我删掉自己建的编译目录与仓副本，删前 `du -sh`：
- `~/.local/share/singlefs/work/m3-formal-code-b36-r1-sonnet/work` 20M、`…/target` 2.4G、`…/target-opt` 1.5G、`…/target-naga` 205M；
- `~/.local/share/singlefs/work/m3-formal-code-b36-r1-sonnet/rerun/work` 20M、`…/rerun/target` 2.4G、`…/rerun/target-opt` 1.5G、`…/rerun/target-naga` 207M、`…/rerun/wgsl` 2.8M、`…/wgsl` 2.1M（拼好的内核源码与变异后的内核源码，`tools/wgsl_validate.py` 可重生成）。

没删：`~/.local/share/singlefs/work/m3-formal-code-b36-r1-sonnet/draft/`（报告草稿，112K）、`…/rerun/*.log` 与同目录各 `t-*.log`（原样输出，已拷进 `research/prompts/m3-formal-code-b36-r1-sonnet-model/results/`，留作对照）、`…/naga-check`（24K，一个 Cargo 小工程的源码，不是编译目录）；`~/.local/share/singlefs/work/m3-formal-code-b36-r1/tree` 是主 agent 的冻结副本，不归我。
