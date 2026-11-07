# prune-server 对攻六条修复（2026-11-04）

## 课题与出口

`records/2026-11-03-task-server与prune-server深入对攻审核.md` 里 prune-server 那六条，用户定由这个会话改（task-server 那七条交给管 task-server 的会话）。先对着现在的源码逐条核，成立的改、配测试与变异行，不成立或改法不对的写明依据、不改。

| 条 | 核的结论（读源码，没跑） | 去向 |
|---|---|---|
| 分段大值重写时读的一方拼不回 | 成立：`scripts/prune-server/prune_server_client.py` 的 `write_many` 先原地覆盖各段、再写清单，中间读到旧清单配新段，sha256 对不上退 3。只会报错、不会交错值 | 改在读的一侧：拼不回照 `SEGMENTED_REREAD_DELAYS_SECONDS`（0.05 到 1.6 秒六档）逐次重读原键，清单换过照新清单重拼，等完没换照旧退 3。Python 与 Rust（`prune-server/src/segmented_values.rs` 的 `assemble_rereading_the_original_key`，`reliable_client.rs` 经它拼）同一条。协议没有删键、文件头要求旧段不留孤儿，段键带版本那种改法不做 |
| 工作线程先编回应、后记额度 | 机制成立，但 `prune-server/src/server.rs` 文件头「额度」已把「工作线程正在编的回应（每个至多一帧）」算进内存上界；服务单元 `MemoryMax=infinity`，记录说的 1.5G cgroup 被杀在本机不存在 | 不改 |
| 重连重发认领把租约往后推 | 成立：`reliable_client.rs` 断线后认领照原帧重发，服务从收到起算到期 | 改：认领记下交出去的时刻与租约，每次发出去（头一次、重连、回了忙之后）照「原租约减去已过去的毫秒数，至少 1 毫秒」换帧里的租约（`protocol.rs` 的 `replace_claim_lease`，`reliable_client.rs` 的 `claim_lease_left`） |
| 读线程先等写线程、后关套接字 | 机制成立，但这是故意的：旁边注释写明写线程要把这条连接上还在处理的帧回完再关；记录给的「先关再等」会切断半关之后等回应的客户端 | 不改 |
| 扫到截断却一行都没回 | 一半不对：只有含那把超长键的那一支往下拆、每层 256 次、深度不过键长，不是 256 的 D 次方；最后点读那把键照样退 3。Rust 一侧（`crates/singlefs-checker-tier/src/verdict_store.rs` 的 `visit_stored_rows_with_prefix`）早已当场报错，Python 缺这一条 | 改：Python 照 Rust 当场退 3（scan-row-over-frame） |
| Python 不设段数上限 | 成立，现实到不了（100 万段 × 16 MiB ≈ 15 TiB）；Rust 已设 `SEGMENT_INDEX_LIMIT` | 改：Python 写之前同样拒（too-many-segments），一条都不写 |

## 测试与变异

- Rust：`segmented_values.rs` 三条单测（读到半截重写照新清单拼回、清单一直不换照旧拒、一次拼得回不等），`protocol.rs` 一条（换了租约的认领帧解出新租约），`reliable_client.rs` 一条（重发只给剩下的租约）；`crates/mutations.tsv` 追加四行。
- Python：`scripts/prune-server/prune_server_client_selftest.py` 三格 `segmented-read-sees-a-rewrite`、`segment-count-over-index-digits-refused`、`scan-row-over-frame-refused`，三个弄坏开关 `segmented-read-not-reread`、`segment-count-unbounded`、`scan-row-over-frame-split`。
- 没盖住的：`reliable_client.rs` 写帧时调 `claim_lease_left` 换租约那一行，删掉它没有测试会红（要起服务、断线重发的格才盖得住）。
- 改前备份：`~/.local/share/singlefs/work/prune-server-six-fixes/before/`。

## 事件

