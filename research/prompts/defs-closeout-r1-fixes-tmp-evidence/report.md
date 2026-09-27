# defs-m2-closeout-r1 改法 F1–F16 落地报告

写于 2026-09-26 01:4x UTC（JST 10:4x）。依据：`research/prompts/defs-m2-closeout-r1-main-verification.md` 第二、三节；原文、行号与探针取自 `research/prompts/defs-m2-closeout-r1-opus-output.md`（O1–O15）与 `research/prompts/defs-m2-closeout-r1-sonnet-output.md`（D2-e、D3-b）。
改前备份：`/tmp/claude-1000/defs-closeout-r1-fixes/before/`（12 份，开工时 `cp -p`）；改后全量 diff：`/tmp/claude-1000/defs-closeout-r1-fixes/my-changes-final.diff`（287 行，本报告末尾原样附上）；改前 / 改后 sha256：同目录 `start-sha256.txt`、`end-sha256.txt`。
只动了放行的 12 个文件；没 checkout / restore / reset / clean，没提交。两个脚本（74 号、弹窗闸）都是写同目录临时文件、`chmod --reference`、`mv` 换上。

## 一、F1–F16 逐条

| F | 文件与小节 | 改了什么 |
|---|---|---|
| F1 | `.claude/agents/three-way-attack.md` 第 3c 步；`.claude/agent-common.md`「不做」一节「长活可以等」一条与「执行前拒绝的写法」④ | 3c：并行几件各写 `{ <命令>; echo "$?" > <草稿目录>/<名字>.rc; } &`，最后单独一个不带参数的 `wait`，按起的次序逐个读 `.rc`；不用 `wait "$pid"`。④ 后面补同一种写法作出路。「长活可以等」里 `&` 那一句补「每件的退出码照 ④ 的写法收」 |
| F2 | `three-way-attack.md` 第 3b 步最后一条 | 删「限时」：每批段数按先跑一小段估的时长定，不设包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`、不套 `timeout`；报告写段数与估时 |
| F3 | `three-way-attack.md` 第 3b 步「崩溃点」那条 | 照层 0 枚举域：调每一段都展开的 `enumerate_layer0` / `enumerate_layer0_versions`（屏障与 FUA 切段、段内整写子集逐个枚举，不是只截前缀），每个崩溃状态恢复后都跑 checker；缩只缩历史条数与长度、候选与几何，留下的每段历史崩溃状态一个不落 |
| F4 | `agent-common.md`「不做」一节第 46 行重型测试括注；`three-way-attack.md` 第 3b 步新加一条 | 括注里「层 0」改成「名字含 `layer0` 的测试二进制与 54 号」。3b：原型里调枚举函数跑小流不算重型——测试目标名字不带 `layer0`；每次跑前用 `closed_form_state_count` 算状态数，不超过约 10⁶（几段合起来超过就分几次跑）；一段自己就超过的不跑、交主 agent |
| F5 | `.claude/agents/experiment-runner.md` 第 4c 步 | 点名对象加上：字段名表示违例、不匹配、歧义、失败的计数（名字里带 `violation`、`mismatch`、`ambiguous`、`fail`，例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）取值不是 0 的；「一个都没有」那句同步改 |
| F6 | `.claude/agents/three-way-local-defense.md` 开头一段与「文件名形态」一条 | 读攻方「输入」「做什么」「写范围」「产出」四节；文件名形态的括注改成「攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换」 |
| F7 | `experiment-runner.md` 第 4 步 | 删「跑前删旧输出」：`research/results/` 已有的一个都不删、不挪、不覆盖；这一次写新文件名（今天日期或 `rN`，写前 `ls` 确认不存在），按新文件名认完成标记；重跑时新文件与已有的逐字节一致就删掉这一次的新文件、对不上两份都留 |
| F8 | `.claude/agents/crash-verifier.md`「输入」加一项、第 1b 步；`.claude/main-agent.md`「派发提示怎么写」加一段 | 输入：54、55、57 各自的内存上限，55 号不小于 `MODES` 档数 × `VM_MEM`；1b 上限改取输入给的。main-agent：派崩溃验证员给三道上限（算法指到崩溃验证员「输入」），派门禁分诊员给 `gate.sh` 的上限 |
| F9 | `.claude/agents/gate-triage.md`「输入」加一项、第 1b、2 步 | 输入加「`gate.sh` 整条经内存包装的上限」；1b 删 `gate.sh` 那半句「外面不包」，改成整条经包装、上限取输入给的，指到包装文件头「包装里再经包装跑的」那一句；第 2 步命令写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/scripts/gate.sh --staged`。单跑门禁阶段照旧直接跑 |
| F10 | `.claude/agents/implementation-writer.md` 第 4 步；`.claude/gate.d/74-model-differential.sh` 第 46–47 行 | 第 4 步「交回前的验证只到这几样」加进「阶段归属表登记给你的门禁阶段（74 号在内）」，「门禁阶段都不跑」改成「其余门禁阶段…都不跑」。74 号出路的单跑命令改成 `bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture`，前一行写明上限取派发提示给的、没给用 `replay.sh` 的 `REPLAY_MEMORY_CAP` 默认值 8G |
| F11 | `agent-common.md`「执行前拒绝的写法」Bash 检出 hook 那一行 | 写成「② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒」 |
| F12 | `agent-common.md`「执行前拒绝的写法」 | 引导句改成「项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝…只挂在一方才用的工具上的在条目里写明」；补五条：交回闸 `handback-scratch-check.sh`、收工闸 `gate-reuse-check.sh`、弹窗闸 `ask-user-claim-guard.sh`、派发闸 `runner-dispatch-guard.sh`、续做闸 `continuation-guard.sh`（现查见第二节） |
| F13 | `.claude/hooks/ask-user-claim-guard.sh` 的 `FILE_AND_LINE`、文件头判法第 22 行、自证 | 新加 `FILE_NAME_CHARACTER = [\w.-]`；`文件:行号` 里最后一个 `/` 之后的文件名、以及不带 `/` 的「文件名.扩展名」认非 ASCII 字；`/` 前面紧挨的字仍要 ASCII（仓里没有非 ASCII 的目录名，现查见第三节）。自证加 6 格 + 1 例 stdin |
| F14 | `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」开头清单；`main-agent.md`「禁止」第 4 条 | 清单改成按类的列表，逐类与 `lib_heavy_tests.py` 的 `classify` 对齐（现查与探针见第四节），另写「只认命令位置、包装里的同样算、命令位置上执行的脚本读进去逐行判（例 `fetch-deps.sh --check` 第 81 行）、门禁阶段与 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判」。main-agent 第 14 行删六类括注与「等」，「全单以」改「完整清单以」；第 20 行本来只有指过去的一句，没动 |
| F15 | `.claude/agents/mutation-triage.md`「输入」 | 加一项：给的是 `crates/mutations.tsv` 里的条目时，崩溃验证员跑的那一次 59 号的输出路径 |
| F16 | `experiment-runner.md` 第 6、7 步 | 第 6 步：84 号除外，放到第 7 步写完实验页之后跑；这一次不写实验页的，在第 6 步最后跑、红了照写。第 7 步末尾加「写完（连同第 7b 步）跑 84 号，贴原样末行与退出码」 |

## 二、F12 现查：settings 里注册着、会退 2 的 hook

`python3` 读 `.claude/settings.json` 的 `hooks` 键，原样：

```
PreToolUse Agent|Task bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/runner-dispatch-guard.sh
PreToolUse SendMessage bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/continuation-guard.sh
PreToolUse Bash bash "$CLAUDE_PROJECT_DIR"/.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh
PreToolUse Bash bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/bash-command-detector.sh
PreToolUse Bash bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/heavy-test-guard.sh
PreToolUse Write|Edit bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/write-guard.sh
PreToolUse AskUserQuestion bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/ask-user-claim-guard.sh
PreToolUse SubagentHandback bash "$CLAUDE_PROJECT_DIR"/.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh
SessionStart startup|resume|compact bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/session-start.sh
PostToolUse Write|Edit|Bash bash "$CLAUDE_PROJECT_DIR"/.claude/hooks/kb-scribe-followup.sh
Stop  bash "$CLAUDE_PROJECT_DIR"/.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh
SubagentStop  bash "$CLAUDE_PROJECT_DIR"/.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh
SubagentStop  bash "$CLAUDE_PROJECT_DIR"/.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh
```

逐份 `grep -nE 'exit 2\b|return 2\b|sys\.exit\(2'` 与读判定函数：会退 2 的 9 份——`runner-dispatch-guard.sh`（第 415、433、450、461、464 行 `return 2`）、`continuation-guard.sh`（第 142、148 行）、`pattern-process-guard.sh`（第 220 行 `exit 2`）、`bash-command-detector.sh`（第 2351–2453 行各处 `return 2`）、`write-guard.sh`（第 77、93、102、139、153 行）、`ask-user-claim-guard.sh`（改后第 162 行 `return 2`）、`heavy-test-guard.sh`（`decide` 里第 578 行 `Decision(2, …)`，`main` 返回它的 code）、`handback-scratch-check.sh`（第 106 行 `return 2`）、`gate-reuse-check.sh`（第 83 行 `return 2`）。不退 2 的：`session-start.sh`（只有 `sys.exit(0)` 与自证的 0/1）、`kb-scribe-followup.sh`（`sys.exit(0)`，文件头写只记不拦）。清单里原有 4 道，补上其余 5 道。

## 三、F13 弹窗闸：自证改前、改后与证红

| 时点 | 自证结果（原样） |
|---|---|
| 改前（备份那一份） | `✓ 自检通过（查了 23 种）：…三例走真实的 stdin 入口`，exit=0 |
| 只加新格、判法没改 | 4 格 ✗，exit=1（原样见下） |
| 改判法之后 | `✓ 自检通过（查了 30 种）：…中文文件名没带行号、中文词后面跟冒号与数的照拒；四例走真实的 stdin 入口`，exit=0 |

只加新格时的原样输出（`/tmp/claude-1000/defs-closeout-r1-fixes/f13-selftest-before.txt`）：

```
  ✗ 自检：必放：不在反引号里、文件名是中文的文件:行号 应当是 []，实际 ['records/2026-09-16-subagent拆分提案.md:951 写着静态分支一定要走定义三方']
  ✗ 自检：必放：多层目录下中文文件名的文件:行号 应当是 []，实际 ['.claude/kb/decisions/08-核心索引结构.md:40 写着这一格一定不变']
  ✗ 自检：必放：不带 / 的中文文件名，全角冒号行号 应当是 []，实际 ['见 08-核心索引结构.md：40，这一格一定不变']
  ✗ 自检：stdin：中文文件名的文件:行号放行 应当是 (0, True)，实际 (2, False)
    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest
exit=1
```

新加的 3 格「必拒」证它们挡得住放得过宽的判法：把改后那一份拷进草稿目录（`mutant-*/`），只换 `FILE_AND_LINE`，跑拷贝自己的 `--selftest`（它的 stdin 例调的也是拷贝），原样（`f13-mutants.txt`）：

```
== unicode-dirs
  ✗ 自检：必拒：斜杠分开的中文词跟冒号与数不是文件:行号 应当是 ['抓到/无效/没红：40/0/0，这张表一定全抓了']，实际 []
exit=1
== loose-colon
  ✗ 自检：必拒：中文词后面跟冒号与数不是文件:行号 应当是 ['拆分提案第四十节：40 行写着它一定要走三方']，实际 []
  ✗ 自检：必拒：斜杠分开的中文词跟冒号与数不是文件:行号 应当是 ['抓到/无效/没红：40/0/0，这张表一定全抓了']，实际 []
exit=1
== no-line
  ✗ 自检：必拒：中文文件名没带行号 应当是 ['records/2026-09-16-subagent拆分提案.md 里写着静态分支一定要走定义三方']，实际 []
  ✗ 自检：必拒：斜杠分开的中文词跟冒号与数不是文件:行号 应当是 ['抓到/无效/没红：40/0/0，这张表一定全抓了']，实际 []
exit=1
```

（unicode-dirs：目录名也放开非 ASCII；loose-colon：`\S+[:：]\d+`；no-line：行号变成可有可无。每行 ✗ 后面紧跟的 `→` 行这里略去，内容同上。）

仓里没有非 ASCII 的目录名，所以 `/` 前面紧挨的字仍只认 ASCII：`git -c core.quotepath=off ls-files | awk -F/ 'NF>1 { for (i=1;i<NF;i++) print $i }' | sort -u | LC_ALL=C grep -P '[^\x00-\x7F]'` 输出为空；非 ASCII 的都在文件名上（同一条 ls-files 里带非 ASCII 的 385 个）。

攻方的 A1–A4 原样重喂改后的闸（攻方的 `probe.py`，另加一例 A5 无出处）：

```
A1	ask	主 agent	fg	exit=0	
A2	ask	主 agent	fg	exit=0	
A3	ask	主 agent	fg	exit=0	
A4	ask	主 agent	fg	exit=0	
A5	ask	主 agent	fg	exit=2	✗ 弹窗里有 1 句带断言词，同一句里却没有出处，也没写明是推的——这样送到用户面前，推断就被当成了事实：
```

A1、A3 改前是 exit=2（攻方报告 D3 节原样）。A5 是「拆分提案第四十节：40 行写着静态分支一定要走定义三方」。

## 四、F14 现查：`heavy-test-guard.sh` 今天实际拒的

判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`（第 284 行起）与 `cargo_use`（第 158 行起），放行表在 `heavy-test-guard.sh` 的 `AGENT_KINDS`。新清单按这两个函数逐类写，不照判决举例抄。用攻方的 `probe.py` 喂主 agent（不带前缀、前台）的 JSON，只看退出码，一条都没执行（`probe/f14-cases.json`、`probe/f14-out.txt`），原样：

```
K1	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：herd7（herd7）：主 agent 跑「herd7」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K2	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：qemu-system-x86_64（QEMU）：主 agent 跑「QEMU」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K3	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：直接执行名字含 layer0 的测试二进制（second_transaction_step_zero_layer0）（层 0）：主 agent 跑「层 0」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K4	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：在工作区根（/home/fy5090/code/singlefs）上不带 -p / --test / --lib / --bin 的 cargo test（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-req
K5	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：在工作区根（/home/fy5090/code/singlefs/research）上不带 -p / --test / --lib / --bin 的 cargo test（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 
K6	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test 不挑目标，而包的范围是整个工作区（/home/fy5090/code/singlefs/research 的全部 1 个成员），等于全量（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-r
K7	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K8	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：research/scripts/mutate.sh 跑 crates/mutations.tsv 整表（crates 变异整表）：主 agent 跑「crates 变异整表」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K9	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：.claude/scripts/lkmm.sh（herd7）：主 agent 跑「herd7」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K10	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：gate.sh --selftest（整轮门禁）：主 agent 跑「整轮门禁」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K11	heavy	主 agent	fg	exit=0	
K12	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：research/scripts/vm-bench.sh（--selftest 也起虚机）（QEMU）：主 agent 跑「QEMU」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K13	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo run --bin e152-file-system-benchmark（E152 装置）：主 agent 跑「E152 装置」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K14	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：e152-run.sh（E152 装置）：主 agent 跑「E152 装置」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K15	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test 不挑目标，会跑到名字含 layer0 的测试二进制（first_transaction_step_seven_layer0、second_transaction_parallel_line_one_layer0、second_transaction_paral
K16	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test --test second_transaction_step_zero_* 命中名字含 layer0 的测试二进制（second_transaction_step_zero_layer0）（层 0）：主 agent 跑「层 0」要带 SINGLEFS_HEAV
K17	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：脚本 /home/fy5090/code/singlefs/.claude/scripts/fetch-deps.sh:81 里的 herd7（herd7）：主 agent 跑「herd7」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request
K18	heavy	主 agent	fg	exit=0	
K19	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：check.sh（里面是全量 cargo test）（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K20	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：门禁 87 号（87-replay.sh）（全部实验复跑）：主 agent 跑「全部实验复跑」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K21	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test --test second_transaction_step_zero_layer0（层 0）：主 agent 跑「层 0」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K22	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：门禁 54 号（54-layer0-replay.sh）（层 0）：主 agent 跑「层 0」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K23	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：门禁 57 号（57-lkmm.sh）（herd7）：主 agent 跑「herd7」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K24	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：门禁 59 号（59-crates-mutation-replay.sh）（crates 变异整表）：主 agent 跑「crates 变异整表」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K25	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：门禁 55 号（55-qemu-first-transaction.sh）（QEMU）：主 agent 跑「QEMU」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K26	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test 带 --workspace / --all（全量测试）：主 agent 跑「全量测试」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K27	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：research/scripts/gate-staged.sh（跑 gate.sh --staged）（整轮门禁）：主 agent 跑「整轮门禁」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带
K28	heavy	主 agent	fg	exit=2	✗ 重型测试被拒：cargo test 不挑目标，会跑到名字含 layer0 的测试二进制（first_transaction_step_seven_layer0、second_transaction_parallel_line_one_layer0、second_transaction_paral
K29	heavy	主 agent	fg	exit=0	
```

K 号对应的命令（cwd 是仓根，K5、K13、K29 是 `research/`，K6 是 `research/e7-index-bench/`）：K1 `herd7 -version`；K2 `qemu-system-x86_64 --version`；K3 `./target/release/deps/second_transaction_step_zero_layer0-0123456789abcdef`；K4、K5、K6 裸 `cargo test`；K7 `cargo t --workspace`；K8 `mutate.sh … crates/mutations.tsv`；K9 `lkmm.sh --herd7-version`；K10 `gate.sh --selftest`；K11 `gate-staged.sh --selftest`（放）；K12 `vm-bench.sh --selftest`；K13 `cargo run --release --bin e152-file-system-benchmark`；K14 `e152-run.sh`；K15 `cargo test -p singlefs-harness --tests`；K16 `--test 'second_transaction_step_zero_*'`；K17 `fetch-deps.sh --check`；K18 `cargo test -p singlefs-harness --lib`（放）；K19 `check.sh`；K20–K25 门禁 87、`--test …layer0`、54、57、59、55；K26 `cargo test --all`；K27 `gate-staged.sh`；K28 `cargo test -p singlefs-harness`（不挑目标、包里有 layer0 目标）；K29 `cargo test -p e7-index-bench --lib`（放）。

新清单里每一类都有一格拒的：裸 `herd7` 与只取版本号（K1、K9）、`qemu-system-*`（K2）、直接执行 layer0 测试二进制（K3）、工作区根裸跑（K4、K5）与在唯一成员里裸跑（K6）、`cargo t`（K7）、不挑目标而包里有 layer0 目标（K15、K28）、通配命中（K16）、脚本读进去（K17）、`gate.sh --selftest`（K10）。放行的三格对上清单里的「不算」：K11、K18、K29。

## 五、F10：74 号新建议命令喂 `heavy-test-guard.sh`

`probe/f10-cases.json`、`probe/f10-out.txt`，原样：

