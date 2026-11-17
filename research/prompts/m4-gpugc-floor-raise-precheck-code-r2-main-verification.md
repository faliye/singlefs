# m4-gpugc-floor-raise-precheck-code-r2：判决（2026-11-10）

正文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r2-body.md`，改动全文 `research/prompts/_m4-gpugc-floor-raise-precheck-code-r2-diff.md`，开工快照 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-snapshot/`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端辩方（Sonnet） | `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-sonnet-output.md`，模型 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/sonnet-model` | 采纳的改法够得着缺口，也打中非宽护状态：新那一项无条件加，没有宽护时多放行的写白推、不换出路；检查单八格成立 |
| 云端攻方（Opus） | `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-output.md`，模型 `research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-model/` | T1-2 多算、T1-3 白推打中（推的，待探针）；T1-4 推过一串之后丢出路一半（推的）；T2-1 出路这一报不要读故障就可达；类型与名字字面不成立 |

主 agent 的核查（量的标「量的」）：

1. 快照倒推任务 11965 退 0：「清单点名 2 份，倒推出 2 份，sha256 对上 2 份」，两份都取工作区此刻那一份（量的）。`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-floor-raise-precheck-code-r2-opus-output.md research/prompts/m4-gpugc-floor-raise-precheck-code-r2-sonnet-output.md --root ~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/tree --background research/prompts/_m4-gpugc-floor-raise-precheck-code-r2-background.md` 退 0，末行「✓ 核了 50 处引文、50 处裸行号，对上 26 处，没判 74 处（其中裸行号没有引文可比 20 处；逐处列在上面）」，没有对不上的；没判的多在树外（验证包、`allocator.rs`、`mount.rs`），判决依赖的 `mounted_session.rs` 先算那几行主 agent 现读过。
2. 辩方探针（主 agent 拷模型到 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/sonnet-model`，经包装 `rerun-from-a-clean-work-directory.sh` 交任务 11960，退 1：`forms` 那一项撞同名镜像 panic，别的项退 0）：并排那一份 `side-by-side.txt` sha256 前缀 `4ea3eb75a0cdfbb3`，`SUMMARY cells=200 same=200 lost_path_in_old=0 recovered_in_new=0 white_push_new=40 inside_positive_cells=200`（量的）。已有用例上新那一项翻面数（主 agent 从 `.precheck` 读数数的）：`existing-accept` 判了 18、槽对那一判 4、翻面 0；`existing-runtime` 判了 196、槽对 110、翻面 0；`existing-wide` 判了 430、槽对 2、翻面 0（量的）。
3. 攻方探针（主 agent 拷模型到 `~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-code-r2-main-check/opus-model`，经包装 `rerun-opus.sh` 交任务 11964，第 3 次以 9216M 上限退 0）：`opus-run/out/probe.out` sha256 前缀 `8e72cdeed7477443`（量的），整行：

   `probe PROBE_RESULT configurations=372 slot_pair_refusals=49610 vain_series_by_the_inside_term=18105 vain_empty_publishes=41026 vain_with_stuck_report=235 vain_without_outlet=17870 vain_remount_lets_it_through=0 vain_remount_still_refuses=1645 outlet_kept_by_the_inside_term=0 outlet_lost_after_a_series=0 outlet_lost_first_round=0 published_where_the_old_bound_fired=0 arms_differ=28670 stuck_reports=7730 c_ring_holding_at_probe=61 panics=0`

   两条自证：那一截读成 0 的一臂 `arms_differ=0 … vain_series_by_the_inside_term=0`；那一截读成大数的一臂 `vain_series_by_the_inside_term=4140`；汇总 `name=self_checks arms_differ_when_both_zero=0 vain_when_the_inside_term_is_huge=4140`，`name=exit_codes arm_a_zero=0 arm_a_huge=0 probe=0`（量的）。

两份探针互不共用代码（辩方按 `.precheck` 读数并排比，攻方另写配置扫描），读数同向：新那一项在真分配器上改结局的格全是白推（`recovered_in_new=0`、`outlet_kept_by_the_inside_term=0`、`vain_remount_lets_it_through=0`），没有一格换来写进或出路。

## 二、判决

