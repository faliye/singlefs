# 合入后验证快照放固定槽位（治理文件轮第一轮）攻方腿报告

攻方腿，判正文「判据格」第 1、5 格，并逐句核「已经量的」。被判文件照开工快照（`research/prompts/merge-verify-slots-governance-r1-snapshot/judged-sha256.txt`）：`scripts/gate/merge-and-verify.sh` 开工那一版 c88d382f…（主工作区这份在我开工之后被别的会话又改过文件头的开关说明，此刻是 da0ee570…；开工那一版就是此刻暂存区里那一份，我用 `git show :scripts/gate/merge-and-verify.sh` 取出放进草稿目录 `frozen/`，sha256 对上），`scripts/gate/merge-and-verify-snapshot-lib.sh` 545f52c3…、`scripts/gate/merge-and-verify-slots.conf` 167d6333… 与快照一致。下文引 `scripts/gate/merge-and-verify.sh` 的行号是交回这一刻主工作区那一份的行号；第 431 行之前与开工那一版逐行相同，挑槽那段 python 在开工那一版里靠前 8 行（diff 见草稿目录 `frozen/`，没进仓）。

## 复跑命令与文件 sha256

模型目录 `research/prompts/merge-verify-slots-governance-r1-opus-model/`，冻结副本 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-opus/frozen`（开工那一版的两份脚本）。复跑整条写在末尾「要主 agent 跑的命令」第 1 条。

```
a293d39f8f502eed5b77d1ea3d357e80a94f9602f070ab4d02693570dbabf699  slot_probe.py
59cd5fc96dd88872eb947ebdc9e4fbb6e46c57007ff3da50ef677d829f9e294b  refresh_probe.sh
8245116a956ed343fe3767a686d82a11e833c0cf1fc3a40ac24b8d8cf4e26f19  remove_stub.py
22afc08986db7e950f3c0dee8708c5e01312af22d811fafd4f1f63d2a414bf1c  out/expected-name-lines.txt
43176010a713c3484b39749f937fcfeb6eacd6a62a0ca4b73745a7b76256720f  tools/rerun.sh
e3294c15eaa1efb0b22aa9c8178c4f8af08c41373d855e04f89bf6de3dfa380b  tools/rerun-from-a-clean-work-directory.sh
c88d382fbeb9104b3903c09bd14955f34d9b55ab3eff609703c53814ee4c04d5  frozen/merge-and-verify.sh
545f52c3d23b5ca1805044ab3d816f5bfa899702bc2b34ab5c586e4c8652390b  frozen/merge-and-verify-snapshot-lib.sh
```

模型没跑过（子 agent 不交任务）：`out/expected-name-lines.txt` 七行全是推的，主 agent 复跑出来之前，下面标「模型」的都按推的算。探针不编、不起 cargo，只起 python 挑槽进程与 rsync；内存与时长估不超过 200 MiB、一分钟（推的，没量）。第 3b 步的崩溃状态枚举在这一题用不上：被判的是脚本的挑槽与拷贝，没有盘上的崩溃状态。

## 各格判定一览

| 格 | 情形 | 判定 | 依据 | 量的还是推的 |
|---|---|---|---|---|
| 第 1 格 | 同一槽先后两趟，改过 .rs 以外输入的包不重编而判绿 | 不成立（锁与时钟正常时） | 拷进槽对内容变了的每份文件都写新、修改时刻是此刻，cargo 照 dep-info 里的修改时刻重编；`slot-changed-sources` 只收 .rs 是重编核对少核了几种输入，不是让 cargo 不编 | 拷法与时刻：模型 non-rs-change（推的，待复跑）；cargo 一侧推的 |
| 第 1 格 | `slot-changed-sources` 只收 .rs 漏掉的输入 | 成立（少核），不致判绿 | .wgsl、`include_str!` 读的 `.claude/heavy-thread-default`、`Cargo.toml` 改了，重编核对报「本次未判」或不核那个包 | 模型 non-rs-change；真跑的 rebuilt.log「不是包里的 .rs 的 110 个文件不核」（量的） |
| 第 1 格 | 两台同时挑槽，NFS 4.2 local_lock=none 上排他 | 推不出：这一条路从没走到 | 第二台那一趟拿到 multi-host-run.sh 给的 CARGO_TARGET_DIR，一律不占槽，挑槽只在本机发生；本机工作根是本地 ext4、本机是 NFS 服务端 | 量的：真跑的第二台那一次尝试打「这一趟不占槽：调用方给了编译目录 CARGO_TARGET_DIR」；租约过期丢锁那一支推的 |
| 第 1 格 | 被 cgroup 杀的那一次尝试留下的槽被抢错 | 不成立 | 锁随进程放；别的任务先占了那一槽也只是整份按内容拷，判不错；下一次尝试照「同一任务上一次尝试占过的那一槽」挑，被占了就挑别的 | 推的（真跑的撞上限重来那一次，尝试 1 在第二台、没占槽，没走到这一支） |
| 第 1 格 | 留着的槽占满三槽之后照一趟一份跑而不报 | 成立（只报一行 `!`） | 三槽都留着时这一趟打一行「这一趟不占槽：…」，退出码、汇总都不带它；留着的槽没有期限，被取消的那一趟也照留 | 量的：此刻 slot.4u3rewvh 是一趟被取消（type=cancelled exit=239）的合入后验证留着的；模型 all-kept |
| 第 1 格 | 占盘上限的读法与超限处理 | 成立（三处打中） | 调低 `slots` 压不住已有的槽；`slots=0` 不再清槽的编译目录；上限只量这一台的那份编译目录，槽里快照下的 `repo/target` 在共享根上、不量不删 | 模型 lowered-limit、trim-scope、excluded-left-in-slot；量的：slot.kyz05duu/repo/target 此刻 2.5G |
| 第 5 格 | 两台都跑得起、判得对 | 成立；文件头三处描述与第二台的实际不符 | 第二台照一趟一份跑、判得对，但永远不占槽；文件头「同一槽在两台上各有一份」「服务不给它编译目录」在第二台不成立 | 量的（真跑日志一行） |
| 第 5 格 | `slots=0` 与登记读坏算不算给旧做法留的开关 | 读坏退 1：不算；`slots=0`：不算兼容旧东西，但它关槽后不清槽 | 一趟一份那一种此刻还在用（CARGO_TARGET_DIR、槽满、取不到根都走它），`slots=0` 只是选它，不认旧格式旧数据 | 推的（读代码） |
| 第 5 格 | 别的为旧路径旧行为留的分支 | 没找到 | 槽位那段代码里没有认旧版本、旧布局的分支 | 推的（读代码，范围见各节） |
| 已经量的 | 十句，另加判据格引的「两台的锁互斥」一句 | 七句核到原文（其中自证、开关复跑两句核到的那一趟对不上开工快照那一版）；两句对不上；两句日志里核不到 | 对不上的是「省盘没量到：拆分之后还没有一趟真跑…」与「剩 66 处都在拆分前就有的文件上」；核不到的是 replace-batch 那一句与「两台的锁互斥」 | 量的（逐行贴在后面） |

## 第 1 格：槽位拦不拦得住它要拦的事

### 1.1 同一槽先后两趟，改过的包不重编而判绿：不成立（锁与时钟正常时）

- 拷进槽：「#     内容没变的不动、留着上一趟的修改时刻，内容变了与新加的整份重写、修改时刻是此刻，cargo 照修改时刻判，改过的必重编；另带 --delete（源树里没有的在槽里删掉，」（`scripts/gate/merge-and-verify.sh:101`）。这一句不分后缀：.wgsl、`Cargo.toml`、`include_str!` 读的数据文件内容一变，槽里那一份就是此刻写的。
- 正推：cargo 判本地包新不新鲜看 dep-info 里每个输入的修改时刻（`include_str!`、`include_bytes!` 读的文件 rustc 照记进 dep-info；构建脚本按它自己报的 `rerun-if-changed`），一份文件内容只要与这一台这一槽上一次编时不同，它就是在那一次编之后被写的（槽里只有拷进槽与打补丁两种写法，都写此刻的时刻），修改时刻比产物新，cargo 必重编。这一步是推的，没在本仓编过对照。
- 前提有二，破了才会「不重编而判绿」：同一槽同一台的编译目录在这一趟里只有这一趟在用（锁，见 1.3）；写源文件的那一方的时钟不比编的那一台慢（跨台才有，见 1.3：第二台此刻不占槽，没有跨台）。
- 反推（要是错了该看到什么）：真跑过的四趟槽位那一种的 `rebuilt.log` 里应当有「没重编」的红。现查 `<工作根>/merge-and-verify/` 下 2026-11-19 一个、2026-11-20 三个用了槽的日志目录，`rebuilt.log` 末行都是「✓ 改过的包这一趟都重编了（核了 …个包…」（量的；核了 15、11、15、8 个包），没有红。这只说明这四趟没出事，不证明不会。
- 模型 non-rs-change（推的，待复跑）：只改 `shader.wgsl` 一份，槽里每份文件先拨成旧时刻，再拷一次；量槽里 `shader.wgsl` 的时刻比旧时刻新、`main.rs` 留着旧时刻、`slot-changed-sources` 零行。

### 1.2 `slot-changed-sources` 只收 .rs 漏掉的输入：成立（少核），不致判绿

- 只收 .rs：「  awk '{ code = substr($0, 1, 11); path = substr($0, 13) } (code ~ /^>f/ || code ~ /^\*deleting/) && path ~ /\.rs$/ { print path }' "$itemized" > "$3"」（`scripts/gate/merge-and-verify-snapshot-lib.sh:271`）。
- 漏掉的：.wgsl（`crates/singlefs-verification/src/crash_judge_gpu.rs` 用 `include_str!("crash_judge_common.wgsl")`）、`.claude/heavy-thread-default`（`crates/singlefs-verification/src/heavy_threads.rs` 经 `include_str!` 编进来）、`Cargo.toml`、`Cargo.lock`。只改了这几样的那一趟，带 `--skip-apply` 的报「重编核对本次未判」，不带的照补丁目录也只认 .rs（`scripts/gate/lib_rebuilt_packages.py` 的 `changed_packages` 本来就只认包里的 .rs）——所以这不是槽位新开的口子，是重编核对原有的范围；槽位那一种把「每趟都从头编」换成了「靠修改时刻增量编」，这道核对在槽位那一种里才第一次真要靠得住。
- 真跑里的样子（量的）：2026-11-20 那一趟 `rebuilt.log` 原文「（不是包里的 .rs 的 110 个文件不核：.claude/」开头的那一段（这 110 份是 `.claude/gate.d/fixtures/` 下样本树里的 .rs，不是 .wgsl 一类），说明核对把认不出包的直接放过、只打一句。
- 判绿要另有一处让 cargo 不重编（1.3 的丢锁、跨台时钟），两样都不在时，少核只是少一层保险。

### 1.3 两台同时挑槽时排他锁在 NFS 上靠不靠得住：推不出，这条路此刻走不到

- 文件头说的：「空着 = 锁拿得到而且不是留着的槽。锁是 POSIX 记录锁（fcntl.lockf）：本机加的锁与第二台经 NFS 加的锁落在服务端同一种锁上、互斥；flock 在服务端那一台本地加的与」（`scripts/gate/merge-and-verify.sh:91`）。
- 现查：本机就是 NFS 服务端——`/proc/fs/nfsd/exports` 里工作根所在的 `/home/fy5090/.local/share/singlefs` 导出给第二台（选项 `rw,root_squash,async,…`），本机 `findmnt -T` 工作根是本地 ext4。所以本机这一侧的锁是本地 POSIX 锁，与 nfsd 替第二台加的锁同落在 ext4 那一个 inode 上，互斥成立（这一句按我对 Linux 锁实现的印象，没读内核源码：`.claude/kb/tooling.md` 第 92 行指的内核源码树此刻不在，该目录是空的——这条 kb 与现状对不上，交主 agent）。
- 走不到：第二台那一趟根本不挑槽。`scripts/hosts/multi-host-run.sh` 给第二台那条命令一律设 CARGO_TARGET_DIR：「第二台那条命令：env -C <这一刻的工作目录> 起（同一个绝对路径），环境里 CARGO_TARGET_DIR=<这一趟的本地目录>/target、」（`scripts/hosts/multi-host-run.sh:64`），而挑槽第一句就是「  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:312`）。真跑（量的）：一条带 `env MERGE_AND_VERIFY_KEEP_COPY=1 bash scripts/gate/merge-and-verify.sh --unresolved-to-commit-gate --item gpu-verification-server:…` 的合入后验证，尝试 1 派到第二台、撞内存上限退 250（memory-cap-hit），它的日志里是「  ! 这一趟不占槽：调用方给了编译目录 CARGO_TARGET_DIR（不是哪一槽的那一份，按内容拷进槽的快照对着它编会把内容变了的文件判成已编）；快照与编译目录照一趟一份（收尾删，编译从头编）」。所以「两台同时挑槽」此刻不发生，NFS 上的锁一次都没被用过，靠不靠得住推不出。
- 将来第二台也挑槽时，我推得出的一处缺口（推的，没量）：挑槽进程拿到锁之后只每半秒核起它的 shell 在不在：「        os.kill(parent, 0)」（`scripts/gate/merge-and-verify.sh:1156`），不核锁还在不在。第二台与本机断开超过 nfsd 的租约（默认 90 秒；本机 `/proc/fs/nfsd/nfsv4leasetime` 读不了，权限拒）时服务端可以收回那把锁，本机一趟就能拿到同一槽、`rsync --delete` 改写第二台那一趟正在编、正在测的 `repo/`；第二台那一趟的持锁进程不对锁文件做读写，拿不到 EIO，照跑。这是「判得不对」而不是「跑不起」。
- 时钟一支（推的）：同一槽两台各一份编译目录，源文件的修改时刻是本机（服务端）时钟、第二台编译目录里指纹的时刻是第二台时钟；第二台比本机快 δ 时，本机一趟在第二台上一次编完 δ 之内写过的文件，第二台下一趟会当成没变。δ 在时钟同步时是毫秒级，而两趟之间至少隔一整趟，够不着；第二台此刻不占槽，也走不到。

