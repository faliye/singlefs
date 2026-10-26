# m3-close-i718-oldslot-r1 判决（推论轮第一轮，2026-10-23）

被判对象：满环时根槽写失败、恢复之前的镜像上 I-7.18（根记录的回收门槛等于现算的门槛）该怎么判，正文 `research/prompts/_m3-close-i718-oldslot-r1-body.md` 第三节三个候选（甲：最新根之后那一槽 N+1 没有自证过的根时按它的旧内容 N+1−3S 再现算一次、t 等于两种现算之一就成立；乙：这一态报不适用；丙：认成已知红、留给 C560）。腿：正推（Sonnet）`research/prompts/m3-close-i718-oldslot-r1-sonnet-output.md`（569ec948…），攻方（Opus）`research/prompts/m3-close-i718-oldslot-r1-opus-output.md`（0e5b65c6…）。

## 一、核查

1. 开工快照：`judged-sha256.txt` 6 行，写判决时 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` FAILED，其余 5 行 OK。改它的是主 agent 打的两份补丁：C506（读系统配置时根环步长三判）与 C478（码 2 头 key 区间），两份都带「快照冲突已判：m3-close-i718-oldslot-r1 …」一行，都不碰 I-7.18 求等那一判。倒推：拷主工作区、把 C506 补丁对这一份的改动反着打（`git apply -R --include=crates/singlefs-checker-tier/src/crash_judge_checker.wgsl`），得冻结树 `~/.local/share/singlefs/work/m3-close-b16-verify/i718-r1-frozen`，6 份逐份过快照（C478 补丁在倒推之后才打，冻结树里没有它）。`kb-sha256.txt` 里 `.claude/kb/checks-owed.md` FAILED：快照之后别处改了一行，两腿引的欠账行（C560、C773、C863）在主工作区现查原文都在。攻方引 GPU 判器写的是交回时的行号 4606（快照那一版 4600），同一行。
2. `python3 scripts/agents/cite-check.py <两份腿报告> --root . --background research/prompts/_m3-close-i718-oldslot-r1-background.md`：「✓ 核了 62 处引文、21 处裸行号，对上 62 处，没判 21 处（其中裸行号没有引文可比 21 处…）」。
3. 主 agent 现查：
   - 攻方格 B 造洞用的 `crash_a_mount_before_its_row_root` 是仓里已有的函数（`crates/singlefs-harness/tests/common_root_ring_holes/mod.rs` 第 207 行「pub fn crash_a_mount_before_its_row_root(image: &MemoryPool) -> (MemoryPool, u64) {」）。
   - D16（发布语义） 第 62 行写「**回收门槛取环里最旧有效根的代价**：崩在记录之后、根槽之前那一形留在环里的旧根…」：环洞是条文点名的合法形。
4. 复跑：见第六节。

## 二、逐格判定

| 格 | 正推 | 攻方 | 主 agent 判 |
|---|---|---|---|
| V1 甲的判别力 | 甲比基线多认一个 t（N+1−3S），只在目标态；第一圈内「p 写 255 而其实钉着下一槽」那一形放过，它与诚实写者逐字节相同（属 C863 同类）；被抛弃的根住在下一槽时写者的 t 是 N+2−3S 一支（推的） | 环洞形上写者把 t 写成 15（真值 3、不安全方向），写坏那张镜像上甲收、今天红；同一条根「根槽写之前崩」那张兄弟镜像全部候选都红（量的） | 两腿量的都成立，方向一致：甲多认的那个值在单张镜像上分不清对错，要靠兄弟镜像兜。记进第二轮 |
| V2 这一态的范围 | 到得了的只有「N+1 的槽读不出」一类；纯掉电 104 个前缀 0 红、扇区边界撕不出、挂载时读坏的镜像全绿（量的）；另列一形三个候选都罩不住：环没满时最旧那条根事后读坏（N=3、5、7、9、10 全红，量的）；N+1 = 3S 那一格投稿的甲（`>`）红，要写 `>=`（量的） | **打中甲**：环洞落在下一槽（格 B），下一槽的旧内容是 N+1−2·3S 那一条，不是 N+1−3S；合法镜像上甲红 36/36，调查员补丁原样打上同红（量的）。**打中乙**：挂载时钉住的槽正好轮到下一槽（格 F），条文点名的两类坏写者（t 写成 F、p 写 255）乙全放过（11/11、12/12，量的）。乙照正文字面第一圈不设闸，S = 4 第一圈 9 张里 8 张不判，入库坏镜像用例 `the_root_ring_health_invariant_reddens_when_the_previous_generation_is_gone_but_not_right_after_mkfs` 会红（量的） | 两腿都量过、互不矛盾、各补了对方没走到的形：正推没造环洞，攻方没造「环没满时最旧根事后读坏」。**甲照定义打穿**（括注「N+1−3S 那一条根」被格 B 打穿，判决采攻方）；**乙打穿**（判别力）；N+1 ≥ 3S 那条边采正推 |
| V3 与已定条款 | 甲要补两处条文、改两行变异锚（第 5214、5220 行）；乙补一处；丙与用户 2026-10-20 对 C773 的态度（`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 386 行「不记成已认代价，另立一笔新欠账」）、C560 要的自证、I-7.19 口径都相反；D16 第 36 行「六处同取」在目标态上今天就不逐字成立；C560 那一行写的红是 I-3.1，C773 之后已挪到 I-7.18（量的） | 甲照字面与 D16 已定项 1「写失败的槽按旧内容算」打架（环洞形上旧内容不是 N+1−3S）；丙要让凡是转满一圈的流都豁免 I-7.18（推的） | 采：丙不取（两腿都给了代价，正推量出这一态上 I-7.18 与 I-3.1 没有共同绿的 t，丙等于把一个写者改不出来的红写进条文）。C560 那一行要改写、D16 第 36 行要补一句，随定案写回 |
| V4 两侧判器 | GPU 同函数可照改，naga 校验通过、阴性对照红（量的）；对拍没跑；甲 7 盘、乙 3 盘 | 甲、乙在 GPU 上多一次 divmod64 与一遍根表；丁要在 GPU 一侧查 journal 记录，代价没量（推的） | 记进第二轮 |
| 改法（零轮） | 「每个缺位」版：环里每个读不出的位置按同位置在 (N−3S, N] 里的 txg 补进去再现算（推的，没写没量） | 甲-任一圈、甲-任一读不出槽、丁甲、丁乙（只在攻方模型上量过）：丁甲在 A、B、F 合法镜像全绿，F 的两类坏写者都红；剩两格收：F「p 写 255、N+1 在飞」（推的，靠记录落盘之前那张兄弟镜像兜）、B「t 写 15」（量的，靠根槽写之前那张兄弟镜像兜）；甲-任一读不出槽让入库坏镜像用例红 | 零轮，组合进第三节第二轮的形态 |

