# 实审 B3c-2 报告（implementation-writer）：崩溃注入第二、三截交记录核对器的写表补全（代码审阅第 2 条收尾）

开工 2026-09-27。按规格在草稿目录的仓副本里做、交补丁，主工作区一个字没动。

## 一、结论

- **第 1 件（第三截 = 二次崩溃）改好了**：第三截交给记录核对器的写表从「挂载那一段到当前段为止的前缀」换成「整条历史录制流 + 整条挂载流 + 挂载之后那次发布」，持久集合逐条对应（历史那一段 = 第一次崩溃的集合；挂载那一段按二次崩溃：更早的段全落、当前段按子集、更晚的段全没落；之后那次发布全没落）。`crash.rs` 的 `publishes_in` 认接缝（`RecordStreamContinuity::ResumedAfterACrash`）：两条流各自分发布，截断那条流末尾没等到根槽写的写不归挂载的第一次发布；截断那条流里只留崩溃之后时间线上的发布，也就是根槽写落了盘的，加上第一次崩溃后恢复落到的那一版（由记录重建）。
- **第 2 件（第二截 = 挂载再发一次布之后的池）改好了**：记录核对器改成收两份镜像，`check_records_against(crash_state_image, recovered_image, writes, persisted, continuity, effective_root)`。崩溃态镜像判「根在而记录不在」（择根与前缀），恢复后的镜像判单元在不在、读落到那一版的实例表。第二截接上了它：崩溃态 = 第一次崩溃的镜像，恢复后 = 挂载与那次发布之后的池，写表同第三截（挂载与那次发布全落），恢复自称的那一版取挂载与那次发布写出的最新那条根。第一截、第三截、层 0 都改用同一个接口，两份镜像交的是同一份。
- **先红后改**：两件各有一个会红的状态，在今天的代码上先摆出来，看着今天的交法不报（第四节），改后都报。**不假红的对照**有三个：第一次崩溃截在发布中间（44 个状态、832 次判定都不红，同一批状态上两种「接成一条」的交法各有判红）；落到的那一版的实例表只在恢复后的镜像上；崩溃注入整段跑下来没有新发现。
- **变异**：追加 13 行、替换 3 行（三行老锚点被这次改没了），16 行都在副本上证过红（第五节）。第一次跑有 1 行没红，补了断言之后在第二次抓到。最终代码在新取的副本上又整批证了一遍（第九节）。
- **途中多发现一处**：挂载之后那一次发布的写原来不在 `mount_operations` 里（取的是挂载返回那一刻的流），可是第二截的池上有它。长历史（40 次覆盖写，过了根环 24 槽）上，它会合法复用历史发布换下的槽，写表里没有它就解释不了，判红，这是假红。所以把 `publish_operations` 单独拿出来放进写表，留了一行变异。
- **什么现象会推翻它**：① 把 16 行变异里任一行改回去，点名的那条测试还绿；② 提交时崩溃验证员跑快档（release），第二、三截冒出记录核对器判的新发现，查下来是交法假红（比如 (实例, txg) 撞了而实例表读不出、回收谓词在接起来的写表上判得过严）；③ 真实历史上出现第一次崩溃之后恢复没落到、根槽写又落了盘、却不在挂载那一版时间线上的发布（我的剔除条件认它在时间线上）。

## 二、写过的文件（都在副本里，补丁在 `patch/`）

| 文件 | 改了什么 |
|---|---|
| `crates/singlefs-harness/src/crash.rs` | 加 `RecordStreamContinuity`（`:731`）；`publishes_in` 带持久集合与接缝（`:753`），一条流的分法挪进 `publishes_of_one_recording`（`:785`）；`check_records_against` 收两份镜像与接缝（`:878`；`:897` 读崩溃态、`:900`/`:904` 读恢复后）；`check_records` 两份交同一份（`:821`）；`some_publish_persisted_without_its_root`、`newest_persisted_root` 按一条流调；文档注释照新交法写（规格引的 `:767`、`:848` 两句，取副本时在 `:769`–`:774` 与 `:854`；回收谓词那一段补一句接缝处的松紧，`:1060`） |
| `crates/singlefs-harness/src/crash_injection.rs` | 加 `HistoryThenWritableMountRecords`（`:1207`，第二截 `:1295`、第三截 `:1325`）与 `PublishAfterTheMountInTheState`（`:1347`）；第一截改调新接口（`:891`）；第二截接记录核对器（`:958`）与计数 `record_checks_after_the_writable_mount`（`absorb`、`render` 跟着改）；第三截改整条流（`:1640`）；`WritableMountOnTheCrashImage` 多出 `publish_operations`（`:1186`），它与 `mount_and_publish_once_on_the_crash_image`、`materialized_crash_image`、`WritableMountOutcome` 改成 `pub`（新用例要用）；模块头、`CrashStateStage`、第三截函数的文档照改 |
| `crates/singlefs-harness/tests/second_transaction_supplement_three_crash_injection.rs` | B3c-1 两条用例改调新接口；小快档多钉一项「挂载之后的池上记录核对器每个崩溃状态都跑了」 |
| `crates/singlefs-harness/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`（新建） | 8 条用例，见第五节 |
| `crates/mutations.tsv` | 没直接改。`patch/mutations-append.tsv` 13 行、`patch/mutations-replacements.tsv` 3 行（名字见第五节） |

