# m3-verus-land-code-r2 正推腿（Sonnet）报告（2026-10-01）

读的是冻结副本 `/tmp/claude-1000/m3-verus-land-code-r2/tree/`（改后）；改前取 `git show fcc4d340:crates/singlefs-core/src/recovery.rs` 的原样拷贝，入库在 `research/prompts/m3-verus-land-code-r2-sonnet-model/recovery_fcc4d340.rs`（下文「改前」指它，引改前行号都指它的行号）。没读主仓 `crates/`、没读攻方腿的产出。
模型、装置、日志在 `research/prompts/m3-verus-land-code-r2-sonnet-model/`（README.md 一页说明）。线程一律 6，跑编译出来的代码都经 `run-with-memory-cap.sh 8G` 与 `capped.sh 6`；没跑 Verus、没跑 checker 档、没跑 E165 装置（见末尾「没做什么」）。
本轮判据照正文第五节：H1 改后与改前在产品输入上结局不同 ⇒ 打中；H3 一种核失败映射错或没有成员 ⇒ 打中。

## 一、H1　改后核与 fcc4d340 那一版等价

### 1.1 量法

沿用第一轮正推的差分装置（`m3-verus-land-code-r1-sonnet-model/zz_diff.rs` 的拷贝，改动三处：Tally 多记「只有改后 panic」「只有改前 panic」「两边都不 panic 而结局不同」，多记两种报错的条数，多一条「盘上读出的计数器小于 2^48」测试）。把改前的 `replay_journal`（`gen_old.sh` 取改前的 record_ends_its_publish、readable_record_of_the_same_publish_follows、counter_of_the_last_record_the_root_covers、replay_journal，改名成 `*_before`）放进冻结副本的拷贝，与改后的 `replay_journal` 在同一个内存读盘器上各跑一次，比：返回值整体相等（`JournalScanReport`、`RootRecord`、`RecoveryFailure` 都 `PartialEq`，含报告五字段、`valid_records`、重建的根、报错种类与内容）、读盘日志相等（盘号、偏移、长度的次序）、两边是否 panic 相同。每条记录的 `new_tree_identifier_watermark` 各不相同，施加的是哪一条的新根段分得出来。
输入：所选根 instance 0–2 与 txg（含 0、`u64::MAX`、`u64::MAX - 1`）、0–9 条记录、实例与 txg 乱跳、跳号、重复键、序号乱、末条标志乱、提交标记缺、反向链坏、前一条在两块盘上的六种在盘状态、点名项类别与两份位置的三态、在飞上限 0–13、点名校验开关；穷举一族与第一轮同。

### 1.2 原样结果（`logs/` 里的日志逐行抄的）

