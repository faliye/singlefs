# m3-close-b19-reason-r2：I-3.1 整份改记读法、I-7.9 丁全放开那一形与 C944 归属（2026-10-31）

## 一、题面

第一轮判决 `research/prompts/m3-close-b19-reason-r1-main-verification.md` 第四节列了被攻过零轮的三样：Q2 三处简化放开的实现（两种都算、一致才判、逐层往下认），Q3 只认整份改记读法加逐槽两判，以及 I-3.1 部分没判说明补的数。第五节把 C944（走到的单元起点没有自己的记录而被别的记录罩着）登记成「第二轮先攻归哪条不变量」。实现员照判决写完、补丁已打进主工作区（报告 `~/.local/share/singlefs/work/m3-close-b19-reason-r1-impl/result.md`，补丁 4071cb1f…；GPU 一侧 I-3.1 对拍另一份补丁 `~/.local/share/singlefs/work/m3-close-b19-i31-gpu-parity/`）。实现员另留了三处读法交主 agent 认（报告第八节第 1–3 条），主 agent 已认，没被攻过。这一轮攻的是这几样写出来之后的判法本身。

第二轮的分工照 `.claude/rules/three-way-inference.md`「多轮」一节：攻第一轮的修补本身、产生结论的方法、从没被看过的地方。第一轮攻过的角度不重复：丁的前提在今天的写者上成不成立（Q1 三问）、三处简化放开前后的代价比例（Q2）、逐判混取拼出的宽窄两张最小镜像（Q3）。

## 二、实现今天的样子（主 agent 的观测，2026-10-31）

方案按 `crates/` 今天的实现来谈。行号是此刻主工作区的（批 19 写段的补丁都已打，验证段还没跑：编译与测试的结论以开工快照之后那一趟合入后验证为准）。

**I-3.1 取宽（`crates/singlefs-checker/src/walk.rs`）**

- `records_with_the_spans_the_span_invariant_reddens_set_to_their_unit_classes`（10509 起）：最新根里每条未释放记录，起点槽的单元头用得了、那一类的槽数 `!=` 记录自述跨度（10521），就改记成那一类的槽数；一条都没改交回 `None`。
- `allocated_statistic_lies_within_the_bounds_of_the_unit_class_reading`（10541 起，文档 10535–10540）：只读改记那一份，(L) 记账不少于改记那一份的 |E(θ_high)|；(U) 记账不多于 min(改记那一份的 |E(t)|, |Ω ∪ U|)；逐槽下沿：遍历走到的每一槽都在改记那一份 E(θ_high) 里；逐槽上沿：改记那一份 E(t) 的每一槽都在 Ω ∪ U 里。四样都成立报部分没判（说明点名 I-3.18，带记账的已分配、遍历加还在 defer 的、两种读法各自的下沿上沿，单位字节），否则判红。
- `AllocatedStatisticBoundsOfADevice`（10395 起）多一个字段 `slots_counted_at_the_root_reclaim_threshold_outside_the_walk_and_the_deferred`；`allocated_statistic_bounds_of_a_device`（10416 起）多收隔离豁免的槽集合。
- 条文 `crates/singlefs-checker/invariants/05-空间记账.md` 第 8 行 I-3.1「按记录自述的跨度三判不成立时……不成立判红」那一段；I-3.18 在第 25 行；I-5.4（分配记录罩住的槽互不相交） 在 `crates/singlefs-checker/invariants/07-分配器.md` 第 9 行，只按记录自述的跨度判重叠。
- GPU 扫描内核 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`：`c_slot_exempted_as_quarantined_against_every_walked_version`（2953 起）、`c_allocated_statistic_bounds_of_a_device`（2968 起，逐槽下沿按 `span_reading` 本身数）、`c_allocated_statistic_lies_within_its_bounds`（3037 起，两种读法共用一个调用点再与逐槽上沿相与）。GPU 对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_writer_identity_allocation_record_spans_and_new_root_segments_of_every_record_like_the_cpu_checker.rs`。
- 坏镜像 `crates/singlefs-harness/tests/checker_known_bad_images.rs`：宽 `record_widened_over_a_walked_one_slot_neighbor_without_its_own_record_reddens_the_allocated_statistic_and_the_span_invariants`（3834 起，映射树根那条记录跨度改 3、树表单元自己那条记录拿掉、记账不动）；窄 `narrowed_record_with_a_leaked_record_over_an_all_zero_slot_reddens_the_allocated_statistic_and_the_span_invariants`（3934 起）。两张断言都只红 I-3.1 与 I-3.18。实现员的证红：变异「退回逐判混取」让两张只红 I-3.18、不红 I-3.1。

**I-7.9 丁全放开那一形（`crates/singlefs-checker/src/walk.rs`）**

