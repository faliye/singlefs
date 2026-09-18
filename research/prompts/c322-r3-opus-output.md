# C322（取号那一步的屏障怎么放没有条款） 第三轮 · Opus 攻方腿报告（2026-09-14）

- 立场：攻候选包 P（背景材料第二节五句，第 20–24 行）。主攻 T1（取号 I/O 错、重试用尽、回卷那条路）与 T2（多次挂载下的藏号），辅攻 T3（D23（journal 的角色与格式） 已定项 14 改写句）与 T4（① ② 在 checker 上）。
- 输入：提示 `research/prompts/c322-r3-opus.md`；背景材料 `research/prompts/_c322-r3-background.md`；第二轮 `research/prompts/c322-r2-main-verification.md`（判决在第四节）、`research/prompts/c322-r2-opus-output.md`、`research/prompts/c322-r2-opus-model/`（只读；这一轮两个模型都是新写的，没有拷第二轮的来改）。
- 模型在 `research/prompts/c322-r3-opus-model/`，构建与运行都在 `/tmp/claude-1000/-home-fy5090-code-singlefs/72d37caa-c52c-4f5d-81fd-138d4e9f00d1/scratchpad/c322-r3-opus-build/`：
  - 模型 L `layer0_q7/`（`src/main.rs` 580 行）：依赖仓里四个 crate（只读、不改），在内存盘上跑 mkfs → 取号 → 暖机 → 第一个事务（与 `crates/singlefs-harness/tests/common/mod.rs` 同一串调用），再以实例 2 借 `publish_first_file` 发一次替身，只取它第一道屏障之前的 16 个单元写。做三件事：G2 审计（那条流里每次超级块槽写的世代号对不对得上 G2）；层 0 全部崩溃状态逐个判今天 checker 的 I-7.7、①、② 的三种读法、下一次取号撞不撞；9 个点名镜像。产物 `layer0-q7.out`（20 行，跑 169 秒）。
  - 模型 M `mount_history/`（`src/main.rs` 821 行）：只用 std。从第一个事务写完起连着三次可写挂载、第四次只取号，64 条臂，穷举：取号写各报 Ok / Err；报 Err 的那一份落没落、重读看不看得见；重试写同一槽还是重读重算世代号；屏障报错并丢那块盘的易失缓存；报错之后全或无失败回卷，还是重发屏障成功就继续；回卷写的实例代号三种读法与不回卷；取号之后做到哪一步；崩溃时没过屏障的写各自落没落。每次挂载结束的盘上判 ① ② 并分三类。另有两段按 D18（块里携带什么信息） 第 879 行的写行规则与谓词算的四次挂载历史（T3）。产物 `mount-history.out`（96 行，跑 3 秒）。
- 两个模型都是确定性的（没有 I/O、随机源、并发），跑一遍与跑 N 遍给的信息一样。判别力靠对照：模型 L 的 ① 在干净镜像上成立、在「单元带 2 而取号没落」的镜像上红；模型 L 数出今天 checker 的 I-7.7 在层 0 上判红 2 个状态，与仓里层 0 测试钉的数（`crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` 第 107 行）相同、状态总数 262165 也相同，说明枚举是同一条规则。模型 M 的撞号判定在两组臂上给非 0、换成乙′ 同一组臂给 0。副本模型上的数照实报，主 agent 在入库装置上重做之前只当线索。

## 〇、结论表

| 问 | 结论 | 跑前条款 | 细节 |
|---|---|---|---|
| T1 回卷，按 P 的自然读法 | 没打中：回卷写的实例代号取「新号 − 1」或「这块盘择到的旧号」、或者不回卷，重试写同一槽或重读重算，屏障报错按取号全或无失败——这 6 条臂各三次挂载，撞号 0、危险态 0、单见证态 0 | 不触发 | 二 |
| T1 回卷，P 没写死的两处 | 打中两处，都写得出持久写集合，最少 2 个故障（两块盘的甲″ 屏障都报错并丢缓存）：① 回卷写的实例代号取「被取号写盖掉的那一槽原来的号」，198 / 85184 条历史撞号；② 屏障报错后重发屏障成功就当「完成了的屏障」继续发单元，四种回卷读法都撞（不回卷时 13256 / 110592）。换乙′ 两处都是 0；换 G1 或「择到的那一份」都不消，后者最少 1 个故障 | 字面触发反向接受条款第 1 句（换乙′ 就不触发）；另有零屏障的出路：写死回卷的号 = 新号 − 1（或删掉回卷）、写死屏障报错 = 取号全或无失败，同一个模型上全 0 | 二 |
| T2 藏号 | 没打中：同一个模型（三次挂载、恢复退一格用「根落了而下一次读不出」表示、实例切换走同一个取号过程）上 P 按自然读法 0 撞号；会藏号的只有 T1 那两处 | 不触发（T1 那两处除外） | 三 |
| T4 checker | ① 可实现，层 0 上 0 误报。② 的「各盘超级块实例代号」有三种读法、P 没写死：按每个槽读，层 0 上 6 个合法状态判红；按每盘择到的读，层 0 为 0，而模型 M 有良性态判红；按每盘全部自证过的槽里最大的读，两处都是 0。坏镜像语料里 I-7.7 那一份在 ① + ②（每盘最大）下不再红，要换一份 | ② 按每个槽读时触发；① 不触发 | 四 |
| T3 改写句 | 语义那一半没打中：崩在回退根之前，下一次挂载的所选根、施加集合与没发起回退时相同，回退那次的孤儿按 (2, 3, 0) 判未发布。字面那一半不成立：「取号写进超级块的新号是唯一留下的字节」——回退那次发布的单元与记录可以已经落盘（D16（发布语义） 已定项 7 的次序），不生效，但在盘上 | 语义不触发；字面改一句 | 五 |
| 附带（不分辨臂，与 P 无关） | 恢复退一格之后新实例复用被跳过那条根的 txg：一次暂时读不出 + 两次崩，第四次挂载选回那条根，按写行规则给第三次挂载的实例写 (3, 4, 0)，第三次挂载诞生 4 的孤儿判成已发布；普通挂载与管理员回退都有，对照（那条根读得出）判未发布 | 不分辨臂 | 五 |
| T6（顺带的数） | G2 在第一次挂载那条流上写出的世代号就是 2、3、4、5（8 次超级块写逐条对上），第一个事务的字节不变；层 0 上 I-7.7 违例 2 → 0（① + ② 按每盘最大或择到的） | — | 六 |

攻方倾向（不是判决）：甲″ 留下，P 再写死三句——回卷写的实例代号 = 取号之前全部自证过的槽里的最大值（或者删掉回卷，① ② 下它已经没有用处）；甲″ 那道屏障报错 = 取号全或无失败，不许只重发屏障就继续；② 的「各盘超级块实例代号」= 那块盘全部自证过的槽里最大的实例代号。第 5 句把「唯一留下的字节」改掉。

## 一、现查清单

