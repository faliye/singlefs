# m3-verus-land-code-r1 正推腿（Sonnet）报告（2026-10-01）

读的是冻结副本 `/tmp/claude-1000/m3-verus-land-code-r1/tree/`；改前 `recovery.rs` 是 `git show fcc4d340:crates/singlefs-core/src/recovery.rs` 的原样拷贝 `/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs`（下文引改前行号都指它）。
简写：F = 冻结副本 `crates/singlefs-replay-prefix/src/lib.rs`，R = 冻结副本 `crates/singlefs-core/src/recovery.rs`，O = 改前 `recovery_old.rs`。
模型、装置、日志在 `research/prompts/m3-verus-land-code-r1-sonnet-model/`（README.md 一页说明）。线程一律 ≤10，经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10`；没跑 Verus、没跑 checker 档。

## 一、结论摘要

1. C1：改前（O 第 3057–3229 行）与改后（R 第 3042–3162 行）在 56 924 864 个输入上逐个比结局（返回的根、报告五个字段、报错种类与内容、读盘次序、是否 panic），零差异；四个改前变异体作阳性对照都红。一处契约变化要写明：核靠「切片按键升序、键等于记录字段」，公开函数 `replay_journal` 的入参类型没挡键与字段对不上的表，那种表上改前改后不同（300 000 个里 15 734 个）；今天所有调用方建表都键对字段，所以只是未写明的前置条件，不是产品路径上的差。
2. C2：spec 与 `ensures` 只说出 M2 的「整次发布」那一半和「严格大于所选根」那一句；五条口径里 jsn 连续（已知）、反向链、提交标记、点名单元校验、末条之后同发布还有读得出的记录、在飞上限、链首 txg，再加报告里除 `prefix_applied` 外的字段，都没进 spec。一个「一条都不施加」的实现满足 `m2_holds`。按跑前判据第五节第 2 条，打中 C2「spec 太弱」，并进第 4 批。`verus-mutations.tsv` 38 行只改 spec / 不变式 / `ensures`（第一条数据行连 `record_ends_its_publish` 的函数体一起改成恒真，那是一个一行函数），没有一行改核里真正做判断的可执行码。
3. C3：平时编的是 `verus!` 擦除后的普通 Rust（已用 `-Zunpretty=expanded` 取出读过一遍）；没有发现 cfg、`#[verifier::…]`、幽灵代码留下的可执行差别。Verus 自带发布与 crates.io 的 vstd 是不是同源，本机没有 Verus 发布目录，复核不了。
4. C7：判法摘要变（改前 44 份文件、改后 45 份，摘要不同）；「`use` 自动进闭包」成立（探针里 `crates/singlefs-replay-prefix/src/lib.rs` 的收录原因是 referenced by recovery.rs）；GPU 的 WGSL `replay_journal` 是改前 Rust 的另写一份、不调核，与核的分支次序逐条对得上，但我没有机器比过它。

## 二、C1　核与原文等价

### 2.1 量法

装置：把改前的 `replay_journal` 与 `readable_record_of_the_same_publish_follows`（O 第 2941–2953、3057–3229 行）原样放进冻结副本的拷贝，改名 `replay_journal_before`，与改后的 `replay_journal` 在同一个内存读盘器上各跑一次；读盘器记每次 `read` 的（盘号、偏移、长度）。比：返回值整体相等（`JournalScanReport`、`RootRecord`、`RecoveryFailure` 都 `PartialEq`）、读盘日志相等、两边是否 panic 相同。输入含：所选根 instance 0–2 与 txg（含 0、`u64::MAX`、`u64::MAX - 1`）、0–9 条记录、实例与 txg 乱跳、跳号、重复键、序号乱、末条标志乱、提交标记缺、反向链坏、前一条在两块盘上的六种在盘状态（两份好、只一份、都没有、一份被改一份好、两份都被改）、点名项的类别与两份位置的三态（好 / 坏 / 读不到）、在飞上限 0–13、点名校验开关。
命令（每条都在副本拷贝里经内存包装跑，日志在模型目录 `logs/`）：
- 随机一族 `zz_diff::random_family`，`DIFF_N=3000000`。
- 穷举一族 `zz_diff::exhaustive_structural` 与 `zz_diff::exhaustive_chain_and_named`：所选根自己那次发布的 5 种形态 × 每条记录 gap/txg/序号/末条标志/提交标记/事务号/实例各取值，k 条（k=1、2、3）；再加 k=3 时 3 位反向链断点掩码 × 3^3 点名项状态。
- 计数器 `u64::MAX` 一族 `zz_diff::max_counter_family`（20 000 个）。