### 1.4 被 cgroup 杀的那一次尝试留下的槽，会不会被抢错：不成立

- 锁跟着挑槽进程：「锁跟着那个 python 进程：收尾 kill 它、等它退（release_slot）；这一趟被 cgroup 杀时它一起没；它另外每半秒核一次起它的 shell 还在不在，不在就退。」（`scripts/gate/merge-and-verify.sh:93`）。本机上进程退出锁就放，这一点不靠 NFS。
- 下一次尝试先挑它：「同一任务的下一次尝试挑槽时先挑它记下的那一槽（尝试号更小、最大的那一次尝试的日志目录里的 slot，earlier_attempt_slot），空着就接着那一槽的编译目录编」（`scripts/gate/merge-and-verify.sh:73`）。中间别的任务先占了，那一槽照内容整份拷、照修改时刻增量编，判的是它自己的树；下一次尝试挑别的槽或新开一槽，从头编。两边都判不错，只多花编译时间。
- 杀在半路的几种我逐个过了：拷到一半（rsync 先写临时名再改名，下一趟 `--delete` 删掉残的临时文件、按内容补齐）；打补丁到一半（下一趟按内容改回源树的样子，时刻是此刻）；cargo 编到一半（没写完的单元没有新指纹，下一趟重编）。都推的。
- 唯一一处窄窗（推的）：带 `MERGE_AND_VERIFY_KEEP_COPY=1` 的那一趟在收尾写完 `kept` 之后、`release_slot` 之前被杀，这一槽记成留着、锁已放；服务重排的下一次尝试挑它时 `is_kept` 为真、跳过，另占一槽再留一份。多占一槽，不判错。
- 真跑里撞上限重来的那一次（量的）：尝试 1 在第二台、没占槽；尝试 2 在本机，日志「  删了上一次尝试留下的快照 …/repo（共 2.2G；…的尝试 1，…）」后「  占了槽 …/slot.4u3rewvh（新开一槽：这一刻 2 槽，登记的上限 3 槽）」。没走到「尝试 1 占了槽」那一支。

