# 第九批 landing-audit-b9-watch-cost：看门狗两类误报、主会话按记录时刻切天、结局账落盘（tooling-writer，2026-10-23）

依据：`records/2026-09-16-subagent拆分提案.md` 第九十节 ⑬、⑭、⑲、㉒ 四行；㉒ 用户定案「每批收尾跑一次（推荐）」（`research/prompts/landing-audit-r3-main-verification.md` 节「用户定案（2026-10-23，弹窗）」）。

## 结论

四条的验收都做到了，改动已经定点换进主工作区，换上之后在主工作区跑过：

- ⑬：峰值表算出来的阈值（峰值 ×1.5）不到下限的，改取下限。下限是 `--process-rss-floor-mebibytes`（默认 512，`watch.conf` 写 512）和 `--process-rss-gibibytes` 里小的那个；峰值记成 0 的条目照旧不拿来当阈值，走固定线。
- ⑭：登记成 light 的自证在自己临时目录里写的 ✗ 不再告警，报告里只留一行「✗ 不报」；写在别处的照报。怎么认自证，用的是看门狗现有的 `registered_light_selftest`（同一份登记表）。怎么认临时目录分两种：
  - 单给这条自证的 TMPDIR 整个都算（light-selftest-guard.py 给的私有 TMPDIR、lib_selftest_scratch 拉起子进程时给的 TMPDIR）；
  - 和上层共用的 TMPDIR 只算下一层：名字像 mktemp 起的，或以 `lib_selftest_scratch.py` 的 `PASS_PROBE_PREFIX`、`CHILD_PROBE_PREFIX` 起头的（这两个前缀运行时从那份库读）。
- ⑲：`usage`、`dispatch` 统计主会话时，改成按每条记录自己的时刻筛，切天用的是同一个 `window_bounds`，按东京日期切；`--until` 放开给四个子命令都能用。
- ㉒：`outcome` 加了 `--out <文件>`。文件只排他新建，已经存在就拒（退 2，给出路）；写完回读，与标准输出逐字相同才退 0。用法写进了文件头，带一条每批收尾落账的写法。

自证（主工作区）：

| 自证 | 结果 |
|---|---|
| `agent-cost.py --selftest` | 退 0，查了 22 项（原来 18 项，新加 4 项） |
| `agent-watch.py --selftest` | 退 0，175 秒，判定行 7 份日志（原来 2 份，新加 5 份），常驻内存那一段新加 2 格 |
| `watch.sh --selftest` | 退 0 |

新加的 10 个弄坏开关逐个打开过，都红在点名的那几格。

什么现象会推翻这些结论：

- ⑭：一趟门禁 code-tooling 的 research-script-selftests 格，看门狗还报 `light-selftest-guard-*/tmp/…/switch-wheel-*.out` 的「长跑里有判红的行」；或者一条真红的长跑日志，被写成「在登记成轻的自证 … 的临时目录里，✗ 不报」。
- ⑬：`cat`、`sleep` 这类常驻几 MiB 的进程还被报「进程常驻内存过线」，阈值写的是 0.00 GiB。
- ⑲：`usage --since D` 统计的主会话调用里，还有早于东京 D 日零点的记录。
- ㉒：`outcome --out` 覆盖了已经存在的文件，或者文件内容与标准输出不同却退了 0。

## 改过的文件

只改了派发提示点名的四份，都是定点替换（`scripts/kb/replace-batch.py`，同目录临时文件改名换上），没有整份覆盖：

- `scripts/agents/agent-watch.py`
- `scripts/agents/agent-cost.py`
- `scripts/agents/watch.conf`：加了 `process-rss-floor-mebibytes` 键与说明
- `scripts/agents/watch.sh`：只往键表 `known=` 里加了一个键

换之前核过：这四份的暂存区版本与我取的基线逐字相同，所以下面的 `git diff --stat`（工作区对暂存区）恰好就是这一批的改动。别的会话（singlefs-f6）加的 `selftest_root_free`、`AGENT_WATCH_BREAK=rootfree`、`root-free-gib-alert` 都还在（`grep -c` 读数：agent-watch.py 19 处，watch.sh 3 处，watch.conf 2 处）。

