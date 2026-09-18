# m2-step6-checker-r1 核查员报告（2026-09-17）

核对表里的 ✗ 不免除主 agent 对推论的逐条现查，这一句照抄进报告开头。

依据：`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」；
`.claude/singlefs-ai-sop/rules/evidence-discipline.md`「校验路径本身也要证明它会红」「引产物就整行抄」。

## 判别力自证

挑 Opus 报告对 `.claude/kb/invariants.md:50`（I-7.4 那一行）的引用，在草稿目录
`/tmp/claude-1000/m2-step6-checker-r1-verifier/discriminative-selftest/invariants.md`
里把行号从 50 故意改成 51（行号 +1）再核：

```
=== 原快照 line 50（I-7.4 那一行，与 Opus 报告逐字一致）===
| I-7.4 | 近 K 代块未被复用 | 回退候选集里每一个根（按实例表判仍然有效 ∧ txg ≥ F_生效……

=== 草稿副本里，故意核第 51 行（行号+1）===
| I-7.5 | 根槽按判定宽度对齐 | 每个根槽的**起始偏移**是挂载时探测的 `physical_block_size` 的整数倍……
```

第 51 行是完全不同的一条（I-7.5，不是 I-7.4），与 Opus 引文逐字不符 ⇒ **判 ✗**。
核查方法能分辨。原样结果附上，其余各表格不再重复贴这一步。

## 前提说明：行号一律对快照核，snapshot 路径
`/tmp/claude-1000/-home-fy5090-code-singlefs/e166d536-b665-4a88-8372-3091b9a0fc41/scratchpad/step6-r1-legs-snapshot/`
（`crates/` 与 `.claude/kb/` 都在其下）。复跑用副本
`/tmp/claude-1000/m2-step6-checker-r1-verifier/rerun-copy/`（Opus 的 zz_attack_* 与 P1 补丁）与
`/tmp/claude-1000/m2-step6-checker-r1-verifier/rerun-copy-sonnet/`（Sonnet 的 mutations.tsv 两行复跑），
均由快照 `crates/` + `.claude/kb/` 叠加到从主树 rsync 出的可编译工作区上构成，`nice -n 19` 跑。
原仓（`/home/fy5090/code/singlefs`）未做任何写操作。

## 一、云端攻方（Opus，`m2-step6-checker-r1-opus-output.md`）

| 引用 / 命令 | 核的结果 |
|---|---|
| `.claude/kb/invariants.md:50`（I-7.4 定义，整行开头） | ✓ 快照第 50 行逐字一致 |
| `.claude/kb/invariants.md:153`（I-4.8 定义） | ✓ 快照第 153 行逐字一致 |
| `.claude/kb/decisions/16-发布语义.md:375`（F_生效 定义） | ✓ 快照第 375 行逐字一致 |
| `crates/singlefs-checker/src/walk.rs:793-797`（`newest_rollback_floor` 取值） | ✓ 逐字一致 |
| `walk.rs:803-804`（`below_floor` / 候选集准入） | ✓ 逐字一致 |
| `crates/singlefs-core/src/recovery.rs:351`（`effective_rollback_floor`） | ✓ 函数签名一致 |
| `walk.rs:744`（`valid_roots` 调用） | ✓ 一致 |
| `crates/singlefs-checker/src/lib.rs:203`（`rollback_floor` 字段） | ✓ 一致 |
| `crates/singlefs-checker/src/image.rs:53-66`（`Judgements::judge`） | ✓ 逐字一致，含 `or_insert_with(detail)` 内嵌代码 |
| `image.rs:79`（`into_report` 签名） | ✓ 一致 |
| `walk.rs:158-168` / `walk.rs:169-174`（`read_referenced_unit` 先于 `visited_units.insert`） | ✓ 逐字一致，顺序（158 在前、169 在后）与「排在 insert 之前」的论断相符 |
| `walk.rs:781-785`（最新根 I-4.8 判定） | ✓ 逐字一致 |
| `walk.rs:832`（I-7.2 判定） | ✓ 一致 |
| `image.rs:301-324`（`read_referenced_unit` 全函数） | ✓ 逐字一致 |
| `crates/singlefs-harness/tests/checker_known_bad_images.rs:307-313`（I-2.1 坏镜像） | ✓ 逐字一致 |
| `walk.rs:229-233`（`walk_root` 读树表、失败 `return`） | ✓ 逐字一致 |
| `walk.rs:238-243`（读中央映射树的根） | ✓ 逐字一致 |
| `walk.rs:118` / `walk.rs:179`（`walk_failures` 两处 push） | ✓ 一致 |
| `walk.rs:298-304`（`not_applicable("I-7.2", …)` 死码位置） | ✓ 逐字一致 |
| `crates/singlefs-core/src/mount.rs:437-439`（候选集并集注释） | ✓ 逐字一致（引文只抄第 437 行第一句，与「整行抄第一句」的说法相符） |
| `walk.rs:800-802`（`abandoned` 判定） | ✓ 逐字一致 |
| 三个测试函数名（`roots_below_a_floor_carried_by_only_one_device_remain_rollback_candidates`、`slots_reclaimed_by_raising_the_floor_are_not_handed_out_before_the_floor_takes_effect`、`reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red`） | ✓ 均在快照 `second_transaction_step_five_reuse.rs` 里存在，起始行号与引用相符 |
| `walk.rs:203-227` / `349-381` / `484-501`（另外三处 `visited_units` 调用点） | ✓ 均含 `read_referenced_unit` 在前、`visited_units` 在后，与「四处都排在 insert 之前」一致 |
| `.claude/kb/milestone/02-second-txn.md:230`（验收标准「报出评估次数」） | ✓ 快照第 230 行逐字一致 |
| sha256（`p1-effective-floor.patch`、`zz_attack_floor.rs`、`zz_attack_probe.rs`） | ✓ 三个哈希与仓里文件现算一致 |

### 复跑（副本 `rerun-copy/`，`nice -n 19 cargo test`）

| 命令 | 结果 |
|---|---|
| `cargo test -p singlefs-harness --test zz_attack_floor -- --nocapture` | ✓ 3 个测试全 ok；`F_生效 = CheckpointTxg(0)`、`最新根 F = CheckpointTxg(11)`、候选 txg 列表、`判违例的：["I-3.1"]`、26 行判定表（`I-2.1: Holds` … `I-7.4: Holds`）、`退到 (1, 2)` 的 `Err(Recovery(UnitUnreadable { slot: SlotNumber(50178) }))`、对照组 `F_生效 = CheckpointTxg(11)` 后到「对照跑完」之间无 non-Holds 行——逐字与报告贴的原样输出一致 |
| `cargo test -p singlefs-harness --test zz_attack_probe -- --ignored --nocapture` | ✓ 3 个测试全 ok；「甲」「乙」两组判定行（I-2.1/I-4.8/I-7.2/I-7.4）与报告 Y2 节贴的原样输出逐字一致 |
| 手工施加 `p1-effective-floor.patch`（按 `walk.rs:793-797` 定点替换，命中 1 次）后重跑 `zz_attack_floor` | ✓ 判违例集合 `["I-2.1", "I-3.1", "I-4.8", "I-5.1", "I-7.4"]` 与报告贴的三行（I-2.1/I-4.8/I-7.4）逐字一致；对照格仍无 non-Holds 行（I-5.1 多红一条与报告「对照格上仍绿；I-5.1 多红一条」的表述需要核对——见下方说明） |

**关于「对照格施加 P1 之后仍然 26 条全成立」**：复跑时两个测试用例并行输出交错，「对照：F_生效 = CheckpointTxg(11)」与「对照跑完」之间只插入了另一个测试的打印行（`F_生效 = CheckpointTxg(0)…`），本身没有出现 non-Holds 行，与 Opus 的表述一致；I-5.1 多红一条只出现在「判违例的：[...]」那组（打中格），不在对照格，Opus 原文写的也是「上面那次 I-5.1 就多红了一条」（指判违例那组），核对无误。

### Opus 腿计数

核了 25 处引用 + 3 处复跑命令 = 28 处；✓ 28 处；✗ 0 处；核不动 0 处。
「没打中的形状」表格里逐条给出的读代码范围（`walk.rs:298-304` 等）已随上表核过，
表里关于「三类树长内部节点」「CRC32 碰撞」等属于「读代码、未造历史」的观察，未要求复跑，未再单独计数。

## 二、云端正推（Sonnet，`m2-step6-checker-r1-sonnet-output.md`）

| 引用 / 命令 | 核的结果 |
|---|---|
| `walk.rs:811-813`（`walked_into_reused_or_erased_unit` 析取式） | ✓ 逐字一致 |
| `walk.rs:76`（`judge_unit_header` 起始）、`walk.rs:140`（收尾 `}`） | ✓ 边界一致 |
| `image.rs:314`（`judge("I-2.1", matches, …)`） | ✓ 一致 |
| `checker_known_bad_images.rs:242-252`（I-7.4 坏镜像：改字段 + `reseal_unit`） | ✓ 逐字一致 |
| `checker_known_bad_images.rs:254-262`（I-4.8 坏镜像：全零、不重封） | ✓ 逐字一致 |
| `second_transaction_step_five_reuse.rs:313-338`（`reclaiming_without_raising_the_floor_…`） | ✓ 函数起始行 313 一致（区间未逐行核到 338，见下「核不动」） |
| `second_transaction_step_zero_layer0.rs`「E 步快用例」（`Script::ReuseAfterRaisingFloor`） | ✓ 复跑该测试通过（见下方复跑） |
| `walk.rs:793-797`（`newest_rollback_floor` 只读单一根） | ✓ 逐字一致（与 Opus 引用同一处） |
| `.claude/kb/decisions/16-发布语义.md:375-376`（生效 / 回退候选集两行） | ✓ 逐字一致 |
| `research/prompts/m2-step45-code-r2-main-verification.md:71`（决策点 2「F_生效 回落」表行） | **✗ 行号差 1**：该表行的真实位置是**第 72 行**，第 71 行是另一条决策点（C340（回退之后记录链从哪条之后接没有定义）：P1 还是 P2），与 Sonnet 引文内容完全无关。第 72 行内容与 Sonnet 整行引文逐字一致，是纯粹的行号 off-by-one，不是内容伪造 |
| `walk.rs:798-826`（`older_candidates` 循环整体） | ✓ 已随下面细粒度引用逐段核过，边界一致 |
| `walk.rs:827-830`（候选集只剩最新根时 `not_applicable`） | ✓ 逐字一致 |
| `.claude/kb/invariants.md:107`（I-2.1 状态列 2026-09-17 修订） | ✓ 快照第 107 行逐字一致 |
| `.claude/kb/invariants.md:12`（文件开头「判 26 条」那句，声称整行抄） | **✗ 与快照不符，且原因可定位**：快照第 12 行实为「…判 26 条（…），状态列写「已实现」的就是这 **26** 条…」，没有 Sonnet 引文里的「24 条」也没有那句「；原文接着写的是步 3 那一处：（实例表行唯一且低于挂载根））」。逐字核对发现：Sonnet 那句引文（含「24 条」与整段插入语）**与 `research/prompts/_m2-step6-checker-r1-background.md:794`、`_m2-step6-checker-r1-appendix.md:7` 的文本逐字相同**，而这两处材料本身自称「出处 `.claude/kb/invariants.md:9-15`（整段抄，未转述）」。也就是说：Sonnet 的「整行抄」实际内容只在背景材料里找得到，不在当前（也不在 `git diff` 显示的任何一个已知历史版本，包括提交 HEAD 的旧版「就是这 23 条」）invariants.md 里存在。这不构成 Sonnet 报告本身的凭空捏造——很可能是背景材料生成时携带了一处过渡态/拼接缺陷（`24` 在算术上等于 `23 + 1`，是「只加了 I-3.8、还没加 I-7.4/I-4.8」那个中间态该有的数），但**背景材料本身要先核这条纪律在这里被打穿**，Sonnet 因此继承了一处不存在于当前文件的引文。**建议记：`24` 那句「误写成背景材料 `_m2-step6-checker-r1-background.md:794`（及 `_m2-step6-checker-r1-appendix.md:7`），当前 `.claude/kb/invariants.md` 实为第 12 行、内容是「26 条」，无插入语」**。Sonnet 据此指出的「26 与 24 矛盾」这一发现本身是真实、有价值的（且已被主 agent 在「五、处置」第 2 项写回修正），只是它标注的信息来源（「整行抄」自 kb 文件本身）不准确 |
| `.claude/kb/verification-build.md:138`（26 条那句） | ✓ 快照第 138 行逐字一致 |
| `.claude/kb/milestone/02-second-txn.md:216`（步 6 现状段） | ✓ 快照第 216 行逐字一致（核对了引文摘录的关键短语，未逐字比对整行超长文本之外的部分） |
| `crates/singlefs-harness/src/crash.rs:742-746`（`Layer0Tally::checker_evaluated_states`） | ✓ 逐字一致 |
| `crates/singlefs-core/src/make_filesystem.rs:260-270`（mkfs 写 `ROOT_RING_REGIONS` 份创世根） | ✓ 逐字一致；`ROOT_RING_REGIONS = 3` 在 `singlefs-format/src/lib.rs:185` 核实 |
| `crates/mutations.tsv:59-60`（步 6 两行） | ✓ 快照第 59-60 行逐字一致，与 Sonnet 报告引文一致 |
| `grep -vc "^#\|^$" crates/mutations.tsv` → 55 | ✓ 快照现数为 55，一致 |

### 复跑（副本 `rerun-copy-sonnet/`，`nice -n 19 cargo test`）

| 命令 | 结果 |
|---|---|
| `cargo test -p singlefs-harness --test second_transaction_step_five_reuse reclaiming_without_raising_the_floor_reuses_a_slot_a_candidate_root_still_references_and_the_checker_goes_red` | ✓ 通过（不含 Sonnet 自己加的 `SONNET_DEBUG_CANDIDATE` 调试打印，见下方「核不动」） |
| `cargo test -p singlefs-harness --test second_transaction_step_zero_layer0 every_crash_state_outside_the_two_unit_segments_recovers_to_the_version_its_root_claims` | ✓ 通过；`grep -n "tally.states, 108"` 命中 `second_transaction_step_zero_layer0.rs:358`，与「108 状态」断言一致 |
| 定点替换 `walk.rs` 第 59 行的 `.judge("I-7.4", !walked_into_reused_or_erased_unit, ...)` → `.judge("I-7.4", true, ...)`（命中 1 次），跑 `checker_known_bad_images::the_clean_image_holds_every_invariant_and_each_mutation_violates_its_target` | ✓ `panicked at crates/singlefs-harness/tests/checker_known_bad_images.rs:566:9: I-7.4 那份坏镜像应当判违例，实际 Holds`，`FAILED`——与报告贴的原样输出逐字一致 |
| 还原后同样定点替换 I-4.8 那行（命中 1 次），跑同一测试 | ✓ `panicked … I-4.8 那份坏镜像应当判违例，实际 Holds`，`FAILED`——逐字一致 |
| 还原后 `diff` 与快照 `walk.rs` | ✓ `diff exit=0`（逐字节相同），再跑同一测试 | ✓ `ok`，与报告「还原后复跑」贴的输出逐字一致 |

### 核不动 / 部分核实

- `SONNET_DEBUG_CASE` / `SONNET_DEBUG_VERDICTS` / `SONNET_DEBUG_CANDIDATE` / `SONNET_DEBUG_EVAL` 四组调试打印输出：Sonnet 报告未随附任何 diff 或模型文件（本轮没有 `m2-step6-checker-r1-sonnet-model/` 目录），这些 `eprintln!` 是 Sonnet 在自己的副本里临时加的、事后已还原，**核不动**（无法在不猜测具体插桩位置的前提下逐字复现）。间接交叉验证：`SONNET_DEBUG_CASE` 报的判定行（I-2.1/I-4.8/I-7.2/I-7.4 四条）与本报告「一、Opus」节里独立复跑 `zz_attack_probe.rs` 得到的「乙」组判定行逐字相同，构成有力的旁证但不是直接复跑核实。
- `second_transaction_step_five_reuse.rs:313-338` 区间的 338 行终止边界未逐行核对（只核了起始行 313）。
- `.claude/kb/milestone/02-second-txn.md:216` 是一整段超长文本（含大量子句），只核对了 Sonnet 引文摘录到的几个关键短语在该行内逐字存在，未做整行字符级 diff。

### Sonnet 腿计数

核了 21 处引用 + 5 处复跑命令 = 26 处；✓ 24 处；✗ 2 处（`m2-step45-code-r2-main-verification.md:71` 行号差 1；`invariants.md:12` 引文实源自背景材料的过渡态文本）；核不动/部分核实 3 处。

## 三、本地攻方（`m2-step6-checker-r1-local-attack-*`）

转述核对表 `m2-step6-checker-r1-local-attack-translation-audit.md`：18 行「英文项 / 原文文件:行 / 首稿缺的 / 定稿」。

| 原文文件:行 | 核的结果 |
|---|---|
| `invariants.md:50`（I-7.4 主句 + 候选集缩窄那句） | ✓ 两处引文均在快照第 50 行内逐字存在 |
| `invariants.md:153`（I-4.8 主句 + 判别力句「假的回退候选」） | ✓ 逐字存在 |
| `decisions/23-journal的角色与格式.md:1209`（候选集定义 + (i,T) 可选两个析取分支） | ✓ 该行（一整段长文本）内逐字包含核对表所引的子句 |
| `decisions/16-发布语义.md:376`（回退候选集复述行） | ✓ 逐字一致 |
| `walk.rs:751-756`（`newest_index` 按 (checkpoint_txg, instance) `max_by_key`） | ✓ 逐字一致 |
| `walk.rs:781-784`（I-4.8 单格判据） | ✓ 与 `walk.rs:781-785` 内容一致（785 行是分号收尾） |
| `walk.rs:807-809`（`mismatches_before`/`failures_before` 时序） | **✗ 行号偏 1**：`mismatches_before` 实在第 806 行、`failures_before` 第 807 行、`walk_root` 调用第 808 行；引文范围（807-809）少了 806、多了 809（注释起始行）。内容本身真实存在，「在 walk_root 之前取」的论断成立，但引用范围与真实行号不严格对应。本地辩方的同类引用（见下节）写的是正确范围 `806-807` |
| `walk.rs:811-823`（同一个布尔值喂两次 judge） | ✓ 逐字一致 |
| `walk.rs:827-829`（I-7.4 `not_applicable`，I-4.8 无对应调用） | ✓ 逐字一致 |
| `image.rs:56-65`（`judge` 方法，`or_insert_with` 只在无值时写入） | ✓ 逐字一致（实际范围 53-66，56-65 是其子集） |
| `image.rs:308-316`（`read_referenced_unit` 逐位置读字节比较） | ✓ 逐字一致 |
| `checker_known_bad_images.rs:31`（mkfs 树表单元注释） | ✓ 逐字一致 |
| `checker_known_bad_images.rs:245-247`（改 42 偏移 8 字节、`reseal_unit`） | ✓ 内容存在（实际语句落在 245-247 范围内，与 242-252 大范围一致） |
| `checker_known_bad_images.rs:242-251`（`propagate` 未被调用） | ✓ 核实：闭包体内只有 `read`/`set_u64`/`reseal_unit`/`write`，`propagate` 函数定义在同文件第 111 行，闭包内确未调用 |
| `checker_known_bad_images.rs:254-260`（I-4.8 坏镜像整单元清零、不 reseal） | ✓ 逐字一致 |
| **`checker_known_bad_images.rs:288-296`（「测试只断言目标不变量」的循环）** | **✗ 行号错**：288-296 行实际是另外两个不相关的坏镜像 case 定义（`I-1.4`超级块槽 mutate 与 `I-1.6` 容器身份段），不是断言循环。真正的 `for (invariant, mutation) in cases { … assert!(matches!(found, InvariantVerdict::Violated(_)), …) }` 循环在**第 562-567 行**（`for` 在 562，`assert!` 在 564-568）。内容本身真实存在，只是行号错误 |
| `common/mod.rs:1`（mkfs → 取号 → 暖机 → 第一个事务） | ✓ 逐字一致 |
| `common/mod.rs:194-211`（`warm_up` 调用、`assert_eq!(instance, InstanceGeneration(1))`） | ✓ 逐字一致 |
| **`decisions/16-发布语义.md:901`（F 恒 0 那句）** | **✗ 行号错，且更严重**：快照第 901 行是**空行**。真正含「F 恒 0（mkfs 与第一次发布时非空有效根不足 4 个，上限取最旧有效根 txg 0）」这句的位置是**第 412 行** |


### 本地攻方运行记录（`m2-step6-checker-r1-local-attack-runlog.md`）复核

| 声明 | 核的结果 / 命令 |
|---|---|
| 词数：s1=667、s2=725 | ✓ `wc -w` 现算逐字一致 |
| void1（第一次调用）判红，字词损坏 | ✓ `corruption-check.py` 判绿（cjk=0 汉字复读=0 英文复读=0…），`oov-check.py` **不带提示文件参数**时判红「生词=11 拼接=2」；**带提示文件参数**（`ask-local.sh` 内部实际调用形态 `python3 oov-check.py "$TXT" "$1"`）时判红「生词=5 拼接=2（`coincidental(=coincide+ntal)`、`qualifiesqualifies(=qualifies+qualifies)`）」——**与 runlog 逐字一致**；两种调用形态都判红（退出码 1），拒绝判定本身不受影响 |
| s1／s2 判绿，`oov-check.py` 生词表 | **部分核实**：**不带**提示文件参数复算得到 s1「生词=5（mutations checker's invariant's）」、s2「生词=11（comment's mutations checker's unjustified）」，与 runlog 逐字一致；但**带**提示文件参数复算得到的是 s1「生词=0」、s2「生词=1（unjustified）」——两种口径下判定都是绿（退出码 0），不影响样本被采信的结论，但 runlog 表格里这一列具体数字的口径（带不带提示文件参数）与 void1 那一行不一致，未在 runlog 里注明 |
| s1、s2 通读复查未见断句/孤立标点/缺头粘连词 | 核不动：这是人工通读结论，无法用命令复核，只能确认样本文件本身存在且内容与 runlog 摘录的样本特征相符（逐条核对翻译对照表时已通读全文，未发现新的损坏迹象，与 runlog 结论不矛盾） |

### 本地攻方腿计数

核了 18 处 kb/代码引用 + 3 处运行记录声明 = 21 处；✓ 17 处；✗ 2 处（`checker_known_bad_images.rs:288-296` 应为 562-567；`decisions/16-发布语义.md:901` 应为 412，且 901 行为空行）；部分核实/核不动 2 处（`oov-check.py` 口径差异；人工通读结论）。

## 四、本地辩方（`m2-step6-checker-r1-local-defense-*`）

转述核对表 `m2-step6-checker-r1-local-defense-translation-audit.md`：25 行。

| 原文文件:行 | 核的结果 |
|---|---|
| `image.rs:216-227`（`valid_roots` 函数范围） | ✓ 逐字一致，闭合 `}` 确在 227 行 |
| `decisions/23-journal的角色与格式.md:1209`（候选集定义，两处引用） | ✓ 逐字存在（同一超长行内） |
| `decisions/16-发布语义.md:376`（回退候选集复述） | ✓ 一致 |
| `decisions/16-发布语义.md:360-361`（用户定案逐字「选 1 最近四个状态……后来放弃了」） | ✓ 快照第 360 行逐字一致（含「这个本来是承诺所有 但是后来放弃了」原文的口语化表述） |
| `decisions/16-发布语义.md:374`（抬 F 上限公式） | ✓ 快照第 374 行逐字一致 |
| `invariants.md:50`（I-7.4 主句） | ✓ 一致 |
| `invariants.md:153`（I-4.8 主句 + 判别力句） | ✓ 一致 |
| `invariants.md:107`（I-2.1 定义 + 2026-09-17 状态列修订） | ✓ 一致 |
| `verification-build.md:111-112`（独立性硬约束） | ✓ 逐字一致 |
| `.claude/rules/fs-design.md:24`（传统遍历规则表行） | ✓ 逐字一致（此文件不在 kb 快照内，对主树当前版本核实，未见该行在本轮被改动的迹象） |
| `_m2-step6-checker-r1-background.md:18`（crates 里没有的三项） | ✓ 逐字一致 |
| `image.rs:44-49`（`Judgements` 结构体字段范围） | ✓ 逐字一致，闭合 `}` 确在 49 行 |
| `image.rs:53-63`（`judge` 方法） | ✓ 内容一致（实际范围 53-66，53-63 是其子集） |
| `image.rs:67-69`（`violation_count`） | ✓ 逐字一致 |
| `image.rs:71-77`（`not_applicable`） | ✓ 逐字一致 |
| `image.rs:79-98`（`into_report`） | ✓ 逐字一致，闭合 `}` 确在 98 行 |
| `image.rs:300-325`（`read_referenced_unit` 文档注释 + 函数体） | ✓ 与 Opus 引用的 301-324（不含文档注释行）一致，范围略宽属合理 |
| `walk.rs:143-194`（`read_index_node` 函数范围） | ✓ 起止行确为函数签名行与收尾 `}` |
| `walk.rs:158-164`（读失败分支） | ✓ 内容一致（实际语句落在 158-168 内） |
| `walk.rs:169-174`（`visited_units` 去重分支） | ✓ 逐字一致 |
| `walk.rs:47-65`（`Walk` 结构体） | ✓ 逐字一致，闭合 `}` 确在 65 行，`impl Walk<'_> {` 确在 67 行（首稿曾把两者混在一起，定稿已修正，本核对确认修正后的范围准确） |
| `walk.rs:690-904`（`check_pool_image` 函数整体范围） | 部分核实：起始/收尾未逐行核对（函数本体过长），已随上述多处子范围引用间接核实其存在 |
| `walk.rs:777`（最新根 `walk_root` 调用） | ✓ 一致 |
| `walk.rs:798-826`（候选根循环体范围） | ✓ 一致（实际内容 798-825，826 为循环收尾 `}`，边界基本准确） |
| `walk.rs:804`（候选根准入条件所在行，定稿修正后） | ✓ 逐字一致 |
| `walk.rs:806-807`（`mismatches_before`/`failures_before`） | ✓ 逐字一致——与本地攻方对应引文（807-809，行号偏 1）相比，这一份是准确的 |
| `walk.rs:808`（候选根 `walk_root` 调用） | ✓ 一致 |
| `walk.rs:811-813`（`walked_into_reused_or_erased_unit`） | ✓ 逐字一致 |
| `walk.rs:809-810`（机制注释） | ✓ 逐字一致 |
| `walk.rs:815-820`（I-7.4 判定调用） | ✓ 逐字一致 |
| `walk.rs:821-824`（I-4.8 判定调用） | ✓ 逐字一致 |
| `walk.rs:818`（I-7.4 违例消息文本） | ✓ 精确落在第 818 行 |
| `walk.rs:805`（`older_candidates` 自增） | ✓ 一致 |
| `walk.rs:827-829`（`older_candidates == 0` 分支与理由串） | ✓ 逐字一致，理由串精确落在第 829 行 |
| `walk.rs:776-785`（最新根 I-4.8 单格判据） | ✓ 逐字一致（776 行「先走最新根」注释、781-785 判定调用均核实） |
| `walk.rs:832`（I-7.2 复用 `newest_failures.is_empty()`） | ✓ 一致 |
| `walk.rs:776`（「先走最新根」注释） | ✓ 精确落在第 776 行 |
| `checker_known_bad_images.rs:544-549`（干净镜像逐条判 Holds 循环） | ✓ 逐字一致 |
| `image.rs:36-40` / `checker_known_bad_images.rs:551-561`（26 条清单对应） | 核不动：本轮提示未展开引用具体行号内容（核对表自陈「本轮未在提示中单独引用行号」），只借用了已核实过的「26」这个数（见 Sonnet 节 `grep -vc` 核实） |


