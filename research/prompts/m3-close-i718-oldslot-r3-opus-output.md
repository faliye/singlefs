# m3-close-i718-oldslot-r3 攻方腿（Opus）报告（2026-10-23）

立场：假设丁甲、丁乙、丙与前两轮报的数都有错，造合法历史与坏写者去打。数全在我的仓副本上量（副本上的数，不是入库装置上的数）。这是最终轮：下文我提的两样改法（丁甲加第 0 代根、丁乙保 I-7.19）只在我的模型上量过、被攻过零轮。

## 复跑命令与文件 sha256

```
bash scripts/hosts/run-with-memory-cap.sh 8G bash research/prompts/m3-close-i718-oldslot-r3-opus-model/tools/rerun.sh \
  --repo /home/fy5090/code/singlefs --frozen /home/fy5090/code/singlefs --work <新的草稿目录>
```

- `--frozen` 给仓根即可：开工时主工作区与 `research/prompts/m3-close-i718-oldslot-r3-snapshot/judged-sha256.txt`、`kb-sha256.txt` 逐行 OK（`sha256sum -c` 13 行全部 OK）。rerun 只拷 `crates/`、`Cargo.toml`、`Cargo.lock`、`.cargo`、`.claude/heavy-thread-default`，walk.rs 依次装四道开关（第一轮两道、第二轮一道原样拷来，第三轮一道 `walk-candidate-switch-round-three.py` 新写），探针包 `crates/i718-round-three-probe` 加进副本工作区，`cargo build --release`，19 件单线程并行跑，再汇总、重数第二轮辩方的 raw。
- 末尾每份产物一行 `name=… same_as=…`，任一行 `DIFFERS` 就看它点名的 diff。`compare-sonnet-r2-leg-vs-verdict-rerun` 那一行要第二轮判决复跑的目录还在（默认 `~/.local/share/singlefs/work/m3-close-b16-verify/i718-r2-rerun/sonnet-work/out`，`RERUN_R2_VERDICT_RERUN` 可改），不在就打 skipped。
- 挂钟：编译约 1–2 分钟；19 件在 loadavg 70–90 的本机上 `nice -n 19` 并行约 20 分钟（我那一批 `real 19m36.873s`），空闲时应短得多（推的）。内存：每件我给 2G 的包装上限都没超，8G 是按 19 件同时在一个包装里粗估的（推的）。
- rerun.sh 整条没在前台跑过（超前台时限）；各步我在草稿目录分开跑过，`out/` 里就是那几步的产物。z2-alternating 那一件跑过两次：头一次探针里「每隔三轮试一次挂载写坏」的条件写成按 txg 取余、一次都没取到，改成按轮取余后重跑，`out/` 里是重跑那一份。
- 不碰 checker 档的测试：探针是依赖 `singlefs-checker-tier` 的二进制包（只用它算全量崩溃状态数），只 `cargo build`、直接执行编出来的二进制（定义第 3b 步）。产物里没有内存包装打的提示行（`grep -l run-with-memory-cap out/*.txt` 无命中）。

```
8204fd2f7c6b3dbd8305cbef9b6a54a0896f4cae6b14bf68d547bd6a493d8cf7  out/compare-sonnet-r2-leg-vs-verdict-rerun.txt
d2593a233a76513247a0c3ccdd09e1c240caf71821fa49d48c59f4ea1734e6c4  out/counts-S4.txt
db0ac3a7b8386006cde62bf6165649ddf1d2b4bd2fb5af3ad98ea56c41006f02  out/p-scan-summary.txt
faf834ac6ebf3f7cd45fb8a9819d9a97d249dd7a1589597e6a7379d38a6e3bfc  out/recount-extra-sonnet-r2.txt
d20af07d1107f8a89f6d22ff082c5ffbef114f8bddfd3b2d30f2b995bde10b05  out/recount-keys-sonnet-r2.txt
ba5da52258d90706502bcc00023ec371825fe8d9e8f03993e26978169368072d  out/tally.txt
bb18dcf384d2b139080e2b54761d7dd26a1bfb0f927f12456173a708780f7076  out/z1-S4.txt
863459002ca91dc6529ed0782e0fcb9b1555d22a35b043aee382c4b27f54172f  out/z2-alternating-S4.txt
c29a48b7ca47c15f5f1830599f2506554edfd45b7f84fe63c7939bf30934fd87  out/z2-floor-S4.txt
1c0e4b3b9ade6faee4e94f80d1f37cc80a997dea6987c8e7c2ff658b46057f6a  out/z2-gen0-S4.txt
9d762c4776c6fc26c78f123e5f10868b46cb8a2516a76ec40359ea187a96e055  out/z2-gen0-S5.txt
c59bdbcc70a1cacdf6dddc1fde712d1db48f6c1e51aa851d3279999cf237c229  out/z2-gen0-S8.txt
22e66e18e2ef1f3a3749a9f9cbd3a28063fb2910c9296693481bde51a6bf9c9b  out/z2-holes-S4.txt
678a65417b1c932aa6cb350d1f050ce49fda2c0f225e7497ae60fb645792adba  out/z2-p-S4.txt
f39411c752b5e1b806e42db488b595c7a53956834fad8606e1d39ef1727f85b1  out/z3-br-S16.txt
5f57f5578f26c05e1a82ddf51da3bbfb923a01fe861e1bd7d0d6a0b052eea458  out/z3-br-S4.txt
77e08f7d98a87a5b5cb39b25caebf426a1013645173d91db143134471322e42d  out/z3-br-S5.txt
09b4d2887d93580e3e93e08e20049f9ac6b6a76fc9ebd57b585d27521cc9250b  out/z3-br-S8.txt
e95e999f8ffd3ce1223d29ef6ad49ec2823207fded317e807a45b67e3c3bab29  out/z3-b-S4.txt
3ecd449ebe446c9abe4392952380c5f359860f76783f97679ea9988355a600fd  out/z3-f-S4.txt
d408149e4909cdf1f6dac47559750bd263b5c76bd2f0a7f6d3443520959ff791  out/z3-g-S16.txt
6408408bf4c8492ec31140b774fd8ed50d23c8c4661f22840132c4ae83fad07e  out/z3-g-S4.txt
061fb17c52ba1b40daae6601212554587a6b127fcdccadc70059cdfdc106b62a  out/z3-g-S5.txt
acdf614e1934357d2561023a333e8ea3462d2b06e555fe5da2b2e953e261e64f  out/z3-g-S8.txt
3d06eec79125b2d711d6e7bb6ad9d3321f4e380e7619f2e47864c11baf276006  probe/Cargo.toml
4a343ab7a592f400265d1923b26a19e63d4dfcfeca4170b82bf35ccef2f6f50d  probe/src/main.rs
ee9f3c4b425b9339110d96c8d02cabacc4657c0626222e13f075c29082ddbc1c  probe/src/ring_hole_pools.rs
2e5a1b715e539d4e6dd4d9e9d3cb5570dcb9e639072f7eca0f9c595d29e7b4d0  tools/compare_runs.py
6cdc8ec9fca283f761bde730125eae1ba347a670cc7421b49d5aee82c58d781f  tools/p_scan_summary.py
3454fd298244fbedb389f30fc108505143fed61749028e01a23224ecc0f3c22f  tools/recount_extra.py
9fc42ad2642e1c06c2ab7e40022a0d1e1b984ce9a91c02c7c1f8161d86582e2e  tools/recount_keys.py
4c7e14e6c934289ddad11e93bf4ef00cf42839f33027b60aa611d6e17cd09abb  tools/rerun.sh
bf8203cd81b41b01e08d79d983e3efb1ed419c9b57d514a46815c89049550395  tools/tally.py
2751684b2bbbeb40d4b4be4f5553fe25e04aa128698380c2742dbd1b1a04a408  tools/walk-candidate-switch-ding.py
bd90ed1e1addf6388c2ba5e613694498edcead8aa0f8ee20699cff84567fc4d9  tools/walk-candidate-switch-ding-round-two.py
e3cb973e32d46786f7b1eabad4ff2d42106ed8c3e98a7326bffce76c3001e480  tools/walk-candidate-switch.py
aaeb5bc243e4184edbc8a23802cf0568886be20b01ee159f44d909d6f3117892  tools/walk-candidate-switch-round-three.py
```

