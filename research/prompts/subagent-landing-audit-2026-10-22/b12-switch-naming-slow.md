# 第十二批任务 52：慢自证的开关点名与看门狗内存误报

日期：2026-10-24（开工时 2026-10-23）。复跑工具照兄弟任务 51 的 `tools/` 原样拷进本草稿目录 `tools/`（`replay.py` 只改了一行：仓根可由 `REPLAY_REPOSITORY` 换，给第二台那一侧用；判法与 code-tooling 的 break-switch-replay 格同一份）。另写了几份草稿工具：`fg.py`（前台限时下补跑续跑表，跑满前台限时的不记、记进 `runs-timeout/`）、`stable.py`（两趟各判一次、分稳的与只一趟有的）、`fixnames2.py`（按两趟都有的少红 / 多红出开关表的 hook-swap 规格）、`simulate.py`（规格套上之后拿已有记录按改后的表判）、`peer-run.sh`（第二台那一侧的复跑）。续跑表在本目录 `runs/`（`<sha256 前 16 位>-<开关>[#second].json`），开工时从兄弟任务 42 的 `runs/` 按此刻 sha256 拷来沿用。

## 结论

- 第 1 条 agent-watch.py：代码与开关表都改完换上（最后一版 sha256 `67645496e832ac1d…`）。没红的 infowakes 是格没判到（见下），超时的 memoryscan 是单查那一格两条路先后各空等满 90 秒，都修了；按两趟记录补齐了开关表；另把核数那一格的忙等干活进程从三个改成八个（负载高时三个合计掉到阈值以下，五条开关下偶发多红）。**最后一版只跑了 25 条（全部逐格相同，其中含点名要修的 infowakes、memoryscan、memorypoll 与新加的两条），其余 97 条没跑**：本机负载在 30 到 120 之间晃，前台 240 秒一次调用里并行 8 到 12 条时常过限时；deadline、finished、stale、pipeupstreamalways 四条单跑就过 233 秒，前台跑不了。剩下的交主 agent 放后台跑（命令在「要主 agent 跑的命令」第 1 条）。agent-watch.py 这一版**还没登记**进 break-switch-replay-files.tsv，等两趟跑完。
- 第 2 条 mutate.sh：33 条在最后一版（`fbbee331a49770e9…`）上跑了两趟，66 趟全部逐格相同。补齐开关表 15 条，没点名的 2 条写上了点名；sharedcopy 原来「红不红不定」（几个工作进程并发改同一份源码），改成开关下各工作进程逐条加锁轮流用第 1 份副本，只红数副本那一格。已登记进 break-switch-replay-files.tsv。
- 第 3 条看门狗内存误报：「进程常驻内存过线」加上「登记成轻的自证底下不判」，认法与「重型测试没带前缀」共用 `registered_light_selftest`；报告那一行写「登记成轻的自证底下过线的 N 个不报」；自证加了一红一绿两格与两条弄坏开关（memoryselftest、memoryselftestanyweight），最后一版上都红在点名的格。
- 第 4 条四份慢自证：**没做完**。用户新定可以分到第二台；第二台那一段要放后台等，照定义停在这里交回。broad-mutation.py 本机能跑（基准趟 164 秒、退 0），另三份基准趟在本机前台限时下跑不完（crash-amplification.sh 前一轮记的 535 秒；crash-point-replay.sh、mutation-shard-run.sh 233 秒没跑完）。已经打通第二台（拿一条只打核数与负载的命令试过，退 0；第二台 32 核、负载约 0.5），复跑工具、已有记录与第二台驱动放在 `peer-send/`。命令在「要主 agent 跑的命令」第 2、3 条。开关表一条没改。
- 第 5 条：登记了 mutate.sh 一行；agent-watch.py 与第 4 条四份等复跑做完再登记。

推翻条件：主 agent 照第 1 条命令跑完 agent-watch.py 两趟之后，`replay.py judge --only scripts/agents/agent-watch.py` 报出「不同」，且同一条的 #second 那一趟也不同（不是只一趟有的）；或门禁 break-switch-replay 格在提交前那一趟对 mutate.sh 报不对、单独重跑一次照样报。

