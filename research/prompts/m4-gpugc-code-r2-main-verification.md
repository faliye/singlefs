# m4-gpugc-code-r2：七·3 GPU 垃圾回收代码轮第二轮判决（2026-11-01）

正文 `research/prompts/_m4-gpugc-code-r2-body.md`；辩方腿 `research/prompts/m4-gpugc-code-r2-sonnet-output.md`（sha256 `f086cfee62606a0036a678c2f948dee599e90fdd7aff3ff5fdf72680199ce8c4`）；攻方腿 `research/prompts/m4-gpugc-code-r2-opus-output.md`（sha256 `85a7d5508b2e5f6c3cf00c66590faf34eaede499a2e174b7c69d09b834a1f1d6`）。被判的是第 54、55 批合入的 9 份产品代码文件：`crates/singlefs-background-gpu/src/adapter_inventory.rs`、`crates/singlefs-background-gpu/src/background_gpu.rs`、`crates/singlefs-background-gpu/src/background_gpu_compute.rs`、`crates/singlefs-background-gpu/src/gpus_of_this_host.rs`、`crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs`、`crates/singlefs-background-gpu/src/region_record_kernels.rs`、`crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs`、`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-core/src/mounted_session.rs`。下面的行号都取冻结副本 `~/.local/share/singlefs/work/m4-gpugc-code-r2/repo/`。

## 一、核查

- 快照倒推：`task-server/task submit --cwd ~/.local/share/singlefs/work/m4-gpugc-code-r2-main-check -- python3 <仓根>/scripts/agents/snapshot-tree.py m4-gpugc-code-r2 --out ~/.local/share/singlefs/work/m4-gpugc-code-r2-main-check/m4-gpugc-code-r2-tree --repository <仓根>`（任务 2482）：清单 14 份，倒推出 13 份、sha256 都对上；倒推不出的 1 份是 `crates/mutations.tsv`，快照之后别的会话改过它（差异存在 `~/.local/share/singlefs/work/m4-gpugc-code-r2/mutations-tsv-after-snapshot.diff`，14 行，没有一行以「七·3」起头；另一会话告知的两份补丁清单记在调度记录）。冻结副本对 `research/prompts/m4-gpugc-code-r2-snapshot/sha256sums.txt` 14 份全对上，所以引文核对用冻结副本当根。
- 引文：`python3 scripts/agents/cite-check.py research/prompts/m4-gpugc-code-r2-sonnet-output.md research/prompts/m4-gpugc-code-r2-opus-output.md --root ~/.local/share/singlefs/work/m4-gpugc-code-r2/repo --background research/prompts/_m4-gpugc-code-r2-background.md`，末行「✓ 核了 129 处引文、48 处裸行号，对上 128 处，没判 49 处（其中裸行号没有引文可比 48 处；逐处列在上面）」；没判的那一处引文是 `research/prompts/_m4-gpugc-code-r2-body.md:26`，正文在仓里、不在冻结副本里。判决要用的裸行号逐处读码核过（下面各条写出）。
- 复跑：攻方六条探针，模型目录拷到 `~/.local/share/singlefs/work/m4-gpugc-code-r2-main-check/opus-model/`（三份文件 sha256 与报告开头相同），经服务交 `rerun.sh`（任务 2451，本机）：「test result: ok. 6 passed」，ATTACK 9 行；脚本退 1 只因腿把 ATTACK 写成 10 行，读数不受影响。辩方探针（任务 2420）被服务派到第二台，命令引的本机草稿目录在那边不存在，退 127，没跑成；它要量的重交记账已由攻方探针同一处量到。辩方第八节第 2 条（证红两行变异）不跑：被引的成员在源码里已一处都没有，静态就定了（第二节 T3）。

## 二、判定与去向

### T1　第一轮的修补本身

