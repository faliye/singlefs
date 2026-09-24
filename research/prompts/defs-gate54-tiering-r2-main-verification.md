# 三处定义改动与门禁 54 号分档：第二轮判决（2026-09-24）

<!-- doc-lint:not-numbers K1 K2 K3 S1 S2 S3 S4 S5 V1 V2 V3 V4 V5 -->

正文 `research/prompts/_defs-gate54-tiering-r2-body.md`，背景材料 `_defs-gate54-tiering-r2-background.md`，diff `_defs-gate54-tiering-r2-diff.md`，开工快照 `defs-gate54-tiering-r2-snapshot/sha256sums.txt`。判决只引产物与文件原文，腿的结论句当线索。

被判的定义与脚本（门禁 72 号按路径点名）：`.claude/gate.d/54-layer0-replay.sh`、`.claude/gate.d/stage-inputs.tsv`、`.claude/main-agent.md`、`.claude/agents/crash-verifier.md`、`.claude/agents/experiment-runner.md`、`.claude/hooks/agent-write-scope.tsv`、`.claude/rules/implementation-workflow.md`、`README.md`；这一轮判决新定要改的 `.claude/agents/gate-triage.md`（第四节第 6 条）。

## 一、这一轮交了什么

| 腿 | 格 | 报告 | sha256 |
|---|---|---|---|
| 云端辩方（Sonnet） | V5 | `defs-gate54-tiering-r2-sonnet-output.md` | 7c6c4a2f0b622b4b941596126df9809c0dd524e338570f67ed766a148a5890a9 |
| 云端攻方（Opus） | V1、V2 | `defs-gate54-tiering-r2-opus-output.md`，模型 `defs-gate54-tiering-r2-opus-model/` | bf9225572c4656cb3a7b868c7fd2d975b47c72ae8dc6b5e92786bd0756fda877 |
| 本地攻方 | V3、V4 | `defs-gate54-tiering-r2-local-attack.md`（改过一句之后 7b170cd1…）与样本 s1、s2（干净）、void1、void2（闸判红作废，起因是提示自己的 `AGREE` 标签与答案首词同名，见 `-runlog.md`） | 见核查员报告 |
| 核查员 | 全部 | `defs-gate54-tiering-r2-verifier-output.md` | 98f83eb21367f01551624353ce4bb9474ab89011443a6fe6af941cf373c43e59 |

核查员核了 62 处，✓ 57，✗ 3，分不清 2。攻方模型的 `run-all.sh` 在副本上重跑，9 个产物掩掉时间戳、临时目录后缀与 pid 之后与入库 `outputs/` 逐字节相同。

- **✗ 3 处都在本地腿的转述核对表**：FACT 1 的行号差 1（注释行与 `input_hash=` 实为 377、378），FACT 2 有两处把背景附录的行号（1013–1015）当成 54 号的行号（实为 248、249）。转述的**内容**我对着 54 号现行第 248–288 行逐句核过，与代码相符；行号错不改 V3 那几格的判定。
- **分不清 2 处是同一处漂移**：`.claude/main-agent.md` 那一行快照时在第 44 行，腿交齐之后挪到第 49 行，字面不变。
- **开工快照**：主 agent 腿交齐之后 `sha256sum -c`，11 份里 9 份 OK。`.claude/main-agent.md` 如上。`.claude/kb/checks-owed.md` 是主 agent 同日按用户定案写回 C495、C483 两行改的；三条腿对它按行号引用零处（核查员现跑 `grep`）。

## 二、跑前条款，各触发没触发