`git diff --stat -- scripts/agents/agent-watch.py scripts/agents/agent-cost.py scripts/agents/watch.sh scripts/agents/watch.conf` 的原样输出（终版，开关表补的 7 处已在内；写报告时又重跑过一次，读数相同）：

```
 scripts/agents/agent-cost.py  | 218 ++++++++++++++++++++++++++-----
 scripts/agents/agent-watch.py | 295 +++++++++++++++++++++++++++++++++++++++---
 scripts/agents/watch.conf     |   4 +
 scripts/agents/watch.sh       |   2 +-
 4 files changed, 473 insertions(+), 46 deletions(-)
```

终版 sha256（主工作区）：

- agent-watch.py `72819e05ff2ec6da3a3d26057b9c0bf5d170e31a9161afcd09eeefbac477ad71`
- agent-cost.py `2fbd9c46de3e0965f4f2e9a491f041993977aa0cbfd82967e5aa75f2e1bd0918`
- watch.sh `fa9b6a8fb4a779af1a42bc8f5010570f10cd24ac46a6ceb89b7dc027deba08cb`
- watch.conf `03ec5af7ea21bad5707b11da723cc648c95fbf6124ae89bf1ec5145970b33ef0`

（四个 sha256 是在三份规格全部施加之后取的，写报告时重取过，读数相同。）

语法检查：`python3 -m py_compile` 两份 .py 都过，`bash -n scripts/agents/watch.sh` 也过。

回滚用的规格在草稿目录里：`reverse-spec.json`、`reverse-spec-switch-table.json`、`reverse-spec-cost-table.json`，三份的新旧串与次序都已倒过来。按下面的次序跑 `python3 scripts/kb/replace-batch.py <规格>`：先 cost-table，再 switch-table，最后 spec。

改前的基线在 `base/` 下。

## ⑬ 常驻内存的阈值：峰值表的条目不到下限时取下限

改法（`scripts/agents/agent-watch.py`）：

- `memory_threshold`（第 2659 行起）的 ② 那一支：`limit = 峰值 × CAP_FACTOR`，不到 `min(--process-rss-floor-mebibytes × MiB, --process-rss-gibibytes)` 的取这个下限，来源记成 `peakfloor`，阈值的文字写「… 不到下限，取下限（--process-rss-floor-mebibytes 512 与 --process-rss-gibibytes 里小的那个）」。
- 峰值是 0 的条目照旧不进 ② 这一支（`if peak:`），走 ③ 的固定线。
- 报告的汇总行写「照峰值表 N 个（其中峰值乘出来不到下限、取下限的 M 个）」。
- 新常数 `MEMORY_PEAK_FLOOR_MEBIBYTES_DEFAULT = 512`（第 655 行）。
- 新选项 `--process-rss-floor-mebibytes`（第 6056 行）。
- `watch.conf` 加 `process-rss-floor-mebibytes=512`，文件头补了说明；`watch.sh` 的键表加了这个键。键用 MiB 计，因为 `watch.sh` 只认整数值。
- 文件头「进程常驻内存过线」② 一段补了下限的规则与两个开关。

下限为什么取 512 MiB：

- 现读 `~/.local/share/singlefs/memory-peaks.tsv`（2026-10-22，30742 条）：峰值不到 8 MiB 的 2787 条，不到 64 MiB 的 11855 条，不到 512 MiB 的 19163 条，中位 165 MiB。
- 峰值表记的是包装 scope 的 `memory.peak`，含页缓存与外壳 bash，和单个进程的常驻内存不是同一个量。几十 MiB 以下按 1.5 倍去比，比不出东西：`cat` 记成 0.00 GiB，阈值就是 0；`agent-cost.py --selftest` 记成 0.01 GiB，1.5 倍是 0.02 GiB，实跑 0.03 GiB 就报。
- 512 不到本机 60 GiB 的 1%，是 `--process-rss-gibibytes` 默认 8 GiB 的十六分之一。到这个量级才可能是在涨的无界分配。
- 512 这个数本身是推的，没量过；依据写在了选项的 help 与 `watch.conf` 里。
- 拿 `--process-rss-gibibytes` 当上界，是为了让「峰值表里有、但峰值很小」的进程，阈值不会比「表里没有」的进程还高（自证里固定线压到 0.125 GiB 时也成立）。

