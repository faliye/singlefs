# C794 第一轮正推腿报告（2026-10-15）

腿：云端正推（Sonnet）。背景材料 `research/prompts/_c794-r1-background.md`，正文 `research/prompts/_c794-r1-body.md`。
冻结副本 `/home/fy5090/.local/share/singlefs/work/c794-r1/repo/`。模型目录 `research/prompts/c794-r1-sonnet-model/`。
本报告里「推的」是读代码或读条款推出来的、没造镜像量；「量的」后面都跟模型输出行（`research/prompts/c794-r1-sonnet-model/out/` 下的文件与行号）。

## 〇、结论

1. **四个候选里站得住的有两个：保持现状（不动代码，写明怎么认这种红）、盘上隔离位（动格式，消掉这种红）。一律豁免、曾被引用旁证两个站不住。**
2. 站不住的原因都是量出来的，不是推的：瞬时隔离的镜像与写者漏释放的镜像**逐字节相同**（0 个扇区不同），所以任何只读镜像的 checker 规则对这两种镜像只能给同一个判定。要么两种都红（保持现状），要么两种都绿（一律豁免：真泄漏漏检，且 checker 返回的全部不变量里没有别的接得住），要么靠一个会过期的旁证半路换色（曾被引用旁证：根环转过那一圈以后红回来，漏释放那一种在这之前一直是绿）。只有往盘上多记一位，才能让 checker 把两种分开。
3. 盘上隔离位动用户 2026-09-25 定的「核出坏的才豁免」那一句（豁免的依据从 checker 自己读出坏，换成写者释放核那一刻记下的隔离位），**必须单列交用户**。
4. 两个候选都站得住，按正文「五、跑前条款」最后一行，主 agent 要写岔路单；代价数在 H4。我不替主 agent采纳。

## 一、前提核对

- 冻结副本的 `crates/` 与主工作区此刻的 `crates/` 逐文件相同（量的）：

```
$ diff -rq crates /home/fy5090/.local/share/singlefs/work/c794-r1/repo/crates | wc -l
0
```

  所以下面引的 `crates/...` 行号对主工作区与冻结副本都成立。