| 条款 | 触发没触发 |
|---|---|
| 一格判「站得住」：两条攻方腿在那一格都构造不出判错的历史（本地腿两次抽样一致） | **V1 的 f5 本身**：出路三行在「暂存区为空」「只有别人的东西」「第 1、2 行之间 HEAD 变了」三种现场下都没有误判（攻方 V1-a 到 V1-d）；V1-a（代码轮刚改完、没暂存，快档必红）是次序问题，与第一轮 S1 同类，已由 f5 与 `.claude/main-agent.md` 第 49 行管着，不另判 |
| 一格判「站不住」：攻方给出可达历史，该跑全量的没跑、不该红的红了、该红的没红 | **V1 H-A、V2 H-B、H-B2、H-C 触发**（攻方）；**V3 触发**（本地腿 V3DUPLICATE，两份样本一致，我读代码坐实）；**V4 触发**（本地腿 V4WRONGPATH，我读代码坐实，而且比腿说的更糟，见第三节）；**V5 触发一处**（辩方构造的「全量一次没跑、门禁分诊把红归成不是这一轮」） |
| V5 判第一轮某个打中「不分辨」⇒ 另立一笔账，不拿来判分档 | **没触发**：辩方逐条核了 S1、S3、S4、S5，分档之前的 54 号都不中（S5 的底层风险分档前就有，分档把它从「这一次判错」放大成「标记记下之后一直判错」，放大是分档特有的，判「分辨」照旧成立） |

打中之后照 `.claude/singlefs-ai-sop/rules/evidence-discipline.md`「判据自己也会写错」四句过了一遍：H-A、H-B、H-C 都分辨（pre 臂在同一段历史上判对）；判别子看得到（键里缺的那几样在做决定那一刻都算得出）；打中归的是「该红的没红 / 不该红的红了」的字面分句；每个改法对打中的格各核了一遍（第三节的表）。

## 三、逐格判决

### V1、V2　一格的键缺了判它的 54 号与工具链：站不住，改

攻方的三处打中病根是同一个：**一格的键只含登记的三条路径的内容**，判它的 54 号、工具链都不在键里；54 号第 11 行「同一批输入的结果是确定的」这条前提在这两样变了时不成立。`research/scripts/stage-must-run.sh` 第 64 行对复用判定立的是相反的规矩（「阶段脚本自己进比对：判据本身变了，上一次的判定就不再作数」），54 号没跟上。

| 历史 | 打中 | 修它的改法（攻方在副本上量过） |
|---|---|---|
| H-A 建完 worktree 之后别人暂存了一份更严的 54 号 | 4 个时刻里 3 个出一处误绿 | g1：判这一格的 54 号自己进输入哈希 |
| H-B 内容退回到有旧格的一版，旧格是另一份 54 号或另一版工具链写的 | 5 种用户动作里 2 种误绿 | g1（漂移 = 54 号）、g2（漂移 = 工具链） |
| H-B2 工具链升级之后照 `SINGLEFS_GATE_FULL=1` 重新验 HEAD | 带不带 `--staged` 都误绿 | g2 |
| H-C 同一批输入上另一趟 `--full` 判红，EXIT trap 删掉别人的绿格 | wip 与 edit 两种各 4 格里 2 格误红 | g1（wip）、g4（edit：「跑的过程中输入变了」那一支不删开跑那一格） |

没有哪一个单独修得了全部格；g124 三样一起在 12 格上全修（`outputs/fix-arms-g124.out`）。g1 的代价（攻方对照格）：X 那一趟是旧 54 号跑的，别人在它跑完之后暂存了新 54 号，门禁要求再跑一趟全量——这是「新 54 号没跑过」的正当红，不是误判。

**f5 本身站得住**，V1-c（出路第 1、2 行之间 HEAD 变了）只多白跑一趟全量、不误判；与共享 `gate.sh --staged`「套不上就停」的建法不一致，记进第五节，不改。

**结论**：g1、g2、g4 一起采纳。g2 的清单行取 `cargo -V` 与 `rustc -V` 两行原样（攻方量过的只有 `cargo -V`；多一行只会让更多情形作废旧格，判得更严或相同）。

### V3　快档核两行 `exhaustive=true`：站不住，改

本地腿九格，两份样本在判定列上一致。54 号现行第 270–279 行的判法是「计数行共 3 行」加「`^LAYER0B? ` 且带 `exhaustive=true` 的共 2 行」，两个数都只数总数、不分是哪一行：

