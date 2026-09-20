# m2-supp3-item2 代码三方第一轮改法：实现员报告

时刻：开工 2026-09-19 11:16 UTC，会话在 11:56 之后中断、15:02 UTC 续上（东京时间 +9 小时）。判决 `research/prompts/m2-supp3-item2-code-r1-main-verification.md` 第三节第 1、2、4 条照做；第 3 条（74 号）归主 agent，没碰。

## 一、中断之后核过什么

- 续上时 `proof-progress.txt` 末行是 `11:56:48 全部跑完`，每份证明日志末行都是 `exit N`：中断之前变异证明已经全部跑完，没有要重跑的。后台任务的 pid 我没记（用的是 Bash 的后台任务、完成通知已到），所以按日志末行判的，没按 pid 看。
- 续上时 `sha256sum -c` 对证明开跑时记的源文件哈希全 OK（`logs/source-at-proof-start.sha256`），`crates/mutations.tsv` 的哈希也与开工快照一致：中断期间没有别的会话改 `crates/`。
- 续上之后新跑了：攻方的长历史设置（base 与 W1 两份副本）、门禁 33 / 53 / 74 号、最后一遍 `check.sh`。

## 二、改了什么（照判决的形态，不照攻方的临时改法）

### 1. M2：分配记录墙的区间收窄（判决第三节第 1 条）

- `crates/singlefs-checker/src/walk.rs` 第 1260 行起新增 `allocation_record_count_under_root(reader, &RootView) -> Option<usize>`：按 checker 自己的字段表读这条根的树表、找种类 3 的条目、数分配记录树根节点叶里的条目。树表里没有分配记录树（树表 0 条的一版）时是 0 条；树表 / 节点读不出、有内部节点时 None。
- `crates/singlefs-harness/src/history.rs` 第 672 行 `allocation_records_on_the_image_under`（按 checker 的 `valid_roots` 找 (txg, 实例) 那条根再数）、第 691 行 `allocation_records_counted_for_the_wall`：只在理由是分配记录墙时数，数的是**模型点名的那一版**，不读分配器。
- `crates/singlefs-harness/src/model.rs`：
  - 第 370 行 `ModelAnswer::root_whose_allocation_records_the_wall_counts`：逐次判的（发布、抬 F）数拒之前最后写出的那一版，一串判完的（可写挂载、回退）数这一步的起点——与实现的准入基数同一处（`publish_admission` / `publish_sequence_admission`）。
  - `ObservedOutcome::Refused` 加 `allocation_records_counted_on_the_image`（第 468 行）；每次计划的发布加「按准入口径新增几条」（重写的角色 × 盘数，收口表第 39 行用户定的上界准入）。
  - 第 1293 行 `capacity_wall_is_permitted`：分配记录墙放行要两头都过——上界那一头不动；真条数那一头 = 镜像上数的基数 + 到那一次为止的新增 > 812。数不出基数不放行。
  - 对不上时实现那一格的文字带「镜像上数的准入基数 N 条」。
- 为了让门禁走得到墙：`history.rs` 第 2435 行新增 `PerStepChecker { Run, Skipped }`、第 2476 行 `execute_history_with`；`run_history_campaign` / `shrink_*` 多一个参数，原有调用全传 `Run`（行为不变）。第 422 行新增比重 `TOWARD_THE_ALLOCATION_RECORD_WALL`。

### 2. M3：拒绝理由带判别字段（判决第三节第 2 条，连同 C368 仍欠的那一半）

