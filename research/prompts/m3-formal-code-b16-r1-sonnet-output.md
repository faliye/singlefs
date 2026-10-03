# m3-formal-code-b16-r1 云端正推腿报告（Sonnet，格 K1、K3、K4，2026-10-04）

范围：正文 `research/prompts/_m3-formal-code-b16-r1-body.md` 第四节分给正推腿的三格。代码读冻结副本 `/tmp/claude-1000/m3-formal-code-b16-r1/tree/`（下文路径都是那棵树里的，行号是在那棵树里 `grep -nF` 现取的）；kb 行号是在仓里 kb 文件里现取的。方案按 `crates/` 今天的实现来谈。

## 〇、复核的前提与我跑了什么

冻结副本与快照清单对得上：

```
$ cd /tmp/claude-1000/m3-formal-code-b16-r1/tree && sha256sum -c .../m3-formal-code-b16-r1-snapshot/crates-sha256.txt | grep -c ': OK$'
264
$ ... | grep -vc ': OK$'
0
```

我自己拷了一份工作区跑 harness 档（只跑 `singlefs-harness` 的两个新测试文件，经 `capped.sh 8` 与 `run-with-memory-cap.sh`；checker 档一条没跑）：副本 `/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/ws/`（冻结副本的 `crates/` 加仓根 `Cargo.toml`、`Cargo.lock`），编译目录 `/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/target/`。基线：

