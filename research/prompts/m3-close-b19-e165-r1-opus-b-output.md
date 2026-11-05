# m3-close-b19-e165-r1 攻方腿：模型改成照规矩跑（Opus）

日期：2026-11-05。立场与攻击面同前一条攻方腿（正文第三节 Q2），承接 `research/prompts/m3-close-b19-e165-r1-opus-output.md`。
这一件只改模型与复跑脚本，不重做论证：第一节到第七节的推理、历史与改法都照前一份报告，这里一个字不改，也不重复。
模型一次都没编、没跑（子 agent 不交任务）；下面凡是数，除了标「量的」的，都是「推的」。

## 各格判定一览

判定照搬前一份报告「各格判定一览」，没有新的读数；这一张只多一列「哪条用例出读数」。

| 格 | 判定（前一份报告，推的） | 新模型里哪条用例出读数 |
|---|---|---|
| Q2.1 形甲（同 txg、不同实例同进择根） | 造不出（读码，推的） | `e165_attack_estimate` 的两行 `name=control`、`e165_attack_full` 各格 `form_a=` |
| Q2.1 形乙，正文前提原样（系统配置两槽都自证过） | 造不出，挡它的是 N-配置（推的） | `e165_attack_full` 里 `newest_system_configuration_fault=none` 的格 |
| Q2.1 形乙，系统配置最新那一槽读回全零或自证不过 | 打中（推的，待模型） | `e165_attack_minimal_history_fixed_crash_point` 六行（固定一个崩溃点）；`e165_attack_full` 里 `roots_hidden=2` 且系统配置为 `all_zero` / `not_self_verified` 的六格 |
| Q2.1 系统配置那一槽设备报读错 | 造不出：R2 拒（推的） | `e165_attack_full` 里系统配置为 `device_error` 的格的 `cell_mount … h2_mount=refused:…` |
| Q2.1 根槽三种读坏 | 三种同一结局（推的） | 最小历史六行按根槽读坏分三组，逐组比 |
| Q2.2 回退、清扫、被抛弃判在择根之前筛不筛 | 不筛（读码） | 不出模型读数 |
| Q2.3 v1823、v1825 | 经 `choose_root` 走得到的状态上同值（推的） | 最小历史六行的 `v1823=`、`v1825=` 与 `baseline=`；全量各格 `v1823_differs`、`v1825_differs` |
| Q2.3 v1826 | 界外不同值（推的，待模型） | 最小历史六行的 `v1826=`；全量各格 `v1826_differs` |
| 附带打中：可写挂载丢已确认的 C | 推的，待模型 | 最小历史六行的 `h2_mount=ok … effective=(1,4)`、各序的 `<序>_content=`、`<序>_c_lost=`、`mount_after_c_lost=` |

## 复跑命令与文件 sha256

模型目录 `research/prompts/m3-close-b19-e165-r1-opus-b-model/`，两份文件；前一份模型目录 `research/prompts/m3-close-b19-e165-r1-opus-model/` 不动。
另用仓里一份现成文件（不拷进模型目录，复跑时从仓根拷）：`scripts/experiment-apparatus/heavy_threads_default/threads.rs`。

| 文件 | sha256 | 是什么 |
|---|---|---|
| `e165_attack_form_b_sliced.rs` | `4f67d72e89d251cdf7c801473292c36b5e32dc1abfa6e7708dd6f7e62c7fe2fc` | harness 档测试目标源码（拷进副本的 `crates/singlefs-harness/tests/`），四条用例：`e165_attack_minimal_history_fixed_crash_point`、`e165_attack_estimate`、`e165_attack_sliced_matches_single_thread`、`e165_attack_full`（`#[ignore]`） |
| `tools/rerun.sh` | `b7d06dffe9e47a1ac6ddbdc5f85a03998810b92925b7ab49b6727b026c8ac9b8` | 从 `scripts/agents/three-way-rerun-template.sh` 拷，改了 `rerun_steps`，另改了 `parse_arguments` 里 `--work` 非空那一判与自证的三格（第三节写为什么） |
| 仓里的 `scripts/experiment-apparatus/heavy_threads_default/threads.rs` | `87184c401540a38e31dbab0c24e65209ae6ae9b7b41a1489afdef9e126ee34b0` | 线程数取法；复跑时拷成副本里的 `crates/singlefs-harness/tests/e165_attack_support/threads.rs`（子目录里的不成测试目标），模型用 `#[path]` 引进来 |