| 查什么 | 命令 | 结果 |
|---|---|---|
| 背景材料附录与 kb 逐字节对 | `cmp <(sed -n 'Np' kb 文件) <(sed -n 'Mp' 背景材料)`，多行的按区间 | D18:879 = 背景 1294；D22:1005 = 913；D23:691 = 803；D23:695 = 807；D23:1206 = 818；D23:1240 = 852；D16:207 = 935；区间 D16:205–245 = 933–973、D16:285–306 = 979–1000、D16:358–416 = 1006–1064、D23:1204–1241 = 816–853，全 SAME。失败条款要对的几处都在其中 |
| D18（块里携带什么信息） 第 879 行 | `sed -n '879p' .claude/kb/decisions/18-块里携带什么信息.md \| wc -c`；`grep -n '^### 已定项 11' …` | 7920 字节，在第 775 行起的已定项 11 之下；凡是标 D18:879 的「」引文，都是用 `grep -o` 从这一行抽出的原样子串 |
| D22（单元原子性怎么合成） 已定项 16 | `grep -n '已定项 16' .claude/kb/decisions/22-单元原子性怎么合成.md` | 索引 232、正文 1003–1019，定案句 1005 |
| D23（journal 的角色与格式） 已定项 14 | `grep -n '已定项 14' .claude/kb/decisions/23-journal的角色与格式.md` | 索引 691；正文 1204–1241；显式例外 1206；「按今天的条文不成立」那条警示在 1240 |
| D23（journal 的角色与格式） 已定项 16 | 同上加 `grep -n '^#\{2,4\} ' … \| grep '已定项 1[4-9]'` | 只有索引行 695。它末尾写「正文见 D23（journal 的角色与格式）「已定项 16」」，而文件里没有这个小节（带编号的小节标题只有 14、18、19）——指路悬空，15、17 同 |
| D16（发布语义） 已定项 1 / 7 / 8 | `grep -n '^### 已定项' .claude/kb/decisions/16-发布语义.md` | 358 起 / 285 起 / 205 起 |
| C314（回退可以复用被抛弃的根引用的单元） | `grep -n '^\| C314 ' .claude/kb/checks-owed.md` | 296 |
| I-7.7（超级块实例代号不低于根环） | `grep -n '^\| I-7.7 ' .claude/kb/invariants.md` | 53 |
| 今天的取号与择超级块 | `grep -n 'SUPERBLOCK_GENERATION\|let slot_index\|fn acquire_instance' crates/singlefs-core/src/transaction.rs`；`grep -n 'pub fn choose_superblock\|if one.slot_generation' crates/singlefs-core/src/recovery.rs` | transaction.rs 49 常量 2、129 槽 = 世代号 mod 2、176–177 发布之后 txg + 2、181–187 `acquire_instance`；recovery.rs 191 起、210 世代号大的那槽、返回第一块盘择到的那一份 |
| checker 的 I-7.7 与单元扫描 | `grep -n '"I-7.7",' crates/singlefs-checker/src/walk.rs`；读 walk.rs 486–503、image.rs 133 起 | walk.rs 546 各盘择到的相等、583 都 ≥ 根环最大；扫描方向只读码 2 头（`scanned_tree_identifiers`，按 `candidate_unit_slots`）；image.rs 133 每盘择世代号大的那一槽 |
| 层 0 把 I-7.7 钉成 2 | `grep -n 'expected_violated' crates/singlefs-harness/tests/first_transaction_step_seven_layer0.rs` | 107 |
| 坏镜像语料里 I-7.7 那一份 | `grep -n '盘 1 被择的那个超级块槽里实例代号改成 0' -A 7 crates/singlefs-harness/tests/checker_known_bad_images.rs` | 373–380：盘 1 槽 1 字节 477..481 改 0，`mutate_superblock_slot`（185–196）重算槽校验和 |
| 附带那一格要的两句 | `grep -n '崩溃后该号会被重发' .claude/kb/decisions/05-快照-空间记账机制.md`；`grep -n '^\| C149 ' .claude/kb/checks-owed.md` | D5:21；checks-owed.md:153（C149 行里引 D16（发布语义） 已定项 6「每次发布把 checkpoint_txg 加一」）；整行在文末附录 |
| 附带那一格有没有登记过 | `grep -n '中间实例' .claude/kb/checks-owed.md`；`grep -n '暂时读不\|又读得出\|瞬时读\|读不出的根' .claude/kb/checks-owed.md` | 都是 0 行。C88（根环的时间线判别未实现，第 98 行）管「同 (实例代号, checkpoint_txg) 双根」，不管写行与谓词；只按这几个词查过，没排除换了说法的登记 |

## 二、T1：取号 I/O 错、重试用尽、回卷那条路

### 条款（「」里是原样子串，整行在背景材料附录）

- P 第 1 句（背景材料第 20 行）：「取号那两次超级块写之后、本实例第一个非超级块写之前至少一道完成了的屏障」「取号全或无失败时的回卷写发生在任何单元之前」。
- P 第 2 句（第 21 行）：「每一次超级块槽写（取号、发布末尾的轮换、回卷）写「这块盘上自证过的槽里最大的世代号 + 1」，槽 = 世代号 mod 2」。
- D18:879：「写超级块与数据单元写同一个取向：一次 I/O 错要重试到 T_retry 用尽才算失败；用尽后全或无判失败时**先把已经写出的那几份回卷成旧代号**，回卷不成才只读挂载——回卷期间盘上出现的「同一集合内代号不等」是 I-7.7（超级块实例代号不低于根环） 的一条显式例外，下一次挂载按 max + 1 修复」。

P 与 D18:879 都没写三件事：「旧代号」是哪个号；重试是同一份写重发，还是一次新的「超级块槽写」（按重读出的槽重算世代号）；屏障报错算什么（取号全或无失败，还是重发屏障成功就算「完成了的屏障」）。模型 M 把三件事各做成一维（`RollbackCode`、`RetryTarget`、`BarrierErrorPolicy`）穷举。「撕裂」不单列：超级块 481 字节住槽的第一个 512 扇区、补齐恒 0，按扇区原子撕开只能是整份旧或整份新，与层 0「撕裂态并进没持久」同一口径。「回卷不成转只读」就是回卷写落或没落两种结局，已在崩溃子集里。

### G2 下回卷写落在哪一槽（推理，模型核过）

取号写在每块盘上落世代号较小的那一槽。它报 Ok 之后，回卷写按实现相信的状态算世代号（取号那份 + 1），于是落在另一槽——正是取号之前那块盘上最新的一槽，带着上一个实例的号 N。两写之间没有屏障（P 只要求回卷先于任何单元）。于是崩溃时可以是「取号写没落、回卷写落了」：这块盘剩下「取号之前较旧的那一槽」加「回卷写」。回卷写带的号 ≥ N，N 还在；带的号 < N，N 在这块盘上就没了。两块盘都要这样才藏得住 N，而只有「两份取号写都报 Ok、之后又判失败」才会回卷两块盘——在两盘池里就是甲″ 那道屏障在两块盘上都报错。

### 按自然读法：没打中

回卷写取「新号 − 1」「这块盘择到的旧号」或不回卷 × 重试写同一槽或重读重算 × 屏障报错按全或无失败，`mount-history.out` 整行抄：

```text
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=NewCodeMinusOne retry=SameSlot barrier_error=FailAcquisition histories=85184 colliding_histories=0 fewest_faults_to_collide=none states=87164 dangerous=0 single_witness=0 benign=87164 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/0/0 second_by_disk_chosen=0/0/0 second_by_every_slot=15186/0/0
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=NewCodeMinusOne retry=RereadSlots barrier_error=FailAcquisition histories=438976 colliding_histories=0 fewest_faults_to_collide=none states=444828 dangerous=0 single_witness=0 benign=444828 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/0/0 second_by_disk_chosen=0/0/0 second_by_every_slot=42762/0/0
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=DiskChosenBefore retry=SameSlot barrier_error=FailAcquisition histories=85184 colliding_histories=0 fewest_faults_to_collide=none states=87164 dangerous=0 single_witness=0 benign=87164 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/0/0 second_by_disk_chosen=0/0/0 second_by_every_slot=15186/0/0
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=DiskChosenBefore retry=RereadSlots barrier_error=FailAcquisition histories=438976 colliding_histories=0 fewest_faults_to_collide=none states=444828 dangerous=0 single_witness=0 benign=444828 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/0/0 second_by_disk_chosen=0/0/0 second_by_every_slot=42762/0/0
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=NoRollback retry=SameSlot barrier_error=FailAcquisition histories=19683 colliding_histories=0 fewest_faults_to_collide=none states=20439 dangerous=0 single_witness=0 benign=20439 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/0/0 second_by_disk_chosen=0/0/0 second_by_every_slot=5532/0/0
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=NoRollback retry=RereadSlots barrier_error=FailAcquisition histories=79507 colliding_histories=0 fewest_faults_to_collide=none states=81399 dangerous=0 single_witness=0 benign=81399 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/0/0 second_by_disk_chosen=0/0/0 second_by_every_slot=13500/0/0
```