| 条 | 读数与依据 | 判决 |
|---|---|---|
| 第一轮第三节第 1 条（上界加聚簇段里没挡的槽对）白推一串（攻方 T1-2、T1-3，辩方 T1） | 第一节第 2、3 条：白推 18105 串、41026 次空发布，其中 17870 串 C 那一圈不罩、不报出路；卸载重挂让它写进 0 次；新那一项换来出路 0 次、换来写进 0 次 | **打中，撤回**：违 `.claude/kb/decisions/16-发布语义.md` 第 275 行射程「推抬 F 放得出的全放出来也放不下就一串都不推」；换不来第一轮判决采纳它要保的东西 |
| 第一轮 T3-4（槽对那一判吃掉被宽护卡住的出路）的前提 | 那一截读成 0 的一臂（第一轮修补之前的判法）上 `outlet_lost_first_round=0`，`stuck_reports=7730`、`c_ring_holding_at_probe=61`（宽护那一报在取样里出现过）；攻方 T2-1 读码：出路那一报按 C 那一圈整圈判，不带读故障的崩溃之后就可达 | **第一轮采纳的理由在取样里没出现**：撤回第 1 条不丢出路（这一份取样）；一份样本不撑「没打中」，登欠账 |
| 推过一串之后再先算判出放不下、丢出路（攻方 T1-4） | `outlet_lost_after_a_series=0` | **这一份取样没撞上**；与聚簇段那一截无关，登欠账 |
| 上界算小、误拒推了本来写得进的写（第一轮 T1-1 复核） | `published_where_the_old_bound_fired=0`，372 个配置，与第一轮任务 11937 的 `hits=0` 是两份互不共用代码的取样 | **两份取样都没撞上**，记「没打中」 |
| 攻方改法一：先算判出放不下时 C 那一圈罩着就带上卸载重挂那条出路 | 腿提的，被攻过零轮；这一份取样里没有它要修的格（`outlet_lost_first_round=0`、`outlet_lost_after_a_series=0`） | **不改**，登欠账，标零轮 |
| 类型与名字（攻方：槽对与槽共用裸 `u64`） | 槽对字段就是新那一项 | 撤回第 1 条之后结构体里只剩槽数字段，**题面消失** |
| 新那一项读成 0 没有用例红（T2-2） | 两腿读码一致 | 撤回之后题面消失 |

检查单：

| 格 | 判决 |
|---|---|
| 走不到的理由 | 成立（两腿逐处追过那一处 `expect`）；撤回之后那一处随新那一项一起删 |
| 类型与名字 | 撤回之后成立（见第二节表倒数第二行） |
| 不兼容旧代码与旧格式、不新建判定库、所有任务多机可用、测试优先多线程、带 --item 的命令与项、新崩溃流多罩了什么 | 成立（两腿一致） |

被判的产品代码文件逐份：

| 文件 | 判决 |
|---|---|
| `crates/singlefs-core/src/mounted_session.rs` | **要改**：第三节第 1 条 |

## 三、改法（下一批的条款）

1. 撤回第 122 批补丁（`~/.local/share/singlefs/work/m4-gpugc-floor-raise-precheck-fixes-writer/patch/crates.patch`）里 `mounted_session.rs` 的聚簇段那一截：`SlotsAFloorRaiseCouldRelease` 的 `unblocked_slot_pairs_inside_the_cluster_segments`、`of_the_device` 多收的聚簇段参数、`most_a_floor_raise_could_make_available` 加上的那一截、判法用例里补的字段与新用例 `an_unblocked_slot_pair_inside_the_cluster_segments_counts_toward_the_slot_pair_bound`；`crates/mutations.tsv` 那一行「抬 F 先算：槽对那一判的上界不算聚簇段里没挡的槽对」一起删。文档「九种」、运行库两条崩溃循环用例上的不并行理由留着。
2. kb 不改：D16 已定项 10 射程本来就没写那一截（`grep -rn '聚簇段里没挡\|卸载重挂放得出' .claude/kb/` 零命中）。
3. 欠账登一行：先算判出放不下时 C 那一圈罩着、卸载重挂写得进的那一格（头一次与推过一串之后两种）只有两份取样没撞上；攻方改法一零轮。

改完开第三轮（最后一轮），攻撤回之后的形态。

## 交用户的

- 攻方改法一（判出放不下时带出路）零轮，取样里没有它要修的格，登欠账。

## 这一轮被判过的文件

- crates/singlefs-core/src/mounted_session.rs：判了槽对那一判上界加上的聚簇段那一截，撤回（第三节第 1 条）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 10 | 不影响 | 2026-11-10 不受影响：射程没写聚簇段那一截，撤回之后代码与射程对齐 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-10 不受影响：宽护那条出路在取样里没被先算吃掉 |
| D28（挂载期承诺量） 已定项 1 | 不影响 | 2026-11-10 不受影响：先算读它的式子，条款不动 |
