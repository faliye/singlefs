# 合入后验证槽位治理第三轮：攻方腿报告（2026-11-21）

攻击面：正文判据格第 1 格（第二轮改法第 1 条的修补）与第 4 格（第二轮只抽查过的地方、「已经量的」逐句核）。被判文件现读 sha256 与开工快照 `research/prompts/merge-verify-slots-governance-r3-snapshot/judged-sha256.txt` 十份逐份相同；三份合入后验证脚本与冻结副本 `~/.local/share/singlefs/work/merge-verify-slots-r3-verdict/frozen/` 逐份相同。

## 复跑命令与文件 sha256

模型目录 `research/prompts/merge-verify-slots-governance-r3-opus-model/`（没跑过，读数全部待主 agent 跑出）：

| 文件 | sha256 |
|---|---|
| `probe/slot-root-pairing.sh` | `7354eb3f954137857d2e3ef8a4a919a5bab61b2eebcfbfc2eda5c3daf3d0513f` |
| `probe/configured-default-work-root.py` | `ba834838ced3b7a3f67e3fe9324b02302fe642b9cfe47d8222ad695f5ce09759` |
| `tools/rerun.sh`（模板拷出，只改 `rerun_steps`） | `9ccbe53ca4aeba1e7aa641042dc86b7576095688c072878294feaf68ddd3a5f5` |
| `tools/rerun-from-a-clean-work-directory.sh`（外层，文件头带会编声明） | `547bb46ac7f7457da85abc3e0b95dd0ae49ca38919e4c78169c23f85104d093f` |

复跑（两项，照「要主 agent 跑的命令」一节）：先 `mkdir -p /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-opus-pairing-run`（不交服务），再交外层脚本。模型做什么：拿冻结的 `merge-and-verify-snapshot-lib.sh` 原样（`intact`）与只换了第 410 行取缺省工作根那一条的一份（`fix-configured-root`，缺省工作根改成多机配置 `SHARED_DATA_ROOT` 下的 `work`，不看 `SINGLEFS_DATA_ROOT`），在每格自己的假数据根、假多机配置、假编译根里直接调 `choose_slot`，与自证 `slot_default_choose` 同一种调法；调之前同一份环境里先取编译根、工作根与两种缺省工作根，有一样不落在这一格的根下就不调。取样：两臂 × 布局两种（`plain`、`data/work` 是符号链接的 `work-symlink`）× 配置里 `SHARED_DATA_ROOT` 两种写法（真目录、符号链接）× `SINGLEFS_DATA_ROOT` 四种（不设、`data`、`data-link`、`other-data`）× `SINGLEFS_WORK_ROOT` 六种（不设、`data/work`、`data-link/work`、真路径、`other-data/work`、`other-work`）× 登记 3 槽与 0 槽，共 384 格。用户动作（交命令时带哪个根）放开扫，固定的只有槽那一层的前缀（空着的 `slot.idle`、留着的 `slot.kept` 各有一份编译目录）。估时：头 4 格之后打 `name=slot_root_pairing_estimate`，按它估全量超过 2400 秒就退 3、不跑全量（推的：一格起七八个 python 与一个多机配置判法，约 2–4 秒，全量约 13–26 分钟）。不编、不起 cargo，没有崩溃状态要枚举。

## 各格判定一览