### 本地辩方运行记录（`m2-step6-checker-r1-local-defense-runlog.md`）复核

| 声明 | 核的结果 / 命令 |
|---|---|
| void1 判红，`生词=5、拼接=1（"mistakenlyes"）` | ✓ **带提示文件参数**复算：`红 … 生词=5 拼接=1`，`拼接: mistakenlyes(=mistaken+lyes)`，`生词: invariant's overwrote contradicting misreports mistakenlyes`——逐字与 runlog 一致（不带参数复算是生词=14，同本地攻方节同一口径差异） |
| s1（第 1 次）人工核出损坏：第 2 行 "confl"（截断词）、"resetting reset"（孤立多余词） | ✓ `sed -n '2p' m2-step6-checker-r1-local-defense-output-s1.md` 现读，逐字确认「However, this confl not distinguish…」与「without resetting reset, a node…」两处缺陷真实存在；`oov-check.py`/`corruption-check.py` 对此文件判绿，与 runlog「闸没抓到、人工核出」的表述一致 |
| s3（第 4 次）人工核出损坏：第 14 行 "verifiesuses" 被计入生词栏而非拼接栏 | ✓ `sed -n '14p'`现读确认原文；`oov-check.py`（带提示参数）报「生词=5…verifiesuses…」，未出现在「拼接」栏——与 runlog 描述的失效形态（计入生词栏而非拼接栏）逐字相符；`corruption-check.py` 判绿 |
| s2、s4 判定为「干净」（计入两份干净样本） | ✓ `corruption-check.py`、`oov-check.py`（带提示参数）对 `-output-s2.md`、`-output-s4.md` 均判绿，退出码 0 |
| 「5 次调用中 1 次触发字词损坏闸（退出码 5）」 | 部分核实：直接复算 void1 判红（退出码 1，工具本体），与 runlog 描述的 `ask-local.sh` 外层退出码 5（工具判红后脚本自身的退出码）一致；两者是同一事件在不同层面的编号，不矛盾 |

