# 结局账：agent-cost.py 加 outcome 子命令

批次：records/2026-10-22-子agent结果落地审计.md#3。出口四条全部达成：子命令、汇总、自证（两个弄坏开关）、真会话目录跑一次并排对表。

## 改过的文件

- `scripts/agents/agent-cost.py`（加 outcome、`--repo`、`--summary`、`--jobs`；usage、timing、dispatch 的口径没动，只把用量求和抽成 `usage_totals()` 给 outcome 共用）
- `scripts/agents/README.md`（`agent-cost.py` 那一行说明）
- `.claude/hooks/selftest-registry.tsv`（agent-cost.py 那一行的理由改成「outcome 在临时目录用 git 造小仓和一棵工作树」，判定仍是 light）

`git diff --stat -- scripts/agents/agent-cost.py scripts/agents/README.md` 原样：

```
 scripts/agents/README.md     |    2 +-
 scripts/agents/agent-cost.py | 1021 +++++++++++++++++++++++++++++++++++++++++-
 2 files changed, 1002 insertions(+), 21 deletions(-)
```

登记表那一行（`git diff -- .claude/hooks/selftest-registry.tsv | grep agent-cost` 原样；表里别的改动是别的会话的）：

```
-scripts/agents/agent-cost.py	light	# 造几条会话记录在内存里算账，不起子进程
+scripts/agents/agent-cost.py	light	# 造几条会话记录在内存里算账；outcome 那一段在临时目录里用 git 造一个小仓和一棵工作树（git init、commit、worktree add），再按它算桶，不起 cargo、编译、虚机
```

`.claude/gate.d/code-tooling.sh` 的 research-script-selftests 格里 `python3 scripts/agents/agent-cost.py --selftest` 原来就在 runner 表里（第 321 行），没动。动手前 `gate-overlap.py --list` 看过：没有门禁或钩子管「子 agent 结局」，这是研究脚本加子命令，不新增门禁和钩子，所以没有 `gate-similar` / `hook-events` 行要写。

## outcome 做了什么

`agent-cost.py outcome [--root …] [--since …] [--repo 仓根] [--summary] [--jobs N]`：每个子 agent 一行（agent 号、类型、开始日期、交回否、截断原因、结局桶、折算花费（百万）、返工字样、描述），再汇总（按桶、并成调度记录那张表的五行、按类型 × 桶的个数与花费占比、返工字样派发的个数与花费占比、没交回的按截断原因计数）。成功句：`✓ 查了 N 个子 agent`；没有子 agent 退 77；git 出错退 1 并写出路。

两条漏判怎么修的：

- 引用：交回里的路径取到空白、引号、括号、全角标点为止，扩展名后直接粘的汉字说明去掉（`report.md（298 行）` 取到 `report.md`；目录名里的汉字如 `rot-verify-甲` 保留）。主会话与后来子 agent 的工具调用文本：同一条命令里赋过的变量先展开；路径段里带 `$变量`、`*`、`?`、`{a,b}` 的转成正则，对草稿目录名（取自派发提示的「草稿目录：」，加交回路径里工作根底下直接一层的目录名）；还认完整路径（含上层目录）、agent 号。目录名要带连字符、不短于 6 个字，字面字符不足 4 个的通配不用。共用同一个目录名的几个子 agent，按目录名点到的只算在下一个共用者开工之前。
- 写入落地：Edit / Write / MultiEdit 之外，认 Bash heredoc 写文件（`cat >`、`>>`、`tee`、`<<EOF > 文件`）、`replace-once.py` / `insert-row.py` / `replace-batch.py` 的实写（`--dry-run` 不算；规格取同一子 agent 前面 heredoc 写的，没有再读磁盘）、python 写文件。tool_result 报错的调用不算。比对集合：`git log -p -U0 --branches HEAD`（`--since` 往前多取 6 天）加的行 ∪ `git worktree list` 列出的每棵工作树里已跟踪和未跟踪未忽略的文件。
- python 写文件只取字符串字面量，那是「写了什么」的上限（模板、命令、说明也在里面）。实测不加这条处理，把 python 字面量当确定写入，会把 142 行脚本里的模板行算成没落地，落地率被拉低（agent aca0f3516 的 Edit 132 行全落地，加上 Bash 字面量 142 行后变 162/274）。所以 python 字面量只作加分：在仓里找得到才算进「写了」和「落地」，找不到不扣分。heredoc 写文件和三个定点脚本的新串是确定写入，照常算。

