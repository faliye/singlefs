# m3-close-g4-rewrite-r1 正推腿（Sonnet）报告（2026-10-22）

立场：正推。只从 `crates/` 今天的代码与已定条款推「应该是什么」，再与判官、红测试逐格比。不读攻方腿这一轮的任何产物。
没跑编译与测试（派发提示：不起编译与测试）；凡写「没跑」的都是读码推的。开工快照核过：`mount.rs`、`mount_transparency.rs`、`crash_injection.rs`、`make_filesystem.rs`、`mutations.tsv` 五个文件此刻的 sha256 与 `research/prompts/m3-close-g4-rewrite-r1-snapshot/code-sha256.txt` 逐个相同。

复跑（算术核，纯 python，不碰仓）：`nice -n 19 python3 -I ~/.local/share/singlefs/work/m3-close-g4-rewrite-r1-sonnet/mkfs_last_segment_states.py`
- 脚本 sha256：`51660de3fc15507fa8d158c79abb10667543e7a8e2bcb9cdf0d6fc844a43321c`
- 输出 sha256（`mkfs_last_segment_states.out`）：`eccaab463379e52bd187b44d9b4ae0cb34bba1aceab8f2ec64572a3eb845e4ac`

名字约定（本报告自己用的，不是仓里的）：
- 盘面全好：每块盘两槽里至少一槽本池自证过。
- 有盘无槽、闸过：某块盘一槽都没有，补写闸五件都过（核心层今天挂得上，补写）。
- 有盘无槽、闸不过：某块盘一槽都没有，补写闸有一件不过（核心层今天拒）。

## 一、V1：只有崩溃、没有读故障的历史，留不留得下「某块盘两槽都自证不过」

结论：分两段。
- 从 mkfs 之后起的历史（崩溃注入第一、二、三截的全部候选段）：留不下。取号两轮、发布末尾轮换、抬 F、补写本身，每一处系统配置槽写之前，同一块盘上前一个槽写都已过屏障，同段同盘至多一个槽写，所以崩溃至多撕坏在写的那一槽，另一槽留着。
- mkfs 自己的崩溃（新盘与用过的盘都算）：留得下，三处，逐条在下表。它们不进崩溃注入第二、三截，由 mkfs 自己的崩溃枚举流按 mkfs 中途崩溃的条文判，那一条流只做只读恢复、不做可写挂载。

### 正推链

