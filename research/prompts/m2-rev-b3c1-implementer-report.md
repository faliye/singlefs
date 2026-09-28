# 实审 B3c-1 报告（implementation-writer）：崩溃注入第一截交记录核对器整条流、快档标 ignore 加小快档

开工 2026-09-27。

## 一、结论

- **装置错修了**：`crates/singlefs-harness/src/crash_injection.rs:882-883` 第一截交给记录核对器的写表，从「到崩溃点所在段为止的前缀」换成整条历史录制流 `&writes`，持久集合用同一张 `persisted`（更早的段整段持久、当前段按崩溃点子集、更晚的段全没持久），与层 0 的交法一致。种子 7463871032432355115 那个崩溃状态（第 56 段扣 #355 留 #356，恢复落到由记录重建的 (5, 15)，根槽写 #357 在第 57 段）上，记录核对器读得出 (5, 15) 的实例表、豁免被它抛弃的 (1, 4)，不再判「恢复自称新态而单元缺席」。
- **新用例三条**，两条证过会红：用例 A（种子基 + 2 那段历史，改前红、改后绿）、用例 B（整条流下扣 (5, 14) 的一个节点照样红，判别力还在）。第三条是小快档，**在当前工作区上红**，红因是 checker 的 I-7.4 新发现（种子 …115 第 29 段），不是这次改动；它在基线红集里，照定义停下没证红，交主 agent（第六节第 1 条）。
- **快档标了 ignore**，原因写在属性里（交崩溃验证员经 54 号跑）；`stage-inputs.tsv` 要登记的 `crash-case:` 行原文在第五节。
- **变异表**（`crates/mutations.tsv` 没碰，行在草稿目录）：`mutations-append.tsv` 三行，证过两行，一行待证；`mutations-replacements.tsv` 两行：第 155 行的锚点被这次改动改没了（门禁 33 号点名），替换行证过；第 152 行点名的快档标 ignore 之后跑不到，替换行改点小快档，同样卡在小快档的基线红上，没证。
- **什么现象会推翻它**：一是改回前缀交法（变异 M1）之后用例 A 还绿；二是整条流下扣掉 (5, 14) 的节点之后记录核对器不红，那就是整条流把判据弄瞎了。这两样都造过，都红了（第四节）。

## 二、写过的文件

- `crates/singlefs-harness/src/crash_injection.rs`：只动第一截那一处，删掉 `writes_up_to_this_segment`，改了调用与注释。
- `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs`：快档标 ignore，改了规模常量的注释，加 `SMALL_FAST_TIER_SEEDS` 与小快档，加用例 A / B 和它们的辅助函数，`use` 补齐。
- 草稿目录里的变异行（不是仓内文件）：`/tmp/claude-1000/impl-rev-b3c1/mutations-append.tsv`（3 行）、`/tmp/claude-1000/impl-rev-b3c1/mutations-replacements.tsv`（2 行）。
- 没碰 `crash.rs`（B3a-2 在改）、`crates/mutations.tsv`。
- 这两份相对开工快照的 diff 在草稿目录：`my-crash_injection.diff`（33 行）、`my-test.diff`（526 行）。`git diff --stat` 原样在第七节末尾，别的会话同时在改 `crates/`，那份 stat 分不出谁改的。

## 三、改法与依据

- 条款：D13（验证路线） 已定项 7（`.claude/kb/decisions/13-验证路线.md:124` 小节标题「已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像」，第 15 行表里写了需要第二个输入的核对归记录核对器）；D23（journal 的角色与格式） 已定项 15（`.claude/kb/decisions/23-journal的角色与格式.md:433`「已定项 15：崩在记录持久之后、根槽持久之前，恢复由记录重建那次发布的根」）。
- 整条流比前缀多出来的，只有根槽写落在崩溃点之后的那几次发布。按 `crash.rs` 今天的判据（`check_records_against`，我读的那一版是开工后的工作区）逐条过了一遍：
  - 「根在而记录一条都不在」要求根槽写在盘上。崩溃点之后的根槽写都没持久，根记录又带着 txg，不会与盘上的旧字节逐字相同，所以判不出新红。
  - 「恢复自称新态而单元缺席」只罩 txg ≤ 恢复落到那一版的发布。txg 全程单调：新实例的第一次发布取环里与记录里的最大 txg 加 1（`crates/singlefs-core/src/mount.rs:740-759`），回退取现行版加 1（`mount.rs:3990`）。所以崩溃点之后发起的发布 txg 都更大，进不了这一条；能进来的只有「记录已持久、根槽没持久、恢复就落在它身上」的那一次在飞发布。它的单元写在它的记录之前的段里（整段持久），按条款本来就该在。前缀交法把这一次整个漏掉了，整条流把它补上，是收严。
  - 缺席判定里，只有「更晚、在持久集合里、过了回收谓词」的写才算解释。崩溃点之后的写全不持久，所以解释集合与前缀交法相同。
