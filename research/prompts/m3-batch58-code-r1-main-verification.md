# m3-batch58-code-r1 判决：删除线第 58 批代码轮第一轮（2026-10-10）

被判的改动（按路径点名）：`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/corrupt_on_disk_content_is_refused_instead_of_panicking.rs`、`crates/mutations.tsv`；`crates/*/src/` 一行没动。全文 `research/prompts/_m3-batch58-code-r1-diff.md`，正文 `research/prompts/_m3-batch58-code-r1-body.md`。

## 一、证据核对（主 agent）

| 证据 | 核法 | 结论 |
|---|---|---|
| 开工快照 | 冻结副本 `~/.local/share/singlefs/work/m3-batch58-code-r1/repo/` 与主工作区各 `sha256sum -c research/prompts/m3-batch58-code-r1-snapshot/crates-sha256.txt`；仓根 `kb-sha256.txt` | crates 4 份（含变异表）两处都对；kb 全对 |
| 正推腿 `research/prompts/m3-batch58-code-r1-sonnet-output.md`（205 行，sha256 595a0cb1ea9d…）、攻方 `research/prompts/m3-batch58-code-r1-opus-output.md`（232 行，sha256 fe3c8f793d34…） | `cite-check.py` 两份合判，`--root` 仓根，`--background` 背景材料 | 「✓ 核了 51 处引文，对上 51 处，没判 0 处（逐处列在上面）」 |
| 攻方模型 | 拷到 `~/.local/share/singlefs/work/m3-batch58-code-r1-verify/model/`，`rerun.sh run all` | 见第六节 |
| 合入后验证（主 agent 在主工作区跑） | `merge-and-verify.sh`、checker 档 CPU 快档、带 gpu 那一趟、8 行变异点名、J4 那一条单跑 | harness「✓ 测试全绿（194 个测试二进制，共 2008 条测试）」；CPU 快档「61 targets passed 668 failed 0 ignored 16」；带 gpu 那一趟（`~/.local/share/singlefs/work/m3-batch58/feature-gated-b58-r2.log`）「61 targets passed 1065 failed 1 ignored 20」，红的一条是 J4 改造后的 `the_gpu_judge_agrees_with_the_cpu_when_product_tombstone_containers_after_a_full_one_skip_a_number`，J3 三条 GPU 旋钮对拍绿；8 行变异「抓到 7 条、没红 0 条、无效 0 条」，「基线不绿 1 行」是第 3849 行（点名的就是 J4 那一条）。J4 单跑（`~/.local/share/singlefs/work/m3-batch58/j4-single.log`）原样「CRASH_JUDGE_GPU flow=product-tombstone-containers-after-a-full-one states=25 cpu_red=0 gpu_red=25 cpu_checker_red=21 gpu_checker_red=0 recovery_mismatches=25 cross_check_mismatches=25 mismatches=25 checker_mismatches=25」，每个状态对拍字 `gpu_could_not_judge: true` |
| 攻方 K1-3 的 GPU 那一半 | 冻结副本拷到 `m3-batch58-code-r1-verify/k13-repo/`，`crash_judge_checker_scan.wgsl` 第 3357 行改成只判倒数第二片（`container_index + 2u == d_container_count && …`），跑三条 `…written_under_changing_knobs`（`m3-batch58-code-r1-verify/k13.log`） | 改坏下三条照绿，原样「test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 190 filtered out」：坐实 K1-3 |

## 二、逐格判