原样输出（`logs/run1-random-controls-summary.txt`、`logs/run2.log`、`logs/run3.log` 里逐行抄的）：
```
CONTROL[m1 去掉 take(limit)] diverged=1581 / 40000
CONTROL[m2 点名失败不断] diverged=3821 / 40000
CONTROL[m3 序号 FIRST 改 >2] diverged=545 / 40000
CONTROL[m4 max 只看第一条] diverged=520 / 40000
test recovery::zz_diff::random_family ... RESULT[random] Tally { total: 3000000, diverged: 0, panics_old: 0, panics_new: 0, errs: 327781, applied_some: 271293, verif_failed: 290134, anchor_err: 0, shown: 0 }
test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 134 filtered out; finished in 46.49s
test recovery::zz_diff::exhaustive_chain_and_named ... RESULT[chain+named k=3] Tally { total: 47775744, diverged: 0, panics_old: 0, panics_new: 0, errs: 11833344, applied_some: 218784, verif_failed: 2118414, anchor_err: 0, shown: 0 }
test recovery::zz_diff::exhaustive_structural ... RESULT[structural k=1 inst=true verify=false slots=40] Tally { total: 480, diverged: 0, panics_old: 0, panics_new: 0, errs: 128, applied_some: 16, verif_failed: 0, anchor_err: 0, shown: 0 }
RESULT[structural k=2 inst=true verify=true slots=40] Tally { total: 46080, diverged: 0, panics_old: 0, panics_new: 0, errs: 15360, applied_some: 1072, verif_failed: 0, anchor_err: 0, shown: 0 }
RESULT[structural k=3 inst=false verify=true slots=40] Tally { total: 552960, diverged: 0, panics_old: 0, panics_new: 0, errs: 220160, applied_some: 8704, verif_failed: 0, anchor_err: 0, shown: 0 }
RESULT[structural k=3 inst=false verify=false slots=40] Tally { total: 552960, diverged: 0, panics_old: 0, panics_new: 0, errs: 220160, applied_some: 8704, verif_failed: 0, anchor_err: 0, shown: 0 }
RESULT[structural k=3 inst=false verify=true slots=6] Tally { total: 552960, diverged: 0, panics_old: 0, panics_new: 0, errs: 220160, applied_some: 8704, verif_failed: 0, anchor_err: 0, shown: 0 }
RESULT[structural k=3 inst=true verify=true slots=40] Tally { total: 4423680, diverged: 0, panics_old: 0, panics_new: 0, errs: 1761280, applied_some: 68608, verif_failed: 0, anchor_err: 0, shown: 0 }
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 135 filtered out; finished in 359.84s
RESULT[max-counter] Tally { total: 20000, diverged: 0, panics_old: 2091, panics_new: 2091, errs: 323, applied_some: 702, verif_failed: 0, anchor_err: 0, shown: 0 }
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 136 filtered out; finished in 0.00s
```
说明：`run1` 汇总里那行 `FAILED. 2 passed; 1 failed` 是装置自己的毛病（`max_counter_family` 里我写的 `100 + s.counter` 在 `u64::MAX` 上溢出，装置 panic，已改成 `wrapping_add`），不是被测对象；计数器 `u64::MAX` 那一族另在 `run2.log` 重跑，即上面倒数第二行。总数：3 000 000 + 6 129 120（穷举结构一族六行之和）+ 47 775 744 + 20 000 = 56 924 864，`diverged` 全 0。`panics_old == panics_new == 2091`：两边在 `u64::MAX` 计数器上 panic 的输入一一相同（`Cargo.toml` 的 `overflow-checks = true`，见 `/tmp/claude-1000/m3-verus-land-code-r1/tree/Cargo.toml:19`）。
阳性对照：改前变异体 m1（去掉 `.take(in_flight_limit)`）、m2（点名失败不断）、m3（序号 `FIRST` 改 `>2`）、m4（最大事务号只看第一条）各 40 000 个输入，装置分别抓到 1581、3821、545、520 个，所以差分抓得到这一类差。

### 2.2 逐处读代码核（题面点名的五处）

