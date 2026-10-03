# m3-conc-r1 攻方腿（Opus）报告（2026-10-02）

<!-- doc-lint:not-numbers Q1 Q2 Q3 Q4 Q5 Q6 Q7 Q8 Q9 Q10 C1 C2 C3 C4 C5 C6 C7 C8 C9 C10 K1 K2 H1 H2 H3 H4 H5 A1 A2 A3 A4 A5 B1 B2 B3 B4 B5 B6 B7 X1 X2 X3 R1 R2 -->

攻击面：① 过度外推与越位，② 内部矛盾与循环依赖，③ 最脆弱的一格（C8）。正文没写主 agent 的倾向，下面按「若定案写成 X」攻每格最自然的那个「现在定」。
引代码一律引冻结副本 `/tmp/claude-1000/m3-conc-r1/tree/`；引 kb 写 kb 文件自己的行号（`grep -nF` 现取）。

## 复跑

模型目录 `research/prompts/m3-conc-r1-opus-model/`，全是独立 Python，不引 `crates/`。仓根执行：

```
bash research/scripts/run-with-memory-cap.sh 2G bash research/prompts/m3-conc-r1-opus-model/rerun.sh /tmp/claude-1000/<草稿>/rerun
```

原样输出（本轮跑的）：

```
same c58.out
same ring_window_inflight.out
same fsync_return_model.out
same k2_capacity.out
same c58_selfproof_sweep.out
```

（上面五行是接手段 2026-10-02 用 2G 上限跑的原样输出；`c58_selfproof_sweep.py` 是接手段加的，`rerun.sh` 随之加了一行、改了用法注释。）

`fsync_probe.py` 是本机此刻的取样（fdatasync 与内存拷贝），不复跑相等，不进 `rerun.sh`；`c58_alarm_model.py` 读入库的 `fsync_probe.out`，所以可复跑。取样命令：`bash research/scripts/run-with-memory-cap.sh 2G bash research/scripts/capped.sh 8 python3 research/prompts/m3-conc-r1-opus-model/fsync_probe.py <草稿目录>`。

`sha256sum`（模型目录，原样）：

```
545b8ae82c5ca39680a6ce89089eabd3ae3e046ac64cb13dddfd789fa59a62e4  c58_alarm_model.py
7c8f74b2d37e34aef9a917615b464d1eb8f30807d989601d2b1d35b59752630f  c58.out
03b13ef76e4002555254fe637ecce8a84ea7f8426507aa69267f82e82a28a65e  c58_selfproof_sweep.out
411dec1adfcbf4d2a588c8987833a20542832c4899362376f793a57a450638e5  c58_selfproof_sweep.py
c01d6f5549548a18e5cb5c619393a28c0061c0cd0bd443497aa25591cfbdeb1d  fsync_probe.out
31df0040423903027a5bfca2a6e0304392f0e5b90085ddee19adad499ab54940  fsync_probe.py
2c2f78e5c2fabcf5feaecf7459ac3cc41620f221f5fa3f2ad65876dcf044d4d9  fsync_return_model.out
924342c24ee46e4f0b20f18396760b4371369d81cedfc2a25ee94f39014d75d4  fsync_return_model.py
ce54f088f2ed9717ec05cf2f866650f404cc7ddb8c612fe5fc3b4aabd0e1bf19  k2_capacity.out
76ef11b3a8af80a28e4881de212751dc961f7ada2b135ca5be03aeeeb8b28a4b  k2_capacity.py
a164ec3323d8ebbfaa3dd956e7e582bd6ad59f6faa283c2c6f3503fb9cfa0c82  rerun.sh
64c9ee4f594f32287ac8626ab085e5cfc5c832c73632b60c4f46f0a1cc10dc0a  ring_window_inflight.out
b334cf2348bf4bbdc5098588fa1231be12193150606917b311ac950d0c7e4be2  ring_window_inflight.py
```

## 各格判定一览

| 格 | 攻击面 | 打中没有 | 一句话 | 依据 |
|---|---|---|---|---|
| C1 Q1 | ① | 打中「现在定」的任何一支 | E166、E169 都不计吞吐；E169 第三段是抽象单位、两个拍的代价点；够判条件（C488 本机等待）一个数都没有 | 第一节 A1 |
| C2 Q2 | ①② | 打中两处 | 第三析取支若读成「返回成功」，B 在 100% 崩溃态上丢（模型）；V_dangling 在多飞·根乱下分得开甲与乙，Q2 的判定随 Q1 子形态翻 | A3、B4 |
| C3 Q3 | ① | 打中「现在改规约加 txg 连续」 | 越过 txg 只在对照丁（无纪律）出现，三条候选臂下 0；而 D23 已定项 14 写着根槽写失败要推进 txg，提前加「txg 连续」会和那一支撞 | A4 |
| C4 Q4 | ② | 部分 | 「整体独占」不论哪个读法，与 Q1丙、Q1甲 合起来都撞 D23 已定项 14 这一版的失败处置（重发成功才建下一次发布） | B1 |
| C5 Q5 | ② | 打中「环窗口不变、无重扣」 | 环窗口 (3S+1)·W 的「+1」就是串行下的那一次在飞；k 个在飞时峰值 (3S+k)·W，30 个取样点 30 个超（模型）；Q5 的两个候选又不互斥（D28 已定项 2 已定案） | B2 |
| C6 Q6 | ①② | 打中「现在加两写者 litmus」 | Q1 没选，同步点列不出；现在加只能写 `none` 锚点，lkmm.sh 检查 4 对它不绑代码 | A5 |
| C7 Q7 | ①② | 打中「定 K2」 | K1 中翻面观测前半（漏），K2 中后半（真实发布形 k = 2 已 2.8e14 态、k = 3 越过 u64）：两个候选都中，打中不分辨臂；Q1 取「准备并行、落盘串行」时 K1 就是精确的 | A2 |
| C8 Q8 | ③ | 打中五处 | 永不响（CPU 代价随盘容量线性，条目数看不见）；总在响（内存盘 mkfs 实测交点 0）；C58 字面的自证漏 2/5 到 4/5 个坏实现（放开负载条目数扫 4 万档；用「≥」时内存盘上计数器没接也绿）；mkfs 实测撞「同参数两次 mkfs 逐字节相同」；计数器的对象今天没有、攒批没落地，「接在会话层」的坏实现今天造不出能抓它的自证 | 第三节 H1–H5 |
| C9 Q9 | ② | 打中「并行候选原样转过去」 | D8 已定项 16 第 6 条（用户 2026-10-01 定）逐字要求每次用户改动先走空，Q9 的「并行」候选与它直接冲突，转过去要带「重开第 6 条」 | B3 |
| C10 Q10 | — | 没打中 | 见「没打中的形状」 | — |
| 单列 | ② | 问题单以外的空白两条 | 并行下的失败处置与「已作废」的定义；Q8「运行时配置」与 D22 已定项 26「系统运行配置」同名不同义 | B1、B6 |

## 一、过度外推与越位

### A1（C1）：Q1 的任何一支现在定都是外推