- `crates/singlefs-core/src/mount.rs`：`RollbackTargetNotACandidate { target, reason: &'static str }` 改成 `{ target, exclusion: RollbackCandidateExclusion }`（第 48 行），新枚举在第 122 行，四个成员：`NotInRing`、`BelowEffectiveFloor`、`OnAbandonedTimeline`、`TargetVersionWithoutFileUnsupported`——照判决列的四条。原来单列的 `RollbackTargetNotInRing(RollbackTarget)` 与 `RollbackToVersionWithoutFileUnsupported(RollbackTarget)` 两个成员并进这个字段、删掉（第 1203、1218、1228、1236 行四处发出）。理由见第六节第 1 条。
- `crates/singlefs-core/src/transaction.rs`：`PublishError::NoSpaceFor { unit }` 换成 `PlacementRefused { unit, refusal: PlacementRefusal }`（第 930 行），发布路径改调 `try_allocate_*`、原因原样带上（第 1603 行）。`allocator.rs` 两个丢原因的 `allocate_*` 只改了文档注释（测试还在用）。
- 胶水 `crates/singlefs-harness/src/model_comparison.rs`：第 170 行 `refusal_reason_of_placement_refusal` 只把 `NoFreeSlotOnAnyDevice` 映射成单元区墙，另三种映射成 `Unexplained`；第 183 行 `refusal_reason_of_rollback_candidate_exclusion` 按字段一对一映射。`ObservedRefusalReason::Explained` 从 `Vec<理由>` 改成单个理由（第 449 行），「两条之一」那条路不再存在。
- 调用方的 `match` 跟着改，没写 `_ =>`：`history.rs` 的成员名字（新增 `placement_refusal_member`）、四个测试文件的断言、快档「各条路径都跑到了」那组成员名。

### 3. M4 那一格的设计事实（判决第三节第 4 条）

`crates/singlefs-harness/src/model.rs` 模块文档第 14–18 行：模型不记落点，落点与分配记录写对没有归池级 checker 判；分配记录的真条数模型也不记，墙拒时执行器从镜像上现数。

## 三、新测试与「改坏哪一行 → 哪条断言红」

证明的做法：`rsync -a --exclude target --exclude .git` 拷仓，每份副本自己的 target，debug，每次跑点名测试所在的**整个测试二进制**。基线副本（不改动）四个二进制全绿（`proof-base-1..4.log` 末行 `exit 0`），基线红集为空。红点全在测试断言行上，没有 `debug_assert` 先红的。脚本 `scripts/run-proofs.sh`，进度 `logs/proof-progress.txt`。

新测试（行号是函数所在行）：

| 测试 | 位置 |
|---|---|
| `allocation_record_wall_sampling_without_the_checker_refuses_only_above_one_node_by_the_true_count`（逼近墙的取样点，种子 [0, 32) × 150 步，不跑 checker） | `tests/second_transaction_supplement_three_random_history.rs:286` |
| `allocation_records_counted_on_the_image_are_one_per_unit_per_device_and_zero_without_a_file` | 同文件 :339 |
| `an_overwrite_that_fills_the_allocation_node_to_exactly_812_records_succeeds_and_the_next_is_refused`（写死的一段：四次挂载 + 45 次覆盖写，第 44 次正好 812 条） | 同文件 :389 |
| `rolling_back_onto_an_abandoned_root_above_the_floor_is_refused_for_being_abandoned`（R1 形态） | 同文件 :455 |
| `model::tests::the_allocation_record_wall_is_permitted_only_when_the_counted_records_plus_this_publish_exceed_812` | `src/model.rs:1998` |
| `model::tests::mount_and_raise_count_the_wall_from_the_version_their_admission_starts_from` | `src/model.rs:2040` |
| `model_comparison::tests::only_a_placement_refused_on_every_device_passes_as_the_unit_area_wall`（非容量的落点拒绝） | `src/model_comparison.rs:330` |
| `model_comparison::tests::each_rollback_candidate_exclusion_maps_to_its_own_reason` | `src/model_comparison.rs:376` |

改过断言的旧测试：`second_transaction_step_four_rollback.rs:89`、`:297`（改成按字段断言），`second_transaction_supplement_two_unequal_devices.rs:235 / 325 / 390`（改成断言分配器的原因），`second_transaction_step_one_overwrite.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs` 各一处（`NoFreeSlotOnAnyDevice`）。

