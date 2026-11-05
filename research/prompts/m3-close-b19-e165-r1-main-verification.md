# m3-close-b19-e165-r1 判决：C650 剩下那一族造不造得出（2026-11-05）

正文 `research/prompts/_m3-close-b19-e165-r1-body.md`；腿报告：正推 `research/prompts/m3-close-b19-e165-r1-sonnet-output.md`、攻方 `research/prompts/m3-close-b19-e165-r1-opus-output.md`（5a54d061…）、攻方模型改跑法 `research/prompts/m3-close-b19-e165-r1-opus-b-output.md`（63fc857b…）。第一轮。

## 一、核查

- 开工快照：`task-server/task submit --cwd ~/.local/share/singlefs/work/m3-close-b19-e165-r1-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m3-close-b19-e165-r1 --out ~/.local/share/singlefs/work/m3-close-b19-e165-r1-main/m3-close-b19-e165-r1-tree --repository /home/fy5090/code/singlefs`（任务 7629）末行「清单点名 10 份，倒推出 8 份、sha256 对上 8 份；倒推不出 2 份、对不上 0 份」。倒推不出的是 `.claude/kb/decisions/16-发布语义.md`、`.claude/kb/decisions/23-journal的角色与格式.md`（「快照之后有提交改过它」）；腿引这两份的地方逐处人工核：D16 第 195 行「一个 txg 只有一个落点槽，平局只剩第 0 代那几份同字节的根」、第 215 行「一次发布的持久顺序恒为」、D23 第 471 行「崩在记录持久之后、根槽持久之前」，此刻逐字都在。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r1-sonnet-output.md research/prompts/m3-close-b19-e165-r1-opus-output.md research/prompts/m3-close-b19-e165-r1-opus-b-output.md --root /home/fy5090/code/singlefs --background research/prompts/_m3-close-b19-e165-r1-background.md` 末行「✓ 核了 36 处引文、114 处裸行号，对上 33 处，没判 117 处（其中裸行号没有引文可比 74 处；逐处列在上面）」。两腿共同依靠的代码主 agent 现读：`crates/singlefs-core/src/recovery.rs:1888` `choose_root` 只按自证与否摊键、不按实例表滤被抛弃的根；`recovery.rs` 第 5042–5049 行 R2 的注释「撕裂的系统配置轮换（读得出、自证不过，设备没报错）不算判不出」；`recovery.rs:1930` `highest_root_txg` 只数此刻读得出、自证过的根。
- 正推点名的三条测试（主工作区）：7218 `root_ring_readers_take_a_root_outside_its_formula_slot_for_a_bad_read`「test result: ok. 2 passed」，7219 `root_choice_on_a_tied_checkpoint_txg_takes_the_higher_instance_in_a_later_slot`「1 passed」，7220 `a_root_sits_only_in_the_ring_slot_its_txg_targets`「1 passed」。
- 攻方模型：第一份（`e165_attack_form_b.rs`）拷进 `~/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree` 跑估时两次（7230、7436），都被撤：它单线程判 393241 态、不打进度，超了 `.claude/rules/verification.md` 指定范围 20 万态量级（主 agent 交之前没核，记在调度记录）。改跑法的第二份（`e165_attack_form_b_sliced.rs`，sha256 4f67d72e…）拷进同一棵树，7628 跑最小历史、对拍、估时三条「test result: ok. 3 passed」，`OPUSMODEL` 行全份存在 `~/.local/share/singlefs/work/m3-close-b19-e165-r1-main/model-7628-opusmodel-lines.txt`（43 行）。整行：
  - `OPUSMODEL name=cross_check threads_sliced=31 selected_states=600 state_lines_single=600 state_lines_sliced=600 single_seconds=18.7 sliced_seconds=1.1 identical=true first_difference=none`
  - 最小历史六格（根槽三种读坏 × 系统配置最新槽两种读坏）都是 `form_b=true`、`baseline_c_lost=false`、`v1826_c_lost=true`（数的：6 / 6 / 6），其一：`OPUSMODEL name=minimal_history root_fault=all_zero newest_system_configuration_fault=all_zero h2_mount=ok instance=2 chosen=(1,4) effective=(1,4) row_txg=5 crash_point=before_root_of_instance_2_txg_6 persisted_writes=45/69 roots=[(0,0),(1,3),(1,6),(1,1),(1,4),(1,2),(2,5)] form_a=false form_b=true baseline=(1,6) baseline_content=version_d baseline_c_lost=false v1823=(1,6) v1823_content=version_d v1823_c_lost=false v1825=(1,6) v1825_content=version_d v1825_c_lost=false v1826=(2,5) v1826_content=version_b v1826_c_lost=true core_choose_root=(1,6) checker_red=[I-3.1,I-3.9,I-4.2,I-3.5,I-8.14,I-3.21] mount_after=refused:NewerStateStillUnreadableAfterOneReread mount_after_content=not_mounted mount_after_c_lost=true`
  - 抽样：`OPUSMODEL name=cell roots_hidden=2 root_and_record_fault=all_zero newest_system_configuration_fault=all_zero sample=evenly_spaced selected=19663 stride=20 domain=393241 states=19663 unchosen_system_configuration=0 form_a=0 form_b=3277 form_b_first_publish_complete=3277 v1823_differs=0 v1825_differs=0 v1826_differs=3277 core_disagrees_with_local_baseline=0 checker_red_states=3277 …`
  - 挡法分得开（同一份产物）：`OPUSMODEL name=cell_mount roots_hidden=2 root_and_record_fault=device_error newest_system_configuration_fault=none h2_mount=refused:NewerStateStillUnreadableAfterOneReread …`；`OPUSMODEL name=cell_mount roots_hidden=1 root_and_record_fault=device_error newest_system_configuration_fault=device_error h2_mount=refused:NewerStateStillUnreadableAfterOneReread …`；25 格里放行而造出形乙的，恰是系统配置最新槽读回全零或自证不过的那几格。
  - 对照：`kind=writer_variant_first_txg_not_above_the_unreadable_root … verdict=red_as_expected`、`kind=writer_variant_same_txg_in_another_slot … verdict=misplaced_root_read_as_bad_form_a_absent`，与 7230 撤之前打出的两行逐字相同。
  - 全量没跑：估时行 `projected_full_seconds=7507`、`total_states=5636445`，超出日常验证量级；判决只用最小历史与抽样。

## 二、逐格判决

| 格 | 判决 | 依据 |
|---|---|---|
| 背景材料两处与代码不合 | **认（两腿都点出，主 agent 现读坐实）**：正文第二节「未被抛弃的摊成择新键」不对，`choose_root` 不滤被抛弃的根；「三条现由 Verus 必红世界与 KL 罩着」对 v1823、v1825 不对，E165 实验页第 1296 行 LQ3 那一行 KL 上两条都是 0、只有 v1826 红 6 | 第一节 |
| Q1 形甲（同 txg 不同实例） | **造不出（正推、攻方一致；读者侧三条测试绿，模型对照 `form_a_absent`，抽样 `form_a=0`）**：一个 txg 只有一个落点槽，读者把错位的根当读坏；平局只剩第 0 代三份同字节的根。挂载那一刻读坏根环槽时照样成立。**挂起**：这是「没打中」，攻方只有一份样本论证它（第二份只改模型），按 `.claude/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」要再抽一次 | 两腿 Q1.1、Q2.1；第一节 |
| Q1 / Q2.1 形乙（实例更高而 txg 更低） | **造得出（模型上量的，攻方打中）**：单故障之内造不出（N-配置与 R2 分别挡，`cell_mount` 两行 `refused`）；同一次可写挂载里两条根与它们的记录读坏、**每块盘最新那一槽系统配置读回全零或自证不过**（R2 按撕裂轮换放过）时，新实例首版 txg 落到被隐藏的根之下，盘上 (1,6) 与 (2,5) 并存。正推给的另一段最小历史（同一次挂载 ≥ 5 处读坏、三次挂载两次崩溃）没建模型，算推的 | 第一节最小历史六格与抽样行 |
| Q1.2 形乙下各择哪条、已确认的发布丢不丢 | **量的**：基线、v1823、v1825 择 (1,6)，读出 version_d，C 不丢；v1826 择 (2,5)，读出 version_b，C 丢。另量到（零轮线索）：这些状态上池级 checker 红 I-3.1、I-3.5、I-3.9、I-4.2、I-8.14、I-3.21，之后在不带故障的盘上可写挂载被拒（`mount_after=refused:NewerStateStillUnreadableAfterOneReread`） | 第一节 |
| Q2.2 被抛弃判、回退、清扫会不会在择根前筛掉 | **不会（两腿一致，读码，主 agent 现读）**：被抛弃判只在择根之后按所选那条根的表用 | `recovery.rs:1888`、`recovery.rs:2230` |
| Q1.3 / Q2.3 三条变体 | v1823、v1825：在两腿论证与全部量过的状态上与基线同值（抽样 `v1823_differs=0 v1825_differs=0`），合法历史里没有让它们红的一族；按 `.claude/rules/mutation-sampling.md` 字面它们不是等价变异（Verus 必红世界杀得了），动态一侧无对象。v1826：形乙那一族就是它的敏感取样点（抽样 3277 态不同、C 丢），属第三类「取样点不敏感」，该补的就是这一族 | 两腿；第一节 |
| Q3 判据 | **改（两腿一致）**：「改坏择根序这一族要红」把三条捆成一句，前提「合法域里有敏感点」只对 v1826 成立；改成按变异分开写杀伤来源：v1823、v1825 写「合法历史无对象，由 Verus 必红世界罩」，v1826 写「KL 与多处读坏一族」 | 正推第六节、攻方第四节 |
| 改法甲、乙、丙（攻方第五节） | **零轮，不在这一轮判**：多处读坏下可写挂载写出池级 checker 红的镜像、之后可写挂载被拒，是不是在故障承诺之内、要不要改 R2 或首版 txg 的取法，各条路没有代价数 | 攻方第五节 |
| C88 与 D16 第 195 行相抵（正推附带） | 交写回时核 C88 那一行是不是腐烂 | 正推第三节 |

