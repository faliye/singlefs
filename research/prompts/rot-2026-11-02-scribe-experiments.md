# 阶段同步要改的实验页六件写回：报告

## 结论

六件都按判决写完，三语页同改：54 处替换、24 个文件，`replace-batch.py` 回读一致。没有动决策文件、`experiments-history`、实验页「历史版本」节与 `### 影响的决策`（规格改的是复跑写法，没碰任何分项的依据段）。

## 规格与检查

- 规格：`/home/fy5090/.local/share/singlefs/work/rot-2026-11-02-scribe-experiments/spec.json`（54 条），同内容 `spec.md`。
- sha256：spec.json `172469f0b2f830dad9c7bf8b18b7e74900b9876de0b1793f5b4385cb5c74ff18`；spec.md `30e6457a485b82f09575274e8a96f4ddf7eda5f7a7515783d894325fab2d5edb`。
- `python3 scripts/kb/kb-spec-check.py <spec.json>` 与 `<spec.md>` 末行（两次都是这一行，退出码 0）：
  `  ✓ 规格 54 条（1 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过（查了 54 条）`
- 旧串是原文里恰好命中一次的片段，不是整行（几条行长 600 字以上，整行抄易错）；每条恰好 1 次由规格检查与 `--dry-run` 两道核过。

## 找不到的目标

0 条。

## 写范围核对

命令 `grep -P '^kb-scribe\t' .claude/hooks/agent-write-scope.tsv` 读出的第一行模式是 `.claude/kb/**`；规格 24 个目标文件全在 `.claude/kb/experiments/` 下，落进这一行（文件清单：草稿目录 `files.txt`）。

## replace-batch.py 原样输出

dry-run（退出码 0）：`✓ 54 处都命中得对（24 个文件），--dry-run 没写`
实写（退出码 0）：`✓ 54 处替换，写了 24 个文件（改名换上新 inode），回读一致`

## 逐条改动（旧串 → 新串；三语各一份，下面写中文页的，英日页同义，全文在 spec.md）

1. E185（GC 一批搬迁的挂钟拆分）「复跑」段，原第 296 行：
   - `bash scripts/hosts/capped.sh 31 cargo build --release -p …` → `cargo build --release -p …`
   - `装置经 capped.sh 31 起，环境` → `装置线程不设，取登记值 .claude/heavy-thread-default，环境`（同页第 308 行已是这个说法）
   - 判决：F2-F7 报告 68–70 行。
2. E156（alloc-basis 四条岔路的代价数）第 606 行：
   - `特性与清场经 CHECKER_TIER_FEATURES、CHECKER_TIER_QUIESCE_PREFIX 带上` → `特性经 CHECKER_TIER_FEATURES 带上，清场由统一任务服务在命令外面套`
   - 判决：F2-F7 报告 93、96 行；F1 报告里没有中文页这一句，照英日页同改。
3. E176、E177、E178 复跑块最后一条变异命令：
   - E176 `MUTATE_JOBS=4 MUTATE_CARGO_JOBS=2 MUTATE_MEMORY_MAX=4G bash ../mutation/mutate.sh …` → `../../task-server/task submit -- bash ../mutation/mutate.sh …`
   - E177 `MUTATE_JOBS=4 MUTATE_MEMORY_MAX=4G bash …`、E178 `MUTATE_JOBS=4 bash …` 同样换成 `../../task-server/task submit -- bash …`
   - 判决：F2-F7 报告 150–158 行。
4. E165（恢复重放有界穷举与变异杀伤界）第 1389、1391 行：
   - 1389：`E165_KILL_BOUNDS_MEMORY_CAP=8G、--jobs 2 --threads 3` → `--jobs 2 --threads 3（内存额度由服务定）`；`**上限不能取 4G**：…第一趟 4G 时 tsv-1823 撞顶退 250（驱动日志第 8 行）；` → `…撞顶退 250（驱动日志第 8 行），补跑 8G 才得出结果；现在内存额度由服务按峰值预测，不由调用方写；`（「补跑 8G」有页内第 779 行为证）
   - 1391：`r2、r4 模式 8G、r3 模式 1G，--jobs 2 --threads 3` → `--jobs 2 --threads 3，内存额度由服务定`；`内存上限 16G` → `内存额度由服务定`
   - 判决：F2-F7 报告 237–238、244–245、250–251 行。
