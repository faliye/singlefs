# 第十二批任务 56：三份自证两趟都对不上的开关

日期：2026-10-24。草稿目录 `D=~/.local/share/singlefs/work/landing-audit-b12-switch-stable-diffs`，复跑记录与判法 `S=~/.local/share/singlefs/work/landing-audit-b12-switch-naming-slow`（`runs/`、`tools/`），兄弟任务 55 的交回 `research/prompts/subagent-landing-audit-2026-10-22/b12-selftest-isolation.md`。

## 结论

前三条都改了。第 4 条照要求只归类，没改代码。第 5 条的命令写在「要主 agent 跑的命令」一节。

1. **mutation-shard-run.sh ① 只在设了开关时红，根子是门禁共用库打的一行带着场景小仓的路径，不是外层临时根。** `.claude/gate.d/lib/stage-cells.sh` 第 248 行：环境里有 `*_BREAK`、又认出是自证搭的小仓时，往 stdout 打一行「`<被判的根>` 是自证搭的小仓…认环境里的弄坏开关：…」。`<被判的根>` 是各场景自己的 `…/single/repo`、`…/main/repo`。① ③ ⑮ ④（后一格）都拿 `verdict_view` 比单机那一趟与本场景的 stdout，这一行一不同就判红。不设开关不打这一行，所以基准趟绿。任务 55 建的外层临时根与 ⑫ 的数法跟这件事无关：两趟 diff 里两条路径的外层前缀逐字相同，只差 `single` 与 `main`。改法是让 `verdict_view`（第 804 行起）把本场景小仓的路径（照写的与 `pwd -P` 的两种）换成 `<场景的小仓>` 再比。③ ⑮ ④ 在两趟里没被判成多红，是因为它们在十几条开关下都红，判法把它们当成了汇总行；改了之后它们只在真弄坏了的开关下红，所以 no-final-pass 要补点 ⑮。
   - 坏字节（peer-library-kept、store-not-fetched）：第 964 行「覆盖那一行：」用 `cut -c1-120` 截断。GNU cut 的 `-c` 按字节截，截在「引」字中间，两趟都在 stdout 第 3660、2768 字节。改成 `sed 's/；收尾.*$//'`，整句保留到「没记成 N 份」。全文件只有这一处按字节截非 ASCII 的文本（`grep -n 'cut -c\|head -c'`，其余 4 处截的是 sha256 的十六进制）。
   - 没点名（selftest-tmpdir-outside）：原来的点名全用『』，判法读不到。另一处更要紧：这一条单开时 ⑫ 本来就判不红（场景建在外面那一层，跑完照样删，「外面多出来的」「tmp/ 里剩下的」两栏都空）。两趟里它红，只是因为 ① ③ ⑮ ④ 被路径那一行连带红了，修掉那一行之后它会退 0、判成「没红」。所以 ⑫ 加了一样按跑的时候看的判据：假 cargo 每起一次把工作目录记进自己目录下的 `working-directories.log`（第 678–680 行）。⑫（第 1112 行起）要求本机那一侧的假 cargo 在 `singlefs-mutation-replay-*` 里至少起过一次，而且每一次都在自证自己的 `tmp/` 下。开关表那一条照新失败行写了「」。
   - no-peer-cleanup-on-failure 下 ④ 多红：点名写成了『』，改回「」。派发提示里说这一行出在 peer-without-memory-cap，`stable.py` 列的是 no-peer-cleanup-on-failure（`$D/stable-msr.txt`），peer-without-memory-cap 只多红 ①。
