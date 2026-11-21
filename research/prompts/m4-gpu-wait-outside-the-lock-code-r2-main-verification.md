# m4-gpu-wait-outside-the-lock-code-r2：判决（2026-11-20）

正文 `research/prompts/_m4-gpu-wait-outside-the-lock-code-r2-body.md`，开工快照 `research/prompts/m4-gpu-wait-outside-the-lock-code-r2-snapshot/`。这一题代码轮第二轮，攻第一轮判决 `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md` 的七处修补与合入之后修的两行变异，并复核第一轮的判决。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 辩方（Sonnet） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r2-sonnet-output.md` | 第一轮判决 11 条里 9 条站得住；第 5 条补齐只做了一半（空洞数、要搬数没补）；第 8 条「删意图也清零」的口径只落在一处，停着时单发删意图那一支发成后不清；修补七条里 1 到 4 与 7 做的是那一条说的；九格 8 格成立，「带 --item 的命令与项」推不出（拆成目录后变异行的 `--exact` 短名） |
| 攻方（Opus） | `research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-output.md` | 修补第 1 条打中两形（甲：被门压住的删意图只在停着那一条路上补发；乙：回锁那一刻破例条件成立照样被门压住）；第 7 条打中判据字面；第一轮只抽查的三处都坐实；第 2、3、4、6 条没打中；九格成立 |

核查：
- `task-server/task submit --cwd ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-main -- python3 /home/fy5090/code/singlefs/scripts/agents/snapshot-tree.py m4-gpu-wait-outside-the-lock-code-r2 --out ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-main/m4-gpu-wait-outside-the-lock-code-r2-tree --repository /home/fy5090/code/singlefs`，`type=succeeded exit=0`：「清单点名 29 份，倒推出 29 份，sha256 对上 29 份」，29 份都取工作区此刻那一份（腿跑着的时候这 29 份没被改过）。
- `python3 scripts/agents/cite-check.py research/prompts/m4-gpu-wait-outside-the-lock-code-r2-sonnet-output.md research/prompts/m4-gpu-wait-outside-the-lock-code-r2-opus-output.md --root ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-main/m4-gpu-wait-outside-the-lock-code-r2-tree --background research/prompts/_m4-gpu-wait-outside-the-lock-code-r2-body.md`：「核了 118 处引文、48 处裸行号，对上 104 处，没判 62 处」；没判的是快照之外的文件（GPU 包别的文件、变异判法、`crates/mutations.tsv`、上一轮腿的模型）。同一条命令 `--root` 换成仓根：「核了 118 处引文、48 处裸行号，对上 118 处，没判 48 处（其中裸行号没有引文可比 48 处）」。
- 攻方腿的探针由主 agent 拷进 `~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-main/model/`（三份文件 sha256 与腿报告第一节的表逐份相同）在副本上复跑：`task-server/task submit --cwd ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-main -- bash …/model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-main-run …/model`，`type=succeeded exit=0`，三格自证 `failed=0`，汇总三行原样：
  - `RESULT name=held_back_drop_after_the_gate_opens_summary histories=8 control_held_back=4 control_dropped=4 switched_held_back=4 switched_carried_on=4 switched_dropped=0`
  - `RESULT name=held_back_drop_while_the_exception_holds_at_the_relock_summary histories=2 same_state_at_the_relock=2 control_published=1 window_published=0`
  - `RESULT name=stopped_drop_at_the_trip_start_and_the_yields_in_a_row_summary histories=4 premises_held=4 control_not_cleared=0 drop_not_cleared=2`
  三行都与腿推的读数逐字相同，腿写的三条推翻条件（`switched_carried_on=0`、`window_published=1`、`drop_not_cleared=0`）一条都没出现。
- 攻方腿的四条候选变异：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy singlefs-verification '攻方候选：回锁签发前判的空洞数取票里那一版的记录' '攻方候选：回锁签发前判的要搬数记录变了也取锁外扫出的能搬数' '攻方候选：回锁签发重判资格不看这次挂载开过的段' '攻方候选：回锁签发重判资格不看钉住与隔离的槽'`，`type=command-failed exit=2`：「✗ 基线没有证实 攻方候选：回锁签发重判资格不看钉住与隔离的槽 的点名测试通过：一行都没出现（那一批退出码 0；参数筛不到它，或名字认不出）」。候选行点名的测试用的也是不带模块前缀的裸名，四条都没判成，不算证红；缺口改由改法第 5 条的新用例证红。
- 合入之后修的两行变异：`task-server/task submit --cwd /home/fy5090/code/singlefs -- bash scripts/mutation/prove-red.sh --copy ~/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r2-opus/copy singlefs-background-gpu '七·3 设置交给实现：切换生效的那一批开头交给实现的是换上之前的那一组' '七·3 GPU 代码轮一：一批三段怎么交的不记进那一批的记账'`，`type=succeeded exit=0`：「✓ 点名 2 条：现判 2 条、复用 0 条、拒算 0 条；跑了 2 条，抓到 2 条（其中复用 0 条），本次未跑 0 条，跑了的都抓到了」。两行点名的测试用 CPU 实现的替身，不上真卡。
- 主 agent 另外量到的（拆后快照上的门禁 crates-mutation-replay 点这一批 19 行，`type=command-failed exit=1`）：运行库 3 行、background-gpu 库 2 行抓到；验证包 14 行「基线不绿——点名的测试 … 一行都没出现」，原因是变异行的 `--exact` 还是拆分之前的裸名。这一条坐实辩方腿第五节推的那一句。

