# m4-gpugc-mount-runtime-code-r2：里程碑四七节 GPU 垃圾回收，运行库第一轮修补，代码轮第二轮正文（2026-11-09）

<!-- doc-lint:not-numbers T1 T2 T3 -->

## 一、题面

判决 `research/prompts/m4-gpugc-mount-runtime-code-r1-main-verification.md` 第二节打中的修补（第 111 批，补丁目录 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-fixes-writer/patch/`，实现员报告同目录 `../result.md`）已合入。

这是这一题的第二轮，照 `.claude/rules/three-way-inference.md`「多轮：一次打穿不算数，三轮里多数打穿才算」第二轮分工：第一轮的修补本身、产生结论的方法、从未被看过的地方。不重复第一轮已判的格（撤回的满盘 accept、插队、反向饿线程不再攻，除非修补改变了它们）。

改动全文：`research/prompts/_m4-gpugc-mount-runtime-code-r2-diff.md`。

## 二、实现今天的样子（主 agent 的观测，2026-11-09）

- 冻结副本 `~/.local/share/singlefs/work/m4-gpugc-mount-runtime-code-r2/repo/`；被判文件 sha256 在 `research/prompts/m4-gpugc-mount-runtime-code-r2-snapshot/crates-sha256.txt`。
- 被判的产品代码：`crates/singlefs-mount-runtime/src/lib.rs`；`crates/singlefs-core/src/` 的 `compaction_plan.rs`、`mounted_session.rs`、`mount.rs`；`crates/singlefs-background-gpu/src/gpu_opener_of_this_process.rs`。
- 量到的（在实现员副本上，合入之前）：运行库与开卡器目标绿（任务 11150，24、6、10 条，对应关系是推的）；验证包 3 个目标绿（任务 11151，13、16、10 条）；真卡真线程加前台并发 1 条绿（任务 11152，第二台）。合入后验证这一轮写正文时没有结局（任务 11184），活意图期间写类改动那条流缩域那一趟也没有结局（任务 11137）。
- 已知红，不归这一批：可写挂载回退相关 15 条拒在 `NewerStateStillUnreadableAfterOneReread`，别的会话在修。
- 任务 11152 服务报 wrong-card：分到第二台卡 0，进程在卡 1、卡 2 上各占 202 MiB。这是 `background_gpu.rs` 文档写明的「没参与的卡也开（之后的切换可能点它，切换不重开设备）」，产品行为不是这一批带来的。
- 实现员留给这一轮的三处（报告原话是线索）：`publish_at_most_one_step_of_the_queue` 抄了 `crates/singlefs-core/src/transaction.rs` 选下一步的次序，两份次序不一致没有用例拦；持续写盘报错时每个 P 重发一次、没有退避；预算扣完而队列不空时新的一趟不起跑。
- 实现员补的三处判决没明说的（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-fixes-writer/result.md`）：低于底额时为 fsync 让路的那一趟不留着；已起跑的趟接着走删除队列时不受预算挡、不用掉破例；开卡器建实例途中丢过的实例不留给下一次开卡。
- 压着的条款：`.claude/kb/decisions/26-后台整理与放置回收.md` D26（后台整理与放置回收） 已定项 1、已定项 13；`.claude/kb/decisions/16-发布语义.md` D16（发布语义） 已定项 1、17；`.claude/kb/decisions/24-后台重活能不能卸给GPU.md` D24（后台重活能不能卸给 GPU） 已定项 2。
- 方案按 `crates/` 今天的实现来谈。

## 三、格

### T1　第一轮的修补本身
余量取 S 槽：S=5、6、7、8 上门与 K=16 走不走得完、连崩撑几次；破例排在让路之前会不会让破例在前台 fsync 高峰一直抢锁；每拍先重发：重发持续失败时前台与维护线程各发生什么、没有退避有没有害（盘坏时写放大、日志量）；卸载先试一次重发之后的状态；删除走空放锁之后，两次发布之间前台插进来改了队列会怎样；开卡器放闸之后的二次开卡、不持锁建实例之后的竞争（两个线程同时建实例会不会超上限）。