六条臂 `colliding_histories=0`、`dangerous=0`、`single_witness=0`；① 在良性态上红 0；② 按每盘最大、每盘择到的在良性态上红 0，按每个槽红 15186 / 42762 / 5532 / 13500（见「四、T4」）。重读重算那一维没打中的原因：重试写带的是新号，它盖掉取号之前最新那一槽时，要么落了（这块盘有新号，比 N 大），要么没落（N 还在）。

### 打中一：回卷写取「被取号写盖掉的那一槽原来的号」

`mount-history.out` 整行抄（最少故障的那条历史）：

```text
EXAMPLE arm=P rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=FailAcquisition fewest_faults=2
  mount=2 faults=0 acquire(code=2,collides=false) acquisition(d0s0:g6i2) acquisition(d1s0:g6i2) reached:both_ok_crash_before_barrier crash_landed_mask=0 => [d0(g4i1 g5i1) d1(g4i1 g5i1) units_or_records=[0, 1] readable_roots=[0, 1] unreadable_roots=[]]
  mount=3 faults=0 acquire(code=2,collides=false) acquisition(d0s0:g6i2) acquisition(d1s0:g6i2) reached:units_or_record_no_root => [d0(g6i2 g5i1) d1(g6i2 g5i1) units_or_records=[0, 1, 2] readable_roots=[0, 1] unreadable_roots=[]]
  mount=4 faults=2 acquire(code=3,collides=false) acquisition(d0s1:g7i3) acquisition(d1s1:g7i3) FAULT:barrier_error_both_cache_lost rollback(d0s0:g8i1) rollback(d1s0:g8i1) crash_landed_mask=11 => [d0(g8i1 g5i1) d1(g8i1 g5i1) units_or_records=[0, 1, 2] readable_roots=[0, 1] unreadable_roots=[]]
  mount=5 acquire(code=2,collides=true)
EXAMPLE arm=P rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=RetryFlushThenProceed fewest_faults=2
```

写的历史与持久写集合（盘、槽、世代号、实例代号）：

| 挂载 | 写 | 盘 | 槽 | 世代号 | 实例代号 | 结局 |
|---|---|---|---|---|---|---|
| 2 | 取号 | 0、1 | 0 | 6 | 2 | 屏障之前崩，都没落 |
| 3 | 取号 | 0、1 | 0 | 6 | 2 | 屏障完成，都落；之后带 2 的单元与记录落了，根没落（崩在根之前） |
| 4 | 取号 | 0、1 | 1 | 7 | 3 | 两盘的屏障都报错并丢了易失缓存，都没落 |
| 4 | 回卷 | 0、1 | 0 | 8 | 1（槽 1 原来是 (5, 1)） | 都落（`crash_landed_mask=11`） |
| 5 | 取号 | — | — | — | 2 | max(全部槽 1, 根环 {0, 1}) + 1 = 2，撞第 3 次挂载的单元与记录 |

末态两盘都是 {槽 0 = (8, 1), 槽 1 = (5, 1)}，盘上还有带 2 的单元与记录。模型 L 在真字节上拼了同一个末态（替身单元带 2），`layer0-q7.out` 整行抄，第二行是回卷写取 2 的对照：

```text
IMAGE name=rollback_code_is_overwritten_slot_before_1 slots=[d0s0:g8i1 d0s1:g5i1 d1s0:g8i1 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=true second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/false next_by_every_slot=2 collides=true next_by_chosen=2 collides=true
IMAGE name=rollback_code_is_new_minus_one_or_disk_chosen_2 slots=[d0s0:g8i2 d0s1:g5i1 d1s0:g8i2 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/true next_by_every_slot=3 collides=false next_by_chosen=3 collides=false
```

后果与第一轮甲的撞号同一类：第 5 次挂载写行只给 [1, 2)，实例 2 重发之后是当前实例，第 3 次挂载的孤儿落进 D18:879「i == i_now ⇒ b ≤ 挂载根的 txg；i > i_now ⇒ 判损坏」的前一支，新实例 2 发布到 txg 4 时它们（诞生 4）判成已发布。今天 checker 的 I-7.7 在这个镜像上成立（漏报），① 红。

### 打中二：屏障报错之后重发屏障成功就继续

```text
EXAMPLE arm=P rollback=NoRollback retry=SameSlot barrier_error=RetryFlushThenProceed fewest_faults=2
  mount=2 faults=0 acquire(code=2,collides=false) acquisition(d0s0:g6i2) acquisition(d1s0:g6i2) reached:both_ok_crash_before_barrier crash_landed_mask=0 => [d0(g4i1 g5i1) d1(g4i1 g5i1) units_or_records=[0, 1] readable_roots=[0, 1] unreadable_roots=[]]
  mount=3 faults=0 acquire(code=2,collides=false) acquisition(d0s0:g6i2) acquisition(d1s0:g6i2) reached:both_ok_crash_before_barrier crash_landed_mask=0 => [d0(g4i1 g5i1) d1(g4i1 g5i1) units_or_records=[0, 1] readable_roots=[0, 1] unreadable_roots=[]]
  mount=4 faults=2 acquire(code=2,collides=false) acquisition(d0s0:g6i2) acquisition(d1s0:g6i2) FAULT:barrier_error_both_cache_lost reached:flush_retried_ok_proceed reached:units_or_record_no_root => [d0(g4i1 g5i1) d1(g4i1 g5i1) units_or_records=[0, 1, 2] readable_roots=[0, 1] unreadable_roots=[]]
  mount=5 acquire(code=2,collides=true)
T3_HISTORY rollback_at_mount2=false root_(2,4)_unreadable_at_mount3=false | mount2 selected=(1,3) acquired=2 first_root=(2,4) table_of_that_root=[RowModel { instance: 1, published_txg: 3 }] crash_before_second_warm_up_root | mount3 selected=(2,4) acquired=3 rows=[RowModel { instance: 2, published_txg: 4 }] first_publish_txg=5 units(instance 3, birth 5) landed crash_before_root | mount4 selected=(2,4) acquired=4 table_after_first_publish=[RowModel { instance: 1, published_txg: 3 }, RowModel { instance: 2, published_txg: 4 }, RowModel { instance: 3, published_txg: 4 }] mounted_root=(4,5) | predicate(mount3 units)=unpublished
IMAGE name=units_of_instance_2_without_acquisition slots=[d0s0:g4i1 d0s1:g5i1 d1s0:g4i1 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=true second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/false next_by_every_slot=2 collides=true next_by_chosen=2 collides=true
IMAGE name=disk1_barrier_error_then_proceed slots=[d0s0:g6i2 d0s1:g5i1 d1s0:g4i1 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=true first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=true/true/true next_by_every_slot=3 collides=false next_by_chosen=3 collides=false
```

- 持久写集合：取号 (6, 2) 两盘都没落（屏障报错丢了缓存），重发屏障成功，之后带 2 的单元与记录落了，根没落。盘上与第一、二轮甲的撞号镜像逐字节同形（`units_of_instance_2_without_acquisition`）：① 红，今天的 I-7.7 成立。
- 只有盘 1 的屏障报错时不撞：盘 0 留着 2（`disk1_barrier_error_then_proceed`）；这时 ② 三种读法都红、今天的 I-7.7 也红——那是一个真的单见证态，红得对。
- P 为什么放得进来：第 1 句只写「至少一道完成了的屏障」，重发一次返回成功的屏障按字面就是「完成了的」；它本该让其持久的那两份写已经丢了。设备在屏障报错时丢不丢易失缓存、之后的屏障成功能不能说明之前的写在盘上，本项目没有量过，这里按「丢了、之后照样成功」建模，是推理。

### 反向接受条款：换哪一句能消掉这两处

`mount-history.out` 整行抄（两组臂各四条：P、换 G1、换择到的那一份、换乙′）：

