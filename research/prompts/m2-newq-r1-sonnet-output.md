# m2-newq-r1 云端正推（Sonnet）报告

分工：N4、N5、N6、N7 逐格判「已定分项推得出吗」；N1、N2、N3 各一句「已定分项里有没有推得出的」。
方法：先找到题面对应的已定分项原文，逐句核对是否已经把这道题的答案写死；写不出「原文 → 答案」这一步的，判「推不出」并引用 kb 自己承认空白的那句话。全部引文已用 `grep -nF` 在对应文件里核过一次，行号是 kb / 代码文件自己的行号，不是背景材料里的行号。

## N1（读盘失败归哪个错误成员 / 发布照不照成 / 隔不隔离）

**核实**：
```
$ grep -n "读盘本身失败归哪个错误成员" .claude/kb/decisions/19-块指针的结构与宽度预算.md
105:...硬规则 1 的读盘核只定到「核、对不上就隔离」：读盘本身失败归哪个错误成员、从盘上重建上一版的路径（`rebuild_version`）与发布路径（`placements_to_release_via_mapping`）要不要同一条判据、隔离的槽跨不跨重挂与进不进准入式子，没有条款（C394（释放判定不核映射条目位置项里的单元校验和））。
```

**判定：推不出，条款空白。** D19（块指针的结构与宽度预算）已定项 5（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:87-115`）的硬规则 1 只定了「核、对不上就隔离」这一支（校验和不对的那一支），第 105 行原文逐字承认「读盘本身失败归哪个错误成员……没有条款」——这正是 N1 问的那件事本身，条款自己承认没写。没有第二个已定分项覆盖读盘失败（设备报错）这条支路：D19 已定项 2 / 已定项 4 只定校验和字段的存在与位宽，不定谁在何时核、核不到时怎么归类。

**什么现象会推翻它**：以后的决策文件里，D19 已定项 5 或新开的已定项，把"读盘本身失败"显式归进某个错误成员、并写出"发布照不照成""隔不隔离"，那时候这一格改判「推得出」。

## N2（rebuild_version 与 placements_to_release_via_mapping 要不要同一条判据）

**核实**：同一处（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:105`），原文同一句里紧接着写「从盘上重建上一版的路径（`rebuild_version`）与发布路径（`placements_to_release_via_mapping`）要不要同一条判据……没有条款」。

