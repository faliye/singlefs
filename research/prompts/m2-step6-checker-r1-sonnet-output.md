# m2-step6-checker-r1 云端正推腿报告（Sonnet）

立场：核「代码做的是不是条款说的」。分到的格：Y4（代码与条款）、Y6（写回）。不替攻方找新反例。

## 各格一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y4-1 I-7.4 两种坏法（重新分配 / 清扫抹头）与「校验和对不上或头用不了」 | 一致，但有一处需要说清 | 两份坏镜像都经「校验和对不上」（I-2.1）路径命中；「头用不了」这条独立分支在这两种坏法下逻辑上几乎不可达，因为它要求先通过校验和匹配 |
| Y4-2 I-4.8 判别力句在固定脚本上真被评估过 | 一致 | 「复用窗口置 0」用例（`second_transaction_step_five_reuse.rs`）与固定脚本 E 步的快用例都实测评估到 I-4.8；前者判违例、后者判成立，两者都不是「不适用」 |
| Y4-3a 候选集用「最新根自己带的 F」代替 F_生效，差在哪、对新两条的影响 | 不一致（但是已登记的旧问题，非步 6 新增） | 代码字面读的是单一根记录里的 F 字段，不是跨盘取最大值再取最小值；这个字段被 I-2.1（已有）与 I-7.4 / I-4.8（步 6 新增）共用同一段候选集计算，新两条不带来新的偏差面 |
| Y4-3b 候选集只剩最新根时 I-7.4 记「不适用」，条款允不允许 | 一致 | 条款文字与 I-4.8 判别力句本身说明了为什么两条不对称处理是合理的；未见条款字面禁止 |
| Y6 kb 四处写回 | 三处一致、一处不一致 | `invariants.md` 两行状态列、`verification-build.md` 26 条那句、里程碑步 6 现状都与代码相符；`invariants.md` 文件开头「判 26 条」那句末尾的「24 条」与代码不符，应为 26 |

## Y4-1：I-7.4「重新分配给其他对象」「清扫抹头」两种坏法，「校验和对不上或头用不了」罩不罩得住

**条款原文**（`.claude/kb/invariants.md:50`，整行抄）：

> I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效，D23（journal 的角色与格式） 已定项 14）所引用的块，其物理范围均未被重新分配给其他对象、也未被清扫抹头。……

**代码判据**（`crates/singlefs-checker/src/walk.rs:811-813`）：

```rust
let walked_into_reused_or_erased_unit = walk.judgements.violation_count("I-2.1")
    > mismatches_before
    || walk.walk_failures.len() > failures_before;
```

这是一个析取式：I-2.1（校验和不匹配）计数增加，**或**走读失败数增加，任一为真就判违例。走读失败（`walk_failures`）有两类来源：

1. `read_referenced_unit`（`crates/singlefs-checker/src/image.rs:301-325`）两份位置都读不到匹配内容，调用方各自 push「两份都读不到对得上的」——这类 push **总是**伴随 I-2.1 违例计数增加（因为 `read_referenced_unit` 内部对每个位置都调用 `judgements.judge("I-2.1", ...)`，返回 `None` 意味着至少两次判定里没有一次 `matches=true`）。
2. `judge_unit_header`（`walk.rs:76-140`）在 `header_holds` 为假或类标签不符时 push「{what} 的头用不了」（`walk.rs:117-120`）。但 `judge_unit_header` 只在 `read_referenced_unit` **已经成功**返回内容之后才会被调用（见 `read_index_node`，`walk.rs:158-177`：先 `read_referenced_unit`，失败直接返回，不到 `judge_unit_header`）。「已经成功」意味着当前字节的整单元 CRC 与候选根位置条目里记录的旧校验和**完全一致**——对于真正被「重新分配」或「清扫抹头」的块，内容已经改变，全字节 CRC 匹配旧值的概率可忽略；「头用不了」分支因此在这两类坏法下逻辑上几乎不可达。

**逐种坏法实测**（副本 `/tmp/claude-1000/m2-step6-checker-r1-sonnet/`，在 `checker_known_bad_images.rs` 的两份坏镜像上，临时给测试循环加了一行 `eprintln!` 打印完整判定表，跑完已还原、与原仓 `diff` 逐字节相同）：