```
F10-new-implementation-writer	heavy	implementation-writer	fg	exit=0	
F10-new-crash-verifier	heavy	crash-verifier	fg	exit=0	
F10-new-gate-triage	heavy	gate-triage	fg	exit=0	
F10-new-main	heavy	主 agent	fg	exit=0	
F10-old-implementation-writer	heavy	implementation-writer	fg	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（implementation-writer 不经 run-with-memory-cap.sh）
F10-old-crash-verifier	heavy	crash-verifier	fg	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（crash-verifier 不经 run-with-memory-cap.sh）
F10-old-gate-triage	heavy	gate-triage	fg	exit=2	✗ 不经内存包装跑编译出来的代码被拒：cargo test（gate-triage 不经 run-with-memory-cap.sh）
F10-old-main	heavy	主 agent	fg	exit=0	
```

new 是 74 号改后打印的那条（`$TEST_BINARY` 代成 `second_transaction_supplement_three_random_history`）：四个身份都放行；old 是改前那条，三个子 agent 都被「不经内存包装」拒（改前 H23 同样）。

## 六、F1：新写法喂 `bash-command-detector.sh` 与 `heavy-test-guard.sh`

`probe/f1-cases.json`、`probe/f1-out.txt`，原样：

```
F1-rc-files	detector	three-way-attack	rib	exit=0	
F1-rc-files-fg	detector	three-way-attack	fg	exit=0	
F1-capped-detector	detector	three-way-attack	rib	exit=0	
F1-capped-heavy	heavy	three-way-attack	rib	exit=0	
F1-old-3c	detector	three-way-attack	rib	exit=0	
```

检出记录（`probe/probe-detections-9dkizimv.jsonl`）只有一条，命令开头是 `nice -n 19 bash …/a.sh > /`，即 F1-old-3c（改前 3c 的逐个 `wait "$pid"`），内容 `['run_in_background 里又自己放后台（nohup / setsid / disown / 后面没有 wait 的 &）：完成通知当场发出，跑完的那个不会叫醒你']`；F1-rc-files、F1-rc-files-fg、F1-capped-detector 三条新写法一条检出都没写。F1-capped-heavy：`{ … run-with-memory-cap.sh 8G cargo test … ; echo "$?" > … ; } & … & wait` 对攻方腿放行（经包装的认得出）。

## 七、门禁判定行（原样，都在最后一次改动之后或不受它影响的时点跑，`nice -n 19`）

| 门禁 | 原样判定行 | 退出码 |
|---|---|---|
| 47（第一次，01:3x UTC 起跑） | ✗ bash research/scripts/mutate.sh --selftest 没过（退出码 1）：<br>→ 怎么办：按上面那份自证给的下一步修被测脚本，再单独跑这条命令看它转绿。 | 1 |
| 47（第二次，01:45:33 UTC 跑完时 1 分钟负载 16.35） | ✓ research 脚本的自证都通过（本阶段跑了 31 条；research/scripts/ 里声称有 --selftest 的 35 份中 34 份有门禁阶段在跑） | 0 |
| 62 | ✓ 阶段归属表与门禁目录一致（76 个阶段，归 9 个 agent） | 0 |
| 63 | ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，按模式找进程的上游钩子注册着、文件在，会话开始 hook（压缩后提示与启动时的 OOM 报告）注册着、自证通过，书记官写入后的核对 hook 注册着、自证通过，表与定义一致（3 个有 Write 或 Edit 的定义、18 条路径模式），共用切词模块的 14 个函数只在 lib_shell_words.py 里定义（查了 .claude/hooks/ 下另外 12 个文件），共用重型测试判定模块的 15 个函数只在 lib_heavy_tests.py 里定义（查了 .claude/hooks/ 下另外 12 个文件；selftest、load_sibling_module 不算，见 NOT_SHARED_JUDGMENT） | 0 |
| 73 | ✓ 门禁自检通过：84 个脚本（.sh 与 .py）、260 条拒绝都带了出路<br>✓ 查了 72 个脚本：.sh 都可执行，暂存区里的模式与工作区一致<br>✓ 进程安全：查了 168 个脚本（.sh 127 个、.py 41 个），发信号的写法都只打得到点名的一个进程；own-scope 标注放行 1 处（research/scripts/run-with-memory-cap.sh:952）；没判的：.claude/process-safety-pending 里的 2 个文件 research/scripts/agent-watch.py（research/scripts/agent-watch.py:2162）；research/scripts/mutate.sh（research/scripts/mutate.sh:351）<br>✓ shell 纪律检查通过（共 34 个脚本）<br>✓ shell 纪律检查通过（共 8 个脚本）<br>✓ shell 纪律检查通过（共 9 个脚本） | 0 |
| doc-lint（文档铁律） | ✓ 文档铁律检查通过（检查 485，跳过 0；DOC_LINT_VERBOSE=1 看全部） | 0 |
| 规则纪律（项目本地，rules-lint 照 gate.sh 的 RULES_LINT_DIR / RULES_LINT_FILES 起） | ✓ 规则只写怎么做（扫了 29 份文件 1794 行；没扫 0 个；记录小节 0、论证小节 0、带日期的行 0（另有 8 行的日期只在「」或反引号里）、解释性段落 0、解释性半句 0、没带劝阻句的链接 0；词法说明判了 1671 行（围栏与表格行不判），命中 0；使用者名字这一条无对象可判：被扫的仓没有 I18N 或没登记 consumers=） | 0 |
| gate-overlap（门禁查重） | ✓ 相对 97f5904b44cd：新加的门禁与钩子 1 个（.claude/gate.d/84-verdict-false-named.sh）都写明了比过谁，改过的 14 份脚本对照已有的 132 份没有整段相同 | 0 |

47 号第一次红在 `research/scripts/mutate.sh --selftest` 的一格（自证用例里两条变异「2 秒没跑完」，原样见 `gate47.log`）；第二次同一道全绿。`mutate.sh` 与 47 号这一轮都没碰（47 号在工作区里是别的会话改的 ` M`）。第一次跑完之后一分钟内另一个会话在跑 `run-with-memory-cap.sh 24G … cargo test`，`uptime` 的 1 分钟负载 55.30（32 核，01:42:37 UTC）；判成负载下的超时是推的，没复现。两次一红一绿，按 test-discipline 记「不稳定」，不算这一轮的红，也不算两次都绿。
62、63 号是最后一处改动（共用约束弹窗闸那一条补全放行词）之后重跑的；73、gate-overlap 在那之前跑，那一处改的是 `.md`、不在它们的射程里；doc-lint 与规则纪律在最后一处改动之后重跑。63 号那一行里「弹窗断言闸…自证通过」就是改后弹窗闸的 `--selftest`。

## 八、没做的与原因

- D4（重型闸的静态分支、版本号白名单）与 84 号门禁（O5 的门禁那一半）：判决第四节延后，没碰。`.claude/scripts/fetch-deps.sh --check` 今天照旧被拒（第四节 K17），F14 只把它写进清单当例子，没改闸。
- `bash-command-detector.sh` 自己给 ④ 的拒绝出路没改：不在放行文件里。F1 只改定义与共用约束两处文字。
- 共用约束第 46 行的重型测试括注只按 F4 换了「层 0」那一词，没改成指到 workflow 清单；它写的是类名（QEMU、herd7 等），不自称全单。
- 74 号出路里写死了 8G（取 `research/scripts/replay.sh` 第 24 行 `REPLAY_MEMORY_CAP="${REPLAY_MEMORY_CAP:-8G}"` 的默认值）；峰值表里这个测试二进制整跑的几次在 3.7–5.1 GB（`research/scripts/memory-peaks.tsv` 第 170、300、341 行，键是 `capped.sh … cargo test --offline -p singlefs-harness --test …`、没带 `--release`，口径含页缓存，那几次上限 16G / 24G）。8G 够不够没实跑，推的。
- 没跑重型测试，没编译，没执行被探的任何一条命令（探针只喂 JSON）；没跑 72 号。

## 九、给第二轮的线索（推的，没跑）

- F16 同形的问题不止 84 号：执行员登记里的 40 号（`research/results/` 的产物要在 `experiments.md` 点名）与 86 号（`research/` 下的 `eNN` 要有 kb 正文）对一个新实验，在第 6 步（实验页与索引行第 7 步才写）照字面也会红。判决只点了 84，我没扩。
- 改名覆盖（F6）靠辩方自己在攻方定义里认 `-local-attack`；攻方定义第 27 行样本的形态是 `<前缀>-output-s<n>.md`，前缀由派发给，辩方定义把样本写成 `research/prompts/<轮>-local-defense-output-s<n>.md`，两处写法不同（改前就这样，F6 没动）。
- F9 让 `gate.sh` 整条进包装；包装文件头只写了「里层另起一个 scope、挪出外层」，`gate.sh --staged` 在包装的 scope 里跑时 `stage-must-run.sh`、55 号起虚机这类有没有别的影响，没跑过。
- F8 的 55 号下限只算了客机内存（6 × 2048 MiB），宿主侧 qemu 进程与 55 号自己的编译没算进去。

## 十、草稿与清理

草稿目录 `/tmp/claude-1000/defs-closeout-r1-fixes/`：改前备份 `before/`、diff、各门禁日志、探针用例与输出 `probe/`、弹窗闸改宽的三份单文件拷贝 `mutant-*/`、只加新格那一刻的弹窗闸拷贝 `ask-guard-cases-only.sh`。没有编译目录、没有仓副本。攻方 `probe.py` 默认把检出记录写到 `/tmp/probe-detections-*.jsonl`（4 份，都是我这几次探针建的），已拷进 `probe/` 并删掉原处。

## 附：改后全量 diff（相对开工时的备份）