1. `is_above_the_chosen_root` 的抽取：改前筛选在 `/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3079`「record.instance == root.instance && (record.instance, record.checkpoint_txg) > water」，改后在 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:240`「record.instance == root_instance && (record.instance > root_instance || (record.instance」（该句在下一行续写）。`water` 的第一分量就是 `root.instance`，两句逻辑同值；改后 `record.instance > root_instance` 那一支在 `&&` 左边已为真之后恒假，是死支，不影响结局。
2. 两个回调的懒求值：反向链在 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:452` 的 `if back_chain_broken(index) {` 才调，位置在 `/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3121` `match judge_back_chain_against_the_previous_record_on_disk(` 同一处（链首判定之后、序号判定之前）；点名校验在 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:491` `let all_verified = !verify_named_units || named_units_verified(index);` 才调，对应 `/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3173`，开关关着不调；回调里的 `.all` 短路写法逐字照搬（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3090`–3105）。日志相等的数据：上面 56 924 864 个输入里读盘日志一并比过，零差异。
3. 在飞上限：`/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3109` 是 `.take(in_flight_limit)` 套在已筛过的 `above` 上，`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:427` `if taken >= in_flight_limit {` 在筛过之后才计数，两者都只数水位之上的记录；m1 变异体证明装置分得出这一差。
4. `unreachable!`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3122`「Ok(_) => unreachable!("核报了所选根那次发布带末条标志的多于一条")」）：核报该错的条件是第二条「同实例、同 txg、带末条标志」的记录出现（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:215`–219），改前是同一过滤上 `len() > 1`（`/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:2967`–2976），两者读同一张表（核入参由 `records.values()` 摊出，`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3062`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3064`），所以再调原函数必然报同一个错；随机与穷举里报该错的输入（`errs` 列含它）都没触到 panic，且 `Err` 内容（含 counters 清单）与改前逐字相等。
5. 拼根：改前每次施加都用 `rebuilt.filesystem_identifier`、`rebuilt.instance_table`、`rebuilt.allocation_record_tree_root`（`/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3211`、3221、3225），而这三项从未被换过、恒等于 `*root` 的；改后直接取 `root` 的（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3143`、3153、3157），等价。`outcome.rebuilt` 的 `None => *root`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3138`）对应一条都没施加。装置里每条记录的 `new_tree_identifier_watermark` 各不相同，施加的是哪一条的新根段在比对里分得出。

### 2.3 核比改前多出来的前提（打中，但不在产品路径上）

核的 `readable_record_of_the_same_publish_follows` 自己写明（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:149`）：「才与原文同义；这一条没有写成前置条件，也没有证」，前半句是「靠切片按键升序」。改前在 `BTreeMap` 上按键 `range` 查（`/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:2949`），并且在 `/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3082` 把水位之上的记录按记录字段再排一次序；改后既不查键也不再排序，只信入参切片的次序，而切片次序来自键序（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3062`）。两者同值要求「键 = (记录.instance, 记录.counter)」。
`replay_journal` 是 `pub`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3042`），入参类型 `BTreeMap<(InstanceGeneration, u64), JournalRecord>` 不挡键与字段对不上的表。量：随机一族的每个输入里随机把约 1/6 的记录改挂到别的键上（`zz_diff::key_mismatch_family`，300 000 个），改前改后不同的 15 734 个（`logs/keymis.log`）：
```
RESULT[key-mismatch] Tally { total: 300000, diverged: 15734, panics_old: 0, panics_new: 0, errs: 31174, applied_some: 24122, verif_failed: 24507, anchor_err: 0, shown: 24 }
```
今天所有建表的地方键都等于字段：扫环 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:2923` `.entry((record.instance, record.counter))`，E165 装置 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs:764` 与 `:3028`。所以它是「未写明的前置条件」而不是行为差。
推翻条件：找到一处调用方建表键与字段不等（grep 全仓 `replay_journal(` 的调用点，上面列的是我查到的全部建表处）。
零轮改法（最小一改，不涉及性能）：在 `replay_journal` 的文档里写「键必须等于记录的 (instance, counter)」，或在 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3062` 之后加一句 `debug_assert!` 逐条比。

### 2.4 计数器 `u64::MAX` 与前置条件

核的 `requires` 要所有记录的计数器 `< u64::MAX`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:273`「前置条件只有一条：计数器不是 u64 的最大值」）。产品路径上担保它的只有解析：`JournalRecord::parse` 的计数器是 6 字节（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/journal.rs:292` `let counter = reader.get_six_byte_unsigned();`），小于 2^48；`replay_journal` 本身没有运行时判，调用方手建的记录可以违反它。违反时：改前 `counter + 1`（`/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3096`、`/tmp/claude-1000/m3-verus-land-code-r1-sonnet/recovery_old.rs:3158`）与改后（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:350`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:478`）都在 `overflow-checks = true` 的 profile 上 panic，量过两边 panic 的输入一一相同（2091 / 2091）。所以不是新增差异，只是 Verus 证明里「调用方担保」那一格没有类型或运行时的担保。

## 三、C2　证明证到了什么

### 3.1 spec 与条款逐句对

D13 已定项 1 的 M2 行，`.claude/kb/decisions/13-验证路线.md:32`：「任一 `jsn` 连续前缀，其重放结果等于某个整数个已发布的 checkpoint 的结果——不存在被前缀切开的发布；前缀停在一次发布中间时，那次发布整体不施加」。
D23 已定项 14 定案句，`.claude/kb/decisions/23-journal的角色与格式.md:355`：「恢复只施加 (实例代号, checkpoint_txg) 严格大于所选根的记录」。
五条口径，`.claude/kb/decisions/23-journal的角色与格式.md:357`：「前缀判定的完整口径是五条，缺一不可」，第一至第四条在 358–360 行，第五条在 361 行。
D23 已定项 22，`.claude/kb/decisions/23-journal的角色与格式.md:570`：「前缀判据是 jsn == expected，不等即止。」

`m2_holds`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:93`–118）说了六件事：下标在范围内、严格递增、每条都「同实例且 txg 大于所选根」（`above_the_chosen_root`，`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:52`–58）、记录序列是若干整次发布的拼接（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:75`–84、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:61`–72）、`prefix_applied == applied.len()`、`rebuilt` 等于末条的新根段或所选根的段。
对照结果：
| 条款句 | spec 里有没有 |
|---|---|
| M2 整次发布、不存在被切开的发布 | 有（`is_concatenation_of_whole_publishes`：同 txg、序号 1..k、只有第 k 条带末条标志） |
| 14 定案句 严格大于所选根、注 1 实例不同即停 | 有，但写成「施加的每条同实例且 txg 大」的过滤，没有「停」 |
| 五条第 1 条 jsn 严格连续、22 断号即止 | 没有（已知，主 agent 已现跑坐实） |
| 五条第 3 条 提交标记齐全 | 没有（`CoreRecord.is_commit` 不出现在 `m2_holds`） |
| 五条第 4 条 点名单元逐项验证 | 没有（回调结果不进 `ensures`） |
| 反向链（已定项 8、I-8.6） | 没有 |
| 已定项 4 读者规则：带末条标志那条之后同 txg 还有读得出的记录就断 | 没有 |
| 14 注 1 链首与锚点（读法乙）、无锚点时 txg = 根 + 1 | 没有 |
| 在飞上限（已定项 18） | 没有 |
| 报告字段 `above_water`、`verification_passed`、`verification_failed`、`maximum_applied_transaction` | 没有，只有 `prefix_applied` |
| 报错 | `Err(_) => true`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:304`）：任何输入返回任何错都满足 |

### 3.2 打中 C2「spec 太弱」的具体实现

- 实现 K：「一条都不施加」。读 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:93`–118：`applied` 为空时下标三条全称式空真，`is_concatenation_of_whole_publishes` 在 `applied.len() == 0` 时为真（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:80`），`prefix_applied == 0`，`rebuilt == root_segment`，全部满足；另外 `Err(_) => true` 让「恒报错」也满足。违反条款：D23 已定项 14 的前缀要施加（恢复丢掉已 fsync 的版本）。
- 实现 B / C / D / E / F / H：各自删掉反向链断、提交标记两判、末条之后同发布断、点名校验、无锚点 txg 判；它们违反五条口径或已定项 4，但不碰 `m2_holds` 里任何一件。
- Verus 预测（**推的，Verus 没跑**）：以上 K、A–H、J 全部仍验过（对应 `ensures` 不提这些、循环不变式只留 `taken <= index` 与 `verification_failed == 0` 的 invariant_except_break 这类与它们无关的弱条件）；删第 456 行与 463–466 行那两判（M、N）、`prefix_applied` 少加（L）会红，因为 `is_open_publish` 与 `prefix_applied == applied.len()` 管着。主 agent 要核，把 `research/prompts/m3-verus-land-code-r1-sonnet-model/mutate_core.py` 里每条的（原文、替换文）按 `verus-mutations.tsv` 的五列格式搬去现跑；期望列 A–K 写「verified, 0 errors」、L–N 写「verified, 」加「errors」。
- 差分量（改前为准绳，核的可执行码改一处）：下表是 `mutate_core.py` 在副本拷贝上的原样输出（`logs/mutants_out.txt`），每条列 4 个取样集的「差分抓到数 / 取样数」：
```
MUT A 去掉在飞记录数上限那一断 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 0/46080; 3 inst=false slots=40: 0/552960; 3 inst=false slots=6: 912/552960; random: 19550/500000
MUT B 去掉反向链回调那一断 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 7520/46080; 3 inst=false slots=40: 78400/552960; 3 inst=false slots=6: 78400/552960; random: 180640/500000
MUT C 去掉『上一条没提交标记而换了事务号』那一断 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 48/46080; 3 inst=false slots=40: 0/552960; 3 inst=false slots=6: 0/552960; random: 3450/500000
MUT D 去掉『末条不带提交标记』那一断 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 1104/46080; 3 inst=false slots=40: 8966/552960; 3 inst=false slots=6: 8928/552960; random: 10029/500000
MUT E 去掉『末条之后同发布还有读得出的记录』那一断 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 512/46080; 3 inst=false slots=40: 8064/552960; 3 inst=false slots=6: 8064/552960; random: 5206/500000
MUT F 点名单元校验恒过 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 0/46080; 3 inst=false slots=40: 0/552960; 3 inst=false slots=6: 0/552960; random: 62649/500000
MUT G 去掉 jsn 严格连续那一断（已知：核没证它） | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 4784/46080; 3 inst=false slots=40: 68178/552960; 3 inst=false slots=6: 68064/552960; random: 41270/500000
MUT H 无锚点时不看 txg = 根 txg + 1，只看序号 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 2336/46080; 3 inst=false slots=40: 22848/552960; 3 inst=false slots=6: 22848/552960; random: 36439/500000
MUT I 无锚点那一支不看序号（tsv-448 原来那条；核里另有第 456 行那一判兜着） | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 896/46080; 3 inst=false slots=40: 9472/552960; 3 inst=false slots=6: 9472/552960; random: 726/500000
MUT J maximum_applied_transaction 不更新 | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 1072/46080; 3 inst=false slots=40: 8704/552960; 3 inst=false slots=6: 8704/552960; random: 45210/500000
MUT K 一条都不施加（取到第一条就收工） | Verus 预测=绿 | 编译=过 | 2 inst=true slots=40: 8480/46080; 3 inst=false slots=40: 87104/552960; 3 inst=false slots=6: 87104/552960; random: 189553/500000
MUT L 控制：prefix_applied 少加 | Verus 预测=红 | 编译=过 | 2 inst=true slots=40: 1072/46080; 3 inst=false slots=40: 8704/552960; 3 inst=false slots=6: 8704/552960; random: 45210/500000
MUT M 控制：去掉『一次发布第一条序号是 1』那一断 | Verus 预测=红 | 编译=过 | 2 inst=true slots=40: 2144/46080; 3 inst=false slots=40: 20706/552960; 3 inst=false slots=6: 20640/552960; random: 8006/500000
MUT N 控制：去掉『txg 与攒着的上一条相同』那一断 | Verus 预测=红 | 编译=过 | 2 inst=true slots=40: 144/46080; 3 inst=false slots=40: 2084/552960; 3 inst=false slots=6: 2048/552960; random: 1425/500000
```
读表：A 的 `slots=40` 两列为 0 是因为那两个取样集的记录条数不到上限（13），`slots=6`（上限 2）与随机一族抓到；C 的穷举结构集抓不到是因为取样集里没有「上一条不带提交标记且事务号不同、后面又不被别的判截住」的组合，随机一族抓到；F 的结构集里没有点名项所以为 0，随机一族抓到。11 个 Verus 预测为绿的改动，差分都抓到了，即：它们今天只靠平时的单测与 harness 档的那些用例拦。
I 行（对应 `crates/mutations.tsv` 第 448 行原来那条「无锚点那一支不看序号」，核里第 456 行那一判兜着）的特别之处：把差分的返回值与读盘日志拆开看，它的 78 个展示的差异块里返回值全相同、只有读盘日志不同（脚本在草稿目录，取法：对 `mut_I.log` 里每个 `DIVERGE` 块比 `old=` 与 `new=` 两行）。也就是它在返回值上确实等价，在「读盘次数与次序」上不等价（反向链回调先于序号判定，多读一次盘）。题面 C1 把读盘次序列进「同一结局」，所以这一支今天的 `crates/mutations.tsv` 第 448 行已经换成真变异是对的；用 Verus 证的「等价」不覆盖读盘次序（读盘次序不在 `ensures` 里，Verus 也说不了它）。

### 3.3 `requires` 在产品路径上谁担保

- 计数器 `< u64::MAX`：解析的 6 字节（见 2.4）。运行时无判、类型无挡。
- 回调的 `requires`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:289`–293，对每个下标 `.requires((record_index,))`）：`replay_journal` 里两个回调（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3080`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3090`）是不带前置条件的闭包，任何下标都满足；核只用 `index < records.len()` 的下标调它们（循环不变式 `/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:375`–382），回调里 `records_in_key_order[index]`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3083`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3091`）因此不越界。
- 空真 / 恒真排查：`requires` 两组都可满足（平时单测与 56 924 864 个输入都走进了核的 `Ok` 分支，`applied_some` 非零）；`m2_holds` 的每个全称式在 `applied` 非空时都有内容；`is_concatenation_of_whole_publishes` 有两条冒烟证明（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:570`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:587`）分别证真例与假例；`ensures` 的 `Err(_) => true` 与 K 一样是空真，见 3.2。循环不变式里 `report.verification_failed == 0`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:367`）是 `invariant_except_break`，失败那一支先加一再 `break`，所以不构成过宽；`expected_next is None ==> open@.len() == 0`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:417`）是证第 443 行 `assert(open@.len() == 0)` 用的，有用。
- 没有发现一处靠空真撑着的证明；有的是 3.1 表里的「ensures 本身没说」。

