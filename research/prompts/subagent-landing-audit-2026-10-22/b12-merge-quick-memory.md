# 第十二批任务 50：合入后验证 checker 档快档那一步的内存上限

## 结论

四件都做完。`scripts/gate/merge-and-verify.sh` 给快档那一步单独一个上限，撞上限报本次未跑，自证加了 7 格和 6 个弄坏开关；`.claude/main-agent.md` 三处命令写法补了新参数。自证 `bash scripts/gate/merge-and-verify.sh --selftest` 在最终文件上退 0、0 个 ✗、「自证通过（查了 52 种）」（改前 baseline 同样退 0）。

## 改过的文件

- `scripts/gate/merge-and-verify.sh`：经 `scripts/gate/hook-swap.py` 脚本规格换上两次（第一次 18 处，`--verify` 给 `--selftest`；第二次 2 处，同样 `--verify`，退 0），之后一处注释经 `scripts/kb/replace-once.py` 定点补。别的会话没提交的改动（编译目录写法那一段）原样留着：换上前核过基线 sha256 与草稿起点相同，hook-swap 每次也核。
- `.claude/main-agent.md`：`scripts/kb/replace-batch.py` 三处定点（第 29、52、96 行命令写法补 `[--checker-quick-memory 上限]`；第 52 行补内存上限一句、本次未跑清单加「撞了内存上限」、「线程与内存包装已拼好」后补「撞了内存上限的那一条里上限已加大一档」）。别的行没动。

`git diff --stat -- scripts/gate/merge-and-verify.sh .claude/main-agent.md`（工作区对暂存区，别的会话已暂存的部分不在里面）：

```
 .claude/main-agent.md            |   8 +-
 scripts/gate/merge-and-verify.sh | 275 ++++++++++++++++++++++++++++++++++++---
 2 files changed, 259 insertions(+), 24 deletions(-)
```

sha256：`scripts/gate/merge-and-verify.sh` f3ff8f462fa957936b8afce97fefc7305a8d72d3d9c5552d4aa3e7ec52f21923；`.claude/main-agent.md` 82711099c2d71079ea1beb161af97d2c2d29d033d28f2a183c5fb92dbba7a6f9。

## 四条逐条

1. 单独上限。新参数 `--checker-quick-memory <上限>`（写法同 `--memory`，不合写法 `die`）。`run_capped` 加 `RUN_CAPPED_MEMORY` 覆盖，只有起快档那一道用它，test、fmt、clippy、build 仍用 `--memory`。没给时的取法在 `checker_quick_resolve_memory`，读表、取键、算「近几次最大」全经 `scripts/hosts/memory_peaks_lib.py`（`command_key`、`recorded_peak`、`CAP_FACTOR`、`headroom`），没另写一份：
   - 峰值表有这一步的记录：近几次最大峰值乘 `CAP_FACTOR`（1.5）向上取整到整数 GiB，写「量过的」；近几次里有撞顶的，取法里写「其中有一次撞了顶，真峰值不小于它」。
   - 表里没有：取推的 `CHECKER_QUICK_MEMORY_INFERRED=36G`，写「推的」。依据只有一次观测：2026-10-23 第一次真跑 24G 撞了上限（表里记 24G:hit，真峰值没量到），乘 1.5；没有按 36G 跑过，脚本里与文件头都写了「推的」。
   - 三种都不比 `--memory` 小，不超过本机 slice 总上限的整数 GiB（本机 40.1 GiB）。
   - 键就是 `run_capped` 起这一步的那条命令：`bash <capped.sh> <线程数> env -u MERGE_AND_VERIFY_BREAK bash <那一道> --check <两格> <根>`，和起它用同一个数组。拿本机真表实读（命令见下），得到「36G　量过的：峰值表近几次实测最大 24.0 GiB（其中有一次撞了顶，真峰值不小于它），乘 1.5 留余量」，与 2026-10-23 那趟的表行（24G:hit）对得上。
   - 成功句 `✓ checker 档快档过（…；内存上限 <上限>，<取法>；日志 …）`、开跑那一行、汇总行 `过（现判 x、复用 y、内存上限 <上限>）` 都写出上限与取法。