```diff
--- a/.claude/agents/three-way-attack.md
+++ b/.claude/agents/three-way-attack.md
@@ -29,12 +29,13 @@
 3b. 造历史、跑模型时这样取样：
    - 盘用内存稀疏盘（`crates/singlefs-harness/src/crash.rs` 的 `SparseBlockDevice`），不在磁盘上建镜像文件；每个起点状态只建一次池（mkfs 与起点历史只跑一次），之后每段历史从内存里那一份拷（`SparseDevice`、`MemoryPool` 都能 `clone`）。
    - 正式跑之前先跑一小段，按它估全量的挂钟；估出来超过 40 分钟，先缩历史、候选与几何的取样，报告里写明缩了什么、缩前缩后各多少、估时怎么算的。
-   - 崩溃点不在缩的范围里：按那一串写逐点穷举（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），用同一份文件里层 0 那一套（录制写流、按崩溃点截断重放），checker 判结束状态。
-   - 随机跑批小批量、限时，每批的段数与限时写进报告。
+   - 崩溃状态不在缩的范围里（共用约束「不做」一节「崩溃点测试不衡量时间成本」那一条），照层 0 的枚举域取：调同一份文件里每一段都展开的 `enumerate_layer0` / `enumerate_layer0_versions`（录制写流，屏障与 FUA 切段，段内写的整写子集逐个枚举，不是只截前缀），每个崩溃状态恢复之后都跑 checker。缩只缩历史条数与长度、候选与几何的取样；留下的每段历史，它的崩溃状态一个不落。
+   - 在自己的原型里这样调枚举函数跑小流不算重型：测试目标的名字不带 `layer0`（带了 `heavy-test-guard.sh` 按名字拒）；每次跑之前用同一份文件的 `closed_form_state_count` 算出这一次全量的状态数，不超过约 10⁶ 个（几段历史合起来超过的，分几次跑）；一段历史自己就超过的，那段不跑，写进报告交主 agent。
+   - 随机跑批分小批：每批的段数按先跑一小段估出的时长定，不给批设限时（包装的 `RUN_WITH_MEMORY_CAP_TIME_LIMIT`、外面套 `timeout` 都不用）；每批的段数与估时写进报告。
 3c. 内存与进程：
    - 编译与跑都经内存包装：跑的照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，`cargo build` 也经它（`bash research/scripts/run-with-memory-cap.sh <上限> cargo build …`）。
    - 只停自己起的进程，一次一个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（共用约束「执行前拒绝的写法」那一条的 ⑥）。
-   - 并行起的几件各记下 `$!`，逐个 `wait "$pid"` 收退出码。
+   - 并行起的几件，每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <草稿目录>/<名字>.rc; } &`），最后单独一个不带参数的 `wait` 等齐，再按起的次序逐个读 `.rc` 文件（共用约束「执行前拒绝的写法」那一条的 ④）；不用 `wait "$pid"` 收退出码。
    - 等一行字之前先确认那一行真会写进那个文件。
    - 不改正在跑的脚本，要改的写同目录临时文件再 `mv` 换上（共用约束「执行前拒绝的写法」那一条的 ⑦）。
 4. 打中之后先答四句：分不分辨臂；被判的系统当时看不看得到判别它的东西；满足的是判据字面的哪一个分句；跑前条款给的每个改法在打中的那几格上还中不中。
--- a/.claude/agents/three-way-local-defense.md
+++ b/.claude/agents/three-way-local-defense.md
@@ -8,7 +8,7 @@
 
 # 本地辩方腿（three-way-local-defense）
 
-开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「做什么」「写范围」「产出」三节：做法与它逐条相同，只有下面几处不同。
+开工先读 `.claude/agent-common.md`（这份定义开了 `omitClaudeMd`，要用的规则照共用约束「规则怎么读」一节读），再读 `.claude/agents/three-way-local-attack.md` 的「输入」「做什么」「写范围」「产出」四节：做法与它逐条相同，只有下面几处不同。
 开工先读：`.claude/agents/three-way-local-attack.md` 里「开工先读：」那一行点名的小节。
 
 ## 与本地攻方不同的地方
@@ -16,7 +16,7 @@
 - 立场是辩方：替被判出局、被攻、或被计划排除的一方，找它站得住的理由，写出具体机制（哪个文件、哪一步）；站不住就说站不住、为什么。辩护落得成数的（这一方在某段历史上是几、判红没有），同样按攻方第 1 步给事实表、要模型逐格填。提示里先替每一条写一个攻方问题（它被质疑的那一句，整行抄出处），再让本地模型逐条辩护；攻方问题同样要逐句核转述。
 - 输入里的「攻击面」换成「要辩护的一方」（主 agent 给：被复核的判决或被排除的候选，原文路径）。
 - 行号同攻方第 1 步：提示里明令答复不写行号，样本自带的行号在运行记录里标「模型自给、未核」。
-- 文件名形态（「写范围」「产出」两节里的文件名照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
+- 文件名形态（攻方定义里出现的 `-local-attack` 文件名，不论在哪一节，一律照这里换）：提示 `research/prompts/<轮>-local-defense.md`，核对表 `research/prompts/<轮>-local-defense-translation-audit.md`，样本 `research/prompts/<轮>-local-defense-output-s<n>.md`，运行记录 `research/prompts/<轮>-local-defense-runlog.md`。
 
 ## 没做什么（固定会有的）
 
--- a/.claude/agents/experiment-runner.md
+++ b/.claude/agents/experiment-runner.md
@@ -28,12 +28,12 @@
 2. 写 `research/e7-index-bench/src/bin/e<号>_<简称>.rs`，在 `research/e7-index-bench/Cargo.toml` 末尾追加它的 `[[bin]]`（⚠️ **跑前登记把装置写死成「入库装置」时改写 `crates/singlefs-harness/src/bin/e<号>_<简称>.rs`**：岔路问的是「`crates/` 今天这份代码的性质」时，另写一份独立模型答不了，写范围闸按 `crates/singlefs-harness/src/bin/e*.rs` 放行。那种 bin 只读、只驱动与观测，不许改 `crates/singlefs-core` 与 `crates/singlefs-checker`；变异表改追加进 `crates/mutations.tsv` 末尾、归门禁 59 号跑。**另有三条只对入库装置生效**：① **不许从 `crates/` 里引决定工作量、几何或判据门槛的常量**——这类参数在装置里写本地常量，值抄自 kb 的登记位并注明文件与行号，另加一条断言拿它与实现侧同名常量回比；引了就等于装置与实现共用一个数，两边一起错而逐格相等。这一条挡不住「kb 与实现一起取错」。② **写了入库装置就要让它进代码轮判决的点名清单**：门禁 56 号的正则罩得到 `crates/*/src/bin/*.rs`，不点名那一道会红，而那条义务落在跑整轮门禁的一方身上、不落在你身上——交回时写明你新建了哪个 bin。③ **什么算「入库装置」以跑前登记里写死的那一句为准**；登记里没写死就走 `research/` 那一支，别自己判）（`name` 用连字符，与 `replay.sh` 登记行里的 bin 名一致；只追加，不改别人的）：纯算术的计数模型、单测、钉绝对值的断言；取样点覆盖登记里每一个会跑到的取值。写完跑 `bash .claude/scripts/naming-lint.sh`，新文件里的缩写与单字母名在这一步改完再往下走，不留给后面的门禁阶段。
 3. 写变异表 `research/mutations/e<号>_<简称>.tsv`，在 `research/` 下跑完整张表：`cd research && nice -n 19 bash scripts/mutate.sh <bin 名> e7-index-bench/src/bin/<源文件> mutations/<变异表>`（两个路径都相对 `research/`；bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准，没有显式声明的取文件名去掉 `.rs`；报「基线就是红的」时先查是不是在仓根下跑、bin 名是不是写错）（收尾要有「已还原，基线仍全绿」）；没红的逐条按 `mutation-sampling.md` 分类。
 3b. 登记里的停机条款（第十一节里标「停机」的，通常是装置与 `crates/` 实现逐项对拍）在跑产物之前做，照原文真跑，不许只推；做不了就停下交回，不跑产物、不写实验页。登记的量一次做不完的，做完的那一段照常交，没做的逐条列进报告，实验页与索引行的状态写「部分已跑」，不写「已跑」。岔路单每一行都够判了（登记第十一节的够判停机），剩下的量不跑：实验页逐条标「够判后未跑」、写明各自对应哪一行，状态写「已跑（够判）」；够不够判由你按登记写的够判条件报，续不续由主 agent 定。
-4. 跑产物进 `research/results/`；跑前删旧输出，跑完认本轮才有的完成标记。重跑已有实验时，`research/results/` 里已有的产物不覆盖：与它逐字节一致就不新存；对不上就按今天的日期另存一份，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
+4. 跑产物进 `research/results/`。那里已有的产物一个都不删、不挪、不覆盖（共用约束「执行前拒绝的写法」那一条的 ⑤）：这一次的输出写一个新文件名（带今天的日期或 `rN`，写之前 `ls` 确认没有同名的），跑完按这个新文件名认本轮才有的完成标记。重跑已有实验时拿新文件与已有的那份比：逐字节一致就删掉这一次的新文件、不新存；对不上就两份都留，实验页两份都点名、写明对不上，交主 agent 定哪份承重。生成产物用的二进制编在 `research/target/` 下，用最后一次源码改动编出来；生成产物与第 5 步跑 `replay.sh` 时都不设 `CARGO_TARGET_DIR`；草稿目录里的 target 只拿来迭代。
 4b. 产物出来之后先自查齐不齐：按登记的「臂 × 历史 × 几何」数一遍产物里每一类结果行的条数与字段，缺行、缺字段（例如某一族没有臂字段）、某一族一次都没造出被测形状，都列进报告，不许只看末行完成标记。报告里每个判据按每条臂全列判定，不挑「突出发现」报——对某条候选不利的那一半同样要写。分族、分臂的计数用一条命令从产物里数，报告与实验页贴那条命令和它的原样输出，不用文字重写族名单；给「这几族为什么是 0」写解释之前，先把那几族的产物行贴出来，确认它们真是 0。