```
SONNET_DEBUG_CASE I-7.4 full_verdicts=[..., ("I-2.1", Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")), ..., ("I-4.8", Violated("候选根 txg 0 出发的遍历有单元对不上或读不出")), ..., ("I-7.4", Violated("候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）")), ...]
SONNET_DEBUG_CASE I-4.8 full_verdicts=[..., ("I-2.1", Violated("树表单元 在盘 0 槽 50178 的那一份与位置条目里的校验和对不上")), ..., ("I-4.8", Violated("候选根 txg 0 出发的遍历有单元对不上或读不出")), ..., ("I-7.4", Violated("候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）")), ...]
```

两份坏镜像（「重新分配」= 改内容重封，「清扫抹头」= 全零不重封）在 checker 里**都**先命中 I-2.1（校验和不匹配），I-7.4 / I-4.8 的违例都是经 `mismatches_before` 那一支触发，不是经 `failures_before`（走读失败计数）那一支——即便「清扫抹头」这份坏镜像整单元被抹成全零、`read_referenced_unit` 对两个位置都返回不匹配从而额外触发「两份都读不到对得上的」push，这个 push 仍然与 I-2.1 违例计数同一次调用产生，不是独立证据。

**逐种镜像形态与 checker 判红的具体行**：

| 坏法 | 镜像形态 | checker 走到哪一行判红 |
|---|---|---|
| 重新分配给其他对象 | `checker_known_bad_images.rs:242-252`：`GENESIS_TREE_TABLE`（槽 50178）整单元内容改一个字段再 `reseal_unit`（自洽地重新封印，模拟「这个槽被分配给别的对象、正常写入」），**不**调用 `propagate()`，即不更新任何引用它的位置条目 | `image.rs:314`（`read_referenced_unit` 里的 `judgements.judge("I-2.1", matches, ...)`），经 `walk.rs:285-287` 的 `walked_into_reused_or_erased_unit` 判定，最终在 `walk.rs:816`（`judge("I-7.4", ...)`）落地为违例 |
| 清扫抹头 | `checker_known_bad_images.rs:254-262`：同一个槽两份都写全零，不重封 | 同上路径，`image.rs:314` 先判 I-2.1（两个位置都不匹配），走失败也累加，但触发点仍是 I-2.1 计数变化那一支 |

**什么观测会推翻这条判定**：如果能构造一份镜像，让某个候选根引用的单元真的被复用或抹头，但 CRC-32 恰好与旧校验和一致（要求发起一次 32 位 CRC 碰撞）或者以某种方式让 `read_referenced_unit` 返回 `Some` 却让 `judge_unit_header` 判「头用不了」——这需要构造一份「内容与旧校验和完全相同但头部不合法」的字节，在诚实哈希前提下这不成立；若真构造出来（哪怕靠暴力枚举）并让 I-7.4/I-4.8 判 Holds，这条「析取式两支实质等价於一支」的结论就被推翻。

## Y4-2：I-4.8 判别力句「抓『本事务内释放的块被重新分配并写入』」在固定脚本上哪一步真被评估过

**条款原文**（`.claude/kb/invariants.md:153`，整行抄）：

> I-4.8 | 近 K 代根校验和自洽 | 任一崩溃点重放后，从回退候选集……里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致……**判别力**：抓「本事务内释放的块被重新分配并写入」这一类——那会让旧根指向的块被写入新内容，**使旧根成为假的回退候选**，而只验最新那一代根是发现不了的

**实测一：`reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`**（`crates/singlefs-harness/tests/second_transaction_step_five_reuse.rs:313-338`）——不抬 F、直接回收释放代 ≤ 11 的落点（18 个槽），再覆盖写一次，新内容落回槽 50176（原实例表单元所在槽）。副本上加一行 `eprintln!` 打印 `check_pool_image` 全表（跑完已还原）：

```
SONNET_DEBUG_VERDICTS [..., ("I-2.1", Violated("实例表单元 在盘 0 槽 50176 的那一份与位置条目里的校验和对不上")), ..., ("I-4.8", Violated("候选根 txg 0 出发的遍历有单元对不上或读不出")), ..., ("I-7.4", Violated("候选根 txg 0 引用的单元已被复用或抹头（校验和对不上或头用不了）")), ...]
```

副本上再加一行打印每次候选根走读时的 `root_txg` / `bad` 判定（同样已还原）：