- E166 自己写「只是模型层的计数：不计时，没有吞吐。」（`.claude/kb/experiments/166-并行提交的共享状态分法.md:166`）。E169 第三段的等待模型写「这些数是抽象单位里的离散事件模型，两个代价取样点是登记拍的」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:237`）。Q1 的够判条件是「每个候选在固定负载上的吞吐与每次 fsync 的等待时间（C488）」，这一轮一个本机数都没有；C488 登记行写的就是「等待时间那一半全仓没量过」（`.claude/kb/checks-owed.md:396`）。
- 外推的具体形状：拿 E169 第三段的 W_max 比值（例：同发 k = 64 的 10.505 对串落 269.120）去支撑「选一次发布装多个事务」。那两行出自单盘、一个先进先出设备队列、P1 / P2 两个拍的代价点；换一个代价点，同发的「长」由 n_dev 换成 n_cpu（几何敏感性 14 行翻面全在同发），数本身不是吞吐。
- 事务数：E166 n ≤ 3，E169 崩溃那两段 k ≤ 3（「单盘、单实例、没有残留记录、没有环回绕；k ≤ 3、r ≤ 2；一次发布只有一个共享单元」，`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:290`），而目标负载「并发 fsync 的个数不设上限，下限 16」（`.claude/kb/decisions/25-目标负载优先级.md:142`）。
- 推翻条件：拿出 C488 那一格的本机数（每个候选在 k ≥ 16 下的吞吐与 fsync 等待），A1 不再成立。

### A2（C7）：「定按事务切（K2）」——两个候选各中一半翻面观测，是打中不分辨臂

Q7 的翻面观测有两半：「某个候选漏掉一种跨两个事务的落盘子集；或某个候选在一条两事务交错的流上状态数超出崩溃放量装置的能力」。

- 前半打中 K1：E167 第二、三段 K1 在两盘上「只在某些录制流上漏」，最大漏 702 / 729（`.claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md:384` 那一行 L11）。
- 后半打中 K2：E167 自己只做了算术，「K1 在真实发布形上的最大一段（L8 的动态规划）与加一种按积展开的方式之后 K2 的代价没量」（`.claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md:165`）。K2 = 各事务链的积（E167 L4「K2 与积的闭式相等」）。我的 `k2_capacity.py` 把它按 k 算出来（原样，节选）：

```
E name=chain check=closed_form_24_2_1_2 value=16777223 e167_s1_reads=16777223 equal=true
E name=chain check=e167_products g1_k2=64 e167_g1_k2=64 g4a_k2=784 e167_g4a_k2=784
E name=k2_states chain=new_pool_file_creation_publish_24_2_1_2 per_transaction=16777223 in_flight=2 states=2.815e+14 exceeds_u32=true exceeds_u64=false cpu_years=1.058e+03 gpu_years=9.349e+01
E name=k2_states chain=new_pool_file_creation_publish_24_2_1_2 per_transaction=16777223 in_flight=3 states=4.722e+21 exceeds_u32=true exceeds_u64=true cpu_years=1.776e+10 gpu_years=1.569e+09
E name=k2_states chain=e167_g4a_two_disks per_transaction=28 in_flight=16 states=1.427e+23 exceeds_u32=true exceeds_u64=true cpu_years=5.367e+11 gpu_years=4.741e+10
E name=u64_overflow_chain_size in_flight=16 smallest_per_transaction_states=16
E name=serial_reference chain=new_pool_file_creation_publish_24_2_1_2 publishes=16 states=2.684e+08 gpu_hours=0.78
```

  两条锚点先对上 E167 的数（64、784、16777223），再往外算。挂钟用的两档吞吐是实测的（CPU 双机 4108 + 4320 态/秒，`briefs/2026-09-28.md` 第 19 行；GPU 取提交 fcc4d340 说明里 26.16 亿态 7 小时 37 分）。u64 是 `closed_form_state_count` 的返回类型：「pub fn closed_form_state_count(segments: &[Vec<usize>]) -> u64 {」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-harness/src/memory_pool.rs:611`）——k = 16 时任何一条超过 16 个状态的单事务链，积就装不进它。
- 四句：① 分不分辨臂：不分辨，K1、K2 各中一半，按 evidence-discipline「打中不分辨臂」不拿 E167 判 Q7。② 系统看得到吗：看得到（状态数是算术）。③ 满足的分句：K1 中前半、K2 中后半，字面各一个分句。④ 改法：「按事务切但只展开罩住段串」在真实发布形上一段 1.46e18 态（E167 锚点 33 行，`.claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md:77`），照样中后半。
- 循环依赖：E167 V1「P1 与 P2 格：K1 状态数 = 真值」（`.claude/kb/experiments/167-多在飞发布的崩溃枚举切法.md:121`）。Q1 取「准备并行、落盘串行」或「一次发布装多个事务」时 K1 就是精确的，Q7 不用换切法；只有 Q1丙 才逼出 Q7。Q1 没定，Q7 定不了。
- 推翻条件：给出一种按事务切的展开法，在真实发布形 k = 2 上状态数进 u32 段内序号、挂钟在 GPU 档内，A2 后半不成立。

### A3（C2）：第三析取支「已确定实例切换或转只读」读成「返回成功」，B 必丢

