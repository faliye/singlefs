# watchdog-knowledge-sync-governance-r2：判决（2026-11-18）

正文 `research/prompts/_watchdog-knowledge-sync-governance-r2-body.md`，开工快照 `research/prompts/watchdog-knowledge-sync-governance-r2-snapshot/`。这是这一题的第二轮（治理文件轮），前一轮判决 `research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md`。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 攻方腿（第 1 到 7 问） | `research/prompts/watchdog-knowledge-sync-governance-r2-opus-output.md` | 第 1、2、3、5、6 问推不出（打中），第 4 问主体成立、两处低的边角打中，第 7 问成立；改法 F1 到 F9 |
| 辩方腿（第 8 到 11 问） | `research/prompts/watchdog-knowledge-sync-governance-r2-sonnet-output.md` | 第一轮坐实的 9 条理由都站得住，第 4、6、7 条改法没盖全；按线索记的 2 条够了；不采的 4 条里两条理由够不着；两批修补少做 1 处、做偏 3 处；主 agent 整行换登记表对不上 `CLAUDE.md` 那一行的字面 |

核查命令（原样）：

- `sha256sum -c research/prompts/watchdog-knowledge-sync-governance-r2-snapshot/judged-sha256.txt`：七份全 OK，腿跑着的时候没有文件被改，开工树就是主工作区此刻那一棵，没另倒推。
- `python3 scripts/agents/cite-check.py research/prompts/watchdog-knowledge-sync-governance-r2-sonnet-output.md research/prompts/watchdog-knowledge-sync-governance-r2-opus-output.md --root /home/fy5090/code/singlefs --background research/prompts/_watchdog-knowledge-sync-governance-r2-background.md`：「✓ 核了 65 处引文、30 处裸行号，对上 65 处，没判 30 处（其中裸行号没有引文可比 30 处；逐处列在上面）」。判决采的条目都对着下面两趟探针的读数核。

判决引的读数由主 agent 复跑：

- 辩方腿探针：任务 20425，退 0，`DEFENSE name=done emitted=10`，`DEFENSE name=watchdog` 的 sha256 是 `5b6030270788…`，与开工快照里 `scripts/agents/agent-watch.py` 那一行相同。
- 攻方腿探针：任务 20429，退 0，`PROBE name=done emitted=89 setup_failures=0`、`name=rerun_done probe_exit=0 probe_lines=89`，`PROBE name=watchdog` 的 sha256 同上。
- 主 agent 现查：`git rev-parse --local-env-vars` 打 15 个名字；`scripts/agents/agent-watch.py:611` 的 `GIT_ENVIRONMENT_DROPPED` 列 8 个。

## 二、判决

坐实 12 条：

