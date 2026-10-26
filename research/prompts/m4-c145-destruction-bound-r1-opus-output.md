# m4-c145-destruction-bound-r1 攻方腿（Opus）报告（2026-10-26）

第一轮，攻击面：正文第三节 B1–B3；角度：过度外推与越位 / 内部矛盾与循环依赖 / 哪一条最脆弱。没读禁读清单里正推腿的产出。

## 复跑

```
bash scripts/hosts/run-with-memory-cap.sh 1G bash research/prompts/m4-c145-destruction-bound-r1-opus-model/tools/rerun.sh --repo <仓根> --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work <新的草稿目录>
```

本机忙时前面加 `RERUN_ON_THE_SECOND_MACHINE=1`（编译与测试经 `scripts/hosts/multi-host-run.sh --peer` 挪到第二台，模型那一段仍在本机）。它先跑抽象模型、与 `out/` 里三份逐字节比，再在冻结副本的拷贝上跑基线与三个变体、退出码与 `out/exit-codes.txt` 逐字节比。

模型目录每个文件的 sha256：

```
8c559474100c5f5ef3c52c19c9058d79e88088db1027052e172c3a4cf1c09538  ./destruction_bound_model.py
9efbc2b550a153a783ca7f7354c9a2a25c00f5544d0ad4a36491382fc0005dc3  ./out/exit-codes.txt
cec74f8c8e1b2370029f0430a0475daa5bf7970ce0a2b7971922d44dbc16337d  ./out/model-product-transaction-depth10.jsonl
a459eb005c67b3e2dd06e37275ced5e8ac0b9e67db093136fca43e049fdde697  ./out/model-summary.txt
31398cb5f3fd2daa29e33620c2d990d7d6cd9a9b4cdb56d5f4ba41dddac06dad  ./out/model-unguarded-depth9.jsonl
5e282a14621ead90a82c88bdf1ed8ff7e55ffe1c1ad4ae0aefe25a16863ccfa3  ./out/rounds/baseline-attack.txt
f0f55ae3184de9d5a39aac22b2730b2dddf4c958f1bcbd940476b5719f0ab7eb  ./out/rounds/baseline-checker-lib.txt
247cdf7f4d60bd734fdca1d4cbb4a5de82991851e304eebf7da3433c02f48955  ./out/rounds/baseline-destroy.txt
354fd6d9730d586809d62ab96047f6785696632b309562e2fb024906bb679b32  ./out/rounds/baseline-must-red.txt
c28734a9d6e1d698b08631137b402b9bc4c80fbdc8ad55f47d1e1e05fc35a504  ./out/rounds/checker_bound_lo_plus_one-attack.txt
19dfa70db80b818dd922c21b4ae613a6cd366110a9e0dd274e62137920103d6b  ./out/rounds/checker_bound_lo_plus_one-checker-lib.txt
c6eb7f40dbec55c50fffed267bc49a78204e4d8360015d22f8a664bfb866dd10  ./out/rounds/checker_bound_lo_plus_one-destroy.txt
42f377a51fcb0fadb30fab7de6b463806d4a2e8261ed5cb4ed6d5020555eafb3  ./out/rounds/checker_bound_lo_plus_one-must-red.txt
ffe672180c4f2c2380f456063df7abe9d2de2454eae87576cd3c9b89cf49bd5e  ./out/rounds/core_hi_lo_plus_one-attack.txt
bc7c986daf159ce3878bb0f3b0182afc5b4bd4c7bfd3aaffbb7b203de88587a8  ./out/rounds/core_hi_lo_plus_one-checker-lib.txt
e400bd8a228b79255054c0c85e0ba73413ed47be8ea44c8db9c4c5dac9b9a0f6  ./out/rounds/core_hi_lo_plus_one-destroy.txt
6d13cf8b778d5d0a173e461d3244dd232b7e564ae90391a5d6b1d48ff320afb1  ./out/rounds/core_hi_lo_plus_one-must-red.txt
bc30c5afb44c6cbc647afd7c35a70d8fe53acb5feda37596b0c5fca444d65a53  ./out/rounds/core_hi_published_at_the_batch-attack.txt
0eebdeccb3b8359e98af7b8fc158bdfd6a868ea0c25092b2f71d99a8027fdaec  ./out/rounds/core_hi_published_at_the_batch-checker-lib.txt
5b42b5c0981098dc52dfa8564d967953229beafce35033831f9e01c1fc1b4cf7  ./out/rounds/core_hi_published_at_the_batch-destroy.txt
0a01be600a9927cc44e7df197cf65a6eb2228997cd2d91e5851242360946d34c  ./out/rounds/core_hi_published_at_the_batch-must-red.txt
9efbc2b550a153a783ca7f7354c9a2a25c00f5544d0ad4a36491382fc0005dc3  ./out/rounds/exit-codes.txt
9485e54bed2458d5d7f39346eccb40d2ae2a8127d006fa2eec5a07d2971b5e9d  ./out/rounds/mutants-applied.txt
2277d0ff366523f1f588fcdca49de1a03e2287e04499ac37d378a63b8afd3523  ./prototype/attack_upper_bound_of_an_open_destruction_intent.rs
b4fd07fa0616e98ab0b49827d28e5f23e2d1d218334f1f68643ad7c476220478  ./tools/mutate.py
2164ebea647459c352588456cab6791e15f92b9f48d662bb612d92e1098d9b89  ./tools/rerun.sh
e4948e3667b5004be98417e7231cd8a19c27a5ca5cd4878bdc6745e24d48f5c6  ./tools/run-rounds.sh
4db94c371e8890c0017b588a68ebac3baffb81c44a07a34dfd35cb413430cc1c  ./tools/summarize_model.py
```

