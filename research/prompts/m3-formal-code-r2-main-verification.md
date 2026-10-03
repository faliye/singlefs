# 形式化验证第 11 批合入的代码：代码轮第二轮判决（2026-10-02）

<!-- doc-lint:not-numbers M2 J1 K1 K2 K3 K4 L1 L2 L3 L4 L5 H1 U1 U5 V1 W1 D3 D6 D7 D9 R1 R2 -->

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 背景材料 / 改动全文 | `research/prompts/_m3-formal-code-r2-body.md`、`_m3-formal-code-r2-background.md`、`_m3-formal-code-r2-diff.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-r2-snapshot/crates-sha256.txt`（冻结副本 `/tmp/claude-1000/m3-formal-code-r2/tree/`） | 两腿都在冻结副本上 `sha256sum -c --quiet` 无输出；仓里工作树之后被第 12 批（12-2）合入改过 `crash_injection.rs`、RefFS `lib.rs`，腿的引文写冻结副本路径 |
| 云端辩方（Sonnet） | `research/prompts/m3-formal-code-r2-sonnet-output.md`（128 行） | 只读码与 `crates-mutation-rows.py static-check`，没编译 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-r2-opus-output.md`（262 行，sha256 ca05644f…），模型 `…-opus-model/`（47 个文件） | 主 agent 在草稿目录照报告「复跑」一节跑 `tools/rerun.sh <冻结副本> <草稿> k3 k4`：K3 三份日志（`k3slot`、`k3slot0`、`k3mkfs`）去掉耗时与包装行后与 `outputs/` 同名日志 sha256 逐份相同（65f2f5e5…、c3e49f36…、7fccd812…）；K4 见第二节 K4-a 一行。K2 的 147 806 + 234 773 态（约 40 分钟）没复跑，只当攻方的数引 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本> --background <背景材料>`：「✓ 核了 83 处引文，对上 77 处，没判 6 处」；没判的 6 处是模型目录与 `research/results/` 的产物行，主 agent 逐行现查：`k2-mnt-first.log:5`「ALONE judged count=20496」、`k2one-fms12-0-b2.log:57`「SECOND[warm_up] seg=5 field_not_judged=二次崩溃崩在抬 F 开始之后的 F count=4095」、`k3mkfs.log:1`「name=k3_mkfs mkfs_operations=25 zero_fills=10 barriers=4 …」三行逐字在；`research/results/e170-o3-spec-executor-2026-10-02-s1-export.out:17` 与 `…-s3-bad-states-export.out:9` 两行 `local=23 crates=23` 逐字在（旧产物，留档） |

## 二、逐格判决