- 条款原文（引用写 kb 文件自己的行号；这几行较长，下面引的是其中完整的句子）：
  - 「释放之前按位置项读盘核校验和，读盘本身失败或核出对不上，都先重读一次，两次都读不出或都对不上才按对不上处置」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）
  - 「任一份核出对不上，这个单元在每块盘上的分配记录都留在」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）。后文接「已分配」，紧跟的括注与「落点」句同在这一行：「落盘即跨重挂，准入里照已分配算、不另进式子」（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:113`）
  - 「隔离的记录：已分配而没有任何根引用的记录，按单元豁免」（`crates/singlefs-checker/invariants/05-空间记账.md:18`），同一行后文「每一份都读得出且对得上，照旧判违例（用户 2026-09-25 定」（`crates/singlefs-checker/invariants/05-空间记账.md:18`）接「核出坏的才豁免」（`crates/singlefs-checker/invariants/05-空间记账.md:18`）。I-3.1 那一行也写了同一句（`crates/singlefs-checker/invariants/05-空间记账.md:8`）。
- 实现今天的样子（量的 / 读的）：
  - 释放前读盘核：`copies_failing_the_release_checksum_check`（`crates/singlefs-core/src/transaction.rs:3651`）逐份读，注释写明「// 第一次读不出、或读出来核出对不上，都先重读一次」（`crates/singlefs-core/src/transaction.rs:3700`）。
  - 隔离的做法：`release_leaving_the_record_allocated_on`（`crates/singlefs-core/src/allocator.rs:1858`）里「if devices_whose_record_stays_allocated.contains(&device.device) {」（`crates/singlefs-core/src/allocator.rs:1865`）那几块盘的记录一个字节不动。盘上没有任何别的东西记「被隔离过」。
  - checker 豁免：`quarantined_slots_exempted_per_device`（`crates/singlefs-checker/src/walk.rs:7055`）判豁免的就是「let some_copy_fails = devices_of_the_unit」（`crates/singlefs-checker/src/walk.rs:7112`）那一句；没有位置项可比时退回自证，「None => check_unit(&unit).is_ok(),」（`crates/singlefs-checker/src/walk.rs:7109`）。
  - I-3.11 的判法：「matches!((allocated, deferred), (Some(allocated), Some(deferred)) if deferred.checked_add(referenced_by_the_newest_root) == Some(allocated)),」（`crates/singlefs-checker/src/walk.rs:13181`）；I-3.1 的判法：「judgements.judge("I-3.1", allocated == Some(walked), || {」（`crates/singlefs-checker/src/walk.rs:13144`）。**两条都吃同一份隔离豁免**，所以 C794 的红不只 I-3.11，见 H1。

## 二、H1 现象与可达性

### 二·一 哪些路径做释放前的读盘核（读的）

| 路径 | 读盘核在哪 | 能到「两次坏读之后单元完好」吗 |
|---|---|---|
| 发布的释放（覆盖写、第一个文件之后的各版、清扫、删除批、抬 F 的空发布，都经 `publish_version`） | 「(Some(previous_version), ReleaseChecksumCheck::ReadEveryReplacedCopyBeforeReleasingIt) => {」（`crates/singlefs-core/src/transaction.rs:6783`），随后「copies_failing_the_release_checksum_check(」（`crates/singlefs-core/src/transaction.rs:6784`）；清扫走「SweepPublishPlacements::TakenFromTheAllocatorOfTheMount => publish_version(」（`crates/singlefs-core/src/instance_table_sweep.rs:650`） | 能。覆盖写与抬 F 两处我造出来了（下面量的）；清扫、删除批是读代码推的，没造 |
| 管理员回退（挂着时的向前发布） | 回退释放现行那一版的用户可见单元时「copies_of_a_released_unit_failing_the_release_checksum_check(」（`crates/singlefs-core/src/mount.rs:8255`），随后「allocator.release_leaving_the_record_allocated_on(」（`crates/singlefs-core/src/mount.rs:8307`） | 能（读代码推的；回退目标引用隔离单元那一半 C572 已判，不在射程） |
| 可写挂载取号之前的预演 | 「ReleaseChecksumCheck::SkippedByTheDryRunBeforeAcquisition,」（`crates/singlefs-core/src/mount.rs:4855`）不读盘 | 不读盘，没有这一形 |

写者认「隔离单元」的方式本身也是推断：回退那一支只凭「现行账里仍分配而现行那一版不引用」认隔离单元，注释写「只有释放之前读盘核出一份对不上、两份都留在已分配」（`crates/singlefs-core/src/mount.rs:7728`）——它相信没有第二种来路，而 checker 要查的正是「有没有第二种来路」。

### 二·二 造出来了（量的）

造法：`BurstAtOneOffset`（连着坏两次的读，之后放行，盘上字节不动）包住两块盘，A 的数据单元那一槽的读先坏两次，覆盖写 B：
`let b = overwrite(&mut pool, &first, 5, 1, Some((a_slot, 2, form))).expect("B");`（`research/prompts/c794-r1-sonnet-model/c794_model.rs:243`），紧跟断言两盘那一份都被隔离。两种坏法（读报块设备错、读回翻一个字节）都造了。

池级 checker 的结果（`keep_current` 是今天的 checker，没改一个字）：

- 发布 B 之后：「C794RESULT mirror=transient_two_read_errors stage=txg4_after_B candidate=keep_current red=["I-3.11"] I-3.1=holds I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:1`）
- 一直不抬 F、只靠根环轮转：发布 A 的根还在环里时只红 I-3.11，到 A 的根被环覆写那一版 I-3.1 也红：「C794RESULT mirror=transient_two_read_errors stage=txg26 candidate=keep_current red=["I-3.11"] I-3.1=holds I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:13`）、「C794RESULT mirror=transient_two_read_errors stage=txg27 candidate=keep_current red=["I-3.1", "I-3.11"] I-3.1=violated I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:17`）。根环 24 槽，A 的 txg 是 3，3 + 24 = 27。
- 一直红到我停的那一版（txg 44）：「C794RESULT mirror=transient_two_read_errors stage=txg44 candidate=keep_current red=["I-3.1", "I-3.11"] I-3.1=violated I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:45`）。隔离的记录落盘即跨重挂：「6. 隔离之后重挂：那一槽的记录从盘上读回来仍是已分配，能回收的都回收了它也不回空闲池、不被再发出去；」（`crates/singlefs-harness/tests/release_checksum_quarantine.rs:15`）。所以这种红不是一阵子的事：这个池一旦碰上一次瞬时隔离，只要写者不再去动那两条记录，红就一直在（推的：`crates/` 里没有把已隔离的记录再释放的路径，下面「没做什么」里的 grep）。

### 二·三 抬 F 那一站也造出来了（量的）

抬 F 的空发布在读上一版记账树那一单元时连坏两次，该单元被隔离：「C794RESULT mirror=transient_inside_floor_raise accounting_slot=50298 publishes=3 quarantined_copies_in_the_publishes=2」（`research/prompts/c794-r1-sonnet-model/out/c794_transient_inside_the_floor_raise_publishes.txt:1`），checker 判「C794RESULT mirror=transient_inside_floor_raise stage=after_floor_raise candidate=keep_current red=["I-3.11"] I-3.1=holds I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_transient_inside_the_floor_raise_publishes.txt:2`）。所以被隔离的不只数据单元，释放核的单元是每次发布换下的十二个角色，哪个都可能。

### 二·四 取样里撞得上吗（读的 / 推的）

