# 写回报告：batch-2026-10-21 阶段同步 c-later 7 行（写回员 e）

## 结论
7 行加英日同句全部写完（19 处替换、13 个文件），变更史加 4 项。分项进度、状态未翻；不变量清单、decisions.md 未动；未跑 decision-items-sync（没有索引行、状态、计数变动）。

## 现查（以 HEAD 为准）
- C668、C296、C403 在已还清表：`git show HEAD:.claude/kb/checks-owed.md | awk 'NR>=591 && /^\| C(668|296|403) /{print NR": "substr($0,1,40)}'` 输出 854: C403、860: C296、862: C668（「### 已还清」标题在 591 行）。
- C409：HEAD 的 checks-owed.md 294 行仍在欠账表，去向「挪出里程碑三，归 04-fourth-txn.md 八·3」；新串没写 C409，不受影响。
- `empty_segments`：`git show HEAD:crates/singlefs-core/src/allocator.rs | grep -n 'pub fn empty_segments' -A3` 输出 846 行起返回 `self.full_segments_without_an_unreleased_record`；工作区对 allocator.rs 的改动没碰这个函数。
- E184：HEAD 有 `.claude/kb/experiments/184-常规层与大负载层的代价计数.md`、产物 `research/results/e184-regular-layer-cost-counts-segment-one-2026-10-19.out`（950 行）；索引行状态「部分已跑」；该页「影响的决策」表 D13 已定项 9 一行关系是备料。

## 改了哪几处（文件:行 = 改后行号；改前、改后各一句）
- 13-验证路线.md:266（en:263，ja:264）：改前「无实验：……负载与挂钟都还没有，没有可量的量」；改后仍写「无实验」，改成「常规层、大负载层在门禁里的负载还没有定，E184（常规层与大负载层的代价计数） 第一段已量候选负载的规模与态数区间重叠，只备料；第二到四段（作废面、冷吞吐、两种落法的固定开销）未跑」。
- 13-验证路线.md:445（en:442，ja:443）已定项 17 欠块：去掉 C668。
- 19-块指针的结构与宽度预算.md:175（en、ja 同行）已定项 7 欠块：去掉 C296。
- 22-单元原子性怎么合成.md:392（en:391，ja:391）已定项 18 欠块：C403 的销账指针行改成「**欠**：无。」（en「None.」，ja「なし。」）。
- 03-空间分配.md:212（en、ja 同行）已定项 10 ①：改前「代码今天的 empty_segments 是占用口径、要改成记录口径」；改后「代码 empty_segments 已是记录口径，C372（全空聚簇段数两份定义相反） 的代码那一半已做」。
- 变更史 `.claude/kb/decisions-history.md`：D3、D13、D19 各新开 `### 2026-10-21` 块加一项；D22 并进已有的 `### 2026-10-21` 块末尾加一项。每项带「依据」「改前（git show HEAD:<中文路径>）」两行。

## 偏离派发提示的两处（要主 agent 知道）
1. D22 欠块不是整行删，而是留成「无」：format-evolution.md「决策正文长什么样」要求每个分项正文四块，欠块「没有写「无」」；整行删掉会让 kb-shape 的四块形态判红。指针里「销账后此行删」的意思按「指针删」办。
2. D13 的「无实验：」行没有照判决的「改后：- 实验：……」把前缀改成「实验：」：改成「实验」后 E184 成了依据段引的实验，doc-experiments 的 decision-links ③ 要求 E184 页回指这条分项且关系是支撑或推翻，而页里那一行是备料。保持「无实验：」，E184 写在理由里（门禁 ⑧ 说理由里点名的实验不算引）。若主 agent 要把 E184 升成依据，需另判撑不撑，并改 E184 页表。

## 剔出的行
无。第 1 行 D22:19（已定项 8 索引行进度）是「要人看」，未改，原样交主 agent：已定项 8 索引行写「进度：未实现」的两条理由（make_filesystem.rs 清零仍在一段里、没有作废屏障，fsid 仍由调用方给）现已在工作区实现（作废屏障与 make_filesystem_drawing_the_filesystem_identifier）；已定项 8 里是否还有别的没实现的部分（系统配置自愈闸、离线出路、取号两轮的合入后验证）判不了，要主 agent 现查后决定进度改不改；已定项 26 的未实现还有挂载时回写（C423）作理由，不受影响。

## 要主 agent 判的点
- kb-spec-check 对译文的冲突：译文里 `E184 (译名)` 按三语规则写半角括号译名，spec-check 的编号引用格要求「E184（中文简称）」，所以译文 10 条规格单列 `spec-translations.json`，该检查在这份上只红 2 条（13 en、13 ja 的 E184 没带中文简称），别的 8 条绿；中文与变更史 9 条的 `spec.json` 全绿。
- HEAD 里 checks-owed.md 的 C372 行仍写「仍欠代码 empty_segments 从占用口径改成记录口径」，与新写的 D3 句子不一致（工作区未提交的 checks-owed.md 已改成「代码 empty_segments 已改成记录口径」）。提交时让那份改动一起进即可，我没碰 checks-owed.md。
- 变更史「依据」引的 `research/prompts/batch-2026-10-21-sync.md` 与 `-judge-5/6.md` 目前只在工作区、不在 HEAD（派发提示指定要引），提交时要一起进。