条款（整行抄，行号在 kb 文件里现查）：
- `.claude/kb/decisions/22-单元原子性怎么合成.md:181`：「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:181`：「作废持久之前崩溃：旧池读回最后确认的一版；可写挂载的结局按现行补写条款」
- `.claude/kb/decisions/22-单元原子性怎么合成.md:181`：「后一种里系统配置的冗余可以缺，缺新槽的那块盘的可写挂载按现行补写条款处置」
- `.claude/kb/decisions/13-验证路线.md:187`：「「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4 判」
- `.claude/kb/decisions/13-验证路线.md:187`：「mkfs 返回之前的崩溃态不在 G4 的射程」

代码里的步：
- 写槽选哪一槽：`crates/singlefs-core/src/transaction.rs:592`：「let slot_generation = verified_on_this_device」；世代号取这块盘上自证过的最大者加一（同文件 596 行 `.unwrap_or(0)` 之后加一，两槽都读不出时从 1 起），落的槽是世代号对 2 取余：`crates/singlefs-core/src/transaction.rs:643`：「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」
- mkfs 清零段：`crates/singlefs-core/src/make_filesystem.rs:625`：「device.write_zeroes_at(DeviceOffsetInBytes(0), root_ring_base.0)?;」，两盘各一写，之后各一道屏障：`crates/singlefs-core/src/make_filesystem.rs:629`：「device_whose_system_configuration_area_was_zeroed.barrier()?;」
- mkfs 末段：每盘两槽都写世代号 1、Plain、同段无屏障，屏障在全部槽写之后：`crates/singlefs-core/src/make_filesystem.rs:732`：「device.write_at(offset, &slot, WriteDurability::Plain)?;」（循环在 728–734，屏障在 735–737）
- 取号两轮：`crates/singlefs-core/src/transaction.rs:1178`：「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」是第一个取号写之前的屏障；`crates/singlefs-core/src/transaction.rs:1235`：「for _round in ACQUISITION_WRITE_ROUNDS {」每轮之后一道屏障（1254 行）
- 发布末尾轮换：`crates/singlefs-core/src/transaction.rs:1584`：「/// 发布的最后三步（D16（发布语义） 已定项 7 的持久顺序）：根槽 FUA 写 → 系统配置槽轮换 → 屏障，屏障完成才返回。」
- 抬 F 与回卷各自在第一个槽写之前一道屏障：`crates/singlefs-core/src/transaction.rs:801`：「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」；`crates/singlefs-core/src/transaction.rs:693`：「if let Err(barrier_error) = self.perform_commit_step(CommitStep::Barrier) {」
- 这条性质有现成的录制流用例钉着：`crates/singlefs-harness/tests/system_configuration_slot_is_overwritten_only_after_a_barrier.rs:7`：「判据按录制流量：mkfs 之后切段（屏障切段），没有一段里同一块盘上有两次系统配置槽写。」（同文件 2 行写明发布末尾那一处靠发布自己的屏障自然成立）
- 崩溃注入候选段不含 mkfs：`crates/singlefs-checker-tier/src/crash_injection.rs:4777`：「起点那一段（取号、暖机、第一个文件）在内——2026-09-20 才放开；mkfs 那几段不在内，与崩溃点重放同一条界：」

### 逐条历史

| 历史 | 崩溃点 | `crates/` 里的步 | 留得下无槽盘？ |
|---|---|---|---|
| 新盘或用过的盘上 mkfs，崩在末段（四个槽写，同段无屏障） | 某盘两个槽写都没整落（未落或撕裂），另一盘至少一槽整落 | `crates/singlefs-core/src/make_filesystem.rs:728-737` | 留得下。算术：每写取未落、整落、撕裂三态，81 态，其中 65 态至少一槽整落（与 `crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs:309` 的「8, 65」里的 65 同数），65 态里恰有一盘无自证槽的 40 态（盘 0、盘 1 各 20）。脚本输出见上。其余 16 态全池无槽（整池拒）。算术只用了「前面各段全落」，65 与用例里写死的 65 相同，是相容、不是独立证明 |
| 用过的盘上 mkfs，崩在清零段（作废之前） | 一盘的 [0, 根环基址) 清零整落、另一盘未落或撕裂 | `crates/singlefs-core/src/make_filesystem.rs:625`（清零）、`crates/singlefs-core/src/make_filesystem.rs:629`（作废屏障） | 留得下：整落那块盘旧池的两槽被清、另一盘旧池的槽还在，恢复读回旧池最后确认的一版（用例里这类态数 8）。推的，没逐态量 |
| 用过的盘上 mkfs，崩在作废之后、末段之前 | 两盘清零都持久 | 同上 | 留不下「某盘无槽而别盘有」：两盘都无槽，整池拒（`NoValidSystemConfiguration`），不是本题的盘面 |
| mkfs 之后取号里覆写较旧槽的那一轮 | 该轮内某盘的槽写撕裂 | `crates/singlefs-core/src/transaction.rs:1235` 起 | 留不下：写的是较旧那一槽（世代号加一的余数），另一槽早已过屏障 |
| mkfs 之后取号里覆写最新槽的那一轮、发布末尾轮换、抬 F、取号回卷 | 同上 | `crates/singlefs-core/src/transaction.rs:1254`、`1584`、`801`、`693` | 留不下，理由同上 |
| 补写本身（有盘无槽的盘上取号） | 补写的先一轮（落槽 1）撕裂 | `crates/singlefs-core/src/transaction.rs:920`：「if devices_rewritten_by_the_acquisition.contains(&slots_of_this_device.device) {」放行，世代号从 1 起落槽 1，后一轮世代号 2 落槽 0 | 不新增：先一轮崩回的是起点同类盘面；后一轮崩，槽 1（世代号 1）已过屏障，这块盘有槽。留下的无槽盘全部沿袭自 mkfs 那两处 |
| mkfs 末段只整落了「槽 0」的盘，之后挂载取号里覆写较旧槽的那一轮 | 该轮覆写槽 0 时撕裂 | 世代号 1 的唯一自证槽在槽 0；写的世代号是 2，落槽 `2 % 2 = 0`，正好覆写这唯一一份（`crates/singlefs-core/src/transaction.rs:643`）。另一形（只整落槽 1）世代号 2 落槽 0，安全 | 留得下，推的，没造镜像。沿袭自 mkfs 末段，不在 mkfs 之后的候选段里 |

补写闸五件在这类盘面上怎么判，见 V2：mkfs 末段那一类盘面所选那一版是第 0 代，单元清单是实例表与树表（`crates/singlefs-core/src/mount.rs:1173`：「树表 0 条的一版是根记录直接指着的几个单元：」），清单非空，闸一能过。所以那一类盘面核心层今天往下挂、补写，不再拒。

推翻 V1 的观测：
- 在 mkfs 之后的候选段内摆出一个没有读故障的崩溃态，某块盘两槽都不自证（比如一份崩溃枚举里出现同段同盘两个槽写，或一道缺了的屏障）。上面那条录制流用例会先红。
- mkfs 末段 65 态里某一态在可写挂载上被拒而闸五件都过（那说明闸一对第 0 代的清单不成立）。

## 二、V2：判官 G4 那一臂在丙' 之后对不对，要改的候选

结论（三句）：
1. 在今天的调用域内（崩溃注入第二、三截，没注入读故障，盘面取在 mkfs 之后）这一臂判得对：V1 说那一域里没有「某块盘无槽」的盘面，核心层又只有这一类盘面才会以这一拒因拒，所以那一域里这一拒因一律不合法，与判官今天的结果一致。
2. 这一臂的依据句过宽，且丙' 之后它不再罩住 D13 写的那条性质：「没有读故障的历史留不下它」对 mkfs 中途崩溃不成立（V1），仓里 kb 已写明 mkfs 不在射程，代码注释没写；这一臂只在拒了才判，丙' 让「有盘无槽、闸过」的盘面不再被拒、改为补写后挂成，那类盘面从此一个判官也没人判。
3. 要改的话，罩得住的是「按盘面判」那一条候选（加在拒绝通道之外）；其余候选或是不改判定只改文字，或是在域内无差、域外还更差。判据带闸那条要把补写闸五件在判官里独立重写一遍，代价大、域内无收益。

### 今天的判法逐格

条款：
- `.claude/kb/decisions/13-验证路线.md:72`：「拒绝合法 ⇔ 拒因在 D23（journal 的角色与格式） 已定项 14 那张单子上，且照那一条的判据在崩溃镜像上自己复算成立，且盘上逐字节不变，且只读读出在允许集合里」
- `.claude/kb/decisions/13-验证路线.md:187`：「「无读故障的崩溃态上每块盘至少一份自证过的系统配置」这一性质由 G4 判」
- `.claude/kb/decisions/23-journal的角色与格式.md:396`：「这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着）」
- 上面三处与 `.claude/kb/decisions/22-单元原子性怎么合成.md:181` 的补写例外是同一件事的两头：D23 那一句没带例外（交叉引用缺），D22 写了例外。判官的「判据」一格取的是 D23 那一句的读法，没带闸。

代码：
- 判官 G4 臂：`crates/singlefs-harness/src/mount_transparency.rs:3064`：「// 改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在没有读故障的历史里留不下它。」；臂里调 `crates/singlefs-harness/src/mount_transparency.rs:3069`：「if system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)」
- 复算的判据：`crates/singlefs-harness/src/mount_transparency.rs:2411`：「.any(|slots| slots.self_verified.is_empty())」，没有补写闸五件；择不到系统配置也按真（同文件 2422 行起）。
- 判据为假另记一条：`crates/singlefs-harness/src/mount_transparency.rs:3173`：「.push(RefusalConditionNotMet::CriterionFalseWhenRecomputedOnTheCrashImage);」；合法只在没满足的条件为空时：`crates/singlefs-harness/src/mount_transparency.rs:3194`：「if conditions_not_met.is_empty() {」
- 所以对这一拒因、没注入读故障、被拒的挂载读的是不带故障层的崩溃镜像：判据为假记一条「判据为假」，判据为真记 G4 那一条，两头都不合法。G4 臂实际等于「这一拒因在没读故障的崩溃态上一律不合法」，判据复算只决定列哪一条。
- 调用点：第二截只在拒了才判：`crates/singlefs-checker-tier/src/crash_injection.rs:3498`：「WritableMountOutcome::MountRefused {」。第三截挂成走另一支：`crates/singlefs-checker-tier/src/crash_injection.rs:4459`：「WritableMountAlone::Mounted { pool_after, .. } => pool_after,」，只比读出内容；第三截只把带「判据为真」的那几条（G4、C670、Q3）报红：`crates/singlefs-checker-tier/src/crash_injection.rs:4743`：「criterion_true_without_read_faults.then_some((reason, conditions_not_met))」。

### 盘面类 × 读故障 × 挂载结局：今天的判定

「被拒」列只看拒因是读阶段判完之后系统配置又读不出的那一拒。

| 盘面 | 崩溃态 | 挂载结局 | 第二截今天 | 第三截今天 |
|---|---|---|---|---|
| 盘面全好 | 没注入 | 被拒 | 不合法（判据为假），报红 | 只计数，不报红 |
| 有盘无槽、闸过 | 没注入 | 被拒 | 不合法（G4），报红 | 不合法（G4），报红 |
| 有盘无槽、闸不过 | 没注入 | 被拒 | 不合法（G4），报红 | 不合法（G4），报红 |
| 有盘无槽、闸过或不过 | 没注入 | 挂成 | 不判这一性质 | 不判这一性质 |
| 盘面全好 | 注入过 | 被拒 | 不合法（判据为假），报红 | 只计数 |
| 有盘无槽、闸过 | 注入过 | 被拒 | 合法（判据为真，其余三条满足） | 只计数 |
| 有盘无槽、闸不过 | 注入过 | 被拒 | 合法 | 只计数 |

两处今天的缺口（都不是红测试的原因，是丙' 带出来的）：
- 缺口一：「有盘无槽、闸过、挂成」没有任何判官。丙' 之前这类盘面一律被拒，被拒就走 G4；丙' 之后它挂成，G4 那条性质（D13 写的「每块盘至少一份自证过的系统配置」）在这类盘面上不再被判。红测试自己的盘面就是这一类（盘 0 两槽撕裂、单元都在，调查员探针 `Mounted`，`~/.local/share/singlefs/work/m4-investigate-two-red-groups/report.md` 第一组 ③ 节）。
- 缺口二：「有盘无槽、闸过、注入过读故障、被拒」今天判合法，而核心层对这类盘面的正确做法是补写后挂成（被拒是实现错）。判据没带闸。域内这一格是空的（V1），所以只对手造盘面与坏实现有差。

### 候选：各自改 `crates/` 哪一步，与今天判得不同的盘面

| 候选 | 对实现的改动 | 与今天判得不同的格 | 要不要读补写闸五件 | 要不要与核心层共用代码 |
|---|---|---|---|---|
| 照旧 | `crates/singlefs-harness/src/mount_transparency.rs:3064` 的注释与 `mount_transparency.rs:2945-2948` 的枚举成员文档补一句：依据只对 mkfs 之后的盘面成立（D13 `.claude/kb/decisions/13-验证路线.md:187` 已写 mkfs 不在射程）；判定不动 | 无 | 不要 | 不涉及 |
| 收窄到「闸也不过」才判不合法 | `mount_transparency.rs:3069` 的条件加「且补写闸有一件不过」，U6 判据不动 | 「有盘无槽、闸过、没注入、被拒」：今天报红，改后判合法（判据为真，G4 不推）。这是退步：这类拒是实现错（该补写而没补），改后放过。另两格不变 | 要（五件） | 不许共用：见下 |
| 判据带闸 | `mount_transparency.rs:2419` 起那一个复算改成「判不出，或某盘无槽且闸不过」；G4 臂改调不带闸的那个谓词（`crates/singlefs-harness/src/mount_transparency.rs:2406`），照旧在没注入读故障的这一拒因上一律推 G4 | 「有盘无槽、闸过、注入过、被拒」：合法改不合法（补缺口二）；没注入那一行列出的条件多一条「判据为假」，红绿不变；闸不过的格不变 | 要（五件） | 不许共用：五件里闸一要所选那一版的单元清单（核心层 `crates/singlefs-core/src/mount.rs:5860` 的 `every_unit_of_the_version_is_on_the_device = !selected_version.units.is_empty()` 靠重建那一版得出），闸三闸四要扫环与单元区，判官要在崩溃镜像上另写一份；`.claude/rules/fs-design.md:35`：「运行时与 checker 必须用不同的算法。」 |
| 按盘面判（拒绝通道之外） | 在 `mount_transparency.rs` 里加一个公开函数，复用 `mount_transparency.rs:2406` 那个「某盘无自证槽」的谓词（它已在判官里，读槽走判官自己的 `system_configuration_slots_read_on_the_crash_image`）；调用点两处：第二截 `crash_injection.rs:3485` 的 `judge_the_writable_mount_on_the_crash_image` 在分流挂载结局之前、第三截 `crash_injection.rs:4426` 起的挂载之前，对没注入读故障的崩溃镜像直接判，为真就报红；加计数字段与变异行 | 「有盘无槽、闸过或不过、没注入、挂成」：不判改报红（补缺口一）；被拒的几格不变 | 不要 | 不涉及（谓词是判官自己的） |
| 撤掉 G4 臂 | 删 `mount_transparency.rs:3064` 起的臂与枚举成员 | 「有盘无槽、没注入、被拒」两格：报红改合法；`crates/mutations.tsv` 2406、2407、2408 三行作废 | 不要 | 不涉及 |

`crates/` 里「按盘面判」那个函数今天没有：`grep -rn 'device_without_any_self_verified_system_configuration' crates/singlefs-checker-tier/src` 零命中；`mount_transparency.rs:2406` 的谓词只在 `mount_transparency.rs:2440` 那一处被调。

### 域内外的差别

- 域内（崩溃注入第二、三截，没注入读故障，mkfs 之后的盘面）：V1 说「有盘无槽」的盘面是空集，所以上表里所有只在「有盘无槽」格上有差的候选，在真实历史上都没有差；它们只对手造盘面（红测试那一类）与坏实现有差。
- 坏实现的差：写槽协议被改坏、同一块盘上同段写两个槽（攻方先前量过的那一类），造出来的盘面里单元通常都在，落在「有盘无槽、闸过」。照旧、收窄、判据带闸三条在这类盘面上一个红都不出（挂成了，没人判）；只有「按盘面判」会红。这一句是推的：没拿坏实现跑。
- 「收窄到闸也不过」与「判据带闸」的问题同源：判官要回答「正确实现会不会补写」，而正确实现的答案取决于五件闸，判官不许复用核心层的算法，要自己独立写五件。闸一的单元清单是最难的一件。

### 推翻 V2 的观测
- 域内摆出一个「有盘无槽」的没读故障崩溃态（推翻 V1，随之推翻第 1 句）。
- 一份坏实现（同段写两个槽）在崩溃注入里被照旧臂抓到、且那份盘面单元齐全：说明挂载没把它补好，那就说明缺口一不存在。
- `crash_injection.rs:4459` 那一支以外，另有一处在挂成的盘面上判了 G4 那条性质（我读到的只有两处调用点，grep 命令：`grep -n 'judge_writable_mount_refusal_of_a_crash_state_knowing' crates/singlefs-checker-tier/src/crash_injection.rs`）。

## 三、V3：红测试走甲、乙还是别的，三条变异各由谁杀

结论：
1. 红的原因就一处：测试造的盘面（盘 0 两槽各翻一个字节、别的都没动）过得了补写闸五件，再挂补写后做成，测试前提「再挂被拒」不成立。证据：`crates/singlefs-checker-tier/src/crash_injection.rs:9032`：「.get(&RemountIdempotenceNotJudged::TheRemountWasRefused)」那一句断言的失败行是 9029 行起的 `assert_eq!`；调查员探针在 harness 档同一盘面上 `Mounted`，把闸恒设成不过之后变回 G4 那一拒（`~/.local/share/singlefs/work/m4-investigate-two-red-groups/report.md` 第一组 ③ 节）。
2. 在 V2 选「照旧」「收窄」「判据带闸」时走甲：保留这条测试、只让盘面过不了闸一，断言一个字不改。选「按盘面判」时走乙的形状：这条测试的原盘面变成「挂成且报红」的那一条，另造一条过不了闸的盘面钉第三截那条拒绝通道的 G4。乙与「按盘面判」的第二条测试，造盘面的办法就是甲。
3. 甲的盘面可以抄 harness 档现成做法：`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs:140`：「/// 这几条判的是「拒」那一面，所以让盘 0 少一份那一版的单元（补写闸一不过），取号之前的逐盘核照旧拒、拒因不变。」（helper 在同文件 142 行，两条 G4 用例 231、255 行各调一次）。变异 2406、2407 的杀者本来就在这个 harness 文件里，它们的盘面早已带这一步，丙' 没动它们。

### 三条变异今天与改完之后各由谁杀

| 变异行 | 今天 | 甲 | 乙 | 按盘面判（加在乙上） |
|---|---|---|---|---|
| `crates/mutations.tsv:2406`：「第29批 G4 判官整条不判」 | harness 用例 `a_refusal_after_the_read_stage_without_read_faults_where_a_rotation_tore_and_the_other_slot_of_device_zero_was_zeroed_turns_from_legal_to_illegal`（该文件 469 行；盘面带单元坏一字节，闸一不过），另一条 `..._where_device_zero_has_both_system_configuration_slots_torn_is_illegal`（417 行）也会杀（推的，没跑） | 不变 | 不变 | 不变；新增的「按盘面判」另要自己的变异行 |
| `crates/mutations.tsv:2407`：「第29批 G4 注入过读故障的崩溃态也判新拒因的判据」 | harness 用例 `the_same_refusal_after_the_read_stage_in_a_crash_state_with_read_faults_injected_is_legal`（445 行；盘面带单元坏一字节） | 不变 | 不变 | 不变 |
| `crates/mutations.tsv:2408`：「第29批 G4 只判第二截不判第三截」 | 红测试（`crash_injection.rs:8964`）本该杀它；丙' 之后测试自己红，杀不了。这一行在全仓只有这一个杀者：`grep -n 'remount_refusals_with_the_system_configuration_unreadable_criterion_true_without_read_faults' crates/singlefs-checker-tier/src/crash_injection.rs` 除计数结构体、合并、渲染（596、701、818、1039 行）外只在 4731（更新）、9050（断言）两处用到那个计数 | 同一条测试加上「盘 0 上第一个文件的树表那一份坏一字节」之后杀（盘面闸一不过，再挂照旧以 G4 拒，断言 (1, 1) 与「第三截记一条」不变；变异把 4730 行那一支换成 `{}`，计数为 (1, 0)、发现 0 条，断言红）。推的，没跑 | 原测试改成断言新行为，杀 2408 的是另一条新测试（甲的盘面）；2408 的最后两列要改指它的名字，不改这一行就无杀者 | 同乙 |

「测试的镜像在闸哪一件不过」：
- 今天的红测试：五件全过（盘 0 两槽翻字节、单元与环都在；闸五不适用，内存盘不报读错）。
- 甲：闸一不过（`crates/singlefs-core/src/mount.rs:5860`：「let every_unit_of_the_version_is_on_the_device = !selected_version.units.is_empty()」，盘 0 那一份树表字节与验得过的那一份不同）。别的件也能当手段（闸三要往盘 0 的环里写一条更新的记录，闸四要写一个更晚实例的单元，闸二要清掉那一版根在盘 0 上的槽），都比闸一多一步造盘面，闸五要读错、内存盘造不出。
- 乙里「挂成」那条：用今天的盘面，五件全过。

### 甲与乙差在哪

- 甲：改动最小（一条测试多一步造盘面加一句文档注释），2408 照旧由这一条杀。它测的是「第三截认 G4 那一拒」，不测补写。
- 乙多罩的：第三截的「挂成」那一支（`crash_injection.rs:4459` 起，读出内容与没崩过的那一次相同、`remount_idempotence_judged_by_phase` 记一次）在有盘无槽的盘面上今天没有 checker 档的用例钉；harness 档有补写用例（`crates/singlefs-harness/tests/acquisition_rewrites_a_device_that_lost_both_system_configuration_slots_only_through_the_gates.rs:159` 起那一条，补写后池级 checker 绿），但不过第三截。乙少罩的：若只改测试断言、不另造过闸的盘面，2408 无杀者。所以乙必须是两条测试。
- 甲的盘面和原盘面一样不是崩溃摆出来的（「手工各改一字节」那类），两条路在这一点上没有差别；域内造不出过不了闸的盘面（V1），所以这一类盘面只能手造。
- 测的东西与 V2 是否一致：V2 选「照旧」「收窄」「判据带闸」，甲的盘面（拒、判据为真、闸不过）同在三者的不合法一侧，一致。V2 选「撤掉 G4 臂」，2406、2407、2408 三行与甲都没有意义。V2 选「按盘面判」，原盘面的测试要断言新的红（挂成、没注入读故障、盘 0 无自证槽），甲的盘面留作拒绝通道的钉。
- 文档注释要改：`crates/singlefs-checker-tier/src/crash_injection.rs:8959` 起那一段写「盘 0 两个系统配置槽各改一个字节」，要补「盘 0 第一个文件的树表那一份另坏一个字节（补写闸一不过）」，否则读的人以为这一盘面没有补写就拒。

### 前提与没验的

- 甲的断言保持有一个没验的前提：盘 0 的树表一份坏了之后，只读恢复（`read_recovery_fields`）照样落到第一个文件那一版并读出内容（退到盘 1 那一份），第三截那条 `once` 才是 `Ok`，不然测试会走进「这一次没判」的另一支。harness 那一条同一做法下用的是 `fields.report.effective_root`，没有直接证明 checker 档的 `content_read_by_the_recovery_report` 同样 `Ok`。没跑。
- 这条测试在 checker 档的库单测里（`-p singlefs-checker-tier --lib`），按共用约束属重型，我不跑；要主 agent 跑的见文末。

### 推翻 V3 的观测
- 把甲的改动放进去之后，这一条仍红且红在「再挂做成」：说明单元坏一字节没让闸一不过（比如那一份落在闸一比的那份字节之外），要换一件。
- 变异 2408 在甲改完之后没被杀：说明第三截别处另有一处记同一计数。

## 四、V4：C670、Q3 两臂会不会因丙' 同样失效

结论：都不会。两臂的拒因都产生在补写闸之前，判据也不读「某块盘有没有槽」，丙' 没动它们的盘面，也没动它们被调到的位置。同形的只是接法（没注入读故障、判据复算为真就判不合法），盘面前提不同：G4 的前提是「无槽盘不会出现」，丙' 把这类盘面改成挂成，前提才失效；C670、Q3 的前提不依赖槽。

### 逐臂

| 臂 | 判据在什么盘面为真 | 核心层在哪一步拒 | 与补写闸的先后 | 丙' 之后还会不会被调到 |
|---|---|---|---|---|
| C670（拒因：判据 N-配置） | 见证值超前：自证过的槽里 journal tail 最大值，大于所选那一版那次发布末条的计数器（或末条读不出而该计数的记录比所选版新，或两条都读不出）。判官取槽时跳过自证不过的（`crates/singlefs-harness/src/mount_transparency.rs:2543`：「自证不过的槽（撕裂的、没写过的）跳过」，函数在 2546 行）。盘面例：轮换落了而根与记录没落 | 读阶段重读之后仍判真：`crates/singlefs-core/src/mount.rs:8100`：「return Err(MountError::NewerStateStillUnreadableAfterOneReread(」，读阶段在 `crates/singlefs-core/src/mount.rs:8352`：「) = read_stage_settled_at_most_on_the_one_reread(」 | 读阶段在前，取号之前的逐盘核（闸）在后：补写闸的调用 `crates/singlefs-core/src/mount.rs:6885`：「devices_without_the_selected_version(」在 `establish_instance` 里，该函数 `crates/singlefs-core/src/mount.rs:8490`：「establish_instance(」在读阶段之后才被调 | 会。判据为真的盘面照旧在读阶段被拒，到不了闸。无槽盘不改 c_见证：核心层与判官都只取自证过的槽 |
| Q3（拒因：重建分配器读根环整环重读仍一条有效根都没有） | 按最新那条根的实例表，根环里没有一条不被抛弃的自证有效根（盘面例：根环每槽清零） | 重建分配器那一遍：`crates/singlefs-core/src/mount.rs:3085`：「MountError::NoValidRootInTheRootRingOfTheRebuiltAllocatorAfterOneReread,」，调用在 `crates/singlefs-core/src/mount.rs:8458`：「let rebuilt = rebuilt_allocator(」 | 同在 `establish_instance`（8490）之前 | 会，理由同上。判据读的是根环，不读系统配置槽 |

判官里「Q3 与 G4 同形」那句只管接法：`crates/singlefs-harness/src/mount_transparency.rs:3076`：「// Q3 与 G4 同形：没有读故障时设备不报读错，读阶段择得到的根按它自己的实例表不被抛弃——这一拒的判据为真只能是盘面本身如此。」。Q3 的前提「没有读故障时，择得到的最新根按它自己的表不被抛弃」是实现的性质（`crates/singlefs-harness/src/mount_transparency.rs:2597` 一整段注释），不靠系统配置槽齐不齐。

### 域内是否另有被丙' 改了结局的盘面

- C670：判据为真的盘面里，有盘无槽且闸过的，核心层读阶段先拒，结局不变；判据为假的盘面，核心层今天挂成（补写），判官没有拒可判。派发提示与调查员报告里只有红测试这一条归到丙'，与此相容（我没跑别的测试，只读码）。
- Q3：盘面是根环被清，与槽无关。
- 但第三截里 Q3 与 C670 那两支有一处不对称：C670 有数条第三截用例钉着（`crates/singlefs-checker-tier/src/crash_injection.rs:8877` 起那一条，9141 起「通过调用点」那一条等），Q3 那个第三截计数除结构体、合并、渲染外（`grep -rn 'remount_refusals_with_the_no_valid_root' crates` 只命中 `crash_injection.rs` 一份文件）只在更新处（`crash_injection.rs:4735`）出现，没有测试断言它，`crates/mutations.tsv` 里也没有一行点名它：Q3 在第三截的判别力是空的。这不是丙' 造成的，是这一轮看到的另一处缺口，不在本轮出口里，只列出。（推的：没逐行读完 `mutations.tsv` 全表，只 grep 了上面两个名字。）

### 推翻 V4 的观测
- 读阶段与取号之前的逐盘核换了先后（核心层 `mount.rs` 里 `establish_instance` 之前就调了补写闸）。
- 判官里 C670 或 Q3 的复算去读了「某盘有没有槽」：今天 `mount_transparency.rs:2546` 与 `2601` 两个函数都没有。
- 一份注入读故障之外的历史里，某块盘被补写之后读阶段的判据 N-配置 变了真假：补写写的 tail 是取号那一刻的见证值（`crates/singlefs-core/src/transaction.rs:1190`：「let witnessed_journal_tail =」），按构造不抬它。

## 五、一致与冲突

规则与实现里读到的几处不一致，整行并排：
- D22 那一头（补写例外已写）：`.claude/kb/decisions/22-单元原子性怎么合成.md:181`：「某块盘两槽全废（一份本池自证过的系统配置都没有）时，取号那一步补写它的两槽，要这几样都成立，有一样不成立就拒」
- D23 这一头（没带例外）：`.claude/kb/decisions/23-journal的角色与格式.md:396`：「这一步在某块盘一份自证过的系统配置都没有时也拒（用户 2026-10-03 定留着）」
- 池级 checker 的 I-8.5 条文与注释还写「会拒」：`crates/singlefs-checker/invariants/03-journal.md:12`：「单盘全废不判违例，走旁路报告：另报哪块盘少于两槽自证过、可写挂载会拒。」；`crates/singlefs-checker/src/image.rs:350`：「/// I-8.5（V-池）：全池至少一块盘有一个自证过的系统配置槽，全池皆废判红。单盘全废、哪块盘少于两槽自证过（可写挂载会拒）」；对面是 `crates/singlefs-core/src/mount.rs:5794`：「devices_rewritten_by_the_acquisition.push(device);」（过闸的盘不拒，补写）。
- 判官与测试文档注释里的依据句没带 mkfs 例外：`crates/singlefs-harness/src/mount_transparency.rs:3064`：「// 改法 G4：没有读故障时设备不报读错，某块盘两槽都自证不过只能是盘面本身如此——正确实现在没有读故障的历史里留不下它。」；对面 `.claude/kb/decisions/13-验证路线.md:187`：「mkfs 返回之前的崩溃态不在 G4 的射程」。

这四处里 D13 那一句和判官的两头，不矛盾于 V2 结论「域内判得对」，是文字漏了限定。D23 与 I-8.5 两处是丙' 之后没跟着改的旧句。

## 六、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| V1 mkfs 之后的历史（取号两轮、发布轮换、抬 F、回卷、补写本身） | 一致（留不下） | 每个槽写之前同盘前一个槽写都已过屏障，同段同盘至多一个槽写；有录制流用例钉着 |
| V1 mkfs 自己的崩溃（末段四个槽写同段；用过的盘上清零段只落一盘） | 冲突（判官依据句说留不下，其实留得下） | 末段 81 态里 65 态至少一槽整落，其中 40 态恰一盘无槽；mkfs 单槽盘上取号可能覆写唯一一份（推的）；都不在第二、三截域内 |
| V2 G4 臂在第二、三截调用域内 | 一致（判得对） | 域内无「有盘无槽」盘面，这一拒因一律不合法 |
| V2 G4 依据句 | 冲突 | 判官、测试注释没带 mkfs 例外，D13 已写 |
| V2 G4 在丙' 之后罩不罩 D13 那条性质 | 冲突 | 只在被拒时判；有盘无槽且闸过的盘面挂成，没人判；红测试盘面就是这一类 |
| V2 判据与 D23 已定项 14 的拒因句 | 冲突 | D22 写了补写例外，D23 与判官判据没带，判据为真不等于正确实现会拒 |
| V2 候选 | 推荐「照旧加按盘面判」 | 收窄是退步，判据带闸要在判官里独立重写五件，撤臂放过实现错；「按盘面判」只用判官已有的谓词，不读闸 |
| V3 路线 | 甲（V2 取照旧、收窄、判据带闸时）；乙的形状（取按盘面判时） | 甲只改一条测试的盘面与注释，2408 仍由它杀；乙必须有两条测试，否则 2408 无杀者 |
| V3 变异 2406、2407 | 一致 | 杀者在 harness 档的 G4 用例，盘面早已带闸一不过，不受影响（没跑） |
| V3 变异 2408 | 冲突（今天无杀者） | 全仓只有红测试一个杀者，丙' 让它红，甲或乙之后重新有杀者 |
| V4 C670 | 一致（不失效） | 拒因在读阶段，早于补写闸；判据只取自证过的槽 |
| V4 Q3 | 一致（不失效） | 拒因在重建分配器，早于补写闸；判据读根环 |
| V4 Q3 第三截 | 规则没说（发现） | 第三截计数无测试断言，变异表无行点名；不是丙' 造成 |
| 池级 checker I-8.5 的旁路说明 | 冲突 | 仍写「可写挂载会拒」，丙' 之后过闸的盘不拒 |

## 七、要主 agent 跑的命令

都是重型（checker 档）或我没跑的，按共用约束交主 agent。内存上限取 `scripts/experiment-apparatus/replay.sh` 的 `REPLAY_MEMORY_CAP` 默认 8G，线程 31。工作目录仓根。

1. 核红测试今天红在哪一行（应是 `crash_injection.rs:9029` 那条 `assert_eq!`）：
   `SINGLEFS_HEAVY_TESTS=user-request SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-checker-tier 的库单测 一条; 范围=1 个测试; 为什么=核 G4 第三截红测试今天红在哪一行' bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-checker-tier --lib -- crash_injection::tests::a_remount_refused_on_a_second_crash_image_where_device_zero_has_both_system_configuration_slots_torn_is_an_illegal_refusal_after_the_read_stage --exact`
   看 `test result:` 一行与 panic 行号。
