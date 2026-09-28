# K3-⑨ 机理调查报告（2026-09-28）

对象：判决 `research/prompts/m2-supp3-item3-code-r2-main-verification.md` 第 32 行（K3-⑨）与第 53 行（第四节第 2 题）。
代码：工作区（HEAD `e5253e8a` 加已暂存改动；`git diff --stat -- crates` 为空，说明 crates/ 下没有未暂存的改动）。
副本 `rsync -a --exclude target --exclude .git` 到 `/tmp/claude-1000/k3-9-mechanism/repo`，几份关键文件与主工作区的 sha256 逐一相同：
crash.rs `ddf64360…`、crash_injection.rs `bbfe7d37…`、transaction.rs `af33d518…`、recovery.rs `2051deb8…`。

## 结论

1. **真假**：在今天的代码上拿掉那道屏障（`crates/singlefs-core/src/transaction.rs:1058` 的 `writer.perform_commit_step(CommitStep::Barrier)?;`），
   第一截的三条 oracle（checker、理想模型、记录核对器）和第二截（崩溃镜像上可写挂载加发一次布之后的 checker、记录核对器、拒没拒）
   在我摆出的 1433 个崩溃状态上**一条都没红**。这些状态是在 harness 档里照同一套做法摆的，不是跑 checker 档那条测试得来的。
   第三截（挂载途中二次崩溃）没复刻，checker 档测试本身也没跑，见「没做什么」。
2. **机理**：攻方候选 ① **成立**，候选 ② **不成立**。另外还有一条攻方没提的：新实例的第一次发布，它的记录不会被施加。
   - 拿掉 1058 之后，单元写与 journal 记录并进同一段；记录与根之间那道屏障（`transaction.rs:1065`）还在，所以根槽写总是在更晚的一段里。
     枚举域是「更早的段整段持久、当前段取真子集、更晚的段一个都不持久」（`crash_injection.rs:803-809`）。
     因此在这些状态里，只要某次发布有单元缺席，它的根就一定没落盘。恢复要落到那次发布的 txg，只剩施加它的 journal 记录这一条路。
   - 施加记录之前，恢复会读出每一条记录点名的每个单元的每一份副本，逐份核 CRC（`recovery.rs:3161-3179`）。
     有一份不对就 `verification_failed += 1` 并 `break`，这次发布整体不施加。
     崩溃注入调恢复用的是 `JournalPolicy::Consult`（`crash_injection.rs:846`），它传给 `verify_named_units` 的是 `true`（`recovery.rs:4246`）。
     点名的是这次发布重写的全部角色（`transaction.rs:5338` 起的 `roles_named_by_each_record_of_the_publish`，每个角色都在某一条记录里）。
   - 于是 `report.effective_root` 停在上一版，`crash.rs:145` 的前置 `txg >= publish.checkpoint_txg` 不成立，判不到 `crash.rs:159`。
     理想模型那边，底线是「已持久的最新根槽」（`model.rs:2262-2276`），光有记录落盘不抬这条底线。恢复落到上一版，模型认那一版，所以不红。
     checker 看到的盘面是上一版加上一些没被引用的单元写，也不红。
   - 攻方没提的那一条：一个新实例的第一次发布（这段历史里的 (2,5)）的记录，恢复根本不当候选。
     `recovery.rs:3064-3069` 只收与所选根同一实例的记录，而此时所选根还是实例 1 的 (1,4)。所以即使关掉点名验证，它也抬不上去。
   - 候选 ②（`written_over_later` 把副本豁免掉）：这个函数今天已经没有了，换成了按扇区、看持久集合判缺席（`crash.rs:191-244`）。
     关掉点名验证那一臂上，A 类状态（记录全持久、某个单元两份都扣下）每一个都判红（见 run3），说明豁免没有吞掉它。
3. **最小复现**：能用 harness 档造出来，不用跑 checker 档。命令与输出见「最小复现」。

## 推翻条件，以及造它的那一次