```
SONNET_DEBUG_CANDIDATE root_txg=0 instance=0 bad=true
SONNET_DEBUG_CANDIDATE root_txg=3 instance=1 bad=true
SONNET_DEBUG_CANDIDATE root_txg=9 instance=3 bad=false
SONNET_DEBUG_CANDIDATE root_txg=12 instance=3 bad=false
SONNET_DEBUG_CANDIDATE root_txg=1 instance=1 bad=true
SONNET_DEBUG_CANDIDATE root_txg=10 instance=3 bad=false
SONNET_DEBUG_CANDIDATE root_txg=13 instance=3 bad=false
SONNET_DEBUG_CANDIDATE root_txg=2 instance=1 bad=true
SONNET_DEBUG_CANDIDATE root_txg=11 instance=3 bad=false
SONNET_DEBUG_CANDIDATE root_txg=14 instance=3 bad=false
```

这十行确认候选集里 10 条根都真的被 I-7.4 / I-4.8 各自独立 `judge()` 过一次（`walk.rs` 792 行注释「(i, T) 有效 ⟺ ...」的候选集算法在这个场景下选出了 txg 0/1/2/3（mkfs + 暖机 + A，均引用被回收的槽 50176）与 txg 9-14（回退之后的实例 3）共 10 条）；txg 0/1/2/3（都是实例 1，都仍引用被复用槽）判违例，txg 9-14（实例 3，不引用该槽）判成立。这正是条款说的「本事务内释放的块被重新分配并写入」——`overwrite_in_process` 是同一进程里发起的一次覆盖写，把此前释放的槽重新分配并写入了新内容。

**实测二：固定脚本 E 步快用例**（`second_transaction_step_zero_layer0.rs` 的 `every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims`，`Script::ReuseAfterRaisingFloor`，这一支的 E 是**先抬 F 再复用**，属于「应该干净」的对照）。副本上在 `assert_checker_and_record_checker_clean` 调用前加一行打印评估计数（跑完已还原）：

```
SONNET_DEBUG_EVAL I-7.4 evaluated=Some(108) violated=None; I-4.8 evaluated=Some(108) violated=None
```

`Layer0Tally::checker_evaluated_states`（`crates/singlefs-harness/src/crash.rs:742-746`）只在 `check_pool_image` 返回 `Holds` 或 `Violated`（非 `NotApplicable`）时才计数，这里 108/108（`tally.states` 断言也是 108）说明这条脚本上**全部**崩溃状态都真的评估了 I-7.4 与 I-4.8，且全部 Holds（0 violated）——这与该场景「F 已正确抬高，复用是合法的」一致，是条款判别力句的**阴性对照**：真正释放-复用的块窗口内（floor 抬高之后）不该被当作「假的回退候选」误判。

**这两条证据合起来覆盖了条款判别力句要求的两面**：负面对照（不抬 F 就复用 ⇒ 必须判违例，实测一）、正面对照（抬 F 之后复用 ⇒ 不该误判，实测二）。

**什么观测会推翻这条判定**：如果实测一里 I-4.8 报告 `NotApplicable` 而不是 `Violated`，或实测二里 108 个状态里有任何一个不是 `evaluated`（比如落进 `NotApplicable` 分支导致计数缺口），说明判别力句在固定脚本上并没有真被评估到。

## Y4-3a：候选集用「最新根实例表有效 ∧ txg ≥ 最新根自己带的 F」——与条款 F_生效 差在哪，对新两条的影响

**代码**（`crates/singlefs-checker/src/walk.rs:793-797`）：

```rust
let newest_rollback_floor = u64::from_le_bytes(
    roots[newest_index].2.record_bytes[130..138]
        .try_into()
        .expect("8 字节"),
);
```

这一句只读**当前被选为「最新」的那一条单一根记录自己携带的 F 字段**，不做任何跨设备聚合。

**条款定义**（`.claude/kb/decisions/16-发布语义.md:375-376`，整行抄）：

```
| 生效 | 每块幸存盘上都有带新 F 的持久根才生效；恢复后生效值 = 各幸存盘所带 F 最大值的最小值 |
| 回退候选集 | 按实例表判仍然有效 ∧ txg ≥ F_生效（D23（journal 的角色与格式） 已定项 14 的候选集随之加这一条） |
```

