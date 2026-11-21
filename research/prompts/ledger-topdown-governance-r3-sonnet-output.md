# 辩方腿：治理文件轮第三轮第 8 到 11 问（ledger-topdown-governance-r3，2026-11-18）

草稿目录 D = `/home/fy5090/.local/share/singlefs/work/governance-r3-defense`（任务日志全文拷在 `D/log-<号>.txt`，任务状态 json 摘在 `D/js/`、`D/swaps.tsv`、`D/status-range.tsv`）。没有仓副本、没有编译目录，没跑任何项目代码（只读 `task-server/task status <号> --json` 与 `log <号> --all`、`grep`、`jq`、`awk`、`diff -rq`、`git show`）。

## 结论一览

| 问 | 回答 |
|---|---|
| 第 8 问：第二轮坐实的 10 条 | 理由都成立，回「判决站得住」；第 5 条里一句转述不准（下面写），结论不动；第 4 条判决当时看得出「副本带 `.git`」，判决缺的是判据（第 8 问第 4 条） |
| 第 9 问：记账的 5 条 | 都回「判决站得住」；换上工具没有锁那条，触发条件对锁够，但「窗口相交」那一样没有人例行数，补一条数法 |
| 第 10 问：库认一族、服务认一个 | 工具员做成的现状以它为准；判决第三节「整词相等」那一句要更正，原因是判决对「宽」的转述不准 |
| 第 11 问：各批修补对不对得上「三、改法」 | 做偏 2 处（规格工具的判据、注释历史的判法形态）、多做 3 处、少做 2 处；证红任务号逐条列了，规格工具那一行只有证红、没有翻绿 |

## 我读到的范围

- 判决 `research/prompts/ledger-topdown-governance-r2-main-verification.md` 全文，正文，五份工具员报告（`governance-r2-fix-guards`、`governance-r2-fix-guards-second`、`governance-r2-fix-swap-tools`、`governance-r2-fix-not-run-marker`、`governance-r2-fix-comment-history` 下的 `result.md`），第二轮两条腿的报告（`research/prompts/ledger-topdown-governance-r2-opus-output.md`、`ledger-topdown-governance-r2-sonnet-output.md`）。
- 范围里文件换上之后的版本，读了的段：`.claude/hooks/bash-command-detector.sh` 文件头与开关表（第 300 到 320、1053 到 1067、1415 行一带）、`.claude/hooks/write-guard.sh`（第 88、760 到 779 行）、`scripts/gate/hook-swap.py`（文件头结局文件一段、第 1944、2065 到 2090、2226 到 2262 行）、`scripts/gate/make-swap-spec.py`（第 18、265 到 300、498 行）、`task-server/task_server.py`（第 253 到 262、895 到 904、1135 行）、`task-server/task_server_lib.py` 第 1803 行、`scripts/gate/lib_libtest_result_lines.py`（第 13、115 到 128、599 到 607 行）、`scripts/gate/comment-history-check.py`（第 38 到 43、128、176 行）、`.claude/gate.d/doc-text.sh` 与 `doc-process-records.sh`（grep 与第 610 到 617 行）、`.claude/agents/implementation-writer.md` 第 47、48 行、`tooling-writer.md`、`.claude/agent-common.md` 第 42、89、101 行、`.claude/gate.d/break-switch-replay-files.tsv`（第 61、75、76、91 行）。
- 没读：检出钩子第十四项的判法实现（几百行内嵌的 python）、换上工具 `SearchPaths` 的实现、`task_server_selftest.py` 与三语 README 的正文；这几处只按工具员报告与任务读数判，写在各条里「没读代码」。

## 量的读数（这一轮我自己取的）

- 任务 20153 到 20357 里真换上的 `scripts/gate/hook-swap.py`（命令不带 `--dry-run`、`--check-callers`、`--selftest`、`--feed-only`、`--read-outcome`、`--seal-outcome`），`task-server/task status <号> --json` 取，存 `D/swaps.tsv`：10 趟（20211、20242、20244、20267、20291、20292、20330、20333、20346、20349），两个会话（本会话 8 趟、做门禁的会话 2 趟）；任务窗口（交上去到结束）两两相交 0 对（awk 逐对比，原句：`真换 10 趟，相交 0 对`）。换会话的相邻两趟，前一趟结束到后一趟交上去：20211 到 20242 为 519 秒，20333 到 20346 为 1185 秒，20346 到 20349 为 170 秒（`D/swaps.tsv` 里时间戳相减）。
- 任务 20240 到 20357 的状态摘要存 `D/status-range.tsv`（117 行，本会话 88 行），下面各条的任务号、退出码都从它与 `D/log-<号>.txt` 取。