`git diff --stat -- crates litmus` 在副本里没法跑（副本不带 `.git`）。补丁的 stat 在第九节，是在临时仓里用 `git diff --stat` 原样出的。

## 三、改法与依据

- 条款：D13（验证路线） 已定项 7（`.claude/kb/decisions/13-验证路线.md:125`「#### 已定项 7：O2（独立解析器 + checker） 判的是**一个**镜像」）写明记录核对器的入参是「(崩溃前镜像, 记录流, 崩溃后镜像, 持久集合)」，「崩溃后镜像」是两份：「择根与前缀判定看**崩溃态镜像**……比对的对象是**实现恢复后的镜像**」。已定项 4（同文件 `:69`）定了持久集合的形状（更早的段全落、当前段子集、更晚的段全没落），第三截挂载那一段的持久集合照它记。代码审阅第 2 条的用户定案见 `records/2026-09-27-代码审阅38条去向.md`「用户定案」表第 2 行。
- 哪一份镜像判什么：第一条判据（根在而记录一条都不在）看的是「根槽写与记录写在不在盘上」，属于择根与前缀，读崩溃态镜像。第二条判据（恢复自称的 txg ≥ 某次发布、它的单元全缺）看的是单元在不在，读恢复后的镜像。落到那一版的实例表也从恢复后的镜像上读，因为第二截恢复自称的那一版是挂载自己写出的，它的表只在挂载之后的池上（用例 B4 与变异 3 证了这一点）。
- 接缝处的分法（第 1 件）：用规格点名的做法，让 `publishes_in` 认「一段流从这里断开」。我落到代码上是两条规矩，都写在 `crash.rs:753` 的文档注释里：
  1. 两条流各自按根槽写分发布，截断那条流末尾没等到根槽写的写不归挂载的第一次发布（用例 A3）；
  2. 截断那条流里只留崩溃之后时间线上的发布：根槽写落了盘的，加上第一次崩溃后恢复落到的那一版（用例 A2）。

  这里的「截断处」由持久集合与恢复落到的那一版来认，接缝只记一个下标（`first_write_after_the_crash` = 历史录制流的写数）。第 2 条是我自己定的，理由是真实状态上量出来的：整条历史接一条来分，截断处之后历史自己重开的实例与挂载取到同一个号、txg 接在同一处，身份撞上，而挂载那一版的实例表不带本实例的行，豁免不到，结果判红（用例 B2 的阳性对照：同一批 832 次里判红 92 次）；只接到截断处为止，截在中间那次发布没落的写归进挂载写行那次发布，判红 4 次（这就是 B3b 报告第二节第 4 条那一形）。
- 回收谓词（`reuse_is_not_proven_illegal_by_the_reclaim_predicate`）没改。接缝之后的写在取「这次写之前」时，照样把截断处之后那一截没发生过的历史写算进去。那一截的根 txg 都比截断处之前的大、带的 F 不比之前小，算进去只会让界更宽，不会把合法复用判成不合法。这是推的，没造状态量过松了多少，已写进 `crash.rs:1060`。
- 第二截恢复自称的那一版：取挂载与那次发布写出的最新那条根，挂载一条根都没写出时退回第一次崩溃落到的那一版。第二截的恢复后镜像是整次挂载加一次发布之后的池，池上现行的是那一版（用例 B3 挂载发布那一半与变异 11 证了这一点）。
- 今天的交法与改后的交法各在哪：今天第三截取 `mount_writes[..first_write_of_the_segment + writes_in_the_segment]`，只交挂载前缀（取副本时 `crash_injection.rs` 第 1422–1461 行）；今天第二截只跑 `checker_violations_of`，观察里的记录核对器结论写死成 `RecordCheck::default()`（取副本时第 1231 行）。
- 没停在「条款没写」的分支上：这次只改装置怎么交入参、判定用哪份镜像，没有新加错误成员、`todo!` 或 `assert!`。新加的 `assert_eq!` 都是入参长度的不变量：持久集合与写表逐条对应。

## 四、先红：今天的代码上两个状态都不报