产品输入族（计数器都小于 2^48，键等于记录字段）：
```
test recovery::zz_diff::random_family ... RESULT[random] Tally { total: 3000000, diverged: 0, new_panics_only: 0, old_panics_only: 0, both_ok_differ: 0, panics_old: 0, panics_new: 0, errs: 327781, applied_some: 271293, verif_failed: 290134, anchor_err: 0, shown: 0 }
test recovery::zz_diff::random_family ... RESULT[random] Tally { total: 2000000, diverged: 0, new_panics_only: 0, old_panics_only: 0, both_ok_differ: 0, panics_old: 0, panics_new: 0, errs: 218742, applied_some: 180956, verif_failed: 193428, anchor_err: 163996, txg_err: 54746, other_err: 0, shown: 0 }
test recovery::zz_diff::exhaustive_chain_and_named ... RESULT[chain+named k=3] Tally { total: 47775744, diverged: 0, new_panics_only: 0, old_panics_only: 0, both_ok_differ: 0, panics_old: 0, panics_new: 0, errs: 11833344, applied_some: 218784, verif_failed: 2118414, anchor_err: 11833344, txg_err: 0, other_err: 0, shown: 0 }
RESULT[structural k=1 inst=true verify=false slots=40] Tally { total: 480, diverged: 0, ... }
RESULT[structural k=2 inst=true verify=true slots=40] Tally { total: 46080, diverged: 0, ... }
RESULT[structural k=3 inst=false verify=true slots=40] Tally { total: 552960, diverged: 0, ... }
RESULT[structural k=3 inst=false verify=false slots=40] Tally { total: 552960, diverged: 0, ... }
RESULT[structural k=3 inst=false verify=true slots=6] Tally { total: 552960, diverged: 0, ... }
RESULT[structural k=3 inst=true verify=true slots=40] Tally { total: 4423680, diverged: 0, ... }
```
（第一行是种子 1、第二行种子 2；第一行的 `errs`、`applied_some`、`verif_failed` 与第一轮同种子的数逐个相同；穷举结构一族六行的 `...` 处各列也是 0，原样见 `logs/batch2a.txt`。）总数 3 000 000 + 2 000 000 + 6 129 120 + 47 775 744 = 58 904 864（命令算的：`python3 -c "print(3000000+6129120+47775744+2000000)"` → 58904864），`diverged` 全 0，`panics_old`、`panics_new` 全 0。两种报错两种映射在同一批里各出现过：种子 2 那一行 `anchor_err: 163996`、`txg_err: 54746`、`other_err: 0`。
装置会不会红（阳性对照）：
- 改前的四个变异体（去掉 `.take(in_flight_limit)`、点名失败不断、序号 `FIRST` 改 `>2`、最大事务号只看第一条）各 40 000 个输入，抓到 1581、3821、545、520 个（`logs/batch1.txt` 的 `CONTROL[...]` 四行）。
- 对改后核本身做 22 个单点改动（`mutate_new_core.py`，每个先断言旧串恰好命中一次，跑完换回原文；每个用随机 150 000 个加结构快档约 110 万个输入），22 个全红，合计 diverged 数见 `logs/mutate_new_core_out.txt`：limit_gt 6712、backchain_off 218810、first_ordinal_unchecked_open 45825、txg_change_unchecked 4706、tx_change_unchecked 1057、last_without_commit_unchecked 22080、followers_unchecked 18175、named_skipped 18801、passed_plus_two 96970、above_water_plus_two 774297、root_not_updated 32102、applied_count_one 4381、expected_next_plus_two 66954、anchor_error_needs_three 334552、no_successor_off 4064、above_ge 730980、ordinal_step_two 13381、head_ordinal_unchecked 20075、head_txg_unchecked 58871、verification_failed_not_counted 14525、follower_same_txg_blind 18175、maximum_takes_minimum 32102。
所以差分对「改后核」每一处判定、每一个报告字段都分得出来；产品输入上改后与改前零差异，**H1 在产品输入上没打中**。
推翻它的现象：任一产品输入（键等于字段、计数器小于 2^48）上改后与改前的返回值、读盘日志、是否 panic 有一项不同；或某个 22 个变异之外的单点改动在 58 904 864 个输入上全绿而 Verus 之外没有别的东西抓它。
（说明：第一行种子 1 那一趟跑在装置加「报错分类」之前，`anchor_err` 恒是 0 不是真的没有那种报错；分类从种子 2 那一趟起才有。）

### 1.3 越出产品输入的两族：两条新断言让改后与改前不同，是契约收紧