该红的输入：峰值表给一个只睡的 `sleep` 配峰值 4 KiB（显示成 0.00 GiB）。用的是草稿探针 `probe_watch.py`，按文件路径导入被测的 agent-watch.py，调 `memory_report`。

改前（主工作区原文，`probe-watch-before.out`）：

```
⑬ 峰值 4 KiB 的条目、只睡的 sleep：报了 ['进程 2160221 常驻内存 0.00 GiB（阈值 0.00 GiB，峰值表里这条命令的峰值 0.00 GiB 的 1.5 倍），已跑 0s：sleep 120；归属：（单查不认归属）']
```

改后（草稿 `probe-watch-draft.out`，换上后主工作区 `probe-watch-after.out`，读数相同）：

```
⑬ 峰值 4 KiB 的条目、只睡的 sleep：没报 []
```

自证新加的两格在 `selftest_memory` 里（第 4845 行起）：

1. 「峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」：峰值 4 KiB 的只睡 python 进程与峰值 0 的 `sleep`，`--process-rss-gibibytes 8`，两个都不报，报告写「取下限的 1 个」。
2. 「峰值很小、常驻过了下限的进程应当照下限报」：峰值 4 KiB、`--process-rss-floor-mebibytes 128`，写满 256 MiB 的进程照报，阈值写「… 不到下限，取下限 …」。

弄坏开关（单跑 `selftest_memory` 用 `run_cells.py`，整份 `--selftest` 用 `full-break-*.out`，两种读数相同）：

| 开关 | 弄坏哪一处 | 红的格 |
|---|---|---|
| `AGENT_WATCH_BREAK=memorypeakfloor` | 不设下限（改前） | 「峰值表里记成 0.00 GiB 的条目配只睡的小进程不该报」「峰值很小、常驻过了下限的进程应当照下限报」 |
| `AGENT_WATCH_BREAK=memorypeakfloorfixed` | 不到下限的退回 `--process-rss-gibibytes` | 「峰值很小、常驻过了下限的进程应当照下限报」，另外红「峰值表里记成 0.00 GiB …」「峰值表里有这条命令的进程应当照峰值判」（报告里取下限的计数变成 0） |

原有的三个开关也会红这两格，开关表已经照实补上：

- `memoryfixedline`：两格都红。
- `memoryscan`：两格都红。
- `memoryunit`：红前一格。按 MiB 算，下限成了 8 MiB，只睡的 python 常驻超过 8 MiB 时红。

## ⑭ 自证临时目录里的 ✗ 不判「长跑判红」

改法（`scripts/agents/agent-watch.py`）：

- `verdict_line_report` 对有 ✗ 的文件调 `light_selftest_scratch_owner`（第 2159 行起）。它从写这份文件的进程往上走，到看门狗的根进程为止，用 `registered_light_selftest` 找登记成 light 的自证。登记表就是 `.claude/hooks/selftest-registry.tsv`，没有另立名单。
- 找到自证，而文件落在它的临时目录里，就只在报告里写一行「（进程 N 写着，在登记成轻的自证 … 的临时目录里，✗ 不报）：✓ g ✗ r」，不告警。汇总行加了「登记成轻的自证临时目录里有 ✗ 的 K 份不报」。
- 「它的临时目录」分两种：
  - `private_temporary_base`（第 2199 行起）判它的 TMPDIR 是不是单给它的：先往上跳过只是包着它的几层，判据是这一层的参数以它的参数收尾，strace … --、light-selftest-guard.py … --、timeout、nice 都是这样；到第一个不是包着它的上层，那一层的 TMPDIR 与它的不同，就是单给它的，整个 TMPDIR 都算。
  - 和上层共用的 TMPDIR，只算 TMPDIR 下一层：名字像 mktemp 起的（沿用现有的 `SELFTEST_TEMPORARY_NAME`），或以 `lib_selftest_scratch.py` 的 `PASS_PROBE_PREFIX`、`CHILD_PROBE_PREFIX` 起头的。两个前缀由 `selftest_scratch_prefixes`（第 2145 行）按文件路径读那份库，读不了时报告里写明。