2. 撞上限。包装退 250 且日志里有「撞了内存上限」（2026-10-23 那趟 `checker-quick.log` 第 130 行就是这一句，用 grep 核过）：不记红、不算过，报「本次未跑：checker 档快档撞了内存上限 <上限>」，打出只重跑这一步、上限加大一档的整条命令（`SINGLEFS_HEAVY_TESTS=…` 与 `SINGLEFS_RUN_SCOPE=…` 在前，为什么一段写撞内存上限；包装次序 capped.sh、run-with-memory-cap.sh、命令），整条退 77（其余有红时仍退 1）。一档 = 上限乘 1.5 向上取整到整数 GiB，不超过本机 slice 总上限的整数 GiB；已到总上限的写「没有更大的一档」，出路写降线程上限，不打命令。退 250 而日志里没有那一句的照旧按红判。文件头退出码与「内存上限」一段已改。
3. 自证加 7 格，开关 6 个，见下一节。`.claude/hooks/selftest-registry.tsv` 那一行没动，档位不变（仍是假 cargo、假包装）。
4. `.claude/main-agent.md`：见上。

## 自证新格与弄坏开关

新格都在 `selftest()` 里，假包装 `fake-wrapper-record.sh` 把收到的上限记进文件，快档那一道换成已有的假门禁；峰值表与 slice 总上限在自证里固定（`RUN_WITH_MEMORY_CAP_PEAKS` 指向不存在的表、`MERGE_AND_VERIFY_SLICE_TOTAL=64G`），不读本机的。失败行句首都以「checker 档快档那一步：内存上限」或「checker 档快档那一步：撞内存上限」起头：

| 格（句首后半） | 绿的判据 |
|---|---|
| 内存上限 --checker-quick-memory 给了 20G | `--memory 8G --checker-quick-memory 20G`：快档那一趟收到 20G、别的步（fmt、clippy、build）收到 8G，成功句与汇总行写出上限与取法 |
| 内存上限 峰值表里这一步近几次最大 10 GiB | 表行近几次 `8 GiB:clean,10 GiB:hit,1 GiB:clean`：取 15G，写「量过的」与撞顶 |
| 内存上限 峰值表算出来的比 --memory 小时 | 表行 1 GiB：算出 2G，取 `--memory` 8G，写「比 --memory 8G 小，取 --memory」 |
| 内存上限 峰值表里没有这一步时 | 取推的 36G，写「推的」 |
| 撞内存上限（包装退 250，…） | 假包装撞上限：退 77、无 ✗、报「本次未跑：checker 档快档撞了内存上限 36G」、命令里 `run-with-memory-cap.sh 54G`、带 SINGLEFS_HEAVY_TESTS 与 SINGLEFS_RUN_SCOPE |
| 撞内存上限而上限已是本机总上限 64G 时 | 给 64G：退 77、报没有更大的一档、不打重跑命令 |
| 撞内存上限 退 250 而日志里没有… | 假包装只退 250 不打那一句：退 1、`✗ checker 档快档红` |

弄坏开关（文件头已逐项写了点名的格），各自打开后跑 `--selftest`，实际红的格逐格核过，与文件头所写相同（输出在草稿目录 `break-quick-memory-*.txt`，退出码都是 1）：

| `MERGE_AND_VERIFY_BREAK=` | 实际红的格 |
|---|---|
| `quick-memory-shared`（快档共用 `--memory`，连新参数也不认） | 给了 20G、峰值表、比 --memory 小、推的、撞上限、撞上限而已是总上限，共 6 格 |
| `quick-memory-table-ignored`（不读表，一律推的数） | 峰值表、比 --memory 小，共 2 格 |
| `quick-memory-below-memory`（算出来比 `--memory` 小也照用） | 比 --memory 小，1 格 |
| `quick-memory-oom-as-red`（撞上限照旧记红） | 撞上限、撞上限而已是总上限，2 格 |
| `quick-memory-oom-exits-zero`（报本次未跑却退 0） | 同上 2 格 |
| `quick-memory-same-tier`（出路里的上限不加大） | 同上 2 格 |