### 3.4 `verus-mutations.tsv` 38 行改的是不是有意义的判定

38 行数法：`grep -vc '^#' crates/singlefs-replay-prefix/verus-mutations.tsv` 输出 38（该文件共 46 行含 8 行注释）。按第一列前缀数：「ensures·」8 行、「循环不变式·」30 行（`awk -F'\t' … split($1,a,"·")` 数出）。这些行的原文列抽看：`Err(_) => true` 改 `false`（第 12 行）、`above_the_chosen_root` 前加 `!`（第 13、16、36 行）、不变式下标界收紧一格（第 17 行）、回调前置条件多要一格（第 32 行）、`rebuilt == if applied.len() == 0` 改 `!= 0`（第 39 行）、`expected_next is None` 改 `is Some`（第 46 行）。除第一条数据行（函数体与 `ensures` 一起改恒真）外，没有一行动可执行判定而保持 spec 不变；而 3.2 的量说明，真正要拦的是那些「改可执行判定、spec 不动」的世界，它们一行都不在这张表里。按跑前判据第五节第 3 条（「一行变异或一条测试证不到它的名字说的那件事 ⇒ 打中 C5」）：这 38 行证到的是「证明没有空转、注解写错会红」，证不到「核的判定被改坏会红」；后一件今天靠 harness 档与 `crates/mutations.tsv` 的行。

