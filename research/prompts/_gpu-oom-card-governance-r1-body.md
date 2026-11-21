# GPU 服务接住 OOM 与卡坏上报：治理文件轮第一轮（gpu-oom-card-governance-r1，2026-11-19）

这一轮判的是 GPU 批处理验证服务（`gpu-verification-server/`）接住显存不够（OOM）、卡坏上报、装判器读数三件事，任务服务（`task-server/`）跟着加的「GPU 服务卡坏」告警，以及算键库 `scripts/gate/lib_function_units.py` 为它们补登的两行。改动都已在主工作区。

起因：2026-11-18 本机服务进程的日志里，卡 0（RTX 5090）判一块 1 态的待判块时 wgpu 报 Out of Memory，原行「GPU_VERIFICATION_SERVER_SERVING_PROBLEM stream=73d9765b… block=1 被主循环拒了：CardFailed { card: CardIndex(0), reason: "wgpu error: Out of Memory\n" }」；之后约 7 小时这张卡一块不判，设备并没有丢（那个进程日志里 GPU_DEVICE_LOST 0 行）。用户原话「OOM 需要倍catch 不能影响好的卡」；卡真坏了报给谁，用户在弹窗里选「control check 判红 + 任务服务告警 (Recommended)」。调度记录在 `records/2026-11-18-GPU服务OOM后整卡停用.md`。方案按 `crates/` 与这些治理文件今天的实现来谈。

## 问题

1. 换上的实现做的是不是各文件头与 README 说的事（正推腿）。
2. 哪一格会错：OOM 之后漏判、重判、丢块、卡永远不再用、好卡受连累；该报卡坏而没报、不该报而报；账算的仍挡不住 OOM；告警发错人、发不出、刷屏；算键登记放过了真该拒算的子进程边（攻方腿）。

## 实现今天的样子

主 agent 的观测（读的是此刻主工作区的文件）：

- 这一轮没有改 `crates/` 下的代码。OOM 的 panic 出在 `crates/singlefs-verification/src/gpu_unit_checks.rs:343`（`handle_an_uncaptured_wgpu_error` 先把 OOM 记进 `DeviceConditionOfACard` 再 panic），那一处在 GPU 判器单元闭包里，改了库里的 GPU 结论全换键，所以不动；判表用 `create_buffer_init` 建在 `crates/singlefs-verification/src/crash_judge_dispatch.rs:1406` 起，同样不动。
- **接住 OOM**：
  - `gpu-verification-server/src/card_worker.rs`：`CardAfterAPanic`（第 51 行）、`JudgesOnACard::after_a_panic`（第 90 行，缺省报判法自己的 panic）、`card_worker_loop`（第 164 行起，接住 panic 先问 `after_a_panic`，OOM 报 `OutOfMemoryCaught`、线程不退）。
  - `gpu-verification-server/src/gpu_judges_on_a_card.rs`：真卡那一份 `after_a_panic`（第 301 行）读设备情况：`lost_reason()` 有就是丢了；`out_of_memory_reports()` 比上一回多（第 304 行）就丢掉卡上全部判器、报 OOM。
  - `gpu-verification-server/src/service_loop.rs`：`take_an_out_of_memory`（第 1220 行）清这张卡的驻留账、把这一波在判的块撤回重新入池、等腾空的卡单独判；单独判又 OOM 的报拒 `OutOfMemoryOnAnEmptiedCard`（第 82、622 行）；设备丢了照旧 `take_a_failed_card`（第 1089 行）。
  - `gpu-verification-server/src/batch_scheduling.rs`：`plan_a_lone_block_on_an_emptied_card`（第 760 行）。