- 文件头「长跑里有判红的行」一段写了这套认法、射程与 7 个开关。

为什么不只照 `lib_selftest_scratch.py` 认：派发提示的例子 `/tmp/light-selftest-guard-*/…/switch-wheel-*.out` 不是那份库建的。它是 `scripts/gate/light-selftest-guard.py` 的 `guard()` 用 `mkdtemp(prefix="light-selftest-guard-")` 建的，再把 `<那个目录>/tmp` 当私有 TMPDIR 交给自证（那份文件第 565、578 行）。崩溃放量的自证又在那里面用 `mktemp -d "${TMPDIR:-/tmp}/crash-amplification-selftest.XXXXXX"` 建目录（`scripts/crash/crash-amplification.sh` 第 791 行），这个名字不像 `tmp.XXXX`，只认 mktemp 名字与库前缀的话认不出。所以加了「单给它的 TMPDIR」这一种：只看进程树与环境，不认任何脚本名。

该红的输入有两份，都用草稿探针 `probe_watch.py`（登记表照 `selftest_registry_swapped` 换成探针自己的一行 light）：

- 甲：登记成 light 的 `scripts/crash/sample-wheel.sh --selftest` 在 `mktemp -d` 里写带 ✗ 的开关轮输出，并且开着这份文件写。
- 乙：照守卫的形状，`timeout` 包一层，与自证同一个私有 TMPDIR；自证用 `mktemp -d "$TMPDIR/crash-amplification-selftest.XXXXXX"` 建目录，由子 shell 写 `switch-wheel-3.out`。

改前（`probe-watch-before.out`，节选原样）：

```
⑭ 登记成轻的自证临时目录里的开关轮 ✗：报了 ['进程 2160822 写着的 /tmp/probe-watch-58wax_o3/tmp-base/tmp.l5Ktylr0bo/switch-wheel-0.out 里判定行 ✓ 1 ✗ 1；✗ 的是：✗ 开关轮 beta 红在点名那一格']
⑭ 照守卫形状（私有 TMPDIR、包一层、子 shell 写开关轮输出）的 ✗：报了 ['进程 2165072 写着的 /tmp/probe-watch-58wax_o3/light-selftest-guard-probe/tmp/crash-amplification-selftest.wFfQFc/switch-wheel-3.out 里判定行 ✓ 0 ✗ 1；✗ 的是：✗ 开关轮 omega 红在点名那一格']
```

改后（`probe-watch-after.out`，主工作区，节选原样；草稿那一份同样两条都没报）：

```
⑭ 登记成轻的自证临时目录里的开关轮 ✗：没报 []
   报告行：判定行：跑满 0.02 分钟的进程开着写的 1 份文件，1 份里有 ✓ / ✗ 开头的行，报 0 条，登记成轻的自证临时目录里有 ✗ 的 1 份不报 ｜   判定行 /tmp/probe-watch-ddxce3we/tmp-base/tmp.mmN4d…
⑭ 照守卫形状（私有 TMPDIR、包一层、子 shell 写开关轮输出）的 ✗：没报 []
```

自证新加一个函数 `selftest_verdict_selftest_scratch`（第 4658 行起，接在 `selftest_verdict_lines` 后面跑）。它起三样：

- 共用 TMPDIR 的一份自证：经 `bash -c` 起，上层与它同一个 TMPDIR；
- 单给 TMPDIR 的一份自证：经 `timeout` 起；
- 一个没有自证上层的长跑进程。