| 格 | 辩方 | 攻方 | 判决 |
|---|---|---|---|
| K1 RefFS 包 | 没判 | 没攻，读出三处与 H1 不同（只认 fsync 抬 lo、m 只数已确认发布、恢复读出不进 m） | **已被第 12 批关掉**：12-2 照 H1 改了包（调度记录第 12 批「12-2 交回」：H1-a 到 H1-f 逐条对上用例与变异），三处都在其中；冻结副本是 12-2 之前的包。下一轮代码轮攻 12-2 那一版 |
| K2-a 挂载透明在合法历史上误红 | 没判 | 没打中：11 个窗口第一次崩溃层 0 全量 147 806 态 0 红 | **站住**（一次抽样，正文第五节跑前判据「两腿都没打中 ⇒ 站住」） |
| K2-b 二次崩溃三个相位 | 没判 | 没打中：二次崩溃 234 773 态 0 红 | **站住**（同上） |
| K2-c 挂载推了抬 F 时暖机那一次发布的 F | 没判 | 半中：暖机与挂载推的抬 F 同一相位（冻结副本 `crash_injection.rs:270`），抬 F 第一个写之前的 4 107 态 F 也整段不判（`k2one-fms12-0-b2.log:57` 现查在） | **打中，漏判不误红，收严**：照改法 K2-1 把「暖机」与「挂载推的抬 F」拆成两段，抬 F 的第一个写起才算 F 不判；暖机那几态按 U5 前半句判 F 不变。被攻过零轮，下一批 |
| K2-d 拒绝合法的复算与 D23 已定项 14 的单子 | 没判 | 没打中：五步逐条对上，165 次拒全是 N-配置、复算全判合法；另报单子上第三种拒（取号那一刻有一槽读不出）实现报成同一个拒因 | **站住**；主 agent 现查冻结副本 `crates/singlefs-core/src/mount.rs:4043`「ExpectedInstanceAcquisitionFailed::DeviceWithAnUnreadOrUnverifiedSystemConfigurationSlotWhenReadingTheWitness {」映到 `refusal_because_a_system_configuration_slot_turned_unreadable_after_the_read_stage`，与第一种同一个拒因：读错是瞬时的时，镜像上复算两槽都读得出，判法会把这一种合法拒判成不合法。不带故障的历史走不到；挂载透明接进带读故障的档之前要分开，记欠账 |
| K2-e 卸载记号 | 没判 | 覆盖缺口：崩溃注入历史里没有卸载一步；原型 `um` 窗口补跑 24 618 态 0 红 | **站住**；缺口已在 r5 判决第五节第 3 条记着，不另记 |
| K3-a checker 逐档与实现逐档的结局 | D5 三份五项同构（读码）；D6 S 越界：逐档里实现整池拒、checker 与 GPU 记不可择继续试，从 2 个偏移放大到 2033 个 | **打中**：13 种手放槽 5 种结局不同（主 agent 复跑逐字同）。第 1 种「在飞上限 + 1」只在逐档路上分叉：`inflight_plus1_spacing_8192_at_12288` 与 `inflight_plus1_spacing_equals_offset_gen0` 两行「recovery=refused(SystemConfigurationValueRefused …) checker_chose=[dev0:spacing8192,gen1 dev1:spacing8192,gen1] checker_red=[]」，普通读法对照两边都判坏 | **第 1 种打中，复跑坐实，改 checker**：D22 已定项 16 定案第 333 行「逐档读到自证过而池级字段越界的槽整池拒」不要求槽距等于偏移，实现照字面（先判值、`?` 整池拒，冻结副本 `recovery.rs:921`、`:991`），checker 先核偏移、比世代号，被丢掉的槽 I-8.1 不判。改法 K3-1：checker 与 GPU 判器逐档读到 `Usable` 先按实现那张「读者不收之值」表判（含 I-8.1 三个字段），带不收之值的当场交回、不再核偏移；被攻过零轮，下一批。**第 2、3 种（S 越界、R 越界、incompat 不认得加格式版本不对）不分辨逐档**：普通读法上同样分叉，病根是两边对这几个字节的分类与先后不同（实现 S 越界整池拒、R 那一字节不读，checker R、S 越界记不可择；incompat 与格式版本谁先判两边反着），条款「池级字段越界」没逐个点名哪些字段、也没写 incompat 与不收之值同在时谁先。合法历史走不到（W1 之后 [4096, 1 MiB) 只剩本池槽 1，推的）。不挡这一轮：记新欠账，下一轮开一轮三方定分类与先后 |
| K3-b mkfs 写零之后的钉值 | D8 屏障位置对（读码），钉值没跑 | 打中漏列：E170 两份 exact 产物钉 `local=23 crates=23`，crates 今天 25（量过） | **坐实，已被 E170 第 1 次重跑关掉**：`replay.sh` E170 那几行今天指 `e170-r1-o3-spec-executor-2026-10-03-s2-export.out` 与 `e170-o3-spec-executor-2026-10-02-s3-bad-states-export-2026-10-03.out`，主 agent 现查两份都是「local=25 crates=25 holds=true」；旧产物留档。层 0 与崩溃放量钉值不随 mkfs 变（攻方读码，`draw_crash_points` 按 `after_make_filesystem` 滤、放量按 `mkfs_operation_count` 起算），提交时 checker 档快档判 |
| K3-c GPU 判器 | D5 第五列同构 | 没打中（读码）：逐档函数与 CPU 同序同判；GPU 只读表里落点，合法流上同值（推的） | **站住**；K3-1 在 GPU 那一份同改 |
| K4-a ② 借槽距与 ③ 逐档分不分得开 | D7：恢复一臂已分开（`core_review_tree_table_duplicates_and_slot_one_search.rs:592` 那条用例），checker 一臂分不开（第 979 行只管「照 4096」），GPU 一臂没有变异行 | 量过：core 删 ② 被那一条杀（527 过 1 红），checker 删 ② 在 harness 档 108 个二进制上全绿 | **checker 一臂打中，core 一臂 11-11 的推测不成立**；复跑数见第四节。改法 K4-1：那条借槽距用例在同一份镜像上加问 `chosen_system_configurations`（盘 0 应择 4096 处世代号 1），`crates/mutations.tsv` 补「checker 不借、落到逐档」一行点名它；GPU 一臂要「槽 0 无效加旧槽」的坏盘输入才分得开，今天层 0 流没有这种输入，记进 K3-2 那笔新欠账一起开 |
| K4-b E165 r3 装置 | D9：冻结副本里第 1812 行变异锚点命中 2 次（r3 第二段新加的 `system_configuration_for_the_root_choice` 与旧函数同两行） | 没攻 | **坐实，已改**：仓里工作树那一行原文已带函数头，主 agent 现跑 `crates-mutation-rows.py static-check .`，ANCHOR 红单里只有 E168 的 1829、1830、1839–1842、1894、1895 八行（别的会话），没有 1812 |
| 辩方 D3 checker 择根平局取后一个 | `crates/singlefs-checker/src/walk.rs:6715`「.max_by_key(\|(_, (_, _, root))\| (root.checkpoint_txg, root.instance))」键相同交回最后一个，注释第 6711 行自称与 `choose_root` 同一个读法；与代码轮第一轮 K1-d 同一形，第一轮只改了 `newest_root_among` 一臂 | — | **打中同形别臂**（主 agent 现查仓里第 6715 行仍是 `max_by_key`）：改成键相同取读根环次序里最前（与 `choose_root` 同），加一条同键异内容的用例与一行变异；D22 已定项 24 补「键完全相同取前一个」那一句（代码轮第一轮第三节第 5 条，待写回）写回时点名 checker 同一读法。被攻过零轮，下一批 |
| 辩方 D4 合判推论不加去掉 J1 那一行 | 站得住：J1 由 M2 推得出，J1 的出口另有必红行（`verus-mutations.tsv` 第 111、141、160–162 行） | — | **站住**；推论体第 3072 行注释「≤ 落点（J1）」读起来像在用 J1 前提，下一批顺手改成「由 M2」 |
| 辩方 D5、D8 | 三份逐档五项同构；W1 写零在系统配置两槽之前、隔着原有屏障 | — | **站住** |

被点名的 `crates/*/src/*.rs`：`crates/singlefs-reference-filesystem/src/lib.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker/src/image.rs`、`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/make_filesystem.rs`、`crates/singlefs-core/src/mount.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-replay-prefix/src/lib.rs`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/singlefs-checker-tier/src/bin/e170_o3_spec_executor_export.rs`。

## 三、打中的改法（零轮，第 13 批实现，改完与 12-2 一起开代码轮第三轮）

1. K3-1：`crates/singlefs-checker/src/image.rs` 逐档与 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 逐档，读到可用槽先按实现那张不收之值表判（含 I-8.1 三个字段），带不收之值当场交回、不再核偏移与比世代号；用例照攻方 `inflight_plus1_spacing_8192_at_12288`、`inflight_plus1_spacing_equals_offset_gen0` 两种槽，在同一份镜像上恢复整池拒、checker 报带不收之值；一行变异证红。
2. 辩方 D3：`crates/singlefs-checker/src/walk.rs` 择根键相同取读根环次序里最前；同键异内容用例与一行变异。
3. K4-1：借槽距用例加问 checker；`crates/mutations.tsv` 补「checker 不借、落到逐档」一行证红。
4. K2-1：`crates/singlefs-checker-tier/src/crash_injection.rs` 与 `crates/singlefs-harness/src/mount_transparency.rs` 的二次崩溃相位把暖机与挂载推的抬 F 拆开，抬 F 第一个写之前的态 F 判不变；用例与一行变异。
5. 辩方 D4 注释一句（`crates/singlefs-replay-prefix/src/lib.rs` 第 3072 行一带）。
6. 新欠账两笔（随下一份写回登进 `.claude/kb/checks-owed.md`）：系统配置槽 R、S 越界与 incompat 加不收之值的分类与先后，实现与 checker 两读者不同、条款没逐个点名（K3-a 第 2、3 种与辩方 D6，含 GPU 一臂 ② 的坏盘输入）；挂载拒绝合法的复算把「取号那一刻一槽读不出」与 N-配置报成同一个拒因，挂载透明接进带读故障的档之前要分开（K2-d）。

## 四、复跑核对

K4（`tools/rerun.sh <冻结副本> <草稿> k4`，主 agent 在草稿目录复跑，两批共 108 个 harness 档测试二进制）：「k4-core-k4-targets binaries=53 passed=527 failed=1 ignored=62」「k4-core-k4-rest binaries=55 passed=207 failed=0 ignored=1」「k4-checker-k4-targets binaries=53 passed=528 failed=0 ignored=62」「k4-checker-k4-rest binaries=55 passed=207 failed=0 ignored=1」；core 一侧红的那一条是 `a_device_whose_slot_zero_fails_borrows_the_spacing_another_slot_zero_records_instead_of_searching`（`core_review_tree_table_duplicates_and_slot_one_search.rs:605` 断言），与攻方报告同。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 16 | 不影响 | 2026-10-02 不受影响：实现照条款字面，改的是 checker；R、S 越界的分类记新欠账 |
| D22（单元原子性怎么合成） 已定项 24 | 不影响 | 2026-10-02 不受影响：平局句等写回，写回时点名 checker 同一读法 |
| D23（journal 的角色与格式） 已定项 14 | 不影响 | 2026-10-02 不受影响：五步与判法对上；第三种拒的拒因记欠账 |
