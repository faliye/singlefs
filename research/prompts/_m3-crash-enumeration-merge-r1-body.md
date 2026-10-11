# 层 0 与崩溃放量合成一套崩溃点重放：设计轮第一轮正文（2026-10-10）

<!-- doc-lint:not-numbers K1 K2 K3 K4 K5 K6 K7 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 W F5 -->

## 一、题面

崩溃状态的枚举与判定今天有两个入口、两个名字：门禁 checker-tier-layer0-replay（「层 0」，13 条 `crash-case:`）与 `scripts/crash/crash-amplification.sh`（「崩溃放量」，8 项）。两边各有快档与全量；同一条流的两套全量常写在同一个测试文件里，各录一遍、各举一遍、各判一遍。用户 2026-10-10 定合成一套，原话：
- 「层 0 如果是 崩溃放量 为什么不用 gpu加速呢？ 很显然这样分开是不对的，要改。现在就改吧」
- 「那么 现在跑之前就要合并。 两个的快档和慢档都要合并到一起。」「如果不合并跑了，等于下一次又要重跑」
- 「CPU 对拍 肯定也是日常剪枝跑  每次提交选全量和剪枝 跑的时候可以逐项。 这是写所有测试的标准。」
- 弹窗定「提交跑整域，剪枝或全量二选一」（问的是合并之后每次提交跑快档还是整域）。

**要不要合、提交跑不跑整域、CPU 对拍跑不跑全部状态，用户已定，不在这一轮攻。** 这一轮攻第三节写死的那一种合并形态：落得下来吗、丢没丢今天两边判的东西、漏了什么。