1. **提醒间隔最长约两倍 `--max-minutes`。** 在跑的看门狗到期不退，只在退出时判满。依据：任务 20429 `PROBE name=exit-chain-worst arm=as-is max_gap_minutes=124.9 max_minutes=60`；`alerts=one-alert-at-59.9 restart_delay=5.0 reminders=[124.9, …]`。
2. **查不成时两头都错。** 起来时就查不成，不落盘、上次提醒的时刻每只重置，告警比 `--max-minutes` 密时一次都不出；之后查不成，上次提醒的时刻不推进，每次退出都再打。依据：任务 20429 `PROBE name=unreadable-from-start alerts=alerts-every-30 horizon_minutes=180 sections_printed=0`、`PROBE name=digest-timeout-chain kind=later alerts=alerts-every-10 horizon_minutes=180 sections_printed=18`；任务 20425 `DEFENSE name=persistent-failure-repeats … printed_lines_per_alert_exit=[5, 5, 5] … reminded_at_unchanged=True`。
3. **两只看门狗读同一份落盘都打，后一只报的分钟数是错的。** 依据：任务 20429 `PROBE name=two-watchers … first_printed=True second_printed=True second_claims_minutes_since_last_reminder=120.5 actual_minutes_since_last_reminder=0.5`；任务 20425 `DEFENSE name=duplicate-reminders both_loaded_same_state=True first_printed=8 second_printed=8`。
4. **落盘在打印之前。** 打印前被杀，这一次的变动就丢了。依据：任务 20429 `PROBE name=replaced-before-print named_in_unprinted_lines=[[".claude/kb/k.md", "工作区改了没提交"]] … named_by_next_watchdog=[]`。
5. **落盘时被杀留下的临时文件永远不删。** 依据：任务 20429 `PROBE name=killed-inside-replace … leftover_temporary_files=1 … leftover_after_next_watchdog=1`。
6. **落盘的时刻被改成 NaN、无穷或将来，就再也不提醒。** 依据：任务 20429 `PROBE name=hand-edited-reminded-at value=NaN … printed_after_ten_hours=False`，`value=Infinity`、`value=year-2100` 同。
7. **写盘与算摘要有逃出的异常。** 范围里一条目标不是合法 UTF-8 的符号链接，看门狗抛出异常、留下临时文件。依据：任务 20425 `DEFENSE name=non-utf8-symlink-target outcome='抛出 UnicodeEncodeError' state_directory_entries=['.agent-watch-knowledge-sync.json.1xn9vwk5.tmp']`。辩方腿另点名两处读环境变量转成数的地方（`scripts/agents/agent-watch.py:1560`、`:1646`）没人接；那两个变量只给自证用，按线索记，随第 7 条一起在外层接住。
8. **同一路径出两条时合成一条，计数错。** `git rm --cached` 之后同一路径既是已暂存的删除、又是未跟踪。依据：任务 20429 `PROBE name=duplicate-path raw_entries=["D  .claude/rules/r.md", "?? .claude/rules/r.md"] … counts_staged_unstaged_untracked=[0, 0, 1]`。
9. **路径里带换行时输出被拆成两行，能造出像「→」出路的一行。** 依据：任务 20429 `PROBE name=newline-path key_is_exact=True section_rows=9 printed_lines=10 arrow_like_lines=2`。同一类：文件名不是合法 UTF-8 的改了不点名（`PROBE name=non-utf8-path … named=[]`）。
10. **不继承的 git 仓内变量只盖了一半。** 主 agent 现查 `git rev-parse --local-env-vars` 15 个，`GIT_ENVIRONMENT_DROPPED` 8 个，`GIT_CONFIG`、`GIT_CONFIG_PARAMETERS`、`GIT_CONFIG_COUNT`、`GIT_IMPLICIT_WORK_TREE`、`GIT_GRAFT_FILE`、`GIT_NO_REPLACE_OBJECTS`、`GIT_REPLACE_REF_BASE`、`GIT_SHALLOW_FILE` 没盖（`GIT_NAMESPACE` 不在清单里，照留）。
11. **起来时干净、之后只改执行位的文件仍点名。** 第一轮判决第 6 条自己的红样本是这一形，修补只盖了起来时已改着的那一形。依据：任务 20425 `DEFENSE name=chmod-clean-at-baseline baseline_status=[] … named=[('.claude/kb/k.md', '工作区改了没提交')]`，对照 `chmod-dirty-staged-at-baseline … named=[]`。
12. **文档与行为对不上三处。** `--processes-only` 告警退 3 的输出没有「→ 接着盯」那一行，「→ 同步：」与技能段却叫主 agent 照它做；文件头「几只看门狗同时盯一个会话时谁先满谁打」与第 3 条的读数不符；文件头「读不出就重新记一份并落盘」在起来时查不成时不成立（第 2 条）。依据：攻方腿报告第 6 问，主 agent 对着第 2、3 条读数核。

按线索记、这一轮一并改的 3 条：

13. 「没查成」下面给的手动命令看不见被忽略的条目（任务 20429 `PROBE name=timeout-commands-miss-ignored … any_output_mentions_rule=False`）。
14. 只改忽略规则、内容没变的文件标成「被 git 忽略、内容摘要变了」，标签说的不是发生的事（任务 20429 `PROBE name=untracked-to-ignored named=[[".claude/kb/notes.md", "被 git 忽略、内容摘要变了"]]`）。
15. 自证「给人抄的命令照原样跑得起来」那一格只在不带特殊字符的仓路径上跑，引法写坏了照绿。任务 20429 `PROBE name=special-root-commands commands=6 exits=[0, 0, 0, 0, 0, 0] injected_marker_seen=False` 说明此刻的引法是对的，缺的是自证格。

