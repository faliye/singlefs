# d3-fallback-richest-leaf-code-r2：判决（2026-11-13）

<!-- doc-lint:not-numbers R -->

正文 `research/prompts/_d3-fallback-richest-leaf-code-r2-body.md`，开工快照 `research/prompts/d3-fallback-richest-leaf-code-r2-snapshot/`。这一题的代码轮第二轮；第一轮判决 `research/prompts/d3-fallback-richest-leaf-code-r1-main-verification.md`。这一轮判第一轮第三节五条改法落成的那一批修补，重头是用户 2026-11-12 定的「改实现 优化算法」（用户数据落点扫描换算法，落点不许变）。

## 一、腿与核查

| 腿 | 报告 | 结论（腿的原话，当线索） |
|---|---|---|
| 云端攻方（Opus） | `research/prompts/d3-fallback-richest-leaf-code-r2-opus-output.md` | 攻击面 1「产品路径上没打中」，聚簇段起点的对齐「只靠调用约定守」判打中（轻）；攻击面 2「打中，但不改结论」；攻击面 3、4 没打中；检查单类型与名字、测试优先多线程两格打中 |
| 云端辩方（Sonnet） | `research/prompts/d3-fallback-richest-leaf-code-r2-sonnet-output.md` | 第一轮没采的与只记欠账的「理由此刻在主工作区上都还成立」「没有一条该采而没采」；五条改法第 1、2、3、5 条落全、第 2 条缺一处、第 4 条漏一处同类注释；检查单成立 5 格、不成立 2 格、推不出 1 格 |

核查命令与它们判了什么：

- `sha256sum -c research/prompts/d3-fallback-richest-leaf-code-r2-snapshot/crates-sha256.txt research/prompts/d3-fallback-richest-leaf-code-r2-snapshot/kb-sha256.txt`：五份全 OK。腿跑着的时候与写判决这一刻，快照里的文件都没变，所以没有倒推快照树（`scripts/agents/snapshot-tree.py` 没跑）；腿引的行就是此刻主工作区的行。
- `python3 scripts/agents/cite-check.py research/prompts/d3-fallback-richest-leaf-code-r2-opus-output.md research/prompts/d3-fallback-richest-leaf-code-r2-sonnet-output.md --root /home/fy5090/code/singlefs --background research/prompts/_d3-fallback-richest-leaf-code-r2-background.md`：末行「✓ 核了 50 处引文、125 处裸行号，对上 50 处，没判 125 处（其中裸行号没有引文可比 125 处；逐处列在上面）」。没判的裸行号里，判决承重的那几处主 agent 逐处现读：`crates/singlefs-core/src/allocator.rs` 第 591–594 行（聚簇段起点在段边界上的断言）、第 1345 行（`.expect("64")`）、第 1403–1413 行（找全空段只走整段、排除活 `R` 那一段）、第 3073 行（`cluster_segments` 唯一一处插入，`grep -n 'cluster_segments.insert'` 只这一行）；`crates/singlefs-verification/src/model.rs` 第 4204–4217 行（上界文档）、第 8703 行一带（`Unexplained` 交 `None`、走对不上那一支）；`crates/singlefs-core/src/mounted_session.rs` 第 2135 行（固定点不收敛的结局在原样交回那一组）；`crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs` 第 1096 行（「回落到最低空槽」）。都与腿写的相符。
- 判决引的读数都由主 agent 经任务服务跑出：
  - 攻方的两组单测，主 agent 把它的副本拷到自己的草稿目录（副本相对主工作区只多 244 行测试，`diff` 核过）复跑：分叉输入一条红（任务 15188，`left: Some(SlotNumber(50208))`、`right: Some(SlotNumber(50272))`）；64 槽格子上取遍聚簇段集合与 `R` 的四条对拍绿（任务 15189，「test result: ok. 4 passed」）。
  - 这一批新写与重写的 22 行变异在主工作区真表上入库（任务 15053）：「计数：抓到 22 条、没红 0 条、无效 0 条、锚点失效 0 行（共 22 条）」。
  - 崩溃点重放都缩到展开深度 5，不算门禁通过：任务 15055 里快照下截另一个 inode 一项绿，近满池一项判红而红状态 0（`states=88 reused=1 red=0 … exhaustive=false verdict=red`，带六段没展开的段，红在缩域后不算穷举；回落政策那一轮同一项是 `states=89 reused=0 red=0 … exhaustive=false verdict=red`、六段相同）；辩方点名的两条崩溃枚举用例绿（任务 15186：整理搬迁流 34 块、新池新建文件流 12 块，红 0）。
- 核不动的一处：近满池那一项两趟的 `states` 是 89 与 88、`reused` 是 0 与 1，是同一个数的两种记法还是这条流少了一个状态，各片日志里没有逐块的行可比，没核出来；进下一批清单。

正文的一处错：第三节第 2 条写头一趟没红的变异是「挡上一侧不摘段」。两条腿都指出不是；任务 14606 的日志里没红的那一行是「用户数据落点按段取：槽对放开时不把那一段加回有槽对的段集合」。主 agent 写错了，腿照日志判的。

## 二、判决

