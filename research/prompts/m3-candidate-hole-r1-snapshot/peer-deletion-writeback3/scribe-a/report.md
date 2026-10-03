# 写回3 乙之外的甲：E83/E102 第 2 次与 K 代回扫的决策侧写回（报告）

## 做了什么（29 处替换，14 个文件，一次 replace-batch）

- 规格 `/tmp/claude-1000/m3-writeback3/scribe-a/spec.json`（sha256 6969b708fa2274233eef373f82c9eea5dcda107b58bf240183280eefc9b55660），同内容 `spec.md`（sha256 f14cc59738c0e3987fefd6081ded70eb3e27721a9c34450177ea6987b412d2ce），生成脚本 `build2.py`（同目录）。
- `kb-spec-check.py` 原样末行（json 与 md 两种写法各一次）：`✓ 规格 29 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史快查都过（查了 29 条）`
- 写范围闸预检 14 个文件，退出码全 0。
- `replace-batch.py --dry-run`：`✓ 29 处都命中得对（14 个文件），--dry-run 没写`；实写：`✓ 29 处替换，写了 14 个文件（改名换上新 inode），回读一致`，退出码 0。
- 没跑 `doc-decisions.sh --write`（并行写回，归主 agent）；没跑 `relabel-item.py`（不翻状态）。

### 条目 1：D18（块里携带什么信息） 三条分项（4 处）
- 已定项 11 「数字」段：旧「E83（墓碑的粒度）/ E84（打包墓碑的钉住量） 仍按 91 字节头算（583），重跑欠在 C89（单元头收口的遗留重算） ⑤」→ 新「E84（打包墓碑的钉住量） 仍按 91 字节头算（583），重跑欠在 C89（单元头收口的遗留重算） ⑤（E83（墓碑的粒度） 的那一半已在第 2 次按记录区起点 136、记录 40 字节重跑）」。
- 已定项 11 依据末句：旧「码 3 头 107 下两个下游容量都不动。」→ 新「码 3 按记录区起点 136 算，两个下游容量 cap(136, 40) = 815、cap(136, 140) = 233 与 kb 值对得上（第 2 次，墓碑记录 40 字节）。」
- 已定项 24 依据：旧「- 无实验：数是三方腿的模型……还没在入库装置上量过。」→ 新三条：E83 第 2 次（复出第 2 条 815 与第 4 条 76.7 MiB）、E102 第 2 次（cap(136, 40) = 815）、墓碑树 / 谱系树的其余数仍是三方腿模型与副本原型上量的（保留原句内容，没因删「无实验」丢掉「还没在入库装置上量过」这条不知道）。
- 已定项 16 依据：E117 那条之后加「E102（单元类登记表与打包记录单元）：第 2 次复出 cap(136, 40) = ⌊32632 / 40⌋ = 815。」
- 实验页同批：E83 页「影响的决策」表已定项 11 行（仍备料，回看改成写明 E83 那一半已还）、已定项 24 行（备料 → 支撑，改了）；E102 页已定项 11 行（回看去掉「由主 agent 在同一次改动里改」，改成「已改写成按 136 算」）、已定项 24 行与已定项 16 行（备料 → 支撑，改了）。E83 页已定项 16 行没动（该分项依据没引 E83，仍备料）。

### 条目 2：K 代回扫，决策正文 7 处
D5 第 134 行（已定项 4 射程第 12 项）、D5 第 141 行（已定项 4 依据 E71 一条）、D8 第 175 行（已定项 6 水位行括注）、D16 第 169 行（已定项 6 骑手条款 4）、D16 第 224 行（已定项 9 定案）、D23 第 371 行（已定项 14）、D26 第 26 行（已定项 1）。旧串全部 `grep -cF` 恰好 1 次、新串全按回扫员建议新串（只给裸编号的补了简称），没有停的行。逐处旧串新串在 `spec.md` 里。

### 条目 3：K 代回扫，实验页回看格 7 行
E54 第 130 行、E71 第 152 / 153 / 156 行、E112 第 130 行、E113 第 139 行、E123 第 112 行；全部写成「2026-10-04 改了：……（上一次回看 2026-09-20：不受影响：原文）」，旧回看原样并进。

### 变更史
- D5 节新开 `### 2026-10-04` 块（在 `### 2026-10-03` 之上）写 K 代回扫全文（结论 / 依据 / 改前）；D8、D16、D23、D26 节各一行「见 D5（快照 / 空间记账机制） 节同日条目「…」」（D8、D16 加进已有 10-04 块的块首；D23、D26 新开块）。
- D18 节已有 10-04 块的块首加一条（E83 / E102 第 2 次写回）；条目里没写与标题同日的日期。
- 跑前跑后各 `git diff -- .claude/kb/decisions-history.md`（`hist-diff-before.txt` / `hist-diff-after.txt`）：新增的行只有上述六处，别的会话的条目原样。

