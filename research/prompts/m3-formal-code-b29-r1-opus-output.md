# m3-formal-code-b29-r1 攻方腿（Opus）报告（2026-10-05）

被判对象：冻结副本 `/tmp/claude-1000/m3-formal-code-b29-r1/tree/`（下文 `crates/…` 路径与行号都指这棵树）。立场：找反例。量过的数全部出自攻方副本 `/tmp/claude-1000/m3-formal-code-b29-r1-opus/tree/`（冻结副本 rsync 过来，只加了三份原型测试；变体只在副本上打、跑完还原），算副本上的数，不算入库装置的数。

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-formal-code-b29-r1-opus-model/`。复跑（主 agent 在自己的草稿目录，每一步单独起、各在前台时限以内）：

```
M=research/prompts/m3-formal-code-b29-r1-opus-model
D=<主 agent 的草稿目录>
for step in prep tombstone baseline kx1 kx2 extend retain controls lib broad-kx1; do bash $M/tools/rerun.sh $D $step; done
```

单步挂钟（本机，副本上）：prep 约 10 秒；tombstone 在新草稿目录头一次编译约 70 秒（`rerun.sh` 在 `/tmp/claude-1000/m3-formal-code-b29-r1-opus/rerun-check/` 里试跑过这两步，`real 1m8.517s`）；之后各步 30–115 秒，最长的 broad-kx1 约 115 秒。没有一步超过前台时限。判结果看每步打的 `name=variant` 行与缩进的 `test … ok / FAILED` 行，与 `out/variants2-summary.txt` 逐行对。

```
d91d13bcd96ec4c1114ec741ecf3ee5e39eed636f9640d00427d3628c9ff76fb  tools/apply_variant.py
d162f5b1c918c422ea496cb8d387999ba940c5eabcaa63c30c19b09aef421a62  tools/run_one_variant_lib.sh
6d50365ff5611fd7eb91bf78b8786f40d1965fb3b1f76763bd4473f9bcba16fa  tools/run_one_variant.sh
3528433b72f88b27b4b5ea71dc0fa552905e2f59bf29d1e96b965c692db5ed85  tests/b29_opus_fourth_leaf_on_disk.rs
a0a55e3435d469ae245547fc956507940dec2d5f7e1a1205839c7fa158b5370a  tests/b29_opus_leaf_container_sampling_points.rs
da52184cfc556816a31bc5d808a13ae1cc4c06a84fa9aa7212f2a786176ccdf0  tests/b29_opus_tombstone_container_pointer_head.rs
2d79f37368ba964fcc8a3c6385403924734ccb2513ab96434fd2ba56e62811a7  out/variants2-summary.txt
```

`tools/rerun.sh` 的 sha256 在报告末尾「模型目录收尾」一节（它在写报告之前改过一处默认理由，末尾那份是最终值）。

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| L1-a `judge_the_confirmed_loss_shapes` 四个调用点 | 打中（坐实已知差别并补形） | 唯一调它的内联用例在根写前置判那一支就退了，四个调用点连同其后六个出口没有一条用例走到；变异表里点名它的行 0 条 | 推的（装置内联用例是 checker 档，没跑） |
| L1-b 27-2 的 O1 跨叶容器用例换取样点 | 打中 | 四个坏变体（胶水封顶第 3 片、模型封顶第 3 片、回退只取回第 2 片、回退只丢第 3 片）在 27-2 两条用例上全绿；换到第 4 片 / 目标带第 3 片的回退，四个全红 | 量过（副本） |
| L1-c E171 逐件计数漏算 / 重算 | 没打中（三个形状，今天到不了） | 逐件重做路上几步胶水不在任何一件的 `catch_unwind` 里；各件列不互斥；读回被并进两件 | 推的 |
| L2-a G1 运行配置期望字节 | 打中（判据写法） | T_time 的编码单位条款没写，G1 的 5 取的是 core 的「秒」；D16 定的可设区间下端 200 ms 在「秒」里写不出 | 推的 |
| L2-b I-9.16 ④、I-5.4、journal_tail、G3、G5、C3 / B4 | 没打中 | 只读码，没造出合法历史误红或坏盘放过 | 推的 |
| L3 G4 误红 | 没打中 | 崩溃注入整写子集、轮换写不覆写最大世代那一槽、每段系统配置写前有屏障：合法历史的崩溃态每盘至少一槽自证过；唯一反例在 mkfs 那一段里，两套装置都不摆 | 推的 |
| L3 与 C670 同不同形 | 同形 | 第二截、第三截两条的接法、计数、取舍逐处同构 | 推的 |
| L4-a 墓碑树 / 谱系树容器指针的 extent 偏移 | 打中（CPU、GPU 池级 checker 一起漏） | 偏移改 1：CPU 池级 checker I-2.4 `Holds`、违例 0 条；同一条指针 MAC 改 1 照红；core 恢复与可写挂载都拒；GPU 走读内核同处不判 | CPU 量过（副本）；GPU 推的 |
| L4-b I-9.16 ③ ④、428..457、分配记录树根全零、超额交 CPU | 没打中 | 逐格读码，GPU 与 CPU 判同一组字节、同一组根；12 棵树的表在扫描里置判不了、整态交 CPU | 推的 |
| L4-c 对拍新加六条 / 改一条各罩哪一处 | 补形 | 七条流罩七类指针；GPU 恢复内核在墓碑树 / 谱系树条目上的 extent 判没有一条流罩 | 推的 |
| L5-a cfg 改动在两个特性都开时 | 没打中 | 门禁从测试文件 cfg 里现读特性名、两个都开，改过的用例照样编进来 | 推的（读门禁脚本） |
| L5-b #9 两个钉值出处 | 没判 | 要跑 checker 档才核得了 | 没做 |

## L1　第 27 批

### L1-a　`judge_the_confirmed_loss_shapes` 四个调用点：改坏哪一处、哪条测试红

四个调用点是 `stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed` 在它里面的四次调用，分别收对拍、阳性对照「判态路 panic 单列」、反向取样点、主取样点那四组态：「if let Some(exit_code) = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8080`），同一行原文又在第 8113、8187、8248 行。

