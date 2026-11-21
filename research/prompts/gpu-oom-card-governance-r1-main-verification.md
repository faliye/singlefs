# gpu-oom-card-governance-r1：判决（2026-11-19）

正文 `research/prompts/_gpu-oom-card-governance-r1-body.md`，开工快照 `research/prompts/gpu-oom-card-governance-r1-snapshot/`。这是这一题的第一轮，治理文件轮，照 `.claude/rules/implementation-workflow.md`「代码轮与治理文件轮怎么收」收：有坐实的打中就改，改完再攻一轮，最多三轮。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 正推（sonnet，第 1 到 4 问） | `research/prompts/gpu-oom-card-governance-r1-sonnet-output.md` | 第 1 问「不成立（两句）、推不出（一句）」；第 2 问读代码成立、真卡那一份 `after_a_panic` 没有测试调过；第 3 问成立，另报登记之后大卡坏了再入池的块挂着；第 4 问四环字面成立，另报几处文档对不上 |
| 攻方（opus，第 5 到 12 问） | `research/prompts/gpu-oom-card-governance-r1-opus-output.md` | 第 5、7、8、9、11 问打中；第 6 问打中一半；第 10、12 问不成立；「最脆弱的一处：真卡那一份 after_a_panic」 |

核查命令（原样）：

- `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py gpu-oom-card-governance-r1 --out /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-tree --repository /home/fy5090/code/singlefs`：结局 `type=command-failed exit=1`，「清单点名 19 份，倒推出 18 份、sha256 对上 18 份；倒推不出 1 份」。倒推不出的是 `task-server/task_server_selftest.py`：开工之后改任务服务的那一路会话换上了它那一批，改动没进 git，diff 里没有它的基底。正推腿引它一处（自证格 `health-gpu-card-failed-notified` 的登记），主 agent 在此刻那一份里人工核，登记照在；攻方腿没有引它。
- `task-server/task submit -- python3 scripts/agents/cite-check.py research/prompts/gpu-oom-card-governance-r1-sonnet-output.md research/prompts/gpu-oom-card-governance-r1-opus-output.md --root /home/fy5090/.local/share/singlefs/work/gpu-oom-card-governance-r1-tree --background research/prompts/_gpu-oom-card-governance-r1-body.md`：结局 `type=succeeded exit=0`，「✓ 核了 143 处引文、150 处裸行号，对上 128 处，没判 165 处」，没有对不上的。没判的多是 wgpu 源码与没有引文可比的裸行号；判决要用的那几处由主 agent 逐处人工核，写在「二、判决」各条下。

主 agent 复跑的读数：

- 正推腿的替身探针：`task-server/task submit --cwd <正推腿的仓副本> -- cargo test -p gpu-verification-server --test service_loop_with_judges_off_the_card -- probe_a_block_only_the_failed_big_card_could_hold_is_refused_not_left_hanging --exact`，结局 `type=command-failed exit=101`，红在它要验的那一条断言「大卡坏了之后再入池的块 3 秒里没有回话：没有哪张没坏的卡装得下它，却没被拒、挂在池里」；同一份日志里第 122 行两处 panic 是替身照设定造的设备丢失。
- 攻方腿的替身探针五条，在它冻结副本的拷贝上复跑（`research/prompts/gpu-oom-card-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh`），结局 `type=command-failed exit=101`，「4 passed; 1 failed」：
  - `OOM_ATTACK_PROBE name=put_back_blocks_serialized blocks=40 out_of_memory_caught=1 put_back=40 waves=41 lone_waves_after_the_first=40 loads_on_card0=41 evictions_on_card0=39 judged_runs_on_card1=0 judged_states=40 blocks_judged=40`
  - `OOM_ATTACK_PROBE name=judge_panic_fails_every_card cards_failed=[CardIndex(0), CardIndex(1)]`
  - `OOM_ATTACK_PROBE name=facts_job_retry_empties_the_other_card … verified_card_and_judges_on_it=[(0, 0)] evicted_on_card1=0 answered_card=[CardIndex(0)]`：攻方腿预计重核派到卡 1、换出卡 1 的判器，实测派回卡 0、卡 1 换出 0 个，这一条的断言红，攻方的推论被推翻。
  - `out_of_memory_then_drain`：6 块各判一次；`split_block_put_back`：跨卡的块整块放回、判一次。