## 要主 agent 判 / 做的点

1. **条目 4（回扫旧树值第 50 条，D16 第 57 行）没改。** 回扫员给的“建议新串”是“不机械替换：要么限定为「（当时的第一版几何）」，要么按今天的 c_max 重算。主 agent 判。”，是两种读法，没有一条可直接施加的新串；而且第 57 行现文已经写着“第一版几何上每次 8 个单元、两盘合计 21 次写调用、279040 字节”（并加了“树表 0 条的一版上不推”），“限定为第一版几何”那一半现文已有。请定：保持现状，还是按今天的 c_max 重算（回扫员说现状空发布写 4 个固定点单元 × 2 盘、layout/01 空发布行 `8+2+1+2`、13 次写）。定了再派一次定点替换。
2. **关系列只认 支撑 / 推翻 / 备料 / 不影响**（门禁 doc-experiments 第 1059 行附近），回扫员建议里的“关系列改成「支撑（只剩一半）」”“关系列由「不影响」改「改了」”写不进去。我的做法：E54 第 130 行关系仍写支撑、E71 第 153 行关系仍写不影响，把“只剩一半”“改了”放进回看格。若要换关系值（例如 E54 改成备料、E71 第 153 行改成备料），是判关系，归主 agent。
3. **E71 第 156 行回看写“改了”，它指的决策是 D22（单元原子性怎么合成）**，而我没改 D22 文件；门禁“改了没改”要求那份决策文件在“这次改动”里。D22 文件今天在 git 里已有暂存的改动（`M `），提交时是否算数请主 agent 在整轮门禁里核；不算的话，这一行回看要改成“不受影响：……”。
4. **规格外、没改的两处现状句**（也许要另写规格）：D18 已定项 11 定案里“打包记录类型”那一条（现第 300 行附近）仍写“E83（墓碑的粒度） 的 56 是「五元组 + 区间」的量级假设……按 56 算出的 583 是容量下界……E83（墓碑的粒度）/ E84（打包墓碑的钉住量）/ E102（单元类登记表与打包记录单元） 重跑”：E83、E102 已重跑，E84 还欠。E83 执行员报告第七节第 1 条已点名这一句，派发规格里没给，我没动。
5. **变更史依据里引的 K 代回扫报告原件在会话草稿目录、没入库**（门禁 evidence-in-repo 不许 kb 里把 /tmp 当依据，所以我写了“原件在会话草稿目录、尚未入库，入库后路径由主 agent 补”，没写 /tmp 路径）。主 agent 把 `m3-sweep-k-generations/report.md` 存进 `research/prompts/` 之后，要在 D5 节 10-04 条目的依据里补上路径。E83 / E102 两份执行员报告同样在草稿目录，我没引（引的是仓内产物 `research/results/…`、登记 `research/prompts/…prereg.md` 与问题单）。
6. 另外两条兄弟写回员乙的活（`checks-owed.md` C141、`invariants.md` I-9.11、`milestone/02-second-txn.md` 第 299 行，回扫表 A 的另 3 行）我没碰；E83 执行员第七节第 3 条（`checks-owed.md` 的 C89 ⑤ 里 E83 那一半）也在乙那边。
7. 其余交主 agent 的执行员遗留（与本批无关，没动）：E83 的 0 字节文件 `research/results/e83-tombstone-grain-2026-10-04.out` 要不要删；E84 装置写死 CAP=583 仍欠；`.claude/gate.d/experiment-constant-reading-lag.tsv` 第 7 行保留。
8. 第 3 步 `doc-decisions.sh --write` 归主 agent，在两份交回后跑一次。D18 的分项状态没翻，`decisions.md` 状态列不该变；D5 / D16 / D23 / D26 / D8 也没翻，索引行没动。

## 文件与哈希

开工时（`sha-start.txt`）：