2. **broad-mutation.py term-leaves-step-running 是点名不全，不是哪一趟真慢。** 两趟失败行逐字相同：第二项「✗ 被 TERM / INT 停了：在途那一条没落行」是那一格转述的驱动输出，任务 55 把转述里的 ✗ 换成「〔转述〕」之后它不再自成失败行，两趟都少红这一项。删掉这一项，注明原因。1662 秒与 236 秒差在哪一台、多大负载：本机那一趟 75 条的中位数 1494 秒、最慢 1738 秒；第二台 76 条的中位数 226 秒、最慢 241 秒。1662 秒在本机那一趟里很普通。rerun-done 在本机超 1800 秒也是同一个原因（同一趟本机最慢的已到 1738 秒），开关本身两趟没有差别（第二台 241 秒，判「对」）。
3. **agent-watch.py finished 是点名多了。** 两趟都少红同样四项，都是 selftest_session_watch_in 里的格。任务 55 给那一段的看门狗加了 `--idle-minutes 600`（此刻第 5729–5731 行），交回了的子 agent 被这个开关当成没交回，也到不了告警的线，于是这四格不再跟着红。改前那两版（`86ffa88e…`、`67645496…`）两趟都红这四格，改后（`978165ea…`）两趟都不红。删掉这四项，注明原因。
4. 14 条只一趟不同的，归成三类，见「第 4 条」一节。
5. 两趟复跑交主 agent。

拿两趟记录推一遍改后的样子（只是推断，不替代复跑；装置与输出见各节）：mutation-shard-run.sh 16 条两趟都判「对」；term-leaves-step-running、finished 两趟、rerun-done 第二台那一趟都判「对」。

推翻条件：
- 主 agent 两趟复跑之后，mutation-shard-run.sh 某一条开关下 ① ③ ⑮ ④（后一格）仍然多红，而且 diff 里不只是小仓那一行。那就是判定真有别的差别，这里的推断错了。
- 不设开关的基准趟里 ⑫ 红、「起了 0 次」。那就说明假 cargo 没在 `singlefs-mutation-replay-*` 里起，⑫ 新加的那一样判据要改。
- selftest-tmpdir-outside 单开退 0。
- term-leaves-step-running 或 finished 在复跑里还有少红、多红。

## 第 1 条：mutation-shard-run.sh

两趟记录（sha256 `6682ab4d…`，第二台跑）里每条开关下 ① 那一行的 diff 都只有一段 `1c1`，两边都是 stage-cells.sh 打的「`…/tmp.<外层>/tmp.<work>/single/repo` 是自证搭的小仓…」对「`…/main/repo` 是自证搭的小仓…」，外层与 work 两级临时目录名逐字相同（`$D/stable-msr.txt`）。所以外层临时根（任务 55 第 574 行那一处）与 ⑫ 只数自己的，都没有让路径进到被比的那一行。路径是门禁那一行自带的。

改了五处（`$D/msr.diff`，对原件 38 加、11 删）：

| 位置（此刻行号） | 改了什么 | 关的是哪一条 |
|---|---|---|
| 第 804–820 行 `verdict_view` | 比之前把本场景小仓的路径（stdout 所在目录下的 `repo`，照写的与 `pwd -P` 两种）换成 `<场景的小仓>`，用 awk 的 `index`/`substr` 按字面换（路径不进正则） | ① ③ ⑮ ④（后一格）在每条开关下多红 |
| 第 964 行 | 「覆盖那一行：」不用 `cut -c1-120` 按字节截，改成 `sed 's/；收尾.*$//'` | peer-library-kept、store-not-fetched 的坏字节 |
| 第 678–680 行（假 cargo） | 每起一次 test 往自己目录下的 `working-directories.log` 追加 `os.getcwd()` | selftest-tmpdir-outside 单开要能红 |
| 第 1112–1126 行 ⑫ | 另判：本机那一侧（`$work/*/local-bin/`）的记录里，在 `singlefs-mutation-replay-*` 里起的至少 1 次，而且全在 `$(cd $work/tmp && pwd -P)/singlefs-mutation-replay-` 下；细节里报次数与第一处不在的路径。第二台那一侧照第二台的 TMPDIR（peer-host-lib.sh 第 113 行 `env -i`），不比 | 同上 |
| 文件头开关表 | no-final-pass 补 ⑮（第 80 行：「✗ ⑮ 第二台取值缺了：」「判定仍与单机逐字相同：退 1」）；no-peer-cleanup-on-failure 的 ④ 改成「✗ ④ 两台的构建环境不同：驱动拒、两份都没起、第二台这一趟的树删了：stderr：」「第二台留着 1 个目录」；selftest-tmpdir-outside 改成「✗ ⑫ 场景都跑在自证自己的 tmp/ 下：」「工作目录不在自证自己的 tmp/ 下的」；自证说明 ⑫ 那一段（第 564–565 行）补上新判据 | 补点名 |

