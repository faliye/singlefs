# 核查员报告：m2-supp3-item4-code-r1（增补 3 第 4 件「故障注入」代码轮第一轮）

我交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

## 〇、判别力自证

抽取待核引用：正推腿（Sonnet）报告第 60 行「`fault_injection.rs:11-15`」附近对
`invariants.md:12`「池级 checker...判 29 条...状态列写「已实现」的就是这 29 条」的引用。
在草稿副本 `/tmp/claude-1000/m2s3i4-r1-verifier/selftest/invariants.md` 里把行号人为
+1 改成 13，重新按「取该行内容、比对报告抄的原文」的方法核：

```
被核引用（人为偏移后）：invariants.md:13
该行实际内容：I-7.7（系统配置实例代号不低于根环） 2026-09-14 按 C322 …
自证结果：核验方法判 ✗（按偏移后的行号取不到「判 29 条」这句，方法有判别力）
```

判别力自证通过：方法在行号被人为偏移一行时能正确判 ✗，不是形同虚设。

## 一、开工快照与报回文件核对

`sha256sum -c research/prompts/m2-supp3-item4-code-r1-start-snapshot.sha256`：9 个文件全部 `OK`
（工作区当前内容与开工快照逐字节相同），因此下面对 `crates/` 与
`.claude/kb/milestone/02-second-txn.md` 的核验直接对工作区做，等价于对快照做。

`invariants.md`、`checks-owed.md`、`decisions/13-验证路线.md` 等 kb 文件不在这 9 个快照文件之列，
也不在这一轮的 diff（`_m2-supp3-item4-code-r1-diff.md` 里 grep 不到这几个文件名）里，
说明它们不是这一轮的改动对象；`git status` 显示 `decisions/13-验证路线.md` 当前有未提交改动（另一会话），
但没有一条腿按行号引用它（只在背景材料小节清单里点名，未被任何一条腿的正文引用具体行号），
不影响下面的核验。`checks-owed.md` 按派发提示的提醒，一律按内容定位，不按行号信任。

报回 sha256：
```
$ sha256sum research/prompts/m2-supp3-item4-code-r1-opus-output.md research/prompts/m2-supp3-item4-code-r1-sonnet-output.md
b5842ce4427d7ff8e080747557a22ff259f50ce994384afa403e7c2a44f91613  ...-opus-output.md
b887018df9e2805d6b568af6634f970c206da1f8a0ac420f53e189009076112b  ...-sonnet-output.md
```
与派发提示给的两个 sha256 逐字符相同；行数 378 / 93 与派发提示一致。
`m2-supp3-item4-code-r1-opus-model/` 下 10 个文件的 sha256 与报告表格逐条比对，10/10 相同。

## 二、云端正推腿（Sonnet，K4/K5）核对表