- 第三截（二次崩溃，`crash_injection.rs:1422-1461`）是同一种前缀交法，我没改，理由与分析在第六节第 2 条。

## 四、新用例与证红

证红一律在仓副本 `/tmp/claude-1000/impl-rev-b3c1/copy` 上跑（取于 2026-09-27，`rsync -a --exclude target --exclude .git`，另删了 `research/target`；那一刻主工作区 `crash.rs` 的 sha256 是 `f689915b…`，`walk.rs` 是 `9b636aa8…`，都已进副本）。用的是 `bash research/scripts/capped.sh 4 bash research/scripts/prove-red.sh --copy <副本> singlefs-harness <名…>`，变异行先追加进副本的 `crates/mutations.tsv`。副本用它自己的 target。

**基线红集**（副本上不改动、整个二进制跑一遍，debug，`capped.sh 4`，`logs/copy-baseline-whole-binary.log`）原样：

```
test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 186.95s
```

红的只有 `crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed`，新发现全是种子 …115 第 29 段的 I-7.4（五截各一个签名，各 2 次），原样：

```
      2 崩溃状态上的新发现 CheckerViolations { invariants: ["I-7.4"] }（after_the_writable_mount_and_one_publish）：种子 7463871032432355115，第 29 段
      2 崩溃状态上的新发现 CheckerViolations { invariants: ["I-7.4"] }（crash_image）：种子 7463871032432355115，第 29 段
      2 崩溃状态上的新发现 CheckerViolations { invariants: ["I-7.4"] }（second_crash_inside_acquisition）：种子 7463871032432355115，第 29 段
      2 崩溃状态上的新发现 CheckerViolations { invariants: ["I-7.4"] }（second_crash_inside_row_publish）：种子 7463871032432355115，第 29 段
      2 崩溃状态上的新发现 CheckerViolations { invariants: ["I-7.4"] }（second_crash_inside_warm_up）：种子 7463871032432355115，第 29 段
```

基线红集之外，这次加的另两条都绿，读数原样：

```
── 崩溃注入：恢复落到由记录重建的 (InstanceGeneration(5), CheckpointTxg(15))（根槽写在写表第 357 项，崩溃点所在段 [355, 357)），它的实例表抛弃 (1, 4) ──
test crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons ... ok
── 崩溃注入：落到 (InstanceGeneration(5), CheckpointTxg(15))，扣下 (5, 14) 的单元：9 个偏移里干净的扣法 8 个，都判红 ──
test on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red ... ok
```