| 什么观测会推翻这个定位 | 造它的那一次（run3，副本上加变异 C，`PROBE_POLICY=without` 把 `verify_named_units` 关掉） |
|---|---|
| 关掉点名验证后，A 类状态里恢复仍到不了那次发布的 txg，或 `claimed_state_missing_unit` 仍是 0 | 没出现：五次同实例发布的 A 类合计 283 个状态，`effective_root_reaches_the_publish` 283、`claimed_state_missing_unit` 283、`model_red` 283（逐行在 run3 日志里） |
| 带验证那一臂上，有 A 类状态恢复到了那次发布的 txg | 没出现：run2 / run5 里 A 类 337 个状态，`effective_root_reaches_the_publish` 全是 0；其中同实例的 283 个 `verification_failed` 都是 1 |
| 不带变异时，关掉验证也红（那就说明红不来自拿掉屏障） | 没出现：run4（副本复原，`PROBE_POLICY=without`）62 个状态全绿，也没有「同一次发布的单元与记录并在一段」的段 |

「这个判别子确实会红」由 run3 自己证明：同一套探针、同一批状态，只关掉验证，记录核对器就判红 752 次，模型也红 752 次。

## 复现与探针

原样复现攻方那条命令（旧的 `second_transaction_supplement_three_crash_injection` 在 singlefs-harness 里）今天做不到：崩溃注入已经搬进 checker 档
（`crates/singlefs-checker-tier/src/crash_injection.rs`，用例在 `crates/singlefs-checker-tier/tests/crash_injection_campaign.rs`），跑它属于重型测试。
写死的那段历史今天也变了样：单元写每段 16–24 个（攻方那时是 10 个单元写加 2 条记录，一段 12 个写）。所以攻方的「12347 个状态」「86」这两个数今天复现不出来，也不可比。

探针只在副本里：`crates/singlefs-harness/tests/zz_k3_9_probe.rs`，另存了一份在 `/tmp/claude-1000/k3-9-mechanism/probe/`。
- 历史与 `crash_injection_campaign.rs:379-393` 逐项相同（CloseAndMountWritable、第一个文件、覆盖写、零单元发布、CloseAndMountWritable、覆盖写），
  执行参数与 `UNCHECKED_ON_FOUR_GIBIBYTE_DEVICES` 相同。
- 第一截照 `crash_injection.rs:796-887` 的做法：基线叠更早的段，`CrashImage`，`recover`，理想模型 `crash_recovery_disagreement`，
  `check_pool_image`，记录核对器。
- 记录核对器是 `crash.rs:104-333` 的原样拷贝，第二截是 `crash_injection.rs:1049-1331` 的原样拷贝，都用 `include!` 引进来。
  `diff <(sed -n '104,333p' …crash.rs) record_checker_copy.rs`、`diff <(sed -n '1049,1331p' …crash_injection.rs) writable_mount_copy.rs` 都没有输出（`copies-identical`）。
- 摆哪些状态：写数 ≤ 12 的候选段整段全枚举。更长的段只在「同一次发布的单元写与记录写并在这一段」（变异 C 才有）时摆，摆法是定点加随机：
  每个单元两份都扣下、其余全持久；每一份单独扣下；这次发布的单元全扣下；再加 200 个随机掩码，去重。

变异 C（`probe/mutation-C.py`）在副本上的 diff：
```
1058d1057
<     writer.perform_commit_step(CommitStep::Barrier)?;
```

### 四次运行（命令都在副本根目录跑，经 `run-with-memory-cap.sh 12G` 与 `capped.sh 16`，`--release`）

命令形态（run2 的原样；另外几次只改 `PROBE_*` 与变异开关）：
```
PROBE_POLICY=consult PROBE_SEGMENT_WRITES=12 PROBE_RANDOM=200 nice -n 19 bash /home/fy5090/code/singlefs/research/scripts/run-with-memory-cap.sh 12G bash /home/fy5090/code/singlefs/research/scripts/capped.sh 16 cargo test --release --offline -p singlefs-harness --test zz_k3_9_probe -- --nocapture
```