### 本地辩方腿计数

核了 37 处 kb/代码引用 + 4 处运行记录声明 = 41 处；✓ 39 处；✗ 0 处；核不动/部分核实 2 处
（`image.rs:36-40`/`checker_known_bad_images.rs:551-561` 未展开引用；`check_pool_image` 整体范围未逐行核）。

## 五、总计

| 腿 | 核了 | ✓ | ✗ | 部分核实/核不动 |
|---|---|---|---|---|
| 云端攻方（Opus） | 28 | 28 | 0 | 0 |
| 云端正推（Sonnet） | 26 | 24 | 2 | 3（含 1 项旁证支持但不算直接核实） |
| 本地攻方 | 21 | 17 | 2 | 2 |
| 本地辩方 | 41 | 39 | 0 | 2 |
| **合计** | **116** | **108** | **4** | **7** |

四处 ✗ 汇总（均为引用定位错误，不是内容伪造）：

1. Sonnet 引 `research/prompts/m2-step45-code-r2-main-verification.md:71`，实际内容在**第 72 行**（第 71 行是另一条无关决策点）。
2. Sonnet 引 `.claude/kb/invariants.md:12`（声称整行抄，含「24 条」与一句编辑残留插入语），当前快照该行实为「…就是这 **26** 条…」、无插入语；这段「24 条」文本**逐字见于** `research/prompts/_m2-step6-checker-r1-background.md:794` 与 `_m2-step6-checker-r1-appendix.md:7`（两处均自称「整段抄，未转述」），据此判断 Sonnet 的引文实际来自背景材料里一处过渡态/拼接文本，而不是当前 kb 文件本身；这条纪律对应 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「喂给多方论证的背景材料，本身要先核」——背景材料自己的「整段抄」标注在这里不实。
3. 本地攻方引 `checker_known_bad_images.rs:288-296`，实际断言循环在**第 562-567 行**；288-296 行是另外两个坏镜像 case 定义，与引用描述的内容无关。
4. 本地攻方引 `decisions/16-发布语义.md:901`，该行**为空行**；真正含「F 恒 0」那句的是**第 412 行**。