断言 5 份文件，分三格：

1. 「登记成轻的自证在自己临时目录里写的 ✗ 不该报」：三份不报，各有一行「✗ 不报」。三份是共用 TMPDIR 那一份在 mktemp 目录里写的、在探查目录前缀目录里写的，和单给 TMPDIR 那一份在 `sample-wheel-selftest.XXXXXX` 里写的。
2. 「共用 TMPDIR 的自证直接写在 TMPDIR 底下的 ✗ 应当报」。
3. 「没有自证上层的进程写在像 mktemp 的临时目录里的 ✗ 应当报」。

弄坏开关（整份 `--selftest` 逐个跑，`full-break-*.out`；单跑两个判定行函数的 `cells-verdict-*.out` 读数相同）：

| 开关 | 弄坏哪一处 | 红的格 |
|---|---|---|
| `verdictselftest` | 一律不认自证临时目录（改前） | 格 1 |
| `verdictselftestwrapperseen` | 不跳过包着它的那几层 | 格 1 |
| `verdictselftestprivateignored` | 不认单给它的 TMPDIR | 格 1 |
| `verdictselftestanyfile` | 自证写的文件不看路径一律不报 | 格 2 |
| `verdictselftestsharedtmpdir` | 共用的 TMPDIR 也整个算 | 格 2 |
| `verdictselftestpathonly` | 只看路径里有没有像 mktemp 的一层 | 格 3 与格 1 |
| 原有的 `verdictlines` | 不数判定行 | 原有两格之外又红格 1、2、3（开关表已补） |

上表开关名前都省了 `AGENT_WATCH_BREAK=`。

射程之外（文件头写了）：和上层共用 TMPDIR 的自证，用自己起的名字 mktemp（`mktemp -d "$TMPDIR/<名>.XXXXXX"`）建的目录认不出，那里的 ✗ 照报。这种情形只在不经 light-selftest-guard.py、直接在门禁阶段的 TMPDIR 里起自证时出现；research-script-selftests 格经守卫起，不在这一种里。

## ⑲ 主会话那部分：按每条记录的时刻筛

改法（`scripts/agents/agent-cost.py`）：

- 新函数 `records_between`（第 167 行）：只交出时刻落在 `[since, until)` 里的记录，不带时刻的元数据条不收，时间戳不带时区的当 UTC。
- `read_transcript(path, since, until)` 给了窗口就走 `records_between`；子 agent 那一侧不给窗口，照旧按起点筛。
- `command_usage` 的主会话一节改调它。「子agent」那一列改数起点落在窗口里的（`transcripts.started_between`）。
- `command_dispatch` 逐条记录改走 `records_between`。
- 修改时刻早于 `--since` 的会话文件仍整份跳过：文件里不会有比修改时刻更晚的记录。
- 四个子命令都收 `--until`（原来只有 outcome 收）；三个子命令的标题经 `window_label`（第 183 行）写出窗口。
- 文件头写了主会话的筛法。

该红的输入：一份跨东京两天的主会话记录。前一天深夜有两次调用，其中一次派 sweep；后一天零点刚过有一次，派 clerk，这一次在 UTC 里还是前一天。取 `--since` 后一天（草稿探针 `probe_cost.py`）。

改前（`probe-cost-before.out` 原样）：

```
⑲ usage --since 2026-10-21：主会话调用 3 次（只该收后一天那 1 次）
⑲ dispatch --since 2026-10-21：派发 {'sweep': 1, 'clerk': 1}（只该收 clerk 1 次）
```

改后（草稿与主工作区 `probe-cost-after.out`，原样）：

```
⑲ usage --since 2026-10-21：主会话调用 1 次（只该收后一天那 1 次）
⑲ dispatch --since 2026-10-21：派发 {'clerk': 1}（只该收 clerk 1 次）
```

自证新加 `selftest_main_session_window`（第 1523 行起），两项：