## 三、去向

第一轮两腿结论不一致：甲照定义被格 B 打穿，乙被格 F 打穿，丙两腿都不取。开第二轮。

### 第二轮攻的形态（主 agent 合成，被攻过零轮）

**丁甲**：最新根（txg N）之后那一槽读不出自证过的根，且 journal 里有点名 N+1 那次发布的记录（D16 已定项 7 的持久顺序：记录先于根槽落盘），且 N+1 ≥ 3S 时，I-7.18 判 t 等于下面几种现算之一就成立：

1. 今天的现算（照镜像上的环）；
2. 把那一槽按它在更早任一圈住过的根（txg ≡ N+1 (mod 3S)、小于 N+1）补进环之后的现算；
3. 被抛弃的根住在那一槽时的那一支（正推第 V1 格推的 N+2−3S）。

不满足前提（那一槽有自证过的根，或 journal 里没有点名 N+1 的记录，或 N+1 < 3S）照今天判。CPU 池级 checker（`crates/singlefs-checker/src/walk.rs`）与 GPU 走读内核（`crash_judge_checker.wgsl`）同改。

**丙**作退路留着，对照用。

### 第二轮要答的格

| 格 | 问什么 |
|---|---|
| W1 | 丁甲第 2 种现算「任一圈住过的根」是不是只看镜像就定得出（判器看得到的只有 t、t ≡ N+1 (mod 3S) 与环上别的槽）；它比攻方量过的丁甲多放过哪些错 t、错 p；「p 写 255、N+1 在飞」与「环洞形 t 写 15」两格靠兄弟镜像兜住，在崩溃点重放的枚举里兄弟镜像是不是一定也被判 |
| W2 | 前提「journal 里有点名 N+1 的记录」：D16 已定项 7 的持久顺序是不是保证这一态上记录一定在；记录读坏、journal 环转过一圈之后这一前提是不是还判得出；判器读 journal 在 CPU、GPU 两侧各有没有现成的交接 |
| W3 | 从未被看过的形：正推列的「环没满时最旧那条根事后读坏」（三个候选都红）该不该绿、丁甲罩不罩；撕裂域里的邻槽；零单元发布与无文件版本上的复制根（I-7.18 条文里「不判」的那一类）与这一态叠在一起 |
| W4 | 产生结论的方法：两腿的装置各只取了 S = 4（环长 12）一种几何、手挑崩溃状态，没调 `enumerate_crash_states` 把写表整段展开；换几何、换历史（零单元发布、空发布）结论翻不翻 |