| # | 发现 | 现查 | 判 |
|---|---|---|---|
| 1 | 切换校验分两次枚举本机的卡，两次答得不同时，点名一张挂载时没开成的卡的切换被收下，交给实现时 panic（攻方 T4，第一轮第 2 条的改法带出来的） | `background_compute.rs` 第 920 行文档写「卡清单只读一次」，第 929、930 行两个字段各调一次 trait 方法；外壳 `background_gpu.rs` 第 2182、2192 行每次都现枚举，枚举超时答空（第 630 行）。探针原行：`ATTACK name=switch_naming_a_card_not_opened second_enumeration_empty=true admitted=true outcome=panicked:核心层照卡清单校验过：参与的卡都在本机、都不是挂载那一刻没开成的`；对照行 `second_enumeration_empty=false admitted=false outcome=refused:ParticipatingGpuDeviceNotOpenedAtTheMount(…)` | 打中，采。条款第 4 条要拒切换，代码 panic、锁中毒 |
| 2 | 重交次数在等前一次超时提交那一等又超时时照样加一，记账偏大（两条腿都报） | `waiting_for_the_gpu.rs` 第 206–211 行。探针：`earlier_waits_timing_out=2 … resubmissions_actually_made=1 resubmissions_after_timeouts=3`；`earlier_waits_timing_out=3 … resubmissions_actually_made=0 resubmissions_after_timeouts=3` | 打中，采。判卡用不了的时机（第 4 次等超过上限）照条款对；错的是第一轮第 8 条要的记账「重交了几次」 |
| 3 | 压一半重切之后，同一张卡上已做完的片从头再交（攻方 T1 第 5 条） | 探针：`ATTACK name=free_memory_recut_redoes_completed_chunks … submissions_completed=8 pieces_submitted=8 recuts_after_running_out_of_memory=2 submissions_made_on_the_card=8 chunks_of_the_cut_at_264_bytes=Ok(6)` | 打中行为，判定逐条照常交回；条款没写重切时已做完的片留不留，交用户（第五节） |
| 4 | 接手的卡把同一种建不成再撞一次 | 探针：`card_taking_over_hits_the_same_shortage_again … recuts_after_running_out_of_memory=5 … submissions_made_on_the_card_short=0 submissions_made_on_the_card_taking_over=8 devices_given_work=2` | 结果照常交回，条款「这一批不给这张卡分活，它那份转给别的参与的卡」照做；接手的卡自己再压一半是它自己的空闲显存不够，不违约。不采 |
| 5 | 两份回读暂存建在只接显存不足的错误范围之外 | `background_gpu.rs` 第 926–950 行 `buffers_created_within_the_free_memory` 只包 `create_the_buffers`；第 970 行 `dispatch_both_kernels_and_read_back` 里的两份 `MAP_READ` 缓冲在范围之外建 | 读码坐实，没量（要真卡）。采：显存不足时这两份建不成不会走条款第 4 条「这一批转给别的卡」那一支 |
| 6 | 量代价那一段建不成不按更小的片重切（第 911 行） | 第 905–914 行量代价直接调 `both_kernel_outcomes_of_a_checksum_chunk` | 不采：条款第 4 条只写「挂载那一刻量代价的提交超时不重交，算那张卡没开成」，量代价那一段建不成同样算没开成，是读法，不违约 |
| 7 | 切换时把超时后还在跑的提交算进已占显存，这张卡一直不用（攻方 T1 第 4、6 条） | 探针两行都是 `left_out=0 … batch_ok=true` | 没打中，不采 |
| 8 | 两次挂载同时开着时，后丢的那次泄漏账盖掉先丢的（攻方 T1 第 6 条） | `gpus_of_this_host.rs` 第 691 行一带；探针：`ATTACK name=two_mounts_alive_at_once bytes_leaked_by_the_two_mounts=900 bytes_leaked_on_the_card_in_the_process=300 account_of_a_third_mount_starts_at=300` | 打中，采 |
| 9 | 第一轮第 1–8、10 条的原反例在今天的代码上都不再成立（辩方 T1） | 两条腿各自逐条读码；攻方没打中的形状列在报告「没打中的形状」一节 | 采。每种没打中的形状只抽了一次样，不当「没有问题」用 |

### T2　两批替条款做的选择

| # | 项 | 判 |
|---|---|---|
| 1 | 第 55 批第 2 项后半：攒批会话判过起跑之后排空待发没发成，交回 `Err`，不报警、不计叫起（两条腿都报） | 打中，采。`mounted_session.rs` 第 5590–5603 行 `Err(refusal)` 直接 `return`，第 5615 行取报警那一句走不到；条款第 4 条（`.claude/kb/decisions/26-后台整理与放置回收.md` 第 248 行）只豁免「入口在任何读写之前就拒的那一次」，这一次已判过起跑、发过一次发布，不在豁免里 |
| 2 | 第 55 批第 1 项：出错停下的那一趟就数作第 1 次；在叫起里停下与在挂载续做里停下数法对不上 | 读法，交用户（第五节）：条款写「停下以来叫起了几次」，出错那一趟算不算「停下以来」没写 |
| 3 | 第 55 批第 4 项：`#[must_use]` 只抓整份丢掉，接住不读报警字段编译器不管；`crates/` 下 `tests/` 之外没有读报警字段的地方（辩方），`e185_gc_batch_wall_clock_split.rs` 与 `history.rs` 接住不读（攻方） | 不采：条款第 4 条写「通道是返回值里必须处理（`#[must_use]`）的报警…调用方不处理编译就警告」，代码照写；两处接住不读的都是装置与测试代码，不是产品调用点 |
| 4 | 其余十一项 | 辩方逐项判「只是读法」，两项（第 54 批第 1、3 项）条款已由用户 2026-10-31 定案写进第 248 行；攻方没逐项攻第 55 批第 5–7 项与第 54 批七项。照收，这一轮只抽了一次样 |