- `--since 后一天`：usage 收 1 次调用，dispatch 收 `{"clerk": 1}`；
- `--since 前一天 --until 前一天`：收 2 次调用，dispatch 收 `{"sweep": 1}`。

样本里另放了一条不带时刻的元数据条。

弄坏开关：

- `AGENT_COST_BREAK=main-session-by-mtime`：`records_between` 整份收，即改前的样子。红「主会话按记录时刻切天：」两条（`cost-break-main-session-by-mtime.out`）。
- 原有的 `day-cut-by-record-zone` 也红这两条，开关表已补。

## ㉒ 脚本一侧：`outcome --summary --out <文件>` 落账

改法（`scripts/agents/agent-cost.py`）：

- `main` 收 `--out`（第 1739 行，只给 outcome）。算账之前先判两件事：文件已经存在就拒，退 2，写明出路；上层目录不在也拒。
- 算完把「汇总 + 成功句」拼成一份文本，交 `write_ledger_exclusively`（第 1688 行）：以 `x` 模式排他新建，写完回读。逐字相同才把同一份文本打到标准输出、退 0。
- 算账这段时间里别处建了同名文件，退 2；写不进退 1；回读不同就删掉这次建的文件，退 1。
- 窗口里没有子 agent 时退 77，不建文件。
- 成功句末尾写「这份输出也落进了 <文件>（排他新建，回读与标准输出逐字相同）」。
- 文件头的用法写了 `--out`，并给了一行每批收尾落账的写法：`outcome --since <批开工那天> --until <收尾那天> --summary --out <那一轮调度记录目录>/<轮>-outcome-<日期>.txt`。主 agent 定义里那一句不归我，没碰。

改前（`probe-cost-before.out` 原样，窗口里放了 1 个子 agent，同一个路径跑两次）：

```
㉒ 第 1 次 outcome --summary --out：退出码 2，文件不在；输出末行：  → 怎么办：只认 --root 与 --since YYYY-MM-DD；outcome 另认 --until YYYY-MM-DD（含那一天）、--rep
```

改后（`probe-cost-after.out` 原样，截在 250 字）：

```
㉒ 第 1 次 outcome --summary --out：退出码 0，文件与标准输出逐字相同；输出末行：  ✓ 查了 1 个子 agent（--since 2026-10-20、--until 2026-10-20（含那一天），日期按东京日期（Asia/Tokyo）切天，按首条
㉒ 第 2 次 outcome --summary --out：退出码 2，文件与标准输出不同；输出末行：  → 怎么办：换一个还没有的文件名（例如文件名里带这一次 --until 的日期或批名）；旧的那份要作废，先核过它�
```

第 2 次的「与标准输出不同」是指这一次的标准输出是拒绝说明，账文件没被动过。自证里核的是账文件一个字没变。

自证新加 `selftest_outcome_out`（第 1562 行起），两项：

- 第一次跑：退 0，账文件与捕获到的标准输出逐字相同，内容有汇总与成功句；
- 同一路径再跑：退 2，有「✗ --out 指的文件已经在」与「→ 怎么办」，账文件一个字不变。

弄坏开关：

- `AGENT_COST_BREAK=outcome-out-clobber`：不排他、整份覆盖。红「结局账落盘：」一条（`cost-break-outcome-out-clobber.out`）。
- 原有的 `SUBAGENT_TRANSCRIPT_BREAK=start-by-mtime` 也红这一格：样本起点被当成此刻，窗口里没有子 agent。开关表已补。

agent-cost.py 在 `.claude/gate.d/break-switch-replay-files.tsv` 里，所以这份文件的 12 个开关全部逐个跑过。点名的失败行开头都在失败行里，读数如下（每个开关跑一次整份 `--selftest`，`cost-break-*.out`，一行写开关与红的项）：