| 用例（测试文件行号） | 钉什么 | 变异（改坏哪一行） | 红在哪条断言（原样） | 同时红了谁 |
|---|---|---|---|---|
| A `crash_state_landing_on_a_version_rebuilt_from_records_exempts_the_publishes_its_instance_table_abandons`（`:900`） | 种子基 + 2、24 步、每段 4 个崩溃状态（与快档同比重）：先按这段历史上摆出来的崩溃状态逐个重建、恢复，核这一形真摆到了（恢复施加过记录、落到那一版的根槽写不在前缀里、它的实例表读得出且抛弃一次单元已被盖掉的发布）；再钉崩溃注入的计数「单元缺席 0 次、根在而记录不在 0 次、每个崩溃状态都跑过记录核对器」 | M1：`crash_injection.rs` 第一截 `check_records_against(&image, &writes, &persisted, report.effective_root);` 改回前缀 `&writes[..first_write_of_the_segment + writes_in_the_segment]` / `&persisted[..同上]` | `:925` 那条 `assert_eq!`：「被落到的那一版的实例表抛弃的 (1, 4) 的单元不算「该在」：记录核对器要在交给它的写表里找得到那一版的根槽写（写表第 357 项）」，`left: 1`、`right: 0`（`prove-red-logs-1/001.log:691-694`） | prove-red 按行里的参数只跑了点名那一条（`-- crash_state_landing_on_a_version_rebuilt_from_records`），整个二进制下同时红谁没看（第六节第 1 条：整个二进制的基线是红的，prove-red 在基线红时整次不跑） |
| B `on_the_whole_stream_a_kept_publish_missing_a_unit_under_a_version_rebuilt_from_records_is_still_red`（`:941`） | 同一个崩溃状态、整条流：不另扣时一条不判（对照）；把 (5, 14) 写在某个偏移上的单元两份都扣下，只算「干净」的扣法（扣下之后恢复仍落 (5, 15)、它的实例表仍读得出且仍抛弃 (1, 4)），每个干净的扣法都判「单元缺席」，干净的扣法至少一个（实测 9 个偏移里 8 个） | M2：`crash.rs:816` `.is_some_and(|table| table.abandons(publish.instance, publish.checkpoint_txg))` 改成 `.is_some()`（实例表读得出时每次发布都当被抛弃） | `:1010` 那条 `assert!`：「(5, 14) 没被落到的那一版 (InstanceGeneration(5), CheckpointTxg(15)) 抛弃，它写在偏移 827359232 上的单元两份（写表第 [318, 319] 项）都扣下，照判红」（`prove-red-logs-1/002.log:631-632`） | 同上，只跑了点名那一条 |
| 小快档 `crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed`（`:163`） | 快档的头 4 段（含种子基 + 2），清单外的失败一条都不许有；每个崩溃状态三截都真跑了（问模型、池级 checker、记录核对器、可写挂载、挂载之后的 checker 次数都等于崩溃状态数；二次崩溃摆出来过，每个都问过模型、跑过 checker 与记录核对器）；恢复失败 0 次 | M3 与 M1 同一处改法，必须红的是小快档 | **没证**：它在基线红集里（I-7.4），照定义停下 | — |

prove-red 两次的原样末行：

```
✓ 点名 2 条：跑了 2 条，跳过 0 条，跑的都抓到了
```
（第一次：M1、M2，`logs/prove-red-1.log`；两组参数的基线各自先跑、都绿，`prove-red-logs-1/baseline.log` 里留的是后一组的：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 8.84s`）

```
✓ 点名 1 条：跑了 1 条，跳过 0 条，跑的都抓到了
```
（第二次：第 155 行的替换行，见第五节。基线 `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 13.85s`；变异下红在 `crates/singlefs-harness/src/crash_injection.rs:2060:9`，消息「每个崩溃状态都跑过记录核对器」，`prove-red-logs-2/001.log`）

三次红的都是测试自己的断言（消息原样见上），不是被测代码里的 `debug_assert` 先红，所以没有在 `--release` 下再跑。M1 下小快档与快档也会红，这是推的（种子 …115 在两档里都有，调查报告第 1 条在前缀交法下量到过），这一次没有跑。

## 五、快档标 ignore、小快档、`crash-case:` 登记行、变异行

- 快档 `crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`（`:132`）上面加了 `:131` 那行 `#[ignore = "快档：每个崩溃状态上起可写挂载、发一次布、摆三个二次崩溃之后 debug 下单跑一百来秒；交提交时的崩溃验证员经门禁 54 号按 crash-case:crash-injection-fast-tier 在 release 下跑，普通 cargo test 由 crash_injection_small_fast_tier_recovers_only_into_versions_the_model_committed 守"]`。快档名字没改，所以 B3b 报告第四节末尾那一行照用，只改了注释。它只调一次 `run_crash_injection_campaign`，恰好打一行 `CRASH_INJECTION_FINISHED`（`crash_injection.rs:1908`，在 `run_crash_injection_campaign` 里）。
- `.claude/gate.d/stage-inputs.tsv` 要登记的行原文（四列制表符分隔）：

```
crash-case:crash-injection-fast-tier	crates/ Cargo.toml Cargo.lock	test=singlefs-harness:second_transaction_supplement_three_crash_injection:crash_injection_fast_tier_recovers_only_into_versions_the_model_committed count-line=CRASH_INJECTION_FINISHED	# 随机崩溃注入快档（种子基起 24 段、每段 24 步、每段 4 个崩溃状态）：每个崩溃状态上只读恢复与判定、再起可写挂载发一次布跑 checker、挂载途中取号/写行/暖机各一个二次崩溃（代码审阅第 2 条）；debug 下单跑一百来秒，标了 ignore，release 跑；普通 cargo test 由同文件的小快档（头 4 段）守。计数行 CRASH_INJECTION_FINISHED 只调一次、恰好一行；不打 LAYER0_PARALLEL_FINISHED，没登记 threads=；抽样，没登记 exhaustive=
```

