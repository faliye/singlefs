# m2-rest-batch2 实现员报告（2026-09-28）

## 结论

- ① 靶向用例 + 变异行：做了。`CrashPointDraw` 加了第三支 `ExactlyOneCrashState`（段号 + 从段首数的写数 + 扣下哪几个），两条判据各一条靶向用例，mutations 加了攻方变异 A 与第二条判据各一行。这些都在 checker 档，只做到编得过（`cargo check -p singlefs-checker-tier --tests --bins` 退 0），**没跑、没证红**，归提交时崩溃验证员（59 号）。
- ② harness 档用例：做了。`cargo test -p singlefs-harness --test named_unit_verification_stands_in_for_the_barrier_before_the_record` 2 条绿；两行变异都在副本上用 prove-red.sh 证红。
- 补丁叠在第一批之上：副本 = 主工作区（含已暂存）+ `research/prompts/m2-rest-batch1/crates.patch` + 第一批 mutations 追加，打完第一批后建成基线。`git apply --check` 在这个基线上过。
- 需要主 agent 知道的：74 号在副本上红，原因在 HEAD `e5253e8a`，不在这次改动里（见「门禁」一节）。
- 没拷进 `research/prompts/m2-rest-batch2/`：我的写范围不含 `research/`（见「没做什么」）。

## 靶向状态怎么造的（交主 agent 的判断点）

今天的持久顺序（`crates/singlefs-core/src/transaction.rs:1058`、`:1065` 两道屏障）下，两条判据防的状态在「更早段整段持久、当前段取真子集」这个枚举域里一个都摆不出来。所以任何「判据该红」的状态都要越过屏障。我没有用 `ConsultWithoutNamedVerification`：它要给 `inject_crashes_into_history` 加恢复策略参数，而且光关验证还不够，还是要越过 1058 才有「记录在、单元缺」的状态。

做法是让第三支的持久集合可以越过段尾，越过的屏障当设备没守：
- `ExactlyOneCrashState { segment_index, writes_from_the_segment_on, withheld_among_them: &'static [usize] }`（`crash_injection.rs:197`，分支在 `:1746`）。更早的段整段持久，这几个写之后的一个都不持久。输入不成立就 panic：段号不是候选段、写数为 0 或越过写表末尾、末尾越过最后一个跑完的操作、扣下的下标为空或越界。
- 用 `&'static [usize]` 是为了让 `CrashPointDraw` 保留 `Copy`，这样 `CrashInjectionCampaign`（derive Copy）、e158 装置和别的测试一处都不用改。所以派发列的 `e158_root_choice_repair.rs`、`crash_injection_writable_mount_after_the_crash.rs`、`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs` 我没碰：它们只构造 `Sampled`，没有对 `CrashPointDraw` 的 match。全仓对它的穷举 match 只有 `draw_crash_points` 这一处。
- 连带两处：`withheld_write_kinds` 与崩溃点 `step` 的下标，从 `segment[k]` 改成段首加 k（段是写表里连续的一截，`inject_crashes_into_history` 里 `writes[first..first+len]` 早就这样假设了）。`candidate_segments` 为空时的提前返回，挪进了 `Sampled` 那一支（新支要报出「不是候选段」，不能静默返回空；`EveryProperSubsetOfShortSegments` 在空候选上本来就是空，行为不变）。

**这是设计判断，交主 agent 定**：「越过屏障 = 设备没守屏障」这种状态拿来当靶向用例合不合适。推翻它的现象：有人论证记录核对器只该对守屏障的状态有定义，那两条用例就是在判它的定义域外。

## 两条靶向用例（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs`）

共用历史 `history_of_a_first_file_and_two_overwrites`（`:1192`）：`AfterFirstFile` 起点，再做两次覆盖写，得到发布 (1,3)、(1,4)、(1,5)。段形是在副本上用 harness 探针量的（见下），用例里先逐项核段形，核不上就在那一步红。