| 格 | 情形 | 判定 | 依据 | 量的 / 推的 |
|---|---|---|---|---|
| 1 | 正常的一趟（服务交的两个根）在本机被判成「不是缺省的那一个」 | 不成立（不会误拒） | `task-server/task_server_scheduler_building.py:1495`、`scripts/gate/merge-and-verify-snapshot-lib.sh:413` | 推的；模型 `safe=yes` 那几格待主 agent 跑 |
| 1 | 同上，在第二台 | 不成立；而且到不了比对那一段 | `scripts/gate/merge-and-verify-snapshot-lib.sh:392-397` | 推的 |
| 1 | 符号链接（数据根、`work`、工作根各是链接） | 不成立（两边都 `realpath -m`） | `scripts/gate/merge-and-verify-snapshot-lib.sh:413` | 推的；模型 `work-symlink`、`data-link` 那几格待跑 |
| 1 | NFS 挂载点 | 不成立（本机是本地 ext4；第二台两串同前缀、同解法） | `findmnt` 现读 | 本机量的，第二台推的 |
| 1 | `SINGLEFS_DATA_ROOT` 被换了、工作根跟着它走 | **打中**：比对照放行，清编译目录拿假数据根下的槽那一层对着这一台真的编译根比，真的槽那一层各槽的编译目录当成槽已不在删掉 | `.claude/hooks/lib_work_root.py:210`、`scripts/gate/merge-and-verify.sh:1262`、`scripts/gate/merge-and-verify-snapshot-lib.sh:312` | 推的；模型 `data-other-data` 那几格待跑 |
| 1 | 只换 `SINGLEFS_DATA_ROOT`、工作根是服务给的真的那一个 | 成立而偏严：误拒一趟本可安全占槽的（只费编译） | 同上 | 推的 |
| 1 | 取不到缺省工作根时的出路 | 结局成立（不占也不清）；那一句「跑不起来」说的不是实情，原因只在 stderr | `scripts/gate/merge-and-verify-snapshot-lib.sh:410-411` | 推的 |
| 1 | 三格自证的假根会不会碰到真的 | 不成立（不会碰） | `scripts/gate/merge-and-verify-selftest-lib.sh:1807-1810` | 推的；日志里三格在两台都跑过 |
| 1 | 两条新开关与已有开关连带红新格而开关表没点名 | 推不出（推的：没有连带）；点这份脚本的开关复跑在跑、188 条判完 0 条 | 开关表 `scripts/gate/merge-and-verify.sh:1046-1050` | 待结局 |
| 4 | 第二轮判决写明不改的四处 | 都站得住（不改判定）；`run_build_directory.py:109` 那句的出路会把人引去修 merge-and-verify 自己，只影响读的人 | 第 4 格一节 | 推的 |
| 4 | 「已经量的」逐句 | 十三句里十一句核到；一句被日志推翻（看门狗开关复跑第 4 次尝试判到 128 条之前，第 122 条 `progressnote` 已经「没对上（不符）」）；两句不走服务、日志里核不到 | 第 4 格一节 | 量的（日志行） |

## 第 1 格：第二轮改法第 1 条的修补

### 1-①：两个根在本机、第二台各是什么，正常的一趟会不会被误拒

本机现读：多机配置的 `SHARED_DATA_ROOT` 就是 `/home/fy5090/.local/share/singlefs`（与 `NFS_SHARED_ROOTS` 第一段相同），家目录到 `singlefs`、`work` 各层都是目录、没有一层是符号链接，`realpath` 打回原样，`findmnt -T` 那一层是 `/`（本地 ext4）。这一条 shell 里 `SINGLEFS_DATA_ROOT`、`SINGLEFS_WORK_ROOT`、`SINGLEFS_MULTI_HOST_CONFIG` 都没设。