### 1.5 留着的槽占满三槽之后照一趟一份跑而不报：成立（只报一行 `!`）

- 三槽都留着时挑槽进程答「SLOT none 登记的 3 槽都有了、没有空着的（3 槽是带 MERGE_AND_VERIFY_KEEP_COPY=1 留下快照、还没放的，别的有别的趟在跑）」（模型 all-kept 量这一句，推的），入口照文件头「槽都占着或留着又到了上限、挑槽出错。」（`scripts/gate/merge-and-verify.sh:95`）那一种退回一趟一份，只打一行「    echo "  ! 这一趟不占槽：$SLOT_FALLBACK_REASON；快照与编译目录照一趟一份（收尾删，编译从头编）"」（`scripts/gate/merge-and-verify.sh:2419`）。`grep -n '不占槽\|SLOT_FALLBACK_REASON'` 在三份库与入口里没有别处再打它：汇总行不带，退出码不变，任务结局行看不出这一趟是从头编的。
- 留着没有期限：`kept` 里写的 `created` 没有谁读；放槽只有「#   留着的槽：别的趟不占它，直到放槽——rm -f -- <槽>/kept <槽>/.owner <槽>/.deliverables，快照与编译目录留给下一趟接着编；repo/ 被删了的槽不算留着（下一趟占它就整份拷、从头编）。」（`scripts/gate/merge-and-verify.sh:109`）那一条手动命令。完工清理闸认槽里的 `.owner`，只在写它的那个会话收工时拦；那个会话没了，槽就一直留着。
- 被取消的那一趟也留：收尾那一支「    if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:64`）不看这一趟的结局。真跑（量的）：mainagent(A) 交的一条 `env MERGE_AND_VERIFY_KEEP_COPY=1 bash scripts/gate/merge-and-verify.sh --unresolved-to-commit-gate --item gpu-verification-server:…` 结局 type=cancelled exit=239，日志末尾照打「  ! 留着快照 /home/fy5090/.local/share/singlefs/work/merge-and-verify/slot.4u3rewvh/repo（槽 /home/fy5090/.local/share/singlefs/work/merge-and-verify/slot.4u3rewvh 记成留着，MERGE_AND_VERIFY_KEEP_COPY=1：别的趟不占它；…」；此刻 `slot.4u3rewvh/kept` 还在（`ls -la` 现查），三槽里已有一槽被一趟作废了的验证占着。
- 占满之后的代价（推的）：每一趟退回在 TMPDIR 下从头编一份（文件头说上百 G），加在三槽各自的编译目录之上；登记第 4 行说的「两台各自至多 slots × slot-target-limit-gib」不算这一份。

### 1.6 占盘上限的读法与超限处理：成立（三处打中）

读法本身（`read_slot_table`）我没找到错：两项必填、非负整数、认不出的行退 1，`10#$value` 去前导零；同一键写两次后一次生效、不报，算小毛病。超限处理有三处与登记、文件头的说法不符：