| 次 | 代码 | 恢复策略 | 第二截 | TOTAL 行（原样） | 耗时 |
|---|---|---|---|---|---|
| run1 | 干净 | Consult | 开（没有并段，一次没跑） | `PROBE TOTAL crash states 62 ; root_without_record 0 ; claimed_state_missing_unit 0 ; checker red 0 ; model red 0 ; recoveries with verification_failed>0 0 ; with prefix_applied>0 18` | 1.20s |
| run2 | 变异 C | Consult | 开 | `PROBE TOTAL crash states 1433 ; root_without_record 0 ; claimed_state_missing_unit 0 ; checker red 0 ; model red 0 ; recoveries with verification_failed>0 903 ; with prefix_applied>0 8` | 571.44s |
| run3 | 变异 C | ConsultWithoutNamedVerification | 关 | `PROBE TOTAL crash states 1433 ; root_without_record 0 ; claimed_state_missing_unit 752 ; checker red 0 ; model red 752 ; recoveries with verification_failed>0 0 ; with prefix_applied>0 911` | 12.30s |
| run4 | 干净（复原后 sha256 `af33d518…`） | ConsultWithoutNamedVerification | 关 | `PROBE TOTAL crash states 62 ; root_without_record 0 ; claimed_state_missing_unit 0 ; checker red 0 ; model red 0 ; recoveries with verification_failed>0 0 ; with prefix_applied>0 18` | 1.08s |
| run5 | 变异 C | Consult | 关 | `PROBE TOTAL crash states 1433 ; root_without_record 0 ; claimed_state_missing_unit 0 ; checker red 0 ; model red 0 ; recoveries with verification_failed>0 903 ; with prefix_applied>0 8` | 12.06s |

每次改了 transaction.rs 之后都 touch，编译日志里有 `Compiling singlefs-core`（run2、run4、run5 都看过），跑的不是旧二进制。

变异 C 下的段形（run2 原样）：`PROBE publish (1, 3): unit writes 24 (distinct device+offset 24), record writes 2, unit segments {12}, record segments {12}, root segment 13`。
其余五次带单元的发布也一样，单元与记录在同一段、根在下一段。干净代码下是 `unit segments {12}, record segments {13}, root segment 14`（run1）。

run2 里每一类状态的读数（`stage2_*` 各项全是 0，省去；逐行原样在 run2 日志里）：

| 发布 | A：记录全持久、某单元两份都扣 | B：记录全持久、只扣了某份 | D：有记录扣下 |
|---|---|---|---|
| (1,3) | 58 个；到达那次 txg 0，verification_failed 58 | 24；0；24 | 155；0；106 |
| (1,4) | 59；0；59 | 27；0；27 | 151；0；96 |
| (2,5) 新实例第一次发布 | 54；0；**0** | 21；0；0 | 153；0；0 |
| (2,6) | 54；0；54 | 20；0；20 | 151；0；109 |
| (2,7) | 48；0；48 | 25；0；25 | 152；0；95 |
| (2,8) | 64；0；64 | 24；0；24 | 149；0；94 |

A 类的一个状态（run2 原样，第一个）：
`effective_root Some((InstanceGeneration(1), CheckpointTxg(2))) journal JournalScanReport { valid_records: 3, above_water: 1, prefix_applied: 0, verification_passed: 0, verification_failed: 1, maximum_applied_transaction: 0 } record_check RecordCheck { root_without_record: false, claimed_state_missing_unit: false } checker_red [] model None`
同一个掩码在 run3 里：`effective_root Some((InstanceGeneration(1), CheckpointTxg(3)))`、`claimed_state_missing_unit: true`、模型 `走读失败：UnitUnreadable { slot: SlotNumber(50252) }`。

## 最小复现（harness 档）

在仓副本里放进 `probe/zz_k3_9_probe.rs` 与 `probe/zz_k3_9/`（放到 `crates/singlefs-harness/tests/` 下），跑 `python3 probe/mutation-C.py` 并 touch transaction.rs，然后跑两次：
```
PROBE_POLICY=consult PROBE_STAGE2=0 … cargo test --release --offline -p singlefs-harness --test zz_k3_9_probe -- --nocapture
PROBE_POLICY=without PROBE_STAGE2=0 … cargo test --release --offline -p singlefs-harness --test zz_k3_9_probe -- --nocapture
```
第一次（run5）：`claimed_state_missing_unit 0 ; checker red 0 ; model red 0 ; recoveries with verification_failed>0 903`，跑了 12.06s。
第二次（run3）：`claimed_state_missing_unit 752 ; … model red 752`，跑了 12.30s。
两次只差 `verify_named_units` 这一个开关。这说明拿掉屏障造出的状态，被恢复端的点名验证整批挡在了「回到上一版」这个结局上。

