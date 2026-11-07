#!/usr/bin/env python3
# admission: always 每一次换钩子都要现判这一刻的规格、草稿与主工作区那几份，上一次换过不替这一次作保
# run-condition: command bash python3
"""先验后换：.claude/hooks/ 下的钩子与它读的共用模块、登记表，或 scripts/ 下的脚本，先在草稿里套上定点替换验过，再在主工作区定点换上；
换上之后再验一遍，不对就按反向定点替换回滚（只把这一次换上的几处改回去，不整份覆盖，别的会话这段时间里改的照留）。
两种规格各走一遍，不混在一份里：钩子规格（文件在 .claude/hooks/ 这一层）要 --hook、拿钩子逐条喂；脚本规格（文件在 scripts/ 下，不限层数）不喂钩子，
换上之后改跑 --verify 给的命令（一般是被换脚本的 --selftest）。

用法：
  hook-swap.py <规格.json> --hook <钩子> --scratch <草稿目录> [--feeds <喂法.json>] [--root <仓根>] [--dry-run]
  hook-swap.py <规格.json> --scratch <草稿目录> [--verify <命令>] [--root <仓根>] [--dry-run]   # 脚本规格：文件都在 scripts/ 下，不带 --hook 与 --feeds
  hook-swap.py --feed-only --hook <钩子> [--feeds <喂法.json>] [--root <仓根>]   # 只喂不换：拿此刻的钩子逐条喂，一条一行写退出码，不写任何文件
  hook-swap.py --check-callers [<库.py>…] [--root <仓根>]   # 只核不换：调用方引到的名字库里都有没有（不写库就核 .claude/hooks/ 下全部 .py），不写任何文件
  hook-swap.py --read-outcome [--root <仓根>] [--scratch <草稿目录>]   # 只看结局文件：上一趟换没换完（文件头「结局文件」），不写任何文件
  hook-swap.py --seal-outcome [--root <仓根>] [--scratch <草稿目录>]   # 上一趟几份文件都已是换后的内容、只缺完成行：验过之后补写完成
  hook-swap.py --selftest        # HOOK_SWAP_BREAK=<项> 时必须判红

<规格.json> 同 scripts/kb/replace-batch.py：JSON 数组，每项 {"file": 路径, "old": 旧串, "new": 新串, "count": 可省，默认 1}，
  同一个文件的多处按数组次序依次施加；file 写相对仓根的路径或绝对路径，只收 <仓根>/.claude/hooks/ 这一层里已有的普通文件，
  或 <仓根>/scripts/ 下（任意层）已有的普通文件，两处的文件不能写进同一份规格；旧串、新串都不许空。
<钩子> 是要喂 JSON 的那一份（.claude/hooks/ 下的 .sh），不在规格里也照样做语法检查与喂法。
<喂法.json>：JSON 数组，每项 {"label": 说明, "command": Bash 命令, "exit": 0 或 2, "agent_type": 可省（省了是主 agent）, "cwd": 可省（默认 {root}），
  "neutral": 可省（true 是新旧两版判得一样的，回滚之后拿它们再核一遍）}；也可以写 "input": 一整份钩子 JSON 代替 command、cwd 与 agent_type。
  command、cwd 与 input 里的 {root} 换成这一次判的仓根（草稿里是草稿树，换上之后是主工作区）。
  不给 --feeds 时用内置的那一份（BUILTIN_FEEDS，按钩子相对仓根的路径取；没有内置的就要给）。
<命令>（--verify，只给脚本规格）：换上之后在 <仓根> 里用 bash -c 跑一遍，退 0 才算验过；写被换脚本的 --selftest（例：python3 scripts/kb/replace-once.py --selftest），
  跑得过 VERIFY_TIMEOUT_SECONDS 秒的算不过。命令只在换上之后跑，草稿里（没有仓的样子）只做语法检查；不给 --verify 就只做换上之后的语法检查与回读。
  起编译或真虚机的命令不要写进来（自证都用假驱动、秒级；要跑别的重活的命令经 task-server 交）。
在哪台跑：两台都跑得起，经统一任务服务交、派到哪台由服务定；第二台在共享的活树里、同一个绝对路径上跑，读写的就是本机主工作区那一份，
  规格、草稿目录与 --root 都写落在多机配置 NFS_SHARED_ROOTS 共享根下的绝对路径。几条共用一个 --scratch 的逐条跑：草稿树 <草稿目录>/hook-swap/tree 是同一棵，
  并行时互相删掉、盖掉。
撤回一次换上：把规格倒过来（old 与 new 对调、次序倒排；每次换上都写一份在 <草稿目录>/hook-swap/reverse-<时刻>.json）再跑一遍，
  --feeds 给一份不含这一次新判法的喂法。

结局文件（真换时写，--dry-run 不写）：几份文件是逐份改名换上的，两份之间被杀（内存上限、断电），回滚来不及跑，几份文件就停在半截。
  所以换上之前先把这一趟要换哪几份、各自换前与换后的 sha256、反向规格在哪写进结局文件（同目录临时文件写完、刷盘再改名），全部换完、回读过、换上之后再验过，才写「完成」。
  位置：<仓根>/.git 的 git-dir 下的 singlefs-hook-swap-outcome.tsv（每次真换整份重写，只留最近一趟）；仓根不是 git 工作树的根时放 <草稿目录>/hook-swap/outcome.tsv（下一趟要用同一个 --scratch）。
  格式：制表符分隔，`#` 起头的行是说明，其余每行第一段是行种类：
    开始<TAB>仓根<TAB>进程号<TAB>开始秒数<TAB>反向规格路径<TAB>反向规格 sha256
    文件<TAB>相对仓根的路径<TAB>换前 sha256<TAB>换后 sha256            （开始之后、动手换之前写全，每份一行）
    回读<TAB>相对仓根的路径<TAB>回读算出的 sha256                        （全部换上之后每份一行）
    完成<TAB>份数<TAB>结束秒数                                          （换上、回读、换上之后再验都过了才写；--seal-outcome 补写的末尾另有一段「补记」）
    退回<TAB>份数<TAB>结束秒数                                          （换上之后验不过、或换的时候出错，已按反向定点替换回滚干净之后写，替代完成）
  没有完成也没有退回就是没换完。判法只看内容：对着主工作区现算每份文件的 sha256，等于换后的是「已换上」，等于换前的是「没换上」，都不等的是「被别的改动动过」。
  下一趟（含 --dry-run）起手先读这份文件：没换完、几份文件不是都在换前，就拒绝再换，退 7、一个字节不写，拒绝说明逐份点名状态，写出怎么退回（已换上的几份按反向规格里属于它的几处
  用 Edit 改回）或补完（没换上的几份按规格用 Edit 换上，全到换后；先 --feed-only 与 --check-callers 验过，再 --seal-outcome 补写完成）。几份文件全回到换前（手工退回之后，或被杀在
  第一份之前），按内容判就不再拦，不用另外清；全到换后、没写完成的，要 --seal-outcome 补写。
  --read-outcome 只看不写：退 0 换完了（有完成，每份文件现在都等于回读），1 已退回（有退回，每份都等于换前），2 读不了、没有结局文件或不是结局文件，
  3 没换完（逐份列已换上、没换上、被动过），4 完成或退回之后文件又被改了（逐份点名）。

做法：
  1. 读规格里每份文件此刻的内容、记下字节；在内存里依次施加，每处要恰好命中 count 次；再从改后的文本倒着施加（新旧对调、次序倒排），
     要回得到原文——回不到（新串在改后的文件里不止 count 处）就停：反向定点替换回滚不了的规格不收。
  2. 草稿树 <草稿目录>/hook-swap/tree：.claude/hooks/ 整份拷一份（不带 __pycache__），套上改后的文本；仓根与 .claude/ 下别的项一律符号链接到主工作区
     （钩子按自己所在的位置找仓根、登记表与 preflight，链接过去判的就是主工作区的那几份）。
     脚本规格的草稿树只放改后的那几份文件（按它们在仓里的相对路径），不链接仓里别的东西：脚本规格不在草稿里跑脚本，写进去的路径也就不会穿过链接落到主工作区。
  3. 草稿里验：规格里的文件与 <钩子> 是 .sh 的跑 `bash -n`，再把里面喂给 python 的 heredoc（`python3 … <<'PY'`）逐段编译；.py 整份编译；
     逐条喂 <喂法>（CLAUDE_PROJECT_DIR 设成草稿树、检出记录写草稿目录；草稿树不是 git 工作树，多机配置经 scripts/hosts/lib_repository_copy.py 的
     environment_for_commands_in_the_copy 显式交给 SINGLEFS_MULTI_HOST_CONFIG），退出码要等于写的那一个，
     stderr 里不许有「没判成」「读不到共用模块」（钩子坏了放行时说的话）。有一条不对就停，主工作区一个字节不写。--dry-run 停在这里。
     脚本规格只做语法检查（.sh 的 bash -n 与内嵌 python、.py 整份编译，别的后缀不查），不喂钩子。
     规格里有 .py 库的（钩子规格与脚本规格都算），另核调用方引到的名字：本仓里载入这些库的钩子、共用库、task-server/ 与 scripts/ 下的文件（.py 整份；.sh 里喂给 python 的 heredoc），
     凡是 <载入它的变量>.<名字>（含 from lib import 名字、getattr(库, "名字")）用到被换库的名字，换上之后那一版的模块层里都要有；缺了就当草稿验不过，点名「谁在哪一行引了哪个库的哪个名字」。
     怎么认「载入了哪个库、绑在哪个变量上」写在 CALLER_SCAN_DIRECTORIES 前的注释；认不出去向的写法逐处列成「本次未判」，不当成查过。只核不换用 --check-callers。
  4. 再读主工作区那几份，要与第 1 步记的逐字节相同（这段时间里别的会话改了就停，一个字节不写）；整批先预演一遍（同目录临时文件写完、fsync，
     改名之前停下），全过了逐份改名换上（scripts/kb/lib_atomic_replace.py），回读。
  5. 主工作区里照第 3 步再验一遍（CLAUDE_PROJECT_DIR 设成仓根，调用方引到的名字按盘上此刻的文件再核）；脚本规格再跑 --verify 的命令（输出写 <草稿目录>/hook-swap/verify-<时刻>.log）。
     不对就回滚：逐份在此刻的内容上做反向定点替换（每处要恰好命中 count 次），改名换上；再对回滚后的文件做语法检查、喂标了 neutral 的几条
     （脚本规格回滚后只做语法检查，不再跑 --verify 的命令）。
  草稿树跑完就删；喂法的输出与检出记录留在 <草稿目录>/hook-swap/ 下。

钩子在 bash 这一层坏了时，每一条 Bash 调用都会被它拒（退 2），连改回去的那一条也起不来：这条脚本在同一个进程里回滚，不靠下一条 Bash；
退出码 6 时只剩 Edit / Write 能改，提示行逐处写出要改回的地方，反向规格在草稿目录里。
脚本规格收进来的原因：scripts/ 下被所有会话当工具用的脚本（scripts/kb/replace-once.py 这几份）换到一半或换坏了，别的会话每一次调用都受影响；
原来靠手工协议（核基线、同目录临时文件加 mv、回读、跑自证、红了换回），现在同一套先验后换、基线核对、整批预演、换上之后验证、反向定点回滚都落在这一个进程里。

退出码：0 换上了、换上之后验过（--dry-run：草稿里验过；--check-callers：调用方引到的名字都在）；1 草稿里验不过（一个字节没写；--check-callers：有缺的名字）；2 参数、路径或规格不对（一个字节没写）；
3 草稿验过之后主工作区那几份被改了（一个字节没写）；4 换上时失败（已换上的几份回滚了）；
5 换上之后验不过（脚本规格含 --verify 的命令退非 0 或超时）、已按反向定点替换回滚、回滚后验过；6 回滚不成或回滚后还验不过（看提示行）；
7 上一趟换上没写完成（被杀或回滚不成，几份文件停在半截，见「结局文件」），拒绝再换，一个字节没写（--read-outcome 与 --seal-outcome 的退出码另见「结局文件」）。

弄坏开关（只给自证判红用，生效时往 stderr 打一行；平常跑设了退 2、点名只在自证里认：--selftest 本身与它起的子进程认）。
一条一行「HOOK_SWAP_BREAK=<项>  弄坏哪一处」，下一行起写打开后自证红的项（失败行 `✗ hook-swap 自检：` 之后的句首）：
  HOOK_SWAP_BREAK=swap-on-draft-red  草稿里验不过也换上
      「草稿红不换」「花括号写法上放行的钩子，先验停下」「脚本规格：草稿红不换」
  HOOK_SWAP_BREAK=skip-embedded-python  不编译 .sh 里喂给 python 的 heredoc（喂法也红，但说明里点不出是哪一段）
      「草稿红不换」
  HOOK_SWAP_BREAK=skip-base-check  换上之前不核主工作区那几份在验的时候改没改
      「验的时候主工作区那份被改了」「脚本规格：验的时候主工作区那份被改了」
  HOOK_SWAP_BREAK=skip-post-swap-check  换上之后不再验
      「换上之后红回滚」「脚本规格：过」「脚本规格：验证红回滚」
  HOOK_SWAP_BREAK=rollback-whole-file  回滚时整份写回换上之前的内容（盖掉别的会话这段时间里的改动）
      「换上之后红回滚」「脚本规格：验证红回滚」
  HOOK_SWAP_BREAK=stale-builtin-feed  内置喂法里混进一条脚本搬家之前的旧路径
      「内置喂法「旧路径」里的 scripts/mutate.s?」
  HOOK_SWAP_BREAK=brace-feeds-dropped  内置喂法里不带花括号与参数通配那六条
      「内置喂法带花括号与参数通配的六条」「花括号写法上放行的钩子，先验停下」
  HOOK_SWAP_BREAK=script-verify-skipped  脚本规格换上之后不跑 --verify 的命令（输出里没有「验证命令退 0」）
      「脚本规格：过」「脚本规格：验证红回滚」
  HOOK_SWAP_BREAK=script-paths-unrestricted  规格里的文件不限在 .claude/hooks/ 与 scripts/ 下（仓里已有的普通文件都收）
      「脚本规格：scripts/ 之外的文件不收」
  HOOK_SWAP_BREAK=skip-caller-names  调用方引到的名字缺了也不报（换上的库去掉了调用方还在引的名字，照样验过、换上）
      「引到的名字：库里去掉一个名字，各种载入写法的调用方都被点名，草稿验不过、一个字节不写」
  HOOK_SWAP_BREAK=outcome-start-omitted  换上之前不写开始与文件行，拖到回读之后才一起写（被杀的那一趟什么都没留下）
      「结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上」「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」
  HOOK_SWAP_BREAK=outcome-complete-first  开始行写完马上写完成行（没换完也像换完了）
      「结局文件：换完的一趟依次写开始、文件、回读、完成，--read-outcome 退 0」「结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上」
      「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」「结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换」
  HOOK_SWAP_BREAK=pending-not-checked  下一趟起手不读上一趟的结局文件（半截照换）
      「结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写」「结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换」
  HOOK_SWAP_BREAK=outcome-reader-trusts-file  判每份文件的状态时信结局文件里写的换后，不对着主工作区现算 sha256
      「结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上」「结局文件：完成之后文件又被改，--read-outcome 退 4 并点名」
不认识的值直接拒绝，免得拼错的开关被当成「没开」。
"""
import ast
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from typing import NamedTuple

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path[:0] = [os.path.join(REPOSITORY_ROOT, ".claude", "scripts"), SCRIPT_DIRECTORY]
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "kb"))
from lib_atomic_replace import ReplaceRefused, replace_file_contents_by_rename  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
import lib_repository_copy as repository_copy  # noqa: E402

BREAK_VARIABLE = "HOOK_SWAP_BREAK"
KNOWN_BREAK_MODES = ("swap-on-draft-red", "skip-embedded-python", "skip-base-check", "skip-post-swap-check", "rollback-whole-file", "stale-builtin-feed",
                     "brace-feeds-dropped", "script-verify-skipped", "script-paths-unrestricted", "skip-caller-names",
                     "outcome-start-omitted", "outcome-complete-first", "pending-not-checked", "outcome-reader-trusts-file")