（路径都在 `research/prompts/m3-close-i718-oldslot-r3-opus-model/` 下。）`out/counts-S4.txt` 原样（新造的几条流的全量崩溃状态数，见「这条腿自己的限度」第一条）：

```
S=4 stream=first-mount-on-a-mkfs-pool writes=14 segments=8 states=41
S=4 stream=mount-after-the-first-mount-crashed-before-its-row-root writes=27 segments=12 states=308
S=4 stream=overwrite-onto-the-slot-generation-zero-survived-in writes=25 segments=4 states=1048588
```

## 各格判定一览

候选列名（探针输出里的名字）：`today` 今天，也是丙；`jia` 第一轮甲；`yi_literal` 第一轮乙字面；`ding_jia_same_instance` 第二轮打穿的同实例读法；`ding_jia` 第三轮正文的丁甲（任一实例、N+1 ≥ 3S、第 2 种按补进环之后现算，第 3 种与第 1 种相同不另装）；`ding_jia_gen0` 我这一轮提的丁甲加第 0 代根（零轮）；`ding_yi` 第三轮正文的丁乙（任一实例、N+1 ≥ 3S 时 I-7.18 不适用，walk.rs 里 I-7.19 照判）；`ding_yi_literal` 丁乙照字面连 I-7.19 一起不判（探针按 I-7.18 不适用的那几张改记）。G 成立、R 判红、NA 不适用。

| 格 | 判定 | 量的还是推的 | 一句话 | 依据（kb 文件:行） | 什么现象会推翻它 |
|---|---|---|---|---|---|
| Z1-甲 丁乙照字面连 I-7.19 一起不判 | **打中丁乙（字面）** | 量的（I-7.18、I-7.19 两列） | 钉槽写掉之后不解钉的坏写者，N 做完那张上只有 I-7.19 红；写坏镜像上丁乙字面两条都不判，丁甲、丙照红 | `crates/singlefs-checker/invariants/02-根环.md` 第 26 行「口径同 I-7.18」；`.claude/kb/checks-owed.md` 第 548 行「判得出三类」 | 丁乙定成「I-7.19 照判」；或真坏写者（记录与账跟着写）在写坏镜像上另有一条不变量红 |
| Z1-乙 三个形态各要改哪一句 | 列清单 | 推的（读条款） | 丁甲改 I-7.18 两句与 C863 一句；丁乙另改 I-7.19「口径同」；丙改两处场景期望与 C560 的自证句 | 02-根环.md 第 25、26 行；checks-owed.md 第 351、548 行；`.claude/kb/decisions/16-发布语义.md` 第 22、224 行 | 条款里另有一句已经写了这些例外 |
| Z1-丙 坏写者按 C863 怎么登记 | 列做法 | 推的 | 那个坏写者原在 C863「判得出三类」里，要改那一句；照用户「不记成已认代价」只能写成运行期判或交出兄弟镜像 | checks-owed.md 第 548 行；`records/2026-10-18-里程碑三收尾与近期欠账.md` 第 386 行 | 找到一条单张镜像上分开它与合法写坏镜像的判法 |
| Z2-甲 第 0 代根留过第一圈（清单没列） | **打中丁甲** | 量的，S = 4、5、8 | 第一次可写挂载崩在写行或暖机根槽之前，区域 1 或 2 槽 0 的第 0 代根留到 3S+1、3S+2 那次发布写坏它；丁甲只补得进 N+1−3S，合法写坏镜像判红；独立判法同红 | 16-发布语义.md 第 224 行（txg 1、2 盖掉第 0 代根）；`crates/singlefs-core/src/make_filesystem.rs` 第 826 行 | 再挂的写者不把第 0 代根当有效根（t ≠ 0）；入库装置上这张镜像 I-7.18 不红 |
| Z2-乙 几个洞 | 没打中 | 量的，S = 4 | 洞挨着、隔开，两圈全扫，丁甲 0 张误红 | — | 有洞里的根 txg 不 ≡ N+1 (mod 3S) 的历史（Z2-甲就是一种） |
| Z2-丙 洞与抬 F | 没打中（卸载抬 F） | 量的，S = 4 | F 压过最旧根时今天就绿；F 之下丁甲绿 | — | 准入抬 F 那一形误红 |
| Z2-丁 多实例交替 | 没打中 | 量的，S = 4 | 覆盖写写坏与下一次挂载写行、暖机写坏，丁甲 0 张误红 | — | — |
| Z2-戊 p 的其余取值 | **打中丁乙（字面）**；量出丁甲加第 0 代根的代价 | 量的，S = 4 | 36 份坏写者在写坏镜像上：今天、丁甲各放过 1 份（p 钉着下一槽、t 跟着钉，今天自己的盲点），丁乙字面放过 36 份，丁乙保 I-7.19 放过 4 份，丁甲加第 0 代根放过 3 份（多出两份 t = F） | 02-根环.md 第 25、26 行 | 别的几何上 p 扫描换向 |
| Z2-己 `WithTreesAndNoFile` 之后的写行 | 没量 | 推的 | 写行带单元、建账，推与第二轮格 G 同形 | `crates/singlefs-core/src/mount.rs` 第 1153 行 | 造出来误红 |
| Z2-庚 GPU 交接与对拍盘 | 给了写法与盘单 | 推的 | 记录表是逐盘草稿区，要在判里重走一遍位置表；第 2 种写成常数步；丁乙要保 I-7.19 就得把那一段挪出提前 return | `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl` 第 389、4672 行 | — |
| Z3 第一轮格 B 36/36、格 F 11/11 与 12/12 | 核上 | 量的（另一份探针重造、另一份判法） | B 甲 36/36 红；F 乙字面 36 张全不适用，t 写成 F 12 张里 1 张与合法相同 | — | — |
| Z3 第二轮 G 3/3、BR 14 与 50、1890 张 | 核上（G 的口径要补一句） | 量的 | G 的 3/3 只数了写 0 那一种；BR、1890 逐字相同 | — | — |
| Z3 第二轮「33 个键分类不变」 | 结论核上，说法宽了 | 量的（重数） | 33 是过筛之后的键数，四个 S 都有的只 29 个；全部 57 个键 0 个变类 | — | — |
| Z3 第二轮判决第六节的复跑差异 | **来历归错一部分** | 量的（重数、读退出码） | 归类 0 差异；张数差异 56 个键里 12 个是复跑那两件被 225 秒封顶截断（退出码 124），不是探针版本 | — | 那两件的退出码文件不是 124 |

## 几个改法各修哪一格（我提的两样只在我的模型上量过、被攻过零轮）

「量过」= `out/` 里的原样输出（S = 4，括号里另注几何）；「推的」= 没在这一轮的副本上跑。p 扫描一列写的是 36 份坏写者里在写坏镜像上整张放过（I-7.18、I-7.19 都不红）的份数。

| 改法 | Z2-甲 第 0 代根写坏（合法） | 第一轮格 B 写坏（合法） | 第二轮格 G 写行写坏（合法） | Z1-甲 不解钉坏写者（写坏那张） | Z2-戊 p 扫描（写坏那张） | 「p 写 255、N+1 在飞」（坏） | BR 满环最旧根读坏（第二轮判该红） |
|---|---|---|---|---|---|---|---|
| 今天 / 丙 | 红（量过，S = 4、5、8） | 红（量过） | 红（量过，S = 4、5、8、16） | I-7.19 红（量过） | 放过 1（量过） | 红（推的，第一、二轮量过） | 红（量过，S = 4、5、8、16） |
| 丁甲（第三轮正文） | **红（量过，S = 4、5、8）** | 绿（量过） | 绿（量过） | I-7.19 红（量过） | 放过 1（量过） | 收（推的，第二轮两腿量过） | 红（量过） |
| 丁甲加第 0 代根（零轮） | 绿（量过，S = 4、5、8） | 绿（量过） | 绿（量过） | I-7.19 红（量过） | 放过 3：多出 t = F 的两份（量过） | 收（推的） | 红（量过） |
| 丁乙照字面（I-7.19 一起不判） | 不适用（量过） | 不适用（量过） | 不适用（量过） | **放过**（量过） | **放过 36**（量过） | 不适用（推的） | 红（量过） |
| 丁乙保 I-7.19（零轮） | 不适用（量过） | 不适用（量过） | 不适用（量过） | I-7.19 红（量过） | 放过 4（量过） | 不适用（推的） | 红（量过） |