在今天代码的副本 `/tmp/claude-1000/impl-rev-b3c2/today/` 上跑。这份副本里 `crash.rs`、`crash_injection.rs` 与取副本时的主工作区逐字相同，只把挂载那一步的函数与结构改成 `pub`，这是它能被草稿用例调到的前提。草稿用例 `tests/scratch_today_demonstration.rs` 不进补丁，副本删了之后留了一份在 `/tmp/claude-1000/impl-rev-b3c2/scratch_today_demonstration.rs`。状态与新用例 B1、B3 相同：只覆盖写的 40 步历史，第一次崩溃扣下最后一次根槽写，挂载加一次发布，二次崩溃摆在挂载最后一段、扣下最后一个写。原样输出（`logs/today-demo.log`）：

```
TODAY third stage: second crash landed Some((InstanceGeneration(2), CheckpointTxg(46))); clean damages of history units 12, reported by today's record check 0
TODAY second stage (checker only): tried 1, first damage the checker does not report Some(((InstanceGeneration(1), CheckpointTxg(11)), 824770560))
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.32s
```

- 第 1 件：二次崩溃镜像上，历史发布的单元有 12 处被干净地弄坏（每份副本翻一个字节，恢复仍落到 (2, 46)）。今天第三截交的是挂载前缀，12 处一处都报不出来。改后 B1 这 12 处都判红。
- 第 2 件：挂载之后的池上，(1, 11) 写在偏移 824770560 的单元两份都弄坏了，池级 checker 一条都不判。这一版的根已经被挤出根环，单元也已被换下。今天第二截只跑 checker，所以报不出来。改后 B3 判红。
- 小快档前后的红集相同。今天的代码上（`logs/today-small-fast-tier.log`）与改后（`logs/existing-binary-1.log`）都是「崩溃状态：以已知红收尾 {}、新发现 15」，新发现全是种子 7463871032432355115 第 29 段的 I-7.4，五截各一个签名，是规格说的 C554 那一形。改后第二截的记录核对器 16 次、第三截 48 次，一次都没判红。

## 六、快档（`crash-case:crash-injection-fast-tier`，标了 ignore）哪些判定会变（推的，快档没跑，归提交时的崩溃验证员）

- 输入指纹会变：`crates/` 下改了四份文件。
- 第二截在每个崩溃状态上多跑一次记录核对器，新计数 `record_checks_after_the_writable_mount` 打在报告第「崩溃后镜像上的可写挂载」那一行末尾（`之后的池上 checker 跑了 N 次、记录核对器跑了 N 次`）。判红的途径有两条：
  - 挂载之后的池上，恢复自称那一版时间线上的某次发布缺单元，而且没有落了的、过得了回收谓词的更晚写来解释（历史发布与挂载写的发布都算）；
  - 第一截判过的「根在而记录不在」会在这一截重复判一次。两截的崩溃态镜像是同一份，所以第一截判红的状态第二截也会红，新发现里会多一行 `after_the_writable_mount_and_one_publish`。
- 第三截的记录核对器多核了历史那几次发布，以及挂载写出、根槽写落在更晚段的那一版（写行由记录重建时）。所以在二次崩溃镜像上历史单元缺席的状态会新红。
- 小快档（头 4 段，快档的一部分）在 debug 下量过：第二截 16 次、第三截 48 次，一次都没判红（第四节）。剩下 20 段没跑，会不会冒新红，要等快档跑出来。冒红时先对照第一节的推翻条件，分清是交法假红，还是实现真的丢了单元、做了不合法的复用。
- 耗时：每个崩溃状态多了一次写表克隆（历史写表接挂载写表）和四次记录核对器调用。小快档用时，改前是 11.40 秒（今天代码副本，`today-small-fast-tier.log`），改后在第九节。

## 七、不做、只列：树表 0 条的那一版，实例表与分配代比不了（B3b 报告第二节第 2 条第 1 小条）

今天的样子：`crates/singlefs-harness/src/model_comparison.rs:280` `observed_root_of_version_without_file` 交回 `ObservedInstanceTable::NotInTheOutput`（`:288`）与 `ObservedUnitAllocationRecords::RewrittenRolesOnly`（`:291`）。模型那一侧，这两种各走一臂只计数、只比重写集合（`crates/singlefs-harness/src/model.rs:2014`、`:2041`）。行号是取副本时的，主工作区后来打进了 A2c、C11、C11b 的补丁，行号要现查。