HOOKS_DIRECTORY = os.path.join(".claude", "hooks")
SCRIPTS_DIRECTORY = "scripts"
VERIFY_TIMEOUT_SECONDS = 900
WORK_DIRECTORY_NAME = "hook-swap"
FEED_TIMEOUT_SECONDS = 120
FEED_EXIT_CODES = (0, 2)
# 钩子坏了、照常放行时 stderr 里说的话（project-code-guard.sh 的「没判成」与「读不到共用模块」）
BROKEN_HOOK_MARKERS = ("没判成", "读不到共用模块")
# .sh 里喂给 python 的 heredoc 起头那一行：python / python3 …… <<'PY'（<<- 去掉行首制表符）
EMBEDDED_PYTHON_START = re.compile(r"\bpython3?\b[^\n]*?<<(-?)\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\2")
EXIT_SWAPPED, EXIT_DRAFT_RED, EXIT_BAD_INPUT, EXIT_CHANGED, EXIT_SWAP_FAILED, EXIT_ROLLED_BACK, EXIT_ROLLBACK_FAILED, EXIT_PENDING_OUTCOME = 0, 1, 2, 3, 4, 5, 6, 7
OUTCOME_NAME = "singlefs-hook-swap-outcome.tsv"   # <git-dir> 下的结局文件（文件头「结局文件」）；仓根不是 git 工作树的根时放 <草稿目录>/hook-swap/outcome.tsv
OUTCOME_COMMENTS = (
    "# 上面是这份结局文件的开始行；格式见 scripts/gate/hook-swap.py 文件头「结局文件」，制表符分隔，每行第一段是行种类：",
    "#   开始 仓根 进程号 开始秒数 反向规格路径 反向规格sha256 | 文件 路径 换前sha256 换后sha256 | 回读 路径 回读算出的sha256 | 完成 份数 秒数 | 退回 份数 秒数",
    "# 没有完成也没有退回就是没换完：对着主工作区现算每份文件的 sha256，等于换后的已换上，等于换前的没换上（hook-swap.py --read-outcome）",
)
WRITER = "implementation-writer"
# 花括号与参数里的通配照 bash 展开再判（钩子自己的自证表 project-code-guard.sh「花括号与参数里的通配照 bash 展开再判」那几格，C644）：
# 换上的钩子在这六条上退回放行，换之前的先验照样过，所以内置喂法里要有。路径写现位置 scripts/mutation/；正常写法用仓里真有的路径（stale_feed_paths 要展开得到）
BRACE_AND_PARAMETER_GLOB_FEEDS = [
    {"label": "实现员花括号起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutate.{sh,x} crates/mutations.tsv", "exit": 2},
    {"label": "实现员花括号起 crash-point-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/crash-point-replay.{sh,} {root}", "exit": 2},
    {"label": "实现员参数里的通配点到 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutate.sh crates/mutation?.tsv", "exit": 2},
    {"label": "实现员花括号写在包装上（{bash,-x}）照拒", "agent_type": WRITER, "command": "{bash,-x} scripts/mutation/mutate.sh crates/mutations.tsv", "exit": 2},
    {"label": "实现员合法的花括号 ls 放行", "agent_type": WRITER, "command": "ls scripts/{kb,gate}/*.py", "exit": 0, "neutral": True},
    {"label": "实现员合法的参数通配 cat 放行", "agent_type": WRITER, "command": "cat crates/*.toml", "exit": 0, "neutral": True},
]
# 内置喂法（钩子相对仓根的路径 → 喂法）。neutral 的几条新旧两版判得一样（回滚之后拿它们核钩子没坏）；
# 别的几条钉的是这一版的判法（命令词带通配的照展开判，test-script-align-r2 判决 U1-1），旧版喂了会红
BUILTIN_FEEDS = {
    ".claude/hooks/project-code-guard.sh": [
        {"label": "主 agent 跑 ls -la 放行", "command": "ls -la", "exit": 0, "neutral": True},
        {"label": "实现员跑 git status 放行", "agent_type": WRITER, "command": "git status --short", "exit": 0, "neutral": True},
        {"label": "实现员跑 cargo test --all 照拒", "agent_type": WRITER, "command": "cargo test --all", "exit": 2, "neutral": True},
        {"label": "实现员跑 验证包照拒", "agent_type": WRITER, "command": "bash scripts/hosts/run-with-memory-cap.sh 4G cargo test -p singlefs-verification",
         "exit": 2, "neutral": True},
        {"label": "实现员单独跑自证照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "python3 scripts/gate/admission.py --selftest", "exit": 2, "neutral": True},
        {"label": "实现员 ls 合法的通配放行", "agent_type": WRITER, "command": "ls scripts/*/*.sh", "exit": 0},
        {"label": "实现员通配起 crash-point-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/crash-point-replay.s? {root}", "exit": 2},
        {"label": "实现员通配起 QEMU 那一道照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/qemu-device-streams.s? {root}", "exit": 2},
        {"label": "实现员通配起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutate.s? crates/mutations.tsv", "exit": 2},
        {"label": "实现员通配起 vm-bench.sh --selftest 照拒", "agent_type": WRITER, "command": "bash scripts/devices/vm-bench*.sh --selftest", "exit": 2},
        {"label": "实现员通配起登记成重的自证照拒", "agent_type": WRITER, "command": "bash scripts/devices/vm-bench.s? --selftest", "exit": 2},
        # 本仓的脚本写成通配起，展开之后照样是直接跑项目代码，一律拒
        {"label": "实现员通配起 compile-then-swap 自证照拒", "agent_type": WRITER, "command": "python3 scripts/hosts/compile-then-swap.p? --selftest", "exit": 2},
        {"label": "实现员通配起双机驱动自证照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification-two-host?.sh --selftest", "exit": 2},
        # 带 POSIX 字符类、等价类、整理符号的交给 bash 展开；崩溃放量三份入口是本仓的脚本（test-script-align-r3 攻方 A1、A2）
        {"label": "实现员 POSIX 字符类起 mutate.sh 跑 crates 整表照拒", "agent_type": WRITER, "command": "bash scripts/mutation/mutat[[:alpha:]].sh crates/mutations.tsv", "exit": 2},
        {"label": "实现员等价类起 crash-point-replay 照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/crash-point-replay.s[[=h=]] {root}", "exit": 2},
        {"label": "实现员整理符号起 QEMU 那一道照拒", "agent_type": WRITER, "command": "bash .claude/gate.d/qemu-device-streams.s[[.h.]] {root}", "exit": 2},
        {"label": "实现员合法的 POSIX 字符类 ls 放行", "agent_type": WRITER, "command": "ls scripts/*/[[:alpha:]]*.sh", "exit": 0, "neutral": True},
        {"label": "实现员起崩溃放量全量照拒", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh", "exit": 2},
        {"label": "实现员起崩溃放量分项驱动照拒", "agent_type": WRITER,
         "command": "bash scripts/crash/crash-amplification-item.sh crash_amplification_fixed_script_through_rollback", "exit": 2},
        {"label": "实现员起崩溃放量双机驱动照拒", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification-two-hosts.sh", "exit": 2},
        {"label": "实现员单独跑 crash-amplification.sh --selftest 照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh --selftest",
         "exit": 2, "neutral": True},
        {"label": "实现员列崩溃放量的分项照拒（子 agent 交主 agent 经 task-server 提交）", "agent_type": WRITER, "command": "bash scripts/crash/crash-amplification.sh --list-items", "exit": 2, "neutral": True},
        # 本仓之外的外部程序要经 task-server 起（lib_project_code_runs.py 的 EXTERNAL_PROGRAM_NAMES），只问版本的放行
        {"label": "实现员直接起 herd7 照拒", "agent_type": WRITER, "command": "herd7 x.litmus", "exit": 2},
        {"label": "实现员直接起 qemu-system-x86_64 照拒", "agent_type": WRITER, "command": "qemu-system-x86_64 -nographic", "exit": 2},
        {"label": "主 agent 直接起 herd7 照拒", "command": "herd7 x.litmus", "exit": 2},
        {"label": "实现员只问 herd7 的版本放行", "agent_type": WRITER, "command": "herd7 -version", "exit": 0},
        # 管道里读 JSON、往标准输出打字的 python3 -c，代码里的 'task' 字样不算写了后面执行的 task-server/task；真往它写了再执行的照拒
        {"label": "主 agent 管道读 JSON 的 python3 -c 提到 'task'，后面再起 task-server/task：放行",
         "command": "task-server/task wait 7191 >/dev/null 2>&1; task-server/task status 7191 --json | python3 -c \"import json,sys; d=json.load(sys.stdin); print(d['task'].get('compiles'))\"; "
                    "task-server/task log 7191 --all 2>&1 | head -6", "exit": 0},
        {"label": "主 agent 循环里管道读 JSON 的 python3 -c 提到 'task'，后面再起 task-server/task：放行",
         "command": "for n in 6778 6780; do task-server/task status $n --json | python3 -c \"import sys,json; d=json.load(sys.stdin); print(d['task'].get('compiles'))\"; done; "
                    "task-server/task status 6778 --json | python3 -c \"import sys,json; print(sorted(json.load(sys.stdin)['attempts'][0].keys()))\"", "exit": 0},
        {"label": "主 agent python3 -c 往 task-server/task 写了再执行它：照拒", "command": "python3 -c \"open('task-server/task','w').write('true')\"; task-server/task status 7191",
         "exit": 2, "neutral": True},
        *BRACE_AND_PARAMETER_GLOB_FEEDS,
    ],
}


def builtin_feeds_for(hook_relative, mode):
    """钩子相对仓根的路径 → 内置喂法，没有交 None。brace-feeds-dropped 下去掉花括号与参数通配那六条（只给自证判红用）。"""
    feeds = BUILTIN_FEEDS.get(hook_relative)
    if feeds is not None and mode == "brace-feeds-dropped":
        feeds = [feed for feed in feeds if feed not in BRACE_AND_PARAMETER_GLOB_FEEDS]
    return feeds


def stale_feed_paths(feeds, root):
    """喂法命令里 scripts/、.claude/ 开头的词（带通配的交 bash 展开）在 root 下一个都对不上的：[(label, 词)]。
    脚本搬进分类子目录之后旧路径只会让「该拒」的喂法退 0、「该放」的喂法空放，不报错。"""
    stale = []
    for feed in feeds:
        for word in str(feed.get("command", "")).split():
            if not word.startswith(("scripts/", ".claude/")):
                continue
            # 花括号要先于变量展开才展得开，所以这里 eval；词只收路径里会出现的字符，别的字符（$、;、引号、反引号）不 eval
            if not re.fullmatch(r"[A-Za-z0-9_./*?\[\]{},:=+@-]+", word):
                stale.append((feed.get("label", "?"), word))
                continue
            found = subprocess.run(["bash", "-c", 'shopt -s nullglob; eval "found=($1)"; [ -e "${found[0]:-}" ]', "bash", word], cwd=root, stdin=subprocess.DEVNULL,
                                   capture_output=True)
            if found.returncode != 0:
                stale.append((feed.get("label", "?"), word))
    return stale


class BadInput(Exception):
    """参数、路径或规格不对：str 是两行，第二行以「→」开头写下一步。"""


class Edit(NamedTuple):
    number: int     # 规格里第几处
    relative: str   # 相对仓根的路径
    path: str       # 主工作区里的绝对路径（解过符号链接）
    old: str
    new: str
    count: int


class RehearsalStoppedBeforeRename(Exception):
    """预演：临时文件写完、fsync 完，在改名换上之前停下。"""


def break_mode():
    mode = os.environ.get(BREAK_VARIABLE, "")
    if mode and mode not in KNOWN_BREAK_MODES:
        raise BadInput(f"{BREAK_VARIABLE}={mode} 不认识，一个字节没写\n"
                       f"→ 怎么办：弄坏开关只认 {'、'.join(KNOWN_BREAK_MODES)}；平时不设这个变量")
    if mode:
        print(f"  ! {BREAK_VARIABLE}={mode} 生效：故意走坏的写法，只给自证判红用", file=sys.stderr)
    return mode


def is_inside(path, directory):
    return os.path.realpath(path).startswith(os.path.realpath(directory) + os.sep)


def read_bytes(path):
    with open(path, "rb") as handle:
        return handle.read()


def read_text(path):
    return read_bytes(path).decode("utf-8")


def sha256_of(data):
    return hashlib.sha256(data if isinstance(data, bytes) else data.encode("utf-8")).hexdigest()


def resolve_edits(spec, root, mode=""):
    """规格 → [Edit]；路径不在 .claude/hooks/ 这一层、也不在 scripts/ 下、不是已有的普通文件、两处的文件混在一份里、旧串或新串为空、count 不是正整数，抛 BadInput。
    mode 是弄坏开关（script-paths-unrestricted：scripts/ 的限制放开成整个仓根，只给自证判红用）。"""
    if not isinstance(spec, list) or not spec:
        raise BadInput("规格不是非空的 JSON 数组\n→ 怎么办：写成 [{\"file\": …, \"old\": …, \"new\": …}, …]（同 replace-batch.py）")
    hooks = os.path.realpath(os.path.join(root, HOOKS_DIRECTORY))
    scripts = os.path.realpath(os.path.join(root, SCRIPTS_DIRECTORY if mode != "script-paths-unrestricted" else ""))
    edits = []
    for number, item in enumerate(spec, 1):
        if not isinstance(item, dict) or not all(isinstance(item.get(key), str) for key in ("file", "old", "new")):
            raise BadInput(f"规格第 {number} 处缺 file、old、new 之一，或不是字符串\n→ 怎么办：每处写 {{\"file\", \"old\", \"new\", \"count\"（可省）}}")
        count = item.get("count", 1)
        if not isinstance(count, int) or isinstance(count, bool) or count < 1:
            raise BadInput(f"规格第 {number} 处的 count={count!r} 不是正整数\n→ 怎么办：省掉 count（默认 1），或写这处旧串该命中的次数")
        if not item["old"] or not item["new"]:
            raise BadInput(f"规格第 {number} 处的旧串或新串是空的：空串回滚时找不回原处\n→ 怎么办：两边都带上前后文（删一段就把它前后的一行一起写进旧串与新串）")
        written = item["file"] if os.path.isabs(item["file"]) else os.path.join(root, item["file"])
        path = os.path.realpath(written)
        if (os.path.dirname(path) != hooks and not is_inside(path, scripts)) or not os.path.isfile(path):
            raise BadInput(f"规格第 {number} 处的文件 {item['file']} 不是 {hooks} 这一层、也不是 {scripts} 下已有的普通文件\n"
                           "→ 怎么办：这条脚本只换 .claude/hooks/ 下的钩子、共用模块与登记表，和 scripts/ 下的脚本；别的文件照原来的办法改，新文件先排他建好再换")
        edits.append(Edit(number, os.path.relpath(path, root), path, item["old"], item["new"], count))
    kinds = {is_script_edit(edit, root) for edit in edits}
    if len(kinds) > 1:
        raise BadInput("规格里既有 .claude/hooks/ 下的钩子文件、又有 scripts/ 下的脚本：两种验法不同（钩子拿 JSON 喂，脚本换上之后跑 --verify 的命令），混在一份里换不了\n"
                       "→ 怎么办：拆成两份规格各跑一次；脚本规格先换（它不影响钩子），钩子规格后换")
    return edits


def is_script_edit(edit, root):
    """这一处的文件不在 .claude/hooks/ 这一层：按脚本规格的验法办。"""
    return os.path.dirname(edit.path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY))


def apply_edits(texts, edits, reverse=False):
    """依次施加（reverse 时新旧对调、次序倒排），每处要恰好命中 count 次；交 (新文本表, None) 或 (None, 问题一句)。"""
    texts = dict(texts)
    for edit in (reversed(edits) if reverse else edits):
        if edit.relative not in texts:
            continue
        old, new = (edit.new, edit.old) if reverse else (edit.old, edit.new)
        hits = texts[edit.relative].count(old)
        if hits != edit.count:
            return None, (f"第 {edit.number} 处 {edit.relative}：{'倒着施加时' if reverse else ''}期望命中 {edit.count} 次，实际 {hits}："
                          f"{old[:80]!r}")
        texts[edit.relative] = texts[edit.relative].replace(old, new)
    return texts, None


def embedded_python_segments(text):
    """.sh 里喂给 python 的 heredoc：[(起头那一行的行号, 正文, 有没有收尾行)]。注释行里的不算。"""
    lines, segments, index = text.split("\n"), [], 0
    while index < len(lines):
        match = EMBEDDED_PYTHON_START.search(lines[index])
        if match is None or lines[index].lstrip().startswith("#"):
            index += 1
            continue
        strip_tabs, delimiter = match.group(1) == "-", match.group(3)
        body, cursor = [], index + 1
        while cursor < len(lines) and (lines[cursor].lstrip("\t") if strip_tabs else lines[cursor]) != delimiter:
            body.append(lines[cursor])
            cursor += 1
        segments.append((index + 1, "\n".join(body) + "\n", cursor < len(lines)))
        index = cursor + 1
    return segments


def static_problems(path, label, mode):
    """.sh：bash -n 与内嵌 python 逐段编译；.py：整份编译。交 (问题列表, 编了几段内嵌 python)。"""
    problems, segments_compiled = [], 0
    try:
        text = read_text(path)
    except (OSError, UnicodeDecodeError) as error:
        return [f"{label}：读不了（{error}）"], 0
    if path.endswith(".sh"):
        checked = subprocess.run(["bash", "-n", path], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if checked.returncode != 0:
            problems.append(f"{label}：bash -n 不过（{(checked.stderr.strip().splitlines() or [''])[0][:200]}）")
        if mode != "skip-embedded-python":
            for number, (start, body, closed) in enumerate(embedded_python_segments(text), 1):
                segments_compiled += 1
                if not closed:
                    problems.append(f"{label}：内嵌 python 第 {number} 段（第 {start} 行起）没有收尾行")
                    continue
                try:
                    compile("\n" * start + body, f"{label}（内嵌 python 第 {number} 段）", "exec")
                except (SyntaxError, ValueError) as error:
                    problems.append(f"{label}：内嵌 python 第 {number} 段（第 {start} 行起）编译不过：{getattr(error, 'msg', error)}"
                                    f"（第 {getattr(error, 'lineno', '?')} 行）")
    elif path.endswith(".py"):
        try:
            compile(text, label, "exec")
        except (SyntaxError, ValueError) as error:
            problems.append(f"{label}：编译不过：{getattr(error, 'msg', error)}（第 {getattr(error, 'lineno', '?')} 行）")
    return problems, segments_compiled


def substitute(value, root):
    if isinstance(value, str):
        return value.replace("{root}", root)
    if isinstance(value, list):
        return [substitute(item, root) for item in value]
    if isinstance(value, dict):
        return {key: substitute(item, root) for key, item in value.items()}
    return value


def hook_input_of(feed, root):
    if "input" in feed:
        return substitute(feed["input"], root)
    hook_input = {"tool_name": feed.get("tool_name", "Bash"), "cwd": substitute(feed.get("cwd", "{root}"), root),
                  "tool_input": {"command": substitute(feed["command"], root)}, "session_id": "hook-swap"}
    if feed.get("agent_type"):
        hook_input["agent_type"] = feed["agent_type"]
    return hook_input


def validate_feeds(feeds):
    if not isinstance(feeds, list) or not feeds:
        raise BadInput("喂法不是非空的 JSON 数组\n→ 怎么办：写成 [{\"label\": …, \"command\": …, \"exit\": 0 或 2}, …]")
    for number, feed in enumerate(feeds, 1):
        if (not isinstance(feed, dict) or not isinstance(feed.get("label"), str) or feed.get("exit") not in FEED_EXIT_CODES
                or not (isinstance(feed.get("command"), str) or isinstance(feed.get("input"), dict))):
            raise BadInput(f"喂法第 {number} 条缺 label、exit（0 或 2）、command 或 input\n"
                           "→ 怎么办：每条写 {\"label\": 说明, \"command\": 命令, \"exit\": 0 或 2}，可加 agent_type、cwd、neutral")


def feed_environment(root, detections_path):
    """喂钩子的环境：草稿树（符号链接拼的、不是 git 工作树）里起钩子，多机配置照 lib_repository_copy.py 的 environment_for_commands_in_the_copy
    显式交给 SINGLEFS_MULTI_HOST_CONFIG（取这份脚本所在的主工作区那一份），不靠钩子照树的位置找。"""
    environment = repository_copy.environment_for_commands_in_the_copy(REPOSITORY_ROOT)
    environment.update(CLAUDE_PROJECT_DIR=root, AGENT_HOOK_DETECTIONS=detections_path, PYTHONDONTWRITEBYTECODE="1")
    return environment


def feed_only(hook, root, feeds=None):
    """只喂不换：拿 root 下此刻的钩子逐条喂，一条一行 ✓/✗ 加实际退出码；交 (退出码, 提示行)。检出记录写临时目录、用完就删，仓里一个字节不写。"""
    mode = break_mode()
    root = os.path.realpath(root)
    hook_path = os.path.realpath(hook if os.path.isabs(hook) else os.path.join(root, hook))
    if os.path.dirname(hook_path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY)) or not hook_path.endswith(".sh") or not os.path.isfile(hook_path):
        raise BadInput(f"--hook {hook} 不是 {os.path.join(root, HOOKS_DIRECTORY)} 这一层里已有的 .sh\n→ 怎么办：--hook 写要喂 JSON 的那一份钩子（例如 .claude/hooks/project-code-guard.sh）")
    hook_relative = os.path.relpath(hook_path, root)
    if feeds is None:
        feeds = builtin_feeds_for(hook_relative, mode)
        if feeds is None:
            raise BadInput(f"{hook_relative} 没有内置喂法\n→ 怎么办：--feeds 给一份喂法（放行几条、该拒几条，写法见文件头）")
    validate_feeds(feeds)
    # 起的是 <root>/.claude/hooks/ 这一层里点名的那一份（上面核过）：路径照「钩子目录 + 文件名」拼，函数单元库把这一层的钩子都算进闭包
    hook_file = os.path.join(root, HOOKS_DIRECTORY, os.path.basename(hook_path))
    lines, wrong = [], 0
    with tempfile.TemporaryDirectory(prefix="hook-swap-feed-only-") as scratch:
        environment = feed_environment(root, os.path.join(scratch, "detections.jsonl"))
        for feed in feeds:
            hook_input = hook_input_of(feed, root)
            command = hook_input.get("tool_input", {}).get("command", "") if isinstance(hook_input, dict) else ""
            try:
                done = subprocess.run(["bash", hook_file], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
                                      env=environment, cwd=root, timeout=FEED_TIMEOUT_SECONDS)
                code, stderr = done.returncode, done.stderr
            except subprocess.TimeoutExpired:
                code, stderr = "超时", ""
            marker = next((marker for marker in BROKEN_HOOK_MARKERS if marker in stderr), None)
            good = code == feed["exit"] and marker is None
            wrong += 0 if good else 1
            lines.append(f"  {'✓' if good else '✗'} 退 {code}（应当 {feed['exit']}）{'：stderr 里有「' + marker + '」' if marker else ''} {feed['label']} ← {command}")
    lines.append(f"{'✗' if wrong else '✓'} 只喂不换：{hook_relative} 喂了 {len(feeds)} 条，{wrong} 条不对")  # gate-lint:summary
    if wrong:
        lines.append("→ 怎么办：不对的几条是钩子此刻的判法与喂法对不上；先看是钩子退回了、还是喂法写错了，再决定改哪边（这个模式不改任何文件）")
    return (EXIT_DRAFT_RED if wrong else EXIT_SWAPPED), lines


def feed_problems(hook_path, root, feeds, log_path, detections_path, only_neutral=False):
    """逐条喂 hook_path（<root>/.claude/hooks/ 这一层里的一份）；交 (问题列表, 喂了几条)。"""
    environment = feed_environment(root, detections_path)
    hook_file = os.path.join(root, HOOKS_DIRECTORY, os.path.basename(hook_path))  # 照「钩子目录 + 文件名」拼，函数单元库把这一层的钩子都算进闭包
    problems, fed = [], 0
    with open(log_path, "a", encoding="utf-8") as log:
        for feed in feeds:
            if only_neutral and not feed.get("neutral"):
                continue
            fed += 1
            hook_input = hook_input_of(feed, root)
            try:
                done = subprocess.run(["bash", hook_file], input=json.dumps(hook_input, ensure_ascii=False), capture_output=True, text=True,
                                      env=environment, cwd=root, timeout=FEED_TIMEOUT_SECONDS)
            except subprocess.TimeoutExpired:
                problems.append(f"喂「{feed['label']}」：{FEED_TIMEOUT_SECONDS} 秒没判完")
                log.write(f"{hook_path}\t{feed['label']}\t超时\n")
                continue
            log.write(f"{hook_path}\t{feed['label']}\t退 {done.returncode}（应当 {feed['exit']}）\t{json.dumps(hook_input, ensure_ascii=False)}\n"
                      f"{done.stderr}\n")
            wrong = []
            if done.returncode != feed["exit"]:
                wrong.append(f"退 {done.returncode}（应当 {feed['exit']}）")
            marker = next((marker for marker in BROKEN_HOOK_MARKERS if marker in done.stderr), None)
            if marker:
                wrong.append(f"stderr 里有「{marker}」")
            if wrong:
                first = (done.stderr.strip().splitlines() or [""])[0]
                problems.append(f"喂「{feed['label']}」：{'；'.join(wrong)}；stderr 头一行 {first[:160]!r}")
    return problems, fed


def check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode, only_neutral=False):
    """root 下这几份文件的语法检查与喂法；交 (问题列表, 一句做了什么)。hook_relative 是 None（脚本规格）时只做语法检查，不喂。"""
    problems, segments = [], 0
    checked = set(relatives) | ({hook_relative} if hook_relative else set())
    for relative in sorted(checked):
        found, compiled = static_problems(os.path.join(root, relative), relative, mode)
        problems += found
        segments += compiled
    if hook_relative is None:
        return problems, f"{len(checked)} 份文件语法检查、内嵌 python {segments} 段编译"
    fed_problems, fed = feed_problems(os.path.join(root, hook_relative), root, feeds, log_path, detections_path, only_neutral)
    problems += fed_problems
    return problems, f"{len(checked)} 份文件语法检查、内嵌 python {segments} 段编译、喂了 {fed} 条"


# ---- 调用方引到的名字：换上的库里要有（载入它的变量上 <变量>.<名字> 用到的每个名字） ----
# 调用方 = 本仓里载入这些库的钩子、共用库、task-server/ 与 scripts/ 下的文件（.py 整份，.sh 只读喂给 python 的 heredoc）。
# 认「载入了哪个库、绑在哪个变量上」的写法（别的写法逐处报「本次未判」，不当成过）：
#   载入：importlib.util.spec_from_file_location("<库名>", …<库文件名>…) 再 module_from_spec；名字里带 load / import 的调用（load_module、import_module、__import__、
#         自己包的 load_sibling_module 这类），和本文件里函数体内起了这几样的函数（module("lib_x")），参数里有库名（lib_x）或库文件名（lib_x.py，含 os.path.join 里
#         单拎出来的那一段、同一层函数里变量的赋值）；import lib_x [as y]；from lib_x import …
#   绑定：赋给变量或属性（self.library）、元组两边逐项对上（a, b = module, module.shell_words）、global 声明过的名字、函数返回值、
#         append 进一个列表后按下标取出、作参数交给本文件里定义的函数（形参接着当库）、作参数交给另一份目标库里的函数（lib_x.f(…, module)，那份库里的形参接着当库）、
#         库自己导出的另一份目标库（module.shell_words 载入的是目标库 lib_shell_words）。
CALLER_SCAN_DIRECTORIES = (os.path.join(".claude", "hooks"), "task-server", "scripts", os.path.join(".claude", "scripts"), os.path.join(".claude", "gate.d"))
CALLER_SKIPPED_DIRECTORIES = frozenset({"__pycache__", "target", "node_modules", ".git", "fixtures"})
LOADER_CALLEE = re.compile(r"load|import|spec_from")
HARMLESS_CALLEES = frozenset({"hasattr", "getattr", "isinstance", "id", "repr", "str", "print", "type", "callable", "bool", "len", "dir", "exec_module"})
LOADER_BODY_CALLEES = frozenset({"spec_from_file_location", "module_from_spec", "load_module", "import_module", "__import__"})
CONTAINER_METHODS = frozenset({"append", "add", "insert"})
DYNAMIC_NAME_CALLEES = frozenset({"globals", "vars", "setattr", "exec"})
FLOW_ROUNDS = 30
CROSS_FILE_ROUNDS = 5


def callee_name(function_node):
    if isinstance(function_node, ast.Name):
        return function_node.id
    if isinstance(function_node, ast.Attribute):
        return function_node.attr
    return None


def dotted_path(node):
    parts = []
    while isinstance(node, ast.Attribute):
        parts.append(node.attr)
        node = node.value
    if isinstance(node, ast.Name):
        parts.append(node.id)
        return ".".join(reversed(parts))
    return None


def annotate_parents(tree):
    """每个节点记下父节点（_flow_parent）和外面最近的函数（_flow_scope）；父节点先于子节点处理（ast.walk 是自上而下的）。"""
    tree._flow_scope = None
    for node in ast.walk(tree):
        inside = node if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.Lambda)) else node._flow_scope
        for child in ast.iter_child_nodes(node):
            child._flow_parent = node
            child._flow_scope = inside