- 小快档跑一次的用时（debug，`capped.sh 4`，4 段 4 个工作线程，报告里的「用时」）：主工作区上 11.4 秒，副本上 11.6 秒。这比派发要的「几秒内」长。按段数缩短不了：4 段已经各占一个线程，墙钟由最慢的一段历史定（24 步、4 个崩溃状态、每个都起挂载加三个二次崩溃）。要再短，得缩每段步数或每段崩溃状态数，那样就不再是「快档的头几段」。我没缩，交主 agent 定。
- 变异行（都在草稿目录，六段，格式同 `crates/mutations.tsv`，名字表里现有的都没有）：
  - `mutations-append.tsv` 三行：M1「实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（恢复落到由记录重建的一版时找不到它的根槽写、读不出它的实例表，被它抛弃的发布的豁免落空）」，**证过**；M2「实审 B3c-1：记录核对器在实例表读得出时把每次发布都当被抛弃（整条流让由记录重建的那一版的实例表读得出之后，没被抛弃的前一版缺单元不再判红）」，**证过**；M3「实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（小快档：头 4 段里种子基 + 2 那一段判红）」，**没证**（小快档在基线红集里）。三行都用 `-p singlefs-harness --test second_transaction_supplement_three_crash_injection -- <过滤>`，跟表里这个二进制已有的行一样按名字过滤。M2 的文件是 `crash.rs`，原文在那份副本里恰好命中 1 次（`crash.rs:816`）。B3a-2 还在改这份文件，追加之前请再核一次命中数。
  - `mutations-replacements.tsv` 两行（按第一段的名字整行替换）：
    - 「增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器」（现第 155 行）：旧原文是第一截那段前缀调用，被这次改动改没了（门禁 33 号原样：`crates/mutations.tsv:155 增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器：原文在 crates/singlefs-harness/src/crash_injection.rs 里命中 0 次`）。新原文是 `        let record_check =\n            check_records_against(&image, &writes, &persisted, report.effective_root);\n        tally.record_checks += 1;`，在主工作区命中 1 次；替换文、参数、必须红的测试都不变。**证过**（第四节第二次 prove-red）。
    - 「增补 3 第 3 件：判崩溃点时取择根而不是施加记录前缀之后实际走的那条根（记录已写、根槽未写那一格内容与根身份配不上）」（现第 152 行）：它的参数 `-- crash_injection_fast_tier`、必须红的是快档。快档标 ignore 之后，这样跑一条都跑不到，59 号会判它没红。替换行把参数改成 `-- crash_injection_small_fast_tier`，必须红的改成小快档，原文与替换文不变。**没证**：理由同 M3。另一条路是保留点快档、参数加 `--include-ignored`，那样 59 号每复跑一次这一行都要跑一遍快档，交主 agent 挑。

## 六、交主 agent 的问题与发现

1. **小快档在当前工作区上红，红因不在我这两份文件里**。红的是种子 7463871032432355115 第 29 段那个崩溃状态上的 I-7.4，五截都判（第四节那张原样表）。第一截判它的是 `checker_violations_on`，它只跑池级 checker、不读记录核对器的结论；所以它与这次改的交法无关，是由 `check_pool_image` 的结构推出来的。这条消息的原样（主工作区那一趟，`/tmp/claude-1000/impl-rev-b3c1/run-new-tests-1.log`）：`I-7.4：被抛弃的根（实例 1、txg 4）引用的单元已被重新分配或抹头（校验和对不上或头用不了）：数据单元 在盘 0 槽 50182 的那一份与位置条目里的校验和对不上；数据单元两份都读不到对得上的；映射条目指的单元两份都读不到对得上的`。调查员的快照上快档没有这一条（调查报告第一节的三条里没有它），同一个种子、同样摆 4 个崩溃状态，快档本来就跑得到这一格。那之后 `crates/singlefs-checker/src/walk.rs`、`image.rs`、`crates/singlefs-core/src/` 下几份、`history.rs` 都被别的会话改过；是哪一处带进来的，我没查。
   - 这一条同样会让标了 ignore 的快档红（种子 …115 在快档的 24 段里），崩溃验证员在提交时会看到它。
   - 我**没有**为了让小快档绿去挑别的种子：头 4 段是照「快档的头几段」定的；换成现在碰巧干净的种子，等于把这一格从普通 `cargo test` 里藏起来。要换就请主 agent 定。
   - 小快档绿了之后，M3 与第 152 行的替换行才证得了红（`prove-red.sh` 在基线红时停）。