sha256 是这一次 `sha256sum` 现算的（量的）。

复跑：`task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-b-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/rerun-opus-b`。
冻结副本用主 agent 那棵树（它根上有 `.cargo`、`.claude/heavy-thread-default`、`crates`、`prune-server`、`Cargo.toml`、`Cargo.lock`，`ls -a` 现查过）；rerun.sh 只读它、拷进 `--work` 下的 `tree/`。

## 一、改的三条各落在哪

### 1. 最小历史单独一条用例（`e165_attack_minimal_history_fixed_crash_point`）

- 历史照前一份报告第一节「最小历史」那张表：起点历史（第 1、2 步）照旧由 `build_starting_history` 用真写者造；第 3 步实例 2 的可写挂载在读故障下跑（`mount_under_faults`，读坏 txg 5、6 两槽根与 C、D 的记录，两块盘最新那一槽系统配置读坏）；录下这次挂载的写流。
- 崩溃点固定一个，不枚举：写表里找实例 2、txg 6 的那一笔根槽 FUA 写，录制流里排在它前面的写全部持久，它与它后面的都不持久（第 4 步写行那一次 txg 5 完整持久，第 5 步崩在暖机写 txg 6 的根之前）。找不到那一笔写的格打 `crash_point=absent_…` 一行、不 panic。
- 格：根槽三种读坏（`device_error`、`not_self_verified`、`all_zero`）× 系统配置槽两种读坏（`all_zero`、`not_self_verified`），六格，每格一行 `OPUSMODEL name=minimal_history …`，末行 `name=minimal_history_done cells=6`。
- 每行写：这次挂载的结局 `h2_mount=`；崩溃点与持久了几条写；读故障消失之后读出的全部根 `roots=[…]`、`form_a=`、`form_b=`；基线、v1823、v1825、v1826 各择哪条（`baseline=`、`v1823=`、`v1825=`、`v1826=`）；各自那一版的第一个文件读出来是哪一版（`<序>_content=`，取 `first_file`、`version_b`、`version_c`、`version_d`、`other` 或 `unreadable:<卡在哪>:<错误名>`）；已确认的 C 丢没丢（`<序>_c_lost=`：读出来既不是 C、也不是叠在 C 上而没确认的 D，就算丢）；真 `choose_root` 择哪条；池级 checker 红哪几条 `checker_red=[…]`；再在没有读故障的盘上做一次可写挂载 `mount_after=`，读它写行那一版的文件 `mount_after_content=`、`mount_after_c_lost=`。
- 读文件用 `singlefs_core::mounted_read::open_pool_for_read` 打开那条根、`open_file` 第一个文件（inode 1）、`read_at` 从 0 读到 size，与起点历史四版内容逐字节比。
- 几秒内跑完：推的。一格只做一次挂载、一次 checker、四次读文件、一次可写挂载；起点历史前一份模型在任务 7230 里打的是 `starting_history_seconds=0.0`（量的，`/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/probe-7230-before-cancel.txt` 第一行）。

### 2. 枚举照规矩跑