```text
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=FailAcquisition histories=85184 colliding_histories=198 fewest_faults_to_collide=2 states=87164 dangerous=132 single_witness=4320 benign=82712 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/1434/132 second_by_disk_chosen=1422/1676/132 second_by_every_slot=13728/1434/132
ARM name=P_with_G1 rule=PoolWidePlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=FailAcquisition histories=85184 colliding_histories=186 fewest_faults_to_collide=2 states=87164 dangerous=120 single_witness=3324 benign=83720 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/1006/116 second_by_disk_chosen=966/1040/116 second_by_every_slot=13728/1006/116
ARM name=P_with_chosen_reading rule=PerDiskPlusOne reading=ChosenSlotPerDisk barrier=JiaDoublePrime rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=FailAcquisition histories=85184 colliding_histories=646 fewest_faults_to_collide=1 states=87164 dangerous=456 single_witness=4020 benign=82688 first_sentence(red_on_benign/green_on_dangerous)=0/312 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/1450/444 second_by_disk_chosen=1446/1704/456 second_by_every_slot=13752/1450/444
ARM name=P_with_yi_prime rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=YiPrime rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=FailAcquisition histories=13824 colliding_histories=0 fewest_faults_to_collide=none states=14424 dangerous=0 single_witness=650 benign=13774 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/165/0 second_by_disk_chosen=352/237/0 second_by_every_slot=4167/165/0
ARM name=P rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=NoRollback retry=SameSlot barrier_error=RetryFlushThenProceed histories=110592 colliding_histories=13256 fewest_faults_to_collide=2 states=112944 dangerous=5246 single_witness=43122 benign=64576 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/34208/3694 second_by_disk_chosen=0/34208/3694 second_by_every_slot=29443/35614/2288
ARM name=P_with_G1 rule=PoolWidePlusOne reading=EverySelfVerifiedSlot barrier=JiaDoublePrime rollback=NoRollback retry=SameSlot barrier_error=RetryFlushThenProceed histories=110592 colliding_histories=13256 fewest_faults_to_collide=2 states=112944 dangerous=5246 single_witness=43122 benign=64576 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/34208/3694 second_by_disk_chosen=0/34208/3694 second_by_every_slot=29443/35614/2288
ARM name=P_with_chosen_reading rule=PerDiskPlusOne reading=ChosenSlotPerDisk barrier=JiaDoublePrime rollback=NoRollback retry=SameSlot barrier_error=RetryFlushThenProceed histories=110592 colliding_histories=13256 fewest_faults_to_collide=2 states=112944 dangerous=5246 single_witness=43122 benign=64576 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/34208/3694 second_by_disk_chosen=0/34208/3694 second_by_every_slot=29443/35614/2288
ARM name=P_with_yi_prime rule=PerDiskPlusOne reading=EverySelfVerifiedSlot barrier=YiPrime rollback=NoRollback retry=SameSlot barrier_error=RetryFlushThenProceed histories=32768 colliding_histories=0 fewest_faults_to_collide=none states=33824 dangerous=0 single_witness=12510 benign=21314 first_sentence(red_on_benign/green_on_dangerous)=0/0 second_by_disk_highest(red_on_benign/red_on_single_witness/green_on_dangerous)=0/11270/0 second_by_disk_chosen=0/11270/0 second_by_every_slot=11270/11270/0
EXAMPLE arm=P rollback=OverwrittenSlotBefore retry=SameSlot barrier_error=FailAcquisition fewest_faults=2
EXAMPLE arm=P rollback=NoRollback retry=SameSlot barrier_error=RetryFlushThenProceed fewest_faults=2
```

| 臂 | 打中一（被盖掉那一槽的号 + 全或无）撞号 / 最少故障 | 打中二（重发屏障继续，不回卷）撞号 / 最少故障 |
|---|---|---|
| P | 198 / 2 | 13256 / 2 |
| 第 2 句换 G1 | 186 / 2 | 13256 / 2 |
| 第 3 句换「择到的那一份」 | 646 / 1（① 在 312 个危险态上漏报：取号与 checker 读法不同） | 13256 / 2 |
| 第 1 句换乙′ | 0 | 0 |

- 乙′ 为什么两处都不中：任何时刻至多一块盘上有「取号写报了 Ok 而还没过屏障」的写，而两处都要两块盘同时处在那个状态。
- 按字面，反向接受条款第 1 句触发：在 P 下写出撞号，第 1 句换成乙′ 就不触发 ⇒ 那一句换回去。
- 但两处都落在 P 没写死的读法上，另有零屏障的出路，同一个模型上已经量过：回卷写取「新号 − 1」或「择到的旧号」或者不回卷、屏障报错按全或无失败，四个维度组合全 0（「二、T1」里「按自然读法：没打中」那六行）。要写死的是两句：(a) 回卷写的实例代号 = 取号之前全部自证过的槽里最大的实例代号，或者干脆删掉回卷；(b) 甲″ 那道屏障在任一块盘上报错 ⇒ 取号全或无失败，不许只重发屏障就继续。乙′ 的代价（非首次挂载多一道屏障、首次挂载段序列变成 262164 个状态）照第二轮判决，没有重算。
- 删掉回卷还有两条理由：① ② 下它已经没用处——回卷写只改「择到的那一槽」，全部槽读法照样看得见取号那一槽的新号（模型 L `disk1_write_error_rollback_landed` 那一行 `next_by_every_slot=3`）；而且它带进一个新的只读理由（「回卷不成才只读挂载」）和一处条文空白：回卷成了之后这次挂载做什么，D18:879 没写，与 C286（只读后 remount 取不取新实例代号，checks-owed.md 第 271 行，整行在文末附录）是同一类空白。

## 三、T2：多次挂载下的藏号

- 模型 M 的历史：第一个事务之后三次可写挂载，第四次只取号（`CONFIG` 行）。「恢复退一格选较旧的根之后再发布」用「根落了而下一次挂载读不出」表示（`root_unreadable_next_mount`）：下一次取号的根环那一项看不见它，只剩超级块作见证。「轮换写撕裂」按整份旧或整份新进崩溃子集。「回卷」是「二、T1」那些分支。「实例切换」的取号与挂载时的取号是同一个过程（D23:691「切换要先取号，取号要写独占打开成功的每一份超级块、全或无」）。
- 结果：P 按自然读法的 6 条臂（每条 19683 到 444828 条历史）撞号 0、危险态 0、单见证态 0（「二、T1」按自然读法那六行）。会藏号的只有 T1 那两处。
- G2 + 全部槽读法下为什么藏不住（推理；模型 M 的 0 是它的一次检验，不是证明）：一个号 N 被单元带着，说明它那两份取号写在两块盘上都过了屏障。此后每块盘上的每次写都落在世代号较小的那一槽，带的号都 ≥ N（取号带更大的号，轮换带当前号，回卷按自然读法带「取号之前的最大值」≥ N）。世代号最大的那一槽只会被「同一段里、紧跟在让它变成最大的那一写之后」的写盖掉，这样的写只有回卷与重读重算的重试两种。所以某块盘上 N 消失，要么有一写带着 < N 的号落在那块盘取号之前最新的一槽上（T1 打中一），要么带 N 的两份取号写根本没持久却被当成过了屏障（T1 打中二）。
- 今天实现的藏号（取号恒写世代 2、发布写 txg + 2）第二轮已写（`research/prompts/c322-r2-opus-output.md` 三、五）；模型 L 在真字节上重做同一格：`today_constant_generation_2_then_units` 那一行 `next_by_chosen=2 collides=true`、`next_by_every_slot=3 collides=false`。它不是 P 的洞：实现照 G2 写，第二次取号的世代号是 6，不会被藏。

打中二那条历史（T2 要的格式；打中一那张在「二、T1」）：

| 挂载 | 写 | 盘 | 槽 | 世代号 | 实例代号 | 结局 |
|---|---|---|---|---|---|---|
| 2 | 取号 | 0、1 | 0 | 6 | 2 | 屏障之前崩，都没落 |
| 3 | 取号 | 0、1 | 0 | 6 | 2 | 屏障之前崩，都没落 |
| 4 | 取号 | 0、1 | 0 | 6 | 2 | 两盘屏障报错并丢缓存，都没落；重发屏障成功，带 2 的单元与记录落了，根没落 |
| 5 | 取号 | — | — | — | 2 | max(全部槽 1, 根环 {0, 1}) + 1 = 2，撞 |

