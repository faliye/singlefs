# 工具与治理批的定义与规则改动：第二轮判决（2026-10-13）

<!-- doc-lint:not-numbers R1 R2 R3 R4 R5 R6 R7 R8 R9 A1 A2 A3 A4 B1 B2 B3 C1 C2 D1 D2 E1 E2 E3 G1 G2 G3 G4 G5 G6 G7 G8 G9 G10 G11 H1 H2 H3 H4 J01 J02 J03 J04 J05 J06 J07 J08 J09 J10 J11 J12 J13 J14 J15 K1 K2 K3 K4 K5 K6 K7 K8 K9 K10 K11 P1 P2 P3 P4 P5 P6 P7 P8 P9 P10 S21 S26 O04 O09 -->

正文 `research/prompts/_m3-defs-b3-r2-body.md`；腿：辩方 `research/prompts/m3-defs-b3-r2-sonnet-output.md`（sha256 9976f8c1…1fec，模型 `research/prompts/m3-defs-b3-r2-sonnet-model/`）、攻方 `research/prompts/m3-defs-b3-r2-opus-output.md`（sha256 42b53504…9126，模型 `research/prompts/m3-defs-b3-r2-opus-model/`）。第一轮判决 `research/prompts/m3-defs-b3-r1-main-verification.md`。岔路单 `research/prompts/m3-defs-b3-r2-forks.md`。

被判的定义与规则文件（按路径）：`.claude/main-agent.md`、`.claude/agent-common.md`、`.claude/rules/three-way-inference.md`、`.claude/rules/verification.md`、`.claude/agents/experiment-designer.md`、`.claude/agents/experiment-runner.md`、`.claude/agents/implementation-writer.md`、`.claude/agents/investigator.md`、`.claude/agents/mutation-triage.md`、`.claude/agents/three-way-attack.md`、`.claude/agents/tooling-writer.md`。

## 核查记录（主 agent 做）

- 开工快照：冻结副本 `~/.local/share/singlefs/work/m3-defs-b3-r2/repo/` 对 `research/prompts/m3-defs-b3-r2-snapshot/frozen-copy-sha256.txt` 30 份 `sha256sum -c` 全过（非 OK 0 行），腿跑着的时候没被改。
- `python3 scripts/agents/cite-check.py <两份报告> --root <冻结副本> --background research/prompts/_m3-defs-b3-r2-background.md`：「✓ 核了 104 处引文，对上 104 处，没判 0 处」。
- 攻方模型拷进主 agent 草稿目录，`find … | sha256sum` 与报告第 20–77 行逐行相同；经 `scripts/hosts/run-with-memory-cap.sh 4G` 跑 `tools/rerun.sh`（带 `FROZEN_ROUND1`）：八行 `name=compare … same`。
- 辩方模型拷进主 agent 草稿目录，sha256 与报告第 7 节逐行相同；同样经内存包装跑 `tools/rerun.sh`：九行里八行 same，`r7_abandoned` 一行不同，diff 只有第 29 行 note 里符号链接解到的绝对路径（腿的工作目录换成主 agent 的工作目录，那一处没换成 `<WORK>`），判定（`blocked_by_snapshot=True rc=2`）相同，按相同记。
- 门禁格由主 agent 在冻结副本上跑（带 `SINGLEFS_HEAVY_TESTS=commit`）：攻方 `tools/run-gate-samples.sh` 八份样本结局与腿推的逐份相同——`pv-big-untracked` 绿、`pv-filename-only` 绿、`pv-filename-and-content` 红且红行 `research/results/e999-sample-peer-box-17.out:1  键名：PEER_SSH_HOST、QUIESCE_STOP_COMMAND（值不打印）` 的路径里带着（编的）值、`pv-nul-head` 绿并报「没判的：…二进制 1 份」、`r7-date-not-iso` 红「第 1 行日期不是 YYYY-MM-DD：10月13日」、`r7-duplicated-rows` 红、`r7-no-source` 绿「登记撂下 1 轮」、`r2-thread-gaps` 只点名 `crates/demo/src/device_control.rs:2` 与 `scripts/experiments/own_control.py:3`；辩方 `feeds/make_gate_sample_roots.py` 四个样本根：gate-bad-date、gate-duplicated、gate-halted-only 红，gate-weak-source 绿，与腿推的相同。
- 第一轮两处私有值：主 agent 现查主工作区两份文件里 `PEER_SSH_HOST` 的值与本机主机名命中都是 0（只数不打）；冻结副本里还在，是集成副本取自用户 H1 定案之前的主工作区，不算没修。
- heavy-thread-defaults 格在冻结副本那一版（集成副本标签 `after-b4-fixup2`）上现跑绿（扫 831 份、可疑写法 94 处全在排除表、表 66 行每行都命中）。
- 核不动的：J07（带 gpu 特性那一趟 `--test-threads 1` 让 `--lib` 与 16 份测试文件全串行的挂钟）要跑带 gpu 特性的快档，是重型，没跑；C1 的后果（子 agent 真起了 checker 档测试）两腿都只喂了命令行，没执行。