no-final-pass 补 ⑮ 的理由：⑮ 那一格要求 `exit == 0`，no-final-pass 下门禁退 1（两趟记录：「判定仍与单机逐字相同：退 1」），这一格真红。原来它在 14 条开关下都因路径那一行红，判法当它是汇总行（一半以上的开关下都多红的同一行），不要求点名。路径那一行修掉以后，它只在 no-final-pass 与 peer-rocksdb-unchecked 下红，不点就判成多红。③ ④（后一格）在 no-final-pass 下原来就点了，③ 在 peer-library-kept 下也点了（判定库记录行是「复用抓到 3 条」，真红）。

### 红绿样本

**verdict_view**（装置：从脚本里抠出 `verdict_view` 函数，喂几份造出来的 stdout，比单机那一份；输出 `$D/vv-samples.txt`，路径换成 W）：
```
--- 改前（主工作区那一份，sha256 6682ab4d…）
single/stdout main/stdout: 不同：1c1 <   · W/single/repo 是自证搭的小仓（…），认环境里的弄坏开关：MUTATION_SHARD_RUN_BREAK=import-skipped --- >   · W/main/repo 是自证搭的小仓（…）…
single/stdout main/stdout-red: 不同：1,2c1,2 …
single/stdout main/stdout-otherswitch: 不同：1c1 …
--- 改后（新串）
single/stdout main/stdout: 相同
single/stdout main/stdout-red: 不同：2c2 <   ✓ adds 抓到 --- >   ✗ adds 没抓到
single/stdout main/stdout-otherswitch: 不同：1c1 <   · <场景的小仓> 是自证搭的小仓（…）…：MUTATION_SHARD_RUN_BREAK=import-skipped --- >   · <场景的小仓> 是自证搭的小仓（…）…：X=1
```
（括号里的「…」是我在报告里省的，原样在那份文件里。）只差路径的判相同（绿）；判定行真不同的（红）、开关不同的（红）照旧不同。

**⑫ 新判据**（装置：抠出 ⑫ 那一段放进函数，造 `work/{main,stopped,signalled}/local-bin/working-directories.log`；输出 `$D/t12-samples.txt`，路径换成 O）：
```
--- 绿：在 singlefs-mutation-replay-* 里起的都在自证自己的 tmp/ 下（另一行不在运行目录里的不算）
  ✓ ⑫ 场景都跑在自证自己的 tmp/ 下：跑完外面的临时目录（O）里没多出 singlefs-mutation-replay-*，自己的 tmp/ 下也一个不剩
--- 红：有一次在外面那一层（selftest-tmpdir-outside 的样子）
  ✗ ⑫ 场景都跑在自证自己的 tmp/ 下：…：本机那一侧的假 cargo 在 singlefs-mutation-replay-* 里起了 4 次，工作目录不在自证自己的 tmp/ 下的 1 次（O/singlefs-mutation-replay-cccc/shard-1-q）；外面多出来的「 」；自证自己的 tmp/ 里剩下的「 」
--- 红：一次都没记
  ✗ ⑫ 场景都跑在自证自己的 tmp/ 下：…：本机那一侧的假 cargo 在 singlefs-mutation-replay-* 里起了 0 次，工作目录不在自证自己的 tmp/ 下的 0 次（没有）；…
```
假 cargo 那几行：抠出改后的假 cargo，在一个只有 `crates/tiny/src/lib.rs` 的目录里起 `test -p tiny --lib -- adds`，打「test tests::adds ... ok」、退 0，`working-directories.log` 记了那一个目录；`python3 -m py_compile` 过。