### T3　产生结论的方法

| # | 发现 | 判 |
|---|---|---|
| 1 | `crates/mutations.tsv` 第 5787、5788 行（冻结副本行号）两行替换文还构造第 54 批删掉的 `InputNotSupportedOnTheGpu::TablesLargerThanABindingOnTheDevice`（辩方） | 采。`grep -rnF TablesLargerThanABindingOnTheDevice` 在冻结副本与主工作区 `crates/` 下都只剩这两行，源码 0 处，替换之后必编不过，整表复跑会记「无效」；两行不在任务 2170 选的 370 行里（日志 0 处，`.claude/gate.d/crates-mutation-light-rows.tsv` 0 行），所以轻型没报出来 |
| 2 | 停着表 `EVERY_MEMBER` 手写，加成员只逼着补两个 match 臂、不逼着补进表（辩方） | 采。`background_compute.rs` 第 49 行手写长度 1 的数组，第 68–75 行编译期只核表里已有的成员；第一轮第 3 条验收「加成员不改它就编不过，或有一条对每个成员断言的用例」没做到 |
| 3 | 显存不足的错误范围（`background_gpu.rs` 第 934 行）与切换时的现枚举（第 622 行）两处没有变异行；这两批追加的 55 行变异都是替身行，要真卡的 6 行与第 55 批真卡用例没跑 | 采变异行缺口；真卡那一半已在验证段待跑 |
| 4 | 第一轮的「打中」复核：「超时按片计」那一反例只排掉「按片计」，不打中「一段一次调用」；选哪一种是用户 2026-10-31 定的（辩方） | 采，第一轮判决不改 |
| 5 | `region_scan.rs`、`checksum.rs` 两处快写法这一轮还是只有一份样本（sha256 与第一轮开工快照相同，不在这一轮的格里） | 记：第三轮或七·4 覆盖时补一份样本 |

### T4　从未被看过的地方

| # | 发现 | 判 |
|---|---|---|
| 1 | 叫起计数只有一个加一点，四个调用点各调一次，`&mut self` 下别的叫起插不进来；漏计就是 T2 第 1 条那一支（辩方） | 采；修 T2 第 1 条一并关 |
| 2 | 一批放弃时 `GpuWorkOfTheBatch` 的记账交回、不丢（辩方）；攻方同样没造出丢的 | 采 |
| 3 | 现查清单枚举超时当一张都不在，拒切换；选 `cpu` 的切换不读清单（辩方） | 采；两次枚举答得不同那一种见 T1 第 1 条 |

## 三、交给下一批的实现员（按文件聚簇）

一簇，文件有交集：`crates/singlefs-background-gpu/src/`（`background_gpu.rs`、`waiting_for_the_gpu.rs`、`gpus_of_this_host.rs`）、`crates/singlefs-core/src/background_compute.rs`、`crates/singlefs-core/src/mounted_session.rs`（第 5590–5615 行一带，改之前先发消息给在改这份文件第 5209–5320 行的那一路）、测试与 `crates/mutations.tsv`。