四处均已用 `awk 'NR==A,NR==B'` 现查快照 / 主树给出正确行号，不只是打 ✗。

## 没做什么

- 不判一条打中成不成立、该不该采纳；不核推理本身（Y1-Y6/U1 各格的判定是否正确、P1 改法该不该被接受、"24 应改 26" 这条发现本身有没有价值），只核引用、产物与复跑。
- 不判「误写成背景材料」这一条发现（invariants.md:12）对应的现象应该怎么改（例如是否要在 `_m2-step6-checker-r1-background.md` 里补一条自我更正）——这是主 agent 与写材料的会话之间的事，本报告只如实指出现象。
- 未跑门禁 54 号层 0 全量（本机在跑，遵派发提示要求未碰）；未跑门禁 59 号完整 55 条变异表（只复跑了 Sonnet 报告点名的「步 6」两行，与 Sonnet 自己的说明一致）。
- Sonnet 的 `SONNET_DEBUG_*` 系列调试打印输出（Y4-1/Y4-2 节）未直接复现（无随附 diff/模型文件可用），只用 Opus 独立提供的 `zz_attack_probe.rs` 做了一次间接交叉验证（判定行逐字相同），不构成直接复跑核实。
- `walk.rs:690-904`（`check_pool_image` 整体范围）、`second_transaction_step_five_reuse.rs:313-338`（区间终止行）、`.claude/kb/milestone/02-second-txn.md:216`（超长段落的完整逐字核对）未做完整逐行/逐字核对，只核了关键片段与边界。
- 本地攻方/辩方 runlog 里「人工通读，未见断句/孤立标点/缺头粘连词」这类结论性判断核不动（无法用命令复核纯人工阅读结论），只确认了样本文件真实存在、内容与摘录特征相符。
- `oov-check.py` 带不带提示文件参数会给出不同的「生词」计数（均不影响红/绿判定本身），本报告已如实记录这一口径差异，但未去追溯本地攻方/辩方两条腿的 runlog 表格具体是用哪种调用形态产生每一行数字的（可能是分别用两种形态手工核对后各自摘录，也可能是笔误）——这属于文字表述准确性问题，留给主 agent 判断是否需要补注。
- 未核实 main-verification.md 正文（第一至第八节）本身的判决内容，按任务范围只字核了四条腿各自的报告，main-verification.md 只作为理解上下文之用被读过。
- 未复跑 `crates/singlefs-checker` 里 26 条不变量清单与 `IMPLEMENTED_INVARIANTS` 常量逐一比对（Sonnet 报告 Y6 节自称已核对，未独立复算）。