| 用例 | 状态 | 探针量到的判定（`Consult`） |
|---|---|---|
| `a_crash_state_with_the_root_but_neither_of_its_records_is_a_finding_of_the_record_checker_alone`（`:1233`） | 第 21 段起 3 个写 = (1,5) 的两份记录 + 它的根槽写；扣 [0,1] | 恢复落到 (1,5)、读回内容；模型 None；checker `[]`；`RecordCheck { root_without_record: true, claimed_state_missing_unit: false }` |
| `a_crash_state_landing_after_a_publish_whose_data_unit_never_landed_is_a_finding_of_the_record_checker`（`:1298`） | 第 12 段起 56 个写 = (1,3) 的单元写起，到 (1,4) 的根槽写；扣 [0,1] = (1,3) 数据单元的两份 | 恢复落到 (1,4)；模型 None；checker 红 I-2.1、I-4.8、I-7.4（环里 (1,3) 的根走不通）；`RecordCheck { root_without_record: false, claimed_state_missing_unit: true }` |

断言：只摆 1 个崩溃状态；对应计数 == 1；崩溃镜像那一截的新发现签名是 `RecordCheck { aspects: [那一条] }`。第一条另断言 checker 与模型都不红，第二条另断言模型不红。签名按 `FailureSignature::of` 的优先次序，是「记录核对器先于 checker」。

**探针**：量这些用的不是 checker 档测试，是 harness 包里的草稿测试（`/tmp/claude-1000/impl-m2-rest-batch2/probe/zz_probe_batch2.rs`，记录核对器是 `crash.rs:104-333` 的原样拷贝，`diff` 无输出）。它手工摆同一个持久集合，跑 `recover`、理想模型、`check_pool_image`、记录核对器；再在崩溃后镜像上起一次可写挂载、发一次布（两个状态都 `mount ok, publish Ok(())`）。它没跑 `inject_crashes_into_history` 本身，也没跑第三截（二次崩溃）。原样输出在 `/tmp/claude-1000/impl-m2-rest-batch2/probe2.log`。

### 证红（checker 档，全部留给提交时 59 号，没跑）

| 变异行（追加进 mutations） | 改坏哪一行 | 预期红在哪（推的，没跑） |
|---|---|---|
| 增补 3 第 3 件第 1 题：崩溃镜像那一截保留记录核对器的调用与计数、只把判据从通过条件里丢掉（攻方变异 A…） | `crash_injection.rs:905-908` 的 `if !(… && record_check == RecordCheck::default())` 去掉第三个合取项 | 第一条用例的 `finding_on_the_crash_image` 那一处 panic「崩溃镜像那一截没有判红」（那个状态 checker 与模型都绿）。第二条用例在这个变异下仍绿（checker 红，照样有新发现、签名仍是 RecordCheck），所以这一行只点名第一条 |
| 增补 3 第 3 件第 1 题：崩溃镜像那一截交给记录核对器的恢复落到的那一版换成 None（…） | `crash_injection.rs:895` 的 `report.effective_root,` 换成 `None,` | 第二条用例的 `record_claimed_state_missing_unit == 1` 那条断言（计数变 0；新发现签名变成 CheckerViolations） |

## ② harness 档用例（`crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs`，新建，用的就是派发给的名字）

状态（`record_persisted_without_one_of_its_units`，`:50`）：`AfterFirstFile` 起点，覆盖写一次得到 (1,4)，用录制器录下整条流。持久集合 = (1,4) 根槽写之前的全部写，扣掉 (1,4) 第一个单元写那个偏移上的两份（两块盘各一份）；根槽写与之后的都不持久。函数自己先断言：扣下的正好 2 份，(1,4) 的记录每份都持久，盘上已持久的最新根槽是 (1,3)。core 没改。

| 用例 | 断言 | 变异（在副本上用 prove-red.sh 证红） | 红在哪（原样） |
|---|---|---|---|
| `with_named_unit_verification_a_record_whose_unit_never_landed_falls_back_to_the_previous_version`（`:149`） | `Consult`：`verification_failed > 0`、`prefix_applied == 0`、`effective_root == (1,3)`、模型不分歧 | `recovery.rs:3161` 的 `let all_verified = !verify_named_units` 改成 `= true` | `:153:5` 那条「点名验证读到了缺的那个单元、判了失败：JournalScanReport { valid_records: 4, above_water: 1, prefix_applied: 1, verification_passed: 1, verification_failed: 0, maximum_applied_transaction: 2 }」 |
| `without_named_unit_verification_the_same_state_is_a_model_disagreement`（`:178`） | `ConsultWithoutNamedVerification`：施加了记录、`effective_root == (1,4)`、模型报分歧 | `recovery.rs:4246` 的 `policy == JournalPolicy::Consult,` 改成 `policy != JournalPolicy::Ignore,`（关不掉验证） | `:182:5` 那条：`left: (false, Some((InstanceGeneration(1), CheckpointTxg(3))))` / `right: (true, Some((InstanceGeneration(1), CheckpointTxg(4))))` |

