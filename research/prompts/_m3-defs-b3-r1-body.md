# 工具与治理批的定义与规则改动：第一轮正文（2026-10-12）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 D6 D7 -->

## 一、这一轮要判什么

批 3 把这一批落到定义与规则上的文字一次改完（调度记录 `records/2026-10-12-里程碑三工具与治理批.md` 课题 9、10、11），补丁 `~/.local/share/singlefs/work/m3-tooling-b3-defs/patch.diff`（sha256 70ff4f10…e93f，+25/−14）。它压着批 2 的钩子与脚本补丁（同一份冻结副本里都已打上）。用户 2026-10-12 定这几处定义改动「攒到下一批走三方」。

被判的文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/verification.md`、`.claude/rules/implementation-workflow.md`、`.claude/rules/three-way-inference.md`、`.claude/skills/crash-test/commit-stages.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b3-r1/repo/`（主工作区 2026-10-12 那一版加批 2 全部补丁与批 3 补丁，没有 `.git`）；被判文件与它们压着的钩子、脚本共 21 份的 sha256 在 `research/prompts/m3-defs-b3-r1-snapshot/frozen-copy-sha256.txt`。腿只读冻结副本，不读主工作区（主工作区没有这几份补丁）。

| 格 | 改后的形态 | 在哪 |
|---|---|---|
| D1 | 线程数：派发提示「线程上限：N」的 N 取登记值（`.claude/heavy-thread-default`，31），双机两台各取一份，机器忙时也固定、不看忙闲；删掉「N = 整机核数 ÷ 同时在跑的重活数」与「重活数变了给在跑的 agent 发消息改 N」；`capped.sh default` 取登记值；`implementation-workflow.md` 写目标形态（没设取登记值），`crates/` 现状由排除表的「代码批」行承接 | `main-agent.md`「派发提示怎么写」「派出去之后」「交回怎么读」、第 13 条；`agent-common.md` 线程上限那条；`commit-stages.md`；`implementation-workflow.md`「测试与崩溃检测优先多线程」 |
| D2 | `verification.md` 补四句：日常验证压在 20 万态量级、几十万态以上大域对拍先报时长；GPU 测试二进制 `--test-threads 1`、CPU 一侧在用例里多线程；GPU 与内存对拍不压并发、开满、只限不让哪台撞顶；显卡先用满一张再分下一张、派活与判法拆开；慢档用例没有派发提示时经 `capped.sh default` | `verification.md` |
| D3 | 只核自己的改动：共写文件里有别人的改动时只核这一批自己的还在不在，别人的不追查、不拦，按哈希登记的照现状登记；第 9 条改成别人的改动挡着这一批的活时才协商 | `main-agent.md` 第 9 条；`agent-common.md` |
| D4 | 私有主机与服务信息不进仓（只写规矩、不写值），进仓文字写「第二台」，真实值只在被 git 忽略的 `multi-host.env` 与 `~/.config/singlefs/` | `agent-common.md` |
| D5 | 「执行前拒绝的写法」补批 2 新增的四种拒（依赖 checker 档包的 crate 跑测试按重型、主工作区合入锁被占时起 cargo、往别人持锁的草稿目录写、线程上限不等于登记值）；「派出去之后」补看门狗新报的两样；共用约束加「开工先认锁」 | `agent-common.md`；`main-agent.md` |
| D6 | 用户叫停、不写判决的三方轮写 `research/prompts/<轮>-halted.md`（一行出处加叫停原话）收开工快照，派发闸 ⑥ 认它 | `three-way-inference.md` |
| D7 | 长活默认分片、断点续跑、按配置的机器自适应，派发提示不写死台数与额度 | `main-agent.md`「派发提示怎么写」 |

## 二、实现今天的样子（主 agent 的观测，2026-10-12 现查）

- `crates/` 这一轮不改；`crates/` 里没设变量时取 `std::thread::available_parallelism` 的 13 处（`crates/singlefs-checker-tier/src/crash_amplification.rs`、`crash_injection.rs`、`layer0_enumeration.rs`、`crates/singlefs-harness/src/fault_injection.rs`、`crates/singlefs-harness/tests/random_histories.rs` 的 `.min(16)` 等）照旧，登记在冻结副本 `.claude/heavy-thread-exempt` 的「代码批」行。
- 批 2 的钩子与脚本在主 agent 搭的集成副本上复核过：23 份自证绿、42 个新弄坏开关红在点名格；收尾补丁后又 16 项复核对（结果 `~/.local/share/singlefs/work/m3-tooling-b2-integration/verify.summary`、`verify2.summary`）。新门禁格 heavy-thread-defaults 在集成真树上绿：扫 831 份、可疑写法 90 处全在排除表（62 行）。
- 还没改成 31 的：变异门禁 `scripts/mutation/crates-mutation-rows.py` 的 `worker_count_for`（`min(16, 核数 // 2)`）与 `mutate.sh`（等依赖产物共用一起改，归课题 10 第 8 项）；E171 的 `rows_orchestrator.py`（被 sha256 钉住，进 E171 下一次修订）。
- 批 3 实现员自报的冲突：新规则写 GPU 测试二进制 `--test-threads 1`，而门禁 `checker-tier-layer0-replay` 带 `gpu` 特性那一趟（冻结副本 `.claude/gate.d/checker-tier-layer0-replay.sh` 第 760 行附近）套的是 `capped.sh default`，`RUST_TEST_THREADS` 会是 31、没带 `--test-threads 1`。
- 写者锁只管 Write / Edit / MultiEdit 工具，Bash 里的写看不见（批 2 实现员自报）。
- 批 3 实现员自己定的四处：删了共用约束「主 agent 发消息改了 N，从下一条命令起照新的」与「交回怎么读」里「线程上限 N 变了时收一条消息」；第 9 条改成挡着才协商；慢档用例补「没有这一行的经 `capped.sh default`」；执行前拒绝的射程照钩子文件头写。

## 三、七格要答的

### D1　线程数

规则说「默认 31、固定不看忙闲」之后，还有哪条可达的工作流里实际线程数不是 31 而规则让人以为是：`crates/` 的默认、变异门禁、E171、GPU 那一趟、`capped.sh N` 显式给的数、子 agent 自己起的命令没经 `capped.sh`。删掉「N 变了发消息」之后，用户要临时让出 CPU（例：别的会话的提交阶段在跑、请求别起满线程）时，主 agent 怎么把在跑的子 agent 的线程降下来，有没有一条规则还管这件事。派发闸 ⑮ 要求 N 等于登记值、否则写「线程上限已判」，这一句与「可以输入参数调整」是否一致。

### D2　verification 四句

四句与已有条款（「凡用 GPU 先清场，跑完不复原」、内存包装排队、「两档：轻型与重型」、门禁的实际跑法）有没有互相矛盾；「开满」与内存包装的准入在什么情形下打架；「20 万态量级」与 checker 档快档、轻型三条的现有域对不对得上。

### D3　只核自己的改动

「别人的改动不追查、不拦」在什么历史里会让真丢失（别的会话的改动被这一批的写冲掉、或这一批的改动被别人冲掉）没人发现；与上游 `.claude/singlefs-ai-sop/rules/session-wrapup.md` 第 4 条、`.claude/main-agent.md` 别的条款有没有冲突；按哈希登记「照现状」会不会把别人没审过的内容登记成豁免。

### D4　私有主机信息不进仓

射程写全没有（例：产物里打出的主机名、日志、`multi-host.env.example`）；与仓里已有文字有没有冲突；能不能做成会红的检查（只答形态，不改）。

### D5　执行前拒绝的写法与看门狗两样

定义文字说的拒绝与钩子代码（冻结副本里 `lib_heavy_tests.py` 的 `depends_on_checker_package`、`lib_workspace_cargo_lock.py`、`write-guard.sh` 第九道与 `lib_writer_lock.py`、`runner-dispatch-guard.sh` 的 ⑮ ⑯）逐条对不对得上：文字说拒的钩子不拒、钩子拒的文字没写、出路句照着做走不通。「开工先认锁」那一句照做时，Bash 里的写、子 agent 自己再派的子 agent、重启后的旧进程各会怎样。

### D6　叫停轮写 halted

能不能用一份 halted 文件把一个其实还要判的轮关掉（原话只要在某份 `.md` 里找得到就收）；派发闸 ⑥ 的判法与规则文字对不对得上；与 `research/prompts/abandoned-rounds.tsv` 那张表是不是同一件事两处登记。

### D7　长活不写死台数与额度

「派发提示里不写死台数与额度」与「派发提示必须写线程上限：31」「派发闸 ⑭ 要分片、续跑、多机三行」放在一起，照做的人写得出一份同时满足三条的提示吗。

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端正推（Sonnet）** | 逐条核「定义与规则写的就是钩子与脚本做的」 | D1、D5、D6、D7 | 每一句被改的定义文字，对着冻结副本里的钩子与脚本源码（文件:行）判「一致 / 不一致 / 文字没写到」，不一致的给出钩子的实际行为与能复现的喂法（喂 JSON 给钩子入口、不跑 `--selftest`） |
| **云端攻方（Opus）** | 造工作流历史打改后的形态 | D1、D2、D3、D4、D6 | 每条历史写成可复现的步骤（派发提示原文、钩子喂法、命令序列），在冻结副本的拷贝里真跑钩子与脚本的判定；交「照规则做、结果错」的那一步与错在哪 |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 改后的文字照留 | 两条腿在那一格都给不出不一致或判错的历史 |
| 一格判「站不住」⇒ 改定义、规则或钩子，改法并进下一批；照「第三轮之后停」 | 给出一条可达历史：照定义做却违反用户定案（默认 31、固定不看忙闲、只核自己的、私有信息不进仓），或定义说的拒绝钩子不拒、钩子拒的定义没写 |
| 一格的打中也打中改前的文字 ⇒ 不拿它否这次改动，另立一笔账 | 同一段历史在改前的定义上给出同样的错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b3-r1-<腿名>-output.md`（正推 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b3-r1-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。攻方的驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b3-r1-opus-model/`。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数。
- 不跑登记的 `--selftest`（子 agent 在副本里跑会被重型测试闸拒），核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数。