F_生效 是「各幸存盘所带 F 最大值的最小值」（跨设备的 min-max），代码读的是「被选中的那一个根记录自己带的 F」——两者只在「所有幸存盘上最新持久根的 F 字段恰好相等」时重合，其余情况（比如某一块盘的最新根写失败、F 抬升还没在两块盘上都生效）会不一致。

**这个差异不是步 6 新出现的**：主 agent 在背景材料第一节点名的「第二轮判决第六节 2」（`research/prompts/m2-step45-code-r2-main-verification.md:71`，整行抄）：

> | 2 | D16（发布语义） 已定项 1：抬 F 生效之后 F_生效 会不会回落 | 各幸存盘所带 F 最大值的最小值，新实例的根写它 | 一个字节翻掉一块盘唯一的载体根 ⇒ F 回落到 0、三条不变量一起红。改法 (a) 新根写 max(上一条根的 F, F_生效)：修 X3-A / B，不修 C / D；(b) 候选集也用 max：修 C / D，「窄化」回来；(c) 只改 checker：修一半。要「F 只许涨」就得在盘上或条款里定一个不随载体根丢失的下界 |

这一格在第二轮（步 4/5 代码三方）就已经作为待用户决策的开放问题登记，**尚未定案**。`newest_rollback_floor` 这一变量、以及它所在的那一段候选集计算逻辑（`walk.rs:798-826` 的 `older_candidates` 循环），是 I-2.1（已在步 4/5 那一轮改成「只在候选集里的根上判」，`.claude/kb/invariants.md:107` 状态列有「2026-09-17 起『被引用』按 I-3.1 那个候选集里的根算」字样）、I-7.4、I-4.8 三条**共用同一段代码**（同一个 `newest_rollback_floor` 变量、同一个 `for (index, (_, _, root)) in roots.iter().enumerate()` 循环、同一个 `below_floor` 判断）。步 6 新加的 I-7.4 / I-4.8 只是往这个已经存在的循环体里多加了两个 `judge()` 调用（`walk.rs:815-824`），**没有引入新的候选集计算路径**，也没有让既有的 F 近似值以任何方式变得更宽或更窄。

**结论**：F 与 F_生效 的差异是已登记但未定案的旧问题（决策点 2），步 6 对它的**增量影响是零**——I-7.4 / I-4.8 继承的是与 I-2.1 完全相同的候选集口径，不放大也不缩小这个已知缺口。这一格按照背景材料反向接受条款「Y3 / Y4 打中且落在条款没写的地方 ⇒ 记进决策点」，**不需要新开决策点**，因为它不是步 6 新落在条款没写的地方，而是已有决策点 2 在新两条不变量上的延伸——建议在决策点 2 的「每条路各中哪一格」那一列补一句「I-7.4 / I-4.8（步 6 新增）与 I-2.1 共用同一段候选集计算，受影响方式相同」，供用户定案时一并参考。

**什么观测会推翻这条判定**：如果能在代码里找到 I-7.4 或 I-4.8 专属的、与 I-2.1 不同的候选集过滤逻辑（比如另一处单独读 F 或另一种 abandoned 判断），说明步 6 确实引入了新的偏差面，上面「增量影响为零」的结论就不成立。经通读 `walk.rs` 全文（904 行）确认 `newest_rollback_floor` 只出现在 793、803 两行，`older_candidates` 循环只有一份，没有找到第二套候选集计算。

## Y4-3b：候选集只剩最新根时 I-7.4 记「不适用」，条款允不允许

**代码**（`crates/singlefs-checker/src/walk.rs:827-830`）：

```rust
if older_candidates == 0 {
    walk.judgements
        .not_applicable("I-7.4", "回退候选集里只有最新根，没有更早的根可判");
}
```

I-4.8 没有对应的「不适用」分支——它在 `walk.rs:781-785` 无条件对最新根judge 一次（`newest_failures.is_empty() && violation_count("I-2.1") == 0`），此后每条更早候选根再各 judge 一次（`walk.rs:821-824`）；候选集只剩最新根时，I-4.8 至少仍有「对最新根自己的这一次」判定，不会落进「未评估」状态。

**条款字面是否允许这种不对称**：