```
test the_model_does_not_answer_unmount_on_a_version_without_trees ... ok
test unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees ... ok
test an_unmount_whose_roots_carry_the_floor_of_the_current_version_instead_of_its_txg_is_reported_at_the_rollback_floor ... ok
test unmount_on_an_overwrite_at_txg_four_pushes_three_empty_publishes_carrying_floor_four_and_the_model_agrees ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.49s
test a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal ... ok
test the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal ... ok
test a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

（输出原样存 `/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/baseline1.log`。）

脚本与模型都在 `research/prompts/m3-formal-code-b16-r1-sonnet-model/`：`run_harness_mutation_rows.py`（在副本里逐行套 `crates/mutations.tsv` 的行、跑点名参数、还原并逐字节比对）、`verify_shared_loop_verbatim.py`（核 K1 的共用循环逐字不变）、`read_faults_in_a_crash_state_standalone.rs`（K4 里两条纯逻辑内联用例的独立小程序）。

## 一、K1　16-2 卸载的答对不对

### K1.1 条款逐句对代码

条款（D16 已定项 1 表里「正常卸载」一行，整行抄）：

「推一串空发布把 F 抬到现行那一版的 txg（「抬 F 那一串」照准入的走法，上限见「抬 F 的上限」那一行）；这一串写的根在根记录 flags 里带卸载记号（D22（单元原子性怎么合成） 已定项 7）；第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次（现行 txg 除以 3 余 1 时 3 次，余 0 或余 2 时 2 次）；不加干净关闭标记，下次挂载照旧走恢复（D23（journal 的角色与格式） 已定项 14）」（`.claude/kb/decisions/16-发布语义.md:42`）

| 条款这一句 | 代码做的 | 判定 |
|---|---|---|
| 「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`） | 「let floor_raised_to_the_current_txg = current.key.checkpoint_txg;」（`crates/singlefs-harness/src/model.rs:2480`），交给共用函数当新 F；实现一侧「let rollback_floor = current_file_version.root.checkpoint_txg;」（`crates/singlefs-core/src/mount.rs:2352`） | 一致 |
| 抬 F 那一串「空发布推到每块盘上都有一条带新 F 的根为止」（`.claude/kb/decisions/16-发布语义.md:41`） | 共用函数的循环条件 「&& u64::try_from(roots.len()).expect("次数") < ROOT_RING_REGIONS」（`crates/singlefs-harness/src/model.rs:2433`）：有盘没罩到且次数不到 R 就接着推 | 一致 |
| 「第一版几何（R = 3、区域归属 0 / 1 / 0）下空发布 2 或 3 次」 | 「pub const ROOT_RING_REGION_DEVICES: [u32; 3] = [0, 1, 0];」（`crates/singlefs-format/src/lib.rs:288`）；我按 txg 余数推：余 0 → 推 txg 余 1、余 2 两条（盘 1、盘 0）共 2 次；余 1 → 余 2、余 0、余 1（盘 0、0、1）共 3 次；余 2 → 余 0、余 1（盘 0、1）共 2 次，与条款三种余数都对上。测试钉了余 0（txg 3 → 2 次）与余 1（txg 4 → 3 次），余 2 没有用例钉 | 一致（余 2 一格是推的、没有用例） |
| 卸载「上限见『抬 F 的上限』那一行」：「**卸载抬 F**：抬到现行那一版的 txg，不另判上限（等价于上限取最新的持久有效根，不按盘取小）」（`.claude/kb/decisions/16-发布语义.md:37`） | 模型给 `required_refusals`、`permitted_refusals` 都传 `BTreeSet::new()`（`crates/singlefs-harness/src/model.rs:2486` 起那个调用），不答上限；实现 「RaiseFloorEntry::Unmount => None,」（`crates/singlefs-core/src/mount.rs:2689`） | 一致 |
| 「这一串写的根在根记录 flags 里带卸载记号」，D22 已定项 7 flags 行「位 0 = 卸载记号：正常卸载那一串写的根置 1」（`.claude/kb/decisions/22-单元原子性怎么合成.md:137`） | 实现：「RaiseFloorEntry::Unmount => UnmountMarker::WrittenByTheUnmountSequence,」（`crates/singlefs-core/src/mount.rs:2247`）。模型与对拍都不比 flags：「pub struct ObservedRoot {」（`crates/singlefs-harness/src/model.rs:809`）只有 key、jsn、F、文件、实例表、分配记录六项，没有 flags | 规则说了，这条路径的对拍没覆盖；不冲突。记号另由 checker 档用例钉（`crates/singlefs-checker-tier/tests/rollback_floor_written_into_the_system_configuration_first_and_normal_unmount.rs`，本轮没跑） |
| 「不加干净关闭标记，下次挂载照旧走恢复」 | 模型没有这一项，实现注释写「不写干净关闭标记」（`crates/singlefs-core/src/mount.rs:2328` 同一行） | 模型不涉及；不冲突 |
| 做完会话关掉（`session_after_success = Closed`） | 模型：「answer.session_after_success = ModelSessionAfterSuccess::Closed;」（`crates/singlefs-harness/src/model.rs:2493`）；实现注释「做完之后这个写入口不再发布。」（`crates/singlefs-core/src/mount.rs:2328`） | 规则没说：D16 表与 D22 已定项 7 都没写「卸载之后会话关掉」；依据只有实现注释与射程那一句「卸载之后 F = 现行那一版的 txg，回退候选集只剩现行那一版」（`.claude/kb/decisions/16-发布语义.md:57`）。模型的 Closed 与 F 生效值 = 现行 txg 与射程那句相容 |
| 先写系统配置那一步（`.claude/kb/decisions/16-发布语义.md:41` 同一行） | 模型不建系统配置（`crates/singlefs-harness/src/model.rs` 的 `answer_unmount` 文档自己写明）；`observed_unmount` 不交 `system_configuration_writes_before_the_first_publish` | 模型没覆盖，文档已声明；不冲突 |

模型对 `answer_raise_rollback_floor` 有一道新 F 低于现行 F 就报 `NotModeled` 的判断（「if new_floor < current.rollback_floor {」，`crates/singlefs-harness/src/model.rs:2377`），`answer_unmount` 没有（转述；`answer_unmount` 的文档只写了「模型里没有可写会话」一种 Err 与树表 0 条）；实现一侧卸载与抬 F 共用「if new_floor < effective_floor {」（`crates/singlefs-core/src/mount.rs:2663`，比的是盘上现算的 F 生效值）。模型里 F 恒不高于现行 txg，所以新 F = 现行 txg 在模型里走不到这一拒；实现里走到它要「系统配置先写了一部分盘」或被抛弃根带更高 F 的构造镜像，模型不建这两样。推的，没造这一形。

### K1.2 `observed_unmount` 交给对拍的是哪几条根、漏没漏

- 带文件的一版：交抬 F 那一串写出的**每一条**空发布的根，一条不落——「.map(observed_root_of_file_version)」（`crates/singlefs-harness/src/model_comparison.rs:651`），与随机历史里抬 F 的交法同一个写法（「.map(observed_root_of_file_version)」，`crates/singlefs-harness/src/history.rs:3254`）。条数对不上由对拍比长度报 `PublishCount`（「if expected_roots.len() != observed_roots.len() {」，`crates/singlefs-harness/src/model.rs:2817`）；我在副本里把交回截成第一条，点名用例红（K4 行 2220）。
- 树表 0 条：「Unmounted::NothingWrittenOnAVersionWithoutFile { .. } => Vec::new(),」（`crates/singlefs-harness/src/model_comparison.rs:653`）交零条根；但模型在这一版上不答（K1.4），这一支今天没有调用点会比它。
- 没交的：`reclaimed`、`system_configuration_writes_before_the_first_publish`、`abandoned_roots_unreadable`、flags 里的卸载记号、拒绝一侧（没有 `observed_unmount` 的拒绝版：`mount::unmount` 返回 `Err(MountError)` 时没有函数把它翻成 `ObservedRefusalReason`）。前四样模型本来不比；拒绝一侧模型有答法（`walls_are_judged_before_the_first_write`、容量墙区间）而对拍没有人把实现的拒交过来，这一格今天空着。
- 没接线：`observed_unmount`、`answer_unmount` 全仓只在新测试文件里被调（`grep -rn 'observed_unmount\|answer_unmount' singlefs-*/src singlefs-*/tests`，除新测试文件外 src 里只剩两处定义行）；`HistoryOperationKind` 只有十种、没有卸载（「pub enum HistoryOperationKind {」，`crates/singlefs-harness/src/history.rs:503`）；E171 的执行器注释仍写着「模型没有卸载的答」（「卸载那次调用在 `calls.tsv` 里的标签：模型没有卸载的答（登记第一节读法 6），这次调用不对拍。」，`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs:147`）。所以 C673 ④ 补的这个 oracle 只被手搭的单元用例用到，E171 自己的流、随机历史、崩溃注入都没有调它。

判定：交的根不漏；没交的几样都在模型文档里声明了；「拒绝一侧没有观测函数」与「没接进任何执行器」是两处今天空着的格，不是错，但补 oracle 的覆盖面只到那四条手搭用例。

### K1.3 抽出的共用循环对抬 F 路径是不是逐字不变

做法：把背景 diff 里 `model.rs` 那一段反向套回冻结副本得到合入前的文件（`patch -R`），取合入前 `answer_raise_rollback_floor` 里 `let mut roots …` 到循环体收尾那一段，与合入后 `empty_publishes_carrying_the_floor_to_every_device` 里同一段逐行比。脚本 `research/prompts/m3-formal-code-b16-r1-sonnet-model/verify_shared_loop_verbatim.py`，输出原样：

```
合入前循环段行数 28 ；合入后 29
合入后多出的行： ['        // 迭代上界是根环区域数 R；跨轮带的是已写出的根与已罩到的盘；没有提前出口。']
合入前被去掉的行： []
去掉新加注释后逐行相同： True
```

调用点：抬 F 一侧「let roots = self.empty_publishes_carrying_the_floor_to_every_device(session, new_floor);」（`crates/singlefs-harness/src/model.rs:2403`）接在原来的 `match ceiling` 之后、`answer_for_publishes` 之前，与合入前同一位置；`answer_for_publishes` 的实参（`RaiseRollbackFloor`、`required_refusals`、`permitted_refusals`、`roots`）与之后的 `rollback_floor_ceiling`、`walls_are_judged_before_the_first_write` 两行在 diff 里只是被 git 挪了位置。判定：一致。局限：合入前文件是用主 agent 给的 diff 反推的，不是独立取到的合入前备份（`/tmp/claude-1000/` 下没有 batch16-premerge）；diff 若本身有误，这一条跟着错。

### K1.4 树表 0 条模型报「答不了」是不是唯一的读法

不是唯一读法，但报「答不了」站得住。

- 条款字面：「推一串空发布把 F 抬到现行那一版的 txg」（`.claude/kb/decisions/16-发布语义.md:42`）没有排除树表 0 条的一版；D16 已定项 9 写明树表 0 条的一版上空发布有定义：「第一次可写挂载的暖机时树表 0 条 ⇒ 零单元」（`.claude/kb/decisions/16-发布语义.md:221`）。
- 实现里这种发布造得出来、也能带 F：`ZeroUnitPublishPlan` 有 F 字段（「pub struct ZeroUnitPublishPlan {」，`crates/singlefs-core/src/transaction.rs:1053`；字段「pub rollback_floor: CheckpointTxg,」，`crates/singlefs-core/src/transaction.rs:1060`），`publish_without_units` 没有卸载记号参数（「pub fn publish_without_units<Device: BlockDevice>(」，`crates/singlefs-core/src/transaction.rs:1313`）。
- 所以至少两种读法：甲，树表 0 条的一版上推零单元的空发布带新 F（条款字面）；乙，一个字节都不写（实现：「PoolVersion::WithoutFile(current_version_without_file) => {」，`crates/singlefs-core/src/mount.rs:2343` 起直接 `return Ok(Unmounted::NothingWrittenOnAVersionWithoutFile …)`）。乙的出处只有实现注释的「主 agent 2026-09-26 定、没进 D16」；我在 kb 里找它：

```
$ grep -rn '树表 0 条.*卸载\|卸载.*树表 0 条' .claude/kb/*.md .claude/kb/decisions/*.md | wc -l
0
$ grep -rn 'NothingWrittenOnAVersionWithoutFile\|抬 F 买不到' .claude/kb | wc -l
0
```

  kb 里没有乙的条款。模型在这一格 `NotModeled`（`crates/singlefs-harness/src/model.rs:2470` 的 `if !current.contents.has_trees() {`）对两种读法中立，不替实现背书；说「条款没判」是对的，说它是唯一读法不对。
- 模型的「树表 0 条」与实现的 `WithoutFile` 对得上：`has_trees` 对 `WithoutTrees` 假、其余四种真；实现 `WithFile | WithTreesAndNoFile` 走抬 F、`WithoutFile` 不写（`crates/singlefs-core/src/mount.rs:2342`）；模型的 `InodesWithoutTheFirstFile` 文档写明实现把它归带文件的一版（`crates/singlefs-harness/src/model.rs:312` 的枚举注释）。

### K1 判定

一致：新 F 取现行 txg、推到每块盘都有一条、不判上限、共用循环逐字不变、交的根不漏。冲突：无。规则没说：卸载之后会话关掉；树表 0 条的一版推不推；卸载记号对拍没覆盖；拒绝一侧没有观测函数。

什么会推翻：① 余 2 的 txg（如 5）上卸载，实现推的次数不是 2；② 把 `answer_unmount` 的 `new_floor` 改成别的值而 `unmount_on_the_first_file…` 与 `…overwrite…` 两条仍绿（我套了变异，见 K4 行 2215、2216，两条都红）；③ kb 里找到乙的条款原文，则「条款没判」改判「条款已判、模型可以答」。

## 二、K3　16-4 GPU 那一格与 CPU 是不是同判

全部是读码推的：GPU 判器要显卡，属于 checker 档，我没跑；下面每一条「一致」都是两份源码逐句比出来的，不是跑出来的。

### K3.1 `newest_root` 与 CPU 判器的「最新那条根」是不是同一条

| 比什么 | CPU（`crates/singlefs-checker/src`） | GPU（`crates/singlefs-checker-tier/src`） | 判定 |
|---|---|---|---|
| 候选根集合 = 自证过的根 | 「let roots = valid_roots(reader, &geometry);」（`crates/singlefs-checker/src/walk.rs:6865`），`valid_roots` 取 `check_root_slot` 过了的槽（`crates/singlefs-checker/src/image.rs:1136`） | 「if (!candidate.valid) {」后 `continue`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:277`），`checker_collect_roots` 只把自证过的根记进根表 | 一致：撕裂的、没写过的槽两边都被滤掉，进不了「最新根」的比较 |
| 槽的次序（平局时谁在前） | 「pub fn root_slot_positions(geometry: &PoolGeometry) -> Vec<(u64, u64, u32, u64)> {」（`crates/singlefs-checker/src/image.rs:850`）：区域在外、槽在内 | `checker_collect_roots` 的双层循环同序：区域在外（「for (var region = 0u; region < g.regions; region = region + 1u) {」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:271`）、槽在内（「for (var slot = 0u; slot < g.slots_per_region; slot = slot + 1u) {」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:273`），偏移式子同为 base + region × prime_step × chunk + slot × spacing | 一致 |
| 择根键 | 「let choice_key = (root.checkpoint_txg, root.instance);」（`crates/singlefs-checker/src/walk.rs:6955`） | 比 `root_key_less(newest_root.txg, newest_root.instance, r.txg, r.instance)`（`wgsl:2875`），键同为（txg，实例） | 一致 |
| 平局 | 「Some((_, newest_key)) if choice_key <= newest_key => newest_so_far,」（`crates/singlefs-checker/src/walk.rs:6957`）：只在严格更大时才换，相等取读根环次序最前的 | `if (root_key_less(…))` 才换（`wgsl:2875`），也是严格小于才换，初值是第 0 个（「var newest_root = checker_root_of(0u);」，`wgsl:2872`） | 一致：两边平局都留先读到的那条 |
| 一条根都没有 | 根环空时返回、不走到 I-7.8（`walk.rs` 里 `if roots.is_empty() {` 那一支，即 `crates/singlefs-checker/src/walk.rs:6938` 的「if roots.is_empty() {」） | 「if (root_count == 0u) {」后 `violate(INVARIANT_I_7_1); return 2u;`（`wgsl:3600`），走不到 `walk_and_hand_off` | 一致：新格读的 `newest_root`（初值取第 0 个）在 root_count = 0 时到不了 |

GPU 的 `newest_root` 在函数顶上取一次、之后没有再赋值（`grep -n newest_root` 只有 2872、2875–2877、2881–2882、2901、3157、3228、3236 这几处，其中赋值只在 2877），新格 3157 读到的就是择根那一条。判定：一致（读码）。

### K3.2 `highest_seen` 两边口径一样吗

新格与取 max 那一格读的是**同一个** `highest_seen`：CPU 先算一次（「let highest_seen = seen.iter().max().copied().unwrap_or(0);」那一行在 `crates/singlefs-checker/src/walk.rs:7503`），两格都比它（「judgements.judge("I-7.8", watermark > highest_seen, || {」，`walk.rs:7504`；「judgements.judge("I-7.8", newest_root_watermark > highest_seen, || {」，`walk.rs:7510`）；GPU 同样算一次（「var highest_seen = vec2<u32>(0u, 0u);」，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:3107`），两格都比它（「if (!lt64(highest_seen, watermark)) {」，`wgsl:3151`；「if (!lt64(highest_seen, newest_root.watermark)) {」，`wgsl:3157`）。16-4 没有动 `highest_seen` 的算法，所以「CPU 与 GPU 的 `highest_seen` 等价」是取 max 那一格早就有的前提，新格不添新的口径。不等式方向：CPU「违例 ⟺ !(最新水位 > highest_seen)」，GPU「违例 ⟺ !lt64(highest_seen, 最新水位)」，同一个式子。水位字段：GPU 读根记录偏移 122（「r.watermark = rd64(r.base, 122u);」，`crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl:183`）。判定：一致；`highest_seen` 在两边到底算得一不一样，我没有独立核（那是 GPU 对拍用例的事）。

### K3.3 条款

kb 里已有这一格的条款：I-7.8 行「判法补一句」写「另判最新那条根的水位不低于全池最大树 ID」（`.claude/kb/invariants.md:62`），并写它在池级 checker 里已实现；C674 行写补的方向「GPU 判器补最新那条根自己的水位那一格，与 CPU 判器同判」（`.claude/kb/checks-owed.md:532`）。16-4 加的一格就是这一句在 GPU 一侧的对应。条款没有写 GPU 判器一侧，这一格的「要求 GPU 同判」来自 C674 的欠账行，不是已定项。

### K3.4 判法摘要随内核原文变、复用的旧判定会不会被误用

- 摘要里收内核原文：「message.extend_from_slice(GPU_JUDGE_CHECKER_SOURCE.as_bytes());」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:167`），该常量是「pub const GPU_JUDGE_CHECKER_SOURCE: &str = include_str!("crash_judge_checker.wgsl");」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:30`）。
- 流水线实际用的判法版本是判器闭包摘要：「JudgeVersion(self.code.digest)」（`crates/singlefs-checker-tier/src/crash_amplification.rs:677`），闭包里五份内核按原文进摘要，有用例断言（「assert!(code.raw_files.contains(&path), "内核 {path} 要按原文进摘要");」，`crates/singlefs-checker-tier/src/crash_identity.rs:1377`），还有一条用例断言「内核原文改一个字节要动判法摘要」（`crates/singlefs-checker-tier/src/crash_identity.rs:1488`）。
- 存的键带判法版本：判定块与违例按（块键，判法版本）存，「判法代码的摘要 [`gpu_judge_digest`] 进判定表的键：内核改一个字，旧行留着当历史，新行另起。」（`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:14`）；复用只认同一版本：「同一块在几个版本下的判定并存、旧版本的行留作历史，复用只认同一版本；录入表只按块键。」（`crates/singlefs-checker-tier/src/verdict_store.rs:48`）。

所以 wgsl 加了一格之后摘要变、旧摘要下判过的块不复用。后果：提交信息里「26.16 亿态零红（含 σ）」那批是旧摘要下判的，新摘要下要全量重判才能再引；这不是错用旧判定，是它们被正确地隔开了。我没有跑流水线，不知道重判会不会冒新红。

### K3.5 这一格的判别力（读码推的）

新测试文件先在 CPU 上断言前提：原样流 I-7.8 不红，压低最新根水位之后**只**红 I-7.8，且说明以「最新那条根」开头、即取 max 那一格没红（`crates/singlefs-checker-tier/tests/gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker.rs` 里 `NEWEST_ROOT_WATERMARK_VIOLATION_PREFIX` 与对 `lowered["I-7.8"]` 的断言），再逐状态比 GPU 与 CPU 的池级违例位。变异（`mutations.tsv` 第 2229 行把新格改成 `&& false`）下，B 的根已落的状态上 CPU 红、GPU 不红，逐状态比必红。这条推理的两个前提：压低的水位（mkfs 时的水位）确实不高于盘上出现过的树 ID——测试自己断言了（压到的水位低于 B 的根原来的水位，且 CPU 前提判红）；显卡上 GPU 判器本身对这些状态判得出、不报「判不了」——测试把「判不了」也算不一致。基线绿、变异红都没跑，归提交时。

### K3 判定

一致（读码）：最新根是同一条（候选集、次序、择根键、平局、空环），`highest_seen` 与不等式方向同，摘要随内核原文变。冲突：无。规则没说：GPU 一侧要与 CPU 同判没有已定项，只有 C674 欠账行。

什么会推翻：① 在显卡上造一个最新根水位压低而取 max 那一格不红的状态，GPU 与 CPU 违例位不同；② 一个有平局根（同 txg 同实例）的镜像，两边选的「最新根」水位不同；③ 一份旧摘要下的判定行在新摘要的库里被读出来复用。

## 三、K4　变异行与测试的判别力

15 行：`crates/mutations.tsv` 第 2215–2229 行，六列都在（数过）：

```
$ sed -n '2215,$p' crates/mutations.tsv | wc -l
15
$ sed -n '2215,$p' crates/mutations.tsv | awk -F'\t' '{print NF}' | sort | uniq -c
     15 6
```

锚点（第 3 列原文）在各自源文件里都恰好命中一次（我用同一个核法逐行数过，15 行都是 1；`mutate.sh` 的锚点闸也是这一条）。点名的测试在各自文件里各恰好一个 `fn`、且 `--` 后的过滤串选得中它（脚本核过，15 行都是）。

### K4.1 套在副本里跑过的 10 行（harness 档，第 2215–2224 行）

跑法：`research/prompts/m3-formal-code-b16-r1-sonnet-model/run_harness_mutation_rows.py <副本> <行号…>`，在 `/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/ws` 里套改、跑该行第 5 列的 cargo 参数、还原并逐字节比对（跑完四个被改的源文件与冻结副本 `cmp` 相同）。每行都是「基线绿、变异下点名的测试红」：

| 行 | 改的是什么（第 3 列锚点处） | 点名的测试 | 变异下红的测试 | 判定 |
|---|---|---|---|---|
| 2215 | `model.rs`：「let floor_raised_to_the_current_txg = current.key.checkpoint_txg;」改成 `current.rollback_floor` | `unmount_on_the_first_file_at_txg_three…` | 仅它（1 红） | 点名处对、测试红 |
| 2216 | `mount.rs`：「let rollback_floor = current_file_version.root.checkpoint_txg;」（`crates/singlefs-core/src/mount.rs:2352`）改成 `…root.rollback_floor`（与第 706 行同一个原文、同一个替换文，只是点名的测试换成 harness 档的模型对拍用例） | 同上 | txg 3 与 txg 4 两条都红（2 红 2 绿） | 点名处对、测试红；与第 706 行重复的是改法，不是判别面：新加的是「模型对拍也抓得到」 |
| 2217 | `model.rs`：树表 0 条那道 `if !current.contents.has_trees() {` 改成 `if false {` | `the_model_does_not_answer_unmount_on_a_version_without_trees` | 仅它 | 点名处对、测试红 |
| 2218 | `model.rs`：「if expected.rollback_floor != observed.rollback_floor {」（`crates/singlefs-harness/src/model.rs:2854`）改成 `if false {` | `an_unmount_whose_roots_carry_the_floor…` | 仅它 | 点名处对、测试红 |
| 2219 | `model.rs`：「answer.session_after_success = ModelSessionAfterSuccess::Closed;」（`crates/singlefs-harness/src/model.rs:2493`）改成 `StaysOpen` | `unmount_on_the_first_file_at_txg_three…` | 仅它（断言 `!has_writable_session`） | 点名处对、测试红 |
| 2220 | `model_comparison.rs`：交回只取第一条根（`.take(1)`） | `unmount_on_the_first_file_at_txg_three…` | 仅它 | 点名处对、测试红 |
| 2221 | `model.rs`：循环上界「< ROOT_RING_REGIONS」（`crates/singlefs-harness/src/model.rs:2433`）改成 `< 2` | `unmount_on_an_overwrite_at_txg_four…` | 仅它 | 点名处对、测试红；这一行同时改了抬 F 的共用循环，抬 F 路径的别的用例没在这一条的过滤里 |
| 2222 | `mount_transparency.rs`：新判官里 `conditions_not_met.push(…WitnessCriterionTrueOnACrashImageWithoutReadFaults(branch))` 换成不推 | `a_witness_refusal_on_a_crash_image_without_read_faults_where_only_a_rotation_landed_is_illegal` | 仅它（2 绿） | 点名处对、测试红 |
| 2223 | 同文件：调用点改成走 `newer_publish_witnessed_on_the_crash_image(…).then_some(Neither…)`（撕裂的槽按甲当判据为真） | `a_refusal_on_a_crash_image_without_read_faults_where_a_device_has_both_system_configuration_slots_torn_is_legal` | 它与 `…only_a_rotation_landed_is_illegal` 两条红 | 点名处对、测试红；另一条红是因为判据为真的分支名换了，不算误中 |
| 2224 | 同文件：`NoneInjected` 臂改成 `NoneInjected \| Injected` | `the_same_witness_refusal_in_a_crash_state_with_read_faults_injected_is_legal` | 仅它（2 绿） | 点名处对、测试红 |

逐行输出原样存 `/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/` 下的 `mut_2215_2217.log`、`mut_2216_2219.log`、`mut_2220_2222.log`、`mut_2223_2224.log`。`mut_2215_2217.log` 里第 2216 行那一条退 251（包装拒绝：上限 8G 超过峰值表里这条命令实测峰值 1148M 的 2 倍），包装的结局，不算结果；`mut_2216.log` 同样是那次退 251 的复现；按包装说明设 `RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON` 之后第 2216 行的结果在 `mut_2216_2219.log`。下面是第 2216 行与第 2223 行两条最能说明问题的原样输出：

```
=== 行 2216: C673 ④ 理想模型给卸载的答（D16 已定项 1「正常卸载」）：实现卸载把 F 抬到现行那一版带的 F（原地不动）而不是它的 txg（E171 第 42 行的形态，由模型对拍判）
    文件 crates/singlefs-core/src/mount.rs；锚点命中 1 次；点名测试 unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees
    退出码 101；编不过 False
    红的测试 ['unmount_on_the_first_file_at_txg_three_pushes_two_empty_publishes_carrying_floor_three_and_the_model_agrees', 'unmount_on_an_overwrite_at_txg_four_pushes_three_empty_publishes_carrying_floor_four_and_the_model_agrees']
    绿的测试数 2；点名测试红 True；点名测试绿 False
    汇总行 ['test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.65s']
```

### K4.2 checker 档 5 行（第 2225–2229 行）：只能写「推的」

**第 2225、2226 行**（`crash_injection.rs` 里 `read_faults_in_a_crash_state` 的两改）：被测的是一个纯函数，不需要池也不需要显卡。我把函数体原样手搬进一个独立小程序（`research/prompts/m3-formal-code-b16-r1-sonnet-model/read_faults_in_a_crash_state_standalone.rs`，只配了最小的枚举桩，函数体是手搬的、没有机器比对），用 `rustc` 编基线与 `--cfg m2225`、`--cfg m2226` 两个变体，在两条内联用例的输入上跑（经 `run-with-memory-cap.sh 1G`）：

```
base: 用例一 绿；用例二 绿
m2225: 用例一 红；用例二 绿
m2226: 用例一 绿；用例二 红
```

用例一 = `a_crash_state_counts_as_read_faulted…`，用例二 = `an_abandoning_recovery_that_sent_nothing…`。第 2225 行（`if false && …`）只红用例一，第 2226 行（去掉「这一步发了东西」那半个条件）只红用例二，点名的测试各自对得上、互不串。这只证明函数逻辑与两条用例的输入；真正的 `cargo test -p singlefs-checker-tier --lib` 没跑。

**第 2229 行**（GPU 新格改成 `&& false`）：见 K3.5，推的，没跑。

**第 2227、2228 行**（E171 第 242、269 行改指崩溃注入内联用例）。问的是「今天的实现上基线绿、变异下红」推不推得出。

*变异下红（推得出，条件见末）*：

- 第 2227 行把发布末尾的「根槽 FUA → 系统配置槽轮换 → 屏障」换成「轮换 → 根槽 FUA → 屏障」（`persist_the_root_then_rotate_the_system_configuration`，`crates/singlefs-core/src/transaction.rs:1286`；条款是 D16 已定项 7 的「一次发布的持久顺序恒为 COW 单元 / 节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）→ 系统配置槽 → 屏障」，`.claude/kb/decisions/16-发布语义.md:182`）。
- 切段规则：「FUA 写只放行它自己那块盘：它与那块盘上前面没被屏障隔开的写一起被放行，别的盘上的写不因它持久。」（`crates/singlefs-harness/src/segments.rs:103`）。基线顺序下根槽 FUA 自己一段、轮换另一段，切段用例钉的字面就是「"[unit_write×2,barrier]|[root_record_fua]|[system_configuration_slot×2,barrier]"」（`crates/singlefs-harness/src/segments.rs:352`），所以崩溃态里「轮换落了、根没落」摆不出来。换序之后，两块盘的轮换写与根槽 FUA 落在同一段（FUA 只放行它自己那块盘，另一块盘的轮换写还没放行，段到末尾的屏障才关），三个写、不超过内联用例给的 `segment_writes: 4`，真子集全枚举，「两个轮换落了、根没落」是其中一个子集。该态上记录（上一段屏障之后已持久）计数器比所选那一版（停在上一条根）的末条记录新，见证值 > 它，判据 N-配置 为真，实现拒可写，没有读故障，新判官判「拒得不合法」。E171 自己对这一行的读法同向：「写的内容逐条相同、次序变了」（`.claude/kb/experiments/171-三oracle合谋测试.md:168`），三条 oracle 放行是因为这份盘面上没人判「拒」本身。
- 第 2228 行把取号写的见证值从 journal tail 换成槽世代号（`highest_journal_tail_of`，「.map(|system_configuration| system_configuration.quantities.journal_tail)」，`crates/singlefs-core/src/transaction.rs:741`；该值进取号写：「highest_journal_tail_of(&self_verified_system_configurations_of_every_device(pool)?);」，`crates/singlefs-core/src/transaction.rs:931`）。世代号随每次系统配置槽的写加一、计数器随记录加（推的），世代号比计数器大。E171 在这一行量到的数同向：「流二另有第 92、93 条 op（可写挂载取号那一写）0x04 → 0x06」（`.claude/kb/experiments/171-三oracle合谋测试.md:169`）：同一类写上 tail 4 被换成世代号 6。内联用例的历史（起点第一个文件 + 一次关掉重挂）覆盖重挂的取号写，取号写落了、暖机的根没落的几个态上见证值 6 对着停在旧根的所选版本、判据为真，实现拒，新判官判不合法。
- 条件：这两条红的推理都依赖「实现在这些态上确实拒了可写」；拒的判据与新判官复算的是同一个判据（D23 已定项 14 的 N-配置，实现在 `mount.rs` 的 `newer_publish_witness`，新判官在 `mount_transparency.rs` 里自己写了一份），两份代码我没逐行比，也没跑。

*基线绿（只能推到「没有理由红」，推不到「绿」）*：基线顺序下摆不出「轮换落了、根没落」（上面的切段）；取号写带的是当前盘上的最大 tail，等于所选那一版末条记录的计数器，判据「c_见证 大于它」为假；崩溃态都是整写子集，没有撕裂的槽，撕裂那一支（甲判真）走不到。内联用例还断言 `tally.writable_mounts_after_the_crash == tally.crash_points`、`crash_points_after_a_step_reading_under_injected_faults == 0`、`new_findings` 为空——最后一条是整条流水线的全部判官（模型、O1、O2……）在这些态上都不报新发现，这一条我推不出，只有 E171 在别的流上量过的「三 oracle 放行」间接支持。所以：红那一边推得出，绿那一边是「按切段与条款没有理由红」，归提交时跑。

### K4 判定

15 行都改在各自点名的那一处（锚点唯一、替换文与行名一致）。10 行 harness 档跑过：基线绿、变异下点名的测试红。第 2225、2226 行两条纯逻辑内联用例在独立小程序里红对了；第 2227–2229 行推的、没跑。

什么会推翻：① 第 2227 行变异下 `cargo test -p singlefs-checker-tier --lib -- crash_injection::tests::no_crash_state_without_read_faults…` 仍绿（说明换序没摆出「轮换落了、根没落」的态，或实现在这些态上没有拒）；② 同一用例在基线上红（说明还有别的拒绝理由被新判官判成不合法，或模型/O1/O2 在这段历史上有新发现）；③ 第 2229 行变异下 GPU 用例仍绿。

## 四、判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| K1 新 F、推的串、上限 | 一致 | 新 F = 现行 txg，推到每块盘都有一条，不判上限；余 0、余 1 两种余数有用例钉，余 2 只是推 |
| K1 `observed_unmount` 的根 | 一致 | 交每一条空发布的根，一条不落；没交的（回收、系统配置写、记号、拒绝一侧）模型本来不比 |
| K1 共用循环逐字不变 | 一致 | 反推合入前文件，循环段 28 行逐行相同，只多一行注释（合入前文件是用 diff 反推的） |
| K1 卸载之后会话关掉 | 规则没说 | D16、D22 都没写，依据是实现注释与射程「卸载之后 F = 现行那一版的 txg」，相容 |
| K1 记号对拍 | 规则没说（说了、没比） | D22 说了卸载记号，模型与 `ObservedRoot` 不比 flags，由 checker 档别的用例钉，本轮没跑 |
| K1 树表 0 条「答不了」 | 规则没说 | 不是唯一读法（零单元带 F 的空发布条款字面也读得通、core 造得出），kb 里没有实现那一读法的条款；模型中立 |
| K1 接线 | 规则没说 | `answer_unmount`、`observed_unmount` 只被新用例调，随机历史、E171 执行器、崩溃注入都没接，拒绝一侧没有观测函数 |
| K3 同一条最新根 | 一致（读码） | 候选集（自证过的）、次序（区域外槽内）、择根键、平局取先读到的、空环，两边同 |
| K3 `highest_seen` 与不等式 | 一致（读码） | 新格与取 max 格共用同一个 `highest_seen`，不等式方向同；`highest_seen` 两边是否算得相等没独立核 |
| K3 摘要与旧判定 | 一致 | 内核原文进两份摘要、判定按（块键，判法版本）存，旧摘要下的 26 亿态零红要重判才能再引 |
| K4 15 行改的点 | 一致 | 锚点唯一、替换文与行名对得上 |
| K4 10 行 harness 档 | 一致（跑过） | 基线绿，变异下点名的测试红；第 2216 行与第 706 行同改法，新加的是模型对拍这条路 |
| K4 第 2225、2226 行 | 一致（独立小程序跑过纯函数） | 各只红自己点名的那条内联用例；`cargo test` 没跑 |
| K4 第 2227、2228 行 | 推的 | 变异下红推得出（切段、E171 的 4 → 6），基线绿只推到「没有理由红」 |
| K4 第 2229 行 | 推的 | GPU 档没跑；CPU 前提在测试里先断言 |

## 五、没做什么

- checker 档（`singlefs-checker-tier` 的测试、GPU、崩溃放量）一条没跑，第 2225–2229 行与 K3 的判别力都是推的。
- K2（新判官误红、读故障认定漏步、撕裂那一支）不归正推腿；第 2222–2224、2225–2228 行我只判了「变异改的是不是点名处、点名的测试红不红」，没替攻方判新判官在合法崩溃态上会不会误红。
- 没跑 GPU 判器与 CPU 判器的逐状态对拍，所以 K3 的「同判」是源码逐句比对的结论。
- 没独立取到合入前的 `model.rs`，K1 的「逐字不变」靠主 agent 给的 diff 反推。
- 没采纳、没出判决；改法建议一条没提。

## 六、要主 agent 跑的命令

（提交时归主 agent，checker 档）：

```
cd /home/fy5090/code/singlefs
SINGLEFS_HEAVY_TESTS=commit bash research/scripts/capped.sh 8 bash research/scripts/run-with-memory-cap.sh 16G cargo test -p singlefs-checker-tier --lib -- crash_injection::tests
SINGLEFS_HEAVY_TESTS=commit bash research/scripts/capped.sh 8 bash research/scripts/run-with-memory-cap.sh 16G cargo test -p singlefs-checker-tier --features gpu --test gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker
```

第一条看 `no_crash_state_without_read_faults_after_a_remount…` 与前两条内联用例基线绿；第二条在有空闲独显的机器上看是绿而不是 `NOT_RUN_NO_GPU_ADAPTER`。变异五行（第 2225–2229 行）用 `crates/mutations.tsv` 的门禁整表跑。

## 七、草稿与副本（交回时报，为什么不删）

- 已删：`/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/ws/`（16M，冻结副本拷出的工作区）、`target/`（1.1G，编译目录）、`pre/`（624K，合入前文件的反推）、`sa/`（13M，独立小程序的产物）。要复核时：工作区用冻结副本 `crates/` 加仓根 `Cargo.toml`、`Cargo.lock` 重拷，脚本在模型目录里。
- 留着：`/tmp/claude-1000/m3-formal-code-b16-r1-sonnet/` 下的 `baseline1.log`、`build1.log`、`mut_2215_2217.log`、`mut_2216.log`、`mut_2216_2219.log`、`mut_2220_2222.log`、`mut_2223_2224.log`（共 32K，跑出来的原始输出，主 agent 复核 K4 要对）。