| 引用 | 核的结果 | 命令 |
|---|---|---|
| `history.rs:806-901`（`HistoryPool::start`） | ✓ | `awk 'NR==806,NR==810' / 'NR==897,NR==901' history.rs`，函数签名与末尾内容对得上 |
| `history.rs:2844`（`Cell::new(StepPosition::StartingPoint)`） | ✓ | `awk 'NR==2844'` 逐字相同 |
| `fault_injection.rs:1190-1196`（`marks.push(StepMark{...})`） | ✓ | `awk 'NR==1190,NR==1196'` 逐字相同 |
| `fault_injection.rs:1108-1156` 的 `draw_faults`、`(1..marks.len())` | ✓（与主 agent 现查一致，独立复核） | `awk 'NR==1125,NR==1127'`：`(1..marks.len())` 逐字相同 |
| `fault_injection.rs:1123`（注释「起点那一段不摆」） | ✓ | `awk 'NR==1123,NR==1124'` 逐字相同 |
| **`make_filesystem.rs:57-82`（`MakeFilesystemError`，宣称 `BlockDevice(_)` 在 79 行）** | **✗（行号错、范围也不全）** | `awk NR==79` 实际是文档注释「参数给的不是它：暖机次数与第一个事务的 txg 3 都压在这个归属上」，与 `BlockDevice` 无关；`grep -n "BlockDevice(BlockDeviceError)" make_filesystem.rs` 命中的真实行号是 **84**，且整个枚举在 85 行才 `}` 收尾，超出报告写的 `57-82` 范围——报告给的行区间本身没有把这一句包进去 |
| `grep -n "make_filesystem(&parameters" crates/*/tests/*.rs crates/*/src/*.rs` 命中「全部 9 处调用」，列出 `first_transaction_step_one_mkfs.rs:77,317,378` 等 | **✗（命令输出与报告列的不一致，实际位置见下）** | 原样重跑该命令，真实 9 处命中里**不含** `first_transaction_step_one_mkfs.rs:317`（该行实际是 `make_filesystem(&layout_parameters, ...)`，不含子串 `&parameters`，命令抓不到它），**含** `crates/singlefs-harness/src/history.rs:832`（报告的清单里没有这一条）。即：命令真实输出 = {77,378,239,84,146,110,326,36,832}，报告写的清单 = {77,317,378,146,239,110,36,84,326}，两者不同（317 vs 832 互换）。`first_transaction_step_one_mkfs.rs:317` 这一行本身确实存在、内容也确如报告描述（测的是 `RegionDevicesNotTheFirstVersionLayout` 参数校验错），但它**不是**这条具体 grep 命令的命中结果，是报告把「用别的方式找到的一处相关测试」混进了「这条命令的输出」里 |
| `instance_acquisition.rs:128-153`、`:155-175`（两条用例） | ✓ | 函数体、注释、断言逐字对得上 |
| `transaction.rs:374-379`（`acquire_instance`）、`:398-413`（`acquire_expected_instance`）、`:416`（`write_acquired_instance` 签名起点）、`:451`（「块设备报的错原样交回」）、`:469`（`warm_up_after_journal_counter` 文档注释） | ✓ | 逐行核对，函数签名、注释文字逐字相同 |
| `transaction.rs:473-499`（`warm_up_after_journal_counter` 函数体） | ✓ | 逐字相同 |
| `mount.rs:983-1024`（`establish_instance` 起始段，含 1011 行 `acquire_expected_instance` 调用） | ✓ | 逐字相同，1011 行确为该调用 |
| `mount.rs:762-767`（`publish_empty_after` 签名） | ✓ | 逐字相同 |
| **`mount.rs:849`（宣称是「回卷只管取号自己那几次写报错，管不到之后的发布失败」的注释）** | **✗（行号错误，实际位置不同）** | `awk 'NR==849'` 实际内容是 `MountError::FormattedPoolMountNotShapedLikeTheFirstTransaction {`，与宣称的注释毫无关系。全仓 grep「回卷」「rollback」定位，真正与该语义逐字相符的注释在 **`mount.rs:867`**（「回卷只管取号自己那几次写报错，管不到取号之后的发布失败。」），另有一句相近意思的在 `mount.rs:89`。报告在「推翻条件」段落里写「我现查过 `establish_instance`……」，但引用的行号本身没有核对到位 |
| `model_comparison.rs:162,177,201,243`（四处 `Unexplained`） | ✓ | 四行逐字确认为 `ObservedRefusalReason::Unexplained` 相关 |
| `model.rs:1399-1402`（`Unexplained => None`） | ✓ | 逐字相同 |
| `history.rs:2070-2110`、`:2094`（`apply_publish_first_file` 调用 `publish_first_file`） | ✓ | 逐字相同 |
| `history.rs:389`（`CloseAndMountWritable, 70`） | ✓ | 逐字相同 |
| `history.rs:394`（`PublishFirstFile, 60`） | ✓ | 逐字相同 |
| **`history.rs:492`（宣称与 394 一起构成「`with_session_open_without_file` 里占 60/95」中的「95」）** | **✗** | `awk 'NR==492'` 实际是 `(HistoryOperationKind::CloseAndMountWritable, 95)`，**不是** `PublishFirstFile`。全仓 `grep -n "PublishFirstFile," history.rs` 核出该文件里 `PublishFirstFile` 的权重取值只有 `60,4,85,1,85,1,90,90`，**没有任何一处是 95**。「占 60/95」这句里的「95」既不在 492 行，也不在文件里任何一处 `PublishFirstFile` 权重上出现过 |
| `history.rs:2896`（`model_disagreement = ...`）、`:2935`（`if !violations.is_empty() ...`）、`:2950-2957`（`continues_past_the_ring_turn_form` 例外分支） | ✓（含报告自己用「……」省略中间代码，省略处经核对属实且报告在别处补全了该例外分支） | 逐字相同 |
| `.claude/kb/checks-owed.md:333`（C378 整行） | ✓ | 内容与整行匹配；本文件为已知被重排文件，按内容核对而非只信行号，恰好当前行号与之相符 |
| `invariants.md:12`、`:17` | ✓（与主 agent 现查一致，独立复核） | 逐字相同 |
| **`fault_injection.rs:1236-1238`（宣称是三行模块级文档注释「注入那一步的结局是不是……」）** | **✗（行号偏一行）** | `awk 'NR==1235,NR==1240'` 显示 1236 行是空行，真正的三行注释在 **1237-1239**（1240 行是 `fn the_fault_surfaced_as_an_error(`） |
| **`fault_injection.rs:1240-1265` 的函数名 `the_fault_surfaced_as_error`（正文两处，第 64、73 行各一次）** | **✗（函数名转录错误，行号范围本身对）** | `grep -n "fn the_fault_surfaced_as_an_error" fault_injection.rs` → 该函数真名是 `the_fault_surfaced_as_an_error`（多一个 `an`），行号范围 1240-1265 与函数体对应准确 |
| `fault_injection.rs:1400`、`:1410-1425`（checker 违例并入 failures） | ✓ | 逐字相同 |

