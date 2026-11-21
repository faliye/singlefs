# m4-gpu-wait-outside-the-lock-design-r2 辩方腿（Sonnet）报告（2026-11-18）

立场：复核第一轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md`，逐条看「打中」够不够得着、是不是同样打中今天锁内等的做法与别的替代形态；替今天的做法（锁内等、只认 fsync）与「活意图那一支只核 R」辩护；逐格复核第三节形态（Q1 到 Q4）。
没读禁读清单（第二轮攻方腿的报告、模型、草稿目录）。引文行号一律在被引文件里 `grep -nF` 现取；设计轮没有冻结的树，`crates/` 读的是主工作区此刻的样子。
状态：模型两样（旧底 × 前台改动种类的探针、忙窗口基线测试）写好了，**没编、没跑**（子 agent 不起 cargo、不交任务）；只做了 `rustfmt --check` 的语法解析（无 `error` 行）与 `bash -n`、`py_compile`。凡是等这两样读数的结论一律「推的，待主 agent 跑」。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-model/`。步骤零（主 agent 直接跑，不经服务，建冻结副本，要带运行库测试用的共用搭建）：

```text
python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet/copy --with-test-inputs
```

之后整条交任务服务（包装先删掉运行目录，运行目录名以 `-run` 收尾）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-model
```

| 文件 | sha256 |
|---|---|
| `probe/Cargo.toml` | `453b4567273f2b874321b4aecb92130751207f4fd889609b1b3dc07d925bc094` |
| `probe/src/main.rs` | `f4ac3b2a4c995dc4fee20e2df5bca6ff58e1365715888c4138d248a97390d81e` |
| `probe/src/pool.rs`（第一轮攻方模型原样拷） | `d536c3f27449573c4a92e40f4f5cf5692122165c151cb6504805d339fcb114ac` |
| `probe/src/stale_base_kinds.rs` | `ff6d54bf5fde1582f90ffa9bf9a6fd534ffe27fb5bddde176c27db79ab03e45a` |
| `probes/mount_runtime_busy_window_foreground_wait.rs` | `d7d2fdc481721ce5cc42e33fe59396ae3c736607ca4bf79eb46cf268f72b13b9` |
| `tools/patch_the_copy.py` | `9e552c724c3ca045577a1cf481dc2bf48ce27df16e0e594c87c18473aa156b9a` |
| `tools/rerun.sh`（拷自 `scripts/agents/three-way-rerun-template.sh`，只换 `rerun_steps`） | `3f5bf7a225ccb40875c55874fe0735493e693dcc912bdda6d926a6de8e40fb5e` |
| `tools/rerun-from-a-clean-work-directory.sh`（第一轮攻方的原样拷） | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

探针的内存与时间（推的，没量过；比照第一轮同类任务日志）：第一轮攻方探针任务 19650 日志里 `MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=8943181824`、`MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=1398447223`、`step=peer-command event=end elapsed_seconds=90.8`（量的）；这里 210 段历史（第一轮 216 段、`WALL_SECONDS` 13.4）、最大 k 从 8 加到 48，估内存同量级、挂钟几十秒；忙窗口基线六种配置各 6 次重复，第一轮基线任务 19653 日志 `finished in 132.80s`（10 次重复，量的）折成每次重复约 13 秒，估六种配置共 8 分钟上下加编译。

## 判定一览（判决条目 / 辩方判定 / 证据）

判决文件里每一条的行号：`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md` 第 31、33、35、36、37 行是二节判决表的 Q2 弱判据、活锁、缺口 2、Q4、Q5 锁序行。

| 判决条目 | 辩方判定 | 证据（节） |
|---|---|---|
| 「会丢已答成功的发布」不采（依据 54 段历史 `lost_foreground=0`） | **够不着**：45 段旧底历史全部停在整理第一次发布的释放目标核上，前台改动只摆了一种、k 只到 8；该改写成「这一格没扫到，未定」 | 二 |
| 「回锁复核不比现行那一版的 txg 不行」 | **站得住**（落点是整理发布失败，不是丢失）；它的量依据 `arm=txg_guard` 够不着：那一臂 45 段都没整理，「不丢」是没整理的必然；这条同样打中每一个拆开的候选（共用前提），不打中今天 | 二、四 |
| 「其后 30 段历史里前台写被拒」 | **够不着**判决要的意思：探针把 `session.current` 留在旧底上（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:103`），整理被拒之后前台再发的写被拒是手造的；真第三段发布被拒不换 `current` | 二 |
| 改法 1「量过」（任务 19650 `arm=txg_guard`） | **站得住但只证了同义反复**：txg 不等就不整理，当然不丢；证不了「比 txg 就够」 | 二 |
| 缺口 2 撤回（基线不支持前台抢不到） | **够不着**：取样 56279 次里夹了发布的只有 46 次，均值与 p99 量的是维护线程闲着的那一段；单线程、200 微秒一睡、睡眠 5 毫秒一档；还缺的格见三 | 三 |
| 活锁「有」 | **存在性站得住**（结构论证与闭式不靠摆的数）；「大半作废」「要多大的下限」**够不着**（W、到达率是摆的）；不打中今天，打中每个整批作废的形态 | 四 |
| 破例批上必现 | **形态够得着、没量过**（两腿都是读码推）；同样打中所有整批作废的形态，不打中今天 | 四 |
| 让路只在取输入之前问 | **站得住** | 六 |
| Q5 锁序：切换在会话锁里等满整段（等 1595 对持 1600 毫秒） | **站得住（读数）**；「比今天更糟」**够不着**：今天前台等的就是整批（会话锁被批持着），切换这条路只是没变好；同样打中今天 | 五 |
| Q5 锁序：持着 GPU 锁回会话锁死锁 | **够不着第三节形态**（第 4 条已写「算完放掉再回会话锁」）；打中的是沿用今天守卫范围的写法 | 五 |
| Q5 卸载照样要等 | **站得住**；同样打中今天，不分辨臂 | 五 |
| Q5 把手锁中毒 | **站得住（推的）**；终态同今天（运行库报坏、要重挂），只多了一个毫秒级的窗 | 五 |
| Q4 严判据下没打中 | **站得住**，只在「严判据 + 回锁后在现行那一版上发」下；一次没打中不算数 | 七 |
| 「有活意图那一支只核 R」被整批作废盖过 | **该翻回「挂起」**：最脆弱前提在 `crates/` 里核得动一半，核出来的洞与处置见六 | 六 |

什么现象会推翻这一张表：见各节末「推翻条件」。

## 二、Q5 第一问：54 段取样够不够得着「丢失」那一格（主攻）