| 格 | 正推 | 攻方 | 判 |
|---|---|---|---|
| K1-1 新用例只因点名的写错而红 | 一致；拷贝上六行 CPU 变异（第 3852–3857 行）各只红点名的几条 | 没打中，量过：基线 9 条绿；只判叶序第一片红在清单变空那一句 | 两条腿同向，过 |
| K1-2 转根环后 [2,1,2,1] 判到没有 | 一致；不转根环的变体在「只判倒数第二片」下照绿（`row-3854-variant-no-ring-turn.log`），注释「要转出去」成立 | 没打中，量过 | 一致，过 |
| K1-3 GPU 三条是不是在比旋钮支 | 一致（读码），红绿复核不了 | **打中**：GPU 别的旋钮流都只两三片、少的那片只在倒数第二位，内核改成只判倒数第二片时全部 GPU 对拍照绿；CPU 替身量过，替身会红已自证 | 采。主 agent 在卡上坐实（第一节最后一行）：J3 三条 GPU 对拍分不开「只判倒数第二片」，判决 J3 验收在 GPU 那一半没罩到这一形（L1） |
| K1-4 只判最右之前两片 | 没报 | **打中（弱）**，量过：三种排布少的那片离最右都不超过两片，改坏下默认档与慢档逐字节同基线 | 采。这一族对任意 k 关不死；修法把少的那片放到离最右更远的地方（第三节 L2） |
| K1 条款侧 | 规则没说：kb 的 I-9.22 行只写「≤ 旋钮值」，I-9.37 在 kb 里没有行 | — | 已在 kb 写回清单，不改判法 |
| K1 慢档标签 | 规则没说：三条 `#[ignore]` 用时不稳，忙时单条 51–170 秒、闲时三条合计 2.5 秒 | — | 第 59 批实现员在 debug 下各单跑一次量用时，按 `.claude/rules/verification.md`「harness 档里再分快档用例与慢档用例」的 60 秒判（第三节 L5） |
| K2-1 新起点镜像下状态集合 | 一致：新写表 56 条与旧造法尾部逐条相同，旧造法前 6535 段不出状态 | 没打中，量过：两种造法都是 25 个状态，段、写、取几态、撕裂镜像逐项相同 | 两条腿同向，过 |
| K2-2 撞内核别的读宽上限 | 一致（读码加量）：枚举写 56 ≤ 2048、落点写 2 ≤ 64 | **打中**：各版 1242 条分配记录、1202–1204 条已释放，GPU 恢复内核 `MAX_ALLOCATION_RECORDS` 1024（`crash_judge_common.wgsl:151`）、池级 checker `C_MAX_RELEASED` 1024（`crash_judge_checker_common.wgsl:467`）都过；逐个 unlink 时 txg 294 就过 1024 | 主 agent 在卡上单跑坐实结局（第一节）：25 个状态 GPU 全判不了，不是判错。正推核的是表的读宽，没核内核里的容量表，两腿不冲突。J4 两条改法都中，交用户（第五节） |
| K2-3 改号那一形 GPU 与 CPU | 复核不了 | 推的，同 K2-2 | 同 K2-2 |
| K3-1 三张新盘只由点名的次序分得出 | 一致；三行新变异（第 3859–3861 行）各只红点名的用例 | 没打中，量过 | 一致，过 |
| K3-2 删记账根 key 最小一行 | 一致；两种读者改法下五条全绿；附线索：`core_review_tree_table_duplicates_and_slot_one_search.rs` 第 584 行那条老用例仍把这一版自己那行水位的标签改成 13，同一补丁下变红 | 没打中，量过：删的是「已分配字节」、盘 0、代 4，两种读者改法下 41 条照绿 | 一致，过。线索主 agent 现查：那条用例（第 540、584 行）造坏法是 `own_row.statistic = STATISTIC_INODE_WATERMARK + 1`，与第 57 批 K3-3 同病，进第 59 批（L4） |
| K3-3 可写挂载钉的次序 | 一致：重建分配记录逐条判（`recovery.rs:2894`）在数映射条目（`recovery.rs:2944`）之前 | **打中**，量过：钉住「重建里分配记录判在数映射条目之前」；没钉「重建把分配记录判挪到条目宽与逐条判之后」，41 条全绿，J1 ② 那张盘交可写挂载分得出、但没有可写挂载用例；另外第 3861 行那一形第 57 批可写挂载那条也红，证不出新用例自己多钉了什么 | 采。补 J1 ② 盘的可写挂载用例，换一行只有新用例红的变异（L3） |

## 三、采纳的改法（第 59 批）