- `possible_roots_in_the_ring_slot_at_the_raise`（9012 起）交回一槽可能住的每一种：被盖那一槽从 u 起往下逐层认（u、u − 3S、…）。那一层与抬 F 那条根同一实例，或那个实例有行且每一行 u < T，就认作落了盘、停下；u ≥ T、被抛弃、实例没有行时它算一种，再往下认。落点式的正 txg 认完了算空着一种。槽 0 而抬之前的 F 为 0 时，第 0 代根另算一种（树表判不了，`None`）。那一层 txg 低于抬之前的 F 时记一种空着、不再往下（实现员报告第八节第 1 条的读法：「窗口外」= 那一层 txg 低于抬之前的 F）。某一层 journal 里没有带提交标记的记录、或有几个实例，照旧报环重建不出。
- `rollback_floor_against_the_ring_reconstructed_at_the_raise`（9109 起）：每槽取一种拼成一种可能的环，各滤有效根、各算上限、各判一次。全同才照它报，不一致报 `PossibleRingsDisagree`（8961，I-7.9 部分没判）。组合上界 `POSSIBLE_RINGS_AT_THE_RAISE_JUDGED_AT_MOST = 4096`（8977），超了报环重建不出。外壳 `rollback_floor_against_the_journal_reconstruction_of_the_ring_at_the_raise`（9357 起）认的版本改成「重放读的那一份环里带提交标记、txg 在 (0, 抬 F 那条根) 之间、不低于抬之前的 F、落点式住在被盖那几槽里的每一个 txg」；`judge_rollback_floor_raises_against_their_ceilings`（9455 起）多一臂 `PossibleRingsDisagree`（9582）。
- GPU：`c_root_in_the_ring_slot_at_the_raise`（1814 起）带参数 `choice`，交回第 choice 种与这一槽一共几种；`judge_rollback_floor_raises_against_their_ceilings`（1969 起）先走窗口一遍，再对可能的环一种走一遍，乘积过 4096 记射程里判不了。对拍 `crates/singlefs-checker-tier/tests/crash_judge_gpu_rollback_floor_ceiling_rebuilt_from_the_journal.rs`。往下认两层以上、第 0 代根那一形没有 GPU 镜像（实现员报告第八节第 6 条）。
- 条文 `crates/singlefs-checker/invariants/02-根环.md` I-7.9 一行。

**C944 那一形今天归谁**：`crates/singlefs-checker/invariants/` 下 grep「没有自己的记录」零命中。I-3.18 从记录往单元看（每条未释放记录的跨度等于它起点单元那一类）；I-3.15（未释放记录罩住的单元由映射指在这一槽）、I-3.10（已分配记录的分配代等于它罩住的单元的诞生代号）也都从记录往单元看。从单元往记录看的那一边（遍历走到的单元，起点槽上有没有一条起点就在这里的未释放记录）没有一条不变量点名。I-3.1 (P) 只要求走到的槽在某条记录罩住的槽里，不要求起点对齐。

## 三、格

### Q1　I-3.1 整份改记读法加逐槽两判：还漏不漏、误不误红

1. **漏**：有没有一张镜像，按记录自述三判不成立、整份改记读法四判都成立、I-3.1 报部分没判，而统计漏了或多了的那几槽没被 I-3.18 的红说全。第一轮宽窄两张都靠逐判混取，这一轮不重复。要攻的是改记本身带出来的形：改记之后两条记录罩同一槽（I-5.4 只按自述跨度判，看不见）；一条记录改宽之后罩进一个隔离豁免的槽、或罩进一个只有被抛弃根引用的槽；改记那一份 E(θ_high) 与 E(t) 在释放代落在 (t, θ_high] 的槽上不同时，逐槽上沿把 U 里的槽当成合法。
2. **误红**：合法写者写出的镜像（含崩溃点重放的崩溃态、读坏槽钉门槛那几形、隔离记号）进得了这一支吗？进得了的，四判会不会不成立而判红。I-3.18 在合法镜像上不红，所以这一支只在坏镜像上进得来。要核这句话在「头用得了」的认法上是不是处处成立，比如起点槽上是一个已被合法复用的别类单元，而那条记录是未释放的。
3. **说明里的数**：部分没判那一句带的五个数够不够判部分没判的那块到底多大（第一轮正推腿指的缺口），有没有一种盘面上两种读法的下沿上沿都把真病包在里面、读说明的人却看不出来。

要交的：每一问一个结论（造得出 / 造不出 / 推的）。造得出的给最小镜像（盘上每条记录、单元、记账数），逐条列它红的不变量与违例说明，判真病有没有说全。造不出的给代码里挡住它的那一行。

### Q2　C944：走到的单元起点没有自己的记录，归哪条不变量

第一轮正推腿小世界里，取宽之前就藏得住的 209 个盘面：一条宽记录盖住一个没有自己记录的邻居单元，记账照单元记，I-3.1 按记录自述三判成立，I-3.18 只说跨度。

