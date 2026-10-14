# 工具与治理批的定义与规则改动：第二轮正文（2026-10-13）

<!-- doc-lint:not-numbers D1 D2 D3 D4 D5 D6 D7 G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 H1 H2 H3 R1 R2 R3 R4 R5 R6 R7 R8 R9 -->

## 一、这一轮要判什么

第一轮判决 `research/prompts/m3-defs-b3-r1-main-verification.md` 判 D1–D7 站不住或部分站不住，给了改法 G1–G11（全部被攻过零轮）；岔路单 `research/prompts/m3-defs-b3-r1-forks.md` 三条 H1–H3 用户已定。批 4（四件工具活：派发闸、重型测试闸与写闸、门禁格、定义文字）与批 4 收尾一件照改法与用户定案改完，调度记录 `records/2026-10-12-里程碑三工具与治理批.md`。这一轮攻这批修补本身。

照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」的第二轮分工：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。这一轮派攻方腿与辩方腿：辩方复核第一轮判决够不够得着、改法是不是对每一格都修到了；攻方攻改法本身与改法带进来的新机制。

被判的定义与规则文件（判决按路径点名）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`。

冻结副本：`~/.local/share/singlefs/work/m3-defs-b3-r2/repo/`（集成副本标签 `after-b4-fixup2` 那一版：主工作区 2026-10-12 那一版加批 2、批 3、批 4、批 4 收尾两件全部补丁，没有 `.git`）；被判文件与它们压着的钩子、脚本、门禁的 sha256 在 `research/prompts/m3-defs-b3-r2-snapshot/frozen-copy-sha256.txt`。第一轮的冻结副本（改前那一版）在 `~/.local/share/singlefs/work/m3-defs-b3-r1/repo/`，两份逐文件比就是这一批的改动。腿只读这两份冻结副本，不读主工作区（主工作区没有这几份补丁）。

| 格 | 第一轮改法与用户定案 | 改后的形态 | 在哪 |
|---|---|---|---|
| R1 | G1、H3 | 派发闸 ⑮：「线程上限：」后面只认一个十进制整数（后面只许接中文括注，括注里不许有数字、斜杠、「一半」），六种任务带重活的类型（implementation-writer、investigator、tooling-writer、experiment-runner、mutation-triage、three-way-attack）没写这一行拒；不等于登记值要另写「线程上限已判：理由」；在跑的子 agent 不改线程，只管新派的 | `.claude/hooks/runner-dispatch-guard.sh` ⑮；`scripts/agents/dispatch-prompt.py`；`main-agent.md`「派发提示怎么写」；`agent-common.md` 派发闸那条；六份定义「输入」 |
| R2 | G2、G3、G5 | 子 agent 起重活一律 `capped.sh N`、`capped.sh default` 只给主 agent 与脚本；`capped.sh` 设的变量补 `E161_THREADS`；门禁 code-tooling 的 heavy-thread-defaults 格加判装置读自己的线程变量没设时的回退、`capped.sh` 变量清单、shell 局部变量、测试模块 | `scripts/hosts/capped.sh`；`.claude/gate.d/code-tooling.sh`；`.claude/heavy-thread-exempt`；`agent-common.md` |
| R3 | G4 | 门禁 `checker-tier-layer0-replay` 带 gpu 特性那一趟与轻型 `crash-point-replay.sh --quick` 开了 GPU 的每一趟快档带 `--test-threads 1` | `.claude/gate.d/checker-tier-layer0-replay.sh`；`scripts/crash/crash-point-replay.sh`；`verification.md` |
| R4 | G6 | `verification.md`：「20 万态」限定为日常验证（harness 档与轻型三条）的崩溃状态数、不管提交与全量；「机器与卡都用」限定为驱动，单个 GPU 测试二进制只用本机点名的卡 | `verification.md` |
| R5 | G7 | 重型测试闸：`-p` / `--package` 先在当前目录与 `--manifest-path` 那一份的包名里找、再找工作区成员；`cargo bench` 同判；工作根底下仓副本里登记为 light 的自证放行（比副本里的登记行与主仓那一行） | `.claude/hooks/lib_heavy_tests.py`；`.claude/hooks/heavy-test-guard.sh`；`agent-common.md`「执行前拒绝的写法」 |
| R6 | G8（含主 agent 更正）、G9 | 写者锁照旧只认最近一层（批 4 乙改成嵌套判的那一版已撤回）；共用约束写明子目录另派之后父目录持锁者不写它；回执对不上时拒绝说明点名持锁者；「开工先认锁」只对 tools 里有 Write / Edit 的定义；看门狗「被停的同样算」 | `.claude/hooks/lib_writer_lock.py`；`.claude/hooks/write-guard.sh`；`agent-common.md`；`main-agent.md`「派出去之后」 |
| R7 | G10 | 叫停轮只登记在 `research/prompts/abandoned-rounds.tsv` 一处：派发闸 ⑥ 读它；出处必须是 `records/` 下现存文件的第 N 行（不带 `..`、解到真路径仍在 `records/` 之内），那一行里有轮名与「」括的 6 字以上原话；`research/prompts/m3-batch63-code-r1-halted.md` 删掉、改成 tsv 里一行 | `.claude/hooks/runner-dispatch-guard.sh` ⑥；`research/prompts/abandoned-rounds.tsv`；`three-way-inference.md` |
| R8 | G11 | ⑭ 的「多机：」行写台数取多机配置、每台线程取登记值，不现算；⑭ 判的类型扩到改法 G1 那份类型表；两份实验定义同改；`detached-run.sh`、`host-dispatch.py` 文件头说明两种读法各管哪一处 | `.claude/hooks/lib_long_run.py`；`runner-dispatch-guard.sh` ⑭；`dispatch-prompt.py`；`main-agent.md`；`experiment-designer.md` 3c；`experiment-runner.md` 1d；七份定义「输入」的「预计挂钟」 |
| R9 | H1、H2、第一轮 E3 | 门禁 doc-text 新格 `private-values`：这次改动新加的行里不出现 `multi-host.env` 里 8 个字以上的词与本机主机名，值不打印，没有配置退 77；`main-agent.md` 第 9 条：按哈希登记的豁免表遇到别的会话没审过的改动不登记；两处已推到远端的私有值只在工作区换掉 | `.claude/gate.d/doc-text.sh`；`main-agent.md` 第 9 条；`agent-common.md` |

## 二、实现今天的样子（主 agent 的观测，2026-10-13 现查）

- `crates/` 这一轮不改；`crates/` 里自己定并行度默认值的写法照旧登记在冻结副本 `.claude/heavy-thread-exempt` 的「代码批」行，改成取登记值归代码批。
- 批 4 与收尾的钩子、脚本、门禁在集成副本上复核（`~/.local/share/singlefs/work/m3-tooling-b2-integration/verify3.sh`，结果 `verify3.summary`）：派 133 项、收 133 项，16 项自证与门禁格、约 113 个弄坏开关按期望判；判错的 4 项分诊后，一项是真冲突（heavy-thread-defaults 红在 `runner-dispatch-guard.sh` 拒绝说明与自证样本里点名 `available_threads` 的 4 行），由批 4 收尾二在 `.claude/heavy-thread-exempt` 登记 4 行后那一格转绿（扫 831 份、可疑写法 94 处全在排除表、表 66 行每行都命中；冻结副本就是这一版，集成副本标签 `after-b4-fixup2`）；另三项是复核脚本把开关挂错了脚本、集成副本没有 git 历史、31 路并行时探工具链版本没回来，单独重跑都对。分诊逐条在调度记录。
- 双机驱动 `scripts/crash/crash-amplification-two-hosts.sh` 里 8 格「现算线程」与 threads-default-cores 开关这一批没改（不在冻结副本的改动里），等别的会话的提交落地之后改；E171 的 `rows_orchestrator.py` 被 sha256 钉住，固定 31 归 E171 下一次修订；变异门禁的 `worker_count_for` 等依赖产物共用一起改。
- 第一轮判决另立的两笔账没在这一批里修：`compile-then-swap` 只核开跑之后的改动；派发闸「快照冲突已判」理由非空就放行。

## 三、九格要答的

### R1　⑮ 线程上限

整数 + 中文括注的判法有没有绕得过的写法（全角数字、上标、零宽字符、括注里写别的量、两行「线程上限：」各写一个数、「线程上限已判」的理由写什么都收）；六种类型的表与「任务带重活」实际派的类型对不对得上（clerk、sweep、kb-scribe、three-way-forward、three-way-defense 有没有带重活而不判的派法）；H3「在跑的不动」写成文字之后，与「派出去之后」「交回怎么读」别的句子有没有冲突。

### R2　capped 与装置自己的线程变量

`capped.sh N` 设的变量清单与仓里装置实际读的线程变量逐个对得上吗（只补了 `E161_THREADS`，还有别的吗）；heavy-thread-defaults 格新加的四样判法在真树上会不会漏判或误判，排除表是不是只缩不涨。

### R3　GPU 那一趟 `--test-threads 1`

开了 GPU 时每一趟快档都带 `--test-threads 1`（纯 CPU 目标在 GPU 档下测试线程也是 1）：会不会让 CPU 目标的快档挂钟变长到超出别的条款、或与「默认 31」那一句互相拉扯；双机驱动与 admission 拼的 crash-case 命令没改，规则文字有没有说成已经全改了。

### R4　verification 两句

限定之后的两句与「凡用 GPU 先清场，跑完不复原」「两档：轻型与重型」、`.claude/main-agent.md`「禁止」第一条还有没有冲突。

### R5　重型测试闸的三处放宽与收严

「工作根底下仓副本里登记为 light 的自证放行」：能不能借一份仓副本把重型的东西伪装成 light 自证跑掉（改副本里的登记行、改副本里的脚本正文、副本里多一行同名登记、符号链接、副本不在工作根而在别处）；`-p` 先找当前目录那一份包名，探针包与工作区成员同名时判成什么。

### R6　写者锁

撤回之后「子目录另派之后父目录持锁者不写它」只是文字：照做与不照做各会怎样，钩子拦不拦；「开工先认锁」收窄到有 Write / Edit 的定义之后，Bash 里的写、重启后的旧进程有没有新的漏洞。

### R7　叫停轮只登记一处

伪造或误用出处能不能关掉一个还要判的轮（`records/` 里任意一行只要同时有轮名和 6 字以上「」句子就收：引别的轮的叫停原话、引用户别的话、同一行里两个轮名）；把 halted 文件删掉改登记，门禁 doc-process-records 的 abandoned-rounds 格与派发闸 ⑥ 读同一张表的判法一不一致（一处收、一处拒的行）。

### R8　长活三行与多机线程

「台数取多机配置、每台线程取登记值」与 `detached-run.sh`、`host-dispatch.py` 仍按 `host-resources.py` 现算的读法并存：照派发提示做的执行员最后跑出的是哪一个数；⑭ 扩到的类型里，哪些类型的派发其实从不带长活、却被逼写三行。

### R9　私有值与豁免表

`private-values` 格只判新加的行、按 8 字以上的词：值换了写法（大小写、拆开、URL 编码、写进文件名或路径）、写进 commit message、写进 `research/results/` 产物、写进被 git 忽略的文件之外的未跟踪文件，各判不判；输出会不会把值漏出来（红行、出路句、日志）；没有 `multi-host.env` 的机器退 77 会不会让整轮门禁说成「判过了」。第 9 条「有别人没审过的改动就不登记」：一批自己的改动与别人的改动在同一份文件里时，这一批怎么过 doc-process-records 的两格，规则给没给出路。

## 四、分工

| 腿 | 立场 | 格 | 要它交什么 |
|---|---|---|---|
| **云端辩方（Sonnet）** | 复核第一轮判决：改法 G1–G11 与用户定案 H1–H3 落到冻结副本里，是不是对第一轮打中的每一条历史都修到了（逐条拿第一轮两腿报告里的喂法重喂改后的钩子与脚本），有没有改过头（第一轮判对的行为被改坏）；第一轮哪一格判重了、判轻了 | R1、R3、R4、R6、R7、R8 | 第一轮两腿报告 `research/prompts/m3-defs-b3-r1-sonnet-output.md`、`research/prompts/m3-defs-b3-r1-opus-output.md` 与模型 `research/prompts/m3-defs-b3-r1-opus-model/` 里每条打中的历史，在改后的冻结副本上重喂，逐条交「修到 / 没修到 / 改过头」与喂法、退出码 |
| **云端攻方（Opus）** | 造工作流历史攻改法本身与改法带进来的新机制 | R1、R2、R5、R7、R9 | 每条历史写成可复现的步骤（派发提示原文、钩子喂法、命令序列），在冻结副本的拷贝里真跑钩子、脚本与门禁格的判定；交「照规则做、结果错」的那一步与错在哪；不重复第一轮已攻过的角度（第一轮报告里有的历史不再交，除非改后又中） |

## 五、跑前条款

| 条款 | 什么观测会让它触发 |
|---|---|
| 一格判「站得住」⇒ 改后的文字与钩子照留 | 两条腿在那一格都给不出没修到、改过头或新的判错历史 |
| 一格判「站不住」⇒ 改法进下一批；照「第三轮之后停」，第三轮只攻前两轮站住的形态 | 给出一条可达历史：照定义做却违反用户定案（默认 31、固定不看忙闲、只核自己的且豁免表不登记别人没审过的、私有信息不进仓、在跑的不动），或定义说的拒绝钩子不拒、钩子拒的定义没写 |
| 一格的打中也打中改前的文字 ⇒ 不拿它否这次改动，另立一笔账 | 同一段历史在第一轮冻结副本上给出同样的错 |
| 辩方报「改过头」⇒ 照「站不住」办 | 第一轮判对、改前钩子判对的一段历史，改后判错 |

⚠️ 打中之后先按 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」那四句过一遍。

## 六、各条腿交什么

- 方案按冻结副本今天的样子谈（`crates/` 照「实现今天的样子」那一节）。
- 报告先写进 `research/prompts/m3-defs-b3-r2-<腿名>-output.md`（辩方 `-sonnet-output.md`、攻方 `-opus-output.md`），每次工具调用写进文件的内容不超过 150 行，分段追加，第一段排他新建，回复只写短句。
- 草稿放腿自己的目录 `~/.local/share/singlefs/work/m3-defs-b3-r2-<腿名>/`；要改钩子或脚本做实验的，在那里另拷一份冻结副本改，不改冻结副本本身。攻方的驱动脚本与喂法随报告拷进 `research/prompts/m3-defs-b3-r2-opus-model/`，辩方的重喂脚本拷进 `research/prompts/m3-defs-b3-r2-sonnet-model/`。
- 冻结副本根下有本机私有配置 `multi-host.env`（被 git 忽略）：R9 要用它造样本可以读，报告、模型目录与回复里一个值都不许写出来（主机名、别名、路径、服务名都算），要指它们写「配置里 X 键的值」。
- 引规则、定义与 kb 条款写那份文件自己的行号，去冻结副本里现查，不从背景材料里数。
- 不跑登记的 `--selftest`，核钩子用喂 JSON 给钩子入口、核脚本用它的判法函数；门禁格（`code-tooling.sh`、`doc-text.sh`、`doc-process-records.sh` 的 `--check`）子 agent 起不了，读格的代码造出小树与命令写进模型目录，标「要主 agent 跑」，主 agent 写判决时在副本里复跑。