2. **第三截（二次崩溃）还是前缀交法**（`crash_injection.rs:1422-1423` 取 `&mount_writes[..first_write_of_the_segment + writes_in_the_segment]`，`:1456-1461` 交给 `check_records_against`）。派发点的是第一截那一处，我没改它。我的分析（推的，没造状态验）：
   - 它的写表只有挂载自己那一段，核的只是挂载写出的发布（写行、暖机），都属于这次挂载新开的实例。按调查员读到的 (5, 15) 那张表（行只有实例 1–4），一版的实例表不带自己实例的行。所以这里实例表读不出，也关不掉哪次挂载发布的豁免，**不会像第一截那样假红**。
   - 缺口在另一边：二次崩溃落在写行那次发布的记录已持久、根槽没持久时，恢复落到由记录重建的写行那一版，前缀里没有它的根槽写，那次发布整个不在 `publishes_in` 里，它的单元在不在没人核。第一截在改之前也有同一个缺口，改成整条流之后补上了。
   - 要补就把第三截换成 `&mount_writes` 加上逐条对应的持久集合（后面全记没持久），两行的事。这算不算这一件活，请主 agent 定。
3. **`crash.rs` 里有两处文档注释与这次的交法对不上**（B3a-2 的文件，我没改）：
   - `crash.rs:767`「那份镜像（`reader`），而要核的记录流是到当前段为止的整条前缀（`writes`，`persisted` 与它逐条对应）——两者的写表不是同一张，」：第一截现在交的是整条流；第三截还是挂载那一段的前缀。
   - `crash.rs:848` 那句前提「根槽没落盘、由记录重建的那一版也在记录流里」：第一截改完之后成立，第三截不一定（见第 2 条）。
4. **门禁 33 号**：表里第 155 行是我这次改动弄坏的，替换行见第五节。同一趟里别的红（`walk.rs` 的第 308–310、319、494、759、865、870、874、877、878 行命中 0 次，`e158_root_choice_repair.rs` 的第 837–839、844、846、851 行命中 2 次）点名的文件都不在我这两份里，我没碰。
5. **没有停在「条款没写」的分支上**：这次只改装置怎么交入参，没有加新的错误成员、`todo!` 或 `assert!`。
6. **受影响的层 0 流与崩溃枚举用例**：checker 没动，这一节不适用。崩溃注入快档的输入指纹会因为这两份文件变化而变（`crash-case:crash-injection-fast-tier` 登记上之后）。

## 七、验证（第 4 步那几样，末尾原样）

开跑前 `ps` 看到的：别的会话的 `cargo test`（`second_transaction_step_three_formatted_pool` 一组、`second_transaction_supplement_three_fault_injection`、`first_transaction_step_five…`），没有 qemu / vm-bench / e152 / fio。cargo 自己排队，我没记等锁的时长。都跑在 `nice -n 19`、`capped.sh 4` 下；跑编出来的测试时经 `run-with-memory-cap.sh 8G`，没有撞到包装的 250–254。

- 主工作区 `cargo test -p singlefs-harness --lib`（`logs/main-lib.log`）：
  `test result: ok. 89 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 127.13s`
- 主工作区整个 `--test second_transaction_supplement_three_crash_injection`（`logs/main-crash-injection-binary.log`），红因与副本基线相同，都是小快档上种子 …115 第 29 段的 I-7.4，五截各 2 次：
  `test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 223.15s`
  这一趟里小快档报告的两行记录核对器读数原样（第 106、207 行）：`崩溃状态上 checker 跑了 16 次；记录核对器跑了 16 次：根在而记录一条都不在 0 次、恢复自称新态而单元缺席 0 次`；小快档「用时 12.0 秒」。