1. 合法写者写得出「遍历走到的单元起点上没有一条起点就在这里的未释放记录」吗？逐种列出可能的来源，每一种给写者代码里的那一行，或给一段合法历史：中央映射换位（引用按映射位置记，I-3.1 条文那一段）、隔离、由记录施加出来的那几版与最新根的记录树不是同一棵、只被更早候选根走到的单元（记录已释放）、打包记录单元、第 0 版树表单元。
2. 按第 1 问的结论，给这一形挑归属，候选至少写三个：I-3.18 加一个反方向（单元往记录看）；I-3.15 加一个反方向；新立一条。每个候选写清射程（判哪几版、只看未释放还是都看、走到指哪个遍历集合），各自在第 1 问列的合法来源上误不误红，在 Q1 那些镜像上多抓到什么。
3. 给推荐的那个候选写一张字节级坏镜像的造法，照 `crates/singlefs-harness/tests/checker_known_bad_images.rs` 已有用例的改法写（改哪条记录、拿掉哪条、记账动不动），以及只红哪几条不变量。

### Q3　I-7.9 丁全放开那一形：枚举全不全、一致才判与读法

1. **枚举全不全**：「全部可能的环都成立才判成立」只在真环一定在枚举出的那几种里时才不放坏写者。造一段历史，真住户不在 `possible_roots_in_the_ring_slot_at_the_raise` 交回的那几种里，同时每一种可能的环都判成立，而真环上 F 越过了上限；或证明真住户一定在里面（逐层停下的条件、「落点式的正 txg 认完算空着」、journal 里有几个实例时报重建不出，三处逐一核）。
2. **窗口外那一停（实现员读法第 1 条）**：那一层 txg 低于抬之前的 F 时记一种空着、不再往下。核它在 `rollback_floor_ceiling_of_the_valid_roots` 的式子里与「那一槽住着一条 txg 低于 F 的无效根」是不是处处等价：最旧有效根、每块盘最新有效根、非空有效根去重计数三项逐项核。不等价的给盘面。
3. **第 0 代根树表判不了（读法第 2 条）与组合上界 4096（读法第 3 条）**：第 0 代根那一种按 `None` 记，两沿相同照判、不同报部分没判；核它会不会让一种本该判红的环落到部分没判，以及用 mkfs 树表 0 条这条事实能不能判（池级 checker 不引实现，`crates/singlefs-checker` 只依赖 `singlefs-format`）。4096 在第一轮两腿模型（`research/prompts/m3-close-b19-reason-r1-sonnet-model/`、`research/prompts/m3-close-b19-reason-r1-opus-model/`）那一套合法历史上让多少比例的抬 F 根报环重建不出；与第一轮判决 Q2 那一格「全放开」的部分没判比例（正推 5.0% / 2.1%、攻方 5.1% / 2.2%，S=4 / S=8）合在一起，今天的实现报部分没判的比例是多少。
4. **CPU 与 GPU**：GPU 先走窗口一遍再走可能的环；CPU 先判窗口定不定得住上限，定不住才走丁。核两边在「窗口定得住、丁又会判出不一样」的盘面上是不是同一个结论，往下认两层以上与第 0 代根那两形 GPU 没有对拍镜像，给出各一张该补的镜像（盘面与两边应判什么）。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 辩方腿（Sonnet） | Q1–Q3 | 复核第一轮判决 Q2「定」、Q3「改法」两行写成代码之后够不够得着：实现做的是不是判决说的（逐句对代码），判决挑的改法是不是同样打中被它否掉的替代方案（Q3 的 `!=` 改 `>`、Q2 的保守取法）；替被判出局的一方辩护时给盘面，不给倾向。Q2 C944 的归属给每个候选的合法来源核对 |
| 攻方腿（Opus） | Q1–Q3 | 找反例：Q1 造改记读法本身带出来的漏判与误红盘面；Q2 造「遍历走到的单元起点没有自己的记录」的合法历史（推翻候选的那一种）；Q3 造真住户不在枚举里的历史、窗口外那一停不等价的盘面、CPU 与 GPU 判得不一样的盘面。第一轮攻过的角度不重复（丁的前提在今天的写者上成不成立、三处简化放开前后的比例、逐判混取拼出的两张镜像） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建小模型核的，模型放 `research/prompts/m3-close-b19-reason-r2-<模型>-model/`，草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。第一轮两腿的模型可以拷来改，拷来的那一份放进这一轮的模型目录。
- 腿不起脚本：模型与复跑命令写进报告「要主 agent 跑的命令」一节，主 agent 经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 4G 的上限跑；不跑重型测试、不起 checker 档的 cargo、不碰 GPU。
- 报告写进 `research/prompts/m3-close-b19-reason-r2-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