改后胶水在调核之前对整张记录表断言两条（核的前置条件，Verus 只在核里假定）：`crates/singlefs-core/src/recovery.rs:3024`：「for (key, record) in records {」之后第一条断言键等于 (实例代号, 计数器)，`crates/singlefs-core/src/recovery.rs:3032`：「record.counter < u64::MAX,」。改前没有这两条：键与字段对不上的表上改前按字段再排序（`research/prompts/m3-verus-land-code-r2-sonnet-model/recovery_fcc4d340.rs:3082`：「above.sort_by_key(|record| (record.instance, record.counter));」）、计数器是 `u64::MAX` 的记录只在被走到时才在 `counter + 1` 上溢出（`research/prompts/m3-verus-land-code-r2-sonnet-model/recovery_fcc4d340.rs:3158`：「expected_next = Some((record.instance, record.counter + 1));」；「读得出后面还有同发布记录」那一判对 `u64::MAX` 用 `checked_add` 交 false，见 `research/prompts/m3-verus-land-code-r2-sonnet-model/recovery_fcc4d340.rs:2945`：「let Some(next_counter) = record.counter.checked_add(1) else {」）。
量（同一装置，`logs/batch1.txt`）：
```
RESULT[key-mismatch] Tally { total: 300000, diverged: 161443, new_panics_only: 161443, old_panics_only: 0, both_ok_differ: 0, panics_old: 0, panics_new: 161443, errs: 13265, applied_some: 11451, verif_failed: 11745, anchor_err: 0, shown: 18 }
RESULT[max-counter] Tally { total: 20000, diverged: 5779, new_panics_only: 5410, old_panics_only: 0, both_ok_differ: 0, panics_old: 2091, panics_new: 7501, errs: 167, applied_some: 564, verif_failed: 0, anchor_err: 0, shown: 18 }
```
键对不上那一族：300 000 个里 161 443 个改后断言 panic、改前返回了（第一轮量的是改前对改后「结局不同」15 734 个，现在改成改后一律 panic）。`u64::MAX` 一族：改前 panic 2091 个，改后 7501 个，多出来的 5410 个是「表里有 `u64::MAX` 的记录、改前没走到它而照常返回」；diverged 5779 里另 369 个是两边都 panic 而读盘日志不同（改后在读盘之前就 panic）。两族里 `both_ok_differ` 都是 0：没有「两边都返回而结果不同」的输入。

### 1.4 两条新断言在产品路径上会不会被触发（题面问的）

不会，三层：
1. 表是 `scan_journal` 造的：`crates/singlefs-core/src/recovery.rs:2925`：「.entry((record.instance, record.counter))」，键就是记录自己的字段。
2. 计数器小于 2^48：记录是解析来的，`crates/singlefs-core/src/journal.rs:292`：「let counter = reader.get_six_byte_unsigned();」，而 `crates/singlefs-core/src/bytes.rs:85`：「pub fn get_six_byte_unsigned(&mut self) -> u64 {」只读 6 字节；反方向写盘也写不出：`crates/singlefs-core/src/bytes.rs:32`：「assert!(value < (1u64 << 48), "48 位字段装不下 {value}");」。装置里 `a_parsed_counter_is_below_two_to_the_48`（2000 个随机计数器写盘再解析，读回值等于原值且小于 2^48；`u64::MAX` 写盘那一步 panic）绿（`logs/batch2a.txt` 的 `test recovery::zz_diff::a_parsed_counter_is_below_two_to_the_48 ... ok`）。
3. 调用方逐个读过，建表都走 `scan_journal` 或按字段建键：产品三处 `crates/singlefs-core/src/mount.rs:4360`：「let records = scan_journal(cache, system_configuration);」、`crates/singlefs-core/src/mounted_read.rs:558`：「let records = scan_journal(reader, &system_configuration);」、`crates/singlefs-core/src/recovery.rs:4176`：「let records = scan_journal(reader, &system_configuration);」；装置两处 `crates/singlefs-checker-tier/src/bin/e158_root_choice_repair.rs:2394`：「let records = scan_journal(&plain_devices, &system_configuration);」；E165 装置三处按解析出来的记录建键：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:771`：「.entry((parsed.instance, parsed.counter))」、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:789`：「records.entry((parsed.instance, parsed.counter))」、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:3308`：「records.insert((parsed.instance, parsed.counter), parsed.clone());」。harness 测试里手造的表（K19 的 `table_of`、K02 的 `scan_journal`）也按字段建键（K19 文件第 64 行起的 `table_of`）。
推翻条件：找到一处产品调用方的表键与字段不等，或计数器等于 `u64::MAX`。
（这两条断言的位置落在核之外：公开函数 `replay_journal` 的签名仍不挡这两种表，改后改成 panic 并在 `# Panics` 一节写明——`crates/singlefs-core/src/recovery.rs` 第 3002 行起该节，在 3004 行的函数头之上。属于第一轮判决要的「核对键等于字段的断言」，不是新差异。）

## 二、H3　报错路径与 G6

### 2.1 核失败有几种、各映射到什么（读代码）

核报的失败只有两种：`crates/singlefs-replay-prefix/src/lib.rs:71`：「pub enum CoreFailure {」下面两个成员（`lib.rs:72` 带计数器的一个、`lib.rs:73` 的「ChosenRootCheckpointTxgHasNoSuccessor,」）。核里 `Err` 只从两处出：带末条标志多于一条（`crates/singlefs-replay-prefix/src/lib.rs:1231`：「if counters_carrying_the_last_record_flag.len() > 1 {」，经 `lib.rs:1555` 的 `Err(failure) => {` 原样往外传）与 txg 最大值（`crates/singlefs-replay-prefix/src/lib.rs:1565`：「return Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor);」）。改前 `replay_journal` 里也只有这两个失败点：`research/prompts/m3-verus-land-code-r2-sonnet-model/recovery_fcc4d340.rs:3094`：「let root_own_record_counter = counter_of_the_last_record_the_root_covers(root, records)?;」（锚点那个，里面 2976–2983 行拼 `RecoveryFailure`）与 `research/prompts/m3-verus-land-code-r2-sonnet-model/recovery_fcc4d340.rs:3105`：「what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR,」（txg 那个，3102 行起）。两处的先后在改前改后一样：锚点的先、txg 的后（改后 `lib.rs:1549` 的 `let root_own_record_counter = match counter_of_the_last_record_the_root_covers(` 在 `lib.rs:1564` 的 txg 判之前）。
胶水把它们一一接回去，`match` 没写通配臂，也没有 `unreachable!`：

| 核失败 | 改前 `RecoveryFailure` | 改后（`crates/singlefs-core/src/recovery.rs`） | 两边内容相同的依据 |
|---|---|---|---|
| 所选根那次发布带末条标志的多于一条 | `RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided { instance: root.instance, checkpoint_txg: root.checkpoint_txg, counters: 按键序的带标志计数器 }` | 第 3106 行 `Err(CoreFailure::RootPublishCarriesMoreThanOneLastRecordFlagWhoseAnchorIsUndecided {` 一臂：同名成员，`instance`、`checkpoint_txg` 取 `root`，`counters` 取核带出的那几条（核按入参次序即键序收，第 3113 行「counters: counters.iter().map(\|counter\| counter.0).collect(),」） | 差分：种子 2 的随机一族里这一种 163 996 个、穷举三族里 11 833 344 + 15 360 + 220 160×3 + 1 761 280 + 128 个，返回的 `Err` 与改前整值相等 |
| 所选根 txg 是 `u64::MAX` | `UnitMalformed { what: CHOSEN_ROOT_CHECKPOINT_TXG_HAS_NO_SUCCESSOR }` | 第 3117 行 `Err(CoreFailure::ChosenRootCheckpointTxgHasNoSuccessor) => {`：同一个 `UnitMalformed`、同一个常量 | 差分：种子 2 随机一族里 54 746 个，整值相等；穷举族根 txg 固定 1，没覆盖这一种，靠随机族 |

没有第三种核失败，没有被吞的（核的 `Err` 两臂都 `return Err`），没有映射错的：每一臂的内容在差分里整值比过。所以 **H3 没打中**。
还有一个口子要写明白：改前这条 `replay_journal` 里**不是** `Result` 的出口只有「在飞上限」的 `expect`（`recovery_fcc4d340.rs:3085`）与算术溢出；改后多了两条 `assert`（见 1.3、1.4），属于调用方违反前置条件，不是核失败，产品路径走不到，不需要 `RecoveryFailure` 成员。
推翻条件：核里出现第三个 `Err` 出口，或某输入上改后的 `Err` 与改前不整值相等。

### 2.2 胶水的映射有没有测试钉住（变异）

对胶水的映射、报告拼装、新根来源做单点改动（`mutate_glue2.py` 与 `mutate_glue.py`，每个先断言旧串恰好命中一次，跑完换回原文），只跑五个 harness 目标：K19 回归 `recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw`、`core_review_tree_table_duplicates_and_slot_one_search`、`journal_last_record_flag_of_a_multi_record_publish`、`journal_replay_applies_no_more_records_than_the_in_flight_limit`、`writable_mount_takes_a_second_instance`。
前六个改动（`logs/mutate_glue_out.txt` 原样）：
```
GLUE counters_reversed | 编过=True | 退出码=101 | 红的测试=['two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs_counters', 'two_last_record_flags_in_the_chosen_roots_publish_stop_recovery_and_a_writable_mount_before_any_write']
GLUE counters_drop_first | 编过=True | 退出码=101 | 红的测试=[同上两条]
GLUE error_txg_wrong | 编过=True | 退出码=101 | 红的测试=[同上两条]
GLUE error_instance_wrong | 编过=True | 退出码=101 | 红的测试=[同上两条]
GLUE no_successor_maps_to_no_valid_root | 编过=True | 退出码=101 | 红的测试=['a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking']
GLUE no_successor_wrong_what | 编过=True | 退出码=101 | 红的测试=['a_chosen_root_whose_checkpoint_txg_is_the_largest_value_is_refused_as_damaged_instead_of_panicking']
```
（`...` 处各行完整原文见日志；K19 那条测试 `two_last_record_flags_in_the_chosen_roots_publish_return_the_error_with_only_that_publishs_counters` 在 `crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs` 里，另一条在 `journal_last_record_flag_of_a_multi_record_publish.rs` 里，txg 那条在 `core_review_tree_table_duplicates_and_slot_one_search.rs` 里。）
两种报错的每个字段（instance、txg、counters 的次序与个数、`what`）都被一条测试钉着，改坏都红。

## 三、H6　E165 装置的 `--pinned-cases`（次要）

读的是冻结副本里的装置（sha256 `24a4b2eb…6896`，与快照清单里那一行相同）、主仓的钉表 `.claude/gate.d/e165-pinned-red-examples.tsv`（sha256 `844bcb23…6a6e`）与 `research/scripts/e165-pin-red-examples.py`（sha256 `3ba54f4f…9f2b`）。没跑装置、没跑 `e165-pin-red-examples.py --replay`（都在重型测试里，见「没做什么」）；只做读代码、纯 Python 对表、和把装置里那一个纯函数的原文逐字编进一个 harness 测试里与 Python 那一份对拍。

### 3.1 找状态：②③④

- 入口：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:5041`：「if option == "--pinned-cases" {」，只认恰好两个参数；Python 起它的命令恰是 `[binary, "--pinned-cases", 表的绝对路径]`（`research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read:390`：「command = ["bash", binary, DEVICE_FLAG, os.path.abspath(table)] if binary.endswith(".sh") else [binary, DEVICE_FLAG, os.path.abspath(table)]」）。
- 行的读法两边一致：装置 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1814`：「!line.trim().is_empty() && !line.starts_with('#')」，Python `research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read:211`：「if not raw.strip() or raw.startswith("#"):」；八段都是 8（装置 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1681`：「const PINNED_TABLE_COLUMN_COUNT: usize = 8;」，Python `research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read:73`：「LEDGER_COLUMNS, TABLE_COLUMNS = 13, 8」）。
- ②③ 经 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1570`：「fn domain_and_size_of(domain_text: &str, size_text: &str) -> Result<(Domain, usize), String> {」解成（域, N），N 要在 `smallest_size()` 到 12（第 1579 行 `if size < domain.smallest_size() || size > 12 {`）；④ 按 `u64` 解。Python 一侧不判这三样，只在重放时看装置退码。
- 找状态：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1777`：「fn replay_a_pinned_red_example(example: &PinnedRedExample) -> (Option<String>, u64) {」设记下模式的探针、单线程把这个（域, N）整个走一遍（第 1635 行同款 `tally_of_a_domain_size(domain, size, Implementation::TodayReplayJournal, 1)`，`--replay-state` 用的是同一个调用），只有序号相等的那个状态被判、被记。两条判定路径（`e165_replay_bounded_enumeration.rs` 里 `begin_probing_a_state` 的两处调用，第 2226 行与第 3290 行）都挂着探针，没有哪条路径还在读旧的 `OnceLock`（`grep -n STATE_PROBE` 全文只剩 `Mutex` 一份与原子开关）。
- 对表（`h6_table_check.py`，只读，输出 `logs/h6_table_check.txt`）：151 行数据，每行 8 段、② 在装置的 14 个域名里、③ 在 `smallest_size()`–12 之间（最大 N 是 4）、④ 是非负整数、⑦ 以「E7RESULT name=state 」起头、⑦ 里的 `domain=` `n=` `ordinal=` 与 ②③④ 逐行相同；问题 无。即表头约定的「②③④ 定位一个状态」与装置读法在这 151 行上没有分歧。

### 3.2 重算 ①

表头写的是（`.claude/gate.d/e165-pinned-red-examples.tsv:11`：「① 输入指纹：sha256(<域>…state= 的值，截到 ;first= 之前>) 的十六进制」）。装置的写法：`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1725`：「let (_row_before_the_state, state_text) = state_row.rsplit_once(" state=")?;」，空或含空白交 `None`，再用 `split_once(";first=")`（第 1730 行）截断，拼 `域|N|state` 取 sha256。Python 的写法：`research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read:102`：「state=(\S+)$", line)」、第 103 行「return match.group(1).split(";first=")[0] if match else None」、第 110 行「.encode("utf-8")).hexdigest()」。
读代码看是一致的（取最后一个 ` state=`、不含空白、第一个 `;first=` 之前、拼法相同）。在表上量：`h6_table_check.py` 里两种取法在 151 行 ⑦ 上取出同一个 state、重算的 ① 都等于表里的 ①（`logs/h6_table_check.txt`：「问题： 无」）；在表以外的构造行上两边对不对，是下面 3.3 的对拍。

### 3.3 同一批行上两边各算一遍（对拍）

`zz_h6_fingerprint.rs`：把装置里 `input_fingerprint_of` 的原文（`e165_replay_bounded_enumeration.rs` 第 1724–1737 行，`awk` 逐字取出）原样放进一个 harness 测试里编（用装置同款的 `singlefs_harness::sha256::sha256_hexadecimal`），读 `rows.tsv`（表里 151 个 ⑦ 加 17 条构造的怪行：没有 `state=`、`state=` 为空、值里有空格、两个 ` state=`、`state=;first=`、两个 `;first=`、`;expected=`、行首就是 `state=`、行尾 `\r`、值里有制表符、行尾空格、不换行空格与全角空格、中文值、`name=done` 行）各打一个指纹；`compare.py` 取 Python 脚本里 `state_before_outcome` 与 `input_fingerprint` 的原文（取自 `e165-pin-red-examples.py.read`，见下一段说明）逐字执行，同一批行再算一遍，逐行比。测试经 `run-with-memory-cap.sh 8G` 与 `capped.sh 6` 跑（日志 `logs/h6_rust_run.log`：`test result: ok. 1 passed`、`exit 0`）。原样输出：
```
行数 python=168 rust=168；不同 1 行；None 的行数 python=9 rust=10
不同：行 159 48d7be515655127f93c4a597838cc3546a2f74297ba614f7efd29ce1f0a5a40d None
```
168 行里 167 行相同（含 151 行真表行与 16 条怪行）；不同的是第 159 行（rows.tsv 的第 160 行，怪行里的行首就是 `state=` 那条，原文 `state=s`）：Python 的正则 `(?:^| )state=` 认行首，装置要求 ` state=` 前有一个空格，所以 Python 算出指纹、装置交 `None`。产品里不会碰到：装置打出的逐态行恒以 `E7RESULT name=state ` 起头（装置的格式串在 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:1541`：「"E7RESULT name=state domain={} ring_slots={} n={size} ordinal={ordinal} counters={} state={}",」），钉表 ⑦ 又都从这样的行原样抄来，所以 `state=` 前恒有空格。记一笔：两边认的字符串不是同一个集合，差在「行首裸 `state=`」这一种不可能的输入上，**不构成一处两边各读一种**（表头约定写的是 ` state=` 的值）。
说明：`research/scripts/e165-pin-red-examples.py` 在我读它之后被别的会话改过（工作区 sha256 变了，index 里仍是我读的 `3ba54f4f…`，改动加在文件头注释、`--replay` 的退出码分支与自证里，行号整体后移 4 行；改动没碰 `input_fingerprint` 与读表）；报告里引它的行号都指入库的原样拷贝 `research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read`（`git show :research/scripts/e165-pin-red-examples.py`，sha256 以 `3ba54f4f09b169883a6b` 起头）。

### 3.4 接口：装置打什么、Python 读什么

- 装置每行打一行 `E7RESULT name=state …`，另打开头一行 `E7RESULT name=config …`（第 1817 行）与收尾一行 `E7RESULT name=done …`；Python 只取以 `E7RESULT name=state ` 起头的行（`research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read:72`：「RESULT_PREFIX = "E7RESULT name=state "」），所以多出来的两行不会错位。逐行次序按数据行次序：Python 用 `results[index]` 对 `rows[index]`（`.read` 第 405–408 行，原文「results = [line for line in completed.stdout.split("\n") if line.startswith(RESULT_PREFIX)]」在 `research/prompts/m3-verus-land-code-r2-sonnet-model/e165-pin-red-examples.py.read:405`），装置按数据行次序打，遇到解不出的行或找不到状态的行才少打——那时装置退 1，Python 先看退出码，所以错位不会把一个错的行当绿。
- 装置的退码：有问题退 1（`e165_replay_bounded_enumeration.rs:1878`：`std::process::exit(1);`），读不了表退 2；Python 对非 0 一律报红。
- 逐态行与 ⑦ 比不比得上由 Python 判、装置不判（装置文件头 `crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:29`：「装置不判」同写），与表头约定一致；我复核不了「今天的实现在 151 行上打出的逐态行与 ⑦ 逐字节相同」这一半：那要真跑装置 `--pinned-cases`，是重型测试。
- 结论：**H6 没打中**（读法、重算 ①、找状态、接口都对得上；唯一的差是上面行首裸 `state=` 那条不可能的输入）。推翻条件：真跑 `--pinned-cases` 得到的逐态行某一行与 ⑦ 不同，或装置某一行打出的 ① 与表里不同而 Python 判绿。

后十个改动（`logs/mutate_glue_out.txt` 里这十条是包装退 252 没跑成，不算结果；`mutate_glue2.py` 重跑，等锁等到了，`logs/mutate_glue2_out.txt` 原样，红的测试一栏只列测试名前缀）：
```
GLUE above_water_is_table_size | 红的测试=[last_record_flags_of_other_publishes_…, remount_takes_instance_two_writes_the_row_…, with_an_in_flight_limit_of_one_only_the_first_…]
GLUE applied_is_zero | 红的测试=[last_record_flags_of_other_publishes_…, stray_record_of_the_previous_instance_…, torn_anchor_record_lets_the_chain_start_…, when_the_last_record_of_the_chosen_roots_publish_is_unreadable_…, with_an_in_flight_limit_of_one_only_the_first_…]
GLUE max_tx_zero | 红的测试=[stray_record_of_the_previous_instance_…, torn_anchor_record_lets_the_chain_start_…]
GLUE passed_swapped_with_failed | 红的测试=[with_an_in_flight_limit_of_one_only_the_first_…]
GLUE back_chain_inverted | 退出码=0 | 红的测试=[]
GLUE named_all_to_any | 红的测试=[last_record_flags_of_other_publishes_…]
GLUE counter_assert_le | 退出码=101 | 红的测试=[]（装置的正则认不出 should_panic 测试的名字，日志里是 a_record_whose_counter_is_the_largest_value_trips_the_glue_assertion_before_the_kernel_runs - should panic ... FAILED）
GLUE key_assert_removed | 退出码=101 | 红的测试=[]（同上，日志里是 a_table_whose_keys_disagree_with_its_records_trips_the_glue_assertion_before_the_kernel_runs - should panic ... FAILED）
GLUE skip_mode_inverted | 红的测试=[with_an_in_flight_limit_of_one_only_the_first_…]
GLUE rebuilt_wrong_record | 红的测试=[last_record_flags_of_other_publishes_…, stray_record_of_the_previous_instance_…, torn_anchor_record_lets_the_chain_start_…, when_the_last_record_of_the_chosen_roots_publish_is_unreadable_…, with_an_in_flight_limit_of_one_only_the_first_…]
```
合起来：16 个胶水改动里 15 个在这五个目标上红；两条断言各被 K19 的 `should_panic` 那两条钉住（`counter_assert_le`、`key_assert_removed` 的 `should panic ... FAILED`）；核报告五字段（`above_water`、`prefix_applied`、`maximum_applied_transaction`、`verification_passed`/`verification_failed` 互换、点名开关反转）、新根来源各有至少一条测试钉着。唯一一个这五个目标上全绿的是 `back_chain_inverted`（`Broken => true` 改成 `false`，反向链断了照样往下走）：它不是漏网，`crates/mutations.tsv` 里本来就有一行点名别的目标（第 920 行，见下一行的复跑）。
复跑（`backchain_target.py`，日志 `logs/glue/glue_back_chain_inverted_row920.log`）：同一个改动只跑第 920 行点名的目标 `core_review_geometry_back_chain_and_empty_inode`：`test a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix ... FAILED`、`test result: FAILED. 13 passed; 1 failed`，红的正是第 920 行写的那一条。所以胶水里反向链回调的三态到布尔的映射有测试钉着。
H3 的第二半（胶水映射有没有测试钉住）：没打中。推翻条件：胶水某一处映射或某个报告字段的单点改动，在五个目标加第 920 行点名的目标上全绿。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| H1 改后核与 fcc4d340 等价（产品输入） | 一致，没打中 | 58 904 864 个产品输入（随机两个种子、穷举三族）上返回值、报告五字段、报错种类与内容、读盘次序、是否 panic 零差异；改前 4 个变异体与改后核的 22 个单点改动阳性对照全红 |
| H1 两条新断言在产品路径上会不会触发 | 不会 | 表由 `scan_journal` 按字段建键，计数器 6 字节读来小于 2^48，调用方逐处读过 |
| H1 越出产品输入 | 契约收紧，不是产品差 | 键与字段不等的表 300 000 个里 161 443 个、含 `u64::MAX` 计数器的表 20 000 个里 5410 个改后 panic、改前返回（`both_ok_differ` 两族都是 0）；`# Panics` 一节已写明 |
| H3 核失败到 `RecoveryFailure` | 一致，没打中 | 核只有两种失败，各映射到改前同名成员、内容整值相等（差分里两种各出现过）；没有第三种、没有被吞；先后次序同改前 |
| H3 胶水映射有没有测试钉住 | 钉住 | 16 个单点改动都红（其中 `back_chain_inverted` 要换到第 920 行点名的目标才红）；`back_chain_inverted` 在 K19 那五个目标上全绿但在第 920 行点名的目标上红 |
| H6 `--pinned-cases` 与钉表头约定、`e165-pin-red-examples.py --replay` | 一致，没打中 | ②③④ 找状态、① 重算、接口都对得上；151 行表行全一致；168 行指纹对拍 167 行相同，不同的一行是行首裸 `state=`，装置输出里不会出现；逐态行与 ⑦ 逐字节比这一半没跑（重型） |

腿提的新改法：没有（零轮的也没有）。
几处顺带记：
- 我用 `python3` 的 `open(…, 'w')` 就地改过一次模型目录里的 `zz_diff.rs`（加 Tally 字段）、`sed -i` 改过草稿目录里的脚本；`.claude/hooks/bash-command-detector.sh` 的 ⑧ 没拦下前一次：与第一轮判决第三节记的 `sed -i` 没被拦是同一道闸，工具簇欠账。之后的改动都在草稿目录里做、再拷进模型目录。
- 差分装置的随机一族与第一轮同种子的 `errs`、`applied_some`、`verif_failed` 三个数逐个相同（3 000 000 个输入），说明装置在两轮之间没有漂。

## 没做什么

- 没跑 Verus，所以「32 verified, 0 errors」与 94 行 `verus-mutations.tsv` 全红没有复核（不归这条腿）；H2（spec 补完证到了什么）、H4（K19、K20、K02、G08a 与 3-6 单测证不证得到名字说的事，除 K19 的映射与两条断言外）、H5（编码纪律）不归这条腿，没判。
- 没跑 E165 装置的 `--pinned-cases`，没跑 `e165-pin-red-examples.py --replay`、没跑 `cargo test -p singlefs-checker-tier`（重型测试）：「今天的实现在 151 行钉住的输入上打的逐态行与 ⑦ 逐字节相同」复核不了，原因是要真跑装置。
- 没在入库装置上重做：差分装置是我自己的副本（见 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」那一条）：引这些数前请把 `zz_diff.rs` 的复跑当新一次跑。
- 没读主仓 `crates/`、没读攻方腿的产出与模型目录。
- 没复核 GPU 的 WGSL 判器（`crash_judge_recovery.wgsl`）与改后核的一致，那是另一件事。
- 没把这一轮的数据入 `research/results/`：差分是对判决的复核材料，日志在 `research/prompts/m3-verus-land-code-r2-sonnet-model/logs/`，不是实验产物。
- 草稿目录里的编译目录与仓副本交回前删，见交回正文。
