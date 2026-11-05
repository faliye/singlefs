# m3-close-b19-reason-r3：第三轮，连同批 19 核心改动的代码轮（2026-11-03）

## 一、题面

这一轮合三件：

1. **reason-r2 的第三轮（最后一轮）**。第二轮判决 `research/prompts/m3-close-b19-reason-r2-main-verification.md` 第四节列的被攻过零轮的几样，实现员照判决写完、补丁已打进主工作区。判决本身有两处措辞写宽或写窄，实现照更准的读法写了，这一轮一并攻：
   - 判决第二节「Q1　改法」写「改记之后各条未释放记录罩住的槽区间两两不相交」；实现成「至少有一条是未释放记录的那一对才比，两条都已释放的不比」，因为判决自己验收用的两张镜像（攻方「隔离单元尾槽压在释放代落在 (t, θ_high] 的记录上」、辩方「窄记录加尾槽上另一条记录」）都要拿未释放记录去比已释放记录才抓得到。
   - 判决第二节「Q2　归属」写前置「最新根的树表带种类 3 记录树且读全，不满足报这一条判不了」；实现把「树表 0 条」另算成不适用（0 条的那几版正是判决自己列的合法来源，这一条无从判），「有条目而没有种类 3」与「树表读不出」报部分没判。
2. **代码轮**：批 19 对 `crates/singlefs-core/src/` 的改动。条款是 D16（发布语义） 已定项 1 用户 2026-11-02 三条定案（保留池按 ckpt_cost 现算，代码原本就是；读坏槽卡住上限时推空，受一次准入的 B 次发布限；读回全零的槽并进上限）与 C953（写者与 checker 对第 0 代根记在哪块盘不一致）的改法（写者按根所在那一槽的区域认盘）。
3. **两份实验装置的计数分配器要不要登进 unsafe 放开表**：E183（容量斜率读缓存小盘准入与清扫阈值的代价数）、E197（扫描 journal 粘连内存随设备数）装置各 9 处 unsafe，全在 `unsafe impl GlobalAlloc for CountingAllocator` 那一段；门禁 code-source-discipline 的 unsafe-allowances 格判红（`src/bin/` 每个文件是自己的 crate 根，lib.rs 的 forbid 管不到；新增要登 `.claude/gate.d/unsafe-allowances.tsv` 并走三方）。

第三轮分工照 `.claude/rules/three-way-inference.md`「多轮」一节：「已定」项撑不撑得住、未定项清单的完整性、那一轮报出的数字能不能核。前两轮攻过的角度不重复：丁的前提在今天的写者上成不成立、三处简化放开前后的比例、逐判混取拼出的宽窄两张镜像（第一轮）；改记读法本身带出来的漏判与误红、C944 的合法来源与候选、丁的枚举全不全与窗口外那一停、第 0 代根按区域认盘那一形（第二轮）。

## 二、实现今天的样子（主 agent 的观测，2026-11-03）

方案按 `crates/` 今天的实现来谈。行号是此刻主工作区的；四份补丁都已打（`~/.local/share/singlefs/work/m3-close-b19-d16-impl-writer/patch/`、`…-reason-r2-impl-checker/patch/`、`…-impl-followup/patch/`、`…-impl-i321-notjudged/patch/`），合入后验证还没跑完。三个核心文件相对 HEAD 的差里还有别的会话没提交的改动，这一轮只攻上面四份补丁带进来的那几处，补丁原文就在那四个目录里。

**写者一侧（D16 已定项 1 与 C953）**