服务交给每次尝试的两个根都在服务那一侧算真路径：「        return os.path.realpath(library.work_root())」（`task-server/task_server_lib_configuration.py:433`）、「        return os.path.realpath(library.shared_data_root(dict(environment, **{MULTI_HOST_CONFIG_VARIABLE: path})))」（`task-server/task_server_lib_configuration.py:445`），任务自己带了的照任务带的：「        values = {lib.DATA_ROOT_VARIABLE: self.data_root, lib.WORK_ROOT_VARIABLE: self.work_root}」（`task-server/task_server_scheduler_building.py:1495`）。`lib_work_root.py shared work` 给的是 `SINGLEFS_DATA_ROOT` 下的 `work`。所以正常的一趟两边是同一串、或只差在 `work` 是不是链接；比对两边都解开：「    if [[ "$(realpath -m -- "$work_root")" != "$(realpath -m -- "$default_work_root")" || "${MERGE_AND_VERIFY_BREAK:-}" == "slot-default-work-root-refused" ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:413`）。数据根经链接给、`work` 是链接、工作根经链接给，三种都解成同一个真路径。判：**不成立**（推的）。模型里配得上对（`safe=yes`）的格量这一句：两臂都应当 `verdict=ok`，有一格 `false-refusal` 就推翻。

第二台：服务「第二台原样带本机的值」（`task-server/task_server.py:613` 那一行的原话），两串在第二台同前缀，`realpath -m` 解链接只看链接内容（NFS 照样导出），解出来仍相等。而且派到第二台的那一趟经 `scripts/hosts/multi-host-run.sh` 拿到 `CARGO_TARGET_DIR`，`choose_slot` 头一段「  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:392`）就退了，到不了比对。判：**不成立**（推的；第二台那一层一律给编译目录是第一、二轮攻过的说法，这里不重攻）。日志旁证：带新三格的整份自证与 `slot-default-work-root-refused` 单跑都派到了第二台（`task-server/task status` 现读「第 1 次：第二台」），自证里「工作根就是缺省的那一个时照常占槽」那一格在第二台绿，说明比对在 NFS 上的工作根下照占（那一格的假根在第二台本地 `/tmp` 下，不在 NFS 上，只算旁证）。

### 1-①b：打中——换了 `SINGLEFS_DATA_ROOT`，工作根跟着它走

反例输入（交服务的整条，前缀与故障固定，用户决定的只有带哪两个根）：

```
task-server/task submit -- env SINGLEFS_DATA_ROOT=/home/fy5090/.local/share/singlefs/work/<某个隔离目录>/data \
  SINGLEFS_WORK_ROOT=/home/fy5090/.local/share/singlefs/work/<某个隔离目录>/data/work bash scripts/gate/merge-and-verify.sh <参数…>
```

（或 `env -u SINGLEFS_WORK_ROOT SINGLEFS_DATA_ROOT=…`；或提交方自己的环境里带这两个，服务照带：「    "SINGLEFS_RUN_SCOPE", WORK_ROOT_VARIABLE, DATA_ROOT_VARIABLE, MULTI_HOST_CONFIG_VARIABLE, "SINGLEFS_KEEP_TEST_TEMPORARY_FILES",」（`task-server/task_server_lib_constants.py:402`），提交时只核在不在共享根下，工作根下的目录在。）

每一步许可它的那一句：

1. 没给自证的登记与编译根，`choose_slot` 照常取工作根：`work_root` 是 `SINGLEFS_WORK_ROOT`（或不设时数据根下的 `work`）。
2. 缺省工作根取 `lib_work_root.py shared work`（`scripts/gate/merge-and-verify-snapshot-lib.sh:410`），它看 `SINGLEFS_DATA_ROOT`：「    configured = "" if break_mode() == "data-root-ignored" else environment.get(DATA_ROOT_VARIABLE, "")」（`.claude/hooks/lib_work_root.py:210`）。两边都是假数据根下的 `work`，`:413` 判相等，放行。
3. 编译根不看 `SINGLEFS_DATA_ROOT`，只看多机配置：「path = copying.configuration_path(root)」（`scripts/gate/merge-and-verify.sh:1262`），配置的位置「    return environment.get(CONFIGURATION_VARIABLE) or os.path.join(main_working_tree(source_root), CONFIGURATION_FILE_NAME)」（`scripts/hosts/lib_repository_copy.py:913`）。拿到的是这一台真的编译根。
4. 登记 3 槽：在假的槽那一层新开一槽，占到之后「  sweep_slot_targets "$slot_root" "$build_root/$SLOT_BUILD_DIRECTORY_NAME"」（`scripts/gate/merge-and-verify-snapshot-lib.sh:444`；没占到走 `:438` 同一句）；登记 0 槽：「      sweep_slot_targets "$SLOT_ROOT_DIRECTORY" "$build_root/$SLOT_BUILD_DIRECTORY_NAME"」（`scripts/gate/merge-and-verify-snapshot-lib.sh:427`）。
5. `sweep_slot_targets` 拿假槽那一层逐个比真编译根下的 `slot.*`：「    [[ ! -e "$1/$name" ]] || continue」（`scripts/gate/merge-and-verify-snapshot-lib.sh:312`），假那一层里没有同名的，就「    if output="$(python3 "$RUN_BUILD_DIRECTORY" remove "$directory/target" 2>&1)" && rmdir -- "$directory" 2> /dev/null; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:313`）。此刻真的槽那一层有 `slot.4u3rewvh`（别的会话有意留着的，`kept` 在）与 `slot.kyz05duu` 两槽（`ls` 现读），它们在这一台编译根下的编译目录没有进程开着时都删。

后果与第二轮打中的那一格同型：判定不会错，代价是那几槽下一趟从头编（第二轮判决记两槽 105G、91G）。第二轮攻方腿写过「fix-same-root 只在模型里拿 W1 当缺省工作根，真改要拿 `lib_work_root.py` 的缺省值（数据根下的 `work`）比，SINGLEFS_DATA_ROOT 也被改过时还要再核」（`research/prompts/merge-verify-slots-governance-r2-opus-output.md:96` 那一行里的话），这一句换上时没核。

打中之后的四句：

- 分不分辨臂：分辨。`intact` 在 `data=other-data` 且工作根跟着它（`work=unset`、`work=other-data-work`）的格上 `verdict=deleted-paired-targets`，`fix-configured-root` 同一格 `verdict=ok`（推的，模型待跑）。
- 系统看不看得到判别它的东西：看得到。编译根读的那一份多机配置里就有 `SHARED_DATA_ROOT`，`lib_work_root.py` 的 `configured_data_root` 只读配置、不看环境，同一刻取得到。
- 满足判据字面的哪一句：正文「问题」一节「判这些修补本身对不对、有没有新毛病」。按字面它满足的是第二轮改法第 1 条的条件句「工作根要等于这一台缺省的那一个」（`research/prompts/merge-verify-slots-governance-r2-main-verification.md:68`）——文件头把「缺省」定义成「lib_work_root.py 不设 SINGLEFS_WORK_ROOT 时给的…数据根下的 work」（`scripts/gate/merge-and-verify.sh:90-91`），所以代码照字面没错；错在条件句推不出它自己给的理由「换了工作根，清编译目录就拿那个工作根的槽那一层…对着真的编译根比」（`scripts/gate/merge-and-verify.sh:92-93`）：数据根一换，「缺省」跟着换，编译根不换。属「判据自己写错」那一类，不是判别子观测不到。
- 跑前条款给的改法在这一格上还中不中：第二轮判决只给了两条。`fix-same-root`（已换上的就是它，缺省取 `shared work`）在这一格上照中；`fix-namespace`（编译根下按工作根分层）推的不中，但第二轮判决以「此刻两槽 105G、91G 的编译目录会成孤儿」没取。

只换 `SINGLEFS_DATA_ROOT`、工作根仍是服务给的真的那一个（`data=other-data`、`work=real-work` 或 `data-work`）：`intact` 判成不是缺省的、不占也不清，而这一趟的槽那一层与编译根本是配得上对的。偏严，只费一次从头编。模型里这几格 `intact` 应当是 `false-refusal`、`fix-configured-root` 是 `ok`（推的）。

### 1-②：取不到缺省工作根时的出路

「    if ! default_work_root="$(python3 "$root/.claude/hooks/lib_work_root.py" shared work)" || [[ -z "$default_work_root" ]]; then」（`scripts/gate/merge-and-verify-snapshot-lib.sh:410`）之后是不占也不清、照一趟一份跑。

- 什么时候走到：`:405` 那一条不带参数的 `lib_work_root.py` 成了、`shared work` 不成，只有「设了 `SINGLEFS_WORK_ROOT`、数据根取不到」这一种（`SINGLEFS_WORK_ROOT` 设了，取工作根就不读数据根）。经服务交的到不了：服务每次都带 `SINGLEFS_DATA_ROOT` 的真路径，多机配置没写 `SHARED_DATA_ROOT` 的服务起不来（「    """control install、start、check 判多机配置（文件头「共享数据根」）：只看配置里 SHARED_DATA_ROOT 那一行（lib_work_root.py 的 configured_data_root：写了、绝对路径、」，`task-server/task_server_lib_configuration.py:451`）。剩下的是不经服务、或命令里自己 `env -u SINGLEFS_DATA_ROOT` 又碰上配置写坏。
- 结局：**成立**。取不到就证不了槽那一层与编译根配得上对，不占也不清是安全的那一边；登记 0 槽时连多出来的槽也不清，下一趟再清，不丢东西。
- 说法：**不准**（推的）。那一句写「`lib_work_root.py shared work` 跑不起来」，能走到的那一种它跑起来了、退 2，原因与出路打在它自己的标准错误上（「        print(f"  → 怎么办：{DATA_ROOT_HOWTO}", file=sys.stderr)」，`.claude/hooks/lib_work_root.py:500`），不进 `SLOT_FALLBACK_REASON`，槽位汇总里只剩「跑不起来」。改成「退非 0 或打空行（原因见它打在标准错误上的那一句）」才是实情。只影响读的人。

### 1-③：三格自证的假根会不会碰到真的

**不成立**（推的）。三格都在子 shell 里把 `SINGLEFS_DATA_ROOT`、`SINGLEFS_MULTI_HOST_CONFIG`、`SINGLEFS_WORK_ROOT` 换成这一格的假的（「      export SINGLEFS_DATA_ROOT="$base/data" SINGLEFS_MULTI_HOST_CONFIG="$base/multi-host.env" SINGLEFS_WORK_ROOT="$2"」，`scripts/gate/merge-and-verify-selftest-lib.sh:1805`），调之前在同一份环境里先取编译根与缺省工作根，不是这一格的假的就不调（「      seen_build="$(slot_build_root)"; seen_default="$(python3 "$root/.claude/hooks/lib_work_root.py" shared work 2>&1)"」，`scripts/gate/merge-and-verify-selftest-lib.sh:1807`）。`choose_slot` 读的只有这三样：编译根与缺省工作根是同一份环境里同两条命令的第二遍，工作根是参数给的假目录。会删东西的三处（清多出来的槽、`sweep_slot_targets`、`trim_slot_target`）的两头都落在这一格的根下。弄坏开关下也一样：`slot-work-root-unchecked` 单跑那一趟删的是自证临时目录下的（日志「删了槽已不在的编译目录 /tmp/singlefs-run-with-memory-cap-1958721-ITuARG/tmp.QwJvECo8Yv/slot-default-other-root-zero-slots/build/merge-and-verify-slots/slot.idle」那一句）。想得到的能碰到真的的环境只有临时目录与真编译根重合，`mktemp -d` 给的新目录做不到。

自证的一处够不着：三格里假的 `SINGLEFS_DATA_ROOT` 与假配置的 `SHARED_DATA_ROOT` 永远是同一个（`:1805` 与「      "NFS_SHARED_ROOTS=$base" "SHARED_DATA_ROOT=$base/data" "HOSTS=127.0.0.1 192.0.2.81" > "$base/multi-host.env"」，`scripts/gate/merge-and-verify-selftest-lib.sh:1799`），1-①b 那一格（数据根换了、工作根跟着走）没有一格造，改法表里补一格。

### 1-④：两条新开关与已有开关会不会连带红新格

两条新开关单跑各只红点名的格，日志核到（第 4 格「已经量的」第 3、4 句）。已有开关：

- 「只给另一个工作根」那两格在 `:409-416` 就退了，后面挑槽、清槽、建编译目录、关增量的开关都碰不到它们；能让它们变的只有关掉这一道比对的 `slot-work-root-unchecked`。
- 「工作根就是缺省的那一个」那一格判三样：占的槽在真路径的槽那一层下、为什么为空、留着的 `slot.kept` 与它的编译目录原样。逐条推：`slot-not-reused` 新开的一槽也解成真路径（「            held = os.path.realpath(tempfile.mkdtemp(prefix="slot.", dir=root))」，`scripts/gate/merge-and-verify.sh:1343`）；`slot-kept-taken` 照 `used` 挑最新的，`slot.idle` 比 `slot.kept` 新一天（「    touch -d '-1 day' "$base/data/work/merge-and-verify/slot.kept/used"」，`scripts/gate/merge-and-verify-selftest-lib.sh:1794`），挑的仍是 `slot.idle`；`slot-limit-ignored`、`slot-surplus-*` 在两槽对上限 3 时不起作用；`slot-target-on-shared-root` 只改编译目录建在哪，这一格不判；增量三条没有 `incremental/` 可删；`SINGLEFS_WORK_ROOT_BREAK` 在子 shell 里 unset 了。
- 判：**推不出**，推的是不连带。点这份脚本的开关复跑在跑，日志此刻「开关 188 条、已跑完 2 趟、判完 0 条」，结局出来才算。
- 旁记：`.claude/gate.d/break-switch-replay-files.tsv:73` 理由里写「一百八十六条开关」，复跑那一趟数出 188 条；调度记录已记「第 73 行理由里的开关条数要跟着改」，这里只核到它还没改。

## 第 4 格：第二轮只抽查过的地方

### 4-①：第二轮判决写明不改的四处

第二轮判决原话：「攻方另记的几处只多占盘（不占槽那一趟照写 `incremental/`、调用方自设 0 时旧缓存不删、超上限清理排在删增量之前）与 `scripts/hosts/run_build_directory.py:109` 那句说法的对象，不改判定，这一轮不改。」（`research/prompts/merge-verify-slots-governance-r2-main-verification.md:44`）

| 处 | 判 | 依据（推的） |
|---|---|---|
| 不占槽那一趟照写 `incremental/` | 站得住 | 那一趟的编译目录在这一台 `${TMPDIR:-/tmp}` 下新建、收尾第一步整份删（`scripts/gate/merge-and-verify.sh:76-78`），多出来的只是这一趟编的过程中的盘；不进判定。只在临时目录所在的盘小到装不下时把这一趟写红，红是红、不会把红判成绿 |
| 调用方自设 `CARGO_INCREMENTAL=0` 时旧缓存不删 | 站得住 | 只走 `scripts/gate/merge-and-verify-snapshot-lib.sh:367-369` 那一支；`CARGO_INCREMENTAL` 不在服务照带的变量名单里（`task-server/task_server_lib_constants.py:401-404`），经服务只能写成 `-- env CARGO_INCREMENTAL=0`。旧缓存留到下一趟不自设的来删，期间算进 `trim_slot_target` 的量，超 150 GiB 就整份删、从头编一次 |
| 超上限清理排在删增量之前 | 站得住 | `trim_slot_target 开跑前`（`:445`）在 `slot_incremental_off`（`:448`）之前；关增量之后每一槽头一趟就把 `incremental/` 删了、之后不再攒，只有关增量之前攒下、之后还没被占过的那一槽会多从头编一次 |
| `run_build_directory.py:109` 那句的对象 | 站得住（不改判定），但不只是措辞 | 「        lines = [f"  ✗ 这一趟的编译目录 {directory} 还有 {len(holders)} 个进程开着里面的文件，没删（起它的那一趟的子进程没收干净）："]」（`scripts/hosts/run_build_directory.py:109`）嵌进槽位的三处（`trim_slot_target`、`sweep_slot_targets`、`slot_incremental_off`）。三处删的都不是「这一趟的编译目录」：开跑前与 `sweep` 时开着文件的只能是早先那一趟漏下的进程（锁排他），出路「修起它的脚本，让它停自己的子进程之后再删」把人引去修这一趟的 merge-and-verify。行为不变，只影响读的人 |

### 4-②：「已经量的」逐句核（日志在 `~/.local/state/singlefs/task-server/logs/`，命令与结局取自 `task-server/task status <号>`）

| # | 背景材料的那一句 | 核到的原文 | 判 |
|---|---|---|---|
| 1 | 第 1 条 `hook-swap.py` 先 dry-run 再真换都 type=succeeded exit=0「换上了 3 份」 | dry-run 那一条「结局 succeeded、退 0」，日志「--dry-run：停在草稿，主工作区一个字节没写」；真换那一条「结局 succeeded、退 0」，日志「✓ 换上了 3 份（改名换上新 inode、回读一致），主工作区里再验过：3 份文件语法检查、内嵌 python 0 段编译」，三份「换上之后应当是」的 sha256 与开工快照逐份相同 | 核到；「换上了 3 份」只是真换那一条的句子 |
| 2 | 整份自证 type=succeeded exit=0「自检通过（查了 202 种）」、`✗ 自检` 0 行 | 「结局 succeeded、退 0」，日志「  ✓ 自检通过（查了 202 种）：测试红时退 1 并列出红的与重跑命令……」，`grep -c '✗ 自检'` 得 0；「第 1 次：第二台」 | 核到（派在第二台） |
| 3 | `slot-work-root-unchecked` 单跑 type=command-failed exit=1，只红两格 | 「结局 command-failed、退 1」；`✗ 自检` 恰好 2 行，开头是「  ✗ 自检：槽位：登记三槽、只给另一个工作根而不给自证的登记与编译根时不占也不清，原工作根下各槽的编译目录都在：实际 占了槽……」与「  ✗ 自检：槽位：登记 0 槽、只给另一个工作根而不给自证的登记与编译根时不清，原工作根下各槽的编译目录都在：实际 没占槽的为什么是……」 | 核到（本机） |
| 4 | `slot-default-work-root-refused` 单跑 type=command-failed exit=1，只红一格 | 「结局 command-failed、退 1」；`✗ 自检` 恰好 1 行「  ✗ 自检：槽位：不给自证的登记与编译根、工作根就是这一台缺省的那一个时照常占槽：实际 占的槽「」不在缺省工作根的槽那一层……」 | 核到（第二台） |
| 5 | 点这份脚本的开关复跑交了、等结局 | 命令 `bash .claude/gate.d/code-tooling.sh --check break-switch-replay --item scripts/gate/merge-and-verify.sh`「running」，日志「  · break-switch-replay 还在跑：scripts/gate/merge-and-verify.sh 开关 188 条、已跑完 2 趟、判完 0 条、用了 301 秒」 | 核到（在跑） |
| 6 | 第 2、3 条 `hook-swap.py` 先 dry-run 再真换都 type=succeeded exit=0 | 两条都「结局 succeeded、退 0」；真换日志「✓ 换上了 3 份（改名换上新 inode、回读一致）……」，三份「换上之后应当是」与开工快照逐份相同 | 核到 |
| 7 | `crash-library-totals.sh --selftest` type=succeeded exit=0 | 「结局 succeeded、退 0」，日志「  ✓ crash-library-totals.sh --selftest：11 格都对，起了 15 趟本脚本（文件头 selftest-serial：按次序跑）」 | 核到（第二台） |
| 8 | `agent-watch.py --selftest` type=succeeded exit=0「自检通过」、`✗ 自检` 0 行 | 「结局 succeeded、退 0」，日志「  ✓ agent-watch 自检通过：……」，`✗ 自检` 0 行 | 核到（第二台） |
| 9 | `AGENT_WATCH_BREAK=progresstailhead` 单跑 type=command-failed exit=1，只红一格 | 「结局 command-failed、退 1」；日志 3 行，`✗ 自检` 1 行「  ✗ 自检：进度文件行数多于 PROGRESS_TAIL_LINES 时，告警里附的应当是末几行（['进度行03', '进度行04', '进度行05']）……」 | 核到（本机） |
| 10 | 点 `scripts/agents/agent-watch.py` 的开关复跑在跑（第 4 次尝试，到 128 / 140 条都对上） | 第 4 次尝试日志在；但「  · break-switch-replay 第 122 / 140 条没对上（不符）：scripts/agents/agent-watch.py AGENT_WATCH_BREAK=progressnote，用了 92 秒」排在「  · break-switch-replay 第 128 / 140 条对上：scripts/agents/agent-watch.py AGENT_WATCH_BREAK=repositorychangesblind，用了 91 秒」之前（日志第 9619 行对第 10361 行） | **被日志推翻**：到 128 条时已有 1 条不符；此刻判完 139 条、对上 138 条 |
| 11 | 第 4 条 `replace-batch.py` 先 dry-run 再写「1 处替换」回读一致 | 不经服务，任务日志里没有 | 核不到；只核到 `.claude/agent-common.md` 此刻 sha256 与开工快照相同 |
| 12 | 第 5 条 `replace-once.py` 逐处「命中 1 次」回读确认 | 同上 | 核不到；三份登记此刻 sha256 与开工快照相同 |
| 13 | 改之前那一趟门禁两格与 preflight-lint 见第二轮判决，改之后没再跑 | 本会话交的那两条（`code-tooling.sh --check repository-copy-closure,memory-bounded-parallelism`、`preflight-lint.py`）都「结局 command-failed、退 1」；之后本会话交的任务里没有再起这两条 | 核到，只在本会话交的任务里核（别的会话交的没核） |

第 10 句不符的原因（推的，归正推腿第 2 格，这里只交线索）：`AGENT_WATCH_BREAK=progressnote` 让告警不附进度文件的末几行，第二轮改法第 3 条新加的那一格要找「末 3 行」，照样红；开关表那一条（`scripts/agents/agent-watch.py:404-406`）只点名早先两格，没点新格。核法：等这一趟结局，读它的不符明细里 `progressnote` 那一节是不是多红了「进度文件行数多于 PROGRESS_TAIL_LINES 时……」一格。那一条任务在服务里的长跑确认备注（`task-server/task status` 现读）同样写「已判完 128 条都对上」，调度记录第 183 行也写「已判完 128 / 140 条都对上」（第 182 行「到 110 / 140 条都对上」那一刻还对，第 122 条在它之后），备注与第 183 行要跟着改。

## 改法（攻方腿自己提的，被攻过零轮）

| 改法 | 修哪一格 | 量过 / 推的 |
|---|---|---|
| `fix-configured-root`：`scripts/gate/merge-and-verify-snapshot-lib.sh:410` 的缺省工作根改取多机配置 `SHARED_DATA_ROOT` 下的 `work`（`lib_work_root.py` 的 `configured_data_root`，只读配置、不看 `SINGLEFS_DATA_ROOT`；与 `SLOT_BUILD_ROOT_PROGRAM` 读同一份配置，槽那一层与编译根出自同一处） | 1-①b 两种（换了数据根而照删、只换数据根而误拒） | 推的，待主 agent 跑（模型 `fix-configured-root` 臂）。代价（推的）：配置没写 `SHARED_DATA_ROOT` 而环境里设了 `SINGLEFS_DATA_ROOT` 的那一趟由占槽变成不占；服务没有这份配置起不来，经服务交的碰不到 |
| 自证补两格：`SINGLEFS_DATA_ROOT` 换成另一个、工作根跟着它（登记 3 槽、0 槽各一），配置的槽那一层里各槽的编译目录都在；配一条开关让缺省工作根照 `SINGLEFS_DATA_ROOT` 取（即现行写法），它要红这两格 | 1-③ 那一处够不着 | 推的 |
| `:411` 那一句改成「退非 0 或打空行（原因见它打在标准错误上的那一句）」 | 1-② 说法 | 推的 |
| `scripts/agents/agent-watch.py:404-406` 的 `progressnote` 条目补点名新格 | 4-② 第 10 句 | 推的，待看门狗开关复跑的不符明细 |

预期读数（跑之前写下，推的；量出相反的数就照它改结论：`intact` 的 `deleted_paired_targets` 是 0 就撤回 1-①b）：

```
name=slot_root_pairing_summary arm=intact cases=192 ok=152 deleted_paired_targets=16 false_refusal=24 accepted_unpaired=0 other=0 guard_refused=0 prediction_miss=0
name=slot_root_pairing_summary arm=fix-configured-root cases=192 ok=192 deleted_paired_targets=0 false_refusal=0 accepted_unpaired=0 other=0 guard_refused=0 prediction_miss=0
name=slot_root_pairing_leftover_processes count=0 …
```

`intact` 的 16 格是 `data=other-data` × `work` 不设或 `other-data-work` × 两布局 × 配置两写法 × 两种槽数；24 格是 `data=other-data` × `work` 为 `data-work`、`data-link-work`、`real-work` × 同样的 8 种。`prediction_miss` 不为 0 说明我对 `:413` 的读法错了，那一格的 `choose.out` 在运行目录 `probe-output/s/<格名>/` 下。

## 没打中的形状

- 正常的一趟被误拒：数据根经链接给、配置里的数据根是链接、`work` 是指向别处的链接、工作根经链接给、工作根给真路径，五种都推的不误拒；模型扫 `SINGLEFS_DATA_ROOT` 四种 × `SINGLEFS_WORK_ROOT` 六种 × 两布局 × 配置两写法，`safe=yes` 的格两臂都要 `ok`。没扫：相对路径与 `~` 起头的 `SINGLEFS_WORK_ROOT`（服务提交时拒相对路径，`lib_work_root.py` 展开 `~` 与 `realpath -m` 解相对路径照同一个当前目录，推的相等）。
- NFS 挂载点：本机那一层是本地盘；第二台两串同前缀、同解法，推的不误拒；第二台到不了比对。
- 自证三格碰真的：子 shell 换掉三样、调之前核两样，弄坏开关下删的也在临时目录下，没找到能碰到真的环境。
- 开关连带：逐条推了槽位一族十几条，没找到会红新格的；最终以在跑的开关复跑为准。
- 第二轮不改的四处：都不改判定，没找到能把红判成绿的。

## 这条腿自己的限度

- 模型一格都没跑过，上面的读数全是推的；判决要引的数由主 agent 复跑。
- 模型用仓里此刻的 `lib_work_root.py`、`lib_repository_copy.py`、`multi_host_addresses.py`、`run_build_directory.py`（只读，开头打出各自的 sha256），不是冻结的；别的会话之后改了它们，读数可能变。
- 每格的 `choose_slot` 在模型自己的子 shell 里起，没有放进单独的进程组；它起的挑槽进程由冻结库自己的 `release_slot` 按进程号停，模型末尾只读 `/proc` 数一遍参数里带格目录的进程、只报不停。
- 1-①b 的触发要有人在交 merge-and-verify 时带一个换掉的 `SINGLEFS_DATA_ROOT`（工作根跟着它）；此刻仓里找没找到这样起它的脚本，我只搜了 `scripts`、`.claude/gate.d`、`.claude/hooks`、`task-server` 下点名 `merge-and-verify.sh` 的文件，没逐个读完它们怎么起。
- 「已经量的」只读了本会话交的任务与它们的日志；第 11、12 句不走服务，核不到。别的会话交的那一趟点 `merge-and-verify.sh` 的开关复跑（在这一批换上之前）没读明细。
- 没读真的编译根（不碰）；「两槽 105G、91G」取自第二轮判决。

## 要主 agent 跑的命令

1. `mkdir -p /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-opus-pairing-run`
   - 在哪起：仓根；不交服务。
   - 看什么：目录在、是空的。
2. `task-server/task submit --cwd /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-opus-pairing-run -- bash /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r3-opus-model/tools/rerun-from-a-clean-work-directory.sh /home/fy5090/code/singlefs /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-verdict/frozen /home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-opus-pairing-run /home/fy5090/code/singlefs/research/prompts/merge-verify-slots-governance-r3-opus-model`
   - 在哪起：`--cwd` 是上一项建的运行目录（往上没有 `.git`，服务按它本身给编译目录；模型不编，每格里 unset 了服务给的 `CARGO_TARGET_DIR`）。
   - 看什么：`task wait` 末行 `TASK_SERVER_RESULT` 的 `type=succeeded exit=0`（`exit=3` 是冻结副本对不上或估时超 2400 秒，日志里有 `name=slot_root_pairing_estimate` 那一行的估数）；`task-server/task log <号> --all` 里两行 `name=slot_root_pairing_summary` 与上面「预期读数」逐字段比，`name=slot_root_pairing_leftover_processes count=0`；逐格的 `name=slot_root_pairing` 行里 `verdict=deleted-paired-targets` 的那几格只应在 `arm=intact data=other-data` 上。判决引完数之后删运行目录，服务跟着删那一组编译目录。
3. 等在跑的两趟开关复跑的结局（已交，不重交）：点 `scripts/gate/merge-and-verify.sh` 的那一趟看两条新开关与已有开关的不符明细里有没有新三格；点 `scripts/agents/agent-watch.py` 的那一趟看 `progressnote` 那一节是不是多红了「进度文件行数多于 PROGRESS_TAIL_LINES 时……」一格。

## 没做什么

- 不判正推那几格（第 2、3 格）；第 10 句不符的原因只交线索。
- 没往统一任务服务交任务；模型没跑，判决要引的数没复跑（命令在上一节）。
- 不替主 agent 采纳；改法被攻过零轮，也没量过。
- 没改被判文件，没碰真的工作根与编译根（只 `ls` 了真的槽那一层的目录名）。
- 没跑门禁与 lint。

## 引文核对与草稿清理

草稿目录 `/home/fy5090/.local/share/singlefs/work/merge-verify-slots-r3-opus/` 里只有三份 `task-server/task status` 的读数，没建仓副本与编译目录，没有要删的。