**开关下整份红绿**：没造出来。本机整份自证在前台 240 秒里跑不完（下一节），交主 agent 两趟复跑。代替的推断（`$D/simulate_msr.py`，输出 `$D/simulate-msr.txt`）：拿两趟原记录，删掉推断只因路径那一行而红的格。① 除 no-final-pass 外都删；③ 除 no-final-pass、peer-library-kept 外都删；⑮ 除 no-final-pass、peer-rocksdb-unchecked 外都删；④ 后一格除 no-final-pass 外都删。selftest-tmpdir-outside 补一行新 ⑫ 失败行，再照此刻文件头用 replay.py 的判法判。结果两趟 32 条都是「对 少红 [] 多红 []」（`grep -c` 32 / 32 行）。

弄坏开关：没加新的。改的是自证自己的比法与判据，不是被测代码；能证红的是现有开关：no-final-pass（判定真不同，① 照红），selftest-tmpdir-outside（⑫ 新判据）。

### 本机前台试跑一次（没跑完）

`MUTATION_SHARD_RUN_BREAK=peer-crash-library-home-unset timeout 230 nice -n 19 bash scripts/mutation/mutation-shard-run.sh --selftest`（主工作区，本机一分钟负载 97），退 124。230 秒里只出了第一格「✓ 单机那一趟…」（`$D/msr-partial-peer-crash-library-home-unset.txt`），① 没轮到。到点后，驱动（进程 124287）自己接住 TERM，停了两份、删了第二台这一趟的树：`proc.py wait 124287 --timeout 90` 报「都已退出」，再 `ps` 一个不剩；自证的临时目录已由它的 EXIT trap 删掉（`ls` 报不在）。

## 第 2 条：broad-mutation.py term-leaves-step-running 与 rerun-done

两趟（sha256 `34d2b202…`；不带后缀那一趟在本机，#second 在第二台）的失败行逐字相同，只差进程号与路径。都只有两行 ✗：「✗ 自证：包装自己不停 scope 时被 TERM 停，在途那一步没有一起停（…驱动退 143；那一步的进程 […]、还有活着的）…」与汇总「✗ --selftest：124 格里 1 格判错」。点名的第二项「✗ 被 TERM / INT 停了：在途那一条没落行」在输出里是「  〔转述〕 被 TERM / INT 停了：在途那一条没落行，在途那一步已一起停掉」：那一格转述的驱动输出，任务 55 把 ✗ 换掉以后它不再算失败行（判法只收带 ✗ 的行）。这一格不经 `run()`，所以也没有「这一格起的驱动报的失败行」接在失败行末尾。

改法（文件头第 343–346 行，`$D/bm.diff`，3 加、2 删）：点名只留「✗ 自证：包装自己不停 scope 时被 TERM 停，在途那一步没有一起停」，括注里写明转述那一句不另成失败行。说明里不用「」，免得被判法当成点名。

时长（`runs/34d2b202…-BROAD_MUTATION_BREAK=*.json`）：

| 那一趟 | 条数 | 最短 | 中位数 | 最长 | 记录里的负载 |
|---|---|---|---|---|---|
| 本机（不带后缀） | 75 | 772.77 | 1494.28 | 1737.63 | term-leaves 那一条记下的是 56.6 |
| 第二台（#second） | 76 | 144.53 | 226.395 | 241.47 | term-leaves 那一条记下的是 27.5 |

本机基准趟 560.64 秒。term-leaves 的 1662 秒落在本机那一趟的中间。rerun-done 本机那一次记进 `runs-timeout/`（「前台限时 1800 秒没跑完，负载 55」），同一趟本机已有六条在 1673–1738 秒，是同一个原因：本机 31 路并行，外加别的会话的变异在跑。不是这条开关自己慢，第二台那一次 241.47 秒、判「对」。没改代码。

样本：拿两趟原记录照此刻文件头判（`$D/simulate_headers.py`，输出 `$D/simulate-headers.txt`）：
```
  scripts/mutation/broad-mutation.py 一趟 rerun-done 没跑 少红 [] 多红 []
  scripts/mutation/broad-mutation.py 一趟 term-leaves-step-running 对 少红 [] 多红 []
  scripts/mutation/broad-mutation.py #second rerun-done 对 少红 [] 多红 []
  scripts/mutation/broad-mutation.py #second term-leaves-step-running 对 少红 [] 多红 []
```
改前同一份记录判「不同」，两趟都少红那一项（`$D/stable-bm.txt`）。