| # | 问题 | 验收 |
|---|---|---|
| 1 | 切换校验读两次卡清单（T1 第 1 条） | 一次切换只现枚举一次，判与取参与的卡读同一份；替身用例「第一次枚举全在、第二次枚举为空」点名挂载时没开成的卡，切换被拒（`ParticipatingGpuDeviceNotOpenedAtTheMount`），不 panic；一行变异（改回各读一次）要红 |
| 2 | 重交次数记偏大（T1 第 2 条） | 「重交了几次」只数真交出去的重交；替身用例钉两种时序的绝对值：交活前先等那一等超时 2 次、实际重交 1 次，记 1；超时 3 次、实际重交 0 次，记 0，同时照旧报 `UnusableAfterResubmissions`；一行变异要红 |
| 3 | 排空待发没发成不报警、不计叫起（T2 第 1 条） | 停着时，攒批会话判过起跑、排空待发被拒的那一次叫起照样计一次叫起；交回 `Err` 时带上报警，或改成别的不丢报警的形状；harness 用例造停着 + 排空被拒，断言这一次的报警在、`wakes_while_stopped` 连续；一行变异要红 |
| 4 | 两次挂载同开时泄漏账被盖掉（T1 第 8 条） | 进程里同一张卡的泄漏字节按次累加，不覆盖；纯函数用例钉「两次各 300、600 字节，记 900」；一行变异要红 |
| 5 | 回读暂存在显存不足的错误范围之外（T1 第 5 条） | 两份回读暂存与别的缓冲在同一个错误范围里建，建不成同样走「这一批转给别的卡」；替身层够不着就写真卡用例，命令交主 agent；一行要真卡的变异 |
| 6 | 变异表两行构造已删的成员（T3 第 1 条） | 「七·3 后台 GPU 包：实现把乙段 / 丙段 GPU 出错（含超时）报成第一版不支持的输入」两行的替换文改成今天还存在的错误映射，`crates-mutation-rows.py static-check` 过、证红 |
| 7 | 停着表加成员不逼着补表（T3 第 2 条） | 加一个成员而不补进 `EVERY_MEMBER` 时编不过，或有一条对每个成员断言的用例会红；写法实现员定，报告写明怎么证的 |
| 8 | 两处没有变异行（T3 第 3 条） | 显存不足的错误范围（`background_gpu.rs` 第 934 行一带）与切换时现枚举（第 622 行一带）各补一行会红的变异；要真卡的标真卡 |
| 9 | 出错停下那一趟不算叫起（第五节，用户 2026-11-01 定「改代码」） | 出错停下的那一趟报警里 `wakes_while_stopped` 不计这一趟，下一次叫起才是 1；挂载那一刻就停着的，头一次叫起是 1；在叫起里停下与在可写挂载续做里停下数法相同；harness 用例钉「出错停下之后连着叫起三次」报 1、2、3，两种停下各一条；原来钉 2、3、4 的用例跟着改；一行变异要红 |

## 四之一、第 59 批交回之后用户定（2026-11-01 弹窗）

- 第三节第 5 条：实现员报回读暂存是 `MAP_READ` 缓冲、替身与真卡都造不出建不成（推的，读 wgpu-hal 源码），改法照留、测试与变异行停下；用户定「造不出就换方法继续造」，并进第 60 批。
- 第三节第 9 条：可写挂载续做里停下那一种 harness 造不出（带后台计算设置的挂载入口恒给 `CompactionSwitches::PRODUCT`），先由核心层单测钉数法；用户定「下一批补 mount.rs 入口」，并进第 60 批。

## 四、复跑与核不动的

- 攻方六条探针主 agent 复跑过（任务 2451）；辩方探针没跑成（任务 2420 被派到第二台），它要量的重交记账由攻方探针同一处量到。
- T1 第 5 条（回读暂存）只读码，没量，要真卡。
- 两条腿都没起 GPU；真卡与 WGSL 这一轮仍只读码。
- 攻方「没打中的形状」每种只抽了一次样；辩方 T2 十一项「只是读法」只有一份样本。

## 五、交用户

| 岔路 | 事实 | 候选 | 用户定（2026-11-01 弹窗） |
|---|---|---|---|
| 压一半重切时已做完的片留不留 | 显存不足重切之后同一张卡上已做完的片从头再交（探针 6 片交了 8 次）；判定逐条照常；条款第 4 条对超时写「只重交超时的那一片，同一批已做完的片留着」，对空闲显存不足重切没写 | 已做完的片留着、只重切没做完的（与超时同口径，改 `crates/` 与条款）/ 照今天整段重切（条款补一句） | 「重新切 不留，不然谁知道对不对 完整不完整。已经做完的既然不满足条件就该被视为是错的。」：代码照今天整段重切，条款第 4 条补一句 |
| 出错停下那一趟算不算叫起 | 第 55 批数法：出错停下的那一趟就报第 1 次；挂载那一刻就停着的，头一次叫起报第 1 次；条款写「停下以来叫起了几次」 | 照代码，条款补一句「出错停下的那一趟算第 1 次」/ 改代码，那一趟不算、下一次叫起才数第 1 次 | 「改代码」：出错停下的那一趟不算，下一次叫起才数第 1 次；并进第三节第 9 条 |

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-01 不受影响：这一轮打中的都是代码没照条款做（第 4 条拒切换、记账、每次叫起都报），条款不改；第五节两条交用户，定了再改 |