- I-7.4 定义的是「回退候选集里每一个根……所引用的块，均未被重新分配……也未被清扫抹头」——这是一句**比较**性质的陈述：检查的是某个候选根引用的块「相对当前盘面」有没有被动过。当候选集里只有一个根（就是最新根自己）时，没有第二个可比较的对象——用最新根自己的当前内容去检验它「自己有没有被复用」是重言式（trivial），不构成有意义的检验。条款正文没有出现「候选集只剩一个也要判」这种字样，也没有出现禁止「不适用」的措辞。
- I-4.8 定义是「从候选集里**任一**根出发遍历，所有块的校验和均与其父指针记录的一致」，且判别力句明说「只验最新那一代根是发现不了的」——这句话本身承认「验最新那一代根」是这条不变量判定动作的**一部分**（虽然单独验最新根发现不了 I-4.8 真正要抓的那类问题，但验它仍然是这条不变量判定过程里合法的一步，不是被排除在外的），所以 I-4.8 无论候选集里有没有更早的根，至少对最新根这一次判定都有意义、都该执行——这与代码「I-4.8 总是至少判一次」一致。

**这与 `verification-build.md` 的一般规则也吻合**（`.claude/kb/verification-build.md` 表格，「第一版必须报『不适用』而不许静默通过的」一列，第一版建表时已有先例）——「不适用」这个三态本身就是为「这个镜像上没有该条不变量判定的对象」这种情况设计的，`not_applicable` 调用必须带理由（`image.rs:71-77` 的 `Judgements::not_applicable` 与 `assert!` 同款），代码这里带了理由（"回退候选集里只有最新根，没有更早的根可判"），符合既有的「不许静默通过」要求。

**结论**：条款文字没有禁止这种处理，I-7.4 记「不适用」、I-4.8 总判一次的不对称处理，从两条各自的定义与 I-4.8 自己的判别力句都能找到对应依据，一致。

**什么观测会推翻这条判定**：如果条款某处（比如 D16 已定项 1 或 D23 已定项 14 的候选集正文）明确写了「候选集哪怕只剩一个根，I-7.4 也必须判定（比如判定『恒成立』而非『不适用』）」，这条一致的判定就不成立；已通读 `invariants.md` 中 I-7.4 整行与相关决策段落，没有找到这样的字样。K 的下限是否允许 1、以及能不能构造出让这个「不适用」分支被误用的镜像，属于 Y3（分给本地攻方的格），不在这条腿的判断范围内。

## Y6：写回核对

### `invariants.md` I-7.4 / I-4.8 两行的状态列

两行状态列文字（`.claude/kb/invariants.md:50`、`:153`，已在 Y4-1 / Y4-2 整行抄过）与代码逐句核对：

- 「按每条回退候选集里更早的根各判一格」——对应 `walk.rs:798-826` 的 `older_candidates` 循环，逐条候选根各自一次 `judge()`。一致。
- 「走读它引用的单元时校验和对不上或头用不了，就是它指着的块被复用或抹头」——对应 `walk.rs:811-813` 的析取式判据。一致（Y4-1 已指出「头用不了」这一支在这两类坏法下逻辑上几乎不可达，但代码字面确实是这个析取式，状态列这句话没有夸大）。
- 「坏镜像是 mkfs 树表单元（只有第 0 代根还引用）改内容重封」（I-7.4 行）/「坏镜像是 mkfs 树表单元抹成零」（I-4.8 行）——对应 `checker_known_bad_images.rs:242-262` 两份坏镜像定义。一致。
- 「层 0 每个崩溃状态都判」——用固定脚本 E 步快用例实测验证：108/108 状态 `evaluated`（Y4-2 已贴命令与输出）。这条腿没有跑全量 2104413 状态的那条 `#[ignore]` 测试（该测试单跑就要「debug 下半小时以上」，`second_transaction_step_zero_layer0.rs:378` 注释；且跑这一轮时宿主机上已有另一个进程在跑同一个全量测试，见「这条腿自己的限度」），因此「层 0 每个崩溃状态都判」这句话在**全量**意义上没有被这条腿直接验证，只在快用例的 108 个状态上得到确认。另外，通过读 `crates/singlefs-core/src/make_filesystem.rs:260-270` 确认 mkfs 会往 `ROOT_RING_REGIONS`（3 个区域）各写一份创世根，这解释了为什么即便在最早的崩溃状态（`base = mkfs 之后`，什么都没持久）里也已经有 3 条根记录、`older_candidates` 天然大于 0——这是「层 0 每个崩溃状态都判」这句话在理论上站得住的根源，但仍不构成对全量 2104413 个状态的穷举验证。

