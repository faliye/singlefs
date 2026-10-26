# m3-close-c506-r2 判决（推论轮第二轮，2026-10-22）

被判对象：第一轮判决 `research/prompts/m3-close-c506-r1-main-verification.md` 第三节合成的形态（S 上界 16 与下界 4 同为格式常量、S × 槽距 ≤ P × chunk、S ≤ 85、GPU 根表 48、mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)、E50 那一行改写、D16「冻结前随之抬」）。正文 `research/prompts/_m3-close-c506-r2-body.md`。腿：正推（Sonnet）`research/prompts/m3-close-c506-r2-sonnet-output.md`（5e7c39cc…），攻方（Opus）`research/prompts/m3-close-c506-r2-opus-output.md`（635f28f9…）。

## 一、核查

1. 开工快照 `research/prompts/m3-close-c506-r2-snapshot/kb-sha256.txt` 17 行，`sha256sum -c` 全部 OK。
2. `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-c506-r2-background.md`：核了 91 处引文、35 处裸行号，引文全部对上，35 处裸行号没有引文可比（没判）。
3. 主工作区现查：
   - `crates/singlefs-checker/src/image.rs` 第 773–774 行 `let prime_step = u64::from(read_u32(slot, 363));`、`let chunk_bytes = u64::from(read_u32(slot, 367));`：池级 checker 按盘上字段放区域。
   - `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 173–174 行 `c.prime_step = rd32(base, 363u);`、`c.chunk_bytes = rd32(base, 367u);`：GPU checker 内核同样读盘上字段。
   - `crates/singlefs-core/src/system_configuration.rs` 第 593–594 行写系统配置时 `put_u32` 常量 `ROOT_RING_PRIME_STEP`、`ROOT_RING_CHUNK_BYTES`；`grep -n '363\|367\|PRIME_STEP\|CHUNK_BYTES'` 在 `recovery.rs` 与 `system_configuration.rs` 里没有一处读这两个字段：core 只写常量、不读。
   - `crates/singlefs-checker-tier/src/crash_judge_checker_common.wgsl` 第 567 行 `const C_MAX_CANDIDATES: u32 = 48u;`，与 `crash_judge_common.wgsl` 的 `MAX_ROOTS = 48u` 并存。
   - D22 第 60 行同一格前半句点名「环深」这个词不用，第一轮合成的句子后面写了「环深上界」。
4. 两份模型在草稿副本 `/tmp/claude-1000/-home-fy5090-code-singlefs/59734d6f-e329-44d9-9b92-c8744e8de738/scratchpad/c506-r2-verify/` 复跑（第五节），输出与腿报告抄的逐字相同。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| W1 判本身罩不罩得住 | 罩得住同盘区域相叠的全部形状，前提是右边取第一版常量 3 MiB；读者之间 P、chunk 的读法不一致，I-7.13 八项与 D22 第 348 行都没判这两个字段等于常量 | 打中（副本量过）：P = 0 的盘 checker 判 I-7.17 红而 core 读得对；P = 5 或 chunk = 4 MiB 的盘 checker 在 1/3 的根环上读不到根、一格不红；按盘上字段判时第一轮那一盘把 chunk 写成 4 MiB 就过这一判，25 / 25 照样红 | 两腿同指一处，主 agent 现查坐实（第一节第 3 条）。打中，挂起：第一轮形态少了「P、chunk 与第一版常量比，不等整池拒」（与 D22 已定项 16 第 1 句管根环起点、journal 环起点的那一句同形），这一判才有确定的右边 |
| W1 错拒 | 26429 个 (S, 槽距) 与 mkfs `fixed_structure_extents` 逐形状相等，错拒 0；要写 ≤，写 < 错拒 5 个首尾相接形状；S ≥ 4 时有效槽距上界 768 KiB | 29744 格两边收的集合逐格相同，罩不住的相叠 0 | 两腿一致，采（攻方一次观测，正推独立模型同向，算两份）。写 ≤ |
| W1 落点 | core `system_configuration_values_this_reader_accepts`、checker `geometry_of` 加 `value_refused_by_this_reader`、GPU `parse_system_configuration` 与 `checker_read_system_configuration_slot`（u32 写成 spacing ≤ 3145728 / S 防溢出）；mkfs 不用加，`check_geometry` 已等价；挂 I-7.13 | 没给 | 采正推的落点（推的，实现时核） |
| W2 min 公式 | 恰为 4 在槽距 629248..786432 上取得到；按 D2 已定项 19 取槽距只写 8 或 6 | 打中（量过）：同一段上写 4、mkfs 收 S = 4，「小于 4 拒」与「不取两条边」同句自相矛盾；按条款式子取槽距时「小于 4 拒」那一支永远走不到 | 两腿一致，打中，挂起。改法两选一交第三轮：门槛改成「小于 5 拒绝 mkfs」，或 mkfs 判槽距等于 D2 已定项 19 的式子（今天没有一处代码把槽距绑到 io_min） |
| W2 写死 8 | E50、D28 不依赖 8；`root_ring.rs` 编译期断言只罩常量 8，按槽距算 S 要新的运行时构造入口（77 处直接填 `AT_MAKE_FILESYSTEM`） | 编译期断言与 harness 模型写死 8 | 两腿一致。实现欠：运行时构造入口与 harness 模型跟着 S 走 |
| W3 改字节造盘的方法 | 等价对 I-7.17 成立：mkfs 对 S 的落盘依赖只有偏移 362 与清零长度；仓里已有同一手法的用例先例 | 量过：打补丁做出只照条款判盘长的写者，直接写出 S = 16、512 KiB 的盘，与改字节盘 0 个扇区不同、25 行历史逐字相同、I-7.17 红 25 / 25，一次没挂就红；用过的盘上两种造法差 5 个扇区 | 第一轮结论站住，主 agent 采：红不是改字节造成的。方法射程收窄一句：「改字节等于合法写者」只在新盘上成立 |
| W4「环深上界」 | 没提 | 打中：同一格前半句不用这个词；D16「环深下限」量 S，这里量 R × S | 坐实，打中。改成「每区槽数上界 16（环长 3 × 16 = 48 个槽）」 |
| W4「焊死管理员回退」 | 与已定项 25 字面不冲突，但「焊死」是已定项 25 的坏词，要点名 E92 | 打中：与已定项 25 定案句相反；管理员回退受 F_生效与这个池自己的 3S 限，48 只是各池的上确界 | 两腿同向，打中。删掉这半句，改成「一个池上能退到哪里由回退候选集（txg ≥ F_生效）与这个池自己的 3S 定」 |
| W4 GPU 表 | 不进格式条款：失败即关的实现容量；另有 harness 的 u64 位图（S ≤ 21）与两处 expect 消息；p 一字节（S ≤ 85）进 | 点名不全：漏 `C_MAX_CANDIDATES = 48u`；改法条款不点 GPU 表名 | 两腿同向。GPU 表、harness 位图从格式条款里拿掉，登进第一轮已登的那笔「GPU 字面量钉不住」欠账；S ≤ 85 留在条款里 |
| W4「冻结前随之抬」 | 与 D15 已定项 8 自洽；「抬上界先抬它」也要带「冻结前」 | 没打中；留一句没说的：冻结后 k_tol 等于随系统配置槽一起冻住 | 站住。补一句「冻结之后 k_tol 不再抬」交第三轮 |

## 三、去向

### 第三轮攻的形态（第二轮修补，被攻过零轮）

D22 已定项 2 字段表「每区槽数 S」那一格，把「**上界由 I-7.4（近 K 代块未被复用） 扣住的块数挂载时算**；4..16 之间取值不碰任何一条边。」换成：

> **每区槽数上界 16 是格式常量**，下界 4 同为格式常量（两条边随系统配置槽落 D15（格式冻结政策） 第 1 层，登记在 layout 一「每区槽数 S」那一行旁）；环长上界 3 × 16 = 48 个槽。上界是读者承诺收的最大值，不从 I-7.4（近 K 代块未被复用） 扣住的块数算：那个「K 代」是按 F 与实例表现算的回退候选集，给不出只由池定的数；块复用的界仍是 max(F_生效, 环里最旧有效根) 现算，不是格式常量（已定项 25）；一个池上管理员回退能退到哪里由回退候选集（txg ≥ F_生效）与这个池自己的 3S 定。上界另受根记录钉槽 p 的一字节约束：钉值最大 3S − 1、255 是哨兵 ⇒ S ≤ 85。**读者解槽时读 P（偏移 363）与 chunk（偏移 367），与第一版常量（3、1 MiB）不等一律整池拒；再判 S × 固定结构槽距 ≤ P × chunk**（区域不越过下一个区域的起点），越界与 S 越界同一个处置：读者（core、池级 checker、GPU 判器）整池拒，mkfs 拒。mkfs 写 min(8, ⌊P × chunk ÷ 槽距⌋)，结果小于 5 拒绝 mkfs，所以所写的值不取两条边。

依据段 E50 那一行换成第一轮判决第三节那一句（正推现算过算术，两腿都没打它）。D16（发布语义） 已定项 1「环深下限」那一格的「S 下界随之抬」改成「冻结前随之抬；冻结之后 k_tol 不再抬」。

实现欠（第三轮站住之后派）：三个读者加 P、chunk 等于常量与 S × 槽距 ≤ 3 MiB 两判，挂 I-7.13，越界用例含第一轮那一盘与 P = 0、chunk = 4 MiB 两盘；mkfs 门槛改成小于 5、S 按槽距算的运行时构造入口与 harness 模型跟着走；两条边的 format-const 标记；placeholder 注释与 C506 行同一次撤；`root_ring.rs`、`lib.rs` 引「已定项 1」的改成已定项 2；下界一侧补对称用例。

### 另登欠账（不挡被判对象）

| 欠账 | 出处 |
|---|---|
| 读者之间 P、chunk 读法不一致：core 不读、只写常量，池级 checker 与 GPU checker 读盘上字段放区域（随 C506 实现一起关；写成欠账是因为它也罩 S 之外的几何） | 两腿 W1 |
| 第一轮已登的「GPU 字面量钉不住」一行补上 `C_MAX_CANDIDATES = 48u` 与 harness u64 位图（S ≤ 21）、两处 `expect` 消息里的 48 | 两腿 W4 |

## 四、多轮小结

第一轮挂起的三处，第二轮各有交代：E50 读错量没被打（正推现算过改写句的算术）；读者不判 S × 槽距被第二轮攻方用合法写者复现，第一轮结论站住；两条边没登记两腿都同意要登。第二轮新打中四处（P、chunk 读法、min 公式撞下界、「环深」、「焊死」），都改在第三轮的形态里。第三轮分工：「已定」项撑不撑得住 / 未定项清单的完整性 / 那一轮报出的数字能不能核。第三轮是最后一轮，之后新冒出来的零轮形态写进交用户表。

## 五、复跑

- 攻方模型：`bash scripts/hosts/run-with-memory-cap.sh 4G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen ~/.local/share/singlefs/work/m3-close-c506-r2-opus/tree --work <副本>/opus-work --model <副本>/opus-model`，退出码 0，末行 `name=rerun_compare same=1 lines=108 sha256=022cf91d59f558673ef67ae597393e362dfc187f925025e9f1b56d1bfc44795d`。
- 正推模型：`bash scripts/hosts/run-with-memory-cap.sh 1G bash <副本>/sonnet-model/tools/rerun.sh --repo . --frozen research/prompts/m3-close-c506-r2-snapshot --work <副本>/sonnet-work --model <副本>/sonnet-model`，末行 `name=rerun_compare same=1 lines=51 sha256=d6e9cb44e300d7bdf165a87f5dc758f37b3410040cb135e5d099464dd41f58df`。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D22（单元原子性怎么合成） 已定项 2 | 支撑 | 2026-10-22 不受影响：第二轮打中挂起，修补交第三轮 |
| D22（单元原子性怎么合成） 已定项 16 | 支撑 | 2026-10-22 不受影响：P、chunk 等于常量那一判与第 1 句同形，交第三轮 |
| D22（单元原子性怎么合成） 已定项 25 | 支撑 | 2026-10-22 不受影响：「焊死管理员回退」那半句删掉，已定项 25 不改 |
| D16（发布语义） 已定项 1 | 支撑 | 2026-10-22 不受影响：「冻结前随之抬」站住，补「冻结之后 k_tol 不再抬」交第三轮 |