## 第 3 条：看门狗「进程常驻内存过线」排除登记成轻的自证

改法（`scripts/agents/agent-watch.py`）：
- 新函数 `memory_selftest_owner`：只对过线的进程往上找（到看门狗的根进程为止、不含根进程与它自己），上层里第一个 `registered_light_selftest` 认得的进程就是它的自证，这个过线进程不报；自证那一层自己照判。单查（`memory_alarm_between_checks` 调的同一个 `memory_report`）也走这一条，所以「探针跑 3 秒就叫醒」那一类在单查里同样不叫醒。
- `memory_report` 的汇总行加「登记成轻的自证底下过线的 N 个不报」；`lib_heavy_tests.py` 导入失败时这一条不认、照报，汇总行写原因。
- 文件头「进程常驻内存过线」末段写了判法、射程与两条开关。

自证（`selftest_memory_under_selftest`，在 `selftest_memory` 里写满 256 MiB 的那几个停掉之后调）：造一个仓根与一份登记表（`selftest_registry_swapped` 换过去），`scripts/hosts/sample-memory-cap.sh` 登记 light、`sample-memory-heavy.sh` 登记 heavy，两份的 `--selftest` 底下各起一个写满 100 MiB 的探针，阈值 0.0625 GiB：
- 红样本（不该报）：light 自证底下的探针。开关 `AGENT_WATCH_BREAK=memoryselftest`（不认登记成轻的自证）下红两格：「✗ 自检：登记成轻的自证底下的过线进程不该报」「✗ 自检：报告那一行应当写出登记成轻的自证底下过线不报的进程数」。
- 绿样本（照报）：heavy 自证底下的探针。开关 `AGENT_WATCH_BREAK=memoryselftestanyweight`（上层参数带 --selftest 的一律当自证）下红「✗ 自检：不在登记成轻的自证底下的过线进程应当照报」。
- 另两条已有的开关跟着点名：memoryscan 多红后两格，heavyselftestancestor 多红前两格。
- 最后一版上实测（`runs/67645496e832ac1d-*.json`）：不设开关 175.33 秒退 0；memoryselftest 165.13 秒退 1、memoryselftestanyweight 164.05 秒退 1、heavyselftestancestor 164.3 秒退 1，judge 都判「对」。memoryselftest 那一趟原样失败行（截前 120 字）：
  `✗ 自检：登记成轻的自证底下的过线进程不该报：sample-memory-cap.sh --selftest 底下写满 100 MiB 的探针（阈值 0.0625 GiB）；探针 {'light': 2980524, …`

## 第 1 条：agent-watch.py

| 毛病 | 原因 | 改法 |
|---|---|---|
| infowakes 打开后没红 | 格没判到：`INFO_DETECTION_PREFIXES` 是「写被拒」「重型测试闸没判全」两样，「写被拒」在这个开关下还会被下一道「执行前已被 hook 拒了」挡住、照样不叫醒；自证里没有「重型测试闸没判全」的检出 | 检出那一格（`read_detections` 单元）多写一条「重型测试闸没判全：…」，要它只记不叫醒；开关表改点名这一格「✗ 自检：hook 检出：写被拒的与重型测试闸没判全的应当只记进报告不叫醒」 |
| memoryscan 过 200 秒（前一轮 225 秒超时） | 单查那一格 `--processes-only` 与 `--agents` 两条路先后各起一个看门狗，开关下单查不报，两个各空等满回报时刻 90 秒 | 两条路的看门狗同时起、都做完第一次整查之后才起那一个过线进程；过线进程起来之后至多等 40 秒，到点没退就停掉那个看门狗、判红。改后 memoryscan 203.78 秒、memorypoll 198.69 秒，都红在点名的格 |
| 19 条点名与实际红的格不同 | 多红的格没点名；coreusage 那一块有一项「」没配平，把后面几项吞成一项 | `fixnames2.py` 按 2d531902 那一版两趟都有的行补点名项（13 条），coreusage 配平、补四条告警记录格，timeout 补两格（两版都红），heavyselftestancestor 补判定行那一格；点名项截得太短的手改长 |
| 五条开关下偶发多红「变异门禁这一趟三个干活进程各忙一条线……不报，实际报了」 | 负载高时三条忙等合计不到阈值 2 核（量到 1.7、0.5） | busy 那一趟起八个干活进程（`FAKE_WORKER_COUNT`），idle 两趟照旧三个；mutationrows 那一条点名跟着改成「八个」 |