-4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（逐字节一致没新存的，读 `research/results/` 里那一份；输出不截断），任何字段取值是 `false` 或 `not_run`，在报告里逐个点名：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段」。
+4c. 交回之前逐行读判决行：`grep -n 'name=verdict' <产物>` 列出这一次每份产物里的全部判决行（逐字节一致没新存的，读 `research/results/` 里那一份；输出不截断），任何字段取值是 `false` 或 `not_run`，或字段名表示违例、不匹配、歧义、失败的计数（名字里带 `violation`、`mismatch`、`ambiguous`、`fail` 的，例 `layer0_violations`、`width_mismatches`、`today_ambiguous`）取值不是 0，在报告里逐个点名：产物文件与行号、字段名、取值、为什么是这个值、是不是登记预期内的；一个都没有，也写一句「判决行 N 条，没有 false / not_run 字段，违例、不匹配、歧义、失败类计数都是 0」。
 5. 在 `research/scripts/replay.sh` 里登记复跑，跑一次 `bash research/scripts/replay.sh E<号>` 确认逐字节一致。送进虚机跑的实验（`vm-bench.sh`，含 `--selftest`）与 E152 装置是重型测试，你不跑：装置、登记与复跑命令写好就停，在交回里写明要跑什么、为什么，由主 agent 问用户（`.claude/kb/vm-harness.md`；共用约束「不做」一节里重型测试那一条）。装置要起子进程、读它的输出时，先把输出读完再转打，计时按登记写的进程与时钟取（`.claude/kb/vm-harness.md`「计时不在转发输出的循环里打时间戳」）；门禁 65 号查读子进程输出的循环里一边取时间一边输出。
-6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节）；其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
-7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。
+6. 跑阶段归属表登记给你的阶段（共用约束 `.claude/agent-common.md`「门禁」一节），84 号除外：它放到第 7 步写完实验页之后跑；这一次不写实验页的，在这一步最后跑，红了照写。其中要编译或复跑整张表的，开跑前照共用约束看负载。15 号（编整个 research 工作区）与 87 号（复跑全部已入库实验）不归你，收尾由 `gate-triage` 统一跑；你只跑第 5 步自己那一个实验的 `replay.sh`。
+7. 主 agent 要写实验页时：`.claude/kb/experiments/<号>-<简称>.md`，结果整行抄自产物并带文件名，写复跑命令、口径、它答不了的；索引行进 `.claude/kb/experiments.md`。写完（连同第 7b 步）跑 84 号，贴原样末行与退出码。
 7b. 实验页的 `### 影响的决策` 一节照第 7 步写实验页的规则写，形态与判据不在这份定义里复述。这一步你要做的三件：① 要写支撑或推翻时，先 `grep -n 'E<号>（' .claude/kb/decisions/<那条决策的文件>` 看那条分项的「**依据**」段引没引这个实验——引了才写支撑或推翻，没引就写备料，并把「这一格该不该升成支撑」列进报告交主 agent，**不自己去改决策文件**（写范围闸也不放行）。② 那条决策还在 `.claude/decision-links-pending` 里时不查依据段（门禁 75 号对清单里的决策整段跳过双向检查，那些决策还没有依据段），写决策级一行，并把它列进报告。③ 重跑已有实验、换了产物、加了历史条目之后，表里每一行都重新回看一遍。
 
 ## 写范围
--- a/.claude/agents/crash-verifier.md
+++ b/.claude/agents/crash-verifier.md
@@ -17,12 +17,13 @@
 
 - 这次改动的范围：`git diff --stat -- crates litmus` 原样，或提交区间。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
+- 54、55、57 号各自的内存上限（第 1b 步用）；55 号的不小于同时起的虚机数乘每台的内存（`.claude/gate.d/55-qemu-first-transaction.sh` 的 `MODES` 档数 × `research/scripts/vm-bench.sh` 的 `VM_MEM`）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 每个阶段开跑前各照共用约束「不做」一节看一次负载；另有别的 `gate.sh` 正在跑 54 号、或有 `54-layer0-replay.sh --full` 在跑时，不起 54 号，等它结束。
-1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条，上限与退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
+1b. 54、55、57 号整条经内存包装跑（共用约束「不做」一节「跑编译出来的代码经内存包装」那一条；上限取输入给的那一道的，退出码 250–254 照那一条办）：第 2 步里起这三道的每条命令，在 `bash <阶段文件>` 前面加 `bash research/scripts/run-with-memory-cap.sh <上限>`，写成 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/gate.d/<文件>`；59 号在里面逐条套了，照第 2 步直接跑，外面不再包一层。
 2. 只在提交时（或主 agent 转达用户要求时）跑你那几道，不跑 `gate.sh` 整轮与全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）。按共用约束 `.claude/agent-common.md`「门禁」一节从阶段归属表取登记给你的阶段，一次只跑一个；54、55、57、59 号命令带输入给的那个前缀：`SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>`，其余登记给你的轻阶段不带；54 号默认不带 `--full`（快档加核全绿标记），派发提示点名要层 0 全量时才带；每个记开始与结束时刻（`date -u`）和退出码。单个阶段超过 Bash 单次上限就后台跑，退出码写进文件（`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/<文件>; echo "exit=$?"; } > <草稿目录>/<阶段>.log 2>&1`；退出码只认日志里的 `exit=` 那一行，不认起后台的那条命令自己的 `$?`），结束后读输出。别的会话同时在跑 cargo 时，编译会卡在「Blocking waiting for file lock」，照实记等了多久，不算这个阶段的耗时。
 3. 每个阶段原样抄它的「✓」行，或「✗」行与紧跟的「→」；退出码 77 记「本次未跑」，不记通过。
 4. 计数行原样抄：崩溃状态数、恢复结果、与 E142 产物或闭式比对的那几行。输出里读不到判定行的阶段记「作废」，不记通过。
--- a/.claude/agents/gate-triage.md
+++ b/.claude/agents/gate-triage.md
@@ -18,13 +18,14 @@
 - 这一轮的改动已经按 `research/scripts/stage-mine.py` 暂存了没有（没暂存就不派你，或者主 agent 明写「跑全量工作区」）。
 - 这一次是提交时跑还是用户要求时跑：决定命令带 `SINGLEFS_HEAVY_TESTS=commit` 还是 `=user-request`。
 - 这一轮暂存区 diff 的范围：`git diff --cached --stat` 原样。主 agent 明写跑全量工作区时，暂存区多半是空的，这时主 agent 另给「这一轮改过的文件清单」，归属按清单判。
+- `gate.sh` 整条经内存包装的上限（第 1b 步用）。
 - 报告路径与草稿目录。
 
 ## 做什么
 
 1. 开跑前照共用约束「不做」一节看负载；另有别的 `gate.sh` 在跑就等它结束。
-1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`gate.sh` 与单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
-2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑不带参数（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
+1b. 自己单跑的编译出来的代码（`cargo test`、测试二进制）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑。`gate.sh` 整条经内存包装跑，上限取输入给的，退出码 250–254 照那一条办；里面 59 号、87 号逐条再经包装的照常跑（嵌套怎么排队见 `research/scripts/run-with-memory-cap.sh` 文件头「包装里再经包装跑的」那一句）。单跑的门禁阶段照第 2 步直接跑，外面不包一层（87 号经 `research/scripts/replay.sh` 逐条套了）。
+2. 只在提交时（或主 agent 转达用户要求时）跑：暂存过的跑 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash research/scripts/run-with-memory-cap.sh <上限> bash .claude/scripts/gate.sh --staged`（用户要求时 `=user-request`）；主 agent 明写全量的跑同一条、去掉 `--staged`（前缀照带）。54、55、57、59 这几道重阶段在 `gate.sh` 里照各自的复用判定走（54 号只核全绿标记），你不直接调它们，也不跑全量 `cargo test`（`.claude/hooks/heavy-test-guard.sh` 拒）；登记给你的其余阶段单跑时不带前缀，87 号单跑照带。超过 Bash 单次上限时后台跑、结束后读输出。
 3. 每个红阶段：抄门禁给的「✗」行与紧跟的「→」下一步原样；看它点名的文件与行在不在暂存区 diff 的块里（`git diff --cached -U0 -- <文件>`）。在块里 ⇒「这一轮」；文件在但行不在块里、或文件不在暂存区 ⇒「不是这一轮」；红句说的是「这一批输入」本身的（54 号的「没有这批输入的全绿标记」「那一格的哈希不同」这类）不按它点名的标记路径判：这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」；工具没装、网关不通、`cargo` 不在 ⇒「环境」，用 `bash .claude/scripts/env.sh` 的对应行佐证；别的会话同时在跑同一个重阶段、进程被外部信号杀掉（输出里一行裸的 `Terminated`）⇒「并发」，贴开跑时与出事前后 `ps` 看到的同名进程。红阶段没有「✗」与「→」可抄的（被杀、崩溃），抄它最后 20 行输出，判「分不清」或「并发」，写明没有下一步可抄。分不清的写「分不清」与原因。
 4. 原样抄未实现清单与「由项目本地阶段覆盖」清单。别的会话同时在改仓、跑全量工作区时「工作区跑的过程中没变」那一条红了：照抄开跑、收尾两个指纹，判「不是这一轮」。`gate.sh` 不打印单个阶段的耗时，取不到的不估。
 