## 各格判定一览

| 格 | 判定 | 一句话 | 量过 / 推的 |
|---|---|---|---|
| B1 | 没打中反例；打中论证（越位） | 第一版可达历史上 ∞、意图 txg、这一批 txg、已发布的号四种判法逐状态同结论，未完成意图那几版里没有纪元 ≥ 意图 txg 的条目；但撑住它的是「意图到最后一批之间建快照被 core 守卫拒」，checker 注释（只论「之后新进来的」）与 core 注释（「都不杀用户可见单元」，事务层为假）各缺一半；守卫一拿掉，8486 个状态有纪元 ≥ 意图 txg 的条目、380 个状态两判法不同，误红的是「意图 txg」判法 | 模型量过（product / transaction 深 10、unguarded 深 9）；副本 9 段删快照 49 版量过 |
| B2 | 打中三处 | ① 以 ∞ 为上界时 C145 第二条判定是空真，纪元抬过这一版 txg 的坏镜像在意图那一版一条都不红（要等最后一批之后才红）；② C145 登记的判别力自证「把上界改成已发布的号」在实现里是等价变体，副本上四个目标全绿；③ 缺「纪元 > lo 的待放条目」这种不许红的世界，checker 待放上界收成 lo + 1 只有攻方原型红 | ① ② ③ 副本与模型都量过 |
| B3 | 打中一处（条件） | 第一版不用加格式；串行放开之后「意图里存 hi = 意图 txg」是错的量（core 按批现取的 hi 会越过它），除非 core 同时改成定死 hi | 模型量过 380 个误红状态；「core 同改就对」是推的 |

攻方自己提的三条收严（待放上界取被判那一版的 txg 并另判「纪元 < 这一版 txg」；相接两版新条目纪元 = 上一版活头前驱；存 hi 要配 core 定死 hi）：只在我的模型上量过或只是推的，被攻过零轮。

## 跑在哪台、等了多久

开工时 `ps -o pid,args -u "$(id -u)"` 看到别的会话的几条 `cargo test`、崩溃放量双机那一条在跑，没有性能测量。本机编译经内存包装排队 25 分钟没起跑（包装每分钟打一行「slice 已占 37–40 GiB，这一条要 8.0 GiB，总上限 40.1 GiB」）；`python3 scripts/hosts/host-resources.py -- <那条 cargo 命令>` 报本机「可用 0 线程」「放不下：差内存 9.5 GiB」、第二台「可用 32 线程」「放得下」。于是停掉自己那一条排队的编译（`proc.py stop`，只停了它的准入进程，整条随之退出），改经 `scripts/hosts/multi-host-run.sh --peer --memory-cap 8G` 在第二台跑：第一趟（`run-rounds.sh` 一次跑完基线与三个变体）退 0、但攻方原型里有一句断言写错（`middle_of_three` 删最旧那个时并没有纪元 > lo 的待放条目），改掉那一句重跑第二趟，这一趟的输出入库在 `out/rounds/`。取回的输出里第二台的仓目录换成了「〔第二台的仓目录〕」（换掉的是第二台上的绝对路径前缀）。

模型的挂钟：先跑深 6（全部三种调度 0.4 秒）、深 9（48 秒）估量，定稿那一趟 product / transaction 深 10 加 unguarded 深 9 共 108 秒。

rerun.sh 在新 shell 里第一次起（`env -i` 起的）退 251（内存包装起不来：`systemctl --user set-property` 连不上会话总线，是 `env -i` 清掉了环境）；那一次不算结果，去掉 `env -i` 照同一条命令重起，结果见文末。
## B1　等价论证成不成立

### 结论

没打中「两种判法结论不同」。在第一版可达的历史上（会话路径，与直接调发布函数的事务层路径），下一侧是活头时「按 ∞ 判」「按写意图那次的 txg 判」「按这一批自己的 txg 判」「按已发布的号判」给的待放集与 checker 结论逐状态相同；更强的一句也成立：未完成意图那几版的 deadlist 里一条纪元 ≥ 意图 txg 的条目都没有。

但撑住它的不是注释写的那条理由。打中的是论证本身（过度外推 / 越位）：