小计：本表共核 24 处引用/命令，✓ 18 处，✗ 6 处（`mount.rs:849` 行号错、`history.rs:492` 权重张冠李戴、`fault_injection.rs:1236-1238` 行号偏一、`the_fault_surfaced_as_error` 函数名转录错、`make_filesystem.rs:57-82`/79 行行号错且范围不全、`make_filesystem(&parameters` grep 命中列表与报告清单不一致）。

## 三、云端攻方腿（Opus，K1/K2）核对表

### 3.1 模型目录与复跑

10 个文件 sha256 逐条比对，10/10 与报告表格相同（见第一节）。`reproduce.sh` 读过全文，
7 个步骤与报告「零、复跑与产物」一节描述的命令、环境变量逐条对得上。

独立复跑（草稿目录 `/tmp/claude-1000/m2s3i4-r1-verifier/opus-repro`，`rsync` 后 `patch -p0 < probes.patch`
加 `opus_probe_one_device_system_configuration.rs`，`nice -n 19 cargo test --release`）：

```
$ nice -n 19 cargo test --release -p singlefs-harness \
      --test opus_probe_one_device_system_configuration -- --nocapture
running 2 tests
test both_system_configuration_slots_dead_on_one_device_makes_the_whole_pool_unrecoverable ... ok
test one_failing_read_on_one_system_configuration_slot_still_recovers ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```
与报告逐字（除挂钟用时，属预期波动）相同。✓