- `cargo fmt --all -- --check`（`logs/fmt-check.log`）：退出 1。`Diff in` 点名的四份文件是 `e158_root_choice_repair.rs`、`admission_checkpoint_cost_per_device_paths.rs`、`entries_after_a_writable_mount_refuse_other_parameters_and_device_tables_before_any_write.rs`、`writable_mount_refuses_a_device_table_disagreeing_with_the_system_configuration.rs`，都不是我的。我这两份单独 `rustfmt --check --edition 2021` 退出 0。
- `cargo clippy --offline --all-targets --all-features -- -D warnings` 加 check.sh 那七条（`logs/clippy.log`）：退出 101，末尾原样：
  `error: could not compile `singlefs-checker` (lib test) due to 4 previous errors`
  报错指的文件是 `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`、`crates/singlefs-harness/tests/admission_checkpoint_cost_per_device_paths.rs`，都不是我的。checker 编不过，全仓 clippy 在它那里就停了，所以我另跑了一趟只查我动到的目标（同一套 lint，`-p singlefs-harness --no-deps --lib --test second_transaction_supplement_three_crash_injection`，`logs/clippy-harness-no-deps.log`），退出 0：
  `Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.89s`
- `cargo build --offline --all-targets`（`logs/build.log`）：退出 101，末尾原样：
  `error: could not compile `singlefs-harness` (bin "e158_root_choice_repair") due to 9 previous errors`
  报的都是 `E0609`（`ThirdRunFamilySummary` 没有 `f13_checked` 这一类字段），在别人的 bin 里。我动到的测试二进制与 `--lib` 都编过、跑过（见上两条）。
- 登记给我的门禁阶段（`stage-owners.tsv` 里 implementation-writer 那几个），末行与退出码原样：
  - 33 号：退出 1，`✗` 1 处，末行 `    → crates 这张表：把原文改到今天源码里逐字存在、只出现一次的那一段（原文里的换行写成 \n），改完单跑那几行证明点名的测试红。`。我这次造成的是第 155 行（替换行见第五节）；其余点名的是 `walk.rs`、`e158` 的行，不是我的。
  - 53 号：退出 0，`✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：…）`
  - 74 号：退出 1，`test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 52.32s`。红的两条是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，在随机历史那个二进制里。那个二进制不经过崩溃注入的第一截，所以不是我这处改动带进来的（推的，没在改前的副本上对照）。
  - 92 号：退出 0，`✓ 布局清单 1 套布局、6 条路径都在，位号都对得上 …`
  - 93 号：退出 0，`✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，…）`
  - 94 号：退出 0，`✓ checker 与实现只共享常量模块 `singlefs-format`（…）`
  - 89 号：退出 77，`⊘ 本次未跑：收口表第 27 行那几笔的前置一个都没进来，今天无对象可判（5 条逐字探针、覆盖 4 笔，逐条对上今天的值）`，这是没跑，不是通过。
- `research/scripts/crash-case-check.py`：退出 1，点名的是 `crash_enumeration_resumes_from_its_progress_file.rs:383` 等，不是这份测试文件（快档不调 `enumerate_layer0` 一族，它不判）。
- `git diff --stat -- crates litmus`（原样末行，全份在 `logs/git-diff-stat.txt`）：`87 files changed, 28974 insertions(+), 11040 deletions(-)`；我这两份的那两行：`crates/singlefs-harness/src/crash_injection.rs     |  821 ++-`、`...transaction_supplement_three_crash_injection.rs |  658 +-`（含开工前别人已有的改动）。

## 八、草稿目录里删了什么、留了什么

- 删了：仓副本 `/tmp/claude-1000/impl-rev-b3c1/copy`，删前 `du -sh` 是 2.2G（含它自己的 target）；空的 `/tmp/claude-1000/impl-rev-b3c1/tmp`（4.0K）。
- 留着的，主 agent 要核或要打：`report.md`、`mutations-append.tsv`、`mutations-replacements.tsv`、`logs/`、`prove-red-logs-1/`、`prove-red-logs-2/`、`run-new-tests-1.log`、`my-crash_injection.diff`、`my-test.diff`、开工快照 `crash_injection.rs.at-start`、`test.rs.at-start`、`before-my-edits.diff`、`sha-at-start.txt`、`build-test-1.log`、`rustfmt-check-1.txt`。都不是编译目录，也不是仓副本。

## 九、没做什么

- 没走三方对抗，没提交；层 0、QEMU、herd7、crates 变异整表归崩溃验证员。快档（已标 ignore）与大档没跑，没带 `--include-ignored`。
- M3 与第 152 行的替换行没证红：小快档在基线红集里。I-7.4 那一格的来源没查，第三截没改（第六节第 1、2 条）。
- `crates/mutations.tsv`、`stage-inputs.tsv`、kb、`crash.rs` 都没碰；要改的在第五、六节。
- 全仓 clippy / build / fmt 因为别人的文件没过，我只单独查了自己动到的目标。