## 二、判决

条款原文在 `.claude/kb/decisions/26-后台整理与放置回收.md` 已定项 13 第 4、5、6 条（第 255–257 行）。

| # | 腿报的 | 判 | 依据 |
|---|---|---|---|
| 1 | 修补第 1 条甲：回锁出错那一支被回退门压住的删意图，只在停着时下一趟开头补发；出错之前已请求切换（停下当场解除）、只放弃这一批的出错（重复键）、停着而在 CPU 上补跑、新卡开卡期间出错停下而门开之前换上了新卡这几条路上，门开之后不补发，意图被接着搬 | **采，量过** | 第 255 行「正在算的那一批放弃，意图也删掉」、第 256 行「门关就不发，意图与 `R` 排他留到门开」、第 259 行重复键「那一批的意图照第 4 条一批出错放弃的规矩删」。探针格一：出错之前已请求切换那一臂 `switched_carried_on=4 switched_dropped=0`，对照（照旧停着）那一臂 `control_dropped=4`。量的只有切换那一形；重复键、补跑、新卡三形读码推，同一个病根：压住之后只有 `crates/singlefs-core/src/mounted_session/compaction_run.rs:345` 那一判（`compaction_stays_stopped`）会再发 |
| 2 | 修补第 1 条乙：取输入时破例不成立、窗口里前台把空间写满、回锁那一刻破例成立而门关，删意图照样被门压住 | **采，量过** | 第 256 行「破例的条件（全空聚簇段数低于底额、前台已写不进）成立时，这一次也照发」。`crates/singlefs-core/src/mounted_session/compaction_batch.rs:755` `is_the_drop_after_a_compute_error_held_back_by_the_rollback_gate` 只看取输入时定的 `RollbackGateAtTheRelock` 与版本动没动，回锁那一刻不重看破例。探针格二 `window_published=0`、对照 `control_published=1`；「写满」用的是只供测试的底额开关，判的谓词与产品同一个 |
| 3 | 停着时一趟开头单发的删意图发成之后不清让路的两个计数（两条腿都报） | **采，量过** | 第 257 行「一批发成两个都清零」，第一轮判决第 8 条定「一批 = 一次整理发布」。`crates/singlefs-core/src/mounted_session/compaction_run.rs:368-375` `Published` 那一支只记预算、根槽、写带宽；全仓清零只在同一份文件第 550、604 行。探针格三 `drop_not_cleared=2`、对照 `control_not_cleared=0` |
| 4 | 修补第 7 条：清零落在写带宽与回退门两判之前，没取到下限也清；kb 写的是「下限到了、取了输入而没让路的那一批也把两个计数清零」 | **采，病根在 kb 写回的措辞，改 kb 不改代码** | 用户 2026-11-19 弹窗选的是第一轮判决「交用户的」表的候选乙，原文「改条款，取了输入而没让路的那一批也清零（「连着」被它打断）」，选项原话「取了输入没让也清零 (Recommended)」，都没有「下限到了」这一限定；代码照这一句做，修补新写的用例断言的也是这一形。写回第 257 行时多出「下限到了、」，与用户定的不符 |
| 5 | 第一轮只抽查的签发那一支：空洞数、要搬数取现行那一版没有用例罩到；重判资格的「开过的段」「钉住与隔离」两个合取项罩不到 | **采** | 第 256 行「签发时对选中的段在现行那一版上重判资格与签发前那一判，各个量取现行那一版的」。`crates/singlefs-core/src/mounted_session/compaction_batch.rs:946-950` 要搬数那一处在变异表里没有行；攻方腿的四条候选证红没判成（「一、腿与核查」），由新用例证红 |
| 6 | 辩方：第一轮判决第 5 条要的四条候选只补了两条 | **采，并进第 5 条** | 同第 5 条 |
| 7 | 辩方：拆成目录之后变异行 `--exact` 短名筛不到 | **采，量过，不归这一题改** | 主 agent 在快照上量到 14 行基线不绿（「一、腿与核查」最后一条）。这一类归管拆分变异筛选词的会话那一件（`records/2026-11-20-拆分带出的门禁与变异红.md` 第 125 行「666 行 `--exact <裸名>` 拆后已筛不到」），它回这 14 行都在它的改法里、与主 agent 列的对照表逐行一致；它合进主工作区之后点这 14 行重判 |
| 8 | 辩方：修补与整批合入新加的注释带「判决 m4-segment-code-r1 第三节 F2」「判决 m4-segment-code-r2 第三节 H3」这类来历 | **采** | `.claude/agent-common.md`「写」一节「代码与脚本的注释、说明只写现状，不写历史」。`crates/singlefs-core/src/mounted_session/compaction_run.rs:606`、`crates/singlefs-core/src/mounted_session/compaction_batch.rs:891`；拆分之前就有的同类注释不在这一题改 |
| 9 | 辩方：第一轮判决第 11 条只答了第 6 条，没答第 5 条括号里「问让路、排空待发」的次序 | **不采** | 第 256 行括号列的是会话锁里取输入那一段做哪几样，没写次序；排空发的是用户改动，第 6 条的让路只约束整理批（第一轮判决第 11 条）。先排空后问让路，前台在等时排空那一次本来也是它要的发布 |
| 10 | 辩方：第一轮判决第 9 条引的「第三轮改法第 11 条候选乙」号对的是设计材料，判决表里是第 13 条 | **不采成缺陷** | 只是引用的编号口径；第 9 条的结论辩方也判站得住 |
| 11 | 攻方：修补第 4 条 fsync 试锁失败到登记之间、放锁到减计数之间两个窗口 | **不采** | 两个窗口别的前台入口同一写法，不是这一次修补改出的；条款第 6 条「前台有任何入口在等会话锁」的判别子在试锁失败那一刻之后才看得到 |
| 12 | 修补第 2、3、6 条 | **没打中**（两条腿都判） | 腿报告各自第三节 |