E169 明写「第三析取支「已确定实例切换或转只读」不在这份登记。」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:289`）。D13 已定项 2 只写「事务号 N 的 fsync 在返回前必须等」（`.claude/kb/decisions/13-验证路线.md:51`），没写返回什么。`fsync_return_model.py`：A、B 整段追加，A 的第 f 条记录写失败，B 的 fsync 走第三析取支返回；恢复按今天的「jsn 严格连续、提交标记齐」（原样，节选）：

```
E name=third_disjunct a=2 b=2 failing_record=1 branch=read_only fsync_returns=success states=8 b_lost_after_success=8
E name=third_disjunct a=2 b=2 failing_record=1 branch=instance_switch fsync_returns=success states=8 b_lost_after_success=8
E name=third_disjunct a=2 b=2 failing_record=1 branch=read_only fsync_returns=error states=8 b_lost_after_success=0
```

`grep -c 'fsync_returns=success'` 16 行全是 `b_lost_after_success` = `states`（B 的记录落在断号之后，前缀永远到不了；实例切换也一样，新实例从前缀末 + 1 接着写，D23 已定项 14 注 3）。模型很小，结论是结构性的：只要 A 有一条记录永远不落盘，B 就不在任何恢复结果里。

- 定案句若照抄「∨ 已确定实例切换或转只读」而不写返回值，就越位成「Q3 翻面观测『fsync 已向 B 的调用方返回而恢复丢掉了 B』可以发生」。不越位的写法（零轮）：第三析取支成立时 N 的 fsync 返回错误，不返回成功。
- 四句：① 分不分辨臂：不分辨 Q2 的甲 / 乙（两者都按号整段），分辨的是「第三析取支返回成功 / 返回错误」两种读法。② 系统看得到：返回那一刻知道自己走的是哪一支。③ 分句：Q3 翻面观测后一分句。④ 改法「返回错误」在打中的 16 格上全 0（模型量过）。
- 另一条线索（推的，未核）：E169 的 R1（已发出即算已追加）与 R2 判决行逐行相同，前提是模型里「屏障罩住它之前发出的全部写」。多个提交者各自往设备发写时，块层的 FLUSH 是否罩住「已发出、未完成」的写，仓里没有条款（C6 登记行写「真盘（不经 QEMU）的 FLUSH 语义也不在这一档射程里」）。定案句不要把 R1 写进去。

### A4（C3）：「现在改 Verus 规约，让前缀核不越过没施加的 txg」越位

- 「越过没施加的 txg」只在没有纪律的对照丁上出现：「三条候选追加臂下四条恢复臂全 0」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:166`）；病根是对照丁按发出次序给 jsn，「丙0 转写 `lib.rs` 只在攒着的那次发布里查 txg」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:157`）。只要 jsn 次序与 txg 次序一致（任何「按事务号整段追加」都满足），越过一个 txg 就要先越过一个断号，严格连续那一条先停。所以它不是前缀核在「有纪律的并行」下的缺陷，判成「今天前缀核在并行下的缺陷」是打中归错了对象（对照丁不是候选）。
- 现在往规约与 `replay_core` 里加「txg 必须连续」会撞一条已定未实现的分支：「根槽写失败重发时 checkpoint_txg 推进一格再发」（`.claude/kb/decisions/23-journal的角色与格式.md:398`），同一节又写「原样重发管这一版的串行提交；探针写、两支分流、实例切换与根槽写失败推进 txg 仍在本项」（`.claude/kb/decisions/23-journal的角色与格式.md:401`）。根槽写失败推进 txg 之后，环里会出现一个记录在、根不在的 txg，接着是下一个 txg 的发布——串行下的合法历史就有 txg 缺口（推的：那次失败的记录重发时怎么编号，条款没写到字节）。提前加「txg 连续」等于替那条分支把字节定了。
- 不越位的写法（零轮）：Verus 规约这一轮不改；转里程碑四时带一句「并行下的前缀判定要么靠追加纪律保证 jsn 序与 txg 序一致（今天的核不用改），要么核里加 txg 判定——后者要先和 D23 已定项 14 根槽写失败推进 txg 那一支一起定」。
- 推翻条件：D23 已定项 14 的根槽写失败分支被写成「推进 txg 时同时废掉旧 txg 的全部记录、新记录接着旧 jsn」（这样缺口之前的记录永远读不成前缀），那一支就不和「txg 连续」撞。

### A5（C6）：「四·1 现在加两写者 litmus」——同步点列不出，加了也绑不到代码

- Q6 的够判条件是「列出 Q1 选定形态的每个同步点」。Q1 没定（A1），同步点没有可列的对象。
- 现在加的 litmus 只能写 `none` 锚点：今天唯一的 `none` 是「singlefs-models: none —— COW 发布序的通用形态（块内容先于指向它的系统配置）；新池新建文件的具体顺序由 new-pool-file-creation-* 两条绑着代码。」（`litmus/commit-publish.litmus:13`），它有两条绑代码的同族兜着；并行提交的新 litmus 没有同族，而 lkmm.sh 检查 4 的理由正是「每条 Never 都要绑到代码。herd7 只判 litmus 写下的那个形态」（`.claude/scripts/lkmm.sh:39`）。现在加 = 一条门禁永远绿、与任何代码无关的 Never。
- 罩不到的同步点（推的）：D13 已定项 2 里「作废是提交管道的内存状态，对并发后继可见」（`.claude/kb/decisions/13-验证路线.md:51`）是一个纯内存的跨线程可见性点；今天 8 份 litmus 建的是设备写序（一个写者一个读者），Verus M2 管的是盘上前缀，两样都罩不到它。所以「两样都扩」也不等于罩全，Q6 的候选清单缺一支「内存里的提交管道同步（Rust 侧，loom 这一类）」——这是问题单以外的一条空白，交主 agent。
- 推翻条件：Q1 选定、同步点清单写出来，且每个点在 litmus 或 Verus 里有一处绑到 `crates/` 的 `fn`。

## 二、内部矛盾与循环依赖

### B1（C1、C4，单列空白）：并行的两种在飞形态都撞 D23 已定项 14 这一版的失败处置

- 条款：「**这一版的失败处置**：发布不接受失败。」与同一行的「重发成功才建下一次发布」（`.claude/kb/decisions/23-journal的角色与格式.md:401`）。实现：落盘失败时「std::mem::replace(allocator, allocator_before_this_publish);」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/transaction.rs:1155`）把分配器换回发布之前，分配器上「frozen_publish: Option<Box<FrozenPublish>>,」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/allocator.rs:1059`）至多挂一个冻结的发布。
- Q1丙（多次发布同时在飞）：txg N+1 的单元在 N 的根之前发出，N 落盘失败时 N+1 已经建好、写已发出——字面违「重发成功才建下一次发布」。Q1甲（准备并行、落盘串行）：N+1 的计划是在「N 之后的分配器」上算的，N 失败、分配器换回之后那份计划作废；条款写的是「建」，准备也算建。只有 Q1乙（一次发布装多个事务）不撞。
- 同一处让 D13 已定项 2 的第二析取支在这一版里没有对象：「作废是提交管道的内存状态，对并发后继可见」（`.claude/kb/decisions/13-验证路线.md:51`），而这一版的失败处置只有冻结与原样重发，没有一条路径把一个事务记成「已作废」。E169 的 H1（作废·窄，32 种）量的是这一版的实现走不出的历史。
- E166 的 A（「锁罩住从读到写回整段」，支持者的样子）与 Q1丙 合起来同样撞：写回的是没落盘的内存态，N+1 在它上面算。所以 C4 的「整体独占读法归谁定」不只是 A 与 A字面之争：A 的定义还要写明「写回」写到哪（内存还是盘），写到内存就要先有并行下的失败处置。
- 去向（问题单以外的空白）：「并行下一次发布落盘失败时，已在飞 / 已准备的后继怎么办，『已作废』由谁、在哪一刻记」，转里程碑四时与 Q1 一起带走，并写明 D23 已定项 14「原样重发管这一版的串行提交」那句是射程，不是并行的答案。
- 推翻条件：D23 已定项 14 里已有一句给并行写了失败处置（我 `grep -nF '作废' .claude/kb/decisions/23-journal的角色与格式.md` 只在别的语境命中，没找到）。

### B2（C5）：环窗口的「+1」就是串行那一次在飞；k 个在飞时窗口少扣 (k − 1)·W

- 条款：「**环窗口**（常驻扣项）：删除各批新写的兜底，环窗口 = (根环槽数 3 × S + 1) × W」（`.claude/kb/decisions/28-挂载期承诺量.md:29`）；那个系数第三轮只在串行下攻过：「「(环里根数 + 1)」这个系数在 W 取对时罩得住」（`research/prompts/m3-delete-r3-main-verification.md:42`）。
- `ring_window_inflight.py`：一串删除批，每批重写同一组 W 个节点，副本只被写它的那一个根引用，根离开根环（3S 个）才回收；同时在飞 k 个。原样，节选：

```
E name=ring_window slots_per_region=4 w=24 in_flight=1 peak_live_slots=312 window_slots=312 closed_form_peak=312 peak_le_window=true deficit_slots=0
E name=ring_window slots_per_region=4 w=24 in_flight=2 peak_live_slots=336 window_slots=312 closed_form_peak=336 peak_le_window=false deficit_slots=24
E name=ring_window slots_per_region=4 w=87 in_flight=16 peak_live_slots=2436 window_slots=1131 closed_form_peak=2436 peak_le_window=false deficit_slots=1305
E name=ring_window slots_per_region=16 w=87 in_flight=16 peak_live_slots=5568 window_slots=4263 closed_form_peak=5568 peak_le_window=false deficit_slots=1305
```

  36 行里 `peak_le_window=true` 6 行（全是 in_flight=1），`false` 30 行；峰值逐行等于 (3S + k)·W（awk 比 36 行，不等 0 行）。k = 1 与条款逐格相等，说明模型的「+1」读法和第三轮一致。
- 所以 C5 若定成「在飞预留与环窗口、墓碑预付不重扣，环窗口不动」，只在 Q1甲、Q1乙（同一时刻至多一次发布在落盘）成立；Q1丙 下环窗口要成 (3S + k)·W，k 由并发数定（D25 已定项 7 下限 16），而环窗口今天按几何现算、不看 k。Q5 依赖 Q1。
- 第二处：Q1乙 把一次删除批与几个用户事务装进同一次发布时，接线那一句按发布分类——「删除类发布（意图、各批、挂载里的续做批）与管理员回退也不判式子」（`.claude/kb/decisions/28-挂载期承诺量.md:36`）——一次混装的发布判不判式子没有定义。
- 第三处：Q5 的两个候选不互斥。D28 已定项 2 已定案「**定案**：准入读数 = 已发布统计量 − 在飞已批准的预留」（`.claude/kb/decisions/28-挂载期承诺量.md:71`），「已批准的预留」就是候选二的「进门时按最坏需求预留」；问题单把候选一指到 D3 已定项 6，而那一格自己写「**定案**：权威正文在 D28（挂载期承诺量） 已定项 2」（`.claude/kb/decisions/03-空间分配.md:112`）。C5 能定的只剩 C87 欠的那一半（overlay 的强制清零开关），不是「选哪个候选」。
- 四句：① 分不分辨臂：分辨 Q1 的子形态（丙中、甲乙不中）。② 系统看得到：在飞数 k 在提交管道里看得到，但环窗口的式子不读它。③ 分句：Q5 翻面观测前一分句「并发准入在某个交错下超卖」（删除批不判式子，超出的是落点那一判之外的槽；按最弱读法记成「环窗口兜不住」）。④ 改法「环窗口改成 (3S + k_max)·W、k_max 是提交管道的在飞上限」在 30 个打中格上按闭式全过（模型里 closed_form_peak 就是它，量过），代价是常驻扣项随 k_max 线性涨，零轮。
- 推翻条件：并行形态里删除批被规定为不与任何发布同时在飞（删除批强制串行），那么 k 对删除副本恒 1，B2 不成立。

### B3（C9）：Q9 的「并行」候选与用户刚定的续做点直接冲突

- 条款：「会话里每一次用户改动（含 unlink、写、建 inode）先把现行那一版的 `deleted_inodes` 队列走空，再做它自己」（`.claude/kb/decisions/08-核心索引结构.md:463`），用户 2026-10-01 定。Q9 的「并行：续做批与用户事务同时在飞，按事务号排序」要求用户事务在队列没走空时就进管道，与「先……走空，再做它自己」字面相反。
- 所以串行下 Q9 已经没有可判的东西（「续做独占」在会话里已被第 6 条推出来）；转里程碑四时不能原样带「两个候选」过去，要带的是「选并行就要重开 D8 已定项 16 第 6 条，而第 6 条是 m3-blanks-r1 判决里只有两个臂守住队列上界的那一臂（另一臂是回退之后续做）」，否则里程碑四会在一个已定条款上重新判一遍候选、而不知道自己在推翻什么。
- 并行下还有一处要带：第 6 条的走空是一串删除批发布，D13 已定项 2 下用户事务 N 的 fsync 要等号更小的删除批全部已追加；走空的批数随被删文件大小涨（D8 已定项 16 第 2 条按 144 个单元一批），那一段等待不在 E169 第三段的任何一格里（它只跑了 H0、每个事务 r 相同）。
- 推翻条件：第 6 条被改写成「先走空」只约束同一个会话线程、而并行提交在多个会话之间（主 agent 判「并发」指的是哪一层）。

### B4（C2）：V_dangling 在多飞·根乱下分得开甲与乙，Q2 的判定随 Q1 子形态翻

- E169 第二段那一行原样：「| Q3-3 V_dangling | 只对照丁 20，四条恢复臂同值：F8（打中不分辨臂） | 乙整段 552、对照丁 488，四条恢复臂同值：F8；甲整段、甲交错 0 |」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:167`）。它对 Q3 的四条恢复臂不分辨（四臂同值），所以按 F8 不判 Q3 是对的；但对 Q2 的两条候选是分辨的：根乱下乙整段 552、甲整段 0。页面把它记成「Q1 条件的共有前提」（`.claude/kb/experiments/169-并行提交的追加纪律与前缀恢复.md:171`），对 Q2 来说是打中归错了判据。
- 结果：若 C2 定案写「E169 分不开单一追加者与预留槽位」，这一格反驳它；若写「定预留槽位（乙）」，这一格在 Q1丙 的根乱子形态下打中乙。Q2 能不能定、定哪个，依赖 Q1 是不是丙、根槽写之间有没有次序约束——又一条依赖 Q1 的。
- 四句：① 分辨 Q2 的甲 / 乙（根乱下），不分辨 Q3。② 被判系统（恢复）看得到悬空的根（根指向没落盘的单元），这个违例是落盘次序造成的，不是恢复判错。③ 分句：E169 登记的 V_dangling 是「根先于前一次发布的单元持久」，属 Q2 翻面观测哪一句，问题单 Q2 行没有对应分句——Q2 行只有「两个半截事务的前缀」与「fsync 等待无界」两句，这一格说明 Q2 行少一句（零轮）。④ 改法「根序（F、C 按 txg 发出）」在那 552 个状态上让乙整段归 0（第二段根序列乙整段 0，页面同一行）。
- 推翻条件：主 agent 判 V_dangling 是 Q1「根乱」这个子形态自己出局的理由（多飞一律加根序），那么 Q2 在剩下的根序下仍分不开，B4 只剩「页面归类不严」。