射程：模型 M 不建 txg（T1、T2 只看实例代号），不建根环轮出、实例表、journal 重放。实例切换时上一次发布末尾的轮换写与切换的取号写同在一段（中间没有屏障），这一格没单独建；按「G2 + 全部槽读法下为什么藏不住」那一条推理，它同样只在有一写带 < N 的号时出事。第一版两盘；三块盘以上（从未打开的盘、错过取号的盘回归）不在模型里。

## 四、T4：① ② 在 checker 上

### 层 0（模型 L）

`layer0-q7.out` 整行抄：

```text
LAYER0 states=262165 red_states today_I-7.7=2 first_sentence_every_slot=0 second_sentence_by_disk_highest=0 second_sentence_by_disk_chosen=0 second_sentence_by_every_slot=6 next_acquisition_collides_by_every_slot=0 next_acquisition_collides_by_chosen=0
LAYER0_FIRST_RED check=today_I-7.7 landed_writes=[superblock_slot@disk0] slots=[d0s0:g2i1 d0s1:g1i0 d1s0:g1i0 d1s1:g1i0] carriers={0}
LAYER0_FIRST_RED check=second_sentence_by_every_slot landed_writes=[superblock_slot@disk0 superblock_slot@disk1 journal_record@disk0] slots=[d0s0:g2i1 d0s1:g1i0 d1s0:g2i1 d1s1:g1i0] carriers={0, 1}
```

| 检查 | 层 0 判红的状态数（共 262165） |
|---|---|
| 今天的 I-7.7（各盘择到的相等，且 ≥ 根环） | 2 |
| ①（全部自证过的槽） | 0 |
| ② 按每盘全部自证过的槽里最大的 | 0 |
| ② 按每盘择到的那一槽 | 0 |
| ② 按每个自证过的槽 | 6 |
| 下一次取号撞号（全部槽读法 / 择到的读法） | 0 / 0 |

- ② 按每个槽读的那 6 个状态都合法：mkfs 之后第一次挂载，取号那两份写已过屏障，暖机第一条记录或第一个根落了，而第一次轮换（世代号 3，写槽 1）还没在两块盘上都落，槽 1 还是 mkfs 的实例 0。6 = 记录那段的 2 个非空真子集 + 根那段的空子集 1 + 轮换那段的 3 个真子集（算术，与计数相符）。
- ⇒ ② 的「各盘超级块实例代号」要写死成「那块盘全部自证过的槽里最大的实例代号」。「每盘择到的」在层 0 上也是 0，但模型 M 里有良性态判红（例如打中一那组 1422 个、屏障报错继续那组 292 个），起因都是 P 没写死的读法；按每盘最大，模型 M 全部 64 条臂在良性态上判红都是 0。
- 模型 L 里 ① 怎么读的：每盘两个超级块槽（槽距 4096，fsid 相同才算）；`valid_roots` 的根；journal 环按提示逐槽 `check_journal_record`（fsid 低 8 字节过滤）；单元区按 `candidate_unit_slots` 逐槽读 512 字节头，按类标签取偏移（码 1 写序 91、fsid 83、头末 105；码 2 写序 68 + 2k、fsid 60 + 2k、头末 86 + 2k；码 3 写序 93、fsid 81、头末 107，照 D18（块里携带什么信息） 已定项 18 的偏移表），头校验和过、fsid 相同才数。
- 做得出来，代价是新读：今天 checker 的扫描方向只读码 2 头（walk.rs 486–503）；① 要加码 1、码 3 头与整条 journal 环。真镜像上没有提示，就是扫整个单元区与整条环（默认 768 MiB，一块盘 196608 个记录槽），属于 `.claude/rules/fs-design.md` 里 checker / 审计「必须遍历」那一格。层 0 全量加上这些读、再跑一遍今天的 checker，本机 release 169 秒；没有与门禁 54 号的耗时并排量过。
- 「读不全报不适用」的射程（推理，没量过）：单元区与环里任何一个读不出的扇区都让 ① 报不适用，空闲空间也算——空闲槽里可能躺着带更大号的孤儿，读不出就判不了，按字面报不适用是对的；代价是老化的盘上 ① 会经常不适用。

### 点名镜像（模型 L）

`layer0-q7.out` 整行抄：

```text
IMAGE name=clean_after_first_transaction slots=[d0s0:g4i1 d0s1:g5i1 d1s0:g4i1 d1s1:g5i1] carriers={0, 1} today_I-7.7_violated=false first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/false next_by_every_slot=2 collides=false next_by_chosen=2 collides=false
IMAGE name=units_of_instance_2_without_acquisition slots=[d0s0:g4i1 d0s1:g5i1 d1s0:g4i1 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=true second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/false next_by_every_slot=2 collides=true next_by_chosen=2 collides=true
IMAGE name=rollback_code_is_overwritten_slot_before_1 slots=[d0s0:g8i1 d0s1:g5i1 d1s0:g8i1 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=true second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/false next_by_every_slot=2 collides=true next_by_chosen=2 collides=true
IMAGE name=rollback_code_is_new_minus_one_or_disk_chosen_2 slots=[d0s0:g8i2 d0s1:g5i1 d1s0:g8i2 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/true next_by_every_slot=3 collides=false next_by_chosen=3 collides=false
IMAGE name=disk1_barrier_error_then_proceed slots=[d0s0:g6i2 d0s1:g5i1 d1s0:g4i1 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=true first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=true/true/true next_by_every_slot=3 collides=false next_by_chosen=3 collides=false
IMAGE name=disk1_write_error_rollback_landed slots=[d0s0:g6i2 d0s1:g7i1 d1s0:g4i1 d1s1:g5i1] carriers={0, 1} today_I-7.7_violated=false first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/false next_by_every_slot=3 collides=false next_by_chosen=2 collides=false
IMAGE name=known_bad_corpus_disk1_chosen_slot_instance_0 slots=[d0s0:g4i1 d0s1:g5i1 d1s0:g4i1 d1s1:g5i0] carriers={0, 1} today_I-7.7_violated=true first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/true/true next_by_every_slot=2 collides=false next_by_chosen=2 collides=false
IMAGE name=today_constant_generation_2_then_units slots=[d0s0:g2i2 d0s1:g5i1 d1s0:g2i2 d1s1:g5i1] carriers={0, 1, 2} today_I-7.7_violated=false first_sentence_violated=false second_violated(by_disk_highest/by_disk_chosen/by_every_slot)=false/false/true next_by_every_slot=3 collides=false next_by_chosen=2 collides=true
```

| 镜像 | 今天的 I-7.7 | ① | ② 每盘最大 / 择到的 / 每个槽 | 下一次取号（全部槽） |
|---|---|---|---|---|
| 干净 | 成立 | 成立 | 成立 ×3 | 2，不撞 |
| 带 2 的单元、取号没落（甲；打中二） | 成立（漏报） | 红 | 成立 ×3 | 2，撞 |
| 回卷写取 1（打中一） | 成立（漏报） | 红 | 成立 ×3 | 2，撞 |
| 回卷写取 2 | 成立 | 成立 | 成立 / 成立 / 红（误报） | 3 |
| 只有盘 1 屏障报错后继续 | 红 | 成立 | 红 ×3（单见证，红得对） | 3 |
| 盘 1 写报错、盘 0 回卷落了 | 成立 | 成立 | 成立 ×3 | 3 |
| 坏镜像语料那一份 | 红 | 成立 | 成立 / 红 / 红 | 2 |
| 今天的写法（世代 2）+ 单元带 2 | 成立 | 成立 | 成立 / 成立 / 红 | 3（按择到的是 2，撞） |

