# defs-gate54-tiering-r1 核查员报告

你交的是观测，不是判决：核对表里的 ✗ 不免除主 agent 对推论的逐条现查。

草稿目录：`/tmp/claude-1000/defs54-verifier/`（`opus-model/` 复跑副本、`local-samples/` 本地腿样本副本、`selftest/` 判别力自证副本）。

## 判别力自证

挑的引用：sonnet 报告对 `records/2026-09-19-里程碑二遗留收拢.md` 第 147 行的引用（声称原文是「| 8 | 全量测挂在哪个节点 | 原话：「每次主 agent 执行完任务后统一执行」 | 收口表第 41 行：……」）。

在 `/tmp/claude-1000/defs54-verifier/selftest/2026-09-19-里程碑二遗留收拢.md`（该文件副本）里把行号人为加 1，改核第 148 行，原样输出：

```
| 4 | 块设备抽象加不加「写零」 | 加 | 并行线四：块设备抽象加写零动作，录制流登记成一种新步骤、层 0 认它 |
```

与引用声称的原文（第 8 问那一行）完全不同 ⇒ 判 ✗。核查方法能分辨，往下按第 2 步做。

## 输入核对

- 三份报告文件现在的 sha256 与交回里给的逐字相同（`sha256sum` 现算）：sonnet `c00b9f8a81efd2d409701ed879dde4424a5ef439e62d42d4025be0e99c90b0cd` 全字符一致；opus `adc411d8458ea2e10bfea9e3ebeec27a491342c2fb6474961d2f452cdaf4190f` 全字符一致；本地攻方四份样本与两份材料的哈希前缀（b29fe1d8、137117a6、7d32eb22、b32761e3、ac381fe7、ec399ade、cc1d83bd）逐一核对全部一致，交回时没有整份给出的部分核对前缀足够长（8 位）不构成误判空间。
- 快照 `research/prompts/defs-gate54-tiering-r1-snapshot/sha256sums.txt` 里 8 份文件，`sha256sum -c` 全部 OK，均与当前主树一致 ⇒ 这一轮对主树核等价于对快照核。
- 背景材料路径 `_defs-gate54-tiering-r1-background.md` 与正文路径 `_defs-gate54-tiering-r1-body.md`：`diff` 两份文件前 65 行，除末尾多出的分隔线外逐字相同——背景材料把正文原样嵌在开头，两者在这个范围内是同一份内容，行号重合。这一点在下表「正推」部分详记。

## 一、云端正推（Sonnet）：`defs-gate54-tiering-r1-sonnet-output.md`

这条腿没有复跑命令（自称「没有真跑任何门禁阶段」），全部是文件引用核对。逐条核（快照 8 份文件对主树核，其余现读主树；`records/` 那份不在快照清单内，单独现读）：