绿样本：不设开关，最终文件 `--selftest` 退 0、0 个 ✗（`selftest-final.txt`）；红样本：上表六个开关各至少一格红。

## 新写的 gate-similar / hook-events

没有新门禁阶段、没有新钩子，没写。

## 要主 agent 知道的

1. **36G 很可能仍不够。** 本机峰值表里同一件事（两格、同一根）另一个键（`bash .claude/gate.d/checker-tier-crash-point-replay.sh --check crash-case-replay,feature-gated-tests /home/fy5090/code/singlefs`，别的会话手动单跑的）近几次记着 40G:hit 与 32G:hit，本机 slice 总上限 40.1 GiB。也就是说那一趟（库里没有的测试函数多，首趟现判 694 条）真峰值可能超过 40G，默认 36G 与加大一档的 40G 都可能再撞；撞了就走新的本次未跑路径，但根因可能在这一道的并行度（31 线程同时编、跑）而不是上限。这是推的，没量过；我没有改那一道，也没有降线程。
2. **退出码 77 是我定的。** 任务写「整条退非 0」，没指定码；选 77 对齐「77 是本次未跑，不是通过」，有红时仍退 1。要换别的码改 `NOT_RUN_EXIT`。
3. 显式给的 `--checker-quick-memory` 比 `--memory` 小时照给的用（「不比 `--memory` 小」只约束缺省取法），没拦。
4. `--peer` 时快档本来就不跑，所以 `peer_rerun_command` 没带新参数。
5. 重跑命令（红、没带前缀、显存不够、`--peer`、`--copy` 各处）里的上限从 `--memory` 改成了这一步用的上限；撞上限那一条再加大一档。
6. `.claude/gate.d/break-switch-replay-files.tsv` 等登记表（兄弟任务 42 的范围）我没碰；merge-and-verify.sh 新增 6 个弄坏开关，登记是否要补由任务 42 看。
7. `.claude/main-agent.md` 改动要走定义三方（门禁 doc-process-records 的 agent-def-adversarial-review 格）；我没走。

## 本机真表实读（证 1 的取法）

```
python3 -B -c "<脚本里 CHECKER_QUICK_MEMORY_PYTHON 那一段>" scripts/hosts default "" 8G 36G bash <capped.sh> 31 env -u MERGE_AND_VERIFY_BREAK bash <那一道> --check crash-case-replay,feature-gated-tests <仓根>
36G	量过的：峰值表近几次实测最大 24.0 GiB（其中有一次撞了顶，真峰值不小于它），乘 1.5 留余量
```

## 没做什么

- 没跑重型测试，没跑门禁阶段与各类 lint（gate-lint、shell-lint 等），没走定义三方，没提交，没碰 `.claude/gate.d/` 与 `.claude/hooks/`。
- 没用真包装、真 cargo 跑一遍快档（自证全用假的；真跑归主 agent）。撞上限的真实路径只用 2026-10-23 日志里那一句 grep 核过。
- 没改 checker 档快档那一道本身，也没降它的并行度。
- 没量过 36G 够不够，推的。
- 草稿目录里没有仓副本或编译目录（只有草稿脚本、日志与 hook-swap 的验证输出，852K），留作证据。

## 推翻条件

- 若 `bash scripts/gate/merge-and-verify.sh --selftest` 在别的机器（slice 总上限不是 40G 量级）上红：我把总上限固定成 64G 才不读本机，若红在读表这一段，说明 `memory_peaks_lib.headroom` 或表格式变了。
- 若主 agent 真跑时仍撞 36G 而日志里是 250 且有「撞了内存上限」，本次未跑路径会打 40G 命令；再撞则是根因不在上限（见第 1 条）。