prove-red 原样输出（最后一次跑在最终源码上，`/tmp/claude-1000/impl-m2-rest-batch2/prove-red-2.log`）：
```
K3-⑨ 两道防线至少留一道：恢复端点名验证短路（all_verified 恒真），记录已持久而它点名的单元两份都没落时照样施加、走读失败	抓到	with_named_unit_verification_a_record_whose_unit_never_landed_falls_back_to_the_previous_version 红了（日志 /tmp/claude-1000/impl-m2-rest-batch2/repo/prove-red-logs/001.log）
K3-⑨ 两道防线至少留一道：只供测试的开关关掉点名验证那一臂照样验（ConsultWithoutNamedVerification 走成 Consult，记录已持久而单元没落的状态上模型不再分歧）	抓到	without_named_unit_verification_the_same_state_is_a_model_disagreement 红了（日志 /tmp/claude-1000/impl-m2-rest-batch2/repo/prove-red-logs/002.log）
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
```
基线（不改源码）是绿的，没有基线红集。「同时还红了哪些测试」：prove-red 只跑点名目标里筛到的那一条，别的没看。harness 档不依赖 checker 档：94 号在副本上绿，闭包里没有 checker 档包。

## 写过的文件（副本里；主工作区一个字没改）

- `crates/singlefs-checker-tier/src/crash_injection.rs`（改）
- `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs`（改：导入 + 末尾 3 个辅助函数 + 2 条用例）
- `crates/singlefs-harness/tests/named_unit_verification_stands_in_for_the_barrier_before_the_record.rs`（新建）
- `crates/mutations.tsv`（只在末尾追加 4 行）：「增补 3 第 3 件第 1 题：崩溃镜像那一截保留记录核对器的调用与计数…」「增补 3 第 3 件第 1 题：崩溃镜像那一截交给记录核对器的恢复落到的那一版换成 None…」「K3-⑨ 两道防线至少留一道：恢复端点名验证短路…」「K3-⑨ 两道防线至少留一道：只供测试的开关关掉点名验证那一臂照样验…」。证过的：后两行（prove-red）。留给 59 号的：前两行（checker 档）。
- 没碰：e158 装置、`crash_injection_writable_mount_after_the_crash.rs`、`crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`（理由见上）。

`git diff --stat -- crates litmus`（副本，相对打完第一批之后的基线）原样：
```
 crates/mutations.tsv                               |   4 +
 .../singlefs-checker-tier/src/crash_injection.rs   |  81 +++++++--
 .../tests/crash_injection_campaign.rs              | 184 +++++++++++++++++++-
 ..._stands_in_for_the_barrier_before_the_record.rs | 193 +++++++++++++++++++++
 4 files changed, 446 insertions(+), 16 deletions(-)
```
补丁目录 `/tmp/claude-1000/impl-m2-rest-batch2/patch/`：
```
8234607e769585fc7f8a642e4b690fd93a81f77478201dd799bfc8704c2252f8  patch/crates.patch
d6059302f5c51a73f3704e9e437fb6b3287ee5cb628c27d9273128e7e1be0f04  patch/mutations-append.tsv
```
另外还有 `report.md`（这份报告）。补丁没有动 `crates/singlefs-checker/src/`，所以不需要「受影响的层 0 流与崩溃枚举用例」一节。

## 交回前的验证（都在副本上跑，最终源码；线程 16，内存 16G）