九格检查单：两条腿在「走不到的理由二次查实」「类型与名字」「不兼容旧代码与旧格式」「不新建判定库」「所有任务多机可用」「测试优先多线程」「新崩溃流多罩了什么」「放的位置」八格都答成立。「带 --item 的命令与项」攻方答成立（只核了点名的测试在拆后的树里各找得到），辩方答推不出：按第 7 条判，变异行点名的测试拆后筛不到，归管拆分变异筛选词的会话那一件。

## 三、改法

都在 `crates/` 改，合成一批交实现员，以主工作区此刻为基线；第 4 条改 kb，主 agent 自己写。七条都量过病根，改法本身在第三轮攻。

1. 回锁出错那一支被门压住的删意图（判决第 1 条）：会话上记一个「出错放弃那一批的意图待删」的状态，压住时置上；之后任一入口、任一条路（照旧停着、停下已解除、重复键只放弃这一批、CPU 补跑、新卡换上之后）在取下一批的输入之前先判它：置着就照停着时那一次只删意图的发布判回退门与破例，门开或破例成立就只发这一次删意图、清掉状态，门关就这一趟不发、不在这条意图下接着搬。补用例：出错之前已请求切换、重复键各一形（门关压住、门开之后下一次调用把意图删掉、之后不再搬它），与变异行。
2. 回锁出错那一支判门之前先在现行那一版上判破例（判决第 2 条）：全空聚簇段数低于底额、前台已写不进时照发，取输入时定的「不判回退门」与否不改变这一判。补用例（取输入时破例不成立、窗口里把全空段降到底额以下、回锁照发）与变异行。
3. 停着时一趟开头单发的删意图发成之后清让路的两个计数（判决第 3 条），`crates/singlefs-core/src/mounted_session/compaction_run.rs:368` 那一支调 `clear_the_yields_in_a_row`；补用例与变异行。
4. `.claude/kb/decisions/26-后台整理与放置回收.md` 第 257 行「下限到了、取了输入而没让路的那一批也把两个计数清零」改成照用户定的那一句：「取了输入而没让路的那一批也把两个计数清零」；变更史 `.claude/kb/decisions-history.md` D26（后台整理与放置回收） 节记一条，改前写 git 引用。
5. 签发那一支补测试（判决第 5 条）：空洞数取现行那一版、要搬数在 R 的记录变了时取现行那一版未释放的记录数、重判资格的「这次挂载开过的段」「钉住与隔离的槽」两个合取项，各一条会让对应候选变异红的用例，候选变异收进 `crates/mutations.tsv`，点名测试写带模块前缀的全名。
6. 这一批新加的注释去掉判决号与节号（判决第 8 条），只写这一行是什么。

