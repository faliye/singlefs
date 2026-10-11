# 第 30 批代码轮第一轮 云端正推腿报告（2026-10-05）

<!-- doc-lint:not-numbers M1 M2 M3 M4 E1 B1 B2 G1 KX1 KX2 I1 -->

腿：云端正推（Sonnet）。格：正文第三节 M1–M4。立场：从判决 `research/prompts/m3-formal-code-b29-r1-main-verification.md`、`.claude/kb` 条款与用户定案正推每处改动做的是不是它们说的，每格给判定、引文与推翻条件。
方案按 `crates/` 今天的实现来谈；代码引文读冻结副本 `/tmp/claude-1000/m3-formal-code-b30-r1/tree/`，写那棵树里的行号；kb 引文写 kb 文件自己的行号（现查）。
模型与脚本：`research/prompts/m3-formal-code-b30-r1-sonnet-model/`（`tools/rerun.sh` 分段复跑，原样输出在 `out/`）。判定词只用：一致 / 冲突 / 规则没说。

## 0　读法与复核状态

- 跑了的：harness 档与池级 checker 的测试二进制（`cargo test -p singlefs-harness --test …`、`--lib`），在冻结副本的草稿拷贝上；变异行用本模型的 `tools/mutants.py`（施加、编、跑、还原）逐行跑，没跑 `crates/mutations.tsv` 整表。
- 没跑的：checker 档（`singlefs-checker-tier` 的测试、GPU 对拍、GPU 变异 9 行）一条没跑；凡是涉及它们的结论写「推的」。
- 三棵树（草稿里，复跑时 `rerun.sh setup` 重建）：`tree`（冻结副本原样加探针）、`tree-old`（只把 30-2 对 `walk.rs` 的改动倒回去，用来比新旧判法同一批镜像）、`tree-secs`（只把 core 的 T_time 倒回写 5，用来取旧字节）。
- 冻结副本的 sha256 清单对照：`sha256sum -c` 不符 0 处（`rerun.sh setup` 第一行 RESULT）。
- 线程上限 7，cargo 命令都经 `run-with-memory-cap.sh 8G`；`ps` 开跑前看到别的会话的 cargo 在跑，没有性能测量进程。

## M1　30-1：第 4 片与回退用例

### M1-a　三条用例对四个变体：一致

判决点名的东西（B1、B2）：`research/prompts/m3-formal-code-b29-r1-main-verification.md:75` 「攻方三条换点用例（建到第 4 片；目标带第 3 片且现行重写过它的回退；目标没有第 4 片的回退）入库，四个变体（胶水封顶第 3 片、模型封顶第 3 片、回退只取回第 2 片、回退只丢第 3 片）各一行变异，逐条证红（B1、B2）」。

- 基线：三条用例在冻结副本上全绿（`out/m1-baseline-new-tests.txt`：`test result: ok. 3 passed; 0 failed`）。
- 四行变异各自施加、跑新文件三条用例（`out/m1-m2-m3-mutation-results.tsv` 的行 0–3；每行锚点在 `crates/singlefs-harness/src/model.rs` 或 `model_comparison.rs` 里恰好命中 1 次，`mutants.py anchors` 的 `NOT_ONE 0`）：

| 变异行（`crates/mutations.tsv` 行号） | 红的用例 |
|---|---|
| 胶水封顶第 3 片（2717） | 建到第 4 片、回退到没有第 4 片 |
| 模型封顶第 3 片（2718） | 同上两条 |
| 回退只取回第 2 片（2719） | 第 3 片被重写过的回退、没有第 4 片的回退 |
| 回退只丢第 3 片（2720） | 没有第 4 片的回退（只这一条） |

- 把这四行的 `cargo test` 结局喂给 `scripts/crates-mutation-rows.py` 的 `verdict_of_run`（本模型 `rerun.sh m4-keys`）：四行都判 `caught`（9 行里 2717–2723、2635、2733 都 `caught`，2636 行见 M4）。
- 变异表行文：`crates/mutations.tsv:2717` 「O1 跨叶容器（胶水）：第 4 片及以后都当第 3 片（第 29 批攻方，判决 L1 第三行）」；`crates/mutations.tsv:2720` 「O1 跨叶容器回退（理想模型自身）：回退只丢现行那一版的第 3 片、更右的留着（第 29 批攻方，判决 L1 第三行）」。

推翻它的现象：把这四行里任一行施加后新文件三条用例全绿；或 `anchors` 报某行原文命中数不是 1。

### M1-b　换取样点：四个变体还红（一致）；再往外一档的三个新变体活下来（冲突）

回退网格探针（`probes/b30_m1_rollback_grid.rs`，照 b29 正推腿同名探针；格写成 建几个:回退到根环第几新:回退后再建几个，先 unlink 第一个文件）。每格：基线 `Completed`；变体下 `NewFinding`（红）或 `Completed`（没抓到）。`out/m1-grids.txt` 是原样输出。

- 基线 35 次（31 个不同的格）全 `Completed`（含第 5 片：建 933、934、935、936、940 个；回退到根环第 1–6、8、9、12、16、40、300 新；回退后建 0、1、3、10 个）。
- 胶水封顶第 3 片、模型封顶第 3 片（行 0、1）：`700:2:3`、`702:2:3`、`703:1/2/3/4/6:3` 这 7 格全红，红在第 700 步（分出第 4 片那一步）。
- 回退只取回第 2 片（行 2）：7 格全红（含回退到更旧的根 `703:4`、`703:6`；回退后不建 `701:2:0`、多建 `701:2:10` 都红在回退那一步）。
- 回退只丢第 3 片（行 3）：红在「目标没有第 4 片」的格（`700:2`、`703:4`、`703:6`、`701:2:*`），目标带第 4 片的格（`702:2`、`703:1/2/3`）和第 3 片被重写过的格（`471:*`）`Completed`。这与它的定义一致（现行有第 4 片、目标没有时才露），新用例三正好落在红的那一类。
- 回退之后建几个不分辨：红都落在回退那一步（`701:2:0` 与 `701:2:10` 同一步红），所以用例里回退之后再建 3 个是多余的、不是承重的。

