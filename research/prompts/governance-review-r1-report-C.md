# C 审查报告：分支 claude/exciting-bohr-4olowk 相对 73ba4a4 的门禁、钩子与脚本（HEAD 71f0cbc）

复现用的草稿与日志都在 `/tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/review1/`（下文简称「草稿」）。钩子喂的是合成的 PreToolUse JSON，`AGENT_HOOK_DETECTIONS=草稿/det.jsonl`。

## 逐条

| # | 文件:行 | 类别 | 原句或代码 | 证据 | 建议改法 |
|---|---|---|---|---|---|
| 1 | .claude/hooks/heavy-test-guard.sh:519（本分支改的 POLICY）对 :39、:129–130、自检 :719、:768–769 | 不一致 / 漏报 | 新文字：「crash-verifier 只跑 55、57、59 号与 qemu-system、lkmm.sh / herd7、crates 变异整表（54 号快档在 gate.sh --staged 里，全量由主 agent 跑）」；但代码 `AGENT_KINDS["crash-verifier"]` 仍含 `"layer0-stage", "layer0-cargo", "layer0-binary"`，文件头 :39 仍写「crash-verifier：层 0、55 号与 qemu-system-*…」，自检 :719「崩溃验证员带前缀跑 54 号全量」期望 0 | 以 crash-verifier 喂 `SINGLEFS_HEAVY_TESTS=commit nice -n 19 bash .claude/gate.d/54-layer0-replay.sh` → `rc=0`（放行）。`.claude/agents/crash-verifier.md:24` 写「54 号不归你」，`.claude/rules/implementation-workflow.md:55` 写「崩溃验证员跑 55、57、59 号」，:58 写「崩溃验证员…跑自己那一部分之外的…拒绝」——钩子实际不拒。拒绝信息说一套、判定做另一套 | 从 crash-verifier 的 AGENT_KINDS 里去掉三个 layer0 kind，改文件头 :39，自检 :719、:768 改成期望 2 并加一条「崩溃验证员带前缀跑 54 号快档 → 2」；或者规矩本意仍许它跑，就把 :519 改回去。两边二选一，别只改文字 |
| 2 | .claude/hooks/runner-dispatch-guard.sh:139、:439、:38、自检 :572 | 不一致 / 漏报 | `DISPATCH_HEAVY_OWN_SHARE["crash-verifier"] = {"层 0", …}`；拒绝信息 :439「crash-verifier 只跑 54、55、57、59 号那几道」；文件头 :38「crash-verifier 放行层 0」；自检 :572 派它「跑 54 号 --full…」期望 0 | 派 crash-verifier「提交流程里跑 54 号快档，命令带 SINGLEFS_HEAVY_TESTS=commit。」→ `rc=0`。本分支改了这个钩子（:40–42、:441），但同一文件里这几处还按旧分工，与 crash-verifier.md:24、main-agent.md:59「派 crash-verifier 跑 55、57、59」相反 | 与第 1 条同批：去掉 crash-verifier 的「层 0」、改 :439 与 :38、自检 :572 改成不含 54 |
| 3 | .claude/gate.d/stage-owners.tsv:37 对 .claude/agent-common.md:61–63 | 错误（照做被拒） | `54-layer0-replay.sh	gate-triage	层 0 崩溃点重放（快档在 gate.sh --staged 里跑；…）`；共用约束：「列出登记给你的…逐个 `nice -n 19 bash .claude/gate.d/<文件>`」 | gate-triage 照共用约束跑：喂 `nice -n 19 bash .claude/gate.d/54-layer0-replay.sh` → rc=2「✗ 重型测试被拒：门禁 54 号（54-layer0-replay.sh）（层 0）：gate-triage 不跑「层 0」」；带 `SINGLEFS_HEAVY_TESTS=commit` 前缀同样 rc=2、同一句。改前归 crash-verifier，它带前缀跑是放行的（上条 #1 的实测）。同形的 87 号归 gate-triage 是改前就有的，不算本分支 | 共用约束「门禁」一节写明 54、55、57、59、87 这几道重阶段不单跑（由 `gate.sh --staged` 或主 agent 跑），或在 gate-triage 定义里写明登记给它的 54、87 不单跑；62 号若要求第二列必须是 agent 名，第三列已写了去处，只补共用约束那一句 |
| 4 | research/scripts/archive-past-rounds.py:262（以及文件头 :20） | 错误（出路指向重型） | `print("     → 下一步：跑共享 gate.sh（「链接指向」阶段）确认没有指空的链接，再跑一次本脚本 --check 判绿。")` | `gate.sh` 是整轮门禁（重型）：主 agent 不带前缀喂 `bash .claude/scripts/gate.sh` → 「✗ 重型测试被拒：gate.sh（整轮门禁）：主 agent 跑「整轮门禁」要带 SINGLEFS_HEAVY_TESTS=commit 或 =user-request，这一条没带」；子 agent 一律拒。那一道单跑就是 `python3 .claude/singlefs-ai-sop/scripts/link-targets.py`（共享 gate.sh:269 `run_stage "链接指向" bash -c 'cd "$1" && python3 "$2"' _ "$ROOT" "$SCRIPTS/link-targets.py"`），实跑 rc=0、末行「  ✓ 文档指向都到得了（1309 条相对链接、47 处「第 N 节」指向）」，钩子放行 | 改成「在仓库根跑 `python3 .claude/singlefs-ai-sop/scripts/link-targets.py`（共享 gate.sh「链接指向」那一道）」 |
| 5 | .claude/gate.d/lib-governance-refs.py:27 | 误报 | `GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.第])((?:[0-9]{2}\s*、\s*)*[0-9]{2})\s*号")` | 草稿/probe 合成治理文档：「提交窗口是 9 月 23 号前后。」→「✗ .claude/agents/p.md:1: 「23 号」在门禁目录里没有 23-*.sh」；「按第 13 号决策办。」（「第」后有空格，负向后顾只看紧挨的一个字）→「「13 号」…没有 13-*.sh」。今天仓里 0 处命中（10 号判绿），是潜在误报 | 后顾改成排除「第\s*」「月\s*」，或只认前面有「门禁」「阶段」或同一串「、」里有门禁上下文的 |
| 6 | .claude/gate.d/lib-governance-refs.py:29、:58–65 | 误报 | `REPO_PATH = re.compile(r"^[A-Za-z0-9_.\-/]+$")`，凡含 `/` 的 ASCII 记号都当仓内路径 | probe：`origin/master`、`claude.ai/code/artifacts`、`text/plain`、`anthropics/claude-code` 各报一条「…在仓里不存在」，rc=1。今天仓里 0 处（10 号绿） | 只认首段是仓里现存顶层目录（或 `.claude/`、`kb/`、`rules/` 这几个解析基准下现存目录）的记号；或者要求带扩展名、以 `/` 结尾之一 |
| 7 | .claude/gate.d/lib-governance-refs.py:98–101 | 漏报（跳过没列出） | `target = resolve(match.group(1), carrier)` / `if target is None or not os.path.isfile(target): continue` | 文件解析不到时整条「小节」不判、不计数、不报。仓里今天就有 8 处被静默跳过（草稿/skipped.py 实测）：experiment-designer.md:63、sweep.md:33、:34、implementation-workflow.md:22（`evidence-discipline.md` / `show-me-test.md` 的裸名在 `.claude/singlefs-ai-sop/rules/` 下，四个解析基准都不含），experiment-runner.md:32、path-moves.md:10、:29、:31（`gate.sh`「转发计时」「链接指向」「编号与简称」）。这 8 处我手查都还指得到，但门禁没查。probe 里 `gone.md`「任何」（文件根本不存在）零报告；成功行「扫 29 份治理文档：门禁号 106 处、路径 309 处、小节 94 处」不提跳过的 8 处，违反 show-me-test.md「没查的是哪些也要逐个列出」 | resolve 加 `.claude/singlefs-ai-sop/rules`、`.claude/singlefs-ai-sop/scripts`、`.claude/scripts` 三个基准；仍解析不到的逐条列成「没查」并计数，文件名带扩展名却哪都找不到的判红 |
| 8 | .claude/gate.d/lib-governance-refs.py:29（REPO_PATH 只收 ASCII） | 漏报（跳过没列出） | 文件头 :9「ASCII 写成、中间带 `/` 的」 | probe：`.claude/kb/decisions/99-不存在的决策.md` 零报告。决策、实验文件名全是中文（`NN-简称.md`），这一类悬空路径全部够不着；仓里今天被跳过的中文路径记号 7 个（其中 2 个是占位 `NN-简称.md`），成功行不列 | 字符集放开到非 ASCII（占位另行排除），或至少把跳过的逐条列出 |
| 9 | .claude/gate.d/lib-governance-refs.py:12–13 对 :102–106 | 不一致 / 漏报 | 文件头：「标题里找不到、正文粗体里找得到的不判」；代码：`haystack = DECORATION.sub("", handle.read())` 后 `if DECORATION.sub("", section) not in haystack` | 实际判的是「去掉空白、反引号、星号、「」之后在全文任意位置出现」，不看标题也不看粗体。草稿里造 `invariants.md` 只有标题「已实现的检查」「历史版本」，引用 `…invariants.md`「检查」与「历史」都判绿（rc=0）：小节被改名、而原名的字还散在正文里时抓不到 | 文件头照实写「在全文任意位置出现就算」；要严一点就先比标题行，再退到粗体 |
| 10 | .claude/gate.d/fixtures/10-kb-rot.sh/green/.claude/agents/sample.md:8 | 判别力不足（样本没罩住） | 「`cd .claude && ls kb/invariants.md` 按 cd 之后的目录解析」 | `kb/invariants.md` 不靠 cd 也能经 resolve 的 `.claude` 基准找到。把 lib 里 `cd` 那一支删掉的副本（草稿/mut/lib-nocd.py，只改 :95–96 与门禁目录）在 green 样本上照样 rc=0、「扫 1 份治理文档：门禁号 3 处、路径 4 处、小节 1 处」：cd 处理这一支没有样本证它有用 | green 里 cd 到一个不在四个基准里的目录（如 `cd research && ls scripts/样本.sh`，并在样本里放那个文件），red 里放一条 cd 之后仍指不到的 |
| 11 | .claude/gate.d/fixtures/10-kb-rot.sh/red/.claude/agents/sample.md:7 与 lib-governance-refs.py:24 | 不清楚（样本脆） | 红样本靠「门禁 23 号」判红；门禁目录取 lib 所在的真门禁目录 | 今天 `.claude/gate.d/` 没有 23-*.sh，样本判得对；将来有人新加 23 号阶段，red 样本的 want「「23 号」在门禁目录里没有 23-*.sh」会因为与本段无关的原因失效 | expect 旁写明这一依赖，或让门禁目录可由环境变量指到样本里的假门禁目录 |
| 12 | research/scripts/ask-local.sh:104、:109、:115 | 错误（环境泄漏） | `UNCHECKED=1` / `if [[ -n "${UNCHECKED:-}" ]]; then … exit 6` —— 循环前没有清零 | 干净正文、两个检测器都在：不带环境变量 rc=0；调用方环境里有 `UNCHECKED=1` 时 rc=6，stderr 只有「ask-local: 作废轮的原样输出已留存 …/u-output-void1.md」「ask-local: 这一份没过完字词损坏检查，按没验过处理：退出 6，不打正文」「下一步：照上面那几行修好检测器…」，而「上面那几行」不存在。`VOID_SAVED` 同形，改前就有 | 循环前写 `UNCHECKED=`（顺手 `VOID_SAVED=`） |
| 13 | research/scripts/ask-local.sh:1–9 与 :113 | 不清楚 | 注释「6 不与判红的 5、网关的 2 / 3 / 4 混用」；文件头没有退出码表 | 实际：2 是取不到 key（:17）或提示为空（:21），不是网关；3 是请求失败 / 非 JSON / 网关报错；4 是正文为空；5 判红；6 没验过。调用方（three-way-local-attack.md:28「退出码非 0 非 5（包括 6…）」）只能从正文里拼 | 文件头加一张退出码表（2 / 3 / 4 / 5 / 6 各一行），:113 的「网关的 2」改准 |
| 14 | research/scripts/ask-local-selftest.sh:10–11 | 不清楚（照做会因别的原因红） | 「自证这份自检会红时，指向一份改回旧写法的副本（ASK_LOCAL_SCRIPT=副本路径）」 | 被测脚本按 `dirname "$0"` 找检测器，oov-check.py 又按自己目录的 `../data/en-words.txt` 找词表。副本放在草稿根目录时，旧版 ① 就红「✗ 损坏正文没判红（退出码 0，应为 5）」（检测器找不到）；新版副本放在只拷了两个检测器、没有 data 的目录时 ② 红「✗ 干净正文被判红（退出码 6，应为 0）」。只有把副本放在检测器与 `../data/en-words.txt` 都在的位置，红的才只是被改的那一支 | 注释写明副本要放在 research/scripts 同构的位置；或自检自己把两个检测器与 data 拷到副本旁边 |