- 随机故障注入：每个注入点只坏一次调用。随机那一路摆的是「FaultSchedule::the_nth_call_across_the_pool(drawn.fault, drawn.call_ordinal),」（`crates/singlefs-harness/src/fault_injection.rs:2938`），它的含义是「/// 只在第 n 次命中的调用上注入一次，之后放行（随机注入那一路用这一种：一段历史一个注入点）。」（`crates/singlefs-harness/src/fault_injection.rs:463`）。释放核第一读坏、重读就是下一次调用，对得上，不隔离；快档注释也写了「随机注入不摆这一形（一次注入只坏一次调用）」（`crates/singlefs-harness/tests/fault_injection_fast_tier.rs:186`）。**所以随机取样撞不上这一形**（读的，不是量的）。
- 崩溃点重放：重放的是写表，读不坏；一个流里要有被隔离的发布，只能是录制那一遍的写者撞过连坏两次的读。checker 档的隔离豁免用例是手改镜像造的（`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs:14865` 起的注释写「txg 7 那一版改成「写者把放掉的第二个单元隔离了」」），不是撞出来的（推的）。
- 手写的坏读包装能撞上，并且已经钉了：用例 13「/// 用例 13：盘 0 上那一份每一读都交回坏了一个字节的内容（坏读一直在，盘上的字节是好的）」（`crates/singlefs-harness/tests/release_checksum_quarantine.rs:614`）；checker 档用例把「改回对得上」的镜像钉成红：「"那两份改回对得上",」（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs:157`）后面跟「.and_violated("I-3.11"),」（`crates/singlefs-harness/tests/common_images_for_the_checker_tier/release_checksum_quarantine.rs:161`）。**也就是说今天这个假红被一条用例钉成了期望。**
- 真设备上「连着两次读失败、之后单元完好」会不会出现：没量（推的：控制器短暂超时一类可能，本机没有这种设备）。

跑前条款第一行（H1 造不出 ⇒ 降为只在故障注入下可达）的结论：我造出来了，经公开入口 `publish_overwrite`、`raise_rollback_floor` 加一个包住 `BlockDevice` 的手写故障设备，所以**不触发降级**；射程要写明：现有的随机取样不摆这一形，只有手写包装撞得上。

## 三、H2 候选处置，与四种镜像上各判成什么

### 三·一 四个候选写成对 `crates/` 的改动

| 候选 | 改哪里 | 在模型里怎么开 |
|---|---|---|
| 保持现状 | `crates/` 不动；只改文字：I-3.11 那一行与 C794 那一行写明这种红的认法（红的差额 = 写者报出的隔离份数 × 单元槽数，写者那一侧的数是「pub quarantined_after_release_checksum_mismatch:」（`crates/singlefs-core/src/transaction.rs:2905`）） | `keep_current` |
| 一律豁免 | 「let some_copy_fails = devices_of_the_unit」（`crates/singlefs-checker/src/walk.rs:7112`）改成恒真，GPU 内核同改（`fn c_quarantine_exempted(newest_base: u32, reference_limit: u32, fsid_low: vec2<u32>) {`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:1655`））。这一改就是已登记的变异：「实二二三 e：已分配而没有根引用的记录不读那一份、一律豁免（读得出且对得上的也豁免）」（`crates/mutations.tsv:594`） | `exempt_every_unreferenced_record`（补丁里的「1 => true,」（`research/prompts/c794-r1-sonnet-model/patches/checker-candidates.patch:40`）） |
| 盘上隔离位 | 分配记录的值里借一位：现在跨度段「最高位借作已释放标志（0 = 仍分配、1 = 已释放），不占表达跨度值的位」（`.claude/kb/decisions/03-空间分配.md:129`），同一行写着跨度「第一版恒为 1 或 2」（`.claude/kb/decisions/03-空间分配.md:129`），次高位空着。`release_leaving_the_record_allocated_on`（`crates/singlefs-core/src/allocator.rs:1858`）给不改的那几条记录置位；checker 的豁免条件换成「该位在册且没有根引用」；GPU 内核同改；回退那支用位代替推断（`crates/singlefs-core/src/mount.rs:7728`） | `on_disk_quarantine_mark`（模型里位由写者的隔离清单代替，标记的槽号走 `C794_MARKED_SLOTS`；真的盘上位没有实现，代价靠数触点，H4） |
| 曾被引用旁证 | 豁免条件换成「有任何一条走过的或根环里没走过的根的位置项指着这个单元」，即 `LocationEntryChecksumsForQuarantine::at(...)` 有值（`crates/singlefs-checker/src/walk.rs:6878`）。别的旁证盘上没有：journal 点名项记的是写下去的单元，「一项一个单元：位置条目 14 × 2 + 类型 1 + 出生树 8 + 出生 txg 8 + key 尾段 10 + flags 1」（`.claude/kb/decisions/23-journal的角色与格式.md:34`），不记释放 | `once_referenced_evidence` |

### 三·二 镜像造法

全在冻结副本的拷贝里、经 `build_pool`（新池新建文件）+ 覆盖写造，每个镜像都先过 `healthy`（没有任何注入）：四个候选、每一步都一条不红（「C794RESULT mirror=healthy stage=txg4_after_B candidate=keep_current red=[] I-3.1=holds I-3.11=holds I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_without_a_floor_raise.txt:1`））。

