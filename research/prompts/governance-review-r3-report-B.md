# 严查第三轮（确认轮）·乙组报告：.claude/agents/*.md 与 .claude/agent-def-review-exempt

分支 claude/exciting-bohr-4olowk，HEAD d6a3bcb，基准 73ba4a4。时刻 UTC 2026-09-26。行号都是那份文件自己的（`cat -n` / `grep -n` 现取）。钩子判定用合成 PreToolUse JSON 喂 `.claude/hooks/*.sh`，`AGENT_HOOK_DETECTIONS` 指到草稿 `…/scratchpad/review3/B/`。没改仓里任何文件，没做 git 写。
已处置的（r1、r2 报告与 defs-r2、defs-r3 判决）不重报；defs-r3 第四节交用户的两项不重报。

## 逐条

| # | 文件:行 | 类别 | 严重程度 | 原句 | 证据 | 建议改法 |
|---|---|---|---|---|---|---|
| 1 | .claude/agents/kb-scribe.md:32（对 :22、:30） | ① 错误（r2 乙 8 写回引入） | 中 | 「红的是这个流程下一步会消掉的（21 号在第 3 步的 `21-decision-items-sync.sh --write` 之前、30 号在变更史条目写之前、49 号在 `--write` 之前、75 号在实验页那几行写之前），照流程往下走；其余的留到第 4 步一起判归属。」 | 第 3 步（:30）只在「分项翻状态」时才跑，它是整份定义里唯一跑 `21 … --write` 的地方；而 :22 收「新立分项、或改一条分项的定案 / 射程 / 依据」的规格，不翻状态。改了索引行或新立分项，21 号照红：把 `.claude/kb` 拷到草稿、把 D5 已定项 1 的索引行「维度元组里有哪几维」改成「维度元组里有哪几个维度」，`bash .claude/gate.d/21-decision-items-sync.sh <草稿>` 退 1，末行「→ 权威记录是各决策正文。改完正文跑： bash .claude/gate.d/21-decision-items-sync.sh --write」（未改前同一份副本退 0、「✓ 决策分项清单与正文同步（350 个分项）」）。照 :32 等第 3 步，第 3 步不跑，第 4 步 21 号仍红、判成「这一轮」却没有出路。r2 乙 8 报告说「第 4 步时它们已绿」，对翻状态以外的规格不成立。 | 把 `21-decision-items-sync.sh --write` 从第 3 步挪成独立一步：规格写了 `.claude/kb/decisions/` 下任何文件（第 1 步实写之后、第 3 步之后各一次）就跑；:32 括号改成「21 号在 `--write` 之前」。推翻它的现象：生成器不把分项名投影进 `decisions.md`（改索引行后 21 号仍绿）。 |
| 2 | .claude/agents/gate-triage.md:27（对 .claude/main-agent.md:59、.claude/gate.d/54-layer0-replay.sh:25、:190–195） | ② 与 main-agent 不一致 | 中 | 「这一轮的改动（`git diff --cached --name-only`；跑全量工作区时是主 agent 给的文件清单）碰了 `.claude/gate.d/stage-inputs.tsv` 里那一道那一行登记的路径 ⇒「这一轮」，下一步原样抄 54 号出路里的三行命令，一条都没碰 ⇒「不是这一轮」」 | main-agent.md:59：「这一批改了门禁 54 号在 `.claude/gate.d/stage-inputs.tsv` 登记的输入、54 号脚本本身或工具链（`cargo -V`、`rustc -V`；这三样都进全绿标记的键）」；54 号 :25「清单末尾再加两行：跑的这一份 54 号的 sha256…、`cargo -V` 与 `rustc -V` 原样输出的 sha256」，:190–195 把两样算出来写进清单。分诊只按登记路径判：一批只改了 54 号脚本（而分支更早碰过 `crates/`，`change-touches-crates.sh` 按 merge-base 判「碰了」、54 号照跑）时，红句「没有这批输入的全绿标记」会被判成「不是这一轮」。这个分支自己就改了 54 号（`git diff --stat 73ba4a4 -- .claude/gate.d/54-layer0-replay.sh` 4+/3-），只是没碰 `crates/`，所以今天 54 号退 77、没撞上。 | :27 那一句补两样：这一轮改了 `.claude/gate.d/54-layer0-replay.sh` ⇒「这一轮」；54 号出路里「与这一次比」列出的不同项只有 `<工具链：…>` ⇒「环境」。推翻它的现象：54 号的键不含脚本与工具链的哈希。 |
| 3 | .claude/agents/implementation-writer.md:18（对 .claude/main-agent.md:61、.claude/agent-common.md:10） | ③ 一步要的输入没人给 | 中 | main-agent:61「红在 `crates/mutations.tsv`（`relabel-item.py` 改写了 `crates/` 下的源码）：派 `implementation-writer` 修锚点，走代码轮」；implementation-writer:18「里程碑文件与步号（或并行线编号，或 `mutation-triage` 报告里要补取样点、补断言的条目），这一步的验收标准。」 | 实现员的输入一节没有「只修锚点」这种活（执行员有：experiment-runner.md:19「或者只修已有实验的变异表锚点…给表名、源文件与 33 号原样输出」）。共用约束:10「输入缺一样就不开工，回复只写缺什么」——主 agent 照 :61 派，实现员照定义要么停下要步号，要么硬套第 3、4 步（每条新测试证红、fmt/clippy/build）去修一行锚点。`relabel-item.py` 确会改 `crates/**/*.rs`（`.claude/gate.d/lib-item-ref-status.py:72–74` 的扫描集合含 `crates/**/*.rs`）。 | 实现员输入一节加一条「或者只修 `crates/mutations.tsv` 的锚点（书记员翻分项状态之后 33 号红）：给变异名、源文件与 33 号原样输出；只把原文改到源码今天的写法，跑 33 号，其余步骤不做」。推翻它的现象：`relabel-item.py` 不改 `crates/` 下的文件。 |
| 4 | .claude/agents/three-way-verifier.md:21、:28 | ③ 不清楚 | 低 | :28「先拿快照清单 `sha256sum -c` 核那个文件；对得上就到主树那一行（区间就取区间）比内容，对不上就只对主 agent 给的倒推副本核，两样都没有的记「分不清：文件可能在腿开工之后被改过」，不记 ✗」 | 定义没说 `sha256sum -c` 在哪个目录跑、怎么只核一个文件；而仓里已有的快照清单形态不一：`research/prompts/m2-final-code-r2-snapshot/crates-src-sha256.txt` 第 1 行「a6fccf64…  tree/crates/singlefs-checker/src/image.rs」带 `tree/` 前缀，同一目录还有 `defs-sha256.txt`、`kb-sha256.txt` 三份清单（`.claude/hooks/runner-dispatch-guard.sh` 文件头第三条②专门把 `./.claude/…`、`tree/crates/…`、`defs/.claude/…` 归一）。在仓根上对这种清单跑 `sha256sum -c` 是「No such file」，照 :28 落进「对不上」→ 没有倒推副本 → 一律「分不清」，本该判 ✗ 的误引核不出来。 | :28 补一句：从快照目录里按文件名挑出那一行，路径前缀照 `runner-dispatch-guard.sh` 的归一法换成仓根相对路径，只把这一行喂 `sha256sum -c`；换算不了的停下交主 agent，不记「分不清」。推翻它的现象：所有快照清单都写仓根相对路径、每轮只有一份。 |
| 5 | .claude/agents/three-way-verifier.md:21、:28 | ③ 不清楚 | 低 | :28「用 `git log --since=<腿开工时刻> -- <文件>` 与 `git status --short -- <文件>` 现查它在腿开工之后有没有被改过，照实写，不一律记成「被改过」」 | `git status` 只说工作区与 HEAD 不同，说不出是开工之前还是之后改的：腿开工之前就带着别的会话未提交改动、之后没再动的文件，照样出现在 `git status` 里。:21 设计轮那支「没改过记 ✗，改过或没给开工时刻记分不清」于是会把它记成「分不清」。方向保守（少报 ✗，不多报），但「现查有没有被改过」这一步用这两条命令做不到。 | 补一条可判的：`git status` 有它时再看 `stat -c %Y <文件>` 与开工时刻比，早于开工时刻的按没改过对主树核；或写明这种情形一律「分不清」、原因写「开工前就有未提交改动」。推翻它的现象：`git status` 能给出改动时刻。 |
| 6 | .claude/agents/three-way-attack.md:37（对 .claude/agents/three-way-verifier.md:30、:32，.claude/agent-common.md:45） | ② 定义之间对不上 | 低 | 攻方 :37「副本与自己的模型可以编译、跑（`cargo test -p <crate> --test <目标>`、`cargo run`，经内存包装、照共用约束看负载与线程上限；重型测试照样不跑）」；核查员 :32「核不动的（要编译、要虚机、要网络）写「核不动」与原因」 | 核查员第 4 步（:30）要在副本里复跑腿的每条复跑命令，而共用约束:45「不编译 Rust、不跑 `gate.sh` 全量，除非定义明写要做」、核查员定义没写要编译。攻方这一轮新放开的 Rust 模型与副本数，核查员一律「核不动」；main-agent.md:54 派核查员的条件正是「有腿交了模型、产物或复跑命令」。 | 二选一写明：核查员定义放开「经内存包装编译、跑腿的模型目录副本」（同攻方那半句的限制）；或攻方 :37 要求交 Rust 模型的同时交一份不用编译就能核的产物（原样输出与 sha256），判决里写明这类数只被攻方自己跑过。推翻它的现象：核查员定义里已有编译许可。 |
| 7 | .claude/agents/mutation-triage.md:27 | ③ 不清楚（这一处检查没有判别力） | 低 | 「59 号没有这句，看它收尾的「计数：」行里「共 N 条」与各栏加起来对得上…；对不上就是中途退出，这一次的数不算，照实报。」 | 59 号的「计数：」行（`.claude/gate.d/59-crates-mutation-replay.sh:407–408`）各栏与「共 {len(rows)} 条」出自同一份 judgements，永远加得起来；中途退出时它根本不打这一行：工作进程死了打「✗ 有工作进程中途死了（已判 …/… 条）」（:382），收回条数不齐打「✗ 派出去 … 条变异，只收回 … 条判定」（:389），都在计数行之前 `sys.exit(1)`。照 :27 做，「对不上」永远不会出现，真正的中途退出长成「没有计数行」。 | 改成：59 号的日志里要有「计数：」一行，「共 N 条」等于那一次 `crates/mutations.tsv` 里不以 `#` 开头的非空行数；没有计数行、或有 :382 / :389 那两句之一，就是中途退出。推翻它的现象：59 号的计数行与各栏来自不同的数据。 |
| 8 | .claude/agents/crash-verifier.md:24 | ③ 不清楚（defs-r3 D3 的收窄多收了一格） | 低 | 「`git ls-files --others --exclude-standard -- <未跟踪文件要查的路径>`（55、59 号查 `crates`，57 号查 `crates litmus .lkmm-static-only`：cargo 会把未跟踪的 `src/bin`、`tests` 文件认作目标…」 | 55 号不只经 cargo 读未跟踪文件：`.claude/gate.d/55-qemu-first-transaction.sh:159` 按 `replay.sh` 的 E142 行拼出 `research/results/<产物>` 去读，:192 起 `research/scripts/vm-bench.sh`。已暂存的 `replay.sh` 指向一份没进暂存区的新产物时，`git diff --quiet` 退 0、只查 `crates` 的未跟踪清单为空，55 号在工作区判绿，而要提交的那一批里没有那份产物。gate-staged 那一趟里 87 号会把它抓出来，所以只标低。 | 55 号那一格改成查 `crates research/results research/scripts`。推翻它的现象：55 号不按名字读 `research/results/` 下的文件。 |
| 9 | .claude/agents/crash-verifier.md:29 | ③ 不清楚（改前就有） | 低 | 「`git rev-parse HEAD`、`git diff HEAD -- crates litmus \| sha256sum`（暂存与没暂存的都算）、`git ls-files --others --exclude-standard -z -- crates litmus \| xargs -0 -r sha256sum \| sha256sum`」 | 第 1 步已经按每道的输入核（55、59 含仓根 `Cargo.toml`、`Cargo.lock`、`research/scripts/`，见 `.claude/gate.d/stage-inputs.tsv` 55、59 两行），第 6 步判「这几个绿对应的是不是同一份源码」的指纹却只罩 `crates litmus`：两道之间别的会话改了 `Cargo.lock` 或 `research/scripts/vm-bench.sh`，指纹不变、报告不写。 | 指纹的路径改成第 1 步那几道输入的并集。推翻它的现象：`Cargo.lock` 与 `research/scripts/` 不是 55、59 号的输入。 |
| 10 | .claude/agents/experiment-designer.md:28（对 .claude/agents/experiment-runner.md:27、:30、:32） | ③ 不清楚（r2 乙 6 / B6 写回引入） | 低 | 「重跑登记照抄原登记的英文名；原登记没有这一行的，从原实验的装置文件名 `e<号>_<英文名>.rs` 取。」 | 执行员第 2 步照英文名写 `research/e7-index-bench/src/bin/e<号>_<英文名>.rs`，重跑时这就是已有的那份源文件、它的 `replay.sh` 行按 `exact` 复跑旧产物。仓里的重跑先例两种都有：E155 的三次重跑各起了新名字（`research/scripts/replay.sh:172`「E155R2\|e155-second-run-fsync-write-volume\|\|…\|exact」、:173、:174；`research/e7-index-bench/Cargo.toml:425`、:429、:433 三个新 `[[bin]]`），E156、E158 改的是原装置。「照抄」把前一种写法排除了，而两份定义都没写重跑改了原装置之后旧产物那一行怎么办（执行员 :30 只写「对不上就按今天的日期另存一份…交主 agent 定哪份承重」）。 | 设计员 :28 写明两种里走哪种：要留旧产物可复跑的，重跑起新英文名（`<原英文名>_r<n>` 一类）、`replay.sh` 另起一行；改原装置的，登记里写明旧产物那一行改指向新产物还是删。推翻它的现象：仓里没有「重跑另起 bin」的先例（有，E155R2–R4）。 |
| 11 | .claude/agents/sweep.md:31（对 :13） | ③ 不清楚 | 低 | :31「在历史版本节或 `research/prompts/` 冻结证据里，不改（旧值是文件或目录路径的除外：路径不在冻结范围内，照 `.claude/rules/path-moves.md`「怎么做」逐步改）」；:13「只找、只分类，不改。」 | :31 的括号读起来是叫回扫员自己照 path-moves 的六步去改，与 :13 相反；回扫员 tools 只有 Read / Bash、写范围只有报告与草稿。照做不会写出去（没有别的指令），但分类落哪一类没写。 | 括号改成「…的除外：归「要改」，改法照 `.claude/rules/path-moves.md`「怎么做」，由主 agent 做」。推翻它的现象：回扫员定义另有改路径的写范围。 |

