# 第 27–29 批代码轮第一轮判决（2026-10-05）

<!-- doc-lint:not-numbers L1 L2 L3 L4 L5 G1 G3 G4 G5 G6 C3 B4 KX1 KX2 T13 MUT A1 A2 A3 B1 B2 C1 D1 E1 E2 K1 -->

被判的文件（第 27、28、29 三批六份补丁与主 agent 两处改动动到的全部 `crates/`，逐份见改动全文 `research/prompts/_m3-formal-code-b29-r1-diff.md`）：
- 第 27 批：`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`、`crates/singlefs-harness/src/model.rs`、`crates/singlefs-harness/tests/model_compares_a_rollback_to_a_version_with_fewer_inode_leaf_containers.rs`、`crates/singlefs-harness/tests/model_compares_inode_creations_into_the_third_inode_leaf_container.rs`、`crates/singlefs-harness/tests/model_compares_inode_creations_across_two_inode_leaf_containers.rs`、`crates/singlefs-harness/tests/inode_tree_leaf_containers_appended_one_at_a_time_up_to_what_one_root_node_holds.rs`；
- 第 28 批：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/mount_transparency_judged_field_by_field_after_a_writable_mount.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_after_the_read_stage_on_a_read_faulted_crash_state_is_judged_apart_from_the_witness_criterion.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`；
- 第 29 批：`crates/singlefs-harness/src/mount_transparency.rs`、`crates/singlefs-checker-tier/src/crash_injection.rs`、`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs`、`crates/singlefs-harness/tests/writable_mount_refusal_on_a_crash_state_without_read_faults.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`、`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`；
- 主 agent：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs`（#9 补钉）、`crates/singlefs-checker-tier/tests/record_checker_judges_absence_by_the_persisted_set.rs` 与 `crates/singlefs-checker-tier/src/crash_amplification.rs`（只开一个特性编不干净）；
- `crates/mutations.tsv` 六份追加、替换与删除。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b29-r1-body.md`、`_m3-formal-code-b29-r1-diff.md`、`_m3-formal-code-b29-r1-background.md`（第一次生成 1.44 MB 那一份挪在会话草稿 `b29-materials-v1/`，欠账表只取表头与五行重做） | — |
| 冻结副本与快照 | `/tmp/claude-1000/m3-formal-code-b29-r1/tree/`（删除线第 19 批之后、根环线 10f 之前），清单同目录 308 行；kb 快照 `research/prompts/m3-formal-code-b29-r1-snapshot/kb-sha256.txt` | 写判决前 `sha256sum -c` 0 处不符 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b29-r1-sonnet-output.md`（sha256 a4e665de…），模型 `…-sonnet-model/` | 主 agent 照 `tools/rerun.sh all` 复跑 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b29-r1-opus-output.md`（sha256 d26d04a0…），模型 `…-opus-model/` | 主 agent 照 `tools/rerun.sh` 十步复跑 |
| 引文 | 两份腿报告 | 主 agent 各跑一次 cite-check：70 / 70、36 / 36 |

## 二、逐格判决

### L1　第 27 批

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 改的是不是 b25 判决与修订 5 说的 | 一致；61 行变异锚点各命中 1 次 | — | 站住 |
| T13 回退用例与 drop-retain / drop-extend | 两行都抓到；换取样点两行各罩一半（目标与现行同片数时 drop-retain 不红、目标没有右侧片时 drop-extend 不红），入库的 468:2:3 恰好两半都罩 | — | 站住：与 b25 判决 K1 第二行「删回退那一支两行中的任一行」同义 |
| 第 3 片用例与 KX1 / KX2 换取样点（L1-b / L1-c） | 打中：`.min(2)` 一类封顶第 3 片的变体在入库两条用例上活下来，703 步上红 | 打中：胶水封顶第 3 片、模型封顶第 3 片、回退只取回第 2 片、回退只丢第 3 片四个变体在入库两条用例上全绿；换成「建到第 4 片」「目标带第 3 片的回退」「目标没有第 4 片的回退」三条新用例四个全红；胶水那一个在 `--lib`、`random_histories`、`unlink_in_histories_judged_by_the_model` 上也全绿 | **两腿同判，坐实**（主 agent 照攻方 `tools/rerun.sh` 复跑十步逐条同：`kx1_cap_at_third` 在 `b29_opus_leaf_container_sampling_points` 上 `1 passed; 2 failed`、在 `lib` 上 `82 passed; 0 failed`、在 `random_histories` 上 `23 passed; 0 failed; 19 ignored`；`kx2_cap_at_third` 只被 `model::tests::the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond` 抓到）。不违反 b25 判决字面（它只要 2 × 233 + 4），但「第 k 片都对」今天只罩到 k = 3。改法 B1、B2：攻方三条换点用例入库，四个变体各一行变异 |
| 135 边界两条用例 | 两个方向都罩，k 逐条全扫 | — | 站住 |
| 逐件 `catch_unwind`、`piece_panics`、`unattributed_panic_states`（L1-c / L1-e） | 没有静默漏算：逐件重做之外的七处语句 panic 时，dump 模式记 `pipeline_panicked`、Q5 进程 panic，都是响的；同一次 panic 在 `layer0_oracles` 与 `crash_judge` 两列各记一次 | 没打中：同样三形（重做路上的胶水不在任何一件里、各件列不互斥、读回被并进两件），今天都到不了 | **两腿同判**：没有既不进逐件计数、也不进 unattributed 的静默态。`piece_panic_states=` 按件计、不按态，这是登记修订 5 的口径，不改装置；写进 E171 实验页的读法说明。攻方 A3（判决行加「逐件那一路走过的态数」）零轮，不做 |
| `judge_the_confirmed_loss_shapes` 的四个调用点（L1-a / L1-f） | 坐实：改坏任一个没有测试会红 | 坐实并补形：U8 两次都在第 8028 行那一支退出，四个调用点连同其后六个出口没有一条用例走到；变异表点名它的行 0 条 | **两腿同判，坐实**（主 agent 现查：`grep -n 'judge_the_confirmed_loss_shapes('` 只命中 `crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion_after_the_oracle_fixes.rs:7981`（`run_confirmed_loss_form`）与第 11837 行（U8）；第 8028 行起的那一支先退；`crates/mutations.tsv` 里两个函数名命中 0 次）。改法 A1、A2：给 Q5 那条路一个注入入口，四个调用点各一条用例、各一行变异。**等 E171 第一段跑完再做**（装置源码在长跑期间不动） |

### L2　第 28 批 CPU

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| I-9.16 ④ 十一种 | 288 段历史 8140 次 `check_pool_image` 0 误红，三份大树镜像都 `Holds`；缺 deadlist、缺 logged_ops 两行都抓到；kb 今天没有 ③ ④ | 没打中 | 站住；kb 补 ③ ④ 归写回批（b26 判决第三节第 3 条已列） |
| I-5.4「成立要至少一棵树走到叶」（L2-b） | 符合判决字面，但宽一档：B 版分配记录树读不出而 A 版读得出时 I-5.4 仍 `Holds`（26 个单元逐个清零，`slot=50257` 到 `slot=50264` 那几行）；整体 checker 仍红（别的不变量） | 没打中 | 站住（实现照判决「至少一棵」）。零轮收严：按版本记「这一版的树走到叶没有」，没走到那一版的全零格进部分没判。它只把「成立」收窄成「部分没判」、不新增违例，并进第 30 批 |
| G1 运行配置期望字节（L2-a / L2-c） | 一致、全：三个常量与条款三行相同，三类改坏各被抓 | 打中判据写法：条款只写「4 字节放 5 秒」、没写编码单位；G1 的 5 用的是 core 的「秒」；D16 已定项 5 定的可设区间下端 200 ms 在「秒」里写不出 | **分歧核过，两边各对一半**：值对（正推），单位不在条款里（攻方）。主 agent 现查：`.claude/kb/layout/01-first-txn.md:149` 一行是「可调值 \| T_time \| 4 \| 5 秒 \| D16（发布语义） 已定项 5」，同文件第 158 行的偏移说明只给偏移 433，没给单位；`.claude/kb/decisions/16-发布语义.md:140` 定 T_time 用户可设、区间 [200 ms, 5 s]、挂载时传的值回写盘上字段；core 今天只写常量（`crates/singlefs-core/src/system_configuration.rs:290` 的 `time_threshold_seconds` 交 `TIME_THRESHOLD_SECONDS`），还没有用户设值的路。这是两条条款（字段表与 D16 已定项 5）之间的格式空白，交用户定：**用户 2026-10-05 定改成毫秒**（弹窗选项原文「改成毫秒：字段表写明「单位毫秒」，默认值 5000；core 与 G1 一起改」），并进第 30 批 |
| journal_tail 两半 | 两半各有会红的用例；判的是择到的那一份系统配置，不逐盘（推的） | 没打中 | 站住；「不逐盘」记欠账（另一块盘上落后的 tail 这一行判不到），不在这一批改 |
| G3 判别子 | 判官靠调用方声明，崩溃注入两处声明站得住（被拒的挂载起的是不带故障层的 `SparseBlockDevice`）；`TheRefusedMountReadThroughInjectedFaults` 没有生产调用点 | 没打中 | 站住 |
| G5 | 与 D13 射程句同向 | 没打中 | 站住 |
| G6 的 C3 / B4（L2-g） | 用例只罩两个点；二次崩溃种子是历史种子、盐、段号、子集指纹四者异或，(种子 4, 段 0) 与 (种子 0, 段 4) 撞到同一个种子 | — | 站住（b24 是最后一轮）。异或对撞是取样独立性的缺口，不是判错：并进 b24 那笔「F3 剩下的漏形」欠账，零轮改法（分开哈希）写进那一行 |

### L3　第 29 批 G4

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 合法历史上误红 | 没找到：192 段 6 步历史与读回全 0 一组 16 段 8 步历史，第一次崩溃约 105 万态、二次崩溃约 34 万态里「某块盘 0 份自证过」0 个；探针在 MUT-B1 同形的坏核心上数到 4 120 / 44 203 | 没打中：崩溃注入整写子集、轮换写不覆写最大世代那一槽、每段系统配置写前有屏障；唯一的反例在 mkfs 那一段（两槽同在一段里写），两套装置都不摆这一段 | **两次独立抽样都没打中**。mkfs 那一形主 agent 现查：`crates/singlefs-core/src/make_filesystem.rs:650-660` 逐盘两槽在同一个循环里写，之后才逐盘屏障；崩溃注入的候选段要在 mkfs 之后（`crates/singlefs-checker-tier/src/crash_injection.rs:3814`），E171 基镜像取 mkfs 之后。改法 D1（零轮）：G4 的射程写明「mkfs 返回之前的崩溃态不在射程」，与两套装置的实际取样一致，归写回批 |
| 第二、三截与 C670 同不同形 | 同接法，三处不同形：① 择不到系统配置时 C670 的复算交 `None`、G4 的按真（这时读阶段本身已拒，到不了）；② C670 的条件带「哪一支」、G4 无载荷；③ G4 无故障态上只剩「某块盘一份自证过的都没有」那一条 | 同形 | 站住；三处不同形都不改判定，① 记进写回批那一行的射程 |
| 两条用例是不是都从条款推得出（L3-c） | 一半：注入口径下合法由 D13 射程句推得出；手工两槽损坏翻成不合法条款推不出，靠 C670 类推与 b26 判决 G4（零轮），不变量清单里没有「每块盘在每个没有读故障的崩溃态上至少一份自证过」 | 由 G4 字面推得出 | **正推对**：G4 今天是一个只有判决依据、没有 kb 行的性质判官。写回批给它一行条款（并进 C670 那一行或在 I-7 立一条），连同 D1 的 mkfs 射程 |
| E171 装置补的那一臂 | 只是 `match` 穷举，没有用例点它 | 同 | 站住 |

### L4　GPU：26-5 与 29-2

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| 墓碑树 / 谱系树里指向容器的子指针 extent 偏移（L4-a） | 读码同：池级 checker 与 GPU 走读内核都只判 MAC / nonce，core 恢复判 extent 偏移而拒 | 打中：副本上 extent 偏移改 1，池级 checker `I-2.4=Holds`、`violated=[]`；同一条指针 MAC 改 1 照红 I-2.4；core 恢复与可写挂载都以「墓碑树或谱系树内部条目的子指针头部 extent 偏移不是 0」拒 | **两腿同判，坐实**（主 agent 复跑 `tombstone` 一步逐行同；现查 `crates/singlefs-checker/src/walk.rs:2064` 只调 `judge_pointer_mac_and_nonce_are_zero`、第 2110 行照样跟进，`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:2928` 只调 `w_judge_pointer_head`）。两臂一起漏，不分辨 CPU 与 GPU，病根在两边共用的「哪几处判 extent 偏移」清单；`.claude/kb/invariants.md:133` I-2.4 那一行说走读跟随的每条指针都判，这一种上不成立。代码出自 deee47f4，不在这一轮 diff 里；I-2.4 extent 偏移这一格是形式化线这几批在补的，改法 E1 并进第 30 批，改前告诉删除线（墓碑树走读那几段是它写的） |
| I-9.16 ③ ④、根槽 428..457、分配记录树根全零 CPU 与 GPU 同不同（L4-b） | 找到一形 CPU 红、GPU 恢复内核绿：树表两条同种类、种类码 ≥ 32——core 恢复拒（实测 K = 7、13、31、32、40），GPU 恢复内核只记 `kind < 32` | 没打中：逐格读码同 | **正推的那一形坐实**（主 agent 现查 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2947` `if (kind < 32u)`，第 2953 行注释自己写明拦不到）。不是合法历史上的误判（正确实现写不出种类 ≥ 32），是造出来的坏盘才露的 GPU 与 core 差别。零轮改法：恢复内核对种类 ≥ 32 的条目逐条与前面比，对拍加一条 K = 40 的重复，并进第 30 批；第 2953 行注释里「走读同样拦不到」与走读内核比整个 `u16` 的现状不符，一起改 |
| 「只违反 ③ 的表超额整态交 CPU」（L4-c） | 冲突：放量路径判不了就整趟停下报错，不退回 CPU | 交得回或判得对 | **正推对**（主 agent 现查 `crates/singlefs-checker-tier/src/crash_amplification.rs:2003-2016`：遇到判不了的态 `work.stop`、发「配置开了 GPU，不退回 CPU 判器」那一句后返回）。更正主 agent 在第 28 批记录 26-5 交回那一条的说法：只违反 ③ 的表超额时是整趟停下报错，不是整态交 CPU。「GPU 上 ③ 没有变异行」那一判的结论不变——判得了的态上去掉 ③ 是等价变异（条目 ≤ 11 而违反 ③ 必同时违反 ④），判不了的态让整趟响停，不是静默放过。`crates/singlefs-checker-tier/src/crash_judge_gpu.rs:6` 文件头那句「判不了的状态……交 CPU 判」与放量路径不符，并进第 30 批改注释 |
| 新加六条、改一条对拍各罩哪一处（L4-d / L4-c） | 逐条对上变异行；没有变异行的三处（已知）外加缺一条 K ≥ 32 | 七条流罩七类指针；GPU 恢复内核在墓碑树 / 谱系树条目上那一判没有流罩，连已知三处共四处 | 站住；墓碑树那一处随 E1 补对拍坏流，多层树三处照旧欠一条长出多层树的流 |

