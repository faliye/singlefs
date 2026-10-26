# m3-close-c506-r1 判决（推论轮第一轮，2026-10-22）

被判对象：用户 2026-10-22 定案「把 D22（单元原子性怎么合成） 已定项 2 字段表『每区槽数 S』那一格的『上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算』改成『上界 16 是格式常量』」。正文 `research/prompts/_m3-close-c506-r1-body.md`，背景材料 `research/prompts/_m3-close-c506-r1-background.md`。腿：正推（Sonnet）`research/prompts/m3-close-c506-r1-sonnet-output.md`（7f783e94…），攻方（Opus）`research/prompts/m3-close-c506-r1-opus-output.md`（645e33fb…）。

第一轮的打中先挂起，不写进正文；改过的形态交第二轮攻。

## 一、核查

1. 开工快照 `research/prompts/m3-close-c506-r1-snapshot/kb-sha256.txt` 12 行，`sha256sum -c` 全部 OK：材料点名的 kb 文件在腿跑着的时候没被改过。
2. `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-c506-r1-background.md`：核了 148 处引文、116 处裸行号，7 处对不上，116 处裸行号没有引文可比（没判）。7 处都是行号漂移，原文在相邻一行找得到：攻方 `crates/singlefs-checker/src/image.rs:778` → 777；正推 `image.rs` 2218 / 734 / 735 / 738 → 各减 1，`.claude/gate.d/code-source-discipline.sh:484` → 487、`:496` 指到标题行。两份文件不在 kb 快照里，腿交回后被别的补丁动过；引文本身逐字在，不影响判定。
3. 裸行号由主 agent 抽查（正推报告附录逐行配了原文，主 agent 现查主工作区）：
   - D22 第 60 行（S 那一格）与第 83 行（依据段 E50 那一行）与腿抄的逐字相同。
   - E50 页第 89 行「| **首个够到 1000 块这条线的 N** | **256** | **32** |」、第 92 行「散开负载下 32 槽就够到了」、第 118 行「S 在 1..16 这个区间里怎么取都不碰任何一条边」逐字在。
   - E139 产物 `research/results/e139-tightened-floor-2026-10-05.out`（6271 行）第 5115、5259 行与攻方抄的两行逐字相同。
   - `crates/singlefs-format/src/lib.rs` 的 placeholder 注释与 `ROOT_RING_SLOTS_PER_REGION_MAXIMUM = 16` 在；`crates/singlefs-core/src/root_record.rs:93` 的 `expect` 消息写「小于环长 3S ≤ 48」。
   - 读者两侧不判区域相叠：`grep -n 'overlap\|相叠\|重叠\|REGION_STRIDE'` 在 `crates/singlefs-checker/src/image.rs`、`crates/singlefs-core/src/recovery.rs`、`crates/singlefs-core/src/system_configuration.rs` 里只命中 `recovery.rs:1344` 的说明句（「mkfs 判固定结构两两不重叠」）与三处讲日志记录跨度的无关行，没有一处判。