- **路一：让 `transaction.rs` 的输出带上这两样**（A2c 在改）。
  - `crates/singlefs-core/src/transaction.rs:821` 的 `VersionWithoutFilePublishOutput` 加两个字段：这一版整条实例表链各片的 `TransactionUnit`（字节 + 位置），与这一版每个角色的分配记录。
  - 三处构造都填上（取副本时在 `:1058`、`:1127`、`:1669`：写行的与零单元的两条路；零单元那条要从现行版本照抄）。
  - `model_comparison.rs:280` 改成照带文件那一版的解法（`observed_instance_table` 那一段，取副本时在 `:181`–`:219`）解实例表，分配记录交 `EveryRoleOfTheVersion`。
  - `model.rs` 那两臂从此不再从这条路进来。模型对树表 0 条那一版的角色全集（`role_written_at`）要能与实现交回的全集两个方向都比，这要不要改、改几处我没逐行核，是推的。
- **路二：在 `history.rs` 里从挂着的分配器与盘上读**。
  - 在观察树表 0 条那一版的地方（取副本时 `crates/singlefs-harness/src/history.rs:2882`，调 `observed_root_of_version_without_file` 那一处），按根记录里的实例表指针，从挂着的盘上沿链读出各片（照 `model_comparison.rs` 解带文件那一版的做法）；再从挂着的分配器上按这一版的每个角色取分配记录。
  - `model_comparison.rs` 另加一个收这两样的构造函数。
  - `model.rs` 同样要让树表 0 条那一版走 `Rows` 与 `EveryRoleOfTheVersion` 两臂，那两臂的判法要能认树表 0 条那一版的角色集，同上是推的。
- 两条路都要动 `model.rs`（C11 在改），这一件照规格不做。

## 八、交主 agent 的问题与发现

1. **kb 要改的句子**（我不写 kb，交 kb-scribe）：`.claude/kb/decisions/13-验证路线.md:134`（已定项 7 的「射程」）里有两处与改后的交法对不上。一处是「记录核对器（只核挂载自己写出的那几次发布）」，改后第三截也核历史那几次发布。另一处是「「比对的对象是实现恢复后的镜像」那一半，今天只有可写挂载之后的池级 checker 落在实现恢复后的镜像上；记录核对器在崩溃注入里与层 0 一样，入参的崩溃后镜像取的是崩溃态那一份」，改后第二截的记录核对器判在不在读的是挂载之后的池，也就是实现恢复后的镜像。整行原样如下：

```
**射程**：指针层目标态下核对一条记录只需格式解析 + 校验和 + 根环择新；逻辑意图记录不点名任何盘上对象，核对方要自行实现树语义 + 记账 + 分配合法性——D23（journal 的角色与格式） 否掉逻辑意图日志的第二条理由据此改写成第一条的一条封堵，写在 D23（journal 的角色与格式） 已定项 5。记录核对器的代码在 `crates/singlefs-harness/src/crash.rs`，接在层 0 的每个崩溃状态上（门禁 54 号）。记录核对器的判定因此不只是镜像的函数：崩溃后镜像逐字节相同、持久集合不同的两个状态，它可以判得不同，按镜像去重的提速对它不适用（O2（独立解析器 + checker） 与 oracle 照旧只看镜像）。 层 0 不覆盖可写挂载：层 0 在每个崩溃状态上只跑只读恢复（看 journal 与不看各一遍），取号、写行、暖机在层 0 的任何崩溃状态上都不跑。可写挂载由崩溃注入覆盖（`crates/singlefs-harness/src/crash_injection.rs`）：每个抽到的崩溃状态上，只读恢复与判定之后在同一份崩溃后镜像上起一次可写挂载、再发一次布、跑池级 checker；挂载途中再崩一次，取号、写行、暖机三段各摆一个二次崩溃状态，每个上只读恢复、问模型、池级 checker、记录核对器（只核挂载自己写出的那几次发布）。「比对的对象是实现恢复后的镜像」那一半，今天只有可写挂载之后的池级 checker 落在实现恢复后的镜像上；记录核对器在崩溃注入里与层 0 一样，入参的崩溃后镜像取的是崩溃态那一份。
```

2. **挂载之后那一次发布的写原先不在挂载录制流里**（第一节第 5 条）。第三截只在挂载途中摆二次崩溃，这一点没变，那次发布的写在第三截的表里记成没持久。第二截的表里记成全落。
3. **B3b 那份用例文件没动**（`crash_injection_writable_mount_after_the_crash.rs`，不在我的文件单里）。它照旧绿（第九节），可它没钉第二截记录核对器的次数。这一项钉在新文件的 B5 与小快档里。
4. **小快档照旧红在 C554 那一形**（种子 …115 第 29 段 I-7.4，五截各一个签名），与改前的红集逐项相同（第四节末条）。B3c-1 暂不进表的两行（M3、第 152 行的替换）照规格仍然不进。
5. **别的会话的文件上看到的红**，都不在我的改动里，没修：
   - 命名 lint：`crash.rs:1241` 的 `iter`、`crash.rs:3882`、`:3932` 两个 `a_` 开头的测试名、`second_transaction_supplement_three_crash_injection.rs:635` 的 `a_kept_unit_withheld`，改前就在（base 的 `crash.rs:1151`、`:3790`、`:3840` 与测试文件 `:631`）；
   - 旧副本上 `cargo fmt --check` 有 `e158_root_choice_repair.rs`、`core_review_unit_area_start_and_publish_limits.rs` 的差；
   - 旧副本上 33 号列出 `allocator.rs`、`admission.rs` 六行锚点 0 次命中；新副本上这六行已经对上（第九节）。