换上：三次脚本规格，`--verify` 都是 `nice -n 19 python3 -B scripts/agents/agent-watch.py --selftest`，三次都「验证命令退 0」（`swap-aw-code*.txt`）：

```
✓ 换上了 1 份（改名换上新 inode、回读一致），主工作区里再验过：1 份文件语法检查、内嵌 python 0 段编译、验证命令退 0（nice -n 19 python3 -B scripts/agents/agent-watch.py --selftest）
```

中间版（2d531902）两趟的判定（`judge2-aw-pass.jsonl`）：一趟 对 99、不同 19、没跑 4；二趟 对 103、不同 14、没跑 5。这两趟的稳的差异就是上表最后一版改的；只一趟有的是核数那一格（五条）与 memoryqueue 下「timeout 包着没带前缀的」一格（一条）。最后一版 25 条全对（`judge-aw-final-partial.jsonl`），复跑表里逐条列着。

## 第 2 条：mutate.sh

- 开关表：先拿改前那一版（39320f67）跑第二趟，两趟都多红的补点名项、两趟都少红的「」换成『』留着当说明（`fixnames2.py`，15 条）；memorynotfail、LIBTEST_RESULT_LINES_BREAK=should-panic-in-name 原来写「同上」、没有「」，照实际红的格写上点名。
- sharedcopy：原来几个工作进程并发改第 1 份副本，别的例红不红看时机（三趟里多红的格各不相同）。改成开关下 `run_worker` 用 `flock` 逐条加锁轮流占那一份副本（还原之后那一趟测试同样加锁），不真的并发改同一份源码；开关表写明只证数副本那一格。改后两趟都只红「✗ 自检：MUTATE_JOBS=4 时变异应当落在至少 2 份副本上…」。
- 换上：两次脚本规格。第一次（开关表）`timeout 236` 下 `--verify` 跑到一半被前台限时停掉（规格已换上、验证没跑完，没有回滚）；随后单跑不设开关的 `--selftest` 190.87 秒退 0，当作这一版的验证。第二次（sharedcopy 加锁）「验证命令退 0」。
- 最后一版（fbbee331）两趟：一趟 对 33、二趟 对 33（`judge2-mutate-final.jsonl`）。红样本是每条开关设上之后点名的格红、退 1（例：sharedcopy 下「✗ 自检：MUTATE_JOBS=4 时变异应当落在至少 2 份副本上、4 个工作进程各用一份副本与一个编译目录，实际施加变异的副本 1 份、编译目录 4 个」）；绿样本是不设开关的基准趟 126.06 秒、136.07 秒退 0。

## 第 4 条：四份慢自证（没做完，分两台的做法与交接）

本机前台能跑的只有 broad-mutation.py（基准趟 163.74 秒退 0，`runs/7d530d22aa7be022-baseline.json`）。crash-amplification.sh 前一轮记了 30 条开关趟与 535 秒的基准趟（sha 没变，沿用）；crash-point-replay.sh、mutation-shard-run.sh 的基准趟本机 233 秒内没跑完。crash-amplification.sh、crash-point-replay.sh 的 `--selftest` 末尾自带开关轮、从文件头读开关表（括号外的每一段算一格名，括号里的失败行原文不读），改开关表时点名的失败行只能写进括号里。

分法（按条数与单趟用时大致对半；第二台并行至多 8、看那边负载，本机至多 4、负载过 64 降到 2）：