### B5（C1–C10）：依赖图——Q1 定不了，Q2、Q5、Q6、Q7 就定不了

| 格 | 依赖 Q1 的哪一点 | 出处 |
|---|---|---|
| Q2 | 根乱子形态下甲 / 乙分得开 | B4 |
| Q5 | 在飞数 k 进不进环窗口；Q1乙 混装发布判不判式子 | B2 |
| Q6 | 同步点清单 | A5 |
| Q7 | 只有 Q1丙 逼出换切法；Q1甲、乙 下 K1 精确 | A2 |
| Q4 | A 的「写回」写到哪；Q1丙 下撞失败处置 | B1 |
| Q3 | 不依赖：今天的核在任何「jsn 序 = txg 序」的纪律下都对 | A4 |
| Q8 | 不依赖：报警立在串行提交上 | 第三节 |
| Q9 | 不依赖 Q1，依赖「并发」指哪一层 | B3 |
| Q10 | 不依赖 | — |

而 Q1 自己现在定不了（A1）。所以「现在定」只可能落在 Q3（不改）、Q8、Q9 串行下的剩余（已由第 6 条推出）、Q10（转）；Q2、Q5、Q6、Q7 写「现在定」都是循环依赖。

### B6（C8，单列空白）：Q8 的「运行时配置」与仓里已有的「系统运行配置」同名不同义，且今天没有通道