6. **层 0**：`check_records` 的判定不变（两份镜像交同一份，接缝是 `OneRecording`，`publishes_in` 的分法与改前逐条相同），checker 没动。所以没有「受影响的层 0 流与崩溃枚举用例」一节。层 0 各流的钉值不会因这次改动变化，这是推的，层 0 没跑。
7. **写表克隆**：`HistoryThenWritableMountRecords::new` 每个崩溃状态克隆一次整条历史写表（含数据单元字节）。快档 24 段、每段 4 个崩溃状态，量级是几 GB 的内存拷贝，推的，没量。要省可以改成借用两段切片，那样 `check_records_against` 要收分段的写表，接口会更绕。我没做。

## 五、新用例与证红

新文件 `crates/singlefs-harness/tests/crash_injection_record_checker_sees_every_publish_across_a_second_crash.rs`，8 条，整个二进制在最终副本上 `8 passed … finished in 20.80s`。A 组是手摆写表（1 MiB 单盘、整扇区写），B 组是真实历史。

| # | 用例（行号） | 钉什么 |
|---|---|---|
| A1 | `the_root_and_its_records_are_judged_on_the_crash_state_image_and_unit_presence_on_the_recovered_image`（`:107`） | 两份镜像各判什么：崩溃态上根在、记录不在；恢复后记录在、单元不在。两条都判红，两份对调交进去则两条都不判 |
| A2 | `publishes_cut_off_by_the_crash_are_on_the_resumed_timeline_only_if_their_root_landed_or_the_recovery_landed_on_them`（`:153`） | 接缝处的剔除：恢复落到 (1,1) 时，截在中间的 (1,2) 不核（不红）；落到由记录重建的 (1,2) 时核它（红）；根落了盘、不是落到的那一版的 (1,1) 也核（红） |
| A3 | `writes_left_without_a_root_at_the_end_of_the_cut_recording_do_not_join_the_first_publish_after_the_crash`（`:213`） | 截断流末尾没等到根的单元写不归挂载的第一次发布（不红） |
| B1 | `history_publishes_missing_units_on_the_second_crash_image_are_red_on_the_whole_stream`（`:588`） | 第 1 件：40 步只覆盖写的历史，二次崩溃落到 (2,46)。不弄坏时第三截不判（对照）；干净地弄坏历史单元 12 处，每处都判红（只交挂载那一段时 0 处） |
| B2 | `first_crashes_cut_inside_publishes_leave_every_second_crash_and_the_pool_after_the_mount_clean`（`:759`） | 对照：历史「覆盖写两次、关掉重开、覆盖写两次」上，第一次崩溃截在发布中间的状态 44 个；它们的第二截与挂载每段两种摆法的第三截，共 832 次判定一次不红。同一批状态上，「接到截断处为止接成一条」判红 4 次，「整条接成一条」判红 92 次（阳性对照，断言各 ≥ 1） |
| B3 | `units_missing_on_the_pool_after_the_writable_mount_are_red_on_the_second_stage`（`:678`） | 第 2 件：挂载之后的池不弄坏时第二截不判（对照）；弄坏历史发布单元 12 处、挂载之后那次发布的单元 14 处，都判红；其中 (1,11) 那一处池级 checker 不判（断言恰好找到 1 处，找到就不再跑 checker） |
| B4 | `the_pool_after_the_mount_exempts_what_the_newest_mount_version_abandons_by_its_instance_table_on_the_recovered_image`（`:892`） | 对照：历史「覆盖写 → 崩溃恢复抛弃最新根 → 覆盖写三次」，第二截恢复自称 (3,14)，那一版的表只在恢复后的池上，照它豁免被抛弃的 (1,4)，一条不判；同一状态从崩溃态读表时判红（阳性对照） |
| B5 | `every_crash_state_runs_the_record_checker_after_the_writable_mount_and_on_every_second_crash_without_new_findings`（`:960`） | 崩溃注入整段（种子 3、11，各 8 步，每段 2 个崩溃状态）：第二截记录核对器的次数等于崩溃状态数，第三截的次数等于二次崩溃数，没有新发现 |