1. **落点不变：站住。** 新旧取法答案相同靠三样对齐。单元区起点与 `R` 的起点由类型守着（辩方给了构造点与类型不变量的位置，攻方同）。第三样，聚簇段起点都在 64 槽格子上，只靠 `cluster_segments` 私有、唯一一处插入守着，新函数对传进来的集合不判；攻方造出的格子外输入让新旧取法分叉（任务 15188 红，坐实这条前提是承重的）。两条腿都没找到产品路径上能放进格子外起点的路，主 agent 现查插入点只有一处。格子上取遍组合的四条对拍绿（任务 15189），其中两条的单元区尾巴不满一段。
2. **对拍方法：打中，不改结论。** 攻方推出仓里那条落点对拍与增量维护用例同种子、同几何、同步数，没走到「一段离开集合又回来」；它的四条对拍断言了集合至少翻过一次 0。这四条收进仓里。「一段最后一对被挡上时不摘段」那条变异在落点答案上与原式同值（多留的段在段内找不到对、接着走下一段），只有集合对扫看得出；它由增量维护那条用例抓着（任务 15053 里这一行是抓到），判成落点对拍的等价变异，不是盲区。代价上界没有会红的检查：记一条欠账。
3. **集合的维护路径、新拒绝成员：没打中。** 两条腿各自逐条路径核过，结论相同。
4. **检查单打中的两格都采：** `allocator.rs` 第 1345 行产品函数里的 `.expect("64")` 没写靠哪条不变量；增量维护那条单测把定形取样与随机历史两个场景拼成一条。段号用裸 `usize` 不采：同文件早有同形的段号集合，要立类型就一并立，另记欠账，不归这一批。
5. **辩方复核第一轮：** 没采的攻击 1c 与固定点终止在这一批之后仍然够不着，主 agent 认。它点出的三处字面问题都采：C1017（准入只数槽不数对） 的字面把「装置走不到」连到「所以没采」；`model.rs` 那句上界「推得出」漏了活 `R` 整段空与单元区尾巴不满一段两个角（代码上成立，主 agent 现读过；上界在这两个角罩不罩得住是腿推的、没跑，并进 C1019（理想模型重写节点上界在隔离撑起的非空段一角推不出））；一处测试注释还写「回落到最低空槽」。
6. **带 --item 的那一格：** 实现员一条崩溃枚举用例没列，是缺项。辩方点名的两条主 agent 补跑了、绿（缩域）。

## 三、改法

并进下一批的实现员一次改，副本从主工作区建：

1. 守住聚簇段起点在格子上这条前提（腿提的，零轮）：`cluster_segments` 的插入点加一条段边界断言（`crates/singlefs-core/src/allocator.rs:591` 那一条 `assert!(offset_in_slots.is_multiple_of(CLUSTER_SEGMENT_SLOTS), …)` 的写法），`lowest_user_data_slot_outside_the_region` 的可见性收到包内。A/B 读数：任务 15223。主 agent 在探针副本里把插入点改成偏两槽（`self.cluster_segments.insert(SlotNumber(segment_start.0 + 2))`，没有这一改法的那一臂），跑分配器模块的单测：「test result: FAILED. 41 passed; 7 failed」，其中 3 条红在已有的那条断言上（「聚簇段起点在段边界上：SlotNumber(50242)」），3 条红在落点对拍上。格子外的起点真进了集合，现有检查抓得到；加断言换到的只是报错提前。照零轮改法的规矩撤回这一条：产品代码不加断言、不改可见性。换成一条变异行钉住它：把插入点改成偏两槽，点名这 7 条里的一条落点对拍（量过：任务 15223）。
2. 攻方的四条格子上对拍收进 `allocator.rs` 的测试模块（量过：任务 15189）；分叉输入那一条不收（它在格子外输入上断言两种取法相等，是按设计会红的探针，前提由第 1 条的变异行钉）。全文在 `~/.local/share/singlefs/work/d3-fallback-richest-leaf-code-r2-opus/` 的两份 `unit-test-*.rs`。
3. 第 1345 行的 `expect` 消息写清靠哪条不变量（量过：两条腿与主 agent 现读）。
4. 增量维护那条单测拆成两条，一条一个场景（量过：同上）。
5. `crates/singlefs-verification/tests/admission_checkpoint_cost_per_device_paths.rs:1096` 的注释改成现行回落政策；`crates/singlefs-verification/src/model.rs` 上界文档把两个漏掉的角写进「推不出的一角」（量过：现读）。

写回 kb（写回员）：C1017 的字面改成判决里不采的真依据；C1019 并进两个角；新登两条欠账——用户数据落点一次调用的代价上界没有会红的检查；段号与位图下标共用裸 `usize`。C1016（新政策下回落固定点终止没有证明） 补一句固定点不收敛的结局原样交回、不推抬下界。

## 交用户的

没有要用户定的岔路。告知两件：这一轮腿提的收严（第三节第 1 条）是零轮改法，A/B 量过之后撤回了产品代码那一半、只留一条变异行；崩溃点重放这一轮都缩过域，全部用例在全量域上的判定归提交时那一趟。

## 这一轮被判过的文件

- crates/singlefs-core/src/allocator.rs：判了用户数据落点按段取与逐候选取答案相同靠的三样对齐、有槽对的段集合的每条维护路径、新拒绝成员的分法、新加的 `expect` 与测试的场景划分
- crates/singlefs-core/src/transaction.rs：判了这一批只动导入、类型与一处包装，固定点循环与上界照旧
- crates/singlefs-core/src/admission.rs：判了这一批的两处注释与现行回落政策相符

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D3（空间分配） 已定项 8 | 不影响 | 2026-11-13 不受影响：这一轮判的是落点取法换实现，政策字面与落点不变；欠账的字面另由写回员改 |
| D3（空间分配） 已定项 10 | 不影响 | 2026-11-13 不受影响：聚簇段 64 槽与用户数据不进开放段照旧，新取法靠段起点在格子上这条前提，改法第 1 条给它配一条变异行 |
| D26（后台整理与放置回收） 已定项 1 | 不影响 | 2026-11-13 不受影响：活 `R` 的排除在新取法里按段起点判，`R` 恒为一整段（腿核的类型不变量），整理搬迁流缩域重放绿（任务 15186） |