## 第 8 问：第二轮坐实的 10 条（判决第二节第 1 到 10 条，`research/prompts/ledger-topdown-governance-r2-main-verification.md:52` 起）

每条回「判决站得住」「判决够不着」「推不出」，再写什么现象会推翻它。

| 判决条 | 判 | 依据（任务日志的原句来自 `D/log-<号>.txt`） | 什么现象会推翻 |
|---|---|---|---|
| 1 检出钩子第十四项漏放十三种 | 判决站得住 | 任务 20176 末句「喂了 20 条，13 条不对」，13 条 `✗ 退 0（应当 2）` 逐条是判决列的那十三种（`cp -b`、`install -b`、`mv --backup=numbered`、`cp -r 草稿/src/.`、`rsync -a`、`cp -r 草稿/src prune-server/`、`ln -sf`、建硬链接、往硬链接追加、`tar -x`、`git apply`、`patch -p1`、`rm`），7 条对照 `✓ 退 2`。工具员的新旧对照复现同一个数：旧版 20310「喂了 23 条，12 条不对」加 20323「喂了 4 条，1 条不对」（往硬链接追加那一条单放），合起来恰好 13；新版 20311、20324 都 0 条不对 | 旧版吃同一份喂法时某条退 2。20310、20323 已经证明没有 |
| 2 写闸对门禁阶段的出路写错 | 判决站得住 | 任务 20178 退 2，说明里有「加 --verify '<被换脚本的 --selftest 命令>' 换上」，对象是 `.claude/gate.d/broad-mutation.sh`；对照 20179 对登记成 required 的 `scripts/mutation/prove-red.sh` 写「不加 --verify 换上」。理由「门禁阶段不在要临时服务的登记表里」现查成立：`.claude/hooks/selftest-temporary-service.tsv` 里 `broad-mutation` 只命中第 56 行的 `scripts/mutation/broad-mutation.py`，以 `.claude/gate.d/` 起头的行 0 行（`grep -c`，量的）。旧版 20326「喂了 8 条，8 处不对」 | `.claude/gate.d/` 顶层某个 `.sh` 出现在那张表里并登记成 required（今天 0 行） |
| 3 换上工具把同名不同目录的模块算进来、两种取法漏核 | 判决站得住 | 20180 退 1，点名 `scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py:49` 引了 `run_directory.device_stream_summaries`，而被核的是 `e171_three_oracle_collusion_after_the_oracle_fixes/run_directory.py`；`comparator.py` 引的是同目录自己的那份：`scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py:49` 一行是「from run_directory import (STREAM_NAMES, device_stream_summaries, device_verdict_line, load_run_directory, parse_result_line,」。20182、20183 对 `sys.modules.get`、`from sys import modules` 都退 0「缺 0 处，本次未判 0 处」。真仓现查（量的）：「client = sys.modules.get("singlefs_prune_server_client")」（`scripts/crash/crash-flow-coverage.py:898`）在，判决点的位置对 | 同名模块的调用方换了 `run_directory.py` 之后 20180 仍该退 0；今天 20271 就是退 0，没有推翻 |
| 4 规格工具在仓副本里起退 3、出路相反 | 判决站得住（理由）；判据缺口见下 | 20190 退 3，出路写「在多机配置（主工作树根的 multi-host.env）里写 SHARED_DATA_ROOT」，对照 20191 在主工作区起退 0「写出 …/spec-from-main.json」。副本是 20189 用 `minimal-copy.py` 建的。`scripts/hosts/minimal-copy.py:35` 的约定是副本里的多机配置不拷、经 `SINGLEFS_MULTI_HOST_CONFIG` 交，所以叫人往副本里写配置确与约定相反 | 在副本里起且交了变量时能写出规格（20190 没有） |
| 5 任务服务认的记号行比门禁那一份库少四种 | 判决站得住；其中一句转述不准 | 20186 服务那一份（调度记录第 348 行写重启前同一份样本只认 3 条），20187 库那一份 `markers=8`；样本 20 行与攻方腿那份逐字相同（`sha256sum` 两边都是 `9bf70eef93033465d6244e31a47a6ed4d3a5b4ec32c48b9e6a243619f9853c9f`，量的）。判决第 57 行写「门禁那一份库把 `NOT_RUN_ANYTHING_HERE` 这种更长的词也认成记号」不准：样本里那一行是「样本日志摘录：CANNOT_RUN_ANYTHING_HERE 不是记号」（`scripts/gate/lib_libtest_result_lines.py:128`），库认出来的是它里面的后半截，`marker` 行打出的是匹配到的那一截 `NOT_RUN_ANYTHING_HERE 不是记号`（20187）；攻方腿当时写的是「门禁库的正则前面没有词界，更长的词的后半截也算记号」（`research/prompts/ledger-topdown-governance-r2-opus-output.md:138`），缺的是左词界，不是名字一族。判决把两件事并成一句 | 样本里那一行真是整词 `NOT_RUN_ANYTHING_HERE`。样本文件里不是 |
| 6 注释历史判法把讲现状的句子判红 | 判决站得住 | 20188「4 行命中」，第 2 行「以前面那一份为准」、第 3 行「可以前面带 ./」、第 6 行「可以前置一段命名空间」三句都不讲过去，第 5 行「以前这里只认 --bin」是对照；换上前 20240 同形「4 行命中」，换上后 20306「1 行命中」 | 三句里有一句改成讲过去；没有 |
| 7 两行特殊缘由标记该改写 | 判决站得住 | 共用约束的特殊缘由是三类：`.claude/agent-common.md:42`「特殊缘由只三类」，三类的头一类是日期本身是数据。被撤的两行（工具员备份 `governance-r2-fix-comment-history/originals/.claude/gate.d/doc-text.sh` 第 659、978 行）写的理由是「这一行引的是判法要判红的样本里的词」「这一行逐字引的是红样本里要判红的两句原文」，引的是词不是日期，三类都不是。反方最强的读法（「解析历史条目的代码讲它认的形态」那一括号盖得住）压不过：那一括号挂在「日期本身是数据」下面 | 共用约束第 42 行的三类加上「样本里要判红的词」一类；今天没有 |
| 8 拿退出码当「哪一道拒的」的同形错 | 判决站得住 | 20193 三条成对读数：主 agent Edit `prune-server/src/protocol.rs`、旧串以空格收尾退 2，不以空白收尾退 0，工具员不以空白收尾退 2，只有第三条分得出是第十一道拒；判决第一节表里「攻 3」对照行的退 2 分不出是哪一道，说法成立 | 20193 第三条退 0；它退 2 |
| 9 实现员定义第 47 行与证红脚本的取法 | 判决站得住 | `git show HEAD:.claude/agents/implementation-writer.md` 第 47 行有「同在这一份 `scripts/` 里」，把证红脚本说成在副本的 `scripts/` 里；证红脚本取法现查：`scripts/mutation/prove-red.sh:287`、`:295` 载入脚本同目录的判法脚本，`:333`、`:706` 读副本的 `crates/mutations.tsv`，`:1023` 的 `MAIN_ROOT` 取主工作区，定义第 47 行现写「证红脚本起的是主工作区那一份」（`.claude/agents/implementation-writer.md:47`）。行键对不上那一形判决已写只有形状（核不动的第 1 条） | 证红脚本读副本里的 `scripts/` |
| 10 少做的两处（`doc-process-records.sh` 另三行、两种取法） | 判决站得住 | `git show HEAD:.claude/gate.d/doc-process-records.sh` 第 610、612、617 行都有「现行」当样本里 current 那一份的叫法（原文「账本登记表：现行 2026-10-29」「写了「拆分提案」的只有正文的现行账本」……）；两种取法即第 3 条，今天 20273、20274 都退 1 | 三行没有「现行」的叫法；判决写的是「按线索记」，我现查在 |

### 第 8 问第 4 条：工具员选的判据没生效，判决当时该不该看得出「副本带 `.git`」

判：**看得出**；判决够得着第 4 条的缺陷，缺的是判据一句。四处现查的依据：

1. 判决自己引的任务 20189（建副本）的日志末行是「MINIMAL_COPY git: …/q4/copy：在副本根 git init 了（没有提交、索引是空的）」（`D/log-20189.txt`）。判决第一节表里「攻 14、15」那一行点了 20189、20190（`research/prompts/ledger-topdown-governance-r2-main-verification.md:40`）。
2. 攻方腿那一问的报告就写着「副本是 git init 的新仓」（`research/prompts/ledger-topdown-governance-r2-opus-output.md:111`），判决表第一行把它列为被核的报告。
3. `scripts/hosts/minimal-copy.py` 文件头写着「副本是 git 工作树」（`scripts/hosts/minimal-copy.py:28`），下一句说明不是的在副本根 git init。
4. 判决给工具员的是两个方案由它先判；落地的那一支（副本里起一律拒）没写判据。工具员拿「仓根下没有 `.git`」当判据：`scripts/gate/make-swap-spec.py:281` 的「not os.path.lexists(os.path.join(repository_root, ".git"))」，自证的小仓也照这个想法造：「git 为真时放一个空的 .git 目录（工作树的样子；仓副本没有它）」（`scripts/gate/make-swap-spec.py:498`）。样本是按它的假设造的，不是 `minimal-copy.py` 建出来的树，所以自证格全绿而真副本不走那一支。判决写的零轮样本是「任务 20190 那一形」，那一形的副本是 `minimal-copy.py` 建的；链里这一格落空了两回：20264、20279 都是「python3: can't open file …/q4-copy/scripts/gate/make-swap-spec.py」（建副本命令没带脚本，退 2，不是规格工具的结局），第三回 20308 建对了副本、20309 才读到退 3 仍是旧出路（调度记录第 359 行「照共用约束建的副本都带 `.git`」）。

没有哪一处是判决看不见的。判决该补的一句是「判据必须用 `minimal-copy.py` 建出来的树验」，它写了样本形状而没写判据；这是判决里的缺口，不是理由不成立。推翻「看得出」的现象：20189 的日志里没有 `git init` 一行（今天有）。

关于「有没有哪种摆法能让它写出规格」这一问归攻方腿，我只给一个读数：判据只看 `.git` 在不在，所以在副本根手建 `.git` 目录就绕得过，工具员报告自己写了这一点（`governance-r2-fix-swap-tools/result.md` 第五节），也就是说这一支不是边界。

## 第 9 问：记账的 5 条（判决第二节第 11 到 15 条）

每条先写替「该在这一轮改」那一方的最强理由，再判压不压得过。

### 第 11 条：换上工具没有锁

最强理由（替「该在这一轮改」）：（一）它是全仓脚本与钩子唯一的换上路径，退 2 的钩子一坏每条 Bash 都被拒；（二）这一轮的换上链里两个会话又靠发消息对了三回时间（调度记录第 327、343、364 行），说明错开靠人不靠机制，第二轮的触发条件一样没有触发，却已经出现三回需要人工协调；（三）触发条件都是事后的：窗口相交要有人去数，退 9 要撞上才有，撞上那一趟换上可能已经改了文件；（四）同类的打补丁工具有现成写法：「同一个仓同一时刻只许一个在换」（`scripts/gate/apply-writer-patch.py:40`）。

压不过，理由四条：

1. 这一轮现数（量的，`D/swaps.tsv`）：真换 10 趟、两个会话、窗口两两相交 0 对，换会话的相邻两趟最短隔 170 秒，比单趟最长 114 秒（20333）还大。第二轮数的是 67 趟、0 对、最短 8 秒；这一轮没有比那更挤的读数。
2. 换上工具不是没有任何拦：结局文件放在 git-dir 下，「root 是 git 工作树的根就放它的 git-dir 下」（`scripts/gate/hook-swap.py:1944`），不管用哪个草稿目录，下一趟都读得到；起头那趟还在，下一趟退 9：「那一趟还在换或在验，退 9」（`scripts/gate/hook-swap.py:60`）。互盖缺的只是读结局文件到写开始行之间那一小段互斥。换之前对每份文件再核一遍基线：「草稿验过之后主工作区这几份被改了」（`scripts/gate/hook-swap.py:2234`）退 3、一个字节不写。第二轮开的缺口（两趟同时过检查再各写）要两趟在毫秒内交上才成立，调度记录里的三回对时间都隔了几分钟以上。
3. 那三回协调要拦的不是两趟换上互盖：第 343 行写的是对方在跑一份门禁的二十条开关、我换上落在 `.claude/gate.d` 里的东西「只会让它那一趟报键变了、不改判对判错」（`records/2026-11-17-subagent账本清欠调度记录.md:343`），第 364 行是「它重跑的那一条开关出结局之前我不换这一份」（`records/2026-11-17-subagent账本清欠调度记录.md:364`）：换上与别人的读之间的先后，装在换上工具里的锁管不到。要管它得是另一种东西（读者那一侧的钉住），不是这一条记账的内容。
4. 退 9 在任务服务那一侧已经有提醒：`task-server/task_server_lib.py:1803` 一行以「previous-swap-running｜提醒照跑｜换钩子的命令」起头。所以判决写的三样触发条件里，「退 9」与「任务服务推上一趟还在换的提醒」是同一件事的两个出口，实际是两样：退 9（含提醒）与窗口相交。

判：**判决站得住**，触发条件对「锁」够。要补一句：「窗口相交」这一样没有人例行去数，建议登记时把数法写进去，例如我这一轮用的：`task-server/task status <号> --json` 取 `command`、`session`、`submitted_at`、`finished_at`，筛命令第二个词以 `scripts/gate/hook-swap.py` 结尾、且不带六个非换上选项的，逐对比窗口。三回对时间不算触发条件满足：它们不是两趟换上相交，也不是退 9。推翻：下一次真换上里有一对窗口相交，或任意一趟退 9。

### 第 12 条：第一轮判决第 4 条的标题说宽了

最强理由：标题写错会被后来的人当事实引。压不过：第一轮判决是冻结证据，不回改；更正已写在这一轮的判决里（`research/prompts/ledger-topdown-governance-r2-main-verification.md:67`），改法没动。判：**判决站得住**。推翻：有人引用第一轮标题当结论而没看到更正；没找到。

### 第 13 条：第一轮记账的另几条

最强理由：账本侧四处过时句（判决写的第 20、27、186、498 行）会误导：我现看第 498 行仍是「去向：① 做完……④⑤⑥ 还欠」，而第 499 行的「去向（续）」已写四、五、六都换上，同一节里两句相反。压不过：账本的去向是追加写法，「（续）」行就是更正的位置，旧行是当时的事实；改旧行反倒违背「原样保存的证据不许事后改」。第 17 条的前提（两台上限相同）我没重核，那一值在被忽略的私有配置里，这里不读。判：**判决站得住**，账本侧是否改去向行由主 agent 判，我没核它改没改。推翻：第 498 行与第 499 行的矛盾被后来的节引用成现状。

### 第 14 条：工具员写明没做的几件

最强理由：`mv` 挪走、`--nocapture` 提一句、认不出类型的子 agent 都是读得到的缺口，都该这一轮顺手补。压不过：（一）`mv` 挪走判决记账，工具员这一轮一起做了（见第 11 问「多做」），所以这一件已无账可记；（二）`--nocapture` 那一句：服务文件头已写明「测试不带 --nocapture 时，通过的测试打的字被测试框架收走」（`task-server/task_server.py:257`），限制是写明的，要改的是「没带就提一句」这个新行为，设计没定（`research/prompts/_ledger-topdown-governance-r3-body.md:17`）；（三）两个服务的构建清单规格不排除：换上工具改名换新 inode、没有改到一半，这一轮没有新读数。判：**判决站得住**。推翻：`--nocapture` 缺失造成一次被记绿的「本次未跑」；今天没有。

### 第 15 条：证红命令缩域那一形没有任务跑过

最强理由：定义第 48 行把证红命令改成了「从主工作区的仓根起主工作区那一份证红脚本、`--copy` 指到副本」（`.claude/agents/implementation-writer.md:48`），缩域那一形（命令前加 `env <缩域变量>=<n>`）是新写的，定义改了而没量，下一次写崩溃枚举用例的实现员头一个撞上，该这一轮先量。压不过：这一轮没有改 `crates/`（`research/prompts/_ledger-topdown-governance-r3-body.md:14`），没有缩域证红可量；换上窗口内唯一跑过的证红是任务 20234，命令是 `bash scripts/mutation/prove-red.sh --copy <副本> singlefs-background-gpu-tests …`，工作目录是仓根（`task-server/task status 20234 --json` 的 `cwd`，量的），退 0；它证了「仓根起、`--copy` 指副本」这个底形，没带 `env`，所以缩域形仍没量。判：**判决站得住**。推翻：某次带 `env <缩域变量>=<n>` 的证红退 2、或证红脚本把副本认成主工作区。

补一句「推的」：第 11 条里「两趟要在毫秒内交上才互盖」是我读 `scripts/gate/hook-swap.py` 的检查次序推的，没造两趟并发的假驱动样本；第一轮判决要求改之前先造两趟并发的假驱动样本证红（第二轮辩方腿报告转述了它，`research/prompts/ledger-topdown-governance-r2-sonnet-output.md:69`；第一轮判决不在工作区，我没读原文），这一轮记账期间没有谁造。

## 第 10 问：不采的 1 条与核不动的 2 条；「库认一族、服务认一个」与判决第三节

### 不采的那条（判决第 16 条）

判决的原话是「攻方腿提的把记号认法整个搬成门禁那一份库的写法」不采（`research/prompts/ledger-topdown-governance-r2-main-verification.md:74`），改法那一行写的是「记号前后以空白或行界为界、整词相等才算，行内任一处都认」（`research/prompts/ledger-topdown-governance-r2-main-verification.md:93`）。「不采」站得住：搬库那时的写法，等于把没有左词界的宽带进服务；两边现在都有左词界与右词界，宽已经去了。

### 以哪个为准

结论：**以工具员做成的现状为准（库认 `NOT_RUN_` 一族、服务只认 `NOT_RUN_NO_GPU_ADAPTER`）；判决第三节「整词相等」那一句要更正**，更正后的写法是「前后词界两边相同；各自认的名字集各管各的」。理由：

1. 判决给的样本句两边都满足：「两边都该认 7 行、不认更长的那一行」。库吃那份样本：任务 20255 末行「RECORDABLE binaries=1 green=0 withheld=7 markers=7」，没有 `ANYTHING_HERE` 行（`grep -c` 0，量的）；服务重启后同一份样本认 7 条（调度记录第 356 行；那一句是依据文字，不在任务日志里，我没有第二处可核）。换上前库是 `markers=8`（20187）。
2. 「更长的那一行」是 `CANNOT_RUN_ANYTHING_HERE`，被去掉的是左边缺词界，不是名字一族：见第 8 问第 5 条；判决把「后半截」转述成「更长的词」。样本文件里「更长的词」只有这一行（`scripts/gate/lib_libtest_result_lines.py:128`），两边的自证都拿它当不认的例子。
3. 把「整词相等」按字面读成名字相等，库就得砍成单个名字。库的约定是一族：「NOT_RUN_<原因> test=<用例> reason=<一句>」（`.claude/gate.d/crash-point-replay.sh:349`），库现在的写法是「NOT_RUN_MARKER_FORM = re.compile(r"(?<![A-Za-z0-9_])NOT_RUN_[A-Z][A-Z0-9_]*(?!\S)", re.ASCII)」（`scripts/gate/lib_libtest_result_lines.py:115`）。砍成单名，别的原因名（工具员报告写的门禁样本里有 `NOT_RUN_GPU_FEATURE_NOT_COMPILED_IN`，转述）会被当成没打记号，那一条用例被记绿；多扣的方向安全，记绿的方向不安全。这条是工具员的判断，我没有另造样本验。
4. 服务只认一个名字有它的用处：服务的提醒讲的是 GPU 取不到卡，写明了与库的差：「那一份认 NOT_RUN_ 一族的名字，这里只认 GPU 这一个」（`task-server/task_server.py:256`）。`crates/` 下往 libtest 输出里打的 `NOT_RUN_` 名字只有这一个：`grep -rnoE 'NOT_RUN_[A-Z_]+' crates --include=*.rs` 去重后，别的命中是 Rust 常量名（`NOT_RUN_EXIT_CODE`、`NOT_RUN_HERE`、`NOT_RUN_LINE`、`NOT_RUN_LINE_PREFIX`、`NOT_RUN_MARKER`，量的），不是打出去的字。

让两边结局不同的行（给得出）：`NOT_RUN_NO_GPU_ADAPTER_X test=b`。我用 `grep -cP` 分别套两边的正则（库的 `(?<![A-Za-z0-9_])NOT_RUN_[A-Z][A-Z0-9_]*(?!\S)`、服务的 `(?<![A-Za-z0-9_])NOT_RUN_NO_GPU_ADAPTER(?!\S)`）喂三行样本（`x NOT_RUN_ANYTHING_HERE test=a`、`x CANNOT_RUN_ANYTHING_HERE test=a`、`NOT_RUN_NO_GPU_ADAPTER_X test=b`）：库那一份命中 2 行（第一与第三行），服务那一份命中 0 行（量的；这是正则的对照，不是两边程序的输出）。哪一边「错」今天定不了：`crates/` 没有打这一行的代码；库认它是扣，服务不认是不提醒。残留的差登账：触发条件是 `crates/` 里出现第二个往 libtest 输出里打的 `NOT_RUN_` 名字，那时服务要不要认一族再判。推翻上面结论的现象：门禁里有某一道靠「记号名等于 `NOT_RUN_NO_GPU_ADAPTER`」才对的判定被一族放宽判错；工具员报告写它现搜没有（`governance-r2-fix-not-run-marker/result.md` 第四节），我没有独立搜。

### 核不动的 2 条

1. 行键对不上那一形：只有形状，我没有读数，仍核不动。
2. 辩方腿抽查的任务号：这一轮我把判决第一节表里 20176 到 20193 的日志都读了（`D/log-20176.txt` 到 `D/log-20193.txt`），与判决表里的读数逐条相符（20176「喂了 20 条，13 条不对」、20177 两行、20178、20179、20180 的 2 处缺、20182 与 20183 的退 0、20185、20186、20187 `markers=8`、20188「4 行命中」、20190 退 3、20191 退 0、20193 三条 `✓`）；20181 与 20189 日志在，没有不符。这一条可以关。

## 第 11 问：各批修补是不是照第二轮判决「三、改法」做的

对的是判决第三节那张表（`research/prompts/ledger-topdown-governance-r2-main-verification.md:87` 起，八行）。任务号都取自 `D/status-range.tsv` 与 `D/log-<号>.txt`，换上前后的读数我逐条核了。「证红」指换上之前旧版吃样本红，「翻绿」指换上之后同一样本绿。

| 改法行 | 证红的任务号（换上前） | 换上 | 翻绿的任务号 | 对不对得上 |
|---|---|---|---|---|
| 1 检出钩子第十四项（判决第 89 行） | 20310「喂了 23 条，12 条不对」、20323「喂了 4 条，1 条不对」，合 13 条；20312、20313 无效（草稿小树缺钩子读的 `lib-forbidden-notations.py`，调度记录第 360 行） | 20331 演练、20333 真换 | 20311、20324、20325（先前九条）都 0 条不对；换上后 20343、20344 同；自证 20335 | 做到。多做一处：`mv` 挪走也判了（判决第 69 行把它列在不改的记账里） |
| 2 写闸门禁阶段出路（第 90 行） | 20326「喂了 8 条，8 处不对」 | 20329 演练、20330 真换 | 20327、20345 都 0 处不对；自证 20334；开关复跑 20337 | 做到。轻微收窄：门禁阶段限定为 `.claude/gate.d/` 顶层的 `.sh`，不含这一层 `lib-` 起头的 `.py`，写在「门禁阶段（GATE_STAGE_DIRECTORY 这一层的 .sh，不含 lib/ 与 fixtures/ 下的、也不含这一层 lib- 起头的 .py 共用库）单走一支」（`.claude/hooks/write-guard.sh:88`）；判决写的是「顶层的脚本」；`tooling-writer.md` 同步补了（判决第 117 行要求） |
| 3 换上工具认库与两种取法（第 91 行） | 20261（e171，退 1）、20262（同名模块探针，退 1）；`sys.modules.get`、`from sys import modules` 两棵的换上前读数是第二轮 20182、20183（退 0），探针树与链里用的 `diff -rq` 三棵都无输出，逐字相同（量的） | 20266 演练、20267 真换 | 20271 退 0、20272 退 0、20273 与 20274 退 1 并点名缺的名字；底 20259、20260 到 20268「691 处引用，缺 0 处，本次未判 4 处」、20269「522 处引用，缺 0 处，本次未判 11 处」 | 做到，多做一处：按路径装的（`importlib` 一类）也折。本次未判从 10 涨到 11、e171 那条 7 涨到 14（20261 到 20271 的「?」行数，量的），判决允许解不出的列本次未判，没越界 |
| 4 规格工具（第 92 行） | 第二轮 20190（退 3 旧句）；链里 20264 无效（「python3: can't open file …/q4-copy/scripts/gate/make-swap-spec.py」） | 随 20267 | **没有翻绿**：20279 又是同一句无效；20308 建对副本后 20309 退 3，出路仍是「在多机配置（主工作树根的 multi-host.env）里写 SHARED_DATA_ROOT」 | 做偏。判决的先判要求做了（工具员报告第五节判甲保不住、做乙），但判据选了「没有 `.git`」，真副本带 `.git`（第 8 问第 4 条）；判决第二个分句「取不到时的出路改成相符的说法」在真副本里没有落地。工具员自证 20282「15 条开关逐格相同」是按它的假设造的假副本，所以绿 |
| 5 记号行同一个认法（第 93 行） | 第二轮 20186（服务 3 条）、20187（库 `markers=8`），样本与攻方腿那份同一个 sha256；链里换上前没有再跑 | 20242 库、20244 服务（先库后服务，次序对） | 库：20255「markers=7」；服务：20254 是旧进程仍 3 条，重启后 20307 认 7 条（调度记录第 356 行）；自证 20245、20253；开关 20246、20247、20249 到 20252 | 做到，轻微偏：库认一族、服务认一个，与「整词相等」的字面不同，第 10 问判以现状为准。样本两边共用一份（服务自证按路径导入库里那份），与「各配同一份样本日志」一致 |
| 6 注释历史判法（第 94 行） | 20240（探针「4 行命中」）；样本档 20216、20224「2 个用例判错」（换上前旧版，调度记录第 347 行写明它们是误在旧版上跑的，仍是有效的换上前读数） | 20291、20292 | 20306「1 行命中」、20297「106 个用例」都对、20294 自证、20295 开关复跑 19 条；真仓 20217、20223（换上前）与 20296（换上后）都是 872 行 | 做偏。判决要的是「以前」作时间词的正面形态（前面是句首、标点或空白，或后面接「是」「的」「只」「曾」「这」）、别的不判；做成的是反面例外（`scripts/gate/comment-history-check.py:176` 的「以前(?![缀者置]|面[那这的带])」加「除乘所可」），文件头写明三处已知漏判。改前该在 872 行命中上量新旧两判，工具员量的是含「以前」的 57 行、新旧各 36 行（转述，用 `grep -P`，不是门禁判法）；872 的对照是后补的 20217、20223、20296。结果不比判决的写法差：正面形态会漏判更多（推的，没造样本比） |
| 7 `doc-text.sh` 两行（第 95 行） | 量过的是现状，没有样本 | 20292 | 20298 之外，样本档 20297 绿；`history-kept` 行数 12 减到 10（工具员数的，我对了原件的行号 659、978 在新版里没了） | 做到 |
| 8 定义第 47 行与另三行叫法（第 96 行） | 改之前现读 | `replace-batch.py` 1 处 | 现读第 47 行与 `prove-red.sh` 行号逐条对得上（第 8 问第 9 条） | 做到；611 行判决没点名，是 610 到 612 同一句的续行，多改一行合理 |

### 多做、少做、做偏（汇总）

- 做偏 2 处：规格工具的判据与出路（第 4 行）；注释历史判法形态与改前量的对象（第 6 行）。
- 多做 3 处：`mv` 挪走一起判；换上工具认按路径装的写法；`doc-process-records.sh` 第 611 行续行。另有主 agent 追加的共用约束第 89、101 行两句，不在「三、改法」里，是攻方腿第 7 问的对象。
- 少做 2 处：登记表第 61、75 行的说明没在复跑之后改。第 61 行仍写「红的格是否只在点名那一格交主 agent 跑过再算数」（`.claude/gate.d/break-switch-replay-files.tsv:61`），第 75 行仍写「这几处没有照这一格的判法复跑过，红的格是否只在点名那一格交主 agent 跑过再算数」（`.claude/gate.d/break-switch-replay-files.tsv:75`），而这两条已被 20337、20351 复跑、逐格相同；工具员当时写明复跑看过之后由主 agent 改成「复跑过」，没改。另一处是改注释历史前没有在 872 行上量新旧（上表第 6 行，后补）。
- 检出钩子第一次换上漏了六个开关名进名单（20339「6 处不对」，第二批工具员 `governance-r2-fix-guards-second/result.md` 补，20351「复跑了 117 条开关（1 份），自证红且点名的格与实际红的格逐格相同 117 条」），不属于「三、改法」那一行，是新写 500 多行没有真跑过一遍造成的；名单与开关表之间没有检查盯着，已登账本。

## 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；没往统一任务服务交任务，没起任何项目代码；判决要引的数里，只有第 9 问的换上趟数是我读 `task status --json` 现数的（量的），别的读数都是任务日志原句。
- 没读检出钩子第十四项的判法实现、换上工具 `SearchPaths` 的实现、任务服务自证与三语 README 的正文，对应的条只按任务读数与工具员报告判。
- 攻方腿的第 1 到 7 问没做；第 8 问第 4 条里「有没有哪种副本摆法能写出规格」只给了一个读数，归攻方腿。
- 第 13 条的账本侧四处过时句没逐处重查；第 17 条的前提（两台上限相同）没重核，它在被忽略的私有配置里。
- 「毫秒内才互盖」「正面形态会漏判更多」「库认一族更安全」是推的，没造样本验。

## 要主 agent 跑的命令

没有。我需要的读数都是只读取得的，判决要引的数在上面标了量的或推的；没有哪一条结论要靠起脚本才量得出。

## 清理

没有仓副本、没有编译目录，没建任何要删的副本。`D` 下只有任务日志与 json 摘要，合计 1.4M（`du -sh`，量的），不是仓副本；报告留在 `research/prompts/ledger-topdown-governance-r3-sonnet-output.md`，`D` 用完由主 agent 删。