- **V3DUPLICATE**：删掉 `LAYER0B` 那一行、把 `LAYER0` 那一行抄一遍，总数 3、`exhaustive=true` 2，快档判绿，而两次发布那条流一行都没有。我照第 270–286 行逐字推过一遍，腿的 TRACE 与代码相符。
- `CHECKER`、线程数、`input_file`、`finished_utc` 四样被改，快档照样判绿（V3CHECKER、V3THREAD、V3INPUTFILE、V3FINISHED）。这是 f4a 的射程本来就不罩的（第一轮判决第三节 S4 那一格只要挡 `exhaustive=false`），标记只由 `--full` 写、不进工作树；手改它要有意伪造，今天没有哪份定义会让 agent 去写它。记账，不改。

**结论**：改法 h1——快档分别数 `LAYER0 `、`CHECKER `、`LAYER0B ` 三种开头的行，各恰好一行，`LAYER0` 与 `LAYER0B` 两行各自带 `exhaustive=true`。这是本地腿打中、主 agent 定的收严，**被攻过零轮**。

### V4　登记表驱动的「碰没碰输入」：站不住，改

本地腿的转述把「文件集为空就判红」写在「碰没碰输入」之前，推出「路径全写错 ⇒ 54 号判红」。**代码的次序是反的**：快档先问 `stage-must-run.sh`、再问 `change-touches-crates.sh`（54 号第 79–94 行），这两问都在快档调 `write_layer0_input_manifest`（第 248 行）之前。我照代码推：

- 登记的路径**全写错**：写错的那一批改了 `stage-inputs.tsv`，复用那一问答「要跑」；`change-touches-crates.sh` 拿写错的前缀比，这次改动一条都不中，54 号退 77（「这次改动没碰它判的东西」）。之后每一批都一样退 77，层 0 从此一次都不跑，而 77 记「本次未跑」、不记红。
- **写错一条**（例如 `Cargo.lok`）：哈希只罩剩下的文件；之后只改 `Cargo.lock` 的一批，改动范围那一问答「没碰」，退 77。
- **整行删掉**：54 号第 66–69 行判红，`stage-must-run.sh` 答「要跑」。两边都不放行，没有误判。
- **改窄**（删掉一条路径）：哈希罩的文件变少，第一次在快档上没有那一格、判红要跑全量；之后被删掉的那条路径上的改动不进哈希、也不进改动范围。这是登记表本身写错，54 号照登记表办是设计；由改登记表的那一次代码轮三方与 72 号管，不另立检查。

**结论**：改法 h2——两问之前先核登记的每一条路径在 git 眼里至少列得出一个文件（`git ls-files -co --exclude-standard -- <那一条>` 非空），有一条列不出就判红，出路指到 `stage-inputs.tsv`。**被攻过零轮**。`stage-must-run.sh` 读同一张表，它的写错情形在 54 号这一问判红之后就轮不到，不另改。

### V5　第一轮判决：K2、K3 站得住，K1 的依据写错了一处，另补一处归属盲点

- **K2、K3 站得住**。S1、S3、S4、S5 都不打中分档之前的 54 号（第二节）。辩方指出第一轮判决引用户定案时掉了「主 agent」三个字（`records/2026-09-19-里程碑二遗留收拢.md` 第 147 行），判决方向不受影响；第一轮判决是冻结证据，不回改，更正写在这里。
- **K1 的依据写错了一处**。第一轮判决说 `.claude/agents/experiment-runner.md` 第 2 步「本来就写」变异表追加进 `crates/mutations.tsv`，辩方现查这一句在 `git log --all` 里零命中，是同一批未提交改动里的新文本。主 agent 现查：那一句与写范围表里 `crates/singlefs-harness/src/bin/e*.rs` 那一行，是 2026-09-23 `m2-runner-in-repo-harness-r1` 那一轮判过的（那份判决第 4 行点名 `.claude/agents/experiment-runner.md` 与 `.claude/hooks/agent-write-scope.tsv`，W2 那一格判的就是写范围放宽到 `crates/`）。**K1 照留的依据改成那一轮的判决**，不是「本来就写」。辩方说的第二处（写范围表的 harness bin 行在第一轮五份产物里零提及）由同一轮判决答了。
- **归属盲点（辩方构造，打中）**：主 agent 漏了第 49 行那一趟 `--full`、直接派 `gate-triage`；54 号快档报「没有这批输入的全绿标记」，红句点名的是 git common-dir 下的标记路径，不在暂存区 diff 里；`.claude/agents/gate-triage.md` 第 3 步按字面把它归成「不是这一轮」，主 agent 照 `.claude/agent-common.md`「不在的不修，照写」提交，这批 `crates/` 一次全量都没跑。f2、f3、f4a、f5 都改不到这一格（它们改的是 54 号里面）。**结论**：改法 h3——`gate-triage` 第 3 步加一条：红句说的是「这一批输入」本身（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类），而暂存区 diff 碰了 `stage-inputs.tsv` 登记给那一道的路径，归「这一轮」，下一步原样抄 54 号给的三行命令。**被攻过零轮**。