## 逐格判决

| 格 | 判 | 依据（腿与主 agent 现查） | 去向 |
|---|---|---|---|
| R1 | 部分站不住 | 第一轮打中的 ÷、/、「一半」、阿拉伯数字的第二台、六种类型缺行都修到（辩方 S08、S09，改前→改后 0→2），没有改过头（辩方第 2 节十四组）。没修全或新带进来的：① 括注里写中文数字或比例词「31（本机；第二台十六）」「31（两台合计）」放行（攻方 A1、辩方 J15，量过），G1 要挡的就是这种，没修全；② 「线程上限：<登记值>」这类非 `<填` 的尖括号被当占位丢掉，绕过缺行拒（攻方 A2、辩方 J02，量过）；③ clerk、three-way-defense 定义许编译却不在类型表里，共用约束第 61 行「default 只给主 agent」与 `verification.md` 第 26 行、`capped.sh` 第 52 行「没有这一行用 default」三句互相矛盾（攻方 A3、辩方 J04；矛盾是 G2 带进来的）；④ investigator、tooling-writer 两份定义「输入」没写「线程上限：N」一行而钩子缺行就拒（辩方 J01，量过 0→2）；⑤ 「用户或别的会话要让出 CPU」与「机器忙也固定」同一行，主 agent 只看得到负载时分不出（攻方 A4，判别子观测不到）；H3「在跑的不发消息」与「派出去之后」两句发消息的条款并存（辩方 J09） | 改法 K1–K4 进下一批；「线程上限已判」理由非空就放行（J03）与 `SINGLEFS_THREAD_CAP` 通道（J12）另立账 |
| R2 | 部分站不住 | capped.sh 补的 `E161_THREADS` 到得了 E161（辩方 S01，1→0）。G3 新加的「读自己的线程变量」一路比格文件头判得宽：行首 `#[cfg(test)]` 只罩一条条目也整份 break、回退按整份文件找字样（攻方 B1、B2；主 agent 跑小树坐实：`device_after_cfg_test.rs`、`mentions_capped.py` 都没点名）。命令前缀赋值、`-j N`、`--test-threads N`、`:=N` 钉数格一种都不认，replay.sh 22 行（攻方 B3，改前也中） | 改法 K5；B3 另立账 |
| R3 | 部分站不住 | 门禁带 gpu 特性那一趟与轻型单机路径加上了 `--test-threads 1`（辩方 S04、O05）。新写的文字与实情不符：规则与 `crash-point-replay.sh` 文件头写成无例外，实际双机分支、`admission.py` 拼的 crash-case 命令、崩溃放量都没带（辩方 J05）；门禁注释与文件头写 CPU 一侧照登记值，实际测试读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`、没设取 `available_parallelism`（本机 32）（辩方 J06、O04）；`--lib` 与 16 份文件的纯 CPU 用例一起串行，挂钟没量（辩方 J07，推的） | 改法 K6；J07 提交时那一趟量挂钟再判 |
| R4 | 站得住 | 第一轮 B2、B3 的冲突修到（辩方 O06、O07）；残留 `verification.md` 第 60 行前半「开满默认线程」与第 59 行并存的读法歧义（辩方 J08 前半），harness 档 20 万态怎么压没写（J08 后半，推的） | J08 前半并进改法 K6 的文字；后半另立账 |
| R5 | 部分站不住 | `-p` / `--package` 先找当前目录、bench 同判修到（攻方九种写法、辩方 S10、S11，0→2）。G7 那一处放宽带进新漏洞：工作根底下仓副本里登记为 light 的自证，副本里正文改成起 checker 档测试照样放（攻方 C1，改后 0、改前 2），共用约束没写这一处只认登记行、不读正文。`env -C` 换目录起 cargo 放行（攻方 C2，改前也中） | C1 交用户（岔路 H4）；C2 另立账、并进下一批工具活（K7） |
| R6 | 站得住 | 撤回之后嵌套四格与第一轮逐条相同、回执对不上点名持锁者、「开工先认锁」收窄（辩方 S13–S17）；回执找不到时先到先得是改前行为、文字已如实写（J11） | J11 另立账 |
| R7 | 部分站不住 | 第一轮 F1–F3、D6 的历史全部修到（辩方 S19–S23，halted 引自己 / 正文 / 规则 / 仓外都 放→拦）。没修全：出处那一行同时写着别的轮的叫停原话（「用户叫停 r-other-r1：…；r-victim-r1 照常跑」）⑥ 照收，而规则第 121 行把别的轮的叫停原话列进 ⑥ 不收（攻方 D1，量过），是第一轮 F1 改后又中；同一张表 ⑥ 与门禁格判法不一（日期写坏、同一轮两行 ⑥ 收而门禁红；没写出处 ⑥ 拒而门禁绿，攻方 D2、辩方，主 agent 跑样本坐实）。开轮行、引规则行、「先放一放」能当出处，规则写了由主 agent 核，按文字没写到不算钩子错 | 改法 K8、K9 |
| R8 | 站得住 | ⑭ 扩到七种类型、多机行改读登记值（辩方 S24、S25）；表外类型只多写一行预计挂钟；额度判法（中文数字、「3 张卡」）改前也中（S26）；照提示做的执行员最后跑出的数取决于被起脚本与双机驱动（正文第二节已列的几处） | S26 另立账 |
| R9 | 部分站不住 | 格形态与第一轮 E3 一致、没有配置退 77 进「本次未跑」、第 9 条有 SendMessage 出路（攻方「没打中」、辩方 O12）。打中：新加的未跟踪文件只读前 8 MiB（攻方 E1；主 agent 跑小树坐实，仓里已有 10 份产物超过 8 MiB）；文件名里的值不判，文件名与内容都带值时红行经路径把值打出来（攻方 E2，坐实）；词源不读 `~/.config/singlefs/`（攻方 E3，推的，误报没量）；带 NUL 的文本整份按二进制跳过、在「没判的」里报了数（形状成立，主工作区 0 份，不算打中）；上游 `evidence-discipline.md` 第 28 行「一个字都不改」与共用约束「这一条优先」两层各说各的（辩方 J14） | 改法 K10；E3 照 K10 验收量了再并；J14 另立账 |

## 改法（全部被攻过零轮；进下一批工具与定义活，改完开第三轮）

| 改法 | 改什么 | 修的是哪一格 |
|---|---|---|
| K1 | ⑮ 括注：括注里出现中文数词（〇一二三四五六七八九十百千两半）、「分之」「合计」「减」「各」判红，只许说明性的词（攻方 P1 的后一种）；只把 `<填` 起头的当占位交 ⑬，别的 `<…>` 按认不出数拒（攻方 P2） | R1 ①② |
| K2 | 类型表 `THREAD_LIMIT_REQUIRED_AGENTS` 加上定义里许编译、跑模型的 clerk、three-way-forward、three-way-defense；`verification.md` 第 26 行与 `capped.sh` 第 52 行拒绝说明改成与共用约束第 61 行一致：子 agent 照派发提示的 N，`capped.sh default` 只给主 agent 与脚本内部 | R1 ③ |
| K3 | investigator.md、tooling-writer.md「输入」补「线程上限：N」一行，写法与 implementation-writer.md 第 27 行同 | R1 ④ |
| K4 | `main-agent.md` 第 53 行与共用约束第 61 行：「让出 CPU」限定成用户或别的会话在消息、弹窗里明说，只看到负载忙不算；「在跑的子 agent 不发消息」限定成「不为改线程发消息」，与「派出去之后」两句不冲突 | R1 ⑤ |
| K5 | heavy-thread-defaults：`#[cfg(test)]` 之后第一行是 `mod` 才跳过余下部分；回退判法看的是同一个变量名出现在回退式里，不按整份文件找字样；格的样本补两份（单条 `#[cfg(test)]`、文件另处提到 capped.sh），各配弄坏开关 | R2 B1、B2 |
| K6 | `--test-threads 1`：`crash-point-replay.sh` 双机分支同带；规则与两处文件头写明还没带的（admission 拼的 crash-case 命令、崩溃放量、双机驱动，与下一批那几处改动同时改成带）；门禁注释与文件头的「CPU 一侧照登记值」改成实情（读 `SINGLEFS_CRASH_JUDGE_CPU_THREADS`，没设取核数，归代码批）；`verification.md` 第 60 行前半限定为用例内部的 CPU 线程 | R3、R4 J08 前半 |
| K7 | 重型测试闸认 `env -C`、`env --chdir=` 换的目录（另立账那一条，攻方 P6，并进同一批工具活） | R5 C2 |
| K8 | ⑥ 出处那一行除这一轮的名字外不许有别的轮名（攻方 P7 前一半）；规则第 121 行写明开轮行、引规则行这类由主 agent 核 | R7 D1 |
| K9 | 门禁 abandoned-rounds 格与 ⑥ 共用一份登记行判法（日期、同一轮重复、出处三样两边都判）（攻方 P8） | R7 D2 |
| K10 | private-values：未跟踪文件整份读，超过上限判红不截断；改动范围里的路径也比；红行里路径命中私有值时打成「〔文件名含私有值〕」；词源加 `~/.config/singlefs/` 先在真树上量误报，新增误报 0 才并入，否则记账 | R9 E1、E2、E3 |
| K11 | 岔路 H4 用户定乙（「收回放宽」）：`lib_heavy_tests.py` 仓副本那一支另比副本里那份脚本与主工作区那一份 sha256 相同才放，不同的照旧按「解到的不是登记表登记的那一份」拒；共用约束「执行前拒绝的写法」与钩子文件头照改 | R5 C1 |