| 变异（`crates/mutations.tsv` 行） | 改坏哪一行 | 红在哪（同二进制里同时红的） |
|---|---|---|
| 156 / 157 W1 | `transaction.rs` `if records_after_this_publish > allocation_node_capacity {` → `>=` | 历史二进制：`an_overwrite_…812…` 红在测试文件第 414 行（`run.ending == Completed`，实际「模型说该成、实现拒了」、基数 796）；取样点红在第 303 行（新发现 9 段：种子 5, 9, 10, 11, 13, 16, 27, 30, 31）。别的红：无 |
| 158 | `model.rs:1326` `upper_bound_exceeds && true_count_exceeds` → `&& (true_count_exceeds \|\| true)` | lib：`the_allocation_record_wall_is_permitted…` 红在 `model.rs:2026`（基数 796 应对不上）；同时红 `mount_and_raise_count…`（`model.rs:2068`） |
| 159 | `model.rs:374` `\|\| publishes_completed == 0` → `\|\| publishes_completed < usize::MAX` | lib：`mount_and_raise_count…` 红在 `model.rs:2050`（做完一次之后数第一次抬 F 那一版） |
| 160 | `walk.rs:1286` `records += node.entries.len();` → `/ 2` | 历史二进制：`allocation_records_counted_on_the_image…` 红在第 366 行；同时红 `an_overwrite_…812…`（414）与取样点（303，29 段「模型说该成、实现拒了」） |
| 161 R1 | `mount.rs:1230` `OnAbandonedTimeline` → `BelowEffectiveFloor` | 历史二进制：`rolling_back_onto_an_abandoned_root…` 红在第 478 行；同时红回退取样点（12 / 48 段）、快档（4 / 96 段）、逼近墙的取样点（5 / 32 段），签名都是「拒绝的理由」。步 4 二进制：`rolling_back_onto_an_abandoned_timeline_or_a_missing_root_is_refused` 红在第 328 行 |
| 162 | 胶水 `OnAbandonedTimeline => RollbackTargetOnAbandonedTimeline` → `…BelowEffectiveFloor` | 历史二进制同 161（四条，段数相同）；lib：`each_rollback_candidate_exclusion…` 红在 `model_comparison.rs:395` |
| 163 | `transaction.rs:1603` 起 `refusal,` → 一律 `NoFreeSlotOnAnyDevice` | 不等盘二进制三条全红：第 304、369、432 行（各自断言分配器的原因） |
| 164 | 胶水 `} => ObservedRefusalReason::Unexplained,` → `explained(UnitAreaWall)` | lib：`only_a_placement_refused…` 红在 `model_comparison.rs:365`（小盘写满应判对不上） |

判别力的数：W1 在门禁取样点 [0, 32) × 150 步上判出 **9 / 32 段**；写死的 812 条用例 1 / 1。攻方的长历史设置（副本上把大档临时改成不跑 checker，96 段 × 200 步，release，`scripts/run-long.sh`）：

| 副本 | broad | reuse |
|---|---|---|
| base（改法在） | 96 段跑完、新发现 0；墙按真条数放行 214 次 | 96 段跑完、新发现 0；放行 1204 次 |
| W1 | 新发现 12 段（种子 3 起，第 189 步覆盖写） | 新发现 33 段（种子 0 起，第 151 步抬 F） |

（攻方改法在它的模型上报 17 / 41 段；两边的数不是同一个判法：我数的是准入基数 + 新增，它读的是错误成员自带的 `records`。）门禁三段在改法之后与改之前逐数相同：快档 1912 步 572 / 34 / 1306，与攻方报告里的基线相同。

## 四、check.sh 与门禁阶段

`nice -n 19 bash .claude/scripts/check.sh`（主工作区，15:05:49–15:07:09 UTC）末尾原样：

```
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

  ✓ 单测通过
exit 0
```

多花的时间：同一台机、`/usr/bin/time -v` 量整道 check.sh。改之前（开工时的树拷到副本上跑）1:18.99，改之后两次 1:23.32、1:20.78 ⇒ 多 2–4 秒墙钟。随机历史那个测试二进制 42.98 s → 51.58 s / 51.98 s（多 9 秒左右）：新加的逼近墙取样点 debug 下 45–46 秒，与快档（52 秒）并行跑，拖慢了同一二进制里别的段；新加的三条写死用例与 lib 里四条单测合计不到 2 秒。

阶段归属表登记给实现员的三个阶段（`awk … stage-owners.tsv` 列出 33、53、74）：

| 阶段 | 原样末行 | 退出码 |
|---|---|---|
| 33-mutation-tables.sh | `改完跑 bash research/scripts/mutate.sh <bin> <源文件> <表> 证明每条都被抓，再来。` | 1 |
| 53-format-const-placeholders.sh | `✓ 格式常量文件里的占位都指得到分项或欠账（2 个占位：D23（journal 的角色与格式） 已定项 19；C323（镜像大小全仓没有条款））` | 0 |
| 74-model-differential.sh | `随机历史：偏向抬 F 之后回退的取样点：模型对拍 913 步：该拒而拒 400、区间里拒 28、该成而成 485；比过根 754 条、分配记录 13200 条、冷启动内容 16 次、抬 F 上限 125 次；回退到 txg = F_生效 > 0 的根做成 7 次；分配记录墙按镜像上的真条数放行 0 次` | 0（26.7 秒） |