## 第 3 条：agent-watch.py finished

两趟（sha256 `978165ea…`，都在本机，记录里的负载 129 与 184）都少红同样四项：「✗ 自检：--discover 应当自己找出没结束的 discoveredlong、确认过的不叫醒、到点退 4」「✗ 自检：--ack 应当落进状态目录、下一次起不带 --ack 也读回」「✗ 自检：同一个会话已有一个看门狗在盯时应当退 5 并写明」「✗ 自检：--discover 时会话里的子 agent 都交回了应当接着盯到点退 4、不报全部交回」，别的点名项两趟都红。改前的两版（`86ffa88e…` 两趟、`67645496…` 两趟、`26a5a48d…` 一趟）这四格都是「实际 3」。差在任务 55 给 selftest_session_watch_in 里的看门狗都加了 `--idle-minutes 600`：交回了的被这条开关当成没交回，也到不了告警的线，不再退 3。

改法（文件头第 285–297 行，`$D/aw.diff`，2 加、3 删）：删掉 selftest_session_watch_in 那一组四项，末尾加一句为什么那几格不跟着红。样本同上：原记录照此刻文件头判，两趟都是「对 少红 [] 多红 []」；改前两趟都「不同」（`$D/stable-aw.txt`）。

## 第 4 条：agent-watch.py 14 条只一趟不同的（只归类，不改代码）

都出在本机。两趟记录里的负载（跑完时记的），不带后缀那一趟 44–303，#second 那一趟 148–370。逐行出自 `python3 -B $S/tools/stable.py scripts/agents/agent-watch.py --only-bad`（`$D/stable-aw.txt`）。

| 类 | 开关（哪一趟） | 多红的那一行 | 根子（推的，没复现） |
|---|---|---|---|
| 一：在盯的那一个把第二个记的 --ack 盖掉 | heavylighttierargs、heavytopmost、heavywatchdog、heavywrapper、heredoc、infoalertwakes、memorypeakfloor、verdictselftestanyfile、verdictselftestsharedtmpdir（不带后缀）；pipeupstreamalways、scratchhog、verdictlines（#second），共 12 条 | 「✗ 自检：已有一个在盯时给的 --ack 应当记进状态目录，好让在盯的那一个读到」 | 在盯的那一个每次整查都把 watch-acks.tsv 读一遍、剪掉条件不在的、写临时文件、`os.replace` 换上（第 3755 行附近 `write_acks(state_dir, pruned_acks(acknowledgements(), …))`）。第二个 `--ack` 的写法也是先读后换（第 3706、3710 行），两边都不加锁。第二个的写落在在盯那一个的读与换之间时，换上去的是不带这一条的旧集合。自证等的正是第二个写完之后在盯那一个的第一次写回（第 5765–5767 行等 inode 变），赶上的恰是那一次。负载越高，读与换之间被挂起越久，窗口越宽 |
| 二：看门狗扫的时候假进程链还没定形 | memoryselftest（#second）；scratchhog（#second）的第一行 | memoryselftest：「timeout 包着没带前缀的只报 timeout 那一个：说明里写「底下另有 1 个」…实际说明：…底下另有 2 个同样没带前缀的重型进程（4148811, 4148813）」；scratchhog：「stop 那个假脚本底下的 sleep 4166699 同形却不是 remind.sh start，应当照报「进程无输出」：['进程 4166604 已跑 3s…：sleep 30']」 | 与任务 55 第 1 条同一类：自证起完假进程就让看门狗扫，没等整条链 exec 到位。memoryselftest 里 timeout 底下中间那一层在 exec 之前，命令行还是重型那一条，被多数成一个；scratchhog 里报到的是父进程 4166604（已跑 3 秒），不是它底下的 sleep。负载下 fork / exec 晚到，扫早了 |
| 三：2 秒的 CPU 采样里那个忙等的假进程没分到 CPU | taskstop（#second） | 「✗ 自检：写着的文件一小时没动、CPU 一直在涨的进程 4166958 被当成告警（它在算，只是不报进度）」「✗ 自检：CPU 在涨的进程 4166958 在报告里没有「在算」那一行」 | 「在算」靠 2 秒里 CPU 时间涨没涨来判。那一趟记下的负载 231，32 核的机器上一个忙等进程 2 秒里可能几乎没被调度，CPU 没涨，就被当成卡住 |