### 文件开头「判 26 条」那句

**原文**（`.claude/kb/invariants.md:12`，整行抄）：

> 池级 checker（`walk::check_pool_image`）判 26 条（第一版 23 条，2026-09-16 里程碑「第二个事务」步 3 加 I-3.8（实例表行唯一且低于挂载根），2026-09-17 步 6 加 I-7.4（近 K 代块未被复用） 与 I-4.8（近 K 代根校验和自洽）；原文接着写的是步 3 那一处：（实例表行唯一且低于挂载根）），状态列写「已实现」的就是这 24 条：每条配一份改坏了正好触发它的镜像（`crates/singlefs-harness/tests/checker_known_bad_images.rs`），层 0 的每个崩溃状态都跑它。

**不一致**：这句话开头说「判 26 条」（23 + 1 + 2 = 26，算术本身没问题），末尾却说「状态列写『已实现』的就是这 24 条」。实测直接数 `invariants.md` 里 `| I-` 开头的表格行中含「已实现（」的行数：

```
$ grep -n "^| I-" .claude/kb/invariants.md | grep -c "已实现（"
26
```

且这 26 行的 ID 逐条比对 `crates/singlefs-checker/src/image.rs:36-40` 的 `IMPLEMENTED_INVARIANTS` 常量数组（26 个 ID），**完全一致**（`I-1.1, I-1.3, I-1.4, I-1.6, I-1.7, I-7.1, I-7.2, I-7.4, I-7.6, I-7.7, I-7.8, I-2.1, I-2.3, I-2.4, I-2.5, I-3.1, I-3.8, I-4.8, I-5.1, I-5.2, I-9.1, I-9.2, I-9.4, I-9.7, I-9.10, I-9.13`）。所以状态列写「已实现」的实际是 26 条，不是 24 条——这句话末尾的「24」是错的，应改成「26」。

这句话中间还有一段读起来不通顺的插入语「；原文接着写的是步 3 那一处：（实例表行唯一且低于挂载根））」，像是编辑过程中留下的中间态文字（既不是对 I-3.8 的简称补注，也不构成完整语义），建议随「24 改 26」一并清理，但这属于文字表述问题，不在这条腿判断代码是否正确落地的范围内，只如实指出。

**这不影响** `verification-build.md:138`（下一小节核对）与里程碑步 6 现状（再下一小节核对）——那两句都写的是「26 条」，与代码一致；只有 `invariants.md:12` 这一句自身收尾的「24」和它前半句的「26」互相矛盾。

### `verification-build.md` 26 条那句

**原文**（`.claude/kb/verification-build.md:138`，整行抄）：

> ……2026-09-14 起三件都接上了：恢复 + oracle、池级 checker（`crates/singlefs-checker` 的 `walk::check_pool_image`，26 条：第一版 23 条加 I-3.8（实例表行唯一且低于挂载根）、I-7.4（近 K 代块未被复用）、I-4.8（近 K 代根校验和自洽））、记录核对器（`crates/singlefs-harness` 的 `crash::check_records`），层 0 全量在门禁 54 号。

**一致**：23 + I-3.8 + I-7.4 + I-4.8 = 26，与 `IMPLEMENTED_INVARIANTS` 长度（26）及内容逐一核对相符。

### 里程碑 `02-second-txn.md` 步 6 现状

**原文**（`.claude/kb/milestone/02-second-txn.md:216`，整行抄，已在背景材料第一节出现，这里核对与代码相符的具体点）：

- 「checker 新接了 I-3.8、I-7.4、I-4.8（后两条按候选集里每条根各判一格：走读它引用的单元时校验和对不上或头用不了就红，坏镜像是 mkfs 树表单元——只有第 0 代根还引用——改内容重封 / 抹成零；`crates/singlefs-checker/src/walk.rs`、`image.rs` 清单 26 条）」——与代码逐句一致（Y4-1、Y6 首段已核）。
- 「I-3.1 与 I-2.1 改按回退候选集判」——`I-3.1` 的「已分配」对比目标（`walk.rs:867-873`）来自最新根自己的记账树（`accounting`/`accounting_seen` 在 `walk.rs:786-787` 于老候选根循环**之前**快照），但用来对比的「遍历得到」的字节数（`walked`，来自 `walk.references`/`per_device`）在老候选根循环执行后累积了**全部**候选根（含更早的）引用的物理范围之和——这与「回退候选集判」的说法一致：即「已分配」应等于整个候选集（而不仅是最新根）引用范围的并集。`I-2.1` 改动见 `invariants.md:107` 状态列文字，已在 Y4-3a 引用确认。两句都与代码相符。
- 「到 E 的快用例 108 个状态零违例」——与 Y4-2 实测的 `evaluated=Some(108) violated=None` 完全对应（这条腿独立复核了这一数字，不是照抄背景材料）。
- 「门禁 59 号 55 条在跑」——实测数了 `crates/mutations.tsv` 的非注释非空行数：