**取样怎么取的**（读 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs`）：
- `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:19`：「const FOREGROUND_PUBLISHES: [usize; 6] = [0, 1, 2, 3, 5, 8];」；`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:20`：「const PUBLISHES_AFTER_THE_COMPACTION: [usize; 3] = [0, 1, 3];」；`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:21`：「const UNITS_WRITTEN_BY_THE_FOREGROUND: [usize; 3] = [1, 2, 39];」。6 × 3 × 3 = 54 段每臂；k = 0 的 9 段前台没发，不是旧底，旧底段只有 45 段。
- 前台改动恒是同一种：`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/pool.rs:87` 的 `UserChange::WriteAtOffset {`，写第一个文件里一个单元的一个字节。「由用户决定的几步放开扫」只放开了 k、m、写哪个单元，没放开「发的是什么改动」。
- 旧底怎么造的：`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:103`：「session.current = snapshot;」，再走显式整理入口，整理在旧底上现算选段，不是「第一段算好计划、第三段照那份计划发」。

**读数**（`~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r1-opus/probe-run/out/stale-base.out`，现数）：
- 第 219 行：`RESULT name=stale_base arm=stale_base_live_intent_check histories=54 compacted_on_a_moved_version=45 lost_foreground=0 lost_after=0 panicked=0 compaction_refused=0 writable_remount_failed=0`。
- `grep -c 'arm=stale_base_live_intent_check k=[1-9].*outcome=Failed(Batch(Publish(ReleaseTargetAlreadyReleased_\]'` 打出 45：k ≥ 1 的 45 段，整理第一次发布全部 `published=0`，结局同一个 `Failed(Batch(Publish(ReleaseTargetAlreadyReleased_`；k = 0 的 9 段整理发成 6 次（`published=6 batches=3`）。单元取 1、2、39 与 k 取 1 到 8，结局逐字相同：这两根轴在这个结局上不分辨。
- `short` 把结局截在 100 个字（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:65`：「fn short(text: &str) -> String {」，`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:66`：「text.split_whitespace().collect::<Vec<_>>().join("_").chars().take(100).collect()」），所以被拒的是哪个落点（`ReleaseTargetAlreadyReleased` 带 `unit`）输出里没有。
- `txg_guard` 臂：`grep -c 'compaction=\[discarded\]'` 打出 45（对 k = 0 的 9 段是 9）：k ≥ 1 的 45 段那一臂**根本没整理**（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-model/probe/src/stale_base.rs:107`：「Arm::TxgGuard if snapshot_txg == txg_before_the_compaction => {」）；它的 `lost_foreground=0` 是没整理的必然，不是「比 txg 就够」的证据。

**这 45 段落在哪一格**：发布路径对每个要释放的落点核「分配记录在册、未释放」，`crates/singlefs-core/src/transaction.rs:5647`：「if record.is_released {」，下一行 `transaction.rs:5648` 报 `ReleaseTargetAlreadyReleased`。前台每发一次，就换下它改的那条路径上的旧节点并在分配器里记成已释放；旧底上的整理发布要释放同一批旧节点，于是撞。撞在写任何根之前（`published=0`，读回的与前台答过的一致），所以探针答的是「被释放目标核拦下的那一格不丢」，不是「旧底上发布做成了会不会丢」（推的：撞的具体落点要看没截断的结局，待主 agent 跑）。

**够不着的格**（逐条写为什么够不着、怎么扫）：

| 没扫到的格 | 为什么这一格可能不被释放核拦 | 扫法（`stale-base-kinds`） |
|---|---|---|
| 前台改动不换下旧底整理要释放的节点（写别的 inode、建 inode） | 整理发布抄上一版的实例表指针：`crates/singlefs-core/src/transaction.rs:11193`：「instance_table: InstanceTablePlan::Carry(previous.root.instance_table),」（整理一批发布不重写实例表，`publish_compaction_step_after_its_check` 内）。前台建 inode 重写了实例表，旧底的指针被抄进新根（推的） | `new_inodes`、`sequential_write_of_the_second_inode` 两种 |
| k 大到前台释放的旧节点过了 defer 窗口、被再分配给别的单元 | `.claude/kb/decisions/16-发布语义.md:35`：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」。根环 3S 槽（第一轮攻方报告写默认 S = 8，转述），k ≤ 8 够不着；再分配之后释放核看到的是新占用者的记录（未释放），跨度相同就放行，旧底发布会释放别人的单元（推的）。这一格正好是 W 长（卡挂住约 40 秒、前台一直在发）的历史 | k 加 24、32、48 |
| 活意图那一支 | 第一轮攻方自己写「只摆了『签发』那一支」（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-opus-output.md` 的「这条腿自己的限度」一节，转述） | 没扫，只写推翻条件 |
| 崩溃状态 | 只取了「写全落盘之后断电」那一态，没取发布中途的崩溃状态 | 没扫 |
| 计划是旧的、版是旧的 | 探针在旧底上现算选段；真第三段拿的是第一段算好的输入。两者差在分配器读数与隔离集 | 没扫 |

**判定**：够不着。「会丢已答成功的发布」该写成「这一格没扫到，未定」；「回锁复核比 txg」仍站得住，且不再依赖「丢失已出现」这一句：旧底整理发布被拒本身就是内部错误（整个调用交回错误，运行库送 `CompactionCallRefused`，`crates/singlefs-mount-runtime/src/lib.rs:630`：「self.deliver(&MaintenanceEvent::CompactionCallRefused(refusal));」）。
**推翻条件**：`stale-base-kinds` 里 `stale_weak` 臂在 `new_inodes`、`sequential_write_of_the_second_inode` 或 k ≥ 24 的段里 `stale_publish_made` 全 0 且 `refused_by_the_release_check` 等于段数，则释放核是普适网，第一轮「不采」成立（仍保留「要比 txg」）；`lost` 大于 0 则「不采」撤回，改法乙（发布路径核上一版是不是现行那一版）升为必要。判法自证：`planted_rollback` 在覆盖写那一种上 `lost` 必须等于段数（`rerun.sh` 里自检，不过退 1）。

## 三、Q5 第二问：撤回缺口 2 只凭一趟基线够不够（主攻）

判决依据（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-main-verification.md` 第 22、23 行，原样）：
- `name=baseline_foreground_wait route=cpu sleep_ms_per_gpu_call=0 samples=56279 wait_micros_mean=14 wait_micros_p99=0 wait_micros_max=54337 samples_with_a_publish_in_between=46 max_publishes_in_between=5`
- 第 23 行同形：`route=gpu_double sleep_ms_per_gpu_call=5 samples=55156 ... samples_with_a_publish_in_between=44 max_publishes_in_between=3`。

**读数撑不撑得住「基线不支持前台抢不到」**：
1. 均值与 p99 量的是维护线程闲着的那一段。探针自己写了：`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/probes/mount_runtime_non_fsync_foreground_wait_baseline.rs:9`：「//! 局限：只有一个池、每次重挂都是新池，整理做完（池里没有可搬的段）之后后台不再发布，采样只在整理还在跑的那一段里有意义，所以重复 PROBE_REPEATS 次；」。五次重复、每次 3 秒的窗（同文件第 238 行 `for _repeat in 0..environment_number("PROBE_REPEATS", 5) {`、第 235 行 `let window = Duration::from_millis(environment_number("PROBE_WINDOW_MILLISECONDS", 3_000));`），56279 次取样里夹了发布的 46 次，占 0.08%（量的：判决里那两行相除）。`wait_micros_p99=0` 是闲着那 99.9% 的读数。夹了整理发布的取样只有 46 与 44 个（被挡的取样个数没单独打印），最长等 54337 与 77884 微秒。
2. 「拿到锁之前后台最多发 3 到 5 次」恰是缺口 2 要量的那个量：夹了发布的那些取样里，最多的一个在两次取到锁之间后台发了 3 到 5 次，即前台连着输了几次交接（读法：推的，取样之间有 200 微秒睡眠，是上界）。「一拍 64 次都抢不到没出现」引的是这个最大值，可探针没打印池里每次重复一共有多少批可发（只数了「夹了发布的取样个数」），所以 5 是「到不了 64」还是「池子只有这么几批」分不出。
3. 取样节奏本身压低了冲突：前台每次取锁之后睡 200 微秒（第 103 行 `std::thread::sleep(gap);`），不是紧循环；单线程，没有前台线程之间互抢。
4. 它量的是今天的锁节奏（整批持着会话锁），缺口 2 说的是拆开之后「放锁后前台抢不抢得到」。拆开之后每次持锁很短，前台输一次交接只多等一小段，不是一整批：缺口 2 在拆开之后多半会缩小，但那是拆开本身的效果，不是这趟基线的读数。
5. 第一轮判决既引了仿真网格、又引了基线；网格的「today」一行（`W_ms=5.0 lam_pub_per_s=1.0` 一格前台均值 3.529 毫秒，网格 `grid.txt` 第 1 行）与基线（均值 14 到 21 微秒）差两个数量级，两者不在一个负载上：网格里后台一直忙，基线里 99.9% 的取样撞上闲着。模型文件头第 13 行明说不建「放锁后原线程立刻再取」（barging）。

**还缺哪几档**（本腿的 `mount_runtime_busy_window_foreground_wait` 放开了前五样，第 6、7 样没做）：

| 档 | 为什么缺它判不了缺口 2 |
|---|---|
| 睡眠 0 与 50 毫秒（第一轮腿建议的没跑），再加几百毫秒仿挂住 | 缺口 2 的量随一批持锁时长涨；5 毫秒一档看不出趋势，50 毫秒才是真卡一批的量级（D24 一次提交约 5.3 毫秒，三段多次提交） |
| 前台 2 条以上读线程、取锁之间不睡 | 单线程时前台唯一的对手是维护线程；多线程时 `std::sync::Mutex` 的放锁抢占才会连着发生 |
| 前台是写（`publish_user_change`）与 fsync、`accept` | 写持锁更久；fsync 今天已让路，不让路的才是缺口 2 的对象 |
| 只报「被挡的取样」的分布与取到锁之前连发了几次 | 全体均值被闲着的 99.9% 稀释 |
| 打印维护线程一共发了几次 | 判 5 是「到不了 64」还是池子见底 |
| 重复趟数（同一读数再跑几趟看方差） | 46 与 44 个被挡取样的最大值只有一趟 |
| fsync、`accept` 的取样 | 本腿没摆（要主 agent 另派） |

**判定**：够不着。「基线不支持」改写成「这趟基线既不支持也不反对，量的是闲着的窗」；缺口 2 的撤回改成「挂起，等忙窗口基线」。
**推翻条件**：忙窗口基线里 4 读线程紧循环、睡眠 5 与 50 毫秒的配置上，`blocked_publishes_four_or_more` 大于 0，或 `blocked_wait_micros_p99` 接近一批整段，则缺口 2 成立（对今天的锁节奏）；被挡取样几乎全落在 `blocked_publishes_none`、`blocked_publishes_one`，则撤回站得住。读数待主 agent 跑。

## 四、Q5 第三问：活锁「要下限」引仿真网格，站不站得住（主攻）

判决原句（第 33 行）：整批作废、前台每秒 100 次发布时，W 取 5 毫秒作废 0.415、连着最多 11 批；W 取 20 毫秒作废 0.869、连着最多 44 批（仿真）。

**两层**：
- **存在性：站得住，不靠网格。** 回锁复核比 txg（第 5 条）下，锁外那一段里前台只要发布一次，这一批就作废；对任意 W > 0，前台每隔小于 W 发布一次的到达序列让每一批都作废（确定性的，不需要泊松）。没有下限就存在后台一批都发不出的历史。第一轮两条腿从结构上都得出了这一点。
- **量级：够不着。** 模型文件头 `research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-model/lock_split_cost_model.py:11`：「参数里 W 是拍的网格，不是量的数」；网格 W 取 5、20、100、1000 毫秒，到达率取 1、10、100、300 每秒，都是摆的；到达率本项目没量过（前台每秒发布数）。作废比例的闭式是 1 − e^(−λW)：λ = 10 每秒、W = 5 毫秒时 0.05，λ = 100 每秒、W = 20 毫秒时 0.87（`grid.txt` 同一族行）。所以「大半作废」只在 λW 不小于 0.7 左右成立（W = 5 毫秒要 λ ≥ 140 每秒，W = 20 毫秒要 λ ≥ 35 每秒），产品负载在哪一格没有量。

**「下限到底带回多少」我补的闭式**（推的，`awk 'BEGIN{…}'` 现算，对拍网格）：设每个锁外那一批作废的概率是 d，下限 k（连着作废 k 批之后下一批整批在锁里），则每一轮锁外至多 k 次、整批在锁里的概率 d^k，锁内批占全部批的比例 f = d^k / (E + d^k)，E = (1 − d^k)/(1 − d)。

| d（取自网格不带下限的作废比例） | k=1 锁内批占比 | k=2 | k=4 | k=8 |
|---|---|---|---|---|
| 0.415（W=5 毫秒、100 每秒） | 0.293 | 0.109 | 0.018 | 0.001 |
| 0.638（W=100 毫秒、10 每秒） | 0.389 | 0.199 | 0.067 | 0.010 |
| 0.869（W=20 毫秒、100 每秒） | 0.465 | 0.288 | 0.148 | 0.059 |
| 0.998（W=20 毫秒、300 每秒） | 0.499 | 0.333 | 0.199 | 0.110 |

对拍：k = 2 时我的作废占比 0.619（d = 0.869）、0.666（d = 0.998）对网格 `split-discard-on-any-publish-floor2` 行的 `discard_fraction=0.617`、`0.666`（W=20 毫秒的 100 与 300 每秒两格，量的）；W=5 毫秒、100 每秒一格我算 0.370、网格 0.356（相差来自网格里取段与收段的两小段锁内时长）。

**网格里下限那一档对今天的位置**（λ = 100 每秒，`grid.txt` 现数）：`today` 前台均值 5.522、16.749、77.238、754.113 毫秒（W = 5、20、100、1000）；`floor2` 前台均值 0.882、4.052、19.820、190.013 毫秒，最长等 11.046、27.927、104.599、1001.937 毫秒（今天最长 20.410、45.942、154.093、1309.627）；后台每秒发布 `today` 113.075、36.225、7.825、0.792，`floor2` 90.283、16.817、3.042、0.300。读法：前台均值只有今天的 1/6 到 1/4（0.882/5.522、4.052/16.749、19.820/77.238、190.013/754.113），最长仍是 W 的量级；下限批占锁内的比例 W = 5、20 毫秒时是 11% 到 29%，W 更大时 d 趋近 1、占比趋近 1/(k+1)；代价是后台吞吐掉到今天的约 40% 到 80%（作废的批白占 GPU）。这些全是仿真。
**同样打中别的臂**：只核 R 的臂（`split-recheck-the-region-only`，q = 0.02 摆的）在 W = 1000 毫秒、300 每秒一格也是 `discard_fraction=1.000`（119 批全作废，`grid.txt` 第 95 行），所以只核 R 的有界也依赖 q 小，不是无条件免下限。不打中今天（今天无作废）。
**判定**：「要下限」站得住（存在性）；「大半作废」「下限取多大」够不着，要等前台发布率与 W 的真量（欠账 C926 一类）。**推翻条件**：量到产品负载的前台发布率与 W 使 λW 远小于 0.1，则作废比例可忽略，下限取大值即可（存在性仍在）。

## 五、每条打中同样打中今天吗、同样打中别的形态吗（逐条）

前提：今天一批在同一次持锁里做完：`crates/singlefs-mount-runtime/src/lib.rs:613`：「let mut locked = self.lock_the_session()?;」，下一句 `crates/singlefs-mount-runtime/src/lib.rs:615`：「let outcome = session.run_compaction_in_the_background_for_at_most_one_publish(」 就在这把锁里；GPU 实现那把锁在批里一直持到发布之后：`crates/singlefs-core/src/mounted_session.rs:5160`：「let mut implementation = gpu_compute.lock();」，`crates/singlefs-core/src/mounted_session.rs:5178`：「implementation.take_the_submissions_of_the_batch();」。

| 打中 | 今天 | 第三节形态（甲乙丙读盘三候选共用三段） | 别的替代形态 | 能不能拿来判臂 |
|---|---|---|---|---|
| 旧底整理（回锁复核不比 txg） | 不中：从取版到换 current 一次持锁，没有旧底 | 三个候选一起中（共用第三段），改法甲（比 txg）对三者同样有效 | 「回锁照新一版重做」不中（旧底没了），中另一格（计划对不上新一版，见四 Q2 之外的第一轮攻方四节） | 只能判「拆」与「不拆」，**判不了**甲乙丙之间：不分辨臂 |
| 待发漏判（只比 txg） | 不中：起跑前先排空，`crates/singlefs-core/src/mounted_session.rs:7555`：「两次调用之间前台收下的待发在下一次调用先排空。」 这一行说的是待发在每一次调用起跑之前先排空；今天一批在一次持锁里做完，批里没有前台收待发的空档 | 三个候选一起中 | 「先排空再发」让每次 `accept` 都作废这一批，价格同活锁 | 不分辨臂 |
| 活锁 | 不中（没有作废） | 整批作废的形态中；只核 R 的形态在 q·λ·W 小时不中（`grid.txt` 的 `split-recheck-the-region-only` 行，q 摆的），大时照中（W=1000、300 每秒一格全作废）；候选丙只拆活意图那一支，作废数受 R 里记录数限（依赖最脆弱前提，见六） | 下限形态（连着作废 k 批后锁内算）：不中，但批回到锁里 | **能判臂**：它随复核规则与下限变，是少数分辨臂的一格 |
| 破例批必现 | 不中（破例批本来就在锁里，且今天连 fsync 都不让） | 整批作废 + 破例批中；破例批恒锁内算则不中 | 同上 | 能判「破例批走哪条」，判不了读盘候选 |
| 锁序：切换在会话锁里等满整段（1595 对 1600） | **同样中**：今天会话锁被整批持着，切换连校验都进不去，等的是同一批的剩余时长；前台读与 fsync 等的也是同一批。第一轮攻方的 `lock-order` 探针没有「今天」那一臂，所以「比今天更糟」没有读数对着 | 中：切换进得了校验、等 GPU 锁，读排在切换后面；读在切换到达之前已经不被挡（今天不论切换来不来，读都被挡整批） | 「每次 GPU 调用各借各的 GPU 锁」把切换等的缩到一次调用（健康卡约一次提交，卡挂住时一次调用仍可到约 40 秒，`.claude/kb/decisions/26-后台整理与放置回收.md:255` 写「累计到第 4 次等超过上限就报这张卡用不了」）；代价是一批之内 GPU 实现要在调用之间放，`crates/singlefs-core/src/background_compute.rs:739`：「借出实现：一批里三段都经这一次借出算。」 的不变量要重核 | 不判臂：今天与三个候选都等；**只说明拆锁没买到这一条路** |
| 锁序：持着 GPU 锁回会话锁死锁 | 不中（次序恒是会话锁在前） | **不中第三节形态**（第 4 条：「算完放掉再回会话锁」）；中的是「沿用今天守卫范围」的写法：今天守卫到 `mounted_session.rs:5178` 的 `take_the_submissions_of_the_batch` 才放，拆开时这一句要挪到锁外那一段末尾 | — | 它是实现提醒，不是形态之间的差 |
| 卸载照样等锁外那一批 | **同样中**：`crates/singlefs-mount-runtime/src/lib.rs:1110`：「fn stop_and_join_the_maintenance_thread(&mut self) {」 在今天也要等线程出来 | 同样中（线程体持 `Arc<Shared>`，`crates/singlefs-mount-runtime/src/lib.rs:1135` 的 `Arc::try_unwrap` 要线程放掉） | 「GPU 包里等的那一处查停标记」：卸载不等，卡整张泄漏 | 不分辨臂，也不属于拆锁买的 |
| 把手锁中毒 | 不中：今天算段 panic 在会话锁里，毒会话锁，之后每个入口先报坏（`lock_the_session` 在 `crates/singlefs-mount-runtime/src/lib.rs:491`，读失败标记在先） | 中，但窗只在 panic 展开到 `note_the_failure` 记坏之间；之后前台进会话锁先看坏标记，到不了 `background_compute.rs:746` 的 `.expect(`；窗里到的切换 panic 在持着会话锁的前台线程上，终态仍是「运行库报坏、要重挂」，同今天 | `lock()` 对中毒交回错误 | 不分辨臂；后果同今天 |
| Q4 严判据没打中 | 不涉及 | 三个候选共用 | — | 不分辨臂 |

**读法**：第一轮判决里「打中」的五条，除活锁与破例批外都是各拆法共用前提，只能拿来判「拆不拆」、不能拿来排候选；按 `.claude/rules/evidence-discipline.md@判据自己也会写错：打中之后先判是哪一种` 属「打中不分辨臂」。锁序那一条「比今天更糟」没有对着今天的读数，且今天等的就是整批，我判够不着。
**推翻条件**：第一轮攻方 `lock-order` 加一臂「会话锁持着等 GPU」（今天的形态），若切换到 gpu 后读被挡的时长在这一臂上**小于**拆开的 `released_before_the_session_lock` 臂（每个到达时刻都小于），则「比今天更糟」成立；读数待主 agent 跑（本腿没有为它写臂：今天形态不需要探针，是读码：等的就是整批剩余时长）。

## 六、替被判出局的一方辩护

### 六甲　今天的做法（锁内等 GPU、只认 fsync）在基线读数下还剩什么站得住

1. **零新状态的那一整族打中都不适用。** 取版、算、换 current 在一次持锁里（`crates/singlefs-mount-runtime/src/lib.rs:613` 与 615），所以没有旧底、没有「一批在途」的会话状态、没有作废账、没有两个下限互相清零的规则、没有 GPU 锁与会话锁的第二个次序、没有线程体持 `Arc<Shared>` 之外的新生命期问题。第五节表里「今天不中」的格就是它。拆开要证的东西比今天多出 5 到 6 条（五节），每一条都带零轮。
2. **基线读数下的代价**（第一轮判决第 22、23 行，量的）：整理在跑的窗里，夹了整理发布的取样 46 与 44 个，最长等 54337 与 77884 微秒、两次取到锁之间后台最多发 5 与 3 次；整理闲着时均值 14 与 21 微秒、p99 为 0。这是睡眠 5 毫秒一档、单线程前台的读数（三节说了它够不着多大范围），在这一档上今天的代价是「整理在跑的窗里被挡一到几批」，窗外为零。
3. **只认 fsync 的位置**：`.claude/kb/decisions/26-后台整理与放置回收.md:257` 写「前台有 fsync 在等时不发下一批」；让路范围改成任何前台入口是用户 2026-11-18 第二次弹窗定的（背景正文第一节），本腿不论该不该；记账的是它的价格：五处登记、新停因成员、两个下限的清零规则（见七 Q3），而基线能换到的最多是上面那 54 到 78 毫秒量级。
4. **今天输在哪**（不抹掉）：卡挂住时一批可持会话锁约 40 秒（`.claude/kb/decisions/26-后台整理与放置回收.md:255`：「累计到第 4 次等超过上限就报这张卡用不了」，上限默认 10 秒，读码）。但同一行写了 `挂着的时候 GPU 出错（含重交 3 次都超时），正在算的那一批放弃，意图也删掉`，那个后台任务随即停下：一次卡挂住事件对应一次这样的整段等待，不是每批一次（推的，读 kb 与码，没量过）。拆锁买的是这一整段；它的频率是「卡挂住事件」的频率，与前台发布率无关。这是今天的做法在缺口 3 上唯一明确输的格；其余格（旧底、活锁、锁序、中毒）今天零代价。
5. **破例批今天与拆开后相同**：`.claude/kb/decisions/26-后台整理与放置回收.md:256` 写「破例排在第 6 条的让路之前，前台有 fsync 在等也照发」；破例批在锁里算，前台等的就是今天那一批，下限形态里它恒在锁里，所以破例批这一格拆锁什么都没买。

**推翻条件**：忙窗口基线里睡眠 50 毫秒、4 读线程配置上被挡取样的 `blocked_wait_micros_p99` 与 `blocked_publishes_four_or_more` 显示前台经常被挡好几批，则今天的做法在非卡挂住的健康卡上也站不住，辩护缩回到「只对卡挂住之外的格零新状态」。

### 六乙　「有活意图那一支只核 R」：被整批作废盖过去了，最脆弱前提核出什么

正推腿 2.3 节（`research/prompts/m4-gpu-wait-outside-the-lock-design-r1-sonnet-output.md` 里「只核 R」那一行）的做法是：算段只对 R，收段在**此刻现行那一版**上核 R 的输入没变，再在此刻现行那一版上计划与发布。它与整批作废、与弱判据都不是一回事：它**不在旧底上发布**，二节那一格（旧底整理被释放核拦下或丢）不适用；判决第 52 行把它与「整批作废」并在一起当零轮下限，合并过粗。

**最脆弱前提**（原话：前台造不出指向 R 里已有记录的新映射条目）在 `crates/` 里核：

| 要核的 | 文件与行 | 核出 |
|---|---|---|
| 前台新写落在哪 | `.claude/kb/decisions/26-后台整理与放置回收.md:30`：「3. **`R` 在意图期内不可作分配目的地**（回落政策的第一条）。」；代码 `crates/singlefs-core/src/allocator.rs:849`：「let is_outside_the_excluded_region = !excluded_region」，同文件第 2521 行 `region_excluded_from_placement` 交出意图的 R | 前台写出的新单元不落进 R。R 里只会有既有记录 |
| 映射条目的 key 能不能指向既有槽 | `crates/singlefs-core/src/records.rs:926`：「/// 中央映射 key 一律 27」……key = 类标签 + 出生树 + 出生 txg + 写序 | 每次写一把新 key、指向它新分配的槽；key 里没有「指向别的单元的槽」的写法。读过 `Overwrite`、`WriteAtOffset` 一类走分配新单元；`DataChangesPublishedTogether`、`Truncate`、`Unlink` 的映射规划**没逐个读**（`crates/singlefs-core/src/mounted_session.rs:147` 的 `UserChange` 共 11 个变体） |
| 回退复活 | `crates/singlefs-core/src/mount.rs:10900`：「pub fn roll_back_by_a_forward_publish<Device: BlockDevice>(」，复活那条在 `crates/singlefs-core/src/mount.rs:10886`：「/// - 复活：R_old 引用的用户可见单元里 cur 的账记成已释放的，改回已分配，分配代写回 R_old 那一版账里的」 | 这是唯一能把已释放记录改回已分配、并让映射条目重新指着它的路径。`grep -rln 'roll_back_by_a_forward_publish' crates --include=*.rs` 去掉 `singlefs-verification` 后只剩 `mount.rs`、`transaction.rs`、`allocator.rs` 三个文件，命中除注释外只有 `mount.rs` 的定义（10900、11236）、会话内回滚里那一次调用（11245）与 `#[cfg(test)]` 的测试模块（13572 起）：**产品库里没有调用方**（量的，范围限 `crates/` 下非 `singlefs-verification` 的 `.rs`）。但它是公开函数，`crates/singlefs-mount-runtime/src/lib.rs:1075` 的 `with_the_session_and_devices` 闭包里能调，管理员回退入口以后要接到运行库的某个前台入口（`crates/singlefs-core/src/mount.rs:7` 的文档写 D23 已定项 14，推的）：前提靠的是没人调，不是类型保证 |
| 快照 | `crates/singlefs-core/src/transaction.rs:10503`：「/// 被快照引用的块这一批不改释放：之后的覆盖写照旧经映射释放被换下的角色」；留存的映射条目另放，区域扫描读的映射表是中央映射加留存条目：`crates/singlefs-core/src/mounted_session.rs:4925` | 建快照不动映射；之后覆盖写把被钉的条目从中央映射挪到留存表、记录不变（挪走，不是新造）。「谁持有」发布时按上一版重判：`crates/singlefs-core/src/transaction.rs:11395`：「None => previous_read_by_key」（取不到角色再去找留存条目）。所以复核里按 key 比位置项要比「中央映射加留存条目」那一份，不然会误判 |

**核得动的**（范围：上表读到的路径，不是「R 里记录只有这几条路会变」的全称）：前台释放 R 里旧单元（覆盖写、删除一类发布）与回退复活，会让 R 里「起点在 R 里、未释放的记录」那一份集合变；释放是前台发布，复活在产品库没有调用方；复核按**集合相等**比（含分配代）两样都看得见，复活的看得见在于：`crates/singlefs-core/src/region_scan.rs:452`：「fn records_starting_in_the_region(」 取的正是「起点在 R 里、未释放」那一份，复活会让它多出一条、分配代也不同（读码推的，没造用例）。隔离只经发布记下：`crates/singlefs-core/src/transaction.rs:15754` 是 `note_quarantined_placement` 在 `crates/` 里唯一的非定义调用点（`grep -rn` 现数，范围 `crates/singlefs-core/src`），在发布路径里，所以前台不发布就不会改隔离集。
**核不动的**（写明）：其余 `UserChange` 变体的映射规划没逐个读；`with_the_session_and_devices` 闭包能做的任意事类型上拦不住；复核集合相等要不要含分配代是实现选择，前提靠它。

## 七、第三节形态逐格复核（Q1 到 Q4，另 Q6 里与本腿读到的码相关的两句）

每格回「成立」「不成立」「推不出」，推不出写缺什么。

### Q1　下限把什么带回来了

| 问 | 回 | 依据 |
|---|---|---|
| 两个下限都让一部分批回到会话锁里等 GPU | **成立** | 按第三节第 7 条的定义：连着作废 k 批之后下一批整批回锁里算（今天的路径），破例批恒在锁里算 |
| 卡挂住时哪些历史让那一批落在锁里 | **推不出频率，形有三种**：①卡健康时前台发布密、连着作废 k 批，卡恰在这一批（锁内那一批）开始挂；②池近满、前台已写不进，每趟第一次发布是破例批（`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「破例排在第 6 条的让路之前，前台有 fsync 在等也照发」），卡此时挂；③卡已挂过一次、任务已停（`.claude/kb/decisions/26-后台整理与放置回收.md:255`：「挂着的时候 GPU 出错（含重交 3 次都超时），正在算的那一批放弃，意图也删掉」），之后的批不再走 GPU，锁里是 CPU 补跑，不是 GPU 等 | ①的概率 ≈ 下限批占比（四节表）乘卡挂住事件率；②要近满池的时间占比与每趟批数；缺：卡挂住事件率、近满池占比、每趟批数分布，三样都没量过。注意一批在卡上挂住时交回的是错误、不是结果，它自己不会被作废，也不计入 k |
| 缺口 3（卡挂住时读写被挡）在这些历史上还剩多少 | **推不出绝对量；有相对量（仿真，摆的）** | 四节表：k = 2、W ≤ 20 毫秒时下限批占 11% 到 29%；`grid.txt` 的 `floor2` 前台均值是今天的 1/6 到 1/4、最长仍是 W 量级。卡挂住时剩下的暴露 = 下限批与破例批占 GPU 等待时间的比例 × 挂住事件率；前台发布稀疏时 d 趋近 0，下限不触发，暴露趋近 0 |
| 有没有不把批带回锁里的下限 | **有候选，够不够推不出** | ①活意图那一支降级成只核 R 再在现行那一版上发（六乙；R 排他使作废数受 R 内记录数限，价格是最脆弱前提）；②签发支在锁里只补算孔洞判与产出判（CPU 内存活，不等 GPU；价格是整版映射走查留在锁里，正推腿 2.3 节第三行）；③缩小 W（批大小 `crates/singlefs-core/src/compaction_plan.rs:433` 的默认 16 是挂载参数，价格是发布更碎）。缺：①的作废上界的真量、②的锁内走查时长、③的 W 随批大小的曲线 |

### Q2　复核判据全不全、作废之后的账对不对

| 问 | 回 | 依据 |
|---|---|---|
| 第 5 条五样之外，什么会让锁外结果过期而五样都不变 | **不成立（严判据下，推的）** | 隔离集只经发布记下：`crates/singlefs-core/src/transaction.rs:15754` 的 `note_quarantined_placement` 是 `crates/singlefs-core/src/` 里唯一的非定义调用点，在 `publish_admitted` 里；被拒段表只有后台写（`note_refusal` 的调用点在 `mounted_session.rs` 5470、5544、5649 与 `compaction_plan.rs` 2220、3033，全是整理代码）；开过的聚簇段、回退、快照删除都是发布。严判据（txg 相等）下它们都已含在「txg 变」里。唯一在不发布时变的账是前台读记的「读到只坏一份」那份账（`crates/singlefs-core/src/allocator.rs:2275` 的 `OtherUnitReadsWithOneCopyFailing`），它的消费点在 `mounted_session.rs:2309` 一带用户改动发布之后另发的搬迁修好批里，不进整理的结果（范围：只在 `crates/singlefs-core/src/` 里 grep 过）。只核 R 的臂要把这些逐项列进核，见六乙 |
| D26 已定项 13 第 8 条的连续不确定计数 | **成立，是岔路不是错** | `.claude/kb/decisions/26-后台整理与放置回收.md:259` 的第 8 条：`一批放弃（GPU 出错、等超过上限）时，那一批里已经交回的段照算一轮`。作废的批按它记一轮，连着作废时同一落点在不同批里各记一轮，更快交 CPU 判；CPU 判权威，所以更保守不更危险，稀释了「隔轮」。要不要「丢掉这一批记的轮数」是设计选择，两边代价写在正推腿 2.4 节 |
| 作废不扣预算、不进备忘，连着作废一拍里会不会空转到 64 次（C1008 同形） | **成立，条件是没有下限或下限计数被清零；有下限且计数跨调用保留时不成立** | `crates/singlefs-mount-runtime/src/lib.rs:61`：「pub const COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST: u32 = 64;」，一拍循环 `crates/singlefs-mount-runtime/src/lib.rs:609`：「while pass.compaction_calls < COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST」，`keeps_going` 只对三种停因为假（`crates/singlefs-mount-runtime/src/lib.rs:667` 起的 `CompactionRunOutcome::Stopped(stop) => {` 一段里 `YieldedToAForegroundFsync`、叫停、写带宽上限）；新的作废停因若不进这一组就连叫。与 C1008（`.claude/kb/checks-owed.md:611`）同形，但这里每次连叫都白花一整段锁外 GPU 等待（推的：无下限时一拍至多 64 × W，W = 20 毫秒是 1.3 秒的白算）。下限 k 让每第 k + 1 次叫起一定发布，一拍内连叫数受它限，前提是 k 计数不被让路、被节奏截住这类停因清零 |

### Q3　让路与两个下限叠在一起

| 问 | 回 | 依据 |
|---|---|---|
| 前台与后台互相等的历史 | **不成立（推的）** | 前台只在会话锁上等，后台只在会话锁上等；锁外那一段后台不持会话锁。没找到「前台持着什么等后台、后台持着什么等前台」的环；唯一的两锁次序在 GPU 锁上，见五节 |
| 某个前台入口永远拿不到锁，或后台永远发不出 | **推不出：缺两个计数的清零规则** | 第三节第 7 条没写：连续作废数被谁清零、连续让路数被谁清零。若让路清作废数、作废清让路数，则「读线程不断（每次取输入前都有人等）+ 前台发布密」历史上：让 N 次、算一批被作废、又让 N 次、又被作废……作废数永远到不了 k，后台一批都发不出。若两个计数都只在「发成一批」时清，则每 k + 1 批至少有一批在锁里发成。要写明：作废数只在发成时清，让路数只在起了一批（发成或作废）时清 |
| `try_lock` 失败才登记时，登记与后台判之间的窗会不会让后台把同一个前台连着越过去 | **不成立（推的）** | 后台在一次调用里先判起跑、排空待发（一次发布）、回退门、才问让路（`crates/singlefs-core/src/mounted_session.rs:4732`：「match caller.answer_before_the_next_batch() {」），窗是进锁到问之间的这一小段；前台在窗里到，这一批照旧取输入，取输入结束放锁、锁外期间前台拿到锁，不是连着越过。连着越过需要后台在放锁后立刻再取锁赢过被唤醒的前台，那是 `std::sync::Mutex` 的放锁抢占（三节 2），每次只多等一小段锁内的取输入或收段 |
| 破例批不让路又在锁里算，前台此刻等多久 | **成立：等一整批，同今天** | 破例批今天就在锁里且不让 fsync（`26-后台整理与放置回收.md:256`）；拆开之后下限里它恒在锁里：前台等的是一整批（健康卡数十毫秒，卡挂住至多约 40 秒）。拆锁在这一格什么都没买 |

### Q4　读盘三候选各错在哪

| 问 | 回 | 依据 |
|---|---|---|
| 甲：每圈回锁核戳，一圈里前台发布一次就整批作废，圈数多时还发得出吗 | **成立，但不是圈数的函数** | 前台只能在锁空着时发布，作废概率取决于锁外的总时长：1 − e^(−λ·锁外总时长)；圈数多意味着有一部分读盘搬回锁里，锁外总时长反而短，作废不加重，加重的是锁内读盘时长（缺口 3 的另一半）。圈数典型一两圈是正推腿的推的（没量过）。每圈核戳还有一个好处：看到戳变就早退，省掉后面的 GPU 白算 |
| 乙：读的是旧一版指到的单元，锁外期间这些槽被释放又复用时读到什么，第 5 条的复核挡不挡得住 | **挡得住，条件是复核含分配代或严判据** | 槽要再分配，释放代要过 defer 窗口：`.claude/kb/decisions/16-发布语义.md:35`：「| 可再分配 | 已释放 ∧ 释放代 ≤ max(F_生效, 环里最旧有效根) |」，需要前台连着发数十次（根环 3S 槽）才可能；读到的是新占用者的内容，区域扫描按头里的 key 查映射（`crates/singlefs-core/src/region_scan.rs:740`：「let header_key_is_mapped = mapping_index.holds_mapping_key(&header.mapping_key);」）会判死或不一致，这份结果过期。严判据下前台一发布 txg 就变，整批作废；只核 R 时「起点在 R 里、未释放的记录」按分配代整体相等会变（`region_scan.rs:452`），也挡住 |
| 盘表分锁之后前台发布的写与锁外的读并发，块设备实现经不经得起 | **推不出：缺真盘实现是否线程安全** | `crates/singlefs-core/src/block_device.rs:297`：「fn read_at(」 是 `&self`，`crates/singlefs-core/src/block_device.rs:312`：「fn write_at(」 是 `&mut self`，trait 没有 `Send`、`Sync` 约束；分锁就得换成 `RwLock` 之类，前台发布写盘要等在读的那一次 `read_at`（一段至多约 1 MiB，正推腿 1.1 节读码算的）。内存录制盘在 `Arc<Mutex>` 里（运行库用例搭建文件头写了），经得起；`singlefs-device-library` 里真盘实现没读，缺它 |
| 丙：签发那一支留在锁里，缺口 3 剩多少 | **推不出绝对量，按批数至少 1/5** | 一条整段意图至多 `1 + ⌈64 ÷ 16⌉ = 5` 次发布（`crates/singlefs-core/src/compaction_plan.rs:1557`），其中签发那一次的丙乙甲仍在锁里等 GPU；按批数 ≥ 1/5。GPU 时间占比缺：选段循环的圈数、每圈调用数，没量 |

### Q6（只写本腿读到的两句）

- 维护线程自己的 `publish_when_due_once` 与重发冻结在锁外那一段**不跑**：它们与取输入、锁外算、回锁在同一条线程上串行（`crates/singlefs-mount-runtime/src/lib.rs:606` 与 685 在同一个循环里，`crates/singlefs-mount-runtime/src/lib.rs:609` 起的 `while`），所以不会作废自己的批；代价是锁外那一段里冻结重发与待发定时发布都推迟，今天它们在批后同样排队，没变（推的）。
- `run_one_maintenance_pass`（`crates/singlefs-mount-runtime/src/lib.rs:1087`）的同步入口若锁外那一段在调用线程上内联算，确定性不变；放到别的线程才有问题（推的，没读测试里录制盘跨线程的限制）。

## 八、岔路（候选加代价，不替用户定）

| 岔路 | 候选与代价 |
|---|---|
| 活锁下限 | ①连着作废 k 批后整批回锁里算：保活，前台每 k + 1 批遇一次今天那种等（四节表，k = 2、W = 20 毫秒时锁内批占 29%）；②活意图那一支降级成只核 R、在现行那一版上发：不回锁，代价是最脆弱前提靠没人调回退（六乙），签发支没有；③签发支锁内补算孔洞判与产出判：不等 GPU，代价是整版映射走查留在锁里；④缩小批（W 变小）：作废比例降，发布更碎。四条都未量够，缺 λ 与 W 的真量 |
| 两个计数的清零规则 | 作废数只在发成时清、让路数只在起了一批时清（保活，推的）/ 互相清零（Q3 里的饿死历史）。这是实现规格，不是岔路，但第三节没写 |
| 切换校验取 GPU 锁 | 缓存卡清单、`try_lock` 拒、校验推迟（正推腿 1.5 节三候选）；再加「每次 GPU 调用各借各的 GPU 锁」：切换等的缩到一次调用，健康卡约一次提交，卡挂住时一次调用仍可到约 40 秒，代价是 `background_compute.rs:739` 那条「一批里三段都经这一次借出」要重核 |
| 读盘三候选 | 甲：锁里读盘占的时长留在缺口 3；乙：分锁，`BlockDevice` 无 `Sync` 约束，要 `RwLock`，前台发布写盘等在读；丙：签发那一支（每意图一次）仍在锁里等 GPU |
| 卸载 | 照旧等锁外那一批（至多约 40 秒，今天同）/ 等的那一处查停标记，那张卡整张泄漏（`26-后台整理与放置回收.md:255`） |
| 缺口 2 | 挂起，等忙窗口基线；基线若不支持，「任何前台等锁都让路」换到的只剩下限批与破例批之外的那 54 到 78 毫秒量级 |

## 要主 agent 跑的命令

1. 步骤零（主 agent 直接跑，不经服务；建冻结副本，要带运行库用例的共用搭建；用完 `du -sh` 记大小再删）：
   `python3 scripts/hosts/minimal-copy.py /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet/copy --with-test-inputs`
   在仓根起；跑完看它列出的副本里有 `crates/singlefs-mount-runtime/tests/common_pool_on_memory_devices` 与 `crates/singlefs-verification`。
2. 探针整份复跑（旧底 × 前台改动种类，加忙窗口基线六种配置；包装先删运行目录）：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-model`
   在仓根起（命令里都是绝对路径）；范围声明闸要它时照 `.claude/hooks/run-scope-guard.sh` 文件头补。跑完看：`TASK_SERVER_RESULT` 末行 `exit=0`；`task-server/task log <号> --all` 里依次有 `name=patched member=gpu-wait-outside-the-lock-r2-probe`、`name=build done`、`name=exit_codes stale_base_kinds=0`、15 行 `RESULT name=stale_base_kinds kind=… arm=…`、3 行 `name=kind_check`、7 行 `name=busy_config tag=…` 各跟 1 行 `name=busy_window_foreground_wait …`、末行 `name=rerun_done`。判这一腿的话：
   - 二节：`name=kind_check` 里 `stale_weak_publish_made`、`stale_weak_lost`、`stale_weak_refused_by_the_release_check` 三个数对 `planted_lost`（判法自证，覆盖写那一种必须等于段数，不等 `rerun.sh` 退 1）；`RESULT … arm=stale_weak_restored` 看「整理被拒后把现行那一版换回去」之后 `lost` 与 `HISTORY` 行 `after_refused`；逐段 `compaction=[…]` 是没截断的结局原文（看被拒的 `unit:`）。
   - 三节：`name=busy_window_foreground_wait` 七行，读 `blocked_samples`、`blocked_wait_micros_p99`、`blocked_publishes_four_or_more`、`maintenance_publishes_total`、写线程那两个 `writer_call_micros_*`；`reproduce_first_round` 一行应与第一轮判决第 23 行同量级（对不上说明装置变了）。
   - 退 1 而日志尾巴是 `cargo build` 的报错：这一腿没编过（探针、测试文件、`MountedRuntime` 是不是 `Sync` 都没验），编不过是它的错，新派同立场的腿照 `$WORK/out/build-kinds.log`、`build-busy.log` 修；退 1 而写「判法自证没红」或「对照臂 txg_guard …读回的与 current_base 不同」：探针判不出它要判的，数作废。
   - 内存与时间：推的，见文首。
3. 模板自证（`tools/rerun.sh` 只换了 `rerun_steps`，`--selftest` 判的是模板那一半）：
   `task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-sonnet-model/tools/rerun.sh --selftest`
   在仓根起；看末行 `TASK_SERVER_RESULT` 的 `exit=0` 与日志里 `✓ 自检通过`。
4. 命名检查：本腿没改 `crates/`、`scripts/` 下的 `.rs`（探针与测试文件在 `research/prompts/…-model/` 下），不适用。

## 没做什么

- 没替主 agent 撤销或维持判决，只交复核结果；岔路写成候选加代价，没替用户定。
- 没往统一任务服务交任务，没起 cargo；两样模型没编没跑，判决要引的数（`stale-base-kinds` 的 `lost`、`stale_publish_made`，忙窗口基线的各数）一个都没复跑，上面凡依赖它们的结论都是「推的，待主 agent 跑」。`rustfmt --check` 只过了语法解析（无 `error` 行），不是编译。
- 没读 `singlefs-device-library` 的真盘实现（Q4 乙的线程安全缺它）；没逐个读 `Truncate`、`Unlink`、`DataChangesPublishedTogether` 的映射规划（六乙）；没读 `waiting_for_the_gpu.rs` 里等的循环；没碰第二轮攻方腿的报告与模型。
- 没摆：fsync 与 `accept` 的忙窗口取样；建快照、删快照的旧底历史；崩溃状态；「第一段算好计划、第三段照那份计划发」的旧底；活意图那一支的旧底。
- 偏差：`stale_base_kinds.rs` 有一次追加写进 160 多行（超过每次不超过 150 行的约束）；没有 Write 与 Edit 工具，新建文件用排他新建的重定向、已有文件用 `scripts/kb/replace-once.py` 定点改。
- 草稿目录 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-sonnet/` 里只有拼模型文件时的中间稿，交回前已删；没建仓副本、没建编译目录。