### L5　主 agent 的两处

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| #9 两个钉值的出处 | 值与两份日志的 `left` 逐字同、出自「第 28 批 + 10e」那棵树（日志里编的分别是主工作区与已删的副本 `n9-copy`）；冻结副本上仍成立是读码推的：那条用例的输入里 29 批、第 19 批与主 agent 两处一个字没动 | 没判（要跑 checker 档） | 站住到提交时那一趟 `checker-tier-layer0-replay` 为止；它的 `left` 不是 `e1ab3f79…`（58 行）或 `2a77c2d7…` 就重取 |
| cfg 改动 | 两个特性都开时编进来的集合不变；单特性组合没复核 | 门禁 feature-gated-tests 格从 cfg 现读特性名、两个都开，改过的用例照样编进来 | 站住；单特性组合主 agent 合入前已在副本上四种都编过（记录第二十一节） |

## 三、去向

1. **第 30 批实现员**（补丁，在今天主工作区上做，已含根环线 10f）：
   - 30-1 harness：攻方三条换点用例（建到第 4 片；目标带第 3 片且现行重写过它的回退；目标没有第 4 片的回退）入库，四个变体（胶水封顶第 3 片、模型封顶第 3 片、回退只取回第 2 片、回退只丢第 3 片）各一行变异，逐条证红（B1、B2）。
   - 30-2 池级 checker 与 GPU：E1（墓碑树 / 谱系树容器子指针判 extent 偏移，CPU 一处、GPU 走读内核一处，坏镜像照攻方 `tests/b29_opus_tombstone_container_pointer_head.rs` 的造法进 `checker_known_bad_images.rs`，对拍加一条坏流）；GPU 恢复内核种类 ≥ 32 的重复（对拍加 K = 40）；I-5.4 按版本记走没走到叶（L2-b 收严）；GPU 恢复内核与 core 在 MAC / nonce、算法类型、压缩码、压后长度几段对齐（第 28 批起记下的那一项）；`crash_judge_gpu.rs:6` 与 `crash_judge_common.wgsl:2953` 两句注释照现状改。
   - 30-3 T_time 改毫秒：core 系统配置写出与读取 T_time 的单位改成毫秒、默认 5000，G1 期望常量照改，受影响的钉值重取（用户 2026-10-05 定）。