- 全零并：`crates/singlefs-core/src/allocator.rs:1687` `ring_slots_bad_or_reading_back_all_zero_at_mount` 把挂载时读坏（`BadAtMount`）与读回全零（`AllZeroAtMount`，这一批起也记能住的最大 txg）两类槽并起来；`crates/singlefs-core/src/mount.rs:4407` `ring_slots_bad_or_reading_back_all_zero_at_mount_by` 准入算上限时取这一份。回收门槛照旧钉最旧 txg。
- 推空（受 B 限）：`mount.rs:4778` `push_one_floor_raise_within_the_admission_budget` 先算不并那几槽的上限与并进之后的上限；并进之后 ≤ F 而不并时 > F，就推一次带现行 F、txg 加一的空发布，预算按「已推 + 1 + 1 ≤ `PUBLISHES_PER_ADMISSION_AT_MOST`」判（`mount.rs:4540` 定 8，第 4810 行那一判）；放不下照今天报预算用完。推的结局装在 `RaisedFloor` 里，新字段 `floor_of_the_publishes`（`mount.rs:4172` `enum FloorOfThePushedPublishes`）分两种；推空发布自己的失败报 `RaiseFloorSequencePublishFailed`。
- 写者按槽认盘：`mount.rs:3784` `roots_of_the_ring_for_the_ceiling` 交回的每条根带它所在的槽，算「每块盘上最新有效根」时按槽的区域认盘，不再从 txg 推。
- 用例：`crates/singlefs-harness/tests/admission_pushes_an_empty_publish_keeping_the_floor_while_the_ceiling_is_held_by_a_slot_unreadable_since_the_mount.rs`（推空之后放行、B 用完停下两条）、`crates/singlefs-harness/tests/admission_floor_ceiling_counts_each_genesis_root_on_the_device_of_its_ring_slot.rs`（攻方那段合法历史，期望上限 0）、`crates/singlefs-harness/tests/raising_the_floor_to_the_ceiling_does_not_pass_a_slot_unreadable_since_the_mount.rs`（读回全零那两份照新规则改：没洞那一份 F 停在 17、I-7.9 不红）。
- 理想模型（`crates/singlefs-harness/src/model.rs`、`history.rs`）不认读坏槽、全零槽，也没有推空（实现员报告推的，没跑）。
- `crates/singlefs-core/src/mounted_session.rs` 这一批只改了一句 `//!` 文档注释。

**checker 一侧（reason-r2 判决）**

- I-3.1 取宽那一支第五判：`crates/singlefs-checker/src/walk.rs:10580` `unreleased_records_overlapping_another_record_in_the_unit_class_reading`、`walk.rs:10706` `allocated_statistic_violated_by_records_overlapping_in_the_unit_class_reading`（违例点名每一对与相交的槽）；部分没判说明补的几个数在 `walk.rs:10693` 一带；GPU 同改在 `crates/singlefs-checker-tier/src/crash_judge_checker_scan.wgsl`。
- 新立 I-3.21（`crates/singlefs-checker/invariants/05-空间记账.md` 第 28 行）：`walk.rs:10748` `judge_walked_reference_starts_against_the_allocation_records_of_the_newest_root`，遍历用 I-3.1 那一份，记录取未释放或释放代高于 θ_high；树表 0 条报不适用，有条目而没有种类 3、树表读不出报部分没判（单测 `walk.rs:25744`、`25770`）；GPU 在扫描内核，对拍在 `crates/singlefs-checker-tier/tests/crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`。
- 第 0 代根两处：`walk.rs:8997` `possible_roots_in_the_ring_slot_at_the_raise` 认完时第 0 代根按 ABSENT 记、槽 0 而抬之前 F 为 0 时不记「空着」；GPU `c_root_in_the_ring_slot_at_the_raise` 同改。实现员判 I-7.9 的上限不用跟 D16 新条款改（理由在 `~/.local/share/singlefs/work/m3-close-b19-reason-r2-impl-checker/result.md` 第 6 节：镜像上整槽全零的槽在抬 F 那一刻也是空着、没有持久根，照条款式子本来就不进上限）。
- 扫描内核 checker-scan 展开后 396576 个表达式，离预算 400000 只剩 3424（C954）。

**装置的计数分配器**

- E183：`crates/singlefs-checker-tier/src/bin/e183_capacity_read_cache_reserve_and_sweep_costs.rs` 第 207–234 行；这一批把测量窗口改成只读发起测量那一条线程的计数（线程本地一份），全进程那一份留给 PC-堆；登记 `research/prompts/e183-r2-prereg.md` 第十二节 12.2。
- E197：`crates/singlefs-checker-tier/src/bin/e197_scan_journal_glue_memory_by_device_count.rs` 第 140–167 行。
- 两份的 unsafe 在 HEAD 里就是各 9 处。放开表 `.claude/gate.d/unsafe-allowances.tsv` 今天一个 crate 一行（第 3、4 段「唯一放开 unsafe 的文件」「唯一放开的函数名」），用户 2026-11-03 定改成一处一行、别的会话的工具员在改那一格。

## 三、格

### Q1　I-3.1 取宽第五判（reason-r2 改法）与判决措辞的更正

1. 「至少一条未释放就比、两条都已释放不比」这一读法：有没有一张合法镜像（含崩溃点重放的崩溃态、隔离记号、读坏槽钉门槛那几形）在这一判上误红；有没有一张坏镜像两条都已释放而相交、藏着真病、这一判放过。
2. 第五判与 I-5.4（只按自述跨度判重叠）在同一张镜像上是不是重复报、或互相矛盾。
3. 部分没判说明补的数：读说明的人能不能从这几个数分出「改记后重叠」与第二轮那几形。