| 哪台 | 跑什么 | 趟数 |
|---|---|---|
| 第二台 | crash-amplification.sh（20 条还没记的 + 50 条 #second）、crash-point-replay.sh（基准趟 + 41 + 41）、mutation-shard-run.sh（基准趟 + 16 + 16）、broad-mutation.py 的 #second（77） | 约 260 趟，每条限时 1800 秒 |
| 本机 | broad-mutation.py 第一趟（77） | 77 趟 |

第二台那一侧的驱动 `peer-send/tools/peer-run.sh`：摘掉 multi-host-run 带过去的第二台角色变量（`SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE` 这几个），自证看到的环境与本机一样；并行度取「核数 − 一分钟负载 − 1」夹在 2 到 8；记录写 `.b12-replay/runs`，由 `--fetch` 取回。`peer-send/runs/` 里放了 crash-amplification 前一轮的 31 份记录与 broad-mutation 的基准趟，第二台照 sha256 沿用、不重跑。树里那几份脚本与本机逐字节相同（multi-host-run 拷的是 git 列得出的工作区文件），sha256 对得上。

## 第 5 条：登记

经 `scripts/kb/insert-row.py` 接在 detached-run.sh 那一行后面（写前复核 sha256 未变，核过表里每行 3 栏、秒数是整数）：

```
scripts/mutation/mutate.sh	420	# 三十三条开关逐个跑过两趟：点名的格与实际红的格逐格相同（sharedcopy 下各工作进程逐条加锁轮流用第 1 份副本，只红数副本那一格）；单跑一条最慢 161 秒，负载过七十时基准趟 202 秒
```

限时取实测最慢 202 秒（负载过七十时的基准趟）的两倍取整。agent-watch.py 等最后一版两趟跑完再登记：建议限时 600（实测最慢的 timeout 一条 226.7 秒，deadline 等四条前台跑不完、前一轮在 900 秒限时下 272 到 338 秒）。

## run-with-memory-cap.sh（不碰、只报）

此刻 sha256 `d56034516139eee0…` 与前一轮记录相同，按前一轮那一趟记录判（`judge-rwmc.jsonl`，70 条）：对 5、不同 29、超时 1（wholejournal）、没跑 35。不同的 29 条：nocap、reject-foreign-reserve、check-reports-foreign-reserve、exitcode、noresult、noslice、nolock、noledger、nopredict、nodefault、ignoretable、noslicehit、notimelimit、keepstale、noprivate、overpeakreject、autonodefault、sessiononejob、nogpuoom、anynvrm、gpuoomonsuccess、queuehopeless、queuebaseline、nopeakmin、ignorequeuebycap、nooomadjust、nooomfirst、noslicemark、nestedqueues。其中 reject-foreign-reserve、check-reports-foreign-reserve、queuehopeless、nooomfirst、nocap、exitcode 有点名的项照不出（少红），其余是多红；多红里「上限 64M 里分配 512 MiB 的探针应当退 250…实际退 137」「限时 1 秒跑 sleep 4 应当 4 秒之内退 253…用了 7 秒」这类在好几条开关下都有，像是前一轮并行 8 条时内存与负载挤出来的，只有一趟、分不出稳不稳。逐条的少红与多红在 `judge-rwmc.jsonl`。交改它的会话。

## 要主 agent 跑的命令

工作目录都是仓根 `/home/fy5090/code/singlefs`，`S=$HOME/.local/share/singlefs/work/landing-audit-b12-switch-naming-slow`。都只起登记成 light 的 `--selftest`（每条经 `nice -n 19`），不起 cargo，不经内存包装（第二台那一段由 multi-host-run.sh 经第二台的 run-with-memory-cap.sh 包，上限取配置的 PEER_MEMORY_CAP）。下面三条互不相干，可同时用 run_in_background 起。

1. agent-watch.py 最后一版两趟（本机，并行 8，每条限时 900 秒；跑完退 0，有没记完的退 3，同一条命令重起只补没记的）：
   ```
   python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py --jobs 8 --seconds 900 --deadline 100000; python3 -B $S/tools/fg.py --only scripts/agents/agent-watch.py --jobs 8 --seconds 900 --deadline 100000 --suffix '#second'
   ```
   看末行「这一趟记了 N 条；还没记的 M 条」。跑完判：`python3 -B $S/tools/judge2.py $S/judge2-aw-final.jsonl scripts/agents/agent-watch.py`，看「一趟」「#second」两组是不是都是 `{'对': 122}`。
