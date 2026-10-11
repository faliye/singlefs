# 层 0 与崩溃放量合成一套崩溃点重放：设计轮第二轮正文（2026-10-10）

<!-- doc-lint:not-numbers G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 G12 G13 G14 L1 L2 L3 L4 L5 L6 K1 K2 K3 K4 K5 K6 K7 F1 F2 F3 F4 F5 F6 F7 F8 F9 F10 W F5 -->

## 一、题面

第一轮正文 `research/prompts/_m3-crash-enumeration-merge-r1-body.md`、判决 `research/prompts/m3-crash-enumeration-merge-r1-main-verification.md`（两腿报告与模型路径在判决开头）。第一轮打中的几格挂起，这一轮攻第三节收严后的形态 G1–G14。

用户已定、不在这一轮攻的（原话在第一轮正文第一节与判决第三节）：要合；快档与整域都合；每次提交跑整域，剪枝或全量二选一；CPU 对拍不抽样、日常剪枝跑、逐项可跑；提交只等 GPU，CPU 对拍日常剪枝跑完；逐状态可写挂载挂在标准前缀上、不得例外；判一张镜像的全上 GPU、要真跑实现的只能 CPU，按判法分界、不按项开例外；多机配置合成一份、默认开 GPU；这些测试问题在这一轮测试（提交前整域与 E171 冻结）之前解决。

条款同第一轮：`.claude/kb/decisions/13-验证路线.md` 已定项 4、9、15；`.claude/rules/verification.md`；`.claude/singlefs-ai-sop/rules/test-script-discipline.md`「入口脚本：不带参数就是全量」「判定可复用：按输入存，按前缀剪」「用上现有的硬件加速」「用例原子化」「分快慢两档」。

## 二、实现今天的样子（主 agent 的观测，2026-10-10）

第一轮正文第二节、事实表 `~/.local/share/singlefs/work/crash-enumeration-merge-facts/report.md` 与第一轮两腿报告照旧有效；第一轮之后主工作区变了的：
- 根环第二十六块合入（`~/.local/share/singlefs/work/m3-r1/batch10z-26a/patch/`、`…/batch10z-26b/patch/`）：`crates/singlefs-checker-tier/src/crash_amplification.rs` 与 `gpu_unit_checks.rs` 只改了测试模块（切口用例各加方括号那一形、故意拒的写法），`crates/singlefs-core/src/mounted_session.rs` 推环入口先核盘表次序。合入后验证在跑。
- 删除线第 52–55 批已合入；第 56 批在做，要改 `crash_judge_checker_scan.wgsl`、`crash_judge_checker_common.wgsl`、`crash_judge_gpu.rs`、`crash_judge_dispatch.rs`、`crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state.rs`（往 GPU 判器里加判法），可能在这一轮腿跑着时合入，判决时照它的补丁核。
- 工具批 T1、T2 已合入（研究脚本、门禁结构；`scripts/gate/lib_libtest_result_lines.py` 新库）；T3 在改 `scripts/gate/admission.py`、`.claude/hooks/lib_heavy_tests.py`、`.claude/gate.d/stage-inputs.tsv` 等，这几份不在这一轮快照里，腿引它们只引事实、不钉行号。
- 方案按 `crates/` 今天的实现来谈。

## 三、被判的形态（收严后，一种）

