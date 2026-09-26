# 丙组严查第三轮（确认轮）：分支 claude/exciting-bohr-4olowk（HEAD d6a3bcb）相对 73ba4a4 的 .claude/gate.d、.claude/hooks、research/scripts

草稿（下文「草稿」）：`/tmp/claude-0/-home-user-singlefs/77b4b1be-00af-5f85-8659-f6e6d51fb840/scratchpad/review3/`。`草稿/runmut.sh <原串> <新串>` 把 `.claude/gate.d/*.sh *.py` 拷进 `草稿/gd/`、在 lib 副本上定点替换一处（写临时文件再 `os.replace`），在两份 10 号样本的临时拷贝上跑副本里的 10 号，逐条对 expect 的 exit 与 want。ask-local 的改坏在 `草稿/al/` 同构副本上做。第一、二轮已处置的与 governance-defs-r3「交用户的」两项不重报。

## 逐条

| # | 文件:行 | 类别 | 严重程度 | 原句或代码 | 证据 | 建议改法 | 什么现象会推翻它 |
|---|---|---|---|---|---|---|---|
| 1 | .claude/gate.d/fixtures/10-kb-rot.sh/red/expect:1–10 对 .claude/gate.d/lib-governance-refs.py:158、.claude/gate.d/10-kb-rot.sh:180 | ① 判别力：第 4 段的红支没有样本 | 高 | expect `exit=1`，第 4 段只钉了 lib 逐条打的 4 行 ✗；lib `return 1 if problems else 0` | `草稿/runmut.sh 'return 1 if problems else 0' 'return 0'` → `green: judged OK`、`red: judged OK`，改坏没被抓。副本在 red 样本上的输出：4 行 ✗ 照打之后紧跟 `✓ 治理文档里的门禁号、路径与「小节」都指得到`，整道退 1 只因为第 1–3 段本来就红。也就是说「lib 报了指不到、10 号照样判这一段绿」这种坏法在今天的样本下判不出来；真仓里第 1–3 段绿时，悬空的门禁号、路径、小节会整道放行。第 4 段的 rc 3、「没跑成」两支同样没有样本 | red expect 加 `want=治理文档里有指不到的指向（上面逐条列出）`（这一行只在 10 号走 rc==1 那一支时出现）；更稳的是另立一份只有第 4 段悬空、第 1–3 段全绿的红样本，钉 `exit=1` | 有人给出别的样本或自检在「lib 返回 0 却有 ✗」时判红 |
| 2 | .claude/gate.d/lib-governance-refs.py:24 对 :106、:145 与 10-kb-rot.sh:177–180 | ② 文件头与行为相反 | 中 | 「python 自己出错退别的码（打不开脚本是 2），调用方当「没跑成」。」 | python 未捕获的异常退 1，正好撞上「1 有指不到的」。草稿里在绿样本拷贝的 `.claude/agents/` 下放一份含 `\xff\xfe` 的 zz.md 跑 10 号：先是 `UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff…` 的 traceback，随后 `✗ 治理文档里有指不到的指向（上面逐条列出）`，出路叫人改门禁号、路径、小节——上面其实一条都没列。被 ③ 点名的目标文件（`.sh`、`.py` 也在 FILE_WITH_SECTIONS 的射程里）读不成 UTF-8 时同样。退出码仍非 0，不会假绿，但「没跑成」被报成「有悬空指向」，出路指错 | `main()` 外包一层 `try/except Exception`，打 `✗ lib-governance-refs.py 自己出错：…` 与 `→` 出路后 `return 4`；文件头改成「python 自己出错退 4（打不开脚本是 2）」 | python 的未捕获异常退出码不是 1（它就是 1，见 CPython 文档；除非有人证明这几处 open 永远不会抛） |
| 3 | .claude/gate.d/lib-governance-refs.py:14、:88–91、:157 | ② 成功行报的「跳过」比文件头列的少 / show-me-test「没查的是哪些也要逐个列出」 | 中 | 文件头「跳过：被 .gitignore 挡着的…、`../` `/` `~` 起头的、`refs/` 起头的 git 引用、带占位（`<` `*` `{`）的」；末行 `跳过 {ignored + not_repo_path} 处（被 .gitignore 挡着 …、不像仓内路径 …）` | 第二轮丙 3 的处置只把两类计了数。`草稿/probe_skips.py` 在真仓上数：`path_token` 返回 None 的带 `/` 记号 95 处（起头被跳过 19，例 agent-common.md:74 `/tmp`、crash-verifier.md:28 `/dev/kvm`；带占位 76，例 agent-common.md:43 `.claude/settings*.json`、:63 `.claude/gate.d/<文件>`），一处都不进计数。真仓末行 `跳过 70 处（被 .gitignore 挡着 69、不像仓内路径 1）`，实际跳过 165 处；读的人会把 70 当成总数。`http` 起头的也跳过，文件头没列 | 末行补两栏（`起头是 ../ / ~ refs/ http 的 A`、`带占位的 B`），总数取四栏之和；文件头的跳过清单补 `http`；绿样本放一条 `/tmp` 与一条 `<占位>/x.md` 并钉末行 | 有人说明这 95 处按 show-me-test 不算「对象」（那文件头就不该把它们写成「跳过」） |
| 4 | .claude/gate.d/lib-governance-refs.py:35–36 对 fixtures/10-kb-rot.sh/green | ① 判别力：两支没有样本 | 低 | `GATE_NUMBER = re.compile(r"(?<![0-9A-Za-z.\-–])…")`；`NOT_A_GATE_BEFORE = re.compile(r"(第\|月\|日)\s*$")` | `草稿/runmut.sh` 删掉后顾里的 `–`：green、red 都 `judged OK`；把 `(第\|月\|日)` 改成 `(第\|月)`：同样都 `judged OK`。文件头 :8–9 承诺的「54–59 号 不判」「日 后面的不算」今天没有样本证它会红。对照：删 `/` 分隔、删 `\-`、删 cd 基准、删「不像仓内路径」计数都被绿样本的末行 want 抓到 | 绿样本加「9 月 23 日 12 号」与「54–59 号」各一处（门禁号计数随之不变，改坏就多数或误报） | 有人给出这两支已被别处样本罩住的证据 |
| 5 | .claude/main-agent.md:59（「暂存之后、提交之前跑门禁」那一行）、.claude/agents/gate-triage.md:26、.claude/rules/implementation-workflow.md:55 对 .claude/hooks/runner-dispatch-guard.sh | ② 定义里的说法照抄进派发提示会被拒 | 低 | 「要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑」 | 喂 runner-dispatch-guard：gate-triage、提示「带 SINGLEFS_HEAVY_TESTS=commit 跑 research/scripts/gate-staged.sh 并分诊：…；要跑时 54 号跑快档并核全绿标记，57 号没有复用、每次现跑。」→ rc=2，`✗ 派 gate-triage 的提示要它跑「层 0」`。这句是这个分支新写进 main-agent.md 的（73ba4a4 里 0 处），属于钩子文件头已登记的「名字在前、动词在后」误拒形态；钩子出路给得出改法，所以只是摩擦，不会误判 | main-agent.md 那一行写成「整轮里的 54 号只核快档与全绿标记」这类不带「跑」作谓语的说法，或给派发提示加一句「不抄这一段，只指到 gate-triage 定义第 2 步」 | 有人说明主 agent 从不把这一段写进派发提示 |
| 6 | .claude/hooks/runner-dispatch-guard.sh:569 | ③ 自检样本里的事实过时 | 低 | `case("重型:只提到、没要它跑", "implementation-writer", "层 0 归 crash-verifier；check.sh 那一套 lint 下的 clippy 要过。", 0)` | 这个分支把层 0 从崩溃验证员那一份拿掉了（:139、:38），样本句子仍写「层 0 归 crash-verifier」。判的是「只提到不算要它跑」，判别力不受影响；但样本是给人照抄的范例 | 改成「层 0 全量归主 agent；…」 | — |
| 7 | research/scripts/ask-local-selftest.sh:11–12 | ③ 注释与行为不符 | 低 | 「副本要放在与 research/scripts 同构的位置：它按自己所在目录找两个检测器，oov-check.py 又按 ../data/en-words.txt 找词表，缺了哪样，红的就是「检测器找不到」而不是被改的那一支」 | oov-check.py:54–56 读词表失败 `sys.exit(EXIT_BROKEN)`，ask-local.sh:105–111 把它报成「字词损坏检查**没跑成**（退出码 2）」，不是「找不到」；只有缺检测器本身才是「找不到」 | 写成「缺检测器红的是「找不到」、缺词表红的是「没跑成」，都不是被改的那一支」 | — |