再往外一档，腿自加的零轮变体（`probes` 之外的 `data/extra.tsv`，由 `tools/make_extra.py` 从行 0–3 派生，不入变异表）：

| 变体 | 新文件三条用例 | 宽测试集（133 个用例：`--lib`、`random_histories`、`unlink_in_histories_judged_by_the_model`、旧两条模型对拍、`reference_filesystem_translation_of_histories` 等） | 哪格能抓到 |
|---|---|---|---|
| 胶水封顶第 4 片（`.min(3)`）（100） | 3 个全过 | 133 个全过 | 建到第 5 片：`933:2:3` 起每格红在第 933 步 |
| 模型封顶第 4 片（101） | 3 个全过 | `--lib` 的 `the_inode_leaf_container_position_reaches_the_last_of_135_leaf_containers_and_panics_one_record_beyond` 红 | 已被现有用例抓到 |
| 回退只取回第 2、3 片（102） | 3 个全过 | 133 个全过 | 目标带被重写过的第 4 片：`703:1:3`、`703:2:3` 红在回退那一步 |
| 回退只丢现行的第 3、4 片（103） | 3 个全过 | 133 个全过 | 现行带第 5 片、目标没有第 5 片：`934:2:3`、`935:3:3`、`936:5:3`、`933:2:3` 红 |
| 回退只丢现行的第 2、3 片（104） | 红（没有第 4 片的回退那条） | — | 已被新用例抓到 |

对应判定：对着判决 B1、B2 写的字面（三条用例、四个变体）一致；对着它要证的行为「第 k 片都对」冲突：今天只罩到 k = 4 的新建，回退两行只罩到「现行第 4 片、目标没有它」与「目标第 3 片被重写过」，第 5 片的回退与「目标带被重写过的第 4 片」没有用例（这与 b29 判决 L1 第三行同形：它写的「只罩到 k = 3」现在是「只罩到 k = 4」）。网格探针证明这些格在 CPU 侧用同一个入口 `execute_history_with` 做得出来、每格 30 秒左右，不是造不出的流。变体 102、103 的下一格要多 1 个到 2 个回退用例，变体 100 要建到 933 个。零轮，主 agent 现查坐实。

推翻它的现象：变体 100、102、103 在入库三条用例之外的某个现有用例上红（我跑的宽测试集 133 个都过；漏掉的测试二进制在我没列的 harness 档文件里时，这一条要改）。

### M1-c　四行变异的锚点与 KX1 / KX2 那几行：一致（不互相掩护）

- 同一原文、不同替换文的对子（`crates/mutations.tsv` 行号，锚点起行在冻结副本里现查）：胶水行 2717 与 2563（原文同一行 `crates/singlefs-harness/src/model_comparison.rs:80`，替换 `.min(2)` 对 `.min(1)`，还有 2461 的 `.min(0)`）；模型落点行 2718 与 2564、2460（原文同一行 `crates/singlefs-harness/src/model.rs:197`）；回退 retain 行 2720 与 2561（同一段三行，起 `model.rs:1719`）；2719 的 `.filter(...)` 三行起 `model.rs:1726`，与 2562 的 extend 三行（起 `model.rs:1722`）相邻、不重叠。
- 一行改坏的是干净副本：变异驱动 `scripts/crates-mutation-rows.py` 的 `judge_row` 每条先读未改的文件，`crates/…` 原文必须命中恰好 1 次才改（`scripts/crates-mutation-rows.py:894` 「assert pristine.count(row.old) == 1, (row.path, row.name)」），改一处、跑点名的测试、写回原样。两行同原文不会一行把另一行的原文改没。
- 静态判（`crates-mutation-rows.py static-check`）：冻结副本 2728 条成形行、只有一行报 `ANCHOR`，是 `litmus/…` 那条（冻结副本不带 `litmus/`，与 b29 判决第四节说的同一条，不在这一轮的改动里）；新加的 4 行、30-2 的 4 行 CPU 与 9 行 GPU、30-3 的 2 行锚点都命中 1 次。
- 点名用例在输出里只许对上一个（`scripts/crates-mutation-rows.py:666` 「if len(results) > 1:」）：新文件三个用例名互不为子串（我逐个核过），两行同名同红也各自判。
- 实际红的是点名的那一条：行 2717、2718 点名的用例（建到第 4 片）红；行 2719 点名的用例红；行 2720 点名的用例红（`out/m1-m2-m3-mutation-results.tsv`，`verdict_of_run` 判 `caught`）。

推翻它的现象：某行的原文在施加另一行的改动之后才命中（驱动改成不还原时才会）；或 `verdict_of_run` 对其中一行判 `failure`。

## M2　30-2：E1、种类 ≥ 32、I-5.4 按版本、GPU 加密 / 压缩对齐

### M2-a　E1 补的那一判让合法历史误红吗：一致（没有）

改动：`crates/singlefs-checker/src/walk.rs:2065` 「judge_pointer_extent_offset_is_zero(&mut self.judgements, &child, &what);」，紧跟 `crates/singlefs-checker/src/walk.rs:2064` 的 MAC / nonce 判。第一版写者恒写 extent 偏移 0，所以合法历史上这一判恒成立；它只在盘上字节被改时红。