证红：在最终副本 `/tmp/claude-1000/impl-rev-b3c2/final/`（取于 2026-09-27，已打上补丁与变异行）上跑，命令是 `bash research/scripts/capped.sh 5 bash research/scripts/prove-red.sh --copy <副本> singlefs-harness <16 个名字>`，日志在 `prove-red-logs-final/`。末行原样：`✓ 点名 16 条：跑了 16 条，跳过 0 条，跑的都抓到了`。基线：新二进制整个跑 8 条全绿，所以基线红集为空；prove-red 各组参数的基线也都绿，留下的最后一份是 `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 9.29s`。每行按测试全名过滤，只跑点名那一条，同时红了谁没看。16 次红的都是测试自己的断言，不是被测代码里的 `debug_assert`，所以没有在 release 下再跑。

| # | 变异名（追加行） | 改坏哪一处 | 红在（最终副本行号，消息原样取头一截） |
|---|---|---|---|
| 1 | 实审 B3c-2 第 2 件：记录核对器判单元在不在读崩溃态镜像（不读实现恢复后的那一份） | `crash.rs` `unit_copy_is_missing_under_the_persisted_set(recovered_image, …)` → `crash_state_image` | A1 `:117`「崩溃态镜像上 (1, 1) 的根在、记录不在；恢复后的镜像上 (1, 1) 的单元不在：两条都判」 |
| 2 | 实审 B3c-2 第 2 件：记录核对器判根槽写与记录写在不在盘上读实现恢复后的镜像（不读崩溃态那一份） | `crash.rs` `PoolReader::read(crash_state_image, …)` → `recovered_image` | A1 `:117` 同上 |
| 3 | 实审 B3c-2 第 2 件：恢复落到的那一版的实例表从崩溃态镜像上读（…豁免落空） | `crash.rs` `instance_table_of_the_effective_root(recovered_image, …)` → `crash_state_image` | B4 `:907`「挂载写出的最新那一版的实例表抛弃 (1, 4)，它被复用盖掉的单元不算该在」 |
| 4 | 实审 B3c-2 第 1 件：接缝处不剔截断的流里没发生过的发布（截断处之后才写根槽写、恢复又没落到的发布也算…） | `crash.rs` 过滤条件前加 `true \|\|` | A2 `:180`「恢复落到 (1, 1)：截在中间的 (1, 2) 没发生过，它没落的单元不算 (2, 2) 该有的」 |
| 5 | 实审 B3c-2 第 1 件：接缝处不剔截断的流里没发生过的发布（真实历史：…撞了身份） | 同 4 | B2 `:788`「第一次崩溃在第 12 段 [true, false]、落到 Some((InstanceGeneration(1), CheckpointTxg(3)))：挂载之后的池上一条不判」 |
| 6 | 实审 B3c-2 第 1 件：接缝处不认第一次崩溃之后恢复落到的那一版（…它缺单元不判） | `crash.rs` `\|\| version_landed…` → `\|\| false && version_landed…` | A2 `:189`「恢复落到由记录重建的 (1, 2)：它在时间线上，它的单元没落照判红」 |
| 7 | 实审 B3c-2 第 1 件：接缝处连根槽写落了盘的发布也剔掉（截断的那条流里只留恢复落到的那一版） | `crash.rs` `persisted[publish.root]` → `false` | A2 `:199`「根落了盘的 (1, 1) 不是恢复落到的那一版，也在时间线上，它的单元没落照判红」 |
| 8 | 实审 B3c-2 第 1 件：两条流接成一条来分（…归进挂载的第一次发布） | `crash.rs` `writes.split_at(first_write_after_the_crash)` → `split_at(writes.len())` | A3 `:224`「截断的那条流末尾没落的单元写不归 (2, 2)」 |
| 9 | 实审 B3c-2 第 1 件：第三截退回只核挂载那一段（…没人核） | `crash_injection.rs` 第三截交 `self.mount_writes()`、`persisted_in_the_mount`、`OneRecording` | B1 `:650`「历史发布 (InstanceGeneration(1), CheckpointTxg(11)) 写在偏移 824770560 上的单元每一份（写表第 [269, 270] 项）在二次崩溃镜像上都坏了…第三截照判红」 |
| 10 | 实审 B3c-2 第 2 件：第二截不跑记录核对器（…只剩池级 checker 看） | `crash_injection.rs` 计数与调用两行 → `RecordCheck::default()` | B5 `:982`「每个崩溃状态之后挂载之后的池上都跑一次记录核对器」 |
| 11 | 实审 B3c-2 第 2 件：第二截恢复自称的那一版取第一次崩溃之后落到的那一版（…不核） | `crash_injection.rs` `newest_root_written_after_the_crash.or(…)` → 只取第一次落到的 | B3 `:731`「挂载之后那一次发布（根槽写在它自己那段写表第 30 项）写在偏移 822935552 上的单元每一份都坏了，第二截照判红」 |
| 12 | 实审 B3c-2 第 2 件：第二截把崩溃态镜像当成恢复后的镜像交（…看不见） | `crash_injection.rs` 第二截第二个入参 `pool_after_the_mount` → `crash_image` | B3 `:683`「对照：挂载之后的池上不弄坏时一条不判」（挂载写的单元不在崩溃镜像上，对照先红） |
| 13 | 实审 B3c-2 第 2 件：第二截的写表不带挂载之后那一次发布的写（…假红） | `crash_injection.rs` `.chain(writes_of_the_publish_after_the_mount)` → `.take(0)` | B3 `:683` 同上 |