2. 第四条的第二台那一段（第二台并行至多 8，看那边负载）：
   ```
   bash scripts/hosts/multi-host-run.sh --peer --label b12slow-replay --send $S/peer-send .b12-replay --fetch .b12-replay/runs -- bash .b12-replay/tools/peer-run.sh scripts/crash/crash-amplification.sh,scripts/crash/crash-point-replay.sh,scripts/mutation/mutation-shard-run.sh scripts/mutation/broad-mutation.py
   ```
   看末尾「第二台那条命令退 0」与 peer-run.sh 打的每一段「== 第二台：… 趟第 N 次」后面 fg.py 的「还没记的 0 条」。取回的记录在 `$HOME/.local/share/singlefs/work/multi-host-run/b12slow-replay/.b12-replay/runs/`，并进本机续跑表：`cp -n $HOME/.local/share/singlefs/work/multi-host-run/b12slow-replay/.b12-replay/runs/*.json $S/runs/`。起不来（退 255、准入退 78、钩子拒）的，把这四份全放本机照第 3 条的约束跑（`--only` 换成四份，`--jobs 4`）。
3. 第四条的本机那一段（broad-mutation.py 第一趟，并行至多 4，本机负载过 64 时 `--jobs 2`）：
   ```
   python3 -B $S/tools/fg.py --only scripts/mutation/broad-mutation.py --jobs 4 --seconds 1800 --deadline 100000
   ```
4. 三条都跑完之后新派一个 tooling-writer 接着做，派发提示指到这份报告：
   - agent-watch.py：两组都是 122 条对的，照第 5 条登记一行（建议限时 600，理由照已有行写）；有不同的，`python3 -B $S/tools/stable.py scripts/agents/agent-watch.py --only-bad` 看两趟都有的，`fixnames2.py` 出规格补点名，再跑一遍。
   - 第四条四份：`stable.py <路径> --only-bad` 看稳的差异，`fixnames2.py` 出开关表规格（crash-amplification.sh、crash-point-replay.sh 的点名失败行只能写进格名后面的括号里，开关轮读括号外的格名，照兄弟任务 51 改 crash-amplification-lib.sh 的写法），经 hook-swap 脚本规格换上，再在最后一版上跑两趟、登记。基准趟过本机前台限时的那三份，最后一版还要放第二台或后台跑。
   - 第二台跑的那几条在复跑表里另起一列标「第二台」（记录里没有这一栏：取回目录里的就是第二台跑的，并表时按文件名记下）。

命名检查不用跑：这一批没改 `.rs`。

## 改过的文件

仓内（改前原件在 `originals/originals/`，每次换上的反向规格在 `hook-swap/reverse-*.json`）：`scripts/agents/agent-watch.py`（三次换上）、`scripts/mutation/mutate.sh`（两次换上）、`.claude/gate.d/break-switch-replay-files.tsv`（加 1 行）。

`git diff --stat -- <这三份>` 原样（agent-watch.py 与 mutate.sh 的改动这期间已被主 agent 暂存，工作区对暂存区只剩登记表一行）：

```
 .claude/gate.d/break-switch-replay-files.tsv | 1 +
 1 file changed, 1 insertion(+)
```

对开工时留的原件比（`git diff --no-index --numstat`，增 删 文件）：

```
199	37	scripts/agents/agent-watch.py
51	14	scripts/mutation/mutate.sh
1	0	.claude/gate.d/break-switch-replay-files.tsv
```

最后一版 sha256：

```
67645496e832ac1d272e1d295c192522e44b49a00fcaef410bd555251754fb5d  scripts/agents/agent-watch.py
fbbee331a49770e9d65bec7db019a6ced295925ac69e8d8950aeb0fbcbe58584  scripts/mutation/mutate.sh
99f8834c7d1d6e2e93dab0ec0b9d480a679c3bf94dbcb46b4b9a3fbe9ea8c033  .claude/gate.d/break-switch-replay-files.tsv
```