三类共 14 条开关（类一 12 条，类二 2 条，scratchhog 同时在类一、类二；类三 1 条）。类一是被测代码里真有的竞争（两个看门狗并发写同一份确认表），不只是自证不稳。登记时建议分开记。

## 改过的文件

| 文件 | 开工时 sha256（与任务 55 交回的一致） | 现在 sha256 | 对原件的改动量（`git diff --no-index --numstat`） |
|---|---|---|---|
| scripts/mutation/mutation-shard-run.sh | 6682ab4dbf23edb9… | e12c18ecdcbc772ca43b131ab60c4cf19c6ae877eef8034e055e5f28930f36d7 | 38 加、11 删 |
| scripts/mutation/broad-mutation.py | 34d2b2024a1de594… | 2931c2c9d7393b1c93e3434338c6cedcc2235c4c185ab6d9a5069d0d4f426165 | 3 加、2 删 |
| scripts/agents/agent-watch.py | 978165ea5dafe9e8… | 0402b0dfbadec6571c168b1b339e61857bcab843f58e62a54792678e2bd91bff | 2 加、3 删 |

`git diff --stat -- <这三份>` 原样（工作区对暂存区；暂存区里是别的会话暂存的任务 55 那一版，broad-mutation.py 那一份暂存的还是任务 55 的第一版，所以这里的数比我这一批的改动大）：
```
 scripts/agents/agent-watch.py          |  5 ++--
 scripts/mutation/broad-mutation.py     | 25 ++++++++++++-----
 scripts/mutation/mutation-shard-run.sh | 49 ++++++++++++++++++++++++++--------
 3 files changed, 58 insertions(+), 21 deletions(-)
```

`git status --short -- <这三份>`：
```
MM scripts/agents/agent-watch.py
MM scripts/mutation/broad-mutation.py
MM scripts/mutation/mutation-shard-run.sh
```

暂存区那三份的 sha256 现查过：mutation-shard-run.sh `6682ab4d…`、broad-mutation.py `d6a14976…`、agent-watch.py `978165ea…`。我没做任何 git 写操作。原件在 `$D/originals/`（`copy-originals.py check`：改过 3 份、不在了 0 份）。改法都经 `scripts/kb/replace-once.py` 换上，旧串、新串从 `$D/edits/` 的文件读，由 `$D/edits/ro.py` 交给它。每处都是「命中 1 次，已替换（改名换上新 inode）并回读确认」。`bash -n scripts/mutation/mutation-shard-run.sh`、`python3 -m py_compile` 两份 .py 都退 0。三份的开关表照复跑的读法解析：mutation-shard-run.sh 16 条、broad-mutation.py 77 条、agent-watch.py 122 条，没写全的都是 0。

## 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`。先设 `S=$HOME/.local/share/singlefs/work/landing-audit-b12-switch-naming-slow`、`F=$HOME/.local/share/singlefs/work/landing-audit-b12-switch-naming-final`、`D=$HOME/.local/share/singlefs/work/landing-audit-b12-switch-stable-diffs`。分法与驱动同任务 55 那一节。起的都是登记成 light 的 `--selftest`（fg.py 每条经 `nice -n 19`），不起 cargo。并行度取登记值：本机 `--jobs 31`（`.claude/heavy-thread-default` 第一行 31），第二台那一侧由 peer-run.sh 自己取 `heavy_thread_default`。三份 sha256 都变了，续跑表按 sha256 存，要全部重跑。第 1、2 条互不相干，可以同时用 run_in_background 起。