## 附带发现（不在乙组射程，只报不判）

- 门禁 12 号现跑红（`bash .claude/gate.d/12-no-prime-marks.sh` 退 1）：命中两处，其中 `research/prompts/governance-review-r1-report-B.md:61`（这个分支 4717077 新加，报告里转引了角标）是这个分支带进来的；另一处 `research/prompts/m2-lastflag-implementer-report.md:130` 在基准之前就在（f669c52 是 73ba4a4 的祖先）。
- `.claude/agent-def-review-exempt` 里 `.claude/agent-common.md` 三行、`.claude/main-agent.md` 两行的哈希都对不上这两份今天的内容（今天 agent-common c86014c2…、各行 2dc9d03a… / 19f44f94… / beb5d43d…），只有 `.claude/agents/three-way-defense.md` 那一行（6023f802…）还对得上。72 号只认哈希，旧行不起作用也不报错；`GATE_BASE=73ba4a4 bash .claude/gate.d/72-agent-def-adversarial-review.sh` 退 0：「✓ 改过的定义与共用约束 16 份都有去向（新写的判决文件 3 份，另有 1 份按 .claude/agent-def-review-exempt 豁免，基准 73ba4a4）」。不算错，列出来供收拢。

（附带第二条补：`.claude/main-agent.md` 今天是 e4e96f2e…，两行豁免 c80794a5… / fc197db3… 都对不上。）

