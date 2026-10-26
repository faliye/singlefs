# 第十二批任务 44：回扫报告检查、定点写入脚本的会话号判法、hook-swap 收 scripts/

结论：三条都做到，五份脚本加一份共用库已换进主工作区，各自的 `--selftest` 在主工作区里是绿的（换上之后立刻跑，没有红过、没有回滚过）。

## 改过的文件

主工作区（换上用的都是 hook-swap 的脚本规格，同目录临时文件加改名换上新 inode，换上之后在主工作区里跑 `--verify` 的自证）：

| 文件 | 换上之后 sha256 |
|---|---|
| scripts/kb/stale-candidates.py | 1fef251f803c9b42a385ff69e9cc8d7db25aba84670d3f480b0985e9e4e8d31f |
| scripts/kb/replace-once.py | 05b4be7c815e59319f8fec19310351e570a28c1fce24c5f076d1188b5923befc |
| scripts/kb/insert-row.py | 0981873086aaf353e604f318e3a6ef813659461519d415ef4744dc599fc04679 |
| scripts/kb/replace-batch.py | 06adadd17bfae3c8b901661667d5cb8e0afd26beb4726161890bc662c94ae781 |
| scripts/kb/lib_notation_check.py（共用库，已有的，加会话号一支） | ba2105c9dbf3ef92c1c0a5449877be3ff8d1ae231b6446b12ffe9140ec50a0df |
| scripts/gate/hook-swap.py | 67267656d20a0da6db169c3ab7ad7affa6c430c84182fe399ace8c1724c42661 |

换上之前的原件（`copy-originals.py` 留的）在 `originals/` 下，换上之前的 sha256 在 `baseline.sha256`。
没动 `.claude/hooks/write-guard.sh`、`scripts/kb/claim-experiment.sh`。

`git diff --stat -- <这六份>` 原样：

```
 scripts/gate/hook-swap.py        | 256 +++++++++++++++++++++++++++++++++------
 scripts/kb/insert-row.py         |  48 +++++++-
 scripts/kb/lib_notation_check.py | 149 ++++++++++++++++++++---
 scripts/kb/replace-batch.py      |  77 +++++++++++-
 scripts/kb/replace-once.py       |  36 +++++-
 scripts/kb/stale-candidates.py   | 138 ++++++++++++++++++++-
 6 files changed, 629 insertions(+), 75 deletions(-)
```

（stale-candidates.py 在索引里本来就有别的会话暂存的改动，上面是工作区相对索引的差。）

换上的次序与结果（都用 `python3 scripts/gate/hook-swap.py <规格> --scratch <草稿目录> --verify '<自证>'`，规格是整份替换，旧串是换上之前主工作区的全文，所以基线一变就停）：

1. hook-swap.py 自己：它还没有脚本模式，不能拿自己换自己，用草稿目录的 `install-one.sh`（核基线、同目录临时文件加 mv、回读、跑自证、红了按原件 mv 换回）换，自证 26 格绿；之后用新版 hook-swap 的脚本模式把头注补丁再换了一次自己（`--verify 'python3 scripts/gate/hook-swap.py --selftest'`，退 0）。
2. stale-candidates.py：`--verify 'python3 scripts/kb/stale-candidates.py --selftest'` 退 0。
3. lib_notation_check.py：先单换（向后兼容，旧的两个调用脚本接着用），`--verify` 跑 replace-once 与 insert-row 的自证，退 0（顺带证明钟点那一支拆成通用函数之后行为没变）。
4. replace-once.py、insert-row.py、replace-batch.py 一次换三份，`--verify` 跑三份的自证，退 0；头注改一遍又换一次，同样退 0。

## 条目一（④）：`stale-candidates.py --check-report` 核「交实现员」

做法：报告里任何表格的行，只要有一格是 `crates/…:行号` 形态的载体、另有一格以「要改」或「要补」开头，载体又不是 `crates/singlefs-checker/invariants/` 下的 `.md`，就要出现在报告单列的「交实现员」一节里。节从标题含「交实现员」的行起，到下一个同级或更高级标题止；节里要有同一个 `文件:行`（行号后面不再接数字）；几份报告的节合起来认；代码围栏里的行不算。退出码 3（头注已列），拒绝说明点名「报告文件:行号 位置（判定）」并给出路（节怎么写，指到 `.claude/agents/sweep.md`「产出」一节第一条）。没有这样的行时不要求有这一节（sweep.md 要的「0 行」和数行的命令靠主 agent 读报告）。