语法检查：`bash -n scripts/mutation/mutate.sh` 与 agent-watch.py 的 `ast.parse` 都过（没用 py_compile，免得写 `__pycache__`）。新写的 `# gate-similar:` / `# hook-events:` 行：没有（没新建门禁阶段与钩子）。动手前跑了 `gate-overlap.py --list`（`gate-overlap-list.txt`），内存这一判归 agent-watch.py 自己，没有别的门禁或钩子管同一件事。

## 没做什么

- 没跑重型测试（checker-tier-crash-point-replay、checker-tier-qemu-device-streams、checker-tier-lkmm、checker-tier-crates-mutation-replay、checker-tier-formal-verification、checker-tier-research-build-and-replay 本身、checker-tier-broad-mutation 与 broad-mutation.py 起 cargo 的用法、gate.sh、全量 cargo test、check.sh）；没跑门禁阶段与各类 lint；没走定义三方；没提交。
- agent-watch.py 最后一版 97 条没跑（上面第 1 条命令）；没登记。
- 第四条四份：开关表一条没改，没登记；crash-amplification.sh 还没记的 20 条、crash-point-replay.sh 与 mutation-shard-run.sh 全部、broad-mutation.py 77 条两趟都没跑（上面第 2、3 条命令）。
- 复跑在本机负载 4 到 138 之间跑（别的会话在跑崩溃点重放与它的编译、拷判定库），并行多在 8 到 13；负载过百时一批 13 条 mutate.sh 全过了 225 秒，那一批的记录没记、后来按 6 到 9 条重跑了。
- 试第二台时起过一趟只打核数与负载的命令（标签 b12slow-probe），第二台那一趟的树删了（回读不在）。一次在 `/tmp` 下试 fg.py 时它按 `peer-send/runs` 判 mutate.sh 没有基准趟、真起了一趟 mutate.sh 的 `--selftest`（136 秒退 0，被挪进后台），等它自己退出后删了那份记录与它建的 `runs-timeout/`。
- 草稿目录里没有仓副本、编译目录与库拷贝（`du -sh` 8.6M）；`draft/` 下两份是改后文本的预览，不是库拷贝。`peer-send/`、`tools/`、`runs/` 写进了 `.deliverables`（交接）。

## 复跑表：mutate.sh

「改前」是前一轮那一趟（sha 39320f67，`verify/judge-slow-mutate.sh.jsonl`）；最后一版是 fbbee331。