另立一笔账（改前也中，不拿来否这次改动）：「线程上限已判」理由非空就放行（J03，与第一轮「快照冲突已判」同形）；`SINGLEFS_THREAD_CAP` 设了时 ⑮ 与骨架照登记值（J12）；命令前缀赋值等钉数写法格不认、replay.sh 22 行（B3）；回执找不到时先到先得（J11）；合入锁子命令集合与 `cp -r` 带 `.git`（J10）；⑭ 额度判法（S26）；harness 档 20 万态怎么压（J08 后半）；上游证据规矩与共用约束私有值那一条两层各说各的（J14，上游只能照 `CLAUDE.md`「规范从哪来」填内容）。第一轮另立的两笔（S21、O09）照旧。

## 交用户（岔路单 `research/prompts/m3-defs-b3-r2-forks.md`）

- H4：工作根底下仓副本里登记为 light 的自证放不放、怎么放（攻方 C1）。用户 2026-10-13 定乙：「收回放宽」。

## 回看决策

不涉及决策：这一轮判的是 agent 定义、共用约束、项目规则的文字与钩子、门禁格的行为，没有一处引用或改动 `.claude/kb/decisions/` 里的分项。

## 更正（2026-10-13 第三轮判决）

这份判决里引的数有对不上产物的，以 `research/prompts/m3-defs-b3-r3-main-verification.md`「数字更正」一节为准，原句不改。