1. **调低 `slots` 压不住已有的槽。** 挑槽时已有的槽一律可占：「    for slot in ([] if broken == "slot-not-reused" else order):」（`scripts/gate/merge-and-verify.sh:1128`），上限只管新开：「    if held is None and (len(slots) < limit or broken == "slot-limit-ignored"):」（`scripts/gate/merge-and-verify.sh:1134`）。登记第 2 行写「# slots：两台合计至多几槽」（`scripts/gate/merge-and-verify-slots.conf:2`）。反例输入：工作根下已有 3 槽（此刻真有 2 槽），用户把 `slots=3` 改成 `slots=1`，三趟同时来，三趟各占一槽，每台照样有 3 份编译目录。模型 lowered-limit（推的，待复跑）：已有 3 槽、上限 1、同时起 3 个挑槽进程，期望 3 个各占一槽；对照 new-slots-capped：没有槽、上限 1，只有 1 个占到。
2. **`slots=0` 之后槽的编译目录再也不清。** `choose_slot` 在「  if ((SLOT_LIMIT == 0)); then SLOT_FALLBACK_REASON="槽位登记 $SLOT_TABLE 的 slots 是 0"; return 0; fi」（`scripts/gate/merge-and-verify-snapshot-lib.sh:315`）就退，清槽那一句「  sweep_slot_targets "$slot_root" "$build_root/$SLOT_BUILD_DIRECTORY_NAME"」（`scripts/gate/merge-and-verify-snapshot-lib.sh:326`）在它后面，且只清「槽目录已不在」的。登记第 2 行说 0「就是一趟都不占槽，每一趟照一趟一份那一种拷快照、从头编、收尾删。」（`scripts/gate/merge-and-verify-slots.conf:2`），没说已有的几份怎么办。此刻本机就有两份：`du -sh` 量的 105G（slot.4u3rewvh）与 91G（slot.kyz05duu），在多机配置 `BUILD_ROOT_BY_HOST` 本机那一项下的 `merge-and-verify-slots/`；改成 0 之后这 196G 一直留着，除非有人手删。这一处是按代码推的，没跑。
3. **上限只量这一台那份编译目录，槽里快照下编出来的东西在共享根上、不量不删。** 量的只有「  bytes="$(du -s --block-size=1 -- "$SLOT_TARGET" 2> /dev/null | cut -f1)"」（`scripts/gate/merge-and-verify-snapshot-lib.sh:279`）；拷进槽对排除的几样不删：「槽里不留上一趟打的补丁与别的残文件；排除的 .git、target、target-*、__pycache__ 不删，槽根的 .git 是这里 git init 的）与 --executability（内容没变而可执行位变了的照改）。」（`scripts/gate/merge-and-verify.sh:102`）。放槽只删 `kept`、`.owner`、`.deliverables`。所以留着的槽在验证段第 2 步被当成树来跑的命令（不带 CARGO_TARGET_DIR 的 cargo 照默认编进 `<树>/target`）编出的东西，放槽之后永远留在共享根的槽里。量的：`slot.kyz05duu/repo/target` 此刻 2.5G（`du -sh` 现查，里面是 `release/crash_library_reader`、`e165_replay_bounded_enumeration`、`new_pool_file_creation_on_device` 等），目录建于 2026-11-19，晚于那一槽第一趟合入后验证收尾；之后这一槽又被 2026-11-20 一趟占过，`repo/target` 照留。`slot.kyz05duu/repo` 合计 4.6G，源树才 2.2G。另有 `repo/.git` 下 62M 的缓存与打补丁记录（`singlefs-applied-writer-patches.tsv` 在槽里逐趟累加，`apply-writer-patch.py` 拿它数「自上次崩溃验证以来打了几个补丁」那一句提示，只多报，不拦）。谁编出的 `repo/target` 我没核到（推的：时间落在那一趟收尾之后，是验证段第 2 步在留着的槽里起的某条命令）。这与文件头「…下的 merge-and-verify-slots/<槽名>/target，不放共享根；同一槽在两台上各有一份，」（`scripts/gate/merge-and-verify.sh:87`）要防的是同一件事：编译产物落在共享根上。模型 excluded-left-in-slot 与 trim-scope（推的，待复跑）：槽里先放 `repo/target/release/tool`（1 MiB）与 `repo/scripts/__pycache__/module.pyc`，拷进槽之后两样都在；上限设 0 时 `trim_slot_target` 叫去删的只有 `SLOT_TARGET`。