--- a/.claude/agents/implementation-writer.md
+++ b/.claude/agents/implementation-writer.md
@@ -26,7 +26,7 @@
 1b. 第 3 步的证红、第 4 步动到的测试二进制（副本里跑的也算）照共用约束「不做」一节「跑编译出来的代码经内存包装」那一条经包装跑；`cargo fmt --check`、`cargo clippy`、`cargo build` 不经它。
 2. 读条款与代码，写实现与测试。名字、分支、类型、错误照 `code-discipline.md`：不缩写、不写 `_ =>`、newtype、`expect` 写清依赖哪条不变量。
 3. 每条新测试证明会红：在草稿目录的仓副本里（`rsync -a --exclude target --exclude .git`）改坏被测代码一处，跑那条测试所在的整个测试二进制看它红，记「改坏哪一行 → 哪条断言红」与同时红了哪些测试；每份副本用它自己的 target（不把 `CARGO_TARGET_DIR` 指到几份副本共用的目录）；改坏的副本要还原时，从原件拷回之后对被改的文件 `touch`，或删掉这份副本重拷，不用 `rsync -a` 拷回了事；先跑一份不改动的副本：已经红的测试记成基线红集，变异证明只看基线红集之外的测试，你要证明的那条在基线红集里就停下交回；被测代码里有 `debug_assert` 的，debug 下先红的可能是它，照实记红在哪一条，要测试断言自己红就在 `--release` 下再跑一次。`crates/mutations.tsv` 在的话（门禁 59 号），每条再加一行变异，原文在文件里恰好命中一次。
-4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`，各贴末尾原样输出。全量 `cargo test --all`、层 0 各流的快档与全量、门禁阶段都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
+4. 变异证红按测试算、不按行算：每条新测试挑一行能让它红的变异证一次就够，其余行照样追加进 `crates/mutations.tsv` 末尾，整表复跑归最后的门禁 59 号；报告写明哪几行证过、哪几行留给 59 号。层 0 不跑：自己新加的层 0 流不在交回前跑，随提交时的层 0 全量验；已有流一条都不跑；新测试落在名字含 layer0 的测试二进制里的，第 3 步的证红也不跑，变异行照样追加、报告写明留给提交时的 59 号。层 0 全量每次提交跑一次（`main-agent.md`「什么时候派哪个 agent」里「暂存之后、提交之前跑门禁」那一步），不由实现员跑。交回前的验证只到这几样：动到的测试二进制（整个二进制，不按名字挑；名字含 layer0 的不跑）、第 3 步的证红、`cargo fmt --check`、`check.sh` 那一套 lint 下的 `cargo clippy`、`cargo build --offline --all-targets`、阶段归属表登记给你的门禁阶段（共用约束「门禁」一节，74 号在内），各贴末尾原样输出。其余门禁阶段、全量 `cargo test --all`、层 0 各流的快档与全量都不跑，留给提交时统一的那一次验证（`crash-verifier` 与整轮门禁）；派发提示另点名要跑的，不是重型测试的照点名跑，重型的照共用约束「不做」一节那一条交回主 agent。
 5. 要改的文件在输入给的「别的会话正在改」清单里：不碰那个文件，也不做只能落在它上面的条款，报告写明卡在哪个文件、哪条条款；新文件要靠清单里的文件才进编译（模块声明、`Cargo.toml` 的 `[[test]]`）的，那条整条停下，不写那个新文件；其余照做。`crates/mutations.tsv` 例外：只在末尾追加整行，不改别人的行。
 6. 改动里碰到条款没写、要做设计判断的地方，停在那一处，写进报告交主 agent，不自己定。「停在那一处」的写法看这条分支走不走得到：
    - 走得到（包括要坏盘、崩溃、瞬时读错才走得到）：在**任何落盘动作之前**返回一个错误成员，名字说清是哪条条款没定、第一版不支持；写一条用例钉住「返回这个成员、盘上逐字节不变」（`DiskSnapshot` 之类比系统配置槽、根环、录制流步数），不钉它之后的行为。判定与后面的写要读同一份输入：写之前重算、对不上就不写。
--- a/.claude/agents/mutation-triage.md
+++ b/.claude/agents/mutation-triage.md
@@ -15,6 +15,7 @@
 ## 输入（主 agent 必须给）
 
 - 变异表：`research/mutations/<名>.tsv`（配 bin 名与源文件）或 `crates/mutations.tsv` 里的条目名。bin 名以 `research/e7-index-bench/Cargo.toml` 里 `[[bin]]` 的 `name` 为准（多是连字符，源文件名是下划线）；主 agent 给的对不上、`mutate.sh` 退出码 6 时，照它报的改法改，报告里写明。
+- 给的是 `crates/mutations.tsv` 里的条目时：崩溃验证员跑的那一次门禁 59 号的输出路径（第 3 步从里面取条目）。
 - 改动前的三个数（有就给）。
 - 报告路径与草稿目录。
 
--- a/.claude/agent-common.md
+++ b/.claude/agent-common.md
@@ -43,7 +43,7 @@
 - 不建、不改 `.claude/agents/`、`.claude/settings*.json`、`.claude/hooks/`、`.claude/rules/`、`.claude/singlefs-ai-sop/`，也不碰 `~/.claude/`。
 - 不读派发提示给的禁读清单里的文件。
 - 不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做。跑脚本、跑模型一律加 `nice -n 19`。
-- 重型测试（层 0、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
+- 重型测试（名字含 `layer0` 的测试二进制与 54 号、QEMU、herd7、crates 变异整表、全量 `cargo test`（`--all` / `--workspace` / 工作区根裸跑）与 `check.sh`、`gate.sh` 整轮、87 号全部实验复跑、E152 装置）只在提交代码时跑一次、或用户要求时跑；子 agent 一律不跑，只跑自己动到的测试二进制（`cargo test -p <crate> --test <自己的目标>`、`--lib`）、fmt / clippy / build 与 `.claude/gate.d/` 下 54、55、57、59、87 之外的阶段。`crash-verifier` 只跑 54、55、57、59 号那几道，`gate-triage` 只跑 `gate.sh` 整轮与 87 号，都在提交时或用户要求时跑、命令带 `SINGLEFS_HEAVY_TESTS=commit`（用户要求时 `=user-request`）。项目 settings 里的 `.claude/hooks/heavy-test-guard.sh` 在执行前拒绝越出这些的命令；被拒了不换写法绕过去，在交回里写明要跑什么、为什么，交主 agent 去问用户。
 - 写进脚本文件、放进 `python3` 的 subprocess、`make` 这类间接起法同样算跑：自己写的验证链里不许出现重型测试的任何一步，名字含 `layer0` 的测试二进制也不许。
 - 跑编译出来的代码经内存包装：`cargo test` / `run` / `bench`（带 `--no-run` 只编不跑的不算）与直接执行 cargo 编出来的二进制（实验装置、测试二进制），一律经 `bash research/scripts/run-with-memory-cap.sh <上限> <命令…>` 跑；写进自己的脚本再跑的，整条脚本经它跑（`bash research/scripts/run-with-memory-cap.sh <上限> bash <脚本>`）。上限取派发提示给的（定义另写了来源的照定义），都没给就取 `research/scripts/replay.sh` 文件头 `REPLAY_MEMORY_CAP` 的默认值。`research/scripts/mutate.sh`、`research/scripts/replay.sh` 与门禁 59 号在里面逐条套了，直接跑，外面不再包一层。退出码 250–254 是包装自己的结局（含义见 `research/scripts/run-with-memory-cap.sh` 文件头「退出码」一段）：那一次的输出不算结果，命令与退出码写进报告，不绕开包装重跑。`cargo build`、`cargo clippy`、`cargo fmt` 不跑编出来的代码，不要求经它。
 - 要停自己起的一条链（脚本、后台任务）时，先列出它的整棵子进程（`ps -o pid,ppid,args --ppid <pid>` 逐层往下），从最底层起逐个 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`（`proc.py` 只停给定的那一个，不带子进程），停完再列一遍，确认一个不剩；不许留下正在跑的那一步「让它跑完」。
@@ -53,21 +53,26 @@
 - **崩溃点测试不衡量时间成本，也不为省时间缩范围**：要加崩溃点、要多枚举一档，就加。挂钟长不是收窄它的理由，也不用先算它值不值。
 - 禁止自行扩大任务范围，只改自己的部分。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - 本机常有别的会话在同一个仓里干活：只动这一轮自己的文件；看到别人没提交的改动不碰、不修。
-- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
+- 长活可以等，不给它设超时、不自己中途杀掉：编译、变异表、复跑这类长活用 Bash 的 `run_in_background` 起，起完结束本轮，完成时会通知你。交给它的命令要一直跑到真正的活结束才退出：不在里面再把活放到后台——不用 `disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux`/`screen` 的分离模式、`systemd-run`，子 shell 或 `$( … )` 里也不写 `&`；`&` 只在同一条命令随后用不带参数的 `wait` 等齐时用，每件的退出码照「执行前拒绝的写法」那一条 ④ 的写法收。结束本轮就是这一条回复只写一句在等什么、不再调工具；前台命令的 `timeout` 不超过 240000 毫秒。
   等长活时不起缓存计时器，结束本轮直接等完成通知。
   等的时候看到进程不动、报错、或等的条件不会成立，把看到的命令、输出与时刻写进草稿目录里的进度记录，接着做定义里还能做的；结不结束、怎么处置由主 agent 定。主 agent 的看门狗（`research/scripts/agent-watch.py`）定时复检你；前台没有超时的等待循环、把活放出追踪、run_in_background 里后面没有 `wait` 的单独 `&` 在执行前被拒（「执行前拒绝的写法」那一条的 ②③④），run_in_background 里的等待循环只记检出，交给主 agent 判断。等日志里的一行字之前，先确认那一行真会写进那个文件；找进程用写死的 pid，不用 `pgrep -f` / `pkill -f`（command-safety.md）。