1. 注释只论证了「之后新进来的条目纪元都小于 lo」，没论证「意图那一刻已在的条目纪元都小于意图 txg」；真正撑住结论的是两件事：纪元总是某次建快照那次发布的 txg（prev(头)），而建快照在第一版被 core 的守卫挡在意图与最后一批之间。注释原文（冻结副本）：「所以下一侧是头时按 ∞ 判与按写意图那次的 txg 判，盘上读得到的条目上结论相同）。」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/walk.rs:16683`），它的前提一句：「birth > P 的条目；hi 在下一侧是快照时取它的 S.txg，是头时取写意图那次发布的 txg——写意图之后头的前驱已是 P < lo，之后新进来的条目纪元都小于 lo，」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-checker/src/walk.rs:16682`）。「头的前驱已是 P」只在意图之后不再建快照时一直成立；守卫在「SnapshotCreationRefusal::SnapshotBeingDestroyed { being_destroyed },」（`crates/singlefs-core/src/transaction.rs:9450`）与「SnapshotDestructionRefusal::AnotherSnapshotIsBeingDestroyed { being_destroyed }」（`crates/singlefs-core/src/transaction.rs:9527`）。拿掉这两道守卫（模型里的 unguarded 调度），「之后新进来的条目纪元都小于 lo」就不成立：8486 个状态里有纪元 ≥ 意图 txg 的条目（量过，见下表），第一例 `snapshot, intent1, snapshot, overwrite0, intent2`。
2. core 那边给同一结论写的理由是另一条，而且在事务层不成立：「都不杀用户可见单元，纪元 ≥ 写意图那次 txg 的条目一条都不会有，两种取法放的是同一组。余下的条目键不动，它们按纪元落进下一侧的那一段」（`crates/singlefs-core/src/transaction.rs:6166`）。第一个文件的覆盖写 `publish_overwrite` 不核承诺类队列（第一个文件之外的整份写在 `plan_of_a_whole_write_of_a_file_other_than_the_first` 里有断言，`crates/singlefs-core/src/transaction.rs:9389` 起），直接调发布函数时意图与最后一批之间可以插覆盖写、真的杀用户可见单元（副本上跑过，见「副本上的实跑」一节的 `overwrites_inside_the_destruction`：四次覆盖写插在四批之间，被杀的单元 birth 7 > P = 4、直接释放，没有新条目）。结论照样成立：被杀单元要么直接释放、要么以纪元 P < lo 进 deadlist（后一种副本上没造出来，模型 transaction 调度里有）；会话路径先走空：「if !current_version_of_trees.queue_of_committed_intents_is_empty() {」（`/home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen/crates/singlefs-core/src/mounted_session.rs:2308`），走不到这一形。
3. 「真实的 hi」在仓里有三种写法，B1 说的「按写意图那次的 txg 判」只是其中一种：kb 表里是「写意图那个开放 checkpoint 的号」（`.claude/kb/decisions/05-快照-空间记账机制.md:283`），kb 正文是「下一侧是头时取写意图那次发布的 txg），不存续做点」（`.claude/kb/decisions/05-快照-空间记账机制.md:291`），core 实际传的是这一批自己的 txg：「deadlist_entries_the_destruction_of_a_snapshot_releases(previous, snapshot, txg).len();」（`crates/singlefs-core/src/transaction.rs:9577`）与「Some(NextSideOfASnapshot::LiveHead) | None => hi_when_the_next_side_is_the_live_head,」（`crates/singlefs-core/src/transaction.rs:6182`）。第一版里三种等价；守卫一拿掉就不等价，而且出错的是 kb 那一种：unguarded 调度下 ∞ 判法与「意图 txg」判法有 380 个状态结论不同，第一例 `snapshot, snapshot, intent2, intent1, snapshot, overwrite0, intent3` 里「意图 txg」判法报 `I-3.5-reverse:6:0`，而 core 按批现取的 hi 在 S3 删完之后会把这一条放掉——误红的是「意图 txg」判法，∞ 与 core 的按批现取一致。
4. 第三个依赖：core 的派生链不看「销毁中」记号：「的记号：第一版删快照串行，带记号的只有正在删的这一条」（`crates/singlefs-core/src/transaction.rs:6132`），checker 的下一侧只数活快照（`next_live_side_upper_bound_after`）。两边只在「至多一个销毁中」时一致；unguarded 下第一例 `snapshot, snapshot, intent2, intent1, batch1, overwrite0` 里 S1 的意图把前驱 0 传给了销毁中的 S2、活头的前驱停在 2，之后杀进来的条目永远放不掉（820 个红状态，∞ 判法照红，不分辨上界）。

### 怎么造的、取样多大

抽象模型 `destruction_bound_model.py`（照冻结副本 core 的写法：覆盖写换下一个单元按 birth ≤ prev(头) 进 deadlist、建快照前驱 = prev(头)、意图把 P 传给派生链上的下一条码 14 或活头、批大小 1、能放的不多于一条就是最后一批），三种调度、七种 core 的「下一侧是活头时 hi」、五种 checker 的上界，逐状态比 checker 结论（④、逆向 ③、I-3.6、③ 恰好一条，照 `judge_the_deadlist_against_the_head_and_the_live_snapshots` 的判法）与释放侧的错。用户决定的动作全放开：每一步可以覆盖写两个位置之一、建快照（至多 3 个）、对任一快照写意图、走一批；product 调度只固定「意图之后走批到底」（会话的走空；崩在任一次发布之后续做走的是同一个计划，所以崩溃续做落在 product 的状态集里）。宽 2 个单元位置，product 与 transaction 深 10 步，unguarded 深 9 步。

原样输出（`out/model-product-transaction-depth10.jsonl`、`out/model-unguarded-depth9.jsonl` 按字段摘成一行一种，摘法是 rerun.sh 里那一段 python）：

```
product batch_txg states 68897 open 34305 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
product published_at_the_batch states 68897 open 34305 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
product intent_txg states 68897 open 34305 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
product infinity states 68897 open 34305 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
product published_at_the_intent states 68897 open 34305 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 749, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
product lo_plus_one states 69066 open 34220 bugs 0 epoch>=I 0 red {'infinity': 274, 'intent_txg': 274, 'lo_plus_one': 918, 'version_txg': 274, 'version_txg_plus_one': 274} diff {'intent_txg': 0, 'lo_plus_one': 644, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'infinity': 7, 'intent_txg': 7, 'lo_plus_one': 6, 'version_txg': 7, 'version_txg_plus_one': 7}
product lo states 80258 open 37042 bugs 0 epoch>=I 0 red {'infinity': 11746, 'intent_txg': 11746, 'lo_plus_one': 12836, 'version_txg': 11746, 'version_txg_plus_one': 11746} diff {'intent_txg': 0, 'lo_plus_one': 1094, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'infinity': 4, 'intent_txg': 4, 'lo_plus_one': 4, 'version_txg': 4, 'version_txg_plus_one': 4}
transaction batch_txg states 118808 open 84216 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 1504, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 1504, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
transaction published_at_the_intent states 118808 open 84216 bugs 0 epoch>=I 0 red {'infinity': 0, 'intent_txg': 0, 'lo_plus_one': 1504, 'version_txg': 0, 'version_txg_plus_one': 0} diff {'intent_txg': 0, 'lo_plus_one': 1504, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'lo_plus_one': 6}
unguarded batch_txg states 163858 open 145678 bugs 0 epoch>=I 8486 red {'infinity': 820, 'intent_txg': 1186, 'lo_plus_one': 4242, 'version_txg': 820, 'version_txg_plus_one': 820} diff {'intent_txg': 380, 'lo_plus_one': 3448, 'version_txg': 0, 'version_txg_plus_one': 0} reddepth {'infinity': 6, 'intent_txg': 6, 'lo_plus_one': 6, 'version_txg': 6, 'version_txg_plus_one': 6}
```

（只摘了 10 行；transaction 与 unguarded 其余各行与同调度 batch_txg 那一行的「states / red / diff」逐项相同或只差对照变体，全文在两份 jsonl 里。）

读法：`red` 是 checker 按那种上界判出至少一条红的状态数；`diff` 是与 ∞ 判法结论不同的状态数；`bugs` 是提早释放或重复释放；`epoch>=I` 是有一条（下一个活侧是活头的）未完成意图、deadlist 里有纪元 ≥ 它意图 txg 的条目的状态数。

### 回退

第一版回退在当前版或目标版树表里有码 14 条目（销毁中的也算）时拒：「let snapshots_in_the_current_version = snapshots_in_the_tree_table(current);」（`crates/singlefs-core/src/mount.rs:9681`），计数函数只按种类数、不看记号（`crates/singlefs-core/src/mount.rs:9016` 起）。所以「回退」这一支在 B1 里没有历史可造，模型不含回退。

### 四句

- 分不分辨臂：B1 的两种判法在可达历史上不分辨（结论逐状态相同）；打中的三处（理由不完整、core 理由在事务层为假、「真实 hi」三种写法）都不分辨 ∞ 与「意图 txg」两条臂，打的是论证与条款措辞。
- 系统看不看得到判别它的东西：意图 txg 盘上读不出（kb `:291` 不存续做点）；但在第一版它不需要看到，因为可达镜像上没有落在 [意图 txg, ∞) 的条目。
- 满足判据字面哪个分句：B1 格「找得到反例就给出那段历史与两种判法各判什么」——没找到；「盘上读得到的条目里有一条纪元 ≥ 真实 hi 而 ∞ 判法放过」只在假想的 unguarded 调度里出现，那一形不是第一版能写出的历史。
- 跑前条款的改法：正文没给跑前改法，这一句不适用。
## B2　成立时的形态：攻推荐形态的判别力

B1 在第一版成立，正文的两种候选形态都要攻。今天实现的是第一种（∞），C145 的第二条判定原文是「未完成意图的区间里没有纪元 ≥ 上界的条目」（`.claude/kb/checks-owed.md:121`）。

### 打中 B2-1：「以 ∞ 为上界判」让 C145 第二条判定成了空真，改坏纪元的一类必红世界在意图那几版上不红

以 ∞ 为上界时，「区间里没有纪元 ≥ 上界的条目」这一句对任何镜像都成立，判不出任何东西。具体的漏法：意图那一版（唯一的快照、下一侧是活头）上 deadlist 每一条的纪元改成远大于这一版 txg 的值，这些条目照样落在 [lo, ∞) 里、birth > P，进了待放集：逆向 ③ 与 I-3.6 豁免都放过。合法镜像上纪元 = prev(头) < 写它那次发布的 txg ≤ 被判那一版的 txg，所以「纪元 ≥ 这一版 txg」是合法镜像上不可能出现的值。

副本上的原样输出见「副本上的实跑」一节 `forged_epochs_far_above_the_version_txg_inside_an_open_intent`：意图那一版（txg 6，意图 txg 6）四条纪元各加 10⁶ 之后 `violations_after=0`，一条都不红（量过）；同一张坏镜像可写挂载续做到最后一批之后（txg 9）I-3.5 逆向与 I-3.6 各红一条（量过，两行原样在实跑一节）。

这条坏镜像在意图那几版上漏，等最后一批删掉那一对条目之后才红（这些条目纪元 ≥ 这一批的 txg、core 不放，快照没了它们就落进活头那一侧、birth > P，I-3.6 红）——延后到最后一批，不是永远漏；删快照停在中间（批被拒、一直续不完）时一直不红。

四句：分辨臂——分辨（∞ 不红，下面的「这一版 txg」判法红）；系统看得到判别它的东西——看得到，被判那一版自己的 checkpoint txg 在根上；满足的分句——B2 格「改坏它时哪个必红世界会红」；跑前改法——正文没给。

### 打中 B2-2：C145 登记的判别力自证「把上界改成已发布的号，BFS 必须在 5 步内判红」在实现里是等价变体，不会红

原文「把上界改成已发布的号，BFS 必须在 5 步内判红」（`.claude/kb/checks-owed.md:121`），条款里的理由是「取已发布的最大号会让区间为空、条目永远不放」（`.claude/kb/decisions/05-快照-空间记账机制.md:283`）。实现里纪元 = 某个快照那次独立发布的 txg，被快照引用的单元在建快照之后的发布里才被杀，意图又在最后一次杀之后，所以最大纪元 ≤ 意图 txg − 2：

- 按批现取「已发布的号」（这一批之前那一版的 txg ≥ 意图 txg）：模型 `published_at_the_batch` 在 product、transaction 两种调度深 10 步内 0 个红状态（上面 B1 的表）；副本上的 core 变体 `core_hi_published_at_the_batch`：四个测试目标全绿（删快照用例 15 个、必红世界 11 个、checker 走读单测 62 个、攻方原型 6 个，汇总行在实跑一节；量过，副本）。
- 连定死在意图那一刻的「已发布的号」（意图 txg − 1，要存进意图才做得到）也是 0 个红状态（模型 `published_at_the_intent`）。
- 对照：区间收成 [lo, lo + 1) 或 [lo, lo) 的两个变体在模型里分别在第 7 步、第 4 步红（`lo_plus_one`、`lo` 两行的 `reddepth`）；副本上的 core 变体 `core_hi_lo_plus_one`：删快照的已有用例 `destroying_the_newer_snapshot_keeps_what_the_older_one_references_and_destroying_the_older_one_releases_it` 红、攻方原型 `older_after_newer` 续做之后池级 checker 红，必红世界与 checker 单测不红（量过，副本）。

所以这一条判别力自证照 d5-item12 模型那一套写进 C145，搬到实现上没有一个必红世界能对它红。四句：分辨臂——这一条打的是判据（自证）本身，不分辨 ∞ 与「意图 txg」两条臂；系统看不看得到——不涉及；满足的分句——「判别力自证：…把上界改成已发布的号，BFS 必须在 5 步内判红」这一分句；改法——要换成实现里会红的变体（区间收窄到 lo + 1，配一段「删新快照再删旧快照」的历史），见下面 B2-3。

### 打中 B2-3：缺「纪元 > lo 的待放条目」这一种不许红的世界，把 ∞ 收窄成 lo + 1 的 checker 变体没有必红世界能抓

「删新快照之后再删旧快照」这段历史里，删旧快照 S1 时待放集里有纪元 = S2.txg > lo 的条目（S2 删完之后留下的、birth ≤ prev(S2) 的那几条）。checker 上界只要比 S2.txg 小就会误红。模型里 checker 上界收成 lo + 1 在 product 调度第 6 步就误红，第一例 `snapshot, snapshot, overwrite0, intent2, batch2, intent1`（报 `I-3.5-reverse:3:0`、`I-3.6:3:0:head`）。副本上把 checker 的待放上界改成 lo + 1（`checker_bound_lo_plus_one`）：已有的三个目标（删快照用例 15 个、必红世界 11 个、checker 走读单测 62 个）全绿，只有攻方原型 `older_after_newer` 在意图那一版就红（纪元 5 > lo = 4 的条目报 I-3.5 逆向与 I-3.6，原样在实跑一节；量过，副本）。改法（只在我的模型与副本上量过、被攻过零轮）：把 `older_after_newer` 这段历史（建两个快照、覆盖写、删新的、再删旧的，每一版过池级 checker）收成一条不许红的世界，或在 `walk.rs` 测试模块里加一个纪元 = S2.txg > lo 的待放条目的合成世界。

### 两种形态的对照（「量过」= 贴了副本或模型的原样输出；「推的」= 按代码推、没跑）

| 形态 | 可达镜像上误红 | 纪元改成 ≥ 这一版 txg 的坏镜像（意图那几版） | 「已发布的号」core 变体 | 依据 |
|---|---|---|---|---|
| ∞（今天） | 0（量过，模型三种调度） | 不红（量过，副本：意图那一版 0 条违例） | 不红（量过，模型；副本 四个目标全绿） | B1 表、本节 |
| 被判那一版的 txg V（判「待放上界 = V」并另立一句「每条纪元 < V」） | 0（量过，模型 `version_txg` 三种调度 diff 都是 0） | 红（推的：逆向 ③ 与 I-3.6 豁免都不再放过；副本上没实现这条判法） | 不红（推的：这个变体等价，什么判法都抓不到） | 只在我的模型上量过、被攻过零轮 |
| 意图 txg（盘上读不出） | 0（量过，模型 product / transaction）；unguarded 下 380 个状态误红 | 红（推的） | 不红 | 不可判，列着只作对照 |

### 推荐形态（攻方自己提的收严，只在我的模型上量过、被攻过零轮）

- 规则：下一个活侧是活头时，待放集的上界取被判那一版自己的 checkpoint txg V（不取 ∞），并在 I-3.6 下另判一句「这一版 deadlist 里每一条的纪元 < V」。理由是纪元 = 杀它那次发布之前那一版的 prev(头) < 杀它那次的 txg ≤ V，这一句在每一张合法镜像上都成立（模型三种调度里 `version_txg` 与 ∞ 结论处处相同）。
- 挂哪条：待放上界那一处同时被 I-3.5 逆向 ③ 与 I-3.6 豁免用，挂在 I-3.6 的豁免句里写明「下一侧是活头时 hi 取被判那一版的 txg」；「纪元 < V」是条目自身的良构，挂 I-3.6（它管的是条目与它那一侧的前驱，纪元越界同属「条目落进哪一侧」）。
- 必红世界：B2-1 那张坏镜像（意图那一版纪元整体抬过 V）要在意图那一版就红；不许红的世界：B2-3 那段「删新快照再删旧快照」历史逐版不红。
- 它判不了的：纪元落在 [意图 txg, V) 之间的坏条目（批走过几次之后 V > 意图 txg），这一段仍要意图 txg 才判得了，盘上读不出。
## B3　不成立时要加什么

B1 在第一版成立，这一格按正文是「不成立时」才答；攻方只攻「要加什么」那几条候选在什么条件下才对，不给第一版加格式。

### 打中 B3-1：「意图里存 hi（= 意图 txg）」在串行一放开就是错的量

D5 已定项 12 的销毁一行写着「级联之间不要次序，销毁头的意图等同一个头上更早的级联做完才动条目」（`.claude/kb/decisions/05-快照-空间记账机制.md:283`），也就是条款本身预见了不串行的那一天。到那一天，core 今天的写法（每批现取 hi，下一侧是活头时取这一批自己的 txg）放的条目会越过意图 txg：模型 unguarded 调度第一例 `snapshot, snapshot, intent2, intent1, snapshot, overwrite0, intent3`，纪元 6 的那一条在 S3 删完之后由 S2 的批放掉，而按存下来的意图 txg 判它不在待放集里，checker 误红（380 个状态，量过）。所以「存 hi」只有在 core 同时改成「hi 在意图那一刻定死、之后不再现取」时才是对的量；两边不一起改，存了也判错。这是推出来的条件，改法没实现、没跑。

### 盘上最少多一个什么量

- 第一版：不用加（B1）。B2 推荐的「上界取被判那一版的 txg」不改格式，盘上已有。
- 串行放开之后：要判「纪元落在 [意图 txg, 这一版 txg) 之间的坏条目」，判别子是意图 txg 本身（8 字节）。kb 里给它留的地方：「| 树表字节 | 0；预留 76 字节留给别的认购者 |」（`.claude/kb/decisions/05-快照-空间记账机制.md:286`）；第一版意图的载体是码 14 条目「只占 flags 位 0（销毁中），预留一个字节不写」（`.claude/kb/decisions/05-快照-空间记账机制.md:291`）。写进码 14 条目的预留字节改的是树表条目字段表的语义，落在 D15 冻结政策管的字段表上（D15 已定项 3：`spec_hash` 对 kb 字段表的完整投影求，那一节从 `.claude/kb/decisions/15-格式冻结政策.md:54` 起）；第一版冻结之前加是改字段表，冻结之后加是一次格式变更——推的，没核 D15 已定项 9 的判定表逐行落在哪一格。

### 不改格式的替代判法（攻方提的，推的、零轮）

相接两版 (R⁻, R) 都在回退候选集里时，R 里新出现的 deadlist 条目，纪元必须等于 R⁻ 的活头前驱（core 的杀法：birth ≤ 上一版活头前驱的进 deadlist、纪元就是那个前驱）。它把「意图之后没有落进 [lo, ∞) 的新条目」这一前提逐对判掉，串行放开之后也判得动；判不了的是候选集里最老那一版之前进来的条目（归纳的起点不在盘上）。没实现、没跑。
## 副本上的实跑（冻结副本的拷贝，第二台；副本上的数，不算入库装置上的数）

原型 `prototype/attack_upper_bound_of_an_open_destruction_intent.rs`：两块内存稀疏盘（`SparseBlockDevice`）上 mkfs、取号、暖机、第一个文件，之后直接调发布函数造六段历史；删快照逐批（批大小压成 1）走，每一版打一行 `name=attack_version`（意图 txg、lo、P、各条（纪元, birth）、`max_epoch_below_intent_txg`、池级 checker 判违例的条数），并在这一版的盘拷贝上可写挂载一次（崩在这一版之后的续做），挂完再过池级 checker。`run-rounds.sh` 跑基线与三个变体（`tools/mutate.py` 逐字替换、旧串恰好命中一次），每一趟四个测试目标。

基线里 49 行 `name=attack_version`，`max_epoch_below_intent_txg=true` 49 行、`=false` 0 行；`violations=0` 49 行；`violations_after_the_continuation=0` 49 行（`grep -c` / `grep -o … | sort | uniq -c` 在 `out/rounds/baseline-attack.txt` 上数的；另有 1 行 `violations_after_the_continuation=2` 是纪元改坏那张镜像续做之后的，不是 `attack_version` 行）。

每一趟四个目标的 libtest 汇总行（`out/rounds/<趟>-<目标>.txt` 里 `^test result` 那一行原样）：

```
baseline attack test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 36.32s
baseline destroy test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.63s
baseline must-red test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.62s
baseline checker-lib test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 20.47s
core_hi_published_at_the_batch attack test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.71s
core_hi_published_at_the_batch destroy test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.53s
core_hi_published_at_the_batch must-red test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
core_hi_published_at_the_batch checker-lib test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 11.91s
core_hi_lo_plus_one attack test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 31.66s
core_hi_lo_plus_one destroy test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.64s
core_hi_lo_plus_one must-red test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.60s
core_hi_lo_plus_one checker-lib test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 11.64s
checker_bound_lo_plus_one attack test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 33.16s
checker_bound_lo_plus_one destroy test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.60s
checker_bound_lo_plus_one must-red test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
checker_bound_lo_plus_one checker-lib test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 49 filtered out; finished in 11.53s
```

`older_after_newer` 删旧快照写意图那一版（基线，原样）：

```
name=attack_version history=older_after_newer step=intent txg=9 chain=[(4, TreeIdentifier(65549), 0)] head_previous=0 being_destroyed=[TreeIdentifier(65549)] lo=4 predecessor=0 intent_txg=9 deadlist_entries=4 epochs=[(5, Some(3)), (5, Some(3)), (5, Some(3)), (5, Some(3))] max_epoch=5 max_epoch_below_intent_txg=true pending_entries_with_an_epoch_above_lo=4 violations=0 continuation_snapshots_left=0 continuation_deadlist_entries_left=0 violations_after_the_continuation=0
```

纪元改坏的世界（基线，原样；`out/rounds/baseline-attack.txt`）：

```
name=attack_forged_epoch_continuation txg=9 snapshots_left=0 deadlist_entries_left=4 violations_after_the_continuation=2
name=attack_forged_epoch_continuation_violation I-3.5:根（实例 2、txg 9）：deadlist 里一条（头 65537、纪元 1000004）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）
name=attack_forged_epoch_continuation_violation I-3.6:根（实例 2、txg 9）：deadlist 里一条（纪元 1000004）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） birth 3 大于它归属那一侧 活头（inode 树 65537） 的前驱 0（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里
```

checker 待放上界收成 lo + 1（`checker_bound_lo_plus_one`）时攻方原型判出的违例（去重，原样）：

```
name=attack_violation history=older_after_newer step=batch1 I-3.5:根（实例 1、txg 9）：deadlist 里一条（头 65537、纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）
name=attack_violation history=older_after_newer step=batch1 I-3.6:根（实例 1、txg 9）：deadlist 里一条（纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） birth 3 大于它归属那一侧 活头（inode 树 65537） 的前驱 0（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里
name=attack_violation history=older_after_newer step=batch2 I-3.5:根（实例 1、txg 9）：deadlist 里一条（头 65537、纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）
name=attack_violation history=older_after_newer step=batch2 I-3.6:根（实例 1、txg 9）：deadlist 里一条（纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） birth 3 大于它归属那一侧 活头（inode 树 65537） 的前驱 0（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里
name=attack_violation history=older_after_newer step=batch3 I-3.5:根（实例 1、txg 9）：deadlist 里一条（头 65537、纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）
name=attack_violation history=older_after_newer step=batch3 I-3.6:根（实例 1、txg 9）：deadlist 里一条（纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） birth 3 大于它归属那一侧 活头（inode 树 65537） 的前驱 0（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里
name=attack_violation history=older_after_newer step=batch4 I-3.5:根（实例 1、txg 9）：deadlist 里一条（头 65537、纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）
name=attack_violation history=older_after_newer step=batch4 I-3.6:根（实例 1、txg 9）：deadlist 里一条（纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） birth 3 大于它归属那一侧 活头（inode 树 65537） 的前驱 0（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里
name=attack_violation history=older_after_newer step=intent I-3.5:根（实例 1、txg 9）：deadlist 里一条（头 65537、纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） 没有一个活快照引用它，也不在未完成销毁意图的待放集里（③ 逆向）
name=attack_violation history=older_after_newer step=intent I-3.6:根（实例 1、txg 9）：deadlist 里一条（纪元 5）钉的单元 （码 1、出生树 65536、出生 txg 3、实例 1、事务号 1） birth 3 大于它归属那一侧 活头（inode 树 65537） 的前驱 0（该直接释放的块进了 deadlist），也不在未完成销毁意图的待放集里
```

core 区间收成 [lo, lo + 1)（`core_hi_lo_plus_one`）时已有用例红在哪一句（原样）：

```
thread 'destroying_the_newer_snapshot_keeps_what_the_older_one_references_and_destroying_the_older_one_releases_it' (407269) panicked at crates/singlefs-harness/tests/snapshot_release_and_destroy_through_the_deadlist.rs:508:5:
再删 S1：落在它那一段（纪元 S2.txg ∈ [S1.txg, hi)）、birth > 0 的四条全放
```
## 没打中的形状

| 形状 | 取样范围 | 结果 |
|---|---|---|
| 会话路径下 ∞ 与意图 txg / 这一批 txg / 已发布的号 / 这一版 txg 判法结论不同 | 模型 product 调度，两个单元位置、至多 3 个快照、深 10 步，68897 个状态（34305 个带未完成意图），用户动作全放开 | 0 个状态不同 |
| 事务层（直接调发布函数，意图与最后一批之间插覆盖写）下结论不同 | 模型 transaction 调度，深 10 步，118808 个状态 | 0 个状态不同；副本上 `overwrites_inside_the_destruction` 一段历史逐版 checker 不红（见实跑一节） |
| 盘上读得到的条目里纪元 ≥ 意图 txg 而 ∞ 放过 | 同上两种调度 | 0 条；副本上 9 段删快照（`only`、`older_after_newer`、`newest_with_an_older_live`、`older_after_the_newest_with_entries_of_two_epochs`、`middle_of_three`、`newest_of_two`、`oldest_last`、`overwrites_inside_the_destruction`、`older_after_overwrites_inside_the_destruction`）共 49 版逐版 `max_epoch_below_intent_txg=true` |
| 崩在意图与最后一批之间任一次发布之后、可写挂载续做 | 副本上上面每一段历史的每一版各挂一次 | 续做之后快照与 deadlist 都清空、池级 checker 不红（见实跑一节）；崩在一次发布的写流中间（撕裂、子集）没枚举，见限度 |
| 回退牵进销毁中的快照 | 读代码 | 第一版有码 14 条目就拒回退（`crates/singlefs-core/src/mount.rs:9681`），没有可造的历史 |
| 纪元改成杀它那次的 txg（不用 prev(头)） | 推的 | 与 prev(头) 落在同一侧、同一个 [lo, hi) 里，第一版是等价写法，不是漏判 |
## 这条腿自己的限度

- 崩溃状态没照崩溃点重放的枚举域跑。原因：任何带「建快照 + 覆盖写 + 删快照」的历史都有 16–26 个写的单元段，单一个 26 写的段全展开就是 2²⁶ 个子集，超过每次约 10⁶ 的上限（照定义第 3b 步「一段历史自己就超过约 10⁶ 的，那段不跑」）。这一句是推的：没在原型里调 `crash_state_count_with_torn_in_place_overwrites` 实算（原型要依赖 checker 档包，这一轮没建）。改成在每次发布的边界上「崩、可写挂载续做」：恢复落到的是已发布的某一版，续做从那一版走同一个计划；撕裂与子集落到的版本集合与此相同这一句是推的。仓里已有删快照崩溃续做的枚举流（checker 档），交主 agent 判要不要拿它核。
- 模型宽 2 个单元位置、至多 3 个快照、深 9–10 步；模型没有截断、unlink、整理与多文件，这几样在会话里都先走空，按代码推它们不在意图与最后一批之间发生。
- 副本上的数是副本的数，不算入库装置上的数；副本上只跑了点名的几个测试目标，没跑 checker 档。
- B2、B3 里攻方提的三条（上界取这一版 txg 加「纪元 < V」、相接两版新条目纪元 = 前驱、存 hi 要配 core 定死 hi）只在我的模型上量过或只是推的，被攻过零轮。
- 没读正推腿的产出（禁读清单）。

## 交回前的整份复跑

新 shell 里起的命令（编译与测试挪到第二台；草稿目录是新的）：

```
bash -c "cd /tmp && RERUN_ON_THE_SECOND_MACHINE=1 nice -n 19 bash /home/fy5090/code/singlefs/scripts/hosts/run-with-memory-cap.sh 1G bash /home/fy5090/code/singlefs/research/prompts/m4-c145-destruction-bound-r1-opus-model/tools/rerun.sh --repo /home/fy5090/code/singlefs --frozen /home/fy5090/.local/share/singlefs/work/m4-close-rounds-frozen --work /home/fy5090/.local/share/singlefs/work/m4-c145-destruction-bound-r1-opus/rerun2"
```

退出码 0（内存包装排队 175 秒后起跑）。输出里判结果的几行（原样）：

```
name=model_output_matches file=model-product-transaction-depth10.jsonl
name=model_output_matches file=model-unguarded-depth9.jsonl
name=model_output_matches file=model-summary.txt
name=exit_codes_match file=exit-codes.txt
```

`exit-codes.txt` 十六行与入库的 `out/exit-codes.txt` 逐字节相同（基线四个目标全 0；`core_hi_published_at_the_batch` 全 0；`core_hi_lo_plus_one` 攻方原型与删快照用例 101；`checker_bound_lo_plus_one` 只有攻方原型 101）。退非 0 的件数：0。

## 没做什么

- 没跑任何门禁阶段、checker 档测试与崩溃点重放；没起 GPU。
- 没照崩溃点重放的枚举域枚举崩溃状态（理由与替代见「这条腿自己的限度」第一条）。
- B2 推荐的「待放上界取这一版 txg、另判纪元 < 这一版 txg」没在副本上实现，它对坏镜像红是推的；B3 的跨版判法与「存 hi 配 core 定死 hi」都没实现、没跑。
- 没判正推腿那几格，没替主 agent 采纳；副本上的数不算入库装置上的数，要引得在入库装置上重做。
- 草稿目录里冻结副本的拷贝（`tree/`，41M）交回前删掉；本机没有建过编译目录（本机那一次编译一直在排队，没起跑），第二台上的树与编译目录由 `multi-host-run.sh` 每趟收尾时删掉（它打了「删了第二台 … 这一趟的树」那一行）。