- G1 名字：统一叫「崩溃点重放」，下分快档与整域；「层 0」「崩溃放量」作为装置名、门禁名、脚本名、英文标识符（`Layer0StateJudgement`、`enumerate_layer0_*`、`layer0_progress`、计数行前缀 `LAYER0…` 这一族）全部退役，照 `.claude/rules/path-moves.md`「改一个全仓术语」登记 `.claude/kb/term-renames.md`。英文标识符改名排在 E171 冻结之前的最后一批，冻结的那棵树里是改完的装置；改名那一批之后整域只重判一次。
- G2 一张登记表、一个入口、一道门禁：13 条 `crash-case:` 与 8 项放量合成一张表，一项一条流；入口认 `--quick`、`--list-items`、`--item <项>`，不带参数 = 快档全部项加整域全部项；门禁 checker-tier-layer0-replay 改名并吸收放量入口；`stage-inputs.tsv` 一种登记形态。改名、重型测试闸（`.claude/hooks/lib_heavy_tests.py` 按名字认入口与阶段那几张表）、自证登记表、门禁样本同一批改，闸的自证加一格「新入口起整域，子 agent 被拒」。
- G3 一套引擎：录写流、按 D13 已定项 4 展开、身份与前缀共享、判定库按块存；第二条流取录制器那一份造法，层 0 独有的三条断言（闭式、按发布分的态数、回退抬 F 判到）搬过去，另加「两份造法写表摘要相等」的快用例；随机历史两项（崩溃注入快档、读故障态不持有见证）也是这个入口与这个判定库里的项。
- G4 观察者挂在节点上：逐状态观察者（I-3.10 读集、F5、io_min 8192 槽 1 核对、删除流的发布边界与停在中间读回上一版）、逐状态再起可写挂载（核挂载做得成、续做走到空、挂载后池级 checker）是节点的一部分；挂在标准前缀节点上的观察者，对共享这个节点的每一项、每一个状态都跑，不按项开例外；观察者的代码进节点的计划哈希。
- G5 能走 GPU 的必须走 GPU，不走的登记明确豁免（用户 2026-10-10「再说一遍 测试能走gpu的必须走gpu 除非明确的豁免。 现在所有的崩溃放量 轻型要合一起， 重型要合一起。」）：判一张崩溃镜像的（两遍恢复、两份 oracle、池级 checker、记录核对器）每个状态都在 GPU 上判；不走 GPU 的计算一截一行登记进一张豁免表（截名、在哪段代码、为什么上不了 GPU：例「要真跑 Rust 实现的挂载路径，被测对象就是它」），今天要登记的是可写挂载、续做、挂载途中二次崩溃、与理想模型比、见证判据 W；门禁核两边逐项对上——代码里每一处不走 GPU 的判定计算都有豁免行、每一行豁免都指得到代码（欠账 C626（走 GPU 的计算类没有登记） 那一格）。随机历史两项的镜像那一判同样上 GPU（从它们录下的写流建判表）。
- G6 CPU 对拍：每个状态 CPU 也判一张镜像，与 GPU 逐状态比红项与恢复字（`recovery_words_of` 四个字）；比不上判红。GPU 判不了的个别状态（判不了位置位、表越过容量）记一条 GPU/CPU 不一致、判红、写哪张表满了，不静默拿 CPU 结论。
- G7 计数随判定块存：每块旁边存计数向量（逐条不变量的判到 / 违例 / 不适用、各观察者计数、按发布分的态数），复用时与判定一起读回；闭式、`states_by_publish`、逐条不变量计数、观察者计数这些断言每一趟（含剪枝那一趟）都判。
- G8 判法版本与复用键：块键 = 这一步带屏障的写表 + 起步镜像（照今天 `crash_amplification.rs` 文件头）；判定按（块键，判法版本）存；判法版本 = 判器闭包（`crash.rs`、GPU 判器）∪ 这一项测试文件出发的闭包 ∪ 引擎（`crash_amplification.rs` 与它的依赖）∪ 观察者代码。随机历史两项的键 = 种子 + 规模参数 + 测试文件出发的整份闭包，判定按段存。两份文件头口径统一成这一条。
- G9 两档：快档 = 每项在「甲二」（`InPlaceSubsetsWithCopyOnWriteNoneOrAll`）与展开 6 两个域的并集上判（随机历史两项用小种子数），GPU 判镜像、CPU 对拍与观察者都跑，每次提交跑；整域 = 每段全展开（`EveryProperSubset`），每次提交跑，默认剪枝（键与判法版本都同才复用、与已判等价的才剪，等价关系有用例钉住），点名全量就清空复用。提交只等 GPU 那一截跑完；CPU 那一截（对拍与要真跑实现的观察者）日常剪枝跑，提交时它没跟上的那一版逐项报「CPU 对拍本次未跑完」，里程碑收尾与 E171 冻结之前必须跟上。点名单跑一项与它在整域里跑判定相同；剪枝自证（关掉剪枝与复用在小输入上逐项相同）进快档。
- G10 整域里装不进 u64 的段：按段登记（不是整条流），照 `test-script-discipline.md`「按登记整批不判的一类不算剪枝」报 `states_skipped_by_ignore`、成功那句逐个列「本次未跑」，同一条流里装得进的段照判。
- G11 跑在哪：多机配置合成一份、默认 `ENABLE_GPU=1`；GPU 照 `scripts/crash/gpu-quiesce-run.sh` 清场与私有显卡配置；配置开了 GPU 而卡用不上（显存借不到、第二台连不上）退 78，不降级、不报绿；双机照今天放量与层 0 分片合成一套。
- G12 条款：D13 已定项 9「全量指两样」改成一样；层表只留「范围」一列（按负载规模分层），「什么时候跑」整块归 G9 两档；已定项 15、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/main-agent.md`、`.claude/skills/crash-test/commit-stages.md` 跟着改。
- G13 旧数：事实表第 6 节 8 处加 `scripts/crash/crash-amplification.sh` 第 42 行探针段形，落地那一批改成现值。
- G14 切批与次序：一、脚本、门禁、钩子、登记表、条款（G2、G9 的入口与档、G10、G11、G12），不碰 `crates/`；二、`crash_amplification.rs` 引擎口子（G4、G6、G7、G8）与只在合并范围里的测试文件；三、GPU 判器那几份（G5、G6），等删除线第 56 批合入后从主工作区拷底；四、英文标识符改名（G1）；之后整域一趟、E171 冻结。

## 四、格

### L1　第一轮修补本身（攻方）

G4、G6、G7、G8、G10 是照第一轮打中补的：每一条补完之后有没有新的漏洞？例：G4 观察者进计划哈希，会不会让本来能共享的前缀不再共享、整域态数翻倍；G7 计数向量随块存，复用时计数读回与这一趟新判的块怎么合、合错了谁会发现；G8 判法版本扩到测试文件与引擎之后，只改一项测试文件是不是只重判那一项、还是整域重判；G6 恢复字逐状态比，GPU 与 CPU 恢复字今天是不是逐位同义。

### L2　产生结论的方法（攻方）

第一轮的代价数（两腿模型）用的速率都是旧记录里的数：哪几条速率的口径与 G5、G9 的跑法不一样（GPU 那一趟含不含存判定、CPU 那一条是层 0 路还是放量路）？G9「提交只等 GPU」下，一次改到 core 的提交挂钟到底是多少、CPU 那一截日常跟上要多久（两台、五张卡、31 线程），给计数模型与原样命令，量过的与推的分开。

### L3　没被看过的地方（攻方）

第一轮两腿都没细看的：`layer0_progress.rs` 断点续跑与双机分片（层 0 那一套）与放量双机（`crash-amplification-two-hosts.sh`）合成一套时，账本、续跑、导入、两台工具链与输入指纹的核法怎么并；`verdict_store.rs`（RocksDB）与层 0 全绿标记两种存法并成一种时，旧标记与旧库的处置；门禁 `checker-tier-layer0-replay` 的 feature-gated-tests 格、`crash-case-check.py` 的 placement / modules 两样在合并后判什么。

### L4　辩方复核第一轮判决

第一轮判决第二节每一格的判（打中 / 没打中 / 定义缺口 / 采做法）够不够得着、是不是同样打中所有替代方案；判决有没有把某一格判重了或判轻了；第三节的用户定案在 G1–G14 里有没有被照错、照漏。

### L5　被判形态对用户定案

G1–G14 逐条对第一节「用户已定」的每一句：有没有一条违背或绕开（例：G9「提交只等 GPU」与「每次提交跑整域」两句同时成立吗；G10 按段登记算不算「例外」；G5 随机历史两项上 GPU 的那一截判表建不建得出）。

### L6　落得下来吗

照 G14 的切批，每一批的出口与验收标准写得清吗；第一批不碰 `crates/` 时，入口与门禁改名之后今天的两套装置还跑不跑得起来（中间态）；哪一批之后快档与整域第一次能整套跑通。

分工：攻方腿 L1、L2、L3、L6；辩方腿 L4、L5（另可复核 L6）。

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写仓里的路径与行号，开工时 `sha256sum` 记下你引的文件；第二节点名的那几份可能在你跑着时变，引它们只引事实。
- 腿自己量的数在自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`）里量，报告写命令与原样输出；量法拷仓副本，不在主工作区改。读码推出的「没打中」尽量换成造出来的反例尝试。
- **不跑 checker 档**：`singlefs-checker-tier` 包的任何测试（含快用例、直接执行它的测试二进制）、链它的库的小程序、门禁阶段（`--list`、`--list-items` 除外）、崩溃放量、GPU，一律不跑；要它们的数写进报告「要主 agent 跑的命令」交回。harness 档、纯算术模型、抽出来的独立小程序可以跑。
- 本机内存 60G、几条线同时在跑：每条编译与测试命令经 `scripts/hosts/run-with-memory-cap.sh` 带不超过 12G 的上限，同时起的件数乘上限不超过 12G。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句。
