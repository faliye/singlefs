# m4-gpu-wait-outside-the-lock-design-r2 攻方腿（Opus）报告（2026-11-18）

立场：假设正文第三节十条形态是错的；Q1、Q2、Q3 主攻，Q4、Q6 兼攻；第二轮三个角度：第一轮的修补本身、产生结论的方法、从未被看过的地方。第一轮攻过的四样（弱判据放行旧底、整批作废的活锁、持着 GPU 锁回会话锁的死锁、卸载要 join）只报修补之后新出现的。
没读禁读清单里的三处。这一腿一条项目代码都没跑：文中「量的」只有 grep 与读文件；形态模型写好了、没跑，凡写「模型该打出」的都是推的，读数等主 agent 跑「要主 agent 跑的命令」。
引代码写主工作区路径与行号：`git rev-parse HEAD` 是 4319aa2b035b2d8238d64aeb68b3ceb7674cc723，正文点名的六份文件 `git status --short` 不列（没有工作区改动），行号逐条 `grep -nF` 过。正文第三节的条目引正文文件 `research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-body.md` 自己的行号。

## 复跑命令与模型文件

模型目录 `research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-model/`。复跑（包装先删运行目录，再调同一模型目录里的 `tools/rerun.sh`）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-snapshot /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-opus/model-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-model
```

| 文件 | sha256 |
|---|---|
| `model/form_model.py` | `d80fe980c0a8dacbf208682e5ad4338325bf2fe0ae01ec65d18dae8cf1e77f25` |
| `tools/rerun.sh` | `0dc17683229543f9561857f27aed2cb9eb9a8c9be3124c2283777ace40d66979` |
| `tools/rerun-from-a-clean-work-directory.sh` | `e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b` |

`out/` 是空的。`tools/rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷来，只换 `rerun_steps`（`bash -n` 过）；包装与第一轮那一份逐字相同。`--frozen` 传第二轮开工快照目录（模型不读它，只为满足模板的参数），设计轮没有冻结副本。`form_model.py` 过 `python3 -m py_compile`，没跑过。

**模型是什么**（读它的数之前先读这一段）：照正文第三节写的离散事件模型，不读 `crates/`、不碰盘。一条维护线程照 `crates/singlefs-mount-runtime/src/lib.rs` 的线程体与一拍（每拍至多 64 次调用、调用之间放锁、trip 在途下一拍隔 1 秒、没在途隔 30 秒、定时兜底间隔 15 秒、预算 20 次），几条前台线程（发布、只收下不发、只读，到达按指数间隔），一把会话锁。两条臂：
- `today`：照今天的代码，每批整批在锁里算，只为登记了的 fsync 让路，破例批不让。这是对面那条路（锁内等）该有的样子，同一套时长下量。
- `form`：照第三节：锁里先问让路、再排空待发、取输入记戳；锁外算；回锁复核 txg 相等且待发为空，对上就发，对不上整批作废（不扣预算、trip 在途）；让路只在取输入之前问，前台 try_lock 失败才登记；连着让路 N 次之后一批不让；连着作废 k 批之后一批回锁里算；破例批恒在锁里、不让。
- 形态没写死、模型放开扫的读法：作废与出错谁先（`error_first`）、让路清不清「连着作废」的计数（`yield_resets_k`）、作废之后这一拍接着叫还是停（`continue_after_invalidation`）、锁是抢占式（`barging`，Linux 上 std `Mutex` 放锁的线程能立刻再取到，推的）还是排队交接。
- 时长全是拍的：一批 GPU 那一段 10 ms，卡挂住时一段 40 s（正文第三节第 9 条读码算的），取输入持锁 1 ms，发布持锁 2 ms，读持锁 0.2 ms。读数只说「照这套形态与这些时长会怎样」，不是产品上的数。
- 五个场景：`hang_brought_back_into_the_lock`（60 秒时卡挂住，k ∈ {1,2,4,8,不设} × 两种先后 × 两种接着叫 × fsync ∈ {1,10,100}/s × 5 个种子，另加 today）、`hang_with_accepts_only`（只收下不发、每秒 1000 次）、`exception_batch_hang`（前台写不进、读照常）、`yield_and_floors`（锁上一直有人等，k × N × 清不清 k × fsync ∈ {10,100,1000}/s）、`idle_pool_spin`（过了水位却没有可选的段）、`registration_window`（try_lock 失败到登记之间隔 0、0.5、5 ms）。共 421 段。
- 判法自证（退 1 的条件）：前台一个改动都不发时一批都不作废、后台发得出；只收下不发时 form 臂照样作废；today 臂没有可选的段时一拍至多叫 2 次；today 臂卡挂住时读等够一段的九成；form 臂不设 k、出错在先时读不等过 1 秒。
- 先跑一段 10 秒的最重配置估全量（`ESTIMATE` 行），超过 40 分钟退 3、不跑全量。推的：全量约 1700 万次前台操作，几分钟到十几分钟；内存是一个 Python 进程，几十 MB。
- 崩溃状态：没取。这一腿的历史判的是锁上等多久、后台发不发得出、作废之后的账，不靠撕裂写；不调枚举函数，这一腿与接手的腿合计 0 个状态。

## 各格判定一览