5. E197（扫环胶水内存随盘数）复跑块与口径，三语页（中文页第 23、25、26、27、33、34、59 行一带）：
   - 23 行注释：`各一条；每格单线程，同时跑 5 格，每格上限 8G` → `；每格一次服务尝试（分片的一片），内存上限与哪台机器由服务定`（「各一条」一并去掉：段现在经 `list-submissions` 打出，不是三条命令）
   - 25 行：`checks` 前加 `task-server/task submit -- `，`--out` 改 `research/results/e197-<日期>-checks.out`
   - 26 行：`segment compare … --peaks <峰值表.tsv> --jobs 5` → `task-server/task submit -- python3 …drive.py compare --binary $BINARY --out research/results/e197-<日期>-compare.out`
   - 27 行：`segment k64-full … --peaks … --jobs 5` → 一行注释（逐行打出要交的整条，第 0 批 checks 与 compare，第 1–10 批各 6 件 segment）加 `python3 …drive.py list-submissions --binary $BINARY --date <日期>`
   - 33 行：「内存包装量到的 cgroup 峰值（…）」→「统一任务服务记下的每次尝试的内存峰值（服务取按轮采到的与内存包装退出时读 cgroup 的 memory.peak 里大的…；驱动不套包装，主 agent 在每件有结局之后读 `task-server/task status <父任务号> --json` 存档，report.py 读存档）」
   - 34 行：`峰值表键 e197/<臂>/…` → `一格的键是（臂, 盘数, 填满档, 点名项数, 重复号），峰值取服务记下的那一次尝试的 memory_peak（结果行带 task= attempt=，report.py 按状态存档对上）`
   - 59 行（表第 3 列）：「内存包装在 scope 里读自己 cgroup 的 memory.peak，驱动从峰值表读，减空跑格」→「服务每次尝试记下的 memory_peak（…取大），主 agent 存 …status … --json，驱动与 report.py 读存档，减空跑格」
   - 判决：F1 报告 256–257 行；F8-F27 报告 229–231 行。
6. E171（三 oracle 合谋测试）第 37 行：
   - `装置 row 经 run-with-memory-cap.sh（限时 1800 秒）、还原` → `装置 row 直接起（限时 1800 秒；整条驱动命令交 task-server/task submit，内存额度由服务给，驱动不套包装）、还原`
   - 判决：F1 报告 32、163、167 行。**与判决有一处不同**：判决写「内存与线程由服务定」，但驱动 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/row_driver.py` 文件头写「线程数变量按线道线程数设进各自的环境」，线程是驱动设的，所以新串只写内存额度由服务给、驱动不套包装，没写线程由服务定。判决点名的路径 `e171_three_oracle_collusion/row_driver.py` 现查不是这个目录，我读的是 `_after_the_oracle_fixes` 那一份。

## 回读（这一次真跑的结果）

对写完的三语页逐组 `grep -c`，全是 0：
- `capped.sh`：E185 三语页各 0。
- `CHECKER_TIER_QUIESCE_PREFIX`：E156 三语页各 0。
- `MUTATE_JOBS`、`MUTATE_MEMORY_MAX`：E176、E177、E178 九页各 0。
- `--peaks`、`--jobs 5`、`peak table`、`ピーク表`、`峰值表`、`segment compare`：E197 三语页各 0。
- `run-with-memory-cap`：E171 三语页各 0。
- `E165_KILL_BOUNDS_MEMORY_CAP`：E165 三语页各剩 2 处，在第 502、1198 行，见下「没改的」；第 1389、1391 行已无。

## 要主 agent 判的点（我没改）

1. E165 三语页第 502、1198 行仍有 `E165_KILL_BOUNDS_MEMORY_CAP=…`：它们在「做法」「两趟与那句对不上的核法」里，是当时那一趟的命令（`E165_KILL_BOUNDS_MEMORY_CAP=1G bash scripts/hosts/capped.sh 6 bash …`），判决的候选表里没有这两行，我按事件句不动。
2. E197 复跑块第 28 行 `report.py … <产物>.out --append` 与 `report.py` 现在的用法（`--artifacts <产物.out…> --status <状态存档.json…> [--out <判决.out>]`，文件头「用法」）对不上；判决没点这一行，我没改，三语页同。
3. E197 路径登记表第 2 条「源码落点」`drive.py:118 cgroup_peak_bytes`：现查 `drive.py`、`report.py` 里都没有 `cgroup_peak_bytes`；读服务峰值的是 `report.py` 里 `pinned_reading`（`def` 在第 242 行）。第 1、3 条的行号也对不上（`CountingAllocator` 现在第 137 行，页上写 133；`report.py` 里 `predicted_allocator_peak` 现在第 123 行，页上写 91）。页内第 15 行写着「换进主工作区之后重数一次」，这一件的判决没覆盖，没动。
4. `mutate.sh` 现在仍读 `MUTATE_JOBS`（只能压小）和 `MUTATE_CARGO_JOBS`（文件头第 124–127 行），只是 `MUTATE_MEMORY_MAX` 没了；E176 的 `MUTATE_CARGO_JOBS=2` 按判决「整行换成」一并去掉。
5. E185 三语页第 296 行同一句里「编译目录取 `CARGO_TARGET_DIR`：派到第二台时是 `multi-host-run.sh` 在第二台本地盘上这一趟新建…」我没改（判决 F2 只点名 `capped.sh` 两处，F3 另判过这一页其余行）。
6. `scripts/experiment-apparatus/replay.sh` 第 2389、2407 行注释还写「内存上限取 8G」「r2、r4 模式 8G，r3 模式 1G」（F2-F7 报告「异常」第 4 条已提）：脚本不归我，交实现一侧。

## 要主 agent 跑的命令

没有决策状态、分项或欠账的改动，不需要 `doc-decisions.sh --write`。三语六格（轻型，读文本）：

1. 命令：`SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 doc-text.sh 的 trilingual-files、trilingual-structure、trilingual-changed-together、trilingual-glossary、trilingual-untranslated、trilingual-omission 六格; 范围=6 格; 为什么=核实验页六件写回的三语对得上、译文用词照术语表' task-server/task submit -- bash .claude/gate.d/doc-text.sh --check trilingual-files,trilingual-structure,trilingual-changed-together,trilingual-glossary,trilingual-untranslated,trilingual-omission`
   - 在哪起：仓根 `/home/fy5090/code/singlefs`
   - 看哪几行：`TASK_SERVER_RESULT` 末行 `exit=0`；`task-server/task log <号>` 里没有 `✗` 行；报「查了 0 项」或退 77 的格不算过。红了先看点名的文件在不在这 24 个里（其余 kb 页有别的会话没提交的改动，不在这一批）。