## 核过、没发现问题的

| 核的 | 怎么核的 | 结果 |
|---|---|---|
| 第二轮丙 1 处置：54 号出路三行（`layer0_tree_ready`、第三行） | 从 54-layer0-replay.sh 抽出 `print_staged_worktree_full_commands` 实打；临时桩仓（桩 54 `exit $STUBRC`、桩内存包装 `exec`）整段跑 | `--full` 退 0 / 1 / 250 时整次调用退 0 / 1 / 250，worktree 清干净；三行当主 agent 的 run_in_background 命令喂 heavy-test-guard、bash-command-detector 都 rc=0；去掉 `SINGLEFS_HEAVY_TESTS=commit` 后 heavy-test-guard 拒——`then` 之后的前缀认得出 |
| 第二轮丙 2 处置：10-kb-rot.sh 的 GATE_DIRECTORY 与退出码 3 | 在 `.claude/gate.d/` 里直接 `bash 10-kb-rot.sh`；读 :168–183 | 退 0；rc 0 / 3 / 其它非 1 / 1 四支各有 ✓ 或 bad+howto（rc 1 撞号见第 2 条） |
| 第二轮丙 3、4、6、7 处置：lib 的跳过计数、「/」「-」、说明文字、样本 | 真仓跑 lib；`草稿/runmut.sh` 逐支改坏 | 真仓 rc=0，末行「扫 29 份治理文档：门禁号 137 处、路径 330 处、小节 94 处；没判 10 处；跳过 70 处（被 .gitignore 挡着 69、不像仓内路径 1）」；删「/」分隔、删 `\-`、删「不像仓内路径」计数、删 cd 基准、删小节判定各自被 green 或 red 抓到；样本里的「10 / 12 号」「2026-09-26 号」「origin/master」「text/plain」判得对（跳过计数与两支例外见第 3、4 条） |
| 75 号 `\s*` | 读 :226 与 20-kb-shape.sh 取数正则、format-evolution.md:26 | 「（3 项未定）」「（两项未定）」都合；stage-selftest 里 75 号红绿判得对 |
| heavy-test-guard.sh 自证与出路 | `--selftest`；`HEAVY_TEST_GUARD_IGNORE_MEMORY_CAP=1 --selftest`；喂 gate-triage / crash-verifier 直接调 54 号 | 自检退 0（562 种）；关掉内存包装那一道时 12 条红，含「崩溃验证员带前缀不经内存包装跑测试目标 应当是 2，实际 0」；两者直接调 54 号都 rc=2，拒绝行与 POLICY、crash-verifier.md:24、gate-triage.md:26、implementation-workflow.md:55 一致 |
| runner-dispatch-guard.sh 自证与出路 | `--selftest`；喂 crash-verifier「跑 55、57、59」「…层 0 全量由主 agent 跑，你不跑」 | 自检退 0（113 种）；两条都 rc=0；出路「提交时 QEMU、herd7、crates 变异整表派 crash-verifier，层 0 全量由主 agent 自己跑」与 main-agent.md 一致 |
| ask-local.sh 与 ask-local-selftest.sh 的 VOID_SAVED / UNCHECKED 与退出 6 | 自检原样跑；同构副本四种改坏：只删 `VOID_SAVED=""`、只删 `UNCHECKED=""`、`exit 6` 换成 `:`、「找不到」支不设 UNCHECKED | 原样「✓ …（4 个用例、11 条断言…）」退 0；四种改坏都 mutrc=1（各报「判红了却没留下 case1-output-void1.md」「干净轮也留了 void 文件」「检测器找不到时 stdout 却有正文」「检测器找不到时没留下作废副本」）；文件头 :11–14 的退出码表与 :22、:26、:37、:48–59、:103、:126 对得上 |
| bash-command-detector.sh 出路里的并行写法 | 把 `{ sleep 1; echo $? > …/a.rc; } & { sleep 2; echo $? > …/b.rc; } & wait` 当 run_in_background 喂主 agent 与实现员 | 都 rc=0；与 agent-common.md:55 的写法一致 |
| stage-owners.tsv 54 → gate-triage、69 号、90 号、20 号注释、archive-past-rounds.py、stale-candidates.py | 读 diff；`link-targets.py` 存在、gate.sh:269 的阶段名是「链接指向」、单跑退 0；stale-candidates.py:75 `= 500` | 说法与代码、规则一致，没丢今天成立的判据 |
| 本地阶段判别力 | `bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d`（后台） | 10、20 号红绿判得对；「✗ 5 个样本判错（判对 136 个，4 个阶段没有样本）」，判错的只有 55、59、74，已知环境原因 |

## 顺带看到、不是这个分支带进来的

- ask-local.sh:98–104：检测器未捕获的异常退 1，会被当成「判为字词损坏」退 5，与 :106–108 注释「检测器自己崩了不等于判红」相反；`case 1)` 一支 73ba4a4 就有。两个检测器的已知失败都走 `EXIT_BROKEN`，只剩意外异常这一格。

## 没做什么

- 没改仓里任何文件、没做 git 写；改坏都在草稿副本里。
- 没跑重型：54、55、57、59、87、gate.sh、gate-staged.sh、check.sh、全量 cargo test 都没跑。
- lib 的 330 处路径、94 处小节、137 处门禁号没逐条人工复核，只核了分类与样本判别力。
- 没审 .claude/agents、rules、main-agent.md 本身的改动，只在判一致性时读了相关行。