- 丁甲加第 0 代根的代价：下一槽是某个区域的槽 0（S 个槽里 1 个）的写坏镜像上，「t 恒取 F」这一类坏写者被收下（p 扫描里 `p=255 t=0` 与 `p=2 t=0` 两份，今天与丁甲都红）。收窄成只在 N+1 ≤ 2·3S 时补 txg 0，能把这一代价限到第一圈边上，但同一槽在第一圈之后又连着出洞时第 0 代根还能再留一圈，那一形会回到误红（推的，没造）。
- 丁乙保 I-7.19 比字面少放 32 份；剩下 4 份里 3 份是「p 钉着下一槽」那一族（下一槽写坏之后 I-7.19 没东西可看），1 份是 t = F、p = 255。

## Z1「已定」项撑不撑得住

### Z1-甲 丁乙照字面连 I-7.19 一起不判，放掉「写掉钉着的槽不解钉」——打中丁乙（新）

丁乙的定义是「这一态报不适用（照 I-7.18 条文里「不判」那一类的写法记）」，而 I-7.19 的适用范围是跟着 I-7.18 走的：`crates/singlefs-checker/invariants/02-根环.md:26`：「只判建账的发布，最新根下面没有记账树时报不适用，口径同 I-7.18（根记录的回收门槛等于现算的门槛）」。今天 walk.rs 里「不判」那两类（没有记账树、没建账）都是两条一起报不适用。照字面，丁乙在写坏镜像上也把 I-7.19 一起关掉。

I-7.19 管的正是这一类坏写者：`crates/singlefs-checker/invariants/02-根环.md:26`：「那一槽已住着本实例的根而 p 还指着它，就是写掉了没解钉」；C863 也把它列在判得出的三类里：`.claude/kb/checks-owed.md:548`：「判得出三类坏实现（恒取 F、写掉钉着的槽不解钉、p 写 255 但其实还钉着槽）」。

造出来的历史（S = 4，`z1` 模式）：

1. 实例 1 写到 txg 14，txg 15 的根槽写成 0 后崩（题面那一形）。
2. 实例 2 可写挂载，那一槽读回全零被钉住（D16 已定项 1「读坏的槽与读回全零的槽」那一句），根记录带 (t, p) = (3, 3)。
3. 实例 2 写到 txg 27，正落在钉住那一槽。诚实写者解钉：(16, 255)。坏写者不解钉，t 照留着的钉算：(3, 3)。
4. 下一次发布 txg 28 的根槽写坏，进程在重发之前崩。

量的结果在下面「Z1-甲的数」一段贴原样。要点：坏写者在 N 做完的镜像上 I-7.18 七列全绿（t 与按钉现算相等），只有 I-7.19 红；到了写坏镜像上，丁乙那一列 I-7.18 报不适用，照字面读 I-7.19 也跟着不判，I-7.18、I-7.19 两列都判不出它（这张改字节造的镜像上 I-3.1、I-8.10 另红，是副产物；真坏写者记录与账跟着写时这两条不红，推的，见「这条腿自己的限度」第二条）。丁甲与今天在同一张上 I-7.19 照判红。

四句：

- 分不分辨臂：分。丁乙（字面）放过，丁甲、丙照红。第一轮、第二轮量的丁乙都是「I-7.19 照判」的写法（我第一轮开关里 `judge_the_pinned_ring_slot_only`），与正文字面不同，所以前两轮丁乙的数不覆盖这一读法。
- 被判的系统看不看得到：看得到。钉槽那一槽住着本实例的根就在镜像上；是丁乙的字面把这一判关掉了。
- 满足的分句：Z1「三个形态各要改哪一句，改了会不会和别的已定项字面打架」：丁乙要么改 I-7.19「口径同 I-7.18」那一句（写明丁乙新加的这一类不适用不传给 I-7.19），要么认下第二个漏报类，与 C863「判得出三类」那一句打架。
- 改法在这一格上中不中：把丁乙写成「只有 I-7.18 报不适用，I-7.19 照判」，这一格不再中（量的：丁乙那一列 I-7.19 照判红）。这是我提的，零轮。

兄弟镜像：同一个坏写者在 N 做完那张上 I-7.19 红（量的）。但第二轮已经说明崩溃点重放的枚举走不到写坏的根槽，给定状态里也不一定交出那张，所以「兄弟镜像兜住」在这里同样没有载体（推的，沿用第二轮的结论）。

### Z1-乙 丁甲、丁乙各要改的句子（读条款，推的）

| 形态 | 要改的句子（kb 文件:行） | 改了与别处打不打架 |
|---|---|---|
| 丁甲 | `crates/singlefs-checker/invariants/02-根环.md:25`：「这一条判得出（后者现算的门槛不带钉、比 t 高，求等红」——在前提成立的写坏镜像上，「p 写 255、N+1 在飞」的坏写者判不出（第二轮已定的真取舍），要加「写坏镜像上除外」 | 与 C863 那一行打架：`.claude/kb/checks-owed.md:548`：「判得出三类坏实现（恒取 F、写掉钉着的槽不解钉、p 写 255 但其实还钉着槽）」，第三类要改成有例外 |
| 丁甲 | 同一行「checker 取这条根自己带的 F 现算回收门槛，要求等于 t」要加三种现算的例外；第 2 种「更早任一圈住过的根」还要写明含 mkfs 的第 0 代根（Z2-甲，新） | 与 D16 已定项 8 那句格式常量不打架，但要连它一起引：`.claude/kb/decisions/16-发布语义.md:224`：「txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘」——第一次挂载崩在这两次根槽写之前，第 0 代根就留过第一圈 |
| 丁乙 | 02-根环.md 第 25 行加第三个不适用的理由；第 26 行「口径同 I-7.18」要写明不随丁乙这一类（上一节） | 不改第 26 行就与 C863「写掉钉着的槽不解钉」判得出那一句打架（量的，上一节） |
| 丙 | I-7.18 条文不动；场景期望要改：`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs:1020`：「&["I-3.1"]」与同文件 `:1360`：「PoolCheckerExpectation::exactly_these_violated(&["I-3.1"])」写的都是 I-3.1 | C560 要的自证在丙下照字面做不到：`.claude/kb/checks-owed.md:351`：「判别力自证：换回现行判法，那条用例必须由绿转红」——丙就是现行判法（第一轮正推已指出，这里只确认它仍是字面冲突） |
| 三者 | D16 已定项 12 的合法性口径：`.claude/kb/decisions/16-发布语义.md:22`：「根 + journal 任意前缀，replay 之后合法」 | 丙在这一态上判一张合法镜像红；丁甲、丁乙不判红。这一条不新，是第二轮真取舍的另一面 |

### Z1-丙 坏写者按 C863 怎么登记（推的）

