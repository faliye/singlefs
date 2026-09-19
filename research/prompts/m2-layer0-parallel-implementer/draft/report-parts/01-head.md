# 层 0 崩溃点重放改成多线程（增补 2 收口表第 41 行）——实现员报告

时刻都是 UTC。仓副本：`/tmp/claude-1000/m2-layer0-parallel/copy/`（自己的 target）；主工作区开工时的快照：`…/original/`（`crates/`、`.claude/gate.d/`）；变异副本：`…/mutant/`；日志：`…/logs/`；草稿脚本：`…/draft/`。主工作区 `crates/`、`target/` 一个字没动。

**先说一件出错的事**：16:13 我误把一条占位文字「(not yet — continuing work; ignore)」经 SubagentHandback 交回了（本该只在最后调一次）。活没停，这份报告就是完整交付；`progress.md` 里记了这件事。

## 一、结论

1. `crash.rs` 里 `enumerate_layer0_*` 那一族全部改走一个按状态序号区间切片、`std::thread::scope` 多线程跑的本体，没加依赖；原有五个枚举函数签名不变，调用点一个没改。线程数环境变量名 **`SINGLEFS_LAYER0_THREADS`**，没设取 `available_parallelism`。
2. 合并是确定的：计数按片的次序相加、第一处违例取序号最小的那一处、观察者在调用线程上按序号次序调用（不要求 `Send`）。
   - 第一条流：线程数 1 与 32 各跑两次（外加一次负载 100 上下时的 32 线程），`LAYER0` / `CHECKER` / `CHECKER_FIRST` 三行彼此逐字相同，也与单进程对照 `research/prompts/m2-wave2-crash-verifier/54-supplement-first-txn-checker-line.log` 的全量那三行逐字相同。
   - 第二条流：线程数 32 跑了两次（负载 100 上下一次、机器空闲时一次），`LAYER0B` 行去掉行首 `LAYER0B ` 之后，与 `54-layer0-replay.log` 第二条 ✓ 行冒号之后的 879 个字符逐字相同。
3. 用时（机器空闲时重测，详见第五节）：第一条流 1 线程 202.45 秒 → 32 线程 7.35 秒；第二条流 32 线程 492.23 秒（单进程对照两条流合计 3 小时 50 分 49 秒，其中第二条流约 3 小时 47 分）。补过的 54 号整道在副本上 8 分 24 秒。
4. 进度：每跑完一片打一行 `LAYER0_PROGRESS`，`--nocapture` 下边跑边出；全量两条流各 512 行。
5. 判别力：丢掉一片、相邻两片重叠，已有的「展开的段按闭式数」断言都红（还有另外 4 到 5 条也红）；`crates/mutations.tsv` 追加 8 行（在补丁里）。
6. 门禁 54 号的改法写成补丁：线程数传进去、成功行报实际线程数、「没显式设成 1 却只用了 1 个线程」判红带下一步、进度行边跑边转出来不删，另把第一条流的 `CHECKER` 行报出来。
7. 补丁两份，19:38 对主工作区的内容 `patch -p1 --dry-run` 都干净（第二节、第十节）；那时主工作区这四个文件与开工快照逐字节相同。
8. 副本上 `check.sh` 绿（第九节）。

**什么现象会推翻**：同一份代码上换线程数、换切法跑出的 `LAYER0` / `LAYER0B` / `CHECKER` 行有任何一个字不同；或者某次全量的 `LAYER0_PARALLEL_FINISHED` 行 `worker_threads=1` 而没显式设成 1；或者主工作区在我快照之后改了 `crash.rs`、测试文件、`mutations.tsv`、54 号，补丁打不上或打上之后编不过。