## 核过没问题的

钩子：合成 JSON 喂 `heavy-test-guard.sh` 与 `bash-command-detector.sh`（部分另喂 `runner-dispatch-guard.sh`），除注明外都退 0。

| 定义 | 照定义写的命令 | 结果 |
|---|---|---|
| crash-verifier | `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/55-…`、`57-lkmm.sh`；`{ SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/59-…; echo "exit=$?"; } > <草稿>/59.log 2>&1`（前台与 `run_in_background` 各一次）；`=user-request` 跑 59；不带前缀跑 74、77、94 | 放行 |
| gate-triage | 带前缀 `research/scripts/gate-staged.sh`、`.claude/scripts/gate.sh`、87 号 | 放行；对照：不带前缀的 87 号退 2「gate-triage 跑「全部实验复跑」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request」；带前缀的 54 号退 2「gate-triage 不跑「层 0」」，与定义 :26「你不直接调它们」一致 |
| experiment-runner | `bash research/scripts/run-with-memory-cap.sh 4G bash research/scripts/replay.sh E158`；`cd research && nice -n 19 bash scripts/mutate.sh …`；`bash .claude/scripts/naming-lint.sh` | 放行；对照：不经包装的 `bash research/scripts/replay.sh E158` 退 2「脚本 …/replay.sh:383 里的 直接执行 cargo 编出来的二进制 ./target/release/e9-keylayout（experiment-runner 不经 run-with-memory-cap.sh）」，与 :32 的括注一致 |
| implementation-writer | `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features -- -D warnings`、`cargo build --offline --all-targets`；在 rsync 副本里 `cd <副本> && bash research/scripts/run-with-memory-cap.sh 4G cargo test -p singlefs-core --test foo` | 放行；`CODE_DISCIPLINE_LINTS` 在 `.claude/singlefs-ai-sop/scripts/check.sh:22–30`，七条 `-D`，与 :28 相符 |
| three-way-attack | 经包装的 `cargo test -p … --test …`（仓里与副本里） | 放行；对照：不经包装退 2 |
| three-way-local-attack | `bash research/scripts/ask-local.sh <提示> > <前缀>-output-s<n>.md`、`>|` 覆盖一份已存在的 `research/prompts/` 文件，`run_in_background` | 放行；`ask-local.sh:68–80` 的 void 取号自增、不覆盖，:12–13 退出码 5、6 都不打正文，与 :28 相符；`oov-check.py:241、:249` 收第二个参数当提示、生词只打前 300 字符，与 :29 相符 |
| mutation-triage | research 表的 `mutate.sh` | 放行；`mutate.sh` 的七种符号（:473–538）、「计数：内存撞顶 … 超时 …」（:647）、「已还原，基线仍全绿」（:663）、退出码 6（:255、:263）与 :17、:27、:28 相符；59 号 `judge_one`（:311–349）的判档次序与 :28 逐句相符（defs-r3 D1、D2 的写回没有引入新错） |
| kb-scribe | `printf '{"tool_name":"Edit","agent_type":"kb-scribe","tool_input":{"file_path":"<路径>"}}' \| AGENT_HOOK_DETECTIONS=<草稿>/precheck.jsonl bash .claude/hooks/write-guard.sh`（去掉 `CLAUDE_PROJECT_DIR` 也跑） | kb 路径退 0；`crates/…/lib.rs` 与 `research/prompts/foo.md` 退 2「kb-scribe 的写范围不含 …」，检出只写进草稿的 precheck.jsonl；`kb-scribe-followups.tsv` 第一行对 `decisions/` 跑 21、30、75 等，与 :32 列的四道相符（第 1 条之外） |
| three-way-verifier | `set -o noclobber; cat > <报告> <<'EOF'` | 放行；`git check-ignore -q .claude/singlefs-ai-sop/rules/test-discipline.md` 退 0、`git ls-files .claude/singlefs-ai-sop` 为空，与 :28 的例子相符；`*.at-snapshot` 命名与 `research/prompts/gate-fix-forks-r1-snapshot/47-research-script-selftests.sh.at-snapshot` 一致 |
| 派发闸 | `runner-dispatch-guard.sh` 喂：崩溃验证员「跑 55、57、59 号」、分诊「跑 research/scripts/gate-staged.sh」、变异分诊「59 号的输出路径是 …」两种写法 | 都退 0（实现员、执行员两条被拒是环境：开着的轮 m2-rollback-forward-r2 快照含 `crates/mutations.tsv`、E160 已有实验页，与定义无关） |