def enclosing_scope(node):
    """node 外面最近的函数（FunctionDef、AsyncFunctionDef、Lambda）；在模块层是 None。"""
    return node._flow_scope


def target_names(target, names):
    if isinstance(target, ast.Name):
        names.add(target.id)
    elif isinstance(target, (ast.Tuple, ast.List)):
        for element in target.elts:
            target_names(element, names)
    elif isinstance(target, ast.Starred):
        target_names(target.value, names)


def module_level_names(source):
    """库源码 → (模块层定义的名字集合, 动态定义名字的写法说明列表)；语法错抛 SyntaxError。
    模块层 = 不在 def / class 里面的语句（含 if / try / with / for / while 里的）；函数里 global 声明过的名字也算（运行时才有，但调用方引得到）。"""
    tree = ast.parse(source)
    names, dynamic = set(), []

    def visit(statements):
        for statement in statements:
            if isinstance(statement, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                names.add(statement.name)
                if statement.name == "__getattr__":
                    dynamic.append("定义了模块层的 __getattr__")
            elif isinstance(statement, ast.Import):
                names.update((alias.asname or alias.name.split(".")[0]) for alias in statement.names)
            elif isinstance(statement, ast.ImportFrom):
                for alias in statement.names:
                    if alias.name == "*":
                        dynamic.append("有 from … import *")
                    else:
                        names.add(alias.asname or alias.name)
            elif isinstance(statement, (ast.For, ast.AsyncFor)):
                target_names(statement.target, names)
                visit(statement.body)
                visit(statement.orelse)
            elif isinstance(statement, (ast.While, ast.If)):
                visit(statement.body)
                visit(statement.orelse)
            elif isinstance(statement, (ast.With, ast.AsyncWith)):
                for item in statement.items:
                    if item.optional_vars is not None:
                        target_names(item.optional_vars, names)
                visit(statement.body)
            elif isinstance(statement, ast.Try) or type(statement).__name__ == "TryStar":
                visit(statement.body)
                for handler in statement.handlers:
                    if handler.name:
                        names.add(handler.name)
                    visit(handler.body)
                visit(statement.orelse)
                visit(statement.finalbody)
            elif isinstance(statement, ast.Match):
                for case in statement.cases:
                    visit(case.body)
            else:
                if isinstance(statement, ast.Assign):
                    for target in statement.targets:
                        target_names(target, names)
                elif isinstance(statement, (ast.AnnAssign, ast.AugAssign)):
                    target_names(statement.target, names)
                for inner in ast.walk(statement):
                    if isinstance(inner, ast.Call) and callee_name(inner.func) in DYNAMIC_NAME_CALLEES:
                        dynamic.append(f"模块层调了 {callee_name(inner.func)}()")
                        break
    visit(tree.body)
    for node in ast.walk(tree):
        if isinstance(node, ast.Global):
            names.update(node.names)
    return names, sorted(set(dynamic))


class CallerFlow:
    """一份 python 源码里，载入了目标库的变量怎么流动、用到了库里的哪些名字。
    种类是 (kind, 库名)：kind 是 "module"（已载入的库）或 "spec"（spec_from_file_location 的结果，还没变成库）。
    名字按函数分层（global / nonlocal 照认），属性路径（self.library）不分层；一份源码内不分先后，反复传播到不再多绑为止。"""

    def __init__(self, tree, targets, exports, hints, own_stem):
        self.targets = targets          # {库名: 库文件名}
        self.exports = exports          # {库名: {库里的名字: 它绑着的另一份目标库的库名}}
        self.own_stem = own_stem        # 这份源码若是一份目标库整份，它的库名；别的是 ""
        self.bound = {}                 # 键 → 种类
        self.containers = {}            # 装过库的列表等的键 → 库名
        self.returns = {}               # 函数名 → 种类（返回的是库）
        self.uses, self.unjudged = [], []
        self.hint_records = []          # ((库名, 函数名), 第几个位置参数或 None, 关键字或 None, 种类)：把库交给另一份目标库里的函数
        if not hasattr(tree, "_flow_index"):
            tree._flow_index = self.build_index(tree)   # 只跟这棵树有关，几轮之间共用
        (self.nodes, self.local_names, self.global_names, self.nonlocal_names, self.definitions, self.classes, self.constants,
         self.loader_functions, self.statements, self.imports) = tree._flow_index
        self.bind_imports()
        self.apply_hints(hints)
        if not self.bound and not any(self.load_call_kind(node) for node in self.statements if isinstance(node, ast.Call)):
            return    # 没载入任何目标库，也没有谁把库交进来
        for _ in range(FLOW_ROUNDS):
            before = (len(self.bound), len(self.containers), len(self.returns))
            for node in self.statements:
                self.flow(node)
            if before == (len(self.bound), len(self.containers), len(self.returns)):
                break
        self.collect()

    @classmethod
    def build_index(cls, tree):
        """这棵树里名字归哪个函数、有哪些函数与类、字符串常量赋给了谁、哪些函数里起了载入的调用。"""
        nodes = list(ast.walk(tree))
        local_names, global_names, nonlocal_names, definitions, classes, constants, imports = {}, {}, {}, {}, {}, {}, []
        for node in nodes:
            scope = enclosing_scope(node)
            if isinstance(node, ast.Name) and isinstance(node.ctx, (ast.Store, ast.Del)):
                local_names.setdefault(scope, set()).add(node.id)
            elif isinstance(node, ast.arg):
                local_names.setdefault(scope, set()).add(node.arg)
            elif isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
                local_names.setdefault(scope, set()).add(node.name)
                if isinstance(node, ast.ClassDef):
                    classes.setdefault(node.name, []).append(node)
                else:
                    definitions.setdefault(node.name, []).append(node)
            elif isinstance(node, ast.Global):
                global_names.setdefault(scope, set()).update(node.names)
            elif isinstance(node, ast.Nonlocal):
                nonlocal_names.setdefault(scope, set()).update(node.names)
            elif isinstance(node, (ast.Import, ast.ImportFrom)):
                imports.append(node)
                local_names.setdefault(scope, set()).update(cls.imported_names(node))
            elif isinstance(node, ast.ExceptHandler) and node.name:
                local_names.setdefault(scope, set()).add(node.name)
        owner = cls.__new__(cls)    # 只借 resolve 用：它读这三张表
        owner.local_names, owner.global_names, owner.nonlocal_names = local_names, global_names, nonlocal_names
        for node in nodes:
            if isinstance(node, ast.Assign):
                texts = [inner.value for inner in ast.walk(node.value) if isinstance(inner, ast.Constant) and isinstance(inner.value, str)]
                for target in node.targets:
                    if texts and isinstance(target, ast.Name):
                        constants.setdefault(owner.resolve(target.id, enclosing_scope(node)), set()).update(texts)
        loader_functions = set()    # 函数体里（含里层的函数）起了载入的调用的函数名
        for node in nodes:
            if isinstance(node, ast.Call) and callee_name(node.func) in LOADER_BODY_CALLEES:
                function = enclosing_scope(node)
                while function is not None:
                    if isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
                        loader_functions.add(function.name)
                    function = enclosing_scope(function)
        statements = [node for node in nodes if isinstance(node, (ast.Assign, ast.AnnAssign, ast.NamedExpr, ast.Return, ast.Call))]
        return nodes, local_names, global_names, nonlocal_names, definitions, classes, constants, loader_functions, statements, imports

    @staticmethod
    def imported_names(node):
        if isinstance(node, ast.Import):
            return [alias.asname or alias.name.split(".")[0] for alias in node.names]
        return [alias.asname or alias.name for alias in node.names if alias.name != "*"]

    def resolve(self, identifier, scope):
        current = scope
        while current is not None:
            if identifier in self.global_names.get(current, ()):
                return (None, identifier)
            if identifier not in self.nonlocal_names.get(current, ()) and identifier in self.local_names.get(current, ()):
                return (current, identifier)
            current = enclosing_scope(current)
        return (None, identifier)

    def key_of(self, node):
        if isinstance(node, ast.Name):
            return ("name", self.resolve(node.id, enclosing_scope(node)))
        path = dotted_path(node) if isinstance(node, ast.Attribute) else None
        return ("path", path) if path else None

    def library_named(self, text):
        """字符串常量 → 库名：库名本身（lib_x）、库文件名（lib_x.py）、以 /库文件名 结尾的路径。"""
        for stem, file_name in self.targets.items():
            if text == stem or text == file_name or text.endswith("/" + file_name):
                return stem
        return None

    def strings_in_arguments(self, call):
        found = set()
        scope = enclosing_scope(call)
        for argument in list(call.args) + [keyword.value for keyword in call.keywords]:
            for inner in ast.walk(argument):
                if isinstance(inner, ast.Constant) and isinstance(inner.value, str):
                    found.add(inner.value)
                elif isinstance(inner, ast.Name):
                    found.update(self.constants.get(self.resolve(inner.id, scope), ()))
        return found

    def load_call_kind(self, call):
        name = callee_name(call.func)
        if name is None or name == "module_from_spec":
            return None
        if not ((LOADER_CALLEE.search(name) and (name[0].islower() or name[0] == "_")) or name in self.loader_functions):
            return None
        for text in sorted(self.strings_in_arguments(call)):
            stem = self.library_named(text)
            if stem:
                return ("spec" if "spec_from" in name else "module", stem)
        return None

    def kind_of(self, node):
        if isinstance(node, ast.Call):
            loaded = self.load_call_kind(node)
            if loaded:
                return loaded
            name = callee_name(node.func)
            if name == "module_from_spec" and node.args:
                inner = self.kind_of(node.args[0])
                return ("module", inner[1]) if inner and inner[0] == "spec" else None
            return self.returns.get(name) if name else None
        if isinstance(node, (ast.Name, ast.Attribute)):
            key = self.key_of(node)
            if key in self.bound:
                return self.bound[key]
            if isinstance(node, ast.Attribute):
                inner = self.kind_of(node.value)
                if inner and inner[0] == "module":
                    exported = self.exports.get(inner[1], {}).get(node.attr)
                    if exported:
                        return ("module", exported)
            return None
        if isinstance(node, ast.Subscript):
            stem = self.containers.get(self.key_of(node.value)) if isinstance(node.value, (ast.Name, ast.Attribute)) else None
            return ("module", stem) if stem else None
        if isinstance(node, ast.IfExp):
            return self.kind_of(node.body) or self.kind_of(node.orelse)
        if isinstance(node, ast.BoolOp):
            for value in node.values:
                kind = self.kind_of(value)
                if kind:
                    return kind
            return None
        if isinstance(node, ast.NamedExpr):
            return self.kind_of(node.value)
        return None

    def bind(self, key, kind):
        if key is not None and kind is not None and key not in self.bound:
            self.bound[key] = kind

    def bind_imports(self):
        for node in self.imports:
            scope = enclosing_scope(node)
            if isinstance(node, ast.Import):
                for alias in node.names:
                    if alias.name.rsplit(".", 1)[-1] in self.targets and (alias.asname or "." not in alias.name):
                        self.bind(("name", self.resolve(alias.asname or alias.name, scope)), ("module", alias.name.rsplit(".", 1)[-1]))
            elif isinstance(node, ast.ImportFrom):
                base = (node.module or "").rsplit(".", 1)[-1]
                for alias in node.names:
                    if base not in self.targets and alias.name in self.targets:
                        self.bind(("name", self.resolve(alias.asname or alias.name, scope)), ("module", alias.name))

    def apply_hints(self, hints):
        """别的文件把库交给了这份目标库的模块层函数：对应的形参在这里当库。"""
        if not self.own_stem:
            return
        for name, definitions in self.definitions.items():
            for position, keyword, kind in hints.get((self.own_stem, name), ()):
                for definition in definitions:
                    if not isinstance(getattr(definition, "_flow_parent", None), ast.Module):
                        continue
                    parameters = [argument.arg for argument in definition.args.posonlyargs + definition.args.args]
                    chosen = parameters[position] if position is not None and position < len(parameters) else keyword
                    if chosen in parameters or chosen in {argument.arg for argument in definition.args.kwonlyargs}:
                        self.bind(("name", (definition, chosen)), kind)

    def bind_target(self, target, value):
        if isinstance(target, (ast.Tuple, ast.List)):
            if isinstance(value, (ast.Tuple, ast.List)) and len(value.elts) == len(target.elts) and not any(isinstance(item, ast.Starred) for item in target.elts + value.elts):
                for inner_target, inner_value in zip(target.elts, value.elts):
                    self.bind_target(inner_target, inner_value)
            return
        kind = self.kind_of(value)
        if kind is None:
            return
        if isinstance(target, (ast.Name, ast.Attribute)):
            self.bind(self.key_of(target), kind)
        elif isinstance(target, ast.Subscript) and kind[0] == "module" and isinstance(target.value, (ast.Name, ast.Attribute)):
            container = self.key_of(target.value)
            if container is not None:
                self.containers.setdefault(container, kind[1])

    def flow(self, node):
        if isinstance(node, ast.Assign):
            for target in node.targets:
                self.bind_target(target, node.value)
        elif isinstance(node, ast.AnnAssign) and node.value is not None:
            self.bind_target(node.target, node.value)
        elif isinstance(node, ast.NamedExpr):
            self.bind_target(node.target, node.value)
        elif isinstance(node, ast.Return) and node.value is not None:
            function = enclosing_scope(node)
            kind = self.kind_of(node.value)
            if kind and kind[0] == "module" and isinstance(function, (ast.FunctionDef, ast.AsyncFunctionDef)):
                self.returns.setdefault(function.name, kind)
        elif isinstance(node, ast.Call):
            self.flow_call(node)

    @staticmethod
    def globals_assignment(call):
        """globals().__setitem__("名字", 值) → 名字（写死的字符串）；不是这种写法交 None。"""
        function = call.func
        if (isinstance(function, ast.Attribute) and function.attr == "__setitem__" and len(call.args) == 2 and isinstance(function.value, ast.Call)
                and callee_name(function.value.func) == "globals" and isinstance(call.args[0], ast.Constant) and isinstance(call.args[0].value, str)):
            return call.args[0].value
        return None

    def flow_call(self, call):
        name = callee_name(call.func)
        if name is None:
            return
        assigned = self.globals_assignment(call)
        if assigned is not None:
            kind = self.kind_of(call.args[1])
            if kind and kind[0] == "module":
                self.bind(("name", (None, assigned)), kind)
        if name in CONTAINER_METHODS and isinstance(call.func, ast.Attribute):
            container = self.key_of(call.func.value)
            if container is not None:
                for argument in call.args:
                    kind = self.kind_of(argument)
                    if kind and kind[0] == "module":
                        self.containers.setdefault(container, kind[1])
        for definition in self.definitions.get(name, ()):
            self.flow_arguments(call, definition, skip_first=isinstance(getattr(definition, "_flow_parent", None), ast.ClassDef) and isinstance(call.func, ast.Attribute))
        for class_definition in self.classes.get(name, ()):
            for member in class_definition.body:
                if isinstance(member, (ast.FunctionDef, ast.AsyncFunctionDef)) and member.name == "__init__":
                    self.flow_arguments(call, member, skip_first=True)

    def flow_arguments(self, call, definition, skip_first):
        parameters = [argument.arg for argument in definition.args.posonlyargs + definition.args.args]
        if skip_first:
            parameters = parameters[1:]
        for position, argument in enumerate(call.args):
            if isinstance(argument, ast.Starred):
                break
            if position < len(parameters):
                kind = self.kind_of(argument)
                if kind and kind[0] == "module":
                    self.bind(("name", (definition, parameters[position])), kind)
        known = set(parameters) | {argument.arg for argument in definition.args.kwonlyargs}
        for keyword in call.keywords:
            if keyword.arg in known:
                kind = self.kind_of(keyword.value)
                if kind and kind[0] == "module":
                    self.bind(("name", (definition, keyword.arg)), kind)

    def context_is_known(self, node):
        """载入的库出现在这个位置，接下来怎么用能跟得上（属性、赋值、返回、比较、交给本文件的函数……），跟不上的是未判。"""
        parent = getattr(node, "_flow_parent", None)
        if isinstance(parent, ast.Attribute):
            return parent.value is node
        if isinstance(parent, (ast.Assign, ast.AnnAssign, ast.NamedExpr)):
            return parent.value is node
        if isinstance(parent, (ast.Tuple, ast.List)):
            grandparent = getattr(parent, "_flow_parent", None)
            return isinstance(grandparent, ast.Assign) and grandparent.value is parent and any(isinstance(target, (ast.Tuple, ast.List)) for target in grandparent.targets)
        if isinstance(parent, (ast.Return, ast.Compare, ast.BoolOp, ast.IfExp, ast.Expr, ast.Assert, ast.Delete, ast.FormattedValue)):
            return True
        if isinstance(parent, ast.UnaryOp):
            return isinstance(parent.op, ast.Not)
        if isinstance(parent, (ast.If, ast.While)):
            return parent.test is node
        if isinstance(parent, ast.keyword):
            call = getattr(parent, "_flow_parent", None)
            return isinstance(call, ast.Call) and self.call_takes_library(call)
        if isinstance(parent, ast.Call) and parent.func is not node:
            return self.call_takes_library(parent)
        return False

    def call_takes_library(self, call):
        name = callee_name(call.func)
        if name is None:
            return False
        if name in HARMLESS_CALLEES or name in self.definitions or name in self.classes or self.globals_assignment(call) is not None:
            return True
        if isinstance(call.func, ast.Attribute):
            receiver = self.kind_of(call.func.value)
            if receiver and receiver[0] == "module":
                return True   # 另一份目标库里的函数：形参在那份库里接着当库（hint_records）
            return name in CONTAINER_METHODS and self.key_of(call.func.value) in self.containers
        return False

    def collect(self):
        kinds = {}

        def kind_cached(node):
            if id(node) not in kinds:
                kinds[id(node)] = self.kind_of(node)
            return kinds[id(node)]

        for node in self.nodes:
            if isinstance(node, ast.ImportFrom):
                base = (node.module or "").rsplit(".", 1)[-1]
                if base in self.targets:
                    for alias in node.names:
                        if alias.name == "*":
                            self.unjudged.append((node.lineno, f"from {base} import *：引到哪些名字认不出"))
                        else:
                            self.uses.append((node.lineno, base, alias.name))
            if isinstance(node, ast.Attribute) and isinstance(node.ctx, ast.Load):
                kind = kind_cached(node.value)
                if kind and kind[0] == "module":
                    self.uses.append((node.lineno, kind[1], node.attr))
            elif isinstance(node, ast.Call):
                if (callee_name(node.func) == "getattr" and len(node.args) == 2 and isinstance(node.args[1], ast.Constant) and isinstance(node.args[1].value, str)):
                    kind = kind_cached(node.args[0])
                    if kind and kind[0] == "module":
                        self.uses.append((node.lineno, kind[1], node.args[1].value))
                if isinstance(node.func, ast.Attribute):
                    receiver = kind_cached(node.func.value)
                    if receiver and receiver[0] == "module":
                        for position, argument in enumerate(node.args):
                            kind = kind_cached(argument)
                            if kind and kind[0] == "module":
                                self.hint_records.append(((receiver[1], node.func.attr), position, None, kind))
                        for keyword in node.keywords:
                            kind = kind_cached(keyword.value)
                            if keyword.arg and kind and kind[0] == "module":
                                self.hint_records.append(((receiver[1], node.func.attr), None, keyword.arg, kind))
            if isinstance(node, (ast.Name, ast.Attribute, ast.Call, ast.Subscript, ast.IfExp, ast.BoolOp, ast.NamedExpr)) and not isinstance(getattr(node, "ctx", None), (ast.Store, ast.Del)):
                kind = kind_cached(node)
                if kind and kind[0] == "module" and not self.context_is_known(node):
                    parent = getattr(node, "_flow_parent", None)
                    self.unjudged.append((node.lineno, f"载入的 {kind[1]} 出现在认不出去向的位置（{type(parent).__name__}"
                                                       f"{'，交给 ' + str(callee_name(parent.func)) if isinstance(parent, ast.Call) else ''}），它的用法没跟"))


class CallerUnit(NamedTuple):
    relative: str    # 相对仓根的路径
    offset: int      # 段里第 1 行在文件里是第几行减一（.py 整份是 0；.sh 里的 heredoc 段是起头那一行的行号）
    tree: object
    stem: str        # .py 整份的库名；.sh 里的段是 ""


class CallerNames(NamedTuple):
    missing: list      # 每项一句：谁在哪一行引了哪个库的哪个名字，库里没有
    unjudged: list     # 每项一句：认不出的写法
    references: int    # 查了几处引用
    files: int         # 读了几份调用方（含 .sh 里的 python 段所在的文件）


def caller_source_files(root, overlay):
    """扫描目录下的 .py、.sh 与带 python 的 #! 行的无后缀文件 → [(相对路径, 文本)]；overlay 里有的相对路径读 overlay 那一版。"""
    found = []
    for directory in CALLER_SCAN_DIRECTORIES:
        for current, subdirectories, files in os.walk(os.path.join(root, directory)):
            subdirectories[:] = sorted(name for name in subdirectories if name not in CALLER_SKIPPED_DIRECTORIES)
            for name in sorted(files):
                path = os.path.join(current, name)
                relative = os.path.relpath(path, root)
                if relative in overlay:
                    found.append((relative, overlay[relative]))
                    continue
                if not os.path.isfile(path):
                    continue
                if not name.endswith((".py", ".sh")):
                    if "." in name:
                        continue
                    try:
                        with open(path, "rb") as handle:
                            if b"python" not in handle.readline(200):
                                continue
                    except OSError:
                        continue
                try:
                    found.append((relative, read_text(path)))
                except (OSError, UnicodeDecodeError):
                    continue
    return found


def caller_units(sources, stems):
    """sources 里提到某个库名的 python 源码（.py 整份、.sh 里的 heredoc 段）解析成 [CallerUnit]；解析不了的交说明。"""
    units, unparsed = [], []
    for relative, text in sources:
        stem_of_file = os.path.basename(relative)[:-3] if relative.endswith(".py") else ""
        is_library = stem_of_file in stems    # 库自己的函数可能收到别处交来的库，不管文本里提没提别的库名都要读
        if not is_library and not any(stem in text for stem in stems):
            continue
        if relative.endswith(".sh"):
            pieces = [(start, body) for start, body, closed in embedded_python_segments(text) if closed]
        else:
            pieces = [(0, text)]
        for offset, body in pieces:
            if not is_library and not any(stem in body for stem in stems):
                continue
            try:
                tree = ast.parse(body)
            except (SyntaxError, ValueError) as error:
                unparsed.append(f"{relative}：python 解析不了（{getattr(error, 'msg', error)}，第 {getattr(error, 'lineno', '?')} 行），它引到的名字没核")
                continue
            annotate_parents(tree)
            units.append(CallerUnit(relative, offset, tree, stem_of_file))
    return units, unparsed


def caller_names_report(root, libraries, overlay):
    """libraries：{相对路径: 换上之后的库文本}；overlay：{相对路径: 换上之后的别的文本}（调用方也按这一版读）。
    调用方里 <载入它的变量>.<名字> 的每个名字，在 libraries 对应那份库的模块层里都要有。"""
    unjudged, targets, definitions = [], {}, {}
    for relative, text in libraries.items():
        stem = os.path.basename(relative)[:-3]
        try:
            names, dynamic = module_level_names(text)
        except (SyntaxError, ValueError) as error:
            unjudged.append(f"{relative}：编译不过（{getattr(error, 'msg', error)}），谁引了它的哪些名字没核")
            continue
        targets[stem] = stem + ".py"
        definitions[stem] = (relative, names, dynamic)
    tracked = dict(targets)    # 跟着走的库：换上的库，加上钩子目录里别的 .py（它们的函数收到库、导出别的库时，名字要跟到底；它们自己的名字不核）
    hooks_directory = os.path.join(root, HOOKS_DIRECTORY)
    for name in (sorted(os.listdir(hooks_directory)) if os.path.isdir(hooks_directory) else []):
        if name.endswith(".py"):
            tracked.setdefault(name[:-3], name)
    sources = caller_source_files(root, {**overlay, **libraries})
    units, unparsed = caller_units(sources, list(tracked))
    unjudged += unparsed
    exports, hints, flows = {}, {}, []
    for _ in range(CROSS_FILE_ROUNDS):
        flows = [(unit, CallerFlow(unit.tree, tracked, exports, hints, unit.stem if unit.stem in tracked else "")) for unit in units]
        derived_exports, derived_hints = {}, {}
        for unit, flow in flows:
            if unit.stem in tracked:
                for key, kind in flow.bound.items():
                    if key[0] == "name" and key[1][0] is None and kind[0] == "module" and kind[1] != unit.stem:
                        derived_exports.setdefault(unit.stem, {})[key[1][1]] = kind[1]
            for hint_key, position, keyword, kind in flow.hint_records:
                derived_hints.setdefault(hint_key, set()).add((position, keyword, kind))
        if derived_exports == exports and derived_hints == hints:
            break
        exports, hints = derived_exports, derived_hints
    missing, references = [], 0
    for unit, flow in flows:
        for line, stem, name in flow.uses:
            if stem not in definitions:
                continue
            references += 1
            library_relative, names, dynamic = definitions[stem]
            if name in names or (name.startswith("__") and name.endswith("__")):
                continue
            where = f"{unit.relative}:{unit.offset + line}"
            if dynamic:
                unjudged.append(f"{where} 引了 {stem}.{name}，{library_relative} 里找不到这个名字，但它{'、'.join(dynamic)}，名字可能是运行时才有的")
            else:
                missing.append(f"{where} 引了 {stem}.{name}，可 {library_relative} 里没有这个名字")
        for line, text in flow.unjudged:
            unjudged.append(f"{unit.relative}:{unit.offset + line} {text}")
    return CallerNames(sorted(set(missing)), sorted(set(unjudged)), references, len({unit.relative for unit in units}))


def caller_names_check(root, relatives, overlay, mode):
    """换上的 .py 库（relatives 里以 .py 结尾的）：调用方引到的名字它们里都要有。交 (问题列表, 一句做了什么, 本次未判列表)。
    库读 overlay 里换上之后的文本，overlay 里没有的读 root 里此刻那一份；调用方同样按 overlay 优先读。
    弄坏开关 skip-caller-names：缺名字也不报（只给自证判红用）。"""
    libraries = {relative: overlay[relative] if relative in overlay else read_text(os.path.join(root, relative)) for relative in relatives if relative.endswith(".py")}
    if not libraries:
        return [], "", []
    report = caller_names_report(root, libraries, overlay)
    problems = [] if mode == "skip-caller-names" else [f"调用方引到的名字：{line}" for line in report.missing]
    summary = (f"引到的名字：{len(libraries)} 份库、{report.files} 份调用方文件里的 {report.references} 处引用，缺 {len(report.missing)} 处，"
               f"本次未判 {len(report.unjudged)} 处（认不出的写法不算查过）")
    return problems, summary, report.unjudged


def unjudged_names_lines(unjudged):
    return [f"  ? 本次未判（引到的名字）：{item}" for item in unjudged]


def check_callers(root, libraries):
    """只核不换：此刻的库与调用方，调用方引到的名字库里都有没有；libraries 是库的路径，空的核 .claude/hooks/ 下全部 .py。交 (退出码, 提示行)，不写任何文件。
    缺名字退 1；认不出的写法逐处列出、不影响退出码，也不算查过。"""
    root = os.path.realpath(root)
    if libraries:
        relatives = []
        for library in libraries:
            path = os.path.realpath(library if os.path.isabs(library) else os.path.join(root, library))
            if not path.endswith(".py") or not os.path.isfile(path) or not is_inside(path, root):
                raise BadInput(f"{library} 不是 {root} 下已有的 .py\n→ 怎么办：hook-swap.py --check-callers [<库.py>…] [--root <仓根>]，不写库就核 {HOOKS_DIRECTORY}/ 下全部 .py")
            relatives.append(os.path.relpath(path, root))
    else:
        hooks = os.path.join(root, HOOKS_DIRECTORY)
        relatives = [os.path.join(HOOKS_DIRECTORY, name) for name in sorted(os.listdir(hooks)) if name.endswith(".py")] if os.path.isdir(hooks) else []
    problems, summary, unjudged = caller_names_check(root, relatives, {}, "")
    lines = [f"  ✗ {problem}" for problem in problems]  # gate-lint:detail
    lines += unjudged_names_lines(unjudged)
    if not summary:
        return EXIT_BAD_INPUT, ["✗ 没有可核的库：只核 .py", f"→ 怎么办：库路径写 {HOOKS_DIRECTORY}/ 下的 .py，或不写（核 {HOOKS_DIRECTORY}/ 下全部 .py）"]
    lines.append(f"{'✗' if problems else '✓'} 只核不换：{summary}")  # gate-lint:summary
    if problems:
        lines.append("→ 怎么办：缺的名字补回库里，或改引它的那几处；换钩子之前先用规格把库和调用方一起改好")
    return (EXIT_DRAFT_RED if problems else EXIT_SWAPPED), lines


def build_scripts_draft_tree(tree, planned):
    """脚本规格的草稿树：只放改后的那几份（按它们在仓里的相对路径），不链接仓里别的东西，写进去的路径不会穿过链接落到主工作区。"""
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    for relative, text in planned.items():
        destination = os.path.join(tree, relative)
        os.makedirs(os.path.dirname(destination), exist_ok=True)
        with open(destination, "w", encoding="utf-8") as handle:
            handle.write(text)


def verify_problem(command, root, log_path):
    """换上之后在仓根跑 --verify 的命令：退 0 返回 None，否则返回一句问题；输出写 log_path。"""
    environment = dict(os.environ)
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    try:
        done = subprocess.run(["bash", "-c", command], cwd=root, stdin=subprocess.DEVNULL, capture_output=True, text=True, env=environment,
                              timeout=VERIFY_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        with open(log_path, "a", encoding="utf-8") as log:
            log.write(f"{command}\n超时 {VERIFY_TIMEOUT_SECONDS} 秒\n")
        return f"验证命令 {VERIFY_TIMEOUT_SECONDS} 秒没跑完：{command[:160]}"
    with open(log_path, "a", encoding="utf-8") as log:
        log.write(f"{command}\n退 {done.returncode}\n{done.stdout}\n{done.stderr}\n")
    if done.returncode == 0:
        return None
    tail = [line for line in (done.stdout + done.stderr).splitlines() if line.strip()][-3:]
    return f"验证命令退 {done.returncode}：{command[:160]}；输出末几行 {tail}"


def build_draft_tree(root, tree, planned):
    """草稿树：.claude/hooks/ 整份拷一份并套上改后的文本，仓根与 .claude/ 下别的项符号链接到主工作区。"""
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    os.makedirs(os.path.join(tree, ".claude"))
    for entry in sorted(os.listdir(root)):
        if entry not in (".claude", ".git"):
            os.symlink(os.path.join(root, entry), os.path.join(tree, entry))
    for entry in sorted(os.listdir(os.path.join(root, ".claude"))):
        if entry != "hooks":
            os.symlink(os.path.join(root, ".claude", entry), os.path.join(tree, ".claude", entry))
    shutil.copytree(os.path.join(root, HOOKS_DIRECTORY), os.path.join(tree, HOOKS_DIRECTORY), symlinks=True,
                    ignore=shutil.ignore_patterns("__pycache__"))
    for relative, text in planned.items():
        with open(os.path.join(tree, relative), "w", encoding="utf-8") as handle:
            handle.write(text)


def rehearse(path, text):
    """走一遍整条换法、停在改名之前：会被拒的在这里抛 ReplaceRefused，原文件一个字节没动。"""
    def stop_before_rename():
        raise RehearsalStoppedBeforeRename()
    try:
        replace_file_contents_by_rename(path, text, check_before_rename=stop_before_rename)
    except RehearsalStoppedBeforeRename:
        return


def manual_rollback_lines(relative, edits):
    """回滚不成时逐处写出要改回的地方（Bash 被坏钩子拒时只剩 Edit / Write）。"""
    lines = [f"    {relative} 要改回的几处（Edit：旧串写下面的「现在」，新串写「改回」）："]
    for edit in reversed([edit for edit in edits if edit.relative == relative]):
        lines.append(f"      第 {edit.number} 处 现在 {edit.new[:120]!r} → 改回 {edit.old[:120]!r}（命中 {edit.count} 次）")
    return lines


def rollback(swapped, edits, base_texts, mode):
    """逐份在此刻的内容上做反向定点替换、改名换上、回读；交 (问题行列表, 回滚了的相对路径)。"""
    problem_lines, rolled = [], []
    for relative, path in reversed(swapped):
        try:
            current = read_text(path)
        except (OSError, UnicodeDecodeError) as error:
            problem_lines += [f"  - {relative} 读不了（{error}），没回滚", *manual_rollback_lines(relative, edits)]
            continue
        if mode == "rollback-whole-file":
            restored = base_texts[relative]
        else:
            restored_texts, problem = apply_edits({relative: current}, [edit for edit in edits if edit.relative == relative], reverse=True)
            if problem:
                problem_lines += [f"  - {relative} 反向定点替换对不上（{problem}）：换上之后有人改了这几处，没回滚",
                                  *manual_rollback_lines(relative, edits)]
                continue
            restored = restored_texts[relative]
        try:
            replace_file_contents_by_rename(path, restored)
        except ReplaceRefused as error:
            problem_lines += [f"  - {relative} 写回时失败（{str(error).splitlines()[0]}），没回滚", *manual_rollback_lines(relative, edits)]
            continue
        if read_text(path) != restored:
            problem_lines += [f"  - {relative} 回滚后回读对不上：有别的进程同时在写", *manual_rollback_lines(relative, edits)]
            continue
        rolled.append(relative)
    return problem_lines, rolled


class OutcomeUnreadable(Exception):
    """结局文件读不了或不是结局文件。"""


class Outcome(NamedTuple):
    path: str
    root: str
    process: str
    reverse_spec: str
    files: list      # [(相对仓根的路径, 换前 sha256, 换后 sha256)]
    readback: dict   # 相对仓根的路径 → 回读算出的 sha256
    ending: str      # "完成" 或 "退回"；都没有是 ""


def git_work_tree_directory(root):
    """root 自己就是 git 工作树的根时交它的 git-dir 绝对路径；不是（含在别的仓的子目录里、没有 git）交 None。"""
    try:
        top = subprocess.run(["git", "-C", root, "rev-parse", "--show-toplevel"], capture_output=True, text=True)
        done = subprocess.run(["git", "-C", root, "rev-parse", "--absolute-git-dir"], capture_output=True, text=True)
    except OSError:
        return None
    if top.returncode != 0 or done.returncode != 0 or os.path.realpath(top.stdout.strip()) != os.path.realpath(root):
        return None
    return done.stdout.strip() or None


def outcome_path(root, scratch):
    """结局文件的位置：root 是 git 工作树的根就放它的 git-dir 下（下一趟不管用哪个草稿目录都看得见），否则放 <草稿目录>/hook-swap/outcome.tsv。"""
    directory = git_work_tree_directory(root)
    if directory is not None:
        return os.path.join(directory, OUTCOME_NAME)
    if scratch is None:
        raise BadInput(f"{root} 不是 git 工作树的根，结局文件在草稿目录里，要 --scratch 点名\n→ 怎么办：加 --scratch <上一趟换用的草稿目录>")
    return os.path.join(os.path.abspath(scratch), WORK_DIRECTORY_NAME, "outcome.tsv")


def write_outcome_atomically(path, text):
    """同目录临时文件写完、刷盘，再改名换上，再刷目录：读的人看到旧的全文或新的全文，没有半截。"""
    directory = os.path.dirname(path)
    os.makedirs(directory, exist_ok=True)
    temporary = f"{path}.{os.getpid()}.tmp"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write(text)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)
    descriptor = os.open(directory, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def outcome_text(rows):
    return "".join("\t".join(str(field) for field in row) + "\n" for row in rows)


def append_outcome(path, rows):
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(outcome_text(rows))
        handle.flush()
        os.fsync(handle.fileno())


def write_outcome_start(path, root, reverse_spec_path, relatives, base_bytes, planned, mode):
    """结局文件整份重写：开始行、说明、每份要换的文件一行（换前与换后的 sha256）。弄坏开关 outcome-complete-first 下紧接着写上完成行。"""
    start = ("开始", root, os.getpid(), int(time.time()), reverse_spec_path, sha256_of(read_bytes(reverse_spec_path)))
    files = [("文件", relative, sha256_of(base_bytes[relative]), sha256_of(planned[relative])) for relative in relatives]
    if mode == "outcome-complete-first":
        files.append(("完成", len(relatives), int(time.time())))
    write_outcome_atomically(path, outcome_text([start]) + "".join(f"{line}\n" for line in OUTCOME_COMMENTS) + outcome_text(files))


def parse_outcome(path):
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except (OSError, UnicodeError) as error:
        raise OutcomeUnreadable(f"读不了结局文件 {path}（{error}）") from error
    rows = [line.split("\t") for line in text.splitlines() if line.strip() and not line.startswith("#")]
    starts = [row for row in rows if row[0] == "开始" and len(row) >= 6]
    if not starts:
        raise OutcomeUnreadable(f"{path} 不是结局文件：没有「开始」行")
    files = [(row[1], row[2], row[3]) for row in rows if row[0] == "文件" and len(row) >= 4]
    readback = {row[1]: row[2] for row in rows if row[0] == "回读" and len(row) >= 3}
    ending = next((row[0] for row in reversed(rows) if row[0] in ("完成", "退回")), "")
    return Outcome(path, starts[-1][1], starts[-1][2], starts[-1][4], files, readback, ending)


def current_digest(root, relative):
    try:
        return sha256_of(read_bytes(os.path.join(root, relative)))
    except OSError:
        return "-"


STATE_LANDED, STATE_NOT_LANDED, STATE_TOUCHED = "已换上", "没换上", "被别的改动动过"


def file_states(outcome, root, mode):
    """每份文件现在的状态，按内容判 → [(相对路径, 换前, 换后, 现在, 状态)]。弄坏开关 outcome-reader-trusts-file 下信文件里写的换后，不现算。"""
    rows = []
    for relative, before, after in outcome.files:
        now = after if mode == "outcome-reader-trusts-file" else current_digest(root, relative)
        rows.append((relative, before, after, now, STATE_LANDED if now == after else STATE_NOT_LANDED if now == before else STATE_TOUCHED))
    return rows


def state_lines(states):
    return [f"    {state}：{relative}（换前 {before[:12]}，换后 {after[:12]}，现在 {now[:12]}）" for relative, before, after, now, state in states]


def recovery_lines(outcome, states):
    """没换完时的出路：已换上的几份按反向规格退回，或没换上的几份按规格补完（逐处列出 Edit 的旧串与新串）。"""
    try:
        with open(outcome.reverse_spec, encoding="utf-8") as handle:
            items = [item for item in json.load(handle) if isinstance(item, dict)]
        problem = None
    except (OSError, ValueError) as error:
        items, problem = [], error
    lines = [f"→ 怎么办：二选一，改完按内容判，几份文件全回到换前、或全到换后并 --seal-outcome 补写完成之后就不再拦；反向规格 {outcome.reverse_spec}"]
    if problem is not None:
        lines.append(f"    反向规格读不了（{problem}）：对着 git diff 手工改")
    landed = [state[0] for state in states if state[4] == STATE_LANDED]
    not_landed = [state[0] for state in states if state[4] == STATE_NOT_LANDED]
    if landed:
        lines.append(f"  - 退回（已换上的 {len(landed)} 份，按反向规格里属于它的几处用 Edit 改回，旧串写「现在」，新串写「改回」；钩子坏了时 Bash 会被拒，Edit 不经它）：")
        for relative in landed:
            lines += [f"      {relative} 第 {number} 处 现在 {item['old'][:120]!r} → 改回 {item['new'][:120]!r}（命中 {item.get('count', 1)} 次）"
                      for number, item in enumerate((item for item in items if item.get("file") == relative), 1)]
    if not_landed:
        lines.append(f"  - 补完（没换上的 {len(not_landed)} 份，按规格用 Edit 换上，旧串写「现在」，新串写「换后」；换完先 hook-swap.py --feed-only --hook <钩子> 与 --check-callers 验过，再 hook-swap.py --seal-outcome）：")
        for relative in not_landed:
            lines += [f"      {relative} 第 {number} 处 现在 {item['new'][:120]!r} → 换后 {item['old'][:120]!r}（命中 {item.get('count', 1)} 次）"
                      for number, item in enumerate(reversed([item for item in items if item.get("file") == relative]), 1)]
    if any(state[4] == STATE_TOUCHED for state in states):
        lines.append("  - 被别的改动动过的几份：先看 git diff 是谁改的，别人的改动不盖；问主 agent")
    return lines


def pending_refusal(root, outcome_file, mode):
    """起手先读上一趟的结局文件：没有完成也没有退回、几份文件又不是都在换前的，交拒绝说明（几行）；别的交 None。弄坏开关 pending-not-checked 下不读。"""
    if mode == "pending-not-checked" or not os.path.exists(outcome_file):
        return None
    try:
        outcome = parse_outcome(outcome_file)
    except OutcomeUnreadable as error:
        return [f"✗ 上一趟的结局文件读不了：{error}，不知道几份文件停在哪，拒绝再换，一个字节没写",
                "→ 怎么办：看一眼这份文件和 git diff；确认几份钩子与库都是该有的样子之后，删掉这份结局文件再跑"]
    if outcome.ending:
        return None
    states = file_states(outcome, root, mode)
    if all(state[4] == STATE_NOT_LANDED for state in states):
        return None   # 被杀在第一份换上之前（或已手工退回）：一份都没换，没有半截
    return [f"✗ 上一趟换上没有写完成（进程 {outcome.process} 起的，被杀或回滚不成）：结局文件 {outcome_file}，几份文件现在是这样（对着主工作区现算 sha256）：",
            *state_lines(states), *recovery_lines(outcome, states), "一个字节没写"]


def read_outcome(path, root, mode):
    """--read-outcome：只读结局文件、对着主工作区现算每份文件的 sha256，判上一趟换没换完 → (退出码, 行)。"""
    try:
        outcome = parse_outcome(path)
    except OutcomeUnreadable as error:
        return 2, [f"✗ {error}", "→ 怎么办：核对路径与 --root / --scratch；没有这份文件说明还没有真换过，格式见 hook-swap.py 文件头「结局文件」"]
    states = file_states(outcome, root, mode)
    if outcome.ending == "完成":
        changed = [state for state in states if state[3] != outcome.readback.get(state[0], state[2])]
        if not changed:
            return 0, [f"✓ 换完了：{path}（进程 {outcome.process}）有完成行，{len(states)} 份文件现在的 sha256 都等于回读"]
        return 4, [f"✗ 完成过，可是这几份现在的 sha256 不等于回读的换后（之后被改了，或被退回了）：{path}",
                   *(f"    {state[0]}（回读 {outcome.readback.get(state[0], state[2])[:12]}，现在 {state[3][:12]}）" for state in changed)]
    if outcome.ending == "退回":
        changed = [state for state in states if state[3] != state[1]]
        if not changed:
            return 1, [f"已退回：{path}（进程 {outcome.process}）有退回行，{len(states)} 份文件现在的 sha256 都等于换前，这一趟一处没留"]
        return 4, [f"✗ 退回过，可是这几份现在的 sha256 不等于换前（之后被改了，或又换上了）：{path}",
                   *(f"    {state[0]}（换前 {state[1][:12]}，现在 {state[3][:12]}）" for state in changed)]
    return 3, [f"✗ 没换完：{path}（进程 {outcome.process}）有开始行，没有完成也没有退回（被杀或回滚不成）；对着主工作区现算 sha256：",
               *state_lines(states), *recovery_lines(outcome, states)]


def seal_outcome(path, root, mode):
    """--seal-outcome：上一趟没写完成，而几份文件现在都等于换后（被杀在回读或换上之后再验之间）：回读一遍，补写回读与完成。→ (退出码, 行)。"""
    try:
        outcome = parse_outcome(path)
    except OutcomeUnreadable as error:
        return EXIT_BAD_INPUT, [f"✗ {error}", "→ 怎么办：核对路径与 --root / --scratch"]
    if outcome.ending:
        return EXIT_BAD_INPUT, [f"✗ 结局文件 {path} 已经有{outcome.ending}行，没有要补的", "→ 怎么办：--read-outcome 看现在的状态"]
    states = file_states(outcome, root, mode)
    if any(state[4] != STATE_LANDED for state in states):
        return EXIT_PENDING_OUTCOME, ["✗ 还有几份不是换后的内容，不补写完成：", *state_lines(states), *recovery_lines(outcome, states)]
    append_outcome(path, [("回读", state[0], state[3]) for state in states] + [("完成", len(states), int(time.time()), "补记")])
    return EXIT_SWAPPED, [f"✓ 补写了完成行：{path}（{len(states)} 份文件现在都等于换后，回读已记）"]


def swap(spec, hook, scratch, root, feeds=None, dry_run=False, after_draft=None, after_swap=None, verify=None, after_file_swapped=None):
    """交回 (退出码, 提示行)。after_draft、after_swap、after_file_swapped 只给自证用：草稿验完之后、全部换上之后各调一次，每换上一份调一次（模拟别的会话在这段时间里改文件、或被杀在两份之间）。
    脚本规格（文件都在 scripts/ 下）：hook 与 feeds 给 None，verify 是换上之后跑的命令（可省）；钩子规格：hook 必给，verify 给 None。"""
    mode = break_mode()
    root = os.path.realpath(root)
    scratch = os.path.abspath(scratch)
    if os.path.realpath(scratch) == root or is_inside(scratch, root):
        raise BadInput(f"草稿目录 {scratch} 在主工作区里\n→ 怎么办：放进派发提示给的草稿目录（工作根 ~/.local/share/singlefs/work/<轮>/ 这类仓外位置）")
    outcome_file = outcome_path(root, scratch)
    unfinished = pending_refusal(root, outcome_file, mode)   # 上一趟换到一半被杀：几份文件停在半截，不在它上面再换
    if unfinished is not None:
        return EXIT_PENDING_OUTCOME, unfinished
    edits = resolve_edits(spec, root, mode)
    script_spec = is_script_edit(edits[0], root)
    if script_spec:
        if hook is not None or feeds is not None:
            raise BadInput("规格里的文件都在 scripts/ 下，不带 --hook 与 --feeds：脚本没有钩子要喂\n"
                           "→ 怎么办：去掉 --hook / --feeds；换上之后要跑的检查（例：被换脚本的 --selftest）写成 --verify '<命令>'")
        hook_relative = None
    else:
        if verify is not None:
            raise BadInput("--verify 只给 scripts/ 下的脚本规格；钩子规格换上之后拿 JSON 喂钩子验\n→ 怎么办：去掉 --verify，要喂的补进 --feeds")
        if hook is None:
            raise BadInput("规格里是 .claude/hooks/ 下的文件，要 --hook 点名拿哪一份钩子喂\n→ 怎么办：加 --hook <钩子>（例如 .claude/hooks/project-code-guard.sh）")
        hook_path = os.path.realpath(hook if os.path.isabs(hook) else os.path.join(root, hook))
        if os.path.dirname(hook_path) != os.path.realpath(os.path.join(root, HOOKS_DIRECTORY)) or not hook_path.endswith(".sh") or not os.path.isfile(hook_path):
            raise BadInput(f"--hook {hook} 不是 {os.path.join(root, HOOKS_DIRECTORY)} 这一层里已有的 .sh\n→ 怎么办：--hook 写要喂 JSON 的那一份钩子（例如 .claude/hooks/project-code-guard.sh）")
        hook_relative = os.path.relpath(hook_path, root)
        if feeds is None:
            feeds = builtin_feeds_for(hook_relative, mode)
            if feeds is None:
                raise BadInput(f"{hook_relative} 没有内置喂法\n→ 怎么办：--feeds 给一份喂法（放行几条、该拒几条，写法见文件头）")
        validate_feeds(feeds)
    relatives = sorted({edit.relative for edit in edits})
    base_bytes = {relative: read_bytes(os.path.join(root, relative)) for relative in relatives}
    try:
        base_texts = {relative: data.decode("utf-8") for relative, data in base_bytes.items()}
    except UnicodeDecodeError as error:
        raise BadInput(f"规格里有文件不是 UTF-8 文本（{error}）\n→ 怎么办：这条脚本只换文本文件") from error
    planned, problem = apply_edits(base_texts, edits)
    if problem:
        raise BadInput(f"{problem}，一个字节没写\n→ 怎么办：对一下这一处的原文（空白、全半角、同义字），改规格再跑")
    restored, problem = apply_edits(planned, edits, reverse=True)
    if problem or restored != base_texts:
        raise BadInput(f"倒着施加回不到原文（{problem or '倒回来的文本与原文不同'}），这份规格换上之后没法定点回滚，一个字节没写\n"
                       "→ 怎么办：给这几处的新串多带一点前后文，让它在改后的文件里只出现 count 次")
    work = os.path.join(scratch, WORK_DIRECTORY_NAME)
    os.makedirs(work, exist_ok=True)
    stamp = f"{time.strftime('%Y%m%dT%H%M%SZ', time.gmtime())}-{os.getpid()}"
    log_path, detections_path = os.path.join(work, f"feeds-{stamp}.log"), os.path.join(work, f"detections-{stamp}.jsonl")
    reverse_spec_path = os.path.join(work, f"reverse-{stamp}.json")
    verify_log_path = os.path.join(work, f"verify-{stamp}.log")
    with open(reverse_spec_path, "w", encoding="utf-8") as handle:
        json.dump([{"file": edit.relative, "old": edit.new, "new": edit.old, "count": edit.count} for edit in reversed(edits)], handle, ensure_ascii=False, indent=1)
    if script_spec:
        head = (f"规格 {len(edits)} 处、{len(relatives)} 份文件（{'、'.join(relatives)}）；脚本规格，不喂钩子；"
                f"换上之后的验证命令 {verify if verify else '无（只做语法检查与回读）'}")
        tail = f"验证输出 {verify_log_path}；反向规格 {reverse_spec_path}；结局文件 {outcome_file}"
    else:
        head = f"规格 {len(edits)} 处、{len(relatives)} 份文件（{'、'.join(relatives)}）；喂的钩子 {hook_relative}；喂法 {len(feeds)} 条"
        tail = f"喂法输出 {log_path}；反向规格 {reverse_spec_path}；结局文件 {outcome_file}"
    lines = [head,
             *(f"  {relative} 换上之前 sha256 {sha256_of(base_bytes[relative])}，换上之后应当是 {sha256_of(planned[relative])}" for relative in relatives),
             tail]
    tree = os.path.join(work, "tree")
    try:
        if script_spec:
            build_scripts_draft_tree(tree, planned)
        else:
            build_draft_tree(root, tree, planned)
        draft_problems, draft_summary = check_tree(tree, relatives, hook_relative, feeds, log_path, detections_path, mode)
    finally:
        shutil.rmtree(tree, ignore_errors=True)
    names_problems, names_summary, names_unjudged = caller_names_check(root, relatives, planned, mode)
    draft_problems += names_problems
    draft_summary += f"、{names_summary}" if names_summary else ""
    lines += unjudged_names_lines(names_unjudged)
    if draft_problems:
        lines += [f"✗ 草稿里验不过（{draft_summary}），主工作区一个字节没写：", *(f"  - {problem}" for problem in draft_problems),  # gate-lint:summary
                  f"→ 怎么办：照上面几条改规格再跑" + ("" if script_spec else f"；喂法的整段 stderr 在 {log_path}")
                  + ("；缺的名字补回库里，或把引它的调用方连同这份库写进同一份规格（调用方按规格换上之后的文本核）" if names_problems else "")]
        if mode != "swap-on-draft-red":
            return EXIT_DRAFT_RED, lines
    else:
        lines.append(f"✓ 草稿里验过：{draft_summary}")
    if dry_run:
        lines.append("--dry-run：停在草稿，主工作区一个字节没写")
        return (EXIT_DRAFT_RED if draft_problems else EXIT_SWAPPED), lines
    if after_draft is not None:
        after_draft()
    changed = [relative for relative in relatives if read_bytes(os.path.join(root, relative)) != base_bytes[relative]]
    if changed and mode != "skip-base-check":
        lines += [f"✗ 草稿验过之后主工作区这几份被改了：{'、'.join(changed)}，一个字节没写",
                  "→ 怎么办：先看是谁改的（git diff 那几份、问主 agent）；照此刻的原文重写规格再跑，不盖掉别人的改动"]
        return EXIT_CHANGED, lines
    for relative in relatives:
        try:
            rehearse(os.path.join(root, relative), planned[relative])
        except ReplaceRefused as error:
            lines += [f"✗ 预演换上 {relative} 时被拒（{str(error).splitlines()[0]}），一个字节没写",
                      "→ 怎么办：照被拒的原因处理那一份（硬链接、只读、目录权限、空间），再跑"]
            return EXIT_SWAP_FAILED, lines
    # 结局文件：动手换之前先写开始与每份的换前、换后 sha256，全部换完、回读、再验过才写完成；换到一半被杀，下一趟读它就知道停在哪（文件头「结局文件」）
    outcome_started = False

    def record(rows):
        """结局文件追加几行；还没写开始行的先写（弄坏开关 outcome-start-omitted 下开始行拖到第一次追加的时候）。"""
        nonlocal outcome_started
        if not outcome_started:
            outcome_started = True
            write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)
        append_outcome(outcome_file, rows)
    if mode != "outcome-start-omitted":
        outcome_started = True
        write_outcome_start(outcome_file, root, reverse_spec_path, relatives, base_bytes, planned, mode)
    swapped = []
    for relative in relatives:
        path = os.path.join(root, relative)

        def unchanged_since_draft(path=path, relative=relative):
            if mode != "skip-base-check" and read_bytes(path) != base_bytes[relative]:
                raise ReplaceRefused(f"{relative} 在换上前一刻被改了\n→ 怎么办：照此刻的原文重写规格再跑")
        try:
            replace_file_contents_by_rename(path, planned[relative], check_before_rename=unchanged_since_draft)
        except ReplaceRefused as error:
            problem_lines, rolled = rollback(swapped, edits, base_texts, mode)
            if not problem_lines:
                record([("退回", len(swapped), int(time.time()))])
            lines += [f"✗ 换上 {relative} 时失败（{str(error).splitlines()[0]}）；已经换上的 {len(swapped)} 份回滚了 {len(rolled)} 份", *problem_lines,
                      "→ 怎么办：照上面的原因处理那一份再跑；有回滚不成的，照写出的几处用 Edit 改回"]
            return (EXIT_ROLLBACK_FAILED if problem_lines else EXIT_SWAP_FAILED), lines
        swapped.append((relative, path))
        if after_file_swapped is not None:
            after_file_swapped(relative)
    readback = [relative for relative in relatives if read_text(os.path.join(root, relative)) != planned[relative]]
    record([("回读", relative, current_digest(root, relative)) for relative in relatives])
    if after_swap is not None:
        after_swap()
    if mode == "skip-post-swap-check":
        record([("完成", len(swapped), int(time.time()))])
        lines.append(f"✓ 换上了 {len(swapped)} 份（弄坏开关：换上之后没再验）")
        return EXIT_SWAPPED, lines
    live_problems, live_summary = check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode)
    live_problems = [f"{relative} 换上之后回读与草稿里验过的不同" for relative in readback] + live_problems
    live_names_problems, live_names_summary, live_names_unjudged = caller_names_check(root, relatives, {}, mode)
    live_problems += live_names_problems
    live_summary += f"、{live_names_summary}" if live_names_summary else ""
    lines += unjudged_names_lines([item for item in live_names_unjudged if item not in names_unjudged])
    if script_spec and verify and mode != "script-verify-skipped":
        failed = verify_problem(verify, root, verify_log_path)
        if failed:
            live_problems.append(failed)
        else:
            live_summary += f"、验证命令退 0（{verify[:80]}）"
    if not live_problems:
        record([("完成", len(swapped), int(time.time()))])
        lines.append(f"✓ 换上了 {len(swapped)} 份（改名换上新 inode、回读一致），主工作区里再验过：{live_summary}")
        return EXIT_SWAPPED, lines
    lines += [f"✗ 换上之后在主工作区验不过（{live_summary}）：", *(f"  - {problem}" for problem in live_problems)]  # gate-lint:summary
    problem_lines, rolled = rollback(swapped, edits, base_texts, mode)
    if not problem_lines:
        record([("退回", len(swapped), int(time.time()))])
    if problem_lines:
        lines += [f"✗ 回滚只成了 {len(rolled)} 份：", *problem_lines,
                  f"→ 怎么办：照上面写出的几处用 Edit 改回（钩子在 bash 这一层坏了时 Bash 调用会被拒，Edit / Write 不经它）；反向规格 {reverse_spec_path}"]
        return EXIT_ROLLBACK_FAILED, lines
    after_problems, after_summary = check_tree(root, relatives, hook_relative, feeds, log_path, detections_path, mode, only_neutral=True)
    if after_problems:
        lines += [f"✗ 已按反向定点替换回滚 {len(rolled)} 份，回滚之后还验不过（{after_summary}）：", *(f"  - {problem}" for problem in after_problems),  # gate-lint:summary
                  f"→ 怎么办：{'这几份脚本' if script_spec else '钩子'}现在可能是坏的；看 {verify_log_path if script_spec else log_path} 里的输出，用 Edit 修，交主 agent"]
        return EXIT_ROLLBACK_FAILED, lines
    lines.append(f"→ 已按反向定点替换回滚 {len(rolled)} 份（{'、'.join(rolled)}），回滚之后验过（{after_summary}）；"
                 f"照上面几条改规格，必要时 --dry-run 先看草稿，再跑" + ("" if script_spec else f"；喂法的整段 stderr 在 {log_path}"))
    return EXIT_ROLLED_BACK, lines