- 线程数：`heavy_threads_default::default_heavy_threads(include_str!("../../../.claude/heavy-thread-default"))`，模块就是仓里那份 `threads.rs`（`#[path = "e165_attack_support/threads.rs"]` 引进来），取法与它的文件头一样：`SINGLEFS_THREAD_CAP` 是正整数取它，否则取登记文件第一条非注释行的数（此刻登记的是 31，`cat .claude/heavy-thread-default` 现查）。
- 切片：选中的状态按位置区间切成至多「线程数」片，`std::thread::scope` 里每片一个线程（`judge_the_selection`）；崩溃状态按下标直接算出（`crash_state_at`，次序与前一份模型的 `for_each_crash_state` 逐个相同：第 k 段之前整段持久、第 k 段掩码 0 到 2^|段|−2、最后一个是全部持久），所以每片独立、不共享可变状态。
- 合并确定：计数按片的次序相加（`CellTally::absorb`）；带形的状态行每片留下标最小的几行，按片的次序接起来再截前三行，等于全域里下标最小的三行；对拍时逐态一行同样按片的次序接。
- 进度：另起一个线程，每 300 秒每片打一行 `OPUSMODEL name=progress … slice=<片> positions=[<起>,<止>) judged=<已判> elapsed_seconds=<秒>`；全部片判完（发送端都丢了）就收。不到 5 分钟的跑法一行都不打。
- 估时（`e165_attack_estimate`）：对照两行；二十五格各做一次那次挂载、打 `cell_mount` 一行（状态数照 `closed_form_state_count`）；估时那一格（读坏两条根、全零、两块盘最新那一槽系统配置全零）不再全量，改成**等距抽样**：域里状态数 393 241（量的，前一份模型在任务 7230 打出的 `states=393241`），抽 stride = ⌈393 241 / 20 000⌉ = 20、共 ⌈393 241 / 20⌉ = 19 663 个（推的，按同一棵树同一份写流算），在 20 万以内；用登记的线程数切片判，按「抽样挂钟 ÷ 抽样数 × 全量要判的数」推 `projected_full_seconds`。缩的是：估时那一格的崩溃状态（全量 393 241 → 抽 19 663）；没缩历史、格与几何。
- 等距抽样的偏：宽段（这一格最宽 18 条写，量的，同一行 `widest_segment=18`）里 stride 20 的下标让掩码低位按固定样式取，抽样计数只当估时与线索，不当这一格的读数；这一格的读数归全量那一条。
- 全量（`e165_attack_full`）照旧 `#[ignore]`：二十五格每格这次挂载的崩溃状态一个不落，切片多线程，每 5 分钟每片一行进度；一段历史超过 10⁶ 个状态的格、越过预算的格不跑、打 `cell_skipped` 一行。预计态数与挂钟见第二节。
- 对拍（`e165_attack_sliced_matches_single_thread`）：估时那一格等距抽 600 个（stride 656，推的），同一个 `judge_the_selection` 以一片（单线程）与登记的线程数（至少 2）各判一遍；比的是逐态一行（形、全部根、四种序各择哪条、真 `choose_root`、checker 红哪几条、没有读故障时可写挂载落到哪一版）、带形的那几行、合并后的计数行，三样连成一份逐字比；不同就打第一处不同的两行、断言红。单线程那一遍逐态行数要等于抽样数，防两边都空。

### 3. `tools/rerun.sh`

- `--work` 非空不拒：打一行 `name=work_not_empty given=<给的目录> new_subdirectory=<新目录>：…`，在它下面新建第一个还不在的 `rerun-<n>`（`mkdir` 不带 `-p`），这一趟只写那里，旧输出不动；`--work` 是文件的照拒、退 2。
- 自证（`--selftest`，假驱动、不编不跑：`rerun_steps` 认自证的标记就只拷副本、打 `name=rerun_roots`）原来那一格「非空退 2」换成三格：非空时打说明、写进 `rerun-1`、旧输出还在；同一个目录再起一次写进 `rerun-2`；`--work` 是文件退 2。文件头两个弄坏开关的「连带红的格」跟着改了。
- `rerun_steps`：冻结副本拷进 `<work>/tree`（`rsync -rl`，不保留修改时刻），放进模型源码与 `threads.rs`，打九份被判源码与登记文件、模型、`threads.rs` 的 sha256；同一棵树依次跑最小历史、对拍、估时三条，任一条退非 0 就停、退那个码；三条都过且 `projected_full_seconds` 不超过 2400 才跑全量，否则打 `name=full_skipped`。
- 与模板「只改 `rerun_steps`」那一句对不上：第 3 条要的行为只能改 `parse_arguments` 与自证；第七节写了。

## 二、全量预计态数与挂钟（推的）

| 量 | 值 | 怎么来的 |
|---|---|---|
| 一格已知的状态数 | 393 241 | 量的：`probe-7230-before-cancel.txt` 的 `cell_mount` 行 `states=393241`（前一份模型、同一份写者） |
| 挂载会成的格 | 16 格（读坏 0 条 1 格；读坏 1 条、系统配置不是设备报错 9 格；读坏 2 条、系统配置全零或自证不过 6 格） | 推的：前一份报告第一节与第七节（N-配置、R2 在哪几格拒） |
| 挂载被拒的格 | 9 格，各 1 个状态（不写盘，闭式 1 + 0） | 推的，同上 |
| 全量态数 | 约 16 × 39 万 ≈ 630 万，上限是预算 9 192 312 | 推的：假设成的那 16 格写流与已知那一格同量级；`e165_attack_estimate` 的 25 行 `cell_mount` 给真数 |
| 预算 | 9 192 312 = 10⁷ − 2 × 393 241（前一条腿两次被撤的估时，按至多各判完一格算）− 20 000（估时）− 2 × 600（对拍两遍）− 6（最小历史） | 算术（`python3` 现算）；模型里 `full_run_budget` 同式 |
| 每态挂钟下界（单线程） | 约 18 毫秒 | 推的：前一份模型单线程判这一格近两小时没判完（派发提示），7 000 秒 ÷ 393 241 |
| 全量挂钟下界（31 线程） | 约 630 万 × 0.018 ÷ 31 ≈ 3 700 秒 | 推的，下界；超过 rerun.sh 的 2400 秒门槛，rerun.sh 多半打 `full_skipped` |