- 丁甲、丁乙收下的「p 写 255、N+1 在飞」不属于 C863 现有的两类（那两类是读故障不留痕），它属于 C863 列为「判得出」的第三类，在写坏镜像这一态上变成判不出。登记时要改 C863 那一句，不是往「判不出的两类」里加一类就完事。
- 用户对 C863 的立场：`records/2026-10-18-里程碑三收尾与近期欠账.md:386`：「不记成已认代价，另立一笔新欠账」，并要做成判得出的形态。第二轮两腿一致：单张镜像上判不开。所以照这一立场登记，还清它的路只剩 C863 自己留的口子：`.claude/kb/checks-owed.md:548`：「判在镜像上还是在运行期、盘上要不要多记什么，没定」——要么运行期判，要么给定状态连同 N 做完的那张一起交出去（第二轮我提的改法）。登记成「已认代价」与用户立场相反；登记成「要做成判得出」就要写明靠哪一条路，镜像上这一条走不通。
- 丁乙（字面）另多一类（上面 Z1-甲）：写掉钉着的槽不解钉。丁乙若照字面定，C863 要登记两类新的「写坏镜像上判不出」。

### Z1-甲的数

`out/z1-S4.txt` 头一行与坏写者写 0 那一张原样：

```
S=4 cell=Z1-not-unpinned carrying_before=(3, 3) honest_after_writing_the_pinned_slot=(16, 255)
S=4 cell=Z1-not-unpinned-bad image=zero newest=(27, 2) t=3 p=3 F=0 next_slot=None rec=same:true,any:true | I-7.18 today=G jia=G yi_literal=NA ding_jia_same_instance=G ding_jia=G ding_jia_gen0=G ding_yi=NA | I-7.19 today=R jia=R yi_literal=R ding_jia_same_instance=R ding_jia=R ding_jia_gen0=R ding_yi=R ding_yi_literal=NA | independent_today=G independent_ding_jia=G | today_red=["I-3.1", "I-8.10", "I-7.19"]
```

`out/tally.txt` 里 Z1 那几行原样（列序见行首）：

```
key | n | I-7.18 today jia yi_literal ding_jia_same_instance ding_jia ding_jia_gen0 ding_yi | I-7.19 today ding_yi ding_yi_literal
S=4 Z1-not-unpinned-honest healthy_n | n=1 | G1 G1 G1 G1 G1 G1 G1 | G1 G1 G1
S=4 Z1-not-unpinned-bad healthy_n | n=1 | G1 G1 G1 G1 G1 G1 G1 | R1 R1 R1
S=4 Z1-not-unpinned-honest before_root | n=1 | G1 G1 G1 G1 G1 G1 G1 | G1 G1 G1
S=4 Z1-not-unpinned-bad before_root | n=1 | G1 G1 G1 G1 G1 G1 G1 | R1 R1 R1
S=4 Z1-not-unpinned-honest zero | n=1 | R1 G1 NA1 G1 G1 G1 NA1 | G1 G1 NA1
S=4 Z1-not-unpinned-bad zero | n=1 | G1 G1 NA1 G1 G1 G1 NA1 | R1 R1 NA1
S=4 Z1-not-unpinned-honest torn_enum | n=1 | R1 G1 NA1 G1 G1 G1 NA1 | G1 G1 NA1
S=4 Z1-not-unpinned-bad torn_enum | n=1 | G1 G1 NA1 G1 G1 G1 NA1 | R1 R1 NA1
S=4 Z1-not-unpinned-honest torn_half | n=1 | R1 G1 NA1 G1 G1 G1 NA1 | G1 G1 NA1
S=4 Z1-not-unpinned-bad torn_half | n=1 | G1 G1 NA1 G1 G1 G1 NA1 | R1 R1 NA1
```

## Z2 未定项清单的完整性

### Z2-甲 mkfs 的第 0 代根留过第一圈，下一槽的旧内容不在 N+1−k·3S 那一族——打中丁甲（新，清单上没有这一形）

前两轮两腿都把第 2 种「更早任一圈住过的根」实现成 txg ≡ N+1 (mod 3S) 那一族（N+1−k·3S），第二轮辩方的理由是例外只有 mkfs 第 0 代根、只在 N+1 = 3S 一点。这一点有一个合法的反例：mkfs 把第 0 代根种在三个区域的槽 0（`crates/singlefs-core/src/make_filesystem.rs:826`：「(0..ROOT_RING_REGIONS).map(|region| RootRingSlot { region, slot: 0 })」），区域 1、2 的槽 0 照落点公式属于 txg 1、2，靠第一次可写挂载的写行与暖机盖掉（`.claude/kb/decisions/16-发布语义.md:224`：「txg 1 落区域 1（盘 1）、txg 2 落区域 2（盘 0），两次正好覆盖两块盘」）。第一次挂载崩在这两次根槽写之前，第 0 代根就在那一槽里留过第一圈。

历史（每一步许可它的那一句）：

1. mkfs。
2. 第一次可写挂载，写行那次（txg 1，零单元发布）的记录已落、根槽写之前崩（`crash_a_mount_before_its_root_write` 第 0 次；第二种是写行落了、暖机 txg 2 的根槽写之前崩）。这与第一轮格 B 造洞是同一种单故障（D16 已定项 1 点名的「崩在记录之后、根槽之前那一形」）。
3. 再挂（实例 2）：写行取 txg 2、暖机 3、4，建第一个文件 5，一直覆盖写。挂完的环原样：`ring=[(0, 0), (0, 0), (2, 2), (3, 2), (4, 2), (5, 2)]`（区域 0 槽 0 与区域 1 槽 0 里都是第 0 代根）。
4. 写到 N = 3S（S = 4 时 12）。写者把那条第 0 代根当环里最旧有效根，t = 0。
5. N+1 = 3S+1 落在区域 1 槽 0（第 0 代根那一槽），根槽写坏，进程在重发之前崩——题面那一形。

S = 4 那一张原样（`out/z2-gen0-S4.txt`）：

```
S=4 cell=Z2-gen0-crash0-N12 image=zero newest=(12, 2) t=0 p=255 F=0 next_slot=None rec=same:true,any:true | I-7.18 today=R jia=R yi_literal=NA ding_jia_same_instance=R ding_jia=R ding_jia_gen0=G ding_yi=NA | I-7.19 today=G jia=G yi_literal=G ding_jia_same_instance=G ding_jia=G ding_jia_gen0=G ding_yi=G ding_yi_literal=NA | independent_today=R independent_ding_jia=R | today_red=["I-7.18"]
```

丁甲（`ding_jia`，第三轮正文那一版：任一实例、`>=`、第 2 种按补进环之后现算）判红；我另写的独立判法也红（`independent_ding_jia=R`）；镜像上红的只有 I-7.18 一条。丁甲只补得进 N+1−3S = 1，现算是 1，写者的 t 是 0。同一形三种几何都中（`out/tally.txt` 原样，每行是一种写坏、一圈里 5 或 6 个 N）：

```
key | n | I-7.18 today jia yi_literal ding_jia_same_instance ding_jia ding_jia_gen0 ding_yi | I-7.19 today ding_yi ding_yi_literal
S=4 Z2-gen0-crash0 zero | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=4 Z2-gen0-crash0 torn_enum | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=4 Z2-gen0-crash0 torn_half | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=4 Z2-gen0-crash1 zero | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=4 Z2-gen0-crash1 torn_enum | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=4 Z2-gen0-crash1 torn_half | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=4 Z2-gen0-control zero | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=4 Z2-gen0-control torn_enum | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=4 Z2-gen0-control torn_half | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=5 Z2-gen0-crash0 zero | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=5 Z2-gen0-crash0 torn_enum | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=5 Z2-gen0-crash0 torn_half | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=5 Z2-gen0-crash1 zero | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=5 Z2-gen0-crash1 torn_enum | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=5 Z2-gen0-crash1 torn_half | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=5 Z2-gen0-control zero | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=5 Z2-gen0-control torn_enum | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=5 Z2-gen0-control torn_half | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=8 Z2-gen0-crash0 zero | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=8 Z2-gen0-crash0 torn_enum | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=8 Z2-gen0-crash0 torn_half | n=5 | G3/R2 G4/R1 NA5 G4/R1 G4/R1 G5 G2/NA3 | G5 G5 G2/NA3
S=8 Z2-gen0-crash1 zero | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=8 Z2-gen0-crash1 torn_enum | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=8 Z2-gen0-crash1 torn_half | n=6 | G4/R2 G5/R1 NA6 G5/R1 G5/R1 G6 G2/NA4 | G6 G6 G2/NA4
S=8 Z2-gen0-control zero | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=8 Z2-gen0-control torn_enum | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
S=8 Z2-gen0-control torn_half | n=6 | G2/R4 G5/R1 NA6 G6 G6 G6 G2/NA4 | G6 G6 G2/NA4
```