- 正推腿另写了两条上真卡的探针（`--features gpu`），任务不拿卡（`.claude/rules/verification.md`「卡只归 GPU 服务，任务不拿卡」），这一轮没跑，核不动；它们问的「真卡那一份 `after_a_panic` 在卡上答什么」照判决第 6 条记成欠的测试。

## 二、判决

坐实的（主 agent 现查过代码或复跑过探针）：

1. **OOM 之后显存没有当场还回去**（两腿都打中）。wgpu-core 30.0.1 `src/resource.rs` 第 1032、1033 行把暂存缓冲交给 `pending_writes.consume` 与 `insert_buffer`，`src/device/queue.rs` 第 445 到 447 行 `consume` 只把它推进 `temp_resources`，要等下一次提交做完才放。所以 `gpu-verification-server/src/card_worker.rs` 第 7 行「判器缓存已丢掉卡上全部判器（显存还给驱动）」说过头了：丢掉判器之后，同一张卡上紧接着单独重判又先建缓冲，前一波装的表与暂存可能还占着显存。
2. **在腾空的卡上又 OOM 的块没人判、也没人知道**（攻方第 5 问，工具员局限第 2 条）。`ServiceRefusal::OutOfMemoryOnAnEmptiedCard`（`gpu-verification-server/src/service_loop.rs` 第 82 行）报拒之后，这个进程不再认领这一块，待判行留在库里；判段只问 `control serves`，`control check` 不读这种情形，等它的那几片一直等到被任务服务打回。
3. **核对器的六张事实表也有暂存，账上只算一份**（攻方第 7 问）。`crates/singlefs-verification/src/crash_verify_gpu.rs` 第 552 行起用 `create_buffer_init` 建表，`gpu-verification-server/src/card_memory_ledger.rs` 第 150 到 160 行 `mebibytes_of_a_facts_verifier` 只算一份表，与判器那边按两份算（`mebibytes_while_loading_a_judge`）不一致。
4. **因显存不够丢掉的设备被当成卡坏**（攻方第 8 问后一半）。验证包把这种丢失认成显存不够（`crates/singlefs-verification/src/gpu_unit_checks.rs` 第 363 行起 `cause_of_the_device_loss`，与 `wgpu::hal::DeviceError::OutOfMemory` 那一句相同的记成 `GpuMemoryShort`）；服务的 `after_a_panic`（`gpu-verification-server/src/gpu_judges_on_a_card.rs` 第 301 行起）只看丢没丢，丢了就报卡坏，这张卡停到服务重起、`control check` 还叫人重置卡。这与用户定案「OOM 需要倍catch 不能影响好的卡」相违。
5. **判法自己的 panic 被报成卡坏，能接连弄坏每一张卡**（攻方第 8 问前一半，工具员局限第 1 条）。复跑读数 `cards_failed=[CardIndex(0), CardIndex(1)]`：一条装判器就断言失败的流，被拒的块回库、租期过了又被认领，先后弄坏两张卡；两台都会。
6. **真卡那一份 `after_a_panic` 没有任何测试调过**（两腿都点名）。替身判器只有一个标记，真卡那一份读 `DeviceConditionOfACard` 的逻辑没有被执行过的证据。上卡测试照规矩交 GPU 服务代跑，接口待定，记成欠的测试，不阻塞改法。
7. **撤回的块排成一串、好卡帮不上**（攻方第 6 问后一半）。复跑读数 40 块放回、41 波、卡 0 换出 39 次、卡 1 判放回的块 0 次。块没丢、没重判，但卡 0 一块一波重传表，卡 1 空着不帮。
8. **大卡坏了之后，只有它装得下的块挂在池里**（正推第 3 问）。复跑红在「大卡坏了之后再入池的块 3 秒里没有回话」。`service_loop.rs` 文件头「池里不留永远派不出去的态」这一句在这一轮之前就有（`git show HEAD:gpu-verification-server/src/service_loop.rs` 第 9 行），不是这一轮带进来的，但同一份文件归这一轮改，一并修。
9. **卡坏告警在读不到时当成好了**（攻方第 9 问）。`gpu-verification-server/control` 的 `cards_failed_answer` 不论第二台连不连得上、这一台日志读不读得了，都退 0；`task-server/task_server_lib.py` 第 2204 行起 `failed_cards_of_the_report` 只收卡坏行，读不到就是空表，任务服务把它当成好了，下一回读到又给每个会话重发一遍；任务服务重起后头一轮同样重发。
10. **卡坏行的字段改名时三份自证都还是绿的**（攻方第 11 问）：任务服务取卡号退成 `?`，同一个进程的几张坏卡并成一个键，后坏的漏报。
11. **文档与代码对不上**（正推第 1、4 问）：`card_worker.rs` 第 19 行「核的时候 panic 同判的时候 panic：报这张卡坏了」与第 6 行（核的时候 OOM 同样接住）和代码相反；`gpu-verification-server/README.md` 说撤回的块「先腾空那张卡再核」，代码派给换出最少的那张空卡；README 第 23 行「设备丢了才报卡坏」漏了判法自己 panic 也报卡坏；`gpu-verification-server/control` 第 311 行注释点名的 `GPU_CARDS_FAILED_READ_PREFIX` 在任务服务里不存在。

