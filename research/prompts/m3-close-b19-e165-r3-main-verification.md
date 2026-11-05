# m3-close-b19-e165-r3 判决：C650 剩下那一族的新判据，第三轮（最终轮，2026-11-05）

正文 `research/prompts/_m3-close-b19-e165-r3-body.md`；腿报告：攻方 `research/prompts/m3-close-b19-e165-r3-opus-output.md`（c9136a46…），辩方 `research/prompts/m3-close-b19-e165-r3-sonnet-output.md`（467658c5…）。被复核的是第二轮判决 `research/prompts/m3-close-b19-e165-r2-main-verification.md`。这是第三轮，之后这一题不再开轮。

## 一、核查

- 开工快照：`sha256sum -c research/prompts/m3-close-b19-e165-r3-snapshot/*.txt` 写判决时全 OK（主 agent 现跑）。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r3-sonnet-output.md research/prompts/m3-close-b19-e165-r3-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-b19-e165-r3-background.md` 末行「✓ 核了 37 处引文、48 处裸行号，对上 37 处，没判 48 处（其中裸行号没有引文可比 48 处；逐处列在上面）」。
- 模型复跑（主 agent 交，`--work` 在 `~/.local/share/singlefs/work/m3-close-b19-e165-r3-main/` 下，复跑留下的仓副本 1.8G 与 48M 读完即删）：
  - 7681：`bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r3-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work …/rerun-model` 退 0。共五棵树：不改的一棵，加 `singlefs-core` 的四种改坏各一棵；每棵跑 6 个测试目标，逐行存在 `…/m3-close-b19-e165-r3-main/model-7681-lines.txt`（282 行）。不改的那棵六个目标全绿。窗口穷举每档 44800 盘面，S = 4、16 两档同数：`OPUSMODEL name=window_disks slots_per_region=4 disks=44800 threads=31 placed_but_not_admitted=0 admitted_outside_the_window=0 product_choose_root_differs_from_today=0 nonzero_txg_at_most_one=44800 txg_zero_one_instance=36352 no_inversion=5848 txg_zero_one_instance_and_no_inversion=4928 differs[v1823,v1825,v1826]=[22, 42, 26154] differs_given_nonzero_txg_at_most_one=[22, 42, 26154] differs_given_txg_zero_one_instance=[0, 0, 20976] differs_given_no_inversion=[22, 42, 0] …`；对拍 `single_thread_equals_sliced=true`。四棵改坏树：读者把 txg 0 放宽到每槽（`reader_admits_txg_zero_in_every_slot`）、读者落点改成按位与（`reader_formula_masks_with_slots_per_region_minus_one`）两棵，只有两个模型目标里的一个红，仓里现有的四个目标（`root_ring_readers_take_a_root_outside_its_formula_slot_for_a_bad_read`、`root_choice_through_the_glue_puts_the_checkpoint_txg_before_the_instance`、`root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot`、`first_file_version_checks_the_previous_record`）全绿；胶水把实例摆在 txg 前面（`glue_puts_the_instance_before_the_txg`）一棵，仓里胶水用例红；挂载下一个 txg 加 0（`mount_next_txg_adds_zero`）一棵，攻方写者侧用例与仓里一个目标红。
  - 7682：第二轮攻方模型照原命令复跑，23 行 `OPUSMODEL` 去掉秒数之后与第二轮存档（`…/m3-close-b19-e165-r2-main/model-7660-opusmodel-lines.txt`）逐行同集（主 agent `diff` 现跑，无输出）。
  - 7683：自证「✓ 自检通过（查了 6 格）」。
- 辩方现读、主 agent 抽核：`.claude/kb/checks-owed.md` C88 那一行与 `.claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:608` 写「设备失而复得会造出两条同 txg 的合法根」；`.claude/kb/decisions/22-单元原子性怎么合成.md` 里 `grep -nF '失而复得'` 零命中。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| 第二轮新判据「等价于『同一非 0 txg 至多一条根进择根』」 | **打穿（攻方，模型上量的）**：「同一非 0 txg 至多一条」在 44800 盘面上全成立（`nonzero_txg_at_most_one=44800`），三条变体仍有不同（22、42、26154）；加「txg 0 只有一种实例代号」之后 v1823、v1825 归 0，加「不倒挂」之后 v1826 归 0 | 第一节 7681 那一行 |
| C650 剩下那一句的判据（定） | 三条变体分写：**v1823、v1825** 与基线择得不同，当且仅当「同一 txg 上两种实例代号同时进择根」；承诺内靠读者落点核（非 0 txg 只收落点那一槽）加写者「txg 0 的根只有 mkfs 写、实例 0」两样挡。**v1826** 与基线择得不同，当且仅当环里倒挂（实例代号更高的根 txg 更低）；承诺内靠「新实例第一次发布的 txg 大于环里一切可见的根」挡，只在 C971 那种双故障形（承诺外）里倒挂。三样各要一条入库留档用例：读者侧逐槽穷举（根恰在落点槽，S 取 4..=16 每一档，不只 2 的幂）、写者侧（mkfs 之后的可写挂载与发布序列里，非 mkfs 写出的根 txg 都大于 0、新实例第一次发布越过环里一切可见的根）、不倒挂（同一写者用例里断言）。三条都被攻过零轮（第三轮之后不再开），量过的是第一节那几行 | 攻方 Q1.1、Q1.2；第一节 |
| 留档判别力 | **认（量的）**：读者那两种改坏在仓里现有测试与 Verus 必红世界下都不红，Verus 必红世界只改 `singlefs-replay-prefix`（攻方数的 296 行） | 第一节 7681 |
| C971 交不交用户 | **交（辩方，改第二轮）**：D22（单元原子性怎么合成） 已定项 8 第 185 行「双故障形不在承诺里（用户 2026-09-28 定不罩）」含「读回全零」（辩方两条独立证据），「读得出而自证不过」是类推，条款没定一个故障怎么数；同形先例 `research/prompts/m3-batch35-36-code-r1-main-verification.md` 第 50、58 行判「都在单故障承诺之外，用户定修」。照 `.claude/rules/three-way-inference.md`「岔路交用户定之前，每条路要有代价数」，甲、乙、丙先量代价再交 | 辩方 Q2.1、Q2.3 |
| D22 第 185 行「那一形抬 F 之前池级 checker 一条都不红」 | **与攻方副本模型相抵，登进 C971**：副本模型在那一形（可写挂载之后、暖机中途崩溃）上量到池级 checker 红六条；读数在副本装置上，要在入库装置上重做才能引 | 第一轮判决第一节 |
| v1826「承诺内无对象」 | **改成「C971 一路之外还有没扫的」**：E165 实验页第 608 行点名的掉盘、设备失而复得、管理员回退、实例切换几类状态没扫，那几类里会不会倒挂没量过（辩方，推的）；留档的写者侧用例要覆盖到它们才算关 | 辩方 Q2.3 |
| 第二轮那一次输与病根 | **输照记，病根写「最近」**：C650 早就记着「取样点不敏感…还欠这一族状态」，两轮之后确认承诺内补不上；病根最近「判别子观测不到」，规则里没有「承诺内等价、承诺外可区分」现成一行 | 辩方 Q2.2 |
| 腐烂两处 | **认**：C88 与 E165 第 608 行那一句与读者落点核相抵；C583 与 `.claude/kb/decisions-history.md` 指的「四条定案」表不存在（表在 `records/2026-09-28-里程碑三作业点规划.md`「三　用户定案（原话）」下） | 辩方 Q2.1、Q2.3 |
| 清扫不核落点 | **线索，零轮，补条件**：`crates/singlefs-core/src/instance_table_sweep.rs` 第 356、361 行只在「实例在根环里没有自证根」时删行；要删掉还有根的行，得是错位读把某槽的真根换成别槽自证过的根，且被换掉的是那个实例在环里唯一的根 | 辩方 Q2.3；第二轮判决 |

## 三、交用户的

| 事 | 交什么 | 状态 |
|---|---|---|
| C971 改法甲、乙、丙 | 先量三条的代价（写放大、跳号、拒挂载的格数），再交用户定修不修、修哪一条 | 零轮；量代价之前不交 |

## 四、被攻过零轮的

第二节「C650 剩下那一句的判据（定）」那一格的三样留档用例；C971 甲、乙、丙；清扫那一条的补条件。

## 五、去向

| 事 | 去向 |
|---|---|
| C650 那一行 | 写回：原判据记一次输（早记的欠，两轮确认承诺内补不上）、判据照第二节那一格改成三条分写、病根写「最近判别子观测不到，规则无现成一行」；还欠改成三样留档用例；三轮判决文件路径进出处 |
| 三样留档用例 | 下一批实现员（`crates/singlefs-harness/tests/`，攻方第三轮写者侧用例 `e165_r3_writable_mounts_on_a_fresh_pool` 与第二轮读者模型作原型；读者那两种改坏各配一行变异，S 取 4..=16）；46 并包之后按新包名写 |
| C971 那一行 | 写回：改两处写过头的话（读出的是没确认的 D、已确认的 C 不丢；读数在副本装置上），补「落在 D22 已定项 8 第 185 行不罩的双故障形里（读回全零那一种）」与 D22 同句「一条都不红」与模型读数相抵；去向写「先量甲乙丙代价再交用户」 |
| v1826 未扫的几类状态 | 并进 C650 写回的还欠 |
| C88、E165 第 608 行、C583 与变更史的「四条定案」表 | 写回时改成现值 |
| 清扫不核落点 | 新登 `.claude/kb/checks-owed.md` |
| C972 | 插回（三行原样在 `~/.local/share/singlefs/work/m3-close-b19-e165-r1-main/c972-rows-to-insert-after-r2.tsv`） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 备料 | 2026-11-05 不受影响：只引它射程里「双故障形不罩」那一句；同句「一条都不红」与副本模型读数相抵，登进 C971，条款不动 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-05 不受影响：读坏槽的处置口径没动 |