| 引用 | 核的结果 | 命令/依据 |
|---|---|---|
| `fault_injection.rs:154-232`（`FaultDeviceSelector`/`FaultPlacement`/`FaultCounting`/`FaultOccurrence`/`FaultSchedule`/`the_nth_call_across_the_pool`） | ✓ | 逐段 `awk` 核对，字段名、变体、构造器函数体逐字相同 |
| `fault_injection.rs:1276-1279`（`SharedFaultPlan::armed(..., the_nth_call_across_the_pool(...))`） | ✓ | 逐字相同 |
| `fault_injection.rs:201-208`（`fires_at`） | ✓ | 逐字相同 |
| `fault_injection.rs:470`（`state.schedule?`） | ✓ | 逐字相同 |
| `fault_injection.rs:1125-1127`（候选步来自测量跑差值） | ✓ | 逐字相同（与第二节对正推腿的复核一致） |
| `recovery.rs:244`（`(None, None) => return Err(RecoveryFailure::NoValidSystemConfiguration { device })`） | ✓ | 逐字相同 |
| `.claude/kb/checks-owed.md` C79 那一行「2 盘的账已算清……」引文 | ✓（标为「整行的一段」，非摘句瞒报） | 与 checks-owed.md 第 81 行该子句逐字比对相同；报告自己写明只引了半句，未隐瞒截断 |
| `.claude/kb/decisions/22-单元原子性怎么合成.md:170`（D22 已定项 8 第 1 条「系统配置每盘放一份……」） | ✓ | 逐字相同 |
| `fault_injection.rs:746-765`（`InjectionPoint::all()`，7 操作 × 6 类=42） | ✓ | 逐字相同；`HistoryOperationKind::ALL` 确为 7 元素数组（`history.rs:321`），`WRITTEN_STRUCTURES` 确为 4 元素数组（`fault_injection.rs:735`），7×(4+2)=42 核对无误 |
| `fault_injection.rs:1236-1238`（K2 部分引用同一句注释） | 同第二节，✗ 行号偏一（真实 1237-1239），Opus 报告本身**没有**引用这一段行号（这一条 ✗ 只记在 Sonnet 名下），此处只是交叉核实同一段代码未被 Opus 曲解 | 见第二节 |
| `fault_injection.rs:790-791`（`SurfacedAsAnError` 文档） | ✓ | 逐字相同 |
| `fault_injection.rs:1091-1106`（`draw_injected_fault`，`below(4)` 四选一） | ✓ | 逐字相同 |
| `.claude/gate.d/stage-owners.tsv` 里 `three-way-attack` 零登记 | ✓ | 原样重跑两条命令，输出与报告第八节逐字相同（第一条零输出，`grep -c` 输出 `0`） |

### 3.2 产物内数字复核（K2-b、K2-c，主 agent 点名要单独复核的两格）

**K2-b**：`grep -n "新发现\|重开走到模型都不允许" probe-k2-continue.txt` → `新发现 38`、
`模型都不允许的 14 次`，与报告表格「0 → 38」「0 → 14」一致。**✓**

**K2-c**：`grep -n "新发现\|I-3.1\|I-7.8" probe-k2-suspend-model.txt` → `新发现 22`，
两条 `CheckerViolations` 逐字与报告引文相同（种子 …119 → I-3.1「记账的已分配 Some(606208)，遍历全部有效根得到 540672……」；
种子 …133 → I-7.8「根环水位最大 11，盘上出现过的最大树 ID 14」）。**✓**

**「这两条单种子各复现过」**——**部分不成立，需要单独说明**：

`probe-k2-single-seeds.txt` 实际只包含种子 133（I-7.8）、116、122 三段单独重放（`reproduce.sh` 第 5 步的 `for s in`
列表就是这三个种子，**不含 119**），种子 119（I-3.1 那一条）**不在**这份产物里。

我独立在草稿副本上补跑了种子 119（同样打上 `probes.patch`、同样的环境变量组合，`SINGLEFS_FAULT_INJECTION_FIRST_SEED=7463871032432355119 SINGLEFS_FAULT_INJECTION_SEEDS=1`），结果：

```
「已知红」第 0 条：2 次……
种子 7463871032432355119 barrier_fails：整池第 19 次barrier，落在第 1 步（PublishOverwrite）：
  I-3.1：盘 0：记账的已分配 Some(606208)，遍历全部有效根得到 540672；机理：根环槽数 24、最新根 txg 7、
  环里自证过的根槽 7 个、最老的自证过的根 txg 0、遍历的候选根槽 7 个、被实例表判抛弃的根槽 0 个、
  回退下界 F 0、低于 F 的根槽 0 个
```
这段与 `probe-k2-suspend-model.txt` 里种子 119 那条逐字相同（我独立重跑得到，不是抄产物）。
**结论**：种子 119 / I-3.1 这条「单独复现」的**事实本身是真的**（我独立验证了），但报告点名的证据文件
`probe-k2-single-seeds.txt` **不包含**这次单独复现的记录——引用的产物与它要支撑的那句话之间有缺口，
是「产物缺一份」而不是「结论有错」。

### 3.3 两处自我纠错

