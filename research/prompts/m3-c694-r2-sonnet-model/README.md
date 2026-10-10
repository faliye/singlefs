# m3-c694-r2 云端辩方腿（Sonnet）的模型

冻结树（`~/.local/share/singlefs/work/m3-c694-r2/tree/`）的拷贝上打探针补丁 `probe.diff`（只改 `crates/singlefs-core/src/mount.rs`：影子账那一函数的各支加计数，C 的挂钩加计数，C / W1 / W2 的候选开关照第一轮正推腿 `research/prompts/m3-c694-r1-sonnet-model/candidates.diff`；开关全关 = 今天的实现），再装三条 harness 档探针（`tests/`）：

- `m3c694_k4_reach.rs`：随机历史（与崩溃注入快档同一个种子基、同一组比重 BROAD、24 步）的每个段内前缀当崩溃状态，每态数「只读恢复落没落到走读失败」「可写挂载走没走到影子账的拒与账读不出那几支」「C 的挂钩有没有东西可隔离」，并比今天与 C 两臂的挂载结局、隔离槽数、空间准入结局、挂载之后池级 checker 的红名。
- `m3c694_k4_floor_raise.rs`：登记的崩溃枚举用例 floor-raise-pushed-by-the-session 在 harness 档的缩小复刻（只取段内前缀），四臂各挂一次。
- `m3c694_k4_unwitnessed_abandon.rs`：被抛弃根的三种造法（树表 0 条的暖机根 / 带文件的第一个文件根 / 树表 0 条池上第二次挂载的写行根），四臂各自从头造，量第二次挂载写掉了被抛弃根的几个单元、第三次挂载拒不拒。
- `check_r1_steps.py`：复核第一轮攻方腿「36 个不拒的起点里 33 个有 R 读不出的步」。

复跑：`SINGLEFS_REPO=<主仓> [PROFILE=quick|full] [THREADS=6] bash rerun.sh <空工作目录>`；比对：`bash compare.sh compare <工作目录>/out/extracted expected-<档>`。
`expected-quick/` 是 quick 档在两个空目录里各从头跑一次、逐文件相同的抽出结果（`compare.sh` 抽取时去掉耗时字段、按字典序排序）；`expected-full/` 是 full 档在空目录里从头跑通一次的抽出结果。
只跑 harness 档（`singlefs-harness` 的测试目标）；编译经 6G、跑经 4G 内存包装，线程上限 6。