SELFTEST_GUARD = """#!/usr/bin/env bash
set -uo pipefail
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
python3 /dev/fd/3 "$HOOK_DIR" 3<<'PY'
import importlib.util, json, os, sys
spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(sys.argv[1], "fake_lib.py"))
lib = importlib.util.module_from_spec(spec)
spec.loader.exec_module(lib)
command = json.load(sys.stdin).get("tool_input", {}).get("command", "")
if lib.refused(command):
    print("✗ 被拒：" + command, file=sys.stderr)
    sys.exit(2)
sys.exit(0)
PY
"""
SELFTEST_LIBRARY = ('MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n\n\n'
                    'def refused_v2(command):\n    return MARKER in command or "*" in command\n')
SELFTEST_LIBRARY_WITHOUT_V2 = 'MARKER = "HEAVY"\n\n\ndef refused(command):\n    return MARKER in command\n'
SELFTEST_FEEDS = [{"label": "只读放行", "command": "ls -la", "exit": 0, "neutral": True},
                  {"label": "重的照拒", "command": "run HEAVY", "exit": 2, "neutral": True},
                  {"label": "新判法：通配照拒", "command": "run GLOB*", "exit": 2}]
SELFTEST_GOOD_EDIT = {"file": ".claude/hooks/fake_lib.py", "old": "    return MARKER in command\n",
                      "new": "    return MARKER in command or \"*\" in command  # 新判法\n"}