| 编号 | 改法 | 验收 | 来源 |
|---|---|---|---|
| L1 | GPU 三棵树各加一条 [2,1,2,1] 的旋钮对拍流，判之前转一圈根环（与 harness 慢档同形） | GPU 内核「只判倒数第二片」那一形（`crash_judge_checker_scan.wgsl` 第 3357 行）一行变异，红在新加的三条上 | 攻方 K1-3 |
| L2 | harness 三棵树各加一张少的那片离最右至少三片的盘（如 [2,1,2,2,1]） | 「只判最右之前两片」变异红；CPU 一行变异 | 攻方 K1-4 |
| L3 | J1 ② 那张盘（分配记录代晚于根 + 这一版自己那行水位代晚于根）加可写挂载一条，报分配记录 | 重建「分配记录判挪到条目宽与逐条判之后」一行变异红；第 3861 行换成「重建读完记账树先数映射条目、再判分配记录」那一形，只红 J1 ① 的可写挂载那条 | 攻方 K3-3 |
| L4 | `core_review_tree_table_duplicates_and_slot_one_search.rs` 那条「这一版自己那行水位坏了、只剩台阶」的用例换造法，不靠标签 13 | 攻方第 57 批两种读者改法（拒标签 > 12、数条目时不算未登记标签）下用例照绿、报的仍是水位行缺；原有判别力那一形（只判水位统计量有一行）照红 | 正推 K3-2 线索 |
| L5 | J3 三条慢档在 debug 下各单跑一次量用时，不到 60 秒的去掉 `#[ignore]` | 报告写三条的用时原样 | 正推 K1 |
| L6 | GPU 两张表 `MAX_ALLOCATION_RECORDS`、`C_MAX_RELEASED` 抬到 2048，草稿区布局与 `SCRATCH_WORDS` 跟着改，dispatch 钉的展开值重核；两处上限各加一条「刚好 2048 判得到、2049 判不了」那一类的边界用例 | J4 那一条用例绿（25 个状态 0 个不一致）、第 57 批 4 行 GPU 变异抓到；两处新上限各一行变异（改回 1024 红） | 攻方 K2-2、主 agent 单跑；用户定（第五节） |

## 四、零轮与欠账

- L1–L6 被攻过零轮，第 59 批合入后开它的代码轮。
- 单次没打中、不作结论的格：K1-1、K1-2、K2-1、K3-1、K3-2 两腿同向、各抽一次。
- K1-4 那一族（只判最右之前 k 片）有限组排布关不死；L2 把窗口推到 3。
- 上一轮记下的欠账与线索照旧（第 56、57 批判决第四节）。

## 五、交用户表

| 岔路 | 候选 | 代价（出处） |
|---|---|---|
| L6 J4 怎么钉 | ① 测试里改写短历史的流，伪造满一片 815 条的墓碑容器（只改测试）；② GPU 两张表 `MAX_ALLOCATION_RECORDS`（`crash_judge_common.wgsl:151`）、`C_MAX_RELEASED`（`crash_judge_checker_common.wgsl:467`）从 1024 抬到 2048；③ 只在 CPU 上钉、GPU 那 4 行登记证不了 | ①没量，伪造的记录可能撞别的不变量（推的）；②每个状态草稿区多 11 264 字，在 `crash_judge_gpu.rs:492` 的 320 696 上约 +3.5%，一趟能派的状态约少 3.4%（推的），要改根环线的 GPU 判器、重核展开预算（`crash_judge_dispatch.rs:247` 上限 400 000，第 1506 行钉 395 566），别的表会不会接着撞没查到底；③GPU 产品支墓碑那一处没有测试盯着。**用户 2026-10-10 定 ②**「抬高 GPU 两张表的上限」 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D18（块里携带什么信息） 已定项 24 | 不影响 | 2026-10-10 不受影响：第 3 条「满了才关」照写，第 58 批只改测试 |
| D5（快照 / 空间记账机制） 已定项 4 | 不影响 | 2026-10-10 不受影响：J2 换造法后用例不再用统计量 13；L4 同向，不改条款 |

## 六、复跑核对（攻方模型）

攻方模型拷到 `~/.local/share/singlefs/work/m3-batch58-code-r1-verify/model/`，在空工作目录 `run/` 从头跑 `SINGLEFS_SCRIPTS=scripts OPUS_THREADS=15 OPUS_MEMORY_CAP=8G bash model/rerun.sh run all`（冻结副本取缺省路径），compare 原样末行「compare：14 份，不一致 1 份」：不一致的是 `mut-row3859.txt`，与腿交的差两行，一行 `OPUS_K3 image=base recover=FileRead …` 被并行测试的 `test the_allocation_record_tree_of_a_version_without_file_is_judged_like_the_cold_walk_on_the_writable_mount ... ok` 插断（第一趟原样存在 `mut-row3859.first-run.txt`），判定行与「test result」行不变。单跑这一段（`rerun.sh run mut-row3859`）与腿交的逐字「一致」。其余 13 份逐份「一致」。第二节引的攻方数（K1 三条流 447 个状态、只判倒数第二片与基线摘要相同、只判第一片违例归 0；K2 新旧造法 25 个状态逐项相同、各版 1242 条分配记录与 1202–1204 条已释放、txg 294 过 1024；K3 删的那一行与两种读者改法下 41 条照绿、`r-rebuild-alloc-after-each-entry` 下 41 条全绿而 j1b 盘交可写挂载报记账条目的代）都在这 14 份里。模型的输出抓取不定序（并行测试的 stdout 交错），记作攻方模型的毛病，不影响判定。