| 引用 | 核的结果 |
|---|---|
| `records/2026-09-19-里程碑二遗留收拢.md:147`「\| 8 \| 全量测挂在哪个节点 \|……」整行 | ✓ 逐字相同（`awk 'NR==147'` 现取） |
| `.claude/main-agent.md:40`「\| 改 `crates/`：条款已定……\| `implementation-writer`……\|」整行 | ✓ 逐字相同 |
| `.claude/main-agent.md:43`「一个阶段任务结束：……→ 下一行」整行 | ✓ 逐字相同 |
| `.claude/agents/crash-verifier.md:23`「1. 每个阶段开跑前……等它结束。」整行 | ✓ 逐字相同 |
| `.claude/agents/crash-verifier.md:24`「2. 按共用约束……不带只跑快档）；……」（省略号后半句未引） | ✓ 引的前半句逐字相同，用省略号截断是合理的整句节选，不是断章 |
| `.claude/gate.d/stage-inputs.tsv:11`「54-layer0-replay.sh\tcrates/ Cargo.toml Cargo.lock\t# 跑 crates 的崩溃点重放……」 | ✓ 逐字相同（含制表符分隔） |
| `.claude/rules/implementation-workflow.md:32`「改动之后要不要重跑某一道，判据是……不是「我改的是不是 `crates/`」。」 | ✓ 该行前一句逐字相同；该行后半句（「一道读的输入常常不止源码……」）未引，但引的是一句完整独立的话，不构成摘句断裂 |
| `.claude/hooks/write-guard.sh:93`「target = absolute if pattern.startswith("/") else relative」 | ✓ 逐字相同 |
| `.claude/hooks/write-guard.sh:94`「glob_to_regex(pattern).match(target)」（子串） | ✓ 是该行子串，该行原文 `if target is not None and glob_to_regex(pattern).match(target):`，未歪曲原意 |
| `.claude/gate.d/63-agent-write-scope.sh:7`「这个 agent 在表里有没有至少一条模式」（转述） | ✓ 该行原文「③ `.claude/agents/` 里 tools 含 Write 或 Edit 的每个定义，在 `.claude/hooks/agent-write-scope.tsv` 里至少有一条路径模式；」，转述准确、未加未减限定词 |
| 报告脚注：「背景材料「二、实现今天的样子」第 23 行提到的文件名 `.claude/hooks/agent-write-scope.sh` 在仓里已不存在（2026-09-17 与 `refuse-overwrite-untracked.sh` 合并改名为 `.claude/hooks/write-guard.sh`，该文件头 13 行自述此事）」 | ✓ 三重核实：① `.claude/hooks/` 目录下确实只有 `agent-write-scope.tsv`，没有 `agent-write-scope.sh`；② `write-guard.sh:13`「两道原本是两个 hook（refuse-overwrite-untracked.sh、agent-write-scope.sh），2026-09-17 用户定合并。」逐字对得上「该文件头 13 行自述此事」；③ 背景材料 `_defs-gate54-tiering-r1-background.md:23` 与正文 `_defs-gate54-tiering-r1-body.md:23` 两份文件在这一行逐字相同（背景材料把正文原样嵌在开头），所以这句引用无论算「背景材料第 23 行」还是「正文第二节」都同样成立，不是把正文行号误标成背景材料行号那一类问题 |
| 报告正文里的 grep 断言：「在 `.claude/gate.d/54-layer0-replay.sh` 里 `grep -n '正在跑\|已有\|lock\|flock\|ps -'` 零命中」 | ✓ 复核跑同一条命令，同样零命中 |
| `.claude/gate.d/54-layer0-replay.sh:219`「if [[ "$marker_input_hash" != "$layer0_input_hash" ]]; then」 | ✓ 逐字相同 |

**计数（正推）**：核了 13 处，✓ 13 处，✗ 0 处，核不动 0 处。

## 二、云端攻方（Opus）：`defs-gate54-tiering-r1-opus-output.md` + `defs-gate54-tiering-r1-opus-model/`

### 复跑

把 `research/prompts/defs-gate54-tiering-r1-opus-model/` 整份拷进 `/tmp/claude-1000/defs54-verifier/opus-model/`（不在腿的原目录跑），`nice -n 19 bash run-all.sh` 重跑全部 6 个场景。逐份 `diff`，把已知会变的字段（ISO 时间戳、`mktemp` 临时目录后缀、副本仓自己新建的 git 提交哈希前缀——这些仓每次都是 `git init` 现建，提交对象带时间戳，树内容不变但提交哈希必然逐次不同、S3c 的 pid）用 `sed -E` 掩掉之后再比：

```
s1-worktree-vs-staged / s2-trigger-gaps / s3-marker-slot / s4-marker-provenance / s5-transient-edit / s6-fix-arms
```

六份掩码后逐字节相同（S3 仅剩 pid 一处差异，属已知会变字段）。掩码前的差异逐处核对，全部落在「时间戳／临时目录名／git 提交哈希前缀／pid」四类里，没有一处判定行、退出码、内容哈希、计数行发生变化 ⇒ 复跑绿，且绿的不是摆设（判定与退出码本身完整重现）。

模型脚本与产物哈希：报告开头列出的 15 个 sha256（`lib.sh`、`run-all.sh`、6 个场景脚本、`fake-bin/cargo`、6 份原始 `outputs/*.out`）逐一用 `sha256sum` 现算，与报告原文逐字相同。报告里另列的「装置从主仓拷的被判对象」4 个哈希中，`54-layer0-replay.sh`、`stage-inputs.tsv` 两个与本轮快照 `sha256sums.txt` 逐字相同（已在「输入核对」一节核过）；`stage-must-run.sh`、`change-touches-crates.sh` 不在快照清单内，报告如实注明前者是工作区未提交版本、后者与 HEAD 相同，属如实披露，不算隐瞒。