样本（`--check-report` 命令行真跑，候选表 2 行：`CLAUDE.md:3` 与 `crates/singlefs-checker/invariants/01-sample.md:5`，都判要改）：

| 报告 | 内容 | 退出码 |
|---|---|---|
| base.md | 只有逐行判定表 | 0，「crates/ 下判要改要补的 0 行都在「交实现员」一节里」 |
| red.md | 另表里 `crates/singlefs-harness/src/lib.rs:12` 与 `:40` 判要改、要补，「交实现员」节里只写了 `:12` | 3，「1 行目标在 crates/ 下、判了「要改」或「要补」，不在「交实现员」一节里，例：red.md:11 crates/singlefs-harness/src/lib.rs:40（要补）」 |
| green.md | red.md 加上 `:40` 那一行进节 | 0，「crates/ 下判要改要补的 2 行都在「交实现员」一节里」 |
| red.md 加 `STALE_CANDIDATES_BREAK=handover-unchecked` | 同上 | 0（弄坏开关让红样本变绿，自证据此判红） |

自证里加的格（都在 `selftest()` 的 `check_reports` 那一段）：红六格（节里没写、没有这一节、判要补、`crates/` 下不是不变量清单的 `.md`、位置只写在别的小节、节里的行号只是前缀如 `:120` 对 `:12`）加「拒绝说明点名位置」一格；绿八格（写进节里、判要补写进节里、子标题下、写「0 行」、没有这样的行也没有节、不变量清单的 `.md`、判不相干或事件句不改、围栏里的示例）加「判定行与节分在两份报告」一格。

弄坏开关：`STALE_CANDIDATES_BREAK=handover-unchecked`，已写进头注。实测打开后自证退 5，红的恰好是上面七格：

```
  ✗ 自证失败：crates/ 下判「要改」的行不在「交实现员」一节里的报告没被判红
  ✗ 自证失败：crates/ 下有判「要改」的行而报告没有「交实现员」一节，没被判红
  ✗ 自证失败：crates/ 下判「要补」的行不在「交实现员」一节里的报告没被判红
  ✗ 自证失败：crates/ 下不是不变量清单的 .md，判「要改」不在「交实现员」里的报告没被判红
  ✗ 自证失败：「交实现员」一节之外的小节里写了同一个位置，报告没被判红
  ✗ 自证失败：「交实现员」一节里写的行号只是这一行号的前缀，报告没被判红
  ✗ 自证失败：crates/ 下判「要改」的行没进「交实现员」，拒绝说明没点名位置
  ✗ stale-candidates 自证没过
```

另扫了仓里 68 份含「交实现员」或「## 逐行判定」的历史报告（`research/prompts/*.md`、`records/*.md`），有 crates 要改要补行的 0 份：新判据对历史报告没有误报，也没有真阳性可对。

## 条目二（㉙b）：三份定点写入脚本写之前判新串里的会话号

判据只有一份：`.claude/gate.d/lib-forbidden-notations.py` 的 `session-names` 形态（写闸 `write-guard.sh` 第七道与门禁 doc-text 的 session-names 格读的就是它）。接法：`lib_notation_check.py` 里把钟点那一支拆成通用的 `form_refusal`，`clock_refusal` 与新加的 `session_name_refusal` 是它的两个薄包装；三份脚本在旧串核对通过之后、写之前调 `session_name_refusal`。范围与写闸判 Write、Edit 一致：仓内任何后缀都判（不只 `.md`），排除前缀、`.sh .py .rs` 里带 `# session-names:allow <理由>` 的一行、仓外不判，读不到库按拒绝办。出路取库里登记的 remedy，头一条就是「写成「别的会话」这类不带号的说法」。

退出码（头注已列）：replace-once 8，insert-row 7，replace-batch 5。拒绝时一个字节不写、不留临时文件（insert-row 与 replace-batch 的干跑也判）。

样本（命令行真跑，新串里的会话名在命令里现拼，没进任何仓内文件）：

| 脚本 | 红 | 绿 |
|---|---|---|
| replace-once.py | `records/t.md` 与 `scripts/probe/p.py` 第 2 行换成带会话名的串：退 8，打 `✗ 新串写了会话号（ListAgents 列的会话名），共 1 处，records/t.md 一个字节没写：` 加 `第 2 行：「…」` 加三条 `→ 怎么办`，文件原样 | 换成「别的会话」：退 0；`.py` 里带 `# session-names:allow 自证输入` 的一行：退 0 |
| insert-row.py | `--after '^第一行' --line "会话 <会话名> 在跑"`：退 7，干跑同样退 7，文件原样 | `--line "会话 别的会话 在跑"`：退 0，插在第 2 行 |
| replace-batch.py | 两处的规格里第一处新串带会话名：真跑与 `--dry-run` 都退 5，列出该处，打 `✗ 2 处里 1 处的新串写了会话号，什么都没写`，两个文件都没动 | 同一规格去掉会话名：退 0，写了 2 个文件 |

