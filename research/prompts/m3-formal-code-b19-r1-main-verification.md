# 第 19 批代码轮第一轮判决（2026-10-04）

<!-- doc-lint:not-numbers K1 K2 K3 H1 H2 -->

被判的文件：`crates/singlefs-checker/src/walk.rs`、`crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`、`crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`、`crates/singlefs-harness/tests/checker_known_bad_images.rs`、`crates/singlefs-harness/tests/pool_checker_judges_tree_table_head_identifiers_and_the_first_tree_building_placement.rs`、`crates/singlefs-checker-tier/src/bin/e165_replay_bounded_enumeration.rs`、`crates/singlefs-checker-tier/src/bin/e171_three_oracle_collusion.rs`，`crates/mutations.tsv` 19-1 追加的 12 行。

## 一、这一轮的材料与证据

| 物 | 路径 | 核法 |
|---|---|---|
| 正文 / 改动全文 / 背景材料 | `research/prompts/_m3-formal-code-b19-r1-body.md`、`_m3-formal-code-b19-r1-diff.md`、`_m3-formal-code-b19-r1-background.md` | — |
| 开工快照 | `research/prompts/m3-formal-code-b19-r1-snapshot/`（crates 275 份，冻结副本 `/tmp/claude-1000/m3-formal-code-b19-r1/tree/`；kb 5 份） | 主仓 crates 里 16 份已被根环那条线第 9 批与删除与并发那条线合入改过（`crash_judge_dispatch.rs`、`crash_judge_gpu.rs`、`mount_transparency.rs`、`allocator.rs`、`mount.rs` 等），不倒推，改对冻结副本核：冻结副本 `sha256sum -c` 对 crates 快照全过；kb 5 份在主仓里对快照全过 |
| 云端正推（Sonnet） | `research/prompts/m3-formal-code-b19-r1-sonnet-output.md`（sha256 e4deab82…），模型 `…-sonnet-model/` | 引文 |
| 云端攻方（Opus） | `research/prompts/m3-formal-code-b19-r1-opus-output.md`（sha256 bb25fbeb…），模型 `…-opus-model/` | 主 agent 照它的 `tools/rerun.sh` 在会话草稿 `b19-opus-rerun/` 复跑，见第四节 |
| 引文 | 两份腿报告 | `cite-check.py <两份报告> --root <冻结副本的 crates 加主仓 .claude、research、records 拼的根> --background <背景材料>`：「✓ 核了 66 处引文，对上 66 处，没判 0 处（逐处列在上面）」 |

## 二、逐格判决