### 表与「量过」原样输出的核对

| 引用/表 | 核的结果 | 命令 |
|---|---|---|
| 「各格判定一览」8 行（K3 触发条件×2、K3 工作区≠暂存区、K3 标记出处、K3 跑中输入改了又改回、K2×K3 标记只有一格、K2×K3 等待闸） | ✓ 8 行的「结果」「分辨」列与复跑的 `s1`–`s5` 各输出逐格对上 | 见下方各场景命令 |
| S2 表（lock-only/toml-only/test-only/mutations-only/other-session-crates 各 3 臂）15 格 | ✓ 全部 15 格退出码与复跑 `s2-trigger-gaps.out` 逐格相同；报告原样引用块（106 行「本阶段跳过……crates/ 底下」+ 退出码 77）与复跑输出逐字相同 | `cat -n outputs/s2-trigger-gaps.out` |
| S1「两样都有、暂存之后」原样块（4 行退出码/标记/内容不同/只在标记里） | ✓ 逐字相同（时间戳已知会变，已掩码核对） | `sed -n '76,90p' outputs/s1-worktree-vs-staged.out` |
| S3a 表（验证员跑过×3 时点、没跑×3 时点、pre 臂）7 格 | ✓ 7 格退出码（0,1,0 / 1,1,0 / 0）与复跑逐格相同 | `cat -n outputs/s3-marker-slot.out` |
| S3b 表（green/red × 4 种先后）8 格 | ✓ 8 格退出码（green: 1,0,0,1；red: 1,0,0,0）与复跑逐格相同，「B 红、A 跑完 B 才开跑」那格原样块逐字相同 | 同上 |
| S3c「命令行里含 gate.sh 的次数：0」 | ✓ 复跑同样为 0（pid 不同，属已知会变字段） | 同上 |
| S4 post/pre/f4a/f4b 4 组原样块（含 `exhaustive=false` 那一行） | ✓ 逐字相同（时间戳已掩码） | `cat -n outputs/s4-marker-provenance.out` |
| S5a/S5b 表（post/pre/f5 × 该红/该绿）6 格 | ✓ 6 格与复跑逐格相同：S5a（0,1,1）、S5b（1,0,0）——「post 该红未红」「pre 该红判红」「f5 各自修对」三句判词都对得上退出码 | `cat -n outputs/s5-transient-edit.out` |
| S6「S3a × f3」6 格、「S3b × f3」8 格、「S4 × f5」 | ✓ S3a×f3 退出码 0,0,0,1,1,0（只剩「没跑·开跑前」「没跑·途中」两格红），S3b×f3 全部 8 格退出码 0；S4×f5「层 0 不是全量」+ 标记不在 + 退出码 1，三处都与报告「量过」的断言逐字相同 | `cat -n outputs/s6-fix-arms.out` |

### 引规则/代码的行号核对（对主树，快照未覆盖的按当前工作区现读）

`.claude/main-agent.md:40`、`:43`；`.claude/agents/crash-verifier.md:23`、`:24`；`.claude/gate.d/stage-inputs.tsv:11`；`.claude/gate.d/54-layer0-replay.sh:8`、`:19`、`:66`、`:213`、`:218`、`:219`、`:225`、`:230`、`:246`、`:310`、`:311`；`.claude/agents/gate-triage.md:24`、`:25`、`:26`；`.claude/agent-common.md:41`；`.claude/agents/implementation-writer.md:27`；`.claude/gate.d/59-crates-mutation-replay.sh:6`；`research/scripts/stage-mine.py:165`-`166`；`research/scripts/change-touches-crates.sh:58`起的 `judge()` 与 `:100`-`110`附近的前缀匹配循环——**全部 25 处逐字核对，与报告原文引用逐字相同**，无一处行号或引文有误。