| 格 | 判定 | 打中的形（节） | 分辨候选或读法吗 | 量的还是推的 |
|---|---|---|---|---|
| Q1 出错被作废吞掉 | 打中 | 第 5 条没排出错与作废的先后；照字面作废在先，卡挂住那一批的出错不停、不报警，下一批照旧挂，连着 k 批之后那一批回锁里挂满一段，读写都等（一·1） | 分辨两种先后 | 推的，待模型 |
| Q1 下限 k 接住挂住那一批 | 打中 | 出错在先时，前台忙、作废率高的负载下每 k+1 批就有 1 批在锁里，卡在那一批上挂住就挂满一段（一·2） | 分辨 k | 推的，待模型 |
| Q1 破例批 | 打中 | 前台写不进时每一趟的头一批破例、恒在锁里；卡挂住落在它上面，读等满一段；today 臂同样（一·3） | 不分辨（拆不拆一样） | 推的，待模型 |
| Q1 只收下不发的写流 | 打中 | 「待发为空」让不 fsync 的写流同样连着作废，下限一样把批带回锁里（一·4） | — | 推的，待模型 |
| Q1 不带回锁里的下限 | 找到一个候选 | 回锁照现行那一版重排计划，不作废；有活意图那一支用不着 k，破例批也能出锁（一·5） | — | 读码推的，零轮 |
| Q2 一拍空转 64 次 | 打中 | 运行库判「这一拍接着叫」是反向匹配，作废的新停因默认接着叫；作废不扣预算、不进备忘，没有可选的段时一拍叫满 64 次、下一拍隔 1 秒（二·1） | 分辨 k（k ≥ 63 或不设才满 64）与接着叫的读法 | 反向匹配那一判读码；次数推的，待模型 |
| Q2 第 8 条计数 | 打中（弱） | 计数住在 GPU 实现里，作废不回滚；交 CPU 判过的那一批被作废，CPU 的判定随批丢、计数已清零（二·2） | — | 推的 |
| Q2 五样之外 | 没打中 | 隔离集、被拒段表、开过的聚簇段、回退、删快照、挂载续做、显式入口（二·3） | — | 读码 |
| Q3 让路清掉「连着作废」 | 打中 | 让路若算打断「连着」，锁上一直有人等时每次强发之前都有一次让路把计数清零，k 永远到不了；后台只靠锁外批碰巧不被作废（三·1） | 分辨清不清的读法；k、N 全失效 | 推的，待模型 |
| Q3 前台永远拿不到锁 | 没打中 | try_lock 失败到登记之间后台至多再取锁两次，下一次取输入就让（三·2） | — | 推的，待模型 |
| Q3 互等 | 没打中 | 修补之后两把锁在维护线程上先后持、不交叉；第二个后台调用者那一格在六·3（三·3） | — | 读码 |
| Q4 候选乙读到复用槽 | 没打中 | 旧一版里未释放记录的槽要被重写，之前必经一次发布，txg 变、复核挡住（四·1） | — | 读码 |
| Q4 候选乙与块设备 | 打中（代价） | 写要 `&mut`；扫描接口借整张盘表的切片，盘表锁罩住一次扫描调用就罩住 GPU 那一等，前台发布持会话锁等它（四·2） | 分辨盘表锁逐读取还是逐调用 | 读码 |
| Q4 候选甲、丙 | 部分 | 甲每圈回锁不让路、成败同整批；丙签发那一支 GPU 活最多却留锁里（四·3） | — | 推的 |
| Q6 维护线程自己的发布 | 打中（缺口 3 剩下的） | 锁外那一段维护线程自己在等卡，`publish_when_due` 与重发冻结都不跑；挪到别的线程就作废自己的批（六·1） | — | 读码推的 |
| Q6 同步一拍 | 打中 | 同步入口在同一线程上走三段，锁外那一段里没有别人：作废、k、N 三条分支在确定性测试与崩溃点重放里进不去（六·2） | — | 读码 |
| Q6 第二个后台调用者 | 打中（产品今天走不到） | 第 1 条的拒放在取输入那一处，判起跑那一处已经换上待生效的切换、在会话锁里取 GPU 实现那把锁（六·3） | 分辨拒放在哪 | 读码 |
| Q6 观测 | 打中 | 作废有原因、让路有停因成员，回锁里算的批（k、破例）没有（六·4） | — | 读条款 |

## 零、撤回第一轮这一腿给过、现在站不住的