### Q2　I-3.21 与「0 条不适用」的更正

1. 「0 条不适用」：那几版（mkfs 第 0 版、树表 0 条写过行的版、暖机零单元发布版）上真病能不能藏在这一条之外、又没有别的不变量罩着。
2. 「未释放或释放代高于 θ_high」这一记录集合：合法镜像上误不误红（第二轮两腿给的合法来源逐种核），坏镜像上漏不漏（第二轮攻方修正模型上 1195 个 C944 形）。
3. 有条目而没有种类 3、树表读不出报部分没判：这两形会不会被合法写者写出来、让部分没判比例大到 I-3.21 形同虚设。

### Q3　第 0 代根两处与 I-7.9 的上限

1. ABSENT 加「槽 0 而 F 为 0 不记空着」两处同批改：第二轮辩方量的「坏写者红 +7（S = 4）、+3（S = 8），合法部分没判降 55、370」能不能在今天的实现上复核；有没有一种合法环因此从部分没判变成红。
2. 实现员「I-7.9 的上限不用跟 D16 新条款改」的理由：写者今天把读回全零的槽并进上限（D16 第 38、39 行），checker 算上限时不并；有没有一段合法历史让两边上限不同而 checker 判红，或让坏写者越过 D16 字面上限而 checker 放过。

### Q4　代码轮：写者一侧做的是不是 D16 已定项 1 与 C953 说的

1. 全零并：`ring_slots_bad_or_reading_back_all_zero_at_mount` 并进上限的是不是条款第 38、39 行说的那一类槽，能住的最大 txg 记得对不对，回收门槛那一边有没有被一起改动。
2. 推空：「已推 + 1 + 1 ≤ 8」这一判是不是条款第 44 行「一次准入最多 B 次发布」的意思（推空计入 B），推空的空发布带现行 F、txg 加一，是不是 E173 登记里推空B 那一臂的定义；推完写掉那一槽之后照抬 F 那一串抬，次序有没有错。
3. 按槽认盘：每块盘上最新有效根按槽认盘之后，别的读环路径（挂载、回退判候选、算 F_生效）有没有还按 txg 认盘、两边对不上。实现员推这一形在挂着的准入路径上大概走不到（可写挂载暖机会推到每块盘都有本实例的根），核它，并给一段真写者上走得到或走不到的论证。
4. 理想模型不认读坏槽、全零槽与推空：随机历史与故障注入的对拍会不会因此报假红或漏真红，要不要同批补模型。

### Q5　两份装置的计数分配器

1. E183 量的是分配字节（`copy_bytes`），E197 量的是什么、离不离得开这个分配器；有没有不用 unsafe 的替代（例：读 `/proc/self` 下的计数、换已有的安全计数库、改由服务记的 cgroup 峰值），各自对登记里的量改不改口径。
2. 登记的话，两份各登几行、每行的理由与出处怎么写才站得住（照放开表文件头第 5、6 段的要求）。

要交的：每一问一个结论（造得出 / 造不出 / 推的），造得出给最小镜像或最小历史，造不出给挡住它的代码行；代码轮各问给代码与条款逐句对照。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 正推腿（Sonnet） | Q1–Q5 | 逐句对代码核实现做的是不是判决与条款说的（Q4 为主），两处判决措辞更正站不站得住（Q1、Q2 第 1 问），I-7.9 不改那一条的理由站不站得住（Q3 第 2 问），Q5 给出替代的代价对照 |
| 攻方腿（Opus） | Q1–Q5 | 找反例：Q1、Q2 造误红与漏判的镜像；Q3 造写者与 checker 上限不同而判错的合法历史；Q4 造实现与条款字面不一致的盘面或历史，按槽认盘在真写者上走得到的历史；Q5 指出登记理由里站不住的一句。前两轮攻过的角度不重复（见第一节末） |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数；引代码写开工快照里的路径与行号。
- 方案按 `crates/` 今天的实现来谈；要建小模型核的，模型放 `research/prompts/m3-close-b19-reason-r3-<模型>-model/`，草稿放腿自己的目录（`~/.local/share/singlefs/work/<腿名>/`），不在主工作区改。第二轮两腿的模型可以拷来改，拷来的那一份放进这一轮的模型目录。
- 腿不交任务：模型与复跑命令拼成 `task-server/task submit --cwd <模型目录绝对路径> -- <命令>` 的整条，写进报告「要主 agent 跑的命令」一节，由主 agent 交；不写内存上限、线程、台数；不跑重型测试、不起 checker 档的 cargo、不碰 GPU。
- 报告写进 `research/prompts/m3-close-b19-reason-r3-<模型>-output.md`，分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