桶与阈值照 classify.py（写了不少于 10 行：不低于 70% 落地、30% 到 70%、更低按主 agent 点没点到分 overwritten / unused；三方腿类型 verdict-archived；没交回的按最后一句分 cut-by-usage-limit / cut-by-api-error / stopped-salvaged / stopped-abandoned；还在跑的 in-flight 改成「最后一条记录距此刻不到 30 分钟」，原来写死了一个日期）。

## 自证（`scripts/agents/agent-cost.py --selftest`，登记为 light）

18 个答案已知的子 agent，覆盖全部 13 个桶，造在假会话目录里；假仓用 git 造：一个提交后又删掉的文件（历史里才有）、留在提交里的文件、主工作树里没提交的文件、另挂工作树里没提交的文件。样本要点：

- 报错的 Write（10 行没有的）不算：编号 1 的 Edit 10 行在历史里，仍 written-landed。
- `--dry-run` 的 replace-batch 规格里另有 8 行没落地：不算，所以编号 4 仍 written-landed（算进去会掉到 partial）。
- 行首是全角空格的 3 行：走字节查不中、要解码去空白的慢路径，编号 5 因此 written-partial（慢路径关掉时掉到 written-unused，已手工验过会红）。
- 路径归一化三种：交回的平铺文件路径后粘全角括号与说明、主会话 `for i in …; do cat "$W/looped-draft-$i/…"`、`ls …/braced-draft-{b,c}/`；加只点到草稿目录名的、只被子 agent 点到的。
- 读会话记录走两个工作进程那一支（`workers=2`）。

绿（原样）：

```
  ✓ 自检通过（查了 11 项，其中 outcome 的 18 个子 agent）：调用按消息 id 去重、读缓存与两档缓存写分开、等待按 3 分钟间隔算、派发只数真正起来的、派发闸的拒绝按钩子归类、每个结局桶都对得上、路径归一化、Bash 里的写入认得出
```

弄坏开关（写在文件头，三项各只红点名那一项，退出码 1）：

| 开关 | 弄坏哪一处 | 红在哪一项（`✗ 自检：` 冒号前） |
|---|---|---|
| `AGENT_COST_BREAK=no-path-normalize` | 交回路径不在全角括号处截断；变量、通配、花括号、目录名一概不展开不对 | `路径归一化`（编号 9 平铺文件路径粘着全角括号、10 循环拼路径、11 花括号拼路径，三个桶都错；编号 6、12 有草稿目录或上层目录的键可退，这个开关下不红） |
| `AGENT_COST_BREAK=no-bash-writes` | Bash 里的 heredoc、python、三个定点脚本的写一概不认 | `Bash 写入`（编号 2、3、4 掉成 report-unreferenced） |
| `AGENT_COST_BREAK=no-dedupe`（原有） | 流式重复行各算一次 | `子 agent 用量` |

红的原样（截断到 230 字）：

```
== AGENT_COST_BREAK=no-path-normalize
  ✗ 自检：路径归一化：交回里粘着全角括号与说明文字的路径、主会话用循环与花括号拼的路径、只点到草稿目录名的，各自都要点得到（预期桶是 written-overwritten、report-us
== AGENT_COST_BREAK=no-bash-writes
  ✗ 自检：Bash 写入：heredoc、python、replace-once / insert-row / replace-batch 的实写都要算进落地（--dry-run 的不算），应得 written-landed，实际（预期, 实际）{'a0000000000000002': ('written-lan
```

语法检查：`python3 -m py_compile scripts/agents/agent-cost.py` 过（pyc 已删）。另跑过：`outcome --since 2030-01-01` 退 77、`--repo /nonexistent` 退 2、`--repo /tmp`（非 git 仓）退 1 带出路、`usage`、`timing`、`dispatch` 照常出表。

## 真会话目录上跑一次

命令：`python3 scripts/agents/agent-cost.py outcome --since 2026-10-01`（默认 12 个进程读，挂钟约 47 秒；错误输出 0 字节）。输出全文：`~/.local/share/singlefs/work/landing-audit-tw-outcome-ledger/run-1001.txt`（sha256 `387883d3d5ff0938e89bc25516d3f11cfec1b1c299c3e2f347d85baf06e6a847`，2268 个，表头 9 列，每个子 agent 一行）。汇总表原样：