1. 第一轮报告四节那一句「所以「只重算变了的那一部分」要整套重判签发前那几判，等于在锁里重扫 R——锁外省下的就只剩丙段」：**撤回**。有活意图那一支 R 是排他的（`crates/singlefs-core/src/mounted_session.rs:5597`：「R 从签发那次发布起就排他：那次发布自己的新落点也不进 R。没发成就撤掉（盘上没有这条意图）。」），前台在锁外那一段改写 R 里的单元只会把它们写到 R 外、把 R 里那条记录释放；回锁比两版里 R 的记录就知道计划里哪几个不能搬，剔掉再算账，用不着重扫、用不着 GPU。今天照抄意图那一支已经有同形的圈：被拒时点名的单元并进跳过的集合、再计划一次（`crates/singlefs-core/src/mounted_session.rs:5694`：「Err(CompactionStepRefused::PlannedUnitsRewrittenAnywayByThisPublish {」）。我第一轮那句把「剔掉」算成了「重扫」。
2. 同一节「`c0` 若改取新一版的 txg，要搬的就比判过的多（攻方 K-g 那一判照旧数算的死局）」：**收窄**。只在签发前那一判照锁外扫出的数算时成立；把新进 R、没扫过的记录全算成要搬的（保守计数）再判，就不是死局。
3. 第一轮七节表里「「三 活锁」没有一个改法修：作废重算本身就是它的来源」：**撤回**。作废是来源，下限不是唯一出路；一·5 的候选不作废（零轮）。

## 一、Q1：下限把卡挂住的那一批带回锁里（主攻）

### 一·1　出错与作废谁先，第 5 条没排

第 5 条原句（`research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-body.md:24`：「有一样不满足就整批作废：不发布、不扣预算、这一趟仍在途、不进「不发布的停」的备忘，GPU 记账照报并记作废的原因」）只说戳对不上怎么办，没说锁外那一段交回出错时先判出错还是先判戳。今天出错只有一条路：一批算着出错就停任务、删意图、报警（`mounted_session.rs` 第 5187 到 5204 行 `stop_the_task_after_a_compute_error` 那一支）。

**历史**（前缀固定、用户动作放开）：
1. 维护线程取输入、放锁，锁外那一段交卡；卡在这一刻挂住。等的上限 `crates/singlefs-core/src/background_compute.rs:905`：「pub const BACKGROUND_GPU_WAIT_LIMIT_BY_DEFAULT: Duration = Duration::from_secs(10);」，同一张卡第 4 次等超过上限才报，一段约 40 秒（正文第三节第 9 条读码算的）。
2. 这 40 秒里前台发一次（fsync、单发一次用户改动、被空间拒推一串抬 F，或只是 `accept` 收下一次——都由用户定，放开扫）。今天这一步许可：锁外那一段不持会话锁，前台入口只取会话锁（`crates/singlefs-mount-runtime/src/lib.rs:1075`：「pub fn with_the_session_and_devices<Answer>(」一类）。
3. 回锁：交回的是出错，戳也对不上。
   - **作废在先**（照第 5 条字面「有一样不满足就整批作废」）：不停任务、不报警；trip 在途，下一批照旧取输入、交同一张卡，又挂一段；连着作废计数 +1。连着 k 批之后，下一批照第 7 条（`research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-body.md:26`：「连着作废 k 批之后，下一批整批回锁里算（今天的路径）；全空段低于底额的破例批恒在锁里算。连着让路 N 次之后，下一批不让。」）回锁里算，持着会话锁挂满一段，读、`accept`、fsync 都排在它后面，约 40 秒；这一批出错才停、才报警。报警晚到约 (k+1) × 40 秒；不设 k 就永远不报，卡挂住的这段时间 D26 已定项 13 第 4 条要的「每次叫起都报警」一次都没有。
   - **出错在先**：挂住那一批在锁外出错、当场停、报警，前台不等。只有挂住时连着作废计数正好到 k（下一节）或那一批是破例批（一·3）才在锁里挂。

模型该打出（推的）：`hang_brought_back_into_the_lock` 里 `error_first=False k=K` 各行 `hung_batches_outside_the_lock_max` ≈ K、`hung_batches_in_the_lock` = 1、`errors_swallowed_by_an_invalidation` ≈ K、`alarm_after_the_hang_s` ≈ (K+1) × 40、`read_wait_max_ms` ≈ 40000；`k=None error_first=False` 各行 `runs_without_an_alarm` 等于种子数。翻面观测：`error_first=False` 各行 `alarm_after_the_hang_s_max` 不超过 41（模型里作废没吞掉出错，模型错）。

**四句**：分辨「作废在先」与「出错在先」两种读法，不分辨 k（任何 k 都在第 k+1 批回锁里挂）；系统看得到（回锁那一刻出错与戳都在手里）；满足 Q1 题面「哪些历史让那一批正好落在锁里：连着作废 k 批之后的那一批」；改法「回锁先判出错」修这一格，不修一·2、一·3。

### 一·2　出错在先，k 照样接住挂住的那一批

前台改动率 λ、锁外那一段 W 时，一批被作废的机会 1 − e^(−λW)。λW 大（前台每秒 100 次发布、W 10 ms 时约 0.63；每秒 1000 次时几乎是 1）时连着作废计数常常停在 k，下一批就在锁里；卡挂住是随机的那一刻，落在锁里那一批上的机会就是锁里批的占比，λW ≫ 1 时约 1/(k+1)。所以 k 越小越安全的直觉反过来：k = 1 时一半的批在锁里，拆锁对卡挂住只买到一半。

模型该打出（推的）：`error_first=True fsync_per_second=100.0` 那几组里 k 小的 `hung_batches_in_the_lock_max` 为 1 的种子比 k 大的多，`read_wait_max_ms_max` 跟着到 40000；`fsync_per_second=1.0` 那几组几乎都在锁外。翻面观测：`error_first=True` 各组 `hung_batches_in_the_lock_max` 全是 0。

**四句**：分辨 k；系统看得到（k 计数在会话上）；满足 Q1 题面同一句；「回锁先判出错」不修这一格，一·5 的候选修（没有 k）。

### 一·3　破例批恒在锁里算：前台写不进，读还在

破例批是「全空段低于底额、前台已写不进」时每一趟的头一次发布（`crates/singlefs-core/src/mounted_session.rs:4721`：「let is_this_publish_the_exception = budget_rule」那一判）。条款给它不让路的理由是「前台已写不进，让路换不来什么」（`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「破例排在第 6 条的让路之前，前台有 fsync 在等也照发（前台已写不进，让路换不来什么）」）。第 7 条再把它恒放在锁里：写不进的前台照样读（`with_the_session_and_devices` 里读文件），卡挂在破例批上读就等满一段。这一类历史就是池子快满、碎得要整理的那一刻；预算扣完之后每一趟只剩这一批，每一趟都在锁里。拆锁在这一类历史上与今天一样，缺口 3 一点没消。

模型该打出（推的）：`exception_batch_hang` 里 today 与 form（k=4、不设）各行 `read_wait_max_ms_max` 都约 40000、`exception_batches_in_the_lock` 大于 0。翻面观测：form 各行 `read_wait_max_ms_max` 不到 1000。

**四句**：不分辨 k（破例与 k 无关），不分辨拆不拆（today 同样）；系统看得到（破例那一判在锁里做）；满足 Q1 题面「破例批」那一句；一·5 的候选修（破例批不再怕被作废，见下）。

### 一·4　只收下、不发的写流同样把批带回锁里

第 5 条复核里有待发为空一判（来自屏障那一句，`crates/singlefs-core/src/mounted_session.rs:7528`：「整理是屏障——待发里不并进整理的发布」）。一条只 `accept`、不 fsync 的写流（日志追加、缓存写回）在锁外那一段里收下一次就让待发不空，批作废；它不动 txg，第一轮按「发布率」算的作废率没把它算进去。这样的写流每秒上千次时几乎每批作废，k 下限把每 k+1 批里的一批带回锁里，卡挂住同一节一·2 的样子。

模型该打出（推的）：`hang_with_accepts_only` form 各行 `invalidations` 大于 0（这一条也是判法自证），`k=2`、`k=8` 两行 `batches_in_the_lock` 约为 `gpu_batches` 的 1/(k+1)。

### 一·5　不把批带回锁里的下限：回锁照现行那一版重排计划，不作废（候选，读码推的，零轮）

找不把批带回锁里的下限，试过三种：
- 前台反向等锁外那一批（前台在锁外那一段里延后发布）：前台等的又是 GPU，缺口 3 回来，不要。
- 破例批与第 k+1 批改走 CPU：与 D26 已定项 13 第 1 条「开了 gpu 就照开着跑」顶，要改条款（第一轮已提过，不重报）。
- **回锁不作废，照现行那一版重排计划**：下面这一条站得住（推的）。

形：回锁之后先排空待发（屏障要的是「排空在整理发布之前」，回锁里排也满足），再拿现行那一版与戳里那一版比 R 那块盘上 R 里的记录：
1. 有活意图那一支：R 排他，前台只能让 R 里的记录变少（改写、删文件都是把记录释放）。计划里记录变了的单元剔掉，其余照锁外判过的分类与 GPU 的核（字节没变：记录没变、写时复制，那一槽没被重写过）在现行那一版上算账、发布；发布路径照旧逐条核计划与读盘核（D26 已定项 13 第 8 条，`.claude/kb/decisions/26-后台整理与放置回收.md:259`：「同一个条目连续 2 轮（N = 2）都不确定，交 CPU 那一份实现单独判一次，按 CPU 的判定处理」那一条所在的定案里「CPU 照发布路径逐条核计划与读盘核」），旧底问题没有：`previous` 就是现行那一版。剔掉之后一个不剩就不发、不算一次作废，下一批重扫。
2. 签发那一支：R 不排他。新进 R 的记录全算成要搬的（保守），在锁里重判签发前那一判（空洞够不够）与资格（R 这期间被前台开成聚簇段就没资格，换段），`c0` 取现行那一版的 txg。只有「R 没资格了」才算这一次没成，下一批选别的段——同一段不会连着没资格两次（开过的聚簇段这次挂载里不再当候选），不成活锁。
3. 停（`NoEligibleSegment` 这一类）照锁外那一版的 txg 记备忘：照现行那一版记会把旧版上判的「没得选」安到新版上。

锁里多做的只有比记录、算账与今天本来就有的发布；不读计划外的盘、不交卡。所以 k 用不着，破例批也不怕作废（前台写不进时推的抬 F 是空发布，不碰 R 的记录），第 7 条两个下限都可以拿掉，一·1 到一·4 都不再把卡挂住的那一批带回锁里。

代价与没核过的（推的）：「两版里 R 的记录逐条相同（盘、槽、跨度、世代）⇒ 单元字节与分类都没变」靠写时复制与映射只随记录变，我只读了整理这一侧，没核映射有没有不动记录就改指向的路；GPU 判过的计划在新一版上用，第 8 条「交 CPU 判」的那几条要随计划一起留；选段打分照旧版算，选得不一定最好（不是对错）。

**四句**（对作为改法的它）：它分辨第 7 条两个下限要不要；系统看得到（两版的记录都在锁里）；它答的是 Q1 题面「有没有不把批带回锁里的下限」；它修一·1 到一·4、二·1、二·2、三·1，不修六·1、六·2。

## 二、Q2：复核判据全不全，作废之后的账（主攻）

### 二·1　一拍空转到 64 次：C1008 的同形，换成作废

运行库判这一拍接着叫不接着叫是反向匹配：只有三种停因停这一拍（`crates/singlefs-mount-runtime/src/lib.rs:671`：「!matches!(」，第 673 行起列 `CompactionRunStop::YieldedToAForegroundFsync`、`YieldedToTheCallerStoppingItsThread`、`WriteBandwidthLimitReached`），别的停一律接着叫，到 `crates/singlefs-mount-runtime/src/lib.rs:61`：「pub const COMPACTION_CALLS_PER_MAINTENANCE_PASS_AT_MOST: u32 = 64;」为止。第 5 条给作废加的那个停因（「记作废的原因」）不在这三种里，照今天的代码默认接着叫。

**历史**：
1. 池子过了水位，定时兜底起跑；可是一段都选不出（候选段都钉着、隔离着或被扫描拒过——由历史定，放开）。今天这一批停在 `NoEligibleSegment`，记「不发布的停」的备忘；下一次调用不在途、定时兜底间隔没到，不起跑，这一拍停在第 2 次调用。
2. 拆开之后同一批在锁外算出「选不出」，回锁时前台在这期间发过一次（或收下过一次），整批作废：这个停也一起作废（第 5 条「不进「不发布的停」的备忘」）。
3. trip 仍在途：下一次调用走在途那一支（`crates/singlefs-core/src/mounted_session.rs:3456`：「// 放锁的后台入口上一次调用之后还没完的那一趟接着推：一趟只判一次起跑。整理停着的那一判在上面：停着时这一趟留着，解除之后接着推。」），不再判定时兜底；作废不扣预算，预算挡不住；写带宽上限只数写，作废的批没写，也挡不住。
4. 第 2、3 步重复到一拍 64 次，每次都把选段打分与扫描在卡上算一遍；这一拍结束时 trip 仍在途，下一拍只隔 `publish_poll_period`（`crates/singlefs-mount-runtime/src/lib.rs:700`：「+ if pass.is_a_trip_under_way {」那一判，默认 1 秒）。

结局：今天每 30 秒算一批；拆开、不设 k（或 k ≥ 63）时约每 1.6 秒 64 批（W 取 10 ms，推的）。每一批都白算，D26 已定项 13 第 7 条按省下的主机 CPU 时间算值不值，这些全记在赔的一边。设了 k 时每拍 k+1 批、最后一批在锁里（回到一·2）。作废之后若停这一拍（把作废的停因加进那三种），变成每秒一批，仍比今天多 30 倍。

模型该打出（推的）：`idle_pool_spin` today 三行 `max_calls_in_a_pass` 为 2（判法自证）、`gpu_batches` 个位数；form `k=None continue_after_invalidation=True` 那三行 `max_calls_in_a_pass` 为 64、`passes_reaching_the_call_limit` 几十、`gpu_batches` 几千；`continue_after_invalidation=False` 那三行 `gpu_batches` 约 120。翻面观测：form `k=None continue=True` 各行 `max_calls_in_a_pass` 远小于 64。

**四句**：分辨 k（只在 k ≥ 63 或不设时满 64）与「作废之后接着叫」的读法；系统看得到（回锁复核就在锁里）；满足 Q2 题面「连着作废时一拍里会不会空转到 64 次」；改法「作废的停因停这一拍」把 64 压到 1，不到今天的 1/30；一·5 的候选让「选不出」照旧当停记下，回到今天。

### 二·2　第 8 条的连续不确定计数：作废不回滚，交 CPU 判过的那一判随批丢

计数住在 GPU 实现里，不在会话上（`crates/singlefs-background-gpu/src/two_kernel_reconciliation.rs:7`：「//! 一轮里确认或交 CPU 判过的条目，计数清零：「连续」从下一次不确定重新数起。没出现在这一批里的条目，计数原样留着。」）。锁外那一段里两份核比对、该交 CPU 的交 CPU 判、清零，都在回锁之前做完；回锁作废时这些改动收不回来。

**历史**：一个条目（落点）两份核一直不一致（一份核的错、或那片显存位翻转，第 8 条要防的就是这种）。轮 a：不确定，计数 1，这一批照第 8 条不搬它（有活意图时停在 `OnlyRecordsUncertainThisRoundLeftToMove`，意图留着）。轮 b：计数到 2，交 CPU 判出可搬，计数清零；可这一批回锁时被作废，CPU 的判定跟着丢。轮 c 又从 1 数起。前台发布的节奏若总让 b 那一类批被作废（比如每两批之间前台恰好 fsync 一次——用户定的节奏，放开扫），这条意图永远做不完、R 一直排他。k 下限要「连着」作废才起作用，a、b 交替（a 不被作废、b 被作废）时连着计数在 1 与 0 之间来回，k ≥ 2 永远到不了。

强弱：要相位对得上，随机到达下只是拖慢（推的，没进模型）。另一半「作废的批多记一轮」：换了落点的条目（旧一版里那一槽被释放、复用）计数带到新条目上，只是早一轮交 CPU，CPU 是权威，不伤对错。

**四句**：不分辨 k；系统看不到（回锁那一刻会话里没有「这一批交过 CPU」的痕迹，在实现里）——判别子观测不到，按 `.claude/rules/evidence-discipline.md`「判据自己也会写错：打中之后先判是哪一种」，要改的是形态（作废时把这一批交 CPU 判出的那几条带回来，或一·5 不作废），不是加判据；满足 Q2 题面「D26 已定项 13 第 8 条的连续不确定计数（作废的批已经记过一轮）」；改法一·5 修，别的不修。

### 二·3　五样之外：没打中的（读码）

- 隔离集：只经发布记下，发布不成立就撤（`crates/singlefs-core/src/compaction_plan.rs:881`：「/// 发布没成立时分配器换回进来时那一份，这一笔跟着没了。」），新增必伴 txg 变。
- 被拒段表：只有整理自己写，「只增不减」（`crates/singlefs-core/src/compaction_plan.rs:49`：「/// 这一次挂载里区域扫描整次拒过的聚簇段（段起点 → 拒的理由）：[`choose_region`] 不再选它们。只增不减，随挂载会话一起丢。」）。作废的那一批若照旧把拒绝记进表：跨段那一种只看那一版的记录（与今天一样会留着），两条可搬记录同 key 那一种要旧一版的映射同时指着两槽，换掉的字节造不出来。不是拆锁的新形。
- 开过的聚簇段：只随分配变，前台分配要么进待发（收下），要么随发布；两样都被第 5 条挡住。
- 读文件改的只坏一份的计数（`read_the_file` 收 `&self`、改分配器上的计数）：整理的计划不读它，另发那一批归下一次用户改动，与今天相同。
- 回退、删快照：运行库没有回退入口（前台入口就是 `accept`、`publish_pending_now`、`publish_user_change`、`switch_background_compute`、`with_the_session_and_devices` 五个，回退要交出会话、走卸载那一路）；删快照是一次用户改动的发布，txg 变。挂载续做在起线程之前跑完（`crates/singlefs-mount-runtime/src/lib.rs:746`：「/// 挂载：校验周期 → 开卡（只在选 `gpu` 时碰开卡器，读任何一块盘之前）→ 可写挂载（挂载续做在这里跑完，线程还没起，没有交错）→ 接会话 → 起维护线程。」）。显式整理入口在运行库里没有产品调用方，撞上在途批的那一格在六·3。

## 三、Q3：让路与两个下限叠在一起（主攻）

### 三·1　让路把「连着作废」清零时，k 永远到不了

第 7 条写「连着作废 k 批」，没写一次让路算不算打断「连着」。让路发生在取输入之前、一批都没开，读成「这一次调用没作废」就会清零。

**历史**：
1. 锁上一直有前台在等（几条读线程，或很多客户端）：照第 6 条（`research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-body.md:25`：「6. **让路**：任何前台入口等会话锁都让」）每次取输入之前都让；让路之后这一拍停，下一拍隔 1 秒（trip 在途）。
2. 连着让路 N 次，下一批不让：取输入、放锁、锁外算；这期间前台发一次（fsync 每秒 1000 次、W 10 ms 时几乎必然），回锁作废，连着作废计数 1。
3. 下一次调用照第 1 步让路——若让路清零，计数回到 0；第 2 步的那一批永远是「连着作废 1 批」，k ≥ 2 永远到不了；k = 1 也到不了：回锁里算的判在「不让」那一批开头，而那一批前面紧挨着的就是一次清零的让路。
4. 后台只在第 2 步那一批碰巧没被作废时发得出：机会 e^(−λW)，λ = 1000/s、W = 10 ms 时约 4.5 × 10⁻⁵，每 N+1 秒一次机会，期望约 (N+1) × 6 小时发一次。k 与 N 两个下限都不起作用。

不清零的读法下有界：每 (k+1)(N+1) 秒至少发一次，那一次在锁里（W，卡挂住时一段）。

模型该打出（推的）：`yield_and_floors` 里 `fsync_per_second=1000.0 yield_resets_k=True` 的八行 `background_publishes` 全是 0 或个位数、`batches_in_the_lock` 为 0；`yield_resets_k=False` 的同几行 `batches_in_the_lock` 约 120/((k+1)(N+1))、`background_publishes` 与它同量级。翻面观测：`yield_resets_k=True` 那几行 `batches_in_the_lock` 大于 0。

**四句**：分辨「清不清」两种读法，k、N 在清零读法下全不分辨（全失效）；系统看得到（两个计数都在会话上）；满足 Q3 题面「有没有……后台永远发不出的历史」；改法「让路不碰作废计数」修，一·5 修（没有 k）。

### 三·2　try_lock 失败才登记的窗：没打中（有界）

前台 try_lock 失败到登记之间，后台能做的：放掉回锁那一次、`publish_when_due_once` 再取一次锁（不问让路，`crates/singlefs-mount-runtime/src/lib.rs:559`：「fn publish_when_due_once(&self) -> Result<(), RuntimeFailed> {」）、下一次调用取锁取输入——这时前台多半已登记，就让；没登记就放锁去锁外算，锁空出来前台当场拿到。所以同一个前台被越过至多约两次取锁，等的是两段短持锁，不是一批。锁是抢占式时被唤醒的前台会输给刚放锁又取锁的后台，次数一样有界。

模型该打出（推的）：`registration_window` form 各行 `max_overtakes` 不超过 3、`fsync_wait_max_ms`、`read_wait_max_ms` 在十几毫秒以内（含前台彼此排队）。翻面观测：`registration_delay=5.0` 的行 `max_overtakes` 随时长涨。

### 三·3　互等：没打中

修补之后（第 4 条回锁之前放 GPU 实现那把锁、第 8 条切换校验读快照），维护线程是 GPU 锁 → 放 → 会话锁，前台入口只取会话锁，不再在会话锁里取 GPU 锁，没有环。破例批在锁里算、不让，前台此刻等一批（W）或卡挂住时一段，见一·3。剩下一处会话锁里取 GPU 锁的是判起跑与开批（`crates/singlefs-core/src/background_compute.rs:1125`：「.put_the_gpus_opened_since_the_settings_were_handed_in_in_effect(),」那一处取锁），只在后台调用里，第二个后台调用者那一格在六·3。

## 四、Q4：读盘三候选

### 四·1　候选乙读到被释放又复用的槽：没打中（读码）

锁外读的是戳里那一版未释放记录的槽。那一槽要被重写，先要在某一版里被释放、过了 F 与 defer 才可再分配，释放与抬 F 都是发布；只在内存里收下的写（`accept`）落在那一版的空槽上，不碰已分配的槽；discard 只发在可再分配的整段上。所以锁外读到复用槽的那一批，回锁时 txg 一定变了，第 5 条挡住。试过的形：前台改写 R 里的单元（发布，挡住）、删文件放掉 R 里的单元再写满（发布加抬 F，挡住）、冻结的发布写了一半（冻结在戳里，挡住）、只收下不发（待发不空，挡住）。读到的是撕开的字节（读与写在同一槽上并发）同样要先有一次发布，挡住。

### 四·2　候选乙与块设备的实现：要的锁的粒度，正文没写（打中，代价）

`BlockDevice` 的读收 `&self`、写收 `&mut self`（`crates/singlefs-core/src/block_device.rs:298`：「&self,」是 `read_at` 的接收者，第 313 行「&mut self,」是 `write_at` 的）。两个产品后端的字段都是普通数据（`File` 与计数），`Sync` 由编译器给，读写锁包住盘表在类型上走得通，读与读并发、写独占。块设备本身经得起：同一槽上的并发读写要先有一次发布（四·1），挡在复核。

错在粒度：锁外读要的是整张盘表的切片，扫描接口一次调用借整张（`crates/singlefs-core/src/mounted_session.rs:5256`：「devices_to_read: &[(DeviceIdentity, Device)],」），GPU 那一段在同一次扫描调用里先读后交卡（`crates/singlefs-core/src/region_scan.rs:569`：「Some(read_unit_bytes(record.device, record.slot, unit_bytes)),」在前，第 593 行「gpu_compute.classify_region_records(」在后）。盘表锁照接口罩一次扫描调用，就罩住了交卡之后那一等；前台发布要写锁，持着会话锁在盘表锁上等它，读文件要会话锁，排在发布后面——卡挂住时缺口 3 整个回来，只差进门的是第一个写。候选乙要写明盘表锁逐次读取、不罩扫描调用（读的闭包里取放），并改扫描接口的借用；正文没写。

**四句**：分辨「逐读取」与「逐调用」两种粒度；系统看得到；满足 Q4 题面「盘表分锁之后前台发布的写与锁外的读并发，块设备的实现经不经得起」——经得起，错在锁罩多大；改法「盘表锁逐读取」修。

### 四·3　候选甲、丙

- 甲：每圈回锁读一段，回锁不问让路（第 6 条「只在取输入之前问」），每圈取一次会话锁、核一次戳。一批留得下的机会与整批锁外算相同（e^(−λ·ΣW)），只是早作废早停；圈数是选段那一圈「先扫」的次数，上界是单元区的整段数，平常一两圈（推的）。「圈数多时永远发不出」只在不设下限时成立；设了 k 就回到一·2。不新增打中。
- 丙：签发那一支（丙段对整个单元区打分、选段时逐段扫）整段留锁里，它在卡上的活最多；一条意图一次签发、之后约 R 里单元数 ÷ 批大小（≤ 4）批，签发约占整理批的五分之一，卡挂住落在锁里的机会同量级（推的）。缺口 3 剩约五分之一，加上破例批（一·3）。

## 六、Q6：从没被看过的地方

### 六·1　维护线程自己的发布在锁外那一段里不跑（缺口 3 剩下的一块）

`publish_when_due_once` 与 `resend_the_frozen_publish_if_any`（`crates/singlefs-mount-runtime/src/lib.rs:578`：「fn resend_the_frozen_publish_if_any(&self) -> Result<bool, RuntimeFailed> {」）都在维护线程上、一拍之间跑；锁外那一段也在这条线程上（正文第 4 条），线程在等卡，这两样都不跑。卡挂住一段约 40 秒：
- 只收下、没 fsync 的改动这 40 秒里不按 `T_time` 发；前台自己的 `accept` 到点会触发（转述 D16 已定项 2：触发只在收下一个完整改动之后与调用方按周期调的 `publish_when_due` 里判），所以只在前台收下之后就停下时才晚。D16 已定项 5 的丢失窗口把「排队时长」算在里面（`.claude/kb/decisions/16-发布语义.md:170`：「排队时长是调用方线程上排在这次判定前面那次调用的时长」），不违字面；可拆锁对这一项一点没买到，今天在锁里挂与拆开之后在锁外挂，这一项一样长。
- 写盘报错冻结之后，重发冻结的那一次照 D26 已定项 13 射程「冻结着的发布每一拍都重发、不退避」；锁外那 40 秒里一次都不重发，前台写一直被拒（前台 fsync 自己会重发一次，`accept` 不会）。
- 挪到另一条线程跑（定时器线程）：`publish_when_due` 是一次发布，txg 变，锁外那一批被自己作废；前台只收下不发的负载下每个 P 都作废一批，与一·4 叠在一起。

**四句**：不分辨 k；系统看得到；满足 Q6 题面「在锁外那一段期间还跑不跑、跑了算不算作废自己的批」——不跑；跑了就作废；改法：没有不碰题面的（要么认这一项拆锁不管，写进条款射程；要么锁外那一段换一条线程跑、维护线程照常一拍，作废率按一·4 算）。

### 六·2　同步一拍：三段之下照旧确定，可新分支一条都进不去

同步入口（`crates/singlefs-mount-runtime/src/lib.rs:8`：「//! 线程体之外有同步的 [`MountedRuntime::run_one_maintenance_pass`]：确定性测试不起线程，一拍一拍推，录制盘（不能跨线程）也走得通。」）在调用方线程上走三段，锁外那一段里没有别的线程：它照旧确定（录制盘不跨线程，候选乙的锁外读也在同一条线程上），可回锁时戳永远对得上——作废、连着作废 k 批回锁里算、连着让路 N 次三条分支在确定性测试与崩溃点重放里一条都进不去。后台入口今天传给一批中间的钩子是什么都不做的那一种（`crates/singlefs-core/src/mounted_session.rs:3626`：「&mut InTheMiddleOfEachBackgroundBatch::NothingHappens,」），有钩子的那一种只收后台计算状态（`crates/singlefs-core/src/mounted_session.rs:5141`：「in_the_middle_of_each_batch.in_the_middle_of_the_batch(background_compute);」），拿不到会话、做不了一次前台发布。`.claude/rules/fs-design.md:78`：「2. **每条分支必须能被测试强制进入**：留一个只供测试用的开关，无视判据直接走指定分支；进不去就等于没测。」——要一个只供测试、在锁外那一段里拿会话做一次前台动作的钩子，崩溃点重放（录制盘）里才进得去作废那一支；真线程用例进得去，可录制盘过不了线程。

**四句**：不分辨候选；系统看得到；满足 Q6 题面「`run_one_maintenance_pass` 的同步入口在三段之下怎么保持确定」——确定，但没有分支可测；改法「锁外那一段的只供测试钩子」修。

### 六·3　第 1 条的「再取输入被拒」放得太靠后：第二个后台调用者照样撞断言（修补之后新出现的）

第 1 条（`research/prompts/_m4-gpu-wait-outside-the-lock-design-r2-body.md:20`：「会话上记「有一批在锁外」与它的戳，这时再取输入被拒。」）把拒放在取输入。可后台入口在取输入之前、判起跑那一处就把待生效的切换换上（`crates/singlefs-core/src/mounted_session.rs:3419`：「.put_the_switch_requested_from_the_next_batch_in_effect();」，一趟开头 `crates/singlefs-core/src/mounted_session.rs:4516`：「background_compute.put_the_switch_requested_from_the_next_batch_in_effect();」又一次），换的时候在会话锁里取 GPU 实现那把锁（`crates/singlefs-core/src/background_compute.rs:1125` 那一处，选 `gpu` 时每次都问）。

**历史**：维护线程一批在锁外、持着 GPU 实现那把锁等卡；前台切换（只记成待生效，今天照收）；另一个后台调用者——同一个运行库上再调公开的 `crates/singlefs-mount-runtime/src/lib.rs:1087`：「pub fn run_one_maintenance_pass(&self) -> Result<MaintenancePass, RuntimeFailed> {」，或以后 D26 未定项 14 的手动整理入口（转述：手动触发先做运行库的控制接口）——取到会话锁、判起跑：
1. 在会话锁里取 GPU 实现那把锁，等锁外那一批放（W，卡挂住时一段）：所有前台排在它后面，缺口 3 回来。
2. 锁外那一批放掉 GPU 锁之后，它把待生效的那一组换上（交给实现），再走到取输入才被拒。锁外那一批回锁收尾时开批那一刻读的设置已不是生效的那一组：`crates/singlefs-core/src/background_compute.rs:2111`：「"后台计算的设置一批之内不换：挂着的切换只记成待生效，下一批开头才换上"」那一判断言失败，在会话锁里 panic，运行库从此坏。

产品今天没有第二个后台调用者（推的）；同步入口的文档写给不起线程的测试，类型不拦。这是第一轮六·2「两拍并行」那一格在修补之后的剩余：拒要挪到判起跑进门那一刻（在任何换设置、取 GPU 锁之前），或把运行库的同步入口与起了线程的运行库在类型上分开。

### 六·4　观测：回锁里算的那一批没有成员

`.claude/rules/fs-design.md:80`：「4. **分支必须可观测**：运行时看得出走了哪一条。」第 5 条给作废记原因，第 6 条给让路加停因成员；第 7 条的两条（连着作废 k 批之后回锁里算、破例批在锁里算）与锁外算的批记账上分不开：今天的记账没有「这一批持着会话锁等了 GPU」这一样，k 计数、N 计数也不在观测里。一·1「作废在先」时出错被吞，报警没有、作废原因里也看不出那一批是出错——观测要能把「出错又作废」与「只作废」分开。

## 七、改法各修哪一格（都只在我的推理与我的模型上过、被攻过零轮；量过的一格都没有）

| 改法 | 一·1 出错被吞 | 一·2 k 接住挂住 | 一·3 破例批 | 一·4 只收下 | 二·1 空转 64 | 二·2 计数 | 三·1 清零 | 四·2 盘表锁 | 六·1 自己的发布 | 六·2 分支进不去 | 六·3 第二个调用者 | 六·4 观测 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 回锁先判出错、后判戳 | 修（推的，待主 agent 跑） | 不修 | 不修 | 不修 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 作废的停因停这一拍 | 不碰 | 不碰 | 不碰 | 不碰 | 压到每秒一批（推的，待主 agent 跑） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 让路不碰作废计数 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的，待主 agent 跑） | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 |
| 回锁照现行那一版重排计划、不作废（一·5） | 修（没有作废可吞） | 修（没有 k） | 修（破例批出锁） | 修 | 修（停照记） | 修 | 修（没有 k） | 不碰 | 不修 | 不修 | 不碰 | 不碰 |
| 盘表锁逐读取、改扫描接口借用 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 | 不碰 | 不碰 | 不碰 |
| 锁外那一段换一条线程、维护线程照常一拍 | 不碰 | 不碰 | 不碰 | 加重（自己的发布也作废） | 不碰 | 不碰 | 不碰 | 不碰 | 修一半（推的） | 不碰 | 不碰 | 不碰 |
| 锁外那一段的只供测试钩子 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 | 不碰 |
| 在途批的拒挪到判起跑进门 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） | 不碰 |
| 记账加「持着会话锁等了 GPU」与原因、k 与 N 计数进观测 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 不碰 | 修（推的） |

模型里有读法开关的几格（一·1 两种先后、二·1 接着叫与否、三·1 清不清）是同一个形态的两种读法，不是改法的 A/B；改法的 A/B 要在探针里做（`.claude/rules/three-way-inference.md`「判决由主 agent 做，不由投票做」）。一·5 的候选连模型都没进（模型只有作废那一条路），是读码推的。

## 没打中的形状

- Q2 五样之外：见二·3（隔离集、被拒段表、开过的聚簇段、读路径计数、回退、删快照、挂载续做、显式入口），读码，取样范围是 `advance_compaction_by_one_publish_computing_on` 读的每一样会话状态（`mounted_session.rs` 第 5355 到 5732 行）与运行库五个前台入口。
- Q2 txg 与 `Arc` 指针 ABA：锁外那一段攥着旧一版的 `Arc`，那块地址不会被新一版复用；把现行那一版换回同一个 `Arc` 的路只有删除续做一步没走成（`*current` 回到同一份），那时会话状态也没变。没打中。
- Q3 前台永远拿不到锁、互等：三·2、三·3。
- Q4 候选乙读到复用槽：四·1 列的五种形。
- 切换在锁外那一段里到：第 8 条之后切换不取 GPU 锁，只记成待生效、清掉停下表（`crates/singlefs-core/src/background_compute.rs:1962`：「self.replace_the_tasks_stopped(Vec::new());」）；这一批开批时读的设置不变，收尾的断言不撞（只有六·3 那种第二个后台调用者才撞）。
- 第一轮四样的修补后新形：弱判据——严判据（txg、`Arc`、待发空）下二·3 没造出过期；活锁——新形是二·1（空转）与三·1（清零），已报；持 GPU 锁回会话锁——修补后没有环（三·3），剩六·3；卸载 join——第 9 条两候选这一轮没攻（卡挂住时卸载等多久与第一轮同一结论，不重报）。

## 这条腿自己的限度

- 模型一次没跑：Python 照形态手写，跑不通是这一腿的错，承接的腿照 `out/form-model.log` 修。读数方向是推的，翻面条件逐节写了。
- 模型是形态的模型，不是 `crates/` 的：时长、持锁时长全是拍的；锁是抢占式还是排队只扫了两档；前台到达是泊松、单线程一条一种；「锁上一直有人等」用一个开关代替一群读线程；卡挂住只摆了「之后每一批都挂满一段」一种（重交、片、多卡都没摆）；预算与 trip 只摆了定时兜底一种触发；破例批之后的预算照「写不进时不重给」摆。
- 一·5 的候选只读了整理这一侧；映射会不会不动记录就改指向，没查全仓。
- 二·2 的相位锁定、六·1 的丢失窗口、六·3 的第二个调用者都没进模型，是读码推的。
- 卡挂住真实时长（约 40 秒）用的是正文读码算的数，没在真卡上量（欠账 C926 那一类）。

## 要主 agent 跑的命令

1. 形态模型整份复跑（包装先删运行目录 `model-run`，再调同一模型目录里的 `tools/rerun.sh`）：

```text
task-server/task submit -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-snapshot /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-opus/model-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-model
```

   - 在哪起：仓根（命令里的路径都是绝对路径），新 shell。运行目录在这一腿认领的草稿目录下、名字以 `-run` 收尾，包装只删它。不编译、不调枚举函数，0 个崩溃状态。范围声明闸要它时照 `.claude/hooks/run-scope-guard.sh` 文件头补。
   - 跑完看：`TASK_SERVER_RESULT` 那一行 `exit=0` 算跑完（退非 0 的件数为 0）；`task-server/task log <号> --all` 里依次有 `name=rerun_roots`、`form_model.py` 的 sha256 那一行（该是 `d80fe980c0a8dacbf208682e5ad4338325bf2fe0ae01ec65d18dae8cf1e77f25`）、`ESTIMATE name=form_model runs=421 …`、每组一行 `SUMMARY name=…`、`WALL_SECONDS`、`name=self_checks checked=… failed=0`、`name=exit_codes form_model=0`、`name=rerun_done …`。逐段的 `RESULT` 行在运行目录 `out/<场景>.out`。
   - 判这一腿的话：一·1 看 `hang_brought_back_into_the_lock` 的 `error_first=False` 各组 `alarm_after_the_hang_s_max` 与 `hung_batches_in_the_lock_max`、`runs_without_an_alarm`；一·2 看 `error_first=True` 各组 `hung_batches_in_the_lock_max` 随 k 怎么变；一·3 看 `exception_batch_hang` 的 `read_wait_max_ms_max`；二·1 看 `idle_pool_spin` 的 `max_calls_in_a_pass_max` 与 `gpu_batches_max`；三·1 看 `yield_and_floors` 的 `background_publishes_max` 按 `yield_resets_k` 分开；三·2 看 `registration_window` 的 `max_overtakes_max`。各节翻面观测照节里写的判。
   - 退 3：估时超过 40 分钟，交承接的腿缩种子与时长。退 1 而有 `✗ 自检：` 行：模型判不出它要判的，数作废，交承接的腿修。
   - 内存与时间：推的，一个 Python 进程几十 MB，几分钟到十几分钟。
   - 承接：新派同立场的腿，派发提示写「承接：research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-output.md」与任务号，它读 `task-server/task log <号> --all` 与运行目录 `out/`，把各节「模型该打出」换成读数、按翻面条件改判定。

2. 引文核对（文本工具，交回前这一腿自己跑过，末行贴在文末）：`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-output.md`。

## 没做什么

- 不判辩方那几格（Q5 归辩方腿），不替主 agent 采纳；七节的改法都是零轮，进产品代码之前要照 `.claude/rules/three-way-inference.md` 在探针里做 A/B。
- 没往统一任务服务交任务，模型没跑，判决要引的数都没复跑（命令在上一节）。
- 没建仓副本、没建编译目录，没在 `crates/` 上写探针：这一轮要判的是形态的读法，形态还没有代码；`crates/` 今天的行为只读码。
- 没攻第 9 条卸载两候选、第 10 条把手锁中毒（第一轮攻过卸载，这一轮分工没点名第 10 条）。
- 草稿目录 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-design-r2-opus/` 里只有 `draft/`（`rerun.sh` 与包装的草稿，几 KB），不是副本、不是编译目录。

## 引文核对末行

`python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-design-r2-opus-output.md` 退 0，末行：

```text
  ✓ 核了 36 处引文、4 处裸行号，对上 36 处，没判 4 处（其中裸行号没有引文可比 4 处；逐处列在上面）
```