4. 两条腿的模型都在草稿目录的副本里复跑（第五节），输出与腿报告抄的逐字相同、sha256 相同。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| V1 原条款防什么 | 防的是空间可用性（环窗口 (3S+1)W、被抛弃根隔离、洞里多压一圈），不是正确性；原条款自己算不出（循环） | 同向：平时抬 F 把候选集收到 4 个状态、与 S 无关；原条款只在「F 抬不动」那几类历史上有意义 | 两腿一致，采。「改成常量丢掉原条款的保护」只在原条款本来就兑现不了的意义上成立 |
| V1 依据段 E50 那一行 | 冲突：E50 散开档首个够线的 N = 32，R = 3 折 S ≈ 10.7，在 4..16 之内 | 打中：同一冲突，另算 S = 10 散开档 1020 块、S = 16 是 1632 块 | 两腿一致，主 agent 现查 E50 页第 89、118 行与 D22 第 83 行坐实。打中，挂起：D22 依据段那一行读错了量（N 当成 S），撑不住「16 在界外」 |
| V1 `disk_stuck` | 引 D16 第 62 行「没有上界」 | 引 E139 产物：S = 16 占用 0.70 八个种子全卡、S = 4 同格 0 | 产物两行坐实。不分辨「格式常量 / 挂载时现算」两臂，是 k_tol 之外的射程外性质，D16 已定项 1 射程第 62 行已写「没有上界」。不打中被判对象，不另立账 |
| V2 落层 | 第 1 层，随系统配置槽，不新增登记表行 | 同 | 一致，采 |
| V2 两条边登记 | kb 0 处；下界那一处只是 Rust 文档注释 | 同；指出正文「下界已有 format-const 标记」前提不实 | 坐实。正文那半句是主 agent 写错，只对 Rust 侧成立。两条边都要登记在 layout 一 S 那一行旁 |
| V2 钉槽 p 与 S ≤ 85 | 推出 S ≤ 85 | 推出 S ≤ 85，并指出常量那一臂反而扣住了这一处 | 一致，采。候选句要写这一条 |
| V2 D15 判定表 | 没有「放宽读者收的取值域」一行，规则没说 | 同，打中 | 坐实（D15 已定项 9 九行逐行读过）。与被判的这一句不冲突，是 D15 的缺口：另登欠账 |
| V2 D16「随之抬」 | D16 第 46 行「S 下界随之抬」与「下界 4 是格式常量」冻结后够不着 | 没提 | 坐实两句并排。候选句写「冻结前随之抬」交第二轮 |
| V3 S × 槽距 | 读代码推出读者不判区域相叠，没造镜像 | 副本上量过：S = 16、槽距 512 KiB，core 与 checker 都收，盘 0 区域 0 槽 12..15 与区域 2 槽 0..3 同一段字节，暖机挂载 24 次 24 / 24 判 I-7.17 红，四条对照臂 96 / 96 干净 | 攻方模型在主 agent 草稿副本复跑逐字相同；主 agent 的 grep 坐实读者不判。打中，挂起。入库 mkfs 写不出这一盘，写得出它的是只照 D22 已定项 16 第 3 句判盘长的写者，而格式常量正是对任何写者的承诺 |
| V3 mkfs 不分盘判相叠 | S = 8 在槽距 > 384 KiB 时 mkfs 拒 | 量过：S = 8 / io_min 512 KiB、S = 16 / 256 KiB 两格 mkfs 拒，改字节造的这两份池各挂 24 次 0 红 | 坐实（`out/probe.out` 的 `name=mkfs_geometry` 行）。与 16 无关，是 mkfs 的判法比盘上要的严；io_min = 512 KiB 是不是常见值是外部事实，未在本项目验证。另登欠账 |
| V3 GPU 根表 | `MAX_ROOTS = 48u` = 3 × 16，抬上界得先抬它 | 没提 | 坐实 `crash_judge_common.wgsl:241`。候选句交代 |
| V3 字节域、区域装进一个 chunk | 一致 | 没打中 | 正文第三问「R × S 个根槽落进一个 chunk」前提不对：区域间距是 P × chunk = 3 MiB，条款没有一个区域装进一个 chunk 的要求；两腿都纠正了 |
| V4 候选句 | 候选 A：上界 16 与下界 4 同为格式常量、写明块复用的界仍现算（已定项 25） | 逐字候选：加 S × 槽距 ≤ P × chunk 读者两侧与 mkfs 同判、S ≤ 85、环深上界进格式焊死的是管理员回退深度不是块复用的界、mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋) | 两份候选合成一份交第二轮（第三节）；都被攻过零轮 |
| V4 销账件 | 列七件：两个标记、三语改条款、placeholder 与 C506 行同撤、旧编号「已定项 1」、越界用例已在、wgsl 与 e173 `_LOCAL` 钉不住、读者缺口另登 | 列旧编号与 placeholder | 坐实 `root_ring.rs:18`、`lib.rs` 两处引「已定项 1」，S 在已定项 2 |

## 三、去向

### 第二轮攻的形态（这一轮合成，被攻过零轮）

D22 已定项 2 字段表「每区槽数 S」那一格，把「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。」换成：

