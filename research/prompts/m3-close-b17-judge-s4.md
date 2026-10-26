# 批 17 阶段同步第二段：逐行判 F12-F14（2026-10-26）

## 结论

- 候选表 `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-facts/candidates.tsv`（sha256 `2e508f2bf76f47f66a2fea7300f53e2e1ec2b7a4fa71fa539b38d50836a94ee4`）F12 45、F13 82、F14 88 行，共 215 行逐行判完；另加反向问 4 行（M1–M4）。
- 判定合计：事件句不改 137、不相干 40、要改 33、要人看 4、要补 1（F12：要改 13、要补 1、要人看 4、不相干 10、事件句不改 17；F13：要改 11、不相干 23、事件句不改 48；F14：要改 9、不相干 7、事件句不改 72）。
- 33 行「要改」里 26 行在结束那一版（提交对象 A）是旧的，暂存区或工作区已经是新写法（别的会话的改动、改名后的英日新文件、对象外按清单补的译文），只要随它们一起提交；真要写回员动手的只有 7 行，列在下一节甲类。
- 交实现员：0 行。
- 推翻条件：照下面「F14：两处哪一处是现状」的现查，若暂存区里 C656、C659 的挪表与实现被那个会话撤回（暂存区 checks-owed.md 第 874、875 行又回到欠表、或两份新用例文件从暂存区拿掉），F14 那 6 行欠账表的判定要反过来，改的应是里程碑三二·4 那一行；若某行判「事件句不改」的调度记录行其实是活的现状表（主 agent 维护着、要随事实改），那一行判错。

## 要主 agent 处理的

**甲类：结束那一版与工作区都还是旧的，要写回员改（7 行）**

1. `.claude/kb/checks-owed.md:583`、`.claude/kb/checks-owed.en.md:28`、`.claude/kb/checks-owed.ja.md:585`（C909 行）：问题列「D18 已定项 2 的「欠」写「无」」与前置列「立号之后…要接上这一笔」已不是真话。结束那一版 D18 中文第 64 行是「**欠**：C909（容器树层级 1 分隔 key 等于容器号只有 core 核）。」，工作区英日 D18 新文件第 64 行同。改后的句子在表里。
2. `.claude/kb/milestone/02-second-txn.md:576`、`.claude/kb/milestone/02-second-txn.en.md:563`、`.claude/kb/milestone/02-second-txn.ja.md:563`：「实做时定下的四处」里「inode 树根的 key 区间取分隔 key 的首尾、不取子树范围（跟 checker 今天的 check_index_node_keys 走，口径的账在 C478）」写着「今天」，C478 已还清，已不成立。
3. `crates/singlefs-checker/invariants/02-根环.md:24`（I-7.17）：「core 那一半随根环实现批，今天没写。」C743 的公式核已在暂存区（`crates/singlefs-core/src/root_ring.rs` 第 183 行 `root_sits_in_the_ring_slot_the_formula_gives`，暂存区 recovery.rs 3 处、mount.rs 2 处调它）。不变量清单的 `.md`，按文档行走写回员，不进交实现员。

**乙类：要人看（4 行，同一个问题）**

`.claude/kb/layout/03-third-txn.md:13`、`.en.md:13`、`.ja.md:14` 的 `deleted_inodes` 根状态格写「key 区间是格式级空白…key 区间归 C478」，`.claude/kb/experiments/172-删除写出的字节的干跑.md:162` 写「td、tt 头的层级与 key 区间只记值，对错归 C478 与 C713」。C478 已还清（还清行只列 inode 树与容器树），C713 在暂存区也已还清；`deleted_inodes` 根的 key 区间按 D18 已定项 2 定案原话「叶层码 2 节点上它就是首末两条条目的 key」字面已有条款、代码写的 [N, N] 同值，但池级 checker 读这个根只核层级、key 宽与条目宽（暂存区 `walk.rs` 的 `deleted_inodes_queue_without_judging`，只读了这一个函数，推的）。要定：这几句改指哪里，要不要另立一笔「`deleted_inodes` 根的 key 区间没有检查」。候选改法写在表里。

**丙类：结束那一版是旧的、暂存区或工作区已是新写法（26 行要改 + 1 行要补 + M1–M4）**

- 欠账表三语 C656、C659（F14 六行）与 C713（F12 三行）：暂存区已挪进已还清表。
- D16 三语两处 C743（中文在暂存区第 191、207 行；英日在改名后的新文件第 191、207 行）。
- D18 英日两处、D22 英日 C743 两处与 C659 一处：都在改名后的新文件里（`18-information-carried-in-a-block.en.md`、`18-ブロックに含める情報.ja.md`、`22-synthesizing-unit-atomicity.en.md`、`22-ユニット原子性の合成手法.ja.md`）；D22 中文欠块的 C659 在暂存区第 201 行已去掉。
- I-1.1 补孩子是码 3 容器那一半（`crates/singlefs-checker/invariants/01-块自描述.md:7`，暂存区已补）。
- M1–M4：D22 已定项 7、8 与 D16 已定项 6 的进度、`decisions.md` 里 D22 的计数，暂存区已是新值。
- 这一类不用再派写回员。但提交对象 A 若单独提交，这些行在那一版里是旧的；C478、C743、C656、C659 的代码与用例也都只在暂存区，HEAD 与对象 A 里都没有（现查见「命令与验证」），对象 A 的 kb 说它们已还清。两样要一起提交，或者按提交次序排好，由主 agent 定。

## F14：两处哪一处是现状

- 提交对象 A（`1e3079d7a…`）：`milestone/03-third-txn.md:71` 二·4 那一行写 C656「已照用户 2026-10-20 定案做完、已挪进已还清」、C659「已钉，已还清」；同一对象 `checks-owed.md` 第 426、428 行 C656、C659 还在欠表（「### 已还清」在第 599 行），写的是做之前的样子；D22 中文第 197 行欠块还有 C659。英日两份同样（en 第 440、442 行，ja 第 429、431 行，表头都在第 598 行）。
- 暂存区：`checks-owed.md` 里 C656、C659 在第 874、875 行，「### 已还清」在第 597 行（工作区第 883、884 行，表头第 606 行）；D22 中文第 201 行欠块没有 C659；实现在暂存区：`make_filesystem.rs` 里 `make_filesystem_drawing_the_filesystem_identifier` 9 处，两份新用例文件 `crates/singlefs-checker-tier/tests/crash_enumeration_make_filesystem_over_a_used_pool_stream.rs`（345 行）、`crates/singlefs-harness/tests/checker_a_pool_remade_over_an_old_pool_sharing_the_first_eight_identifier_bytes.rs`（240 行），`crates/mutations.tsv` 以 C656、C659 起头 10 行；HEAD 与对象 A 里这两份用例文件都不在。
- 判定：现状是暂存区那一份（两笔已还清）。里程碑三二·4 那一行与它一致，判「不相干」；对象 A 里欠账表 6 行与 D22 三语欠块 3 行是旧的，判「要改」，改法就是暂存区那一份，不用另写。

## 逐行判定

「暂存区」指 `git show :<路径>` 那一份；「工作区」指磁盘上的文件；「结束那一版」指提交对象 `1e3079d7a07304ceaf5a9de84f080853d61f52b5`。调度记录 `records/2026-10-18-里程碑三收尾与近期欠账.md` 的行号是结束那一版的编号。