-- 执行前拒绝的写法：项目 settings 里的几道 hook 在执行前拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
-  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
+- 执行前拒绝的写法：项目 settings 里注册的这几道 hook 在执行前（收工闸在收工前）拒绝这几类写法（退出 2，stderr 写原因与出路），主 agent 与子 agent 一样拒，只挂在一方才用的工具上的在条目里写明；拒的只是这一次调用，不停已经在跑的东西。被拒了照出路改写法，不换写法绕过去；改不成的写进报告交主 agent。
+  - Bash 检出 hook（`.claude/hooks/bash-command-detector.sh`）：② 只在前台拒（run_in_background 里的等待循环只记检出、不拒），④ 只在 run_in_background 里拒，其余前台与 run_in_background 一样拒：
     ① 起看门狗的错误写法：`research/scripts/watch.sh`、`research/scripts/agent-watch.py watch` 不用 run_in_background 起，或命令里带单独的 `&`、`nohup`、`disown`、`setsid`、把输出丢进 `/dev/null`。看门狗只有主 agent 起。
     ② 前台没有超时的等待循环：`until` / `while` 里有 `sleep`，外面没套 `timeout`。要等就用 run_in_background 起、结束本轮等完成通知，或 `python3 .claude/singlefs-ai-sop/scripts/proc.py wait <pid> --timeout <秒>`。
     ③ 把活放出追踪：`disown`、`coproc`、`setsid -f`、`nohup … &`、`tmux` / `screen` 的分离模式、不等到结束的 `systemd-run`。
-    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。
+    ④ run_in_background 里有作业以单独的 `&` 收尾，之后同一条命令里没有 `wait`。要并行就每件把退出码写进自己的文件（`{ <命令>; echo "$?" > <文件>; } &`），最后单独一个不带参数的 `wait` 等齐，再按派活的次序逐个读那几个文件；不用 `wait "$pid"` 收。
     ⑤ 整份覆盖 `research/results/` 下已存在又没进 git 的产物：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` / `mv` / `install` 的目标、`dd of=`、`truncate`。追加（`>>`、`tee -a`）与新文件名不拦；要换就按日期另存新文件名，旧的留着。
     ⑥ 终止进程不是点名一个自己起的进程号或任务号：`kill` 的目标带负号、是 0 或 `$PPID`、一次给几个、是命令替换或通配，在循环里逐个发（`proc.py stop` 同样），按名字、按 cgroup 或 `/proc` 挑进程，写死的进程号指到自己会话的祖先、别的会话起的进程或 SSH / VSCode / 本地模型服务，`systemctl` 停 SSH、登录会话、本地模型服务这类单元，关机重启。放行的是点名一个：`kill "$!"`、`kill %1`、单独一条 `python3 .claude/singlefs-ai-sop/scripts/proc.py stop <pid>`。
     ⑦ 在同一个 inode 上改已经存在的脚本（`.sh`、`.py` 或带执行位的文件，在仓里或 `/tmp/claude-1000/` 下）：`>`、`>|`、`&>`、不带 `-a` 的 `tee`、`cp` 的目标、`dd of=`、`truncate`、python 的 `open(…, 'w')` 这一类。改脚本写到同目录临时文件再 `mv` 换上，或用 `research/scripts/replace-once.py` / `insert-row.py` 定点改；追加与新建不拦。
   - 上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/pattern-process-guard.sh`：命令位置上的 `pgrep -f`、`pkill -f`、`killall`。
   - 重型测试闸（`.claude/hooks/heavy-test-guard.sh`）：越出重型测试那一条的命令（主 agent 不带 `SINGLEFS_HEAVY_TESTS` 前缀也拒）；子 agent 不经内存包装跑编译出来的代码（「跑编译出来的代码经内存包装」那一条）。
   - 写闸（`.claude/hooks/write-guard.sh`，只看 Write / Edit 工具）：Write 整份覆盖仓里已存在又没进 git 的文件；项目子 agent 写到写范围表（`.claude/hooks/agent-write-scope.tsv`）里自己那几行之外；写进的内容里有撇号类角标（字符与起名的写法见 `.claude/rules/path-moves.md`「变体起新名字，不用角标」）。
+  - 交回闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/handback-scratch-check.sh`，挂交回工具 SubagentHandback 与 SubagentStop，只判子 agent）：临时目录里自己建的编译目录、工作树与仓副本还在，交回报告里又没逐个写全路径与为什么不删（`.claude/singlefs-ai-sop/rules/session-wrapup.md`「5. 子 agent 交回之前，删掉自己建的编译目录与仓副本」）。
+  - 收工闸（上游的 `.claude/singlefs-ai-sop/scripts/claude-hooks/gate-reuse-check.sh`，挂 Stop 与 SubagentStop）：这个会话新建或改过的门禁与钩子没写 `# gate-similar:` / `# hook-events:`、该点名的已有门禁与钩子没点全、或整段抄了已有的一份（`.claude/singlefs-ai-sop/rules/sop-first.md`「加门禁或钩子之前，先找已有的」）。
+  - 弹窗闸（`.claude/hooks/ask-user-claim-guard.sh`，挂 AskUserQuestion，主 agent 用）：问句或选项说明里一句话带断言词（不可能、造不出、从来不、从来没有、一定、必然、永远不、绝不会、恒为），同一句里没有出处（反引号里的路径或命令、文件:行号、`research/results/` 下的文件、`name=` 开头的产物行、「实测」「量过」「产物」「输出」旁边带数或路径），也没写「推的」「没量过」「推测」「估计」「粗估」之一。
+  - 派发闸（`.claude/hooks/runner-dispatch-guard.sh`，挂 Agent / Task，主 agent 用）：派发提示要子 agent 跑重型测试；派 `experiment-runner` 没写「这一段回答的岔路：…」，续做没写「上一段岔路表里还差：…」或写了一行都不差；派 `kb-scribe`、`implementation-writer` 要改的文件落在还没写判决的三方轮的开工快照里。
+  - 续做闸（`.claude/hooks/continuation-guard.sh`，挂 SendMessage，主 agent 用）：给已经交回过、或最近一次任务通知是 failed / killed 的子 agent 发消息。
 
 ## 门禁
 
--- a/.claude/main-agent.md
+++ b/.claude/main-agent.md
@@ -11,7 +11,7 @@
 - **禁止自行扩大任务范围，只改自己的部分**。一个任务一个出口，达成出口即终止任务，扩大任务必须弹窗。禁止因为产物变化无限继续任务。尤其是门禁检测等任务，不属于自己任务的验证，如果验证为红给其他任务发消息，严禁自行修改扩大任务范围。
 - **测试结果与预期不符，禁止立刻直接修改方向和结论，先检查代码中有没有bug。**
 
-- **禁止在subagent中跑重型测试（层 0 全量、QEMU、herd7、`crates` 变异整表、全量测试、整轮门禁）等**，全单以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准。
+- **禁止在subagent中跑重型测试**，完整清单以 `.claude/rules/implementation-workflow.md`「重型测试只在提交时跑」那一节开头的清单为准。
 
 ## 一轮怎么开、怎么收
 
@@ -45,6 +45,8 @@
 
 同时在跑的重活（编译、测试、变异、产物、原型扫描）不止一个时，每份派发提示写一行「线程上限：N」，N = 整机核数 ÷ 同时在跑的重活数（向下取整，至少 1）；重活数变了，给在跑的 agent 发消息改 N。
 
+派崩溃验证员时给 54、55、57 号各自的内存上限，55 号的不小于同时起的虚机数乘每台的内存（算法在 `.claude/agents/crash-verifier.md`「输入」一节）；派门禁分诊员时给 `gate.sh` 整条经内存包装的上限。
+
 在不影响正确性的前提下写简练：只写对方干这件活要的东西——定义「输入」一节要的项、为哪几行岔路或哪个问题、定义与共用约束里没有的约束、交付什么交到哪；不复述定义与规则里已经写着的做法，不讲来龙去脉。事实、路径、行号、数与判据一个不少，嫌长就指到文件，不删内容。
 
 ## 什么时候派哪个 agent
--- a/.claude/rules/implementation-workflow.md
+++ b/.claude/rules/implementation-workflow.md
@@ -45,7 +45,18 @@
 
 ## 重型测试只在提交时跑
 
-**重型测试**：层 0 全量（`.claude/gate.d/54-layer0-replay.sh` 与 `--test` 目标名含 `layer0` 的测试）、QEMU（55 号、`vm-bench.sh`）、herd7（57 号、`.claude/scripts/lkmm.sh`）、`crates` 变异整表（59 号）、全量 `cargo test`（`--all`、`--workspace`、`check.sh`）、整轮门禁（`gate.sh`、`gate-staged.sh`）、全部实验复跑（87 号）、E152 装置。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
+**重型测试**，与 `.claude/hooks/heavy-test-guard.sh` 拒的逐类相同（判定在 `.claude/hooks/lib_heavy_tests.py` 的 `classify`）：
+
+- 层 0 全量：`.claude/gate.d/54-layer0-replay.sh`；会跑到名字含 `layer0` 的测试二进制的 `cargo test`——`--test` 的名字含 `layer0` 或通配命中它，或者不挑目标（不带 `--test` / `--lib` / `--bin` 这类、或带 `--tests` / `--all-targets`）而包里有这种测试目标（例 `cargo test -p singlefs-harness`）；直接执行名字含 `layer0` 的测试二进制（`<target 目录>/<profile>/deps/<名字>-<16 位十六进制哈希>`）。
+- QEMU：55 号、`qemu-system-*`、`research/scripts/vm-bench.sh`（`--selftest` 也算）。
+- herd7：57 号、`.claude/scripts/lkmm.sh`、`herd7`；这两样带什么参数都算，只取版本号的也算。
+- `crates` 变异整表：59 号、参数里有 `crates/mutations.tsv` 的 `research/scripts/mutate.sh`。
+- 全量 `cargo test`（`cargo t` 同）：带 `--all` / `--workspace`；在工作区根（仓根与 `research/`）上不带 `-p` 也不带 `--test` / `--lib` / `--bin` 这类挑目标选项的；不挑目标而包的范围是工作区全部成员的（在 `research/e7-index-bench/` 里裸跑也算）；`.claude/scripts/check.sh`。
+- 整轮门禁：`gate.sh`（带什么参数都算）、`research/scripts/gate-staged.sh`（`--selftest` 不算）。
+- 全部实验复跑：87 号。
+- E152 装置：`e152-file-system-benchmark`（直接起、或 `cargo run --bin` 它）、`research/scripts/e152-run.sh`。
+
+只认命令位置；`bash -c`、`capped.sh`、`run-with-memory-cap.sh`、`nice`、`timeout`、`env` 这类包装里面的同样算。命令位置上执行的脚本（`bash x.sh`、`./x.sh`、`source x`）闸读进去，逐行照同一个 `classify` 判，里面有一样就整条算（例：`.claude/scripts/fetch-deps.sh --check` 第 81 行调 `herd7 -version`，整条算 herd7）；门禁阶段与 `lib_heavy_tests.py` 的 `KNOWN_SCRIPT_LOCATIONS` 登记的仓内脚本按名字判、不读正文。`.claude/gate.d/` 下 54、55、57、59、87 之外的阶段不是重型，谁都能跑。
 
 | 场合 | 跑不跑 |
 |---|---|