**计数（攻方）**：表格/原样块核了 8（各格判定一览）+15（S2）+1（S1「两样都有」原样块）+7（S3a）+8（S3b）+1（S3c）+4（S4）+6（S5）+15（S6 三组：S3a×f3 6 格、S3b×f3 8 格、S4×f5 1 格）=65 处；文件行号引用核了 25 处；哈希核了 15+4=19 处。合计 65+25+19=109 处；✓ 109 处，✗ 0 处，核不动 0 处。

## 三、本地攻方：`defs-gate54-tiering-r1-local-attack*.md`

### 复跑字词损坏闸

把提示文件与四份样本拷进 `/tmp/claude-1000/defs54-verifier/local-samples/`，对每份样本单独重跑 `corruption-check.py` 与 `oov-check.py`（单参数），与运行记录 `-runlog.md` 表格逐项比对：

| 样本 | corruption-check（现跑） | oov-check（现跑） | 与 runlog 声称的数是否相同 |
|---|---|---|---|
| s1 | 绿，`cjk=0 words=1661 …全部计数为 0` | 红，`生词=32 拼接=3`（`cratesating`、`cratesing`、`cratesatch` 均在生词表内） | ✓ 生词数、拼接数、拼接内容三者都与 runlog 表逐字相同 |
| s2 | 绿，`words=932` 全部计数为 0 | 红，`生词=28 拼接=3` | ✓ 数字相同 |
| s3 | 绿，`words=875` 全部计数为 0 | 红，`生词=24 拼接=4` | ✓ 数字相同 |
| void1 | 红，`cjk=0 words=1027…实词自复读=1`，复读词 `appending` | 红，`生词=23 拼接=2` | ✓ `words=1027`、`实词自复读=1`、复读词 `appending` 与 runlog 逐字相同 |

`grep -n 'cratesating\|cratesing\|cratesatch\|permits the'` 在 s1 里现查，确认 runlog 指出的三处具体损坏（第 3 行 `cratesating`、第 3 行「permits the ___ to」缺名词、第 37 行 `cratesing cratesatch`）在样本原文里真的存在，逐字与 runlog 描述相同。

四份样本的 `wc -w` 现数：s1=1685、s2=943、s3=893、void1=1033 词，与 runlog 表格「词数」列逐字相同（void1 那行 runlog 标「未统计」，本核查另行数出仅供交叉核对，不算 runlog 的错）。

### 转述核对表逐条核（`-translation-audit.md` 的 11 条 FACT 行号）

| FACT | 核对表给的文件:行 | 核的结果 |
|---|---|---|
| FACT 1 | `.claude/agents/experiment-runner.md:39` | ✓ 逐字相同（「入库装置的变异行……不改别人的行」括注） |
| FACT 2 | `.claude/hooks/agent-write-scope.tsv:13` | ✓ 逐字相同（三段制表符分隔） |
| FACT 3 | `.claude/hooks/write-guard.sh:10`-`11`，`decide_scope` 函数 `:75`-`98`，`:84` patterns 查找，`:92`-`95` glob 匹配循环 | ✓ 全部行号核对相同：`:10`-`11` 两行注释逐字相同；函数体 `:75`-`98` 范围核对相符；`:84` 正是 `patterns = load_scopes(...)`；`:92`-`95` 正是 `for pattern in patterns:` 到 `return 0, None` 那四行 |
| FACT 4 | `.claude/gate.d/33-mutation-tables.sh:76`-`77`，`:98`-`121`，尤其 `:114`-`118` FileNotFoundError 分支、`:119`-`121` hits!=1 分支 | ✓ `:76`-`77` 注释逐字相同；`:114`-`118`、`:119`-`121` 两个分支的起止行号精确对应实际代码（`try:`/`except FileNotFoundError:`/`continue` 为 114-118，`hits=`/`if hits!=1`/`print` 为 119-121） |
| FACT 5 | `.claude/gate.d/59-crates-mutation-replay.sh:4`-`7`、`:33`-`43`、`:92`-`106`、`:129`-`139`；`.claude/gate.d/33-mutation-tables.sh:11`-`12` | ✓ 五段范围逐一现读，内容与核对表转述一致（判据注释、复用判定、锚点腐化检查、拷贝范围预检查、33 号头部说明 59 号耗时几个钟头） |
| FACT 6 | `research/prompts/_defs-gate54-tiering-r1-body.md:30` | ✓ 逐字相同（「几个实现员在各自的仓副本里做、交补丁……门禁修复会话按表批量改名」） |
| FACT 7 | `.claude/agents/crash-verifier.md:23` | ✓ 逐字相同 |
| FACT 8 | `.claude/agents/crash-verifier.md:24` | ✓ 逐字相同 |
| FACT 9 | `.claude/agent-common.md:48`-`50` | ✓ 逐字相同（含核对表明确标注「有意略去」的两段操作细节） |
| FACT 10 | `.claude/main-agent.md:43` | ✓ 逐字相同（核对表明确标注「it does not itself rerun the full layer 0 replay」是从「只核这个标记」显式化出来的否定句，不是新事实——判读合理） |
| FACT 11 | `.claude/gate.d/54-layer0-replay.sh:8`-`9`、`:34`、`:243`、`:308`-`337` | ✓ 各段逐字核对相符（含 `rm -f -- "${full_green_marker_path:?}"` 在 `:243`、`marker_being_written` 先写临时文件再 `mv` 在 `:308`-`337` 范围内） |