- **账给新装判表的暂存留地方**：wgpu-core 30.0.1 `src/device/resource.rs` 1163 行起，对不带 MAP_WRITE 的缓冲另开同样大的 StagingBuffer，装表那一刻表占两份。`gpu-verification-server/src/card_memory_ledger.rs` 的 `mebibytes_while_loading_a_judge`（第 82 行）；`gpu-verification-server/src/batch_scheduling.rs` 的 `tables_while_loading`（第 361 行，第 378、541、705 行用它）。验证包那一侧的 `mebibytes_a_judge_needs` 没动。
- **卡坏上报**：
  - 服务打三种行（`gpu-verification-server/src/serving_from_the_library.rs` 第 96 到 101 行的前缀，第 220、246 行起打）：`GPU_VERIFICATION_SERVER_CARD_FAILED`、`GPU_VERIFICATION_SERVER_OUT_OF_MEMORY_CAUGHT`、`GPU_VERIFICATION_SERVER_JUDGE_LOADED`；STOPPED 那一行带 `cards_failed`。
  - `gpu-verification-server/control`：`card_failed_lines`（第 1035 行）读单元此刻主进程的卡坏行，`check` 读到退 1；新子命令 `cards-failed`（`cards_failed_here` 第 1416 行起，第二台经已登记的 `forward` 走 ssh）。文件头第 95、306 行写前缀与源码逐字相同、自证从源码读来核。这份文件在这一轮之后又被改 GPU 服务族名的那一路会话换过一次（加 packed-tables 第三族），那一次的 28 处不在这一轮判；它的反向规格在 `~/.local/share/singlefs/work/batch123-control-serves-packed-tables-rebased/control-swap-spec-reverse.json`，施加后回到这一轮换上的那一版。
  - `task-server/task_server.py`：`GPU_CARD_FAILURE_CHECK_SECONDS = 60`（第 1323 行）；`gpu_cards_failed_command`（第 5529 行）、`gpu_card_failures`（第 5538 行，挂在 `watch_health` 上，至多每 60 秒另起线程读一次）、`read_gpu_cards_failed`（第 5557 行）。
  - `task-server/task_server_lib.py`：开关 `gpu-card-failure-unwatched`（第 1186 行）、`GPU_CARD_FAILED_ALERT_KIND`（第 2198 行）、`failed_cards_of_the_report`（第 2204 行）、选格表第 2344 行。
  - `task-server/task_server_selftest.py`：自证格 `health-gpu-card-failed-notified`（第 15125 行登记）。这三份在这一轮之后又被改任务服务的那一路会话换过一批，它那一批的改动不在这一轮判。
- **算键登记**：`scripts/gate/lib_function_units.py` 的 `UNKEYED_PROGRAM_WORDS` 补了两行：第 13703 行 `("task-server/task_server.py", "read_gpu_cards_failed", "command")`，第 13729 行 `("gpu-verification-server/control", "card_failed_lines", "paths.journalctl")`。两行都是照已有的同形登记补的（第 13701 行 `check_prune_server` 里的 `run`、第 13727 行 `version_mismatch_lines`）；补之前开关复跑「488 条里拒算 488 条」。
- **验过的**：
  - 服务补丁在段树上：clippy `--keep-going --all-targets` 退 0；点名测试五组 30 条全过；服务 12 个弄坏开关逐个打开都红在点名那一条（「0 passed; 1 failed」）。
  - 合入后验证 `scripts/gate/merge-and-verify.sh`：「✓ 测试全绿（4 个测试二进制，共 54 条测试…现判 54、复用 8、拒算 0）」，fmt、clippy 过；第 2 步 `crash-case-check.py`、`refused-test-registry.py list` 退 0。
  - control：工具员副本里自证 36 格全绿，4 个开关各只红点名那一格；换上后主工作区「验证命令退 0（python3 gpu-verification-server/control --selftest）」。
  - task-server：换上后两格自证退 0；整份自证「✓ task_server_lib.py 自证 418 格都对」（读卡坏那一处最后一次改写之前的一版）；开关复跑「开关 487 条、对上 475 条」，没对上的 12 条不是这一轮动过的开关，那一趟跑的过程中树变了（「键变了 10 条」），只当参考。
- **工具员报告写明的局限**（交腿判该不该算缺口），报告在 `~/.local/share/singlefs/work/gpu-oom-fix/result.md`：
  - 判法自己 panic（设备没丢、也没记过 OOM）照旧当卡坏，control check 叫人重置卡，重置对软件错没用。
  - 在腾空的卡上也 OOM 的块，这个进程不再认领，待判行留在库里、不写红判定。
  - 撤回的块只等撤回它的那张卡，那张卡坏了才换别的卡。
  - 任务服务的告警去重只在内存，服务重启后同一张坏卡再写一条。
  - 「另一张卡逐块相同」只核成性质（卡 1 没坏、每态只判一次、每块结论对）。
  - 上卡的两条测试（`tests/gpu_service_path_agrees_with_the_direct_judge.rs`）这一轮没跑，「新表两份」之后在 `state_count/3 == 1` 时登记会不会被拒，没量过。
  - 驱动份额 2560 MiB、开卡后额度不再重读、别的进程开卡这三样只记进 README「还没做的」。

## 范围

这一轮被判的文件（读此刻主工作区那一版）：

- `gpu-verification-server/src/` 下 `card_worker.rs`、`gpu_judges_on_a_card.rs`、`service_loop.rs`、`batch_scheduling.rs`、`card_memory_ledger.rs`、`serving_from_the_library.rs`、`break_switches.rs`，`gpu-verification-server/tests/` 下 `service_loop_with_judges_off_the_card.rs`、`batch_scheduling_requirements.rs`、`library_pending_pool_on_a_temporary_library.rs`，`gpu-verification-server/README.md`：这一轮的改动是 `~/.local/share/singlefs/work/merge-and-verify/2026-11-19-332486/merge-patch/crates.patch` 加 `~/.local/share/singlefs/work/gpu-oom-fix/readme.patch`。
- `gpu-verification-server/control`：只判这一轮换上的那 18 处，规格 `~/.local/share/singlefs/work/gpu-oom-fix/control-swap-spec.json`。
- `task-server/task_server.py`、`task-server/task_server_lib.py`、`task-server/task_server_selftest.py`：只判这一轮换上的 20 处（规格 `~/.local/share/singlefs/work/gpu-oom-fix/task-server-swap-spec.json`）与之后 `read_gpu_cards_failed` 的改写（今天的第 5557 行起那一段）。
- `scripts/gate/lib_function_units.py`：只判第 13703、13729 行那两条登记。