另有两处不算打中、记一笔：上限只在占槽后、编之前与收尾量，一趟编的过程中可以长过上限（量不到峰值）；被 cgroup 杀的那一趟收尾不跑，超了的那份要等这一台下一次占这一槽才清。这两处文件头写的就是这样（「#   占到槽之后、编之前与收尾各量一次」那一句的意思），没有说成硬上限，只是「两台各自至多 slots × slot-target-limit-gib」（`scripts/gate/merge-and-verify-slots.conf:4`）读起来像硬上限。

## 第 5 格：多机与不兼容旧东西

### 5.1 槽的编译目录按台分、快照在共享根，两台都跑得起、判得对：成立；文件头三处与第二台的实际不符

- 判据：`.claude/agent-common.md` 第 30 行「所有任务都要多机可用，不许绑定某一台」那一条，要「派到两台任一台都要跑得起、判得对」。
- 跑得起、判得对（量的）：第二台那一次尝试照一趟一份跑（快照拷进共享根上的日志目录、修改时刻是此刻，编译目录是 multi-host-run.sh 给的这一趟新建的那一份），判法与本机那一种相同。
- 不符的三处（量的，同 1.3 那一行日志）：
  1. 「下的 merge-and-verify-slots/<槽名>/target，不放共享根；同一槽在两台上各有一份，」（`scripts/gate/merge-and-verify.sh:87`）——第二台一份都没有，挑槽只在本机发生。
  2. 「命令那一层也没有 cargo，服务不给它编译目录；」（`scripts/gate/merge-and-verify.sh:98`，这一句前面是「所以这份脚本文件头不写会编的声明」）——第二台那一层（multi-host-run.sh）不看这份声明，照给。
  3. 那一行的说法「调用方给了编译目录」把第二台包装给的当成调用方给的，主 agent 读日志会以为自己的命令里带了 CARGO_TARGET_DIR。
- 后果（推的）：省盘、省时只在本机有；派到第二台的每一趟都从头编。判定不受影响，所以不违反「判得对」；违反的是文件头与登记第 4 行对两台的描述。槽的增量编是不是「绑定某一台」的一种，要主 agent 判：它不改判定结局，我倾向不算。

### 5.2 槽位登记读坏退 1、不退回旧做法；`slots=0` 算不算给旧做法留的开关

- 读坏退 1（读代码，推的）：「      die "槽位登记 $SLOT_TABLE 里有认不出的一行」（`scripts/gate/merge-and-verify-snapshot-lib.sh:186`），缺项、值不是非负整数同样 `die … 1`；读登记排在 `--list-items` 之后、建日志目录之前，`--list-items` 不受登记坏的连累。不退回旧做法，合 `.claude/agent-common.md` 第 46 行「里程碑 17 收尾之前不兼容任何旧东西，代码与格式尤其如此」那一条。
- `slots=0`：不算兼容旧东西。那一条管的是认旧版本、旧布局、旧数据的分支；一趟一份那一种此刻仍是活路（调用方给了 CARGO_TARGET_DIR、第二台、槽都占着或留着、取不到工作根或编译根、挑槽出错都走它，见文件头第 94–95 行），`slots=0` 只是不挑槽、直接走它，没有为它多留一行代码。自证里多数格就是靠 `slots=0` 跑一趟一份那一种的。问题不在「算不算开关」，在 1.6 第 2 处：关了之后已有的槽与编译目录没人清。

### 5.3 别的为旧路径、旧行为留的分支：没找到

查的范围：`scripts/gate/merge-and-verify-snapshot-lib.sh` 第 175–331 行（槽位那一段全部）与 `finish` 的槽位那一支，`scripts/gate/merge-and-verify.sh` 文件头第 60–113 行、挑槽与编译根那两段 python、开跑时选槽与拷快照那一段。

- `remove_earlier_attempt_snapshots` 删的是一趟一份那一种留在日志目录里的 `repo/`，此刻第二台每一次尝试都留这种（量的：撞上限重来的尝试 2 删了尝试 1 的 2.2G），是活路。
- 「repo/ 被删了的槽不算留着」是给手删腾盘那一种的，不是认旧槽。
- 调用方给 CARGO_TARGET_DIR 不占槽：理由写在文件头第 96 行，自证的重编核对几格靠它造「共用编译目录」的局面，第二台也走它，是活路。
- 没找到认旧槽布局（例如槽里没有 `lock` 或 `used`）的分支：`used` 不在时 `used_at` 交 0，只是排在最后，不是为旧槽留的。