1. 本机：agent-watch.py 两趟，broad-mutation.py 第一趟（每条限时 1800 秒）。
   ```
   python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py,scripts/mutation/broad-mutation.py --jobs 31 --seconds 1800 --deadline 100000 --baseline-only; python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py,scripts/mutation/broad-mutation.py --jobs 31 --seconds 1800 --deadline 100000; python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py --jobs 31 --seconds 1800 --deadline 100000 --suffix '#second'
   ```
   三段各看末行「这一趟记了 N 条；还没记的 M 条」，M 都要是 0；不是 0 的，同一条命令重起，只补没记的。上一次本机这一段 broad-mutation.py 中位数 1494 秒、最慢 1738 秒，rerun-done 超了 1800 秒（第 2 条那一节），这一次多半还会有几条超时，要重起补。
2. 第二台：mutation-shard-run.sh 两趟，broad-mutation.py 的 #second（每条限时 1800 秒）。标签换成新的（`multi-host-run.sh` 标签已在就拒）：
   ```
   bash scripts/hosts/multi-host-run.sh --peer --label b12-stable-diffs-replay --send $S/peer-send .b12-replay --fetch .b12-replay/runs -- bash -c 'bash .b12-replay/tools/peer-run.sh scripts/mutation/mutation-shard-run.sh scripts/mutation/broad-mutation.py; bash .b12-replay/tools/peer-run.sh scripts/mutation/mutation-shard-run.sh scripts/mutation/broad-mutation.py'
   ```
   看末尾「第二台那条命令退 0」（bash -c 的退出码是第二次 peer-run.sh 的：0 是都记了，3 是还有没记的），再看每一段「== 第二台：… 趟第 N 次」后面 fg.py 的「还没记的 0 条」。取回之后并进续跑表：`cp -n $HOME/.local/share/singlefs/work/multi-host-run/b12-stable-diffs-replay/.b12-replay/runs/*.json $S/runs/`。起不来的（退 255、准入退 78、钩子拒），两份都放本机，照第 1 条的写法跑（`--only` 换成这两份）。
3. 判（两条都跑完之后）：
   ```
   python3 -B $S/tools/judge2.py $D/judge2-after.jsonl scripts/agents/agent-watch.py scripts/mutation/broad-mutation.py scripts/mutation/mutation-shard-run.sh; python3 -B $F/tools/limits.py scripts/agents/agent-watch.py scripts/mutation/broad-mutation.py scripts/mutation/mutation-shard-run.sh
   ```
   每份要「基准 退 0」，两组都是 `{'对': N}`：mutation-shard-run.sh 16，broad-mutation.py 77，agent-watch.py 122。有「不同」的，跑 `python3 -B $S/tools/stable.py <路径> --only-bad` 分清是两趟都有的还是只一趟有的。agent-watch.py 第 4 条那三类这一批没根治，再出只一趟的差异是预期之中的：那一份先不登记，照第 4 条的归类登记欠账。mutation-shard-run.sh 要特别看三样：selftest-tmpdir-outside 不是「没红」；基准趟 ⑫ 绿；peer-library-kept、store-not-fetched 不再「坏字节」。