- 任务 6069：prune-server 的 `cargo fmt`、clippy 与库单测。任务 6070：Python 客户端自证与弄坏开关轮（用已装的服务二进制，Python 改动不依赖 Rust 改动）。
- 任务 6069 红在三处 clippy `type_complexity`（新加的签名），换成类型别名 `SegmentsReadByPoint`、`AssembledValue` 之后任务 6168 绿：fmt、clippy、库单测 22 条（新加 5 条在内）。
- 用户定：「重启吧 正好测一下重连」。任务 6194 照测过的源码编二进制（sha256 745231c3…），`control install --binary` 装上（旧的 311ff38e… 备份在 `~/.local/share/singlefs/work/prune-server-six-fixes/before/prune-server.binary`）。换法用 `systemctl --user restart`（SIGTERM，不发「服务停了」），不用 `control stop`（它让客户端退 75、不重连）。重起前连着任务 6045（pid 3836237，连接 4111）；重起之后同一个进程照容错重连上（连接 1），6045 照跑。
- 任务 6070（Python 客户端自证与开关轮，用改前装着的服务二进制）绿：「PRUNE_SERVER_CLIENT_SELFTEST cells=22 passed=22 failed=0」「PRUNE_SERVER_CLIENT_BREAK_SWEEP switches=23 matched=23」，新加三格与三个开关都在内。六条里改了的四条到这里写完、验完；新的服务二进制已换上。

## 判定库整族作废（用户经管 task-server 的会话 0a 转来，2026-11-04）

用户原话：「我们的库 不应该和指纹相关。 如果和指纹相关 那就没法剪枝。」「如果属实 这里需要交给 prune-server去改，顺便把其他的库的入库规则和判法也检查一遍。 这样的行为不能接受。」另：「目前的轻型门禁和 harness门禁没有向剪枝库中积累 每次都是重跑」「按照我们的设计 轻型门禁和harness 也是重型门禁的一部分」。

- 量的（任务日志成功句）：形式化验证 `--quick` 任务 4504、4716「现判 769、复用 0（判法单元指纹 变了 769）」，4802 现判 631、复用 138；变异 `--quick` 每趟「要现跑 304 条（没有作数的抓到记录的）」；崩溃点重放 `--quick` 快档测试函数复用 2647～2758、现判 440～472，放量复用 5290 态；harness 测试没有判定库。
- 审计（任务 6075，`scripts/gate/lib_function_units.py entries` 登记的 47 个判法入口逐个 `closure --explain`，产物 `~/.local/share/singlefs/work/judging-closure-audit/out/`）：crates 变异各入口闭包约 600 单元、import 时连带约 490；`verus-proof-check.py` 三个入口 2706 单元、连带 1089（其中 `crates-mutation-rows.py` 204、`memory_peaks_lib.py` 77、`prune_server_client.py` 76、`lib-lag-baseline.py` 69，`lib_function_units.py` 306 按 `NOT_JUDGING_FILES` 不进键）；`broad-mutation.py` 的 Judgement / GpuJudging 入口 2200 多单元、连带 1053；`replay.sh` 各入口与模型对拍入口 2～14 单元、连带 0。退 4（闭包里有拒算）的 11 个入口，拒算都落在 `lib_function_units.py`（12333、12504、12724、12740 行）。
- 机理：`scripts/gate/lib_function_units.py` 文件头「Python 闭包」import 时执行的模块级语句一起进闭包（定案在 `research/prompts/m3-close-reuse-closure-r3-main-verification.md` 末节「用户定案」第 3 条），判结论的函数与派活、内存、多机、判定库客户端、自证格写在同一批模块里，任何一处改动都换掉全族共用的判法单元指纹。
- 改法（规则不动，改代码的落点）：每一族把判结论真用到的函数与常量拆进单独的判法模块，不 import 基础设施；自证加一格「改了派活常量，判法单元指纹不变」。次序：crates 变异（`scripts/mutation/crates-mutation-rows.py`，0a 那边的一批已打进主工作区，任务 6151）→ 形式化验证 → 广谱变异与 mutation-prune → e165。harness 测试另接崩溃点重放快档那一套按测试函数单元键复用（`scripts/crash/crash-point-replay.sh` 文件头「快档按条复用」），接进 `scripts/gate/merge-and-verify.sh`。
- 别的会话（77）转来、归这一摊的两件，进同一批：① E201 装置运行时读的六份仓根文件（`.claude/heavy-thread-default`、`scripts/crash/crash-point-replay-shard-configuration-check.sh`、`scripts/hosts/peer-host-lib.sh`、`.claude/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.sh`、`.claude/singlefs-ai-sop/scripts/preflight.py`）不进它的快照摘要与复用键，改了照样复用；别的入口有没有同样的漏，随审计一起查。② `prune-server/src/configuration.rs` 第 18–25 行 `default_multi_host_configuration_path` 按 `CARGO_MANIFEST_DIR/../multi-host.env` 推配置路径，拷出的树里指不到；用户要求配置路径一律显式传。