## 跑过的门禁与自证（原样末行、退出码、是不是这条分支带进来的）

| 跑的 | 退出码 | 原样末行 | 本分支带进来的吗 |
|---|---|---|---|
| `bash .claude/gate.d/10-kb-rot.sh "$PWD"` | 0 | `  ✓ kb 腐化审计通过`（第 4 段：`  扫 29 份治理文档：门禁号 106 处、路径 309 处、小节 94 处`） | — |
| 10 号 red 样本（stage-selftest 起法：拷进 mktemp、阶段绝对路径、临时目录当根） | 1 | `               放着不动会让下一轮再审一遍同样的东西。`；expect 6 条 want 全中 | — |
| 10 号 green 样本（同上） | 0 | `  ✓ kb 腐化审计通过`；expect 4 条 want 全中 | — |
| `bash .claude/gate.d/62-stage-owners.sh "$PWD"` | 0 | `  ✓ 阶段归属表与门禁目录一致（75 个阶段，归 9 个 agent）` | — |
| `bash .claude/gate.d/63-agent-write-scope.sh "$PWD"` | 0 | `  ✓ 写范围闸、Bash 检出 hook、重型测试闸、续派闸、续做闸与弹窗断言闸注册着、自证通过，…（全文见草稿/gate-63-agent-write-scope.log）` | — |
| `bash .claude/gate.d/73-research-gate-lint.sh "$PWD"` | 0 | `  ✓ shell 纪律检查通过（共 9 个脚本）` | — |
| `bash .claude/scripts/gate-lint.sh`（不设 GATE_LINT_DIR，全仓扫） | 1 | `  ✗ 门禁自检失败：27 处（共 460 个脚本、831 条拒绝）` | 否：27 处落在 17 个文件（`.claude/gate.d/fixtures/73-research-gate-lint.sh/red/` 下 2 个红样本与 `research/prompts/` 下 15 个冻结的模型脚本），逐个 `git diff --quiet 73ba4a4 HEAD -- <文件>` 全部 same。我负责的 13 个文件一处没有 |
| `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py .` | 0 | `    没判的：装进来的 SOP 副本 30 份只当对照（.claude/singlefs-ai-sop/scripts）`（上一行 `  ✓ 相对 73ba4a4c019b：新加的门禁与钩子 0 个（无）都写明了比过谁，改过的 7 份脚本对照已有的 124 份没有整段相同`） | — |
| `bash .claude/hooks/heavy-test-guard.sh --selftest` | 0 | `  ✓ 自检通过（查了 562 种，其中该拒 98 种）：…`（全文见草稿/self-heavy-test-guard.log） | —（但自检 :719、:768 钉的正是 #1 的旧分工） |
| `bash .claude/hooks/runner-dispatch-guard.sh --selftest` | 0 | `  ✓ 自检通过（查了 112 种）：…`（全文见草稿/self-runner-dispatch-guard.log） | —（自检 :572 钉的是 #2 的旧分工） |
| `bash .claude/hooks/bash-command-detector.sh --selftest` | 0 | `  ✓ 自检通过（查了 430 种）：…`（全文见草稿/self-bash-command-detector.log） | — |
| `bash research/scripts/ask-local-selftest.sh` | 0 | `  ✓ ask-local 判红分支自检通过（4 个用例、11 条断言：损坏留证退 5 且 stdout 为空、干净不留证退 0 且 stdout 等于正文、检测器崩了与找不到各退 6 且 stdout 为空并留证）` | — |
| 同上，`ASK_LOCAL_SCRIPT=` 73ba4a4 版 ask-local.sh（放在拷了两个检测器的目录） | 1 | `    测试缝是 ASK_LOCAL_FAKE_TEXT（指向一份现成正文时跳过网关）。`；红的正好是新加的 6 条（崩了、找不到各 3 条：「应退 6，实际 0」「stdout 却有正文」「没留下作废副本」），①② 不红 | 新断言会红，证实 |
| `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d` | 1 | `                样本该改就改 expect，检查坏了就改那个阶段——别两边一起改到自洽为止。`（汇总行 `  ✗ 5 个样本判错（判对 136 个，4 个阶段没有样本）`） | 否：10 号 red、green 都「判得对」；判错的是 55 号 red/green、59 号 red/green、74 号 green，这三个阶段与它们的样本 `git diff --stat 73ba4a4 HEAD` 为空。原因没查；容器以 root 跑、没有用户级 systemd scope，55/59 那几条可能是环境（推的，没量过） |

