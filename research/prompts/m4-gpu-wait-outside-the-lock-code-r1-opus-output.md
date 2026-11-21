# m4-gpu-wait-outside-the-lock-code-r1：攻方腿（Opus）报告

立场：假设合进主工作区的这一批 `crates/` 改动是错的，去造打穿它的历史、输入与操作序列。攻击面：过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱。读码引主工作区今天的路径与行号；改动全文 `~/.local/share/singlefs/work/merge-and-verify/2026-11-19-1029961/merge-patch/crates.patch`。没有往统一任务服务交任务、没有起 cargo；凡写「推的，没量过」的都只是读码推出。

## 各格判定一览

| 格 | 判定 | 量没量 | 节 |
|---|---|---|---|
| 回锁里出错那一支单发删意图，不过回退门（门关时写出改动前写不出的一次整理发布） | 打中；D26 第 5 条两句在这一格互相顶 | 推的；探针格一待主 agent 跑 | 甲一、乙一 |
| 让路下限的「连着」「累计 T」：一次让路后接一批没发布的停，过 5 秒之后前台再等就永久不让 | 打中（字面比条款宽） | 推的；探针格二待主 agent 跑 | 甲二 |
| 回锁丢批只认锁外期间新冻的；取输入前就冻着的照发、被拒、记成整理失败 | 字面打中，后果轻（不写盘） | 推的；探针格三待主 agent 跑 | 甲三 |
| 卸载与 Drop 不等锁外那一批：CPU 路径（默认）照等整批算完 | 按第三轮第 13 条前半句打中，按 D26 第 4 条原句不算 | 推的，没量 | 甲四 |
| 「一批发成清零」的「一批」两种口径 | 内部不一致，后果推的 | 推的 | 乙二 |
| 检查单两处走不到 | 成立 | 读码 | 乙三 |
| 新行为没有测试或变异行 | 8 处（表） | grep | 第四节 |
| 检查单九格 | 成立 5、部分成立 1、推不出 2、成立带口径不一 1 | 见表 | 第五节 |
| 回锁剔不干净、隔离记号漏比、让路互等、停标记漏读（GPU 那一处）、维护线程交错写出新序列 | 没打中 | 读码 | 第七节 |

## 复跑命令与模型文件

模型只有一份探针（没编过、没跑过：这条腿不起 cargo），三格历史各两臂，打 `RESULT` 与 `SELF` 行；交主 agent 跑的整条在末尾「要主 agent 跑的命令」第 1 条。探针照现有用例 `crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs` 的搭法（`common::build_pool`、攒批会话、只供测试的开关），几个小函数照那份用例抄（`push_one_empty_publish_sequence`、`issue_an_intent_on`、写错的盘那一层），不碰崩溃枚举、不调枚举函数，全量崩溃状态数是 0。它在副本里放进验证包的 `tests/` 当一个测试目标编（只在副本里，不进仓）。

| 文件 | sha256 |
|---|---|
| `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model/src/probe_relock_gate_yield_floor_and_frozen_input.rs` | bcefda8383b7873038022bb823c16b7c23282e523011c1bf2edaaf4d98bb5bde |
| `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model/tools/rerun.sh` | 2734af179f93735ab995ebbcc233ae4f3dc0a6214889746c8ef2cd2dd8a991a4 |
| `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh` | e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b |

- 冻结副本：`/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus/copy`（`scripts/hosts/minimal-copy.py --with-test-inputs` 建的，55.6 MiB；在副本根对 `research/prompts/m4-gpu-wait-outside-the-lock-code-r1-snapshot/crates-sha256.txt` 跑 `sha256sum -c --quiet` 退 0、没有输出，23 份被判的 `crates/` 文件与开工快照逐字节相同）。在草稿目录的 `.deliverables` 里写成「交接」。
- `rerun.sh` 从 `scripts/agents/three-way-rerun-template.sh` 拷，只换了 `rerun_steps`：rsync 冻结副本进运行目录、拷探针进 `crates/singlefs-verification/tests/`、`cargo test --offline -p singlefs-verification --test probe_relock_gate_yield_floor_and_frozen_input -- relock_error_ignores_the_gate yield_floor_after_a_stop frozen_before_the_input --exact --nocapture`，`RESULT`/`SELF` 行另存 `out/result-lines.out`，末行 `name=rerun_done probe_exit=<退出码> result_lines=<行数>`。
- 外包一层 `rerun-from-a-clean-work-directory.sh <仓根> <冻结副本> <运行目录> <模型目录>`：只删工作根下名字以 `-run` 收尾的那一个运行目录，别的退 2；照范例 `research/prompts/m4-gpugc-floor-raise-precheck-code-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh` 抄。
- 规模与估时（推的，没量过）：格一 12 个 k × 2 臂 = 24 段历史，格二 5 个间隔 × 2 臂 = 10 段，格三 2 段；每段起一个池（两块 4 GiB 稀疏镜像，`common::image_path` 落在 `TMPDIR`，池丢掉就删）、30 到 60 次建 inode、至多 48 串空发布。挂钟主要在副本里头一次编验证包（估 10 到 20 分钟），跑估 5 分钟以内；内存估 2 GiB 以内。没先跑一小段，40 分钟的停机线这条腿判不了，交主 agent 看第一趟的挂钟。
- 读结局：`task wait` 末行 `exit=0`；`out/result-lines.out` 里三行 `SELF … failed=0`（自检只判对照臂分得开：对照臂门关时回锁一个不发、中间那次发成之后每个间隔都让、锁外期间冻结的回锁丢批），各格的打中与否看 `RESULT` 行，判法写在各节「推的结果」里。

