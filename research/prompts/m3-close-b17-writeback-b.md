# 批 17 阶段同步写回：里程碑、验证落地与不变量清单（2026-10-26）

## 结论

六条都关掉了：改了 17 个文件共 32 处，三语每组三份一起改。另外三处工作区已经是新句，没动：verification-build 第 6、130 行（三语），I-1.1，以及「14 条」那一句。`kb-spec-check.py` 绿；`replace-batch.py` 先 dry-run，再实写，回读一致。没写变更史，没跑 `doc-decisions.sh --write`：这一轮是并行写回，也没碰决策正文。

推翻条件：`crash_enumeration_fixed_script_stream.rs` 不再钉 `1_678_377_095` / `1_678_377_220`（第 101、107 行）；或者 `crash_enumeration_new_pool_file_creation_stream.rs` 不再钉 `16_777_243` / `16_777_268`（第 774、778 行）；或者 `new_pool_file_creation_publish.rs` 第 378、414 行不再是 `(6, "2+2", 7)` 和 61 步 16_777_243。出现任何一种，里程碑那几处的新数就站不住。另外，`image.rs` 的 `IMPLEMENTED_INVARIANTS` 下标 104、105 如果不再是 I-7.18、I-7.19，README 补的那一句就要改。

## 规格

- `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-b/spec.json` sha256 `47b60039cb52e7438345259a5547a65480a41613f452ac8fb288653e5f0aea6b`
- `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-b/spec.md` sha256 `9562d036e336aa5871186226e455055f0a2e281fd82df872723c43ad9238a121`
- 喂 replace-batch 的 `/home/fy5090/.local/share/singlefs/work/m3-close-b17-sync-fix-b/batch.json` sha256 `a7015319323f3134825a1008e711435ba0bfc91f84a7b4a172efd6331e90523d`（与 spec.json 同内容，路径写成绝对路径）
- `kb-spec-check.py spec.json` 与 `spec.md` 两次都 rc=0，末行原样：

```text
  ✓ 规格 32 条（1 份）：旧串各恰好 1 次，编号引用、指代、依据配对、依据没有别的会话还没提交的改动、变更史形态都过（查了 32 条）
```

- 写范围闸预检：17 个路径逐个喂 `write-guard.sh`，退出码全 0。
- `replace-batch.py --dry-run` rc=0：`✓ 32 处都命中得对（17 个文件），--dry-run 没写`
- 实写前 `sha256sum -c sha-start.txt` 全部 OK（开工以来没有别的会话改过）；实写 rc=0：`✓ 32 处替换，写了 17 个文件（改名换上新 inode），回读一致`
- 另外自己回读了一遍：32 条新整行各命中 1 次，旧整行 0 次。

## 逐条改了哪几处