不成立、推翻或不采的：

- 攻方第 6 问前一半（卡 0 OOM 那一刻卡 1 的账与驻留判器被动到）：不成立，复跑「卡 1 换出 0 个」。
- 攻方「放回的核对组把卡 1 的判器全部换出」：复跑推翻，重核派回卡 0。
- 攻方第 5 问的判两次、写错卡、写错流、停止信号、借卡：复跑 `out_of_memory_then_drain` 6 块各判一次、`split_block_put_back` 整块放回判一次，不成立。
- 攻方第 10 问（两条 `UNKEYED_PROGRAM_WORDS` 登记）：不成立。两条登记的替身取法只给自证用，平常那一种字面列表照连进键，与已有的同形登记（`check_prune_server` 里的 `run`、`version_mismatch_lines`）一致。
- 攻方第 12 问（多机撞键、漏一台）：不成立。
- 正推第 2 问三种情形在代码上与文件头一致：成立，但没在真卡上执行过，并进判决第 6 条。
- 正推第 3 问「新表两份」三处调用、驻留账一份、对拍测试比的同一个量：成立。

## 三、改法

下面九条都是判决里推的改法，攻过零轮；全部要写进产品代码，照 `.claude/rules/three-way-inference.md`「腿提的、判决里标「零轮」的改法，写进产品代码之前先在探针里对它做 A/B」，由下一批工具员把每条做成替身判器上「没有这一改动就红」的用例与弄坏开关，主 agent 经任务服务跑出两臂读数，读数记在这一节末尾。

1. 判决第 1 条（零轮）：丢掉判器之后、单独重判之前，先在那张卡上做一次空提交并等它做完（让待写列表里的暂存与表放掉），再建缓冲；`card_worker.rs` 第 7 行改成照实写。
2. 判决第 4 条（零轮）：`after_a_panic` 认出「因显存不够丢的设备」（照验证包 `cause_of_the_device_loss` 同一个判法，不另抄一份），这种情形重开这张卡的设备、卡不进 failed、只打接住 OOM 那一行；别的原因丢的才照旧进 failed。
3. 判决第 5 条（零轮）：判法自己的 panic 不算卡坏，算这一条流的毛病：这一块报拒、卡照常用；之后怎么处置同第 4 条，交用户定。
4. 判决第 2 条（零轮）：在腾空的卡上又 OOM 的块，处置交用户定。
5. 判决第 3 条（零轮）：核对器借显存照「表两份」算，与判器一致。
6. 判决第 7 条（零轮）：撤回的块不只等撤回它的那张卡：别的没坏、空着、装得下的卡也可以拿；只在哪张都装不下时才等那张卡腾空单独判。
7. 判决第 8 条（零轮）：一张卡坏了之后，池里只有它装得下的流的块当场报拒，不挂着。
8. 判决第 9、10 条（零轮）：`control cards-failed` 读不到（unreachable、unreadable）时打明一行「没读成」，任务服务据此保留上一回的卡坏、不当成好了；告警去重落盘，重起之后不重发；卡号取不到就报错，不退成 `?`；自证另加一格，卡坏行字段与任务服务的解析逐字段对。
9. 判决第 11 条：四处文档照代码改。

A/B 读数：任务 <号>

## 交用户的