| 格 | 正推 | 攻方 | 判决 |
|---|---|---|---|
| K1 f2 对条款、会不会误红 | 七项与条款对上（`walk.rs` 第 4320–4347 行）；77 格（k 次暖机挂载 × z 次零单元发布）I-5.5 全成立、判红清单全空；128 MiB 环与两单元用例过 | 原实现合计建成 621 次（1 MiB 环与默认环；挂载 1–12 次、零单元发布 0–30 次、单元区 64–1600 槽），I-5.5 红 0，池级 checker 全不红；落点相对 U 只有 0 与 4 | **没打中**。两条腿立场不同、都没造出误红，攻方给了理由（这一类版本的第一个落点只在 [U, U + 64) 里，f2 在那一段只看 U、U + 2 两条记录是不是这一版的，推的）。按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」，「没打中」各只一次，不算定论，下一轮代码轮的攻方接着攻这一格 |
| K1-a 开放段取法在回落时 | 「开放段取树表单元所在段」规则没说、推的一致 | 提交内生块回落（D3（空间分配） 已定项 8 第 2 条）那一刻实现清掉开放段，树表单元不在任何开放段里，f2 仍排除它所在的 64 槽段；回落落在 [U, U + 64) 就误红；617 次建成回落 0 次（准入在单元区 720 槽还拒，740 槽起放行，那时至少 11 个整段），今天的准入下够不着（推的） | **缺口坐实为推的、合法历史够不着**。记进 C687（从 mkfs 水位建树那一版落点判不了的几格）：准入式或段长变了要重看；攻方改法甲零轮，不做 |
| K1 其余近似（隔离、抬 F 扣住、R、回收之后、多盘） | 一致，推的；隔离一格「规则没说、误红条件窄」 | 没打中；回收之后量过，其余推的 | 照两腿；被抛弃根、抬 F 的历史两条腿都没造，记作没验 |
| K1-6 kb 与实现 | 冲突两处：`.claude/kb/invariants.md` 第 199 行 I-5.5 状态格仍写「今天的实现仍写死 U + 4，回收之后会误红」；`walk.rs` 第 4311–4312 行注释说几样近似「都照实写进」I-5.5 那一行，kb 只写了回收门槛一样 | — | **坐实**（主 agent 现查第 199 行状态格原文与 `walk.rs` 第 4311–4320 行）。注释是对的、kb 落后：同日改 I-5.5 那一行，状态格写现值，判据格把四样近似写全 |
| K2-1 H2（回收门槛放松一个 txg） | — | **打中（复跑见第四节）**：mkfs、可写挂载 9 次、第 9 次挂载上写第一个文件（txg 26）；正确实现落 U + 4，H2 落 U；两种几何最终镜像上 `i55=holds red=[]`，池级 checker 一条不红；同一次发布取样 13 个崩溃态，I-2.1、I-4.8、I-7.4 各红 4 个，I-5.5 红 0 | **坐实**：f2 把「这一版自己的记录」当空，重用把旧记录改写成这一版的，门槛过没过在这一版的分配记录里不留痕；对回收门槛两个方向都判不出，太严一边是 C689 的 H1，太松一边是 H2。H2 不是 I-5.5 管得了的：最终镜像上它没有不一致，坏处只在根落盘之前那几个崩溃态（txg 2 还是候选、它的实例表被盖了），那是 I-7.4 的活。用户定 f2 时接受的限度是「回收门槛近似」，H2 落在这句里；不改 I-5.5、不翻用户定案。改法：① C689 补 H2 这一半（攻方改法丙）；② 实现那一侧补一条 harness 用例钉「挂载 9 次再写第一个文件落 U + 4」，配一行变异「释放代 ≤ floor + 1」（今天的变异表只有第 30 行「回收不看释放代」那一种整个去掉门槛的，差一个 txg 的没有），进下一批；③ 攻方改法乙（f3 两个方向用）零轮，与 C689 已记的 f3 同一笔，不另开 |
| K2-2 到 K2-5 | — | 放松 3 个 txg、ForcedToZero、同一次发布释放又重用 U + 2：I-5.5 不红、别的不变量红；跳过第一个空对在挂载 10 次那一格判成立（H1 一族）；开放段算错碰不到 t1 | 照攻方：都在 H1 / H2 两族或由别的不变量抓，不另立 |
| K2-6 GPU 与 CPU | 只读代码推了候选集覆盖最小合格起点 | 把 WGSL 候选法照字面转写，与 CPU 在 300000 组对抗取样上 0 处不同，阳性对照 22919 处不同；WGSL 本身没跑 | 没打中，只到「转写与 CPU 一致」；naga 编出来在 GPU 上跑的对拍归提交时 |
| K3 I-9.27 证据补丁 | 条款五句对上；四张坏镜像各被 2298–2301 单独杀；两条库单测 2302、2303 各杀一条；I-5.5 六条变异 2292–2297 全杀 | — | 站住 |
| K3-3 写 0 的七种 | 「种类 X 一律不判」变异：种类 3、1、9、10 被杀，4、6、7、8、11、12 全活；这六种在那张树表里都存在 | — | **采**：`walk.rs` 按种类分臂判，按种类漏判是真会发生的形态。下一批给六种各补一张坏镜像（头 ID 改成非 0），各配一行「种类 X 一律不判」变异 |
| K3-6 19-2 改名 | 纯改名：记号流等长，27 处差全是标识符对标识符 | — | 站住；装置输出不变归提交时复跑行核 |

## 三、去向