2. 提交前的整轮门禁（含 doc-experiments、doc-text 的指代与钟点格）照 `.claude/skills/gate/staged-gate.md` 由主 agent 跑，我没跑。

## 改过的文件与前后 sha256

前后两份清单原样放在草稿目录：`sha-before.txt`（开工时）、`sha-after.txt`（收尾时）。24 个实验页前后哈希全不同（`diff` 数出 24 个新哈希）；`.claude/kb/decisions.md`（`aa5f8962…`）与 `.claude/kb/decisions-history.md`（`9115c197…`）前后相同，没动。

24 个文件：
- E156：`156-alloc-basis四条岔路的代价数.md`、`156-alloc-basis-の四つの分岐に伴うコストの数値-一部実施済み.ja.md`、`156-cost-figures-for-the-four-alloc-basis-forks.en.md`
- E165：`165-恢复重放有界穷举与变异杀伤界.md`、`.en.md`、`.ja.md`
- E171：`171-三oracle合谋测试.md`、`171-3オラクル共謀テスト.ja.md`、`171-three-oracle-collusion-test.en.md`
- E176：`176-inode树叶合并三候选的容量与写代价.md`、`176-inode-葉のマージ3候補の容量と書き込みコスト.ja.md`、`176-capacity-and-write-costs-of-three-inode-leaf-merge-candidates.en.md`
- E177：`177-逐树不写墓碑的复活数.md`、`177-ツリーごとに墓石を書かない場合の復活数.ja.md`、`177-per-tree-resurrection-counts-without-tombstones.en.md`
- E178：`178-叶容器与记录死亡的计数.md`、`178-葉コンテナとレコードの死亡数.ja.md`、`178-counts-of-leaf-container-and-record-deaths.en.md`
- E185：`185-GC一批搬迁的挂钟拆分.md`、`185-GC-一バッチ分の移動の壁時計分解.ja.md`、`185-wall-clock-breakdown-of-a-batch-of-gc-migrations.en.md`
- E197：`197-扫环胶水内存随盘数.md`、`197-リングスキャン接着部のメモリとディスク数.ja.md`、`197-ring-scan-glue-memory-by-device-count.en.md`
（路径都在 `/home/fy5090/code/singlefs/.claude/kb/experiments/` 下）

## 没做什么

- 没判决策、没写变更史（`decisions-history`、`experiments-history` 都没加条目）、没碰 E56 等先编后跑块与 E172、E173 页。
- 没跑门禁、没跑三语六格（写进上面「要主 agent 跑的命令」）、没提交。
- 译文用词我对着术语表与同页已有说法选的（`统一任务服务`＝unified task service＝統一タスクサービス；`shard`＝シャード；`登记值`＝登録値），没跑 trilingual-glossary 格验证。
- 推翻这批写法的现象：(a) 服务实际不拒 `task submit -- python3 …drive.py checks` 这类不带范围声明的写法（判决给的形态，我没在服务里试过）；(b) `drive.py list-submissions` 打出的整条与页上描述的批数不符（我只读了文件头与 `submissions_by_batch` 的调用，没跑）。