--- a/.claude/hooks/ask-user-claim-guard.sh
+++ b/.claude/hooks/ask-user-claim-guard.sh
@@ -19,7 +19,8 @@
 #   ② 句子里有「推的」「没量过」「推测」「估计」「粗估」之一的放行。
 #   ③ 句子里有出处的放行，出处是下面任一样（在原句上找，引号与反引号里的也算）：
 #      反引号里的路径（带 / 的，或带登记过的扩展名 PATH_EXTENSIONS 的文件名）或命令（全 ASCII、至少两段，第一段是小写命令名、
-#      ./ 开头的路径或 VAR= 赋值）；文件:行号（文件带 / 或带登记过的扩展名，冒号全角半角都认）；research/results/ 下的文件名；
+#      ./ 开头的路径或 VAR= 赋值）；文件:行号（文件带 / 或带登记过的扩展名，冒号全角半角都认；最后一个 / 之后的文件名里认汉字这类非 ASCII 的字，
+#      / 前面紧挨着的那个字要是 ASCII）；research/results/ 下的文件名；
 #      name= 开头的产物行；「实测」「量过」「产物」「输出」前后 MEASUREMENT_WINDOW 个字以内带着一个数或路径，
 #      而且那个数或路径与这个词之间没有隔着断言词（「输出一定为 0」里 0 与「输出」之间隔着「一定」，不算）。
 #   ①有、②③都没有的句子逐句列出，退出 2；一句都没有就放行。stdin 读不出 JSON、questions 不是列表的放行。
@@ -45,14 +46,16 @@
 PATH_EXTENSIONS = ("rs", "md", "sh", "py", "tsv", "csv", "out", "txt", "log", "json", "jsonl", "toml", "yaml", "yml",
                    "conf", "litmus", "cat", "diff", "patch", "lock", "img")
 ASCII_PATH_CHARACTER = r"[A-Za-z0-9_.-]"
+# 文件名里的字：ASCII 之外还认汉字这类非 ASCII 的字（\w 在 str 上认 Unicode 字母与数字，不认，。：（）「」这类标点）；目录名只认 ASCII
+FILE_NAME_CHARACTER = r"[\w.-]"
 EXTENSION_ALTERNATION = "|".join(PATH_EXTENSIONS)
 # 路径：带一个 /，或以登记过的扩展名收尾的文件名
 PATH_LIKE = re.compile(
     rf"{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
     rf"|(?<![A-Za-z0-9_.-])[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION})(?![A-Za-z0-9])")
 FILE_AND_LINE = re.compile(
-    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{ASCII_PATH_CHARACTER}+"
-    rf"|[A-Za-z0-9_-]{ASCII_PATH_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
+    rf"(?:{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]/{FILE_NAME_CHARACTER}+"
+    rf"|[\w-]{FILE_NAME_CHARACTER}*\.(?:{EXTENSION_ALTERNATION}))[:：]\d+")
 RESULTS_FILE = re.compile(rf"research/results/{ASCII_PATH_CHARACTER}*[A-Za-z0-9_-]")
 PRODUCT_LINE = re.compile(r"(?<![A-Za-z0-9_])name=[^\s`'\"]")
 CODE_SPAN = re.compile(r"`[^`\n]*`")
@@ -164,6 +167,7 @@
 def selftest(hook_dir):
     original = "可达状态里 defer 一项不可能为 0……重新搭环境也造不出来"
     one_sourced_one_not = "实测 `research/results/e156-r3.out:12` 这一格 defer=0。重新搭环境也造不出别的值"
+    chinese_file_name_sourced = "records/2026-09-16-subagent拆分提案.md:951 写着静态分支一定要走定义三方"
     # (说明, tool_input, 应当被拒的句子——空列表就是应当放行)
     cases = [
         ("必拒：原话那句", {"questions": [{"question": original, "options": [{"label": "采纳", "description": "自证用造的基底"}]}]}, [original]),
@@ -189,6 +193,15 @@
         ("必放：反引号里的路径", {"questions": [{"question": "回收写在 `crates/core/src/mount.rs` 的挂载路径里，defer 一定会被清掉", "options": []}]}, []),
         ("必放：不在反引号里的文件:行号", {"questions": [{"question": "见 transaction.rs：120，回收一定排在提交之后", "options": []}]}, []),
         ("必放：research/results/ 下的文件名", {"questions": [{"question": "research/results/e156-r3.out 那一格必然是 0", "options": []}]}, []),
+        ("必放：不在反引号里、文件名是中文的文件:行号", {"questions": [{"question": chinese_file_name_sourced, "options": []}]}, []),
+        ("必放：多层目录下中文文件名的文件:行号", {"questions": [{"question": ".claude/kb/decisions/08-核心索引结构.md:40 写着这一格一定不变", "options": []}]}, []),
+        ("必放：不带 / 的中文文件名，全角冒号行号", {"questions": [{"question": "见 08-核心索引结构.md：40，这一格一定不变", "options": []}]}, []),
+        ("必拒：中文文件名没带行号", {"questions": [{"question": "records/2026-09-16-subagent拆分提案.md 里写着静态分支一定要走定义三方", "options": []}]},
+         ["records/2026-09-16-subagent拆分提案.md 里写着静态分支一定要走定义三方"]),
+        ("必拒：中文词后面跟冒号与数不是文件:行号", {"questions": [{"question": "拆分提案第四十节：40 行写着它一定要走三方", "options": []}]},
+         ["拆分提案第四十节：40 行写着它一定要走三方"]),
+        ("必拒：斜杠分开的中文词跟冒号与数不是文件:行号", {"questions": [{"question": "抓到/无效/没红：40/0/0，这张表一定全抓了", "options": []}]},
+         ["抓到/无效/没红：40/0/0，这张表一定全抓了"]),
         ("必放：没有 questions", {}, []),
     ]
     results = []
@@ -199,7 +212,8 @@
     for label, tool_input, want_code, want_in_stderr in (
             ("stdin：原话那句拒绝", cases[0][1], 2, "重新搭环境也造不出来"),
             ("stdin：两句里拒没出处的那句", cases[3][1], 2, "重新搭环境也造不出别的值"),
-            ("stdin：带出处的放行", cases[7][1], 0, "")):
+            ("stdin：带出处的放行", cases[7][1], 0, ""),
+            ("stdin：中文文件名的文件:行号放行", {"questions": [{"question": chinese_file_name_sourced, "options": []}]}, 0, "")):
         completed = subprocess.run(["bash", script], input=json.dumps({"tool_name": "AskUserQuestion", "tool_input": tool_input}),
                                    capture_output=True, text=True)
         stderr_ok = want_in_stderr in completed.stderr if want_in_stderr else completed.stderr == ""
@@ -211,8 +225,8 @@
         print("    → 看 sentences_of() / without_code_and_quotations() / assertion_spans() / has_provenance() 的判法与 stdin 入口，改完再跑 --selftest")
         return 1
     print(f"  ✓ 自检通过（查了 {len(results)} 种）：带断言词、同一句里没有出处也没写「推的」的句子拒绝，只拒没出处的那一句；"
-          "出处（反引号里的路径或命令、文件:行号、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
-          "反引号与「」里的断言词、「不一定」放行；三例走真实的 stdin 入口")
+          "出处（反引号里的路径或命令、文件:行号（文件名是中文的也认）、research/results/ 下的文件、name= 产物行、实测带数）、「推的，没量过」、"
+          "反引号与「」里的断言词、「不一定」放行；中文文件名没带行号、中文词后面跟冒号与数的照拒；四例走真实的 stdin 入口")
     return 0
 
 def main():
--- a/.claude/gate.d/74-model-differential.sh
+++ b/.claude/gate.d/74-model-differential.sh
@@ -43,7 +43,8 @@
   if ! cargo test --release -p singlefs-harness --test "$TEST_BINARY" -- --nocapture >"$log" 2>&1; then
     tail -40 "$log"
     echo "  ✗ 随机历史的测试二进制判红（上面是 cargo test 的尾部）"
-    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
+    echo "     → 怎么办：单跑看细节（经内存包装，上限取派发提示给的，没给就用 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值 8G）："
+    echo "                bash research/scripts/run-with-memory-cap.sh 8G cargo test --release -p singlefs-harness --test $TEST_BINARY -- --nocapture"
     echo "                签名是 ModelDisagreement 的，是实现与理想模型对某一步的结局答得不一样：先看模型那一格引的条款，再判改实现还是改模型。"
     rm -f "$log"
     exit 1
```