- 问题单 Q8 候选二定义成「由调用方给，不进盘」；C58 登记行写的是「写进系统配置或运行时配置」（`.claude/kb/checks-owed.md:59`）。仓里「运行时配置」已经是盘上的一档：「| 系统运行配置 | 36 字节 | 无 | mkfs 时设默认值；**每次挂载可重设，而且要回写盘上** |」（`.claude/kb/decisions/22-单元原子性怎么合成.md:229`），代码注释「系统运行配置：36 字节，改它无代价、每次挂载可重设并回写」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/system_configuration.rs:217`）。C58 的「运行时配置」按仓里的用法读是进盘的，问题单把它改成了不进盘——Q8 的三个候选里实际缺一个「挂载时传、回写进系统运行配置」。
- 「由调用方给」今天没有入口：「**没有字段**：挂载选项通道不存在（C422（挂载选项通道不存在）），这三行今天既没有可设的入口、」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/system_configuration.rs:220`）。实现员照「运行时配置」写不出来，除非先还 C422。
- 调用方不给时报警用什么值，候选定义没写；`c58.out` 那一行：`E name=threshold source=runtime_caller_given env=caller_gives_nothing T_entries=undefined note=default_not_in_Q8_definition mount_option_channel=C422_absent`。默认不定 = 默认不响。

### B7（C8，推的）：编译期常量与「按盘与机器的性能定」的口径冲突

「**定案**：并发 fsync 的个数不设上限，下限 16；具体取多少按盘与机器的性能定，不写死成常量。」（`.claude/kb/decisions/25-目标负载优先级.md:142`）管的是另一个量，不直接约束 C58；但同一个仓对「按机器定的数」已有一次「不写死成常量」的用户定案，编译期常量那一支要说明为什么这里可以写死。只记一笔，不当打中。

## 三、最脆弱的一格：C8（C58 报警）

报警按 C58 登记行：「⚠️ **判别力自证**：造一个条目数刻意翻倍的负载，检查必须红；不红说明计数器没接到提交路径上」（`.claude/kb/checks-owed.md:59`），交点「单线程合并速率 × 一次 fdatasync 的时长」。模型 `c58_alarm_model.py`，本机取样 `fsync_probe.py`。

### H1：永不响——天花板已经在 CPU 上，条目数还没到交点

- C58 的前提「**单线程 sort-merge 是关键路径**」（`.claude/kb/checks-owed.md:59`）在 `crates/` 里没有对象：冻结副本上 `grep -rlE 'write_buffer|WriteBuffer|sort_merge|sort-merge' crates --include=*.rs | wc -l` 输出 `0`。今天提交路径的 CPU 里至少有一项与条目数无关、随盘容量线性：「整个分配器拷一份、失败就换回去：第一版每次发布约 450 KiB 的拷贝（两盘各一张 211968 位的位图）」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/transaction.rs:5050`），C567 登记行「`crates/singlefs-core/src/transaction.rs` 每次发布 clone 一份分配器——分配与发布的代价随盘容量线性」（`.claude/kb/checks-owed.md:442`）。
- 本机取样（`fsync_probe.out` 原样）：

```
E name=fsync_round target=nvme_ext4_draft_dir round=1 syncs=200 median_us=589.6 mean_us=590.8
E name=copy target=first_version_allocator_450KiB bytes=460800 median_us=8.9 bytes_per_second=5.204e+10
E name=copy target=one_tib_bitmaps_128MiB bytes=134217728 median_us=30197.4 bytes_per_second=4.445e+09
```

  `c58.out` 原样：

```
E name=silent_band allocator_copy=one_tib_bitmaps_128MiB copy_bytes=134217728 cpu_floor_us=30197.4 fsync_us=589.6 cpu_over_fsync=51.217 source=compile_time_constant T_entries=11200 ceiling_on_cpu_from_entries=1 alarm_silent_entries=[1,11200)
E name=silent_band allocator_copy=one_tib_bitmaps_128MiB copy_bytes=134217728 cpu_floor_us=30197.4 fsync_us=589.6 cpu_over_fsync=51.217 source=mkfs_measured_nvme_today T_entries=18395 ceiling_on_cpu_from_entries=1 alarm_silent_entries=[1,18395)
```

  具体历史：两块各约 1 TiB 的盘建池（128 MiB 是推的：每盘 6400 万槽、一槽一字节，C567 写的是审阅员推的 190 MB），之后每次覆盖写一个单元、每次 fsync。光拷分配器的下界就是一次 fdatasync 的 51 倍，天花板在 CPU；条目数是个位数，三个交点来源哪一个都不响。第一版几何上拷贝只占 0.015 倍，所以这一格今天的测试几何上看不见。
- 四句：① 不分辨臂：三个交点来源一起中，病根是共用的前提「CPU 代价 ∝ 条目数」。② 判别子观测不到：报警做决定时只看得到条目数，「天花板在 CPU」与「天花板在设备」两种情形条目数逐个相同——按 evidence-discipline 是「判别子观测不到」，要改判据，不是换来源。③ 分句：Q8 翻面观测前一分句「某个候选在别的机器上让报警永不响」（这里是别的盘容量，同一台机器）。④ 三个候选改法都不碰这一格；见文末改法表 X3。
- 推翻条件：量到大盘上一次发布的 CPU 时长仍小于一次 fdatasync（例如分配器改成不整拷，C567 还清），H1 不成立。

### H2：总在响——mkfs 实测在测试盘上量到 0

- 内存盘的屏障是空的：「fn barrier(&mut self) -> Result<(), singlefs_core::block_device::BlockDeviceError> {」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-harness/src/memory_pool.rs:209`）下一行就是 `Ok(())`。harness 与崩溃放量的池都建在它上面。`c58.out` 原样：

```
E name=threshold source=mkfs_measured env=harness_sparse_block_device_barrier_noop merge_rate=c58_point fsync_us=0.0 T_entries=0 rings_on_one_entry_publish=yes
E name=threshold source=mkfs_measured env=tmpfs_dev_shm_median_max merge_rate=c58_point fsync_us=0.1 T_entries=3 rings_on_one_entry_publish=no
```

  「mkfs 时实测」在每一个内存盘测试池上把交点写成 0，每次发布都越过；文件盘落在 tmpfs 上时交点 2–3 条。具体历史：任意一条 harness 用例（mkfs → 新池新建文件 → 覆盖写）。
- 四句：① 分辨臂：只打中「mkfs 实测」。② 看得到：mkfs 那一刻知道自己量到的是 0，但候选定义没写量到 0 怎么办。③ 分句：Q8 翻面观测「总在响」那一半。④ 改法：「mkfs 量到低于某个下限就不写、报不适用」修这一格（推的），但随后 H3 的自证就要在一块会等的测试盘上做，见 X1。
- 推翻条件：mkfs 实测这一支规定只在真设备上量、内存盘与 tmpfs 上写一个固定值，且那个固定值的来源另有自证。

### H3：C58 字面的判别力自证——计数器没接上提交路径，检查照样绿

C58 的自证只写了一面：「造一个条目数刻意翻倍的负载，检查必须红」。没写基线负载不红，也没写交点走哪条路。

- 具体反例（`c58_alarm_model.py` 的坏实现 `counter_not_wired`：计数器恒 0，比较写成「≥」）：harness 用例在内存盘上建池，mkfs 实测交点 0（H2）；自证造 2·base 条的负载。正确实现 2·base ≥ 0 红，坏实现 0 ≥ 0 也红，自证绿，计数器没接上而检查不红。
- 这一格放开扫过（接手段新加 `c58_selfproof_sweep.py`：自证负载的基线条目数 base 取 1..40000 共 4 万档，屏障等待取 0 / 0.1 / 50 / 100 / 359.1 / 589.6 µs 六档，比较符号取「>」「≥」两种）。`c58_selfproof_sweep.out` 原样：