1. **「49 次 panic、全在 `history.rs:832`」**：`probe-k1-schedule.txt` 里 `persistent-one-device` 段确实
   `注入之后 core panic 的次数：49`，✓；但该段落打印的 panic 位置原文是 **`history.rs:833`**，
   不是报告正文写的 832（`awk 'NR==832,NR==834' history.rs` 显示这是一条跨 832-834 三行的
   `make_filesystem(...).expect(...)` 语句，832 行是 `.expect(` 那一行，833 行是被 `.expect` 的字符串字面量，
   panic 运行时报的位置落在 833）。**✗（行号偏一，但落在同一条语句内，不影响「这是执行器自己的 mkfs expect、不是 core 缺口」这个论点）**
2. **「草稿里写『kb 里按降级那几个词 grep 零命中』，现查不是零命中（10 个文件命中）」**：
   `grep -n "零命中" m2-supp3-item4-code-r1-opus-output.md` 只命中第 352 行（第七节「这条腿自己的限度」的自我纠错段落），
   第一~六节（正文论证）里没有任何一处使用这个被撤回的「零命中」数字；正文 2.4 节反而正确引用了
   「10 个文件命中，其中 C79 那一行正面答了这一格」这个更正后的结论。**✓ 作废的数确实没有流入正文**。

小计：本节共核 22 处（含两处自我纠错与「单种子复现」专项复核），✓ 20 处，✗ 2 处
（`history.rs:832` 应为 833；`probe-k2-single-seeds.txt` 缺种子 119 的单独复现记录，但经我独立重跑证实该缺口不影响底层事实的真实性）。

## 四、本地攻方腿（K3/K6）核对表

本地腿提示明令答复不写文件名与行号，只写 fact 编号，因此下面核的是**转述核对表**
（`m2-supp3-item4-code-r1-local-attack-translation-audit.md`）里登记的「原文文件:行」与源码/kb 原文是否一致，
以及四份样本输出的机械性质（词数、markdown 强调、字词损坏）。