> **上界 16 是格式常量**，下界 4 同为格式常量（两条边随系统配置槽落 D15（格式冻结政策） 第 1 层，登记在 layout 一「每区槽数 S」那一行旁）。上界是读者承诺收的最大值，不从 I-7.4（近 K 代块未被复用） 扣住的块数算：那个「K 代」是按 F 与实例表现算的回退候选集，给不出只由池定的数；块复用的界仍是 max(F_生效, 环里最旧有效根) 现算，不是格式常量（已定项 25）。环深上界 3 × 16 进格式，焊死的是管理员回退最多能退的发布数。上界另受两处约束：根记录钉槽 p 一字节、钉值最大 3S − 1、255 是哨兵 ⇒ S ≤ 85；GPU 判器根表容量 48 = 3 × 16，抬上界先抬它。**同一槽另判 S × 固定结构槽距 ≤ P × chunk**（区域不越过下一个区域的起点），越界与 S 越界同一个处置：读者（core、池级 checker、GPU 判器）整池拒、mkfs 拒。mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，小于 4 拒绝 mkfs；所写的值不取两条边。

依据段 E50 那一行换成：

> E50（根环槽数的上下界）：扫的是槽总数 N = R × S；R = 3 时粗粒度档（13 块/次）S ≤ 25 不过跑前定的 1000 块线，散开档（34 块/次）S ≥ 10 就过线 ⇒ 上界随负载与那条线变，没有只由池定的数；16 是读者承诺，不是 E50 量出的上界。

D16（发布语义） 已定项 1「环深下限」那一格的「S 下界随之抬」改成「冻结前随之抬」。

第二轮要实现的改动（第二轮站住之后才派）：读者三处与 mkfs 加「S × 槽距 ≤ P × chunk」判与越界用例（含 S = 16、槽距 512 KiB 这一格整池拒）；两条边的 format-const 标记；placeholder 注释与 C506 行同一次撤；`root_ring.rs`、`lib.rs` 引「已定项 1」的改成已定项 2；下界一侧补 checker 库单测与 GPU 对拍的对称用例。

### 另登欠账（不挡被判对象）

| 欠账 | 出处 |
|---|---|
| D15 已定项 9 判定表没有「字段宽度不变、读者收的取值域变宽」一行：旧读者照区间整池拒，不经 incompat 位 | 两腿 V2 |
| mkfs 按区域长 ≤ P × chunk 不分盘判固定结构相叠，比 0 / 1 / 0 归属下盘上要的严：S = 8、io_min = 512 KiB 被拒 | 攻方 V3 `name=mkfs_geometry` 行 |
| S 区间的两条边在 GPU 内核三处是 `4u` / `16u` 字面量、`e173` 装置用 `_LOCAL` 名字，format-const 钉不住 | 正推 V4 第 ⑥ 件 |

### 记下、不另立

- E139 `disk_stuck` S = 16 卡死：D16 已定项 1 射程第 62 行已写「一块盘持续写不进根时扣住的单元随发布线性涨、没有上界」，是射程外性质，不分辨被判的两臂。
- 正推的环窗口表（1 GiB 盘 S = 16 占 13.46%）是 `admission.rs` 的 Python 复写，不是实现读数；只作线索。

## 四、多轮小结

第一轮。打中三处挂起：依据段 E50 读错量、读者不判 S × 槽距（副本实测 I-7.17 红）、两条边在 kb 没有登记。第二轮攻第三节合成的形态，分工照 `.claude/rules/three-way-inference.md`：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方。

## 五、复跑

- 攻方模型：拷到 `/tmp/claude-1000/-home-fy5090-code-singlefs/59734d6f-e329-44d9-9b92-c8744e8de738/scratchpad/c506-verify/model`，`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c506-r1-opus/tree --work <副本>/../rerun --model <副本>`，退出码 0，末行 `name=rerun_compare same=1 lines=159 sha256=902fb7ad15ce07bc1ba00d3db1664a4e39df45b1fe9d5acd1dd02c4a1f793e24`；`rerun/probe.out` 与腿的 `out/probe.out` sha256 都是 902fb7ad…。
- 正推模型：拷到同一草稿目录的 `sonnet/model`，`bash scripts/hosts/run-with-memory-cap.sh 2G bash <副本>/tools/rerun.sh --repo . --frozen research/prompts/m3-close-c506-r1-snapshot --work <草稿>`，末行 `name=rerun_compare result=identical lines=55`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 2 | 支撑 | 2026-10-22 不受影响：第一轮打中挂起，S 那一格与依据段的改写交第二轮 |
| D16（发布语义） 已定项 1 | 支撑 | 2026-10-22 不受影响：「随之抬」改「冻结前随之抬」交第二轮 |
| D15（格式冻结政策） 已定项 9 | 备料 | 2026-10-22 不受影响：判定表缺一行另登欠账，不改判定表 |