```
no-dedupe 子 agent 用量 | no-path-normalize 路径归一化 | no-bash-writes Bash 写入 | narrow-rework-words 返工词表 | fix-no-word-end 返工词表
day-cut-by-record-zone 东京日期切天×2、主会话按记录时刻切天×2 | three-way-by-type 三方腿判决不在、花费 | three-way-history-unread 三方腿判决在、花费
main-session-by-mtime 主会话按记录时刻切天×2 | outcome-out-clobber 结局账落盘 | refused-counts-delivered 交回与起点×2
start-by-mtime 东京日期切天、交回与起点、结局账落盘×2
```

每个开关各跑了多久没有单独计时；不开开关的整份 `--selftest` 是 9 秒（改前基线）。

## 自证总数与读数（主工作区，换上之后）

- `nice -n 19 python3 scripts/agents/agent-cost.py --selftest`：退 0。末行原样开头是「  ✓ 自检通过（查了 22 项，其中 outcome 的 20 个子 agent）：…；主会话按每条记录的时刻切天（跨两天的主会话只收窗口里那几条）；outcome --out 排他落账、回读与标准输出逐字相同、已在就拒」。改前是 18 项，新加 4 项（⑲ 两项、㉒ 两项）。
- `timeout 232 nice -n 19 python3 scripts/agents/agent-watch.py --selftest`：退 0，175 秒。成功句里判定行那一段原样是「（7 份日志）（查了 57 个子 agent、58 个进程（其中假的重型进程 21 个）、50 次看门狗）」，末行是临时目录三格的「  ✓ 自检：临时目录在通过、判红、抛异常三条路上都删了（3 格：…）」。
  - 判定行 7 份 = 原有 2 份 + 新加 5 份；常驻内存那一段的新加两格在 `selftest_memory` 里。
  - 改前的基线在本机负载重时跑了 230 秒没跑完（`timeout` 退 124），它在临时目录里留下的 `/tmp/selftest-scratch-pass-probe-ln0iaubb`（1.6M，生成时刻与这一趟开跑时刻相同）已经删了。
- `nice -n 19 bash scripts/agents/watch.sh --selftest`：退 0，「  ✓ watch.sh 自检通过：…」。
- `env -u CLAUDE_CODE_SESSION_ID bash scripts/agents/watch.sh --dry-run --processes` 打出的命令里有 `--process-rss-floor-mebibytes 512`，说明 watch.conf 的新键能经 watch.sh 转到 agent-watch.py；agent-watch.py 的 argparse 认这个选项。
- 新开关逐个打开跑整份 `agent-watch.py --selftest`：八个都退 1，失败只落在点名那几格（见 ⑬、⑭ 两节的表）。
  - 前四个（memorypeakfloor、memorypeakfloorfixed、verdictselftest、verdictselftestwrapperseen）的退出码是逐个 `wait "$pid"` 收的，都是 1。
  - 后四个那一趟我写错了收法（把收的循环接进了管道，`wait` 不认那几个进程），四个进程成了孤儿。之后照写死的进程号逐个 `proc.py wait`，四个都自己跑完退出了。它们的退出码没收到，但四份输出都打出了「✗ 自检：」失败句，并且有收尾那一行，`selftest_in` 有失败句时 `return 1`，所以按退 1 记，这是推的。
  - 每份输出里红的格见上面两张表。

## 新写的 `# gate-similar:` / `# hook-events:` 行

没有。这一批没加门禁阶段，也没加钩子，只改了两份已有的研究脚本，外加 watch.conf 与 watch.sh 的键表。开工时跑过 `python3 .claude/singlefs-ai-sop/scripts/gate-overlap.py --list`（退 0，59 行，`gate-overlap-list.txt`），没有门禁或钩子在管 agent-watch.py、agent-cost.py 的这几条判据。

## 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`。不用内存包装，也不起重活。

1. 复跑方确认 agent-cost.py 的新开关（它在 break-switch-replay 表里），跑门禁 code-tooling 的 break-switch-replay 格：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 break-switch-replay 格; 范围=1 格; 为什么=agent-cost.py 新加 main-session-by-mtime 与 outcome-out-clobber 两个开关、开关表补了两处点名' bash .claude/gate.d/code-tooling.sh --check break-switch-replay /home/fy5090/code/singlefs`
   看那一格的成功句里 agent-cost.py 那一行，应当 12 条开关都对上。