```
$ grep -vc "^#\|^$" crates/mutations.tsv
55
```

与「55 条」一致。这条腿没有跑全部 55 条（那是门禁 59 号整道的工作），只按任务指示复跑了「步 6」那两行（见下一节）。

## `crates/mutations.tsv`「步 6」两行复跑（副本上跑，任务指示动作）

两行原文（`crates/mutations.tsv:59-60`）：

```
步 6：I-7.4 不判候选根引用的单元被复用或抹头	crates/singlefs-checker/src/walk.rs	                .judge("I-7.4", !walked_into_reused_or_erased_unit, || {	                .judge("I-7.4", true, || {	-p singlefs-harness --test checker_known_bad_images	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
步 6：I-4.8 只看最新根、不看别的候选根	crates/singlefs-checker/src/walk.rs	                .judge("I-4.8", !walked_into_reused_or_erased_unit, || {	                .judge("I-4.8", true, || {	-p singlefs-harness --test checker_known_bad_images	the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target
```

副本（`/tmp/claude-1000/m2-step6-checker-r1-sonnet/`）上按这两行各自把 `walk.rs` 里对应的 `.judge(...)` 调用替换成「恒真」版本、跑点名的测试、再还原，逐行原样输出（`nice -n 19 cargo test`）：

**第一行（I-7.4 → 恒真）**：

```
returncode=101
RESULT: 红了（the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target FAILED，符合预期）
thread 'the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target' panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:566:9:
I-7.4 那份坏镜像应当判违例，实际 Holds
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

**第二行（I-4.8 → 恒真）**：

```
returncode=101
RESULT: 红了（the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target FAILED，符合预期）
thread 'the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target' panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:566:9:
I-4.8 那份坏镜像应当判违例，实际 Holds
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

**还原后复跑**（确认 `walk.rs` 与原仓 `diff` 逐字节相同之后再跑一次）：

```
$ diff /home/fy5090/code/singlefs/crates/singlefs-checker/src/walk.rs crates/singlefs-checker/src/walk.rs
diff exit=0
running 1 test
test the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
```

两行都「改坏 → 红、还原 → 绿」，与 `crates/mutations.tsv` 声明的预期一致。跑法：Python 脚本按官方 `59-crates-mutation-replay.sh` 的同一套字段解析与替换逻辑，只过滤「步 6」前缀的两行，不是完整跑一遍门禁 59 号（那是 55 条全表，归 `gate-triage`/门禁本身，不在这条腿的任务范围内）。

## 这条腿自己的限度