2. **等 E171 第一段跑完**：A1、A2（Q5 四个调用点的注入入口、用例与变异），与 E171 要不要在新树上重跑一起判。
3. **写回**（并进写回批）：G4 那条性质的条款行与 mkfs 射程（L3-c、D1）；I-2.4 那一行照 E1 之后的实测清单写（E2）；E171 实验页写明 `piece_panic_states=` 按件计；欠账三笔：journal_tail 不逐盘、二次崩溃种子异或对撞（并进 b24 那笔）、多层树三处没有变异行（要一条长出多层树的流）。
4. **T_time 改毫秒**（用户 2026-10-05 定）：30-3 实现员改 core 写出与读取的单位（默认 5000）与 G1 的期望常量，重取受影响的钉值；字段表 `.claude/kb/layout/01-first-txn.md` 第 149 行写明单位毫秒、默认 5000，归写回批；系统配置槽字节变，合入前告诉根环线与删除线哪些钉值会动。
5. **下一轮代码轮**（第 30 批合入后）：攻 E1、K ≥ 32、I-5.4 按版本与 GPU 加密压缩几段的对齐。

## 四、复跑核对

攻方：把模型目录拷到会话草稿 `b29-opus-rerun/model/`，`bash model/tools/rerun.sh <草稿>/draft <步>` 逐步跑 prep、tombstone、baseline、kx1、kx2、extend、retain、controls、lib、broad-kx1，十步都退 0。逐行与报告和 `out/variants2-summary.txt` 同：`tombstone` 一步 `extent_offset_1 checker I-2.4=Holds`、`extent_offset_1 checker violated=[]`、`mac_byte0_1 checker violated=["I-2.4"]`；`baseline` 三份全过；`kx1` / `kx2` 在换点用例上 `1 passed; 2 failed`、入库两条 ok；`extend` 红两条、`retain` 红一条；两个对照变体各红入库那一条；`lib` 一步 `kx1_cap_at_third` `82 passed; 0 failed`、`kx2_cap_at_third` `81 passed; 1 failed`；`broad-kx1` 两份全过。