- `cargo fmt --all -- --check`：退 0，没有输出
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 `CODE_DISCIPLINE_LINTS` 那 7 条：退 0。先 touch 了三份改过的文件，末两行原样：`    Checking singlefs-checker-tier v0.1.0 (…/repo/crates/singlefs-checker-tier)` / `    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.76s`
- `cargo build --offline --all-targets`：退 0，`    Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 9.94s`
- `cargo check --offline -p singlefs-checker-tier --tests --bins`：退 0
- `cargo test -p singlefs-harness --test named_unit_verification_stands_in_for_the_barrier_before_the_record`（经内存包装）：`test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s`
- `research/scripts/crash-case-check.py`（四样）：`✓ crash-case-check 判了 4 样（placement、modules、file-names、one-scenario），0 样无对象可判`

## 门禁（登记给我的：33、74、92、94；53、93、89 号表里的名字今天在 gate.d 里找不到文件）

- 33 号 `--check mutation-tables`：退 0。`✓ … crates/mutations.tsv 1429 条的原文各命中源码一次 …`、`✓ 实验与变异源码纪律跑了 1 格，判过 1 格：mutation-tables；本次未跑 0 格：无`。中途红过一次：我把 `draw_crash_points` 的候选段判定抽成闭包，碰掉了第 155 行变异的锚点。已改回原写法，没有改那一行。
- 92 号：退 0（`✓ 布局清单 1 套布局、6 条路径都在 … 111 个格式常量里变了 0 个 …`）
- 94 号：退 0（`✓ checker 与实现只共享常量模块 … harness 档的依赖闭包 3 个里没有 checker 档包，它的 102 份源码零处引它`）
- **74 号：退 1**，原样：`✗ 测试跑过了，这几段却没有「模型对拍 N 步」那一行：「随机历史快档」「随机历史：偏向抬 F 之后复用的取样点」…（六段全缺）`。这不是这次改动造成的：HEAD `e5253e8a` 给 `crates/singlefs-harness/tests/random_histories.rs` 的这几段标了 `#[ignore = "harness 耗时用例…"]`（`git show HEAD:…random_histories.rs | grep -c '^#\[ignore = "harness 耗时用例'` 得 15），74 号跑 `cargo test --release … --test random_histories` 时没带 `--include-ignored`，这几段就一段都不跑。我的补丁没碰 `random_histories.rs`、`history.rs` 和 74 号。不归我修，交主 agent 转给 74 号的归属者。

## 停下交主 agent 的设计问题

1. 「越过屏障的持久集合」当靶向状态合不合适（见上文）。
2. 第二条靶向状态在 checker 上也红（I-2.1、I-4.8、I-7.4），所以攻方变异 A 只能钉第一条；第二条靠「effective_root 换成 None」这一行钉。要不要找一个只有记录核对器红、并且判的是第二条判据的状态：我没找到。缺单元的发布只要它的根还在环里，从那条根出发的 checker 遍历就会红。

## 没做什么

- 没跑 checker 档的任何测试（重型）：两条靶向用例绿不绿、两行 checker 档变异红不红，都没验，归提交时 59 号与 54 号快档。靶向用例的第二、三截（可写挂载之后、二次崩溃）会不会给出意料之外的 panic：探针只起过一次挂载加一次发布，二次崩溃没复刻。
- 没把补丁拷进 `research/prompts/m2-rest-batch2/`：写范围表 `.claude/hooks/agent-write-scope.tsv` 里 implementation-writer 只有 `crates/**`、`litmus/**`、`/tmp/claude-1000/**`，定义也写了「不写 research/」。请主 agent 从 `/tmp/claude-1000/impl-m2-rest-batch2/patch/` 拷。
- 没走三方对抗；checker 档（54 号快档与全量、QEMU、herd7、crates 变异表）归 crash-verifier；没提交。
- 线程上限在收尾阶段改成了 10（主 agent 消息），那之后没有再起编译与测试。

## 草稿目录

- 删了：`/tmp/claude-1000/impl-m2-rest-batch2/repo`（仓副本连同它的 target，删前 `du -sh` 22G）。prove-red 的日志删前拷到了 `/tmp/claude-1000/impl-m2-rest-batch2/prove-red-logs/`（上面原样输出里的 `repo/prove-red-logs/` 路径指的就是它们）。
- 留着：`patch/`（要交）、`probe/`（探针与记录核对器拷贝，复查用）、各份日志（`probe2.log`、`prove-red-2.log`、`clippy-2.log`、`build-2.log`、`gate-*.log` 等）、`attack-report.md`（从 `11a551b2^` 取出的攻方报告，只作参考）。