2. 核变异 2406、2407 的杀者在丙' 之后今天绿（我读到它们的盘面带闸一不过，没跑）：
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 writable_mount_refusal_on_a_crash_state_without_read_faults; 范围=1 个测试目标; 为什么=核 G4 三条变异里两条的杀者在丙 之后照旧绿' bash scripts/hosts/capped.sh 31 bash scripts/hosts/run-with-memory-cap.sh 8G cargo test -p singlefs-harness --test writable_mount_refusal_on_a_crash_state_without_read_faults`
   看 `test result:` 一行，应是 5 passed（该文件的 `#[test]` 条数：`grep -c '^#\[test\]' crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs` 现查）。
3. 选甲之后：改完测试，重跑第 1 条，再按 `crates/mutations.tsv` 第 2408 行的参数跑那一行变异（门禁 `checker-tier-crates-mutation-replay --quick --item` 一格，写法见 `.claude/rules/verification.md`「两档：轻型与重型」）。

## 没做什么

- 没跑任何编译与测试；凡写「没跑」「推的」的格都是读码得出，没拿盘面或坏实现实测。尤其没验：甲的盘面下只读恢复仍落到第一个文件那一版；「按盘面判」候选抓得到坏实现；mkfs 单槽盘取号覆写唯一一份的推断。
- 没读攻方腿这一轮的提示与产出；没读 `research/prompts/` 下别轮的 opus 模型目录。
- 没写 `crates/` 与 `mutations.tsv`，候选只写成对实现的改动。没替主 agent 判采纳哪条候选；「推荐」列只是正推腿从代码推到的一个组合，没被攻过零轮。
- 没逐行读完 `mutations.tsv` 全表（只按名字 grep）；没逐个读 `crash_injection.rs` 里所有第三截用例，只读了 G4、C670 那几条。
- 没判「用过的盘上 mkfs 清零段只落一盘」那 8 态里哪几态有盘无槽（只写了推的形状）。
- 没动共用的草稿目录之外的任何东西；草稿目录里只有一个算术脚本与它的输出，无编译目录与仓副本。