不采的 6 条：

- 辩方腿「`status.renames` 这类配置还能改读数」：任务 20429 `PROBE name=porcelain-config config=status-renames-copies … same_named=True same_counts=True`，十种配置里点名与计数都相同，推翻。
- 攻方腿「只在索引里改了、工作区原样的不点名」（`PROBE name=index-only now_code="MM" named=[]`）与「改了又改回去的不点名」（`PROBE name=changed-and-back named=[]`）：读到的文件内容与上次一样，不用重读，判这是对的行为。
- 攻方腿「指到目录的符号链接改了指向、权限 000 的文件改了内容不点名」（`PROBE name=links-in-untracked-directory directory_link_named=[]`、`PROBE name=unreadable-file … named=[]`）：本仓的知识类文件里没有这两形，不改、不记欠账。
- 攻方腿「被忽略的大文件、很多文件拖慢」：任务 20429 `PROBE name=ignored-many files=50000 … baseline_seconds=6.40`、`PROBE name=ignored-sparse bytes=8589934592 baseline_seconds=15.57`，都在 30 秒时限之内，超了走「没查成」；不改。
- 攻方腿「自证里还剩四处固定时限，只会误红」：不会把错的判成对，不改、不记欠账。
- 辩方腿「判决没拿『只把版本号文件算进范围』比过」：只看版本号点不出是哪一份规则变了，按内容摘要逐份比是更细的那一个；判决站得住。

对第一轮判决的更正：

- 第一轮「核不动的」那一句写「上限 20 时最多 70 行」，任务 20425 `DEFENSE name=line-bound limit=20 lines=75 bound_by_reading=75` 量到 75 行。
- 辩方腿第 10 问说第一轮不采的「多做 6 项」里，范围第八项与内容摘要两项在派给工具员的记录里没有写；这两项是看门狗读得到项目规则变化所必需的，判不算多做，写进「交用户的」告知。
- 主 agent 用 python 整行换 `.claude/gate.d/break-switch-replay-files.tsv` 那一行，对不上 `CLAUDE.md`「项目本地事实」表 `scripts/kb/replace-once.py` 那一行「几个会话共写一批文件时只许这样改，不许整份重写」。那一次断言了恰好一行、内容现读核过，判流程偏离、内容无误；之后共写的登记表一律经 `scripts/kb/replace-once.py` 改。

第二轮有打中的，坐实 12 条。用户 2026-11-18 定「没有致命的bug的话 不需要第三轮了」：这一题在第二轮收，不开第三轮；下面「三、改法」各行的修补都没被攻过，按零轮看待，验证只到换上后自证与弄坏开关逐条复跑。

## 三、改法