（上面代码块最后一行 `exit 0` 是我包在外面的 `echo "exit $?"` 打的，不是 check.sh 自己的输出；整份在 `logs/check-after-2.log`，四段 ✓：格式、clippy、构建、单测，36 个测试二进制 `test result: ok`、0 处 FAILED。）

33 号红的原样（它点名的是另一个会话的实验表，`research/mutations/e155_second_run_fsync_write_volume.tsv`，`git status` 显示它是未跟踪的新文件，不在这一轮的改动里，没修）：

```
  ✗ 这些变异条目的「原文」在源码里不是恰好命中一次（mutate.sh 会退出码 3，后面的条目一条都不跑）：
      e155_second_run_fsync_write_volume 的 M7_乙在fsync写全部祖先（表第 7 行，命中 2 次）
      e155_second_run_fsync_write_volume 的 R2_M8_中间版漏掉数据单元（表第 23 行，命中 2 次）
      e155_second_run_fsync_write_volume 的 R2_M15_乙M在一批里写全部祖先（表第 25 行，命中 2 次）
```

`crates/mutations.tsv` 自己的锚点我另数了一遍：159 行变异、六段齐、原文全部恰好命中一次（33 号没点名它）。74 号只查快档与两个取样点这三段，这一轮新加的第四段（逼近分配记录墙的取样点）不在它的名单里——74 号归主 agent 改。

## 五、这一轮写过的文件

- `crates/singlefs-checker/src/walk.rs`（新函数）
- `crates/singlefs-core/src/mount.rs`、`crates/singlefs-core/src/transaction.rs`、`crates/singlefs-core/src/allocator.rs`（后者只改两段文档注释）
- `crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/src/model_comparison.rs`、`crates/singlefs-harness/src/history.rs`（这三份在开工前就是未提交的第 2 件改动，我在上面接着改）
- `crates/singlefs-harness/tests/second_transaction_supplement_three_random_history.rs`、`second_transaction_step_four_rollback.rs`、`second_transaction_step_one_overwrite.rs`、`second_transaction_supplement_two_unequal_devices.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`
- `crates/mutations.tsv` 末尾追加第 156–164 行（九行，变异名即第三节表里那九条的第一段；原有 155 行一字没动）
- `crates/singlefs-harness/src/lib.rs` 没碰（`git diff --stat` 里那 2 行是开工前就有的）

这一轮相对开工快照（`baseline-snapshot/crates-litmus-before.tar`）的增删行：walk.rs +38；allocator.rs +4 −2；mount.rs +28 −11；transaction.rs +20 −9；history.rs +258 −74；model.rs +260 −40；model_comparison.rs +169 −17；三号随机历史测试 +246 −12；步 4 测试 +19 −20；步 1 测试 +6 −5；回落测试 +7 −5；不等盘测试 +45 −31；mutations.tsv +9。

`git diff --stat -- crates litmus` 原样（含开工前就有的第 2 件改动；`model.rs`、`model_comparison.rs` 是未跟踪文件，不在 stat 里）：

```
 crates/mutations.tsv                               |  19 +
 crates/singlefs-checker/src/walk.rs                |  38 +
 crates/singlefs-core/src/allocator.rs              |   6 +-
 crates/singlefs-core/src/mount.rs                  |  39 +-
 crates/singlefs-core/src/transaction.rs            |  29 +-
 crates/singlefs-harness/src/history.rs             | 897 +++++++++++++++++----
 crates/singlefs-harness/src/lib.rs                 |   2 +
 .../tests/second_transaction_step_four_rollback.rs |  39 +-
 .../tests/second_transaction_step_one_overwrite.rs |  11 +-
 ..._transaction_supplement_three_random_history.rs | 392 ++++++++-
 ...ion_supplement_two_commit_generated_fallback.rs |  12 +-
 ...d_transaction_supplement_two_unequal_devices.rs |  76 +-
 12 files changed, 1304 insertions(+), 256 deletions(-)
```

## 六、停下交主 agent 的设计问题