四句：

- 分不分辨臂：分。丁甲红（误红），丁乙不适用，丙红（已知红那一类，范围多一形），我提的「丁甲加第 0 代根」绿。
- 被判的系统看不看得到：看不到那一槽住过的是第 0 代根还是 txg 1 那一条（槽写坏了）；看得到的是「下一槽是某个区域的槽 0」。所以改法只能按槽认，不能按 txg 认。
- 满足的分句：Z2「环里同时有几个洞」那一类之外、清单没列的形（第一圈里的洞）；也落在 Z1「丁甲的第 2 种要改哪一句」：正文字面「在更早任一圈住过的根」照字面包括第 0 代根，前两轮两腿的实现都没包括它，第三轮要把这一句写明。
- 改法在这一格上中不中：丁甲的第 2 种另补 txg 0（下一槽是某个区域的槽 0 时）——这一格不再中（量的，`ding_jia_gen0` 那一列全绿）。代价是在下一槽是槽 0 的写坏镜像上多收 t = F（见 Z2-戊 的 p 扫描），只在我的模型上量过、被攻过零轮。

崩溃点重放的枚举走不到这一态（根槽写一个扇区，第二轮两腿已量），这一形只在写失败的给定状态与故障注入里出现，与题面那一形同一处。

顺带看到、不归这一轮的（推的）：写者那一侧同一个口子。挂载那一刻读坏的槽钉在「它可能住过的最旧 txg」，代码按落点公式在 [0, min(3S, first_txg)) 里找最小的 txg（`crates/singlefs-core/src/mount.rs:2908`：「for txg in 0..first_txg.0.min(ring_length_in_txgs) {」），区域 1 槽 0 得 1，不得 0；钉槽 p 的编码也是 3·槽 + 区域（`crates/singlefs-checker/src/walk.rs:8615`：「0 到 3S 之内落点公式是 (txg mod R, txg div R)」），表达不了 0。第 0 代根留在区域 1 槽 0、而第二次挂载那一刻那一槽读坏时，钉在 1 而不是 0；第 0 代根引用的单元有没有释放代 ≤ 1 的，没查、没量。

### Z2-乙 环里同时有几个洞——没打中

两种：连着两次挂载都崩在写行根槽之前（洞挨着，S = 4 时洞在 txg 15、16 那两槽，里面是 3、4），与隔三版各崩一次（洞隔开）。洞之后的实例写过两圈，每个 N 都试一次「下一次覆盖写的根槽写坏」。丁甲在所有写坏镜像上绿（第 2 种的 k 取到 2 就补得到洞里那条最旧根），丁乙不适用，今天在下一槽正好是最旧那条根时红（`out/tally.txt` 的 `Z2-holes-*` 行，原样见下）。几个洞叠在一起不改变「下一槽的旧内容 ≡ N+1 (mod 3S)」，所以与一个洞同形。

```
key | n | I-7.18 today jia yi_literal ding_jia_same_instance ding_jia ding_jia_gen0 ding_yi | I-7.19 today ding_yi ding_yi_literal
S=4 Z2-holes-adjacent zero | n=24 | G7/R17 G22/R2 NA24 G24 G24 G24 NA24 | G24 G24 NA24
S=4 Z2-holes-adjacent torn_enum | n=24 | G7/R17 G22/R2 NA24 G24 G24 G24 NA24 | G24 G24 NA24
S=4 Z2-holes-adjacent torn_half | n=24 | G7/R17 G22/R2 NA24 G24 G24 G24 NA24 | G24 G24 NA24
S=4 Z2-holes-apart zero | n=24 | G8/R16 G22/R2 NA24 G24 G24 G24 NA24 | G24 G24 NA24
S=4 Z2-holes-apart torn_enum | n=24 | G8/R16 G22/R2 NA24 G24 G24 G24 NA24 | G24 G24 NA24
S=4 Z2-holes-apart torn_half | n=24 | G8/R16 G22/R2 NA24 G24 G24 G24 NA24 | G24 G24 NA24
```

### Z2-丙 洞与抬 F 叠在一起——没打中（只量了卸载抬 F）

洞之后的实例写两版、正常卸载（推空发布把 F 抬到现行那一版），再挂、写过一圈多，每个 N 试一次写坏。卸载之后最新根带的 F 是 19（`floor_of_newest=19`），洞里那条 txg 3 还在环里。F 压过环里最旧有效根时 t = F，今天就绿；F 之下那几段今天红、丁甲绿、丁乙不适用（原样见下）。准入抬 F（`raising_the_floor_after_holes_at_the_oldest_end_of_the_one_revolution_window.rs` 那一类）没造。

```
key | n | I-7.18 today jia yi_literal ding_jia_same_instance ding_jia ding_jia_gen0 ding_yi | I-7.19 today ding_yi ding_yi_literal
S=4 Z2-hole-floor zero | n=14 | G5/R9 G14 NA14 G14 G14 G14 NA14 | G14 G14 NA14
S=4 Z2-hole-floor torn_enum | n=14 | G5/R9 G14 NA14 G14 G14 G14 NA14 | G14 G14 NA14
S=4 Z2-hole-floor torn_half | n=14 | G5/R9 G14 NA14 G14 G14 G14 NA14 | G14 G14 NA14
```

### Z2-丁 多实例交替——没打中

每个实例写 1、2、3 版就崩（不卸载），下一个实例接着挂，转过两圈后环里住着实例 4 到 7 的根（`ring=[(17, 4), (18, 5), …, (28, 7)]`）；之后每一步试一次覆盖写写坏，每隔三轮试一次下一次挂载的写行、第一次暖机写坏。

```
key | n | I-7.18 today jia yi_literal ding_jia_same_instance ding_jia ding_jia_gen0 ding_yi | I-7.19 today ding_yi ding_yi_literal
S=4 Z2-alternating-overwrite zero | n=12 | R12 G12 NA12 G12 G12 G12 NA12 | G12 G12 NA12
S=4 Z2-alternating-overwrite torn_enum | n=12 | R12 G12 NA12 G12 G12 G12 NA12 | G12 G12 NA12
S=4 Z2-alternating-overwrite torn_half | n=12 | R12 G12 NA12 G12 G12 G12 NA12 | G12 G12 NA12
S=4 Z2-alternating-mount-root0 zero | n=4 | R4 G4 NA4 R4 G4 G4 NA4 | G4 G4 NA4
S=4 Z2-alternating-mount-root0 torn_enum | n=4 | R4 G4 NA4 R4 G4 G4 NA4 | G4 G4 NA4
S=4 Z2-alternating-mount-root0 torn_half | n=4 | R4 G4 NA4 R4 G4 G4 NA4 | G4 G4 NA4
S=4 Z2-alternating-mount-root1 zero | n=4 | R4 G4 NA4 G4 G4 G4 NA4 | G4 G4 NA4
S=4 Z2-alternating-mount-root1 torn_enum | n=4 | R4 G4 NA4 G4 G4 G4 NA4 | G4 G4 NA4
S=4 Z2-alternating-mount-root1 torn_half | n=4 | R4 G4 NA4 G4 G4 G4 NA4 | G4 G4 NA4
```

### Z2-戊 p 的其余取值

满环无洞（S = 4，N = 2·3S+1 = 25，单一实例），最新根的 p 换成 0..11 每一个与 255，t 取真值、按这个 p 在健康镜像上现算的值、F 三种（去重），健康镜像与写坏（写 0、枚举撕法）镜像各判一遍。这一点 N+1 = 26 落在区域 2 槽 0，正好也量「丁甲加第 0 代根」的代价。

`out/p-scan-summary.txt` 摘行（`p=2`、`p=255`、`p=0` 那几行与末尾计数，整份 48 行；每格写「I-7.18/I-7.19」，丁乙那一列是照字面那一读法）：

```
p=0 t=0 healthy_today=G/R bad=True zero:today=G/R,ding_jia=G/R,ding_jia_gen0=G/R,ding_yi=NA/NA torn_enum:today=G/R,ding_jia=G/R,ding_jia_gen0=G/R,ding_yi=NA/NA
p=0 t=14 healthy_today=R/R bad=True zero:today=R/R,ding_jia=R/R,ding_jia_gen0=R/R,ding_yi=NA/NA torn_enum:today=R/R,ding_jia=R/R,ding_jia_gen0=R/R,ding_yi=NA/NA
p=2 t=0 healthy_today=R/R bad=True zero:today=R/G,ding_jia=R/G,ding_jia_gen0=G/G,ding_yi=NA/NA torn_enum:today=R/G,ding_jia=R/G,ding_jia_gen0=G/G,ding_yi=NA/NA
p=2 t=2 healthy_today=G/R bad=True zero:today=G/G,ding_jia=G/G,ding_jia_gen0=G/G,ding_yi=NA/NA torn_enum:today=G/G,ding_jia=G/G,ding_jia_gen0=G/G,ding_yi=NA/NA
p=2 t=14 healthy_today=R/R bad=True zero:today=R/G,ding_jia=R/G,ding_jia_gen0=R/G,ding_yi=NA/NA torn_enum:today=R/G,ding_jia=R/G,ding_jia_gen0=R/G,ding_yi=NA/NA
p=255 t=0 healthy_today=R/G bad=True zero:today=R/G,ding_jia=R/G,ding_jia_gen0=G/G,ding_yi=NA/NA torn_enum:today=R/G,ding_jia=R/G,ding_jia_gen0=G/G,ding_yi=NA/NA
p=255 t=14 healthy_today=G/G bad=False zero:today=R/G,ding_jia=G/G,ding_jia_gen0=G/G,ding_yi=NA/NA torn_enum:today=R/G,ding_jia=G/G,ding_jia_gen0=G/G,ding_yi=NA/NA
combinations 37 bad_on_the_healthy_image 36
bad_writers_passed_on_the_written_bad_image image=zero candidate=ding_jia count=1
bad_writers_passed_on_the_written_bad_image image=zero candidate=ding_jia_gen0 count=3
bad_writers_passed_on_the_written_bad_image image=zero candidate=ding_yi count=36
bad_writers_passed_on_the_written_bad_image image=zero candidate=ding_yi_keeping_i719 count=4
bad_writers_passed_on_the_written_bad_image image=zero candidate=today count=1
```

读法：

- p 写成环里任一个槽、t 跟着钉算（健康镜像上 I-7.18 绿）的坏写者：这一池单一实例，钉着的槽都住着本实例的根，I-7.19 在健康镜像与写坏镜像上都红；丁乙照字面在写坏镜像上把这一条也关掉，36 份坏写者全放过。
- p 钉着下一槽（区域 2 槽 0，p = 2）、t = 2：健康镜像上 I-7.19 红；下一槽写坏之后那一槽没有根，I-7.19 无从判，I-7.18 与按钉现算相等，今天与丁甲都放过——今天自己的盲点，C863「凭空钉一个读得出的槽」那一类在写坏镜像上的样子，不分辨臂。
- t = F（= 0）、p = 255 或 2：「t 恒取 F」一类。今天、丁甲判红；丁甲加第 0 代根收下（下一槽是槽 0 时多补的 txg 0 正好让 t = F 对得上）——这是那一改法的代价，量的。

### Z2-己 `WithTreesAndNoFile` 之后的写行——没量（推的）

`crates/singlefs-core/src/mount.rs:1153`：「写行与暖机照带文件的一版那样做」，写行带单元、建账，与第二轮格 G（另一实例的写行写坏）同形；前提取任一实例之后格 G 不误红（第二轮两腿量过），推这一形同。没造删文件把一版走到树表有树、文件集合为空的那一段历史。

### Z2-庚 GPU 交接的写法与对拍要加的盘（推的，没写、没编、没过 naga）

GPU 走读内核的求等在 `crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4672`：「judge(INVARIANT_I_7_18, eq64(reclaim_threshold, recomputed_threshold));」。丁甲、丁乙在这一侧要三样东西：

1. 「下一槽读不出自证过的根」：同一个函数里已经逐条走根表（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:4656`：「if (w_root_abandoned(r.instance, r.txg)) {」那一圈），顺手比「这条根的 txg 落到的槽 = N+1 落到的槽」即可，不用另开缓冲。槽号要一次 64 位除以 3S；第 0 代根住在区域 1、2 的槽 0 时它的 txg（0）不落在那一槽的公式上，比槽号时要按根表记着的区域与槽比，不能按 txg 反推（Z2-甲那一形）。
2. 「journal 里有任一实例点名 N+1 的记录」：记录表是逐盘清零重填的草稿区（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:389`：「record_count = 0u;」），判 I-7.18 时只剩最后填的那一块盘，不能直接查。最省的写法是在这个函数里照逐盘那一圈的过滤（`crates/singlefs-checker-tier/src/crash_judge_checker.wgsl:392`：「if (location_word(l, 0u) != device || location_word(l, 4u) != LOCATION_JOURNAL_RECORD) {」）把位置表再走一遍，不分盘，自证过的记录读 checkpoint_txg 与 N+1 比；O(位置数)，不写草稿区。
3. 第 2 种现算：不能照 CPU 原型那样按 k 逐圈循环（N 可以很大）。按现算的式子 t* = max(F, min(环里最旧有效根, 钉值, X))，X 取遍 {N+1−k·3S, k ≥ 1}（加第 0 代根那一版再加 X = 0），收下的 t 只有三种：今天的现算；t > F、t 低于今天的现算、t ≡ N+1 (mod 3S)、t ≤ N+1−3S；t = F 且 (N+1) mod 3S ≤ F（加第 0 代根那一版：下一槽是某区域的槽 0 时 t = F 也收）。常数步。

丁乙在 GPU 上照「不判」写就是提前 return（今天两类「不判」都这么写，I-7.19 也一起不判）；要照 Z1-甲 的改法保住 I-7.19，就得把 I-7.19 那一段挪到 return 之前或另起一个函数（与我第一轮 CPU 开关里 `judge_the_pinned_ring_slot_only` 同形）。

对拍要加的盘（CPU 与 GPU 判同一张，逐不变量比）：

| 盘 | 为什么 |
|---|---|
| Z2-甲 第 0 代根（区域 1 槽 0、区域 2 槽 0 各一张，写 0 与枚举撕法） | 丁甲按 txg 反推槽、或第 2 种漏了 X = 0，CPU 与 GPU 会各错一边 |
| 第一轮格 A、B（洞落在下一槽，k = 2） | 第 2 种的 k ≥ 2 |
| 第二轮格 G（另一实例写行写坏） | 前提要任一实例 |
| 第一轮格 F 的 N 做完那张，连同 t 写成 F、p 写 255 | 前提不成立时照今天判 |
| Z1-甲 不解钉的坏写者（写坏那张） | 丁乙要不要保住 I-7.19 |
| 前提落空一张：满环最旧根事后读坏（第二轮 BR） | 没有 N+1 的记录时照今天红 |
| N+1 = 3S 与 N+1 = 3S+1 两张 | `>=` 那条边 |

## Z3 前两轮报出的数能不能核

两条路，互不共用原腿的代码：

- 路一（重量）：我这一轮另写的探针重造历史、重判（历史搭法用仓里 `crates/singlefs-harness/tests/common_root_ring_holes/mod.rs` 的拷贝；第一轮我那份探针直接用的也是这份模块，所以对第一轮的数，造历史那一半不独立，独立的只有判法与计数；对第二轮辩方的数，造历史与判法都不共用；候选开关沿用第一、二轮我的那几道，丁乙换成第三轮正文那一版），另在探针里按根槽字节自己算一遍今天与丁甲（`independent_verdicts`，不经 walk.rs 的开关），与 walk.rs 那两列逐张比。全部输出里两路不一致 0 张（`out/tally.txt` 末行）。
- 路二（重数）：第二轮辩方交的 `out/raw/`（4095 行 ROW）用我另写的脚本解析（`tools/recount_keys.py`、`tools/recount_extra.py`、`tools/compare_runs.py`），不用辩方的 `summary.py`。

| 被核的数 | 出处 | 核法 | 结果 | 量的还是推的 |
|---|---|---|---|---|
| 第一轮「格 B 36/36」（甲在合法写坏镜像上红） | 第一轮我的报告格 B | 路一：L = 3S..6S−1 一圈 12 段，每段写 0、枚举撕法、对半撕三张 | 路一甲 36/36 红（写 0、枚举撕法、对半撕各 12/12），今天也 36/36 红，丁甲 36/36 绿（原样见下） | 量的 |
| 第一轮「格 F 11/11、12/12」（乙放过两类坏写者） | 第一轮我的报告格 F | 路一：L 一圈 12 段，N 做完那张上合法、t 写成 F、p 写 255 | 乙字面 36 张全不适用；t 写成 F 那 12 张里 1 张与合法逐字节相同（今天也绿），所以是 11 张真坏 + 12 张真坏，与原报的口径一致 | 量的 |
| 第二轮「格 G 3/3」（同实例读法误红） | 第二轮辩方 #8 | 路一：S = 4、5、8、16 各取 N = 3S+2、6S+1、6S+4 三段，写 0、枚举撕法、对半撕；路二：重数辩方 raw | 路一每个 S 三种写坏各 3/3 红；路二：辩方的「3/3、3/3、2/2、2/2」只数了写 0 那一种，三种都数是 9、9、6、6，全红 | 量的 |
| 第二轮「BR 14 张、50 张」 | 第二轮辩方 #14 | 路一：S = 4、5、8、16，N 从 3S−1 到 6S；路二重数 | 路一 S = 4、5、8、16 为 14、17、26、50 张，今天、丁甲、丁乙全红，甲（`>`）各在 N+1 = 3S 那一张红（原样见下）；路二 S = 4、5、8、16 为 14、17、26、50 张，今天与丁甲全红 | 量的 |
| 第二轮「33 个键分类不变」 | 第二轮辩方 #1、W4 | 路二 | 33 是过了 `summary.py` 那道筛（只留 zero、healthy_n、before_root 等几类镜像名）的键数；四个 S 都有的只 29 个（W 格只在 S = 4，F 的 healthy_n 只在 S = 4、5）；不筛的全部 57 个键、逐候选比，在 S 之间变类的 0 个。结论（不变类）站得住，「33 个键在四个 S 之间」的说法宽了 | 量的（重数） |
| 第二轮「1890 张写坏镜像，红单只有 `["I-7.18"]` 1173 或 `[]` 717」 | 第二轮辩方 W4 | 路二 | 1890 = 1173 + 717，逐字相同 | 量的（重数） |
| 第二轮判决第六节的复跑差异（rows.diff 只差张数，来历归「较早一版探针」） | 第二轮判决第六节 | 路二：腿的 raw 与判决复跑那份（`~/.local/share/singlefs/work/m3-close-b16-verify/i718-r2-rerun/sonnet-work/out/`）逐键比 | 归类差异 0（核上）；张数不同 56 个键：44 个在 F 格（四个 S，复跑每处多 1 张，合扩过取样点的说法），**12 个在 S = 16 的 H1、H2：复跑少了**（569 → 557 行、539 → 504 行）——复跑那两件退出码 124，被腿自己的 225 秒封顶截断（复跑目录 `S16-cell_h1.exit`、`S16-cell_h2.exit` 都是 124，输出里没有 `test result` 行；腿交的 raw 里各有一行）。判决把差异全归到探针版本上，这 12 个键的来历不是它 | 量的（重数、读退出码） |


路一原样（`out/tally.txt` 里 Z3 的行）：

```
key | n | I-7.18 today jia yi_literal ding_jia_same_instance ding_jia ding_jia_gen0 ding_yi | I-7.19 today ding_yi ding_yi_literal
S=16 Z3-BR oldest_root_unreadable | n=50 | R50 G49/R1 NA50 R50 R50 R50 R50 | G50 G50 G50
S=4 Z3-BR oldest_root_unreadable | n=14 | R14 G13/R1 NA14 R14 R14 R14 R14 | G14 G14 G14
S=5 Z3-BR oldest_root_unreadable | n=17 | R17 G16/R1 NA17 R17 R17 R17 R17 | G17 G17 G17
S=8 Z3-BR oldest_root_unreadable | n=26 | R26 G25/R1 NA26 R26 R26 R26 R26 | G26 G26 G26
S=4 Z3-B before_root | n=12 | G12 G12 G12 G12 G12 G12 G12 | G12 G12 G12
S=4 Z3-B zero | n=12 | R12 R12 NA12 G12 G12 G12 NA12 | G12 G12 NA12
S=4 Z3-B torn_enum | n=12 | R12 R12 NA12 G12 G12 G12 NA12 | G12 G12 NA12
S=4 Z3-B torn_half | n=12 | R12 R12 NA12 G12 G12 G12 NA12 | G12 G12 NA12
S=4 Z3-F-legit healthy_n | n=12 | G12 G12 NA12 G12 G12 G12 G12 | G12 G12 G12
S=4 Z3-F-bad_t_as_floor healthy_n | n=12 | G1/R11 G1/R11 NA12 G1/R11 G1/R11 G1/R11 G1/R11 | G12 G12 G12
S=4 Z3-F-bad_p_255 healthy_n | n=12 | R12 R12 NA12 R12 R12 R12 R12 | G12 G12 G12
S=16 Z3-G before_root | n=3 | G3 G3 G3 G3 G3 G3 G3 | G3 G3 G3
S=16 Z3-G zero | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=16 Z3-G torn_enum | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=16 Z3-G torn_half | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=4 Z3-G before_root | n=3 | G3 G3 G3 G3 G3 G3 G3 | G3 G3 G3
S=4 Z3-G zero | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=4 Z3-G torn_enum | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=4 Z3-G torn_half | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=5 Z3-G before_root | n=3 | G3 G3 G3 G3 G3 G3 G3 | G3 G3 G3
S=5 Z3-G zero | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=5 Z3-G torn_enum | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=5 Z3-G torn_half | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=8 Z3-G before_root | n=3 | G3 G3 G3 G3 G3 G3 G3 | G3 G3 G3
S=8 Z3-G zero | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=8 Z3-G torn_enum | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
S=8 Z3-G torn_half | n=3 | R3 G3 NA3 R3 G3 G3 NA3 | G3 G3 NA3
```

路二原样（`out/recount-keys-sonnet-r2.txt` 整份、`out/recount-extra-sonnet-r2.txt` 整份、`out/compare-sonnet-r2-leg-vs-verdict-rerun.txt` 头两行与末三行，整份 59 行）：

```
keys_per_S {4: 57, 5: 51, 8: 50, 16: 50} keys_total 57
keys_matching_the_summary_filter 33
keys_present_at_S_4_5_8_16 50 of_which_matching_summary_filter 29
key_candidate_pairs_changing_class_across_S 0
total_ROW_lines 4095
keys_not_at_all_four_S [(('F', 'healthy_n', 'legit'), [4, 5]), (('W', 'before_record', 'legit'), [4]), (('W', 'before_root', 'legit'), [4]), (('W', 'healthy_n', 'legit'), [4]), (('W', 'torn_enum', 'legit'), [4]), (('W', 'torn_half', 'legit'), [4]), (('W', 'zero', 'legit'), [4])]
written_bad_images 1890 {'[]': 717, '["I-7.18"]': 1173}
G S=4 {('torn_enum', 'ding_jia_same_instance', 0): 3, ('torn_half', 'ding_jia_same_instance', 0): 3, ('zero', 'ding_jia_same_instance', 0): 3}
G S=5 {('torn_enum', 'ding_jia_same_instance', 0): 3, ('torn_half', 'ding_jia_same_instance', 0): 3, ('zero', 'ding_jia_same_instance', 0): 3}
G S=8 {('torn_enum', 'ding_jia_same_instance', 0): 2, ('torn_half', 'ding_jia_same_instance', 0): 2, ('zero', 'ding_jia_same_instance', 0): 2}
G S=16 {('torn_enum', 'ding_jia_same_instance', 0): 2, ('torn_half', 'ding_jia_same_instance', 0): 2, ('zero', 'ding_jia_same_instance', 0): 2}
BR-oldest S=4 {('any_lap', 1): 14, ('ding_jia', 0): 14, ('jia_seed', 1): 14, ('today', 0): 14}
BR-oldest S=5 {('any_lap', 1): 17, ('ding_jia', 0): 17, ('jia_seed', 1): 17, ('today', 0): 17}
BR-oldest S=8 {('any_lap', 1): 26, ('ding_jia', 0): 26, ('jia_seed', 1): 26, ('today', 0): 26}
BR-oldest S=16 {('any_lap', 1): 50, ('ding_jia', 0): 50, ('jia_seed', 1): 50, ('today', 0): 50}
keys leg 208 rerun 208 only_leg 0 only_rerun 0
count (4, 'F', 'before_record', 'legit') 3 4
count (16, 'H2', 'zero', 'legit') 90 84
keys_with_count_difference 56 key_column_class_differences 0
rows leg 4095 rerun 4092
```

## 没打中的形状

- 环里几个洞（挨着、隔开），S = 4，洞之后各扫两圈 N，每个 N 三种写坏：丁甲 0 张误红（量的）。
- 洞与卸载抬 F 叠加，S = 4，扫一圈多：丁甲 0 张误红（量的）；准入抬 F 没造。
- 多实例交替（每个实例 1 到 3 版就崩），S = 4，覆盖写写坏扫一圈：丁甲 0 张误红（量的）。
- 第二轮丁甲的前提（任一实例）在 G 形四种几何上不误红（量的，Z3 那一行）。
- `WithTreesAndNoFile` 之后的写行：没造，推与 G 同形。
- 取样范围：几何 S = 4（全部格）、5、8（第 0 代根、G、BR）、16（G、BR）；第 0 代根两种崩法各扫 N = 3S−3 到洞那一槽之后一版；环洞两种各扫两圈；卸载抬 F 一圈加两版；多实例交替一圈；p 扫描 13 个 p × 至多 3 个 t × 3 张；Z3 的 B、F 各一圈 12 段，G 每个 S 三段，BR 每个 S 从 3S−1 到 6S。每张写坏取写 0、照枚举撕法撕、对半撕三种，另带根槽写之前那张。

## 这条腿自己的限度

- 崩溃状态没有照定义第 3b 步整段展开。新造的几条流只算了全量状态数（`out/counts-S4.txt` 原样三行，见「复跑命令」一节之后）：第 0 代根那一形最后那次覆盖写 1048588 个状态，超过约 10⁶，按定义不跑；第一次挂载、崩后再挂两条流 41、308 个状态，没展开判——第二轮两腿已量过枚举走不到「下一槽读不出」这一态（根槽写一个扇区），展开这两条只能判出今天与各候选一样，判不了被判的那一态。这一条照实记为与定义不符。本腿原型跑的全量合计 0 个状态。
- 坏写者是改根槽字节造的，记录与账没跟着改：Z1-甲的坏写者在 N 做完那张与写坏那几张上 I-3.1、I-8.10 跟着红（`today_red=["I-3.1", "I-8.10", "I-7.19"]`），是副产物；判臂只看 I-7.18、I-7.19 两列。「真坏写者在写坏镜像上一条都不红」是推的。
- 「有没有 txg = N+1 的记录」仍是第二轮那一份写法（walk.rs 调用处现扫逐盘记录表、经线程局部变量传进判里），独立判法也借用这一个布尔，没另写一份 journal 解析。
- 独立判法把环里每条自证过的根都当有效根，只在没有被抛弃根的历史上与 walk.rs 等价；这几段历史里两者 0 张不一致。
- 丁乙两种读法里，「连 I-7.19 一起不判」那一读法没装进 walk.rs，是探针把「I-7.18 报不适用」的那几张改记成 I-7.19 不适用；这与今天 walk.rs 两类「不判」的写法同形（读代码），没另编一版。
- GPU 一侧一行都没写，Z2-己（GPU）全是推的。
- 资源：本机这段时间 loadavg 54–94（别的会话的变异整表在跑）。我的件都是单线程、`nice -n 19`，经内存包装（每件 2G，有几件在包装里排队 2 到 7 分钟）；没用第二台。
- 量的数全在副本上，不是入库装置上的数。

## 没做什么

- 没跑 checker 档（给定状态用例、崩溃点重放、GPU 对拍）；没写 GPU 一侧的任何候选，没过 naga。
- 没造 `WithTreesAndNoFile` 之后的写行、准入抬 F 与洞叠加、第 0 代根在第一圈之后又连着出洞那一形（Z2-甲 改法代价那一句里推的那一形）。
- 没把新流的崩溃状态整段展开（「这条腿自己的限度」第一条）。
- 第 0 代根那一形只量了 I-7.18、I-7.19；写者那一侧「钉在可能住过的最旧 txg」对第 0 代根算成 1 的后果（Z2-甲末段）没量。
- 不判辩方那几格，不替主 agent 采纳；副本上的数不算入库装置上的数。

## 要主 agent 跑的命令

1. 复跑（约 20 分钟，放后台）：上面「复跑命令与文件 sha256」那一条，工作目录随意；看末尾 `name=… same_as=…` 各行。
2. 核一件我推出来、没能跑的事（checker 档，归主 agent 定跑不跑）：`crates/singlefs-harness/tests/common_images_for_the_checker_tier/rollback_by_a_forward_publish.rs` 里两处「恢复之前」的期望写的是只红 I-3.1（第 1020、1360 行），而我量的普通覆盖写写坏镜像上今天红的是 I-7.18、I-3.1 不红（本报告各写坏镜像的 `today_red=["I-7.18"]`；第一轮正推也量过）。照这个推，checker 档 `crates/singlefs-checker-tier/tests/crash_given_states_of_the_images_harness_scenarios_hand_out.rs` 里 `plain_overwrite_whose_root_slot_write_zeroes_the_oldest_ring_root_leaves_the_pre_recovery_image_red_only_by_what_that_root_alone_referenced` 与 `the_failed_root_slot_write_of_a_rollback_to_the_oldest_ring_root_{zeroed,torn_after_the_first_half}_is_resent_in_process_or_replayed_after_a_crash` 三条今天可能是红的（推的；那两个场景的几何与我的池不同，没量）。三个形态里丙要改这两处期望，丁甲、丁乙也要改（它们在这一态上不再红 I-7.18）。
3. 核引文：`python3 scripts/agents/cite-check.py research/prompts/m3-close-i718-oldslot-r3-opus-output.md`。

## 清理

交回前删掉的（删之前 `du -sh`）：仓副本 `repo` 37M、编译目录 `target` 1.6G 与 `target2` 1.6G（为 z2-alternating 那一件的修正另编了一次），都在 `/home/fy5090/.local/share/singlefs/work/m3-close-i718-oldslot-r3-opus/` 下；那里只留原始输出、日志、退出码文件、草稿与几个辅助脚本（772K），其中 `out/z2-alternating-S4-before-the-mount-fix.txt` 是修正之前那一份，没进模型目录。

## 引文核对

`python3 scripts/agents/cite-check.py research/prompts/m3-close-i718-oldslot-r3-opus-output.md` 末行（交回前现跑）：

```
  ✓ 核了 20 处引文、0 处裸行号，对上 20 处，没判 0 处（其中裸行号没有引文可比 0 处；逐处列在上面）
```
