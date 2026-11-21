# split-governance-hooks-gates-r1：判决（2026-11-20）

正文 `research/prompts/_split-governance-hooks-gates-r1-body.md`，开工快照 `research/prompts/split-governance-hooks-gates-r1-snapshot/`。这是这一题的第一轮；同一批拆分的脚本与服务在另一题 `split-governance-scripts-services-r1`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方（opus） | `research/prompts/split-governance-hooks-gates-r1-opus-output.md` | 第 1 问推不出（算打中）：写闸、派发闸、完工清理闸引导读不到部分文件时抛错退 1；`lib_shell_words` 部分模块按裸名字进 `sys.modules`。第 2 问打中：change-range-single-source 格扫描范围漏了 doc 门禁的 13 份库。第 3 问不成立。第 4 问打中：agent-write-scope 格按通配认族多认、`source_family` 把字符串点名的文件与共用库算进族 |
| 正推（sonnet） | `research/prompts/split-governance-hooks-gates-r1-sonnet-output.md` | 第 5、6、7 问都成立；附带：`project-code-guard.sh:587`、`:590` 说两条新开关「推的，没复跑」与 151 条复跑不符；正文「拆前原文在 HEAD」对 13 道入口不成立 |

核查命令：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py split-governance-hooks-gates-r1 --out /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/split-governance-hooks-gates-r1-tree --repository /home/fy5090/code/singlefs`：结局 type=command-failed exit=1。127 份里 25 份倒推不出（开工之后别的会话在工作区改过，diff 里没有基底）：`.claude/gate.d/` 下 `break-switch-replay-files.tsv`、`checker-independence-and-sync.sh`、`code-source-discipline.sh`、`code-tooling.sh`、`doc-process-records.sh`，`.claude/gate.d/lib/` 下 `code-source-discipline-mutation-and-reading-cells.sh`、`code-source-discipline-unsafe-and-registry-cells.sh`、`code-tooling-break-switch-replay-cells.sh`、`code-tooling-copy-closure-and-selftest-shape-cells.sh`、`code-tooling-fixtures-and-layout-cells.sh`、`code-tooling-selftest-cells.sh`、`code-tooling-shared-libraries.sh`、`code-tooling-source-scan-cells.sh`、`code-tooling-stage-break-switch-replay-cells.sh`、`code-tooling-unit-verdict-and-entry-cells.sh`、`doc-process-records-rounds-and-ledger-cells.sh`，`.claude/hooks/` 下 `bash-command-detector-scope-and-index.py`、`bash-command-detector-selftest-entries-and-main.py`、`bash-command-detector-selftest-waits-and-scope.py`、`bash-command-detector.sh`、`runner-dispatch-guard-selftest.py`、`selftest-registry.tsv`、`write-guard-decisions.py`、`write-guard-selftest.py`、`write-guard.sh`。攻方腿把读到的那一版存在它草稿目录的 `judged-at-reading/`。这几份上的打中（写闸那一条）主 agent 在此刻的树上复跑探针坐实，不靠快照那一版。
- `python3 scripts/agents/cite-check.py research/prompts/split-governance-hooks-gates-r1-opus-output.md research/prompts/split-governance-hooks-gates-r1-sonnet-output.md --root /home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/split-governance-hooks-gates-r1-tree --background research/prompts/_split-governance-hooks-gates-r1-body.md`：「✓ 核了 51 处引文、35 处裸行号，对上 44 处，没判 42 处（其中裸行号没有引文可比 17 处；逐处列在上面）」。没判的落在倒推不出的那 25 份与只写文件名的同名路径上；判决引到的几处主 agent 逐处在此刻主工作区里现读：`.claude/hooks/project-code-guard.sh:587`、`:590`，`.claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh:1434`。

主 agent 复跑（探针与模型目录从腿的草稿拷到 `/home/fy5090/.local/share/singlefs/work/split-governance-main-rerun/hooks-gates/`，sha256 与腿的相同，在那里交）：

- 写闸探针 `tools/probe-write-guard-part-missing.sh`：结局 type=succeeded exit=0；四臂读数「hooks-probe-intact exit=0 detections=0」「hooks-probe-scope-library-missing exit=2 detections=1」「hooks-probe-part-missing exit=1 detections=0 first=Traceback (most recent call last):」「hooks-probe-presplit-part-missing exit=0 detections=0」。完好臂与拆前臂都退 0（副本里没有 clerk 的定义，写范围表认不出这个 agent，这两臂判不了拒）；缺部分文件那一臂退 1 抛异常、缺拆前就有的依赖那一臂退 2 按拒判，这两臂分得开。
- 共用切词模块缓存探针 `tools/probe_shell_words_module_cache.py`：结局 type=succeeded exit=0；「arm=split-copy core_file=/home/fy5090/code/singlefs/.claude/hooks/lib_shell_words_core.py shell_tokens=[(False, 'a'), (False, 'b')]」「arm=presplit-copy shell_tokens=['副本']」。
- 拆分族认法探针 `tools/probe_source_family_overreach.py`：结局 type=succeeded exit=0；三行 `reported_unmarked=False`，`source_family` 与 `lag_family` 认出的族在前两行不同（「source_family=['scripts/gate/sample-quotes-sibling.py', 'scripts/gate/sample-own-checker.py'] lag_family=['scripts/gate/sample-quotes-sibling.py', 'scripts/gate/sample_quotes_sibling_part.py']」）。
- change-range-single-source 三臂（`python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/gate.d/code-tooling.sh <探针根> --check change-range-single-source`）：presplit 结局 type=command-failed exit=1，点名「.claude/gate.d/doc-experiments.sh:4687  listed = subprocess.run(["git", "ls-tree", "--name-only", base, "--", path], …」；postsplit 结局 type=succeeded exit=0；renamed 结局 type=command-failed exit=1，点名「.claude/gate.d/lib/code-tooling-doc-experiments-preregistration-and-replay-cells.sh:1434」。
- 副本里的 agent-write-scope 格（`bash .claude/gate.d/code-tooling.sh --check agent-write-scope <副本>`）：结局 type=command-failed exit=1，手抄行只报「.claude/hooks/shadow_shell_words.py:2  def shell_tokens（共用模块里定义在 lib_shell_words_core.py）」，同一份内容的 `lib_shell_words_shadow.py` 不报。
- 正推腿第 5 到 7 问要的读数主 agent 在主工作区交过：`lib_project_code_runs.py --selftest`、`project-code-guard.sh --selftest` 都 type=succeeded exit=0；开关复跑「复跑了 76 条开关（1 份），自证红且点名的格与实际红的格逐格相同 76 条」「复跑了 151 条开关（1 份）…逐格相同 151 条」；`--check gate-structure`「✓ 门禁阶段的结构都对：查了 17 道阶段、共 195 格」。

## 二、判决

| # | 打中 | 判 | 依据 |
|---|---|---|---|
| 1 | 三份内嵌 python 的钩子引导读不到部分文件时抛错退 1、照常放行（拆前写闸缺依赖按拒判；派发闸与检出钩子拆前缺依赖是放行加记检出，第二轮判决第 5 条） | 坐实而采（写闸实测；派发闸、完工清理闸与检出钩子的引导是同一种写法，`runner-dispatch-guard.sh:1014-1018`、`completion-cleanup-guard.sh:417-421`、`bash-command-detector.sh:1389-1393` 都是按写死的部分文件名单 open 再 exec、没有兜底，没单跑探针） | 写闸探针缺部分文件那一臂退 1 抛 Traceback、缺拆前依赖那一臂退 2 |
| 2 | `lib_shell_words` 部分模块按裸名字进 `sys.modules`，同一进程再从别的目录载入时用先载入那一份 | 坐实而采（今天仓里没有从两个目录载入它的调用方，是潜在缺陷） | 缓存探针 split-copy 臂 `core_file` 指回仓里、拆前臂交「副本」 |
| 3 | change-range-single-source 格扫描范围漏了 doc 门禁的 13 份库 | 坐实而采 | 三臂：拆前点名第 4687 行、拆后不报、改名落进通配又点名第 1434 行 |
| 4 | agent-write-scope 格按 `<主干>_*.py` 通配认族多认 | 坐实而采 | 副本里 `lib_shell_words_shadow.py` 的手抄不报、`shadow_shell_words.py` 的报 |
| 5 | `source_family` 把字符串点名的文件与被 import 的共用库算进族；三份认法认出的族不一样 | 坐实而采 | 认法探针两份认法的族不同；它那三行 `reported_unmarked=False` 分辨不了（探针登记理由太短、登记核对先于族检查退了），换成第二轮判决第 6 条辩方复跑的读数 |
| 6 | `project-code-guard.sh:587`、`:590` 说两条新开关「推的，没复跑」 | 坐实而采 | 151 条开关复跑已逐格相同 |
| 7 | 正文「拆前原文在 HEAD」对 13 道入口不成立 | 坐实，改的是正文的说法，不改治理文件 | 攻方腿报告「开工时对不上的两件」；拆前基线在各拆分草稿目录 |
| 8 | 第 3 问（替身换在被测命名空间上）、正推腿第 5 到 7 问 | 没打中，这一题的这几格停 | 两条腿都判成立或不成立且没给反例 |

## 三、改法

1. 写闸、派发闸、完工清理闸与检出钩子的引导：读不到、编不过任何一份部分文件时，照拆前缺依赖的处置按拒判（退 2、点名缺的是哪一份），不让 python 在 main 之前抛错退 1。自证加一格：删掉一份部分文件，钩子退 2。零轮。
2. `lib_shell_words` 的部分模块按带入口路径的唯一名字载入（或载入后核 `__file__` 与入口同目录，不同就报错），同一进程从两个目录载入不串。自证加一格同进程两目录载入。零轮。
3. change-range-single-source 格的扫描范围认全 `.claude/gate.d/lib/` 下的库（不只三道门禁的），样本加一份 doc 门禁库里不带 quotepath 的 git 调用必须报。mainagent(C) 接，并进它正在改这一格的那一件，验收是 `lib/doc-experiments-preregistration-and-replay-cells.sh:1434` 那一行（拆前 `doc-experiments.sh:4687`）被点名。零轮。
4. agent-write-scope 格（以 mainagent(C) 第 3 条那一件换上之后的 `code-tooling-rules-and-agents-cells.sh` 为基线）、`source_family`、`scripts/gate/lib_lag_split_family.py` 三份认族逻辑并成一份：`scripts/gate/lib_lag_split_family.py` 已认 .py 与 .sh 的拆法，但它按命名认（`<模块名>_<部分>.py`、`<名>-<部分>.sh|py`），照搬会带着第 4 条的多认；族的定义取两样的交集：名字落在 `lib_lag_split_family.in_family` 认的命名族里，且入口真载入它（import、source、按 `PART_NAMES` 一类写死名单 exec 的）。只按名字的多认（第 4 条）与只按载入的多认（第 5 条）在真仓上都排掉（第二轮判决第 11 条：探针造的几种布局里三种定义各有判错的）；`scripts/hosts/queue-health.py` 新换上的按 import 认一层的那一份也改调同一个函数。自证各加一格：名字像部分文件而入口没载入的不算族；被 import 的共用库不算族。零轮。
5. `project-code-guard.sh` 第 587、590 行改成现状：两条开关已在整份复跑里逐格相同。
6. 改完照「代码轮与治理文件轮怎么收」再攻一轮，攻这几处修补；第二轮派辩方腿复核这一份判决。

## 交用户的

无。

## 这一轮被判过的文件

- .claude/gate.d/break-switch-replay-files.tsv：开关复跑登记表：正推腿复数两行注释里的开关条数（76、151）与文件头现数相同
- .claude/gate.d/checker-independence-and-sync.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/code-source-discipline.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/code-tooling.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/crash-point-replay.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/doc-decisions.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/doc-experiments.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/doc-process-records.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/doc-registries.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/doc-text.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-background-gpu/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-background-gpu/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-checker/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-checker/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-core/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-core/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-format/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-format/src/tests.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-reference-filesystem/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-reference-filesystem/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-reference-filesystem/tests/reference_publish_reads_the_format_constant.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-verification/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/crates/singlefs-verification/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/expect：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-green/.gate-cells：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-background-gpu/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-background-gpu/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-checker/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-checker/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-core/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-core/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-format/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-format/src/plain.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-format/src/tests.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-reference-filesystem/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-reference-filesystem/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-reference-filesystem/tests/reference_publish_reads_the_format_constant.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-verification/Cargo.toml：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/crates/singlefs-verification/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/expect：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/checker-independence-and-sync.sh/checker-implementation-disjoint-cfg-test-module-file-red/.gate-cells：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/agents/alpha.md：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/agents/beta.md：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/gate.d/10-sample.sh：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/gate.d/stage-owners.tsv：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/agent-write-scope.tsv：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/copied-part-functions.sh：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/lib_project_code_runs.py：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/lib_shell_words.py：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/uses-shared-project-code.sh：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/hooks/uses-shared-shell-words.sh：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/settings.json：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/expect：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/.gate-cells：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/agent-write-scope-parts-red/scripts/devices/vm-bench.sh：agent-write-scope 新样本的一份：正推腿核它与 green 只差一个手抄部分文件函数的 hook，红因唯一；攻方腿第 4 问拿它对照通配认族多认的那一种
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/.claude/heavy-thread-default：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/.claude/heavy-thread-exempt：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/crates/demo/src/lib.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/crates/demo/src/tests.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/crates/demo/src/worker.rs：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/expect：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/.gate-cells：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-cfg-test-module-file-red/scripts/hosts/capped.sh：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/code-tooling.sh/heavy-thread-defaults-quoted-name-red/expect：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/fixtures/crash-point-replay.sh/unit-key-gpu-switch-not-in-key/expect：拆分与修补带进来的样本：正推腿核它跟着的那一格红绿判在名字说的格上，攻方腿第 1 问比拆前拆后同一份样本判得一样
- .claude/gate.d/lib/code-source-discipline-mutation-and-reading-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-source-discipline-naming-and-constants-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-source-discipline-unsafe-and-registry-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-break-switch-replay-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-copy-closure-and-selftest-shape-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-fixtures-and-layout-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-rules-and-agents-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里；第 2 问打中 change-range-single-source 格的扫描范围漏了 doc 门禁的库，第 4 问打中 agent-write-scope 格按 `<主干>_*.py` 通配认族多认
- .claude/gate.d/lib/code-tooling-selftest-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-shared-libraries.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-source-scan-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-stage-break-switch-replay-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/code-tooling-unit-verdict-and-entry-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/crash-point-replay-crash-case-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/crash-point-replay-feature-card-and-audit-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/crash-point-replay-package-pass-verdicts.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/crash-point-replay-selection-and-plan.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-decisions-evidence-and-owed-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-decisions-registry-and-history-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-decisions-status-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-experiments-index-and-evidence-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-experiments-preregistration-and-replay-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-experiments-verdict-and-citation-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-process-records-batch-and-review-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-process-records-rounds-and-ledger-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-registries-field-and-layout-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-registries-reference-and-invariant-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-registries-unit-header-and-debt-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-text-content-and-history-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/lib/doc-text-notation-and-translation-cells.sh：门禁拆出的格库：攻方腿第 1、2 问比拆前拆后的格函数与按文件读源码的地方，正推腿第 7 问核登记的格函数都住在入口或它 source 的库里
- .claude/gate.d/model-differential-and-scenarios.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/gate.d/qemu-device-streams.sh：拆过的门禁入口：正推腿第 7 问核格名表、`--list` 与 `stage_cell` 登记逐格相同，攻方腿第 1 问比拆前拆后入口的参数解析与汇总
- .claude/hooks/bash-command-detector-hooks-and-overwrites.py：钩子拆分族 bash 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/bash-command-detector-literals-and-signals.py：钩子拆分族 bash 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/bash-command-detector-scope-and-index.py：钩子拆分族 bash 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/bash-command-detector-selftest-entries-and-main.py：钩子拆分族 bash 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/bash-command-detector-selftest-waits-and-scope.py：钩子拆分族 bash 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/bash-command-detector.sh：钩子拆分族 bash 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/completion-cleanup-guard-decisions.py：钩子拆分族 completion 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/completion-cleanup-guard-selftest.py：钩子拆分族 completion 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/completion-cleanup-guard.sh：钩子拆分族 completion 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/lib_project_code_runs_launchers.py：项目代码判定模块拆分族（这一轮修补的一件）：正推腿第 5、6 问核 `source_family` 与两条新开关，攻方腿第 4 问打中 `source_family` 把字符串点名的文件与共用库算进族
- .claude/hooks/lib_project_code_runs.py：项目代码判定模块拆分族（这一轮修补的一件）：正推腿第 5、6 问核 `source_family` 与两条新开关，攻方腿第 4 问打中 `source_family` 把字符串点名的文件与共用库算进族
- .claude/hooks/lib_project_code_runs_selftest.py：项目代码判定模块拆分族（这一轮修补的一件）：正推腿第 5、6 问核 `source_family` 与两条新开关，攻方腿第 4 问打中 `source_family` 把字符串点名的文件与共用库算进族
- .claude/hooks/lib_run_scope.py：钩子拆分族 lib_run_scope 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/lib_run_scope_selftest.py：钩子拆分族 lib_run_scope_selftest 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/lib_run_scope_target_sources.py：钩子拆分族 lib_run_scope_target_sources 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/lib_shell_words_core.py：共用切词模块拆分族：攻方腿第 1 问打中部分模块按裸名字进 sys.modules、同一进程再从别的目录载入时用的是先载入那份，第 4 问拿它对照 agent-write-scope 格的认族
- .claude/hooks/lib_shell_words_embedded.py：共用切词模块拆分族：攻方腿第 1 问打中部分模块按裸名字进 sys.modules、同一进程再从别的目录载入时用的是先载入那份，第 4 问拿它对照 agent-write-scope 格的认族
- .claude/hooks/lib_shell_words.py：共用切词模块拆分族：攻方腿第 1 问打中部分模块按裸名字进 sys.modules、同一进程再从别的目录载入时用的是先载入那份，第 4 问拿它对照 agent-write-scope 格的认族
- .claude/hooks/project-code-guard-decisions.py：项目代码闸拆分族（这一轮修补的一件）：正推腿第 5、6 问核自证与开关，攻方腿第 1 问比拆前拆后判法
- .claude/hooks/project-code-guard-selftest.py：项目代码闸拆分族（这一轮修补的一件）：正推腿第 5、6 问核自证与开关，攻方腿第 1 问比拆前拆后判法
- .claude/hooks/project-code-guard.sh：项目代码闸拆分族（这一轮修补的一件）：正推腿第 5、6 问核自证与开关表，读出文件头两行说两条新开关「推的，没复跑」与 151 条复跑不符
- .claude/hooks/runner-dispatch-guard-locks-and-verdicts.py：钩子拆分族 runner 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/runner-dispatch-guard-prompt-checks.py：钩子拆分族 runner 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/runner-dispatch-guard-selftest.py：钩子拆分族 runner 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/runner-dispatch-guard.sh：钩子拆分族 runner 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/run-scope-guard.sh：钩子拆分族 run 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/selftest-registry.tsv：钩子拆分族 selftest-registry.tsv 的一份：攻方腿第 1 问比拆前拆后判法、exec 进同一命名空间的同名覆盖与次序，第 3 问核自证替身换在被测命名空间上
- .claude/hooks/write-guard-decisions.py：写闸拆分族：攻方腿第 1 问打中引导读不到部分文件时抛错退 1、照常放行（拆前缺依赖按拒判），第 3 问核自证替身换在被测命名空间上
- .claude/hooks/write-guard-selftest.py：写闸拆分族：攻方腿第 1 问打中引导读不到部分文件时抛错退 1、照常放行（拆前缺依赖按拒判），第 3 问核自证替身换在被测命名空间上
- .claude/hooks/write-guard.sh：写闸拆分族：攻方腿第 1 问打中引导读不到部分文件时抛错退 1、照常放行（拆前缺依赖按拒判），第 3 问核自证替身换在被测命名空间上

## 回看决策

不涉及决策：这一轮判的是治理文件按 2000 行拆分之后的行为与拆分带出的缺陷，`.claude/kb/decisions/` 下没有一条分项因此改动。
