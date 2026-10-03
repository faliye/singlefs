# m3-verus-land-code-r3 辩方腿（Sonnet）报告（2026-10-01）

<!-- doc-lint:not-numbers H1 H2 H3 H4 H5 H6 J1 J2 J3 J4 J5 S02 S03 S04 S05 S06 S07 S08 S09 S10 S11 S12 K02 K19 K20 G08a -->

读的是第三轮冻结副本 `/tmp/claude-1000/m3-verus-land-code-r3/tree/`（开工先 `sha256sum -c` 对 `research/prompts/m3-verus-land-code-r3-snapshot/crates-sha256.txt`，不「OK」的 0 份）与第二轮冻结副本 `/tmp/claude-1000/m3-verus-land-code-r2/tree/`（改前）；没读主仓 `crates/`，没读禁读清单里的任何文件。要辩护的是第二轮判「打中」的 H5 八处改前的写法、H4-c 没选的「spec 加幽灵读盘日志」、H5-3 没选的「core 直接用核的类型」；同时复核 H1、H3、H6、H2、H4 在今天的代码上还够不够得着，并在冻结副本拷贝上重跑第二轮正推的差分。
模型、脚本、结果摘要在 `research/prompts/m3-verus-land-code-r3-sonnet-model/`（`README.md` 一页说明）。线程 10，编译出来的代码都经 `run-with-memory-cap.sh`（内存上限说明见第七节）。

## 一、判定一览

| 判决条目 | 辩方判定 | 证据 |
|---|---|---|
| H1 改后核与 fcc4d340 等价（「没打中」，58 904 864 个产品输入零差异） | 结局层站得住；「零差异」今天够不着 | 2.2：返回值一处不差，读盘日志 444 399 个输入（0.75%）多一次读；全部由 H5-2 造成，把那半句撤回后归零（2.3）；对判据第一条，字面满足的是「回调调用不同」那个分句 |
| H1 两条新断言产品路径上触发不了 | 站得住 | 5-1 没动那两条，越界族数与第二轮逐个相同（6.6） |
| H2 字面没打中；口子是「读盘次数与次序」 | 字面站得住；口子补上了一半 | 必红表 93 / 93 红（6.1）；补的用例只罩一种候选形状，核头注释那一句比用例宽（5.2） |
| H3 报错路径与 G6 | 站得住 | 差分报错数与第二轮逐个相同；18 个胶水改动（含 1 个等价对照）行为符合预期（6.3） |
| H4 第 4 批变异与用例（含 H4-b） | 站得住 | 13 行变异在今天的树上全红，K20 改锚到 `index + 2` 之后仍红（6.4） |
| H4-c 两条改法：补用例（5-1 选）／spec 加幽灵读盘日志（没选） | 补用例选得站得住；spec 日志是罩得更宽、看不到块层的另一条，没有同样打中 | 5.2、5.4 |
| H5-1 死分支加死子式（中） | 站得住 | 3.2 |
| H5-2 冗余判定（低） | 风格项站得住；「用不到」够不着，删它改变了读盘次数 | 3.3 |
| H5-3 两套同名类型加手抄映射（中） | 站得住；5-1 选的改名只解决一半，没选的一处定义今天可行 | 4.1，原型 Verus 32 / 0、workspace `--all-targets` 编过、测试全绿 |
| H5-4 新根来源写在注释里（低） | 站得住 | 第四节表 |
| H5-5 K19 注释与代码不符（低） | 站得住 | 第四节表，改前两个值确实不等 |
| H5-6 K02 魔法数（低） | 站得住（低）；改前是仓里已有的写法 | 第四节表，今天仍有 16 行 `slot_offset(…, 4096)` |
| H5-7 手抄辅助函数（低） | 判轻：规则对测试代码放宽，改前不违例 | 第四节表，规则 `code-discipline.md:312` |
| H5-8 同一结局两个名字（低） | 站得住 | 第四节表 |
| H6 E165 `--pinned-cases` | 站得住 | 6.5，今天的脚本重算，仍只有行首裸 `state=` 一行不同 |
| 两轮报的数 | 逐个：见下表 | 2.2、6.1、6.2 |
| J1 5-1 是不是纯重构 | 结局层是；回调调用层 H5-2 不是 | 3.1 |
| J2 新用例证不证得到读盘次数与次序 | 名字说的那件事证到了；核头注释那一句比用例宽 | 5.2：S03、S05、S06、S08、S09 红，S04、S07、S10、S11、S12 绿 |

两轮报的数今天核出什么：

| 第二轮报的数 | 今天 |
|---|---|
| 58 904 864 个产品输入 | 总数逐个对得上；「零差异」对不上：diverged 444 399，value_differs 0 |
| m1–m4 阳性对照 1581 / 3821 / 545 / 520 | 对今天的核 1632 / 3872 / 550 / 571，四个都红；撤回 H5-2 那半句后 1581 / 3821 / 545 / 520 逐个相同 |
| 越界两族 161 443 与 5 410（改后 panic、改前返回） | 逐个相同 |
| 改后核 22 个单点改动全红 | 22 / 22 红，22 个合计与第二轮逐个相同（在撤回 H5-2 的树上量，见 6.2） |
| Verus 32 verified | 32 verified, 0 errors |
| 必红表逐行红 | 93 / 93 红（我的脚本判，不是门禁那一格） |