1. **两个回退成员并进了判别字段**。判决列的四条（低于 F_生效 / 被抛弃 / 不在环里 / 树表 0 条）我照字面做成 `RollbackTargetNotACandidate` 的一个字段，原来单列的 `RollbackTargetNotInRing`、`RollbackToVersionWithoutFileUnsupported` 两个成员删掉（不删就有两种写法说同一件事，字段里那两个值也永远构造不出来）。问题：「树表 0 条」按 D16 已定项 1 的候选集条款可以**在**候选集里，它是第一版不支持，不是「不是候选」；成员名 `NotACandidate` 对这一值不贴，我在值名（`TargetVersionWithoutFileUnsupported`）和文档里写明了。要保留原来两个成员、字段只留两值，是机械改回。kb 里点名被删成员的：`.claude/kb/milestone/02-second-txn.md` 第 141、166 行（现状描述）。
2. **`NoSpaceFor` 没拆成三个成员，换成一个带原因的成员** `PlacementRefused { unit, refusal: PlacementRefusal }`：调用方按 `refusal` 的四个值穷举（容量不够 / 小盘写满 / 用户数据各盘落点不同 / 提交内生块各盘去处不同）。拆成三个平级成员要么让某个成员能装进不属于它的原因（非法状态写得出），要么再造一层类型；按「调用方要做的决定」算，容量不够与「第一版不支持的池形状」是两种决定，要不要再提一层成员由主 agent 定。kb 里还写着「发布层统一报 `NoSpaceFor`」的：`.claude/kb/milestone/02-second-txn.md` 第 329、330 行，`.claude/kb/checks-owed.md` 第 335 行。
3. **「真条数」取的是准入口径的数**：镜像上数的这一版的记录条数 + 这次（这一串）每个重写的角色每盘一条。这是收口表第 39 行用户定保留的上界准入，与实现同一口径；这次发布真写完之后的条数会因复用而更少，那个数不是准入判的对象，我没用。树表 0 条的一版上数出 0 条（mkfs 两个单元的记录到第一个文件版本才落盘），比实现分配器里的 4 条少，离 812 远、走不到墙。数不出基数一律不放行——条款没写这一格，我取了严的一边。
4. **为走到墙新加了一种执行方式 `PerStepChecker::Skipped`**（每一步不跑池级 checker）。跑 checker 的历史在根环转过一圈之后停在已知红第 0 条，走不到 812 条（攻方表：checker 开着 200 步最高 566 条）。这一档只由模型、执行器的判定与 panic 判；已知红第 0 条那一类问题这一档看不见。要不要让 74 号把它列为第四段、大档要不要给它一个环境变量开关，由主 agent 定。
5. 分配记录墙「必须拒」那一头（真条数超过 812 而实现做成了）仍不判，与改之前相同：装不下还去写会在建节点时 panic，由第 1 件判。

## 七、没做什么

- 没走三方对抗；层 0、QEMU、herd7 与 crates 变异表整道（59 号）归 `crash-verifier`；没提交。
- 没改 74 号（判决第三节第 3 条归主 agent），没改 kb（第六节列的几处 kb 行要跟着改）。
- 变异证明只在 debug 下跑；W1 在取样点上的段数（9 / 32）另在 release 下量过同一数（探索时 32 × 150：9 段），基线零误红。攻方长历史只量了 broad、reuse 两组比重，没量 rollback 那组。
- 没加层 0 流或崩溃点重放用例。
- 产物没进 `research/results/`（写范围闸不放行）：全部在 `/tmp/claude-1000/m2-supp3-item2-fix-implementer/`——`logs/proof-*.log`（九条变异与基线的整二进制输出）、`logs/proof-progress.txt`、`logs/long-*.log` 与 `logs/long-progress.txt`（攻方长历史设置）、`logs/check-before.log` / `check-after-1.log` / `check-after-2.log`（计时）、`logs/gate-*.log`、`scripts/run-proofs.sh`、`scripts/run-long.sh`、`scripts/apply-tsv-row.py`、`scripts/new-rows.tsv`；副本在 `copies/`（带 target，约几 GB，可删）。门禁 69 号在变异表改了之后要 `research/results/` 里有一份不比它旧的产物，要不要把这些拷进去由主 agent 定。

推翻条件：59 号在主工作区逐条施加第 156–164 行时有一条点名的测试没红；或主 agent 在别的种子区间 / 步数上量到逼近墙的取样点基线误红（这一档不跑 checker，模型之外的问题可能以模型对不上的形态出现）。