共用的自证样本在 `lib_notation_check.session_guard_problems`（三份脚本各接一个 `attempt` 与 `entry`）：`.md` 与 `.py` 目标各一份红（含干跑、含入口退出码与「不写文件、不留临时文件」）；绿四种（干净新串写 `.md`、写 `.py`、`.py` 带放行标记、排除前缀下的文件）加仓外的文件；假仓里没有库、目标是 `.py` 时退拒绝码、说「读不到会话号判据」且点名库文件与 `FileNotFoundError`（目标选 `.py` 是为了和钟点那一道的「读不到钟点判据」分开）。

弄坏开关（头注已写）：`REPLACE_ONCE_BREAK=no-session-guard`、`INSERT_ROW_BREAK=no-session-guard`、`REPLACE_BATCH_BREAK=no-session-guard`；库的 `LIB_FORBIDDEN_NOTATIONS_BREAK=session-names`、`LIB_FORBIDDEN_NOTATIONS_BREAK_ALLOW=session-names`、`LIB_FORBIDDEN_NOTATIONS_BREAK_EXCLUSIONS=session-names` 在头注里点了名。主工作区实测打开后：

- 三份的 `no-session-guard`：自证退非 0；replace-once 红「写会话号：」开头的拒绝几条与入口、insert-row 红「✗ 自检失败：写会话号之前判新行没判对」下面的几条、replace-batch 红「✗ 自证：写会话号之前判新串没判对」下面的几条。
- 库的 `session-names`：replace-once 里 `.md` 与 `.py` 拒绝的各格与入口全红（假仓里没有库那一格本来就拒，所以不红）；`_ALLOW` 只红「`.py` 里带放行标记的一行」；`_EXCLUSIONS` 只红「排除前缀」那一格。
- 照 `.claude/gate.d/code-tooling.sh` 的 break-switch-replay 格的读法（开关表库直接取自那份文件，我用草稿目录的 `replay_switches.py` 复刻：逐条设开关、跑 `--selftest`、核头注点名的「…」原文都在带 ✗ 的失败行里）核过 replace-batch.py 与 stale-candidates.py（这两份在 `break-switch-replay-files.tsv` 里）：replace-batch 三条（含新加的两条）、stale-candidates 六条（含新加的一条）全部 `ok`。replace-once 与 insert-row 不在复刻登记表里；复刻时它们各有一条旧开关（replace-once 的 `no-nearest`、insert-row 的 `vague-anchor-hint` 与 `no-clock-guard`）点名的项对不上失败行，是改之前就有的，没碰。

## 条目三（㉟）：hook-swap.py 收 scripts/ 下的文件

选了收（没选「写明不收」）：手工协议的几步——核基线、换、回读、跑自证、红了换回——正是 hook-swap 已有的先验后换、基线核对、整批预演、换上之后验证、反向定点回滚；缺的只是「验证」一步换成什么。

做法：规格里的文件可以是 `.claude/hooks/` 那一层，也可以是 `scripts/` 下任意层已有的普通文件；两处的文件不许混在一份里（草稿树对钩子规格会把 `scripts` 链接到主工作区，混写会穿过链接写到主工作区，所以直接拒）。脚本规格不带 `--hook` 与 `--feeds`，多一个 `--verify '<命令>'`：草稿里只做语法检查（`.sh` 的 `bash -n` 与内嵌 python、`.py` 整份编译，草稿树只放改后的文件）；换上之后在仓根 `bash -c` 跑命令，退 0 才算验过，超时 900 秒算不过，输出写草稿目录；验不过就按反向定点替换回滚（只改回这一次的那几处，别的会话这段时间加的行留着），回滚后只做语法检查。退出码沿用（5 即验证命令红、已回滚）。头注已写：收的范围、`--verify`、草稿树、退出码、两条新开关与已有开关在脚本规格上的红格。

样本（命令行真跑，假仓里 `scripts/tool.py` 退 0）：