1. 19-1、19-2 的改动站住，不回改。
2. 同日写回（主 agent 自己写）：`.claude/kb/invariants.md` I-5.5 那一行（状态格写现值、判据格写全四样近似）；`.claude/kb/checks-owed.md` C689 补 H2、C687 补回落那一格。
3. 下一批实现员：H2 那条 harness 用例与「释放代 ≤ floor + 1」变异；I-9.27 写 0 的六种各一张坏镜像与变异；连同第 20 批已定的 F3 与根环那条线转来的「U6 照甲」注释措辞。
4. 提交时：GPU 对拍（naga 编出的内核在卡上跑）、E165 与 E171 装置复跑行。

## 四、复跑核对

主 agent 在会话草稿 `b19-opus-rerun/` 照 `REPO=$PWD WORK=<草稿> bash research/prompts/m3-formal-code-b19-r1-opus-model/tools/rerun.sh` 复跑（冻结副本加攻方补丁），16 格全 `exit=0`。与攻方报告抄的行逐字相同的：

- `h2-slack0.log`：「name=F1 unit_area_slots=2000 mounts=9 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[4] tree_table_rel_U=526 i55=holds red=[] detail=」
- `h2-slack1.log`：「name=F1 unit_area_slots=2000 mounts=9 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[0] tree_table_rel_U=526 i55=holds red=[] detail=」
- `h2-default-ring-slack1.log`：「name=F1 unit_area_slots=211968 mounts=9 zero_unit_publishes=0 reuse=reuse_window_gated_by_the_rollback_floor open_rel_U=512 cluster_segments_rel_U=[512] txg=26 data_rel_U=[0] tree_table_rel_U=532 i55=holds red=[] detail=」
- `layer0-sample-slack0.log`：「name=i55 gate_slack=0 evaluated=9 red=0 not_applicable=4」，没有 checker-red 行
- `layer0-sample-slack1.log`：「name=checker-red gate_slack=1 invariant=I-7.4 red_states=4 first=候选根 txg 2 引用的单元已被复用或抹头（校验和对不上或头用不了）」，I-2.1、I-4.8 同样各 4；「name=i55 gate_slack=1 evaluated=9 red=0 not_applicable=4」
- `gpu-transliteration.log`：「name=gpu-transliteration cases=300000 disagreements=0 first=None」；阳性对照 `gpu-transliteration-mutant.log` 报 `disagreements=22919`

K1 的计数：原实现那几格（`f0-small`、`f1-sweep`、`f1-zero`、`h2-slack0`、`h2-default-ring-slack0`）合计 613 次 `i55=holds`、判红清单非空 0 次，准入拒 132 次；攻方报的 621 / 137 另含它 `z-*`、`f01-*` 两组里原实现那几行，复跑把它们分进了 `z-default-ring` 与 `f0-small`，`z-default-ring` 里判红清单非空的 5 行都是 ForcedToZero（K2-2、K2-3），不是原实现。落点相对 U：0 出现 144 次、4 出现 478 次、2 出现 1 次（ForcedToZero 的 K2-3 那一格）。

核不动：checker 档、GPU 上 naga 编出来的内核没跑；H2 那段历史的层 0 全量（`closed_form_states=1048583`）没跑——H2 改法走 harness 用例钉实现的落点，不靠层 0 全量，不为它跑；K1-a 回落、隔离、抬 F 都是推的。

## 五、交用户表（零轮）

| # | 形态 | 来源 | 量过没有 | 去向 |
|---|---|---|---|---|
| 1 | 甲：树表段里有这一版的数据单元时不当开放段 | 攻方 | 没量（合法历史够不着） | 不做；记进 C687 |
| 2 | 乙：f3 两个方向用 | 攻方 | 只在它自己的模型上推了挂载 9、10 次两格 | 与 C689 已记的 f3 同一笔，不另开；要做的话是翻用户 2026-10-04 定的 f2，届时连同代价交用户 |

不涉及这一轮要用户定的岔路。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 8 | 不影响 | 2026-10-04 不受影响：H2 落在射程已写的「回收门槛近似复算」那一句里，定案与射程不改 |
| D3（空间分配） 已定项 7 | 不影响 | 2026-10-04 不受影响：只拿回收谓词当 H2 的改动点 |
| D8（核心索引结构） 已定项 8 | 不影响 | 2026-10-04 不受影响：I-9.27 条款五句与代码对上，补镜像不动条款 |