### T2　产生结论的方法
第一轮的撤回靠的是攻方探针：满盘那一条的盘是 320 MiB、240 段，外推到大盘站不站得住；`rollback_states`、`crash_loop` 用例造的是什么分布；S 扫描的用例是不是真的走产品开关（`FollowTheWatermarks`）；实现员把探针搬成用例时有没有放宽。

### T3　从未被看过的地方
实现员补的那三处判决没明说的；`publish_at_most_one_step_of_the_queue` 与 `transaction.rs` 两份次序；运行库的 `run_one_maintenance_pass` 之外还有没有绕过放锁的路；`observations()` 这些读数自己持不持锁；`ForegroundFsync::CallerIsStoppingItsThread` 这个新成员在全部 `match` 上的行为。

### 检查单：走不到的理由二次查实
实现员报告「走不到的理由」一节列了 9 处（`~/.local/share/singlefs/work/m4-gpugc-mount-runtime-fixes-writer/result.md`）：判决点名的运行库 2 处 `panic!`、核心层 5 处 `expect`，另加 `lib.rs` 的 `expect` 和 `compaction_plan.rs` 的 `panic!("16 不是 0")`。腿逐处追调用点判。

### 检查单：类型与名字
新名字：`ForegroundFsync::CallerIsStoppingItsThread`、`publish_at_most_one_step_of_the_queue`、水位读数的新类型（腿现查全表），余量函数的名字。`WatermarkVerdict` 的单位是不是真分开了。

### 检查单：不兼容旧代码与旧格式
本轮改动没有这一类（主 agent 读补丁文件清单）；腿现查有没有留旧常量 `ROOT_RING_SLOTS_LEFT_BY_A_COMPACTION_PUBLISH` 的兼容。

### 检查单：不新建判定库
本轮改动没有这一类：查过补丁文件清单。腿现查。

### 检查单：所有任务多机可用
新真卡用例用服务交的卡；wrong-card 那一条是产品开卡的行为（第二节）。腿判它对别的任务清场的影响要不要在这一批解决。

### 检查单：测试优先多线程
运行库四条真线程用例补了不并行的理由（实现员报告）；腿核。

### 检查单：带 --item 的命令与项
实现员报告第 5 组：`crash-point-replay.sh --item`（名字带 compaction 的项，上限 20 万态，退 79 时带 `--crash-expand-up-to`）与 `formal-verification.sh --item`（项名先 `--list-items`）。腿核项名存在（读登记，不跑）。

### 检查单：新崩溃流多罩了什么
本轮改动没有新崩溃流（实现员报告）；腿判删除走空放锁、破例前移要不要加流或小用例。

## 四、分工

| 腿 | 格 | 立场 |
|---|---|---|
| 云端辩方（Sonnet） | T1–T3 与检查单八格，T2 主攻 | 复核第一轮判决 `research/prompts/m4-gpugc-mount-runtime-code-r1-main-verification.md`：撤回与打中够不够得着；替被改掉的旧写法辩护，判修补有没有打中所有替代方案；检查单八格逐格回「成立」「不成立」或「推不出」 |
| 云端攻方（Opus） | T1–T3 与检查单八格，T1、T3 主攻 | 在冻结副本的拷贝上造反例，探针能复跑；检查单八格逐格回「成立」「不成立」或「推不出」 |

## 五、交付要求

- 引 kb 条款写 kb 文件名加那份文件自己的行号，行号去 kb 文件里现查，不从背景材料里数。
- 引代码写冻结副本里的路径与行号。
- 腿不往统一任务服务交任务，也不自己起 cargo：要量的数，把模型与命令写进自己的草稿目录（`~/.local/share/singlefs/work/<腿名>/`），命令写成 `task-server/task submit -- <命令>` 的整条（路径写成独立的词，不包 `bash -c`），写进报告「要主 agent 跑的命令」一节；量法拷冻结副本，不在冻结副本里改；复跑脚本要能安全重跑。读码推出来、没量过的写「推的，没量过」。
- 不跑任何门禁阶段。cargo 命令都点名包。
- 报告分段落盘：每一次工具调用写进文件的内容不超过 150 行，第一段排他新建，回复只写短句；早点开写。
- 回复与报告用中文。