## 四、C3　编进二进制的与 Verus 看的

取出平时编译的展开结果：`RUSTC_BOOTSTRAP=1 cargo rustc --offline -p singlefs-replay-prefix --lib -- -Zunpretty=expanded`（冻结副本拷贝，经 `capped.sh 2`；产物 `logs/replay-prefix-erased-expanded.rs`，326 行）。读后结论：
1. `verus!` 里的 spec、`requires`、`ensures`、`invariant`、`decreases`、`proof { }`、`ghost` 全部擦除；留下的是 F 里非幽灵的控制流，逐行与 F 的可执行行同形。检查：展开文件里 `grep -n 'verifier\|cfg('` 零命中（命令输出为空）。
2. 唯一的幽灵残留：`CoreOutcome.applied`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:47`）展开成 `Ghost::assume_new_fallback(|| …::core::panicking::panic("internal error: entered unreachable code"))`（展开文件第 312–324 行）。看 `verus_builtin` 的实现：`/home/fy5090/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/verus_builtin-0.0.0-2026-09-16-0054/src/lib.rs:517` 是 `pub const fn assume_new_fallback(_: fn() -> A) -> Self {`，下一行返回 `Ghost { phantom: PhantomData }`，闭包参数不被调用，所以那个 panic 永远不会触发；56 924 864 个输入里 `panics_new` 与 `panics_old` 恒相等也印证。
3. 算术：Verus 对 `usize`、`u64` 的加法要求证无溢出，擦除后是普通 `+`，在 `overflow-checks = true` 下是运行时检查；证明说它们不溢出，与平时行为一致。`counter + 1`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:350`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:478`、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:162`）靠 `requires`，见 2.4。
4. 判定词序：`&&` / `||` 在 Verus 与 Rust 里都短路，`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:491` 的懒求值同。
5. 闭包调用：Verus 里调回调要证 `.requires`，擦除后是直接调用，次数与次序同 F 的写法。
6. 依赖形状：`Cargo.lock` 里 vstd 要求 `verus_builtin = "=0.0.0-2026-09-16-0054"`、`verus_builtin_macros = "=0.0.0-2026-09-20-0158"`（`~/.cargo/registry/src/*/vstd-0.0.0-2026-09-20-0158/Cargo.toml` 第 52–59 行），所以版本一致由 vstd 自己钉；crates.io 的 vstd 与 Verus 0.2026.09.20.aef82ed 自带那份是否同源，本机没有 Verus 发布目录（`which verus` 无输出，`find` 也无），复核不了。
7. 没有看到别的差别：`#[allow(clippy::too_many_arguments, reason = …)]`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-replay-prefix/src/lib.rs:274`）在展开里保留为属性，不影响语义；`#![forbid(unsafe_code)]` 只管这个 crate，vstd 自己不在 forbid 里（我没读 vstd 全文，未核）。
推翻条件：Verus 自带的 builtin / macros 版本与上面三个版本有源码差（主 agent 已说现查过，我复核不了）；或平时编译换了宏版本而 `Cargo.lock` 没跟（`Cargo.lock` 已钉）。

## 五、C7　判法版本闭包与崩溃放量

### 5.1 判法摘要变不变、`use` 自动进闭包成不成立

量法：冻结副本之外建一个草稿探针 crate（`digest-probe/`，只依赖 `singlefs-harness`），用 `#[path]` 把 `crash_identity.rs` 编进去，Layer0ToolchainIdentity 用固定串占位（只比相对变化，不比绝对摘要），调其中非测试的 `judging_code_of_the_judges`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:837`）在七份仓树上各算一次：HEAD fcc4d340（`git archive` 解出）、冻结副本、冻结副本加五种单点改动。它不是 checker 档的测试二进制，也没起任何判器；是否算越线交主 agent 判（我认为不算，列在「没做什么」）。原样输出（`logs/digest-probe.out` 折成一行一份）：
```
ROOT head  digest cbb9dbdcbfd19bc983d8c7d49f5f184a69dbf5c2ee873d5ccb2764323a556943  files 44
ROOT dv/new  digest 0322723006427b35f9685ef4f605f7294491c3ddfb05d65ba9374f1334af1ce4  files 45
ROOT dv/v_code  digest aac7870d9422a64246a4b1b7739aa2b9351a4f2bcd3c403c3ba53b1e7ebdf70c  files 45
ROOT dv/v_comment  digest 0322723006427b35f9685ef4f605f7294491c3ddfb05d65ba9374f1334af1ce4  files 45
ROOT dv/v_tsv  digest 0322723006427b35f9685ef4f605f7294491c3ddfb05d65ba9374f1334af1ce4  files 45
ROOT dv/v_manifest  digest 81423fd5e09f4d598c2d275c3ef5f540a28b73519d8f25de078d909d39faef71  files 45
ROOT dv/v_recovery_comment  digest 0322723006427b35f9685ef4f605f7294491c3ddfb05d65ba9374f1334af1ce4  files 45
ROOT dv/v_vstd  digest 6175960a9ce0baf6bf684e84da0edaeb65209a8e217933b53ecef33889436745  files 45
```
（`v_code`＝核 `lib.rs` 末尾加一条 `pub const`；`v_comment`＝核 `lib.rs` 末尾加一行 `//` 注释；`v_tsv`＝`verus-mutations.tsv` 末尾加一行；`v_manifest`＝新 crate 的 `Cargo.toml` 末尾加注释行；`v_recovery_comment`＝`recovery.rs` 末尾加注释行；`v_vstd`＝`Cargo.lock` 末尾加注释行。）
读出来的：
1. 摘要变：改前 44 份文件，改后 45 份，摘要不同。多出的一份是 `crates/singlefs-replay-prefix/src/lib.rs`，收录原因是 `referenced by crates/singlefs-core/src/recovery.rs`，与 `crash_identity.rs` 收法「顺着 `use` 收」一致（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:866`「已收文件里所声明类型或 trait 的文件，反复到收不到新文件为止」那一段的文档与此相符）。`use singlefs_replay_prefix::{replay_core, CoreFailure, CoreRecord};`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:20`）经 `workspace_crates` 把 `-` 换 `_` 的名字表（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:442-445`）认到目录，不需要在两条测试清单之外另登记。
2. 判器真的跟核走：核 `lib.rs` 里加一条真语句，摘要变；只加注释，摘要不变（`lexical_tokens` 去注释）；改 `verus-mutations.tsv`，摘要不变（不是 Rust 文件、不被引）；改新 crate 的 `Cargo.toml` 或 `Cargo.lock`，摘要变（每个收到的 crate 的清单与整份 `Cargo.lock` 的原文摘要都拼进去）。
3. 保守方向的差：摘要按词法单元，`verus!` 宏里的 spec、证明块、核里两条单测的改动也让摘要变，即使判定没变；改证明不改判定会白白废一次判法版本。这是宁多不漏，不是漏。
4. 这一批改核之后，已存的崩溃放量判定因判法摘要变而不再被复用，要按身份机制重判（这是 C7 题面「判法版本变不变」的答案：变）。我没有去看存储里是否已有按新摘要重判的记录，按定义不归我。
5. `crash_identity.rs` 两条测试里新加的 `crates/singlefs-replay-prefix/src/lib.rs`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:1164` 在判器闭包必含清单、`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:1275` 在摘要必变清单）：我没跑那两条测试（checker 档），只确认它们要的性质（在闭包里、改它摘要变）被探针独立量到。

### 5.2 GPU 判器另写的 WGSL `replay_journal` 与改后的核

- GPU 一侧不调核：`grep -rn 'replay_core\|singlefs_replay_prefix\|replay-prefix' crash_judge_gpu.rs crash_judge_dispatch.rs crash_verify_gpu.rs *.wgsl`（在冻结副本的 `crates/singlefs-checker-tier/src` 里）零命中。
- WGSL 的 `replay_journal` 在 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:1170`，文件里自己标注是「recovery::replay_journal（Consult：验点名单元）」（同文件 1169 行）的另写一份；它写成时对的是改前的 Rust。核与改前经上面 56 924 864 个输入证明同结局，所以「与改后的核是否一致」归约为「与改前是否一致」，这一归约不需要它调核。
- 读 WGSL 与核的分支次序：锚点个数 >1 报错（wgsl:1185）→ 根 txg 为最大值报错（wgsl:1190）→ 在飞上限（wgsl:1212 `if (taken >= limit) {`）→ 无锚点链首 / 计数器连续 → 反向链（wgsl:1226）→ 一次发布首条序号为 1 → txg 同、序号 +1、提交标记连续 → 末条不带提交标记（wgsl:1250）→ 末条之后同发布还有读得出的记录（wgsl:1254）→ 点名单元（wgsl:1258）；与 F 的 427、335–356、452、456、463–475、480、484、491 行次序一致。
- 它与核共有的前提：记录表按（实例代号，计数器）升序（wgsl 的 `readable_record_of_the_same_publish_follows`，1123–1139 行，遇到别的实例就 `break`，核是 `continue`，在升序表上同值）。
- WGSL 没有「点名校验关着」那一支（只有 Consult）；这与改前一样，不是这一批引入的。
- 我没有机器比过 WGSL 与核（要显卡与 checker 档的判器），所以「一致」只到读代码这一层；推翻条件：GPU 判器与 CPU 判器的对账（`crash_verify_gpu` 一类）在含 `replay_journal` 分歧的状态上出红。

## 六、其余格里顺手看到的（不是我的格，只记看到的）

### 6.1 C5　新 crate 自带的两条单测（量过）

同一批核改动（A–N）再跑 `cargo test --release -p singlefs-replay-prefix --lib`（`mutate_unit.py`，原样输出在 `logs/unit_out.txt`）：
```
UNIT A 去掉在飞记录数上限那一断 | test result: ok. 2 passed; 0 failed
UNIT B 去掉反向链回调那一断 | test result: ok. 2 passed; 0 failed
UNIT C 去掉『上一条没提交标记而换了事务号』那一断 | test result: ok. 2 passed; 0 failed
UNIT D 去掉『末条不带提交标记』那一断 | test result: ok. 2 passed; 0 failed
UNIT E 去掉『末条之后同发布还有读得出的记录』那一断 | test result: ok. 2 passed; 0 failed
UNIT F 点名单元校验恒过 | test result: ok. 2 passed; 0 failed
UNIT G 去掉 jsn 严格连续那一断（已知：核没证它） | test result: ok. 2 passed; 0 failed
UNIT H 无锚点时不看 txg = 根 txg + 1，只看序号 | test result: ok. 2 passed; 0 failed
UNIT I 无锚点那一支不看序号（tsv-448 原来那条；核里另有第 456 行那一判兜着） | test result: ok. 2 passed; 0 failed
UNIT J maximum_applied_transaction 不更新 | test result: FAILED. 1 passed; 1 failed
UNIT K 一条都不施加（取到第一条就收工） | test result: FAILED. 1 passed; 1 failed
UNIT L 控制：prefix_applied 少加 | test result: FAILED. 1 passed; 1 failed
UNIT M 控制：去掉『一次发布第一条序号是 1』那一断 | test result: ok. 2 passed; 0 failed
UNIT N 控制：去掉『txg 与攒着的上一条相同』那一断 | test result: ok. 2 passed; 0 failed
```
读出来：两条单测只拦到 J（最大事务号不更新）、K（一条都不施加）、L（`prefix_applied` 少加）；A–I 全放过（包括反向链、提交标记、点名校验、在飞上限、无锚点判、连续判），连 Verus 预测会红的 M（去掉「首条序号为 1」）、N（去掉「同 txg」）也放过。两条单测的回调是常量闭包（`|_index: usize| false` / `|_index: usize| true`）、点名校验开关关着、记录只有三条，所以这些判定走不到。核的其余判定今天靠 harness 档与 `crates/mutations.tsv` 的行，这是 C5 的格，归攻方腿与主 agent 核，我只给上面这张量好的表。

### 6.2 C4　公开 API 被误用（看到的三处）

1. `replay_core` 的 `records` 入参要升序且键等于字段：没有类型、没有 `debug_assert!`、文档只在内部注释（见 2.3）。
2. 计数器 `u64::MAX`：核的 `requires` 与运行时 panic 两件事并存（见 2.4）；`Cargo.toml` 的 `overflow-checks = true` 是在仓根 workspace 的 `[profile.release]`，作为外部依赖被别的 workspace 引用时不继承。
3. `replay_core` 的 `Segment` 是任意 `Copy` 类型，`rebuilt` 的含义（「末条的新根段」）在 spec 里是 `Segment` 等式，核不知道里面是什么；glue 用 `Option<usize>`（下标），所以「新根段取哪条记录」的正确性实际靠 glue 的 `new_root_segment: Some(index)`（`/tmp/claude-1000/m3-verus-land-code-r1/tree/crates/singlefs-core/src/recovery.rs:3075`）与 `records_in_key_order[index]` 同一次序，由 2.1 的比对覆盖，没有证明覆盖。

### 6.3 C6　顺手看到的

- `is_above_the_chosen_root` 的 `record.instance > root_instance ||` 是死支（见 2.2 第 1 条），注释说「摊开字典序」，实际上那一支永假。
- `.claude/singlefs-ai-sop/rules/code-discipline.md` 我没逐条对，C6 归攻方腿。

## 七、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| C1 改前改后等价（返回值、报告、报错、读盘次序、panic） | 一致 | 56 924 864 个输入零差异，阳性对照四个全红；唯一差在「键 ≠ 字段」的表上（300 000 里 15 734），今天无调用方构造，是未写明的前置条件，零轮改法见 2.3 |
| C1 `unreachable!` 新 panic 路径 | 一致 | 条件与改前 `len() > 1` 同一过滤，同一张表，报错内容逐字相等 |
| C1 懒求值与读盘次序 | 一致 | 日志逐条比过；读盘次序不在 Verus 的 `ensures` 里（I 行：只有日志不同的变异，返回值全同） |
| C2 spec 与 D13 M2 / D23 14、22 | 冲突（spec 太弱） | 只含「整次发布」与「严格大于所选根」，其余五条口径与链首等都不在；「一条都不施加」满足 `m2_holds`（推，Verus 没跑） |
| C2 `requires` 担保 | 规则没说 | 产品路径上靠 6 字节解析；`replay_journal` 无运行时判 |
| C2 空真 / 恒真 | 一致 | 没找到靠空真撑的证明；`ensures` 的 `Err(_) => true` 是空真但单列在「spec 太弱」里 |
| C2 `verus-mutations.tsv` 38 行 | 冲突 | 38 行都在改注解（8 条 `ensures`、30 条不变式；第一条数据行连一行函数体同改），没有一行改核里做判断的可执行码 |
| C3 擦除后的可执行码 | 一致 | `-Zunpretty=expanded` 读过，无 cfg、无 `#[verifier::…]`、`Ghost` 回退闭包不被调 |
| C3 Verus 发布与 crates.io vstd 同源 | 复核不了 | 本机无 Verus 发布目录 |
| C7 判法摘要 | 变 | 44 → 45 份文件，摘要不同；`use` 自动进闭包成立；注释与 `verus-mutations.tsv` 的改动不变摘要，改证明变摘要 |
| C7 GPU WGSL `replay_journal` | 规则没说 | 不调核；分支次序与核逐条对得上；没机器比过 |

## 八、没做什么

- 没跑 Verus（本机没有发布目录，派发也不让跑）；所有「Verus 预测」都是读 spec 与不变式推的，每条的（原文、替换文）在 `mutate_core.py`，主 agent 判决时要核就搬去现跑。
- 没跑 checker 档：`crash_identity.rs` 的测试、E165 装置、`crates/mutations.tsv` 整表都没动；`digest-probe` 是我自己写的草稿 crate，只调了 `crash_identity.rs` 里非测试的 `judging_code_of_the_judges`，不起判器、不读存储；它算不算越线交主 agent 判。
- 没跑 harness 档的 19 个目标；也没做 G1–G12（C4）、C6 逐条、C8（E165）。
- 没读 vstd 全文，没核 `#![forbid(unsafe_code)]` 对依赖的覆盖。
- 没写入或改动冻结副本、仓里的任何 `crates/` 文件；所有改动在草稿目录的拷贝里（`tree`、`tree_m`、`dv`、`head`、`probe`）。
- 不替主 agent采纳或出判决；改法只提「零轮」的最小一改（2.3），没有为性能提任何改法。
- 交回前删掉自己建的编译目录与仓副本，清单见交回正文。