## 交用户的

无。第 4 条改的是写回的措辞，照用户 2026-11-19 弹窗选的那一句改回，不改定案。

## 这一轮被判过的文件

- crates/singlefs-core/src/compaction_plan.rs：判了清零函数 `break_the_yields_in_a_row_by_a_batch_not_yielding` 与它的口径（判决第 4 条，照用户定的成立）
- crates/singlefs-core/src/mounted_session/compaction_batch.rs：判了回锁出错那一支判门（判决第 1、2 条打中）、冻结丢批（没打中）、签发那一支的量（判决第 5 条缺用例）与新加注释（判决第 8 条）
- crates/singlefs-core/src/mounted_session/compaction_publish.rs：判了 `rollback_gate_at_the_relock_of` 的取法，并进判决第 1、2 条
- crates/singlefs-core/src/mounted_session/compaction_run.rs：判了停着时开头单发删意图那一支（判决第 3 条打中）、取输入没让路清零那一处（判决第 4 条）与新加注释（判决第 8 条）
- crates/singlefs-core/src/mounted_session/compaction_entry_points.rs：判了取一批输入那一处只有一个调用点，第一轮「走不到的理由」仍成立
- crates/singlefs-core/src/mounted_session/background_compaction_out_of_the_lock.rs：判了取输入与回锁发布两个入口，冻结丢批的判据已不带取输入时的 txg
- crates/singlefs-core/src/mounted_session/pending_changes.rs：判了 `publish_pending_now` 照旧经运行库 fsync 那一条路走到，与修补第 4 条对得上
- crates/singlefs-mount-runtime/src/lib.rs：判了前台 fsync 先试锁再登记（修补第 4 条成立，判决第 11 条两个窗口不采）
- crates/singlefs-background-gpu/src/background_gpu.rs：判了建缓冲那一等改成交回 future，修补第 3 条成立
- crates/singlefs-background-gpu/src/background_gpu/buffers_of_a_chunk_in_one_out_of_memory_scope.rs：判了建缓冲那一等照这张卡的账取期限、读停标记，修补第 3 条成立
- crates/singlefs-background-gpu/src/background_gpu/waiting_for_wgpu_futures_and_thread_clocks.rs：判了拆出的收期限那一半与原来那一等读同一个停标记
- crates/singlefs-background-gpu/src/background_gpu/tests/buffers_of_a_chunk_in_one_out_of_memory_scope.rs：判了新用例断言的是停时放弃那一等、不判卡坏
- crates/singlefs-background-gpu/src/waiting_for_the_gpu.rs：判了等提交结果那一处读停标记的写法，修补第 3 条照它写
- crates/singlefs-background-gpu/src/memory_limit_and_chunks.rs：判了切小重切时停时那一等的返回照原样交回
- crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs：判了 12 条用例都带不切片的理由（修补第 6 条成立）
- crates/singlefs-mount-runtime/tests/mount_runtime_yields_to_foreground_entries_waiting_for_the_session_lock.rs：判了 fsync 登记三条用例与让路下限用例，测的是用户定的清零口径
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/compute_errors_and_the_rollback_gate_closed_in_the_window.rs：判了出错压住那条用例只钉「门关不发」、不钉「门开补发」（判决第 1 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/entries_while_a_batch_is_out_of_the_lock.rs：判了一批在锁外时再取输入被拒的用例与条款第 5 条对得上
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/issues_at_the_relock.rs：判了签发那一支用例只断言 `c0` 与 `segs0`（判决第 5 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/main.rs：判了六个子模块的声明（第 85–90 行），变异行点名要带模块前缀（判决第 7 条）
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/publishes_frozen_and_drains_refused_at_the_relock.rs：判了取输入前就冻着那一形的用例接住了修补第 2 条
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/relock_after_a_foreground_change_in_the_window.rs：判了窗口里只改隔离记号那一形回锁剔掉的用例
- crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock/yields_to_a_foreground_entry_waiting_for_the_session_lock.rs：判了取输入没让路清零的用例测的是用户定的口径（判决第 4 条）
- research/prompts/_m4-gpu-wait-outside-the-lock-code-r2-body.md：判了题面列的修补落点与九格答案，两条腿逐格核过
- research/prompts/m4-gpu-wait-outside-the-lock-code-r1-main-verification.md：辩方复核，第 1–4、6、7、9–11 条站得住，第 5、8 条只做了一半（判决第 3、6 条）
- research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md：判了改法第 9、11、13 条与修补第 4、3 条对得上
- research/prompts/m4-gpu-wait-outside-the-lock-code-r2-snapshot/mutation-rows-of-the-batch.tsv：判了 23 行锚点各命中 1 次，验证包 14 行点名要带模块前缀（判决第 7 条）

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D26（后台整理与放置回收） 已定项 13 | 支撑 | 2026-11-20 改了：第 6 条清零那一句照用户定的原话改回（改法第 4 条）；第 5 条出错删意图被门压住、回锁破例两处是实现没照条款做，条款不改 |
| D24（后台重活能不能卸给GPU） 已定项 2 | 不影响 | 2026-11-20 不受影响：修补第 3 条只改等的那一处读停标记，没碰 GPU 管哪几类计算的射程 |