**核对表本身的两张附表**（「首稿缺的/改动的」「英文比原文多出来的限定词」）逐条核：FACT 3 补回的两条放行分支（无 agent_type / 无项目定义两种放行）、FACT 4 补回的「文件不在也算腐化」分支、FACT 11 补回的「不问复用、不问改动范围」一句，均现查确认原文确有这些限定词、核对表补得对；FACT 6 两段历史加的操作细节（补丁按顺序应用、批量改名连带改多行）如实标注为「加了原文没有的括注」并给出理由，未伪装成原文——符合「多出来的也要列」的要求。

### 两份干净样本（s2、s3）逐格「一致/不一致」

runlog 明确没做这一步（「未判多次抽样之间答复方向是否一致——那是主 agent 的事」），这里逐格现读 s2.md、s3.md 的九个标签自己比：

| 标签 | 一致/不一致 | 观测 |
|---|---|---|
| K1SCOPEGATE | 基本一致，侧重不同 | s2 说「gate 不区分新增/修改，两者都放行」；s3 说「代理定义限制只追加，因此实际发生的是追加、没有修改」。两者对「合规追加不触发红」这个结论不矛盾，但 s2 额外指出「若被改写，gate 33/59 会抓」，s3 未提这一半 |
| K1PATCHCOPY | **不一致** | s2：认为写操作经过写范围闸、各副本各自只追加自己的新行；s3：认为 PATCHCOPY 里的写是经 shell 命令做的（引 FACT 3「不拦 Bash 里的写」），写范围闸完全不介入。两者对「谁在管这次写」给出了不同的机制解释 |
| K1GATE33 | 一致 | 两份都是「原文命中一次才过，否则红」 |
| K1GATE59 | 一致 | 两份都是「原文命中一次且测试如期红才过」 |
| K1BATCHRENAME | 基本一致，侧重不同 | 结论都是「gate 33/59 只在批量改名产生坏数据时才红，写范围规则本身管不到」，但 s2 从「谁有写权限」入手，s3 从「批量改名是另一个 agent 的事，与实验执行员的新规则无关」入手 |
| K2OWNRULE1 | 一致 | 两份都是「crash-verifier 等待、不冲突、没有门禁会报」 |
| K2LONGWAIT | 一致 | 两份都是「后台起、不设超时不杀、没有门禁会报」 |
| K2MAINSYNC | **不一致** | s2：明确给出一个会导致 54 号判红的条件（「若 crash-verifier 的运行在整轮门禁核验之前删掉了标记」）；s3：断言「不会同时跑，因此标记只在跑的过程中缺失，核验时不缺失」，即认定没有风险。s2 承认有条件性冲突路径，s3 排除了这条路径 |
| K2REDMARKER | 一致 | 两份都是「开跑删标记、跑完才写、判红或被打断都不留标记」 |