## 「已经量的」逐句核

日志在 `~/.local/state/singlefs/task-server/logs/` 下按任务号起名，这里照共用约束不抄日志路径与任务号，写交给服务的那条命令与那一行的原文；结局行由 `task-server/task status <号>` 现查。

| # | 正文那一句 | 核到的原文 | 结论 |
|---|---|---|---|
| 1 | 「换上之后自证 type=succeeded exit=0。」（`research/prompts/_merge-verify-slots-governance-r1-body.md:33`） | 命令 `python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/gate/merge-and-verify.sh --selftest`，本会话交的那一条结局「结局 succeeded、退 0」，日志「  ✓ 自检通过（查了 156 种）：…」，`grep -c '✗ 自检'` 0 | 核到。但那一趟早于入口拆成几份库（日志文件的修改时刻早于 `merge-and-verify-snapshot-lib.sh` 的，`merge-and-verify-selftest-lib.sh` 改于 2026-11-20），自证的不是开工快照那一版；之后别的会话在 2026-11-20 交过两次同一条命令，都是「结局 succeeded、退 0」，是不是开工快照那一版核不到（日志里不打 sha256） |
| 2 | 「规则与技能经 `scripts/kb/replace-batch.py` 写，5 处命中、回读一致。」（`research/prompts/_merge-verify-slots-governance-r1-body.md:34`） | 没有日志：`replace-batch.py` 是直接跑的文本工具，不经服务 | 日志里核不到；只有调度记录第 104 行主 agent 自己记的那一句 |
| 3 | 开关复跑 type=command-failed exit=1，「开关 153 条、对上 149 条」 | `task-server/task status` 现查「结局 command-failed、退 1、…命令 bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh」；日志「  · break-switch-replay 跑完一份：scripts/gate/merge-and-verify.sh 开关 153 条、对上 149 条、用了 5386 秒」 | 核到 |
| 4 | 「槽位那一族 10 条全部对上」（`research/prompts/_merge-verify-slots-governance-r1-body.md:35`） | 同一份日志第 123–133 条，例「  · break-switch-replay 第 124 / 153 条对上：scripts/gate/merge-and-verify.sh MERGE_AND_VERIFY_BREAK=slot-table-unchecked，用了 684 秒」，一族 10 条（开关表那一族逐条数过是 10 条）全在，另 `earlier-attempt-slot-not-preferred` 也对上 | 核到。但同一份日志「  ! scripts/gate/merge-and-verify.sh 基准趟：跑的过程中键变了（开跑 84d402538e03，跑完 refused），不记」「  · 判绿记库：RECORD recorded=0 changed=150 refused=0 red=4 red-unrecorded=0 …」——跑的过程中被判的文件在变，149 条绿一条都没记库，对上的是哪一版核不到 |
| 5 | 不符的 4 条红在别的会话新加的「判不出的留给提交时整轮门禁」几格 | 日志四行「  ✗ scripts/gate/merge-and-verify.sh MERGE_AND_VERIFY_BREAK=ignore-red：不符，失败行没被点名：「✗ 自检：判不出的留给提交时整轮门禁：…」」等 | 核到 |
| 6 | 「省盘没量到：拆分之后还没有一趟真跑的合入后验证编出来。」（`research/prompts/_merge-verify-slots-governance-r1-body.md:36`） | `<工作根>/merge-and-verify/` 下 2026-11-20 有三个日志目录记了 `slot`、都有 `build.log` 与 `rebuilt.log`（「✓ 改过的包这一趟都重编了…」）；本机 `merge-and-verify-slots/` 下两份编译目录 `du -sh` 105G、91G；其中一趟拷进槽「写了 233 份（内容变了或新加，修改时刻是此刻）、删了 5 份」 | **对不上**：开工前已有真跑过、增量编过的槽位那一种，省盘的数此刻量得到（两份编译目录各一百 G 上下，槽里快照 2.2G 与 4.6G） |
| 7 | 「type=succeeded exit=0；证红副本（补丁打回入口模块）」（`research/prompts/_merge-verify-slots-governance-r1-body.md:38`），即用例那一趟 Ran 1 test … OK 与证红副本 FAILED (errors=1) type=command-failed exit=1 | `python3 scripts/agents/tests/test_agents.py WatchTests.test_progress_deleted_before_read_is_reported`「结局 succeeded、退 0」，日志「Ran 1 test in 0.027s」「OK」；证红副本那一条「结局 command-failed、退 1」，日志「AttributeError: 'types.SimpleNamespace' object has no attribute 'dispatch_prompt'」「FAILED (errors=1)」 | 核到（证红那一趟派到第二台） |
| 8 | 「preflight-lint type=command-failed exit=1：拆分带出的十份已消，剩 66 处都在拆分前就有的文件上。」（`research/prompts/_merge-verify-slots-governance-r1-body.md:39`） | 两趟 `python3 .claude/singlefs-ai-sop/scripts/preflight-lint.py .` 都「结局 command-failed、退 1」；前一趟「判了 103 个脚本…，76 处不合格；排除 151 个」，后一趟「判了 93 个脚本（登记目录 74、门禁阶段 19），66 处不合格；排除 161 个」 | 「十份已消」核到（76→66，排除多了 10 个，是登了排除，不是改好）；**「剩 66 处都在拆分前就有的文件上」对不上**：66 处里 12 处落在 4 份 HEAD 里没有、暂存区新加的文件上（`git cat-file -e HEAD:<路径>` 不在），是 `scripts/experiment-apparatus/e203_admission_scan_cost_counters.py`（HEAD 2428 行，此刻 130 行）拆出的 `_selftest.py`、`_row_table_and_module_template.py`、`_insertion_edits.py`、`_applying_edits_and_copying_the_tree.py`，每份三处（没写 admission、没写 run-condition、没调 preflight）；`.claude/preflight-exclude` 里没有 e203 |
| 9 | 「`scripts/kb/sweep-term.py --check` exit=2」（`research/prompts/_merge-verify-slots-governance-r1-body.md:40`），只剩 implementation-writer.md 一条、HEAD 里那一句也不在 | `python3 scripts/kb/sweep-term.py --check .` 后一趟「结局 command-failed、退 2」，日志「  ✗ 豁免登记表里有 1 条按行豁免对不上…」「      .claude/agents/implementation-writer.md:「列一节「受影响的层 0 流与崩溃枚举用」命中 0 行」；`git show HEAD:.claude/agents/implementation-writer.md` 里那一句 `grep -c` 0 | 核到 |
| 10 | 「type=command-failed exit=1，七格都红」（`research/prompts/_merge-verify-slots-governance-r1-body.md:41`） | `bash .claude/gate.d/code-tooling.sh --check heavy-thread-defaults,hardcoded-quota,repository-copy-closure,memory-bounded-parallelism,peer-launch-direct,change-range-single-source,directory-layout`「结局 command-failed、退 1」，日志「  ✗ code-tooling.sh：7 格里 7 格红（…）」 | 核到；「原因归判法」那几句是解释，日志里核不到 |
| 11 | 调度记录同一段的「第二台挂工作根是 NFS 4.2、local_lock=none，两台的锁互斥」（`records/2026-11-18-合入后验证压盘.md:104`，正文第 53 行照抄成判据） | 本机只看得到导出一侧（工作根导出给第二台、`async`）；第二台的挂载选项我看不到 | 核不到；且「两台的锁互斥」此刻没被用过（1.3） |