文字与脚本对照（读源码核过，没发现问题）：

- crash-verifier :24 的每道输入与 `.claude/gate.d/stage-inputs.tsv` 55、59 两行一致；57 号只经 `lkmm.sh` 读 `litmus/`、`crates/`、`.lkmm-static-only`（`57-lkmm.sh:16–19`、`lkmm.sh:254–281`），`lib.sh` 在被忽略的副本里（:42），与定义说法一致。
- gate-triage :26、:32：`gate.sh:100–105` 带 `--staged` 不前移 gate-ok、:535 不带时写 `refs/sop/gate-ok`；`gate-staged.sh:2` 全绿才前移 `refs/sop/staged-green`；调 `stage-must-run.sh` 的阶段只有 54、55、59、74、87（47 号只跑它的 `--selftest`）。
- experiment-runner :27–:35：`replay.sh:455–477` 的 `driver_e156`、`driver_e158` 在 `cd ..` 后 `cargo run`（仓根 `target/`）；E158 多份产物用后缀函数名（:178–191）；`crates/mutations.tsv` 第 1 行是六段表头；写范围表含 `crates/singlefs-harness/src/bin/e*.rs` 与 `crates/mutations.tsv`；56 号正则 `^crates/[^/]+/src/.*\.rs$`（:29）罩得到 bin；`.claude/decision-links-pending` 无非注释行，:35「清单里一条都没有时 ② 不适用」成立。
- experiment-designer :28 的 `claim-experiment.sh --next` / `E<号> 简称` 用法与脚本文件头相符；`_m2-code-r1-diff.md` 用 `git log --all --diff-filter=D --name-only -- "*_m2-code-r1-diff.md*"` 找得到（3cff909），materials :20、:28 的取法可行。
- kb-scribe :20：`lib-index-vs-body.py:29` 认「阿拉伯数字或一二两…十」、75 号 :226 同一字符集；`decisions-history.md` 第 8 行确是「变更史里的分项编号一律是「今天的编号」」；`lib-item-ref-status.py:72–74` 的扫描集合含 `crates/**/*.rs`，:30、:39 新加的 `crates/**/*.rs` 成立。
- sweep :17、:35：`stale-candidates.py:75` `MAXIMUM_CANDIDATE_LINES_PER_FACT = 500`、:344 只许第一个词超限时用 `&&`、:360–375 组号区间要 ASCII `-`，与定义相符；path-moves.md 有「## 怎么做」一节。
- local-defense :11 点名的三节在 local-attack 里都在（「## 输入（主 agent 必须给）」「## 做什么」「## 写范围」）。
- 三方与腿定义里点名的规则小节（three-way-inference.md「多轮：一次打穿不算数，三轮里多数打穿才算」、evidence-discipline.md「判据自己也会写错：打中之后先判是哪一种」、mutation-sampling.md「改了一个格式常量之后，要看「无效」那一栏有没有变多」、crash-test SKILL「判读纪律」）都在。