4. 登记（两组全对的那几份）。限时取 limits.py 打的那一栏，理由里的 `<秒>` 换成它打的「最慢」：
   ```
   python3 scripts/kb/replace-once.py .claude/gate.d/break-switch-replay-files.tsv $'scripts/mutation/mutation-shard-run.sh\t800\t# 十六条开关逐个跑过两趟：点名的格与实际红的格逐格相同（经门禁跑的那几格在每条开关下都先撞上样本根那一判、退 2）；单跑一条最慢 362 秒' $'scripts/mutation/mutation-shard-run.sh\t<限时>\t# 十六条开关逐个跑过两趟：点名的格与实际红的格逐格相同（场景小仓带 .gate-break-sample，经门禁跑的那几格认开关，比判定前把场景小仓的路径换掉；⑥ 按状态等，另跑一趟两份都起不来的；⑫ 另看假 cargo 在哪起）；单跑一条最慢 <秒> 秒'
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/mutation/broad-mutation\.py\t' --line $'scripts/mutation/broad-mutation.py\t<限时>\t# 七十七条开关逐个跑过两趟：点名的格与实际红的格逐格相同（失败行转述里的 ✗ 换掉，这一格起的驱动报的失败行整行接在失败行末尾）；单跑一条最慢 <秒> 秒'
   python3 scripts/kb/insert-row.py .claude/gate.d/break-switch-replay-files.tsv --after '^scripts/mutation/mutate\.sh\t' --absent '^scripts/agents/agent-watch\.py\t' --line $'scripts/agents/agent-watch.py\t<限时>\t# 一百二十二条开关逐个跑过两趟：点名的格与实际红的格逐格相同（自证里的假重型进程按状态等起齐、活到判完，核数那一格照判器的结论核，不看此刻负载）；单跑一条最慢 <秒> 秒'
   ```
   replace-once 那一条的旧串我现查过，在表里恰好一行（`grep -cF` 得 1）。broad-mutation.py、agent-watch.py 此刻表里没有行（`grep -cP` 得 0），两条 insert-row 与任务 55 那一节相同。插完核一遍：每行 3 栏，第二栏是整数。限时比本机那一趟的最慢还短的（broad-mutation.py 若取第二台的数），登记前想清楚门禁那一格在哪台跑。
5. 命名检查不用跑：这一批没改 `.rs`。

## 新写的 `# gate-similar:` / `# hook-events:` 行

无：没有新加门禁阶段与钩子，只改了已有的三份。

## 执行中要知道的

- 本机前台试跑一次 mutation-shard-run.sh 自证（第 1 条末尾），230 秒到点被 timeout 停。驱动自己接住 TERM、停了两份；我只 `proc.py wait` 了它的进程号，没另发信号；`ps` 回读一个不剩，自证的临时目录已不在。
- 第二台那一趟 broad-mutation.py 的记录输出里有第二台上的仓路径。这份报告没有抄它；引用那几行时换成了「第二台」或省掉。
- 开工看负载：一分钟负载 97，`ps` 里有别的会话的 checker-tier-crates-mutation-replay `--quick` 与它们起的 cargo（覆盖插桩、基线），没有 qemu、fio、vm-bench、e152。
- 草稿里的三个样本装置目录（vv 48K、t12 60K、fc 60K）用完已删，删前 `du -sh` 记了大小。草稿目录里没有仓副本、编译目录与库拷贝。

## 没做什么

- 没跑重型测试（`checker-tier-crash-point-replay`、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 `scripts/mutation/broad-mutation.py` 起 cargo 的用法、`gate.sh`、全量 `cargo test`、`check.sh`）。没走定义三方，没提交。
- 三份登记为 light 的 `--selftest` 都没跑到绿：本机负载 88–97 下，mutation-shard-run.sh 230 秒只出了第一格。agent-watch.py（上一次基准 168 秒、负载 26）、broad-mutation.py（本机基准 560 秒）在前台 240 秒的上限里也跑不完。交主 agent，就是上一节第 1、2 条的基准趟。broad-mutation.py 与 agent-watch.py 只改了文件头的开关表（agent-watch.py 的在文档串里，broad-mutation.py 的在注释块里），不改行为。
- mutation-shard-run.sh 的改法只有抠出来的装置与按两趟记录推的判，没有整份实跑。推断依据与推翻条件在「结论」与第 1 条。
- 第 4 条三类只归类，没改 agent-watch.py 的代码，也没复现，根子是照失败行与代码推的。
- 派发提示第 1 条的猜测（外层临时根让路径进了被比的那一行）照查是不成立的，没改外层临时根与 ⑫ 的数法。
- 没碰 `scripts/hosts/run-with-memory-cap.sh` 与 `.claude/gate.d/` 下的文件（`stage-cells.sh` 打的那一行只读了）。
- `scripts/mutation/mutation-shard-run.sh` 第 988 行 `wc -l < … 2>/dev/null` 的重定向次序让「No such file or directory」漏到 stderr（两趟记录末尾那一句），不是 ✗ 行、不影响判，没改。
