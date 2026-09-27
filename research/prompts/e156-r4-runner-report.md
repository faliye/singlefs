# E156（alloc-basis 四条岔路的代价数） 第 4 次重跑执行报告（第二次派发，r4b）

写于 2026-09-26 15:4x UTC（2026-09-27 00:4x JST）。重跑登记 `research/prompts/e156-r4-prereg.md`。主 agent 在上下文 78 万时叫停交接，停在这里；做完的、做到一半的、没开的都在下面。

## 结论

- 开跑前照 S2 用命令一重取五段，与换过的第二节逐字节相同（sha256 `af95e554dd52e87acc7095c129194171cac2e122ca1832d36be10882022e5de4`，产物前又取两次，同值），S2 不触发。命令四锚点脚本重跑，输出与 `--dump` 的 sha256 与登记相同。
- **前提 1：成立**（PQ1-前、PQ1-后：新实例第一个 txg = 洞 + 1、环里没有洞那条根；前洞槽从没写过，后洞槽里是 txg 4 的根）。**前提 2：不成立**（屏障之后 F_生效 已是新 F，两条推空根的 F 与 F_生效 都是 8）⇒ 岔路单第 3 行失效、不量。**前提 3**：只有岔路 2 那一族按旧式子定义；G-adm 三个数在全部历史上为 0。
- **岔路单第 1 行：按登记第十一节「够判停机」已够判**：38 格全部跑到终点、没有截断，12 个 (S, ρ, 回收时点) 上 Q1c-前、Q1c-洞、Q1d 都在 ≥ 2 个 h_缺 上有值，第一次越过 2 槽 / 1 槽的规模都报得出（h_缺 = 1、txg 7）。判定：12 点上 Q1c-前 全部「越过 2 槽」（前洞 h_缺 = 1、2、3、4 时 Δ 峰值 10、18、26、34，12 点逐点相同），Q1c-后 全部越过，Q1d 全部「随 h_缺 增长」；F1 在 12 点全部触发，F2 / F22 / F23 / F24 没触发。第二读法（不进判定）：按注入的洞数 k 分组时 Δ 峰值不随 k 增长。
- 什么现象会推翻：主 agent 不认 S1(i) 的对照造法改动（见「登记修订」第 1 条）⇒ 第一段按 S1(i) 停机、这一份产物作废；或 PQ1 在另一份 `crates/` 上 ② ≠ 洞 + 1 / ③ 有洞那条根。

## 报告项

- 单测：`cargo test -p singlefs-harness --bin e156_allocation_basis_counts` → `test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.27s`（最终源码 sha256 `93cd6dbd14cc5a80e3189153a0d48ca3c92efc1b9182f094d05d1a30ac7d14f0`，5762 行；数法 `grep -c '^test .* ok$'` → 17）。
- 变异（`crates/mutations.tsv`，点着这个装置的 22 行）：新加 M35–M43 九行（第 826–834 行）、改锚 M32–M34 三行（第 656、660、661 行，测试名跟着新单测改、去掉末尾 `$`），其余 10 行没动。门禁 59 号是重型不归我；我在草稿目录的仓副本里逐条施加、跑点名的测试（判法照 59 号 `verdict_of_run`），最终源码上：新 / 改锚 12 行 `TOTALS caught=12 failure=0 invalid=0 wrapper=0 stale=0`，其余 10 行（测试名去掉 `$` 再判）`TOTALS caught=10 failure=0 invalid=0 wrapper=0 stale=0`。三个数：抓到 22 / 无效 0 / 没红 0。门禁 33 号对这 22 行锚点唯一命中通过。
- 产物：`research/results/e156-alloc-basis-counts-2026-09-26.out` 与 `research/results/e156-alloc-basis-counts-2026-09-26-r2.out`（后者是把区域数 R 改成本地常量之后重跑的），两份 `cmp` 逐字节相同，sha256 都是 `1eaf5ea6e8aa164158af025342574927d2eed52433e0ec78405effeea4ac7bd9`，4007 行，完成标记 `E7RESULT name=done emitted=4007`。
- `replay.sh`：第 185 行先改指第一份，跑 `bash research/scripts/replay.sh E156` 报 `E156  @driver_e156             字节一致 e156-alloc-basis-counts-2026-09-26.out`；之后改指 r2（与第一份逐字节相同），**改指 r2 之后没再复跑**（叫停）。E156 注释段加了一段说明（replace-once.py，别的会话当时正在跑 replay.sh E158，所以用换 inode 的写法）。
- 实验页：`.claude/kb/experiments/156-alloc-basis四条岔路的代价数.md`（正文换成第 4 次重跑的现状，第 1–3 次的正文原样挪进页内「历史版本」的 `### 2026-09-26：第 4 次重跑之前的正文` 一条；加了「路径与结论登记」7 行）；索引行 `.claude/kb/experiments.md` E156 那一行；历史条目 `.claude/kb/experiments-history.md` 的 `### 2026-09-26：E156（alloc-basis 四条岔路的代价数） 第 4 次重跑……`（改前放了旧索引行全文）。
- 新建的入库装置 bin：没有新建，改写的是已有的 `crates/singlefs-harness/src/bin/e156_allocation_basis_counts.rs`（门禁 56 号点名清单照旧是这个文件）。