### F12

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F12 | .claude/kb/checks-owed.en.md:28 | 要改 | 改后：问题列末句「the "**Owed**" of D18 (Information Carried in a Block) settled item 2 says "none"」改成「the "**Owed**" of D18 (Information Carried in a Block) settled item 2 points to this item since 2026-10-26」；前置列「once this has a number, the "**Owed**" of D18 (Information Carried in a Block) settled item 2 must point to it (item 2 of 「要主 agent 判的点」 in the report of 「写回甲」)」改成「the "**Owed**" of D18 (Information Carried in a Block) settled item 2 has pointed to it since 2026-10-26」。依据：结束那一版 D18 中文第 64 行「**欠**：C909（容器树层级 1 分隔 key 等于容器号只有 core 核）」。原话「only core checks that a level-1 separator key」；工作区这一行没改，要写回员改 |
| F12 | .claude/kb/checks-owed.en.md:475 | 要改 | 改后：C713 整行照暂存区那一行（已挪进 Settled 表，工作区第 904 行，Settled 在第 606 行，新行不再提 C478）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。结束那一版这一行还在欠表，问题列「key 区间那一格归 C478」指向已还清的 C478。原话「deletion线两个树根的层级」 |
| F12 | .claude/kb/checks-owed.en.md:606 | 不相干 | 已是新写法：这一行在 Settled 表（结束那一版第 598 行之后），原话「Is the Key Range of the Code-2 Header」那一行写写者、读者、池级 checker 与 GPU 判器都已改，与新事实一致 |
| F12 | .claude/kb/checks-owed.ja.md:464 | 要改 | 改后：C713 整行照暂存区那一行（已挪进解決済み表，工作区第 904 行，表头在第 606 行，新行不再提 C478）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「削除线两个树根的层级」所在行在结束那一版还在欠表，「key 区间那一格归 C478」指向已还清的 C478 |
| F12 | .claude/kb/checks-owed.ja.md:585 | 要改 | 改后：问題列末句「D18 (ブロックに含める情報) 確定項目 2 の「**未済**」は「なし」と書いている」改成「D18 (ブロックに含める情報) 確定項目 2 の「**未済**」は 2026-10-26 から本件を指す」；前置列「番号を付けた後、D18 (ブロックに含める情報) 確定項目 2 の「**未済**」をこの一件につなぐ必要がある（「写回甲」報告「要主 agent 判的点」第 2 条）」改成「D18 (ブロックに含める情報) 確定項目 2 の「**未済**」は 2026-10-26 にこの一件へつないだ」。原话「コンテナツリーの階層 1 の区切り key」；依据同 checks-owed.en.md:28 那一行；工作区没改 |
| F12 | .claude/kb/checks-owed.ja.md:869 | 不相干 | 已是新写法：这一行在解決済み表，原话「コード 2 ヘッダの key 区間はエントリキー区間か」那一行写书き手、読み手、プールレベル checker、GPU 都已改，与新事实一致 |
| F12 | .claude/kb/checks-owed.md:460 | 要改 | 改后：C713 整行照暂存区那一行（已挪进已还清表，暂存区第 895 行、工作区第 904 行，新行写「条款已写」、不再提 C478）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「删除线两个树根的层级没有条款」那一行在结束那一版还在欠表，「key 区间那一格归 C478」指向已还清的 C478 |
| F12 | .claude/kb/checks-owed.md:583 | 要改 | 改后：问题列「C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） 实现员报告「交主 agent 的设计判断」第 1 条点名这一处，没登号；D18（块里携带什么信息） 已定项 2 的「**欠**」写「无」」改成「C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） 实现员报告「交主 agent 的设计判断」第 1 条点名这一处，2026-10-25 登成本条；D18（块里携带什么信息） 已定项 2 的「**欠**」2026-10-26 起接上本条」；前置列「立号之后 D18（块里携带什么信息） 已定项 2 的「**欠**」要接上这一笔（写回甲报告「要主 agent 判的点」第 2 条）」改成「D18（块里携带什么信息） 已定项 2 的「**欠**」已于 2026-10-26 接上这一笔」。依据：结束那一版 D18 中文第 64 行。原话「容器树层级 1 分隔 key 等于容器号只有 core 核」；工作区没改 |
| F12 | .claude/kb/checks-owed.md:875 | 不相干 | 已是新写法：这一行在已还清表（结束那一版第 599 行之后），原话「写者、读者、池级 checker 与 GPU 判器都已按它改」与新事实一致 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.en.md:55 | 要改 | 改后：「Checker's check_index_node_keys ... checks the keys of the first and last entries; the root of parallel line 3 wrote out [2, 2] while subtree coverage was [2, 234]; both places must be modified per subtree coverage range (C478 ...)」换成中文第 55 行那几句的译文：Internal nodes whose children are code 3 containers (inode tree, tombstone tree, lineage tree, retirement tree) carry the subtree coverage range ... Implementation: writer, reader, pool-level checker ... 。工作区改名后的 18-information-carried-in-a-block.en.md 第 55 行已是这一段（对象外，按清单补），结束那一版的旧名文件随改名去掉，不用另写。原话「What is extrapolated is the judgment」 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.en.md:64 | 要改 | 改后：「**Owed checks**: C909 (only core checks that a level-1 separator key in a container tree equals the container number).」工作区 18-information-carried-in-a-block.en.md 第 64 行已是这一句（对象外，按清单补）。原话「**Owed checks**: C478」在结束那一版旧名文件里还在 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.ja.md:55 | 要改 | 改后：「checker の check_index_node_keys ... 両箇所ともサブツリーカバー区間に従って修正が必要 (C478 ...)」换成中文第 55 行的译文（子がコード 3 コンテナである内部ノード ... の区間はサブツリーカバー区間である ... 実装：書き手、読み手、プールレベル checker ...）。工作区 18-ブロックに含める情報.ja.md 第 55 行已是（对象外，按清单补）。原话「外挿されるのは「保持するか否か」」 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.ja.md:64 | 要改 | 改后：「**未済の検査**：C909 (コンテナツリーの階層 1 の区切り key がコンテナ番号に等しいことを core だけが検査する)。」工作区 18-ブロックに含める情報.ja.md 第 64 行已是（对象外，按清单补）。原话「**未済の検査**：C478」 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.md:53 | 不相干 | 已是新写法：定案句原话「区间取子树覆盖区间」写的是条款，与 C478 还清后的现状一致，没有说实现欠着 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.md:55 | 不相干 | 已是新写法：原话「孩子是码 3 容器的内部节点（inode 树、墓碑树、谱系树、退役树）的区间是子树覆盖区间」并列了写者、读者、池级 checker 与坏镜像用例，与新事实一致 |
| F12 | .claude/kb/decisions/18-块里携带什么信息.md:62 | 事件句不改 | 依据行原话「用户定案 2026-09-23：区间取子树覆盖区间」记的是那一天的定案，C478 实现与否不改它 |
| F12 | .claude/kb/experiments/172-删除写出的字节的干跑.md:162 | 要人看 | 原话「td、tt 头的层级与 key 区间只记值，对错归 C478」：C478 在结束那一版已还清（checks-owed.md:875），C713 在暂存区也已还清，「对错归」指向的两笔都不再开着。tt（墓碑树根）的区间今天归池级 checker 判；td（deleted_inodes 根）的区间 C478 的实现没管：还清行只列 inode 树与容器树，暂存区 walk.rs 的 deleted_inodes_queue_without_judging 只核层级 0、key 宽与条目宽，不核区间（只读了这一个函数，没全仓找别的判法）。td 那一格改指哪里、要不要另立一笔账，交主 agent 定，与 layout/03 那三行同一问 |
| F12 | .claude/kb/layout/01-first-txn.md:258 | 不相干 | 已是新写法：原话「key 区间（子树覆盖区间 [最左那片的最小记录号, 最右那片的最大记录号]」是这一批写回甲改的，与新事实一致 |
| F12 | .claude/kb/layout/03-third-txn.en.md:13 | 要人看 | 原话「Intent publish (queue transitions」那一行状态格末句「key range belongs to C478 (...)」：C478 已还清，这一句指向一笔不再开着的账；工作区（别的会话改了层级那半句）仍留着这一句。D18 已定项 2 定案写叶层码 2 节点的区间就是首末两条条目的 key，按字面 deleted_inodes 根（层级 0、条目是号）的区间已有条款，代码写 [N, N] 同值，但池级 checker 不核这个根的区间（暂存区 walk.rs 的 deleted_inodes_queue_without_judging 只核层级与宽度，推的）。候选改法：状态格写「key range is settled: a leaf-level code 2 node takes the first and last entry keys (D18 settled item 2); the code writes the first and last queue numbers, the same value; the pool-level checker does not judge it」并另立一笔账。要不要这样改、立不立账交主 agent |
| F12 | .claude/kb/layout/03-third-txn.ja.md:14 | 要人看 | 原话「意図パブリッシュ（キューが空から」那一行状态格末句「キー範囲は C478 (...)に帰属」指向已还清的 C478，工作区仍留着；与 layout/03-third-txn.en.md:13 同一问，候选改法照那一行译，交主 agent 定 |
| F12 | .claude/kb/layout/03-third-txn.md:13 | 要人看 | 原话「key 区间归 C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间）」指向已还清的 C478（结束那一版 checks-owed.md:875），工作区（别的会话改了层级）仍留着这一句与「key 区间是格式级空白，条款没写这一格」。D18 已定项 2 定案原话「叶层码 2 节点上它就是首末两条条目的 key」按字面已罩这个根，代码写的队列首末两个号同值；池级 checker 不核这个根的区间（暂存区 walk.rs 的 deleted_inodes_queue_without_judging，推的）。候选改法：「key 区间已定：叶层码 2 节点取首末两条条目的 key（D18（块里携带什么信息） 已定项 2），代码写队列首末两个号（一条 key 时 [N, N]）与它同值；池级 checker 不核这个根的区间」，另立一笔账。交主 agent 定 |
| F12 | .claude/kb/layout/03-third-txn.md:19 | 不相干 | 已是新写法：原话「key 区间是子树覆盖区间（D18（块里携带什么信息） 已定项 2）」是这一批写回甲改的墓碑树根那一行，与新事实一致 |
| F12 | .claude/kb/milestone/01-first-txn.en.md:88 | 事件句不改 | 原话「Status (kickoff 2026-09-14, all four acceptance items passed)」后面「Checker adds ... check_index_node_keys (... range tightly bounding boundary entries)」记的是里程碑一加了什么；今天这个函数仍判叶的区间贴紧首末条目（结束那一版 lib.rs 第 732 行），C478 改的是孩子是码 3 容器的内部节点，不改这一句 |
| F12 | .claude/kb/milestone/01-first-txn.ja.md:88 | 事件句不改 | 原话「現状（2026-09-14 着工、受入 4 項目すべて合格）」后「checker に ... check_index_node_keys（... 境界エントリに密着した区間）... を追加」记里程碑一加了什么；理由同 milestone/01-first-txn.en.md:88 |
| F12 | .claude/kb/milestone/01-first-txn.md:105 | 事件句不改 | 原话「现状（2026-09-14 开工，验收四条全过）」后「checker 加 ... check_index_node_keys（按树的 key 形态逐字段比、严格递增、区间贴紧首末条目）」记里程碑一加了什么；今天这个函数仍判叶的区间贴紧首末条目（结束那一版 crates/singlefs-checker/src/lib.rs 第 732 行），C478 改的是内部节点 |
| F12 | .claude/kb/milestone/02-second-txn.en.md:374 | 不相干 | 已是新写法：原话「Three measurement bases settled upon Parallel Line 3 landing」那一行描述列记的是当时定下的口径，末格已写「C478 (...) **Repaid**」，与新事实一致 |
| F12 | .claude/kb/milestone/02-second-txn.en.md:563 | 要改 | 改后：「key range of inode tree root takes start and end of separator keys, not subtree range (aligns with checker's current check_index_node_keys, measurement basis ledger in C478 (...))」改成「key range of inode tree root took start and end of separator keys at the time (following the checker's check_index_node_keys of that day); C478 (Is the Key Range of the Code-2 Header the Entry Key Range or the Subtree Coverage Range) is paid off and the inode tree root now takes the subtree coverage range (D18 (Information Carried in a Block) settled item 2)」。原话「Both write path and checker lines completed」所在段的「实做时定下的四处」那一分句去掉日期读是现状句，今天已不成立；工作区这一句没改（工作区只改了大小写） |
| F12 | .claude/kb/milestone/02-second-txn.ja.md:374 | 不相干 | 已是新写法：原话「並行ライン 3 反映時に確定した 3 箇所の計測条件」那一行末格已写 C478「**弁済済み**」，与新事实一致 |
| F12 | .claude/kb/milestone/02-second-txn.ja.md:563 | 要改 | 改后：「inode ツリー根のキー区間はセパレータキーの始端と終端を取り、部分木の被覆範囲は取らない（現在の checker の check_index_node_keys に追従、計測条件の記録は C478 (...)）」改成「inode ツリー根のキー区間は当時セパレータキーの始端と終端を取った（当時の checker の check_index_node_keys に追従）；C478 (コード 2 ヘッダの key 区間はエントリキー区間かサブツリーカバー区間か) は完済し、今は inode ツリー根もサブツリーカバー区間を取る（D18 (ブロックに含める情報) 確定項目 2）」。原话「書き込みパスと checker の両ラインとも完成した」；工作区这一句没改 |
| F12 | .claude/kb/milestone/02-second-txn.md:391 | 不相干 | 已是新写法：原话「并行线三落地时定下的三处口径」那一行末格是这一批写回甲改的「C478（…） **已还**」，写者、core 读者与池级 checker 已按它改，与新事实一致 |
| F12 | .claude/kb/milestone/02-second-txn.md:576 | 要改 | 改后：「inode 树根的 key 区间取分隔 key 的首尾、不取子树范围（跟 checker 今天的 check_index_node_keys 走，口径的账在 C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间））」改成「inode 树根的 key 区间当时取分隔 key 的首尾、不取子树范围（跟当时 checker 的 check_index_node_keys 走）；C478（码 2 头的 key 区间是条目键区间还是子树覆盖区间） 已还清，今天 inode 树根取子树覆盖区间（D18（块里携带什么信息） 已定项 2）」。原话「写路径与 checker 那两条都做出来了」所在段的这一分句写「今天」，已不成立；工作区没改 |
| F12 | crates/singlefs-checker/invariants/01-块自描述.md:7 | 要补 | 结束那一版 I-1.1（原话「块头自述逻辑地址」）的条文只写多层码 2 树（记账树、中央映射树）内部节点区间是子树覆盖区间，没写孩子是码 3 容器的 inode 树、墓碑树、谱系树、退役树，状态列也没列 C478 的判法与用例。要补：条文加「孩子是码 3 容器的那几棵树（inode 树，墓碑树、谱系树、退役树）的码 2 节点头里的 key 区间同样是子树覆盖区间（…）」，状态列加「孩子是码 3 容器的子树覆盖区间那一半：walk::Walk::judge_the_key_range_of_an_inode_internal_node_against_its_leaves 与 walk::Walk::walk_the_internal_node_below_a_container_tree_entry …」（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写） |
| F12 | records/2026-09-22-panic面普查-走得到的那些.md:108 | 事件句不改 | 原话「**index_node_view」是 2026-09-22 panic 面普查当天读码的记录（带当时的行号），说的是那一次查到什么，C478 后来还清不改它 |
| F12 | records/2026-09-22-panic面普查清单.md:604 | 事件句不改 | 原话「crates/singlefs-ch」是 2026-09-22 panic 面普查当天读码的记录（带当时的行号），说的是那一次查到什么，C478 后来还清不改它 |
| F12 | records/2026-09-24-里程碑二收尾调度.md:140 | 事件句不改 | 原话「主 agent 自行定的（2」是 2026-09-25 主 agent 逐条定的那一批判定，C478 只是材料员多收进材料的编号之一，记那一次发生的事 |
| F12 | records/2026-09-24-里程碑二收尾调度.md:216 | 事件句不改 | 原话「实审 B2 中途交接（202」记 2026-09-27 实审 B2 交接与当天主 agent 判的设计问题（「随 C478 定」是那一天的去向），不是现状句 |
| F12 | records/2026-09-28-里程碑三第十项清单.md:180 | 事件句不改 | 原话「疑似已还或过期 」所在的是 2026-09-28 第十项清单当天主 agent 的终判表，记那一天的判定，不随 C478 后来还清改 |
| F12 | records/2026-09-28-里程碑三第十项清单.md:317 | 事件句不改 | 原话「部分已还 」所在的是 2026-09-28「疑似已还的 52 行：核实与处置」那一天的核实结果，记当时仍按首末两条分隔 key，C478 后来还清不改那一天的记录 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:696 | 事件句不改 | 原话「ee 报 checker 档带」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:720 | 事件句不改 | 原话「写回批（批 17）：两个 kb-」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:721 | 事件句不改 | 原话「甲（~/.local/share/singlefs/…」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:722 | 事件句不改 | 原话「乙（~/.local/share/singlefs/…」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:731 | 事件句不改 | 原话「交主 agent 两处：D22」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:733 | 事件句不改 | 原话「写回乙交回（~/.local/share/singlefs/…」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |
| F12 | records/2026-10-18-里程碑三收尾与近期欠账.md:751 | 事件句不改 | 原话「C478、C743 简称译法不统」是批 17 调度记录经过一节里那一次派发、交回或交主 agent 的记录，说的是那一次发生的事，不随 C478 还清改 |

### F13

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F13 | .claude/kb/checks-owed.en.md:582 | 不相干 | 原话「Misplaced reads of other self-verifying structures」：C864 拿 C743 的做法当形态（只管根环槽），C743 还清不改它要查的别的自证结构 |
| F13 | .claude/kb/checks-owed.en.md:607 | 不相干 | 已是新写法：这一行在 Settled 表，原话「the placement formula check when reading the root ring is implemented」与新事实一致 |
| F13 | .claude/kb/checks-owed.ja.md:572 | 不相干 | 原话「他の自己検証構造の位置ずれ読み出し」：C864 只引 C743 的做法当形态，C743 还清不改它 |
| F13 | .claude/kb/checks-owed.ja.md:870 | 不相干 | 已是新写法：这一行在解決済み表，原话「配置先公式の照合は実装済み」与新事实一致 |
| F13 | .claude/kb/checks-owed.md:516 | 不相干 | 命中的是 C773 行里写者一侧的疑点「代码按落点公式在 [0, min(3S, first_txg)) 里找它可能住过的最旧 txg」，说的是挂载读坏槽的回收门槛钉，不是 C743 的公式核，原话「读坏与全零根槽的回收门槛钉只在内存里」 |
| F13 | .claude/kb/checks-owed.md:570 | 不相干 | 原话「别的自证结构的读错位没人查」：C864 拿 C743 的做法当形态，C743 还清不改它要查的系统配置槽、journal 记录、单元 |
| F13 | .claude/kb/checks-owed.md:876 | 不相干 | 已是新写法：这一行在已还清表，原话「读根环时核落点公式已实现」与新事实一致 |
| F13 | .claude/kb/decisions/16-发布语义.en.md:186 | 要改 | 改后：「the formula check is not yet implemented, owed in C743 (...)」改成「the formula check is implemented (root_sits_in_the_ring_slot_the_formula_gives in crates/singlefs-core/src/root_ring.rs, called both when recovery and when mount read the root ring), and C743 (rebuilt allocator recording new class does not verify root occupies slot) is paid off」。工作区改名后的 16-publish-semantics.en.md 第 191 行已是这一句（对象外，按清单补），不用另写。原话「Root records must carry 32-bit instance generation」 |
| F13 | .claude/kb/decisions/16-发布语义.en.md:200 | 事件句不改 | 依据行原话「The tie in rider clause 1」记的是根环三方第三轮判决那一格与各轮经过，C743 还清不改 |
| F13 | .claude/kb/decisions/16-发布语义.en.md:202 | 要改 | 改后：「**Owed checks**: C88 (timeline differentiation in root ring unimplemented) (checker side of rider clause 1, ...; merged from C85 (...), clauses consolidated, check that flags red remains owed).」去掉 C743 那一段。工作区 16-publish-semantics.en.md 第 207 行已是（对象外，按清单补）。原话「**Owed checks**: C88」 |
| F13 | .claude/kb/decisions/16-发布语义.ja.md:186 | 要改 | 改后：「公式照合はまだ実装されておらず、C743 (...) に未済として記録」改成「公式照合は実装済みで（crates/singlefs-core/src/root_ring.rs の root_sits_in_the_ring_slot_the_formula_gives、リカバリでもマウントでもルートリングを読むときに呼ぶ）、C743 (...) は完済」。工作区 16-パブリッシュセマンティクス.ja.md 第 191 行已是（对象外，按清单补）。原话「ルートレコードは 32 ビットのインスタンス代番号」 |
| F13 | .claude/kb/decisions/16-发布语义.ja.md:200 | 事件句不改 | 依据行原话「付帯条項 1 の同値」记第三轮判决那一格与各轮经过，C743 还清不改 |
| F13 | .claude/kb/decisions/16-发布语义.ja.md:202 | 要改 | 改后：「**未済の検査**：C88 (...)（...赤判定を出すべき検査は未済のまま）。」去掉 C743 那一段。工作区 16-パブリッシュセマンティクス.ja.md 第 207 行已是（对象外，按清单补）。原话「**未済の検査**：C88」 |
| F13 | .claude/kb/decisions/16-发布语义.md:186 | 要改 | 改后：「公式核还没实现，欠在 C743（重建分配器记新类时不核根住不住这一槽）」改成「公式核已实现（crates/singlefs-core/src/root_ring.rs 的 root_sits_in_the_ring_slot_the_formula_gives，恢复与挂载读根环时都调），C743（重建分配器记新类时不核根住不住这一槽） 已还清」（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「根记录必须带 32 位实例代号」；暂存区第 191 行就是这一句 |
| F13 | .claude/kb/decisions/16-发布语义.md:200 | 事件句不改 | 依据行原话「骑手条款 1 的平局」记根环三方第三轮判决那一格与各轮经过，C743 还清不改 |
| F13 | .claude/kb/decisions/16-发布语义.md:202 | 要改 | 改后：「**欠**：C88（根环的时间线判别未实现）（骑手条款 1 的 checker 侧，反推腿那一轮找出来的可达破绽；并入自 C85（发布计数器的四个等号没写死），条款已写成一条，会红的检查仍欠）。」去掉「；C743（…）：读根环时的落点公式核」（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「**欠**：C88（根环的时间线判别未实现）」；暂存区第 207 行就是这一句 |
| F13 | .claude/kb/decisions/17-实现分层与第三方管道.en.md:33 | 事件句不改 | 依据行原话「Reads may be misplaced: root ring three-party verdicts」记三轮判决与用户定案，C743 还清不改 |
| F13 | .claude/kb/decisions/17-实现分层与第三方管道.ja.md:33 | 事件句不改 | 依据行原话「読み出しは位置がずれうる」记三轮判决与用户定案，C743 还清不改 |
| F13 | .claude/kb/decisions/17-实现分层与第三方管道.md:24 | 不相干 | 定案句原话「读可以错位（读回别处一份自证得过的内容」写的是失败模型的条款，不说实现欠不欠，C743 落地与它一致 |
| F13 | .claude/kb/decisions/17-实现分层与第三方管道.md:33 | 事件句不改 | 依据行原话「读可以错位：根环三方三轮判决」记三轮判决与用户定案，C743 还清不改 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.en.md:140 | 要改 | 改后：括号末句「the formula check is not yet implemented, owed in [checks-owed.md](../checks-owed.md) C743 (...)」改成中文第 140 行的译文「the formula check lives in the two readers of the root ring: read_root_ring_slot (crates/singlefs-core/src/recovery.rs) ... and the allocator rebuild's pass over the root ring (crates/singlefs-core/src/mount.rs) both call root_sits_in_the_ring_slot_the_formula_gives (crates/singlefs-core/src/root_ring.rs) ...」。工作区改名后的 22-synthesizing-unit-atomicity.en.md 已是（对象外，按清单补）。原话「**Instance generation**」 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.en.md:169 | 事件句不改 | 依据行原话「Tie in the instance generation row」记第三轮判决那一格与各轮经过，C743 还清不改 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.en.md:171 | 要改 | 改后：去掉「C743 (...): the placement formula check when reading the root ring.」那一段，余下「**Owed checks**: C88 (...); C148 (...); C308 (...). (C78 ... C157 ...)」。工作区 22-synthesizing-unit-atomicity.en.md 第 175 行已是（对象外，按清单补）。原话「**Owed checks**: C88 (Timeline discrimination」 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.ja.md:140 | 要改 | 改后：括号末句「公式照合はまだ実装されておらず、[checks-owed.md](../checks-owed.md) C743 (...) に未済」改成中文第 140 行的译文（公式照合はルートリングを読む二つの読み手にある：read_root_ring_slot ... とアロケータ再構築 ... がどちらも root_sits_in_the_ring_slot_the_formula_gives を呼ぶ ...）。工作区 22-ユニット原子性の合成手法.ja.md 已是（对象外，按清单补）。原话「**インスタンスコード**」 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.ja.md:169 | 事件句不改 | 依据行原话「インスタンスコードの行の同点」记第三轮判决那一格与各轮经过，C743 还清不改 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.ja.md:171 | 要改 | 改后：去掉「C743 (...)：ルートリング読み出し時の配置先公式照合」那一段，余下「**未済の検査**：C88 (...)；C148 (...)；C308 (...)。（C78 ... C157 ...）」。工作区 22-ユニット原子性の合成手法.ja.md 第 175 行已是（对象外，按清单补）。原话「**未済の検査**：C88 (ルートリング」 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.md:140 | 不相干 | 已是新写法：原话「公式核在读根环的两个读者里」是这一批写回甲改的，与新事实一致 |
| F13 | .claude/kb/decisions/22-单元原子性怎么合成.md:169 | 事件句不改 | 依据行原话「实例代号那一行的平局」记第三轮判决那一格与各轮经过，C743 还清不改 |
| F13 | .claude/kb/experiments.en.md:27 | 不相干 | 原话「Bounded Exhaustive Recovery Replay」那一行是 E165 各次重跑的状态，命中的是第 7 次重跑照落点公式摆根那一句；C743 还清不改 E165 跑到哪一段 |
| F13 | .claude/kb/experiments.ja.md:27 | 不相干 | 原话「リカバリ再生の有界全数探索」那一行是 E165 各次重跑的状态，C743 还清不改 E165 跑到哪一段 |
| F13 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:628 | 不相干 | 路径登记原话「从镜像的根环现读」那一行的共用项列「根环落点公式（target_for_publish）、…、read_root_ring_slot」写装置读了哪几样，C743 落地不改它读了什么，也不让 E156 的数变（装置只摆合法写者的根） |
| F13 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:631 | 不相干 | 路径登记原话「发完改回」那一行共用项「根槽落点公式」写装置读了什么，C743 落地不改它 |
| F13 | .claude/kb/experiments/156-alloc-basis四条岔路的代价数.md:634 | 不相干 | 原话「路径 1 与 2 在每次挂载交回与每格终点逐根逐盘比」那一段列各组比对共用的前提（含根环落点公式），说的是比对设计，C743 落地不改 |
| F13 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1282 | 事件句不改 | 原话「第 7 次重跑（对抗根环照落点公式摆根」是那一次重跑的节标题，带日期，记那一次做了什么 |
| F13 | .claude/kb/experiments/165-恢复重放有界穷举与变异杀伤界.md:1284 | 事件句不改 | 原话「重跑登记 research/prompts/e165-r7-prereg.md」那一段写第 7 次重跑的起因（C743 的落点核进了实现），说的是那一次发生的事，与新事实一致 |
| F13 | .claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:17 | 事件句不改 | 原话「对拍（停机条款 K1–K6）」那一段记执行员在冻结树拷贝上逐项对过的行号（含落点公式 root_ring.rs:109），说的是那一次的核对 |
| F13 | .claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:228 | 不相干 | 路径登记原话「装置的计数模型」那一行共用项列落点公式常量，说的是装置读了什么，C743 落地不改 |
| F13 | .claude/kb/experiments/173-上限卡在读坏槽的三种处置.md:231 | 不相干 | 原话「共用项问句：条款常量（B、c_max、S、d、落点公式）若错」说的是 E173 两条路径共用前提的风险，C743 落地不改 |
| F13 | .claude/kb/milestone/02-second-txn.md:156 | 不相干 | 原话「暖机（D16（发布语义） 已定项 8 戊）」那一条用落点公式现算暖机次数，不是 C743 的读根环公式核 |
| F13 | .claude/kb/milestone/03-third-txn.en.md:71 | 不相干 | 已是新写法：原话「Gaps in ring and root clauses」那一行里 C743 已写「moved to cleared (the placement formula check when reading the root ring is implemented)」，与新事实一致 |
| F13 | .claude/kb/milestone/03-third-txn.ja.md:71 | 不相干 | 已是新写法：原话「リングとルートの条項空白」那一行里 C743「は完済へ」，与新事实一致 |
| F13 | .claude/kb/milestone/03-third-txn.md:71 | 不相干 | 已是新写法：原话「环与根的条款空白」那一行里「C743（重建分配器记新类时不核根住不住这一槽） 已挪进已还清（读根环时的落点公式核已实现）」，与新事实一致 |
| F13 | .claude/kb/milestone/09-ninth-txn.en.md:73 | 不相干 | 已是新写法：原话「Root ring: bounded slot deduction for holes」那一行写「the formula check is implemented, C743 (...)」已还清，与新事实一致 |
| F13 | .claude/kb/milestone/09-ninth-txn.ja.md:73 | 不相干 | 已是新写法：原话「ルートリング: 穴によるスロット控除の有界化」那一行写「公式による確認は実装済み、C743 (...) は完済」，与新事实一致 |
| F13 | .claude/kb/milestone/09-ninth-txn.md:75 | 不相干 | 已是新写法：原话「根环：洞的扣槽有界」那一行写「公式核已实现，C743（…） 已还清」，与新事实一致 |
| F13 | crates/singlefs-checker/invariants/02-根环.md:24 | 要改 | 改后：条文末句「core 那一半随根环实现批，今天没写。」改成「core 那一半已实现：crates/singlefs-core/src/root_ring.rs 的 root_sits_in_the_ring_slot_the_formula_gives，择根读槽 read_root_ring_slot（crates/singlefs-core/src/recovery.rs）与重建分配器读根环那一遍（crates/singlefs-core/src/mount.rs）都调它，C743（重建分配器记新类时不核根住不住这一槽） 已还清。」原话「根槽位置等于 txg 算出的位置」；暂存区 root_ring.rs 第 183 行有这个函数，recovery.rs 3 处、mount.rs 2 处调它；工作区这一句没改，要写回员改 |
| F13 | records/2026-09-16-subagent拆分提案.md:1325 | 事件句不改 | 原话「现象：写回员起草 checks-owed.md 的规格」记那一次 kb-spec-check 退 1 的现象与原因，C743 只是当时报错的编号之一 |
| F13 | records/2026-09-30-里程碑三开工第一轮.md:669 | 事件句不改 | 原话「m3-batch10-code-」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-09-30-里程碑三开工第一轮.md:677 | 事件句不改 | 原话「第 10h 批交回（opus，补」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-09-30-里程碑三开工第一轮.md:766 | 事件句不改 | 原话「kbwb-10o-r1 交回（k」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-13-里程碑四预开发.md:814 | 事件句不改 | 原话「eb 报新派四件（变异三行 27」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:69 | 事件句不改 | 原话「出口 3 卡在条款上的 13 笔」所在的是批 3 节里排批那一天的批次表，「齐」那一格是那一天的输入状态；C743 后来还清不改那一天的排批记录 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:289 | 事件句不改 | 原话「乙类 123 笔判完（报告 ~/」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:317 | 事件句不改 | 原话「批 7 出口 4 摸底交回（m3」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:323 | 事件句不改 | 原话「用户 2026-10-20 弹窗」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:327 | 事件句不改 | 原话「批 8 派出五件：系统配置落地（」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:337 | 事件句不改 | 原话「出口 3 剩余逐笔估交回（报告」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:339 | 事件句不改 | 原话「C693 摸底派 clerk（o」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:349 | 事件句不改 | 原话「根环一轮正推腿交回（resear」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:352 | 事件句不改 | 原话「根环一轮第一轮判决写成（rese」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:369 | 事件句不改 | 原话「根环第二轮判决写成（resear」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:384 | 事件句不改 | 原话「根环第三轮辩方腿交回（resea」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:385 | 事件句不改 | 原话「根环第三轮辩方模型主 agent」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:454 | 事件句不改 | 原话「C743 并进 C88 的那一行」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:464 | 事件句不改 | 原话「第一波：欠账表第一份（opus，」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:474 | 事件句不改 | 原话「C743、C751、C755 写」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:485 | 事件句不改 | 原话「里程碑九第 59 行 C743」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:492 | 事件句不改 | 原话「D16:184 的 C743 同」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:496 | 事件句不改 | 原话「决策乙交回（报告 ~/.loca」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:498 | 事件句不改 | 原话「已定项 7 也翻未实现（读根环的」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:502 | 事件句不改 | 原话「C743 同源句两处（里程碑九、」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:526 | 事件句不改 | 原话「I-7.17 补 C743；」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:554 | 事件句不改 | 原话「C743 同源句：D16 已定项」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:670 | 事件句不改 | 原话「代码批：根环（C743、C751 甲、C755）」所在的是 2026-10-21 用户「继续吧」那一条里排批 12、13 的表，末格「规格待主 agent 从欠账行与判决汇齐」是那一天的状态，下文第 674、681 行记了批 13 随后派出，这张表是那一天的排批记录，不回改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:675 | 事件句不改 | 原话「等 m4-i315-r2 判决：」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:720 | 事件句不改 | 原话「写回批（批 17）：两个 kb-」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:721 | 事件句不改 | 原话「甲（~/.local/share/singlefs/…」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:722 | 事件句不改 | 原话「乙（~/.local/share/singlefs/…」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:728 | 事件句不改 | 原话「改了 D16 第 36 行、la」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:730 | 事件句不改 | 原话「C743 判可还清，主 agen」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:734 | 事件句不改 | 原话「主 agent 照写回甲的判定（」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |
| F13 | records/2026-10-18-里程碑三收尾与近期欠账.md:751 | 事件句不改 | 原话「C478、C743 简称译法不统」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C743 还清改 |

### F14

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| F14 | .claude/kb/checks-owed.en.md:440 | 要改 | 改后：整行挪进 Settled 表，行文照暂存区 checks-owed.en.md 那一行（工作区第 883 行「The clause is written into D22 (Synthesizing Unit Atomicity) settled item 8 item 3 ...」）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「mkfs 中途崩溃的语义no clause defined」。现查两处：提交对象 A（结束那一版）里这一行还在欠表、写的是做之前的样子，而同一对象的里程碑三二·4 说 C656、C659 已还清；暂存区里 C656、C659 两行已挪进已还清表（暂存区 checks-owed.md 第 874、875 行，已还清在第 597 行；工作区第 883、884 行），实现也在暂存区（make_filesystem.rs 有 make_filesystem_drawing_the_filesystem_identifier 9 处，两份新用例文件 345、240 行，crates/mutations.tsv 以 C656、C659 起头 10 行）。现状是暂存区那一份 |
| F14 | .claude/kb/checks-owed.en.md:442 | 要改 | 改后：整行挪进 Settled 表，行文照暂存区 checks-owed.en.md 那一行（工作区第 884 行「Accepted as a known shape, no product code change ...」）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「Instance generation invariant red when mkfs is redone」。现查两处：提交对象 A（结束那一版）里这一行还在欠表、写的是做之前的样子，而同一对象的里程碑三二·4 说 C656、C659 已还清；暂存区里 C656、C659 两行已挪进已还清表（暂存区 checks-owed.md 第 874、875 行，已还清在第 597 行；工作区第 883、884 行），实现也在暂存区（make_filesystem.rs 有 make_filesystem_drawing_the_filesystem_identifier 9 处，两份新用例文件 345、240 行，crates/mutations.tsv 以 C656、C659 起头 10 行）。现状是暂存区那一份 |
| F14 | .claude/kb/checks-owed.ja.md:429 | 要改 | 改后：整行挪进解決済み表，行文照暂存区 checks-owed.ja.md 那一行（工作区第 883 行）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「mkfs 中途崩溃的语义条項が未定義」。现查两处：提交对象 A（结束那一版）里这一行还在欠表、写的是做之前的样子，而同一对象的里程碑三二·4 说 C656、C659 已还清；暂存区里 C656、C659 两行已挪进已还清表（暂存区 checks-owed.md 第 874、875 行，已还清在第 597 行；工作区第 883、884 行），实现也在暂存区（make_filesystem.rs 有 make_filesystem_drawing_the_filesystem_identifier 9 处，两份新用例文件 345、240 行，crates/mutations.tsv 以 C656、C659 起头 10 行）。现状是暂存区那一份 |
| F14 | .claude/kb/checks-owed.ja.md:431 | 要改 | 改后：整行挪进解決済み表，行文照暂存区 checks-owed.ja.md 那一行（工作区第 884 行）（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「fsid の先頭 8 バイトが同じまま mkfs をやり直す」。现查两处：提交对象 A（结束那一版）里这一行还在欠表、写的是做之前的样子，而同一对象的里程碑三二·4 说 C656、C659 已还清；暂存区里 C656、C659 两行已挪进已还清表（暂存区 checks-owed.md 第 874、875 行，已还清在第 597 行；工作区第 883、884 行），实现也在暂存区（make_filesystem.rs 有 make_filesystem_drawing_the_filesystem_identifier 9 处，两份新用例文件 345、240 行，crates/mutations.tsv 以 C656、C659 起头 10 行）。现状是暂存区那一份 |
| F14 | .claude/kb/checks-owed.md:426 | 要改 | 改后：整行挪进已还清表，行文照暂存区第 874 行「条款写进 D22（单元原子性怎么合成） 已定项 8 第 3 条与射程（用户 2026-10-20 定案「认，照改写稿写回」）：…」（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「mkfs 中途崩溃的语义没有条款」那一行问题列「该挂上旧池、整池拒还是别的，没有条款」与前置列「条款（三方或用户定）」都是做之前的现状。现查两处：提交对象 A（结束那一版）里这一行还在欠表、写的是做之前的样子，而同一对象的里程碑三二·4 说 C656、C659 已还清；暂存区里 C656、C659 两行已挪进已还清表（暂存区 checks-owed.md 第 874、875 行，已还清在第 597 行；工作区第 883、884 行），实现也在暂存区（make_filesystem.rs 有 make_filesystem_drawing_the_filesystem_identifier 9 处，两份新用例文件 345、240 行，crates/mutations.tsv 以 C656、C659 起头 10 行）。现状是暂存区那一份 |
| F14 | .claude/kb/checks-owed.md:428 | 要改 | 改后：整行挪进已还清表，行文照暂存区第 875 行「认作已知形态、不改产品代码（用户 2026-10-20 定「认作已知形态，钉坏镜像」）：坏镜像用例 …」（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「fsid 前 8 字节相同时重做 mkfs 实例代号不变式红」那一行会红的检查列「造一张坏镜像…钉住」是没钉之前的写法。现查两处：提交对象 A（结束那一版）里这一行还在欠表、写的是做之前的样子，而同一对象的里程碑三二·4 说 C656、C659 已还清；暂存区里 C656、C659 两行已挪进已还清表（暂存区 checks-owed.md 第 874、875 行，已还清在第 597 行；工作区第 883、884 行），实现也在暂存区（make_filesystem.rs 有 make_filesystem_drawing_the_filesystem_identifier 9 处，两份新用例文件 345、240 行，crates/mutations.tsv 以 C656、C659 起头 10 行）。现状是暂存区那一份 |
| F14 | .claude/kb/decisions/22-单元原子性怎么合成.en.md:197 | 要改 | 改后：「**Owed checks**: C79 (...): Journal ring mirrored across two disks implemented as forceable branch (...).」去掉末尾「; C659 (...)」。工作区改名后的 22-synthesizing-unit-atomicity.en.md 第 201 行已是（对象外或别的会话的改动），不用另写。原话「**Owed checks**: C79 (Placement of system configuration」 |
| F14 | .claude/kb/decisions/22-单元原子性怎么合成.ja.md:197 | 要改 | 改后：「**未済の検査**：C79 (...)：journal リングの2ディスクミラーリングの分岐を強制到達可能な分岐とする（...）。」去掉末尾「；C659 (...)」。工作区 22-ユニット原子性の合成手法.ja.md 第 201 行已是，不用另写。原话「**未済の検査**：C79 (システム構成と journal」 |
| F14 | .claude/kb/decisions/22-单元原子性怎么合成.md:197 | 要改 | 改后：「**欠**：C79（系统配置与 journal 环的放置没人定）：journal 环两盘镜像那一支做成能强制进入的分支（系统配置每盘一份、整盘掉了两支已能强制进入，用例见 C79（系统配置与 journal 环的放置没人定） 行）。」去掉末尾「；C659（…）」（暂存区与工作区已是这一句，别的会话或对象外的改动，不在提交对象 A 里，随它们提交即可，不用另写）。原话「**欠**：C79（系统配置与 journal 环的放置没人定）」；暂存区与工作区第 201 行就是这一句 |
| F14 | .claude/kb/milestone/03-third-txn.en.md:71 | 不相干 | 已是新写法：原话「Gaps in ring and root clauses」那一行写 C656 done and moved to the settled table、C659 pinned and cleared，与暂存区现状（两行在已还清表、实现在暂存区）一致；对不上的是同一对象里的欠账表，见 checks-owed.en.md:440、:442 两行 |
| F14 | .claude/kb/milestone/03-third-txn.en.md:72 | 不相干 | 原话「Remaining debt clusters belonging to Item II」那一行写 C659「moved to II·4 and removed」，C659 还清后这句仍是真话 |
| F14 | .claude/kb/milestone/03-third-txn.ja.md:71 | 不相干 | 已是新写法：原话「リングとルートの条項空白」那一行写 C656「完済へ移管」、C659 已钉已完済，与暂存区现状一致 |
| F14 | .claude/kb/milestone/03-third-txn.ja.md:72 | 不相干 | 原话「項目二に帰属するその他の未済の検査クラスター」那一行写 C659「は二·4 に移し、除去済み」，C659 还清后仍是真话 |
| F14 | .claude/kb/milestone/03-third-txn.md:71 | 不相干 | 已是新写法：原话「环与根的条款空白」那一行写「C656（…） 已照用户 2026-10-20 定案做完、已挪进已还清」与 C659「已钉，已还清」，与暂存区现状一致（暂存区欠账表两行已在已还清表）；提交对象 A 里欠账表这两行还在欠表，见 checks-owed.md:426、:428 |
| F14 | .claude/kb/milestone/03-third-txn.md:72 | 不相干 | 原话「其余归项二的欠账簇」那一行写 C659「归二·4、已拿掉」，C659 还清后仍是真话 |
| F14 | .claude/kb/translation-glossary.md:125 | 不相干 | 术语表原话「fsid 前 8 字节相同时重做 mkfs 实例代号不变式红」是 C659 简称的三语译名，C659 还清不改简称 |
| F14 | records/2026-10-01-里程碑三形式化接入.md:538 | 事件句不改 | 原话「12-3 写回交回（报告 /tm」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-13-里程碑四预开发.md:753 | 事件句不改 | 原话「丙那份补丁打进主工作区（11:3」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-13-里程碑四预开发.md:814 | 事件句不改 | 原话「eb 报新派四件（变异三行 27」是那一份调度记录里那一次交回或派发的经过，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:69 | 事件句不改 | 原话「出口 3 卡在条款上的 13 笔」所在的是批 3 节里排批那一天的批次表，「齐」那一格是那一天的输入状态；C656 后来还清不改那一天的排批记录 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:70 | 事件句不改 | 原话「C758、C656 量数」所在的是批 3 节里排批那一天的批次表（批 8 那一行），记当时派什么、输入齐没齐，C656 后来还清不改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:289 | 事件句不改 | 原话「乙类 123 笔判完（报告 ~/」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:317 | 事件句不改 | 原话「批 7 出口 4 摸底交回（m3」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:323 | 事件句不改 | 原话「用户 2026-10-20 弹窗」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:327 | 事件句不改 | 原话「批 8 派出五件：系统配置落地（」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:331 | 事件句不改 | 原话「C758 与 C656 量数交回」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:333 | 事件句不改 | 原话「用户 2026-10-20 弹窗」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:334 | 事件句不改 | 原话「出口 4 丙的两份自证主 age」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:336 | 事件句不改 | 原话「C656 三方两条腿派出（正推」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:341 | 事件句不改 | 原话「C656 三方正推腿交回（res」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:348 | 事件句不改 | 原话「C656 攻方腿交回（resea」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:355 | 事件句不改 | 原话「C656 判决复跑：攻方模型整条」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:357 | 事件句不改 | 原话「C656 判决复跑：攻方 too」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:358 | 事件句不改 | 原话「C656 攻方第二次抽样交回（r」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:362 | 事件句不改 | 原话「C656 第二次抽样模型复跑：1」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:368 | 事件句不改 | 原话「空出的四个槽照逐笔估「现在就能派」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:376 | 事件句不改 | 原话「C659、C699、C395 调」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:390 | 事件句不改 | 原话「用户要清盘：只删本会话的编译目录」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:392 | 事件句不改 | 原话「看门狗报主工作区变异轻型（已跑」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:401 | 事件句不改 | 原话「C656 第二次抽样模型复跑排队」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:404 | 事件句不改 | 原话「开 C656 第二轮 m3-cl」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:412 | 事件句不改 | 原话「C656 第二轮攻方腿交回（re」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:413 | 事件句不改 | 原话「C656 第二轮辩方腿交回（re」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:414 | 事件句不改 | 原话「C656 第二轮辩方模型主 ag」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:416 | 事件句不改 | 原话「C656 第二轮两腿模型主 ag」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:417 | 事件句不改 | 原话「C693 第三轮辩方模型主 ag」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:418 | 事件句不改 | 原话「用户 2026-10-20 弹窗」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:420 | 事件句不改 | 原话「开 C656 第三轮（最终轮）m」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:424 | 事件句不改 | 原话「批 9（写段，测试与钉值一侧，副」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:425 | 事件句不改 | 原话「b6 给开工信号（它的两趟轻型因」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:426 | 事件句不改 | 原话「GPU 垃圾回收会话回话接受打补」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:428 | 事件句不改 | 原话「C656 第三轮攻方腿交回（re」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:436 | 事件句不改 | 原话「C656 第三轮攻方模型主 ag」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:443 | 事件句不改 | 原话「C656 第三轮辩方腿交回（re」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:444 | 事件句不改 | 原话「用户弹窗定 C656 第三轮交用」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:446 | 事件句不改 | 原话「批 10（kb 写回批）开头：C」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:450 | 事件句不改 | 原话「C656 第三轮辩方模型主 ag」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:452 | 事件句不改 | 原话「C773 宽度连带 17 处、C」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:453 | 事件句不改 | 原话「milestone/01-fir」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:457 | 事件句不改 | 原话「C659：第 358 行要立的新」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:461 | 事件句不改 | 原话「辩方 T8（C6 仍欠栏）判决没」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:466 | 事件句不改 | 原话「第三波：欠账表第二份（C659、」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:489 | 事件句不改 | 原话「第三波欠账表第二份派出（opus」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:496 | 事件句不改 | 原话「决策乙交回（报告 ~/.loca」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:499 | 事件句不改 | 原话「C659 新简称等第三波定了，主」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:543 | 事件句不改 | 原话「C659、C563、C499、C」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:549 | 事件句不改 | 原话「里程碑三二·5 的 C659 去」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:550 | 事件句不改 | 原话「C659 简称「没查根因」已不成」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:570 | 事件句不改 | 原话「C659 改简称并全仓中文同步；」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:571 | 事件句不改 | 原话「里程碑三二·5 去掉 C659；」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:581 | 事件句不改 | 原话「C659 新简称「fsid 前」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:582 | 事件句不改 | 原话「里程碑三二·5 去掉 C659；」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:592 | 事件句不改 | 原话「变更史里 C659 旧简称，连同」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:598 | 事件句不改 | 原话「变更史 C659 换名；」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:616 | 事件句不改 | 原话「三语乙（九份决策）交回（报告 ~」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:619 | 事件句不改 | 原话「这一批新加的 C659、C863」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:657 | 事件句不改 | 原话「b6 的 I-3.15 调查有了」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:670 | 事件句不改 | 原话「代码批：根环（C743、C751 甲、C755）」所在的是 2026-10-21 排批 12、13 的表，末格「规格待主 agent 从欠账行与判决汇齐」是那一天的状态，下文第 674、681 行记了 C656 mkfs 实现员随后派出，这张表是那一天的排批记录，不回改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:674 | 事件句不改 | 原话「现在派：C656 mkfs（乙两」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:681 | 事件句不改 | 原话「C656 mkfs 实现员（m3」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:683 | 事件句不改 | 原话「看门狗已起。派发闸两次拒在 m4」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:684 | 事件句不改 | 原话「GPU 垃圾回收会话回（m4-p」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:821 | 事件句不改 | 原话「变异轻型（核心部分加点名 12」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:823 | 事件句不改 | 原话「变异轻型只重跑那两行退 0：「计」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:867 | 事件句不改 | 原话「E165 六行变异证红跑完退 1」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:868 | 事件句不改 | 原话「打 K1 补丁（改前四份打包备份」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:870 | 事件句不改 | 原话「C656 mkfs 那一行加逐行」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |
| F14 | records/2026-10-18-里程碑三收尾与近期欠账.md:872 | 事件句不改 | 原话「预开发会话报它 K1 之后那一趟」是批次表或调度经过里那一次派发、交回、判定的记录，说的是那一次发生的事，不随 C656、C659 还清改 |

### 反向问（第 10 步）

做成的事里归 F12-F14 的是范围清单第 8 条（C478、C743 写回）与第 13 条（C909 登号、D18 已定项 2 的欠改成 C909）。逐件找该记它的载体：C478 还清在欠账表三语、D18 三语、layout 01/03、里程碑二都有；C909 在欠账表三语与 D18 中文（英日在改名后的新文件）有；C743 还清在欠账表三语、D22 中文、里程碑九与里程碑三有。下面 4 行是结束那一版里该随之变、却还是旧值的进度标记（暂存区已是新值）。

| 组 | 载体 | 判定 | 改后的句子或理由 |
|---|---|---|---|
| M1 | .claude/kb/decisions/22-单元原子性怎么合成.md:18 | 要补 | 索引表已定项 7「根记录的字段表」在结束那一版是「**进度：未实现。**」（当初翻成未实现的理由是「读根环的公式核欠在 C743」，调度记录结束那一版第 498 行），C743 还清之后该是「**进度：已实现。**」；暂存区与工作区已是「已实现」，英日改名后的新文件也是 Implemented / 実装済み（调度记录工作区第 1732 行 kb 小修 clerk 报过「工作区里三语与索引进度已是「已实现」（HEAD 是旧值），不改」），提交对象 A 没带上 |
| M2 | .claude/kb/decisions/22-单元原子性怎么合成.md:19 | 要补 | 索引表已定项 8「固定结构的放置」在结束那一版是「**进度：未实现。**」，C656 照定案实现（暂存区）之后暂存区与工作区已是「**进度：已实现。**」，提交对象 A 没带上；英日两份同（结束那一版旧名文件 Progress: Unimplemented、進捗：未実装） |
| M3 | .claude/kb/decisions/16-发布语义.md:16 | 要补 | 索引表已定项 6「发布计数器」在结束那一版是「**进度：未实现。**」，暂存区是「**进度：已实现。**」。已定项 6 骑手条款 1 含读根环的公式核（C743），推的：这一格是随 C743 改的，暂存区那一份另有别的会话对 D16 已定项 2、5、10 的进度改动，没逐项核谁改的 |
| M4 | .claude/kb/decisions.md:63 | 要补 | D22 那一行计数在结束那一版是「已实现 15 项 / 未实现 3 项 / 无对象 8 项」，暂存区是「已实现 17 项 / 未实现 1 项 / 无对象 8 项」（M1、M2 两格），由门禁 doc-decisions 的 decision-items-sync 格写回；D16 那一行（结束那一版「已实现 12 项 / 未实现 4 项」、暂存区「已实现 10 项 / 未实现 6 项」）混着别的会话对已定项 2、5、10 的改动，不归这一批 |

## 交实现员

0 行。分到的 215 行里载体在 `crates/` 下的只有两行，都是不变量清单的 `.md`（`crates/singlefs-checker/invariants/01-块自描述.md:7` 要补、`crates/singlefs-checker/invariants/02-根环.md:24` 要改），照定义留在判定表里走写回员。数行命令（在本报告上跑）：

```text
$ grep -E '^\| [^|]* \| crates/[^|]*:[0-9]+ \| (要改|要补)' result.md | grep -vc 'crates/singlefs-checker/invariants/'
0
```

另记（候选表外，不进上表）：结束那一版的 `crates/singlefs-checker/src/walk.rs` 第 358 行「inode 树根、树表与根兼叶的那几棵今天也这样写」与第 3838 行「区间贴紧首末条目，C478（…） 还开着」是 C478 之前的注释；对象 A 里的 `walk.rs` 就是 HEAD 那一份，暂存区那一份已改（第 426 行写「树表与根兼叶的那几棵（`deleted_inodes`、logged_ops 根之外的层级 0 根）也这样写」，第 4248 行写「头里的区间是子树覆盖区间，走完它下面的叶判」），随暂存区的代码一起提交即可，不用派实现员。

## 命令与验证

- 读的版本：结束那一版用 `git show 1e3079d7a07304ceaf5a9de84f080853d61f52b5:<路径>`（`git cat-file -t` 得 `commit`），暂存区用 `git show :<路径>`，工作区直接读。每行先取结束那一版的那一行，再看工作区里有没有逐字相同的一行（辅助脚本 `tools/show.py`），不同的再逐处比（`tools/linediff.py`）。
- 判定表由 `tools/verdicts.py` 生成，`tools/qcheck.py` 载入 `scripts/kb/stale-candidates.py` 的 `quotes_the_line` 逐行核：「bad 0 of 215」（每行都引了候选原文至少 4 个字、判定以五种之一开头）。
- 代码在哪一份（这一次现跑，原样）：

```text
$ git show :.claude/kb/checks-owed.md | grep -nE '^\| (C656|C659) |^### 已还清' | cut -c1-12
597:### 已�
874:| C656 |
875:| C659 |
$ git show 1e3079d7a…:.claude/kb/checks-owed.md | grep -nE '^\| (C656|C659) |^### 已还清' | cut -c1-12
426:| C656 |
428:| C659 |
599:### 已�
HEAD c659用例不在
HEAD c478用例不在
HEAD c743函数不在
1e3079d7a07304ceaf5a9de84f080853d61f52b5 c659用例不在
1e3079d7a07304ceaf5a9de84f080853d61f52b5 c478用例不在
1e3079d7a07304ceaf5a9de84f080853d61f52b5 c743函数不在
$ git show :crates/singlefs-core/src/root_ring.rs | grep -c root_sits_in_the_ring_slot_the_formula_gives
5
```

（「用例」指 `crates/singlefs-harness/tests/checker_a_pool_remade_over_an_old_pool_sharing_the_first_eight_identifier_bytes.rs` 与 `crates/singlefs-harness/tests/key_range_of_code_two_nodes_over_containers_is_the_subtree_coverage.rs`，`git cat-file -e` 判；「函数」指 `crates/singlefs-core/src/root_ring.rs` 里的 `root_sits_in_the_ring_slot_the_formula_gives`，`git grep` 判。`cut -c1-12` 把多字节的「已还清」切断了，行号不受影响。）

## --check-report 输出末行

```text
$ python3 scripts/kb/stale-candidates.py --check-report /home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-facts/candidates.tsv /home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-s4/result.md --groups F12-F14
rc=0
  ✓ 报告判全了：215 行候选都有逐行判定；crates/ 下判要改要补的 0 行都在「交实现员」一节里（不变量清单的 .md 除外）
```

## 没做什么

- 没改仓里任何文件；没跑门禁、lint、重型测试；没提交。草稿目录里只有本报告、`mine.tsv`、`rec-target.md`、`verdicts.json`、`check-report.txt` 与 `tools/` 下几份辅助脚本和表格片段，没有仓副本、编译目录或库拷贝。
- 「事件句不改」的 137 行里调度记录行的理由是按模板生成的（引行首原话、说是那一次的经过），逐行读过内容，没有逐行另写理由；里面三张排批表（调度记录结束那一版第 69、70、670 行）单独写了理由。
- 乙类那一问（`deleted_inodes` 根的 key 区间）只读了暂存区 `walk.rs` 的 `deleted_inodes_queue_without_judging` 一个函数，没全仓找有没有别处判这个根的区间。
- M3 那一格是不是随 C743 改的，没逐项核（推的）。
- 丙类「暂存区或工作区已改」的判断只核了点名的那几行逐字在、在哪一表，没核那些改动属于哪个会话、会不会被撤回。
- 只罩候选表与反向问找到的；检索词搜不到、又不在做成的事里的说法，搜不到。