| 派发条 | 文件:行（工作区现行号） | 改了什么 |
|---|---|---|
| 1 | `verification-build.md:271`、`.en.md:224`、`.ja.md:224` | 「解析出的依赖版本」改成「解析出的闭包真引到的依赖版本」。英文是 resolved dependency versions genuinely referenced by the closure，日文是 解決されたクロージャが実際に参照する依存バージョン，用词照 D13（验证路线） 英日两版已定项 17 的写法。第 287 行（英日第 240 行）没动：s1 判它是事件句 |
| 2+4 | `milestone/01-first-txn.md:161`、`.en.md:134`、`.ja.md:134` | 三处：①世代号改成取号两轮写 2（槽 0）、3（槽 1），暖机写 4 / 5，事务写 6 槽 0；②验收括注改成与八那一节逐字相同（取号 `2+2`、整条 `2+2+2+1+2+2+1+2+24+2+1+2`、闭式 16777243），E142 第八次跑产物另写成取号写一轮那一版（16777240），并写明装置没重跑；③C322 那一段括注改成「当时录制流不变、262165 不变」，C577 那一段后面补「取号写两轮之后是 12 段、43 次写、闭式 16 777 243」 |
| 2 | `milestone/01-first-txn.md:219`、`.en.md:183`、`.ja.md:183` | 括注补「取号写两轮之后是 12 段、43 次写、闭式 16 777 243、全量 16 777 268」 |
| 2 | `milestone/02-second-txn.md:70`、`.en.md:58`、`.ja.md:58` | 改成「2026-10-17 注、2026-10-26 改数：……80 段、485 次写、全量 1 678 377 220 个状态（取号写两轮之后）」，用的是 s5 的写法 |
| 2 | `milestone/02-second-txn.md:227`、`:486`、`milestone/03-third-txn.md:50`，以及英日对应的 `02:212`、`02:474`、`03:50` | 「后来改成 481 次写、1 678 377 204」后面补「取号写两轮之后又成 80 段、485 次写、全量枚举 1 678 377 220」，两态闭式 089 改成 095；「按新流全域重判」改成「按 481 次写那一版全域重判」。2026-09-30 和 2026-10-11 两次运行的事件句没改 |
| 2 | `milestone/02-second-txn.md:229`、`.en.md:214`、`.ja.md:214` | 两条流的数改成 12 段、43 次写、16 777 243 / 16 777 268，以及 80 段、485 次写、1 678 377 220；2026-10-11 注后面补「取号写两轮之后又成 12 段、闭式 16 777 243」 |
| 3 | `milestone/04-fourth-txn.md:76`、`.en.md:76`、`.ja.md:76` | 与里程碑三第 50 行同一句，改法相同（s5 写的是第 74 行，那是结束那一版的行号） |
| 5 | `milestone/02-second-txn.md:576`、`.en.md:565`、`.ja.md:565` | 改成过去时：「当时取分隔 key 的首尾……跟当时 checker 的 `check_index_node_keys` 走」，再接「C478 已还清，今天 inode 树根取子树覆盖区间（D18（块里携带什么信息） 已定项 2）」。英文写 is repaid，日文写 弁済済み，与同一文件第 374 / 391 行的用词一致。派发给的是英日第 563 行，工作区现在是第 565 行 |
| 6 | `crates/singlefs-checker/invariants/02-根环.md:24`（I-7.17） | 「core 那一半随根环实现批，今天没写。」改成「core 那一半已实现：`root_ring.rs` 的 `root_sits_in_the_ring_slot_the_formula_gives`，择根读槽 `read_root_ring_slot`（recovery.rs）与重建分配器读根环那一遍（mount.rs 的 `read_root_ring_slot_telling_never_written_apart`）都调它，C743 已还清。」现查：定义在 root_ring.rs:183；调用点在 recovery.rs:1751（在 `read_root_ring_slot` 里，它从第 1736 行起）和 mount.rs:2796（在第 2774 行起的那个函数里）；C743 在 checks-owed.md 的已还清表（第 907 行） |
| 6 | `crates/singlefs-checker/invariants/README.md:29`（M1） | 在「同日加下标 103 的 I-3.17（隔离记号与引用相符）；」后面补「C773（读坏与全零根槽的回收门槛钉只在内存里） 实现批加下标 104 的 I-7.18（根记录的回收门槛等于现算的门槛） 与下标 105 的 I-7.19（钉着的槽不住本实例写的根），GPU 判器走读内核同判；」。现查 `image.rs` 的 `IMPLEMENTED_INVARIANTS`：下标 104 是 `ROOT_RECORD_RECLAIM_THRESHOLD_EQUALS_THE_RECOMPUTED_THRESHOLD`，105 是 `PINNED_RING_SLOT_HOLDS_NO_ROOT_OF_THE_SAME_INSTANCE`，长度 108 |

## 工作区已经是新句、没改的

- 第 3 条，`verification-build.md:6`、`:130` 与英日的 `:6`、`:103`：工作区已经是 16 777 268 / 16 777 243 和 1 678 377 220，没有旧数。「登记的崩溃枚举用例 14 条」那一句工作区已经改成「条数用 `grep -c '^crash-case:' .claude/gate.d/stage-inputs.tsv` 现数，正文不抄数」，所以不用改。现数是 38，正文按规矩不抄这个数。
- 第 6 条，I-1.1（`01-块自描述.md:7`）：条文已经写了孩子是码 3 容器的 inode 树、墓碑树、谱系树、退役树的子树覆盖区间，状态列也已经写了 `walk::Walk::judge_the_key_range_of_an_inode_internal_node_against_its_leaves` 和 `walk::Walk::walk_the_internal_node_below_a_container_tree_entry`（现查这两个函数分别在 walk.rs:4011、:3530）。文件哈希开工和收尾时相同。