```
E name=sweep comparator=ge latency_us=0.0 test=literal bases=40000 usable_bases=40000 base_min=1 base_max=5599 n=5599 caught=threshold_max_unset,threshold_ignores_device_constant missed=counter_not_wired,counter_at_session_per_user_change,threshold_zero
E name=sweep comparator=ge latency_us=0.0 test=literal bases=40000 usable_bases=40000 base_min=5600 base_max=40000 n=34401 caught=threshold_max_unset missed=counter_not_wired,counter_at_session_per_user_change,threshold_zero,threshold_ignores_device_constant
E name=sweep comparator=gt latency_us=0.0 test=literal bases=40000 usable_bases=40000 base_min=1 base_max=5600 n=5600 caught=counter_not_wired,threshold_max_unset,threshold_ignores_device_constant missed=counter_at_session_per_user_change,threshold_zero
E name=sweep comparator=gt latency_us=0.0 test=literal bases=40000 usable_bases=40000 base_min=5601 base_max=40000 n=34400 caught=counter_not_wired,threshold_max_unset missed=counter_at_session_per_user_change,threshold_zero,threshold_ignores_device_constant
```

  「≥」下 4 万档 base 一档都抓不到 `counter_not_wired`；「>」下抓得到，但 `threshold_zero`（交点量成 0）在 4 万档上全漏，因为内存盘上正确实现自己的交点就是 0。C58 原文是「超过交点即判红」，「超过」按字面是「>」，可这一句没有检查钉着，实现员写成「≥」，自证不会提醒他。
- 上一段（被停的那条腿）的 `c58.out` 把 base 写死成 60，报「C58 字面漏 2/5」。放开之后这个数随 base 翻：`threshold_ignores_device_constant`（交点用编译期常量 11200，不看设备）只在 base ≤ 5600（「>」）或 ≤ 5599（「≥」）时抓得到，base 更大就漏。所以 C58 字面自证漏的坏实现是 2/5 到 4/5，取决于测试作者挑的负载大小与比较符号。规则里点过这种情况（固定的后缀恰好避开落点），上面一览表那一行已按扫过的数改。
- 第二个具体形状：自证走测试开关。测试直接给交点（`knob`），生产那条「量 → 写进配置 → 读出来」的路一步都不走。`c58.out` 原样：

```
E name=selfproof_combo comparator=gt combo=c58_literal_with_knob tests_green_on_correct=1/1 mutants=5 missed=4 missed_list=counter_at_session_per_user_change,threshold_zero,threshold_max_unset,threshold_ignores_device_constant
```

- 两面自证（基线不红、翻倍红）在内存盘上根本写不出来：交点 0 时基线负载必红。原样：

```
E name=sweep comparator=gt latency_us=0.0 test=two_sided bases=40000 usable_bases=0 caught=- base_range=-
```

  要写两面，测试盘就得在屏障里等一个已知时长（收严写法，零轮）。等 100 µs 时 base 只能落在 (T/2, T] 这一窗里（1561..3120），落进去能抓 4/5；等的时长与编译期常量的来源相同（359.1 µs，E44 的数）时，常量那个坏实现在 5602..11200 这一大段 base 上照样漏。原样：

```
E name=sweep comparator=gt latency_us=100.0 test=two_sided bases=40000 usable_bases=1560 base_min=1561 base_max=3120 n=1560 caught=counter_not_wired,threshold_zero,threshold_max_unset,threshold_ignores_device_constant missed=counter_at_session_per_user_change
E name=sweep comparator=gt latency_us=359.1 test=two_sided bases=40000 usable_bases=5602 base_min=5602 base_max=11200 n=5599 caught=counter_not_wired,threshold_zero,threshold_max_unset missed=counter_at_session_per_user_change,threshold_ignores_device_constant
```

- 四句：① 分不分辨臂：不分辨 Q8 的三个交点来源，打中的是三者共用的自证写法；比较符号与负载大小两个旋钮决定漏几个。② 系统看得到吗：自证在测试里，测试看得到计数器的值，所以这一格可以改，属于「判据写得不全」，不是「判别子观测不到」（那一种在 H5）。③ 分句：Q8 翻面观测后一分句「计数器没接到提交路径上（造一个条目数翻倍的负载，检查不红）」——这里是反过来，计数器没接而检查红了、自证过了，按最弱读法记成这一分句打中。④ 改法：Q8 的三个候选都不改自证写法，对这一格都不起作用；收严写法见文末 X1、X2。
- 推翻条件：C58 的自证被写成两面，并规定比较符号与测试盘的屏障等待，且在 4 万档 base 上逐档抓到 `counter_not_wired`。

### H4：「mkfs 时实测、写进系统配置」撞里程碑一已定的「同参数两次 mkfs 逐字节相同」

- 已定：「- 同参数跑两次 mkfs，两个镜像逐字节相同——fsid 与时间戳因此都是 mkfs 参数，不是随机量（E142（新池新建文件的干跑） 2026-09-13：字节表一写「fsid = mkfs 随机」，与这一句矛盾，按这一句办）。」（`.claude/kb/milestone/01-first-txn.md:63`）。代码照它写：「/// mkfs 的参数：fsid 与时间戳都是参数，同参数两次 mkfs 逐字节相同（里程碑步 1 验收）。」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/make_filesystem.rs:65`）。
- 打穿的历史：同一台机器、同一组参数跑两次 mkfs。每次 mkfs 量一次 fdatasync，写进系统配置的交点不同，两个镜像不逐字节相同。本机 5 轮取样里中位数有 3 个不同的值，算出 3 个不同的交点。`c58.out` 原样：

```
E name=mkfs_reproducibility rounds=5 distinct_medians_at_0.1us=3 distinct_T_values=3
```

- 两头都不好：在内存盘上量，屏障是空的，每次都量到 0，镜像能复现，可报警总在响（H2）；在真盘上量，交点跟着抖，镜像就不能复现。要守住那一句，只能把量到的值改成 mkfs 的参数，由调用方量好了交进来。这样「mkfs 时实测」就成了「调用方给、写进盘」，问题单 Q8 的三个候选里没有这一个（B6 说的缺口是同一个）。
- 同一台机器，量的时候不同，交点就差得很远。C58 用的 359 µs 来自 E44（「| `sync_1`（每块一次 fdatasync） | **2785** | 0.42% |」，`.claude/kb/experiments/44-序号位宽的本机实测与代价.md:64`），今天我在草稿目录（NVMe 上的 ext4）里量到中位数 589.6 µs。`c58.out` 原样：

```
E name=disagreement same_machine=yes compile_time_T=11200 mkfs_today_T=18395 entries_judged_differently=[11200,18395) width=7195
```

  条目数在 [11200, 18395) 这 7195 档上，编译期常量与今天的 mkfs 实测判得相反。限定：E44 的 `sync_1` 和我的取样不是同一个装置（我用 4 KiB pwrite + fdatasync 写一个普通文件），差了多少是盘在抖、多少是装置不同，我没分开，这一半是推的。