- 漏报：① 在全部撞号镜像上红；模型 M 的 P 臂上，① 在危险态上绿 0 次。例外只在取号用「择到的那一份」而 ① 用全部槽时（`P_with_chosen_reading` 那些臂 `first_sentence(red_on_benign/green_on_dangerous)=0/312` 等）——取号与 checker 必须用同一个读法，第二轮已写，这一轮在真字节上又见一次（今天的写法那一行）。
- 坏镜像语料：I-7.7 那一份（`crates/singlefs-harness/tests/checker_known_bad_images.rs` 373–380）在 ① + ②（每盘最大）下不再红。它在 P 下本来合法：盘 1 较新的那一槽带较小的号，正是回卷写取「新号 − 1」留下的形态（与 `disk1_write_error_rollback_landed` 同形）。语料要换成 ① 会红的一份，候选是「带 2 的单元、取号没落」；「每个变异都让它的目标红」那条测试跟着改。

## 五、T3：D23（journal 的角色与格式） 已定项 14 改写句

### 条款（「」里是原样子串）

- P 第 5 句（背景材料第 24 行）：「回退行、回退那次发布的单元与回退根在那次发布之前都不生效——崩在它之前，下一次挂载选的根、施加的记录、已发布谓词的判定都与没发起回退时相同；取号写进超级块的新号是唯一留下的字节，它让下一次取号跳过一个号，被跳过的号按 D18（块里携带什么信息） 已定项 11 的写行规则得一行」。
- D23:1206：「回退与它的第一个新根同一次发布，之前没有持久效果，崩了就重做」；「被抛弃时间线的根离开根环之前，它们引用的单元不许重新分配也不许抹头——分配器多查环里每一个可读根的账」。
- D16:207（已定项 8 定案句）：「新实例在本实例写成（FUA 返回）的根覆盖两块盘之前，不让任何 fsync 返回、不向管理员确认回退」。
- D18:879：「给 [所选根的实例, 新实例) 每个实例写 (i, **所选根的 checkpoint_txg**, **属于实例 i 的、被这次重放施加的最大事务号**)」；「恢复恒选最新可读且自证通过的根，它可以不是最新发布过的根（槽坏了就退一格，之后的记录接不上就一条都不施加、W = 0）」；「有行 (i, T_pub, W)：码 1 ⇒ b ≤ T_pub ∨ n ≤ W」「码 2 / 3 ⇒ b ≤ T_pub」。

### 崩在回退根之前：语义那一半成立，字面那一半不成立

`mount-history.out` 整行抄（回退到 (1, 1)，取号 2 落了，回退那次的单元与记录落了，根没落；对照是同一个盘上没发起回退）：

```text
T3_ROLLBACK_CRASHED_BEFORE_ROOT rollback_attempted=true next_mount selected=(1,3) applied_records=0 acquired=3 rows=[RowModel { instance: 1, published_txg: 3 }, RowModel { instance: 2, published_txg: 3 }] rollback_orphans(instance 2,birth 4)=unpublished bytes_left_by_the_attempt=[superblock(instance 2)@disk0 superblock(instance 2)@disk1 units(instance 2,birth 4) journal_record(instance 2,counter 4)]
T3_ROLLBACK_CRASHED_BEFORE_ROOT rollback_attempted=false next_mount selected=(1,3) applied_records=0 acquired=2 rows=[RowModel { instance: 1, published_txg: 3 }] rollback_orphans(instance 2,birth 4)=absent bytes_left_by_the_attempt=[]
```

- 下一次挂载选 (1, 3)，与没发起回退时相同；施加 0 条——所选根覆盖到计数器 3，下一条计数器 4 是回退那次的记录，实例 2 ≠ 1，按 D23（journal 的角色与格式） 已定项 14 注 1「下一条的实例代号与所选根不同即停」停；写行多一行 (2, 3, 0)；回退那次的孤儿（写序实例 2、诞生 4）按 4 ≤ 3 为假判未发布。三样与没发起回退时逐项相同，只多一个被跳过的号与一行。第二轮这一格的结论照样成立。
- 但留在盘上的不只是新号：D16（发布语义） 已定项 7 的次序是「COW 单元/节点 → 屏障 → journal 记录 → 屏障 → 根槽（FUA）」，崩在根槽之前，回退那次发布的单元（含写了回退行的实例表单元）与记录都可以已经落盘（`bytes_left_by_the_attempt` 那一列）。它们不生效，但「唯一留下的字节」按字面是假的，读的人会以为盘上找不到回退那次的任何单元。改法（攻方的读法）：「取号写进超级块的新号，以及那次发布已经落盘的单元与记录（写序与记录都带新号，谓词按被跳过的号那一行判未发布，重放停在实例边界），是它留下的全部字节；新号让下一次取号跳过一个号……」。
- 下一次挂载自己的第一条记录也是计数器 4（计数器全池接着走，D23（journal 的角色与格式） 已定项 14 注 3），落在同一个环槽；它只落了一块盘时，两块盘在计数器 4 上是两条不同实例的记录，都自证过；重放在所选根之后按实例边界停，谁都不施加（推理，没建模）。

### 重做时的候选集与 R_old、影子账在被跳过的号上

- 紧接着再发一次回退：所选根与实例表都没变，候选集（根环里按实例表判有效 ∧ txg ≥ F_生效）与第一次发起时逐项相同。新的回退实例是 3，第一个新根 txg 仍是根环最大 + 1 = 4，与第一次那批孤儿同 txg、不同实例；按「回退写 (r_old, T_old, 0) 与中间实例的 (i, 0, 0)」给 2 写 (2, 0, 0)，那批孤儿按 4 ≤ 0 为假判未发布（算术）。
- 中间隔一次普通挂载：那次写 (1, 3, W) 与 (2, 3, 0)；实例 1 的根 txg 都 ≤ 3，全在候选集里，与没发起回退时相同。
- 影子账：被跳过的号 2 没有根，影子账在它上面没有东西可管。影子账只查「环里每一个可读根」：发起回退那一刻若有一条环里的根暂时读不出，回退那次发布的单元可以落在它引用的单元上；下一次挂载那条根又读得出、被选中，读到被盖掉的单元。同一刻做普通挂载，它的发布同样可能盖掉那些单元，所以这一格不分辨「发没发起回退」，不触发 T3，记作影子账的射程（推理，没建模）。

### 附带：恢复退一格之后新实例复用被跳过那条根的 txg（不分辨臂，与 P 无关）

`mount-history.out` 整行抄（普通挂载两行、管理员回退两行；每组一行是那条根读得出的对照）：

```text
T3_HISTORY rollback_at_mount2=false root_(2,4)_unreadable_at_mount3=false | mount2 selected=(1,3) acquired=2 first_root=(2,4) table_of_that_root=[RowModel { instance: 1, published_txg: 3 }] crash_before_second_warm_up_root | mount3 selected=(2,4) acquired=3 rows=[RowModel { instance: 2, published_txg: 4 }] first_publish_txg=5 units(instance 3, birth 5) landed crash_before_root | mount4 selected=(2,4) acquired=4 table_after_first_publish=[RowModel { instance: 1, published_txg: 3 }, RowModel { instance: 2, published_txg: 4 }, RowModel { instance: 3, published_txg: 4 }] mounted_root=(4,5) | predicate(mount3 units)=unpublished
T3_HISTORY rollback_at_mount2=false root_(2,4)_unreadable_at_mount3=true | mount2 selected=(1,3) acquired=2 first_root=(2,4) table_of_that_root=[RowModel { instance: 1, published_txg: 3 }] crash_before_second_warm_up_root | mount3 selected=(1,3) acquired=3 rows=[RowModel { instance: 1, published_txg: 3 }, RowModel { instance: 2, published_txg: 3 }] first_publish_txg=4 units(instance 3, birth 4) landed crash_before_root | mount4 selected=(2,4) acquired=4 table_after_first_publish=[RowModel { instance: 1, published_txg: 3 }, RowModel { instance: 2, published_txg: 4 }, RowModel { instance: 3, published_txg: 4 }] mounted_root=(4,5) | predicate(mount3 units)=published
T3_HISTORY rollback_at_mount2=true root_(2,4)_unreadable_at_mount3=false | mount2 selected=(1,3) acquired=2 first_root=(2,4) table_of_that_root=[RowModel { instance: 1, published_txg: 1 }] crash_before_second_warm_up_root | mount3 selected=(2,4) acquired=3 rows=[RowModel { instance: 2, published_txg: 4 }] first_publish_txg=5 units(instance 3, birth 5) landed crash_before_root | mount4 selected=(2,4) acquired=4 table_after_first_publish=[RowModel { instance: 1, published_txg: 1 }, RowModel { instance: 2, published_txg: 4 }, RowModel { instance: 3, published_txg: 4 }] mounted_root=(4,5) | predicate(mount3 units)=unpublished
T3_HISTORY rollback_at_mount2=true root_(2,4)_unreadable_at_mount3=true | mount2 selected=(1,3) acquired=2 first_root=(2,4) table_of_that_root=[RowModel { instance: 1, published_txg: 1 }] crash_before_second_warm_up_root | mount3 selected=(1,3) acquired=3 rows=[RowModel { instance: 1, published_txg: 3 }, RowModel { instance: 2, published_txg: 3 }] first_publish_txg=4 units(instance 3, birth 4) landed crash_before_root | mount4 selected=(2,4) acquired=4 table_after_first_publish=[RowModel { instance: 1, published_txg: 1 }, RowModel { instance: 2, published_txg: 4 }, RowModel { instance: 3, published_txg: 4 }] mounted_root=(4,5) | predicate(mount3 units)=published
```