| 改法 | 修哪一条 | 谁提的 | 量过 / 零轮 |
|---|---|---|---|
| 定时回报的截止时刻取「起来时刻 + M」与「落盘的上次提醒 + M」里早的那个，已满就在第一次整查之后当场退 4 | 第 1 条 | 攻方腿 F1 | 量过：任务 20429 同一批时刻表两臂，`exit-chain-worst arm=as-is max_gap_minutes=124.9`、`arm=exit-when-due max_gap_minutes=64.9` |
| 查不成也落盘：起来时查不成落一份只有上次提醒时刻的；带了「没查成」这一节也把上次提醒的时刻记成此刻、快照不换 | 第 2 条 | 攻方腿 F2 | 零轮：任务 20429 `unreadable-from-start`、`digest-timeout-chain kind=later` 与任务 20425 `persistent-failure-repeats` 当红样本 |
| 退出判满之前重读落盘那一份，上次提醒的时刻取文件与内存里晚的那个；分钟数照它算 | 第 3 条 | 攻方腿 F3 | 零轮：`two-watchers` 当红样本 |
| 先打印这一节、`flush`，再落盘 | 第 4 条 | 攻方腿 F4 | 零轮：`replaced-before-print` 当红样本 |
| 起来时删掉会话目录里本文件名前缀的、超过一小时的临时文件 | 第 5 条 | 主 agent | 零轮：`killed-inside-replace` 当红样本 |
| 读回的时刻不是有限数、或晚于此刻，当「读不出」 | 第 6 条 | 攻方腿 F5 | 零轮：`hand-edited-reminded-at` 五形当红样本 |
| 记基线与带这一节两处外层接住全部异常，写成「没查成：<异常名与原因>」，退出码不变；摘要里的链接目标与路径按 `surrogateescape` 存、写盘不抛 | 第 7 条 | 辩方腿 | 零轮：`non-utf8-symlink-target` 当红样本 |
| 同一路径出两条时两码都留，计数照原始条目数 | 第 8 条 | 攻方腿 F7 | 零轮：`duplicate-path` 当红样本 |
| 输出里的路径过一遍转义（换行、控制字符写成可见的转义）；`run_git` 按 `surrogateescape` 读、算摘要按原字节找文件 | 第 9 条 | 攻方腿 F7 | 零轮：`newline-path`、`non-utf8-path` 当红样本 |
| 不继承的变量取 `git rev-parse --local-env-vars` 的全部名字（起来时现取一次，取不到用写死的 15 个） | 第 10 条 | 辩方腿 | 零轮：自证格设 `GIT_CONFIG_PARAMETERS` 指到别的配置 |
| 读状态时带 `-c core.fileMode=false`，只改执行位的一律不算变 | 第 11 条 | 主 agent | 零轮：`chmod-clean-at-baseline` 当红样本 |
| `--processes-only` 告警退 3 也打「→ 接着盯」那一行；文件头删「谁先满谁打」、改成第 3 条修完的行为，「读不出就重新记」照第 2 条修完的行为写 | 第 12 条 | 攻方腿 F8 | 不涉及判定 |
| 「没查成」下面再给一条列被忽略条目的命令 | 第 13 条 | 攻方腿 F6 | 零轮 |
| 被忽略条目的标签按发生的事分：内容变了、新进忽略、移出忽略 | 第 14 条 | 主 agent | 零轮：`untracked-to-ignored`、`ignored-to-untracked` 当红样本 |
| 自证那一格的小仓放进带 `'`、`$`、反引号与空格的目录，配一条把引法改坏的开关 | 第 15 条 | 攻方腿 F9 | 零轮 |

这一批都是治理文件，没有要写进产品代码的零轮改法。每条零轮改法的样本先证红再换上。

## 交用户的

| 事 | 状态 |
|---|---|
| 范围第八项（上游规则）与按内容摘要比，派给工具员的记录里没写、是工具员加的 | 告知：判不算多做，没有这两项看门狗点不出规则变化 |
| 定时回报改成离上次提醒满一小时就退 4（第 1 条改法） | 告知：看门狗退出得更勤，主对话被叫醒的次数会多一些，每次照提示再起一只 |

## 这一轮被判过的文件

- research/prompts/_watchdog-knowledge-sync-governance-r2-body.md：判了它「实现今天的样子」各句与代码对不对得上，两条腿照它的十一问作答
- research/prompts/watchdog-knowledge-sync-governance-r1-main-verification.md：辩方腿复核了它坐实、按线索记与不采的各条；更正了行数上界（75 行）与范围第八项的归属，记了主 agent 整行换登记表那一次的流程偏离
- scripts/agents/agent-watch.py：判了落盘、按时带这一节、被忽略条目、porcelain 读法、摘要没变不点名、给人抄的命令、文件头与开关表；坐实第 1 到 12 条，第 13 到 15 条按线索一并改
- scripts/agents/watch.conf：判了 `repository-report-lines` 几行注释与行为一致，成立
- .claude/skills/dispatch/SKILL.md：判了「知识同步提醒」那一段；坐实第 12 条里「→ 接着盯」在 `--processes-only` 告警退出时没有
- .claude/hooks/selftest-registry.tsv：判了 `agent-watch.py` 那一行与自证起的进程一致，成立
- .claude/gate.d/break-switch-replay-files.tsv：判了 `agent-watch.py` 那一行的条数、四族与复跑读数成立；整行换的那一次流程偏离

## 回看决策

不涉及决策：这一轮判的是看门狗脚本与它的配置、技能、登记表，没有引到任何一条决策分项。
