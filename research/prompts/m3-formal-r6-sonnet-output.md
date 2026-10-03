# m3-formal-r6 辩方腿报告（Sonnet，第三轮，2026-10-02）

<!-- doc-lint:not-numbers X1 X2 X3 O1 O2 O3 Y2 U1 U2 U3 U4 U5 U6 U7 F1 F2 F3 F4 F5 F6 G1 G2 V1 V2 V3 W1 W2 T1 T5 T6 P Q R1 R2 Tm S1 S3 C651 C191 C519 C554 E129 E142 A4c -->

轮名 m3-formal-r6，立场云端辩方，格 X1–X3（正文第四节分工表「云端辩方」一行）。复核第二轮判决 `research/prompts/m3-formal-r5-main-verification.md`、第一轮判决 `research/prompts/m3-formal-r4-main-verification.md`、C651 第一轮判决 `research/prompts/m3-c651-r1-main-verification.md`。用户定的 F1–F6、G1、G2、W1 不改，只判写成原句时哪里站不住。方案按 `crates/` 今天的实现谈。

读了：背景正文第一至五节（第 1–44 行；附录里的 kb 抄录没读，kb 条款一律去 kb 文件里现查）；三份判决；r5 与 C651 的四份腿报告（r4 两份腿报告没读，第一轮的内容以第一轮判决与第二轮腿报告对它的引述为准）；第一轮岔路单 `research/prompts/m3-formal-r4-forks.md` 的 F2 两行；第二轮攻方模型目录里的 `main.rs` 与 `u5-um-r2.log`、`u5-rb-mut.log`；代码读冻结副本 `/tmp/claude-1000/m3-formal-r6/tree/`（开工先核：`sha256sum -c research/prompts/m3-formal-r6-snapshot/crates-sha256.txt | grep -vc ': OK$'` 输出 `0`）。没读禁读清单里的文件。
记号：[读] 读 kb 或代码；[推] 推的，没量；[量] 别的腿量过、我引来的数。**本轮我自己一个数都没量**，没编译、没跑任何测试，没写模型。引文行号都是这一次在被引文件里 `grep -nF` 现取的，kb 条款行号是 kb 文件自己的行号。

## 一、一句话

1. **判决收严的地方，按今天的实现我只找到两处写成原句会把合法状态判红**：U5 的 txg 关系式写成等式（比 D23 的「≥」严，没量）；U1 句子里「操作必须在」按字面读，在 fsync 之后又回退的历史上为假（按「状态下标 ≥ lo」读则成立）。卸载记号、F、全四条、v1 二次崩溃这几处收严，今天的实现上看不出误红。
2. **写成条款句站不住的**：U6 的「由 O1 判官在同一份镜像上复算」今天没有这个函数；G1 的「任一红即红」在故障注入的说谎档没写 RefFS 投什么票；G2 的「第 m 次已确认发布」在今天的历史上定义得出，在「恢复之后接着写、再回退」的历史上定义不出（收窄了合法目标）；U2 ① 的「自动触发的发布不在射程」只说了不在射程、没说不判；V1 条款句的「只收…」与实现在两个出口上不同（第三节 3.6）；X2 白名单少一样（崩溃落在第几个操作里）。
3. **最重的一条不是收严**：V1 把「全池槽 0 都不可用时按 4096 读槽 1」判出局，而这一读法是用户 2026-09-14 收尾弹窗定的「甲」（`records/2026-09-13-总审核.md:441`），C651 的背景、两份腿报告与判决都没提它；核心的逐档是 2026-09-27 才进的。出局要交用户确认翻案（第三节 3.7）。
4. **四个被判出局的写法**：照字面只取 ③ 在「O1 判官不拆」时与全四条是同一件事，输在拆开之后的「没有更旧带文件版本」一格；回退弃权整段历史弃权的代价是每步 4%–10% 的回退比重（8 步历史约 28%–57% 含回退，推）；V1-常量在槽距 4096 的池上与逐档逐态相同、在「旧槽距更大」的残留格上优于逐档，输在 io_min 大于 4096 的单次崩溃；W2 不清在新池上不输，输在 S 组，而判决里「fsid 每次 mkfs 随机」那个假设在 harness 里不成立（第四节）。

## 二、判定一览（第二轮判决第四节八条、G1、G2；第一轮三条）

判定三档：站得住 / 够不着 / 同样打中别的臂；句子要补的写「站得住，原句要补」，原句本身站不住的写「原句站不住」。