## 命令与原样输出
- kb-spec-check（写之前）：中文与变更史 9 条：`  ✓ 规格 9 条：旧串各恰好 1 次，编号引用、指代、依据配对与变更史形态都过（查了 9 条）`；译文 10 条末几行：`  ✗ 第 1 条（.claude/kb/decisions/13-verification-route.en.md）：编号引用不对：E184（没带简称）→ …`、`  ✗ 第 2 条（…13-検証ルート.ja.md）：…同`、`  ✗ 规格 10 条里有 2 处会让门禁红（逐处列在上面）`。写完后重跑自然红（旧串已不在），不作结果。
- 写范围闸：13 个路径各退出码 0（`precheck.jsonl` 无检出）。
- `replace-batch.py --dry-run`：`✓ 19 处都命中得对（13 个文件），--dry-run 没写`
- `replace-batch.py`：`✓ 19 处替换，写了 13 个文件（改名换上新 inode），回读一致`，退出码 0。
- 变更史 `git diff` 写前写后对比（`history-diff-before.txt`、`history-diff-after.txt`）：新增的行只有 4 项（D3、D13、D19、D22），其余是行号位移。
- 三语检查：每个文件三份的 C、E 编号集合哈希一致；标题、行数偏移（中文 22 比译文多 1 行）与写前相同。
- 没跑 relabel-item.py（不翻状态）、没跑 decision-items-sync。

## 哈希（开工时 → 收尾）
```
开工时：
08aab6200515ee87d899cb4514e9528d20e65dbebf243aa335635d9c8bb45981  decisions/03-space-allocation.en.md
96bd1d62d3eb2ec8f0b8b58ed160284ed4c204f332589145117eb5e24d513155  decisions/03-領域割り当て.ja.md
5fa3dd948f6a51a6b5fd12969b28c6df04deb4baf024bf07ca3690f786c5c796  decisions/03-空间分配.md
353386d44bb7778558105e4bbd9b24eef4e7c234392875b77409e95d9af38b8f  decisions/13-verification-route.en.md
6e2bbb710ab361ce093e37f787ac5e954ebf80606061c355f98b7b5a671a6c3c  decisions/13-検証ルート.ja.md
ba57f7135f5fa9f03d268b2fc021f1eb4b1c1c8f1eab2e001b4c0610dd9ed158  decisions/13-验证路线.md
ff6aa88a1e5dc2f8b8d9d6de544c20f4234e42f0c778fc5036ad49fafddbade1  decisions/19-block-pointer-structure-width-budget.en.md
226b36b6f3383383b55454ed25b1a9d00e0bf50e4f260cc8ab6a7e5e7869cc4d  decisions/19-ブロックポインタ構造と幅予算.ja.md
85c20b626ca6c7a2953b64a6de2ff97f759327374b233d656f82f53039c73625  decisions/19-块指针的结构与宽度预算.md
9e5747d852b802e5976068318ab46039d4e88c60656fa619a23416e03c893f20  decisions/22-synthesizing-unit-atomicity.en.md
13708d3f3b7e7e5cf78c6b4859479cfb662a58ea38e3495a24241272403e139b  decisions/22-ユニット原子性の合成手法.ja.md
d22a7ca22c7cc0958b97ee48ffe7724703de6f05e1a156d41fc085fe10a83d94  decisions/22-单元原子性怎么合成.md
75cc435ad5423310cedb45a013466978592febc38812902a8b30da543ad8cd2b  decisions-history.md
7102be6513e299327ada6ed1dc5ae9fcf37bb2777e9f168edbf986bc648e4832  decisions.md

收尾（decisions.md 未改，哈希不变）：
d91c4ba1f2897056abb224eb9a1be35c2d2d9bb58b4852b2299f61c3a13cbee1  decisions/03-space-allocation.en.md
fc8cc2fbe557d4ea3c8fe0b5e13cfa9c7e8a14e88df0e54c80c096b172b5790a  decisions/03-領域割り当て.ja.md
442a096b12075656ddf63ec0a3955aafb6cb01cb3ceb25bfef746a9acc43633e  decisions/03-空间分配.md
88daceb4955d4d18618f77bbfb4f603f3593f0cff0855da5a7d692dcac34fa97  decisions/13-verification-route.en.md
0030db2aa30f8e70a4f562239d5a32f20a8d5e46f6b997e52af58dbf3f5ee132  decisions/13-検証ルート.ja.md
6de9993f76a38d88263893b92574707f35cf0f5845300501dba7b8a8e0059eb2  decisions/13-验证路线.md
47dc4014dd7f4197b84336385b4e788e5583b918d97fa2856830e434a33cf2bb  decisions/19-block-pointer-structure-width-budget.en.md
1ad6283f61695099e4814755bd873c493280d70928e6be57f67860111a0ca1bc  decisions/19-ブロックポインタ構造と幅予算.ja.md
52eb80ccc054a6feebe41a33051fcf771fccd0692d1abcfd75beb4e410c605e9  decisions/19-块指针的结构与宽度预算.md
5e43bf5cd50f6375b6092b2ebdafdcf7307f18ca877b69f773bf14bd201066c3  decisions/22-synthesizing-unit-atomicity.en.md
afe36dc0829d8d220016991aadb7ca60475d401dd1b64406c4cefd9964959309  decisions/22-ユニット原子性の合成手法.ja.md
40728119c3966d08eb6e16263fd5940e759335e6004529d5e1e91470dc7fd559  decisions/22-单元原子性怎么合成.md
ebcf845bd59f56aae114a24731b630dc12244489ee4fb1ec770498252250825c  decisions-history.md
7102be6513e299327ada6ed1dc5ae9fcf37bb2777e9f168edbf986bc648e4832  decisions.md
```