谁调 `judge_the_confirmed_loss_shapes`：全文只两处。一处是 `main` 经 `run_confirmed_loss_form`：「ParsedCommand::ConfirmedLossForm(command_line) => run_confirmed_loss_form(command_line),」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:9723`），没有用例起 `main`；另一处是内联用例 U8：「let exit_code = judge_the_confirmed_loss_shapes(&mut lines, &command_line, shapes, 0);」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11837`）。U8 两次都把一形的 `judged_root_index` 设成 `usize::MAX`，走的是函数开头那一支：「if let Some(shape_name) = shape_without_a_root_write(&shapes) {」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:8028`），退 3、不往下走。27-2b 那条 unattributed 用例直调被调函数：「let exit_code = stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:11948`），不经任何调用点。`grep -c 'stop_the_q5_and_write_the_verdict_when_a_panic_is_unattributed' crates/mutations.tsv` 输出 `0`。

| 改坏的形状（每个调用点各一份） | 哪条测试红 |
|---|---|
| 整个 `if let Some(exit_code) = …` 块删掉（有 unattributed 的态照样往下判甲乙、进 S13） | 没有一条（推的） |
| 实参换成 `&[]` 或换成另一组态（对拍那处传阳性对照的、主取样点那处传对拍的） | 没有一条（推的） |
| `failed_checks` 实参漏加 `totals.failed_checks`（反向、主取样点两处） | 没有一条（推的） |
| 调用点挪到 `report_the_confirmed_loss` 之后（先打 Q5 计数行再停） | 没有一条（推的） |

补形（不止四个调用点）：U8 退出的那一支之后，函数里另有六个出口——`equivalence_pairing_failed`、`piece_panic_control_failed`、`too_many_states`、`too_many_states_after_refinement`、第二道 `no_root_write`、末尾判决行——与 S1 的 k 核对、F9 判定，同样没有一条内联用例走到。它们今天唯一的执行者是 E171 Q5 那一趟实验本身（量的，不是测）。四句：①分不分辨臂——不涉及臂，判的是测试罩不罩得住；②装置当时看得到判别它的东西——看得到（四组态都在手里）；③满足的是正文第三节 L1「改坏了哪个测试会红（重点）」那一问的反面：改坏了没有测试红；④改法见文末改法表 A1、A2，两条都推的、零轮。

为什么说「推的」：这几条都是 `crates/singlefs-checker-tier/src/bin/` 下装置的内联用例，`cargo test -p singlefs-checker-tier --bin …` 是 checker 档，这条腿不跑。结论只靠「调用关系」与「U8 走的分支」两件读码事实；推翻它的现象：主 agent 提交时跑 E171 装置内联用例，对上表任一形状的变异有一条红。

### L1-b　27-2 的 O1 跨叶容器用例换一个取样点还红不红

27-2 那两条用例的取样点：建到第 3 片，「const INODE_CREATIONS_INTO_THE_THIRD_LEAF_CONTAINER: u64 = 2 * INODE_LEAF_RECORDS + 4;」（`crates/singlefs-harness/tests/model_compares_inode_creations_into_the_third_inode_leaf_container.rs:19`）；回退用例只回退到两片的一版，「const INODE_CREATIONS_BEFORE_THE_ROLLBACK: u64 = 2 * INODE_LEAF_RECORDS + 2;」（`crates/singlefs-harness/tests/model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers.rs:17`）。

四个变体（`tools/apply_variant.py`，各改一句）：

| 变体 | 改哪一句 | 坏在哪 |
|---|---|---|
| `kx1_cap_at_third` | 「ModelUnitRole::inode_leaf_container_at(u64::from(index.0))」（`crates/singlefs-harness/src/model_comparison.rs:80`）加 `.min(2)` | 胶水把第 4 片及以后都当第 3 片（KX1 同族，往后挪一片） |
| `kx2_cap_at_third` | 「let position_from_the_leftmost = records_before / INODE_LEAF_RECORDS;」（`crates/singlefs-harness/src/model.rs:197`）加 `.min(2)` | 模型落点封顶在第 3 片（KX2 同族） |
| `extend_only_the_second_leaf` | 「role_written_at.extend(」（`crates/singlefs-harness/src/model.rs:1722`）那个过滤只留第 2 片 | 回退只取回目标那一版的第 2 片（drop-extend 同族） |
| `retain_drops_only_the_third_leaf` | 「role_written_at.retain(|role, _| {」（`crates/singlefs-harness/src/model.rs:1719`）那个过滤只丢第 3 片 | 回退只丢第 3 片、更右的留着（drop-retain 同族） |

我的三条换点用例（`tests/b29_opus_leaf_container_sampling_points.rs`）：建到第 4 片（3 × 233 + 4 条）；第 3 片 5 条时回退到第 3 新那条根（目标那一版第 3 片 3 条、现行那一版重写过它）再建 3 个；第 4 片 2 条时回退到第 3 新那条根（目标那一版没有第 4 片）再建 3 个。每步 checker 不跑，同 27-2 那两条。

副本上的输出（`out/variants2-summary.txt` 原样摘行；`T3` = 第 3 片用例，`TR` = 回退用例）：

```
variant=baseline           T3 ok / TR ok / creations_into_the_fourth_leaf_container ok / rollback_to_a_version_whose_third_leaf_container_was_rewritten_since ok / rollback_to_a_version_without_the_fourth_leaf_container ok
variant=kx1_cap_at_third   T3 ok / TR ok / creations_into_the_fourth_leaf_container FAILED / rollback_to_a_version_whose_third_leaf_container_was_rewritten_since ok / rollback_to_a_version_without_the_fourth_leaf_container FAILED
variant=kx2_cap_at_third   T3 ok / TR ok / creations_into_the_fourth_leaf_container FAILED / rollback_to_a_version_whose_third_leaf_container_was_rewritten_since ok / rollback_to_a_version_without_the_fourth_leaf_container FAILED
variant=extend_only_the_second_leaf       TR ok / rollback_to_a_version_whose_third_leaf_container_was_rewritten_since FAILED / rollback_to_a_version_without_the_fourth_leaf_container FAILED / creations_into_the_fourth_leaf_container ok
variant=retain_drops_only_the_third_leaf  TR ok / rollback_to_a_version_without_the_fourth_leaf_container FAILED / 其余两条 ok
variant=kx1_original_cap_at_second  T3 FAILED（对照：27-2 那一行 KX1 变异本身，证装置会红）
variant=drop_retain_original        TR FAILED（对照：drop-retain 那一行）
```

（上面是把同一变体几份日志里的 `test … ok / FAILED` 行并成一行写的；逐行原文在 `out/variants2-summary.txt`，按日志分节，例如 `== kx1_cap_at_third.b29_opus_leaf_container_sampling_points.log` 那一节里 `test creations_into_the_fourth_leaf_container ... FAILED`，同一节里那一步的失败是 `position: Operation(700)` 的 `ModelDisagreement { aspect: "单元的分配代" }`：模型说这一版有第 4 片（`position_from_the_leftmost: 3`），实现交回的角色里没有——胶水把它记成了第 3 片。）

别的 harness 档测试罩不罩：`kx2_cap_at_third` 在 `--lib` 上被 135 边界那条模型单测抓到（`test model::tests::the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond ... FAILED`，`rc=101 … 81 passed; 1 failed`）。`kx1_cap_at_third` 在 `--lib`（`82 passed; 0 failed`）、`unlink_in_histories_judged_by_the_model`（`8 passed`）、`random_histories`（`23 passed; 0 failed; 19 ignored`）上全绿：胶水封顶第 3 片在今天整个 harness 档里只有我那两条第 4 片用例抓得到（random_histories 里 19 条 ignored 没跑；随机历史一步只建一个 inode、24 步到不了第 4 片，推的）。`extend_only_the_second_leaf`、`retain_drops_only_the_third_leaf` 在 `--lib` 上全绿（`82 passed`）。

四句：①分不分辨臂——这一格没有臂，判的是用例；四个变体都是坏的 O1（模型或胶水），实现不动；②判别它的东西在被判的用例里看得到——只要历史建到第 4 片、或回退目标带第 3 片，O1 逐片比分配代那一步就看得到（换点用例在 baseline 上全绿、在变体上红，证明了这一点）；③满足的是正文 L1「各罩不罩得住它要证的行为（换一个取样点还红不红）」那一分句：不红；④改法见改法表 B1、B2（量过一半：换点用例副本上量过，变异行没写）。

副产品（不是打中，只记一笔）：量的过程中我自己的装置出过一次假红——`cp -a` 还原源码带回旧 mtime，cargo 把上一个变体编出的库当成新的复用，于是「baseline」跑的其实是 `kx1_cap_at_third`。`tools/run_one_variant.sh` 已改成每次跑之前 `touch` 全部源码，上表全部出自改过之后的 `variants2/`。主 agent 复跑若改装置、或同类原型用 `cp -a` 还原，同一个坑在。

### L1-c　E171 逐件计数：有没有一种 panic 或一种态既不进逐件计数、也不进 unattributed，或被算两次

没造出今天到得了的漏算 / 重算。试过的三个形状，都只在读码层面成立：

1. **逐件重做那一路的胶水不在任何一件的 `catch_unwind` 里。** 正常那一路 panic 了的态，块外照序逐个重做：「observations.push(observation_of_a_panicking_state(」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:4391`），这一句外面没有 `catch_unwind`。重做函数里十件各包一层，但落地串、镜像摘要、`RecoverySummary`、`ImplementationVerdict::of_the_pieces` 这几步不在任何一件里；镜像摘要里有一处 `expect`：「.expect("崩溃镜像在盘内、按扇区对齐的区间读得出");」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:2377`）。这几步一旦 panic：Q5 那一路（`judge_the_confirmed_loss_shapes` 外面也没有 `catch_unwind`）整个进程退 101、一行判决都不打；逐行 / 基线那一路被外层接住记成 `pipeline_panicked`、整条流不导出，`piece_panic_states=` 与 `unattributed_panic_states=` 都不计它。今天这几步只读内存里的镜像、到不了 panic（推的）。登记修订 5 第 5.1 节第 2 条要求「回调链上还有这张单子之外的一步，加一件、写明，不并进别的件」，这几步是逐件那一路自己多出来的，不在回调链上，条款没管到。
2. **各件列不互斥。** 一态逐件重做时几件都 panic，那一态在几件下各计一次：「states[piece.position()] += 1;」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3667`）。看 journal 那遍恢复 panic 的态常常连带不看 journal 那遍、层 0 oracle 一起 panic。这是登记「各件 panic 的态数」的本意，不是错；但判决行里没有「逐件那一路走过几态」的总数，`piece_panic_states=` 各项加起来不是 panic 的态数。要核「每个正常那一路 panic 的态要么有件、要么进 unattributed」得另数（见改法表 A3）。
3. **读回被并进两件。** 正常那一路不论适用不适用都先造读回：「let read_back = observed_read_back_after_a_crash(&judgement.consulted);」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3952`）；逐件那一路把造读回包进 `crash_judge` 与 `reference_filesystem` 两件里各造一次。读回 panic 时两件都记 panic（重算一次），模型关掉时两件都不跑、那一态进 unattributed。造读回的函数今天是纯取字段（`crates/singlefs-harness/src/model_comparison.rs` 的 `observed_read_back_after_a_crash`），到不了 panic（推的）。

`unattributed_panic_states=` 的定义本身照登记：「observation.judging_path_panic.is_some() && observation.piece_panics.is_empty()」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:3681`）。k = 6 对拍与 no_root_write 先于对拍各有 U9、U8 钉着，读码对得上修订 5 第 5.2、5.3 节，没攻出别的。

## L2　第 28 批 CPU

### L2-a　G1：运行配置期望字节「取自条款」只取了一半

G1 写死的期望：「const BUILT_IN_TIME_THRESHOLD_SECONDS_OF_THE_CLAUSES: u32 = 5;」（`crates/singlefs-harness/src/mount_transparency.rs:1357`）。条款给的是值不是编码：「| 可调值 | T_time | 4 | 5 秒 | D16（发布语义） 已定项 5 |」（`.claude/kb/layout/01-first-txn.md:149`）——4 字节放 5 秒，没写这 4 字节以什么为单位。G1 用的「秒」与被测 core 同一个单位：「pub fn time_threshold_seconds(self) -> u32 {」（`crates/singlefs-core/src/system_configuration.rs:290`）。所以「期望字节取自条款而不取被测 core」那句话，在值上成立、在单位上不成立：core 若把这个字段改成毫秒（写 5000），G1 判红；而条款判不了谁对。

再一层：D16 已定项 5 那一段（`.claude/kb/decisions/16-发布语义.md:140`，整行抄在附录 Ⅱ）定 T_time 用户可设、可行区间下端 200 ms、越界只警告、挂载时把传的值回写进盘上字段——以「秒」为单位的 u32 写不出 200 ms。条款要求的值域与 G1（连同 core）认的编码对不上。四句：①不分辨臂（这一格判的是 G1 判据本身）；②被判的系统（G1 那一判）看得到盘上 4 字节，看不到单位——单位不在条款里，属「判别子观测不到」那一形：判据要分的「5 秒」与「5 毫秒 × 1000」两种盘面，G1 在做决定那一刻手里只有一个它自己假定单位的常量；③满足的是正文 L2「G1 字节取自条款对不对、全不全」里「全不全」那一分句：不全；④改法 C1（推的）：先在条款字段表里写明 T_time 的编码单位，G1 再照它写死，改完是新一轮。

### L2-b　其余几项（没打中，只记试过什么）

- I-9.16 ④：条款与 CPU 同取十一种（`TREE_KINDS_OF_THE_TREE_TABLE_IN_THE_ISSUING_ORDER`）。树表是写时复制的整单元，崩溃态的树表要么是旧的整张、要么是新的整张；合法历史里非空树表少一种的路我没找到（第一个文件那一版写满十一条，之后各版照抄或整张重写）。没跑随机历史去扫，推的。
- I-5.4「成立要至少一棵树走到叶」：只读码。分配记录树同样整单元写时复制，合法崩溃态上走不到叶只会出在整棵读不出的盘面上，那时报不适用本来就对；坏盘上放过的形状没造出。
- journal_tail 两半、G3、G5、C3 / B4 两形：没攻到可以写下来的形状。G5「注入态不判二次崩溃透明」放过只在注入态上露的坏实现——今天会在读故障下往盘上发写的只有 `WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros` 那一步，取号写落在被藏的最新那一槽、另一槽不动，我没造出一个只在这一步下露、而第一截与第二截都看不见的坏实现。

## L3　第 29 批 G4

### L3-a　合法历史让某块盘两槽都自证不过、因而误红：没打中

G4 那一判在崩溃镜像上复算：「if system_configuration_unreadable_after_the_read_stage_on_the_crash_image(crash_image)」（`crates/singlefs-harness/src/mount_transparency.rs:2693`），为真就记不合法。要误红，得有一段合法历史、不注入读故障，留下「某块盘两槽一份本池自证过的都没有」（或择不到系统配置、或见证判不出）的崩溃态，且实现在那份盘面上以「读阶段判完之后系统配置又读不出」拒。我扫过的形状与为什么都到不了：

1. **崩溃注入不撕裂。** 第二截、第三截的崩溃态是段内整写的真子集：「枚举域与层 0（`crate::crash`）同一个：更早的段整段持久、当前段任意真子集、更晚的段一个都没持久」（`crates/singlefs-checker-tier/src/crash_injection.rs:5`），撕裂并进「没持久」。一个系统配置槽写没落，那一槽就还是旧的自证过的内容。
2. **轮换写不覆写最大世代那一槽。** 每块盘这一写落到「(slot_generation % SYSTEM_CONFIGURATION_SLOTS_PER_DEVICE) * spacing,」（`crates/singlefs-core/src/transaction.rs:542`），世代 = 这块盘自证过的最大世代 + 1，所以永远写另一槽。一槽撕坏（现实里的撕裂）或静默坏了一槽（D22 已定项 8 承诺里的单故障）之后，下一次写正好补那一槽，世代最大那一槽一直在。
3. **同一块盘两次系统配置写之间都有屏障。** 取号、取号回卷、抬 F 先写系统配置三处都在第一个系统配置写之前发池屏障：「if let Err(cause) = pool.perform_commit_step(CommitStep::Barrier) {」（`crates/singlefs-core/src/transaction.rs:697`，同一行又在第 723、1007、1037 行）；发布末尾的轮换在根槽 FUA 之后、返回之前再一道屏障。所以层 0 那种带撕裂的枚举（E171 探针用的）里一段之内一块盘至多一槽在写。
4. **回退、正常卸载再挂、多实例**：回退与抬 F 走第 3 条那几道屏障；卸载再挂是取号 + 暖机 / 写行，同上；多实例只是取号的实例代号不同。都没有一段里同一块盘写两槽的路。
5. **读故障那一步**（`WritableMountWithTheNewestSystemConfigurationSlotOfEachDeviceReadingZeros`）：挂载眼里最新那一槽读回全 0，取号写按第 2 条落在被藏的那一槽上，另一槽不动；这一步发过写之后的崩溃态被认成注入过读故障，G4 不判。
6. **唯一的反例在 mkfs 里**：mkfs 两槽写同一份、同一个世代，「slot_generation: SYSTEM_CONFIGURATION_GENERATION_AT_MKFS,」（`crates/singlefs-core/src/make_filesystem.rs:644`），逐盘两槽「device.write_at(offset, &slot, WriteDurability::Plain)?;」（`crates/singlefs-core/src/make_filesystem.rs:655`）同在根槽 FUA 之后、末尾屏障之前那一段里。崩在这一段、盘 0 两槽都没落而盘 1 落了：盘 1 择得到系统配置、根槽已 FUA，恢复落到第 0 代，可写挂载以「读阶段判完之后系统配置又读不出」拒，没有读故障——G4 那句「没有读故障的历史留不下这份盘面」在这里不成立。但两套装置都不摆 mkfs 里的崩溃：崩溃注入的候选段要整段落在 `(marks.after_make_filesystem..marks.after_the_last_finished_step)` 里（`crates/singlefs-checker-tier/src/crash_injection.rs:3814`），E171 的基镜像取 mkfs 之后：「let base = live.formatted.memory_pool_after_mkfs();」（`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:1987`）。推的；这一形该不该算合法历史（mkfs 没返回，用户手里还没有池）条款没写。

四句（对第 6 条）：①不分辨臂（G4 只有一个判法）；②被判的系统看得到判别它的东西——看不到：盘面本身与「静默损坏两槽」那份手工盘面逐字节可以相同，区分只在「崩在 mkfs 里」这一历史事实，判官手里只有镜像与读故障认定；③满足 G4 文字「没有读故障的历史留不下」的反面；④改法 D1（推的）：G4 的射程句补「mkfs 返回之前的崩溃态不在射程」，与两套装置的实际取样一致。

判别力那一面（不是误红，顺手记）：坏实现 MUT-B1（盘 0 轮换写完新槽之后把另一槽清零、同在一段）在不撕裂的崩溃注入里，第一次轮换摆不出两槽都坏（新槽没落时旧槽还在），第二次轮换才摆得出（上一次已把一槽清零，这次写那一槽、清另一槽，清零落了而新槽没落）。所以 G4 在崩溃注入里要一段历史里至少两次轮换才抓得到 MUT-B1——常见历史都有，推的、没跑（checker 档）。

### L3-b　第二、三截判法与 C670 同不同形

同形。第二截：G4 那一臂与 C670 那一臂在同一个 `match (reason, read_faults)` 里并列，读故障认定为注入时两条都不判，后面「被拒那次挂载在读故障下跑」的 G3 豁免只作用在这一拒因上（与改动前同）。第三截：`reason_and_conditions_not_met_by_a_remount_refusal_with_a_criterion_true_without_read_faults` 对两拒因各记一次判过、各记一次不合法，只差 U6 那几条的都交 None——与 C670 原来那一判逐处同构。复算上的唯一差别：G4 那一复算择不到系统配置按真，C670 那一复算择不到同样按真：「let Ok(system_configuration) = choose_system_configuration(crash_image) else {」（`crates/singlefs-harness/src/mount_transparency.rs:2350`，同一行又在第 2382 行），对称。

手工两槽损坏用例翻成不合法、MUT-B1 最小形状那一条在注入故障口径下合法：前者由 G4 字面推得出（没注入读故障 + 复算为真）；后者由 D13 已定项 1 挂载透明射程「只管不带故障的状态」推得出。两条都只是判官函数的单测，不经崩溃注入。E171 装置补的那一臂只是 `RefusalConditionNotMet` 穷举里多一个词，E171 探针的崩溃态读故障恒交「没注入」，按上面第 1–6 条它在 E171 的层 0 态上同样到不了真。

## L4　GPU：26-5 与 29-2

### L4-a　墓碑树 / 谱系树容器指针的 extent 偏移：CPU、GPU 池级 checker 都不判，core 拒

池级 checker 走墓碑树、谱系树的码 2 根时，对指向码 3 容器的子指针只判 MAC / nonce：「judge_pointer_mac_and_nonce_are_zero(&mut self.judgements, &child, &what);」（`crates/singlefs-checker/src/walk.rs:2064`），接着照样跟进去：「self.note_references_of_a_pointer_followed_by_the_walk(&child, 2, &what);」（`crates/singlefs-checker/src/walk.rs:2110`）。别的码 2 指针都在读节点时判 extent 偏移：「judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);」（`crates/singlefs-checker/src/walk.rs:1271`）；类型段 0 的孩子经那一处，所以只漏「指到容器」这一种。GPU 走读内核照抄同一处：`w_walk_container_tree_root`（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2896` 起）对子指针只调「w_judge_pointer_head(child);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2928`），后面没有 `w_judge_pointer_extent_offset(child)`（inode 根那一处有：「w_judge_pointer_extent_offset(child);」（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2595`））。恢复那一半两边都判：GPU 恢复内核「if (!record_pointer_head_is_the_first_version(n.base, o + CONTAINER_TREE_ENTRY_CHILD_POINTER_OFFSET)) {」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2860`），那个函数就是「return parse_node_pointer(base, o).head_ok && rd16(base, o + 32u) == 0u;」（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:993`）。

副本上量的（`tests/b29_opus_tombstone_container_pointer_head.rs`：三个单元的文件发完 unlink 意图那一版，墓碑树根第 0 条条目里指向墓碑容器的那条子指针，头部偏移 32 的 2 字节改 1，或对照把 MAC 第 0 字节改 1；单元重封、校验和补到根槽，补法照 `common_pointer_head_corruption` 那一套；`out/variants2-summary.txt` 里 `== baseline.b29_opus_tombstone_container_pointer_head.log` 那一节原样）：

```
clean checker I-2.4=Holds
clean checker violated=[]
clean mount_writable ok instance=InstanceGeneration(2)
extent_offset_1 checker I-2.4=Holds
extent_offset_1 checker violated=[]
extent_offset_1 recover outcome=Failed { root: Some((InstanceGeneration(1), CheckpointTxg(5))), failure: UnitMalformed { what: "墓碑树或谱系树内部条目的子指针头部 extent 偏移不是 0" } } effective_root=Some((InstanceGeneration(1), CheckpointTxg(5)))
extent_offset_1 mount_read_only ok=true
extent_offset_1 mount_writable refused=Recovery(UnitMalformed { what: "墓碑树或谱系树内部条目的子指针头部 extent 偏移不是 0" })
mac_byte0_1 checker I-2.4=Violated("墓碑树内部条目的子指针：指针头部 MAC 16 / nonce 12 有非 0 字节，加密关着时它们恒 0")
mac_byte0_1 checker violated=["I-2.4"]
```

也就是：同一条指针，MAC 坏了池级 checker 红 I-2.4，extent 偏移坏了池级 checker 一条违例都没有，而 core 冷启动恢复与可写挂载都按结构损坏拒。这份镜像上 O2（池级 checker）放过了一个 core 拒的坏结构；GPU 走读内核同一处同样不判（推的，GPU 没跑）。不变量清单 I-2.4 那一行说 checker 走读跟随的每条指针都判 extent 偏移（整行在附录 Ⅰ，`.claude/kb/invariants.md:133`），这一种指针上不成立。

四句：①不分辨 CPU / GPU 两臂——两边一起漏，病根在两边共用的那张「哪几处判 extent 偏移」的清单里（GPU 照 CPU 抄）；按 evidence-discipline「打中不分辨臂」处置：不拿它判 GPU 与 CPU 谁对，另立一笔账修；②被判的系统看得到判别它的东西：看得到，子指针已经在手里解开（判 MAC 用的就是它）；③满足的是 I-2.4 那一行「extent 偏移那一半」的分句；④改法 E1（推的）：walk.rs 第 2064 行后补一判、wgsl 第 2928 行后补一判，再加一条对拍坏流；E2：I-2.4 那一行的现状句改成实测的清单。这段走读代码不在本轮 diff 里（本轮 walk.rs 只动了 28-1 那几处），按正文第二节转给写墓碑树 / 谱系树走读的那条线。

谱系树走同一个函数（`walk_container_tree_root` 的两个调用点按种类 10、11 分派），推的同一结论；我只量了墓碑树。

### L4-b　GPU 与 CPU 在 I-9.16 ③ ④、根槽 428..457、分配记录树根全零上：没找到判得不同的盘面

逐格对过（读码，推的）：

- I-9.16 ③ ④：两边都在条目宽读得出种类（≥ 12 字节）时才判，0 条的表不判；③ 都把没登记的种类码算进去；④ 都要十一种各至少一条。CPU 先判 ③ 再判 ④、两条都可能各报一次违例，GPU 第一处违例就返回——违例位相同。
- 「只违反 ③ 的表超额整态交 CPU」：GPU 候选根信息行一张树表最多记 11 棵不同的树，第 12 棵起在 `r_note_tree_table_entry_birth_txg` 里置判不了（`UNDECIDABLE_TOO_MANY_ENTRIES | (28u << 8u)`）、整态交 CPU。只违反 ③ 而条目 ≥ 12 的表要么条目宽 ≥ 200 走进那一处置判不了，要么条目宽 < 200 不进那一处、由走读内核自己判 ③；两种都交得回或判得对。条目 ≤ 11 而违反 ③ 的表必然同时缺一种（违反 ④），GPU 自己判红。
- 根槽 428..457：两边取同一组根（CPU `valid_roots`：magic、整槽校验和、16 字节 fsid、标志位；GPU `checker_collect_roots` → `parse_root`：同四样），都不按候选集滤。
- 带树一版分配记录树根全零（I-5.4）：两边都只看候选根、都要树表读得出且条目非空、都看偏移 342 那 86 字节是否全零。

推翻条件：GPU 对拍跑出这几格任一流上 CPU 与 GPU 违例位不同。

### L4-c　对拍新加六条、改一条，各罩 GPU 哪一处

29-2 七条流各罩一类指针：树表条目根指针、根记录实例表指针、inode 内部条目子指针、上段叶内联数据指针、下段根指针、下段叶数据指针、分配记录树内部条目子指针（diff 里那段测试文件头的说明）。按读码：每条流里被改坏的那条指针只在 GPU 恢复内核与走读内核各一处被读到，所以把那一处的 extent 判删掉，那条流的整份比对就会红（推的，没跑）。没有流罩的：GPU 恢复内核在墓碑树 / 谱系树条目上的那一判（上面 `crash_judge_common.wgsl:2860` 那一句）——删掉它，七条流都不经过墓碑树，一条不红（推的）；连同主 agent 已记的多层树内部子指针、extent 内部子指针、实例表链指针三处，共四处。

## L5　主 agent 的两处

- cfg：`the_gpu_judge_agrees_with_the_cpu_judge_on_sigma_whose_configuration_slot_is_written_more_than_thirty_two_times` 改挂 `all(feature = "gpu", feature = "verdict-store")`。提交时门禁那一格的特性是从测试文件的 cfg 里现读的：「feature_name = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")」（`.claude/gate.d/checker-tier-layer0-replay.sh:621`），一个文件里出现过的特性名全收、并成一串：「feature_list="$(IFS=,; printf '%s' "${features[*]}")"」（`.claude/gate.d/checker-tier-layer0-replay.sh:703`）。这份文件里两个特性名都出现，那一趟两个都开，改过的用例照样编进来、照样跑（推的，没跑门禁）。
- #9 两个钉值：要跑 `crash_enumeration_sharded_across_processes`（checker 档）才核得了，没判。读注释能说的只有：两值取自「第 28 批 + 根环线 10e」那棵树，冻结副本里在那之后又合进了第 29 批与删除线第 18、19 批；池级 checker 判定集合若被后者改过，`GOLDEN_TALLY` 那两行与进度文件内容会跟着变。哪些文件在两棵树之间变了，我手里没有那棵树的清单，没核。

## 改法表（只在我的模型上量过、被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| A1　E171 装置给 `judge_the_confirmed_loss_shapes` 加一个注入口（把 `PiecePanicInjection` 传进主取样点与反向取样点那两次导出），内联用例在反向取样点（无对拍、约 1 秒）注入一条不点名件的规则，断言判决行 `q5_not_run=unattributed_panic` | L1-a 反向、主取样点两处调用点 | 推的 |
| A2　对拍、阳性对照两处调用点各一条用例：k = 2 的对拍流（288 态）上注入，断言在对拍那一处就停 | L1-a 另两处 | 推的（k = 2 的流已有 `equivalence_pairing_stream()` 可用；`judge_the_confirmed_loss_shapes` 今天写死 k = 6，要把 k 交成参数） |
| A3　判决行加一个字段「逐件那一路走过的态数」，与 `unattributed_panic_states` + 有件的态数对账 | L1-c 第 2 条 | 推的 |
| B1　27-2 第 3 片用例改成建到第 4 片（3 × 233 + 4 条） | L1-b 胶水 / 模型封顶第 3 片 | 量过：副本上 `creations_into_the_fourth_leaf_container` 在 baseline ok、`kx1_cap_at_third` 与 `kx2_cap_at_third` 都 FAILED |
| B2　回退用例再加两条：目标那一版带第 3 片且现行那一版重写过它；现行那一版有目标那一版没有的第 4 片 | L1-b 回退两变体 | 量过：`extend_only_the_second_leaf` 两条红、`retain_drops_only_the_third_leaf` 一条红，baseline 三条全绿；四个变体各一行变异没写 |
| C1　条款字段表写明 T_time 的编码单位，G1 照它写死 | L2-a | 推的 |
| D1　G4 射程句补「mkfs 返回之前的崩溃态不在射程」 | L3-a 第 6 条 | 推的 |
| E1　walk.rs 第 2064 行后与 wgsl 第 2928 行后各补 extent 偏移一判，对拍加一条墓碑树容器指针坏流 | L4-a、L4-c | 推的（CPU 那一半没在副本上改着量；坏镜像造法已在 `tests/b29_opus_tombstone_container_pointer_head.rs`） |
| E2　I-2.4 那一行的「走读跟随的每条指针都判」改成实测清单（加 E1 之前少墓碑树 / 谱系树容器指针） | L4-a | 推的 |

B1、B2 的代价：副本 debug 下第 4 片那条约 20 秒、两条回退约 12 秒与 20 秒（`finished in` 合计见 `out/variants2-summary.txt`），与 27-2 两条同一量级。

## 没打中的形状

- L1-c：逐件计数漏算 / 重算（三形，见上）；`judge_runs_continuing_past_panicking_states` 里「panic 序号不在这一段」那条 `assert!` 只在 `judge_layer0_state_range` 判完最后一态之后才 panic 时触发，那一段函数尾只交回状态数，到不了。
- L2：I-9.16 ④ 合法少一种、I-5.4 走不到叶误报不适用、G5 放过的坏实现，都只读码、没造出。
- L3：合法历史两槽都坏——扫的形状：轮换撕裂（层 0 的撕裂与崩溃注入的整写子集两种枚举域）、静默坏一槽之后再崩、取号回卷、抬 F 先写系统配置、回退、卸载再挂、多实例、读故障那一步、mkfs 那一段；取样范围是这些写路径的全部调用点（`crates/singlefs-core/src/` 里 `write_system_configuration_slot(` 的四处调用——轮换那一步、取号回卷、抬 F 先写系统配置、取号——与发布末尾 `RotateSystemConfigurationSlots` 那一处，`grep -rn` 数出来的），不是随机历史；没跑任何崩溃枚举（G4 在 checker 档）。
- L4：③ ④、428..457、分配记录树根全零三格逐格读码对过，没找到判得不同的盘面；GPU 没跑。

## 这条腿自己的限度

- 一次抽样的「没打中」只算这一次（L1-c、L2-b、L3、L4-b）。
- checker 档一条没跑：E171 装置内联用例、崩溃注入、GPU 对拍、`crates/mutations.tsv` 整表都没跑，L1-a、L3、L4 的结论里凡涉及它们的都是推的。
- L4-a 的 GPU 一半、谱系树一半只读码。
- L1-b 的四个变体是我挑的形状；变异表里没加行，判别力自证只在我的三条换点用例上做过。
- 我的变体装置一度因 `cp -a` 带回旧 mtime 复用了上一个变体编出的库（见 L1-b 末段）；最终数据全部来自改过的装置，旧的 `variants/` 目录不作数。

## 没做什么

- 没判正推那一格，没替主 agent 采纳；副本上的数不算入库装置上的数。
- #9 两个钉值没核（要 checker 档）；两棵树之间的文件差没核。
- 没跑门禁、没跑 `crates/mutations.tsv`、没改任何仓内文件（只写了本报告与模型目录）。

## 要主 agent 跑的命令（都是 checker 档，提交时或用户要求时）

- E171 装置内联用例，对 L1-a 表里四种形状各打一行变异看红不红：`SINGLEFS_HEAVY_TESTS=commit cargo test --release -p singlefs-checker-tier --bin e171_three_oracle_collusion_after_the_oracle_fixes`，工作目录主仓根，经 `research/scripts/run-with-memory-cap.sh` 照提交时那一档的上限，看有没有一条在变异后由绿转红。
- GPU 对拍补一条墓碑树容器指针 extent 偏移坏流之后整份比（L4-a、L4-c），看 CPU 池级 checker 与 GPU 走读内核是否同时由绿转红。
- `crash_enumeration_sharded_across_processes` 在冻结树上跑一趟，核 #9 两个钉值。

## 附录 Ⅰ　`.claude/kb/invariants.md` 第 133 行（I-2.4，整行）

```markdown
| I-2.4 | 头校验和覆盖范围 | 任一单元的头校验和覆盖从偏移 0 到该类明文头的最后一个字节，端点写死：码 1 到偏移 104、码 2 到偏移 85 + 2 × key 宽、码 3 到偏移 106（D18（块里携带什么信息） 已定项 7 / 已定项 11 / 已定项 18 的字段表）；头校验和字段自身按 0 参与。**29 字节 nonce / MAC / 算法类型预留位虽然算进头（已定项 16）、也是明文，但不在头校验和覆盖内**——MAC 在头校验和之后才算得出来，罩不住它；单元头的这 29 字节与指针头部的 MAC 16 + nonce 12（D19（块指针的结构与宽度预算） 已定项 3）由同一条规则守：加密关着时恒 0、读者遇到非 0 一律判该结构损坏（已定项 17；指针头部那一半 2026-09-27 主 agent 定，池级判定在 A3-checker-2 里做），加密开启后由 MAC 自己罩（2026-09-14 用户定案把预留位从 28 加到 29，多的那 1 字节是算法类型）。指针头部的 extent 偏移 2 字节第一版恒 0（第一版指针指整个单元），读者遇到非 0 判该指针所在的结构损坏，与 MAC / nonce 同一处置、与加密开不开无关（D19（块指针的结构与宽度预算） 已定项 3；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行）。根记录里的指针偏移非 0 按这条根自证不过处置：core 退回上一条根（靠 journal 重放可以回到同一版，挂载照样成功），checker 红这一条而 core 挂载成功，两边各判各的、不矛盾（`research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 6 行）。加密开启（系统配置级、全有全无，I-6.4（无未加密块））后覆盖范围 = I-6.2（明文头字段白名单） 该类白名单里的全部明文头字段，密文段由 MAC 罩。改动覆盖区内任一字节必须失配、改动净荷字节不许失配 | 已实现（`crates/singlefs-checker` 的 `check_unit`：三类的端点各自写死，里程碑步 2 / 步 3 验收里改坏头任一字节判红，2026-09-14；池级走读在 `walk::judge_unit_header`（`crates/singlefs-checker/src/walk.rs:674` 起）另判单元头偏移 4 的格式版本 = 1（walk.rs:722，不认得就判该结构损坏、不按今天的字段表往下解）与 29 字节预留位全 0（walk.rs:758）；走读跟随的每条指针在 `judge_pointer_mac_and_nonce_are_zero`（`crates/singlefs-checker/src/image.rs:699`）判头部 MAC 16 + nonce 12 全 0，四个入口分别是码 2 节点指针（walk.rs:815）、inode 内部条目子指针（walk.rs:1485）、实例表链指针（walk.rs:1811）、extent 数据指针（walk.rs:1961），只判不断、指针指向的单元照样走下去，2026-09-27 实审 A3-checker-2；坏镜像 `crates/singlefs-harness/tests/checker_judges_reserved_bytes_and_widths_it_used_to_skip.rs`）。**extent 偏移那一半已实现**（checker 的指针判定 `judge_pointer_mac_and_nonce_are_zero` 加这 2 字节，走读跟随的每条指针都判；core 每个读盘上指针头部的读者都判 extent 偏移非 0、判该结构损坏：树表条目的树根指针、inode 内部条目的子指针、extent 叶记录的数据指针、extent 上段叶条目的下段根指针与内联数据指针、extent 树内部条目的子指针、分配记录树内部条目的子指针、实例表链指针各一处，加原有的 journal 新根段、码 2 内部条目、根记录三处（`crates/singlefs-core/src/pointer.rs` 的 `read_judging_the_extent_offset_from`；`grep -rn 'Pointer::read_from' crates/singlefs-core/src/` 只剩 `pointer.rs` 自己的单测一处）；影子账认单元的两个读者（`allocation_record_tree::node_pointers_as_far_as_readable`、`recovery::instance_table_page_pointers_as_far_as_readable`）今天也判，已定改回不判（影子账要多隔离、不漏隔离），改动在下一批；坏镜像遍历偏移 32、33，core 每类读者各一条用例在 `crates/singlefs-harness/tests/pointer_extent_offset_that_is_not_zero_is_refused_by_every_tree_reader.rs`；对应的账是 C581（extent 偏移非 0 没有判据）；`research/prompts/m3-inv-a-r1-main-verification.md` 第二节第 11 行与 `research/prompts/m3-batch4-code-r1-main-verification.md` 第二节第 5 行） |
```

## 附录 Ⅱ　`.claude/kb/decisions/16-发布语义.md` 第 140 行（D16 已定项 5 定案段，整行）

```markdown
**定案**：`T_time` 默认值 5 s，取可行区间 [200 ms, 5 s] 的上端——默认丢失窗口是 5 秒，取更小只减少摊销、不多买任何东西，拦住丢失的是那 5 秒本身。**`T_time` 用户可设，不限制**：挂载时传的值覆盖盘上的字段并回写；超出 [200 ms, 5 s] 不拒绝，只给警告（用户 2026-09-20）。`T_dirty` = 2 GiB（初值，写代码时再调）；它不由丢失窗口定——`∨` 规则下时间那一支已经保证了窗口，`T_dirty` 管的是**量**，它就是脏页内存预算本身。**`T_dirty` 是可调值，不是格式常量**：系统配置里那个字段是格式，字段里放什么数不是。**挂载时还要再夹一次**：有效 `T_dirty` = min(系统配置里那个字段, 环长 ÷ F)；环长是 mkfs 参数（默认 768 MiB、约束环 ≤ 设备容量 ÷ 4），所以「环装不装得下 `F ×` 最坏占用」不由一条 mkfs 时的下界保证，改成挂载时把脏字节预算压下去；默认环下有效值 = min(2 GiB, 256 MiB) = **256 MiB**，系统配置里 `T_dirty` 仍写 2 GiB、字段不动。
```

## 模型目录收尾

```
392a6c10d9c582ccc1162c0e16e2a2541ac54b25b1840cd4e824f1187cbeb20a  tools/rerun.sh
```

## cite-check

命令：`python3 research/scripts/cite-check.py research/prompts/m3-formal-code-b29-r1-opus-output.md --root /tmp/claude-1000/m3-formal-code-b29-r1/tree --root . --background research/prompts/_m3-formal-code-b29-r1-background.md`，末行：

```
  ✓ 核了 36 处引文，对上 36 处，没判 0 处（逐处列在上面）
```