- **瞬时隔离，两份完好**：A 的数据单元那一槽，两块盘的读各先连坏两次（`mirror=transient_two_read_errors` 与 `transient_two_corrupted_reads`），发布 B 隔离两盘那一份，之后盘上字节完好。
- **持久坏，一份坏**：盘 0 上那一份盘上字节翻一个字节（`persistent_one_copy_bad`），盘 1 那一份好。
- **活单元记成已释放（Z10）**：写者把最新根还引用着的数据单元记成已释放（`PoolAllocator::release`），再发空发布（`z10`）；另造一份同时有瞬时隔离的（`z10_plus_transient`）。
- **漏释放，单元悬空**：写者对 A 的数据单元那一槽的释放整个跳过（`writer_forgets_the_release`）。公开入口造不出这一形，所以补丁里加了一个只供模型用的开关，「if placement.slot.0 == C794_SKIP_RELEASE_SLOT」（`research/prompts/c794-r1-sonnet-model/patches/writer-skip-release.patch:17`），开关不设时发布路径一个字节都不变。
- **孤儿**：一个从没被任何根引用过的已分配单元，槽里没写（`orphan_never_referenced`）或写着一个完整的单元（`orphan_never_referenced_written_valid`，分配代取与那个单元头里的诞生代号相同，免得 I-3.10 因模型自己的拷贝而红）。

### 三·三 一张表（量的；每格后面是依据行）

红写 checker 返回的全部不变量里判红的那几条；「绿」是一条都不红。

| 镜像 | 保持现状 | 一律豁免 | 盘上隔离位 | 曾被引用旁证 |
|---|---|---|---|---|
| 瞬时隔离，两份完好 | 红：发布 B 起红 I-3.11（`…txg4_after_B…`），A 的根被环覆写（txg 27）或抬 F 之后加红 I-3.1 | 绿 | 绿 | txg 26 之前绿，txg 27 起红 I-3.1 + I-3.11 |
| 持久坏，一份坏（抬 F 之后） | 绿 | 绿 | 绿 | 绿 |
| 活单元记成已释放 | 红 I-3.9 + I-3.11 | 红 I-3.9 + I-3.11 | 红 I-3.9 + I-3.11 | 红 I-3.9 + I-3.11 |
| 漏释放，单元悬空（曾被引用过） | 红：I-3.11，txg 27 起加 I-3.1 | **绿，全部不变量一条不红** | 红：I-3.11，txg 27 起加 I-3.1 | txg 26 之前绿，txg 27 起红 |
| 孤儿，槽里写着完整单元 | 红 I-3.1 + I-3.11 | 绿 | 红 | 红 |
| 孤儿，槽里没写 | **绿**（自证退路把一块零当成「坏的」豁免掉） | 绿 | 红 | 红 |

依据行（文件都在 `research/prompts/c794-r1-sonnet-model/out/` 下）：