钩子实喂（合成 PreToolUse JSON）另外核过、没有问题的：
- bash-command-detector 新出路 `{ a; echo $? > a.rc; } & { b; echo $? > b.rc; } & wait`：run_in_background 为 true / false 都 rc=0，检出记录为空。
- 54 号 `print_staged_worktree_full_commands` 打出的三行整段当一条 run_in_background 命令：bash-command-detector rc=0、heavy-test-guard（主 agent）rc=0。
- runner-dispatch-guard 新写的「会误拒」说明与实际一致：「不跑重型测试（check.sh、`cargo test --workspace`）」rc=2；「不跑 check.sh、`cargo test --workspace`」rc=0；只指清单出处不列名字的 rc=0。
- 10 号接法：rc 0/1/2 三支都有出路；在不是 git 仓的 mktemp 目录里跑，`git check-ignore` 失败按「没被忽略」算，样本判得对；stage-selftest 起法下 lib 取的是真门禁目录。
- lib 对 `@标题`、`:行号`、`~正则` 后缀的切法与 `cd research && bash scripts/x.sh`、`cd crates/core && ls src/lib.rs` 都没有误报（probe 行 L4–L6）。
- stale-candidates.py 文件头的 500 与代码 `MAXIMUM_CANDIDATE_LINES_PER_FACT = 500`（:75）一致；69 号删掉的那半句、54 号出路加的前缀与内存包装、stage-owners 以外的文字改动没发现问题。

## 没做什么

- 没改仓里任何文件，没做 git 写操作（没建 worktree；73ba4a4 版本用 `git show` 取到草稿里）。
- 没跑重型测试：54 全量 / 快档、55、57、59 单跑、gate.sh、全量 cargo test 都没跑；stage-selftest 里 55、59 的样本判错没追原因。
- lib-governance-refs.py 的误报只在 probe 合成文档上证过，今天仓里这几种写法命中 0 次（10 号绿）。
- 没核 #7 那 8 处之外的 94 处「小节」判得对不对，也没逐条核 309 处路径。
- 没审我负责清单之外的文件（agent 定义、rules、main-agent.md 只在判一致性时读了相关行）。
