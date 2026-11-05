# m3-close-b19-e165-r2 判决：C650 剩下那一族，第二轮（2026-11-05）

正文 `research/prompts/_m3-close-b19-e165-r2-body.md`；腿报告：攻方第二份样本 `research/prompts/m3-close-b19-e165-r2-opus-output.md`（c39c5df8…），辩方 `research/prompts/m3-close-b19-e165-r2-sonnet-output.md`（58084755…）。被复核的是第一轮判决 `research/prompts/m3-close-b19-e165-r1-main-verification.md`。

## 一、核查

- 开工快照：`sha256sum -c research/prompts/m3-close-b19-e165-r2-snapshot/kb-sha256.txt research/prompts/m3-close-b19-e165-r2-snapshot/crates-sha256.txt` 写判决时全 OK（主 agent 现跑）。腿跑着时，主 agent 往 `.claude/kb/checks-owed.md` 插过一行 C972，发现后三语撤回，sha256 回到快照值 9798192f…，所以腿读到的是快照那一刻的树；`mount.rs` 不在快照里（实现员在它末尾追加测试模块，腿引的行号不变）。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r2-sonnet-output.md research/prompts/m3-close-b19-e165-r2-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-b19-e165-r2-background.md` 末行「✓ 核了 44 处引文、44 处裸行号，对上 44 处，没判 44 处（其中裸行号没有引文可比 44 处；逐处列在上面）」。两腿共同依靠的原文主 agent 现读：
  - `crates/singlefs-core/src/recovery.rs` 第 1865–1873 行：`read_root_ring_slot` 解出根之后，若 `!root_sits_in_the_ring_slot_the_formula_gives(...)` 就交 `NotSelfVerified`；
  - `crates/singlefs-core/src/root_ring.rs` 第 188–193 行：txg 0 只认第 0 代那几槽，别的 txg 只认 `target_for_publish` 那一槽；
  - `.claude/kb/decisions/22-单元原子性怎么合成.md` 第 185 行原文「每盘各坏一槽系统配置、同时最新根一时读不出的双故障形不在承诺里（用户 2026-09-28 定不罩），那一形抬 F 之前池级 checker 一条都不红」，第 189 行「双故障形不罩」；
  - `crates/singlefs-core/src/instance_table_sweep.rs` 第 461 行起 `every_root_of_the_ring_self_verified`：逐槽解析根，不调落点核。
- 攻方模型：`task-server/task submit --cwd ~/.local/share/singlefs/work/m3-close-b19-e165-r2-main -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r2-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work ~/.local/share/singlefs/work/m3-close-b19-e165-r2-main/rerun-model`（7660）退 0，`OPUSMODEL` 行全份存 `~/.local/share/singlefs/work/m3-close-b19-e165-r2-main/model-7660-opusmodel-lines.txt`（23 行）；自证 7661「✓ 自检通过（查了 6 格）」。复跑留下的仓副本 775M 读完即删。量到的：
  - 产品读者逐槽摆法，S = 4、8、16 三档：`nonzero_txgs_admitted_in_two_or_more_slots=0`；对照臂（去掉落点核）同三档是 26、50、98。
  - 写者写得出的编码整环随机摆（每档 15000 环）、产品臂：S = 4、8、16 三档都是 `form_a_at_a_nonzero_txg=0 … v1823_differs_from_today=0 v1825_differs_from_today=0`。
  - 任意编码一族、产品臂：形甲只出现在 txg 0（`form_a_at_the_newest_txg_zero` 2703、2691、2686），这一族带 txg 0 而实例非 0 的根，合法写者写不出（mkfs 写实例 0）。
  - 单线程与切片多线程对拍 3000 环 `single_thread_equals_sliced=true`。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| 形甲（第一轮挂起的那一格） | **造不出（定）**：两份攻方样本、正推一致；第二份样本的挡法在读者一侧，对 `choose_root` 读到的任意盘面都成立，与写者历史无关；量的：产品读者上同一非 0 txg 至多一槽进择根，写者写得出的编码上 v1823、v1825 与基线逐环相同 | 第一节模型行；`recovery.rs:1867`、`root_ring.rs:188` |
| C971 那一形在不在故障承诺之内 | **不在（已定，不交用户）**：每块盘最新那一槽系统配置读坏、同时根读不出，落在 D22（单元原子性怎么合成） 第 185 行「双故障形不在承诺里（用户 2026-09-28 定不罩）」那一形里（辩方 Q2.2，主 agent 现读坐实）。第一轮判决「是不是在故障承诺之内还没定」不成立 | D22 第 185、189 行 |
| C971 那一行写过头的两处 | **认（辩方 S9）**：一，「择根读出最新已确认的那一版」不对，最小历史行是 `baseline=(1,6) baseline_content=version_d baseline_c_lost=false`——读出的是落了盘、没确认的 D，已确认的 C 没丢；二，「模型上量的」是攻方副本上的模型，按 `.claude/rules/three-way-inference.md`「一条腿在副本装置上量出来的数，要在入库的装置上重做一次才能引」不进 kb 当量的。另有一处要核：D22 第 185 行「那一形抬 F 之前池级 checker 一条都不红」与攻方模型在这一形（可写挂载之后、暖机中途崩溃）上量到池级 checker 红六条相抵，两边条件不全同，记进 C971 | 第一轮判决第一节最小历史行 |
| v1826 | **合法历史（承诺内）无对象**：C971 那一形在承诺外，不能当它的合法历史一族（辩方 Q2.2）；它由 Verus 必红世界与 KL 杀，KL 是照落点公式摆的对抗根环，不算合法历史 | 辩方 Q2.2；E165 实验页第 1296 行 |
| 第一轮 Q3 改判据 | **是放宽，记一次输（辩方 Q2.1，引 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「臂的定义也在「跑前写死」之列」）**：按原判据「改坏择根序这一族要红」字面，承诺内合法历史上三条都不红，C650 剩下那一句按原判据**没满足**，这一判照写。病根是判据要的输入在承诺内的合法写者下造不出，属同一份规则「判据自己也会写错」表里「判别子观测不到」那一行，处方「改判据，改完是新一轮」 | 辩方 Q2.1；第一节 |
| 第一轮推的却写成判的 9 句 | **认 S1、S2、S3、S4、S6、S7、S8、S9**；S5（等价变异的归类两腿说法相反）照记分歧，按 `.claude/rules/mutation-sampling.md`「没红的三类，判据不同」字面：三条在全部输入上不同值（Verus 世界杀得了），不是等价变异；在承诺内合法输入上同值，要留档的是「合法域上同值」这一句 | 辩方第五节 |
| 清扫读根环不核落点（攻方第二份样本附带） | **线索，零轮**：`instance_table_sweep.rs` 第 461 行起逐槽解析根、不调落点核，而 D16（发布语义） 第 195 行写落点核「恢复与挂载读根环时都调」。推的后果：一次错位读可能让清扫删掉环里还有根的实例行；没造盘面、没量 | 攻方第三节；第一节 |

## 三、交用户的

无岔路。C971 那一形用户 2026-09-28 已定不罩；判据改成什么照下面「去向」那一格的新判据，放进第三轮攻。

## 四、被攻过零轮的

新判据（下一格）；清扫不核落点那一条线索；正推第一轮 5.2 那三件留档测试的形态。

## 五、去向

| 事 | 去向 |
|---|---|
| C650 剩下那一句的新判据（零轮，第三轮攻） | 「三条择根变体由 Verus 必红世界杀，v1826 另由 KL 杀；承诺内的合法写者造不出让它们红的盘面，等价于『同一非 0 txg 至多一条根进择根』，这一句做成读者侧逐槽穷举的入库用例（攻方第二份模型那一条做成 harness 档用例）留档」 |
| 第三轮 | 最终轮：攻新判据撑不撑得住、留档用例的判别力，辩方复核第二轮；第三轮之后不再开 |
| C971 | 第三轮判决之后写回：改那两处写过头的话，补「落在 D22 第 185 行不罩的双故障形里」，D22 那一句与模型读数相抵的那一处登进这一行 |
| 清扫不核落点 | 第三轮判决之后新登 `.claude/kb/checks-owed.md` |
| C972（卸载抬 F 实参的等价测试） | 第三轮判决之后插回（三行原样在 `~/.local/share/singlefs/work/m3-close-b19-e165-r1-main/c972-rows-to-insert-after-r2.tsv`） |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 8 | 备料 | 2026-11-05 不受影响：只引它射程里「双故障形不罩」那一句判 C971 在承诺外；它「池级 checker 一条都不红」那一句与攻方副本模型的读数相抵，登进 C971 待核，条款不动 |
| D16（发布语义） 已定项 1 | 不影响 | 2026-11-05 不受影响：这一轮没动读坏槽的处置口径 |