## 要主 agent 判的点

1. `01-first-txn.md:161`（英日 `:134`）：工作区这一行原本有一句「这一行里取号 `2`、暖机、事务的段序列、世代号与 16 777 240 是取号写一轮时的数，取号写两轮后的数以 `.claude/kb/layout/01-first-txn.md` 与 `crates/singlefs-checker-tier/tests/crash_enumeration_acquisition_barrier.rs` 为准」（英日同义），s5、s7 起草时都没见到它。①②把这一行的数换成两轮之后，这句话就不成立了，所以合并改时把它删了，三语一样。要留的话，得把它改成别的意思，这需要你来定。
2. 同一行，「取号 = 两轮……每轮之后一道池屏障」那个定义句和「首次挂载路径上第二轮之后那道屏障就是暖机开场那道（写入口不再发第二次）」，工作区已经是两轮的写法，与 `transaction.rs` 第 996 行的注释对得上，所以没改成 s7 提的「背靠背、并成一道」。s7 那种说法来自 layout/01 第 413 行，两种说法都成立。
3. `02-second-txn.md:70` 照 s5 加了日期「2026-10-26 改数」。如果嫌这一处带了事件日期，可以删掉，只留「（取号写两轮之后）」。
4. 顺带看到、不在这一轮范围里，没改：`layout/01-first-txn.md` 第 119 行（槽世代号「取号 2，暖机 3、4，新池新建文件发布后 5」）和第 398 行（「根槽之后写槽 1，世代号 5……取号写槽 0 世代 2；暖机……槽 1 世代 3、槽 0 世代 4」）还是一轮那一版的世代号。现查 `new_pool_file_creation_publish.rs` 第 501 行的注释是两轮：2、3，然后 4 / 5，事务 6 槽 0。要不要另派一份写回，由你定。`02-second-txn.md:229` 里「崩溃点重放的负载是两条流」s5 说另议，我照原样留着。

## 找不到的目标

0 条。规格点名的 17 个文件都在主工作区。

## replace-batch.py 原样输出

```text
✓ 32 处都命中得对（17 个文件），--dry-run 没写
✓ 32 处替换，写了 17 个文件（改名换上新 inode），回读一致
```

## relabel-item.py 与 --write

都没跑：没有分项要翻状态，这一轮又是并行写回，`doc-decisions.sh --write --check decision-items-sync` 归主 agent。`decisions.md`、`decisions-history.md` 开工和收尾时的哈希相同（`9a6453472d0e…`、`74aabc38d9c9…`）。

## 改过的文件与前后 sha256