九格里 7 格一致或侧重不同但不矛盾，2 格（K1PATCHCOPY、K2MAINSYNC）方向性不一致——K2MAINSYNC 这一格与 Opus 攻方腿实测出的 S3a/S3b 撞车问题（同一个标记槽位被后来的 `--full` 覆盖或删除）指向同一个疑点，本地模型两次抽样一次判「有条件性冲突」、一次判「不会冲突」，印证了这一格确实存在争议空间，不构成对本地腿方法的质疑（`ask-local.sh` 判定两份都字词干净，损坏闸不适用于此）。

**计数（本地攻方）**：字词损坏闸复跑核了 4 处（s1、s2、s3、void1，各自 corruption-check + oov-check 两项，共 8 项子检查全部相同），转述核对表核了 11 处 FACT 行号 + 2 张附表（放行分支补回、多出限定词说明），九格一致性比对 9 处。合计 4+11+2+9=26 处；✓ 24 处，「不一致但不算错」2 处（K1PATCHCOPY、K2MAINSYNC，这是观测本身，不是核查判红——两份样本各自的字词与行号引用都通过了核对，只是两次抽样的回答方向不同，按 `.claude/singlefs-ai-sop/rules/three-way-inference.md`「一条腿只抽一次样不算一次观测」处理，不计入 ✗），✗ 0 处，核不动 0 处。

## 四、攻方腿是否用到正推腿的报告内容

时间戳现查：`stat -c '%n %y'` 显示 sonnet 报告落盘于 `2026-09-23 23:22:20`；Opus 的模型脚本落盘于 `23:25:35` 起，`run-all.sh` 与 `s6-fix-arms.sh` 落盘于 `23:27:45`，Opus 报告本身落盘于 `23:34:26`——sonnet 报告确实先于 Opus 全部产物落盘，具备被读到的时间窗口。

`grep -in 'sonnet'` 对 Opus 的报告与模型目录全部文件零命中；进一步用 sonnet 报告里特有的措辞（「缺哪一句」「规则没说」）去 `grep -c` Opus 报告，同样零命中。Opus 报告「没做什么」一节自称「派发没给禁读清单，按「无」办；读过的东西都在本仓与派发点名的 `/tmp/claude-1000/gate54-tiering/` 里」。

这只是负面证据：没搜到痕迹不等于确证没读过。Opus 与 sonnet 在 K2、K3 撞车这件事上给出的相似洞察（「碰撞检测只挡得住另一个 `gate.sh`，挡不住主 agent 直接调用 54 号脚本本身」）两条腿各自独立复述了同一句 `crash-verifier.md:23` 与命令行不含 `gate.sh` 这件事，但这是从同一份共享正文（`_defs-gate54-tiering-r1-body.md`「四、分工」表已经点出 K2 要答的问题包含「它与……K3 会不会撞在一起」）与同一份源码各自能推出的观测，不构成需要读对方报告才能得出的证据。

**结论**：字面证据（关键词检索）显示没有污染；无法排除「读过但未留文字痕迹」这种更弱的可能性，这是检索方法本身的边界，不是判决。

## 没做什么

- 不判一条打中成不成立、该不该采纳，不判改法 f2–f5 该不该被写回定义，不判 K1PATCHCOPY / K2MAINSYNC 两处本地腿分歧到底哪一次答对——这些是主 agent 的事。
- 不核推理本身（三方各自的「四句」分析是否合乎逻辑），只核引用、产物与复跑。
- 没有编译 Rust、没有跑真的 `gate.sh` 全量、没有跑 QEMU 或 herd7——三条腿本身也都没有跑这些，核查范围与它们一致。
- 没有重新审计 Opus 装置脚本本身有没有 bug（只核了「装置产出的判定是否可复现、报告是否如实转述装置输出」，没有从零审计假 cargo 与 `lib.sh` 的每一行逻辑）。
- 没有核 `.claude/rules/implementation-workflow.md` 之外的规则文件是否还有别的相关条款漏引——三条腿各自的分工表已限定射程，本核查按它们各自声明的射程核。
- 没有派发或催促主 agent 判决；这份报告只是观测清单。

报告文件：`/home/fy5090/code/singlefs/research/prompts/defs-gate54-tiering-r1-verifier-output.md`
草稿目录：`/tmp/claude-1000/defs54-verifier/`