- **没有跑层 0 全量（2104413 个状态）的 `#[ignore]` 测试。** 任务只要求跑 `second_transaction_step_zero_layer0` 的**快**用例，已照办（108 状态）；全量那条单独跑要「debug 下半小时以上」，而且开工时 `ps aux` 已经看到宿主机上有一个 PID 231760 的进程在跑同一个二进制（`second_transaction_step_zero_layer0-7921cf1fc5f129b1 --include-ignored --nocapture`），这条腿判定「不是这一轮的活」，没有碰它、也没有在副本上重复起一份等量开销的全量跑。因此「层 0 每个崩溃状态都判」这句话在 Y6 里只核到快用例（108/108），全量意义上的验证归主 agent 或门禁 54 号（背景材料已写「层 0 全量在最终的树上重跑（54 号）」）。
- **没有跑完整的 55 条门禁 59 号。** 只复跑了任务点名的「步 6」两行，数量核对（55 条总数）用 `grep -c` 现查，没有逐条跑。
- **没有判 Y1、Y2、Y3、Y5。** 这些格分给了 Opus 攻方与本地攻方，按分工与「不替攻方找新反例」的规矩，这条腿没有主动构造可达历史或坏镜像去打 U1，只在 Y4 范围内核对代码与条款字面是否一致。
- **没有单独验证「候选根按 txg 破平局的顺序是否影响哪个候选根的违例被记成『第一处』」这类归属问题**——这属于 Y2（归错根），分给 Opus 攻方，这条腿在实测输出里看到「候选根 txg 0」被报告为「第一处违例」（即便 A 在 txg 3 也同样违例）只是顺手记录的观测，没有判断这是否构成问题。
- **对 F_生效 差异（Y4-3a）的「结论：增量影响为零」，只核对了代码里 `newest_rollback_floor` 只有一处计算路径（通读 walk.rs 全文 904 行）**，没有反过来构造一个「F 与 F_生效 不一致」的具体镜像去实测 I-7.4/I-4.8 在这种场景下到底输出什么（那需要伪造多盘 F 不一致的状态，接近攻击性质的工作，超出 Y4「核代码与条款对不对得上」的范围）。
- **副本上所有临时调试代码（`eprintln!`、debug 打印）都已用 `research/scripts/replace-once.py` 精确还原并用 `diff` 逐字节核对与原仓一致**，报告里贴的是这些临时调试运行的原样输出，不是对原仓的改动；原仓一个字节都没有碰过，没有任何 git 写操作。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| Y4-1（I-7.4 两种坏法与析取式判据） | 一致，附加发现 | 两支析取式在字面上都存在，但「校验和对不上」这一支实测覆盖了两份坏镜像；「头用不了」这一支因为要求先通过校验和匹配，逻辑上在真实复用/抹头场景下几乎不可达 |
| Y4-2（I-4.8 判别力句在固定脚本上的评估） | 一致 | 「复用窗口置 0」用例（负面对照，Violated）与固定脚本 E 步快用例（正面对照，108/108 Holds）两头都实测确认真被评估 |
| Y4-3a（F 与 F_生效 差异对新两条的影响） | 不一致但非新问题 | 代码用「最新根自己带的 F」代替 F_生效，是已登记的决策点 2（`m2-step45-code-r2-main-verification.md:71`）在步 4/5 就留下的问题；I-7.4/I-4.8 与 I-2.1 共用同一段候选集代码，步 6 增量影响为零 |
| Y4-3b（候选集只剩最新根时 I-7.4「不适用」） | 一致 | 条款文字未禁止；I-4.8 判别力句自陈「验最新那一代根」是判定过程的合法一步，解释了两条为何不对称处理 |
| Y6（invariants.md 两行状态列） | 一致 | 逐句对上代码行为；「层 0 每个崩溃状态都判」只验证到快用例层面 |
| Y6（invariants.md「判 26 条」那句） | **不一致** | 句子开头算术是 26，句末「状态列写『已实现』的就是这 24 条」应改成 26（实测已实现行数 = 26，逐一比对 IMPLEMENTED_INVARIANTS 一致） |
| Y6（verification-build.md「26 条」那句） | 一致 | 23+1+2=26，与代码一致 |
| Y6（里程碑步 6 现状） | 一致 | 逐句核对（新接三条、按候选集各判一格、坏镜像形态、108 状态、55 条变异表）均与代码/实测相符 |

## 没做什么

- 不判 Y1（复用漏判）、Y2（归错根）、Y3（坏镜像与不适用的分辨力/K 下限）、Y5（层 0 误红风险）——按分工归 Opus 攻方与本地攻方。
- 不替攻方找新反例，Y4-1 里指出的「头用不了」分支几乎不可达只是对现有两份坏镜像的观测，没有主动去构造一个专门触发「头用不了」而不触发 I-2.1 的镜像。
- 不采纳或出判决，四条腿的分歧留给主 agent 核实与写判决。
- 没有跑层 0 全量 2104413 状态的 `#[ignore]` 测试，也没有跑完整 55 条门禁 59 号（只复跑任务点名的「步 6」两行）。
- 没有修改 `.claude/kb/invariants.md`（虽然发现「24 条」应为「26 条」），只在报告里指出，写回由主 agent 或 `kb-scribe` 按判决执行。
- 原仓 `crates/`、`.claude/kb/` 一个字节都没有改动，也没有任何 git 写操作；所有临时调试代码只存在于 `/tmp/claude-1000/m2-step6-checker-r1-sonnet/` 副本里且已还原。