正推：`REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b29-r1/tree WORK=<会话草稿>/b29-sonnet-rerun RERUN_ALREADY_WRAPPED=1` 经 `run-with-memory-cap.sh 8G` 与 `capped.sh 10` 跑 `rerun.sh all`，退 0。setup `sha_not_ok=0`；anchors 退 1 是冻结副本不带 `litmus/`、`static-check` 报那一行 litmus 锚点找不到文件（腿的报告第 291 行写明），其余 `NOT_ONE 0`；变异 31 行逐行结局与报告同（入库行 0、1、4、5、18、20–32、36、44–46、48 全 `CAUGHT`，腿加的 102–105 在入库用例上 `SURVIVED`、110、111 `CAUGHT`）；L2 扫描种子 30–53 八段与 `out/l2-final-sweep.txt` 对应八节逐字相同（去掉耗时行比 md5）；L3 合计第一次崩溃 1 028 878（无读故障历史）+ 21 198（有）态、二次崩溃 300 653 + 43 519 态，「某块盘 0 份自证过」全是 0，与报告的分项和相同；阳性对照 `second_cut no_read_fault_history states=4088 disks_with_zero_valid_slots_states=1264` 同；三份大树镜像 `violated=[]`；`L4PROBE kind40 violated=["I-9.16"]`、`L2PROBE slot=50257 … I-5.4=Holds` 同；网格各格的结局与报告同（每行跑的格子比腿那一趟全，703 步那几格在 106–109 下都 `NewFinding(单元的分配代 …)`、468 步各格都 `Completed`）。

