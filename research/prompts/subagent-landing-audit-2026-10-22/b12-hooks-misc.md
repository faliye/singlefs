# 第十二批任务 43：检出钩子、续做闸、进程安全扫描、看门狗与几处文件头

七条都做完。钩子与 scripts/ 下的脚本一律经 `scripts/gate/hook-swap.py` 换上（草稿里验、换上后再验），其余用 `scripts/kb/replace-once.py`。

## 改过的文件（换上前后 sha256）

| 文件 | 换上前 | 换上后 |
|---|---|---|
| .claude/hooks/bash-command-detector.sh | 531b2297e990… | b75940fb8472… |
| .claude/hooks/continuation-guard.sh | b9133473ad67… | f562a23e2dec… |
| .claude/hooks/lib_writer_lock.py（只改文件头） | 5d2832fb25ce… | ee3737462879… |
| .claude/hooks/heavy-test-guard.sh（只改注释） | 3d02af47c5d5… | d196737f0670… |
| .claude/hooks/selftest-registry.tsv（只改 merge-and-verify.sh 一行的理由） | — | 002f0757e113… |
| .claude/gate.d/code-tooling.sh（进程安全那一格的注释与样本表） | 12207baa89a3… | 33ee079a2d5f… |
| .claude/gate.d/break-switch-outside-selftest-pending.tsv（删 2 行） | — | 890996c5485d… |
| scripts/agents/agent-watch.py | dc885cfbe464… | 26a5a48dd593… |
| scripts/agents/watch.sh | fa9b6a8fb4a7… | 6c3dbbb0a273… |
| 新样本 .claude/gate.d/fixtures/code-tooling.sh/research-gate-lint-pending-outside-green/、research-gate-lint-pending-stale-red/ | 新建 | |

`git diff --stat` 对 HEAD 的数是累计了别的会话未提交改动的，不是这一批的增量（详见 logs/diffstat.txt）；这一批的增量以规格为准：specs/bcd.json 19 处、cg.json 6 处、lwl.json 3 处、htg.json 2 处、aw.json 7 处、ws.json 5 处，加 code-tooling.sh 3 处、registry 1 处、pending 表 2 行。

## 第 1 条：检出钩子 ⑫ 认 -p / --tmpdir= / 套一层的 mktemp（㉕）

在别的会话的 `landings_out_of_scope` 判法上改，不动 lib_write_scope.py。新函数 mktemp_option_directory、mktemp_inside_pwd、mktemp_created_directory：`-p <目录>`、`--tmpdir=<目录>`（可带不含 / 的模板名）指到自己认领的草稿目录里的算落点；不带值的 `--tmpdir` 同 -t；`$(cd $(mktemp …) && pwd -P)`（pwd / pwd -L、; 代 &&、反引号写法）按里面那条 mktemp 算，直接写在目标里的也认。`-p` 指到 /tmp、仓里、工作根本身、别的子 agent 或别的会话的草稿目录、带变量与通配的，照旧算不出、拒。

改前（原样）：真入口喂法 10 条，5 条该放行的退 2：
`✗ 退 2（应当 0） tooling-writer: mktemp -d -p 自己的草稿目录，之后写进去`、`--tmpdir=` 带模板名、`$(cd $(mktemp -d -p …) && pwd -P)` 套一层、`$(cd $(mktemp -d) && pwd -P)` 不带 -p、直接写进 `$(cd $(mktemp -d -p …) && pwd -P)/文件`（feeds/bcd.json，用本会话真实的会话号、agent 号与写者锁）。
改后：同一份喂法 10 条全对（hook-swap 草稿里与主工作区里各喂一遍）；5 条该拒的（-p /tmp、-p 仓里、-p 工作根、cd 之后接的不是 pwd）两版都退 2。

自证 `bash .claude/hooks/bash-command-detector.sh --selftest` 绿（778 种 → 799 种，新增 16 格 mktemp、3 格扫脚本、2 格守卫）。弄坏开关各核一遍，红的格与文件头表一致：

| 开关 | 红的格数 | 说明 |
|---|---|---|
| mktemp-option-directory-unknown（新） | 8 | -p、--tmpdir=、不带值的 --tmpdir、套一层里带 -p 的 |
| mktemp-nested-substitution-unknown（新） | 4 | 四种套一层的写法 |
| mktemp-template-owner-unchecked（旧） | 6（原 3，加 -p 别人的、--tmpdir= 别会话的、套一层别人的） | 文件头表已改 |
| mktemp-directory-unknown（旧） | 11（原 9，加不带值 --tmpdir、套一层不带 -p） | 文件头表已改 |
| mktemp-scratch-template-unknown（旧） | 5 | 没变 |

## 第 2 条：lib_writer_lock.py 文件头