- 瞬时隔离：`c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt` 第 1–4 行（txg 4 四个候选）、第 13–16 行（txg 26）、第 17–20 行（txg 27）、第 45–48 行（txg 44）。txg 4：保持现状红 I-3.11，其余三个绿；txg 27：保持现状红 I-3.1 + I-3.11，曾被引用旁证也红 I-3.1 + I-3.11，另两个绿。
- 漏释放：同一份文件第 49–52 行（txg 4：「C794RESULT mirror=writer_forgets_the_release stage=txg4_after_B candidate=exempt_every_unreferenced_record red=[] I-3.1=holds I-3.11=holds I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:50`），盘上隔离位在 txg 4 红：「C794RESULT mirror=writer_forgets_the_release stage=txg4_after_B candidate=on_disk_quarantine_mark red=["I-3.11"] I-3.1=holds I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:51`））、第 61–68 行（txg 26 与 27）。
- 持久坏：`c794_with_a_floor_raise_to_4.txt` 第 21–24 行四个候选全绿；不抬 F 时 A 的根还在候选集，真坏的那一份把 I-2.1、I-4.8、I-7.4 打红，四个候选一样（`c794_without_a_floor_raise.txt` 第 111–114 行），这三条是真坏，不归 C794。
- Z10：`c794_live_unit_recorded_as_released.txt` 第 2、4、5、6 行四个候选都红 I-3.9 + I-3.11。**同时有一份瞬时隔离时**，保持现状的 I-3.11 反而绿：「C794RESULT mirror=z10_plus_transient stage=txg7_after_empty_publishes candidate=keep_current red=["I-3.1", "I-3.9"] I-3.1=violated I-3.11=holds I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_live_unit_recorded_as_released.txt:8`）——两边各差同样大小的单元，和式的等号被抵消（推的；差额大小相同是这个造法的巧合，但说明 I-3.11 在有未解释的隔离红时检不出 Z10）；另三个候选把隔离的那一边消掉以后 I-3.11 红回来（第 10、11、12 行）。
- 孤儿：`c794_with_a_floor_raise_to_4.txt` 第 34–37 行（没写）、第 40、43–45 行（写着完整单元）。

### 三·四 逐字节相同（量的）：为什么只有盘上多一位才分得开

瞬时隔离（两种坏读）与漏释放，造出来的镜像与整池逐扇区比，0 个扇区不同：

```
C794RESULT same_bytes stage=txg4_after_B left=transient_two_read_errors right=writer_forgets_the_release identical=true differing_sectors=0
C794RESULT same_bytes stage=txg7_floor_raised_to_4 left=transient_two_read_errors right=writer_forgets_the_release identical=true differing_sectors=0
```

（原样行在 `research/prompts/c794-r1-sonnet-model/out/c794_transient_quarantine_and_forgotten_release_leave_the_same_bytes.txt` 第 3、4 行；两种坏读互比同样 `identical=true`，第 1、2 行。）

范围写清：量的是数据单元那一站、两盘各坏两读、造出来的两张镜像；抬 F 那一站（记账树单元被隔离）只量了 checker 判成什么（`out/c794_transient_inside_the_floor_raise_publishes.txt`），没造配对的漏释放去逐字节比。推广到别的角色靠读代码：隔离只是在「if devices_whose_record_stays_allocated.contains(&device.device) {」（`crates/singlefs-core/src/allocator.rs:1865`）里把那几块盘跳过、不改它们的记录，隔离清单只在内存里，「从盘上重建的版本不是这个进程发布的，是空的」（`crates/singlefs-core/src/transaction.rs:2904` 起的字段注释），别处没有写盘的动作（推的）。

推论：**只读镜像的 checker 不可能在这两种镜像上给不同的判定**。所以：
- 想让瞬时隔离不红、同时漏释放仍红——只能让盘上多一个两者不同的字节（盘上隔离位）。
- 不多记字节的候选，对这两种镜像只能同判：保持现状同判红，一律豁免同判绿，曾被引用旁证同判（绿，直到旁证过期，随后同判红）。

## 四、H3 判别力

| 候选 | 丢掉哪一种检出 | 别的不变量接得住吗（量的） | 动不动用户 2026-09-25 的「核出坏的才豁免」 |
|---|---|---|---|
| 保持现状 | 相对今天什么都不丢。但今天已有一处盲：从没写过内容的孤儿，因为没有位置项可比、退回自证时把一块零当成「读出来是坏的」而豁免（「None => check_unit(&unit).is_ok(),」（`crates/singlefs-checker/src/walk.rs:7109`）），模型里孤儿槽没写时四种判定只有两个红：「C794RESULT mirror=orphan_never_referenced stage=txg7_floor_raised_to_4 candidate=keep_current red=[] I-3.1=holds I-3.11=holds I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_with_a_floor_raise_to_4.txt:34`）。这一盲不在 C794 射程，只记下 | 不适用 | 不动 |
| 一律豁免 | 漏释放（曾被引用过的悬空单元）与孤儿两类真泄漏：I-3.1 与 I-3.11 是这两类仅有的检出者 | **接不住**：漏释放那一形，checker 返回的全部不变量一条不红，「C794RESULT mirror=writer_forgets_the_release stage=txg27 candidate=exempt_every_unreferenced_record red=[] I-3.1=holds I-3.11=holds I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:66`），txg 4 到 12 每一步、txg 20 到 44 抽到的几步都一样（`c794_without_a_floor_raise.txt` 与同一份文件）；孤儿写着完整单元那一形也全绿（`c794_with_a_floor_raise_to_4.txt` 第 43 行） | 动（整句撤销）；而且这一改正是已登记要被抓住的变异：「实二二三 e：已分配而没有根引用的记录不读那一份、一律豁免（读得出且对得上的也豁免）」（`crates/mutations.tsv:594`） |
| 盘上隔离位 | 写者自己把真泄漏也置了位（谎报）那一种；别的不变量接不住它（同一个字节同一个判，推的）。位在最新根还引用着的记录上时该不豁免（推的，没造） | 漏释放（位没置）照红，「C794RESULT mirror=writer_forgets_the_release stage=txg4_after_B candidate=on_disk_quarantine_mark red=["I-3.11"] I-3.1=holds I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:51`）；孤儿也照红（`c794_with_a_floor_raise_to_4.txt` 第 36、44 行） | **动**：豁免的依据从 checker 自己读出坏，换成写者释放核那一刻记下的位。持久坏那一形仍然绿（第 21–24 行），但依据变了。单列交用户 |
| 曾被引用旁证 | 漏释放在 txg 26 之前（根环还留着引用它的根）一直不红；孤儿照红 | 不接：窗口内全不变量不红，「C794RESULT mirror=writer_forgets_the_release stage=txg26 candidate=once_referenced_evidence red=[] I-3.1=holds I-3.11=holds I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:64`）。窗口外两种镜像一同红回来：瞬时隔离 txg 27 起「C794RESULT mirror=transient_two_read_errors stage=txg27 candidate=once_referenced_evidence red=["I-3.1", "I-3.11"] I-3.1=violated I-3.11=violated I-5.2=holds」（`research/prompts/c794-r1-sonnet-model/out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt:20`） | 动（豁免不再要求 checker 读出坏）；而且不管用：假红只是晚到 |

结论：跑前条款第二行（候选被造出漏判一种真泄漏、而没有别的不变量接得住 ⇒ 出局）打中**一律豁免**与**曾被引用旁证**，两条都有可复现的镜像（`research/prompts/c794-r1-sonnet-model/` 里 `writer_forgets_the_release`）。

## 五、H4 代价

数的口径：「文件数 / 处数」是 grep 数出来的触点上界（命令和输出在下面），**实改多少处是推的**，没动手改。

```
$ for g in crates/singlefs-core/src crates/singlefs-checker/src crates/singlefs-harness/src crates/singlefs-checker-tier/src; do echo "$g $(grep -rl is_released $g | wc -l) files $(grep -r is_released $g | wc -l) hits"; done
crates/singlefs-core/src 6 files 41 hits
crates/singlefs-checker/src 1 files 16 hits
crates/singlefs-harness/src 5 files 18 hits
crates/singlefs-checker-tier/src 4 files 35 hits
$ grep -c 'sw(a + 6u)' crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl
5
```

| 候选 | 要改的条款 | 格式 | checker 与 GPU 判器 | 写者 | 用例与变异表 |
|---|---|---|---|---|---|
| 保持现状 | I-3.11 那一行（`crates/singlefs-checker/invariants/05-空间记账.md:18`）补一句认法；`.claude/kb/checks-owed.md` C794 行验收改写（现行验收「池级 checker 不许判 I-3.11 红」在这一候选下不可能满足）。D19 已定项 5、D5 已定项 4 不动 | 0 | 0 处 | 0 处 | 0；现有钉子（`common_images_for_the_checker_tier/release_checksum_quarantine.rs:161`、`:196`）就是这个期望 |
| 一律豁免 | I-3.1（`05-空间记账.md:8`）与 I-3.11（`05-空间记账.md:18`）两行的豁免措辞；撤销用户 2026-09-25 定案 | 0 | `walk.rs:7112` 一处 + 扫描内核 `c_quarantine_exempted`（`crash_judge_checker_scan.wgsl:1655`）一处 | 0 | 2 个场景钉子（`:161`、`:196`）反过来；GPU 对拍流 20 行注释点名隔离的那几条要重核；`crates/mutations.tsv` 里 593、594、595、764、765 五行要重写（594 那一行变成现行实现） |
| 盘上隔离位 | D3 已定项 7（`03-空间分配.md:129`）借一位的那一句；D19 已定项 5（`19-…md:113`）；I-3.1、I-3.11 两行；C794 行；决策变更史一条；`format-evolution.md` 硬约束：「改格式必须同步更新 `crates/singlefs-checker/invariants/` 和 checker。三者不同步的 commit 一律不收。」（`.claude/rules/format-evolution.md:14`）。动用户 2026-09-25 定案 | 值宽不变（跨度段 2 字节里取次高位，现在读法是 `span_field & !ALLOCATION_RECORD_RELEASED_FLAG`，次高位会被读成跨度的一部分，所以读写两边的掩码都要改）；第一个外部用户之前格式是软的（`.claude/rules/format-evolution.md:7`），代价只算这一次要同步的东西 | 解码 `walk.rs:4505`、`:4507` 两处，豁免条件 `walk.rs:7112` 一处；扫描内核解码 + `c_quarantine_exempted`；扫描内核读已释放位的点 5 处要逐处看是否要带新位 | 编码 `allocator.rs:51`–`:56`、解码 `allocator.rs:93`–`:95`；`release_leaving_the_record_allocated_on`（`allocator.rs:1858`）置位；记录被正常释放或复活时清位（`allocator.rs:1882`、`:1926` 一带）；回退那支（`mount.rs:7728`）可用位代替推断；`recovery.rs` 重建分配器时带位。`is_released` 触点上界 core 6 文件 41 处 | 2 个场景钉子翻过来 + 新造「位被清掉的同一份镜像必须红」；GPU 对拍流同改；五行变异重写 + 新增「忽略位」「位置到活单元上」两行 |
| 曾被引用旁证 | I-3.1、I-3.11 两行；动用户定案 | 0 | `walk.rs:7112` 一处 + 内核一处 | 0 | 同一律豁免；而且没有任何用例能同时满足「瞬时隔离不红」与「根环转一圈之后仍不红」 |

H4 里「文件数」的一项没数的东西要写明：GPU 内核 `crash_judge_checker.wgsl` 有没有另一处读隔离豁免，我只查到扫描内核有 `c_quarantine_exempted` 一处（`crash_judge_checker_scan.wgsl:1655`），`crash_judge_checker.wgsl` 的 4045 行注释提到「第三批判隔离的记录要」，实改处没逐处读（没做）。

## 六、H5 会红的用例与判别力自证（样本形状）

下面是**选定后**要造的样本；模型里已经量过的部分标明。

| 处置 | 样本 | 期望 | 自证（改回去必须翻色） |
|---|---|---|---|
| 盘上隔离位 | 瞬时隔离镜像：`BurstAtOneOffset` 连坏两读 + 覆盖写 B（`research/prompts/c794-r1-sonnet-model/c794_model.rs:243`）。用现有的手写故障设备造，不手改盘上字节 | I-3.1、I-3.11 成立（模型里：`out/…ring….txt` 第 3 行 `on_disk_quarantine_mark` 绿，`keep_current` 第 1 行红） | checker 的豁免改回「读盘判」→ 这份必须红 |
| 盘上隔离位 | 真泄漏镜像：同一份把位清掉。字节相同的结论保证这份与「写者漏释放」是同一份镜像 | I-3.11 红（txg 27 起加 I-3.1）（模型里 `…txt` 第 51、67 行） | 豁免改成一律豁免 → 这份必须由红变绿（模型里第 50 行绿） |
| 盘上隔离位 | 位置在最新根还引用着的记录上 | 不豁免（推的，没造） | 豁免条件去掉「没有根引用」→ 这份必须翻色 |
| 保持现状 | 同一份瞬时隔离镜像 | **判红**——C794 行现行验收「不许判 I-3.11 红」在这一处置下改成「判红且文字写明」，验收文字归主 agent 与用户 | 已有：两个场景钉子（`common_images_for_the_checker_tier/release_checksum_quarantine.rs:157`、`:161`） |

跑前条款第三行（留下的候选动用户 2026-09-25 定案 ⇒ 交用户）：**盘上隔离位**打中。保持现状不打中。

## 七、判定一览

「站得住 / 站不住 / 判不了」；「站得住」不等于主 agent 采纳。

| 格 | 判定 | 一句话 |
|---|---|---|
| H1 现象与可达性 | 站得住（现象成立） | 经 `publish_overwrite`、`raise_rollback_floor` 加手写故障设备造出；I-3.11 从发布 B 起红，I-3.1 在 A 的根出环或抬 F 后加红，之后一直红；随机故障注入摆不出这一形（一次注入只坏一次调用），手写包装撞得上 |
| H2 保持现状 | 站得住 | 四种镜像上的判定与今天逐条相同；代价是瞬时隔离的池永久红 |
| H2 一律豁免 | 站不住 | 漏释放、孤儿两种真泄漏全绿，全部不变量一条不红 |
| H2 盘上隔离位 | 站得住（动用户定案，单列交用户） | 瞬时隔离绿、持久坏绿、漏释放红、孤儿红、Z10 红 |
| H2 曾被引用旁证 | 站不住 | 窗口内漏释放绿，根环转过 24 槽后瞬时隔离红回来 |
| H3 保持现状 | 站得住 | 不丢检出；记下一处已有的盲（从没写过的孤儿）不归 C794 |
| H3 一律豁免 | 站不住 | 丢掉悬空单元与孤儿两类，没有别的不变量接 |
| H3 盘上隔离位 | 站得住（动用户定案） | 只丢「写者谎报隔离」一种；豁免依据从 checker 读出坏换成写者记的位 |
| H3 曾被引用旁证 | 站不住 | 窗口内丢悬空单元，窗口外假红照旧 |
| H4 保持现状 | 站得住 | crates 0 处，文字 2 处 |
| H4 一律豁免 | 站得住（但无意义） | 代码 2 处 + 5 行变异重写，换来真泄漏漏检 |
| H4 盘上隔离位 | 站得住 | 值宽不变；解码 2 处 + 编码 1 处 + 置位清位若干 + 内核；is_released 触点上界 core 41 处 / checker 16 处 / checker 档源 35 处 / harness 源 18 处 |
| H4 曾被引用旁证 | 站不住 | 与一律豁免同规模，买到的东西更少 |
| H5 盘上隔离位 | 站得住 | 样本形状在上面表里；瞬时隔离与漏释放字节相同，所以位清掉的那份就是漏释放样本 |
| H5 保持现状 | 判不了（要改验收文字） | C794 行现行验收「不许判 I-3.11 红」在这一处置下不能满足，改写验收归主 agent 与用户 |
| H5 一律豁免、曾被引用旁证 | 站不住 | 各有一份真泄漏镜像使它们绿（模型里） |

## 八、什么现象会推翻这些结论

- 「逐字节相同」：任何站点上造出一对（瞬时隔离、漏释放）镜像，整池有一个扇区不同——例如隔离时写者还往盘上别处写了东西。我只在数据单元一站量过（0 个扇区不同）；若别的站点（记账树单元、分配记录树节点）不同，曾被引用旁证之外的「用盘上已有字节分开」的候选重新开。
- 「一律豁免出局」：同一份漏释放镜像上，别的实现了的不变量里有一条判红（我取的是 `check_pool_image` 返回的全部）；或 GPU 判器给出不同的判定（我没跑 GPU，见「没做什么」）。
- 「曾被引用旁证出局」：根环不转（R 不增长）也仍然不红的路径；我量的转环窗口是 24 槽，在 txg 27 翻色。
- 「盘上隔离位站得住」：真的加位以后，某条路径把位置在活单元上、或清位漏了，使 H3 里「只丢写者谎报一种」不成立；这一条我只在模型里用写者隔离清单代替位量过，没改真的编码（`crates/` 一行没改）。
- 「随机取样撞不上」：把随机注入的 `FaultOccurrence` 改成可以连续两次命中同一偏移，或者真设备出现连着两次读失败。
- 「现象永久」：`crates/` 里有一条路径把已隔离、没有映射条目的记录再释放。我查了写者把记录改成已释放的位置，只有一处会走到：
  `crates/singlefs-core/src/allocator.rs:1882` `record.is_released = true;`（同一份文件里还有 `allocator.rs:1926` 的 `record.is_released = false;` 是复活，方向相反），而 `allocator.rs:1858` 的 `release_leaving_the_record_allocated_on` 只收释放判定路径（经映射查到的落点）交来的落点，被隔离的单元已经没有映射条目，到不了（推的；没有造一个把它再放出来的历史去试）。

## 九、没做什么

- 没跑 GPU 判器，没跑 checker 档任何测试，没跑门禁阶段、变异表、崩溃点重放（派发提示：重型测试不跑）。内核要同改写在 H4，只数了触点。
- 没实现真的盘上隔离位：模型里位由写者的隔离清单代替（`C794_MARKED_SLOTS`，补丁在 `research/prompts/c794-r1-sonnet-model/patches/checker-candidates.patch`），所以「盘上隔离位」那一列量的是**判法**，不是**编码**；编码的代价是 H4 数触点，推的。
- 没造的：回退站点、清扫站点、删除批站点上的瞬时隔离；单盘（只一块盘坏读）的变体；位被置在活单元上的样本。
- 没量真设备上连着两次读失败的概率。
- 没拿别的项目的做法当依据。
- 不采纳、不出判决：两个站得住的候选交主 agent 写岔路单；这份报告不替用户的 2026-09-25 定案做主。

## 十、复核与复跑

- 模型目录 `research/prompts/c794-r1-sonnet-model/`：`c794_model.rs`（用例，拷进冻结副本拷贝的 `crates/singlefs-harness/tests/` 下跑）、`patches/`（两块只供模型用的补丁：checker 里的候选选择器，写者里的漏释放开关；开关不设时发布路径逐字节不变）、`tools/rerun.sh`（从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换了 `rerun_steps` 那一段；自证 `bash research/prompts/c794-r1-sonnet-model/tools/rerun.sh --selftest` 通过，查了 5 格）、`out/`（六个用例的原样输出行）。
- 复跑命令（工作目录随意；整条经内存包装，线程取 31；全部用例跑完约 6 分钟，前台超 4 分钟，所以 `C794_TESTS` 可切段）：

```
bash scripts/hosts/run-with-memory-cap.sh 16G bash research/prompts/c794-r1-sonnet-model/tools/rerun.sh --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/c794-r1/repo --work <新的草稿目录>
C794_TESTS="<用例名 …>" C794_TARGET_DIR=<复用的编译目录> 同上
```

- 我自己的复跑（本机，分三段，编译目录复用，每段另开新草稿目录）：六个用例全部 `same_as_model_out=true`、`mismatches=0`：
  `name=compare test=c794_transient_inside_the_floor_raise_publishes lines=6 same_as_model_out=true`、
  `name=compare test=c794_live_unit_recorded_as_released lines=12 same_as_model_out=true`、
  `name=compare test=c794_transient_quarantine_and_forgotten_release_leave_the_same_bytes lines=4 same_as_model_out=true`、
  `name=compare test=c794_with_a_floor_raise_to_4 lines=45 same_as_model_out=true`、
  `name=compare test=c794_without_a_floor_raise lines=259 same_as_model_out=true`、
  `name=compare test=c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit lines=96 same_as_model_out=true`。
- 产物 sha256（`sha256sum` 此刻原样）：

```
c53931bb5a85001e8b973af2bae5edb8d9192013c7a42f0dee9514c518de4a42  out/c794_after_the_root_ring_overwrites_the_root_of_the_quarantined_unit.txt
ecd61c06b58b49e0cab9b3b279a01332c63b395f32aa2ea85a5ce1e4b9c5c034  out/c794_live_unit_recorded_as_released.txt
5d660dc2d66531113379a177ee258f76a9d26d09ae183690248cbfbc3e114c83  out/c794_transient_inside_the_floor_raise_publishes.txt
09ad4a5af2b35a84fd3cf4090f3dfc295fc2451753ed9944d2863d16e6719ee1  out/c794_transient_quarantine_and_forgotten_release_leave_the_same_bytes.txt
e0618dc785d793e61274e0133f4d08a90245e81a5d3536902eaf4f1906374480  out/c794_with_a_floor_raise_to_4.txt
8b0f540e154fd5fafb46cc19f588d26654aac0227d8311a01c984121e21c7a63  out/c794_without_a_floor_raise.txt
aa2e6204f9eba2cf3ff81b00f6ca1237236dfd1fb350da6c5edf7e1240a1a178  c794_model.rs
b33e4b4018dce81a0b412e449ce9b0e3ffb939034b0c3c2ad6b15ae5c9e38116  patches/checker-candidates.patch
d1696d43b92ef03b788d7b7d59b547221b8ee84f9582705f4c266a5ad717e60c  patches/writer-skip-release.patch
845c2bf27a3ccd1a60dca75571d7233cac0b8598e471d2d366fa433f7729fcac  tools/rerun.sh
```

- 负载：开工时 `ps` 看到另一个会话的 `cargo test --release -p singlefs-harness --test fault_injection_fast_tier` 在跑，不是性能测量（没有 qemu-system、vm-bench、e152、fio），照常加 `nice -n 19` 跑；没有等锁。
- 草稿目录里的冻结副本拷贝与编译目录在交回前删掉（删之前 `du -sh` 列过）。