轻门禁现跑：

- `bash .claude/gate.d/10-kb-rot.sh` 退 0：「扫 29 份治理文档：门禁号 137 处、路径 330 处、小节 94 处；没判 10 处；跳过 70 处（被 .gitignore 挡着 69、不像仓内路径 1）」「✓ 治理文档里的门禁号、路径与「小节」都指得到」
- `62-stage-owners.sh` 退 0：「✓ 阶段归属表与门禁目录一致（75 个阶段，归 9 个 agent）」
- `63-agent-write-scope.sh` 退 0（「✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，…」）
- `GATE_BASE=73ba4a4 … 72-…` 退 0（见附带发现）；不带 GATE_BASE 退 77（基准就是 HEAD）
- `bash .claude/singlefs-ai-sop/scripts/doc-lint.sh .` 退 0：「✓ 文档铁律检查通过（检查 488，跳过 0；DOC_LINT_VERBOSE=1 看全部）」
- `12-no-prime-marks.sh` 退 1（见附带发现）

## 没做什么

- 重型测试一概没跑（54、55、57、59、87、整轮门禁、check.sh、全量 cargo test）；59 号的判档只读源码核，没真跑。
- `run-with-memory-cap.sh` 在这个容器里退 251（「带内存上限的 scope 起不来：…/proc/meminfo 的 MemTotal 读不到…」），小 crate 编译与实跑 `replay.sh`、`cargo test` 核不动；第 1 条的 21 号只在 kb 副本上跑（21 号不编译）。
- 没审 `.claude/agents/prior-art.md`、`three-way-forward.md`（这个分支没改它们）；main-agent.md、agent-common.md、规则只拿来对照，不在乙组射程。
- 第 2 条的误判情形是推的（这个分支没碰 `crates/`，54 号今天退 77，没撞上）；第 5、8、9 条的后果也是推的，没造现场。