## 判决行（第 4c 步）

`grep -n 'name=verdict' research/results/e156-alloc-basis-counts-2026-09-26-r2.out` 共 14 行（第 1214、3994–4006 行，与第一份同行号）。没有取值为 `false` 或 `not_run` 的字段；名字表示违例 / 不匹配 / 失败 / 作废的整数计数（`g_adm_nonzero`、`s1_failures`、`a11_mismatches`、`a14_mismatches`、`v1_*`、`v2_*`、`v6_*`、`v16_*`、`v17_*`、`s11_*`、`s12_*`、`k8_mismatches`、`referenced_but_free_points`、`abandoned_or_unknown_root_points`、`anchor_rows_missing`、`truncated_cells`）全是 0。拿不准、点名的三个字符串字段（都在第 1214 行）：
- `premise_two_answer=not_holds`：前提 2 的答案「不成立」，登记第一节 1.1 预期内。
- `fork_three=void_not_measured`：岔路单第 3 行按前提 2 失效、不量，预期内。
- `stop_reasons=none`：第一段停机条款一条都没触发。
第 3995–4006 行 12 条 `stage=second_group` 的取值是 `above` / `grows_with_holes`，是判定值不是「没过」。

产物齐不齐（4b）：逐观测点 2141 行，逐格条数与锚点模型 `--dump` 那一格的观测点数逐格相同（`grep '^E7RESULT name=q1_observation ' … | awk '{print $3,$4,$5,$6,$7}' | sort | uniq -c` 对 `grep -o '^DUMP S=[0-9]* k=[0-9] pos=[a-z-]*' anchors.dump | sort | uniq -c`）；38 格 + PC-分配器 + PC-c2 = 40 条 `name=q1_cell`；PC-多扣那 234 + 180 行 × 2 个 ρ 多带四列。

## 登记修订（写在登记第十二节修订 2，第 1–13 条）

1. **S1(i) 在第一段草稿跑上触发，按 S1 行「改装置，写进第十二节」处置**：录写切段镜像与装置原有「发完只改回根槽」镜像每次都差每块盘一个系统配置扇区（实现在根槽 FUA 之后轮换系统配置槽，`transaction.rs` 的 `persist_the_root_then_rotate_the_system_configuration`）；对照镜像改成连系统配置两槽一起改回，S1(i) 判据不动，差的扇区逐洞照报。**要不要认交主 agent**。
2. 判定写在装置里（Rust 函数 + 单测 U18），不写 Python 脚本（执行员写范围不含 `research/scripts/`、门禁 47 号）；M43 进 `crates/mutations.tsv`。
3. W9 盘上路径对 txg 0–2 零单元那几版按根记录两条指针算。
4. G12 的「实」「每」按构造是同一次计算，F24 对 Q1b 触发不了。5. PC-多扣四个组合按构造同值。6. PC-洞按「恒 0 计数器对 A11 锚点」数不符点。7. U13 (a) 同进程实例表行数恒 0。8. K13、S1(k)/K14 的核法。9. M36 / M39 / M42 的替换文写法。10. 点着这个装置的 9 行变异测试名带 `$`，门禁 59 号按字面找不到，没动，交主 agent。11. 两段在一次运行里、第一段停机则只出第一段。12. PC-多扣挂在哪几格。13. S3 快照 sha256（第一份产物前后两次相同：diff `ecf57f9d…`、status `947a8fd6…`）。
- 修订 2 的文字是在第二段草稿跑（草稿目录 `draft3.out`，不入库）出过数之后写成的；第 1 条的改动在第二段任何数出来之前定；第 2–12 条是写装置时的取法。

## 岔路表