推翻条件：`e165_attack_estimate` 的 `name=estimate` 行 `projected_full_seconds` 不超过 2400（rerun.sh 会接着跑全量），或 `total_states` 与 630 万差一个数量级。
估时超过 2400 秒时，照攻方腿定义第 3b 步先缩历史与格再跑：最省的是只跑打中那六格（读坏两条根 × 系统配置全零 / 自证不过），推的约 6 × 39 万 ≈ 236 万态、下界约 1 400 秒；那要另写一条用例，这一件没写（第五节）。

## 三、要主 agent 跑的命令

树：`/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree`（下面写 `<树>`，命令里写全路径）。先把新模型放进去（普通拷贝，不交任务）：

```
mkdir -p /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree/crates/singlefs-harness/tests/e165_attack_support
cp /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-b-model/e165_attack_form_b_sliced.rs /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree/crates/singlefs-harness/tests/e165_attack_form_b_sliced.rs
cp /home/fy5090/code/singlefs/scripts/experiment-apparatus/heavy_threads_default/threads.rs /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree/crates/singlefs-harness/tests/e165_attack_support/threads.rs
```

树根上已有 `.claude/heavy-thread-default`（`include_str!` 读它），不用另拷。前一份 `e165_attack_form_b.rs` 留在树里不碍事（下面每条只编 `--test e165_attack_form_b_sliced`）。

1. 只编（先确认编得过）。起点目录 `<树>`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b_sliced 只编不跑; 范围=1 个测试目标; 为什么=攻方腿改跑法的模型先确认编得过再交四条用例' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b_sliced --no-run`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；非 0 时 `task-server/task log <号>` 里的 `error[…]` 行。
2. 最小历史。起点目录 `<树>`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b_sliced 的最小历史用例; 范围=1 个测试目标; 为什么=攻方腿最小历史固定一个崩溃点逐格一行，几秒内跑完' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b_sliced -- --nocapture --exact e165_attack_minimal_history_fixed_crash_point`
   看：`exit=0`；日志里六行 `OPUSMODEL name=minimal_history root_fault=<三种之一> newest_system_configuration_fault=<两种之一> …`、末行 `OPUSMODEL name=minimal_history_done cells=6 seconds=<秒>`（秒数几秒量级）。要核的推断：六行都 `h2_mount=ok instance=2 chosen=(1,4) effective=(1,4) row_txg=5`、`form_b=true`、`roots=[…]` 里同时有 (1,6) 与 (2,5)、`baseline=(1,6)`、`v1823=(1,6)`、`v1825=(1,6)`、`v1826=(2,5)`、`v1826_content=version_b v1826_c_lost=true`；`baseline_content` 是 `version_d` 还是 `unreadable:…`、`checker_red`、`mount_after` 两列给前一份报告「基线这次挂载会拒或退回」那一句的真结局。有一行 `crash_point=absent_…` 就是那一格的挂载没走到暖机 txg 6，前一份第一节第 5 步的推法不成立。
3. 单线程与多线程对拍。起点目录 `<树>`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b_sliced 的单线程与多线程对拍用例; 范围=1 个测试目标; 为什么=切片多线程与单线程在六百个崩溃状态上逐字相同才用多线程的读数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b_sliced -- --nocapture --exact e165_attack_sliced_matches_single_thread`
   看：`exit=0`；`OPUSMODEL name=cross_check threads_sliced=<数> selected_states=600 state_lines_single=600 state_lines_sliced=600 … identical=true first_difference=none`。`identical=false` 时下面两行 `cross_check_single`、`cross_check_sliced` 是第一处不同，多线程的读数不能用。
4. 估时。起点目录 `<树>`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b_sliced 的估时用例; 范围=1 个测试目标; 为什么=对照、二十五格数状态、抽两万个崩溃状态估全量挂钟，超过四十分钟就不跑全量' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b_sliced -- --nocapture --exact e165_attack_estimate`
   看：`exit=0`；两行 `name=control`（`verdict=red_as_expected` 与 `misplaced_root_read_as_bad_form_a_absent`，与任务 7230 那两行应逐字相同）、25 行 `name=cell_mount`、一行 `name=cell … selected=19663 stride=20 domain=393241 …`、末行 `OPUSMODEL name=estimate … total_states=<数> … projected_full_seconds=<秒> …`。跑过 5 分钟会有 `name=progress` 行。