- 崩溃放量那一侧（推的）：复用键里有「起步的镜像」一项（「/// 一步的两样属性（各是 SHA-256）：这一步写下的字节（带屏障的写表）与它起步的镜像。只有流程进复用键：」，`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-checker-tier/src/crash_identity.rs:89`）。池如果建在真文件盘上、mkfs 又每次量，每次跑的起步镜像都不一样，判定库里的旧行一行都复用不上。今天崩溃放量的池建在内存盘上，所以这一条今天不发作，H2 那一条发作。
- 格式代价（推的，没核到字节）：系统配置是「//! 系统配置（D22（单元原子性怎么合成） 已定项 9 / 已定项 15 / 已定项 16 / 已定项 26）：489 字节住 4096 字节的槽，」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/system_configuration.rs:1`）。槽里还有空间，但加字段要改 `layout/01-first-txn.md` 的字段表，还要判它属于 D22 已定项 26 的哪一档可改性。我没查这算不算 D15 意义上的格式变更。
- 四句：① 分辨臂：只打中「mkfs 时实测」，另外两个候选不在 mkfs 里写测出来的值。② 系统看得到吗：mkfs 知道自己量到了什么，这一格打中的是已定条款之间的矛盾，不涉及观测。③ 分句：Q8 够判条件「每个候选的交点来源能不能复跑」——这里答「不能」，而且和一条已定验收冲突。④ 改法：「量到的值当 mkfs 参数」修这一格（推的），可这样一来它就不是问题单定义的那个候选了，要改问题单。
- 推翻条件：用户把那条验收改成「除交点字段外逐字节相同」，或者规定交点只在第一次挂载时量、写进系统运行配置（那就落到 B6 说的缺口上）。

### H5：计数器接在提交路径哪一步——今天没有可数的对象，「接在会话层」的坏实现造不出能抓它的自证