对判据第五条（「第二轮判决的一格在今天的代码上判反，或报的数今天核出不同 ⇒ 打中 J5」）：报的数里「零差异」与四个对照数今天核出不同，原因单一（H5-2）；没有一格的结局层判断判反，但 H5-2 的「冗余」与 H5-7 的「违例」两处偏重。是否算打中、怎么写回，归主 agent。

## 二、J5 差分重跑：今天的核对 fcc4d340，结局相同、读盘日志不同

### 2.1 量法

装置沿用第二轮正推的 `research/prompts/m3-verus-land-code-r2-sonnet-model/zz_diff.rs`：改前四个函数取自 `recovery_fcc4d340.rs`，用同一个 `gen_old.sh` 生成，放进第三轮冻结副本的拷贝，与今天的 `replay_journal` 在同一个内存读盘器上各跑一次，比返回值、读盘日志（盘号、偏移、长度的次序）、是否 panic。输入族与第二轮相同（随机两个种子、穷举结构六族、链加点名一族），总数逐个相同。
装置只改了一处：Tally 多三栏，`value_differs`（返回值不等）、`log_only`（返回值相等而日志不等）、`log_only_new_has_extra_reads`（日志只差在今天的核多读：改前的日志是今天日志的子序列）。副本在 `/tmp/claude-1000/m3-verus-land-code-r3-sonnet/difftree2/`（今天的核）与 `difftree3/`（只把 H5-2 删掉的那半句撤回，见 2.3），脚本与装置入库在 `research/prompts/m3-verus-land-code-r3-sonnet-model/`。命令都经 `run-with-memory-cap.sh 6G` 与 `capped.sh 10`（派发给的上限是 12G；第一次按 12G 起，整机排不上，退 252，那一次的输出不算结果，没有用）。

### 2.2 原样结果（四份带分栏的日志 `r3core-a.txt`、`r3core-b.txt`、`r3core-h52restored-a.txt`、`r3core-h52restored-b.txt`，全文在 `/tmp/claude-1000/m3-verus-land-code-r3-sonnet/logs/`；RESULT 与 CONTROL 行连同全文的 sha256 抄在模型目录 `logs/diff-results.txt`）

产品输入族（计数器都小于 2^48、键等于字段），今天的核对改前：

| 族 | 输入数 | diverged | value_differs | 其中只是今天多读 | 撤回 H5-2 那半句之后 diverged |
|---|---|---|---|---|---|
| 随机，种子 1 | 3 000 000 | 4 498 | 0 | 4 498 | 0 |
| 随机，种子 2 | 2 000 000 | 3 029 | 0 | 3 029 | 0 |
| 穷举结构（六族合计） | 6 129 120 | 105 096 | 0 | 105 096 | 0 |
| 链加点名 k=3 | 47 775 744 | 331 776 | 0 | 331 776 | 0 |
| 合计（命令算的，见下） | 58 904 864 | 444 399 | 0 | 444 399 | 0 |

合计行由 `python3` 对四份日志里九条产品族 RESULT 逐条累加得出：输入 58 904 864、diverged 444 399、value_differs 0、log_only 444 399、log_only_new_has_extra_reads 444 399、两边 panic 数 0。第二轮报的「58 904 864 个产品输入零差异」里，输入总数今天逐个对得上，「零差异」今天对不上：差在读盘日志上，每一处都是今天的核比改前多读，返回值（根、报告五字段、报错种类与内容）一处不差。

越出产品输入的两族（键与字段不等、含 u64 最大值的表），第二轮报的 161 443 与 5 410（改后 panic、改前返回）今天逐个相同：

| 族 | 输入数 | 今天 diverged | 其中只有今天 panic | 两边都返回而日志不同 | 撤回 H5-2 之后 diverged |
|---|---|---|---|---|---|
| 键与字段不等 | 300 000 | 161 611 | 161 443 | 168（都是今天多读） | 161 443 |
| 含 u64 最大值 | 20 000 | 5 993 | 5 410 | 214（都是今天多读） | 5 779 |

阳性对照（改前四个变异体各 40 000 个输入对今天的核）：

```
CONTROL[m1 去掉 take(limit)] diverged=1632 / 40000
CONTROL[m2 点名失败不断] diverged=3872 / 40000
CONTROL[m3 序号 FIRST 改 >2] diverged=550 / 40000
CONTROL[m4 max 只看第一条] diverged=571 / 40000
```

第二轮报的是 1581、3821、545、520。今天四个数各多 51、51、5、51，多出来的就是 2.3 里那些只差在多读的输入；撤回 H5-2 那半句之后四个数回到 1581、3821、545、520，与第二轮逐个相同（`r3core-h52restored-a.txt`）。四个变异体对今天的核都抓得到，装置还是红得起来。

### 2.3 差在哪、为什么：只有 H5-2 一处