| 转述项 | 核的结果 | 命令 |
|---|---|---|
| K3 Fact 1：`milestone/02-second-txn.md:429`（故障注入逐字要求） | ✓ | `awk 'NR==429'` 逐字相同 |
| K3 Fact 2：`fault_injection.rs:110-123`（六种故障文档注释） | ✓ | 逐字相同，六条注释文本与英文转述一一对应 |
| K3 Fact 3：`fault_injection.rs:610-698`（含 :648/:673/:615-621/:650/:674-677 五个具体行号） | ✓ | 逐行核对，`WriteFails`/`BarrierFails` 报错前不调用 `self.inner`、`ReadReturnsCorruptedBytes` 先调用真读后翻位、`WriteIsSwallowed`/`BarrierIsSwallowed` 同样不转发，五处行号全部准确 |
| K3 Fact 5：`fault_injection.rs:1082-1106`（`draw_injected_fault` 文档注释与函数体） | ✓ | 逐字相同，含「丢一次写要配崩溃注入一起用」原句 |
| K3 Fact 6：`fault_injection.rs:1089-1090`（实测量化声称） | ✓ | 逐字相同 |
| K3 Fact 7：`invariants.md:110`（I-2.1）、`:157`（I-4.8）、`:53`（I-7.4） | ✓ | 三条定义与判别力描述逐字比对，压缩（略去候选集精确定义与「K 代」术语沿革）已在核对表里明确登记为「有意压缩」 |
| K3 Fact 8：`milestone/02-second-txn.md:446`（验收标准） | ✓ | 逐字相同 |
| K3 Fact 9：`fault_injection.rs:1925-1944`、`:1946-1959`（两条孤立单测） | ✓ | 逐字核对完整函数体：`a_swallowed_write_reports_success_and_changes_nothing_on_the_device` 断言读回原字节且 `writes` 计数为 0；`a_swallowed_barrier_reports_success_and_is_counted_apart` 断言 `barriers_swallowed` 加一、`barriers_forwarded` 不变，与核对表描述逐字相符 |
| K3 Fact 10：两条 grep 命令 | ✓ | 原样重跑，输出逐字与核对表「命令与输出」一节相同（零命中退出码 1；全仓命中恰好两个文件） |
| K3 Fact 10 附带的 `bin/first_transaction_on_device.rs` diff 行号（157-165） | ✓ | `awk 'NR==155,NR==165' _m2-supp3-item4-code-r1-diff.md` 与 `swallow_the_next_barrier_on` 函数体逐字相同 |
| K6 Fact 2：`fault_injection.rs:776-785`（`ReopenedVersion` 三个变体） | ✓ | 逐字相同 |
| K6 Fact 3：`fault_injection.rs:860-882`（`FaultInjectionTally` 字段，尤其 880 行注释） | ✓ | 逐字相同 |
| K6 Fact 3：`grep -n "reopened_into_the_version_the_faulted_step_was_writing" .../second_transaction_supplement_three_fault_injection.rs` 零命中 | ✓ | 原样重跑，退出码 1，零输出 |
| K6 Fact 4：`fault_injection.rs:1268-1425`（`inject_one_fault` 评分逻辑，含 :1371-1379/:1380/:1382/:1400-1425 各分句） | ✓ | 逐段核对，`checker_violations_on` 调用（1400 行）与 `failures.push` 分支（1410-1425 行）位置准确 |
| K6 Fact 5：`model.rs:1742-1754`（文档）与 `:1756-1784`（函数体，四条判据） | ✓ | 逐字相同；核对表登记的「首稿漏译④后半句『那一版树表 0 条时只许报没有文件』」经核实**该半句确实存在**（`model.rs:1750`），且确实**没有**被译入 K6 提示的 Fact 5（我读了 `local-attack-k6.md` 对应段落，只到「内容逐字节相同」为止），登记属实 |
| K6 Fact 6：`second_transaction_supplement_three_fault_injection.rs:292-298`（用例注释）、`:300`（函数名）、`:435-444`（断言） | ✓ | 逐字相同 |
| K6 Fact 7：`_m2-supp3-item4-code-r1-body.md:18`（快档实测计数） | ✓ | 逐字相同（与第一节交叉核对的背景材料数字一致） |
| K6 Fact 8：`checks-owed.md:336`（C381 整行） | ✓ | 内容匹配；同 C378 一样，当前行号恰好与引用相符（按内容核对，非只信行号） |
| K6 Fact 9：`c381-r3-main-verification.md:19,37,42,61` | ✓ | 四处逐字核对全部相符 |
| K6 Fact 3 唯一断言：`second_transaction_supplement_three_fault_injection.rs:176-181` | ✓ | 逐字相同 |

### 4.1 四份样本输出的机械性质

```
$ wc -w research/prompts/m2-supp3-item4-code-r1-local-attack-k3-output-s1.md
234 ...s1.md
$ wc -w .../k3-output-s2.md
238 ...s2.md
$ wc -w .../k6-output-s1.md
231 ...s1.md
$ wc -w .../k6-output-s2.md
253 ...s2.md
```
与运行记录（`-local-attack-runlog.md`）里登记的词数逐一相同。**✓**

四份样本通读：无 `**`、`*`、反引号、竖线表格等 markdown 强调（提示明令禁止），未见复读、成对标记落单、
拼接词等字词损坏迹象；每份三道 judgment 均给出 verdict / justification / this would be refuted by 三项。**✓**

小计：本节共核 22 处（19 条转述定位 + 3 项机械性质核对），✓ 22 处，✗ 0 处。

## 五、跨腿一致性交叉核（额外做的，不计入上面三节的计数）

- Sonnet 的 K5 结论「`mkfs`（15 次）与 `warm_up`（24 次）今天全仓零覆盖」与 Opus 的 K1 部分未直接冲突
  （Opus 没有判 K5，两者攻击面按分工不重叠，符合派发提示「两条攻方腿的攻击面不重叠」的要求——
  Sonnet 不是攻方，但同样没有与 Opus 的 K1/K2 结论冲突）。
- Opus 报告 2.4 节与 K3 提示 Fact 1 都独立引用了 `milestone/02-second-txn.md:429` 同一句里程碑原文，
  两处转述文字不同但都与该行原文逐字相符，未见互相矛盾。

## 六、总计