## 一、过度外推与越位

### 甲一　回锁里出错那一支单发的删意图发布不过回退门（打中，推的，探针格一待主 agent 跑）

条款两句：`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「回锁里出错先于一切」；同一行：「回锁那一次发布同样过回退候选集那道门」；同一行另有：「停着时那一次只删意图的发布（第 4 条）同样过这道门：门关就不发」。第三轮改法 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:49`：「4. 回锁里出错先于一切。」与 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:52`：「7. 回锁那一次发布重判回退门。」

代码：回锁头几判里出错那一支先停任务、再单发删意图，`crates/singlefs-core/src/mounted_session.rs:6333`：「let task_after_the_error = background_compute.stop_the_task_after_a_compute_error(」，`crates/singlefs-core/src/mounted_session.rs:6337`：「let step = drop_the_live_intent_on_a_stop(」；这一支在 `judge_the_computed_compaction_batch_at_the_relock` 里就交回，到不了重判回退门的那一处 `crates/singlefs-core/src/mounted_session.rs:6459`：「RollbackGateAtTheRelock::JudgedWhenTheVersionMoved if has_the_version_moved => {」。`drop_the_live_intent_on_a_stop` 本体不判门（`crates/singlefs-core/src/mounted_session.rs:7064` 起，读过）。

历史（每一步指到许可它的那一句）：
1. 照水位判的后台入口、有活意图、根环里不同用户状态够多：取输入时门开（`crates/singlefs-core/src/mounted_session.rs:5070`：「if run.is_the_rollback_gate_judged && !is_this_publish_the_exception {」那一判放行），走 GPU，票交到锁外。
2. 锁外那一段 GPU 实现交回出错（等卡超时重交 3 次都超时、设备丢失这一类；等的上限是秒到分钟级，窗口长）。
3. 同一个窗口里前台 `accept` 被空间拒、当场推抬 F 的空发布（D16 已定项 1「准入」那一行，运行库 `accept` 文档写明一次准入至多 B − 1 次空发布）：空发布占根槽、不添新用户状态，推几串门就关（现有用例 `a_relock_after_the_rollback_gate_closed_in_the_window_publishes_nothing` 就是这样关门的）。由用户决定的是推几串（探针扫 k = 0 到 48）。
4. 回锁：出错先于一切 → 停任务 → 单发删意图发布，不判门。门关时这一次整理发布照发，根环里不同用户状态少于 4 个（不是破例批：破例只在全空段低于底额、前台写不进时，`rollback_gate_at_the_relock_of` 对破例批本来就记 `NotJudged`）。

改动之前锁里一口气推完时，出错那一次删意图紧跟在每批之前那一判后面、同一次持锁，门的判断罩得到它；三段之后这一次删意图与门的判断之间隔着整个锁外窗口。这是今天（改动前）写不出的序列：门关着的一次整理发布。

四句：
- 分不分辨臂：分辨。对照臂（不注入出错）同一段历史回锁停在门上、一个不发；只差出错与否。
- 系统当时看不看得到：看得到。回锁时 `stop_of_the_rollback_gate` 照样读得到根环，正常那一支就在这一刻判它。
- 满足的是判据字面哪一个分句：D26 第 5 条「回锁那一次发布同样过回退候选集那道门」。同一条里「回锁里出错先于一切」被实现读成「先于门」——两句在这一格上互相顶（见第二节乙一），实现选了前一句、没写选的理由。
- 跑前条款的改法在这一格上中不中：第三轮第 7 条「回锁那一次发布重判回退门」写的是重判，实现照它在正常那一支重判了，出错那一支没覆盖；第 4 条没说出错那一支要不要过门。两条改法都碰不到这一格。

推的结果（待主 agent 跑格一）：`arm=no_error` 门关的那几段 `published=0`；`arm=gpu_error_in_the_window` 门关的那几段 `published=1 published_through_a_closed_gate=1 live_intent_after=false`。推翻它的观测：错误那一臂门关时 `published=0`，或 k 扫满 48 门也不关（`gate_before_the_relock=open`，那就是这条历史在这个池上造不出）。

我提的改法（只在我的模型上量过——其实连我的模型也没跑，算推的、被攻过零轮）：出错那一支先照 `trip.rollback_gate_at_the_relock` 在现行那一版上判门，门关就只停任务、报警、不删意图（意图与 R 排他留到门开，同「停着时那一次只删意图的发布……门关就不发」）。

### 甲二　让路下限：「累计 T 毫秒」实现成「从这一串头一次让路起的挂钟」，「连着」不被没发布的停打断（打中，推的，探针格二待主 agent 跑）

条款 `.claude/kb/decisions/26-后台整理与放置回收.md:257`：「连着让路 N 次或让路累计 T 毫秒，哪一个先到下一批就不让，一批发成两个都清零」。

代码：头一次让路记下钟读数 `crates/singlefs-core/src/compaction_plan.rs:1154`：「.get_or_insert(now);」；之后每一批判 `crates/singlefs-core/src/compaction_plan.rs:1145`：「now.0.saturating_sub(yields_began_at.0) >= at_most」；只有整理发成一次才清 `crates/singlefs-core/src/compaction_plan.rs:1170`：「self.reading_when_the_yields_in_a_row_began = None;」；调用它的是 `crates/singlefs-core/src/mounted_session.rs:5108`：「compaction.clear_the_yields_in_a_row();」。计数与读数住在 `CompactionOfThisMount`，跨趟、跨用户改动留着，一次挂载里只有整理发布清它。

历史：
1. 用户改动之后那一趟，前台有入口在等，让路一次（次数 1、读数 t0）。
2. 下一次调用前台不等，这一批没发布地停：回退门关、`NoEligibleSegment`、`HolesOutsideTheRegionFewerThanItsUnitsToMove`、`PublishBudgetSpent`、回锁剔完一个不剩、选中的段没了资格、冻结丢批，任一种（探针用回退门关这一种）。计数与读数都不清。
3. 过了 T（产品默认 5 秒，`crates/singlefs-mount-runtime/src/lib.rs:86`：「Duration::from_secs(5);」）以后的任何时刻——下一次用户改动、定时兜底——前台又有入口在等：这一批判「时长到了」，不让。之后整理只要一直不发布（全空段线越线而没有合格段的池，每一趟都停在不发布的停上），这一次挂载里后台入口再也不为前台让路。次数那一样同理：零零散散让满 8 次（`crates/singlefs-mount-runtime/src/lib.rs:82`：「NonZeroU64 = NonZeroU64::MIN.saturating_add(7);」）就永久不让。

由用户决定的是第 1 步之后多久前台再等：探针扫 gap = 1000、3000、3999、4000、60000 毫秒（第 2 步在 1000 毫秒）。推的结果：中间那次发成的对照臂五个间隔都让；中间停在门上的那一臂 gap ≥ 4000（离头一次让路满 5000 毫秒）的两段不让、`yields_before_the_later_wait=1`。推翻它的观测：停在门上那一臂 gap 60000 也 `yielded_at_the_later_wait=1`。

四句：
- 分不分辨臂：分辨（两臂只差中间那次调用发没发成）。
- 系统看不看得到：看得到。它要的是「这一串让路是不是连着、累计让了多久」：连着与否它自己就知道（上一次调用停在哪），累计让路时长是每次让路之间的钟差之和，这一层读得到钟。
- 字面哪一个分句：「连着让路 N 次」的「连着」与「让路累计 T 毫秒」的「累计」。实现的读法是「自上一次整理发布以来让过的次数」「自这一串头一次让路以来的挂钟」，比字面宽：中间隔着没让路的批、隔着几十秒空闲都算进去。条款「一批发成两个都清零」只说了发成时清，没说别的时候不清；把「只在发成时清」推成「连着」「累计」的全部定义是越位。
- 改法：第三轮第 9 条写的是「连着让路 N 次后下一批不让，N 一批发成才清零」，用户后加「时间和次数都要考虑」；两条都碰不到「没发布的停算不算断开这一串」这一格。

现有用例钉的是实现的读法：`after_yielding_for_as_long_as_the_floor_allows_the_next_batch_does_not_yield` 的断言文字 `crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs:1959`：「"从头一次让路起累计 100 毫秒：这一批不让、交到锁外"」——两次让路之间钟走了 50 毫秒、前台一直在等，这一格两种读法给同一个答案，分不出（见第四节）。

我提的改法（零轮、推的）：一批取输入之前前台不在等就把两个计数清零（「连着」被打断）；时长改成累计每次让路到下一次判之间的钟差，前台不等的那几段不算。

### 甲三　回锁丢批只认「锁外期间新冻的」，取输入之前就冻着的不丢（字面打中，后果轻，推的，探针格三待主 agent 跑）

条款：`.claude/kb/decisions/26-后台整理与放置回收.md:256`：「回锁时有冻结的发布就丢掉这一批、记单独的停因」；第三轮 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:53`：「8. 回锁时有冻结的发布：这一批丢掉，单独的停因。」代码只在冻结的 txg 与取输入时不同才丢，`crates/singlefs-core/src/mounted_session.rs:6396`：「&& txg_of_the_frozen_publish_now != txg_of_the_frozen_publish_at_the_input」。