## 要腿判的

每问回「成立」「不成立」「推不出」，推不出的算打中；打中给能照喂的输入（替身判器的设定、一段假 journal、一条命令）或 文件:行。

正推腿：

1. `card_worker.rs`、`service_loop.rs` 文件头「接住 Out of Memory」那几段，逐句对 `card_worker_loop`、`take_an_out_of_memory`、`plan_a_lone_block_on_an_emptied_card`：撤回的块进池的次序与回话、腾空的卡怎么腾、单独判又 OOM 时报拒、核对组那一支、设备丢了照旧进 failed。
2. `gpu_judges_on_a_card.rs` 的 `after_a_panic`：「OOM 次数比上一回多」这个判法在同一张卡连着 OOM 两回、OOM 之后又判法 panic、OOM 与设备丢失同时到时各判成什么，与文件头说的对得上没有。
3. 「新表两份」：`tables_while_loading` 三处调用（挑新流、新装分草稿区、不饿死那一遍）与登记流时的判，和 `mebibytes_while_loading_a_judge` 的文档串对得上没有；驻留的账是不是仍只记一份；`card_memory_matches_the_verification_package` 比的还是不是同一个量。
4. 卡坏上报一条链：服务打的三种行的前缀、字段 → control 的 `card_failed_lines` 与 `cards-failed` 怎么认 → `failed_cards_of_the_report` 怎么解 → `gpu_card_failures` 发给谁、写什么告警；每一环与下一环的字面对不对得上，README「日志」「起停」两节说的与代码一致没有。

攻方腿：

5. OOM 之后的块：造得出一块被判两次、一块丢掉（待判行留在库里而这个进程再也不取，也没有别的服务会取）、一块的结论写错卡号或写错流的历史吗？至少试：同一波里两条流、OOM 发生在核对组与判块交替时、OOM 之后服务收到停止信号、OOM 之后有人来借卡（`card_lease`）。
6. 好卡受连累：卡 0 OOM 的那一刻，卡 1 在判的块、驻留的判器、显存账有没有被动到；撤回的块等卡 0 腾空时，卡 1 空着而池里只有这一块，会不会一直不派。
7. 账：「新表两份」之后还造得出 OOM 吗（多条新流同时新装、驻留的判器接近额度、核对器借显存）；反过来，会不会让原来装得下的流登记被拒、或永远派不出去（工具员报告局限第 6 条）。
8. 卡坏误报与漏报：判法自己的 panic（例：断言失败）被报成卡坏、叫人重置卡；设备丢失而 `lost_reason()` 还没回调到时 panic 先到，被当成判法 panic 还是 OOM；control check 读的是不是只这一个进程的行（旧进程的、重启前的不算）。
9. 告警：任务服务每 60 秒起一次 `control cards-failed`（经 ssh 问第二台），第二台连不上、ssh 卡住、control 不在、输出多行或乱码时，服务健康那一轮会不会被拖住或抛异常；告警发给「名下有在排在跑任务的会话」，坏卡发生时没有任务在跑的会话会不会永远收不到；服务重启后重发一条是不是可接受。
10. 算键登记：两条 `UNKEYED_PROGRAM_WORDS` 登记的理由是不是实话——替身那一种取法真的只给自证用、平常那一种真的照连进键；登记会不会顺带放过同一个函数里别的、真该拒算的起法。
11. 弄坏开关：服务 12 个、control 4 个、task-server 1 个新开关，有没有两条弄坏同一处、或点名的那一条靠别的开关也会红；control 自证格「从源码读前缀来核」在源码挪了位置或改了名时会不会静默过。
12. 多机：`control cards-failed` 在第二台读的是哪一份 control 与哪一台的 journal；两台的服务进程各自 OOM、各自卡坏时，任务服务的告警种类（`gpu-card-failed:<台>:<进程>:<卡号>`）会不会撞键或漏一台。

## 分工

正推腿做第 1 到 4 问，攻方腿做第 5 到 12 问。两条腿都读「范围」里的文件与那三份补丁 / 规格；只有一条腿读到的范围，在报告里写明。要跑的探针（替身判器的用例、喂 control 的假 journal、喂任务服务的假 cards-failed）不自己跑，写成 `task-server/task submit --cwd <自己的草稿目录> -- <命令>` 的整条进报告「要主 agent 跑的命令」一节；要起 cargo 的，命令里只点名测试函数，不跑整个包。