5. 全量（第 4 条的 `projected_full_seconds` 不超过 2400 才交；超过的先照第二节缩）。起点目录 `<树>`。
   `SINGLEFS_RUN_SCOPE='种类=target; 跑什么=singlefs-harness 的测试目标 e165_attack_form_b_sliced 的全量用例; 范围=1 个测试目标; 为什么=攻方腿二十五格每格挂载写流的崩溃状态一个不落，估时推出的挂钟在四十分钟以内' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree -- cargo test --release --offline -p singlefs-harness --test e165_attack_form_b_sliced -- --nocapture --ignored --exact e165_attack_full`
   看：`exit=0`；每 5 分钟一组 `name=progress`；25 行 `name=cell_mount`，跑了的格各一行 `name=cell … selected=<n> stride=1 domain=<n> …`，不跑的格 `name=cell_skipped … reason=…`；末行 `OPUSMODEL name=done cells=25 cells_skipped=<数> cells_with_form_b=<数> judged_states=<数> …`。要核的几列同前一份报告第 2 条（形乙那六格 `form_b`、`v1826_differs` 大于 0，`v1823_differs=0`、`v1825_differs=0`，全部格 `form_a=0`、`core_disagrees_with_local_baseline=0`）。
6. rerun.sh 整份（判决要引的数由它出）。起点目录：仓根。`--work` 给不在的新目录或已有的都行（已有内容会新建 `rerun-<n>`）。
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-b-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/tree --work /home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-main/rerun-opus-b`
   看：`TASK_SERVER_RESULT` 的 `exit=0`；日志里九行 `name=frozen_source`、`name=model_source path=e165_attack_form_b_sliced.rs sha256=4f67d72e89d251cdf7c801473292c36b5e32dc1abfa6e7708dd6f7e62c7fe2fc`、`name=thread_default_source … sha256=87184c401540a38e31dbab0c24e65209ae6ae9b7b41a1489afdef9e126ee34b0`、`name=minimal exit=0`、`name=cross_check exit=0`、`name=estimate exit=0`，之后 `name=full_skipped …` 或全量那几行，末行 `name=rerun_roots … full_exit=<0 或 not_run>`。退非 0 的件数为 0 算过。
7. rerun.sh 的自证（改了 `parse_arguments` 与自证三格，要证它们）。起点目录：仓根。
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-b-model/tools/rerun.sh --selftest`
   看：`exit=0`、`✓ 自检通过（查了 8 格）…`。弄坏开关两条照文件头：`task-server/task submit -- env RERUN_TEMPLATE_BREAK=root-from-self-path bash /home/fy5090/code/singlefs/research/prompts/m3-close-b19-e165-r1-opus-b-model/tools/rerun.sh --selftest` 与 `… RERUN_TEMPLATE_BREAK=root-check-by-wrapper …`，各应 `exit=1`，红在文件头点名的格。

第 1 到 5 条与第 6 条重复（rerun.sh 里同样这几条）；只要判决用的数，交第 6、7 条就够。跑完新派同立场的腿（写「承接：research/prompts/m3-close-b19-e165-r1-opus-b-output.md」与任务号），读 `task-server/task log <号> --all`，把读数填回前一份报告第一节、第三节，「推的」改「量过」或撤回。

## 四、没打中的形状

这一件不攻新形状，前一份报告第七节「没打中的形状」原样有效（推的，待模型）。新模型覆盖的格与前一份相同：二十五格（读坏 0 / 1 / 2 条根 × 三种根读坏 × 四种系统配置读坏，读坏 0 条只一格），两盘几何，起点历史一段；另加最小历史六格的固定崩溃点。

## 五、这条腿自己的限度