历史（不要前台并发）：维护线程一拍里先重发冻结（`crates/singlefs-mount-runtime/src/lib.rs:789`：「if self.resend_the_frozen_publish_if_any()? {」）、再 `publish_when_due` 一次（`crates/singlefs-mount-runtime/src/lib.rs:792`：「self.publish_when_due_once()?;」）——这一次写盘报错就冻结了；同一拍接着取输入（`crates/singlefs-mount-runtime/src/lib.rs:802`：「let taken = session.take_the_input_of_a_background_compaction_batch(」），起跑那一判不看冻结，票照交到锁外；锁外那一段算得比 `publish_poll_period`（默认 1 秒）快，窗口里没重发；回锁时冻着的是取输入时那同一次 → 不丢，照发 → 发布路径拒（冻结着）→ 这一趟记成 `Failed`，不是单独的停因。推的结果：格三 `arm=frozen_before_the_input` 的 `outcome` 以 `failed:` 起头、`batches_dropped_while_frozen=0`；对照臂 `stopped:BatchDroppedWhileAPublishIsFrozenAtTheRelock`。

后果（推的）：不写盘（冻结着时发布路径在任何写之前拒），整理记一次失败、这一拍不接着叫；改动前锁里推一批时同一格也是这样，所以不是新的写序列，只是「单独的停因」那句的射程被实现收窄、文档写明了（`judge_the_computed_compaction_batch_at_the_relock` 头注「锁外那一段期间冻结了一次发布的，这一批丢掉」）而条款没收窄。四句：分辨臂（两臂只差冻结在取输入前还是窗口里）；看得到（回锁时 `frozen_publish()` 就在手边）；字面「回锁时有冻结的发布」；改法第 8 条在这一格上就是条款原句，实现没照它。

### 甲四　「卸载与 Drop 不等锁外那一批」只在 GPU 等卡那一处兑现，CPU 路径（产品默认）卸载照等整批算完（字面打中，后果有界，推的，没量）

第三轮 `research/prompts/m4-gpu-wait-outside-the-lock-design-r3-main-verification.md:58`：「13. 卸载与 Drop 不等锁外那一批」（后半句写的手段只有 GPU 包里等的那一处查停标记）；D26 第 4 条原句是「卸载时不等还在跑的提交」（`.claude/kb/decisions/26-后台整理与放置回收.md:255`），说的是卡上的提交。

代码：卸载与 Drop 都先置停、再 join 维护线程（`crates/singlefs-mount-runtime/src/lib.rs:1625`：「let _panicked_is_noted_by_the_thread_itself = thread.join();」）；维护线程在 `std::thread::scope` 里等工作线程交回算完的那一批（`crates/singlefs-mount-runtime/src/lib.rs:1148`：「match computed_handed_back.recv_timeout(publish_poll_period) {」），scope 结束时要 join 工作线程。CPU 路径里建映射条目、钉住集、按身份索引、选段那一圈（每圈回会话锁读一段）都不看停标记，卸载要等它们全算完。Drop 的文档只说「锁外那一段正在等卡时不等它」（`crates/singlefs-mount-runtime/src/lib.rs:1722`），实现与这句一致，与第三轮第 13 条的前半句不一致。

后果：等的是一批 CPU 计算的挂钟（第三轮判决第二节的补量线索里替身睡 0 毫秒与 CPU 路径最坏挡前台 51 到 763 毫秒，那是锁里的数，这里没量），不会无界；GPU 路径里等卡之外的 CPU 部分同样要等完。四句：不分辨臂（这一题只有一条实现）；系统看得到停标记（`stopping` 就在 `Shared` 里），选段循环可以每圈查一次；字面是第 13 条前半句；第 13 条后半句给的改法（GPU 包里查停标记）碰不到 CPU 这一格。是否算缺陷交主 agent：D26 条款原句只管提交，按条款原句不算打中，按第三轮改法第 13 条前半句算。

## 二、内部矛盾与循环依赖

### 乙一　D26 第 5 条同一句里「回锁里出错先于一切」与「回锁那一次发布同样过回退候选集那道门」在出错且门关时互相顶

两句都在 `.claude/kb/decisions/26-后台整理与放置回收.md:256`。出错那一支的删意图是回锁那一刻的一次整理发布：前一句要它先于一切（含门），后一句要它过门。条款没说哪句优先；第 5 条后文「停着时那一次只删意图的发布……同样过这道门」又让「只删意图的发布」整体落在门的射程里。实现取了前一句（甲一）。改动前这一格不存在（锁里没有窗口），是三段拆开新冒出来的。交主 agent 定读法，定了要写回 D26 第 5 条并补一条用例（出错 + 窗口里门关）。

### 乙二　「一批发成清零」里「一批」两种口径，代码自己的注释与清零处不一致

`crates/singlefs-core/src/mounted_session.rs:5108` 在每一次整理发布（签发、丢意图、删意图、搬单元）都清零连着让路的计数；紧接着两行的注释 `crates/singlefs-core/src/mounted_session.rs:5110`：「// 一批 = 搬了单元的那一次」把「一批」定义成搬了单元的发布，只有那种才 `batches_published += 1`。条款「一批发成两个都清零」用的是哪一个口径没写；实现在同一个函数里两种都用。后果（推的）：前台一直在等时，下限放行的那一批若只是签发（不搬单元）也清零，下一串又要让满 N 次或 T 才放下一批，搬单元的节奏比按「搬了单元才清」慢一倍左右。现有用例 `a_batch_published_clears_the_yields_in_a_row_and_the_next_batch_yields_again` 用的恰是签发那一次（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs:1986`：「assert_eq!(run.versions_published.len(), 1, "回锁签发了");」），钉的是实现的口径，不是条款的。

### 乙三　检查单两处「走不到」二次查实：都成立（读码，推的）

- `crates/singlefs-core/src/compaction_plan.rs:1190` 那一断言：`hand_a_batch_out_of_the_lock` 全仓只有 `crates/singlefs-core/src/mounted_session.rs:3854` 一处调（grep `hand_a_batch_out_of_the_lock` 在 `crates/singlefs-core/src/` 命中定义与这一处），那一处只经 `take_the_input_of_a_background_compaction_batch`（`crates/singlefs-core/src/mounted_session.rs:9087` 调 `take_the_background_batch_that_starts`，全仓唯一一处）走到，入口第一判 `has_a_batch_out_of_the_lock()` 为真即拒，判过之后到交出之间持着同一个 `&mut self`、中间那几步（判起跑、排空待发、推一趟）都不交出票。成立。
- `crates/singlefs-core/src/mounted_session.rs:9311` 的 `unreachable!`：取输入时这一趟的 `publishes_of_the_run` 是 0（`take_the_next_compaction_batch_or_finish_the_run` 里锁内推完的那一种删意图发布之后照 `IntentDroppedOnAStop` 收尾，到不了交票），回锁发成一次变 1；`judge_before_the_next_compaction_batch` 先判预算、再判「用户改动之后至多一批」、再判 `span_of_the_call == AtMostOnePublish && publishes_of_the_run == 1`，`span_of_the_call` 是开这一趟时从取输入那一次的节奏抄来的 `AtMostOnePublish`；没发成的结局在 `note_the_compaction_batch_advanced` 里就交回 `Err`。成立。

## 三、哪一条最脆弱

按「改一处就翻、现有测试与变异都罩不到」排：

1. 回锁出错那一支与回退门（甲一、乙一）：唯一一处会在盘上写出改动前写不出的序列（门关着的一次整理发布）；罩它的用例 `the_compute_error_with_a_foreground_publish_in_the_window_stops_the_compaction_as_an_error_in_the_lock_does_judged_by` 用的是强制起跑（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs:975`：「CompactionRunForcing::ForcedToStartForTests,」），强制起跑那一趟取输入不判门、回锁也不判（`crates/singlefs-core/src/mounted_session.rs:5618`：「/// 显式入口、可写挂载续做、开关不照水位判、破例的那一批：取输入之前就不判，回锁也不判。」），这条用例碰不到门；没有任何变异行动这一格。
2. 让路下限的计数与读数跨趟留着（甲二）：一次挂载里只要出现「让一次、接着一批没发布」就永久改变前台等锁时的行为，而产品默认参数下没有任何用例走到「没发布的停」之后再判让路。
3. 回锁丢批的冻结判据（甲三）：后果轻，但「回锁时有冻结的发布」与实现的「锁外期间新冻的」之间的差，正落在运行库自己一拍里 `publish_when_due` 写错之后马上取输入的那条产品路径上。

## 四、测试与变异

### 测的是实现而不是条款

- `after_yielding_for_as_long_as_the_floor_allows_the_next_batch_does_not_yield`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs:1931`）：前台一直在等、两次让路相隔 50 毫秒，「从头一次让路起的挂钟」与「累计让路时长」两种读法答案相同；它与变异第 21 行（`>=` 改 `>`）只钉住边界取等号，钉不住哪一种读法。甲二那一格（中间夹一次没发布的停、隔几秒）没有用例。
- `a_batch_published_clears_the_yields_in_a_row_and_the_next_batch_yields_again`（`crates/singlefs-verification/tests/compaction_batch_computed_out_of_the_session_lock.rs:1965`）：拿签发当「一批发成」（乙二），钉的是实现口径。
- 出错先于一切的两条用例（第 1050、1059 行）都是强制起跑，门不在场；变异第 8 行只钉「出错先于版本比对」，不钉「出错与门谁先」。

### 新行为没有测试，或没有变异行（grep 测试目录与 `crates/mutations.tsv`，推的：改回去会不会红没量）

| 新行为 | 代码 | 测试 | 变异行 |
|---|---|---|---|
| 回锁时排空待发被拒（`BackgroundCompactionRelockRefused::PublishOfThePendingChangesRefused`） | `crates/singlefs-core/src/mounted_session.rs:9233` 起 | `crates/*/tests` 里 grep `BackgroundCompactionRelockRefused` 0 命中 | 无 |
| 观测「签发没资格换段」计数 | `crates/singlefs-core/src/mounted_session.rs:6593`：「issues_given_up_for_a_region_no_longer_eligible_at_the_relock += 1;」 | grep 这个字段名在两个测试目录 0 命中 | 无（第三轮第 14 条点名要的观测） |
| 签发那一支记录变了时「要搬的」取现行那一版 R 里未释放记录数 | `crates/singlefs-core/src/mounted_session.rs:6603`：「u64::try_from(records_of_the_region_now.len()).expect("一段 64 槽")」 | 新用例里没有走到 `HolesOutsideTheRegionFewerThanItsUnitsToMove` 的 | 无 |
| 签发那一支重判资格的几个合取项（开过的段、钉住与隔离、扫描拒过的段、R 里还有记录） | `crates/singlefs-core/src/mounted_session.rs:6580` 起 | 运行库用例 `after_the_chosen_segment_is_no_longer_eligible_at_the_relock_the_pass_calls_again` 只走「R 里记录全放掉」那一项 | 核心层这几项无；运行库第 54 行动的是 `the_pass_calls_again_after` |
| 回锁三种新停因算「这一趟没完」（`stop_holds_back_the_trip_without_ending_it` 里剔完、没资格、冻结丢批记 `true`） | `crates/singlefs-core/src/mounted_session.rs:5263` 起 | 冻结丢批那一条用例断言了这一趟还在途；剔完、没资格两种没看到断言 | 无 |
| 锁外读盘时盘表不是挂载那一份就全当读不到 | `crates/singlefs-core/src/mounted_session.rs:9131` 起 | 无（新用例都用挂载那一份盘表） | 无 |
| 出错那一支与回退门（甲一） | `crates/singlefs-core/src/mounted_session.rs:6333` | 无 | 无 |
| 取输入之前就冻着的那一批（甲三） | `crates/singlefs-core/src/mounted_session.rs:6396` | 无（现有用例只冻在窗口里） | 第 17 行只把整判改成 `false` |

### 变异行改回去会不会照样绿（读码判，推的）

- 第 21 行（让路时长 `>=` 改 `>`）：点名的用例在 100 毫秒那一刻判，`>` 时那一刻照让，用例红——抓得到边界，抓不到读法（上面）。
- 第 22 行（一批发成不清计数）：点名用例下限 1 次、让过 1 次之后签发再判，计数不清就不让，红——抓得到。
- 第 16 行（回锁跳过门）：只改正常那一支的 `if has_the_version_moved` 为 `if false`，出错那一支本来就不判门，这一行改不改都与甲一无关。
- 第 77 行与第 9 行三样全同、合入后删了一行（`mutation-rows-deleted-after-the-merge.txt`），不再重复。

## 五、检查单九格

| 格 | 答 | 依据 / 缺什么 |
|---|---|---|
| 走不到的理由二次查实 | 成立 | 第二节乙三，两处都读码核过（推的，没跑） |
| 类型与名字 | 成立，带一处口径不一 | 补丁新加行里 grep `_ =>` 0 命中；票是 `BatchOutOfTheLock(u64)` 新类型，不与别的计数共用裸整数；新加的 `expect` 都写了依赖的不变量。口径不一是乙二：同一个函数里「一批」既指每次整理发布（清让路计数）又指搬了单元的发布（数批） |
| 不兼容旧代码与旧格式 | 推不出 | 锁里一口气推完的后台入口 `run_compaction_in_the_background` 与 `run_compaction_in_the_background_for_at_most_one_publish` 还是公开入口（选 `gpu` 时在锁里等卡）。`crates/*/src` 里 grep 前者，产品运行库不调，调的只有实验二进制 `crates/singlefs-background-gpu-tests/src/bin/e202_gpu_cpu_gc_comparison/main.rs`；后者被验证包的 `history.rs` 与几份用例调。条款「核心层后台入口拆三段」没说旧入口删不删；缺：主 agent 定这两个入口算「旧路径」还是「测试与实验的参照入口」 |
| 不新建判定库 | 成立（推的） | 新用例与九条 CPU 两段版引的是 `crash_library::multi_host_configuration_on_the_cpu_keeping_the_prune_server_of` 与 `DisposableLibrary`，都是已有的；没读 `DisposableLibrary` 本体 |
| 所有任务多机可用 | 成立（推的） | 只读了引入与文件头：CPU 两段版的配置在用例临时目录里现生成；没逐条核 |
| 测试优先多线程 | 成立（推的） | 运行库那几份一个池一条维护线程，注释写了不切片的理由（例 `crates/singlefs-mount-runtime/tests/mount_runtime_computes_the_batch_out_of_the_session_lock.rs:1358`）；崩溃流走 `enumerate_crash_states_*` 一族没逐条核 |
| 带 --item 的命令与项 | 推不出 | `.claude/gate.d/stage-inputs.tsv` 里 `compaction-batch-computed-out-of-the-lock-around` 命中 4 行、`.claude/gate.d/crash-point-replay-items.tsv` 命中 1 行，`on-the-cpu-judge` 两份各 13、10 行（grep -c）；`--list-items` 列不列得出要跑入口，这条腿不跑 |
| 新崩溃流多罩了什么 | 部分成立 | 新流罩到「锁外期间前台单发一次覆盖写或删文件、回锁剔掉再发」。没罩到的三种回锁前写序列：窗口里 `accept` 收下、回锁时先排空待发再发这一批（新流的前台改动是单发，回锁时待发是空的）；窗口里推抬 F 的空发布；出错那一支删意图（甲一）。前两种在改动前也写得出同样的先后（排空、抬 F 都在一批之前），算不上新序列；第三种是新的，但它是门的政策问题、崩溃一致性上就是一次删意图发布，崩溃枚举本来就判不出它（推的） |
| 放的位置 | 成立 | 新测试在被测包自己的 `tests/`；补丁加行里 grep `#[path` 与 `include!` 0 命中 |

## 六、要主 agent 跑的命令

1. 命令：`SINGLEFS_RUN_SCOPE='种类=target; 跑什么=攻方腿探针 probe_relock_gate_yield_floor_and_frozen_input 的 3 条测试（在冻结副本拷出的运行目录里编、跑）; 范围=3 个测试; 为什么=复跑代码轮第一轮攻方腿甲一甲二甲三三格历史，判决要引的数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus/copy /home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus-run /home/fy5090/code/singlefs/research/prompts/m4-gpu-wait-outside-the-lock-code-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus`（草稿目录；副本在它的 `.deliverables` 写成「交接」）。在新 shell 里起。
   - 看哪几行：`task wait` 末行 `TASK_SERVER_RESULT` 的 `exit=0`；`task-server/task log <号> --all` 里 `name=rerun_done probe_exit=0 result_lines=39`（24 + 1 + 10 + 1 + 2 行 RESULT，加 3 行 SELF；行数是推的）与三行 `SELF … failed=0`。`exit` 非 0 先看日志里是不是编不过（这一份探针没编过）：编不过就新派同立场腿承接改探针，不改结论。
   - 范围声明的种类与写法是照「点名测试函数」那一条推的；被范围声明闸拒了照它的说明改。

## 七、没打中的形状

- 回锁剔不干净（记录逐条相同而单元的字节、分类或映射变了）：一个槽上的单元在 COW 下由它那条记录（槽、跨度、分配代、隔离记号）定，窗口里能写盘的几条路——前台单发改动、`publish_when_due`、重发冻结（原位同字节）、推抬 F 的空发布、用户改动之后另发那一批（只坏一份的旧单元换落点，源记录释放）——要么不碰 R 里的记录、要么让那条记录变（释放或隔离），比整条记录就剔掉。R 排他挡住了「放掉再被别的单元占回同一个槽」。会话内回滚（`mount::roll_back_by_a_forward_publish_in_the_session`）带着同一份 `CompactionOfThisMount` 过去、复活的记录是「多出来的」照剔，而且只有核心层调用方能在窗口里做（运行库没有回滚入口）。读码试了这几种，没打中。
- 隔离记号漏比（签发那一支）：现行那一版没变就不重判资格，但常驻隔离集只从发成的发布并进来（`crates/singlefs-core/src/compaction_plan.rs:932` 那一段文档：发布没成立时那一笔跟着没了），发成就变 txg、就重判。没打中。
- 让路互等、死锁：工作线程回会话锁读盘用不登记的取法；维护线程等的时候自己取会话锁发 `publish_when_due`；会话锁里能取 GPU 实现那把锁的几个入口（两个后台入口、显式入口、取输入）都先判「有一批在锁外」就拒，切换的校验不取那把锁（变异第 1 行钉着）。没找到持会话锁等 GPU 锁、同时 GPU 锁持有者等会话锁的环。没打中（推的）。
- 停标记漏读：等一批结果的几处都读标记；开卡那几等按设计不读；切换开成收进来的卡收进来那一刻记上标记（变异第 74 行）。只剩 CPU 路径（甲四）。
- 维护线程自己的发布与锁外那一段交错：`publish_when_due` 在窗口里发一次与前台单发一次改动在盘上是同一种先后，新崩溃流罩着；窗口里重发冻结成功之后回锁，回锁时没冻结、照常剔再发。没找到新的写序列。
- 票认错会话：`BatchOutOfTheLock` 是一次挂载里的序号，另起一个会话从 1 重数，拿 A 会话的票交 B 会话、B 又恰好有同号的一批在锁外时会被认下；造它要在核心层并开两个会话或经 `with_the_session_and_devices` 换掉会话，产品路径没有，不算。
- 取样范围：探针格一扫 k ∈ {0,1,2,3,4,6,8,12,16,24,32,48}，格二扫 gap ∈ {1000,3000,3999,4000,60000} 毫秒，格三两臂；别的都是读码。

## 八、这条腿自己的限度

- 探针没编、没跑：三格的「推的结果」都是读码推的，判决要引的数只认主 agent 复跑出的那一份；编不过的话三格都只剩读码。
- 读了的：补丁里 `mounted_session.rs`、`compaction_plan.rs`、`crates/singlefs-mount-runtime/src/lib.rs` 的改动全段，`waiting_for_the_gpu.rs` 的改动头一半，`background_compute.rs` 的切换与开批那几段，新验证用例与新崩溃流的文件头与几个函数。没读：`region_scan.rs` 与 `background_gpu.rs` 的改动、九条整理流 CPU 两段版的正文、运行库两份新用例的大部分、`merge-patch/result.md` 的实现员报告。
- 甲二甲三的后果大小（前台多等多久）没量，只说了行为；甲四的卸载挂钟没量。
- 第二节乙一是条款自己的读法问题，这条腿不判哪一句优先。

## 九、没做什么

- 不判正推那几格、不替主 agent 采纳；没往统一任务服务交任务，判决要引的数没复跑（命令在第六节）；副本上的数（复跑之后才有）不算入库装置上的数。
- 冻结副本 `/home/fy5090/.local/share/singlefs/work/m4-gpu-wait-outside-the-lock-code-r1-opus/copy`（55.6 MiB）留着交接，主 agent 复跑完删；没有建编译目录。