SELFTEST_OTHER_SESSION_LINE = "# 别的会话这段时间里加的一行\n"
# 脚本规格的样本：scripts/tool.py 退 0 就是验过，scripts/sub/tool.sh 是更深一层的 .sh
SELFTEST_SCRIPT = '#!/usr/bin/env python3\nVERSION = 1\n\n\ndef main():\n    return 0\n\n\nif __name__ == "__main__":\n    raise SystemExit(main())\n'
SELFTEST_SCRIPT_SHELL = "#!/usr/bin/env bash\necho tool\n"
SELFTEST_SCRIPT_VERIFY = "python3 scripts/tool.py"
# 调用方引到的名字的样本（都引 fake_lib.refused_v2）：每份一种载入写法，fake_outer.py 把库交给另一份库 fake_inner.py 里的函数，名字在 fake_inner.py 里引
SELFTEST_CALLER_SAMPLES = {
    "fake_spec_user.py": (
        "import importlib.util\nimport os\n\n\ndef load_library(directory):\n"
        '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        "    library = importlib.util.module_from_spec(spec)\n    spec.loader.exec_module(library)\n    return library\n\n\n"
        "LIBRARY = load_library(os.path.dirname(__file__))\n\n\ndef side_check(command):\n    return LIBRARY.refused_v2(command)\n"),
    "fake-side-guard.sh": (
        '#!/usr/bin/env bash\nHOOK_DIR="$(dirname "$0")"\npython3 /dev/fd/3 "$HOOK_DIR" 3<<\'PY\'\nimport os, sys\nsys.path.insert(0, sys.argv[1])\n'
        "import fake_lib as side_library\nprint(side_library.refused_v2(sys.argv[2]))\nPY\n"),
    "fake_holder.py": (
        "import os\n\n\nclass Holder:\n    def __init__(self, loader, directory):\n"
        '        self.library = loader.load_module("fake_lib", os.path.join(directory, "fake_lib.py"))\n\n'
        "    def check(self, command):\n        return self.library.refused_v2(command)\n"),
    "fake_param.py": (
        "import importlib.util\nimport os\n\n\ndef run(library, command):\n    return library.refused_v2(command)\n\n\ndef main(directory):\n"
        '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        '    module = importlib.util.module_from_spec(spec)\n    return run(module, "x")\n'),
    "fake_container.py": (
        "import importlib.util\nimport os\n\nCACHE = []\n\n\ndef library():\n    if not CACHE:\n"
        '        spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(os.path.dirname(__file__), "fake_lib.py"))\n'
        "        module = importlib.util.module_from_spec(spec)\n        CACHE.append(module)\n    return CACHE[0]\n\n\n"
        "def check(command):\n    return library().refused_v2(command)\n"),
    "fake_global.py": (
        "import importlib.util\nimport os\n\nfake_lib = None\n\n\ndef load(directory):\n    global fake_lib\n"
        '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        "    module = importlib.util.module_from_spec(spec)\n    fake_lib, marker = module, module.MARKER\n    return marker\n\n\n"
        "def use(command):\n    return fake_lib.refused_v2(command)\n"),
    "fake_inner.py": "def helper(library, command):\n    return library.refused_v2(command)\n",
    "fake_outer.py": (
        "import importlib.util\nimport os\n\n\ndef run(directory):\n"
        '    library_spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
        "    library = importlib.util.module_from_spec(library_spec)\n"
        '    inner_spec = importlib.util.spec_from_file_location("fake_inner", os.path.join(directory, "fake_inner.py"))\n'
        '    inner = importlib.util.module_from_spec(inner_spec)\n    return inner.helper(library, "x")\n'),
}
SELFTEST_CALLERS_OF_REFUSED_V2 = tuple(name for name in SELFTEST_CALLER_SAMPLES if name != "fake_outer.py")
# 载入的库交给认不出的调用（register 不在本文件、也不是另一份目标库的函数）：去向跟不上，要列成本次未判
SELFTEST_OPAQUE_CALLER = ("import importlib.util\nimport os\n\nfrom elsewhere import register\n\n\ndef load(directory):\n"
                          '    spec = importlib.util.spec_from_file_location("fake_lib", os.path.join(directory, "fake_lib.py"))\n'
                          "    module = importlib.util.module_from_spec(spec)\n    register(module)\n")


def selftest():
    failures, checked = [], 0
    work = tempfile.mkdtemp(prefix="hook-swap-selftest-")

    def fresh_repository(name):
        root = os.path.join(work, name, "repo")
        os.makedirs(os.path.join(root, HOOKS_DIRECTORY))
        os.makedirs(os.path.join(root, "research"))
        os.makedirs(os.path.join(root, "scripts", "sub"))
        for relative, content in (("research/notes.txt", "记录\n"), ("scripts/tool.py", SELFTEST_SCRIPT), ("scripts/sub/tool.sh", SELFTEST_SCRIPT_SHELL)):
            with open(os.path.join(root, relative), "w", encoding="utf-8") as handle:
                handle.write(content)
        for file_name, text, file_mode in (("fake-guard.sh", SELFTEST_GUARD, 0o755), ("fake_lib.py", SELFTEST_LIBRARY, 0o644)):
            path = os.path.join(root, HOOKS_DIRECTORY, file_name)
            with open(path, "w", encoding="utf-8") as handle:
                handle.write(text)
            os.chmod(path, file_mode)
        return root

    def hooks_file(root, file_name):
        return os.path.join(root, HOOKS_DIRECTORY, file_name)

    def run(label, spec, want_code, want_files, want_text=None, after_draft=None, after_swap=None, want_new_inode=None, verify=None, hook=".claude/hooks/fake-guard.sh"):
        nonlocal checked
        checked += 1
        root = fresh_repository(f"case-{checked}")
        scratch = os.path.join(work, f"case-{checked}", "scratch")
        inode_before = os.stat(hooks_file(root, want_new_inode)).st_ino if want_new_inode else None
        try:
            code, lines = swap(spec, hook, scratch, root, feeds=SELFTEST_FEEDS, verify=verify,
                               after_draft=(lambda: after_draft(root)) if after_draft else None,
                               after_swap=(lambda: after_swap(root)) if after_swap else None)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        for file_name, want in want_files.items():
            got = read_text(hooks_file(root, file_name))
            if got != want:
                problems.append(f"{file_name} 的内容不对（sha256 {sha256_of(got)[:12]}，应当 {sha256_of(want)[:12]}）")
        if want_text is not None and not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        if code not in (EXIT_SWAPPED, EXIT_BAD_INPUT) and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        leftovers = [name for name in os.listdir(os.path.join(root, HOOKS_DIRECTORY)) if name.startswith(".")]
        if leftovers:
            problems.append(f"钩子目录里留下临时文件 {leftovers}")
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if want_new_inode and os.stat(hooks_file(root, want_new_inode)).st_ino == inode_before:
            problems.append(f"{want_new_inode} 换上之后 inode 没变：在原 inode 上写的")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    good_library = SELFTEST_LIBRARY.replace(SELFTEST_GOOD_EDIT["old"], SELFTEST_GOOD_EDIT["new"], 1)
    untouched = {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": SELFTEST_LIBRARY}
    # 过：草稿与换上之后都验过，定点换上（新 inode）
    run("过：草稿里验过、换上、换上之后再验过", [SELFTEST_GOOD_EDIT], EXIT_SWAPPED, {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": good_library},
        want_text="✓ 换上了 1 份", want_new_inode="fake_lib.py")
    # 草稿红不换（开关 swap-on-draft-red 下这三格红）
    run("草稿红不换：钩子 bash 这一层的语法错（尾部多一行 if true; then）",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "\nPY\n", "new": "\nPY\nif true; then\n"}], EXIT_DRAFT_RED, untouched, want_text="bash -n 不过")
    run("草稿红不换：钩子内嵌 python 编译不过，静态检查点名那一段（开关 skip-embedded-python 下红）",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "sys.exit(0)\n", "new": "sys.exit(0\n"}], EXIT_DRAFT_RED, untouched,
        want_text="内嵌 python 第 1 段（第 4 行起）编译不过")
    run("草稿红不换：判法改错，该拒的放行了", [{"file": ".claude/hooks/fake_lib.py", "old": 'MARKER = "HEAVY"', "new": 'MARKER = "HEAVYX"'}],
        EXIT_DRAFT_RED, untouched, want_text="喂「重的照拒」：退 0（应当 2）")

    # 换上之后红回滚：验完草稿之后别的会话删了被调的函数；换上之后它又在同一份钩子里加了一行，回滚只改回这一次的那一处，那一行留着
    # （开关 skip-post-swap-check、rollback-whole-file 下红）
    def other_session_drops_v2(root):
        with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_LIBRARY_WITHOUT_V2)

    def other_session_appends(root):
        with open(hooks_file(root, "fake-guard.sh"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
    run("换上之后红回滚：反向定点替换只改回这一次的那一处，别的会话加的一行留着",
        [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}], EXIT_ROLLED_BACK,
        {"fake-guard.sh": SELFTEST_GUARD + SELFTEST_OTHER_SESSION_LINE, "fake_lib.py": SELFTEST_LIBRARY_WITHOUT_V2},
        want_text="已按反向定点替换回滚 1 份", after_draft=other_session_drops_v2, after_swap=other_session_appends)

    # 验的时候主工作区那份被改了：不换（开关 skip-base-check 下红）
    def other_session_edits_library(root):
        with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE + SELFTEST_LIBRARY)
    run("验的时候主工作区那份被改了：一个字节不写", [SELFTEST_GOOD_EDIT], EXIT_CHANGED,
        {"fake-guard.sh": SELFTEST_GUARD, "fake_lib.py": SELFTEST_OTHER_SESSION_LINE + SELFTEST_LIBRARY}, after_draft=other_session_edits_library)
    # 规格不收的两种
    run("规格里的文件不在 .claude/hooks/ 这一层：不收", [{"file": "research", "old": "a", "new": "b"}], EXIT_BAD_INPUT, untouched)
    run("新串在改后的文件里不止一处（回滚找不回原处）：不收", [{"file": ".claude/hooks/fake_lib.py", "old": '"HEAVY"', "new": "command"}],
        EXIT_BAD_INPUT, untouched)
    # 内嵌 python 的切段：注释行里的不算、<<- 去掉行首制表符、没收尾的报出来
    checked += 1
    segments = embedded_python_segments("# python3 - <<'X'\npython3 - <<-'EOF'\nprint(1)\n\tEOF\npython3 /dev/fd/3 3<<PY\nx = (\n")
    if [(start, closed) for start, _, closed in segments] != [(2, True), (5, False)]:
        failures.append(f"内嵌 python 切段不对：{[(start, closed) for start, _, closed in segments]}（应当 [(2, True), (5, False)]）")
    # 内置喂法里的脚本路径都在：绿是现在的 BUILTIN_FEEDS 一条都不旧；红是搬家之前的旧路径要被点名（开关 stale-builtin-feed 往真表里混一条旧的）
    checked += 1
    stale_sample = [{"label": "旧路径", "command": "bash scripts/mutate.s? crates/mutations.tsv"}, {"label": "带通配的新路径", "command": "bash scripts/mutation/mutate.s? x"}]
    if [label for label, _ in stale_feed_paths(stale_sample, REPOSITORY_ROOT)] != ["旧路径"]:
        failures.append("旧路径的喂法没被点名，或带通配的新路径被误点")
    live_feeds = [feed for feeds in BUILTIN_FEEDS.values() for feed in feeds]
    if break_mode() == "stale-builtin-feed":
        live_feeds = live_feeds + stale_sample[:1]
    for label, word in stale_feed_paths(live_feeds, REPOSITORY_ROOT):
        failures.append(f"内置喂法「{label}」里的 {word} 在 {REPOSITORY_ROOT} 下对不上：脚本搬过家，这条喂法钉不住任何东西")
    # 内置喂法带花括号与参数通配的六条（C644）：钩子自己的自证表里有，换上的钩子在这几条上退回放行时，内置喂法要能先验出来
    # （开关 brace-feeds-dropped 下这一格与下面「花括号写法上放行的钩子，先验停下」红）
    checked += 1
    guard_relative = ".claude/hooks/project-code-guard.sh"
    present = builtin_feeds_for(guard_relative, break_mode()) or []
    missing = [feed["label"] for feed in BRACE_AND_PARAMETER_GLOB_FEEDS if feed not in present]
    exits = sorted((feed["exit"] for feed in BRACE_AND_PARAMETER_GLOB_FEEDS))
    if missing or len(BRACE_AND_PARAMETER_GLOB_FEEDS) != 6 or exits != [0, 0, 2, 2, 2, 2]:
        failures.append(f"内置喂法带花括号与参数通配的六条：缺 {missing}，共 {len(BRACE_AND_PARAMETER_GLOB_FEEDS)} 条，退出码 {exits}（应当 4 条退 2、2 条退 0）")
    # 照内置喂法的表答的假钩子：全对的先验过、换上；只在第一条花括号写法（mutate.{sh,x}）上退 0 的，先验停下并点名那一条
    guard_table = {}
    for feed in BUILTIN_FEEDS[guard_relative]:
        guard_table[feed["command"]] = feed["exit"]
    brace_feed = BRACE_AND_PARAMETER_GLOB_FEEDS[0]
    brace_hook_text = ("#!/usr/bin/env bash\nset -uo pipefail\n# 版本 1\npython3 /dev/fd/3 3<<'PY'\nimport json, os, sys\n"
                       f"table = json.loads({json.dumps(json.dumps(guard_table))})\nforgiven = json.loads({json.dumps(json.dumps([brace_feed['command']]))})\n"
                       "command = json.load(sys.stdin).get('tool_input', {}).get('command', '').replace(os.environ['CLAUDE_PROJECT_DIR'], '{root}')\n"
                       "code = 0 if command in forgiven else table.get(command, 0)\n"
                       "if code:\n    print('✗ 被拒：' + command, file=sys.stderr)\n"
                       "    print('     → 怎么办：自证用的假钩子，只按内置喂法的表答，没有别的出路', file=sys.stderr)\nsys.exit(code)\nPY\n")
    correct_hook_text = brace_hook_text.replace("forgiven = json.loads(" + json.dumps(json.dumps([brace_feed["command"]])) + ")", "forgiven = []")
    for label, text, want_code, want_text in (
            ("内置喂法对全对的钩子：先验过、换上", correct_hook_text, EXIT_SWAPPED, "✓ 换上了 1 份"),
            ("花括号写法上放行的钩子，先验停下（内置喂法点名 mutate.{sh,x} 那一条）", brace_hook_text, EXIT_DRAFT_RED, f"喂「{brace_feed['label']}」：退 0（应当 2）")):
        checked += 1
        root = os.path.join(work, f"guard-case-{checked}", "repo")
        os.makedirs(os.path.join(root, HOOKS_DIRECTORY))
        guard_path = os.path.join(root, guard_relative)
        with open(guard_path, "w", encoding="utf-8") as handle:
            handle.write(text)
        try:
            code, lines = swap([{"file": guard_relative, "old": "# 版本 1\n", "new": "# 版本 2\n"}], guard_relative, os.path.join(work, f"guard-case-{checked}", "scratch"), root)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        if not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        wrong_lines = [line for line in lines if line.lstrip().startswith("- 喂「")]
        if want_code == EXIT_DRAFT_RED and len(wrong_lines) != 1:
            problems.append(f"点名的不对的喂法应当恰好一条，实际 {len(wrong_lines)} 条")
        swapped_text = read_text(guard_path)
        if want_code == EXIT_DRAFT_RED and swapped_text != text:
            problems.append("先验红了钩子还是被改了")
        if want_code == EXIT_SWAPPED and swapped_text != text.replace("# 版本 1\n", "# 版本 2\n"):
            problems.append("全对的钩子没换成新版")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    # 脚本规格：文件在 scripts/ 下，不喂钩子，换上之后跑 --verify 的命令
    def run_scripts(label, spec, want_code, want_files, want_text=None, verify=None, hook=None, feeds=None, dry_run=False, after_draft=None, after_swap=None,
                    want_new_inode=None):
        nonlocal checked
        checked += 1
        root = fresh_repository(f"scripts-case-{checked}")
        scratch = os.path.join(work, f"scripts-case-{checked}", "scratch")
        inode_before = os.stat(os.path.join(root, want_new_inode)).st_ino if want_new_inode else None
        try:
            code, lines = swap(spec, hook, scratch, root, feeds=feeds, dry_run=dry_run, verify=verify,
                               after_draft=(lambda: after_draft(root)) if after_draft else None,
                               after_swap=(lambda: after_swap(root)) if after_swap else None)
        except BadInput as error:
            code, lines = EXIT_BAD_INPUT, str(error).splitlines()
        problems = []
        if code != want_code:
            problems.append(f"退出码 {code}（应当 {want_code}）")
        for relative, want in want_files.items():
            got = read_text(os.path.join(root, relative))
            if got != want:
                problems.append(f"{relative} 的内容不对（sha256 {sha256_of(got)[:12]}，应当 {sha256_of(want)[:12]}）")
        if want_text is not None and not any(want_text in line for line in lines):
            problems.append(f"输出里没有「{want_text}」")
        if code != EXIT_SWAPPED and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        for directory in ("scripts", os.path.join("scripts", "sub")):
            leftovers = [name for name in os.listdir(os.path.join(root, directory)) if name.startswith(".")]
            if leftovers:
                problems.append(f"{directory} 里留下临时文件 {leftovers}")
        if os.path.exists(os.path.join(scratch, WORK_DIRECTORY_NAME, "tree")):
            problems.append("草稿树没删")
        if want_new_inode and os.stat(os.path.join(root, want_new_inode)).st_ino == inode_before:
            problems.append(f"{want_new_inode} 换上之后 inode 没变：在原 inode 上写的")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")

    script_untouched = {"scripts/tool.py": SELFTEST_SCRIPT, "scripts/sub/tool.sh": SELFTEST_SCRIPT_SHELL}
    script_version_edit = {"file": "scripts/tool.py", "old": "VERSION = 1\n", "new": "VERSION = 2\n"}
    script_version_two = SELFTEST_SCRIPT.replace("VERSION = 1\n", "VERSION = 2\n", 1)
    run_scripts("脚本规格：过：草稿里验过、换上、验证命令退 0（新 inode）", [script_version_edit], EXIT_SWAPPED,
                {**script_untouched, "scripts/tool.py": script_version_two}, want_text="验证命令退 0", verify=SELFTEST_SCRIPT_VERIFY, want_new_inode="scripts/tool.py")
    run_scripts("脚本规格：过：深一层的 .sh、不给 --verify 只做语法检查", [{"file": "scripts/sub/tool.sh", "old": "echo tool\n", "new": "echo tool2\n"}], EXIT_SWAPPED,
                {**script_untouched, "scripts/sub/tool.sh": "#!/usr/bin/env bash\necho tool2\n"}, want_text="✓ 换上了 1 份")
    run_scripts("脚本规格：--dry-run 停在草稿，一个字节不写", [script_version_edit], EXIT_SWAPPED, script_untouched, want_text="--dry-run：停在草稿", dry_run=True)
    # 草稿红不换（开关 swap-on-draft-red 下这两格红）
    run_scripts("脚本规格：草稿红不换：.py 编译不过", [{"file": "scripts/tool.py", "old": "return 0\n", "new": "return (0\n"}], EXIT_DRAFT_RED, script_untouched,
                want_text="编译不过", verify=SELFTEST_SCRIPT_VERIFY)
    run_scripts("脚本规格：草稿红不换：深一层的 .sh 的 bash -n 不过", [{"file": "scripts/sub/tool.sh", "old": "echo tool\n", "new": "echo tool\nif true; then\n"}],
                EXIT_DRAFT_RED, script_untouched, want_text="bash -n 不过")

    # 验证红回滚：语法对、验证命令退 1；换上之后别的会话又在同一份脚本里加了一行，回滚只改回这一次的那一处，那一行留着
    # （开关 script-verify-skipped、skip-post-swap-check、rollback-whole-file 下红）
    def other_session_appends_script(root):
        with open(os.path.join(root, "scripts", "tool.py"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
    run_scripts("脚本规格：验证红回滚：验证命令退 1、反向定点替换只改回这一次的那一处，别的会话加的一行留着",
                [{"file": "scripts/tool.py", "old": "    return 0\n", "new": "    return 1\n"}], EXIT_ROLLED_BACK,
                {**script_untouched, "scripts/tool.py": SELFTEST_SCRIPT + SELFTEST_OTHER_SESSION_LINE},
                want_text="验证命令退 1", verify=SELFTEST_SCRIPT_VERIFY, after_swap=other_session_appends_script)

    # 验的时候主工作区那份被改了：不换（开关 skip-base-check 下红）
    def other_session_edits_script(root):
        with open(os.path.join(root, "scripts", "tool.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE + SELFTEST_SCRIPT)
    run_scripts("脚本规格：验的时候主工作区那份被改了：一个字节不写", [script_version_edit], EXIT_CHANGED,
                {**script_untouched, "scripts/tool.py": SELFTEST_OTHER_SESSION_LINE + SELFTEST_SCRIPT}, after_draft=other_session_edits_script,
                verify=SELFTEST_SCRIPT_VERIFY)
    # 规格不收的几种（开关 script-paths-unrestricted 下第一种红）
    run_scripts("脚本规格：scripts/ 之外的文件不收", [{"file": "research/notes.txt", "old": "记录", "new": "笔记"}], EXIT_BAD_INPUT, script_untouched)
    run_scripts("脚本规格：钩子文件与脚本混在一份规格里：不收", [{"file": ".claude/hooks/fake_lib.py", "old": 'MARKER = "HEAVY"', "new": 'MARKER = "HEAVYX"'}, script_version_edit],
                EXIT_BAD_INPUT, script_untouched, hook=".claude/hooks/fake-guard.sh", feeds=SELFTEST_FEEDS)
    run_scripts("脚本规格：带 --hook：不收（脚本没有钩子要喂）", [script_version_edit], EXIT_BAD_INPUT, script_untouched, hook=".claude/hooks/fake-guard.sh")
    run_scripts("脚本规格：带 --feeds：不收", [script_version_edit], EXIT_BAD_INPUT, script_untouched, feeds=SELFTEST_FEEDS)
    run("钩子规格：带 --verify：不收", [SELFTEST_GOOD_EDIT], EXIT_BAD_INPUT, untouched, verify="true")
    run("钩子规格：不带 --hook：不收", [SELFTEST_GOOD_EDIT], EXIT_BAD_INPUT, untouched, hook=None)

    # 调用方引到的名字：换上的库去掉一个名字，各种载入写法的调用方（spec 加函数返回、heredoc 里 import as、self 属性、形参、列表按下标取、global 元组、
    # 转手交给另一份库里的函数）都要被点名；库里名字齐的照换，认不出去向的写法列成本次未判；只核不换用同一套判法
    def callers_repository(name, opaque=False):
        root = fresh_repository(name)
        samples = dict(SELFTEST_CALLER_SAMPLES)
        if opaque:
            samples["fake_opaque.py"] = SELFTEST_OPAQUE_CALLER
        for file_name, text in samples.items():
            with open(hooks_file(root, file_name), "w", encoding="utf-8") as handle:
                handle.write(text)
        return root

    def swap_callers(name, root, spec, feeds):
        try:
            return swap(spec, ".claude/hooks/fake-guard.sh", os.path.join(work, name, "scratch"), root, feeds=feeds)
        except BadInput as error:
            return EXIT_BAD_INPUT, str(error).splitlines()

    def callers_case(label, spec, feeds, want_code, want_library, want_lines=(), opaque=False):
        nonlocal checked
        checked += 1
        name = f"callers-case-{checked}"
        root = callers_repository(name, opaque)
        code, lines = swap_callers(name, root, spec, feeds)
        problems = [] if code == want_code else [f"退出码 {code}（应当 {want_code}）"]
        if read_text(hooks_file(root, "fake_lib.py")) != want_library:
            problems.append("fake_lib.py 的内容不对")
        problems += [f"输出里没有「{want}」" for want in want_lines if not any(want in line for line in lines)]
        if code != EXIT_SWAPPED and not any(line.lstrip().startswith("→") for line in lines):
            problems.append("拒绝没带「→」出路")
        if problems:
            failures.append(f"{label}：{'；'.join(problems)}；输出末几行 {lines[-4:]}")
        return lines

    removed_name_spec = [{"file": ".claude/hooks/fake_lib.py", "old": "def refused_v2(command):", "new": "def refused_v3(command):"}]
    removed_label = "引到的名字：库里去掉一个名字，各种载入写法的调用方都被点名，草稿验不过、一个字节不写"
    removed_lines = callers_case(removed_label, removed_name_spec, SELFTEST_FEEDS[:2], EXIT_DRAFT_RED, SELFTEST_LIBRARY,
                                 want_lines=[f"{name}:" for name in SELFTEST_CALLERS_OF_REFUSED_V2])
    named_lines = [line for line in removed_lines if "引了 fake_lib.refused_v2" in line]
    if len(named_lines) != len(SELFTEST_CALLERS_OF_REFUSED_V2) or any("fake-guard.sh" in line or "fake_outer.py" in line for line in named_lines):
        failures.append(f"{removed_label}：点名的应当恰好是引了 refused_v2 的那 {len(SELFTEST_CALLERS_OF_REFUSED_V2)} 份，实际 {len(named_lines)} 行 {named_lines}")
    callers_case("引到的名字：名字都在的换上，汇总里写缺 0 处", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library, want_lines=["缺 0 处"])
    callers_case("引到的名字：载入的库交给认不出的调用，列成本次未判、仍照常换上", [SELFTEST_GOOD_EDIT], SELFTEST_FEEDS, EXIT_SWAPPED, good_library,
                 want_lines=["本次未判（引到的名字）：.claude/hooks/fake_opaque.py", "交给 register"], opaque=True)
    # 只核不换：缺名字退 1 并逐处点名，名字齐退 0，不是 .py 的库路径拒（退 2）；一个字节不写
    checked += 1
    root = callers_repository(f"callers-case-{checked}")
    with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_LIBRARY.replace("def refused_v2(", "def refused_v3("))
    code, lines = check_callers(root, [])
    problems = [] if code == EXIT_DRAFT_RED else [f"缺名字时退 {code}（应当 {EXIT_DRAFT_RED}）"]
    if len([line for line in lines if line.startswith("  ✗ ") and "引了 fake_lib.refused_v2" in line]) != len(SELFTEST_CALLERS_OF_REFUSED_V2):
        problems.append(f"逐处点名的应当 {len(SELFTEST_CALLERS_OF_REFUSED_V2)} 行")
    with open(hooks_file(root, "fake_lib.py"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_LIBRARY)
    code, lines = check_callers(root, [".claude/hooks/fake_lib.py"])
    if code != EXIT_SWAPPED or not any("缺 0 处" in line for line in lines):
        problems.append(f"名字齐时退 {code}（应当 {EXIT_SWAPPED}）或汇总里没有「缺 0 处」")
    try:
        check_callers(root, ["research/notes.txt"])
        problems.append("库路径不是 .py 的没被拒")
    except BadInput:
        pass
    if problems:
        failures.append(f"只核不换（--check-callers）：{'；'.join(problems)}")
    # 库里模块层的名字怎么认：import、class、def、全局赋值（元组、星号）、for / try / except 里的、函数里 global 的；动态定义的写法要报
    checked += 1
    defined, dynamic = module_level_names("import os as operating\nfrom a import b\n\n\nclass C:\n    pass\n\n\ndef f():\n    global late\n    late = 1\n\n\n"
                                          "for loop_variable in []:\n    pass\ntry:\n    import tomllib\nexcept ImportError as error:\n    tomllib = None\nx, (y, *z) = 1, (2, 3)\n")
    wanted = {"operating", "b", "C", "f", "late", "loop_variable", "tomllib", "error", "x", "y", "z"}
    if not wanted <= defined or "os" in defined or dynamic:
        failures.append(f"库里模块层的名字认错：缺 {sorted(wanted - defined)}，多了 {sorted(defined & {'os'})}，动态写法 {dynamic}")
    defined, dynamic = module_level_names("from a import *\n\n\ndef __getattr__(name):\n    return 1\n\n\nglobals()['v'] = 1\n")
    if len(dynamic) != 3:
        failures.append(f"库里动态定义名字的写法没认全：{dynamic}（应当 3 种：星号导入、__getattr__、globals()）")
    # 结局文件（文件头「结局文件」）：换上之前写开始，全换完、回读、再验过才写完成；被杀在两份之间，下一趟拒、点名反向规格；按内容判，手工退回之后不再拦
    # （开关 outcome-start-omitted 下「被杀在两份之间，只有开始」「被杀在两份之间之后，下一趟拒」两格红；outcome-complete-first 下「换完的一趟依次写」「被杀在两份之间，只有开始」
    # 「被杀在两份之间之后，下一趟拒」「全换上了还没写完成就被杀」四格红；pending-not-checked 下「被杀在两份之间之后，下一趟拒」「全换上了还没写完成就被杀」两格红；
    # outcome-reader-trusts-file 下「被杀在两份之间，只有开始」「完成之后文件又被改」两格红）
    class KilledForSelftest(BaseException):
        """模拟被内存上限直接杀掉：不走 except Exception，也不走回滚。"""

    outcome_spec = [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}, SELFTEST_GOOD_EDIT]
    outcome_hook = ".claude/hooks/fake-guard.sh"
    second_spec = [{"file": ".claude/hooks/fake_lib.py", "old": "# 新判法\n", "new": "# 新判法二\n"}]
    reader_mode = os.environ.get(BREAK_VARIABLE, "")   # 读结局文件的函数直接调，弄坏开关要自己带进去（swap() 里的 break_mode() 读的是同一个环境变量）

    def outcome_repository():
        nonlocal checked
        checked += 1
        root = fresh_repository(f"outcome-{checked}")
        subprocess.run(["git", "init", "-q", root], check=True, capture_output=True)   # 结局文件放在 git-dir 里
        return root, os.path.join(work, f"outcome-{checked}", "scratch")

    def swap_for_outcome(root, scratch, spec, **hooks):
        try:
            return swap(spec, outcome_hook, scratch, root, feeds=SELFTEST_FEEDS, **hooks)
        except BadInput as error:
            return EXIT_BAD_INPUT, str(error).splitlines()

    def kill_after_first_file(relative):
        raise KilledForSelftest()

    def kill_after_all_files():
        raise KilledForSelftest()

    def killed_swap(root, scratch, **hooks):
        try:
            swap_for_outcome(root, scratch, outcome_spec, **hooks)
        except KilledForSelftest:
            return
        failures.append("结局文件：模拟被杀的那一趟没有被杀（自证的钩子没生效）")

    label = "结局文件：换完的一趟依次写开始、文件、回读、完成，--read-outcome 退 0"
    root, scratch = outcome_repository()
    code, lines = swap_for_outcome(root, scratch, outcome_spec)
    path = outcome_path(root, scratch)
    kinds = [row.split("\t")[0] for row in read_text(path).splitlines() if row and not row.startswith("#")] if os.path.exists(path) else []
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    problems = []
    if code != EXIT_SWAPPED:
        problems.append(f"换上退 {code}（应当 {EXIT_SWAPPED}）：{lines[-3:]}")
    if kinds != ["开始", "文件", "文件", "回读", "回读", "完成"]:
        problems.append(f"结局文件的行种类 {kinds}（应当 开始、文件、文件、回读、回读、完成）")
    if not path.startswith(os.path.join(root, ".git") + os.sep):
        problems.append(f"结局文件 {path} 不在仓根的 git-dir 下")
    if read_code != 0:
        problems.append(f"--read-outcome 退 {read_code}（应当 0）")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 完成之后文件又被改：--read-outcome 退 4 并点名
    label = "结局文件：完成之后文件又被改，--read-outcome 退 4 并点名"
    if os.path.exists(path):
        with open(hooks_file(root, "fake-guard.sh"), "a", encoding="utf-8") as handle:
            handle.write(SELFTEST_OTHER_SESSION_LINE)
        read_code, read_lines = read_outcome(path, root, reader_mode)
        if read_code != 4 or not any("fake-guard.sh" in line for line in read_lines):
            failures.append(f"{label}：--read-outcome 退 {read_code}（应当 4），输出 {read_lines[:3]}")
    else:
        failures.append(f"{label}：上一格没留下结局文件")
    # --dry-run 不写结局文件
    label = "结局文件：--dry-run 不写结局文件"
    root, scratch = outcome_repository()
    code, lines = swap(outcome_spec, outcome_hook, scratch, root, feeds=SELFTEST_FEEDS, dry_run=True)
    if code != EXIT_SWAPPED or os.path.exists(outcome_path(root, scratch)):
        failures.append(f"{label}：退 {code}，结局文件{'在' if os.path.exists(outcome_path(root, scratch)) else '不在'}（应当 {EXIT_SWAPPED}、不在）")
    # 没有结局文件
    label = "结局文件：没有结局文件，--read-outcome 退 2"
    read_code, _ = read_outcome(outcome_path(root, scratch), root, reader_mode)
    if read_code != 2:
        failures.append(f"{label}：退 {read_code}（应当 2）")
    # 被杀在两份之间：只有开始与文件行
    label = "结局文件：被杀在两份之间，只有开始与文件行，--read-outcome 退 3、点名已换上与没换上"
    root, scratch = outcome_repository()
    killed_swap(root, scratch, after_file_swapped=kill_after_first_file)
    path = outcome_path(root, scratch)
    read_code, read_lines = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, ["（没有结局文件）"])
    problems = []
    if read_code != 3:
        problems.append(f"--read-outcome 退 {read_code}（应当 3）")
    if not any(f"{STATE_LANDED}：.claude/hooks/fake-guard.sh" in line for line in read_lines) or not any(f"{STATE_NOT_LANDED}：.claude/hooks/fake_lib.py" in line for line in read_lines):
        problems.append(f"没有逐份点名「已换上 fake-guard.sh」「没换上 fake_lib.py」：{read_lines[:4]}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 下一趟拒
    label = "结局文件：被杀在两份之间之后，下一趟拒（退 7），点名反向规格，一个字节不写"
    library_before = read_text(hooks_file(root, "fake_lib.py"))
    guard_before = read_text(hooks_file(root, "fake-guard.sh"))
    code, lines = swap_for_outcome(root, scratch, [SELFTEST_GOOD_EDIT])
    problems = []
    if code != EXIT_PENDING_OUTCOME:
        problems.append(f"下一趟退 {code}（应当 {EXIT_PENDING_OUTCOME}）")
    if not any("reverse-" in line and ".json" in line for line in lines):
        problems.append(f"拒绝说明里没有点名反向规格 reverse-*.json：{lines[:3]}")
    if read_text(hooks_file(root, "fake_lib.py")) != library_before or read_text(hooks_file(root, "fake-guard.sh")) != guard_before:
        problems.append("拒绝了还是写了字节")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 手工退回到换前之后，按内容判不再拦
    label = "结局文件：被杀之后手工退回到换前，下一趟照换"
    with open(hooks_file(root, "fake-guard.sh"), "w", encoding="utf-8") as handle:
        handle.write(SELFTEST_GUARD)
    code, lines = swap_for_outcome(root, scratch, [SELFTEST_GOOD_EDIT])
    if code != EXIT_SWAPPED:
        failures.append(f"{label}：退 {code}（应当 {EXIT_SWAPPED}）：{lines[-3:]}")
    # 全换上了、还没写完成就被杀：下一趟拒，--seal-outcome 补写之后照换
    label = "结局文件：全换上了还没写完成就被杀，下一趟拒（退 7）、--seal-outcome 补写完成之后照换"
    root, scratch = outcome_repository()
    killed_swap(root, scratch, after_swap=kill_after_all_files)
    path = outcome_path(root, scratch)
    problems = []
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if read_code != 3:
        problems.append(f"--read-outcome 退 {read_code}（应当 3：没写完成）")
    code, lines = swap_for_outcome(root, scratch, second_spec)
    if code != EXIT_PENDING_OUTCOME:
        problems.append(f"下一趟退 {code}（应当 {EXIT_PENDING_OUTCOME}）")
    seal_code, seal_lines = seal_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if seal_code != EXIT_SWAPPED:
        problems.append(f"--seal-outcome 退 {seal_code}（应当 0）：{seal_lines}")
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if read_code != 0:
        problems.append(f"补写之后 --read-outcome 退 {read_code}（应当 0）")
    code, lines = swap_for_outcome(root, scratch, second_spec)
    if code != EXIT_SWAPPED:
        problems.append(f"补写完成之后下一趟退 {code}（应当 {EXIT_SWAPPED}）：{lines[-3:]}")
    if problems:
        failures.append(f"{label}：{'；'.join(problems)}")
    # 换上之后验不过、已回滚：写退回
    label = "结局文件：换上之后红回滚，写退回，--read-outcome 退 1"

    def other_session_drops_v2_for_outcome(root_now):
        with open(hooks_file(root_now, "fake_lib.py"), "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_LIBRARY_WITHOUT_V2)
    root, scratch = outcome_repository()
    code, lines = swap_for_outcome(root, scratch, [{"file": ".claude/hooks/fake-guard.sh", "old": "lib.refused(command)", "new": "lib.refused_v2(command)"}],
                                   after_draft=lambda: other_session_drops_v2_for_outcome(root))
    path = outcome_path(root, scratch)
    read_code, _ = read_outcome(path, root, reader_mode) if os.path.exists(path) else (None, None)
    if code != EXIT_ROLLED_BACK or read_code != 1:
        failures.append(f"{label}：换上退 {code}（应当 {EXIT_ROLLED_BACK}），--read-outcome 退 {read_code}（应当 1）")
    shutil.rmtree(work, ignore_errors=True)
    for failure in failures:
        print(f"  ✗ hook-swap 自检：{failure}")  # gate-lint:detail
    if failures:
        print(f"  → 看 swap() / check_tree() / rollback() / embedded_python_segments() 的判法；{BREAK_VARIABLE} 设着的话这里本来就该红")
        return 1
    print(f"  ✓ hook-swap 自检通过（查了 {checked} 种）：草稿与换上之后都验过的换上新 inode；钩子 bash 层语法错、内嵌 python 编译不过、判法改错的草稿一个字节不写；"
          "换上之后验不过的按反向定点替换回滚、别的会话这段时间里加的一行留着；验的时候主工作区被改了不换；"
          "不在 .claude/hooks/ 也不在 scripts/ 的文件、回滚找不回原处的规格不收；内嵌 python 切段认 <<- 与没收尾的；内置喂法里的脚本路径都在仓里；内置喂法带花括号与参数通配的六条，"
          "在花括号写法上放行的钩子先验停下并点名那一条；脚本规格（scripts/ 下任意层）：草稿语法检查、--dry-run 停在草稿、换上之后验证命令退 0 才算验过，"
          "验证命令退 1 按反向定点替换回滚（别的会话加的一行留着）、验的时候被改不换、scripts/ 之外的文件与钩子脚本混写的规格不收、"
          "脚本规格带 --hook 或 --feeds、钩子规格带 --verify 或不带 --hook 都不收；换上的 .py 库去掉一个名字，七种载入写法的调用方（spec 加函数返回、heredoc 里 import as、self 属性、形参、"
          "列表按下标取、global 元组、转手交给另一份库的函数）都被点名、草稿验不过，名字齐的照换，认不出去向的写法列成本次未判；只核不换 --check-callers 缺名字退 1、名字齐退 0；"
          "结局文件：换完的一趟依次写开始、文件、回读、完成，--dry-run 不写，被杀在两份之间只有开始与文件行、下一趟拒（退 7）并点名反向规格、手工退回到换前之后不再拦，"
          "全换上了没写完成的要 --seal-outcome 补写，换上之后红回滚写退回，完成之后文件又被改 --read-outcome 退 4")
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    usage = ("hook-swap.py <规格.json> --hook <钩子> --scratch <草稿目录> [--feeds <喂法.json>] [--root <仓根>] [--dry-run]，"
             "或 hook-swap.py <规格.json> --scratch <草稿目录> [--verify <命令>] [--root <仓根>] [--dry-run]（规格里的文件都在 scripts/ 下），"
             "或 hook-swap.py --feed-only --hook <钩子>，或 hook-swap.py --check-callers [<库.py>…]，或 hook-swap.py --read-outcome / --seal-outcome，或 hook-swap.py --selftest")
    options = {"--hook": None, "--scratch": None, "--feeds": None, "--root": REPOSITORY_ROOT, "--verify": None}
    dry_run, feed_only_mode, check_callers_mode, positional, position = False, False, False, [], 0
    read_outcome_mode, seal_outcome_mode = False, False
    while position < len(arguments):
        argument = arguments[position]
        if argument in options and position + 1 < len(arguments):
            options[argument] = arguments[position + 1]
            position += 2
            continue
        if argument == "--dry-run":
            dry_run = True
        elif argument == "--feed-only":
            feed_only_mode = True
        elif argument == "--check-callers":
            check_callers_mode = True
        elif argument == "--read-outcome":
            read_outcome_mode = True
        elif argument == "--seal-outcome":
            seal_outcome_mode = True
        else:
            positional.append(argument)
        position += 1
    if read_outcome_mode or seal_outcome_mode:
        if (read_outcome_mode and seal_outcome_mode) or positional or feed_only_mode or check_callers_mode or dry_run or any(options[name] is not None for name in ("--hook", "--feeds", "--verify")):
            print("✗ 参数不对：--read-outcome、--seal-outcome 二选一，只收 --root 和 --scratch（仓根不是 git 工作树的根时点名上一趟用的草稿目录）")
            print("→ 怎么办：hook-swap.py --read-outcome [--root <仓根>] [--scratch <草稿目录>]")
            return EXIT_BAD_INPUT
        try:
            outcome_root = os.path.realpath(options["--root"])
            code, lines = (read_outcome if read_outcome_mode else seal_outcome)(outcome_path(outcome_root, options["--scratch"]), outcome_root, break_mode())
        except BadInput as error:
            problem, _, next_step = str(error).partition("\n")
            print(f"✗ {problem}")
            print(f"→ {next_step.lstrip('→ ') or usage}")
            return EXIT_BAD_INPUT
        print("\n".join(lines))
        return code
    if check_callers_mode:
        if feed_only_mode or dry_run or any(options[name] is not None for name in ("--hook", "--scratch", "--feeds", "--verify")):
            print("✗ 参数不对：--check-callers 只收库路径（可省）和 --root，不收 --hook、--scratch、--feeds、--verify、--feed-only、--dry-run")
            print("→ 怎么办：hook-swap.py --check-callers [<库.py>…] [--root <仓根>]")
            return EXIT_BAD_INPUT
        try:
            code, lines = check_callers(options["--root"], positional)
        except BadInput as error:
            problem, _, next_step = str(error).partition("\n")
            print(f"✗ {problem}")
            print(f"→ {next_step.lstrip('→ ') or usage}")
            return EXIT_BAD_INPUT
        print("\n".join(lines))
        return code
    if feed_only_mode:
        if positional or options["--hook"] is None:
            print("✗ 参数不对：--feed-only 只要 --hook（可带 --feeds、--root），不收规格与 --scratch")
            print("→ 怎么办：hook-swap.py --feed-only --hook <钩子> [--feeds <喂法.json>] [--root <仓根>]")
            return EXIT_BAD_INPUT
        try:
            feeds = None
            if options["--feeds"] is not None:
                with open(options["--feeds"], encoding="utf-8") as handle:
                    feeds = json.load(handle)
            code, lines = feed_only(options["--hook"], options["--root"], feeds=feeds)
        except (OSError, ValueError) as error:
            print(f"✗ 读不了喂法（{error}）")
            print("→ 怎么办：喂法是 UTF-8 的 JSON 数组，写法见文件头")
            return EXIT_BAD_INPUT
        except BadInput as error:
            problem, _, next_step = str(error).partition("\n")
            print(f"✗ {problem}")
            print(f"→ {next_step.lstrip('→ ') or usage}")
            return EXIT_BAD_INPUT
        print("\n".join(lines))
        return code
    if len(positional) != 1 or options["--scratch"] is None:
        print("✗ 参数不对：要一份规格与 --scratch（钩子规格另要 --hook）")
        print(f"→ 怎么办：{usage}")
        return EXIT_BAD_INPUT
    try:
        with open(positional[0], encoding="utf-8") as handle:
            spec = json.load(handle)
        feeds = None
        if options["--feeds"] is not None:
            with open(options["--feeds"], encoding="utf-8") as handle:
                feeds = json.load(handle)
    except (OSError, ValueError) as error:
        print(f"✗ 读不了规格或喂法（{error}）")
        print("→ 怎么办：两份都是 UTF-8 的 JSON 数组，写法见文件头")
        return EXIT_BAD_INPUT
    try:
        code, lines = swap(spec, options["--hook"], options["--scratch"], options["--root"], feeds=feeds, dry_run=dry_run, verify=options["--verify"])
    except BadInput as error:
        problem, _, next_step = str(error).partition("\n")
        print(f"✗ {problem}")
        print(f"→ {next_step.lstrip('→ ') or usage}")
        return EXIT_BAD_INPUT
    print("\n".join(lines))
    return code


BREAK_VARIABLES = ("HOOK_SWAP_BREAK",)


def inside_selftest():
    """往上数这个进程的祖先，有一个的命令行里带 --selftest（自证起的子进程，含别的脚本的自证起它的）就是 True。"""
    pid = os.getppid()
    while pid > 1:
        try:
            with open(f"/proc/{pid}/cmdline", "rb") as handle:
                if b"--selftest" in handle.read().split(b"\0"):
                    return True
            with open(f"/proc/{pid}/stat", "rb") as handle:
                pid = int(handle.read().rsplit(b")", 1)[1].split()[1])
        except (OSError, ValueError, IndexError):
            return False
    return False


def settle_break_switches(arguments):
    """弄坏开关只在自证里认（checks-owed.md C622），参数解析之前先过这一道。
    放行：命令行以 --selftest 起头；祖先里有自证（inside_selftest）。
    都不是而 BREAK_VARIABLES 里有设了的：点名变量、说它只在自证里认，交回退出码 2；否则交回 None。"""
    setting = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if not setting or arguments[:1] == ["--selftest"] or inside_selftest():
        return None
    print(f"  ✗ {'、'.join(setting)} 是弄坏开关，只在自证（--selftest 本身与它起的子进程）里认；平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {' '.join(setting)} 再来；要看开关红在哪一项，跑 {setting[0]}=<项> python3 scripts/gate/hook-swap.py --selftest", file=sys.stderr)
    return 2


if __name__ == "__main__":
    refused = settle_break_switches(sys.argv[1:])
    if refused is not None:
        sys.exit(refused)
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
