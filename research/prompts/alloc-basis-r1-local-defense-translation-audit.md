# alloc-basis-r1 本地辩方英文提示：逐句核对表（2026-09-17）

依据 `.claude/rules/three-way-inference.md`「给本地腿的提示一律用英文」一节：英文里的每一句转述写完都要与原文并排核，缺限定词就补。
下表逐项对到 kb 文件自己的行号（不从背景材料 `_alloc-basis-r1-appendix.md` 里数）。定稿即 `research/prompts/alloc-basis-r1-local-defense.md` 现在的内容，发给本地模型的就是它。

| 英文项 | 原文（文件:行） | 首稿缺的 / 放宽的 | 定稿 |
|---|---|---|---|
| F1 准入式（外层）九项与两条逐项出处 | `.claude/kb/decisions/28-挂载期承诺量.md:21`（式子）、`:40`（已分配出处）、`:42`（defer 待释放出处） | 无 | 与原文同范围；不可回收、挂载期承诺量、待删占用、已承诺预留、checkpoint 保留池四项只列名不译出处（不需要） |
| F2 D5 已定项4 式子逐项对照表 + 第2项独立维护警告 | `.claude/kb/decisions/05-快照-空间记账机制.md:345`（已分配→第1项）、`:347`（defer待释放→第5项）、`:398-401`（第2项独立维护、I-5.2恒真式警告） | 无 | 同 |
| F3 两道闸串联 | `.claude/kb/decisions/03-空间分配.md:430`（已定项12定案原文） | 无 | 同 |
| F4 D3 已定项9 两条硬要求 | `.claude/kb/decisions/03-空间分配.md:363`（第1条）、`:365-368`（第2条，界3次发布、空发布不算）；B=4+2k_tol 见 `.claude/kb/decisions/16-发布语义.md:377` | 首稿把「3次发布」的界与「B=8次准入尝试」两个数混写成一句，改稿分成两句并各自标出处 | 分开写：requirement 2 的界=3（D3已定项9）；另一句 admission 内部最多推 B=8 次（D16已定项1），显式加「separately」 |
| F5 D16 已定项1 表格（可再分配、环里最旧有效根、F、抬F上限、生效、回退候选集、准入、df、环深下限） | `.claude/kb/decisions/16-发布语义.md:371`（可再分配）、`:372`（环里最旧有效根）、`:373`（F）、`:374`（抬F上限）、`:375`（生效）、`:376`（回退候选集）、`:377`（准入）、`:378`（df）、`:379`（环深下限） | 无 | 与原文同范围；「df_inner 计入 defer 待释放但只扣保留池/残留/滞后量，不同于外层式子」是我自己从 F1/F5 两式对比推出的一句注，不是原文逐字，标为「Note」以区分于逐字引用 |
| F6 根环几何 R=3、S=8、深度24 | `.claude/kb/decisions/22-单元原子性怎么合成.md:255`（R=3）、`:258`（S下界4）；具体 S=8、深度24 取自 `.claude/kb/layout/01-first-txn.md:153`（根环参数一览：R=3、S=8） | 首稿引用体材料的「前提十二」（`_alloc-basis-r1-body.md:26`）作为 S=8 的出处；改稿直接查到 layout/01-first-txn.md 的权威行，不经背景材料转述 | 引用改为 `layout/01-first-txn.md:153`，不再引背景材料的转述行 |
| F7 实例表有效性谓词、回退行 | `.claude/kb/decisions/23-journal的角色与格式.md:1209`（"(i,T)可选⟺实例表无i的行，或有行(i,Ti,Wi)且T≤Ti"；回退行(r_old,T_old,0)） | 无 | 同 |
| F8 三种"有效根"读法 (i)(ii)(iii) | 读法(i)：`crates/singlefs-checker/src/image.rs:208-221`（`valid_roots` 无实例表过滤、无F过滤）；读法(ii)：`.claude/kb/decisions/16-发布语义.md:372`（"环里最旧有效根"定义里的"按实例表判仍然有效"）；读法(iii)：`.claude/kb/decisions/16-发布语义.md:376`（回退候选集） | 无 | 同 |
| F9 回退机制、影子账、被抛弃根独占量 | `.claude/kb/decisions/23-journal的角色与格式.md:1209`（"defer队列...全部记账统计量的现行值从R_old那棵账重新载入"；"分配器多查环里每一个可读根的账、只隔离其中只被被抛弃根引用的槽"；窄读法）；被抛弃根独占量的算法：`.claude/kb/decisions/28-挂载期承诺量.md:30`（上界=被抛弃根的已分配统计量−R_old的已分配统计量，不遍历树，轮转覆写时清零） | 无 | 同 |
| F10 I-3.1 定义与读法甲、决策时点 | `.claude/kb/invariants.md:120`（I-3.1 定义与 checker 读法⚠️注：按根环里全部有效根的引用取并集）；决策时点上下文（"当时池里只有第一个事务的一次发布：没有释放，根环没有被覆写，F恒为0"）：`_alloc-basis-r1-body.md:9`（这是主 agent 自己写的框架句，不是 kb 原文，但与 `records/2026-09-13-总审核.md:443-457` 描述的场景一致） | 无 | 同；决策时点那句标注来源是本轮正文的框架陈述，与 records 十一·七一致 |
| F11 I-5.2 定义 | `.claude/kb/invariants.md:167`（空闲统计对得上：空闲统计==总空间−已分配空间；纯算术、不涉及 walk） | 无 | 同 |
| F12 I-5.3 定义 | `.claude/kb/invariants.md:168` | 无 | 同 |
| F13 今天代码的事实（allocator/transaction/image/walk） | `crates/singlefs-core/src/allocator.rs:107`（占着的槽数注释）、`:165`（`mark_released` 只加 deferred_slots）、`:196`（`mark_allocated` 加 allocated_slots 减 free_slots，断言未分配）；无回收路径：全仓 grep `reclaim\|reallocat\|可再分配\|oldest\|最旧` 只命中 `allocator.rs:20、352` 两处注释（现查确认，2026-09-17）；`crates/singlefs-core/src/transaction.rs:1248`（item1 写自 allocated_slots）、`:1253`（item2 独立维护注释）、`:1259`（item5 写自 deferred_slots）；`crates/singlefs-checker/src/image.rs:208`（`valid_roots` 定义）；`crates/singlefs-checker/src/walk.rs:194`（`walk_root` 签名，newest 先走再走其余全部、累进同一个 references 表）、`:819`（I-3.1 判定 allocated==walked）、`:826`（I-5.2 判定 free+allocated==capacity） | 无 | 同；全部行号 2026-09-17 现查确认，不从背景材料数 |
| F14 里程碑步2 发布B后的数（23/10槽） | `.claude/kb/milestone/02-second-txn.md:111`（"已分配=占着的槽（mkfs 3 + A 10 + B 10 = 23槽×16384）...因为根环里A的根还引用它们、checker的I-3.1读法甲按全部有效根的引用取并集；defer待释放=10槽×16384"） | 无 | 同 |
| F15 D28 已定项2 在飞合成 | `.claude/kb/decisions/28-挂载期承诺量.md:64-65`（"准入读数=已发布统计量−在飞已批准的预留；窗口内产生的释放不发信用"） | 无（此项为主 agent 新增，用于支撑方向 D1，不在原正文四问的前提表里，但取自同一份决策文件，与 D28 已定项1/已定项2 同源） | 同 |
| Q1 双扣 | `_alloc-basis-r1-body.md:45`（问一，双扣） | 无 | 数字改用 F13/F14 的具体 23/10，未改变论证结构 |
| Q2 两道闸与两个 df | `_alloc-basis-r1-body.md:47`（问二） | 无 | 同 |
| Q3 覆写那一格的时差 | `_alloc-basis-r1-body.md:49`（问三，含 t、t-23、t-24 的推导） | 无 | 同；t≥25、R×S=24 与原文一致 |
| Q4 三种"有效根"读法 + 4a/4b | `_alloc-basis-r1-body.md:51`（问四） | 首稿把问四拆成两个跟进问题 4a/4b 时补了一步"reading (iii) 是否结构性地避开"的推导——这一步是主 agent 自己的分析（基于 F5、F7 逐字重新推导），不是正文原句，标注为可选的比较问，不作为既定事实 | 明确写成"Compare explicitly"，不假冒为已有结论 |
| D1/D2/D3 三个举例方向 | 主 agent 派发提示原文（对话消息，非文件）："D28已定项1defer待释放理由是否只指飞行窗口内的释放…""D3已定项12的串联闸是不是本来就想让defer在第一道闸上不算可用…""读法甲的全部有效根在条款里有没有别的定义…" | 无 | 逐条译出，未增未减，标注"not exhaustive" |