## 四、要落的改动（主 agent 定，不交用户）

1. `.claude/gate.d/54-layer0-replay.sh` 的 `write_layer0_input_manifest`：清单末尾加两行——判它的 54 号自己的 sha256（g1；脚本路径在 `cd "$ROOT"` 之前取成绝对路径），`cargo -V` 与 `rustc -V` 原样的 sha256（g2）。文件头「输入」一段跟着改。
2. 同一份的 `--full`：「跑的过程中输入变了」那一支判红时不删开跑那一格，trap 只在这一趟没写标记、也不是因为输入变了而退出时删（g4）；文件头第 11、12 行的说法跟着改。
3. 同一份的快档：`LAYER0`、`CHECKER`、`LAYER0B` 三种行各恰好一行，`LAYER0` 与 `LAYER0B` 各自带 `exhaustive=true`（h1）。
4. 同一份的快档：复用与改动范围两问之前，登记的每一条路径至少列得出一个文件，列不出判红（h2）。
5. 每一处先在副本里证明会红：g1、g2、g4 拿 `defs-gate54-tiering-r2-opus-model/fix-arms.sh` 以改后的真文件当 `POST_54_OVERRIDE` 重跑，与 `outputs/fix-arms-g124.out` 的门禁与真值退出码逐格对上；h1 喂一份 `LAYER0` 抄两遍、没有 `LAYER0B` 的标记，h2 喂一份登记路径写错的表，各自必须判红，改回去必须判绿。
6. `.claude/agents/gate-triage.md` 第 3 步（h3）：红句说的是「这一批输入」本身、暂存区 diff 碰了那一道登记的输入 ⇒ 归「这一轮」。

这六处都是攻方、本地腿或主 agent 提的收严，**被攻过零轮**。按 `.claude/rules/three-way-inference.md`「第三轮之后停」，第三轮只攻前两轮都站住的形态（f5 与第一轮的 K 判决）；这一轮新冒出来的零轮形态不再为它们开一轮，列在第五节。

## 五、交用户的与记账的

| 项 | 状态 |
|---|---|
| g1、g2、g4、h1、h2、h3 | 零轮，主 agent 采纳；落进真文件时自证会红（第四节第 5 条） |
| 标记里 `CHECKER`、线程数、`input_file`、`finished_utc` 被手改，快档照样绿 | 记账，不改：只有有意伪造才走得到 |
| 出路三行在第 1、2 行之间 HEAD 变了时照跑、白跑一趟全量；共享 `gate.sh --staged` 同一现场是「套不上就停」 | 记账，不改：不误判 |
| 登记表被改窄，删掉的那条路径从此不进哈希 | 不另立检查：登记表是唯一登记位，改它的那一次走代码轮三方与 72 号 |

这一轮一格都没有替用户定。

## 回看决策

不涉及决策：这一轮判的是门禁 54 号、`gate-triage` 与三份 agent 定义的工作流形态，没有动任何一条决策分项的定案、射程或依据，也没有新增或撤回实验结论。用户 2026-09-19 那条定案（层 0 全量挂在任务收尾）是这一轮的依据，不因这一轮改变。