- 新旧判法同一批镜像逐项比：`tree`（新）与 `tree-old`（只把 30-2 的 `walk.rs` 改动倒回）各跑 `b30_m2_checker_sweep`（`probes/b30_m2_checker_sweep.rs`，照 b29 正推腿 L2 探针；每次 `check_pool_image` 另打一行）：种子 60–66 各 1 个、6 步，种子 70–82 各 2 个、8 步，各 8 种命名比重，每段边界与抽到的崩溃态；合计 7 309 次检查。`out/m2-sweep-old-vs-new-summary.txt`：新旧两份排序后的逐行文件 sha256 同为 `2f433faf3e515e385b29472c903107bef5dfb128f793bf79a6042ca218ef7fbd`；`I-2.4=Holds I-5.4=Holds violated=[]` 5 965 次、`I-2.4=Holds I-5.4=NotApplicable violated=[]` 1 344 次，没有一次红、没有一次 `PartiallyJudged`。
- 覆盖到的历史形：覆盖写 714 步、可写挂载 125、抬回退下界 116、挂载中回退 99、unlink 47（删除进行中那几态含墓碑树）、冷启动恢复 21、可写挂载每次取新的实例代号（多实例的态没有单独数，推的）。回退之后的谱系树、删除进行中的墓碑树都在里面；两棵树的容器子指针在这些态上 extent 偏移恒 0，新判恒成立。
- 已知坏镜像：`checker_known_bad_images` 89 个用例在冻结副本上全绿（`out/m2-checker-known-bad-baseline.txt`：`test result: ok. 89 passed`），含新加的两个 E1 用例（`crates/singlefs-harness/tests/checker_known_bad_images.rs:8556`、`:8582`）。变异行 4、5（原文都是删掉 E1 那一句）各被自己点名的用例抓到（`out/m1-m2-m3-mutation-results.tsv`）。
- 一处小事：E1 那一句之后，类型段 0 的内部节点孩子会被 `read_index_node` 再判一次（`crates/singlefs-checker/src/walk.rs:1271` 「judge_pointer_extent_offset_is_zero(&mut self.judgements, &pointer, what);」），同一条指针的 I-2.4 评估计两次。只多一次评估、不改判定类；今天 MAC / nonce 那一句在 2064 与 1270 也是两次，同形。

推翻它的现象：某个合法历史（任何 `check_pool_image` 的输入，写者是 core）上 I-2.4 报违例，原因是墓碑树 / 谱系树的容器子指针 extent 偏移非 0。我的 7 309 次没有一次。

### M2-b　墓碑树 / 谱系树之外，还有没有走读跟随却不判 extent 偏移的指针：一致（没有）；有一类读而不跟随的指针 I-2.4 那一行没说（规则没说）

条款：`.claude/kb/invariants.md:133` 「走读跟随的每条指针都判」。

读码（池级 checker，冻结副本 `crates/singlefs-checker/src/walk.rs`）：跟随指针的五个入口各判 MAC / nonce 与 extent 偏移——`read_index_node`（1256，判在 1270–1271）、容器树内部条目（2046，判在 2064–2065，E1）、inode 内部条目（2202，判在 2229 前后）、实例表链（2549，判在 2570）、extent 数据指针（2717，判在 2731）。树表条目根指针、映射树、分配记录树、extent 树的内部节点都经 `read_index_node`。GPU 走读内核 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 判指针头的入口也是五个（`w_judge_pointer_extent_offset` 的调用点 1482、1584、1769、2595、2929；2929 是 E1 那一处），一一对应。

实测（探针 `probes/b30_m2_pointer_scan_append.rs`：单元区里每条「像指针」的位置——连着两条位置条目、设备 0 与 1、槽号同、前 50 字节是 34 字节全零的头——把 extent 偏移改 1、MAC 第 0 字节改 1，各重封并沿引用链补校验和到根槽，跑 `check_pool_image`）：

| 镜像 | 指针数 | 两种改法都红 I-2.4 | 只红 MAC | 只红 extent | 都不红 |
|---|---|---|---|---|---|
| 覆盖写 / 回退 / 删除意图 / 删除第一批 / 三单元文件 / 两页实例表链 / 300 个 inode | 20 / 66 / 35 / 49 / 23 / 27 / 20 | 全部 | 0 | 0 | 0 |
| 正常卸载 | 56 | 26 | 0 | 0 | 30 |

- 探针会红（跑前证红）：`tree-old`（没有 E1）上同一探针在「删除意图」与「回退」两张镜像上各数到 1 个「只红 MAC、不红 extent」（`out/m2-pointer-scan-old-tree-e1-images.txt`：墓碑树容器指针单元 50280、谱系树容器指针单元 50354），新树上这两个数是 0。
- 「正常卸载」那 30 个都不红的：把那个单元整个清零（两份）再判，18 个单元的判定向量一点不变（死单元，没有任何一版走读读它）；其余 12 个是三个旧树表单元（槽 50252、50264、50276）里各 4 条树根指针，清零只让 I-9.28 变（`out/m2-pointer-scan-new-unmount-zeroing.txt` 的 `ZEROING … differing_invariants=["I-9.28"]`）。也就是池级 checker 为 I-9.28 读了这几个旧树表的条目（取诞生 txg）、没有走读它们、没有跟随这些树根指针。E1 之前、之后都如此，与 30-2 无关。

这 12 条：`.claude/kb/invariants.md:133` 写的是「走读跟随的每条指针都判」，I-9.28 读旧版树表条目不算「跟随」，条款没说它们要不要判。写「规则没说」；core 若把某条旧根当恢复目标，会经 `TreeTableEntry::parse` 全判（`crates/singlefs-core/src/records.rs:449`），那是 core 一侧，我没造这个盘面验证（推的）。

推翻它的现象：在 `rerun.sh m2-scan` 之外的任一镜像（带谱系树 + 删除进行中 + 多片叶容器 + 多实例）上，探针数到「只红 MAC」的指针。

### M2-c　GPU 恢复内核种类 ≥ 32 的逐条比，与 core 在同一盘面上结局是否逐格相同：一致（读码、跑了 core 一侧；GPU 一侧推的）