| 判决条目 | 辩方判定 | 证据 | 什么现象会推翻它 |
|---|---|---|---|
| U1 合成句 | 站得住（按状态下标读）；字面「必须在」原句站不住 | 判决写的是「fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在」（`research/prompts/m3-formal-r5-main-verification.md:21`）。管理员回退是一次普通的向前发布（`.claude/kb/decisions/23-journal的角色与格式.md:377`「管理员回退是挂着时的一次向前发布」），根持久之后才向管理员确认（`.claude/kb/decisions/23-journal的角色与格式.md:389`「回退的根持久之后才向管理员确认」）。所以 fsync 成功、之后回退到更早一版、回退根持久、崩溃：恢复读回回退目标，合法，字面读法判红 [推]。回退在 BROAD 比重里每步 10（`crates/singlefs-harness/src/history.rs:568`「(HistoryOperationKind::RollBackWhileMounted, 10),」）。今天历史里没有 fsync 这一步（`grep -ci fsync crates/singlefs-harness/src/history.rs` 输出 `0`），所以今天没有样本 | D23 里有一句要求回退不许越过已 fsync 的发布 |
| U2 ① 边界只取显式发布点 | 站得住（今天）；原句要补「含自动触发的历史 RefFS 弃权，或 harness 把触发关掉」 | 今天没有触发路径（`crates/singlefs-core/src/system_configuration.rs:278`「也没有触发路径——T_time 与 T_dirty 的发布触发没实现」）。一旦实现，自动发布产生的恢复态不在 Q 的边界集合里，Q 把它判红；判决写的是「自动触发的发布不在 RefFS 射程，归 M2 与记录核对器」（`research/prompts/m3-formal-r5-main-verification.md:22`），「不在射程」不等于「不判」 [推] | 实现出触发路径之后，崩溃注入里出现含自动发布的历史而 RefFS 仍判绿 |
| U2 ② 每次恢复后重新起头 | 站得住；今天没有历史走到 | 崩溃是历史的终点：崩溃点之后只有挂载与一次发布两截，`ColdStartRecover` 是干净冷启动（`crates/singlefs-harness/src/history.rs:414` 起八种操作里没有「崩溃后接着写」）。这句只对将来的历史有对象 | harness 加「崩溃之后接着写」的历史而这句判错 |
| U3 / G1 全四条（用户定） | 站得住；原句在故障注入调用点缺一句 | O1 判官今天是一个函数、四条判据（`crates/singlefs-harness/src/model.rs:2234`「四条判据：」；`crates/singlefs-harness/src/model.rs:2246`「pub fn crash_recovery_disagreement(」），崩溃注入第 902 行与故障注入第 2353 行各调一次，全四条合取比 6 态的字面多 0 行判定代码（判决自己写的）。缺的一句：故障注入的说谎档 RefFS 投什么票（第三节 3.3） | 说谎档下 RefFS 的 lo 取「最后成功的步」而判绿 |
| U3 独立性一句 | 站得住 | RefFS 不读录制流、③ 的入参与枚举同源，记在 C191 那一类账上；今天 `step_containing` 给 (lo, hi)（`crates/singlefs-checker-tier/src/crash_injection.rs:751`「fn step_containing(&self, stream_index: usize) -> Option<StepPosition> {」） | 找到一个只有录制流才有、RefFS 非要不可的量 |
| U4 D17 改句、D13 分工、闭包、门禁⑤⑥⑦、第一版操作集合 | 站得住；输入白名单少一样 | 门禁今天的④是「singlefs-harness 的传递闭包（dev-dependencies 也算）里没有 singlefs-checker-tier」（`.claude/gate.d/checker-independence-and-sync.sh:24`），⑤⑥⑦与它同形、harness 依赖 RefFS 不碰它。白名单少的一样：崩溃落在第几个操作里（hi）——背景材料 X2 列的四样里没有它，第二轮攻方的四样里也没有（`research/prompts/m3-formal-r5-opus-output.md:259`「RefFS 从调用方只收四样输入」）；今天由 `step_containing` 给 | 白名单写成五样之后找到第六样 |
| U5 卸载记号 | 站得住 | R1 带记号的 16 390 态全不同、R2 恒非卸载 24 598 态成立（`research/prompts/m3-formal-r5-opus-model/u5-um-r2.log` 的 `eff.unmount_marker` 三行）；这两个数只当存在性见证，份额由段形定（第五节 X3） | 入库装置上 R2 带卸载记号的态 |
| U5 F | 前半站得住；后半「推了时」今天只在测试开关的前缀上有对象 | 产品准入下扫 N 一次都没让挂载推抬 F（`research/prompts/m3-formal-r5-opus-output.md:295`「产品准入下让可写挂载推抬 F 的前缀」）；推了的 20 496 态来自关空间准入造的前缀（`research/prompts/m3-formal-r5-opus-output.md:302`「fms256-12 的前缀是关空间准入造的」） | 找到产品准入下的前缀让挂载推抬 F |
| U5 系统配置槽拆三样 | 不可变那一样站得住；运行量（槽世代号、journal_tail、F 副本）是推的 | 运行量四个字段在 `crates/singlefs-core/src/system_configuration.rs:314`「pub struct SystemRuntimeQuantities {」，第二轮攻方的 `compare` 一个都没比（`main.rs` 里 `push("` 只有 posix 内容、不可变配置、设备号、F、卸载记号、水位、实例、行、分配计数） | 入库装置上运行量任一字段与式子不符 |
| U5 实例代号与行 | 站得住 | 逐字出自 D18：「再取新代号 = max(」（`.claude/kb/decisions/18-块里携带什么信息.md:316`）、「恢复写的行」（`.claude/kb/decisions/18-块里携带什么信息.md:309`）；57 446 态 [量] | 无文件历史上 R2 的行与式子不符 |
| U5 txg 关系式 | **原句站不住**（等式） | 辩方式子是 `txg(R2) = Tm + n`（`research/prompts/m3-formal-r5-sonnet-output.md:159`），判决「取辩方的式子（推的）」（`research/prompts/m3-formal-r5-main-verification.md:25`「txg 关系式」）。D23 写的是不等式：「checkpoint_txg ≥ max(根环里全部根记录的 txg, 环里全部自证通过的记录的 checkpoint_txg) + 1」，并认下跳号：「代价：跳号比今天大（最多到被抛弃时间线的长度）」（均 `.claude/kb/decisions/23-journal的角色与格式.md:370`）。第二轮攻方的比较里没有 txg 这一项 [读] | 今天的实现上 R2 的 txg ≠ Tm + n 的态存在；或 D23 把「≥」收成「=」 |
| U5 水位、树指针、扫描报告、分配记录 | 水位站得住（量）；其余推的；有四项没碰 | 根记录里 `mapping_root`（`crates/singlefs-core/src/root_record.rs:63`）与 `allocation_record_tree_root`（`crates/singlefs-core/src/root_record.rs:70`）、墓碑树 / 谱系树 / deleted_inodes 三根指针（「墓碑树、谱系树随现行那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:377`））没有行（第五节 X2） | 入库装置上这些指针在只挂载之后变了而没有行罩 |
| U6 二次崩溃比 v1 | 站得住（推）；没有任何二次崩溃态被量过 | 第二轮攻方的 57 446 态全是「挂载做成的」，被拒的 90 个是第三态（`research/prompts/m3-formal-r5-opus-output.md:296`）；第三截崩在挂载途中的态一个没进样 | 二次崩溃镜像上恢复读回比 v1 旧、而按 ③ 合法 |
| U6 拒绝合法四条件 | **原句站不住** | 「由 O1 判官在同一份镜像上复算同样判拒」（`research/prompts/m3-formal-r5-main-verification.md:26`）：O1 的判官只收 (版本目录, 盘上最新根, 读回)，没有镜像入口；O1 的拒绝答案是照脚本操作答的（第三节 3.4） | O1 加了一个吃镜像的函数 |
| U7 C191 一句与三处改动 | 站得住 | D13 已定项 13 的标题是「冲突 2：crash refinement 的 spec 与模型对拍的 model 是同一个东西的两种写法」（`.claude/kb/decisions/13-验证路线.md:293`），「模型对拍的 model」字面就是 O1（D13 已定项 6 表 O1 一行）；我起初疑「model 指 O1」与 U4 的分工相冲，现查后撤回 | D13 已定项 13 的 model 在别处被定义成 RefFS |
| 「任意子集出局」限串行 | 站得住 | D13 自己写了域与重开条件（`.claude/kb/decisions/13-验证路线.md:38`「M3 的单调推论只在串行提交、不带故障的崩溃状态里成立」、`.claude/kb/decisions/13-验证路线.md:33`「开并行提交或把域放宽到带故障时重开」） | 串行下找到合法实现落在任意子集里而不在前缀集里 |
| G2 回退目标作输入（用户定） | 今天的历史上定义得出；将来的历史上原句站不住 | 逐格见第三节 3.5：回退被拒（结局是输入）、回退之后崩溃（允许集合 {回退前, S_m}）今天定义得出；「恢复之后接着写再回退」与「目标是未确认但已落盘的发布」上 m 定义不出 | 候选集条款（D23:381）改成只收已确认发布的根 |

## 二（续）、第一轮三条、F1–F6、C651 四条

| 判决条目 | 辩方判定 | 证据 | 什么现象会推翻它 |
|---|---|---|---|
| T1（第一轮第四节第 1 条） | 够不着 | 已被第二轮 U1 改写，照第二轮：第一轮的原句是「fsync 返回成功之后，调用开始时刻已经接受的全部操作必须在」（`research/prompts/m3-formal-r4-main-verification.md:43`），第二轮把「调用开始时刻」换成「这次 fsync 等到的那次发布带走的」 | — |
| T5（第一轮第四节第 2 条） | 够不着（已被 U5 改写）；口径要选一个 | 第一轮写「比的对象是挂载后那次发布之前的那一版」（`research/prompts/m3-formal-r4-main-verification.md:44`），第二轮 U5 的 R2 是「只挂载、不发布」之后的恢复，`mount+write` 动作才取发布之前那一版；两者在只挂载时是同一版，写回条款时选一个口径 | 两种口径在某个态上给出不同内容 |
| T6 射程 | 站得住；今天没有样本 | 射程句「可写挂载被拒的状态（例：系统配置槽撕裂）不在射程里，判的是拒绝本身合不合法」（`research/prompts/m3-formal-r4-main-verification.md:26`）对的是第三态；崩溃注入的枚举域是「更早的段整段持久、当前段任意真子集、更晚的段一个都没持久」（`crates/singlefs-checker-tier/src/crash_injection.rs:5`），不取第三态，被拒只剩单元区墙一格 | 崩溃注入加第三态 |
| F1 Q（用户定） | 站得住；要带 U2 两句 | 见第二节 U2 | 同 U2 |
| F2 合取（用户定） | 原句「O1 判官的 ③」在今天的实现上读不出与全四条的差别 | 岔路单自己的代价格写「RefFS 的判定在同样的位置并排加一次」（`research/prompts/m3-formal-r4-forks.md:55`），那一格列的调用点是 O1 判官整个函数的调用点：「O1 判官今天已经在崩溃注入第 904、1830 行与故障注入第 2352、2359 行调」（`research/prompts/m3-formal-r4-forks.md:55`）。③ 没有单独的入口，「只取 ③」要先拆函数（第四节 4.1） | 岔路单或用户原话里有「拆 O1 判官」一句 |
| F3 另起 / F4 包 / F5 不算（用户定） | 站得住 | 见第二节 U4、U7 | 同 U4、U7 |
| F6 加二次（用户定） | 站得住；字段一半今天缺入口 | `writable_mount_alone_on`（`crates/singlefs-checker-tier/src/crash_injection.rs:1209`「pub fn writable_mount_alone_on(」）不交回录制流；`content_read_after_the_uninterrupted_mount`（`crates/singlefs-checker-tier/src/crash_injection.rs:1246`「fn content_read_after_the_uninterrupted_mount(」）只返内容。判决已写「字段一半要从恢复报告取字段」 | 入口改成交回录制流与字段 |
| V1 逐档 | 读法站得住；条款句要补两个出口；常量出局要交用户 | 第三节 3.6、3.7 | 见各节 |
| V2 世代号最大同号取偏移小 | 站得住 | 实现里就是这一句（`crates/singlefs-core/src/recovery.rs:945`「几档都收得到时取世代号最大的」、`crates/singlefs-core/src/recovery.rs:946`「世代号相同取偏移小的。」）；W1 之后只在介质损坏造出的残留上有对象，判决自己写了 | 同一池里逐档收到两个槽 1 而 W1 已生效 |
| V3 两张坏镜像与一条 io_min 8192 的层 0 流 | 站得住；比较键要含两个出口 | 两边「照同一句」判的键若只有「择到哪一槽」，第三节 3.6 的两个出口判不出谁对 | 同 3.6 |
| W1 mkfs 清零（用户定） | 站得住；原句缺次序与中途崩溃一句 | 清零范围 [0, 根环基址) 恰盖住逐档的全部 2033 档加槽 0（逐档上界是根环基址减 4096，`.claude/kb/decisions/22-单元原子性怎么合成.md:333`「判固定结构槽距的上界用同一槽自述的根环起点」）；第三节 3.8 | 入库装置上 W1 之后仍有旧池槽被逐档收到（介质损坏之外） |

## 三、X1 展开：写成原句哪一句站不住

### 3.1 U1 与回退：「必须在」读成状态下标

判决句（`research/prompts/m3-formal-r5-main-verification.md:21`「fsync 返回成功之后，这次 fsync 等到的那次发布带走的全部操作必须在」）按「操作在不在恢复出的状态里」读，在这段历史上为假：写 a、覆盖写 b、fsync 返回成功（lo 在 b）、管理员回退到 a 那一版的根、回退根持久、崩溃。回退是一次普通的向前发布，根持久之后才向管理员确认（`.claude/kb/decisions/23-journal的角色与格式.md:389`「回退的根持久之后才向管理员确认」），候选集只看实例表、F、带文件，不看是不是已确认（`.claude/kb/decisions/23-journal的角色与格式.md:381`「被崩溃恢复抛弃的时间线上的根一条都选不中」是候选集的排除项）。恢复读回 a 的内容合法，b 不在，字面读法判红 [推]。
按「S 序列按发布顺序存状态、允许集合是下标不早于 lo」读则成立：回退是 S 序列里的一个非前缀的状态转移（岔路单的记号 S_j =「前 j 个操作施加出来的状态」对回退不再成立，这一点 G2 的句子也要跟着写）。今天的历史里没有 fsync（`grep -ci fsync crates/singlefs-harness/src/history.rs` 输出 `0`），所以今天没有样本；回退这一步在 BROAD 比重里每步 10（`crates/singlefs-harness/src/history.rs:568`「(HistoryOperationKind::RollBackWhileMounted, 10),」），加上 fsync 之后这段历史马上抽得到。
推翻我：D23 或 D16 里有一句写明回退不许越过已 fsync 的发布。

### 3.2 U5 的 txg 等式

辩方式子是 `txg(R2) = Tm + n`、n = 挂载录制流里的根槽写条数，不精确时退成 `txg(R2) ≥ Tm + 2`（`research/prompts/m3-formal-r5-sonnet-output.md:159`「不精确时退成 txg(R2) ≥ Tm + 2」）；判决取了前一半（`research/prompts/m3-formal-r5-main-verification.md:25`「取辩方的式子（推的）」）。两处不稳：
- D23 写的是不等式并认下跳号（`.claude/kb/decisions/23-journal的角色与格式.md:370`「代价：跳号比今天大（最多到被抛弃时间线的长度）」）。等式「首个根的 txg = Tm + 1」比 kb 严，kb 放行的实现（跳号）会被判红 [读]。
- n 不是常量。第二轮攻方 um 窗里同一个动作 `mount`：R1 在 (7, 2) 时 R2 落在 (10, 4)（`research/prompts/m3-formal-r5-opus-model/u5-um-r2.log:22`「chosen=Some((7, 2))」、`research/prompts/m3-formal-r5-opus-model/u5-um-r2.log:23`「chosen=Some((10, 4))」），R1 在 (8, 3) 时 R2 也落在 (10, 4)（`research/prompts/m3-formal-r5-opus-model/u5-um-r2.log:50`「chosen=Some((8, 3))」、`research/prompts/m3-formal-r5-opus-model/u5-um-r2.log:51`「chosen=Some((10, 4))」）。txg 差是 3 与 2，「写行加两次暖机」写成常量 3 对第二个态为假；为什么差 2 我没查（猜与暖机要覆盖两块盘有关，推）。
这个日志里没有 txg 这一项的比较行（`main.rs` 的 `compare` 没有），所以这条关系式零样本。改成「R2 的首个 txg ≥ Tm + 1、末个 txg 等于挂载交回的最后一个根的 txg」不会丢判别力（推）。推翻我：今天的实现上 R2 的首个 txg 恒为 Tm + 1 且 D23 把「≥」收成「=」。

### 3.3 G1 与故障注入的说谎档

U1 句末写了「带故障的状态（说谎的设备、丢整块盘 C519）不适用」（`research/prompts/m3-formal-r5-main-verification.md:21`「带故障的状态（说谎的设备、丢整块盘 C519）不适用」），而 G1 的调用点包括故障注入（`crates/singlefs-harness/src/fault_injection.rs:2353`「crash_recovery_disagreement(&allowed, newest_persisted, &read_back)」），「任一红即红」没有说 RefFS 在说谎档投什么票。
- O1 一侧今天对说谎档有两道豁免：`failed_reopen_is_the_right_answer`（`crates/singlefs-harness/src/fault_injection.rs:2159`「fn failed_reopen_is_the_right_answer(」）在说谎时把「重开走读失败」判成正确答案；`left_by_the_lying_device`（`crates/singlefs-harness/src/fault_injection.rs:2061`「pub fn left_by_the_lying_device(」）把被吞的写造成的违例豁免。但后一道对 `ColdStartReadBack` 一类分歧返回 false（`crates/singlefs-harness/src/fault_injection.rs:2113`「ModelDisagreementAspect::ColdStartReadBack」，下一行 `crates/singlefs-harness/src/fault_injection.rs:2114`「ModelDisagreementAspect::NotModeled => false,」）：说谎档下恢复到更旧版本的判红今天不被豁免，只靠「③ 的入参取自真落盘的写」不误红（`crates/singlefs-harness/src/fault_injection.rs:2271`「let all_persisted = vec![true; writes.len()];」，录制流只含真落到盘上的写）。
- RefFS 的 lo 取历史里「最后成功的那一步」，不看盘上落没落；被吞的恰是根槽写时，若恢复没有被记录兜住，读回比 lo 旧，RefFS 红、O1 绿，合取判红，而且这一类红不进豁免 [推]。
- 今天够不着：根槽住单块盘（`crates/singlefs-harness/src/fault_injection.rs:309`「区域 r 的根槽住哪块盘由系统配置的」…），单次吞写时那次发布的记录与单元还在，恢复由记录重建出同一版（第二轮攻方对这一格的结论同向，`research/prompts/m3-formal-r5-opus-output.md:294`「故障注入调用点上 RefFS-Q 被说谎的设备逼出假红」，单次抽样、没造出）。所以这一条是「句子缺一格」，不是「今天误红」。
补法（零轮）：G1 写一句「说谎档下 RefFS 那一票弃权，O1 判官照旧」。推翻我：造出说谎档下 RefFS 与 O1 判官同时判绿、而读回比 lo 旧的态。

### 3.4 U6 的「拒绝合法」四条件：第一个条件今天接不上

判决句：「拒绝合法 ⇔ 拒的理由在 D23 已定项 14 那张单子上、且由 O1 判官在同一份镜像上复算同样判拒，且盘上逐字节不变、只读读出在允许集合里」（`research/prompts/m3-formal-r5-main-verification.md:26`「由 O1 判官在同一份镜像上复算同样判拒」）。
- O1 的判官只收三样：版本目录、盘上最新根、读回（`crates/singlefs-harness/src/model.rs:2246`「pub fn crash_recovery_disagreement(」，参数在 2247–2249 行：`committed_versions`、`newest_persisted_root`、`read_back`），没有镜像入口。
- O1 的拒绝答案是照脚本里的操作答的：`answer_mount_writable_with_the_newest_root_unreadable`（`crates/singlefs-harness/src/model.rs:1276`「pub fn answer_mount_writable_with_the_newest_root_unreadable(」）的文档写「执行器的 `HistoryOperation::CrashRecoveryAbandoningTheNewestRoot`」（`crates/singlefs-harness/src/model.rs:1267`「崩溃恢复抛弃根（执行器的」…），它读的是模型状态，不读崩溃镜像里系统配置槽的字节。
- 崩溃注入今天的写法自认这一点：单元区墙之外的拒绝一律报红（`crates/singlefs-checker-tier/src/crash_injection.rs:1558`「崩溃后镜像上的可写挂载拒了：{member}」），原因是「而这里没有崩溃点上的模型状态可判区间」（`crates/singlefs-checker-tier/src/crash_injection.rs:1537`）。
- D23 单子的判据是盘上的：「有一槽读不出或本池自证不过 ⇒ 判不出、按真」（`.claude/kb/decisions/23-journal的角色与格式.md:391`）。从镜像复算它要读两盘四个系统配置槽，那是 checker 的读法，不是 O1 的；全池槽 0 不可用时这个读法就是 V1 逐档，两题在这里相接 [推]。
所以第一个条件写成「O1 判官复算」今天没有地方接；第二轮攻方量到的 90 个合法的拒全是第三态（`research/prompts/m3-formal-r5-opus-output.md:296`「没有第三态而挂载被拒：四段历史的 90 个被拒状态全是第三态。」），崩溃注入今天不取第三态，所以这句今天没有样本 [读]。
补法（零轮）：把「由 O1 判官…复算」改成「理由的判据由 checker 的系统配置读法在同一份镜像上复算为真」。推翻我：O1 加了一个吃镜像、按 D23:391 判据复算的入口。

### 3.5 G2「第 m 次已确认发布」逐格

G2 句子：「回退之后等于第 m 次已确认发布的内容」（`research/prompts/m3-formal-r5-main-verification.md:37`），m 由调用方从实现报的回退根翻出来。
| 格 | 定义得出吗 | 依据 |
|---|---|---|
| 回退被拒 | 得出：结局是输入，没有 m，RefFS 不判拒得对不对 | 拒的理由 O1 已有枚举（`crates/singlefs-harness/src/model.rs:302`「RollbackTargetOnAbandonedTimeline,」等四条回退拒因），候选集不合法归 O1 与 checker（判决自己的说法） |
| 回退之后又崩溃 | 得出，但 S 序列要按发布顺序存状态，不是操作前缀 | 崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同（`.claude/kb/decisions/23-journal的角色与格式.md:389`「崩在回退那次发布的任何一点，恢复落到的与任何一次普通发布没成时相同」）：允许集合 {回退前那一版, S_m}；回退根持久后是 S_m。回退是向前发布（`crates/singlefs-harness/src/history.rs:385`「管理员回退：挂着时的一次向前发布」），它自己的根也可以是以后另一次回退的目标，那时 m 指的是「回退发布」这一项，内容是 S_m |
| 目标是被抛弃时间线的根 | 得出的是「查不到」：候选集排除它（D23:381），合法实现拒；实现若接受并报了这条根，调用方的 (txg, 实例) → m 表里没有它 | 这条根属于从没确认、也没被恢复读出的发布，表里本来就没有。条款没写「查不到 m 时」怎么办：只能当作 O1 的拒因处理（`crates/singlefs-harness/src/model.rs:302`），RefFS 不投票。生成器会落到这一格：目标取「候选集里的与候选集外的（被抛弃时间线的、F 之下的、树表 0 条的）都会落到」（`crates/singlefs-harness/src/history.rs:344`） |
| 恢复之后接着写、再回退（U2 ② 之后的历史） | **定义不出** | 「已确认」收窄了合法目标：候选集只看实例表、F、带文件，不看确认（D23:381）。恢复读出的那一版可以来自「根已落盘、发布没返回」的发布，它不是任何一次已确认发布，却是合法的回退目标；m 还要跨「重新起头」数，是按哪一条时间线数没写。今天的历史走不到：崩溃是终点（第二节 U2 ② 一行） |
句子要补成「回退之后等于调用方版本表里那条根对应的版本，表项 = 每次发布返回成功时的版本 ∪ 每次恢复读出的版本」才覆盖最后一格（零轮）。推翻我：D23:381 的候选集被收成只含已确认发布的根。

### 3.6 V1 条款句的两个出口

判决句：「全池没有可用的系统配置槽 0 时，读者在格式允许的每个槽距上各读一槽…只收自证过、且自述的固定结构槽距等于所在偏移的槽」（`research/prompts/m3-c651-r1-main-verification.md:59`「只收自证过、且自述的固定结构槽距等于所在偏移的槽」）。实现每一档先过 `mountable_slot_or_refusal`（`crates/singlefs-core/src/recovery.rs:975`「match mountable_slot_or_refusal(」），再比槽距（`crates/singlefs-core/src/recovery.rs:986`「if recorded_spacing != candidate_spacing {」）。后果两条 [读]：
- 自证过、但池级字段这个读者不收（例：每区槽数 S 越界）的槽落在任一档上，整池拒，不是「不收」：文档写「试到的某一槽自证得过、每区槽数 S 越界」…「整池拒」（`crates/singlefs-core/src/recovery.rs:961`）。
- 自证过、incompat 位图不认识的槽：没有可择的槽时交回它，「不核偏移」（`crates/singlefs-core/src/recovery.rs:958`「（按偏移升序；它的槽距记在哪一格读不出来，不核偏移）」；分支在 `crates/singlefs-core/src/recovery.rs:997`「SystemConfigurationSlotReading::IncompatBitsNotRecognized(incompat_bitmap) => {」）。
句子写「只收…」读成「其余一律跳过」，与实现在这两个出口上不同。V3 要 checker、GPU 判器、O3 执行器三处「照同一句」，句子不写这两个出口，三处各按自己的读法写，判不出谁对。今天没有样本：第一版只有一版格式，mkfs 产不出这类槽，第二轮攻方也没造出（`research/prompts/m3-c651-r1-opus-output.md:178`「逐档撞上一个自证得过、池级字段这个读者不收的残留槽时整池拒」）。补法（零轮）：句子加一句「自证过而池级字段越界的槽整池拒（D22 已定项 16 第 5 句的越界即拒）；incompat 不认识的槽在没有可择槽时交回、不核偏移」。推翻我：D22 已定项 16 的越界即拒只指槽 0 与槽 1、不指逐档。

### 3.7 V1 出局与用户 2026-09-14 的「甲」

[读] `records/2026-09-13-总审核.md:441` 记：「槽 0 无效时按最小槽距 4096 读槽 1（甲，维持现状）」，收尾弹窗（同日），在该文件「十一·六」一节末。checker 的注释同一句：「全池的槽 0 都不可用时才按最小槽距 4096（2026-09-14 用户收尾弹窗定甲」（`crates/singlefs-checker/src/image.rs:564`）。这就是 V1-常量。
- 这件事不在 C651 这一轮的任何材料里：`grep -c '总审核\|定甲\|维持现状'` 对正文 `research/prompts/_m3-c651-r1-body.md`、判决、两份腿报告都输出 `0`。C651 的题面写「两种读法哪个为准没有条款定」（`.claude/kb/checks-owed.md:520`「两种读法哪个为准没有条款定」）；它说的是条款，用户的弹窗只落在 records 与代码注释里。
- 核心的逐档是后来进的：
```
$ git log --format='%h %ad %s' --date=short -S'slot_one_found_by_trying_every_slot_spacing_the_format_allows' -- crates/singlefs-core/src/recovery.rs | tail -1
45d49aaf 2026-09-27 里程碑二收尾：crates 代码与测试（实一至实八、实分一、代码审阅 A/B/C 批到 A4c）
$ git log --format='%h %ad' --date=short -S'2026-09-14 用户收尾弹窗定甲' -- crates/singlefs-checker/src/image.rs | tail -1
53c9f384 2026-09-14
```
  用户的甲在前（checker 那行注释 2026-09-14 进仓），核心的逐档在后（2026-09-27）。C651 的分歧是后者造出来的。
- 所以判决把 V1-常量判出局，程序上是推翻一条用户定案，而判决第五节第 1 条只把岔路 W 交了用户。实质上常量在 io_min 大于 4096 的单次崩溃上输（第一轮攻方 C 组，复跑坐实），这一点不变；我辩的是「该由用户翻」。
- 我没找到弹窗原话（会话暂存里的东西仓里没有）。推翻我：原话只问了「借别的盘槽距」那一档（同一段注释里「实审 A2a 第 37 条改的」是另一件事），没问全池槽 0 不可用的 4096。

### 3.8 W1 写零在 mkfs 中途崩溃时留下什么

[读] 今天 mkfs 的次序：先清根环三区与 journal 环（`crates/singlefs-core/src/make_filesystem.rs:525`「先把根环三个区域与 journal 环整段清零（每块盘每段一次调用、录制流各一步），再写元数据。」，调用在 554 行），再写实例表与树表单元并屏障，再写三条第 0 代根（`crates/singlefs-core/src/make_filesystem.rs:620`「device.write_at(offset, &root_slot, WriteDurability::ForceUnitAccess)?;」），最后每盘两个系统配置槽（`crates/singlefs-core/src/make_filesystem.rs:648`「device.write_at(offset, &slot, WriteDurability::Plain)?;」）加一道屏障。中途崩溃没有条款：「mkfs 中途崩溃第一版没有条款」（`crates/singlefs-core/src/make_filesystem.rs:541`）。
- 「读者照 V1 逐档还择得对吗」：中途态里「对」没有定义，所以这句问不出对错；W1 的主张只在 mkfs 做完之后（第一轮攻方 S 组 230/240 是做完 mkfs 再崩）。崩溃枚举也从 mkfs 之后起（`crates/singlefs-checker-tier/src/crash_injection.rs:740`「mkfs 占了流开头的几步（崩溃状态从它之后起）。」），且一次清零调用在录制流里只记一步（`crates/singlefs-core/src/block_device.rs:130`「mkfs 把 768 MiB 的 journal 环整环清零时，上层只发一次调用、录制流只记一步」），所以枚举里 W1 的写零不会被撕；真设备上 1 MiB 的写可以撕，那一格落在「没有条款」里。
- 今天的中途态比 W1 之后多一格：旧系统配置活到最后一步，所以有「新第 0 代根 + 旧系统配置」的态（同 fsid 时旧配置的几何会拿来读新根，推）；W1 把系统配置清零放进第一轮清零，这一格收掉 [推]。
- 半清（盘 0 清完、盘 1 没清）今天与 W1 之后都有：盘 1 的旧池槽 0 仍可用，② 借它的槽距，旧池可能以缺一块盘的形态被读（推，没造）。W1 既不引入也不解决这一格。
- 范围对得上：[0, 根环基址) 恰是逐档的全部档加槽 0；根环基址就是系统配置里那个读者只收一个值的字段（`crates/singlefs-core/src/system_configuration.rs:244`「pub root_ring_base_slot: SlotNumber,」）。句子是站得住的，缺的只是「中途崩溃不在射程」一句。

## 四、替第二轮被判出局的写法辩护

### 4.1 G1-照字面（RefFS ∧ ③）

判决的写法：「G1-照字面：RefFS ∧ ③，② ④ 那一类靠崩溃注入第二截与层 0 的 oracle 兜」（`research/prompts/m3-formal-r5-main-verification.md:36`「G1-照字面」那一格）。
- **其实不输的格一：今天落不了地。** O1 判官是一个函数、四条判据（`crates/singlefs-harness/src/model.rs:2234`「四条判据：」，`crates/singlefs-harness/src/model.rs:2236`「② 择到的根必须是模型提交过的某一版」，`crates/singlefs-harness/src/model.rs:2240`「④ 读回的内容要与模型记的那一版逐字节相同，那一版树表 0 条时只许报没有文件。」），没有只跑 ③ 的入口；三处调用点今天各调一次（`crates/singlefs-checker-tier/src/crash_injection.rs:902`、`crates/singlefs-harness/src/fault_injection.rs:2353`，二次崩溃那一处在 1828 行）。岔路单给 F2-合取列的代价是「RefFS 的判定在同样的位置并排加一次」（`research/prompts/m3-formal-r4-forks.md:55`），也就是原有的 O1 判官照旧、旁边多一次 RefFS。这样落地，「字面」与「全四条」是同一件事，0 行对 0 行；G1 的两个候选只在「将来把 ② ④ 从调用点拆走」时才分叉，判决比的是一个今天没人写的拆法 [推]。
- **其实不输的格二：6 态没漏。** 第二轮攻方量到的 6 态里层 0 的 oracle 与第二截都红（`research/prompts/m3-formal-r5-opus-output.md:87`「layer0=["journal_consulted_oracle:wrong_content"]」、`research/prompts/m3-formal-r5-opus-output.md:94`「谁判红了：O1 的 ② ④」那一段）；判决说「恰好」是对的，但这 6 态本身没有漏。
- **输的格：「没有更旧带文件版本」的 NoFile 一格。** 层 0 的 oracle 在「走到一条没发布过的更新的根上报没有文件」只在有更旧带文件版本时才判红（`crates/singlefs-harness/src/model.rs:2244`「有更旧的带文件版本时才判红，这一份认得每一条根有没有文件，两个方向都判得出。」）；O1 的 ④ 两个方向都判（`crates/singlefs-harness/src/model.rs:2291`「(None, Some(content)) => Some(ModelDisagreement::new(」）。起点之后第一次发布第一个文件的窗里（窗前没有文件）：实现走到新根却报没有文件，RefFS 的允许集合含没有文件那一版→绿，③ 看根不看内容→绿，层 0 oracle 没有更旧带文件版本→绿，只有 ④ 红；第二截的读阶段是否经同一段读法我没查 [推，没造变异]。这一格字面读法放过，全四条不放过。
- 推翻我：造出「全四条红、字面绿、而第二截与层 0 都红」之外的态，那说明字面输得更多；或查实 ② ④ 在今天的三处调用点之外另有一处判同一件事。

### 4.2 G2-弃权（含回退的历史 RefFS 弃权）

- **其实不输的格：回退那一步及之后的窗。** 这些窗里 RefFS 在今天能多判的东西很少：下界 lo 来自 fsync，今天的历史没有 fsync（第三节 3.1 的 `grep` 输出 `0`），lo 恒在起点；上界今天没有判别力（第二轮辩方腿的结论，`research/prompts/m3-formal-r5-sonnet-output.md:89`「上界在今天的流上没有判别力」那一段）；形状上，今天的内容装不进一个以上数据单元（`crates/singlefs-harness/src/model.rs:288`「ContentExceedsDataUnitPayload,」是被拒的一种），P、Q、任意子集塌成同一个集合。而 G2-输入的 S_m 来自「调用方由实现报的回退根翻出的 m」，RefFS「不判目标选得对不对」（判决自己的话），它判的只剩「读回 ∈ {回退前, S_m}」，与 O1 的 ② ④ 是同一条信息 [推]。
- **输的格：粒度。** 判决写「含回退的历史」整段弃权（`research/prompts/m3-formal-r5-main-verification.md:37`「含回退的历史 RefFS 那一票弃权」）。回退比重每步 4%–10%（`crates/singlefs-harness/src/history.rs:555`「(HistoryOperationKind::RollBackWhileMounted, 4),」、`crates/singlefs-harness/src/history.rs:568`「(HistoryOperationKind::RollBackWhileMounted, 10),」），崩溃注入战役的历史长度有 8 步一档（`crates/singlefs-checker-tier/tests/crash_injection_campaign.rs:99`「operations_per_history: 8,」）：
```
$ python3 -c "print(round(1-0.96**8,3), round(1-0.9**8,3))"
0.279 0.57
```
  含回退的历史约占 28%–57%（每步独立的粗算，关着会话的步不抽回退，所以是上界；推）。整段弃权丢掉的是回退之前那些窗，那些窗里 RefFS 与回退无关。更窄的弃权（零轮）：从「查不到 m 的那一步」（第三节 3.5 最后一格）起弃权，之前照判。这不改用户定的 G2-输入，只改弃权这个兜底的粒度。
- 推翻我：同一个回退窗里，RefFS 的 S_m 与 O1 版本目录里那条根的内容不同的态——那恰是两份独立读法的分歧，RefFS 才有独立价值。

### 4.3 V1-常量

- **其实不输的格：槽距 4096 的池。** 今天所有现役测试的几何槽距都是 4096（第一轮攻方自己写 io_min 512 是「所有现役测试的几何」（`research/prompts/m3-c651-r1-opus-output.md:38`「所有现役测试的几何」）；C 组 io_min 512 的 4 段历史常量全对，`research/prompts/m3-c651-r1-main-verification.md:20` 的汇表「summary label=const group=C states=12 ok=4」）。在这些池上常量与逐档逐态相同，而常量代码少（core 删 78 行、GPU 判器删 32 行，减出来的；`research/prompts/m3-c651-r1-sonnet-output.md:142`「采常量」一行的行数账，推）。
- **其实不输的格二：旧槽距更大的残留。** 第一轮攻方的改法表第 3 行：用过的盘重做 mkfs、旧槽距更大（x > y）时，常量「不中，量过」而逐档「中（世代号决定），量过」。没有 W1 时常量在这一格优于逐档；有了 W1 两者在这一格都归零。
- **输的格：** io_min 大于 4096 的池单次崩溃撕了两盘槽 0：常量读 4096 的空档整池拒（C 组 y = 8192 / 65536），D2 已定项 19 让这种池合法（`.claude/kb/decisions/02-RAID条带策略.md:313`「槽距 = 4096 向上取整到 mkfs 时探测的 io_min 的整数倍」）。
- **程序上的格：** 第三节 3.7，用户 2026-09-14 定的甲。
- 推翻我：槽距 4096 的池上找到常量与逐档给不同结局的态（旧槽在 4096 之外且自述槽距等于所在偏移的残留除外，那是 W 那一题）。

### 4.4 W2（不清）

- **其实不输的格：新池。** 新池上的 C 组逐档 12 / 12 择对、0 行（`research/prompts/m3-c651-r1-main-verification.md:55`「新池上的 C 组（逐档 12 / 12 择对）」）。W1 的代价「凡枚举含 mkfs 那一段的用例都动，范围没数」：崩溃枚举从 mkfs 之后起（`crates/singlefs-checker-tier/src/crash_injection.rs:740`），W1 多的是 mkfs 前缀里每盘一步，枚举态数不变，要动的只有钉 mkfs 步数或流下标的用例；我没数它们（没数）。
- **输的格，而且比判决写的更多：** 判决 W2 一行用「fsid 每次 mkfs 随机时」（`research/prompts/m3-c651-r1-main-verification.md:55`）那一句减轻它（那一句接着写挂上旧池那一半要两次 mkfs 撞同一个 fsid，推的，没量概率）。代码里 fsid 是调用方给的、mkfs 自己不生成（`crates/singlefs-core/src/make_filesystem.rs:530`「// 而 fsid 是调用方给的」），harness 的场景用一个固定值（`crates/singlefs-harness/src/scenario.rs:43`「filesystem_identifier: E142_FILESYSTEM_IDENTIFIER,」）。「撞同一个 fsid」在 harness 里是常态，不是小概率；这个假设对 W2 没有出处，W2 在 harness 里输得更多。
- 推翻我：产品路径的 mkfs 入口自己生成随机 fsid（那是调用方之上的一层，今天 `crates/` 里没有）。

## 五、X2：清单完整性

### 5.1 RefFS 的输入白名单

背景材料列的四样：操作结局、回退目标 m、每次恢复读出的内容、显式发布点。
- **第五样：崩溃落在第几个操作里（hi）。** 判决自己提到 RefFS 要「(lo, hi)」（`research/prompts/m3-formal-r5-main-verification.md:23`「RefFS 的 (lo, hi)」），lo 可由结局加显式发布点推出，hi 不行：它说的是崩溃时哪一个操作在飞，「成 / 在任何写之前拒 / 写了之后报错」三种结局（`research/prompts/m3-formal-r5-opus-output.md:259`「每个操作的结局（成、在任何写之前拒、写了之后报错）」）里没有「在飞」。今天由 `step_containing` 给（`crates/singlefs-checker-tier/src/crash_injection.rs:751`）。不列它，RefFS 的允许集合没有上界输入。
- **第六样候选：故障域标志。** U1 的射程句要 RefFS 知道这是不是带故障、串行的状态；3.3 的「说谎档弃权」也要它。今天是调用点决定调不调 RefFS，不算输入也行，但那要写进条款，不能留在调用点的习惯里。
- 推翻我：hi 可以并进「结局」的第四种（在飞）而白名单仍是四样；那样条款要把「在飞」写成结局之一。

### 5.2 挂载透明逐字段表：根记录与系统配置的全部字段过一遍

根记录字段（`crates/singlefs-core/src/root_record.rs:52` 起）逐个对判决的表：
| 字段 | 判决里有行吗 |
|---|---|
| 池 fsid、卸载记号、实例代号、checkpoint_txg、树 ID 水位、F、实例表指针（行内容） | 有（U5） |
| 树表指针（inode 树、extent 树两根） | 有，推的 |
| 中央映射树根（`crates/singlefs-core/src/root_record.rs:63`「pub mapping_root: NodePointer,」） | **没有** |
| 树表 0 条那一版的分配记录树根（`crates/singlefs-core/src/root_record.rs:70`「pub allocation_record_tree_root: NodePointer,」） | **没有**（「分配记录」一行只比 POSIX 内容与池级 checker） |
| 算法类型、nonce、MAC | **没有**；第一版恒 0（`crates/singlefs-core/src/root_record.rs:98`「writer.put_u8(0); // 算法类型：未加密」、`crates/singlefs-core/src/root_record.rs:99`「writer.skip(12 + 16); // nonce、MAC：第一版留位全 0」），可以白送一行「必须相同且为 0」 |
| 墓碑树、谱系树、deleted_inodes 三根指针（树表条目里） | **没有**；回退那次发布里它们「随现行那一版」（`.claude/kb/decisions/23-journal的角色与格式.md:377`「墓碑树、谱系树随现行那一版」），只挂载更没有理由变 [推] |
系统配置：不可变配置与实例代号有行；运行量四个字段（`crates/singlefs-core/src/system_configuration.rs:314` 起：槽世代号 `crates/singlefs-core/src/system_configuration.rs:315`「pub slot_generation: u64,」、tail、journal 实例代号、F 副本）里槽世代号与 journal_tail 没有式子，判决写的是「推的」。第二轮攻方的 `compare` 也只比了不可变配置、设备号、F、卸载记号、水位、实例、行、分配计数（`main.rs` 里 `push("` 的全部名字），这几项一个态都没比过。
中央映射树根与分配记录树根大概率与分配记录同类（按调整后的分配器现算，回退那次发布就是这样写的，D23:377），应并进「分配记录」那一行并点名；墓碑 / 谱系 / deleted_inodes 三根与算法三字段是「必须相同」。推翻我：入库装置上这几项在只挂载之后变了而式子没罩。

### 5.3 C651：全池槽 0 都不可用还影响哪几步（都是读码推的，没造镜像）

槽 1 的位置之外：
- **tail 与 c_见证。** 可写挂载读两盘全部系统配置槽的 journal tail 取最大（`.claude/kb/decisions/23-journal的角色与格式.md:391`「两槽都自证得过时取全部槽 journal tail 的最大值 c_见证」）。全池槽 0 都被撕时只剩各盘槽 1，tail 是旧的一代，c_见证偏小；「系统配置没见证到最新根」那一形（同一行末句，「系统配置没见证到的最新根，「续」落地之后由下一次挂载在自己的 c_见证 里看到」）的覆盖面随之变宽 [推]。
- **F 的系统配置副本。** 系统配置里 F 另记一份，生效值取两处读得出的最大值（`crates/singlefs-core/src/system_configuration.rs:320`「读出来之后怎么用（恢复时生效值取两处读得出的最大值）不在这个模块」）。槽 1 给旧 F，生效值只在最新带新 F 的根也读不出时才偏低 [推]。
- **取号。** 新代号 = 各盘自证槽与根环里最大的实例代号 + 1，取号的写入集合与 max 的取值集合取同一个（`.claude/kb/decisions/18-块里携带什么信息.md:316`「取号的写入集合与 max 的取值集合取同一个」）。被撕的槽 0 若正是取号写，它烧掉的号丢了，下一次取号可能重发同一个号；安全与否取决于「取号写的屏障先于任何带新号的根」，我没核这一条（没核）。
- **下一次系统配置写落哪个槽。** 「下一次写的槽 = 世代号 mod 2」（`.claude/kb/decisions/22-单元原子性怎么合成.md:331`）：从槽 1 的世代号 g 往下写 g + 1，奇偶翻面，写回被撕的槽 0，自愈 [推]。
- 推翻我：这四步里任一步读的不是「每盘择到的那一份系统配置」而是别处另存的值。

## 六、X3：报出的数能不能核

### 6.1 取样的历史与网格：数是存在性见证，不是发生率
- C651 的 C 组 12 是「io_min 3 档 × n 4 档 = 12 段历史」（`research/prompts/m3-c651-r1-opus-output.md:54`「C 组（新盘，纯崩溃）：io_min 3 档 × n 4 档 = 12 段历史」），每段取一个两盘槽 0 同撕的态；S 组 240 来自 180 段（`research/prompts/m3-c651-r1-opus-output.md:55`「5 × 3 × 2 × 2 × 3 = 180 段」）。所以常量 8 / 12 拒、逐档 52 / 240 择旧这类比例由网格里放了哪几个 io_min、哪几对 x → y 定，换一份网格就换一个比例。稳的只有「哪个候选在哪个格上错」，不是比例；入库装置上重做，比的是格的结局。
- 第二轮 U3 的 6 态是「每窗 3 个」：恰好是记录两份镜像的三个非空持久子集（任一份自证过即在，`.claude/kb/decisions/23-journal的角色与格式.md:364`「任一份自证校验和过即在」）乘「根槽没持久」，数由两份镜像的结构定，不由操作数定 [推]。

### 6.2 RefFS-Q 替身会把数推向哪边
替身取这一窗前后整池持久的镜像上的只读恢复当窗前窗后内容（`research/prompts/m3-formal-r5-opus-output.md:303`「RefFS-Q 是替身：窗前、窗后两份内容取自这一窗前后整池持久的镜像上的只读恢复」）。
- 方向一：实现在干净路径上的读错会同时进替身与读回，替身偏绿（偏向少判红）。
- 方向二：窗里只有一个改内容的操作（同一行「这几段历史上一窗只有一个改内容的操作」），内容装得进一个数据单元（装不进的写被拒，`crates/singlefs-harness/src/model.rs:288`「ContentExceedsDataUnitPayload,」），P、Q、任意子集塌成同一个二元集合。所以「Q 把合法状态判坏（今天的历史上）：RefFS-Q 替身在干净构建上一次都没红」（`research/prompts/m3-formal-r5-opus-output.md:291`；rb 4 108 态、ow 最后一段 48 588 态）只对「二元集合」有证据，对 Q 在多单元写上的边界没有证据。
- 对 6 态的结论方向稳：真 RefFS 的 {S_lo, S_hi} 同样含窗前内容，替身偏绿不影响「RefFS 绿」这一点。

### 6.3 U5 的数
- um 窗（卸载之后的那次可写挂载）：16 390 / 24 598 是 R1 带记号的份额，由窗的段形定；存在性稳，份额不稳。
- fms 窗：前缀关了空间准入造（`research/prompts/m3-formal-r5-opus-output.md:302`），产品准入下推抬 F 一次没扫到（`research/prompts/m3-formal-r5-opus-output.md:295`），盘宽缩成 256 槽（`research/prompts/m3-formal-r5-opus-output.md:301`「盘宽缩了：384 槽（环 6 MiB）与 fms 的 256 槽，不是快档的 4 GiB」）。20 496 / 12 309 随单元区槽数变（推），而且「推了」那一半今天只有测试开关能走到。
- U6 的 3 态是 U3 那个坏实现的副产物（`research/prompts/m3-formal-r5-opus-model/u5-rb-mut.log:11`「RowPublishAdmissionRefusedBeforeAcquisition」），不是独立取样；同一个量要在入库装置上用同一个坏实现（变异开关）才重得出来。

### 6.4 入库装置上「同一个量」怎么定（建议，推）
键 = (历史构造, 被枚举的那一步, 盘宽档, 动作)，值 = (字段, 判定类)，判定类取「恒真 / 存在红态 / 计数」。
- 数由段形定的窗（um、s1、s5、rb、ow）：任一盘宽都行，但登记盘宽；比判定类，计数只在同一窗同一历史构造下比绝对数。
- 数随盘宽与准入变的窗（fms）：同盘宽、同准入开关才算同一个量。
- C651：同一 (io_min, n, 崩溃相位) 格的结局，不比 12 与 240。
- 第一轮攻方的 O1 / RefFS 对比里「3 态」「6 态」要在入库装置上重做时，用同一个坏实现开关，不另写一个。

## 七、没做什么

- 没改用户定的 F1–F6、G1、G2、W1，没替主 agent 撤销或维持判决；只交复核结果、句子要补的地方与会推翻它的现象。
- 没量任何数，没编译、没跑任何测试、没写模型（`research/prompts/m3-formal-r6-sonnet-model/` 没建）。引的数都是前两轮攻方在副本上量的，进 kb 之前要在入库装置上重做。
- 没读第一轮两份腿报告 `research/prompts/m3-formal-r4-sonnet-output.md`、`research/prompts/m3-formal-r4-opus-output.md`，没读背景材料附录里的 kb 抄录。
- 没查到的：用户 2026-09-14 弹窗原话（会话里，仓里只有一行）；3.2 里 R1→R2 的 txg 差为什么是 3 与 2；取号写的屏障是否先于任何带新号的根（5.3）；W1 动到多少个钉 mkfs 步数的用例（4.4）。
- 没造变异验：4.1 的 NoFile 一格、3.3 的说谎档、3.1 的回退历史；它们都是读码推的。
- 草稿目录 `/tmp/claude-1000/m3-formal-r6-defense/` 里只有核引文行号的小脚本 `q.py` 与几份 `pairs*.tsv`，没有编译目录与仓副本，没有要删的。

## 八、cite-check 末行

`python3 research/scripts/cite-check.py research/prompts/m3-formal-r6-sonnet-output.md --root /tmp/claude-1000/m3-formal-r6/tree --root . --background research/prompts/_m3-formal-r6-background.md` 末行（交回前最后一次跑）：

```
  ✓ 核了 134 处引文，对上 134 处，没判 0 处（逐处列在上面）
```