| 行 | 状态 | 算它的命令 | 登记里剩下的量能不能让它翻面 |
|---|---|---|---|
| 问题单前提 1（→ 岔路单第 1 行） | 已够判：成立 | `grep 'name=pq1 ' research/results/e156-alloc-basis-counts-2026-09-26-r2.out` | 不能（登记没有剩下的量）；主 agent 不认 S1(i) 改动则第一段停机、要重跑 |
| 问题单前提 2（→ 岔路单第 3 行） | 已够判：不成立 ⇒ 第 3 行失效、不量 | `grep 'name=pq2 ' …-r2.out` | 不能；PQ2 ① ≠ 新 F 才翻（SP2），量到的是 = 8 |
| 问题单前提 3 | 已够判（登记对文字的核）；G-adm = 0 | `grep 'name=verdict stage=first' …-r2.out`（`g_adm_nonzero=0`） | 不能 |
| 岔路单第 1 行 | 已够判（登记第十一节够判条件逐条满足）；Q1c-前 12 点全部越过 2 槽、Q1d 全部随 h_缺 增长 | `grep -E 'name=(q1c|q1d|verdict|geometry_sensitivity|failure_clauses) ' …-r2.out` | 剩下的只有 Q1e（附带，S = 8、ρ = 1 以外「够判后未跑」），翻不了面 |
| 岔路单第 2、7 行 | 用户已定，不量 | 无 | 不量 |
| 岔路单第 3 行 | 失效（前提 2） | 同前提 2 | 不量 |

## 交主 agent 的

1. **S3 在 r2 那一次触发**：r2 产物前后两次 `crates/` 快照不同（diff `7ec976b9…` → `2ed8d525…`，status `c49e35a0…` → `1a54ee32…`，UTC 15:35:37 → 15:39:18）；逐文件比：跑的那 3 分多钟里 `crates/singlefs-checker/src/walk.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/tests/instance_acquisition.rs` 被别的会话改了（不是这个装置、不是 `crates/mutations.tsv`）。r2 与第一份产物逐字节相同，第一份的 S3 两次相同。按 S3 不自己判「不构成停机」，交你定。
2. S1(i) 对照造法的改动要不要认（登记修订 2 第 1 条）。
3. I-3.11（已分配减 defer 等于最新根走读） 的条款正文还把失效的基底 `beta_hr_rollback_row` 与 `e156-alloc-basis-counts-2026-09-25-fork7-selfproof.out` 当「可达状态第 5 项可以为 0」的依据；今天挂着回退那一版第 5 项是 356 槽（`u13a_rollback_to_oldest_candidate … item5_slots=356`）。`invariants.md` 不在我的写范围，没改。
4. 抬 F 那一串扣住位比 F_生效 晚放（登记第三节 J7、J8），不是岔路单第 3 行的两个候选之一。
5. `crates/mutations.tsv` 第 551 行不是 E156 的变异（问题单第四节点名的行号对不上）；点着这个装置的另 9 行测试名带 `$`（第 197、291、292、651–655、747 行），门禁 59 号按字面判不出红。
6. 实验页「影响的决策」表：没有一条决策的「**依据**」段引 E156（`grep -n 'E156（'` 在 D28 / D16 / D3 / D5 / D8 零命中），关系全写备料，回看 2026-09-26；要不要升成支撑交你定。
7. 门禁（登记给我的 14 道，每道原样末行在草稿目录 `gate-*.log` / `gate2-*.log`）：27、34、52、75、80、84、85、86、96、99 退 0；33 退 1（第 301、690、692、714 行不是 E156 的，锚点在 `mount.rs` / `transaction.rs`，别的会话改的）；40 退 1（`e142_*`、`gate69-*` 几份不是 E156 的；E156 的 11 份产物都已点名）；69 退 1（E142 的装置、`research/prompts/e156-r4-prereg.md:59` 与 `e156-r4-questions.md:24` 引 `/tmp/claude-1000/impl-rbf-4a/report.md`——那是设计员抄进登记的问题单原文，不是我写的）；88 退 1（E142、E158 页的行，E156 页零命中）。

## 做到一半的 / 没开的

- `replay.sh` 改指 r2 之后没再复跑（叫停）；下一任跑一次 `bash research/scripts/replay.sh E156` 应报字节一致（r2 与第一份同 sha256）。
- 实验页里 `复跑` 那一段的汇总行是改指 r2 之前那一次复跑的原样；改指之后若复跑，回对那一段。
- 实验页改完之后门禁 99、75、34、85、84、40、88 已重跑（见上）；最后那次改动（r2 那一句与 7 处行号 +3）之后没再跑 99 / 88 / 40。

## 没做什么

- 没判这个实验的结论能不能推翻或确立决策（要走三方）；没跑门禁全量、15、59、87 号；没提交。
- 没改 `invariants.md`、决策文件、`research/scripts/` 下别的脚本、门禁 47 号。
- 草稿目录 `/tmp/claude-1000/e156-r4b-runner/`：`draft1-3.out`（第一段撞 S1(i) 前后的草稿跑，不入库）、`anchors/`（命令四抽出件与输出）、`quotes/`（三次 S2 重取）、`s3/`（快照）、`mutations/`（变异驱动脚本与 `run1-6.log`）、`gate*.log`、`progress.md`。仓副本 `mutations/copy` 与编译目录 `mutations/mutation-target` 已删；没有别的编译目录与工作树。产物与复跑用的是仓里的 `target/`（`cargo run -q`，与 `driver_e156` 同一条）。