- core：`crates/singlefs-core/src/recovery.rs:2330` 「let mut kinds_seen: BTreeSet<u16> = BTreeSet::new();」，整个 u16 的集合，每条条目先 `TreeTableEntry::parse`、再 `kinds_seen.insert`，插不进就拒（`recovery.rs:2327` 起的 `tree_table_entries_each_kind_at_most_once`）。拒 = 恢复失败，这一版不可读。
- GPU 恢复内核（`crates/singlefs-checker-tier/src/crash_judge_common.wgsl`）：种类 < 32 走位图，≥ 32 走新加的循环 `crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2993` 「for (var earlier = 0u; earlier < e; earlier = earlier + 1u) {」与 `:2994` 「if (rd16(tree_table.base, entry_offset(tree_table, earlier) + 10u) == kind) {」，同种类就 `walk_failed(24u)`。≥ 32 的种类不可能等于 < 32 的，所以「某个 u16 值出现两次」这一个谓词被两半位图 / 循环分拿、合起来与 core 的集合插入等价；两边都只在循环里返回失败、不依赖条目次序（条目次序只影响诊断码：解不开 23 / 65 / 63，重复 24，`walk_failed` 的 code 只当诊断，`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:2786` 起，outcome 恒 `OUTCOME_FAILED`）。
- core 一侧实测（`probes/b30_m2_kind_append.rs`，b29 正推腿 L4 探针原样在这棵树上再跑；`out/m2-kind.txt`）：树表里两条同种类 7、13、31、32、40 五种取值，池级 checker 红 I-9.16（「种类 40 有 2 条（③ 每种至多一条）」），`recover` 都 `Failed … UnitMalformed`（树表里同一种树有两条）。种类 ≥ 32 的两种取值（32、40）core 都拒，所以 GPU 那一半补的就是 core 实际拒的集合。
- GPU 一侧：对拍用例与变异行 9 没跑（checker 档）；读码推：变异行 9 把比较换成 `if (false)` 后，种类 40 两条的树表在恢复内核里不被拒，要读下去，与 core 的拒不同，对拍用例该红（推的）。

推翻它的现象：GPU 对拍里 `…tree_table_carries_two_entries_of_a_kind_not_below_thirty_two` 红，或 `walk_to_file` 在种类 ≥ 32 的重复上有别的出口先于它（读码：`crash_judge_common.wgsl` 里 2976 行起到该循环结束之间没有）。

### M2-d　I-5.4 按版本：合法历史上不多出「部分没判」（一致）；坏盘上不放过（一致）

改动（`crates/singlefs-checker/src/walk.rs`）：`:4901` 「let mut allocation_roots_walked_to_their_leaves: BTreeSet<(u32, u64, u32)> = BTreeSet::new();」按分配记录树根记走到叶的；`:4987` 「if *this_version_allocation_record_tree_walked_to_its_leaves {」逐版定计不计。判决依据 `research/prompts/m3-formal-code-b29-r1-main-verification.md:40` 「不新增违例，并进第 30 批」。

- 合法历史：§M2-a 同一批 7 309 次检查，新旧 I-5.4 逐次相同（5 965 次 Holds、1 344 次 NotApplicable，0 次 PartiallyJudged），含抬回退下界 116 步、挂载中回退 99 步。
- 坏盘，判法只会变严：`probes/b30_m2_i54_units_append.rs`（照 b29 正推腿 L2 探针：覆盖写之后 A、B 两版，把一个单元的两份副本清零，共 26 个单元，`out/m2-i54-units-old.txt` 与 `…-new.txt`）。旧树 27 行（含 clean）全是 `Holds`；新树 17 行 `Holds`、10 行 `PartiallyJudged`，其余字段（`violated=[…]`）一格不差。变的 10 个单元：槽 50257–50261 正是最新一版分配记录树的 5 个节点（探针另打了 `newest_version_allocation_record_tree_node_slots=[50261, 50260, 50258, 50259, 50257]`），槽 50245–50249 是上一版自己的 5 个节点（按同一排布推的，没单独列它的节点表）。b29 判决那一行写的槽 50257 到 50264 里 50262–50264 不是分配记录树节点，清零它们 I-5.4 仍 `Holds` 是对的。
- 红的方向没丢：`checker_known_bad_images` 里 I-5.4 的违例用例全绿（89 个，`out/m2-checker-known-bad-baseline.txt`）；变异行 6 与行 16（互为相反的两种条件）各被自己点名的用例抓到。
- GPU 一侧不需要跟：扫描内核里 `INVARIANT_I_5_4` 只有两处用法，都是 `violate(INVARIANT_I_5_4)`（`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl:889`、`:919`），没有「成立 / 不适用 / 部分没判」那几类的输出，新旧两种计法对 GPU 的红位没有区别（对拍比的是红位，读码推的）。

推翻它的现象：一个合法历史上某条候选根「带树、根记录里分配记录树根全零」而它自己的分配记录树读不出（被合法回收），旧判法 `Holds`、新判法 `PartiallyJudged`。7 309 次里没有；这种态要回收窗口之外的根进候选集才会有，候选集定义在 b26 判决 J1，我没改它。

### M2-e　GPU 加密 / 压缩 8 处与 core 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from`：同一组字段、同一判法（一致，读码；字段谓词跑了翻译）

core 的判（`crates/singlefs-core/src/pointer.rs:210` 起的 `PointerHeadAsRead::judged`）：头部 34 字节里 MAC 16 + nonce 12 全 0（`crates/singlefs-core/src/pointer.rs:191` 「let is_every_mac_and_nonce_byte_zero = reader.take(16 + 12).iter().all(|byte| *byte == 0);」）、算法类型 u8 为 0、压缩算法码 u8 为 0、压后长度 u16 为 0，再判 extent 偏移 u16 为 0。GPU：`crates/singlefs-checker-tier/src/crash_judge_common.wgsl:512` 「return all_zero(base, o, 28u) && rd8(base, o + 28u) == 0u && rd8(base, o + 29u) == 0u && rd16(base, o + 30u) == 0u;」加另判的 `pointer_extent_offset_is_zero`（偏移 32 的 2 字节）。字段、偏移、宽度逐项对上。

| 核心里全判的读者 | GPU 恢复内核的位置 |
|---|---|
| 树表条目根指针 `records.rs:449` | `crash_judge_common.wgsl:2976`（`walk_failed(65u)`） |
| inode 内部条目子指针 `records.rs:503` | `:3231`（`walk_failed(66u)`） |
| extent 叶记录数据指针 `records.rs:523` | `:2547` |
| extent 上段叶：下段根指针 `extent_tree.rs:363` / 内联数据指针 `:370` | `:2389` / `:2398` |
| extent 内部条目子指针 `extent_tree.rs:735` | `:2434`、`:2586`（辅助函数 `:2284`） |
| 分配记录树内部条目子指针 `allocation_record_tree.rs:777` | `:2232` |
| 实例表链指针 `instance_table.rs:199`、`:252` | `:2736` |
| （原有）码 2 内部条目 `code_two_tree.rs:154`、journal 新根段 `journal.rs:324–341` | `:1949`、`:1003`（`record_pointer_head_is_the_first_version`） |
| （只判 extent 偏移）根记录 `root_record.rs:146–153` | `root_record_pointers_carry_no_extent_offset`（`:922`） |

- 8 处新补（上表前 7 行展开 = 树表、inode 内部、extent 叶、上段两种、extent 内部（上下段各调一次）、分配记录树、实例表链）每一处都紧贴原有的 extent 偏移判、同一个作用域、同一个返回口，所以「判几条条目」与 b29 判决认过的 extent 偏移那一半相同（core 的 `upper_leaf_entries`、`parse_extent_record` 一路对节点里每条条目都解）。
- 谓词翻译核对（`probes/b30_m2_pointer_head_predicate.rs`，把 WGSL 两个谓词逐字译成 Rust，与 `NodePointer` / `DataPointer` 的 `read_judging_the_encryption_compression_and_extent_offset_fields_from` 比）：头部 34 字节里每个位置改成每个非 0 值（34 × 255）加 20 000 个随机组合，共 28 671 个头，两边接受与否逐个相同（`B30HEADS checked=28671 disagreements=0`）。翻译是我照 WGSL 原文写的，WGSL 本身这里没跑，写「推的」。
- 没有 GPU 拒而 core 收（或反过来）的一处：core 的影子账读者（`PointerHeadFieldsJudgement::NotJudgedByTheShadowLedger`，`instance_table.rs`、`allocation_record_tree.rs:860` 起、`recovery.rs:2251` 起）刻意不判，GPU 没有影子账那条路（`crash_judge_*.wgsl` 里 `影子`、`shadow` 零命中），不存在对应的差。
- 没变异行也没对拍的两处（主 agent 观测的「造不出流」）：extent 内部条目子指针（`:2434`、`:2586`）与实例表链指针（`:2736`）。读码是同一类谓词；我的字段谓词核对覆盖它们的判法，不覆盖「GPU 在这两处真的接了线」，那一点只能靠读码。

推翻它的现象：GPU 对拍里 6 条 MAC 第 0 字节的坏流（对 6 处各一）有一条不红，或有一处 core 全判的读者在 GPU 恢复内核里没有对应位置（上表之外）。

### M2-f　复核不了的

- GPU 对拍新加的 9 条、GPU 变异 9 行（行 7–15）：checker 档，没跑；读码结论都标了推的。
- 池级 checker 与 GPU 走读内核在 E1 那一处对拍（`…tombstone_container_pointer_carries_an_extent_offset`、`…lineage_…`）：没跑；CPU 一侧跑了（M2-a）。

## M3　30-3：T_time 改毫秒

### M3-a　core 写出、读取、G1 期望、字段表第 149 行：同一个单位、同一个值（一致）

- 字段表：`.claude/kb/layout/01-first-txn.md:149` 「| 可调值 | T_time | 4 | 5000（单位毫秒，即 5 秒） | D16（发布语义） 已定项 5 |」。
- core 写：`crates/singlefs-core/src/system_configuration.rs:86` 「const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;」，槽里写出处 `crates/singlefs-core/src/system_configuration.rs:523` 「writer.put_u32(self.runtime.time_threshold_milliseconds());」；偏移 433、4 字节（偏移累加：槽距 429 起 4 字节，之后 433，`.claude/kb/layout/01-first-txn.md:158` 的偏移说明里「T_time 433」）。
- core 读：没有。`grep -rniE 'time_threshold|TIME_THRESHOLD' crates`（不含变异表）全仓 7 处（core 4 处全是常量、取值方法与写盘那一行，harness 3 处全是 G1 期望），读盘的地方零命中；`crates/singlefs-core/src/system_configuration.rs:280` 「所以这一档拿内置默认值写盘，不带状态。」。池级 checker 与恢复读系统配置槽的偏移清单里没有 433（见 M3-d 的实测）。所以「读取」这一半今天不存在，不是读错单位。
- G1 期望：`crates/singlefs-harness/src/mount_transparency.rs:1359` 「const BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES: u32 = 5000;」，字节来自 `:1372` 「bytes.extend_from_slice(&BUILT_IN_TIME_THRESHOLD_MILLISECONDS_OF_THE_CLAUSES.to_le_bytes());」，不取 core 的常量（照条款写死）。
- 实测三处对上：冻结副本上 `mount_transparency_judged_field_by_field_after_a_writable_mount`（48 个）、`system_configuration_mutability_classes`（2 个）、`unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over`（24 个）全绿（`out/m3-three-pins-baseline.txt`）；G1 判官在改坏的 core 上红，红的原样输出里期望是 `[136, 19, 0, 0, …]`（5000 的小端），观察到 `[5, 0, 0, 0, …]`（退回秒）。变异行 17（退回秒）与行 18（5000 改 4000）都被点名的用例抓到，`verdict_of_run` 都判 `caught`。
- 与判决、定案对：`research/prompts/m3-formal-code-b29-r1-main-verification.md:77` 「core 系统配置写出与读取 T_time 的单位改成毫秒、默认 5000，G1 期望常量照改，受影响的钉值重取（用户 2026-10-05 定）」；`research/prompts/m3-formal-code-b29-r1-main-verification.md:41` 「用户 2026-10-05 定改成毫秒」。

推翻它的现象：`mount_transparency_…_after_a_writable_mount` 里有一个用例在冻结副本上红；或字段表第 149 行与 core 的值不是同一个（5000 对 5000）。

### M3-b　全仓还有没有按秒解这 4 字节的地方

- `crates/`：零（见 M3-a：没有读者）。
- `research/`（`rerun.sh m3-greps` 原样输出）：没有按秒「判」它的读者。有两类要说：
  - E142 装置写系统配置槽，自己写这个字段：主仓工作区的 `scripts/experiments/e142_new_pool_file_creation_dry_run.rs:1524` 「writer.put_u32(5000); // T_time 毫秒」；HEAD 里同一处是 `put_u32(5); // T_time 秒`（`git show HEAD:…` 现查，第 1494 行）。也就是 30-3 报告第七节说「E142 会与 core 对不上」那一条在今天工作区已经改过了（这一份文件还带着别的未提交改动，`git status` 是 `MM`），冻结副本不含 `research/`，所以 E142 是否与新 core 逐字对上我没跑（它是 research 的跑法，门禁 `E142` 行输入含 `crates/`，要重跑）。
  - E170 五个 O3 规约执行器：`on_disk_layout.py` 按偏移 433、宽 4 解出 `checkpoint_time_threshold`，只解不判；五个目录的 `synthetic_pool_for_tests.py` 造合成池写 5。合成池的 5 与 core 今天的 5000 不同，但没有任何地方拿它与 core 比（grep 零命中），换不换由实验那边定。
  - 其余命中（e108、e91、e159、e97、e70、e87）是把 T_time 当模型参数用，不读盘。
- 规则没说的：D16 已定项 5 的用户可设（`.claude/kb/decisions/16-发布语义.md:140` 「**`T_time` 用户可设，不限制**：挂载时传的值覆盖盘上的字段并回写」）与 4 字节毫秒字段：u32 毫秒最大约 49.7 天，「不限制」写成毫秒时有一个隐含的上限，条款没写它（推的，今天没有设值的路）。

推翻它的现象：`crates/` 或 `research/` 里有一处读盘上偏移 433 的 4 字节、按秒比较或换算（我 grep 的覆盖：`T_time`、`time_threshold`、`TIME_THRESHOLD`、`\b433\b`、`checkpoint_time_threshold`、`put_u32(5)`）。

### M3-c　三个新钉值的出处站得住（一致，两条独立的路）

三个钉值：`crates/singlefs-harness/tests/system_configuration_mutability_classes.rs:92` 起的两份摘要（`e5cc28ab…`、`144c1785…`）、`crates/singlefs-harness/tests/unit_area_start_follows_the_ring_and_unreadable_reads_are_not_passed_over.rs:314` 的 `1_329_175_690`。

- 路一（入库版直接现算）：冻结副本上这三条断言全绿（M3-a）。
- 路二（旧字节改一处、不经 core 的校验和函数）：`tree-secs`（core 倒回写 5）上取旧槽字节与旧整盘 CRC，再算新值：
  - 摘要：旧版两份样本槽的 sha256 是 `a065c124…`、`c67464f9…`（即 b29 树里的旧钉值）；新旧槽逐字节只差 6 个位置（偏移 433、434 与整槽校验和的 4 字节，`out/m3-slot-check.txt`：`differing_byte_positions=6 only_433_437_and_155_159=True`）；用纯 Python 的 CRC-32C（`tools/slot_check.py`，不经 core 的 Rust 函数）把旧槽改 5000、字段 [155, 187) 按 0 参与重算，得到的 sha256 与新钉值逐位相同，且新槽里存的校验和也被 Python 算对（`SLOTCHECK_RESULT OK`）。
  - 整盘 CRC：旧版现算 `4048568891`（旧钉值）；把每块盘每个系统配置槽偏移 [433, 437) 写成 5000、重封，再算，得 `1329175690`（新钉值）；入库版直接现算也是 `1329175690`（`out/m3-images-old.txt`、`out/m3-images-new.txt`）。
- 附带：新旧两版的写过的扇区集合相同（1 172 个扇区），逐扇区比只差 4 个（盘 0 与盘 1 的扇区 0 和 8，即两个槽的第一个扇区，`B30SECTORS … changed_sectors=[(0, 0), (0, 8), (1, 0), (1, 8)]`）。

推翻它的现象：用另一个 CRC-32C 实现重算的新槽摘要与钉值不同；或旧版现算的整盘 CRC 不是 `4048568891`。

### M3-d　checker 档读码判不变的那些钉值，判得对不对：对（读码一致；新值复核不了）

30-3 报告第六节（`/tmp/claude-1000/m3-formal-30-3/report.md`）的两半：

- 「会变的」：`crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs:844`、`:850`、`:851` 三个常量。依据 `crates/singlefs-checker-tier/src/crash.rs:2571` 的 `layer0_plan_hash`：它把基镜像每个写过的扇区的内容与每次写的内容都算进去（`for (sector, bytes) in sparse_device.written_sectors() {` 与 `WrittenContents::Bytes(bytes) => {` 两段，读 `crash.rs:2591`–`2604` 一带）。基镜像与写表里都有系统配置槽的扇区，槽的 [433, 437) 与整槽校验和都变了，计划哈希变，进度文件名与 `LAYER0_RESUME` 行里印的名字跟着变。读码一致。新值要在 checker 档跑出来：复核不了。
- 「读码判不变的」：
  - 池级 checker 真不读偏移 433：实测（`probes/b30_m3_checker_433_append.rs`）覆盖写 / 回退 / 删除意图 / 正常卸载四张镜像，每块盘两个槽一起把 [433, 437) 写成 0、1、5、200、4000、5000、4294967295，整槽重封，整份判定向量（Debug 串）与没改的逐字相同，28 次全 `true`（`out/m3-checker-433.txt`）。池级 checker 的红绿、判定集合不依赖这 4 字节。
  - 状态数、段数这一类钉值（`crash_enumeration_*` 一族）：写过的扇区集合不变，新旧只差 4 个槽扇区（M3-c 附带），这 4 个扇区新旧版里本来就因槽世代号与整槽校验和不同、T_time 在两版里同值，所以「原地覆写新旧不同的扇区」「撕裂态的差异跨度」都不变。依据是读码加这一个 CPU 侧的扇区比；这些钉值本身在 checker 档，我没跑：推的。
  - `crash_identity.rs` 里钉的摘要只对内联小字符串求摘要（`crates/singlefs-checker-tier/src/crash_identity.rs:1254` 起），与盘上字节无关：读码对。
- 吃整份系统配置槽、30-3 报告没单列的两样：层 0 进度文件 / 崩溃放量判定库里按镜像字节或计划哈希做键的缓存条目会整体换键（30-3 报告已写「会失效的缓存，推的」）；`crash_identity` 对被覆盖的源码文件取词法摘要，改 `system_configuration.rs` 会换所有包含它的节点身份（缓存键，不是钉值）。这两样不影响红绿，只影响复用。

推翻它的现象：checker 档里除 `crash_enumeration_sharded_across_processes.rs` 那 3 个常量外，还有别的用例因为系统配置字节变了而红（30-3 报告自己写的推翻条件），或池级 checker 的判定向量因 T_time 改变。

## M4　主 agent 两处

### M4-a　变异表那一行改名，会不会让按名字复用的判定记录对不上：结局一致；「锚点与替换文不动」这句与树里的行冲突（冲突）

- 判定记录不按名字复用：`scripts/crates-mutation-rows.py:432` 「text = "\t".join(row.fields) + f"\nmemory_max={memory_max}\ntimeout_seconds={timeout_seconds}\n"」，行键是这一行六段原样（名字是第一段）加上限的 sha256；用时表按六段原样的 sha256 记。所以改名、改锚点、改替换文任一样都是新行键，旧记录不会被复用，也不会被错用，下一次重放这一行当新行跑。
- 主 agent 在改动全文第 3 行写「锚点与替换文不动」（`research/prompts/_m3-formal-code-b30-r1-diff.md` 开头）。对照：b29 树里的那一行（`research/prompts/_m3-formal-code-b29-r1-diff.md` 里的 MUT-A 行）锚点是 `const TIME_THRESHOLD_SECONDS: u32 = 5;`、替换文 `… = 4;`；入树现行 `crates/mutations.tsv:2636` 是 `const TIME_THRESHOLD_MILLISECONDS: u32 = 5000;` 改 `… = 4000;`。名字、锚点、替换文三段都变了（常量改名不得不变）。「不动」只相对 30-3 实现员补丁里那一行成立（补丁行已经带新锚点）。
- 三个行键（`rerun.sh m4-keys`，上限 8G、900 秒）：b29 行 `61266eba493f`、补丁原样（新锚点、旧名）`7400b43507af`、入树现行 `dc5d22c6d891`，互不相同。
- 现行行 2636 在冻结副本上 `verdict_of_run` 判 `caught`（`rerun.sh m3-rows` 行 18）。
- 结论：不会对不上；这句话写成「锚点与替换文随常量改名而变」才对。

推翻它的现象：重放驱动里另有一处按行名（第一段）而不是行键挑旧记录。我读了 `crates-mutation-rows.py` 里 `row_key`、`row_content_hash`、`reusable_record`，没有按名字的地方。

### M4-b　字段表第 149 行与 D16 已定项 5 的定案句：数对得上（一致）；单位出处没落在 D16 里（规则没说）

- 数：`.claude/kb/decisions/16-发布语义.md:140` 「`T_time` 默认值 5 s，取可行区间 [200 ms, 5 s] 的上端」；字段表 5000 毫秒 = 5 秒，区间下端 200 毫秒在毫秒里是整数 200、写得出。
- 条款没说的：字段表的出处列写「D16（发布语义） 已定项 5」，而 D16 已定项 5 的正文没有「盘上单位毫秒」这一句（我读了 `.claude/kb/decisions/16-发布语义.md` 的 138–170 行）；单位的出处只有用户 2026-10-05 的弹窗定案，记在 b29 判决第 41 行。字段表第 149 行自己写了「单位毫秒」，这一行是单位的唯一落点。写回批若要让 D16 单独读也知道，D16 已定项 5 需要一句指到字段表。
- 偏移说明（`.claude/kb/layout/01-first-txn.md:158`）只给偏移 433，没有单位，不冲突。

## 判定一览

| 格 | 判定 | 一句话 |
|---|---|---|
| M1-a 三条用例对四个变体 | 一致 | 四行变异各被点名的用例抓到，`verdict_of_run` 判 `caught`；基线三条全绿 |
| M1-b 取样点：第 5 片及以后、更旧的根、回退后不建 / 多建 | 四变体一致；再外一档冲突 | 四变体在各自定义该红的格上都红（格表见 `out/m1-grids.txt`，红在回退那一步，回退后建几个不分辨）；腿加的零轮变体 100、102、103 在三条用例和 133 个宽测试集用例上全过，只有 5 片的格与「目标带被重写过的第 4 片」的格抓得到 |
| M1-c 锚点互相掩护 | 一致 | 同原文不同替换文的对子各自在干净副本上施加，静态判锚点都命中 1 次，用例名互不为子串 |
| M2-a E1 误红 | 一致 | 7 309 次检查（含删除进行中、回退、可写挂载）新旧逐行相同，0 红 |
| M2-b 其它走读跟随却不判 extent 偏移的指针 | 一致；12 条旧树表根指针规则没说 | 8 张镜像 296 条指针，266 条两种改法都红，30 条不红（18 死单元、12 条只被 I-9.28 读的旧树表条目）；探针在无 E1 的树上数到 2 个缺口、新树 0 个 |
| M2-c GPU 种类 ≥ 32 | 一致（推的） | core 对 7、13、31、32、40 都拒，GPU 补的循环覆盖 ≥ 32，等价于整个 u16 的集合；GPU 一侧没跑 |
| M2-d I-5.4 按版本 | 一致 | 合法历史新旧逐次相同；26 个单元清零里 10 个由 `Holds` 变 `PartiallyJudged`，其余字段不变；GPU 只有红位，不用跟 |
| M2-e 加密 / 压缩 8 处 | 一致（推的） | 字段、偏移、宽度、判法与 core 逐处对上，谓词翻译 28 671 个头 0 分歧；没有 GPU 拒而 core 收的一处；两处没变异行没对拍 |
| M3-a 单位与值三处 | 一致 | core 写 5000 毫秒、G1 期望 5000、字段表 5000（单位毫秒）；core 没有读者 |
| M3-b 全仓按秒解 | 一致（`crates/`）；E142 / E170 另说；不限制的上限规则没说 | `crates/` 零读者；E142 装置工作区已写 5000；E170 五份合成池写 5 只解不判；u32 毫秒约 49.7 天上限条款没写 |
| M3-c 三个新钉值 | 一致 | 两条路（入库版现算、旧字节改 5000 用独立 CRC 重算）都得同一个值 |
| M3-d checker 档读码判不变的钉值 | 一致（读码）；新值复核不了 | 池级 checker 不读 433（28 次判定向量逐字相同）；会变的是 `crash_enumeration_sharded_across_processes.rs` 那 3 个常量，要在 checker 档重取 |
| M4-a 改名与判定记录 | 结局一致；「锚点与替换文不动」冲突 | 行键是六段加上限的哈希，名字、锚点、替换文三段都变了，三个行键互不相同；没有按名字挑记录的地方 |
| M4-b 第 149 行与 D16 | 数一致；单位出处规则没说 | 5 秒 = 5000 毫秒，200 毫秒写得出；D16 已定项 5 正文没有盘上单位那一句 |

## 要主 agent 跑的命令

没有必须由主 agent 起的长活：我跑到的都在 harness 档，单段不超过 4 分钟。复跑要在后台起时：

```
REPO=$PWD FROZEN=/tmp/claude-1000/m3-formal-code-b30-r1/tree WORK=<会话草稿>/b30-sonnet-rerun RERUN_ALREADY_WRAPPED=1 \
  bash scripts/run-with-memory-cap.sh 8G bash scripts/capped.sh 7 \
  bash research/prompts/m3-formal-code-b30-r1-sonnet-model/tools/rerun.sh all
```

逐段预期读数（原样输出在 `research/prompts/m3-formal-code-b30-r1-sonnet-model/out/`）：`setup` `sha_not_ok=0`；`anchors` `NOT_ONE 0` 与只有 `litmus/` 那条 `ANCHOR`（退 1）；`m1-rows` 行 0–3 `CAUGHT`；`m1-extra` 行 100–103 `SURVIVED`、104 `CAUGHT`；`m1-extra-wide` 行 100、102、103 `SURVIVED passed=133`、101 `CAUGHT`；`m1-grid` 见 `out/m1-grids.txt`；`m2-rows` 行 4、5、6、16 `CAUGHT`、`checker_known_bad_images` 89 个全绿；`m2-sweep` 两次 `SWEEP_OLD_NEW_IDENTICAL`、sha256 `2f433faf…7fbd`；`m2-scan` 新树 8 个镜像的 `B30SCAN` 行与旧树两张镜像各 1 个 `only_mac_red=1`；`m2-heads` `disagreements=0`；`m2-kind` 五种种类都 `violated=["I-9.16"]` 与 `Failed`；`m2-i54` `diff 行数 10`；`m3-pins` 三个钉值测试绿、`SLOTCHECK_RESULT OK`、整盘 CRC 旧 `4048568891` 新 `1329175690`；`m3-433` 28 行 `identical_to_clean=true`；`m3-rows` 行 17、18 `CAUGHT`；`m4-keys` 三个行键 `61266eba493f`、`7400b43507af`、`dc5d22c6d891`。`rerun.sh` 我完整手工跑过每一段的同样命令；脚本本身只整段试跑了 `setup`、`anchors`、`m2-heads`、`m3-greps`、`m3-pins`、`m2-i54`、`m4-keys`（其余段的命令与我手工跑的逐字相同，没有整脚本跑过一遍）。

## 没做什么

- 不判别的腿的格：攻方腿的反例没做；不替主 agent 采纳、不出判决。我提的变体 100、102、103 与 `rerun.sh` 里的探针算零轮。
- 没跑：checker 档的任何测试（含 `crash_enumeration_sharded_across_processes.rs` 那 3 个常量的重取）、GPU 对拍与 GPU 变异 9 行、`crates/mutations.tsv` 整表、E142 重放。涉及它们的结论都写了推的。
- 没造的盘面：core 把旧根当恢复目标时全判旧树表条目指针头（M2-b 末句）；多实例的态没有单独数（M2-a）。
- 没改仓内任何 `crates/`、kb 或脚本；变异与探针都在草稿副本里施加后还原。模型目录 `research/prompts/m3-formal-code-b30-r1-sonnet-model/` 里 `data/walk-30-2.patch` 是从改动全文抽出的 30-2 对 `walk.rs` 的那一份，`data/extra-wide.tsv` 是把 `data/extra.tsv` 的测试集列换成宽测试集得到（没有单独的生成脚本）。
- 草稿目录 `/tmp/claude-1000/m3-formal-code-b30-r1-sonnet/` 里自己建的三棵树 `tree`、`tree-old`、`tree-secs`、编译目录 `target`、`target-old`、`target-secs` 与试跑目录 `rerun-test` 交回前已删（复跑时 `rerun.sh setup` 重建，不需要留）；留着的只有原样输出 `out/`、`mutlog/` 与排序后的检查行文件。

## cite-check 末行

命令：`python3 scripts/cite-check.py research/prompts/m3-formal-code-b30-r1-sonnet-output.md --root /tmp/claude-1000/m3-formal-code-b30-r1/tree --root . --background research/prompts/_m3-formal-code-b30-r1-background.md`

  ✓ 核了 24 处引文，对上 24 处，没判 0 处（逐处列在上面）