| 文件 | 开工时 | 收尾时 |
|---|---|---|
| `.claude/kb/milestone/01-first-txn.en.md` | `d91d29584daaede860b8f86c10d5c058220a8c3be267d5bceb81f68464ac99f0` | `55e3c444c88a712169b09469a115e26ee0080c05305383d7b6be310d3eb2bd8f` |
| `.claude/kb/milestone/01-first-txn.ja.md` | `0860da111705e0c64acf7efe25d7cab2d4e6e98b856845a9c50fee8b5f17f570` | `441ee6dca3b48cbc9858955ae08534e3afd21a62ffea314aecaa9a0ee21f7123` |
| `.claude/kb/milestone/01-first-txn.md` | `00dd0e28e68f4b98c03daff5409311063386b0b5278fe5d15f26cedb797bcadf` | `61480c22c47961b0870b6a25c861f0ac0c8fe55308d20ba533e89b338930ebfb` |
| `.claude/kb/milestone/02-second-txn.en.md` | `93fe9d362db3a870675f9351683c1266e342e4b81ca32b4d39c5f2d338190fce` | `00058e986398102bda158a58ddd8aa94b8251212db1b452dca1cc599c921fd5f` |
| `.claude/kb/milestone/02-second-txn.ja.md` | `78bd7eb5c65317c4e70efac2182f553b89870aec07f0c24a751fc37c2fa425c0` | `cb8f6035a9ffb69dd7a953ee23b709b323c19729410e85263d0115517ee4a68a` |
| `.claude/kb/milestone/02-second-txn.md` | `8fbd40e417a40a92420d1b6a7abdfaaf5a4afa0133f9afc769e71542de22c32d` | `96a38a116d5ce3f1932340d113d30900145a7a093000f62ac1c7d9e2423fcd7d` |
| `.claude/kb/milestone/03-third-txn.en.md` | `a1213b2ed1d3c677fd022b09a154aeccb575e36739e80d10b3a73b75297da8a0` | `e71a140811d0a2eb7a4516c25875f517ab68da60b7efe7ab1ea11765e6db9990` |
| `.claude/kb/milestone/03-third-txn.ja.md` | `aaa5daece269bdda4b7e6ca83ba8684b985c9287089c67594ef8ff575523c8d5` | `286667302e4e412b916373299e667670e6c991f74e3064e77173961001530a84` |
| `.claude/kb/milestone/03-third-txn.md` | `400611cf665062c85fc8f17555d4061b64b19aaeedbf58a41f7377c1047d546a` | `84e87251a713be576f89a061ff5a0926c5f8e332816dc0dbbc8d98512109121b` |
| `.claude/kb/milestone/04-fourth-txn.en.md` | `648cfb76046ab4309e311012debb43ba799264ddbbc513529279e2fb76933919` | `5416d404923bb3ba688c08397084fb544e8e512133f0db6508b857fea426b7ec` |
| `.claude/kb/milestone/04-fourth-txn.ja.md` | `e3fafe04b93043b053e383726dbc0c31e0d45655643f61cbeb4c02c84bfdb374` | `19eb5b3318cfc006628d40f0e9d91aa9d402935c7f8836994294c40a588b0f0b` |
| `.claude/kb/milestone/04-fourth-txn.md` | `4a61c64737d22e967e45f6b1150fba7d6512541f5a45305af78058f27652078e` | `226b2103fc5bbae8dc3c93b6d62e061f631ccd520dc6edb8ba1ca7bf4e2b266d` |
| `.claude/kb/verification-build.en.md` | `f430a83554351f36d606e54cbc36364ba2f5090d08f81d65166c0d69a204a559` | `8a8791be3780314b75e74785eeb0a789ae63b4bf41148c0a18075f0b7b54afeb` |
| `.claude/kb/verification-build.ja.md` | `218b8d241fcfd979f264cf2f05a4357e13022561ff508ffeb049a1e006569cfe` | `38d69ac35eb4820e5794793facded6a5c5774926a32c548fd32a6dab0bb7be8f` |
| `.claude/kb/verification-build.md` | `b6baea6a242a21b93b5cb9b0574f5554e0ca5254f1a226eb2c66ef2c9d580b74` | `7a988c15bfb38ce245c91dc99dfed25da4c9a5b845d896c40a66b0aebcec7fd9` |
| `crates/singlefs-checker/invariants/02-根环.md` | `d795737b5575e618fe62ef00b962d686c576cc341f0784cbcbb939f19b3002ea` | `5383b79e7a81c9a26bb198471a9a7bdb6c10470e9121b7bf7e225a1979da8b60` |
| `crates/singlefs-checker/invariants/README.md` | `0e5e872e4ea8191471f11a949346f4e5e28d87222175d167209816f79ae4925b` | `46f7e4e13036683862a9cf83c4e4a9c300d21fb028f2c5968dec3a0c99128772` |

开工时的哈希是开工那一刻的，那时这 17 份都已经带着别的会话暂存或没暂存的改动（`git status` 显示 `M ` 或 `MM`），我只做了这 32 处定点替换。

## 没做什么

- 没判定案对不对，改后的句子按回扫报告的改法和现查的代码写。规格外只多写了两样：一是 I-7.17 的 mount.rs 调用点补了函数名；二是 `01:161` 合并改时删了第 1 点说的那句旧说明。
- 没跑门禁、lint、三语六格和重型测试，没写决策变更史，没跑 `--write`，没提交。
- 草稿目录里只有规格、辅助脚本 `tools/` 和几份输出，没有仓副本、编译目录或库拷贝。