- 第二次挂载第一个根 (2, 4) 落了，暖机第二个根没落就崩（D16（发布语义） 已定项 8：这时没有 fsync 返回过，回退也没向管理员确认过）。
- 第三次挂载那条根暂时读不出 ⇒ 按「恢复恒选最新可读且自证通过的根」选 (1, 3)，取号 3，写行 (1, 3, 0)、(2, 3, 0)；第一次发布 txg 4（D5:21「崩溃后该号会被重发」；C149 那一行引的 D16（发布语义） 已定项 6「每次发布把 checkpoint_txg 加一」），单元（写序实例 3、诞生 4）落了，根没落就崩。
- 第四次挂载那条根又读得出 ⇒ 选 (2, 4)，取号 4，写行 [2, 4)：(2, 4, 0)、(3, 4, 0)。第三次挂载的孤儿对 (3, 4, 0)：码 2 / 3「b ≤ T_pub」= 4 ≤ 4，已发布；码 1 那一支 b ≤ T_pub 已经为真，同样已发布。它们从没被任何根引用过。
- 对照：那条根第三次挂载时读得出，第三次挂载选 (2, 4)、第一次发布 txg 5，孤儿诞生 5 > 4，未发布。管理员回退那一组同形（第二次挂载回退到 (1, 1)，它的根 (2, 4) 暂时读不出时，第三次挂载选的是被回退抛弃的 (1, 3)）。
- 病根在写行规则：它给 [所选根的实例, 新实例) 里每个中间实例都写「所选根的 txg」，隐含「中间实例的单元诞生代号都大于所选根」；中间实例若是在所选根读不出时起的，它的 txg 从更旧的根往上数，诞生代号可以 ≤ 所选根。故障数：一次暂时读不出（D18:879 自己写着「读不到不等于不存在，扇区维」）加两次崩。它不在取号那一步上，不分辨任何一条臂；按「一、现查清单」最后一行那两次 grep 没找到登记，建议立一笔欠账。修法这一轮没想清楚，不给候选；要回答的是「中间实例那一行拿什么当 T_pub，才能不依赖『中间实例都从所选根之后起』」。

## 六、T6（顺带的数；T6 归辩方腿）

`layer0-q7.out` 整行抄：

```text
G2_AUDIT device=0 offset=0 written_generation=2 written_instance=1 highest_self_verified_before=1 generation_by_G2=2 offset_by_G2=0 matches=true
G2_AUDIT device=1 offset=0 written_generation=2 written_instance=1 highest_self_verified_before=1 generation_by_G2=2 offset_by_G2=0 matches=true
G2_AUDIT device=0 offset=4096 written_generation=3 written_instance=1 highest_self_verified_before=2 generation_by_G2=3 offset_by_G2=4096 matches=true
G2_AUDIT device=1 offset=4096 written_generation=3 written_instance=1 highest_self_verified_before=2 generation_by_G2=3 offset_by_G2=4096 matches=true
G2_AUDIT device=0 offset=0 written_generation=4 written_instance=1 highest_self_verified_before=3 generation_by_G2=4 offset_by_G2=0 matches=true
G2_AUDIT device=1 offset=0 written_generation=4 written_instance=1 highest_self_verified_before=3 generation_by_G2=4 offset_by_G2=0 matches=true
G2_AUDIT device=0 offset=4096 written_generation=5 written_instance=1 highest_self_verified_before=4 generation_by_G2=5 offset_by_G2=4096 matches=true
G2_AUDIT device=1 offset=4096 written_generation=5 written_instance=1 highest_self_verified_before=4 generation_by_G2=5 offset_by_G2=4096 matches=true
```

- 第一个事务的字节：不变。G2 在那条流上写出的世代号是 2、3、4、5，落槽 0、1、0、1，与今天的实现逐条相同（8 行 `matches=true`），即 layout/01-first-txn.md 零那张写清单的 a1、w3、w6、t11。
- 层 0 钉的数：I-7.7 违例 2 → 0（① + ② 按每盘最大；按择到的也是 0，按每个槽是 6）。`first_transaction_step_seven_layer0.rs` 第 107 行那条断言要跟着改。
- 坏镜像语料：I-7.7 那一份要换（「四、T4」最后一条）。
- 代码：transaction.rs 49、176–177（世代号按每盘自证过的槽算）、181–187（取号的 max 取全部自证过的槽 ∪ 根环）；recovery.rs `choose_superblock` 只给几何；checker walk.rs 541–548、577–587 换成 ① ②，另加码 1 / 码 3 头与 journal 环的扫描。回卷若按攻方倾向删掉，D18:879 那一句与 P 第 1 句的第二条实现义务一起删。
- 段序列登记表：甲″ 在首次挂载流上与暖机开场那道屏障重合，不变（第二轮已算 262165）；非首次挂载路径的取号一行照第二轮判决的连带去补，这一轮没有新数。

## 七、按跑前写死的条款怎么判（攻方的读法）

- 失败条款：不触发。前提要对的几处（D18:879 的取号句与写行句、D22:1005、D23:695、D23:691 与 D23:1206）与背景材料附录逐字节相同（「一、现查清单」第一行）；前提四引的代码行号（transaction.rs 49、129、176–177，recovery.rs 191）现查都在。
- T1：按 P 的自然读法不触发。P 没写死的两处触发（写出了撞号的历史与持久写集合）；把第 1 句换成乙′ 两处都不中 ⇒ 反向接受条款第 1 句按字面触发。攻方的读法：两处都能用写死两句关掉，零屏障，同一个模型上量过全 0；乙′ 买到的「任何时刻至多一块盘有未过屏障的取号写」，在写死这两句之后没找到别的用处（只在模型 M 的故障集上成立：取号写 Ok / Err、屏障报错丢缓存、崩溃）。判决要在「写死两句」与「换乙′」之间选。
- T2：不触发（T1 那两处除外）。
- T4：② 按每个槽读时触发（层 0 上 6 个合法状态判红）。把第 4 句换回第一轮甲的改写消不掉它：那句也是「各超级块实例代号相等，或不等时……」，按每个槽读同样在这 6 个状态上红（推理，没跑）。要做的是把 ② 的读法写死成每盘最大。① 不触发。
- T3：语义不触发。判据表里 T3 的触发观测是「写出一个状态，它的下一次挂载所选根、施加集合或谓词判定与没发起回退不同」，这一轮没写出这样的状态，只写出一句字面上不成立的话（「唯一留下的字节」）。攻方的读法：改那半句，不必为它再攻一轮；由判决定。
- T5、T6 不归这条腿；顺带的数在「四、T4」「六、T6」。