```
e551f23379afbc7dcbc1f67aaadf320c64455ea7e67f3e03222e280ba5b515a7  .claude/kb/decisions/05-快照-空间记账机制.md
92d97707a0b7a36e7537411d49d6c4af28d65cba9121d1dd1da56f2a963efd6d  .claude/kb/decisions/08-核心索引结构.md
eeeb6563c7896b63cc0031580efe4b401d2c6a61f8286b79a97b9b4695d16619  .claude/kb/decisions/16-发布语义.md
79e254932e83b396fdfa84cd9c58752fb9423f3c9e567e246c8c95c0d2a41c86  .claude/kb/decisions/18-块里携带什么信息.md
b146db7ff8dd8b5ff8f5f78d817256d6c0622132f8492b3a812dd3bb861d4975  .claude/kb/decisions/23-journal的角色与格式.md
a23b036518c152362ccff310de4e92e156bb0fe4e8ff2c3597553f25464a8bc5  .claude/kb/decisions/26-后台整理与放置回收.md
161898fcf8cfdab31a3c791a01d5b48fc2041dd5415b698c8d9e222a3341e062  .claude/kb/decisions-history.md
24e02f837ca8b0cd7e197ab1e77b89652a4997c4cf4b99eee5c39ed1a5d26eef  .claude/kb/decisions.md
63d5ae6c310716a6ba0647e1827981d937d9c6a21d7097e6f0761fbbbfb48587  .claude/kb/experiments/54-记账的代要保留几代.md
a2f8dac59e1f02ef24d603edbfdebbd2c9db5db3ca2a44f288b4a9bf81262cff  .claude/kb/experiments/71-记账key的条目数与宽度.md
bf7a59ba227a5cb42c188e609eada1b406236ddb3a252832b7ca4d64adda49fb  .claude/kb/experiments/83-墓碑的粒度.md
816814f201d4d787fc0a2959bdfe22461b3f5fba274fb607590ae8a6f7b17ece  .claude/kb/experiments/102-单元类登记表与打包记录单元.md
f4684e773f58aad8b55a95a08039911b87a1596635b2fc1212a32fbb3ddd4f93  .claude/kb/experiments/112-旧写者遇到不认识的树.md
911d512d8e7efe6d644a1011cb5ec2f7ee1d00dc42246997734239cd9ae5d8ca  .claude/kb/experiments/113-不认识的树-补齐臂集.md
86e19716b3ac66e7eb2fb353b61d3c7b2076c3d6aaf29a235bb1471ec1cf0789  .claude/kb/experiments/123-K与回退深度的二选一各要付什么.md
```

收尾（`sha-end.txt`）：

```
d143c869edb8df9b32e27b8781686242aa4988bfd4cc16c339eb87e31b8cbf6e  .claude/kb/decisions/05-快照-空间记账机制.md
a8ac7fdd61389af0da4c720445c134ac03de32be0562c0875598a7a5f4e4415b  .claude/kb/decisions/08-核心索引结构.md
4b331275a2d6ec5643549c19c6ae9bb840895d7af21137cd579ce18f50f453bb  .claude/kb/decisions/16-发布语义.md
552987e4da48f7fe939523eaf852c54c7a7fdb84bc70c7c696b3c053b89a8130  .claude/kb/decisions/18-块里携带什么信息.md
f2b0473b5157b263c205c553bd1295da2082fa351201f95d249df59cc291d2c1  .claude/kb/decisions/23-journal的角色与格式.md
111e0b9e3872235357ae75bcd9f51a2c8e1784e85761940b3b4e328aa5de9a33  .claude/kb/decisions/26-后台整理与放置回收.md
acdf8605e2be122b9d00fb1250038dfcd11210d2ddb6805b99eb40bc4dd13b1e  .claude/kb/decisions-history.md
24e02f837ca8b0cd7e197ab1e77b89652a4997c4cf4b99eee5c39ed1a5d26eef  .claude/kb/decisions.md
5d1ceac8131db9665da49b168207487c8402915173c6afadb9c483af840ec120  .claude/kb/experiments/54-记账的代要保留几代.md
e1c94efcda2b64dd19e103bbd8cdffc9a325c5ca82baa755d2a2e85dbd1c9554  .claude/kb/experiments/71-记账key的条目数与宽度.md
930da3e11a2f1c446a7d7bd80dc21de7af13a68e4a131ffea1a015fd67a63b9a  .claude/kb/experiments/83-墓碑的粒度.md
e14ec44f87b3104e7d5bd9bdab23f03bd95a59c314904f06f400c975c4be7477  .claude/kb/experiments/102-单元类登记表与打包记录单元.md
8fdc72b979e2e8027b65979611666afcfa6129132f2f7c532b1e9c12a332ef55  .claude/kb/experiments/112-旧写者遇到不认识的树.md
ab96cf7c2efa402289e7708b6a1f93a08ee0f7d20ba097dca1823412f6954e6f  .claude/kb/experiments/113-不认识的树-补齐臂集.md
52fad5696248691ace24f1ad5622be0db4f96f6e2420db40601cc48f97a5acb5  .claude/kb/experiments/123-K与回退深度的二选一各要付什么.md
```

`.claude/kb/decisions.md` 没改（两次相同）。

## 没做什么

没判定案对不对；没写规格外的句子；没改 D16 第 57 行；没跑 `--write`、`relabel-item.py`、门禁与 lint；没提交；没碰兄弟写回员的文件。会推翻“这批写回对得上门禁”的现象：整轮 doc-experiments 的 decision-links 格或 doc-decisions 的 kb-shape 格在这 14 个文件上红（尤其上面第 2、3 点）。