条款：`.claude/kb/decisions/13-验证路线.md` 已定项 4（崩溃点重放只枚举整写子集）、已定项 9（崩溃点重放要分三层，第 253 行「全量指两样」）、已定项 15（验证代码分两档）；`.claude/rules/verification.md`「checker 档自己分快档与全量」「门禁的结构」；`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「入口脚本：不带参数就是全量」「判定可复用：按输入存，按前缀剪」「用上现有的硬件加速」「用例原子化」。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

事实表 `~/.local/share/singlefs/work/crash-enumeration-merge-facts/report.md`（sha256 63228045…35bc6，clerk 只读列的），主 agent 现查过下面标「现查」的几条，其余以事实表的文件:行号为准、腿自己再核：
- 判法一份（现查）：`crates/singlefs-checker-tier/src/crash.rs` 第 1259 行文档「层 0 的计数（[`evaluate_state_for_versions`] 一族）与崩溃放量流水线（`crash_amplification`）都从这一份取，不各判一遍」，第 1261 行 `pub struct Layer0StateJudgement`；`crates/singlefs-checker-tier/src/crash_amplification.rs` 第 1282 行调 `judge_layer0_state_range_against_the_version_table`。GPU 判器判同一串红项（事实表 2.4：`crash_amplification.rs` 第 1836 行 `gpu_verdict_items`），判不了时整趟报错、不退回 CPU（2033、2643、7120 行）。
- 枚举域一份：D13 已定项 4；放量在展开上限盖住最长一段时就是层 0 的 `EveryProperSubset`，态数同一个 `layer0_plan_state_count`（`crash.rs` 第 1556 行，现查函数在）。
- 重叠 7 行（`.claude/gate.d/stage-inputs.tsv` 第 49–55 行：第二条流、抬 F、σ、多记录、删除、树分裂、按位置寻址）；σ（层 0 一段 2¹⁶ 子集、只判恢复与缺单元；放量 65548 态判全套）与按位置寻址（层 0 四条 `ENUMERABLE_IN_FULL`，放量五条 `ALL`，多出的 `ExtentLowerSegmentGrowsToTwoLevels` 单元写段约 300 写、展开 30 盖不住）两边域不同。放量另有一项 `fixed-script-through-rollback` 没有对应的 `crash-case:`。
- 只在层 0 的 6 行（第 48、56–60 行）：第一条流、io_min 8192 可写挂载、崩溃注入快档、读故障态不持有见证、实例表清扫挂载、清扫分阶段重挂。放量引擎（`CrashFlow` 八个字段，`crash_amplification.rs` 第 201–210 行）缺三样：逐状态观察者、逐状态再起可写挂载、设备故障模型（故障只在 `crash_injection.rs`）。崩溃注入与读故障两行是随机历史加读故障，要理想模型、挂载与二次崩溃，GPU 查表判不了。重叠流里删除、抬 F 两行层 0 也有逐状态可写挂载。层 0 用例另断言计数（`OracleCounts`、`states_by_publish`、闭式相等），放量只断言 red=0、态数录全、对账为空。
- 门禁快档 feature-gated-tests 格已按展开 6 跑那 8 项放量函数；`--full` 不带特性、不编它们（事实表 1.3）。
- 复用键口径两处说法不一：`crates/singlefs-checker-tier/src/crash_identity.rs` 文件头第 6–8 行说复用键罩判法代码，`crash_amplification.rs` 文件头第 7–8 行说「判法不进键」。
- 名字：全仓 170 份文件（不含门禁样本）里有「层 0」「layer0」「崩溃放量」「crash_amplification / crash-amplification」（事实表第 5 节逐文件）。
- 最近的数：放量 2026-09-30 全量 `CRASH_AMPLIFICATION_TOTAL items=7 failed=0 states=2563727926 red=0 seconds=27288`（展开 28）；2026-10-08 展开 30 多记录 `states=1342177375 reused=16777259`、删除 `states=1448083638`；层 0 全量除第一条流的 2026-09-27 预演标记外没有跑出来的数。
- 方案按 `crates/` 今天的实现来谈。

## 三、被判的形态（一种）

- F1 名字：合成之后统一叫「崩溃点重放」（D13 已定项 9 标题与 `.claude/rules/verification.md`「崩溃一致性只能靠崩溃点重放验证」已用这个名字），下面分快档与整域；「层 0」「崩溃放量」作为装置名退役，照 `.claude/rules/path-moves.md`「改一个全仓术语」登记进 `.claude/kb/term-renames.md`。D13 已定项 9 的层表（层 0 / 层 1 / 层 2 讲负载规模）怎么处置，是这一形态要回答的一处。
- F2 一张登记表、一个入口：13 条 `crash-case:` 与 8 项放量合成一张表，一项一条流（`fixed-script-through-rollback` 与第二条流按事实表核出来的关系合或分）；一个入口脚本，`--list-items` / `--item` / 不带参数就是整域，照 `test-script-discipline.md`「入口脚本」。门禁一道（checker-tier-layer0-replay 改名并吸收放量入口），`stage-inputs.tsv` 一种登记形态。
- F3 一套引擎：固定流的项都走放量那条录写流、按 D13 已定项 4 展开、身份与前缀共享、判定库按块存的流水线；同一条流今天两份全量用例合成一份。引擎补三个口子：逐状态观察者（CPU 侧，如 I-3.10 读集计数、F5、io_min 8192 那一形的槽 1 核对）、逐状态再起可写挂载（删除、抬 F、清扫分阶段重挂）、计数断言（闭式相等、`states_by_publish`、`OracleCounts` 逐项）。
- F4 判法：每个状态 CPU 跑 `Layer0StateJudgement`（对拍参照，每个状态都判，不抽样）；建得出 GPU 判表的项 GPU 也判，逐状态比 GPU 与 CPU，不一致判红；建不出表或要真跑实现代码的项，登记表里那一项写「GPU 不判」与理由，不静默退回、不整趟报错。
- F5 随机历史两项（崩溃注入快档、读故障态不持有见证）：同一个入口、同一个判定库与复用键约定、同一种快档 / 整域分法；驱动照今天 `crash_injection.rs` 的随机历史与故障注入，不经录写流引擎；GPU 不判（照 F4 登记理由）。
- F6 两档：快档 = 每项小规模（固定流展开 6，随机历史小种子数），GPU 与 CPU 都判，每次提交跑；整域 = 每项展开盖住每一段（`exhaustive=true`），每次提交跑，默认剪枝（按条复用：键同的、与已判等价的才剪），点名全量就清空复用重跑。点名单跑一项与它在整域里跑判定相同。
- F7 整域盖不住的流（今天按位置寻址的 `ExtentLowerSegmentGrowsToTwoLevels`）：照 `test-script-discipline.md`「按登记整批不判的一类不算剪枝」登记、成功那句逐个列「本次未跑」，不判红也不算全绿。
- F8 跑在哪：GPU 照 `scripts/crash/gpu-quiesce-run.sh` 清场与私有显卡配置；双机照今天放量与层 0 分片各自那一套合成一套；内存上限照 `.claude/main-agent.md`「一轮怎么开、怎么收」第 12 条。
- F9 条款：D13 已定项 9「全量指两样」改成一样，「全量档提交时不默认跑」改成提交跑整域（剪枝或全量），层 0 快档 / 全量的射程句跟着改；`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/main-agent.md`、`.claude/skills/crash-test/commit-stages.md` 里两样分开写的地方跟着改。
- F10 旧数：事实表第 6 节 8 处（`stage-inputs.tsv` 第 51–53 行注释、D13 第 259 行 12230590578、记录里 `exhaustive=true` 的说法、`crash_amplification.rs` 第 1833 行文档、两份文件头复用键口径）在落地那一批里改成现值。

## 四、格

### K1　判别力不丢

今天两边每一项判的、断言的（两份 oracle、池级 checker、记录核对器两条、两遍恢复；各用例的观察者、逐状态可写挂载、F5、见证判据 W、计数断言；放量的 GPU 对 CPU 逐状态比、第 ②③ 段结构核对），在 F1–F10 下逐项都还在吗？哪一项会丢、会变弱，指到代码。σ 合成后取哪一个域、判哪一套才不丢。

### K2　枚举域

每条流合成后的状态集合是不是今天两边的并集（含两边各自多出的那几个态）？第二条流两套造流代码今天只断言段形与写数相同、字节相同没断言，合成取哪一份、要补什么才敢说同一条流？删除流两边态数差 15728640（事实表 3.1，推的是代码版本不同）在今天的树上对不对得上？

### K3　复用、剪枝与逐项

F6 的按条复用：键罩没罩全输入（被测代码、判法代码、参数、工具链、观察者与可写挂载那几截、随机历史的种子）？两份文件头「判法进不进键」口径冲突按哪一份落？剪掉的是不是只有键同与等价两种（等价关系有没有用例钉住）？点名单跑一项与整域里跑判定相同怎么证？剪枝自证（关掉剪枝与复用在小输入上逐项相同）放哪一档？

### K4　GPU 判不了的处置

F4 的「登记 GPU 不判」与今天「判不了整趟报错」：哪几项会落进 GPU 不判（io_min 8192 的槽距、清扫的 12 槽小环池与抹头写、随机历史两项、逐状态再挂载那一截），各缺 GPU 判器的哪一判（`crates/singlefs-checker-tier/src/crash_judge_tables.rs` `JudgeTablesRefusal`、`crash_judge_gpu.rs`）？登记了 GPU 不判的项，F4 的「GPU 与 CPU 逐状态比」在它身上落空，算不算 `test-script-discipline.md`「用上现有的硬件加速」那一条的违背、怎么报？

### K5　提交时的代价

用户已定提交跑整域，这一格不翻案，只要数：照今天各项的态数（事实表 1.2、2.6）与可得的速率，改到 core 的那次提交整域要跑多久（两台、五张卡、31 线程）；只改测试或脚本的那次剪枝能省多少。给计数模型与原样命令，量过的与推的分开写。

### K6　改名与条款

F1、F9 的射程：门禁名、`# gate-covers: 崩溃点重放` 键、`.claude/gate-not-implemented.tsv`、全绿标记前缀（`scripts/gate/admission.py` 第 271 行 `singlefs-crash-case-green.`）、钩子 `.claude/hooks/lib_heavy_tests.py` 认重型的写法、`selftest-registry.tsv`、门禁样本、170 份文件里的名字；冻结证据与已归档产物里的旧名照 `path-moves.md` 怎么留。D13 已定项 9 层表（层 0 / 层 1 / 层 2）合成之后还讲不讲负载规模分层，怎么写才不与 F6 的两档打架。

### K7　次序

落地要动 `crash_judge_*`、`crash_amplification.rs`、`crash.rs`、门禁与钩子：删除线 m3-batch52-54-code-r1 的开工快照里有 `crash_judge_dispatch.rs`、`crash_judge_gpu.rs` 等（那一轮没判完），根环第二十六块补丁待合，E171 冻结排在这一题之后。按文件不相交怎么切批、哪一批先合，才不撞别的会话的快照、不让 E171 冻结的那棵树里是一半合了的装置？

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写仓里的路径与行号（`git rev-parse HEAD` 加工作区，开工时 `sha256sum` 记下你引的文件）。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷仓副本，不在主工作区改。读码推出的「没打中」尽量换成造出来的反例尝试。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 12G；编出来的测试二进制也经它跑，不直接执行。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
- 不跑任何门禁阶段（`.claude/gate.d/` 下的脚本一律不起，`--list`、`--list-items` 除外），不跑 checker 档测试、崩溃放量、GPU。