## 打中的几格先答四句

这一轮是治理文件轮，没有「臂」；第一句改问「打中的是被判的这版槽位自己的缺陷，还是各种做法共用的前提」。

| 打中的 | 是不是这版槽位自己的 | 系统当时看不看得到判别它的东西 | 满足判据字面的哪一分句 | 改法在这一格上还中不中 |
|---|---|---|---|---|
| 1.6 第 1 处：调低 `slots` 压不住已有的槽 | 是：挑槽进程只拿上限挡新开 | 看得到：挑槽进程手里有 `len(slots)` 与 `limit` | 第 1 格「占盘上限的读法与超限处理」；与登记第 2 行「两台合计至多几槽」字面相反 | 改法甲修这一格；乙、丙、丁不碰它 |
| 1.6 第 2 处：`slots=0` 后不清 | 是 | 看得到：`choose_slot` 知道 `SLOT_LIMIT == 0`，也知道编译根在哪 | 同上；也是第 5 格「slots=0」那一问的实质 | 改法甲（0 时照清）修它 |
| 1.6 第 3 处：`repo/target` 等留在共享根的槽里、不量不删 | 是：一趟一份那一种收尾整份删快照，没有这个问题 | 看得到：拷进槽时 `repo/target` 就在那里 | 第 1 格「占盘上限的读法与超限处理」；与文件头第 87 行「不放共享根」要防的是同一件事 | 改法乙修它；甲不碰 |
| 1.5：三槽都留着时只打一行 `!` | 部分是：退回一趟一份是设计里写的，「不报」是这版的选择 | 看得到：挑槽进程算了 `kept` 槽数 | 第 1 格「占满之后这一趟照一趟一份跑而不报」——报了一行，没进汇总、退出码与结局行 | 改法丙修它 |
| 5.1：第二台永远不占槽 | 不全是：病根在 multi-host-run.sh 一律给 CARGO_TARGET_DIR 与这里一律让它，两份都改得动 | 看得到：第二台那一层另设 `MULTI_HOST_RUN_PEER_SCRATCH`（`scripts/hosts/multi-host-run.sh` 第 65 行） | 第 5 格字面（跑得起、判得对）**不中**；中的是文件头第 87、98 行与登记第 4 行的描述 | 改法丁两条路选一条 |

## 改法（只在我的模型上推过、没实现没跑，被攻过零轮）

| 改法 | 修哪一格 | 怎么改（推的） | 标 |
|---|---|---|---|
| 甲 | 1.6 第 1、2 处 | 挑槽进程按 `used` 排好后只在前 `limit` 槽里挑，多出来的槽不占；`slots` 小于已有槽数（含 0）时，`choose_slot` 照样跑一次清理：多出来的、没被占也没留着的槽连同这一台的编译目录删掉（占着的、留着的跳过并打一行） | 推的，待主 agent 跑：复跑 lowered-limit 应变成 held_distinct=1 |
| 乙 | 1.6 第 3 处 | `refresh_slot_snapshot` 拷之前删掉槽里 `repo/` 下的 `target/`、`target-*/`、`__pycache__/`（`.git` 照留），或 rsync 换成 `--delete-excluded` 再用 `--filter 'protect .git'` 护住 `.git`；放槽那一条命令另带 `rm -rf -- <槽>/repo/target` | 推的，待主 agent 跑：复跑 excluded-left-in-slot 应变成 repo_target_left=no |
| 丙 | 1.5 | 「这一趟不占槽」那一行照抄进汇总；三槽里有留着超过若干天的（`kept` 里有 `created`），打一行点名那几槽与放槽命令 | 推的 |
| 丁 | 5.1 | 两条路选一条：认出第二台包装给的编译目录（`MULTI_HOST_RUN_PEER_SCRATCH` 在、CARGO_TARGET_DIR 落在它下面）时照样占槽、用槽的编译目录；或者不改行为，把文件头第 87、98 行与登记第 4 行改成「只在本机占槽」并把那一行说成「第二台那一层给了编译目录」 | 推的；前一条另要过 1.3 丢锁那一支 |
| 戊 | 1.3 丢锁（将来第二台也占槽时） | 挑槽进程每半秒除了核 shell 在不在，另对锁文件做一次小读写，读写报错就停起它的那一趟（打一行、退非 0） | 推的；第二台此刻不占槽，排在丁之后 |