```
## 按结局桶（2026-10-01 起，2268 个，折算合计 7617.31 百万）
桶                             个数     花费占比  说明
written-landed               887    49.8%  写了、不低于 70% 的行进了提交或工作树
written-partial               90     4.3%  写了、30% 到 70%
verdict-archived             473    27.5%  三方腿
report-used-by-main          661    15.3%  只交报告、交回后主 agent 点到
report-used-by-later-sub       7     0.1%  只交报告、只有后来的子 agent 点到
report-unreferenced            7     0.1%  只交报告、没人点到
written-overwritten           26     0.8%  写了、低于 30%、主 agent 点到过
written-unused                 0     0.0%  写了、低于 30%、没人点到
cut-by-usage-limit            36     0.9%  没交回、用量撞顶
cut-by-api-error               2     0.1%  没交回、API 错
stopped-salvaged              58     0.9%  没交回、被停掉、后来有人点到
stopped-abandoned             15     0.1%  没交回、被停掉、没人点到
in-flight                      6     0.2%  没交回、还在跑
## 描述带返工字样（修、重跑、续做、改名、改号）的派发：156 个，占个数 6.9%，占花费 5.5%
## 没交回的 117 个，按截断原因：用量撞顶 36、API 错 2、停掉 73、还在跑 6
  ✓ 查了 2268 个子 agent（2026-10-01 起），逐个归了结局桶
```

## 与调度记录「按结局分」那张表逐桶并排

旧账 = 调度记录那张表的口径（一次性脚本 classify.py 的输出 classified.json，2230 个）；新账 = 这次 outcome（2268 个，多出的 38 个是旧账之后开工的，旧账里的 2230 个新账里一个不少）；同批 = 新账里这 2230 个。「同批」列把新增的 38 个排除，是看修漏判效果的口径。`compare.txt` 是脚本原样输出。

| 桶 | 旧账 个数 / 花费 | 新账 个数 / 花费 | 同批 个数 / 花费 |
|---|---|---|---|
| written-landed | 663 / 42.0% | 887 / 49.8% | 878 / 50.2% |
| written-partial | 61 / 3.1% | 90 / 4.3% | 88 / 4.1% |
| verdict-archived | 473 / 28.1% | 473 / 27.5% | 473 / 28.0% |
| report-used-by-main（旧账含 report-dir-read-by-main 90 个 / 1.8%，那 90 个在调度记录里落在「其余」） | 671 / 17.5% | 661 / 15.3% | 649 / 15.1% |
| report-used-by-later-sub | 17 / 0.5% | 7 / 0.1% | 7 / 0.1% |
| report-unreferenced | 101 / 2.6% | 7 / 0.1% | 7 / 0.1% |
| written-overwritten | 40 / 2.4% | 26 / 0.8% | 23 / 0.7% |
| written-unused | 4 / 0.2% | 0 / 0.0% | 0 / 0.0% |
| cut-by-usage-limit | 36 / 0.9% | 36 / 0.9% | 36 / 0.9% |
| cut-by-api-error | 2 / 0.1% | 2 / 0.1% | 2 / 0.1% |
| stopped-salvaged | 23 / 0.5% | 58 / 0.9% | 53 / 0.6% |
| stopped-abandoned | 42 / 0.2% | 15 / 0.1% | 14 / 0.1% |
| in-flight | 7 / 0.2% | 6 / 0.2% | 0 / 0.0% |

调度记录那张表的五行：「看起来没效果的几桶」旧账约 240 个 / 约 7%（= unreferenced 101 + overwritten 40 + unused 4 + later-sub 17 + abandoned 42 + usage-limit 36，核过），新账 91 个 / 2.0%。

逐桶：对不上的原因，数来自 `transitions.txt`（按子 agent 对旧桶到新桶，原样脚本输出，同批 2230 个里 1825 个桶没变）：