用 checker 档原封不动地回答「今天那条测试红不红」要跑的命令（**没跑**，属于重型测试，交主 agent 去问用户）：
在打了变异 C 的仓副本里跑
`SINGLEFS_HEAVY_TESTS=user-request bash research/scripts/run-with-memory-cap.sh 12G bash research/scripts/capped.sh 10 cargo test --release -p singlefs-checker-tier --test crash_injection_campaign -- --include-ignored every_crash_state_of_a_written_out_history_recovers_into_a_committed_version crash_injection_fast_tier_recovers_only_into_versions_the_model_committed`。
内存与时长都是估计、没量过：内存估计在几个 GiB 以内（每个状态要克隆一次 4 GiB 稀疏盘，只存写过的扇区）。
时长按文件头那句「快档 debug 下单跑一百来秒」、release 更快，估计几分钟。
它能多答的是第三截（二次崩溃）和 `crash_points >= 95`（`crash_injection_campaign.rs:415`）这条规模断言在变异 C 下的值，这两样探针都没复刻。

## 排除掉的解释

| 解释 | 凭哪条观测排除 |
|---|---|
| 抽样抽不到（状态没摆出来） | run2 定点摆了「某单元两份都扣下、记录全持久」这一类 337 个状态，逐单元都摆到了，照样 0 |
| 候选 ②：更晚的写把缺席的副本豁免了 | run3 在同一批 A 类状态上判红 283 个（每一个都红）；豁免这一段代码（`crash.rs:220-238`）两次运行都在，没吞掉 |
| 记录核对器根本没被调用，或者判据坏了 | run3 里同一份拷贝判红 752 次；拷贝与 `crash.rs:104-333` diff 为空 |
| 点名单元不全（有单元没被点名，扣它不影响施加） | run2 里同实例的五次发布，A 类 283 个与 B 类 120 个状态 `verification_failed` 全是 1，没有一个单元扣了而验证照过 |
| 第二截的可写挂载会把记录施加上去，从而红 | run2 第二截跑了 1433 次（每个 A/B/D 状态都跑了），checker 红 0、记录核对器红 0、非单元区墙的拒 0；挂载读阶段同样带验证（`mount.rs:4207-4213` 传 `true`） |
| 红不来自变异 C，而是关掉验证本身就红 | run4 干净代码关验证，62 个状态全绿 |

## 没做什么

- 没修，也没判该怎么改。没改主工作区的任何文件。
- 没跑 checker 档（`singlefs-checker-tier`）的任何测试，也没跑层 0、54/55/57/59/87 号。
  所以没验：第三截（挂载途中二次崩溃）在变异 C 下红不红；`crash_points >= 95` 这条断言在变异 C 下是多少；快档随机历史上的情形。
- 探针第一截是照 `crash_injection.rs:796-887` 自己写的循环，不是原函数。没复刻已知红清单的分流、抽样那一路（`draw_crash_points` 的 salt 与 Sampled）、
  挂载阶段的拒绝理由细分。只有记录核对器和第二截是原样拷贝。
- 干净代码下写数 > 12 的纯单元段（6 段，16–24 个写）没摆状态。理由是推的：这些段的记录在下一段、根在再下一段，恢复不可能到那次发布的 txg。
- 没查变异 C 是否算一个「真缺陷」。这取决于有没有哪条契约要求「记录落了盘，这次发布就要留下」。今天的模型底线只看根槽（`model.rs:2262-2276`），这一格属于判改法，归主 agent。
- 线程上限在 run5 之后改成 10，之后没有再起编译或测试。

## 草稿目录

- 删了 `/tmp/claude-1000/k3-9-mechanism/repo`（仓副本连同它的 target，删前 `du -sh` 为 606M）。
- 留着：`probe/`（探针与变异脚本，复跑要用）、`run1`–`run5` 日志、`build-*.log`、`transaction.rs.clean`、`archive/`（从提交 `11a551b2^` 取出的攻方报告与模型，只作参考），都在 `/tmp/claude-1000/k3-9-mechanism/` 下。
  这些没有入库：派发只给了草稿目录与报告路径，要不要拷进 `research/` 由主 agent 定。