**判定：推不出，条款空白。** 已定项 8（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:160-183`）只定了中央映射装哪几类单元、指针携带 key 的哪几段，没有涉及"两条读路径要不要共用判据"这个问题；已定项 5 本身把这句列为未决。

**什么现象会推翻它**：新写一条已定项，明确要求 `rebuild_version` 与 `placements_to_release_via_mapping` 走同一个校验函数（或明确允许它们各自独立判），那时候改判。

## N3（隔离的槽跨不跨重挂、进不进准入式子）

**核实**：同一处（`.claude/kb/decisions/19-块指针的结构与宽度预算.md:105`），原文同一句末尾「隔离的槽跨不跨重挂与进不进准入式子，没有条款（C394）」。

**判定：推不出，条款空白。** 这里要小心一个容易混淆的地方：D28（挂载期承诺量）已定项 1 第九项「被抛弃根独占量」（`.claude/kb/decisions/28-挂载期承诺量.md:14-53`）与 D23（journal 的角色与格式）已定项 14 的「显式例外：管理员回退」段落（`.claude/kb/decisions/23-journal的角色与格式.md:342-406`）已经定义了**另一类**槽的隔离——「只被被抛弃根引用的槽」（回退／根环轮转产生的影子账），并且这一类**已经**进了准入式子第九项。但 N3 问的是 D19 已定项 5 硬规则 1 那个**新的**隔离来源（位置项校验和核不上而隔离的槽），这是另一件事：`.claude/kb/checks-owed.md:281`（C318）与 `.claude/kb/decisions/28-挂载期承诺量.md:14-53` 的依据表只提到"被抛弃根独占量"这一种隔离，没有提到"校验和核不上"这一种。两种隔离来源不能因为名字都叫"隔离"就互相替代着回答。

## N4（`policy_mismatches` 恒 0：删、改口径、还是留着）

**核实链**（三处原文，逐条 `grep -nF` 核过）：

```
$ grep -n "第一版盘不等大时" .claude/kb/decisions/03-空间分配.md
768:第一版盘不等大时，各盘落点答案必须相同；小盘没有全空段而大盘有时，在动任何状态之前拒绝（发布层统一报假性 ENOSPC）。**写明这是第一版的限制、不是永久判据**，以后放开时要答的是 D2（RAID 条带策略） 已定项 2 那一半（小盘写满之后设备集合怎么选）与已定项 8 待办 ①（w ≥ 2 时各盘聚簇段要不要对齐）。
```

```
$ grep -n "^| C515 " .claude/kb/checks-owed.md
456:| C515 | 政策不一致计数恒 0，口径没改 | 2026-09-23 判决里判盘不等大落点那一格查明它恒 0 是结构性的——该量的是「实际记录的落点」与「单块盘政策函数独立求值的答案」之差，而两者之间插着一道「不等就拒绝」的闸、永不分离；而那一格这一轮定的是维持拒绝，所以这个计数在第一版**不可能大于 0** | 要么把它删掉（别留一个恒 0 的计数冒充证据），要么等盘不等大落点那一格放开时换成真能大于 0 的口径；判别力自证按选定的那条写 | 无 | 2026-09-23 三方判决 `m2-placement-falsepositive-r1-main-verification.md` 里判盘不等大落点那一格（附问）；用户当天定案第一版维持拒绝 |
```

```
$ grep -n "policy_mismatches" .claude/kb/verification-build.md
76:| 用户数据的落点与政策函数不一致的次数 | 政策函数 = 已选设备内槽号最小、过准入合取、不在开放段与 `R` 内（D3（空间分配） 已定项 8 第 1 条 + D26（后台整理与放置回收） 已定项 1 第 3 条），模型对拍时逐次分配比对（C146（无空段时的回落政策全仓无定义） ②） | 非 0 判红 | 同上 `policy_mismatches`；2026-09-17 之后比较那一步被删掉（发出去的落点就是各盘一致的答案），这个计数恒 0、口径要改。 |
```

**推导链**：
1. D3（空间分配）已定项 8（`.claude/kb/decisions/03-空间分配.md:154-178`）与它的射程句（第 768 行）是已定分项：**第一版维持"各盘落点答案必须相同、不同就拒绝"**，且明写这是第一版限制、日后放开要等 D2 已定项 2 与 D3 已定项 8 待办 ① 重开——**这两件都不在这一轮的射程里**（正文分工表没有点这两项，第 27 行"实现今天的样子"也没提这两件已经动手）。
2. 由 1 可得：只要"维持拒绝"这条已定分项在这一轮不变，`policy_mismatches`（两个独立求值的政策函数答案之差）在第一版就**结构性恒为 0**——这是决策项的直接推论，不是猜测。
3. C515 的登记行（第 456 行）把处置写成两条路："删掉"或"等盘不等大落点放开时换口径"，并且用"别留一个恒 0 的计数冒充证据"这句话**明确否定了第三个选项"留着不动"**。
4. 由 1 可知"换口径"的前置条件（D2 已定项 2、D3 已定项 8 待办 ①重开）本轮没有发生、也没有排期——`.claude/kb/milestone/02-second-txn.md:353` 原文也只把这一格判成"这一轮没改仍恒 0，另记一笔账 C515"，没有把"换口径"排进这一轮。
5. 3 减去"留着"、4 排除"换口径在本轮可执行"，剩下唯一可执行的是"删"。

**判定：推得出：删**——但要诚实标出这一步推导里哪一段不是纯粹的 D 编号已定项：第 3、4 步的依据是 `checks-owed.md` 的登记行措辞与 `.claude/kb/milestone/` 的排期记录，不是某条 D 编号已定项本身直接写的"删"字。如果主 agent 认为登记行与排期记录不算"已定分项"，这一格应退回记成**岔路**：候选①删掉、候选②留到 D2 已定项 2 / D3 已定项 8 待办 ① 重开时换口径；翻面观测是"盘不等大落点那一题哪一天被重新打开"；这在 `records/2026-09-19-里程碑二遗留收拢.md:130` 也留了一笔旁证（当时的收口表就写"主 agent 按推荐删掉那个计数"，但今天 `crates/` 里字段仍在，说明这条推荐当时没有被执行，`.claude/kb/checks-owed.md` 的 C515 也在 09-23 那天被重新记成"前置：无"——这份 09-19 的记录只当参考，不当依据，见 `evidence-discipline.md`"所有旧数据都只是参考"）。

**什么现象会推翻它**：D2（RAID 条带策略）已定项 2 或 D3（空间分配）已定项 8 待办 ① 在本里程碑内被重新打开并放开"各盘落点必须相同"这条限制，那时候"改口径"变成可执行路径，"删"就不再是唯一选项。

**N4 判「删」⇒ 删掉要同步改的每一处**（逐处已用 `grep -n` 现查行号；末尾给出一条能核全的命令）：

| # | 文件:行 | 现状 | 要删/改什么 |
|---|---|---|---|
| 1 | `crates/singlefs-core/src/allocator.rs:629` | `pub policy_mismatches: u64,` | 删字段声明 |
| 2 | `crates/singlefs-core/src/allocator.rs:655` | `policy_mismatches: 0,` | 删构造处的初始化 |
| 3 | `crates/singlefs-core/src/allocator.rs:1157` | `assert_eq!(pool.policy_mismatches, 0);` | 删这条单测断言 |
| 4 | `crates/singlefs-harness/src/scenario.rs:61` | `pub policy_mismatches: u64,` | 删字段声明 |
| 5 | `crates/singlefs-harness/src/scenario.rs:133` | `policy_mismatches: allocator.policy_mismatches,` | 删字段赋值 |
| 6 | `crates/singlefs-harness/src/bin/first_transaction_on_device.rs:780-781` | 结果行打印 `policy_mismatches={}`，`run.policy_mismatches` | 从打印格式串与参数里删掉 |
| 7 | `crates/singlefs-harness/src/bin/first_transaction_region_bytes.rs:115,118` | 同上，另一个二进制的结果行 | 同上 |
| 8 | `crates/singlefs-harness/tests/first_transaction_step_two_data_unit.rs:296` | `pool.allocator.policy_mismatches, 0,` | 删这条断言 |
| 9 | `crates/singlefs-harness/tests/first_transaction_step_five_publish.rs:1018-1027` | 元组断言 `(pool.allocator.policy_mismatches, pool.output.key_order_mismatches), (0, 0)` | 拆掉元组，只留 `key_order_mismatches` 那一半 |
| 10 | `crates/singlefs-harness/tests/second_transaction_step_one_overwrite.rs:169` | `assert_eq!(pool.allocator.policy_mismatches, 0);` | 删这条断言 |
| 11 | `crates/singlefs-harness/tests/second_transaction_supplement_two_commit_generated_fallback.rs:144` | `assert_eq!(pool.allocator.policy_mismatches, 0);` | 删这条断言 |
| 12 | `.claude/gate.d/55-qemu-first-transaction.sh:183` | `grep -aq "name=transaction policy_mismatches=0 key_order_mismatches=0 root_txg=3 back_chain=$EXPECTED_BACK_CHAIN"` | 从期望串里删掉 `policy_mismatches=0 ` 这一段，与第 6 项的打印格式同步改 |
| 13 | `.claude/kb/verification-build.md:76` | 「用户数据的落点与政策函数不一致的次数」那一行 | 整行删掉，或按门禁 39 号（`.claude/gate.d/39-field-table-sum.sh`）的口径改写成「已删，见 C515」 |
| 14 | `.claude/kb/checks-owed.md:456`（C515） | 「前置：无」 | 还账：写明哪一次改动落地删掉了它 |

**背景材料的"三份用例断言它为 0"这句现查后要订正**：正文第 25 行说"三份用例断言它为 0"，我数出来的是**四份测试文件**（`first_transaction_step_two_data_unit.rs`、`first_transaction_step_five_publish.rs`、`second_transaction_step_one_overwrite.rs`、`second_transaction_supplement_two_commit_generated_fallback.rs`）加上 `allocator.rs` 自己的单测，另外还有 `first_transaction_region_bytes.rs` 只打印不断言。命令：`grep -rln "policy_mismatches" crates/ --include="*.rs" | grep -v "^./research"`——现查命中 7 个文件（含字段声明文件本身），断言用例是 4 个外部测试 + 1 个内部单测。

**验证命令（删完之后跑，应当零命中）**：`grep -rn "policy_mismatches" crates/ .claude/gate.d/`（注意区分 `research/e7-index-bench/src/bin/e151_arrival_and_container_arms.rs` 里的 `fallback_policy_mismatches`——那是独立研究模型自己的字段，不读 `crates/`，与这次删除无关，不在删除范围内；判据见 `.claude/rules/implementation-first.md` 第 4 条"独立验证的模型不与实现共用代码"）。

## N5（读盘选出的根比内存里记的新时，被重发的在飞 checkpoint 还重不重发）

**核实**：
```
$ grep -n "切换重新读盘择根带出的三件没有条款" .claude/kb/decisions/23-journal的角色与格式.md
386:- ⚠️ **切换重新读盘择根带出的三件没有条款**，另起一题：读盘选出的根比内存里记的新时，被重发的在飞 checkpoint 还重不重发；行与 W 按哪个根写；回退那次发布里读盘选出的根与 R_old 怎么对上（C458（实例切换取内存里的根，不重新读盘））。
```
`.claude/kb/checks-owed.md:403`（C458 登记行）与冻结证据 `research/prompts/c381-r3-main-verification.md:29-34`（第三轮"新冒出的零轮形态"）、`c381-r3-main-verification.md:57-66`（交用户表第 3 行标"零轮"）三处原文一致，都把这三件列为待开的新题，不是已经用别的分项间接答过。

**判定：推不出，条款空白。** D23（journal 的角色与格式）已定项 14（`.claude/kb/decisions/23-journal的角色与格式.md:342-406`）里"实例切换"那一段（第 515 行附近）确实写了"重发在飞 checkpoint"是切换流程的固定一步，但那段文字写的是**常规情形**（切换本身因失败触发，所选根 = 读盘选出的根），**没有讨论"读盘选出的根比内存里记的新"这个更极端的子情形**——即读盘选出的根不仅仅是"取代内存里的根"，而是**领先**内存里原本以为要重发的那个 checkpoint 所对应的状态。定义没有说这时候"被重发的在飞 checkpoint"是否已经被那个更新的根隐含地完成、还要不要照常重发；D23 已定项 14 自己第 386 行原文点名这正是它没答的一件事。

⚠️ 我在读"实例切换"那段主文字时，字面上很容易顺着"所选根"这个词往下读，误以为后面几句（行、W 的写法）已经把"用哪个根"这件事统一交代清楚了。但条款自己在第 386 行明确列出这三件"没有条款"，按 `evidence-discipline.md`"只读结论句、不读它自己的射程句"的纪律，条款自己的射程声明优先于我自己顺着主文字读出的隐含读法——所以这一格仍判"推不出"，不采信我自己那个看似连贯的读法。

**什么现象会推翻它**：D23 已定项 14 或新开的已定项，明确写出"读盘选出的根领先于内存所知状态时，在飞 checkpoint 重发与否"的判据，那时候改判「推得出」。

## N6（切换写的实例表行与 W 按哪个根写：读盘选出的，还是内存里记的）

**核实**：同上 `.claude/kb/decisions/23-journal的角色与格式.md:386`，同一句里的第二件「行与 W 按哪个根写」。另 D23 已定项 14 的"实例切换"段落（`.claude/kb/decisions/23-journal的角色与格式.md:377`）与 D18（块里携带什么信息）已定项 11（`.claude/kb/decisions/18-块里携带什么信息.md:248-336`）定义了实例表**行的格式**（`kind 0/1`、字段宽度、回收规则），但没有定义"填这一行时该用哪个根的信息"这件事——它管的是"行长什么样"，不是"用谁的数据填"。

**判定：推不出，条款空白。** 与 N5 同一处原文点名"行与 W 按哪个根写"没有条款，理由同上：D23 已定项 14"实例切换"那段主文字（（`.claude/kb/decisions/23-journal的角色与格式.md:377`）写"行只落在 [这个根的实例, 新实例)"、"W 写在……切换的所选根按实例切换那一句取"）字面上像是已经统一按"所选根"（读盘）来写，但这仍是我自己在正文里挑出来的一种读法，条款作者自己在同一份已定项的射程段落里明确否定"已经答完"——按同样的纪律，判"推不出"。

**什么现象会推翻它**：同 N5，条款把这句显式定下来（无论定成"按读盘选出的根"还是"按内存里记的根"）都会推翻这一格。

## N7（回退那次发布里，读盘选出的根与管理员选的 R_old 怎么对上）

**核实**：同上 `.claude/kb/decisions/23-journal的角色与格式.md:386`，第三件「回退那次发布里读盘选出的根与 R_old 怎么对上」。D23 已定项 14"显式例外：管理员回退"段落（`.claude/kb/decisions/23-journal的角色与格式.md:369`）定义了回退流程本身（管理员带外选 R_old、写回退行、新实例代号），但那段文字是在"重新读盘择根"这条修订**之前**写定的既有流程——它没有说回退这次恢复自己的"重新读盘择根"步骤，选出来的根要不要等于、或者怎么服从管理员选定的 R_old。

**判定：推不出，条款空白。** 理由与出处同 N5/N6：D23 已定项 14 第 386 行原文点名这正是没有条款的第三件。

**什么现象会推翻它**：条款写清"回退流程里，重新读盘择根这一步要么必须选中 R_old、要么在选不中时怎么处置"，那时候改判。

## 判定一览表

| 格 | 判定 | 一句话 |
|---|---|---|
| N1 | 推不出，条款空白 | D19 已定项 5 第 105 行原文自认"读盘本身失败归哪个错误成员……没有条款" |
| N2 | 推不出，条款空白 | 同一句原文"`rebuild_version` 与 `placements_to_release_via_mapping` 要不要同一条判据……没有条款" |
| N3 | 推不出，条款空白 | 同一句原文"隔离的槽跨不跨重挂与进不进准入式子，没有条款"；注意这与 D28 已定项 1 第九项"被抛弃根独占量"不是同一种隔离，不能互相顶替回答 |
| N4 | 推得出：删（推导第 3、4 步依赖 checks-owed 登记行与里程碑排期，不是纯 D 编号已定项；主 agent 若不认可这两步，退回记岔路） | D3 已定项 8 settle 了"第一版维持拒绝"⇒计数结构性恒 0；C515 登记行否定"留着"，"改口径"的前置条件本轮未开 |
| N5 | 推不出，条款空白 | D23 已定项 14 第 386 行原文自认这是"切换重新读盘择根带出的三件没有条款"之一 |
| N6 | 推不出，条款空白 | 同上第 386 行；主文字第 377 行字面读法看似已统一按所选根写，但条款自己否定"已经答完"，按纪律不采信自己读出的隐含答案 |
| N7 | 推不出，条款空白 | 同上第 386 行；"显式例外：管理员回退"段落（第 369 行）写于"重新读盘择根"修订之前，没有讨论两者怎么对齐 |

## 没做什么

- **不判 N1–N3 的候选与故障历史**：分工表把这部分交给云端攻方（Opus），我只答"已定分项里有没有推得出的"这一句；候选枚举、造历史、跑真实副本这些不在我的分工内。
- **不判 N5–N7 的事实表**：分工表把这部分交给本地攻方（逐格填"读盘选出的根与内存里记的根"×"在飞 checkpoint 状态"的事实表）；我只答"从已定分项能不能推出答案"，不填那张表。
- **没有写岔路单** `research/prompts/m2-newq-r1-forks.md`：N4 那一格如果被主 agent 判定为"不算已定分项直接推出"而退回岔路，需要主 agent（或另一条被指定的腿）另开岔路单登记候选与代价；我的写范围只有这份报告、模型目录（本轮未产出模型，未建目录）与草稿目录（本轮未用到，未落草稿）。
- **没有跑任何 `crates/` 编译或测试**：N4 的"删掉要同步改的每一处"是静态 grep 核出来的位置清单，没有实际执行删除、没有跑 `cargo build`/`cargo test` 验证删除后编译通过——这需要真正动代码的实现员去做。
- **没有核对被禁读文件**：按派发提示的禁读清单，没有读 `m2-newq-r1-opus-output.md`、`opus-model/`、`local-attack*`、`verifier-output.md`，也没有读这一轮别的腿的提示。
- **没有跑 `.claude/gate.d/` 任何阶段**：`stage-owners.tsv` 是否登记给云端正推腿这一次没有查——这份报告只是三方论证的一条腿，不是提交前的门禁触发点。

## 报告完整性自证

```
$ wc -l research/prompts/m2-newq-r1-sonnet-output.md && sha256sum research/prompts/m2-newq-r1-sonnet-output.md
141 research/prompts/m2-newq-r1-sonnet-output.md
5ac3add6a7ceb65ee04e84ad7adac666b47a8f52d559d5c458dd2970f0390438  research/prompts/m2-newq-r1-sonnet-output.md
```
（哈希取自写完前 141 行时的一次现算；本节之后追加的字节不在这个哈希内，仅作为分段写入过程的存证。）