核不动：checker 档用例（E171 装置内联用例、崩溃注入、#9）、GPU 对拍与变异整表，两腿都没跑；L1-a 的「没有测试会红」与 L4 的 GPU 一半是读码。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | T_time 的盘上编码单位：条款字段表只写「4 字节、5 秒」，D16 已定项 5 要求可设到 200 ms 并回写盘上 | 攻方，主 agent 现查两处条款 | 推的（core 今天只写默认值） | 用户 2026-10-05 定改成毫秒，第 30 批 |
| 2 | B1、B2 三条换点用例与四行变异 | 攻方 | 副本上量过、主 agent 复跑同 | 第 30 批 |
| 3 | E1 墓碑树 / 谱系树容器子指针判 extent 偏移 | 攻方量过、正推读码同 | 缺口量过、改法推的 | 第 30 批 |
| 4 | GPU 恢复内核种类 ≥ 32 的重复 | 正推 | CPU 一侧量过、GPU 推的 | 第 30 批 |
| 5 | I-5.4 按版本记走没走到叶 | 正推 | 缺口量过、改法推的 | 第 30 批 |
| 6 | A1、A2 Q5 四个调用点的注入入口 | 两腿 | 推的 | E171 第一段跑完后 |
| 7 | G4 的 mkfs 射程与条款行 | 攻方、正推 | 推的 | 写回批 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D13（验证路线） 已定项 1 | 支撑 | 2026-10-05 不受影响：G5 与 G3 照射程句办；G4 的条款行与 mkfs 射程归写回批 |
| D16（发布语义） 已定项 5 | 备料 | 2026-10-05 不受影响：定案句不改；字段表照它补「单位毫秒」（用户定），D16 正文的 [200 ms, 5 s] 因此写得出 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-05 不受影响：树表字段表不变，GPU 恢复内核照 core 收严 |