替换 3 行（名字不变，只换原文与替换文，参数与必须红的测试不变）：「增补 3 第 3 件（用户 2026-09-20 定案第 3 条）：崩溃状态上不跑记录核对器」红在 `crash_injection.rs:2237`「每个崩溃状态都跑过记录核对器」；「实六 C561：check_records 不把崩溃状态的持久集合交给核对器（当成全落）」红在 `record_checker_judges_absence_by_the_persisted_set.rs:444`；「实审 B3c-1：崩溃注入第一截交给记录核对器的写表退回崩溃点所在段为止的前缀（…）」红在 `second_transaction_supplement_three_crash_injection.rs:929`。三行在新副本上都抓到。

证红的经过：第一次整批 16 行里，第 7 行没红。A2 第三个断言原先落到的就是 (1,1) 自己，剔掉「根落了盘」那一条也照样留下它。改成落到 (1,2)、(1,2) 单元都在、只丢 (1,1) 的单元之后，第二次抓到（`logs/prove-red-2.log`）。之后改过名字、补过文档，最终副本上 16 行又整批证了一遍。

## 九、验证（第 4 步那几样，末尾原样）

都在最终副本 `final/` 上跑：主工作区取副本那一刻的样子，打上本补丁与变异行（取副本时在改文件的 sha256 在 `final-copy-sha256.txt`；我这三份文件那一刻与开工时相同）。build、fmt、clippy、四个测试二进制写在同一个脚本里，整条经 `run-with-memory-cap.sh 8G`、`capped.sh 5` 跑。

`cargo build --offline --all-targets`（整个工作区）：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 33.11s
build exit=0
```
`cargo fmt --all -- --check` 退 1，差全在别人的 `crates/singlefs-harness/src/bin/e158_root_choice_repair.rs`（67 处）。我的四份文件单查（`rustfmt --edition 2021 --check …`）：`fmt-mine exit=0`。

`check.sh` 那一套 lint 的 clippy：只查我这几个目标（`-p singlefs-harness --no-deps --lib --test second_transaction_supplement_three_crash_injection --test crash_injection_record_checker_sees_every_publish_across_a_second_crash --test crash_injection_writable_mount_after_the_crash`）时：
```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.99s
clippy-mine exit=0
```
整个工作区 `--all-targets` 退 101，红在别人的 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs:3312`、`:3880`、`:3882`（`shadow_unrelated`）。

测试二进制（整个跑）：
```
新文件      test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 20.80s
崩溃注入    test result: FAILED. 9 passed; 1 failed; 2 ignored; 0 measured; 0 filtered out; finished in 188.78s
B3b 那份    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.80s
--lib       test result: ok. 90 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 99.10s
```
崩溃注入那一条红的是小快档，只红在 C554 那一形（第四节末条、第八节第 4 条），用时 11.7 秒（改前 11.40 秒）。

证红：`✓ 点名 16 条：跑了 16 条，跳过 0 条，跑的都抓到了`（第五节）。