## 四、另登欠账与改正

| 项 | 去向 |
|---|---|
| C560 那一行写的红是 I-3.1，C773 之后已挪到 I-7.18（正推量的）；行文随定案改 | 随第三轮定案写回 |
| D16（发布语义） 第 36 行「六处同取」在目标态上不逐字成立（正推） | 随定案写回，或交用户 |
| 第一圈内「p 写 255 而其实钉着下一槽」与诚实写者逐字节相同、谁都判不出（正推，C863 同类） | 第二轮看要不要并进 C863 |

## 五、多轮小结

第一轮：甲的括注被一段合法历史（环洞落在下一槽）打穿，乙的判别力被钉槽轮到下一槽那一形打穿，丙两腿都不取。第二轮攻第三节合成的丁甲，分工照 `.claude/rules/three-way-inference.md` 第二轮：第一轮的修补本身 / 产生结论的方法 / 从未被看过的地方，派一条辩方腿复核这一轮的判决。

## 六、复跑

两份模型都拷进副本 `~/.local/share/singlefs/work/m3-close-b16-verify/i718-r1-rerun/` 再跑，`--frozen` 都给冻结树 `i718-r1-frozen`。

- 攻方：`bash scripts/hosts/run-with-memory-cap.sh 8G bash <副本>/opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结树> --work <副本>/opus-work`，退出码 0，末四行：
  - `name=cells_condensed same_as=out/cells-condensed.txt lines=78`
  - `name=scan same_as=out/scan-one-revolution.txt lines=180`
  - `name=investigator_patch same_as=out/investigator-patch-cells-a-b-condensed.txt`
  - `name=regressions same_as=out/regressions.txt`
- 正推：`bash scripts/hosts/run-with-memory-cap.sh 12G bash <副本>/sonnet-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen <冻结树> --work <副本>/sonnet-work --model <副本>/sonnet-model`，退出码 0：六组交叉核「与开关版 b 逐行相同」（104、306、10、20、8、58 行），naga 五行（基线、a、a_seed、b 都 `VALIDATE_OK`，阴性对照报 `'judge_the_reclaim_threshold_and_the_pinned_ring_slot_of_the_newest_root' is invalid`），末行「与 …/sonnet-model/out/summary.txt 逐行相同」。

## 回看决策

| 决策分项 | 关系 | 回看 |
|---|---|---|
| D16（发布语义） 已定项 1 | 支撑 | 2026-10-23 不受影响：第一轮不收敛，开第二轮 |