「两处用这一份」改成「三处用这一份」，加检出钩子一行（只读用 find_lock_directory、read_lock、read_claim、launch_receipt、LAUNCH_SLACK_SECONDS，不建锁不建认领）；弄坏开关一句改成「红在哪几格写在 write-guard.sh 与 runner-dispatch-guard.sh 的文件头；检出钩子只读用，自证在十二项里任何一项打开时都不红」。后半句是实测：十二项逐个设 `LIB_WRITER_LOCK_BREAK` 起检出钩子的自证，十二次都退 0、零红（logs/bcd-lwl-*.txt）。`grep -rln lib_writer_lock .claude/hooks scripts .claude/scripts` 现数四个文件：runner-dispatch-guard.sh、bash-command-detector.sh、write-guard.sh、库自己。

## 第 3 条：续做闸与检出钩子「弄坏开关只在自证里认」（㉝ c）

两份都在入口最前面加 refuse_break_outside_selftest：平常跑（不是 --selftest、也不带自证标记）设了弄坏开关退 2，stderr 点名变量只在自证里认。自证起的子进程靠 `CONTINUATION_GUARD_SELFTEST_MARK` / `BASH_COMMAND_DETECTOR_SELFTEST_MARK` 指到自证建的标记文件（内容对得上）才认。新增弄坏开关 `CONTINUATION_GUARD_BREAK=honor-outside-selftest`、`BASH_COMMAND_DETECTOR_BREAK=break-honored-outside-selftest`（平常跑也照认，改前的样子）。

- 探针（门禁 break-switch-outside-selftest 格同款）：`env CONTINUATION_GUARD_BREAK=disable-check bash .claude/hooks/continuation-guard.sh --force --break-guard-probe-unknown-option </dev/null` 退 2，输出「✗ 续做闸：CONTINUATION_GUARD_BREAK=disable-check 是弄坏开关，只在自证（continuation-guard.sh --selftest）里认…」；检出钩子同款退 2。
- 自证：续做闸 30 种绿，检出钩子绿。新格「平常跑设了弄坏开关：退 2 并点名只在自证里认」、「自证起的子进程（带标记）照认弄坏开关」。开关核红：honor-outside-selftest 下续做闸红 1 格（应当返回 (2, True)，实际 (0, False)）；break-honored-outside-selftest 下检出钩子红 1 格；续做闸原有开关 disable-check（11 格）、anyrecord（2 格）红的数与文件头表一致，没被守卫搅乱。
- 已从 break-switch-outside-selftest-pending.tsv 删这两行（bash-command-detector.sh、continuation-guard.sh）。续做闸在 break-switch-replay-files.tsv 里，复跑时 `--selftest` 自带开关，不被守卫拒。

## 第 4 条：进程安全扫描清单里没扫目录下的行判本次未判（㉝ d）