| 脚本 | 开关 | 改前 | 最后一版一趟 | 最后一版二趟 |
|---|---|---|---|---|
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=memoryascaught` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=memorynotfail` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=droprow` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=runtargetkept` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=countblind` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=sharedcopy` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=nofeatures` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=outertimeout` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=swallowwrapper` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=queueasinvalid` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=squeezedascrashed` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=gpumemoryascaught` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=honorbreakoutsideselftest` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=noparallelreason` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=overridecallerreason` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=baselinequeueasred` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=baselinesqueezedasred` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=baselinelogdropped` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=bareusage` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=exportparallelreason` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=ownnameparse` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `LIBTEST_RESULT_LINES_BREAK=should-panic-in-name` | 不同（没点名） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `RUN_WITH_MEMORY_CAP_BREAK=nocap` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=compileassuccess` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=compilenontest` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=libraryunread` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=calleesignored` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=dependenciesbycrate` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=includesunlinked` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `REUSE_CHECKS_BREAK=rejudged-sentinel-passes` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=no-cap-lints` | 不同（不同） | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=anchorstopsall` | 相同 | 相同 | 相同 |
| `scripts/mutation/mutate.sh` | `MUTATE_BREAK=rocksdb-entry-unchecked` | 相同 | 相同 | 相同 |

## 复跑表：agent-watch.py

「改前」是前一轮那一趟（sha 26a5a48d，`verify/judge-slow-agent-watch.py.jsonl`）；中间版 2d531902 跑了两趟，据它补的开关表；最后一版 67645496 只跑了 25 条，「没跑」的交主 agent（第 1 条命令）。memoryselftest、memoryselftestanyweight 是这一批新加的开关。

| 脚本 | 开关 | 改前 | 中间版两趟（agent-watch 2d531902…） | 最后一版一趟 | 最后一版二趟 |
|---|---|---|---|---|---|
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=ack` | 不同（不同） | 不同（不同）、不同（不同） | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=acksneverpruned` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=acksnotpersisted` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=backgroundwait` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=banned` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=bannedrefused` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=bannedwindow` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=context` | 相同 | 不同（不同）、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=contextzero` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=corerecords` | 不同（不同） | 不同（不同）、不同（不同） | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=mutationrows` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=coreusage` | 不同（不同） | 不同（不同）、不同（不同） | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=cost` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=cpu` | 相同 | 相同、不同（不同） | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=deadline` | 相同 | 没跑、没跑 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=detections` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=detectiontype` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=discoverfallback` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=discovernone` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=display` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=finished` | 不同（不同） | 没跑、没跑 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=handback` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=handoveranytype` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavycommitcheck` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyexempt` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyprefix` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavysampleselftest` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyscan` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestancestor` | 不同（不同） | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestanycopy` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestcondition` | 不同（不同） | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftestpathonly` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavyselftesttemporary` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavylighttier` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavylighttierargs` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavytopmost` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavywatchdog` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heavywrapper` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=heredoc` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=holders` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=idle` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=infoalertwakes` | 相同 | 不同（不同）、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=infowakes` | 不同（没红） | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiry` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiryack` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquirydelivered` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiryended` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquiryreset` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=inquirysent` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=helpformat` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=staleselftestscratch` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keepscratch` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keptcopy` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keptcopyage` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=keptcopyexists` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftover` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftoverack` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftoverexit` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftovergrace` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=leftovernotified` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=loop` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=loopnoise` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryfixedline` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorypeakfloor` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorypeakfloorfixed` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorypoll` | 相同 | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryqueue` | 相同 | 不同（不同）、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorygrew` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memorygrewfloor` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryoutcome` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryscan` | 超时 | 不同（不同）、不同（不同） | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryselftest` | （改后新加的开关） | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryselftestanyweight` | （改后新加的开关） | 相同、相同 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=memoryunit` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=midwaymerged` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=movedbackground` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=nosingleinstance` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=nowake` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=orphan` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=otheragent` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=outcomeignored` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=ownrefusal` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=pipeancestor` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=pipeupstream` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=pipeupstreamalways` | 不同（不同） | 没跑、没跑 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=processexit` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=progressanypath` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=progressnote` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=progressstale` | 相同 | 不同（不同）、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=refusedwakes` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=remindertimer` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=repeat` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=repeatsforgotten` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=restarthint` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=resume` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=resumeturn` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=rootfree` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scopeunlisted` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scratchhog` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scratchhogage` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=scratchhogtag` | 不同（不同） | 不同（不同）、不同（不同） | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=settle` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stale` | 不同（不同） | 没跑、没跑 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stalepending` | 不同（不同） | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stalesession` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=stalesessionrestart` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=taskstop` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=terminateacked` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=terminatezero` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=timeout` | 不同（不同） | 不同（不同）、没跑 | 相同 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=toolrevoke` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictlines` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftest` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestwrapperseen` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestanyfile` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestpathonly` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestsharedtmpdir` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=verdictselftestprivateignored` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `AGENT_WATCH_BREAK=zombie` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `SINGLEFS_WORK_ROOT_BREAK=new-root-unknown` | 相同 | 不同（不同）、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `SINGLEFS_WORK_ROOT_BREAK=legacy-root-unknown` | 相同 | 相同、相同 | 没跑 | 没跑 |
| `scripts/agents/agent-watch.py` | `SUBAGENT_TRANSCRIPT_BREAK=refused-counts-delivered` | 相同 | 相同、相同 | 没跑 | 没跑 |