第一批 DIVERGE 样例里改前日志是空的、今天的日志多一条 4096 字节的读（例：根 (2,1)，第四条记录 txg 2、序号 2、末条）：所选根那次发布读不出锚点，链首候选的 txg 正好是根的 txg 加一，但序号不是 1。
改前这个候选在锚点读不出那一支就被序号那一判拦下，不调反向链回调；5-1 把那一支里的序号判删了，序号改由下面一判管，那一判在反向链回调之后，所以今天先调一次反向链回调（读盘重读上一条记录）再被序号拦下。相关的三行代码和它们的次序：
「            if record.checkpoint_txg.0 != chain_start_txg_without_anchor.0 {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1419`）、
「    if back_chain_broken(index) {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1425`）、
「    if records_of_the_open_publish.is_empty() && record.ordinal_within_publish.0 != 1 {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1428`）。
改前的那半句在第二轮冻结副本里：「                || record.ordinal_within_publish.0 != 1 {」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-replay-prefix/src/lib.rs:1416`）。
因果证据：`difftree3` 对今天的核只做一处改动，把 `|| record.ordinal_within_publish.0 != 1` 加回 1419 行那个 if（`h52-restore.diff`，改动 2 行），再跑同一套差分，九条产品族 diverged 全 0，两条越界族与四个对照数全部回到第二轮的值。所以今天与改前的全部差异，都是 H5-2 这一处造成的，没有别的来源。

推翻本节的现象：用 `difftree3` 的改法再跑，产品族里还有任何一条 diverged 不为 0；或有一个输入上返回值不等（`value_differs` 栏今天全 0，是对这一句的直接量）。

## 三、J1 与 H5-1、H5-2：纯重构吗

### 3.1 结局层：是纯重构；回调调用层：H5-2 不是

- 返回的根、报告五字段、报错种类与内容：九条产品族 58 904 864 个输入上 `value_differs` 全 0（2.2）。H5-1 删掉的「计数器是最大值就交回 false」那一支，对产品输入不改结局；越界族里「只有今天 panic」的两个数（161 443、5 410）与第二轮逐个相同，说明胶水那两条断言的位置与行为没变。
- 回调调用的次数：H5-2 让 444 399 个输入（0.75%）的反向链回调多调一次，次序里插进一次读。形状只有一种：锚点读不出、链首候选 txg 对、序号不是 1。这种输入在盘上对应「所选根那次发布的末条撕了，下一次发布的第一条也没写成，只剩它的后几条」，崩溃后会出现（推的，没造镜像验）；一次恢复最多多读一次，因为链在这个候选上就断了（读代码推的）。结局相同，只多一次读。
- 5-1 的注释只说了结局等价（「它只会断、不改状态」），没说多读一次；核头注释第 11 行：「//! 挂载时每个候选读几次盘、按什么次序读，由 harness 用例 `crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs` 钉住。」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:11`），而这个形状没有一条用例或变异罩着（5.2 的 S12：把改前的半句撤回，新用例照样绿）。判据第一条（「改后核与第二轮冻结副本的核在某个输入上结局或回调调用不同 ⇒ 打中 J1」）字面上满足的是「回调调用不同」那个分句，不是「结局不同」。

### 3.2 H5-1：删掉之后「计数器是最大值」那类输入还有人管吗

- 核里留下的加一在 replay_core 的链上下一号那一处：「expected_next = Some((record.instance, JournalCounter(record.counter.0 + 1)));」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1746`），它的前置条件是 `every_counter_has_a_successor`（计数器小于 u64 最大值），胶水在调核之前断言：「            record.counter < u64::MAX,」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:3043`）。这两处没动，第二轮的越界族数今天原样。
- 被删的那一支对核自己的局部契约不是死的：`readable_record_of_the_same_publish_follows` 的 requires 只有「下标在范围内」与「入参严格升序」，没有「计数器小于最大值」，所以改前的守卫在那个局部契约下可达，只在 replay_core 的整体契约下（`every_counter_has_a_successor`）才死。删掉之后函数里不再有加一，守卫就没有要护的东西；循环本来就找不到更大的同实例记录，交回 false，与改前相同。第二轮「删后 Verus 仍 32 / 0」今天我重跑坐实（6.1）。判定：H5-1 站得住。

### 3.3 H5-2：「冗余」站得住，「用不到」够不着

- 结局上冗余：Verus 把它当等价变异验过（第二轮攻方 S02），今天的差分里 `value_differs` 全 0。
- 不是用不到：它在锚点读不出那一支里先于反向链回调执行，作用是不读盘就拒掉序号不对的链首。第二轮判决那一行写的是「冗余判定」，规则条款是「- **用不到的代码删掉**，注释掉的代码也删。」（`/home/fy5090/code/singlefs/.claude/singlefs-ai-sop/rules/code-discipline.md:181`）。冗余于结局不等于用不到；改前那半句有可观测的作用（少一次读），改前的注释只说冗余没说为什么留，所以第二轮攻方写的「保留它可以说是为了对照条款注 1 的字面」也没看到这个作用。
- 判定：作为风格项「注释没交代为什么留」站得住；作为「用不到的代码」够不着，删它改变了读盘次数。这不是要求改 core 去提速：我不提改法，只记事实，是否把半句加回去、还是在核头注释里把「读盘次数」的承诺收窄到用例罩得住的形状，归主 agent。
- 推翻这一条的现象：`difftree3` 的 2 行改动之后任一族 diverged 不为 0（那就还有别的来源）；或 5.2 的 S12 行红（那就有用例罩着这个形状）。

## 四、H5 八处逐条复核（改前的写法站不站得住）

引规则：`.claude/singlefs-ai-sop/rules/code-discipline.md`，原文行都现 `grep -nF` 过。

| 号 | 第二轮判 | 辩方判定 | 证据 |
|---|---|---|---|
| H5-1 死分支加死子式（中） | 打中 | 站得住 | 3.2。局部契约下可达、整核契约下死；删后 Verus 32 / 0 今天坐实（6.1），差分 value_differs 0 |
| H5-2 冗余判定（低） | 打中 | 风格项站得住，「用不到」够不着 | 3.3。删它改变了 444 399 个输入上的读盘次数 |
| H5-3 两套同名类型加手抄映射（中） | 打中 | 站得住；5-1 选的改名只解决一半，没选的一处定义今天可行 | 4.1 |
| H5-4 新根来源语义只写在注释（低） | 打中 | 站得住 | 改前是裸 `Option<usize>` 加注释，今天是 `NewRootSource`：「enum NewRootSource {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:2989`）；差分 value_differs 0，胶水变异里下标恒 0 的那一个 `index_always_zero` 红，5 个目标共 6 条测试红（6.3） |
| H5-5 K19 注释与代码不符（低） | 打中 | 站得住 | 改前注释说记录与根共用 fsid 低 8 字节：「/// 记录与根共用的 fsid 低 8 字节；读者是空池，不拿它比。」（`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs:26`），根用「        filesystem_identifier: [0x53; 16],」（同文件 31 行，`/tmp/claude-1000/m3-verus-land-code-r2/tree/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs:31`），记录用 0x5349_4E47_4C45_4653，两个值确实不等。今天记录取根的低 8 字节：「        filesystem_identifier: unit_filesystem_identifier(&FILESYSTEM_IDENTIFIER),」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/recovery_anchor_failure_carries_the_counters_the_prefix_kernel_saw.rs:57`），改好了，没有更差 |
| H5-6 K02 魔法数（低） | 打中 | 站得住，但改前是仓里已有的写法 | 改前 `slot_offset(target, 4096)` 在 harness 用例里本来就多处这样写；今天仍有 16 行 `slot_offset(…, 4096)` 留在 6 个文件里（`grep -n 'slot_offset(' *.rs \| grep -c 4096` 的输出 16），含 5-1 改过 H5-7 的那份 `writable_mount_takes_a_second_instance.rs`。这一条针对第 4 批新写的那份，改它没错，但仓里两种写法并存 |
| H5-7 手抄辅助函数（低） | 打中 | 判轻：改前不违例 | 规则对测试代码放宽：「| 测试代码也要 DRY | 通行做法 | **放宽** | 每个测试自己读得出场景；搭建可以抽成辅助函数，断言不藏进去 |」（`/home/fy5090/code/singlefs/.claude/singlefs-ai-sop/rules/code-discipline.md:312`）。改前两个辅助函数各自把场景写全，符合这一条；第二轮攻方自己写的也是「纪律对测试代码是放宽，只记一笔」。今天合成一个、加枚举参数，所有 9 处调用都多了一个实参，没有更差也没有更好 |
| H5-8 同一结局两个名字（低） | 打中 | 站得住 | 改前可执行一侧 `BrokenBeforeTheNamedUnits`，spec 一侧 `fails_a_check_before_the_named_units`；今天成员：「    FailsACheckBeforeTheNamedUnits,」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1314`）、「    NamedUnitsReportedUnverified,」（同文件 1315 行，`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:1315`），与 spec 一侧对齐，Verus 32 / 0 |

八处里：没有一处判反到「改前其实合规」的程度，但 H5-2 的「用不到」和 H5-7 的「违例」是两处偏重的判；H5-3 的改法只做了一半（4.1）。

### 4.1 H5-3 两条改法

第二轮判决给的改法是「两套同名类型改成说清角色的名字或一处定义」。5-1 选改名（实现员报告说 `singlefs-core` 在 `journal.rs` 里给其中两个类型写了固有方法，外 crate 的类型上写不了，主 agent 没核）。我核了这句话，并造了没选的那条改法的原型。

**实现员那句话属实，但不构成只能改名的理由**：core 里四个类型只有两个有固有方法：「impl JournalRecordPlaceInPublish {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/journal.rs:41`）、「impl JournalRecordOrdinalWithinPublish {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/journal.rs:74`）；`CheckpointTxg` 与 `InstanceGeneration`（address.rs 26、30 行，1440 与 968 处使用）没有固有方法，只多派生了 `Hash, PartialOrd, Ord`，核里的版本加上这几个派生就行。固有方法不能写在外 crate 的类型上，但可以把方法连类型一起搬进核（核拥有类型，`impl` 放在 `verus!` 之外的普通 Rust 里）。

**原型**（`/tmp/claude-1000/m3-verus-land-code-r3-sonnet/h53tree/`，改动入库 `h53/h53-prototype.diff`）：核里四个类型改回原名、三个加派生、两组固有方法与末条标志常量放在 `verus!` 块外；core 的 address.rs、journal.rs 里删定义改成 `pub use singlefs_replay_prefix::…`；胶水删掉逐字段手抄与 `JournalRecordPlaceInPublish` 两臂的映射。量到的：

| 项 | 结果 |
|---|---|
| 改动文件 | 4 份：核 lib.rs（-71 +101）、core address.rs（-6 +1）、journal.rs（-53 +4）、recovery.rs（-17 +6）；其余 harness 583 + 795 处与 checker-tier 的使用一字不改 |
| Verus（`--no-cheating --rlimit 10`） | 「verification results:: 32 verified, 0 errors」（`h53/v2-h53-verus.log`），与今天的核同数 |
| `cargo check --offline --workspace --all-targets` | 退出 0，只有一条未用 import 警告（`JournalRecordPlaceInPublish` 在 recovery.rs 52 行，原型没清） |
| `cargo test --release -p singlefs-core --lib` | 139 passed，0 failed |
| 核 `--lib` | 2 passed |
| harness 五个目标（K19、K20、K02、G08a、新读盘用例） | 22 passed，0 failed |

原型的限度：核里 `of_record_at_offset` 的 expect 消息我缩短了、文档注释没搬；没跑整包 harness、没跑 checker 档；是副本上的数，不是入库装置上的数。

两条改法对三处指控各自解决了什么：

| 指控 | 改名（5-1 选的） | 一处定义（没选的） |
|---|---|---|
| 同名两套类型只靠模块路径分开 | 解决：核里四个带 `PrefixKernel` 前缀 | 解决：一套 |
| 一个语义概念全仓一个名字 | 没解决：核里 `PrefixKernel…` 的注释自己写「说的是同一个量」，同一个量两个名字（规则条款：「| **一个语义概念全仓一个名字** | 单位是语义概念，不是词：同一个概念不许这里叫 `generation`、那里叫 `epoch`；…」，`/home/fy5090/code/singlefs/.claude/singlefs-ai-sop/rules/code-discipline.md:31`；核注释：「// 说的是同一个量，名字分开、全仓一搜只落到一处；胶水（recovery.rs 的 replay_journal）逐个换过来。」，`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:23`） | 解决 |
| 胶水手抄映射（「重复要生成，不许手抄」） | 没解决：两臂的 match 还在：「                JournalRecordPlaceInPublish::LastRecordOfThePublish => {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:3061`），下一行「                    PrefixKernelJournalRecordPlaceInPublish::LastRecordOfThePublish」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:3062`）| 解决（原型里胶水 `place_in_publish: record.place_in_publish,`） |
| 两条的代价 | 核里 69 行、胶水与别处 9 行带 `PrefixKernel`（`grep -c`） | 4 份文件，核多一块非验证的 `impl`，`Hash`、`Ord` 派生进了 `verus!` 里的类型（Verus 照验） |

判定：第二轮对 H5-3 的判站得住；两条改法没有同样打中——改名这一条留着两处指控，一处定义两处都解决，今天在冻结副本上可行。代价是核里多了磁盘格式的位常量与两个固有方法（核的「只依赖 vstd」没破）。这是辩方给的原型，被攻过零轮，交用户时写「零轮」。
推翻它的现象：把原型放进入库装置，整包 harness 或 checker 档红；或 Verus 在入库版本上不再是 32 / 0。

## 五、J2 与 H4-c：新用例证得到什么

### 5.1 用例名字说的那件事，证到了

用例钉两样：扫完环之后紧接的 50 次读（两个候选，各一次反向链重读加 12 个点名单元的两份，12 × 2 + 1 = 25）与判完之后环里 0 次读。两处断言原文：「        (12, 12, 50),」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:274`）、「        (10, 0),」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:306`）。冻结副本上原样跑这一条：`test result: ok. 1 passed`（`j2/baseline.log`）。
用例造的形状写在文件头：「//! B（txg 4、jsn 4）、C（txg 5、jsn 5）两次单条发布的记录与单元都在、根槽都坏了 ⇒ 所选根是 A 的，B、C 都施加。」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-harness/tests/mounted_recovery_reads_each_candidate_once_in_order.rs:10`）：两个都带提交标记、序号都是 1、都是末条的单条发布，锚点读得到，只读挂载，点名验证开。

### 5.2 S03–S07 与我加的 S08–S12，套到今天的核上，新用例红不红

改法照第二轮攻方 `call-pattern-rows.tsv` 的 S03–S07，锚点换成今天的写法（S07 的成员名随 H5-8 改名）；S08–S12 是我加的。每行在冻结副本拷贝上套一处、只跑新用例、跑完拷回原文件（`j2/run-j2.sh`，`j2/summary.tsv` 原样入库）。

| 行 | 改法 | 新用例 |
|---|---|---|
| S03 | 反向链回调每个候选调两次 | 红 |
| S04 | 点名验证关掉（Skip）时照样调点名回调 | **绿** |
| S05 | 点名回调提前到逐条判定之前 | 红 |
| S06 | 主循环每条记录都先调一次反向链回调 | 红 |
| S07 | 末条提交标记那一判挪到反向链回调之前 | **绿** |
| S08 | 点名回调每个候选调两次 | 红 |
| S09 | 反向链回调挪到起始判之前 | 红 |
| S10 | 序号大于 1 的候选反向链回调调两次 | **绿** |
| S11 | 没带提交标记的候选反向链回调调两次 | **绿** |
| S12 | 撤回 H5-2 那半句（链首序号不是 1 时不调反向链回调） | **绿** |

第二轮攻方在旧用例上量到 S03、S05 绿、S06 红；今天 S03、S05、S06 都红，H4-c 欠的那一条补上了，补的是它说的那一类里这个形状。S07 还绿，S04 也绿；S10、S11、S12 是用例的形状（序号 1、带提交标记、锚点读得到）之外的同类改法，都绿。
- S04 不是这条用例该管的：Skip 只在 `recover` 带 `ConsultWithoutNamedVerification` 时出现：「        JournalPolicy::Consult | JournalPolicy::ConsultWithoutNamedVerification => {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/recovery.rs:4187`），挂载两条路都传 `true`（`mounted_read.rs:564`、`mount.rs:4556` 两行 `true,`）。但它在仓里也没有别的用例罩着：用到 `ConsultWithoutNamedVerification` 的三份测试（`recovery_reads_the_created_file_after_reopen.rs`、`whole_device_loss_after_warm_up.rs`、checker 档的 `crash_enumeration_new_pool_file_creation_stream.rs`）里 grep 不到任何读计数（`ReadCounting`、`reads_inside`、`reads` 三个词零命中）。
- S07 是核头注释那一句够得着的：核头注释说「挂载时每个候选读几次盘、按什么次序读」都钉住了，S07 改的是没带提交标记的末条候选在反向链之前还是之后判，读盘次数不同，用例的候选都带提交标记，罩不住。

判据第二条（「S03–S07 里有一行套到今天的核上、新用例照样绿，而核头注释说那一类由这条用例钉住 ⇒ 打中 J2」）：S07 满足；S04 在头注释说的「挂载时」之外（Skip 不在挂载路径上），不满足这一句，但仓里也没有别的用例罩着它。辩方能给的站得住的理由是：用例名字、用例头的形状说明、断言都只说了「水位之上两个健康的单条发布、只读挂载」，打中的是核头注释那一句比用例宽，用例本身没夸大。核头注释写没写出罩不住的形状：没写。

### 5.3 mount_read_only 与可写挂载、recover 是不是同一条取前缀路径

三处都调同一个 `replay_journal`（`recovery.rs:3015` 的 `pub fn replay_journal(`），再调同一个 `replay_core`，所以「取前缀的核」是同一份。读盘一层不同：
- 只读挂载传裸读者，用例数的就是它的块层读。
- 可写挂载的读阶段传 `ReadStageCache`，且读两遍：「    let first = read_stage(&cache, devices, system_configuration)?;」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/mount.rs:4587`）、「    let reread = read_stage(&cache, devices, system_configuration)?;」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/mount.rs:4593`）；缓存按 (盘, 偏移, 长度) 记住读出的非零字节：「        if let Some(bytes) = self.bytes_read_by_placement.borrow().get(&placement) {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-core/src/mount.rs:4430`），同一处读第二次不再碰块层。所以 S03 那一类「多调回调」在可写挂载的块层读上看不出来，这条用例的 50 次读只对只读挂载那条路成立。
- `recover` 的读者是调用方给的；块层读数它自己不数。

### 5.4 H4-c 两条改法

H4-c 当时列的两条：给 spec 加幽灵读盘日志，或补一条用例。5-1 选了补用例（并把核头注释从「同原文」改成指到用例）。

| | 补用例（5-1 选的） | spec 加幽灵读盘日志（没选的） |
|---|---|---|
| 看得到什么 | 块层读，连胶水回调里 `reader.read` 的次数一起（今天 S03、S05、S06、S08、S09 红） | 核对回调的调用序列，对所有输入成立；看不到回调里的真读盘：回调在胶水里，不归 Verus |
| 罩得住的形状 | 一个（5.1），S04、S07、S10、S11、S12 绿 | 全部：回调的次数与次序写成 `ensures`，S03–S12 都是 spec 违例（推的，没造） |
| 代价 | 一条用例，今天已落 | 要把 `Fn` 回调换成带幽灵日志的写法并改每个循环不变式，核的证明要重写一遍，量不到（没做原型） |
| 同样打中别的臂？ | 否 | 否 |

判定：补用例选得站得住（它看得到的是核头注释那句话真正在说的块层读），没选的 spec 日志罩得更宽但看不到块层，两条都不能单独担保那句注释；5-1 把注释收窄到用例的名字范围，这一步还差：注释写「每个候选」，用例只罩一种候选。
推翻这一条的现象：给核头注释那一句补上形状限定（例：「两个带提交标记、序号为 1 的单条发布、只读挂载」）后，S07、S10、S11、S12 对应的类仍有人说它们被用例钉住。

## 六、前两轮「没打中」的格与两轮报的数，今天核不核得动

### 6.1 Verus 与必红表

- 今天的核 `--no-cheating --rlimit 10`：「verification results:: 32 verified, 0 errors」，退出 0（`verus/v1-pristine.log`，Verus 0.2026.09.20，2G 包装）。第二轮与今天的数一致。
- 必红表 `verus-mutations.tsv` 93 行逐行套一处、跑一次、删副本：93 行全红，0 行 NOT-RED，0 行锚点不中；错误数分布 47 行「31 verified, 1 errors」、39 行「30 verified, 2 errors」、5 行「29 verified, 3 errors」、2 行「28 verified, 4 errors」，每一行的期望片段都在输出里（`verus/mustred-summary-93-rows.tsv`）。
- 其中 13 行（表的物理行号 23、24、26、28、29、36、38、45、48、72、77、80、87）的输出里有一行 rlimit 用尽的 error，并且另有至少一条非 rlimit 的证明错；同一台机器上把这 13 行再跑一遍，逐栏相同（`verus/mustred-summary-13-rlimit-rows-second-run.tsv`），说明 rlimit 用尽是确定的。主 agent 在正文第一节表里写的是「9 行期望改成实测值（其中 4 行的错误数含一行 rlimit 用尽）」，我数的是全表 13 行含这一行，两个数没对账（口径不同：他数的是改了期望的那 9 行）。换 rlimit 或换 Verus 版本时这 13 行会不会误判不归我（J3），我只报数。

### 6.2 改后核的 22 个单点改动对差分的阳性对照（H1「差分分辨得出每一处」）

第二轮正推对改后核做 22 个单点改动、各用差分量红不红。今天在 `difftree3`（今天的核只撤回 H5-2 那半句，这样基线是 0，改动才分得清）上重跑同一套：22 个全红，22 个合计 diverged 与第二轮 `research/prompts/m3-verus-land-code-r2-sonnet-model/logs/mutate_new_core_out.txt` 逐个相同（例：limit_gt 6712、backchain_off 218810、above_ge 730980、maximum_takes_minimum 32102）。脚本取自第二轮、没改改动表，输出 `mutate_new_core_r3_out.txt`。注意是撤回那半句的树：对今天的核本身直接跑，基线就有 4 498 之类的读盘差，改动红不红分不清。

### 6.3 H3：核失败到 RecoveryFailure 的映射

- 核失败仍只有两个：「pub enum CoreFailure {」（`/tmp/claude-1000/m3-verus-land-code-r3/tree/crates/singlefs-replay-prefix/src/lib.rs:72`）下面两个成员，胶水的 `match` 两臂各接一个，没有通配臂（`recovery.rs` 3108–3135 行读过）。
- 差分里两种报错各出现且整值相等：`value_differs` 全 0；报错数与第二轮逐个相同（种子 1 `errs: 327781`、种子 2 `errs: 218742`、`anchor_err: 163996`、`txg_err: 54746`、链加点名 `anchor_err: 11833344`）。
- 胶水单点改动（第二轮的 16 个，`rebuilt_wrong_record` 的下标按今天的名字改写，另加两个与 H5-4 枚举有关的；6 个 harness 目标，含新用例；`h3/glue-r3-summary.txt`）：

| 改动 | 结果 |
|---|---|
| counters_reversed、counters_drop_first、error_txg_wrong、error_instance_wrong | 红 |
| no_successor_maps_to_no_valid_root、no_successor_wrong_what | 红 |
| above_water_is_table_size、applied_is_zero、max_tx_zero、passed_swapped_with_failed、named_all_to_any、skip_mode_inverted、rebuilt_wrong_record | 红 |
| counter_assert_le、key_assert_removed | 红（红在 `should panic` 那两条，装置的正则认不出名字，日志里 `- should panic ... FAILED`） |
| back_chain_inverted | 这 6 个目标上绿；第 920 行点名的目标上红：`a_journal_record_whose_back_chain_disagrees_with_the_previous_record_is_left_out_of_the_replayed_prefix`（`h4/summary-row-920.tsv`） |
| index_always_zero（新：记录下标恒取 0） | 红，5 个目标共 6 条测试红 |
| chosen_root_arm_equivalent_rewrite_expected_green（等价改写，对照） | 绿（应当绿） |

H3 今天仍站得住。推翻它的现象：胶水某一处映射或报告字段的单点改动，在这 6 个目标加第 920 行点名的目标上全绿。

### 6.4 H4：第 4 批新加变异在今天的树上

`crates/mutations.tsv` 第 512、513、1497–1505、1562、1563 行（1497–1501 是 K19 五行、1502 是 K20、1503 是 K02、1504–1505 是 G08a 两行，1562、1563 是 5-1 追加的 S03、S05，512、513 是第 4 批改写的两行）在冻结副本拷贝上逐行套、只跑点名的 harness 目标（`h4/harness-rows-r3.sh`，`cargo test --release`，经 `run-with-memory-cap.sh 6G`）：13 行全红，红的测试都是那一行第 6 段点名的那一条（`h4/summary-rows-512-513-1497-1505-1562-1563.tsv`）。K20 改锚到 `index + 2` 之后（第 1502 行）仍红，红在 `last_record_flag_followed_only_by_the_very_next_record_of_the_same_publish_breaks_the_chain_at_the_flag`。第 1500、1501 行红在 `should panic` 那两条用例上（日志里 `- should panic ... FAILED`）。H4 今天仍站得住。第 1506–1508（E165）、517、518（层 0）在 checker 档，没跑。
推翻它的现象：这 13 行里任一行在点名的目标上绿。

### 6.5 H6：E165 `--pinned-cases`

装置源文件、钉表的 sha256 与第二轮相同（装置 `24a4b2eb…`，钉表 `844bcb23…`），`research/scripts/e165-pin-red-examples.py` 在第二轮之后被别的会话改过（今天 sha256 以 `9024abb1…` 起头，第二轮读的是 `3ba54f4f…`）。我对今天的脚本重算：取今天脚本里 `state_before_outcome` 与 `input_fingerprint` 的原文逐字执行，对第二轮 168 行（151 张表行加 17 条怪行）各算一遍，与第二轮装置侧输出逐行比：「行数 python=168 rust=168；不同 1 行；None 的行数 python=9 rust=10」，不同的仍是行首裸 `state=` 那一行（行 159）；钉表 151 行的静态核「问题： 无」（`h6/compare_today.out`、`h6/h6_table_check.out`）。H6 今天仍站得住（Python 侧用今天的脚本，装置侧是第二轮的原输出，因为装置源文件没变）。
推翻它的现象：真跑 `--pinned-cases`（重型，我没跑）的逐态行与钉表第 ⑦ 段不同。

### 6.6 H1 两条新断言、H2 字面判据

- 两条新断言在 5-1 里原样没动（`record.counter < u64::MAX,` 在 `recovery.rs:3043`，键断言在它上面），越界两族的 161 443 与 5 410 与第二轮逐个相同，调用点没变（两份冻结副本的 `recovery.rs` 用 `diff` 比，共 44 行、11 处改动，位置都落在 5-1 的 diff 文件里 `recovery.rs` 的那几个 hunk 里）。
- H2 字面判据（只改可执行判定、Verus 照样验得过 ⇒ 打中）：今天必红表 93 行全红、改核的变异没有一行等价；第二轮攻方的 S02（锚点读不出那一支删序号判）今天已经不是变异而是代码本身，等价性现在由 H5-2 的差分量过（结局零差异）。口子（读盘次数与次序）在 5.4。H2 站得住。

## 七、没做什么

- 没替主 agent 撤销或维持判决；没改冻结副本、没改主仓任何文件。所有改动（差分装置、H5-2 撤回、H5-3 原型、变异套用）都在 `/tmp/claude-1000/m3-verus-land-code-r3-sonnet/` 的拷贝里，入库的只有 `research/prompts/m3-verus-land-code-r3-sonnet-model/` 下的脚本、diff 与结果摘要。
- 第一次套 S03–S09 时把 `apply-row.py` 误对着我自己的 `base/`（冻结副本的拷贝）连套了几行，随即用冻结副本的 `lib.rs` 拷回并 `cmp` 无差异（输出 `restored`），之后每行都从 `j2tree/` 起套，每行跑完拷回。冻结副本本身没碰。
- 没跑 checker 档：E165 三行单测与变异（`crates/mutations.tsv` 第 1506–1508 行）、层 0 两行（第 517、518 行）、门禁 verus-proof 与 verus-must-red-worlds 两格、`research/scripts/verus-proof-check.py`、`e165-pin-red-examples.py --replay`、整包 harness、整包 `cargo test`、QEMU、herd7。必红表那 93 行是我自己的脚本（`verus/mustred.py`）逐行直接起 Verus、照 `verus-proof-check.py` 文件头的判红口径（期望片段在输出里、且除 rlimit 用尽与 aborting 之外至少一条 error）判的，不是门禁那一格。
- 没复核 H6 装置侧 Rust 输出：装置源文件 sha256 与第二轮快照清单里同一行相同（`24a4b2eb…`），所以直接用第二轮的 Rust 侧输出 `rust_out.tsv`，只在 Python 侧用今天的脚本重算；没跑装置 `--pinned-cases`。
- 没做 H4-c 的 spec 幽灵读盘日志原型：5.4 表里「spec 日志」一栏的覆盖与代价都是推的。
- 没改 core 的实现代码，也没提为了快去改 core 的改法（用户 2026-10-01 定）。H5-2 里「多读一次」我只记了事实，不当作性能问题。
- 差分装置是我自己的副本（见 `.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」一节「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」那一条）：引 444 399、1632 / 3872 / 550 / 571 这些数之前，请按模型目录 README 把装置当新一次跑。
- 内存包装：差分、变异、Verus 都经 `run-with-memory-cap.sh`，上限取派发给的 12G 之内的 6G（Verus 2G）；按 12G 起的第一次退 252（整机排不上），那次输出作废，没有用。`ps` 看到别的 `cargo`、`gate.sh`、`checker-tier-qemu-device-streams.sh`、`e168_delete_ring_window_borrow` 在跑，没有 `qemu-system`、`vm-bench.sh`、`e152`、`fio`；我的命令都带 `nice -n 19`。
- 没读禁读清单里的任何文件：`research/prompts/m3-verus-land-code-r3-opus-output.md`、`research/prompts/m3-verus-land-code-r3-opus-model/`、`/tmp/claude-1000/m3-verus-land-code-r3-opus/`、`research/prompts/m3-verus-land-code-r3-main-verification.md`。
- 草稿里自己建的编译目录与仓副本（`difftree`、`difftree2`、`difftree3`、`h53tree`、`j2tree`、`h3tree`、`h4tree`、`mut/tree_m`、`base`，都在 `/tmp/claude-1000/m3-verus-land-code-r3-sonnet/` 下）交回前已删；脚本、diff 与结果摘要在模型目录里，日志全文留在草稿目录 `logs/`。

## 八、零轮的东西（交用户表用）

- H5-3 的「一处定义」原型（4.1）：被攻过零轮，辩方给的，只在副本上量过。
- 「核头注释那一句按用例的形状收窄」（5.4）：推的，没改。