- written-landed 新账比旧账 +224（同批 +215）：来自 used-by-main 166、overwritten 20、partial 13、unreferenced 18、later-sub 4、in-flight 3、unused 1，减去掉到 partial 的 10，新增的 38 个里有 9 个。这是修「写入落地」那条的效果：这些子 agent 的改动用 Bash heredoc、python、定点脚本写，旧账只看 Edit / Write，当成只交报告。
- written-partial +29（同批 +27）：同一原因，来自 used-by-main 19、overwritten 8、unreferenced 2、unused 1，再加落地里掉下来的 10，减去升到 landed 的 13。
- report-unreferenced 101 → 7：74 个改判 used-by-main（路径里带变量、通配、花括号的，粘着全角括号的，以及只点到草稿目录名的）；18 个改判 written-landed（Bash 写入）；3 个 later-sub；2 个 partial。这是修「交回之后的引用」那条的效果。no-effect.md 抽的 40 个里有 27 个是这类漏判，方向一致。
- report-used-by-main 671（旧账另有 dir-read 90）→ 661：旧账 761 里 201 个改走写入桶（166 + 19 + 9 + 4 + 3 个因写入或引用口径变了离开），再并入 unreferenced 74、later-sub 13、in-flight 2、新增 12 个。dir-read 那 90 个并进这一桶，因为新账按草稿目录名与路径一起认，分不出「只点到目录」。
- written-overwritten 40 → 26、written-unused 4 → 0：Bash 写入的行算进来后多数升到 landed / partial。
- stopped-abandoned 42 → 15，stopped-salvaged 23 → 58：28 个没交回的停掉的子 agent 因为按派发提示里的草稿目录名认出后来有人读过它的目录（旧账只看它最后一句里写的路径；这 28 个里抽 6 个看过，旧账的产物路径列表都是空的）。
- in-flight：旧账写死了一个日期，新账是「最后一条记录距此刻不到 30 分钟」，随跑的时刻变；这一次此刻别的会话还有 6 个在跑。
- cut-by-usage-limit 36、cut-by-api-error 2、verdict-archived 473：没变。

## 对不上之处（要主 agent 知道）

- 返工字样：调度记录写「描述带修、重跑、续做、改名、改号的派发 337 个，占花费 13.8%」。按这五个词只看 meta 里的描述，旧账 2230 个里是 155 个、5.6% 花费（核过），新账是 156 个、占个数 6.9%、占花费 5.5%。337 复现不出来：描述里再加 resume / fix / rerun / redo（英文描述）是 295 个 / 11.5%，仍不到 337；描述加提示全文是 988 个。我按这一轮派发写的字做（五个词、只看描述），没去凑 337。要换口径给我词表。
- 落地率「偏高 1 到 4 个点」那条抽查结论是旧口径的：新口径多了 Bash 写入，landed 桶变大，partial 也变大（写得多、被改写得多的子 agent 现在看得见了）。没拿新口径再抽样核真实性。

## 局限与推翻条件

- python 写文件只取字符串字面量，且字面量只作加分（见上），所以靠 python 生成内容的子 agent，落地率只会往高里偏、不会误扣。先 heredoc 写出脚本再 `python3 脚本` 执行的，只认同一个子 agent 里写出的那份；在别处写好的脚本认不出。
- Bash 目标路径写在变量里、算不出的，不记（`repository_relative` 看不出仓内）；相对路径一律当仓内相对路径认。
- 目录名当键要求带连字符、不短于 6 个字：不带连字符的草稿目录名不用作键（只有完整路径还认）。
- git 历史取所有分支的 `-U0` 加行；「进过仓」不等于「现在还在」，这与旧口径一样（writers.md 已写明）。一行字在仓里别处恰好有同样的行也算找到（照抄的常见写法），与旧口径一样。
- 推翻条件：随便抽一个新账归 written-landed、旧账归 used-by-main 的子 agent，如果它的 Bash 写入行在 `git log -S` 和各工作树里都找不到，说明 heredoc / 定点脚本的识别过宽；抽一个 report-unreferenced → used-by-main 的，如果主会话那条命令其实和它无关（目录名撞了别的东西），说明目录名当键过宽。这两类我只查了转移表和几个例子（上面 sweep-pronoun-time、E175、E177、E178、E185、rot-verify 的丢失已核到并修掉），没做逐个抽样核。
- 读 11 GB 会话记录用 12 个进程，约 47 秒；`--jobs 1` 退回本进程顺序读（只验过 `--since 2026-10-22`，124 个）。

## 没做什么

- 没跑门禁阶段、各类 lint、重型测试；没走定义三方；没提交；没动 `crates/`、kb、别的 `scripts/agents/` 下的脚本。
- 没把真跑的 2268 行账存进 `research/results/`：它随此刻的会话变（in-flight、新子 agent），是草稿。位置与来历：`~/.local/share/singlefs/work/landing-audit-tw-outcome-ledger/run-1001.txt`（`--since 2026-10-01` 的完整输出，sha256 见上）、`compare.txt`、`transitions.txt`、`new-agents.json`；辅助脚本 `transitions.py`、`compare_tables.py`、`lost_refs.py`、`debug_lines.py`、`compare.py`、`rename-spec.json` 都在同目录，不入库。`agent-cost.orig.py` 是改前的脚本副本。
- 没要主 agent 跑的命令：本脚本的自证登记为 light，已在本地跑到绿、三个弄坏开关逐个红在点名的项；真跑也在上面跑过。