- C58 数的是 write buffer 的条目：前提是「**单线程 sort-merge 是关键路径**（D8（核心索引结构） 的 write buffer 三条硬要求）」（`.claude/kb/checks-owed.md:59`）。第一版不开 write buffer：「| 12 | **write buffer 前端与节点消息缓冲区的衔接** | 一棵树只准一级：走前端的三棵树 ε 恒 0、写一律进前端、发布前排空到叶；第一版两级都不开 **状态：已定。** |」（`.claude/kb/decisions/08-核心索引结构.md:22`），记账也不走前端：「| 10 | **第一版记账走不走 write buffer 前端；`seq` 的取值规则** | 第一版记账直落记账树叶、不走前端；`seq` = 同一个 key 在一次发布里被写进树的第几次，从 1 起，第一版恒 1 **状态：已定。** |」（`.claude/kb/decisions/08-核心索引结构.md:20`）。
- 所以第一版里「本次 checkpoint 的条目数」没有对象。实现员要么数一个别的东西（每次发布的记录数、单元数、树条目改动数），要么什么都不数。数别的东西时，交点的单位是 E17 量的「write buffer 合并条目」，计数器的单位不是，两边比的不是同一种量（推的）。什么都不数时，就是 H3 那个 `counter_not_wired`。
- 「每次 checkpoint」与「每个用户改动」今天逐个相同：攒批已定（「**定案**：攒够一批脏节点（或到时间）再发布一次根，不是每个事务发一次。」，`.claude/kb/decisions/16-发布语义.md:233`），没实现：会话的入口「    pub fn publish_user_change<Device: BlockDevice>(」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/mounted_session.rs:243`）一次收一个「        change: UserChange<'_>,」（`/tmp/claude-1000/m3-conc-r1/tree/crates/singlefs-core/src/mounted_session.rs:246`），发一次。计数器接在会话层、每个改动清零（`counter_at_session_per_user_change`），在今天造得出的任何历史上都与接在 checkpoint 上的计数器读数相同。扫描里只有「一次 checkpoint 装两个改动」那条自证抓得到它，而那条自证今天造不出来。原样：

```
E name=sweep comparator=gt latency_us=100.0 test=batching bases=40000 usable_bases=1560 base_min=1561 base_max=3120 n=1560 caught=counter_not_wired,counter_at_session_per_user_change,threshold_zero,threshold_max_unset,threshold_ignores_device_constant missed=-
```

  `test=literal` 20 行、`test=two_sided` 里可用的（`usable_bases` 不为 0）14 行，全把 `counter_at_session_per_user_change` 记在 missed 里。现数的命令与原样输出（在模型目录里跑）：`grep -c 'test=literal' c58_selfproof_sweep.out` 得 `20`，再接 `grep -c 'missed=[^ ]*counter_at_session_per_user_change'` 得 `20`；`grep 'test=two_sided' c58_selfproof_sweep.out | grep -vc 'usable_bases=0 '` 得 `14`，同样再筛 missed 得 `14`。
- 正文第一节第四条给的现查命令，`grep -rn 'checkpoint_entry\|crossover\|C58' crates/` 在冻结副本上命中 23 行，全是 C583、C585、C586；加上词界（`grep -rnE 'checkpoint_entry|crossover|C58([^0-9]|$)' crates/`）命中 0 行。拿前一种写法做「计数器接上没有」的门禁，零实现也是绿的。
- 四句：① 不分辨臂：三个交点来源都要一个计数器，病根在计数对象上。② 判别子观测不到：今天任何一个测试只能看到「一次发布、一个改动」，「接在会话层」与「接在 checkpoint 上」两种实现读数逐个相同，要等攒批落地才分得开。③ 分句：Q8 翻面观测后一分句（计数器没接到提交路径上）——接在会话层就是没接在 checkpoint 上。④ 改法：Q8 的三个候选都只管交点从哪来，没有一个碰到这一格。
- 推翻条件：第一版先开 write buffer 前端（重开 D8 已定项 12），或者 C58 把计数对象改写成一个第一版里有的量，并且为那个量另外实测合并速率。

### H 系列补一笔：编译期常量换机器（推的，条件式）

`c58.out` 的 `disagreement` 那一行只量了同一台机器。换到 fdatasync 快得多的盘（例：带掉电保护的企业盘），编译期常量 11200 就高于那台盘的真交点，条目数落在 [真交点, 11200) 时报警不响；换到更慢的盘，就落在 [11200, 真交点) 时误响。按 20 µs 一次同步算，真交点约 624 条，这一段不响的有一万多档。20 µs 是拍的，本机测不到那一档，这一条是条件式，不是量到的数。

## 四、改法表（只在我的模型上量过、被攻过零轮）

| 改法 | 定义 | H1 永不响（CPU） | H2 总在响（内存盘 0） | H3 字面自证漏 | H4 撞 mkfs 逐字节相同 | H5 计数对象没有 |
|---|---|---|---|---|---|---|
| X1 | 自证写成两面（基线不红 + 翻倍红）；比较符号钉成「>」；测试盘的屏障等一个已知时长，取得离编译期常量的来源（359 µs）远；mkfs 或挂载量到低于下限时不写交点、报不适用 | 不中（推的：判据仍只看条目数） | 修（推的：低于下限不写） | 修 4/5，漏 `counter_at_session_per_user_change`（量过：`c58_selfproof_sweep.out` 第 15 行；等 359.1 µs 时还漏常量那个，第 18 行） | 不中（推的） | 不中（量过：14 行可用的两面自证全漏会话层那个） |
| X2 | X1 再加交点来源自证：注入已知等待之后，读回的交点必须落在 E17 区间 × 那个等待里 | 不中（推的） | 修（推的） | 与 X1 合起来漏 1/5（量过，`c58.out` 原样见下） | 不中（推的） | 不中（量过，同下） |
| X2b | X2 再加「一次 checkpoint 装两个改动」那条自证 | 不中（推的） | 修（推的） | 0/5（量过，`c58.out` 原样见下）；今天造不出（攒批没落地，H5） | 不中（推的） | 攒批落地之后修（量过：模型里那条抓到；今天造不出） |
| X3 | 换判据：提交路径上直接量这一次发布的 CPU 时长与这一次设备同步的时长，前者 ≥ 后者即判红，不再用「条目数 × 合并速率」折算 | 修（推的：分配器整拷算在 CPU 时长里） | 修（推的：内存盘同步时长 0 时要另定，否则同样总响，要和 X1 的下限一起用） | 要重写自证：注入已知 CPU 延迟与同步延迟（推的） | 修（推的：不往系统配置写测出来的值） | 修（推的：不需要 write buffer 条目这个对象） |
| X4 | 交点仍按 C58 算，但量好的值作 mkfs 参数由调用方交进来 | 不中（推的） | 调用方可以照样交 0（推的） | 不中（推的） | 修（推的：同参数两次逐字节相同） | 不中（推的） |

X2、X2b 那两格的原样（`c58.out`）：

```
E name=selfproof_combo comparator=gt combo=two_sided_latency_plus_source tests_green_on_correct=2/2 mutants=5 missed=1 missed_list=counter_at_session_per_user_change
E name=selfproof_combo comparator=gt combo=two_sided_latency_plus_source_plus_batching tests_green_on_correct=3/3 mutants=5 missed=0 missed_list=-
```

- 这几个改法的「量过」都只在 `c58_alarm_model.py` 的五个坏实现上量过；坏实现是我挑的，挑漏的坏法量不出来。
- X3 的代价没量：提交路径每次发布要多两次计时；测试里的 CPU 时长不稳，自证要靠注入。它改的是 C58 的判据，按 evidence-discipline 是新一轮的事，不能当这一轮的定案。
- Q8 三个候选（mkfs 实测 / 运行时配置 / 编译期常量）对 H1、H3、H5 都不起作用，这三格的病根在三者共用的前提里（打中不分辨臂）。所以 C8 要「判到实现员照着写得出来」，光在三个来源里挑一个不够，至少要先定计数对象（H5）和自证的写法（H3）。

## 五、C10（Q10 读写并发）：没打中

试过的形状，都是推理，没建模型，取样 0：

- 读者持有的版本被写者回收：Q10 的翻面观测自己就列了（C214 的窗口），不是新东西。回收时点今天由环窗口管（副本在它的根离开根环之后才回收），读者持有的版本可能比根环更老，这要读者固定版本的机制，但 Q10 的候选已经把它算进「读者读到一致的版本」的代价里。
- 和 D8 已定项 16 第 6 条「每次用户改动先走空」撞不撞：读者不做用户改动，不撞。
- 和 D13 已定项 2 撞不撞：那一项管的是提交的次序，读者不提交，不撞。
- 和 Q1 有没有依赖：「串行提交下先做」这一支本来就不依赖 Q1 的选择，B5 表里写成「不依赖」。

所以 C10 判「转」还是「放进里程碑三」，我拿不出反例。按 three-way-inference「没打中」一次不算，要再抽一次样才能记成没打中。

## 没打中的形状

| 试过的形状 | 取样范围 | 结果 |
|---|---|---|
| C3：在「jsn 序 = txg 序」的追加纪律下造一条越过没施加的 txg 的历史 | 推理，A4；E169 第二段三条候选臂的判决行 | 造不出：断号先停，只在对照丁上出现 |
| C10：见第五节四个形状 | 推理，取样 0 | 没打中 |
| Q2 后一分句「fsync 等待随并发数无界增长」 | 没建模型；E169 第三段只读了页面 | 没攻，不算没打中 |
| C8 比较符号以外的坏实现（交点单位换算错一千倍、计数器溢出回绕） | 没建；五个坏实现之外的坏法不在模型里 | 没攻 |
| H3 的扫描：自证负载的基线条目数、屏障等待、比较符号 | base 1..40000（4 万档）× 等待 6 档 × 符号 2 种 × 自证 3 种 | 打中，见 H3；这张表只记扫描范围 |

## 这条腿自己的限度

- 我是接手的腿。前一条腿被编辑器重启停掉，没交回。第一到第三节的 A1–A5、B1–B7、H1–H2 是它写的。我核过：模型目录 11 个文件的 sha256 与报告开头那张表逐个对得上；`rerun.sh` 复跑五个模型逐字节相同；`cite-check.py` 在我追加之前对上 36 处引文。我没有逐条重推它的论证。我加的部分：`c58_selfproof_sweep.py` 和它的产物，`rerun.sh` 加了一行，报告开头的复跑命令、复跑输出与 sha256 表，一览表 C8 那一行，H3–H5，「H 系列补一笔」，第四、五节和这几节收尾。
- 全部模型都是独立的 Python，不引 `crates/`。打中的东西都是模型层的数或条款之间的矛盾，没有一处在 `crates/` 上实测过。
- 没调层 0 枚举（`enumerate_layer0` / `enumerate_layer0_versions`），崩溃状态数 0；前一条腿也没调（交接摘要里没有这类命令），两条腿合起来 0，离 10⁷ 的上限远。没有随机跑批。`c58_selfproof_sweep.py` 一次前台调用就跑完了，没计时。
- `fsync_probe.out` 是前一条腿在 2026-10-01 取的一次样：一台机器、一个文件系统、200 次 × 5 轮。E44 的 `sync_1` 用的不是同一个装置。
- H1 的 128 MiB（1 TiB 盘的位图量级）是推的；C567 写的是审阅员推的 190 MB，两个数都没在 `crates/` 上量过。
- C58 的五个坏实现是我们自己挑的，「漏 k/5」只对这五个成立。
- 这一轮这条腿只抽了一次样。C10 与上表「没攻」的几行不能当作没打中。

## 没做什么

- 没跑 checker 档，没跑门禁，没编译 Rust，没跑 cargo。
- 没读禁读清单里的文件（`research/prompts/m3-conc-r1-sonnet-output.md`、`research/prompts/m3-conc-r1-sonnet-model/`、`/tmp/claude-1000/m3-handover/conc-sonnet.md`、这一轮正推腿的提示）。
- 不判正推腿的那几格，不替主 agent 采纳。上面的数全是我在自己的模型目录里跑出来的，不算入库装置上的数。
- 没在入库的装置上复现 H2（内存盘屏障空）对一个真报警实现的影响，因为报警今天没有实现。
- 草稿目录 `/tmp/claude-1000/m3-conc-r1-opus/` 里只有几个模型的输出和三次复跑的副本（`rerun`、`rerun2`、`rerun3`），没有编译目录，也没有仓副本。模型目录里的 `.out` 就是入库的产物，草稿目录里的是同一份内容，留着给主 agent 比对。
- 交回前跑 `cite-check.py` 时，6 处 D13、D23 的行号已经对不上：两份 kb 在工作区里被别的会话改过（`git diff --stat` 显示 D13 +14 −6、D23 +2 −1，改的是 M1–M4 与 E165 那几段，与这一轮无关）。被引的句子一字没变，只是挪了行，我按 cite-check 的提示改了行号（D13 45 → 51，D23 397 → 398、400 → 401）。冻结的背景材料里还是旧行号。