## 八、什么现象会推翻这些结论

| 结论 | 推翻它的观测 |
|---|---|
| T1 按自然读法 0 撞号 | 模型 M 之外还有一种写：在同一段里、取号写之后又落在某块盘最新的那一槽上，带的号比取号之前的最大值小（例如恢复或切换的某一步在回卷之外改写超级块）；或三块盘以上的池里，报 Err 的那块盘不在全部槽读法取的集合里 |
| T1 两处要 2 个故障 | 两块盘共用一个出错点（同一个控制器或传输复位），一次事件就让两块盘的屏障都报错并丢缓存——那时算 1 个故障 |
| 乙′ 两处都不中 | 乙′ 下出现「两块盘同时有报了 Ok 而未过屏障的取号写」，例如实现把盘 1 的取号写提前到盘 0 的屏障之前发 |
| ② 要按每盘最大 | 一个合法状态，某块盘全部自证过的槽里最大的号比另一块盘小，而那个较大的号已被单元、记录或根带着——例如三块盘以上错过取号的盘回归（C120（分叉盘回归的判定与重同步） 那一格） |
| T3 语义那一半成立 | 下一次挂载不给被跳过的号写行，或回退那次发布的记录被重放跨过实例边界施加 |
| 附带那一格 | 条款写明新实例第一次发布的 txg 能排除「≤ 某条读不出的根的 txg」，或写行给中间实例的 T_pub 不再取所选根的 txg |

## 九、复跑与偏差

```bash
S=<临时目录>; rsync -a research/prompts/c322-r3-opus-model/ "$S/model/"
cd "$S/model/layer0_q7" && CARGO_TARGET_DIR="$S/target-l" cargo run --release -q > "$S/layer0-q7.out"        # 20 行，本机 169 秒；按绝对路径依赖仓里四个 crate
cd "$S/model/mount_history" && CARGO_TARGET_DIR="$S/target-m" cargo run --release -q > "$S/mount-history.out"  # 96 行，本机 3 秒
```

本机这一轮的产物就是这两条命令写出的：`layer0-q7.out` sha256 f5e28e53…，`mount-history.out` a80b4f11…，与存进模型目录的两份相同。模型 L 读的是仓里今天的代码，代码一变它的数就可能变。

偏差如实记：
- 一次写文件超过了 150 行：`mount_history/src/main.rs` 第一段 168 行（从文件头到 `impl Pool` 末尾）。内容没有重写，照原样留着；其余每次写都在 150 行以内。
- 报告写成之后用 `research/scripts/replace-once.py` 定点改了三处指方位的说法（「本节」「下面」「上面」各一处），每处命中 1 次、回读确认。
- 两个模型都没有变异表，证据强度靠对照：模型 L 今天的 I-7.7 层 0 计数与仓里测试钉的 2 相同，① 在点名的撞号镜像上红、在干净镜像上成立；模型 M 同一组故障下 P 臂撞号、乙′ 臂 0，自然读法的臂 0 而两处读法的臂非 0。模型 M 的 ① ② 与撞号按条文重写，没有与 checker 源码对拍；模型 L 的 ① ② 是新写的，偏移照 D18（块里携带什么信息） 已定项 18 的表，也没有与仓里任何一段代码对拍。
- 模型 M 的抽象：超级块两盘两槽，单元、记录、根只记实例代号；没有 txg（附带那一格另写了一段按条文算的历史）、根环轮出、实例表、journal 重放；实例切换时上一次发布末尾的轮换与切换的取号同段那一格没单独建。

## 附录：正文引到、背景材料附录没抄的原文（整行抄）

命令一律是 `sed -n 'Np' 文件`。

**`.claude/kb/decisions/05-快照-空间记账机制.md:21`**

```markdown
| `birth(b)` | 块被**发布**的那个 checkpoint 号 | 不是写请求发出时所在的开放 txg——崩溃后该号会被重发 |
```

**`.claude/kb/checks-owed.md:153`**

```markdown
| C149 | 根槽重发之后 birth 口径松一格 | **D23（journal 的角色与格式） 对根槽写失败的处置是「checkpoint_txg 推进一格再发」，而同句「号 ≤ W 的事务**照旧**」，对上 D5（快照 / 空间记账机制） 逐字「`birth(b)` = 块被**发布**的那个 checkpoint 号」⇒ 发生过根槽重发的镜像上，同一个块的 birth 说得出两个号。I-3.5（引用区间的精确性） 那句「**双侧夹死，off-by-one 无处可藏**」在这类镜像上正好松一格。**⚠️ 这是 D5（快照 / 空间记账机制） birth 口径的既有洞，不是 2026-09-06 四条定案造的。⚠️ 同源的另一处空白（2026-09-11 D19（块指针的结构与宽度预算） 已定项 6 三轮对抗第二轮反推腿指出）：重发推进一格那一刻，手里已经是 T+1 的开放 checkpoint 换不换号全仓没写；并进重发的那一次、或改成 T+2 都说得通，两个都用 T+1 被 D16（发布语义） 已定项 6「每次发布把 checkpoint_txg 加一」挡住；前两种读法下都有码 1 单元的诞生代号比真正发布它的号小一格，与这里同向 | 造一个「根槽写失败 → checkpoint_txg 推进一格 → 重发」的镜像，断言其中每个块的 birth 只有一个取值，且 I-3.5（引用区间的精确性） 的双侧夹死在它上面仍然是紧的。判别力自证：让重发不推进 txg，检查必须由绿转红 | D5（快照 / 空间记账机制） 的 birth 口径要补一句说清重发那一格取哪个号（细则，不动格式） | 2026-09-06 D8（核心索引结构） 已定项 8 第二轮反推腿在「反推没打中」那一节自陈「值得单独立一笔」，第三轮辩方腿点出它一直没立 |
```

**`.claude/kb/checks-owed.md:271`**

```markdown
| C286 | 只读后 remount 取不取新实例代号 | **D23（journal 的角色与格式） 已定项 14 对持续失败的处置是「转只读到下次挂载」；D18（块里携带什么信息） 已定项 11 只定了「可写挂载的顺序……再取新代号……之后才动任何单元；只读挂载不取号」与「作废一个 checkpoint 必须换实例代号」；而只读之后 remount 成可写算不算一次可写挂载、许不许，全仓没有一句**（2026-09-11 `grep -rn remount .claude/kb` 零命中）。按最弱读法（remount 不取号）：故障时没发布的 checkpoint T 被作废、固定点单元留在盘上；remount 之后内存从最后发布的根重载，下一个号仍是 T，由同一个实例重发——D18（块里携带什么信息） 已定项 11 那句的理由逐字「当前实例没有行，谓词对它只能问「诞生代号 ≤ 挂载根 txg」，重发同一个号一发布，诞生代号等于它的孤儿全部判已发布」⇒ 两批固定点单元都判已发布，扫描重建复活作废的那批。与 key 族无关，D19（块指针的结构与宽度预算） 已定项 6 的七条候选都受影响 | 两句都写出来：「转只读到下次挂载」不许中途 remount 成可写；从只读 remount 成可写算一次可写挂载、走取号流程。崩溃点重放加一条：只读 → 可写之后超级块实例代号严格变大。判别力自证：让 remount 沿用原代号，检查必须红 | 事务层 + 崩溃点重放 | 2026-09-11 D19（块指针的结构与宽度预算） 已定项 6 三轮对抗第二轮反推腿，主 agent 现查 D23（journal 的角色与格式） 已定项 14 与 D18（块里携带什么信息） 已定项 11 那一行坐实 |
```

**`crates/singlefs-harness/tests/checker_known_bad_images.rs:373-380`**

```rust
        // 盘 1 被择的那个超级块槽里实例代号改成 0。
        (
            "I-7.7",
            Box::new(|image: &mut MemoryPool| {
                mutate_superblock_slot(image, 1, 4096, |bytes| {
                    bytes[477..481].copy_from_slice(&0u32.to_le_bytes())
                })
            }),
```