| 规格 | 结果 |
|---|---|
| 改一行版本号，`--verify 'python3 scripts/tool.py'` | 退 0，「改名换上新 inode、回读一致…验证命令退 0」 |
| `return 0` 改 `return 1`（语法对），同一个 `--verify` | 退 5，「验证命令退 1」，回滚后文件原样 |
| `return (0`（语法错） | 退 1，「编译不过：'(' was never closed（第 6 行）」，一个字节没写 |
| 假仓里 `research/notes.txt` | 退 2，「不是 …/.claude/hooks 这一层、也不是 …/scripts 下已有的普通文件」 |
| `--dry-run` | 退 0，「--dry-run：停在草稿，主工作区一个字节没写」 |

自证 26 格（原来 13 格加 13 格新的）绿。新加的格：脚本规格过（新 inode、验证命令退 0）、深一层的 `.sh` 不给 `--verify`、`--dry-run`、草稿红两格（`.py` 编译不过、`.sh` 的 `bash -n` 不过）、验证红回滚（别的会话换上之后加的一行留着）、验的时候被改、`scripts/` 之外的文件不收、钩子与脚本混写不收、脚本规格带 `--hook` 不收、带 `--feeds` 不收，钩子规格带 `--verify` 不收、不带 `--hook` 不收。

弄坏开关（头注已写）：新加两条，`HOOK_SWAP_BREAK=script-verify-skipped`（脚本规格换上之后不跑 `--verify` 的命令）打开后红「脚本规格：过」「脚本规格：验证红回滚」两格；`script-paths-unrestricted`（规格里的文件不限在 hooks 与 scripts/ 下）红「脚本规格：scripts/ 之外的文件不收」一格。已有的开关在脚本规格上同样生效，实测：`swap-on-draft-red` 红两格「脚本规格：草稿红不换」，`skip-base-check` 红「脚本规格：验的时候主工作区那份被改了」，`rollback-whole-file` 红「脚本规格：验证红回滚」，`skip-post-swap-check` 红「脚本规格：过」「脚本规格：验证红回滚」。

这一轮换 scripts/ 下六份文件就是用这个脚本模式换的（上面「换上的次序与结果」），自己换自己那一次也退 0。

## 没做什么

- 没跑门禁阶段与各类 lint（break-switch-headers、break-switch-replay、gate-lint、shell-lint、preflight-lint、gate-overlap、hooks-registered 都没跑）；没跑重型测试；没走定义三方；没提交。break-switch-replay 格我用 `replay_switches.py` 手工复刻过两份登记表里的脚本，不等于那一格绿。
- 没改 `.claude/gate.d/code-tooling.sh` 第 826 行 NOT_RUN_HERE 里 `lib_notation_check.py` 的理由（「调它的两个脚本」，现在是三个）：该文件正被别的会话改，那一行又是 break-switch 之外的登记文字；交主 agent 排。
- 没改 `.claude/agents/tooling-writer.md` 第 3 点（「改 `.claude/hooks/` 下的钩子只经 hook-swap.py 换上」）：改定义走三步，不在这一批；脚本模式出来之后可以补一句「scripts/ 下被所有会话当工具用的脚本换上也可用 hook-swap.py 脚本规格」。
- replace-batch.py 原来就没有钟点那一道（replace-once、insert-row 写 `.md` 时判），这次只补会话号，没动钟点；属于另一件，没立项。
- 没有 `--verify` 命令的 heavy 登记检查：`--verify` 里写什么由调用方负责，头注写了「heavy 登记的自证不要写进来」，脚本不核。
- 看到没动的：写闸只判 Write、Edit、MultiEdit，Bash 里的 heredoc、重定向、python 的 `open` 写会话号依旧靠提交时门禁 doc-text 的 session-names 格兜底（头注已写）。

## 要主 agent 跑的命令

没有必须由主 agent 跑的重型命令。提交前建议带上的轻型自证（都已在主工作区绿过）：

- `python3 scripts/kb/replace-once.py --selftest`、`python3 scripts/kb/insert-row.py --selftest`、`python3 scripts/kb/replace-batch.py --selftest`、`python3 scripts/kb/stale-candidates.py --selftest`、`python3 scripts/gate/hook-swap.py --selftest`
- 门禁 code-tooling 的 break-switch-headers、break-switch-replay 两格归提交前整轮。

## 清理

删了草稿目录里的 `repo/`（仓副本，1004K）、`t1/`（88K）、`t2/`（20K）；留下 `originals/`（换上之前的原件，224K）、规格与日志（`swapwork/`，956K）、自己的补丁与自查脚本。