| 腿 | 核了几处 | ✓ | ✗ |
|---|---|---|---|
| 云端正推（Sonnet，K4/K5） | 24 | 18 | 6 |
| 云端攻方（Opus，K1/K2） | 22 | 20 | 2 |
| 本地攻方（K3/K6 转述核对表 + 样本机械性质） | 22 | 22 | 0 |
| **合计** | **68** | **60** | **8** |

✗ 的 8 处逐条复述（细节见上面各节表格）：

1. Sonnet：`fault_injection.rs:1236-1238` 应为 `1237-1239`（行号偏一）。
2. Sonnet：正文两处将函数名 `the_fault_surfaced_as_an_error` 转录成 `the_fault_surfaced_as_error`（少一个 `an`；行号范围本身对）。
3. Sonnet：`grep -n "make_filesystem(&parameters" ...` 命令重跑后，真实命中列表与报告列出的「9 处调用」不同
   （报告列了 `first_transaction_step_one_mkfs.rs:317`，真实命中是 `history.rs:832`；317 行本身存在且内容属实，
   但不是这条命令的输出）。
4. Sonnet：`mount.rs:849` 引用的回卷注释实际不在该行（该行是无关的 `MountError` 成员），真实位置是 `mount.rs:867`。
5. Sonnet：`history.rs:492` 被引作 `PublishFirstFile` 权重「95」的出处，实际该行是 `CloseAndMountWritable, 95`；
   全文件里 `PublishFirstFile` 从未取值 95。
6. Sonnet：`make_filesystem.rs:57-82`/「79 行」引用 `BlockDevice(_)` 成员，真实行号是 84，且超出报告给的行区间。
7. Opus：自我纠错段落里「49 次 panic 全在 `history.rs:832`」，产物自己打印的位置是 `history.rs:833`（同一条跨行语句内，偏一行）。
8. Opus：「K2-c 两条单种子各复现过」，指路的 `probe-k2-single-seeds.txt` 实际只含种子 133 一条的单独复现，
   种子 119（I-3.1）不在该产物里；经我独立重跑证实该结论本身真实，缺口在于**证据文件不完整**，不是结论本身错误。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身，只核引用、产物与复跑，判决与是否采纳由主 agent 定。
- 没有重新跑 Opus 报告里除「单一系统配置槽」测试与「种子 119 单独复现」之外的其余复跑步骤
  （`baseline-fast.txt`、`control-random-history-with-probe.txt`、`probe-k2-continue.txt`、
  `probe-k2-suspend-model.txt`、`probe-k1-schedule.txt`、`probe-k1-dead-device.txt` 六项只核对了产物内文字
  与报告引文是否一致，没有逐个重新执行——这些跑一次要构建整个 workspace 并跑随机历史，
  重跑一次即可复核多项数字，我只挑了主 agent 点名的 K2-b/K2-c 两格与种子 119 缺口做了实跑复核，
  其余六项按「产物内容与报告引用逐字比对」处理，未重新触发一次新的 `cargo test`）。
- 没有编译整个仓库做全量测试、没有跑 `check.sh`、没有跑门禁全量（按定义不归核查员）。
- 没有判断 D22 已定项 8「系统配置每盘放一份」与 C79 那条账本身是不是已经被后续任何决策修订过
  （只核了 Opus 引用的那两处原文逐字对不对，没有做「这条决策今天是不是还生效」这类推理性核查）。
- 没有核 Sonnet 报告第三节「反例一 / 反例二」与「判定一览」表格本身的推理是否成立——那是推论，不归我核；
  只核了它引用的每一处代码行号、kb 引文与命令输出。
- 没有核 Opus 报告第四节「改法甲/乙」的代价估算是否准确（挂钟时间那一格 Opus 自己已标注「不可靠，主 agent 要重量」，
  按报告自己的免责声明处理，未重新计时复核）。
- K3/K6 提示本身按规则不写文件名与行号，我没有另外重新审阅两份英文提示 214/217 行全文逐句是否有翻译腔或用词歧义
  （那是「转述准不准」的更细粒度核查，翻译核对表已经把关键 fact 的定位与首稿改动登记清楚，抽样核对之外没有逐句通读两份提示原文）。