## 没打中的形状

- 重编：试过只改 .wgsl、只改 `Cargo.toml`、只改 `include_str!` 读的仓根文件、只删一份 .rs、拷进槽被杀在半路、打补丁之后下一趟按内容改回、`--skip-apply` 接着上一趟打过同一批补丁的槽、同一槽两台轮流（时钟一致）——按代码与 cargo 的修改时刻规则推，都重编或本来就不用编；没找到让 cargo 不编而判绿的一步。没试的：构建脚本读了却没报 `rerun-if-changed` 的输入（看了 `crates/singlefs-verification/build.rs` 末尾那段，它把算键闭包里的文件都报了，没逐个核闭包是否全）。
- 挑槽：`take_slot` 读答话超时（120 秒）那一支——超时后停掉挑槽进程，它要是刚占了槽只会多写一次 `used`、多建一个空槽；`$!` 指的就是 python 那一个进程（本机 bash 5.2.21 现跑 `exec {fd}< <(python3 …)` 核过，`$!` 等于 python 自报的进程号，`kill` 加 `wait` 退 143）。第二台的 bash 版本没核。
- 留着的槽被别的趟占走：本机锁与 `is_kept` 判在同一个进程、拿着锁时判，放槽前写 `kept`，没有缝。第二台 NFS 客户端缓存 `kept` 不存在的那一支（目录属性缓存几十秒内看不到别的台新建的 `kept`）要两趟间隔短于缓存时限，真跑的一趟以分钟计，够不着；且第二台此刻不挑槽。
- 清理误删：`sweep_slot_targets` 只删槽目录不在了的那几份；在用的槽目录不会不在，除非有人手删在用的槽。
- 读登记：重复键、CRLF、带空格的键、超长数字都试了写法，前三种退 1 或后一次生效，超长数字会让 bash 算术溢出（不现实，不算）。

取样范围：只读了被判的三份文件里槽位那几段、`lib_rebuilt_packages.py` 文件头、`multi-host-run.sh` 给第二台编译目录那两段、完工清理闸认合入后验证快照那一段、六份 `build.rs` 的 `rerun-if` 行；真跑的数取自 `<工作根>/merge-and-verify/` 下用了槽的 4 个日志目录与 2 个槽、本机两份槽的编译目录。

## 这条腿自己的限度

- 模型一条都没跑过，`out/expected-name-lines.txt` 是我推的；七行里打中的三行（lowered-limit、excluded-left-in-slot、trim-scope）要主 agent 复跑才算量的。
- 第二台的挂载选项、时钟、bash 版本、nfsd 租约时长我都看不到；NFS 锁那几句全是推的，且没有内核源码可核（`~/kbuild/linux-om` 不在）。
- 「谁编出 `slot.kyz05duu/repo/target`」没核到命令；只核到它在、多大、建于哪天。
- 槽位那一族开关与九格自证我没逐条复核它们证的是不是这份报告里的几件事；只核到这几件里没有一格是开关或自证盖住的（`grep` 自证库「槽位：」九格的失败句，没有调低上限、`slots=0` 后清理、`repo/target`、第二台四样）。
- 只攻了分到的第 1、5 格与「已经量的」；第 2、3、4 格没碰。

## 要主 agent 跑的命令

1. `SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证槽位攻方腿模型的两个探针（挑槽与拷进槽占盘）; 范围=7 行 name=; 为什么=复跑攻方腿打中的调低上限、槽里编译产物不清、上限只量本机编译目录三格与没打中的非 .rs 改动一格，判决要引的数' task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-opus -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r1-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-opus/frozen /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-opus/probe-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r1-opus-model`
   - 在哪起：`/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r1-opus`（草稿目录；冻结副本 `frozen/` 在里面，`.deliverables` 已写成交接）。
   - 看哪几行：末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0` 算跑通；`task-server/task log <号> --all` 里 `name=rerun_compare lines=7 verdict=all-lines-match` 是七行与推的一致；对不上时那一行是 `verdict=lines-differ`，上面 diff 的 `>` 行是量的，照量的改判定。退 2 是用法或运行目录不合；探针退 1 是探针自己出错。
   - 撞上限重排时外面这一层先删 `probe-run` 再交给 `rerun.sh`，`probe-run` 只能是工作根下以 `-run` 收尾的目录，别的退 2 不删。

## 没做什么

- 不判正推那几格（第 2、3、4 格）；不替主 agent 采纳；没往统一任务服务交任务，模型没跑，判决要引的数没复跑（命令在上一节）。
- 没改被判文件；改法甲到戊只写在这里，没在副本上实现。
- 没建仓副本与编译目录：草稿目录里只有 `frozen/`（两份脚本的开工版，交接给上一节那条命令）、`status.txt`、`buildroots.txt`、`preflight-paths.txt` 三份文本草稿。