## 三、交用户的

这一轮不交岔路。多处读坏那一形登进 `.claude/kb/checks-owed.md` 新一行：写清要拦的输入（每块盘最新系统配置槽读回全零或自证不过，加两条根及其记录读坏），量到的后果（第一节最小历史行、`checker_red`、`mount_after`），候选甲、乙、丙标零轮；量出各条代价之后再交用户定。

## 四、被攻过零轮的

攻方第五节改法甲、乙、丙；正推第五节 5.2 的三件留档测试；正推的另一段最小历史。

## 五、去向

| 事 | 去向 |
|---|---|
| 第二轮 | 照 `.claude/rules/three-way-inference.md`「多轮」开：攻方第二份样本专打形甲「造不出」（同一份正文与背景），另派辩方复核这份判决（Q3 改判据那一格、v1826 那一族够不够关 C650） |
| C650 剩下那一句 | 第二轮判决之后写回：v1823、v1825 关成「合法历史无对象」，v1826 补多处读坏那一族；在那之前不动 |
| 多处读坏那一形 | 新登 `.claude/kb/checks-owed.md` C971（多处读坏时可写挂载写出倒挂的新实例根），见第三节 |
| 背景材料两处不合 | 记进第二轮正文的「实现今天的样子」，改正 |
| 攻方模型进仓 | 第二份模型做成 harness 档入库用例之前，按 `.claude/rules/verification.md` 自己逐个造崩溃状态的归 checker 档，随第二轮判决定去向 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 备料 | 2026-11-05 不受影响：这一轮只判 C650 那一族造不造得出，没动读坏槽的处置口径；多处读坏那一形登欠账 |
| D23（journal 的角色与格式） 已定项 14 | 备料 | 2026-11-05 不受影响：新实例首版 txg 的取法没改，零轮改法甲动到它，量过代价再交用户 |