登记给我的门禁阶段（对最终副本跑，`bash .claude/gate.d/<阶段> <副本>`）：
- 33 号，退 0：`  ✓ 148 个实验二进制都有成形的变异表，1690 条变异的原文各命中源码一次；crates/mutations.tsv 1119 条的原文各命中源码一次；没有同名实验二进制、锚点不在本阶段射程的表 6 张：…`
- 53 号，退 0：`  ✓ 格式常量文件里的占位都指得到分项或欠账（4 个占位：D23（journal 的角色与格式） 已定项 19；C506（每区槽数 S 写成编译期常量，条文说它住系统配置）；D22（单元原子性怎么合成） 已定项 1；C323（镜像大小全仓没有条款））`
- 94 号，退 0：`  ✓ checker 与实现只共享常量模块 `singlefs-format`（传递闭包的交集减去它为空：checker 闭包 1 个、实现闭包 2 个，内部依赖图 4 个 crate）；…`
- 93 号，退 0：`  ✓ feature bit 位号在记账表、D15（格式冻结政策） 已定项 4 与代码三处一致（记账表 2 位、登记表分出去 2 位；扫了 56 个 .rs，认出 4 处 feature bit 常量、解出位号 3 处；…）`
- 89 号，退 77，本次未跑，不算通过。末行：`    「alloc-basis 第三轮转来的」那一笔第 27 行写 4 条，这里探针与清单合计 3 条，对不上：清单是手写的，回去按那一轮判决补齐`
- 92 号，退 77：`  ! /tmp/claude-1000/impl-rev-b3c2/work 不是 git 仓，本阶段跳过`。这一次是在旧副本 `work/` 上跑的，副本不带 `.git`，没判；最终副本上没再跑，理由相同。
- 74 号，退 1：随机历史二进制 `test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 40.49s`，红的是 `crash_recovery_abandoning_the_newest_root_then_raising_the_floor_into_its_txg_ends_in_the_known_red_form_of_closeout_row_43` 与 `random_histories_fast_tier_end_only_in_known_red_forms_and_exercise_every_operation`，签名是 ModelDisagreement。**这不是我的改动带进来的**：同一时刻主工作区的副本（`pristine/`，不带我的补丁）上跑同一道，也红这两条（`test result: FAILED. 22 passed; 2 failed; 2 ignored; 0 measured; 0 filtered out; finished in 49.65s`，`logs/pristine-gate-74.log`）；随机历史这一路也不调我改过的函数，只用了没改的 `MemoryPool`、`RecordCheck`、`SparseBlockDevice`。

补丁对主工作区的 `git apply --check`（2026-09-27，那一刻我这三份文件的 sha256 仍是 `baa9fe35…`、`6d37a516…`、`2526b80c…`）：`git apply --check (主工作区) exit=0`；`apply-writer-patch.py <补丁目录> --dry-run`：`✓ 核过了（--dry-run，没改）：补丁 有，变异表合并之后 1124 行`。

补丁的 stat（临时仓里 `git diff --stat -- crates` 原样）：
```
 crates/singlefs-harness/src/crash.rs               |  160 +++-
 crates/singlefs-harness/src/crash_injection.rs     |  289 ++++--
 ...ker_sees_every_publish_across_a_second_crash.rs | 1002 ++++++++++++++++++++
 ...transaction_supplement_three_crash_injection.rs |   12 +-
 4 files changed, 1374 insertions(+), 89 deletions(-)
```

## 十、草稿目录里删了什么、留了什么

- 删了五份仓副本，每份都带着自己的 target：`work/`（15G，开发与第一轮证红）、`today/`（1.8G，今天代码的对照）、`final/`（15G，最终副本）、`pristine/`（1.2G，74 号的归因对照）、`patchrepo/`（772K，出补丁用的临时仓）。
- 留着：
  - `patch/`（交补丁用：`crates.patch`、`mutations-append.tsv`、`mutations-replacements.tsv`、`report.md`）；
  - `report.md`；`logs/`（这一轮全部运行日志）；
  - `prove-red-logs-1/`、`prove-red-logs-2/`、`prove-red-logs-final/`；
  - `base/`（取副本时我那三份文件与变异表的原样）；`copy-sha256.txt`、`final-copy-sha256.txt`；
  - 两份草稿用例 `scratch_today_demonstration.rs`、`scratch_explore_second_crash.rs.today`；
  - `progress.md`；`spec.md`（主 agent 给的规格）。
- 这些都不入库：实验产物规矩管的是 `research/`，这一轮只有代码改动，日志只作第四、五、九节原样输出的出处。要不要留由主 agent 定。

## 十一、没做什么

- 没走三方对抗，没提交，主工作区一个字没动（交的是补丁）。
- 快档、大档、层 0（名字带 layer0 的二进制）、QEMU、herd7、`crates/mutations.tsv` 整表复跑（59 号）都没跑，归提交时的崩溃验证员与门禁。第六节「快档哪些判定会变」全是推的。
- 规格里「不做、只列」的三件照做：树表 0 条那一版的比较只写了两条路（第七节）；单元区墙只计数那件没碰；小快档的 C554 红没绕，没换种子，B3c-1 暂不进表的两行仍然不进。
- 没写 kb（第八节第 1 条交 kb-scribe），没改 B3b 那份用例文件（不在文件单里）。
- 回收谓词在接缝处偏松了多少，没造状态量（第三节末两条）。写表克隆的内存代价没量（第八节第 7 条）。
- 92 号在副本上跑不了（不是 git 仓），89 号退 77，这两道都按没判记。
- 最终副本取得之后，主工作区要是又改了 harness 的公开接口（`history.rs` 的 `execute_history_with`、`HistoryExecution` 这类），新用例可能编不过。补丁交回前在主工作区上 `git apply --check` 过了，但那一刻的主工作区我没重编。