- 模型一次都没编过：照前一份（任务 7230 编过、跑出四行）的调法写，新加的只有 `mounted_read::open_pool_for_read` 一族读文件、`std::thread::scope` 切片与进度线程、`#[path]` 引的 `threads.rs`；语法用 `rustfmt --check` 在草稿目录的拷贝上过了一次（只报排版差异，没有解析错误，量的），类型没核，编不过要承接的腿修。
- 枚举域没有原地覆写的撕裂第三态，也没调 `crash_state_count_with_torn_in_place_overwrites`（它在 checker 档包里，harness 档测试目标引不到）；同前一份。
- 估时那一格的抽样是等距的，宽段里掩码低位取样不均（第一节第 2 条）；抽样的计数不当读数。
- 对拍只在一格、600 个状态上比；没有往切片那一侧塞已知故障证明对拍会红（例如合并时把片的次序倒过来）。逐态一行连同下标名一起比，次序错了逐字比就不等，推的。
- 「已确认的 C 丢没丢」按第一个文件读出来的内容判：读出 C 或叠在 C 上的 D 算没丢；读不出（`unreadable:…`）也算丢。基线择 (1,6) 而那棵树的单元被实例 2 重用时会落在「读不出」，那是真丢还是读者判严，要看 `checker_red` 与 `mount_after` 两列一起判。
- 只跑两盘几何、一段起点历史；单盘、吞写、只坏第一次的瞬时读错都没建格（同前一份第八节）。
- 全量挂钟只有下界（第二节），估时那一条跑出来之前不知道 rerun.sh 会不会跑全量；超过 2400 秒时要缩到打中那六格的那条用例没写。

## 六、对不上的地方（当场说）

1. 全量那一条标 `#[ignore]`，与 `.claude/rules/test-script-discipline.md@入口脚本：不带参数就是不指定范围`（那一节有一条不许用测试框架自带的标记分档）、`.claude/rules/verification.md@harness 档：不分档，红了只重跑红的`（那一节第一条不许按快慢、大小标 `#[ignore]`）对不上。照派发提示留了。会出的事：这份模型若进仓当 harness 档测试目标，不指定范围那一趟跑不到全量；看得出的现象是整包 `cargo test -p singlefs-harness` 的汇总里它报 ignored。能两边都满足的路：全量挪到一个只在 rerun.sh 里编的独立测试目标（不放进 `tests/` 顶层），这一件没做。
2. 模型自己逐个造崩溃状态、写在 harness 档测试目标里，与 `.claude/rules/verification.md@harness 档：不分档，红了只重跑红的` 第三条（调崩溃枚举函数或自己逐个造崩溃状态的测试归 checker 档，这是转述）对不上；它只拷进副本跑、不进仓，同前一份。进仓的话要改写成 checker 档的崩溃枚举用例、调 `enumerate_crash_states`。
3. rerun.sh 照定义第 8 步「只改 `rerun_steps` 那一段」，而派发提示第 3 条要的「非空 `--work` 不拒、打说明、新建子目录」落在 `parse_arguments`，自证那一格也要跟着改；两处都改了，模板本身没动。模板那一判若也要改，交主 agent 定（不归这条腿）。
4. 估时那一条「崩溃状态数压在 20 万以内」：照做（19 663，推的）；前一份估时那一格的全量读数因此不再由估时那一条给，归全量。

## 七、没做什么

- 不判正推、辩方那几格；不替主 agent 采纳；没重做论证，前一份报告的判定一个都没改。
- 没往统一任务服务交任务；模型没编、没跑，判决要引的数一个都没有，命令在第三节。副本上将来量出的数不算入库装置上的数。
- 没读禁读清单里的正推腿产出（`research/prompts/m3-close-b19-e165-r1-sonnet-output.md`）。
- 没改主 agent 那棵树、没改前一份模型目录。
- 没写「只跑打中六格」那条缩过的用例（第二节、第五节）。

## 草稿目录与引文核对

- 删了：`/home/fy5090/.local/share/singlefs/work/m3-close-b19-e165-r1-opus-b/syntax`（124K，`du -sh` 量的；拷了模型与 `threads.rs` 两份源码做 `rustfmt --check`，没编过）。草稿目录里没有仓副本与编译目录。
- `python3 scripts/agents/cite-check.py research/prompts/m3-close-b19-e165-r1-opus-b-output.md` 末行（这一段写进来之前那一次）：`✓ 核了 0 处引文、0 处裸行号，对上 0 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）`。本报告不引代码行号，引规则只写 `文件@小节`。