- 判决第 2 条（在腾空的卡上也 OOM 的块）与第 5 条（判法自己 panic 的流）怎么处置：这一块这一张卡确实判不了，照用户定的「配 GPU 必用 GPU，做不到报错不降级」不退回 CPU。候选：甲，放掉认领、让另一台的服务去试，并打一行可数的「判不了」行，`control check` 判红、任务服务发告警给交任务的会话；乙，只打行报红、不放认领，待判行留在库里等人处置。主 agent 推荐甲。用户 2026-11-19 弹窗选「放给另一台去试，并报红 (Recommended)」，改法第 3、4 条照甲做。
- 判决第 6 条（真卡那一份 `after_a_panic` 没在卡上跑过）：上卡测试归 GPU 服务代跑，接口待定，记成欠的测试。

## 这一轮被判过的文件

- research/prompts/_gpu-oom-card-governance-r1-body.md：正文点名的十二问逐问有判定，「实现今天的样子」里「显存还给驱动」那一层意思被判决第 1 条改正。
- gpu-verification-server/src/card_worker.rs：判了接住 OOM 那条路与文件头逐句对照；第 7 行（显存还给驱动）与第 19 行（核的时候 panic 报卡坏）判为不对，改法第 1、9 条。
- gpu-verification-server/src/gpu_judges_on_a_card.rs：判了真卡那一份 `after_a_panic` 三种情形的答法，与文件头一致；因显存不够丢设备被当卡坏（判决第 4 条），没有测试调过（判决第 6 条）。
- gpu-verification-server/src/service_loop.rs：判了撤回、单独判、报拒、核对组放回、设备丢失进 failed；撤回的块排成一串（判决第 7 条）、大卡坏了块挂着（判决第 8 条）、腾空卡上再 OOM 的块没人判（判决第 2 条）。
- gpu-verification-server/src/batch_scheduling.rs：判了 `tables_while_loading` 三处调用与 `plan_a_lone_block_on_an_emptied_card`，成立；单独判只等撤回它的那张卡归判决第 7 条。
- gpu-verification-server/src/card_memory_ledger.rs：判了 `mebibytes_while_loading_a_judge` 与驻留账，成立；核对器借显存只算一份表（判决第 3 条）。
- gpu-verification-server/src/serving_from_the_library.rs：判了三种行的前缀与字段、STOPPED 带 `cards_failed`，与 control、任务服务逐环对上；被拒块这个进程不再认领归判决第 2 条。
- gpu-verification-server/src/break_switches.rs：判了 12 个新开关是否两两弄坏同一处、点名那一条是否靠别的开关也会红，结论是两样都没有。
- gpu-verification-server/tests/service_loop_with_judges_off_the_card.rs：判了 OOM、空卡再 OOM、设备丢失、核对组、装判器五条新用例罩到的情形；判法 panic 被当卡坏、大卡坏了块挂着两种情形归判决第 5、8 条，要补用例。
- gpu-verification-server/tests/batch_scheduling_requirements.rs：判了「新表两份」那一条用例与四条改了额度的旧用例；不饿死与单独判两条路要补用例。
- gpu-verification-server/tests/library_pending_pool_on_a_temporary_library.rs：判了卡坏行与空卡再 OOM 两条新用例；被拒块待判行留库那一面归判决第 2 条。
- gpu-verification-server/README.md：判了「日志」「起停」两节与代码；撤回块等哪张卡、设备丢了才报卡坏两处对不上（判决第 11 条）。
- gpu-verification-server/control：判了 `card_failed_lines`、`cards-failed` 只读这一个进程的行，成立；读不到时照样退 0（判决第 9 条），第 311 行注释点名的常量不存在（判决第 11 条）。
- task-server/task_server.py：判了 `gpu_card_failures` 挂在服务健康上、至多 60 秒一次、另起线程、不拖住那一轮，成立；读不到当成好了与重起后重发（判决第 9 条）。
- task-server/task_server_lib.py：判了 `failed_cards_of_the_report` 的解析与开关 `gpu-card-failure-unwatched`；卡号取不到退成 `?`（判决第 10 条）。
- task-server/task_server_selftest.py：判了自证格 `health-gpu-card-failed-notified` 的登记照在；字段改名时它仍绿（判决第 10 条）。开工之后这份被别的会话换过，倒推不出，引到它的一处人工核过。
- scripts/gate/lib_function_units.py：判了第 13703、13729 行两条 `UNKEYED_PROGRAM_WORDS` 登记的理由，替身取法只给自证用、平常那一种照连进键，成立。

## 回看决策

不涉及决策：这一轮判的是 GPU 批处理验证服务与任务服务的运行行为和告警，不推翻、不确立 `.claude/kb/decisions/` 下的任何一条。