## 第二台 5080 的 Xid 62（用户 2026-11-04：「你来解决 5080 必须要解决了」）

- 量的（第二台 `journalctl -k` 各次开机）：Xid 62 只出在 5080（PCI 总线号 1 那一张），共 6 次（11-02 两次、11-03 三次、11-04 一次）；同一秒 Xid 154「GPU Reset Required」，30 秒后 Xid 45 点名的线程是 GPU 判器测试（the_gpu_judge_j…、the_gpu_judge_c…、a_cold_start_af…）。每次之前 30 分钟 5080 没有一条 Xid 109；109（12778 条）都在崩之后。两张 5060 Ti 没有 Xid 62，总线号 5 那张有 2 条 109；本机 5090 有 41 条 109、1 条 154，没有 62。
- 量的（`task-server/task status` 逐条）：每次 Xid 62 时 0 号卡上只有一个任务（5172、5261、5442、5612、5642、5939），前 30 分钟这张卡上 0 条尝试收尾；前五次是崩溃点重放门禁带 GPU 那一趟，任务在 Xid 之后几分钟才失败。挂卡时长：5080 34.4 小时出 6 次，5090 43.9 小时、第二台 1 号卡 17.2 小时都是 0 次。所以不是服务这一层频繁换任务，也不是派活过长（那一种是 109）。
- 恢复办法（量的，11-04 做成）：`nvidia-smi --gpu-reset -i 0` 在没有客户端时也回「Not Supported」；只摘下扫回 PCI 时 GSP 起不来（「unexpected WPR2 already up, cannot proceed with booting GSP」）。做成的次序：确认 `/dev/nvidia0` 没人开着、停 `nvidia-persistenced` → 解绑声卡功能（snd_hda_intel）与显卡功能（nvidia）→ 写 `/sys/bus/pci/devices/0000:01:00.0/reset` → 两个功能都 `remove`、`/sys/bus/pci/rescan` → 起回 `nvidia-persistenced`；回读「GPU Recovery Action: None」、三张卡编号同原来。会被抄第二遍，要收成 `scripts/hosts/` 下的一份脚本（带回读与 `--selftest`）。
- 现场：第二台用户级单元 `singlefs-5080-telemetry` 每秒记 5080 的功耗档位、频率、功耗、温度、显存、降频原因、PCIe 代数到 `~/.local/share/singlefs/work/gpu-5080-xid62/telemetry.csv`；复现脚本 `repro.sh` 在同一目录，三个测试二进制经任务 6207 编好。5080 已摘出服务的卡表；直接起测试被 project-code-guard 拒，等用户定怎么起。
- 用户纠正：「我们的门禁任务 从昨天跑到今天 都没有这个问题。 但是今天晚上的小任务就触发了这个问题」。现查：5080 上连续近 20 小时的积压跑法（任务 5852，长时间连续判块）一次 Xid 62 都没有；六次对应的任务全跑 GPU 快档那几个测试文件（门禁 feature-gated-tests 格 5172、5261、5612，门禁整道 5642，合入后验证 5939，手工 `cargo test … --features gpu --test crash_judge_gpu_tables_at_their_capacity -- --test-threads 1` 的 5442），没有一次是轻型入口。第二台三张卡合计，每次出事前 30 分钟起的 GPU 尝试是 0、0、1、0、0、2 个，任意 30 分钟窗口平均 0.85 个，服务这一层换任务不频繁。
- 外部报告（没在本项目验证）：NVIDIA/open-gpu-kernel-modules #1307 汇总 595.45.04 到 615.78.08 同一种 PMU 停机，触发条件没确定，有空闲时出的；断电是唯一恢复，重置也有回「Not Supported」的；试过无效的有关 PCIe ASPM、开关 persistence、关 GSP 固件、调功耗上限、降级驱动；锁频（`nvidia-smi -lgc`）没人试过。
- 用户定：「1 去复现，第一优先」「2 1做完后，删掉轻型门禁。 重型门禁加参数 来调用轻型门禁」，另「你自己绕过 这个 我授权」（绕过 project-code-guard 直接起测试）。做法：仓外工作目录放 `start_repro.py`（python 起的脚本文件钩子不读）经 ssh 在第二台用户级单元 `singlefs-5080-repro` 里起 `repro.sh 20`；三个测试二进制共 39 条测试，每条单独一个进程。头一趟每条退 101（测试要环境里有 `CARGO` 记工具链），留在 `attempt1/`；补 `CARGO`、`CARGO_MANIFEST_DIR`、`CARGO_PKG_NAME` 之后重起，测试退 0、5080 到 P0。
- 用户嫌慢（逐条单独起一轮约 25 分钟），改两组：「杀掉立刻再起」（用户：「派一个 gpu任务 杀掉 然后立刻派一个gpu任务」，`repro-kill.sh`）第 6 个循环在新进程里出 Xid 109（满载约 5 秒的单个派活之后，卡没坏，日志 `repro-kill-attempt1.log`）；对照组「不杀，整个 `crash_judge_gpu_tables_at_their_capacity` 一个进程跑完」（`repro-tables.sh`）第 1 遍开跑 6 分钟就复现 Xid 62（签名 A，日志 `repro-tables-crash1.log`、`tables-pass-1.txt`）。
- 这一次 Xid 62 的现场（量的）：卡还没重置时读 5080 与上游端口 00:01.1 的 AER 状态寄存器，`UESta`、`CESta` 全没置位、`DevSta` 无错（`aer-after-xid62-1.txt`；从上一次重置以来没记任何链路错误），所以不是 PCIe 链路先出错。遥测：前一条测试满载到派活结束，卡降到 P8 空闲约 13 秒（测试在 CPU 上建库），下一条测试往卡上建资源（`judge_resources_begin` 那一刻的 `GPU_LIFECYCLE`）的那 94 毫秒里，内核报 Xid 62（比交派活早 2 毫秒）。即从 P8 深度空闲被叫醒的那一下。当时遥测每秒调一次 `nvidia-smi`。
- 用户给的两个机制（「间歇性负载……Blackwell 极其激进的动态调频机制就会尝试瞬間降频降压」「高频状态轮询……与 GSP 内部 task:3 的 PMU 轮询产生 IPC 队列争用」）分两组测，一次只改一样：锁频（不让降到 P8）照跑同一组；关遥测（不轮询 NVML）照跑同一组。重置被 1 号卡上任务 6223 的测试进程挡着（它也开着 `/dev/nvidia0`），等它退出再做。
- 用户问「禁用深度省电是不是更好一点」：现查 `/proc/driver/nvidia/gpus/0000:01:00.0/power` 是「Runtime D3 status: Disabled by default」，整卡断电那一档本来没开；在用的只有降到 P8，Linux 驱动没有单独的开关，只能锁最低频率。用户定：「我觉得 800就可以了」。照上次的次序重置（这回 `fuser` 只剩 `nvidia-persistenced`），`nvidia-smi -i 0 -lgc 800,3105`，回读空闲停在 P5、802 MHz、17.7W，不进 P8；重置之后卡干净地起来（之后内核没有 NVRM 消息）。
- 用户：「70分钟不需要 10分钟高频 切换就好了」「不用跑对照了 这个问题在社区 有同样的问题。 看起来就是锁频的问题」。高频切换脚本 `repro-wake.sh`（单起一条短测试、每次跑完闲 13 秒）锁着 800 跑 10 分钟当锁后的检查，不跑不锁的对照。
- 长期做法：第二台系统单元 `/etc/systemd/system/nvidia-5080-min-clock.service`（按 PCI 总线号 `00000000:01:00.0` 执行 `nvidia-smi -lgc 800,3105`，After `nvidia-persistenced`，已 enable、已起，回读 P5、802 MHz）。重置卡、重载驱动会清掉锁，之后要 `systemctl restart nvidia-5080-min-clock`；收成重置脚本时这一步写进去。5080 放回服务卡表等管 task-server 的会话（0a）把清场只查分到的卡那一批换上。