判法在 bash-command-detector.sh 的 scan_scripts（门禁格只是调它）：登记表（.claude/process-safety-pending）里一行所在的目录不在这一趟交给 --scan-scripts 的目录里（只扫给它的那一层）、又没排到要放行的写法，判「本次未判」：stdout 打 `  ! 本次未判（这不是通过）：…点名文件与目录`，成功句里写「本次未判：…」，设了 GATE_NOT_RUN_FILE 的往里追加一行；不再算「一处要放行的都没有」。所在目录扫了而一处都没排到的，仍判红。
- 实测（复现主 agent 的趟）：`bash .claude/hooks/bash-command-detector.sh --scan-scripts <仓根> <仓根>/scripts/hosts` 退 0，输出「! 本次未判（这不是通过）：.claude/process-safety-pending:3 scripts/agents/agent-watch.py 所在的目录 scripts/agents 这一趟没扫…」。
- 自证绿 3 格 + 弄坏开关 `pending-outside-scanned-as-stale`（改前的样子）下红 2 格（「扫脚本:待改清单里的行所在的目录这一趟没扫：退出码 0、报本次未判」「…往 GATE_NOT_RUN_FILE 报一行」）。
- 门禁样本两份（红绿各一）：research-gate-lint-pending-outside-green（green 加 scripts/gate/deeper/killer.sh 与清单一行；改前判红，改后判绿）、research-gate-lint-pending-stale-red（清单点名扫了目录里的干净脚本，仍判红）。我用格里同样的目录集合（scripts、.claude/hooks、.claude/scripts、scripts/*/、.claude/gate.d）在两份样本根上起了检出钩子的 --scan-scripts：绿的退 0 且输出含 expect 里的两行，红的退 1 且含「scripts/gate/sample.sh 里一处要放行的都没有」，expect 照这个写。整格（stage-selftest）没起，见下「没做什么」。
- code-tooling.sh 只改了该格的注释、样本表（第 64 行格→样本对照）和样本说明，没改格的代码。

## 第 5 条：看门狗与 watch.sh（㉟）

agent-watch.py：
- `--help` 抛 ValueError 的根因是两条 help 里的 `1%、`、`100% 处`；改成 `%%`（经 HELP_PERCENT，弄坏开关 `AGENT_WATCH_BREAK=helpformat` 退回没转义）。改前原样：`ValueError: unsupported format character '?' (0x3001) at index 359`；改后 `--help` 退 0。自证加一格「--help 应当退出码 0 并打出用法」，helpformat 下红 1 格（正是这格）。
- 被杀留下的 `agent-watch-selftest-scratch-*`：自证建目录时写 `.selftest-pid`，开跑先 remove_stale_selftest_scratch（主人进程已不在的删；没有记号的旧版留下的超过一小时的删；主人还在的、不到一小时的、别的前缀的不动）。新格 selftest_stale_scratch_cleanup 造四种目录核；弄坏开关 `staleselftestscratch` 下红 1 格。
- 自证：正常绿（rc 0）；helpformat 红 1；staleselftestscratch 红 1（logs/aw-self-*.txt）。

watch.sh：键表抽成 KNOWN_CONFIG_KEYS，新加自证⑩：对键表里每个键造 `key=7` 的配置，核转成 `--key 7`，并核每个键在 `agent-watch.py --help` 里是选项。绿；弄坏开关 `WATCH_BREAK=floorkeydropped`（process-rss-floor-mebibytes 不转）下红「自检⑩：键表里这些键没转成同名的长选项： process-rss-floor-mebibytes」。

## 第 6 条：selftest-registry.tsv 的 merge-and-verify.sh 一行

理由补上：快档那一步多数格换上假的快档门禁，另有 quick-judge、quick-reuse、quick-red 三格在门禁 checker-tier-crash-point-replay 的样本根（有 .crash-point-replay-sample-tools/quick-tier）上真起那一道的 `--check crash-case-replay,feature-gated-tests`，起的仍是样本里的假 cargo、不编不跑 checker 档，档位不变，仍是 light。（三格名取自 scripts/gate/merge-and-verify.sh 自证里的 narrow_case。）

## 第 7 条：heavy-test-guard.sh 文件头

经 hook-swap（内置喂法 28 条，草稿与主工作区各喂一遍全对）。两处只改注释：
1. 「自证」一段补门禁目录顶层以 lib- 起头的共用库（lib-lag-baseline.py 这类）同脚本一样只认单独 --selftest、登记行写 light/heavy/heavy-if-set、不当样本档行；登记表里没有那一行的、库本身不实现 --selftest 的（lib-forbidden-notations.py 这类）带 --selftest 按表外自证拒。依据 lib_heavy_tests.py 文件头「自证」一段与 b9-gates-a.md「没做的」那条。
2. 实验复跑入口那一行：自证表里的注释改成「本仓那一份按名字判、不读正文，别处同名拷贝（路径尾部相同的也算）读正文」。文件头第 143 至 146 行别的会话已写好，不重复；所以改的是自证表里原来写「按名字判、不读正文」的那一行注释。
`bash .claude/hooks/heavy-test-guard.sh --selftest` 绿（查了 1442 种）。

## 没做什么

- 没跑重型测试，没走定义三方，没提交；没改定义、共用约束与规则；没改 .rs，命名检查不适用。
- 门禁样本档（stage-selftest 喂 code-tooling.sh）没起：闸要求阶段文件与样本和暂存区一致，code-tooling.sh 已有别的会话未暂存改动，新样本也没暂存。**要主 agent 在暂存之后跑**（仓根）：
  `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling 的样本档; 范围=1 道阶段; 为什么=核进程安全清单里没扫目录下的行判本次未判的两份新样本' bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh $PWD/.claude/gate.d --item code-tooling.sh`
  看 research-gate-lint-pending-outside-green（绿）与 research-gate-lint-pending-stale-red（红）两份。另外 break-switch-outside-selftest、break-switch-headers（续做闸文件头新开关）、break-switch-replay（续做闸复跑多一条 honor-outside-selftest）、agent-write-scope 几格是门禁格，我没起；续做闸与看门狗文件头照这些格的写法写了，推的是读数不变。
- 没动 .claude/process-safety-pending（agent-watch.py 那一行仍在，别的会话说的理由没变）；没动 agent-watch.py 以外两份 pending 里的行。
- 没改 lib_write_scope.py、lib_heavy_tests.py；没碰 handback-guard.sh、runner-dispatch-guard.sh 等兄弟任务的文件。
- 「自证起的子进程（带标记）照认弄坏开关」这一格没有专属的弄坏开关（它的失败形态是整个自证起不来的子进程被守卫拒），红绿只靠其余格间接覆盖。
- 草稿目录 ~/.local/share/singlefs/work/landing-audit-b12-hooks-misc 里：草稿树（draft/tree，3.1M）与样本拷贝（fx）已删；留 specs/（六份规格）、feeds/（喂法）、tools/（生成规格的脚本）、hook-swap/（喂法输出与反向规格）、logs/（各次自证输出）。没有仓副本与编译目录。

## 推翻条件

- bash-command-detector.sh 的 mktemp 新判法：任一条 feeds/bcd.json 里该拒的喂法放行了，或一个子 agent 能用 -p 指到别人的草稿目录写进去，就推翻。
- ㉝(d)：在 scripts/agents 以外的目录交给 --scan-scripts 时，登记行所在目录没扫却仍被报「一处要放行的都没有」，推翻。
- watch.sh ⑩：新加一个 agent-watch.py 认、watch.sh 键表里没有的键不会被它发现（⑩只核键表里的键，不反过来核 agent-watch.py 的选项都在键表里）。