2. 研究脚本的自证格，确认 ⑭ 真起了作用。主 agent 用 `bash scripts/agents/watch.sh --processes` 盯着，跑：
   `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-tooling.sh 的 research-script-selftests 格; 范围=1 格; 为什么=核看门狗不再把开关轮故意判红的输出报成长跑判红、agent-watch.py 与 agent-cost.py 的自证在守卫底下照绿' bash .claude/gate.d/code-tooling.sh --check research-script-selftests /home/fy5090/code/singlefs`
   - 看门狗的报告里「判定行：」那一行应当写出「登记成轻的自证临时目录里有 ✗ 的 N 份不报」，不该再因为 `light-selftest-guard-*/tmp/…/switch-wheel-*.out` 叫醒。
   - 这条格起的是 light 自证，是不是由主 agent 跑、要不要带前缀，照 `.claude/skills/gate/staged-gate.md` 与那一格文件头定。
3. 门禁其余各格（break-switch-headers 判两份文件头开关表的写法、gate-lint 等）照提交前的整轮门禁跑，我没跑。
4. 命名检查：这一批没改 `.rs`，共用约束「命名检查交回前跑」那一条不适用，没跑。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）；没跑门禁阶段与各类 lint；没走定义三方；没提交。
- 没用 `scripts/gate/hook-swap.py`：它的文件头写明只收 `.claude/hooks/` 这一层的文件，`scripts/agents/` 下的不收。换成在草稿里先验、再定点换：
  1. 在草稿目录里搭一棵树，`.claude` 与 `scripts/hosts` 符号链接到主工作区，`scripts/agents` 里放四份的拷贝；
  2. 在拷贝上改完，跑 `py_compile` / `bash -n`，再拿探针喂该红的输入看它变绿；
  3. 用 `make_spec.py` 按「基线 → 草稿」的差出定点规格，规格施加到基线上之后核过与草稿逐字相同；
  4. 核过主工作区那四份与基线逐字相同，`replace-batch.py --dry-run` 之后才真换；
  5. 换上之后核主工作区与草稿逐字相同。
  草稿树里没跑改过的 `--selftest`：重型测试闸不放工作根副本里改过的脚本的自证。自证都在换上之后的主工作区里跑。
- 没改 `scripts/agents/watch.sh` 键表以外的部分，所以 watch.sh 自证 ① 不核新键能不能转过去；我只用 `--dry-run` 手核了一次（见上）。
- `.claude/hooks/selftest-registry.tsv` 里 agent-watch.py 那一行的理由，还写着「判定行那一格是两个开着样本日志的 sleep」。现在判定行一节另外起了三个 bash（其中一个经 timeout），仍然只睡、不起 cargo，档位不变。那份表不归我改，要不要更新理由交主 agent。
- 共用 TMPDIR 的自证用自己起名的 mktemp 目录写的 ✗，仍然照报（射程之外，见 ⑭ 一节）。
- 开关表里原有几条点名不全（比如 `memoryscan` 也红「内存包装里的进程应当照包装上限判」等），这是改前就有的，我只补了与新格有关的点名，别的没追。
- ㉒ 只做了脚本一侧。主 agent 定义里「每批收尾跑一次」那一句归兄弟工具员，我没碰。

## 草稿目录里有什么

`/home/fy5090/.local/share/singlefs/work/landing-audit-b9-watch-cost/`：

- 探针与驱动：`probe_watch.py`、`probe_cost.py`、`run_cells.py`、`debug_owner.py`、`make_spec.py`、`new_verdict_selftest.py`；
- 改前基线：`base/`；
- 规格与反向规格：`spec*.json`、`reverse-spec*.json`；
- 每一次跑的输出：`*.out`、`cost.diff`、`watch.diff`。

草稿树 `tree/`（680K，`.claude` 与 `scripts/hosts` 是符号链接）交回前已经删了。这里没有仓副本、编译目录或库拷贝。
