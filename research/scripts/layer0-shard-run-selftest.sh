#!/usr/bin/env bash
# research/scripts/layer0-shard-run.sh --selftest 的本体：不碰第二台。拷一份仓进临时目录、git init，登记一条 shard=across-machines 的用例，
# 「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），本机两个进程各跑一片，走驱动脚本同一条路：
# 工具链与输入指纹两边比、清场（回读；跑完不复原）、两片、拷账本、merge、判、写 checker-tier-layer0-replay 那一格全绿标记。
# 默认用假 cargo（不编译、不跑用例）：登记的是临时仓里新写的一条标了 #[ignore] 的替身用例，三趟都不带 --include-ignored，不算重型；
# 假 cargo 分片跑时往进度目录写账本、merge 时核两份账本在不在、记的输入指纹对不对，打与层 0 同形的计数行与带 shards= 的线程行。
# 带 SINGLEFS_HEAVY_TESTS（commit 或 user-request）时换真 cargo：登记 crates/singlefs-checker-tier/tests/crash_enumeration_sharded_across_processes.rs
# 里不标 ignore 的小流用例（第一条流按甲二展开 29 个状态），release 下真编真跑；不带时成功行写明真 cargo 那一趟本次未跑。逐格核：
#   ① 不分片跑那条用例打的计数行，与双机分片 merge 之后记进全绿标记的那一行逐字相同；标记作数；清了场；第二台这一趟的目录删了
#   ② --merged-log（门禁 checker-tier-layer0-replay 调的那一条）：merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数
#   ③ 「第二台」的 rustc -Vv 第一行不同 ⇒ 拒（工具链不同），标记不动
#   ④ 「第二台」那一片没写账本 ⇒ 判红，删这批输入那一格标记
#   ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒（退 1），出路指到 multi-host.env.example
#   ⑥ 没登记 shard=across-machines 的用例 ⇒ 拒
#   ⑦ 发现日志：--merged-log 时三趟各有一份，merge 那一份是 <日志文件>.findings.tsv（一节、begin 行不带 shard=、有 summary），
#      两片的在各自日志旁边（begin 行带 shard=0/2、shard=1/2），第二台上那一份拷回之后删了，三份路径都打进驱动的输出；单独跑时三份在 common-dir 下这一趟的目录里
# 假 cargo 设了 SINGLEFS_LAYER0_FINDINGS_FILE 就往里追加一节（行格式照 crates/singlefs-checker-tier/src/crash.rs 的发现日志），标准输出另打一行 LAYER0_FINDINGS。
#   ⑧ 第二台那一片经 research/scripts/run-with-memory-cap.sh 起（上限是配置的 PEER_MEMORY_CAP）：假 cargo 在 1/2 那一片记下自己的 cgroup，
#      要在 singlefs-memory-cap- 那个 scope 里；驱动的输出里那一行写着经它起；配置缺 PEER_MEMORY_CAP、写成不带单位的数，配置判法拒
#   ⑨ 树里没有规范副本 .claude/singlefs-ai-sop/（HEAD + 暂存区的 worktree 就是这样：它在 .gitignore 里）：树是临时仓的一个 orphan worktree，
#      「第二台」那棵树 git init 过、找不回主仓；驱动在 ② 把驱动所在主仓（临时仓）的规范副本拷过去，第二台算得出同一个输入指纹，判绿，这一趟的树删了
#   ⑩ ③ 之后失败（清场之后回读没过）⇒ 退 1，「第二台」上这一趟的树照 ⑦ 删掉，两片的进度目录留着
#   ⑪ 本机设了 RUN_WITH_MEMORY_CAP_RESERVE=10G ⇒ 第二台那一条 run-with-memory-cap.sh 带同一个值（假 cargo 在 1/2 那一片记下它）；没设（① 那一趟）就不带
#   ⑫ 设了 SINGLEFS_LAYER0_PEER_MEMORY_CAP=<覆盖值> ⇒ 覆盖配置的 PEER_MEMORY_CAP（第二台那一片的 scope memory.max 是覆盖值）；没设（① 那一趟）照配置值；
#      设成不带单位的数 ⇒ 拒（退 1）
#   ⑧ ⑫ 的配置值 / 覆盖值按模式取：假 cargo 1G / 2G；真 cargo 8G / 12G（取值时第二台还在这个上限里编测试二进制；编译挪出去之后测试那一趟要多少没量过，
#      没往回改）。scope 的 memory.max 只在假 cargo 模式核
#   ⑬ 两片跑到一半给驱动（只给它自己，kill "$!"）发 TERM ⇒ 驱动先停两片（两片的测试进程里记下的进程号都不在了），再删「第二台」这一趟的树
#      （不复原），退 143；两片的进度目录还在。发 TERM 之前轮询到两片的进程号都记下了才发；等的上限 hold_wait_seconds：
#      假 cargo 600 秒，真 cargo 3600 秒（第二台那棵树每一趟从头编）；设了 LAYER0_SHARD_SELFTEST_HOLD_SECONDS=<秒> 就用它（只供证「没跑成」那一支）。
#      到上限还没都记下：照样发 TERM 收尾，这一格判「没跑成」（✗ 那一行写「没跑成」，不算通过，自证退 1）；驱动先退了判这一格错。
#      两片记下进程号之后停住等驱动来停，至多停「上限 + 120 秒」。
#      这一趟设只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_EXIT_HOLD_FILE（自证按住第二台那一片的外层，放走之前它写不出退出码文件）：发 TERM 之后
#      按住 15 秒（驱动等外层的上限是 30 秒），驱动这期间报了第二台那一片停没停（「第二台那一片（测试二进制）：」「第二台那一片没停下」那一行）或者退了，
#      就是没等外层就往下收尾，判这一格错；驱动退了再放（至多按住 15 秒），没等的驱动删完之后外层又写出一份 runs/<这一趟>.shard-1-of-2.exit。
#      驱动退了之后要报过第二台那一片停没停。第 1/2 片记下外层的进程号，自证等它退了（至多 120 秒）再看 runs/
#   ⑯ 第二台编测试二进制经 run-with-memory-cap.sh、在编译的上限 SINGLEFS_LAYER0_PEER_BUILD_MEMORY_CAP 里（自证设：假 cargo 3G，真 cargo 16G），
#      不在那一片的 PEER_MEMORY_CAP 里：编译那一趟记下的 cgroup 是 singlefs-memory-cap- 的 scope、memory.max 是编译的上限；驱动 ④ 那一行写着它
#   ⑰ 编译的上限太小（假 cargo 64M、编译那一趟先占 256 MiB；真 cargo 128M、真编）⇒ 驱动退 1、报「编译上限太小」，第二台这一趟的树删了
#   ⑱ 跑完不调复原命令：配置的 QUIESCE_START_COMMAND 只往临时目录的 restore-calls 记一笔，① 到 ⑬ 的各趟（跑完、跑红、被 TERM 打断）一笔都没记；
#      ① 与 ⑬ 那两趟驱动报了「清了场、没复原」（⑬ 没跑成时只看 ①）；配置里复原那一对写成空串（清场那一对照写），配置判法照样判得过（退 0）
#   ⑲ 第二台那一边的内存包装认出显存不够：驱动的环境里设 RUN_WITH_MEMORY_CAP_JOURNALCTL 指到假 journalctl（打一行 NVRM … NV_ERR_NO_MEMORY），
#      编测试二进制那一趟（LAYER0_SHARD_SELFTEST_PEER_BUILD_FAIL，只在第二台那棵树里）或第 1/2 片（LAYER0_SHARD_SELFTEST_PEER_SHARD_FAIL）一起就退 3，
#      第二台的包装退 245 ⇒ 驱动退 1、报「显存不够」（编译那一趟判驱动那一支独有的「第二台编测试二进制那一趟显存不够」在、
#      包装退 250 那一支的「第二台编测试二进制撞了编译的内存上限」与「编译上限太小」都不在；不拿「显存不够」单字判，包装自己也打它），第二台这一趟的树删了
#   ⑳ 第二台那一片的内存包装收尾拉长：同 ⑲ 第二格，只是假 journalctl 每次先睡 12 秒（包装退非 0 时调两次，内层退了之后外层约 24 秒才写退出码文件）
#      ⇒ 驱动看外层还在、接着等，不判 vanished，报「第二台那一片退 245：显存不够」、不算结果；第 1/2 片记下外层，自证等它退了（至多 120 秒）再看 runs/，与这一格开跑之前相同
#   ㉑ 第二台那一片的外层没留退出码文件就没了：第 1/2 片把退出码文件换成指向不存在的目录里的符号链接，外层写不进去就退了 ⇒ 驱动退 1、
#      报「第二台那一片退 vanished（外层进程没了、没留退出码）」，runs/ 与这一格开跑之前相同。驱动放后台跑，外层没了 90 秒还不退就只给它发 TERM、判这一格错；
#      外层到 hold_wait_seconds（同 ⑬）还没记下或没退，发 TERM 收尾，这一格判「没跑成」
# 两片的测试进程里的自证记录（⑭ 标准输出指向、⑧ ⑪ ⑫ cgroup、⑬ 记下进程号、停住等驱动来停）与编译那一趟的记录（⑯ cgroup、⑰ 先占内存）在临时目录的
#   shard-hooks.sh、build-hooks.sh 里：假 cargo 自己调；真 cargo 模式 PATH 最前面放一层 cargo，别的照转给真的，cargo test … --no-run --message-format=json
#   时先调编译那一趟的记录，再把它交出的测试可执行文件换成临时目录里的替身（先调两片的记录，再 exec 真的测试二进制），所以真 cargo 下这几格也判得到。
# 自证起的命令不算进调用方会话的「日常一条」：开头清掉 CLAUDE_CODE_SESSION_ID（照 run-with-memory-cap.sh 自证 in_context 的写法；本机当第二台时
#   驱动起的那几条包装带着整份环境，调用方的会话里已有一条日常的在跑就退 252）。为了每次都判得到，假 cargo 模式下自证用假会话号经真包装占一条
#   日常的（16M 的 sleep），① 那一趟前后都在跑。
#   ⑭ 双机开关（定义在 research/scripts/layer0-shard-configuration-check.sh）：配置齐、「第二台」连得上、写了 ENABLE_ACROSS_MACHINES=1 ⇒ 判法退 0、报「双机：开，GPU：关」，
#      --emit-assignments 打 ENABLE_ACROSS_MACHINES=1 与没写的 ENABLE_GPU=0；去掉那一行（没写开关）⇒ 判法报「双机：关」退 3（不是「不能用」的报错），
#      --emit-assignments 同样退 3、不打赋值
#   ⑮ 开关写了 0、1 之外的值（ENABLE_ACROSS_MACHINES=yes、ENABLE_GPU=2）⇒ 判不过（退 1），说出是哪个键
# 弄坏开关（配置判法的）MULTI_HOST_CONFIGURATION_BREAK=switch-off-is-on 打开时 ⑭ 判错，switch-value-unchecked 打开时 ⑮ 判错。
# 弄坏开关（驱动脚本的）LAYER0_SHARD_RUN_BREAK=no-peer-findings-copy、no-merge-findings 各自打开时 ⑦ 判错，peer-without-memory-cap 打开时 ⑧ ⑪ ⑫ 与 ⑲ 第二格判错（真 cargo、假 cargo 两种模式都是这四格，写法以驱动文件头那一项为准）、⑳ 也判错，
# no-sop-copy 打开时 ⑨ 判错，no-peer-cleanup-on-failure 打开时 ⑩ ⑬ ⑰ 与 ⑲ 两格判错（⑳ ㉑ 不红：它们走的那一支失败前自己调 ⑦ 删，不经 EXIT 上那一处），peer-build-under-shard-cap 打开时 ⑯ ⑰ 判错，peer-build-cap-unexplained 打开时 ⑰ 判错，
# peer-exit-not-awaited 打开时 ⑬ 判错，restore-after-run 打开时 ⑱ 判错，gpu-memory-unexplained 打开时 ⑲ 两格与 ⑳ 判错，
# liveness-by-inner-pid 打开时 ⑳ 判错，outer-liveness-unchecked 打开时 ㉑ 判错
# （no-peer-findings-copy、no-sop-copy、peer-exit-not-awaited、restore-after-run、gpu-memory-unexplained、liveness-by-inner-pid、outer-liveness-unchecked 七项的失败句首写在驱动文件头）。
# 弄坏开关（本自证的，开关一行，下一行起是打开后判错的格）：
#   LAYER0_SHARD_SELFTEST_BREAK=package-mismatch  登记与不分片那一趟的 -p 写成用例不在的那个包（真 cargo 模式写成 singlefs-harness，即用例搬进
#     singlefs-checker-tier 之后还写死的旧包名；假 cargo 模式写成 singlefs-checker-tier，假 cargo 照真 cargo 报 no test target、退 101）
#       「不分片跑登记的那条用例」、①（驱动编测试二进制那一步失败），以及靠 ① 产物的几格
#   LAYER0_SHARD_SELFTEST_BREAK=real-binaries-unhooked  真 cargo 模式下不换测试可执行文件、不记编译那一趟（改前的取样：只有假 cargo 记）；只在真 cargo 模式判
#     （假 cargo 模式下打开不改任何格）
#       「⑭ 两片的测试进程」（第 0/2 片没记下标准输出；第 1/2 片没记下标准输出）「⑬ 两片跑到一半」（发 TERM 之前两片没都停在那里）
#       「⑯ 第二台编测试二进制」（编译那一趟没记下）
#   LAYER0_SHARD_SELFTEST_BREAK=keeps-session  不清调用方的会话号 CLAUDE_CODE_SESSION_ID（改前的跑法）；只在假 cargo 模式判（真 cargo 带 SINGLEFS_HEAVY_TESTS，包装不算日常）
#       「① 单独跑」（第二台编测试二进制那一条包装退 252），以及同一趟里判的「⑭ 两片的测试进程」「⑧ 第二台那一片经」「⑯ 第二台编测试二进制」，
#       靠 ① 产物的「① 不分片的计数行」「① merge 那一行」「⑦ 发现日志」「③ 「第二台」rustc」「⑪ 本机设了」「⑫ 设了」，共 10 格
#       （自证占位那一条只在 ① 那一趟前后在跑，之后的几趟照常）
# 成功行报核了几格（现算）。
#
# admission: always 自证判的是这一刻的驱动脚本与仓，每次调都要现跑
# run-condition: command cargo rustc python3 rsync git nproc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
selftest_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

repository_root="$(cd "$selftest_script_directory/../.." && pwd)"
work="$(mktemp -d)"
remove_work() {
  echo "  · 自证的临时目录 $work（$(du -sh "$work" 2>/dev/null | cut -f1)）删掉"
  rm -rf -- "${work:?}"
}
trap remove_work EXIT
checked=0
failures=0
not_run=0
expect() { # expect <名> <判据的退出码：0 为过> <不过时的细节>
  checked=$((checked + 1))
  if [[ "$2" == 0 ]]; then
    echo "  ✓ $1"
  else
    echo "  ✗ $1：$3"  # gate-lint:detail
    failures=$((failures + 1))
  fi
}

selftest_case_key="crash-case:sharded-selftest"
copy="$work/repository"
# 两片（0/2、1/2）的测试进程开头 source 的自证记录：⑭ 标准输出指向哪里，⑧ ⑪ ⑫ 第二台那一片的 cgroup，⑬ 记下进程号、停住等驱动来停
cat > "$work/shard-hooks.sh" <<'SHARD_HOOKS'
find_peer_outer() { # 第二台那一片的外层（写退出码文件的那个 bash：祖先里命令行带 .shard-1-of-2.exit 的第一个），找不到打 0
  local outer="$PPID"
  while (( outer > 1 )) && ! grep -qaF '.shard-1-of-2.exit' "/proc/$outer/cmdline" 2>/dev/null; do
    outer="$(sed 's/.*) //' "/proc/$outer/stat" 2>/dev/null | cut -d' ' -f2)"
    [[ "$outer" =~ ^[0-9]+$ ]] || outer=0
  done
  (( outer > 1 )) || outer=0
  echo "$outer"
}
if [[ "$SINGLEFS_LAYER0_SHARD" == 1/2 && -n "${LAYER0_SHARD_SELFTEST_PEER_OUTER_FILE:-}" ]]; then
  # ⑳ ㉑：记下第二台那一片的外层（先写临时文件再改名，自证读不到半个进程号），自证等它退了再看 runs/
  find_peer_outer > "$LAYER0_SHARD_SELFTEST_PEER_OUTER_FILE.partial" && mv -- "$LAYER0_SHARD_SELFTEST_PEER_OUTER_FILE.partial" "$LAYER0_SHARD_SELFTEST_PEER_OUTER_FILE" || exit 101
fi
if [[ "$SINGLEFS_LAYER0_SHARD" == 1/2 && -n "${LAYER0_SHARD_SELFTEST_PEER_EXIT_FILE_TARGET:-}" ]]; then
  # ㉑：第二台那一片的退出码文件换成指向不存在的目录里的符号链接：外层照常收尾，写退出码写不进去就退了（演外层没留退出码就没了）
  ln -sfn -- "$LAYER0_SHARD_SELFTEST_PEER_EXIT_FILE_TARGET" "${SINGLEFS_LAYER0_FINDINGS_FILE%.findings.tsv}.exit" || exit 101
fi
if [[ "$SINGLEFS_LAYER0_SHARD" == 1/2 && -n "${LAYER0_SHARD_SELFTEST_PEER_SHARD_FAIL:-}" ]]; then
  exit 3   # ⑲：第 1/2 片一起就退 3，第二台的内存包装照假 journalctl 认成显存不够
fi
if [[ -n "${LAYER0_SHARD_SELFTEST_STDOUT_KINDS:-}" ]]; then
  # ⑭：记下这一片的标准输出指向哪里（普通文件的路径，还是 pipe:[…]）
  echo "shard=$SINGLEFS_LAYER0_SHARD stdout=$(readlink "/proc/$$/fd/1")" >> "$LAYER0_SHARD_SELFTEST_STDOUT_KINDS"
fi
if [[ "$SINGLEFS_LAYER0_SHARD" == 1/2 && -n "${LAYER0_SHARD_SELFTEST_PEER_CGROUP:-}" ]]; then
  own_cgroup="$(sed -n 's/^0:://p' /proc/self/cgroup)"
  { cat /proc/self/cgroup; printf 'memory.max=%s\nreserve=%s\n' "$(cat "/sys/fs/cgroup$own_cgroup/memory.max" 2>/dev/null)" "${RUN_WITH_MEMORY_CAP_RESERVE-unset}"; } >> "$LAYER0_SHARD_SELFTEST_PEER_CGROUP"
fi
if [[ -n "${LAYER0_SHARD_SELFTEST_HOLD_FILE:-}" ]]; then
  # ⑬：记下自己的进程号，停在这里等驱动收到 TERM 来停；至多停「⑬ 等两片的上限 + 120 秒」，hold 文件被删（自证放走）就退 101。
  #   第 1/2 片先记下第二台那一片的外层（写退出码文件的那个 bash：祖先里命令行带 .shard-1-of-2.exit 的第一个，找不到记 0），自证等它退了再看 runs/
  if [[ "$SINGLEFS_LAYER0_SHARD" == 1/2 ]]; then
    echo "peer_outer=$(find_peer_outer)" >> "$LAYER0_SHARD_SELFTEST_HOLD_FILE"
  fi
  echo "shard=$SINGLEFS_LAYER0_SHARD pid=$$" >> "$LAYER0_SHARD_SELFTEST_HOLD_FILE"
  held_rounds_limit=$(( (${LAYER0_SHARD_SELFTEST_HOLD_SECONDS:-60} + 120) * 10 ))
  for ((held_rounds = 0; held_rounds < held_rounds_limit; held_rounds++)); do
    [[ -e "$LAYER0_SHARD_SELFTEST_HOLD_FILE" ]] || exit 101
    sleep 0.1
  done
  exit 101
fi
SHARD_HOOKS
# 编测试二进制那一趟（cargo test … --no-run）开头 source 的自证记录：⑯ 这一次编译所在的 cgroup 与 memory.max（连同工作目录），⑰ 先占一块内存
cat > "$work/build-hooks.sh" <<'BUILD_HOOKS'
if [[ -n "${LAYER0_SHARD_SELFTEST_PEER_BUILD_FAIL:-}" && "$PWD" == */peer/runs/* ]]; then
  exit 3   # ⑲：第二台那棵树里编测试二进制一起就退 3，第二台的内存包装照假 journalctl 认成显存不够；本机那一趟照常编
fi
if [[ -n "${LAYER0_SHARD_SELFTEST_BUILD_CGROUP:-}" ]]; then
  build_cgroup="$(sed -n 's/^0:://p' /proc/self/cgroup)"
  printf 'cwd=%s cgroup=%s memory.max=%s\n' "$PWD" "$build_cgroup" "$(cat "/sys/fs/cgroup$build_cgroup/memory.max" 2>/dev/null)" >> "$LAYER0_SHARD_SELFTEST_BUILD_CGROUP"
fi
if [[ -n "${LAYER0_SHARD_SELFTEST_BUILD_ALLOCATE_MIB:-}" ]]; then
  python3 -c 'import sys, time; block = b"\x01" * (int(sys.argv[1]) * 1048576); time.sleep(1)' "$LAYER0_SHARD_SELFTEST_BUILD_ALLOCATE_MIB" || exit 101
fi
BUILD_HOOKS
rsync -a --exclude target --exclude .git --exclude multi-host.env "$repository_root/" "$copy/"
git -C "$copy" init -q
real_rustc="$(command -v rustc)"
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  cargo_mode="真 cargo（SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS}）"
  # 用例所在的包：973e1f58 把这份测试文件从 singlefs-harness 搬进了 singlefs-checker-tier
  selftest_package="singlefs-checker-tier"
  mismatched_package="singlefs-harness"
  selftest_function="the_first_stream_quick_tier_sharded_by_the_environment_keeps_its_pinned_counts"
  selftest_target="crash_enumeration_sharded_across_processes"
  selftest_cargo="$(command -v cargo)"
  if [[ "${LAYER0_SHARD_SELFTEST_BREAK:-}" == real-binaries-unhooked ]]; then
    echo "  ! 弄坏开关 LAYER0_SHARD_SELFTEST_BREAK=real-binaries-unhooked：真 cargo 模式不换测试可执行文件、不记编译那一趟"
  else
    # 真测试二进制里没有自证的记录：cargo test … --no-run --message-format=json 交出的测试可执行文件换成替身（先 source 两片的记录，再 exec 真的）
    mkdir -p "$work/real-cargo-bin" "$work/test-binary-shims"
    cat > "$work/test-binary-shims.py" <<'TEST_BINARY_SHIMS'
import hashlib, json, os, shlex, sys
shim_directory, hooks = sys.argv[1], sys.argv[2]
for line in sys.stdin:
    try:
        message = json.loads(line)
    except ValueError:
        sys.stdout.write(line)
        continue
    target = message.get("target") or {}
    executable = message.get("executable")
    if message.get("reason") == "compiler-artifact" and executable and "test" in target.get("kind", []):
        shim = os.path.join(shim_directory, "shim-" + hashlib.sha256(executable.encode()).hexdigest()[:16])
        with open(shim + ".tmp", "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n# 真 cargo 模式的测试二进制替身：两片先 source 自证记录，再 exec 真的测试二进制\n")
            handle.write('case "${SINGLEFS_LAYER0_SHARD-}" in 0/2|1/2) source %s ;; esac\n' % shlex.quote(hooks))
            handle.write('exec %s "$@"\n' % shlex.quote(executable))
        os.chmod(shim + ".tmp", 0o755)
        os.replace(shim + ".tmp", shim)
        message["executable"] = shim
        line = json.dumps(message) + "\n"
    sys.stdout.write(line)
TEST_BINARY_SHIMS
    {
      printf '#!/usr/bin/env bash\n# 真 cargo 模式 PATH 最前面那一层：别的照转给真的；编测试二进制那一趟先 source 编译的记录，再换交出的测试可执行文件\n'
      printf 'if [[ " $* " == *" --no-run "* && " $* " == *" --message-format=json "* ]]; then\n'
      printf '  source %q\n' "$work/build-hooks.sh"
      printf '  messages="$(%q "$@")"; status=$?\n' "$selftest_cargo"
      printf '  printf "%%s\\n" "$messages" | python3 %q %q %q || exit 101\n' "$work/test-binary-shims.py" "$work/test-binary-shims" "$work/shard-hooks.sh"
      printf '  exit "$status"\nfi\nexec %q "$@"\n' "$selftest_cargo"
    } > "$work/real-cargo-bin/cargo"
    chmod +x "$work/real-cargo-bin/cargo"
    selftest_cargo="$work/real-cargo-bin/cargo"
    export PATH="$work/real-cargo-bin:$PATH"
  fi
else
  cargo_mode="假 cargo（真 cargo 那一趟本次未跑：带 SINGLEFS_HEAVY_TESTS=commit 或 user-request 才跑）"
  # 替身用例写进的包；假 cargo 从它旁边的 stand-in-package 读这个名字，-p 点的不是它就照真 cargo 报错
  selftest_package="singlefs-harness"
  mismatched_package="singlefs-checker-tier"
  selftest_function="stand_in_for_the_driver_selftest"
  selftest_target="sharded_driver_selftest_stand_in"
  printf '#[test]\n#[ignore]\nfn %s() {}\n' "$selftest_function" > "$copy/crates/$selftest_package/tests/$selftest_target.rs"
  mkdir -p "$work/stand-in-bin"
  printf '%s\n' "$selftest_package" > "$work/stand-in-bin/stand-in-package"
  selftest_cargo="$work/stand-in-bin/cargo"
  cat > "$selftest_cargo" <<'STAND_IN_CARGO'
#!/usr/bin/env bash
# 驱动脚本自证的假 cargo：-V 打版本；test 按 SINGLEFS_LAYER0_SHARD 分三种：不设打计数行与线程行，<i>/2 往进度目录写那一片的账本，
# merge/2 核两份账本在、记的输入指纹与这一趟的相同、START_OVER 没设，再打计数行与带 shards= 的线程行。别的一律退 101。
# 驱动先 cargo test --no-run --message-format=json 取测试可执行文件、再直接执行它：可执行文件是指回这份脚本的符号链接 stand-in-test-binary，
# 按被叫的名字认出是在当测试二进制跑，照 test 那一支走（参数是 libtest 的那几个）。
if [[ "$(basename "$0")" == stand-in-test-binary ]]; then set -- test "$@"; fi
if [[ "${1:-}" == -V ]]; then echo "cargo 0.0.0-layer0-shard-selftest"; exit 0; fi
[[ "${1:-}" == test ]] || { echo "假 cargo 只认 -V 与 test：$*"; exit 101; }
# 照真 cargo：-p 点的包不是替身用例所在的那个（旁边 stand-in-package 里写的），就报找不到测试目标、退 101
requested_package="" requested_target="" previous_argument=""
for argument in "$@"; do
  [[ "$previous_argument" == -p ]] && requested_package="$argument"
  [[ "$previous_argument" == --test ]] && requested_target="$argument"
  previous_argument="$argument"
done
if [[ -n "$requested_package" && "$requested_package" != "$(cat "$(dirname "$0")/stand-in-package")" ]]; then
  echo "error: no test target named \`$requested_target\` in \`$requested_package\` package" >&2
  exit 101
fi
if [[ " $* " == *" --no-run "* ]]; then
  source "$(dirname "$0")/../build-hooks.sh"
  test_target="" previous_argument=""
  for argument in "$@"; do [[ "$previous_argument" == --test ]] && test_target="$argument"; previous_argument="$argument"; done
  printf '{"reason":"compiler-artifact","target":{"name":"%s","kind":["test"]},"manifest_path":"%s/Cargo.toml","executable":"%s"}\n' \
    "$test_target" "$PWD" "$(dirname "$0")/stand-in-test-binary"
  exit 0
fi
count_line="LAYER0_SHARDED states=29 closed_form=29 violations=0 exhaustive=true"
passed="test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s"
write_findings_section() { # write_findings_section <begin 行多出来的字段>：设了 SINGLEFS_LAYER0_FINDINGS_FILE 就追加一节（0 个签名），标准输出打 LAYER0_FINDINGS
  if [[ -n "${SINGLEFS_LAYER0_FINDINGS_FILE:-}" ]]; then
    printf 'layer0_findings_begin\tformat=1\tstream=stand-in\tstates=29%s\nlayer0_findings_summary\tsignatures=0\tred_states=0\tstates=29\tstates_by_finding=none\n' "$1" \
      >> "$SINGLEFS_LAYER0_FINDINGS_FILE" || exit 101
  fi
  echo "LAYER0_FINDINGS signatures=0 red_states=0 states=29"
}
case "${SINGLEFS_LAYER0_SHARD-}" in
  "")
    write_findings_section ""
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=0 freshly_run_slices=4 progress_file_after_completion=none elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  0/2|1/2)
    [[ -n "${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}" && -n "${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "分片跑没设进度目录或输入指纹"; exit 101; }
    source "$(dirname "$0")/../shard-hooks.sh"
    shard_index="${SINGLEFS_LAYER0_SHARD%/2}"
    ledger="$SINGLEFS_LAYER0_PROGRESS_DIRECTORY/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
    mkdir -p "$SINGLEFS_LAYER0_PROGRESS_DIRECTORY" && printf 'input_fingerprint=%s\n' "$SINGLEFS_LAYER0_INPUT_FINGERPRINT" > "$ledger" || exit 101
    write_findings_section "$(printf '\tshard=%s\tshard_states=15' "$SINGLEFS_LAYER0_SHARD")"
    printf 'LAYER0_PROGRESS slice=1/2 shard=%s\nLAYER0_SHARD mode=run shard=%s ledger=%s\n%s\n' "$SINGLEFS_LAYER0_SHARD" "$SINGLEFS_LAYER0_SHARD" "$ledger" "$passed" ;;
  merge/2)
    [[ -z "${SINGLEFS_LAYER0_START_OVER+set}" ]] || { echo "merge 那一趟带了 SINGLEFS_LAYER0_START_OVER"; exit 101; }
    for shard_index in 0 1; do
      ledger="${SINGLEFS_LAYER0_PROGRESS_DIRECTORY:-}/layer0-shard-stand-in-shard-${shard_index}-of-2.tally"
      [[ "$(cat "$ledger" 2>/dev/null)" == "input_fingerprint=${SINGLEFS_LAYER0_INPUT_FINGERPRINT:-}" ]] || { echo "merge：第 $shard_index 片的账本缺或输入指纹不同（$ledger）"; exit 101; }
    done
    echo "LAYER0_SHARD mode=merge shards=2 ledgers=${SINGLEFS_LAYER0_PROGRESS_DIRECTORY}"
    write_findings_section ""
    echo "LAYER0_PARALLEL_FINISHED states=29 slices=4 worker_threads=4 configured_worker_threads=4 worker_threads_source=shard_ledgers resumed_slices=0 freshly_run_slices=4 shards=2 shard_worker_threads=2,2 shard_configured_worker_threads=2,2 shard_worker_threads_sources=environment_variable,environment_variable shard_available_parallelism=2,2 shard_resumed_slices=0,0 shard_freshly_run_slices=2,2 shard_elapsed_milliseconds=10,10 progress_file_after_completion=kept elapsed_seconds=0.1"
    printf '%s\n%s\n' "$count_line" "$passed" ;;
  *) echo "假 cargo 认不出 SINGLEFS_LAYER0_SHARD=${SINGLEFS_LAYER0_SHARD}"; exit 101 ;;
esac
STAND_IN_CARGO
  chmod +x "$selftest_cargo"
  ln -s cargo "$work/stand-in-bin/stand-in-test-binary"
  export PATH="$work/stand-in-bin:$PATH"
fi
if [[ "${LAYER0_SHARD_SELFTEST_BREAK:-}" == package-mismatch ]]; then
  echo "  ! 弄坏开关 LAYER0_SHARD_SELFTEST_BREAK=package-mismatch：登记与不分片那一趟的 -p 写成用例不在的 $mismatched_package"
  selftest_package="$mismatched_package"
fi
# 第二台那一片的内存上限：配置的 PEER_MEMORY_CAP 与 ⑫ 设的覆盖值。假 cargo 不编译，1G / 2G；真 cargo 第二台那一片先在这个上限里
# cargo test --release --no-run 编测试二进制，1G 编不下（rustc 被 cgroup OOM 杀），取 8G / 12G（依据：峰值表里 singlefs-checker-tier
# release --no-run 量到的最大一条 4.29 GiB）。覆盖值要与配置值不同，⑫ 才分得出「照覆盖」与「照配置」
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  peer_memory_cap=8G peer_memory_cap_override=12G
else
  peer_memory_cap=1G peer_memory_cap_override=2G
fi
# 第二台编测试二进制的上限（⑯）与撞顶那一趟的上限（⑰）：假 cargo 3G、64M（编译那一趟先占 256 MiB）；真 cargo 16G、128M（真编，从头编撞得上）
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  peer_build_memory_cap=16G peer_build_memory_cap_too_small=128M build_allocate_mib=""
else
  peer_build_memory_cap=3G peer_build_memory_cap_too_small=64M build_allocate_mib=256
fi
gibibytes_in_bytes() { echo $(( ${1%G} * 1073741824 )); } # gibibytes_in_bytes <nG>：scope 的 memory.max 读出来的字节数
printf '%s\t%s\t%s\t%s\n' "$selftest_case_key" "crates/ Cargo.toml Cargo.lock" \
  "test=$selftest_package:$selftest_target:$selftest_function count-line=LAYER0_SHARDED exhaustive=LAYER0_SHARDED threads=LAYER0_SHARDED shard=across-machines" \
  "# 驱动脚本自证临时登记的用例" >> "$copy/.claude/gate.d/stage-inputs.tsv"
selftest_bin_directory="$(dirname "$selftest_cargo")"
write_configuration() { # write_configuration <文件> <PEER_CARGO_BIN_DIRECTORY>
  cat > "$1" <<CONFIGURATION
# 驱动脚本自证用的配置：第二台是本机上的 $work/peer
PEER_SSH_HOST=selftest-peer-is-this-machine
PEER_REPOSITORY_DIRECTORY=$work/peer
PEER_CARGO_BIN_DIRECTORY=$2
PEER_MEMORY_CAP=$peer_memory_cap
ENABLE_ACROSS_MACHINES=1
QUIESCE_STOP_COMMAND=touch $work/quiesced
QUIESCE_STOPPED_CHECK_COMMAND=test -e $work/quiesced
QUIESCE_START_COMMAND=echo restore >> $work/restore-calls
QUIESCE_STARTED_CHECK_COMMAND=test -e $work/restore-calls
CONFIGURATION
}
write_configuration "$work/multi-host.env" "$selftest_bin_directory"
export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env"
export SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
# 三趟 cargo test 都不带 --include-ignored：真 cargo 那条小流用例不标 ignore（同一个测试目标里标了 ignore 的 golden 子进程用例不进过滤范围）；
# 假 cargo 的替身用例标了 ignore，不带它就不算重型
export SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0
unset SINGLEFS_LAYER0_SHARD SINGLEFS_LAYER0_PROGRESS_DIRECTORY SINGLEFS_LAYER0_INPUT_FINGERPRINT SINGLEFS_LAYER0_START_OVER
# ⑪ ⑫ 之外的各格都是「没设」：余量与第二台上限的覆盖开关不从调用方漏进来；编译的上限由自证设（⑯）
unset RUN_WITH_MEMORY_CAP_RESERVE SINGLEFS_LAYER0_PEER_MEMORY_CAP LAYER0_SHARD_SELFTEST_BUILD_CGROUP LAYER0_SHARD_SELFTEST_BUILD_ALLOCATE_MIB SINGLEFS_LAYER0_SHARD_PEER_EXIT_HOLD_FILE
export SINGLEFS_LAYER0_PEER_BUILD_MEMORY_CAP="$peer_build_memory_cap"
# 自证起的命令不算进调用方会话的「日常一条」（文件头）；假 cargo 模式下自证用假会话号占一条日常的，① 那一趟带着同一个会话号起
fake_session="layer0-shard-run-selftest-$$"
if [[ "${LAYER0_SHARD_SELFTEST_BREAK:-}" == keeps-session ]]; then
  echo "  ! 弄坏开关 LAYER0_SHARD_SELFTEST_BREAK=keeps-session：不清调用方的会话号"
  session_settings=(CLAUDE_CODE_SESSION_ID="$fake_session")
else
  session_settings=(-u CLAUDE_CODE_SESSION_ID)
fi
unset CLAUDE_CODE_SESSION_ID
driver="$copy/research/scripts/layer0-shard-run.sh"
admission_module="$copy/research/scripts/admission.py"

# 不分片跑一次那条用例：计数行是 merge 之后要逐字相同的那一行
(cd "$copy" && cargo test --release -p "$selftest_package" --test "$selftest_target" -- --exact "$selftest_function" --nocapture) > "$work/unsharded.log" 2>&1
unsharded_exit=$?
unsharded_count_line="$(grep '^LAYER0_SHARDED ' "$work/unsharded.log")"
expect "不分片跑登记的那条用例：退 0，恰好一行 LAYER0_SHARDED 计数行" "$([[ $unsharded_exit == 0 && $(grep -c '^LAYER0_SHARDED ' "$work/unsharded.log") == 1 ]]; echo $?)" \
  "退 $unsharded_exit；日志尾部：$(tail -5 "$work/unsharded.log" | tr '\n' '|')"

# ① 单独跑：两片、merge、判、写标记（假 cargo 在第二台那一片记下自己的 cgroup，⑧ 核它在内存包装的 scope 里）
export LAYER0_SHARD_SELFTEST_PEER_CGROUP="$work/peer-cgroup"
holder_pid="" holder_sleep=""
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  # 假会话号里先占一条日常的（明写等待上限：slice 一时放不下就排队），① 那一趟前后都在跑
  env -u SINGLEFS_HEAVY_TESTS CLAUDE_CODE_SESSION_ID="$fake_session" RUN_WITH_MEMORY_CAP_WAIT_SECONDS=600 RUN_WITH_MEMORY_CAP_PEAKS="$work/holder-peaks.tsv" \
    RUN_WITH_MEMORY_CAP_KEY="layer0-shard-run.sh 自证 ①：同一个会话里先占着的一条" \
    bash "$copy/research/scripts/run-with-memory-cap.sh" 16M bash -c 'echo "$$" > "$0"; exec sleep 600' "$work/holder.pid" > "$work/holder.out" 2>&1 &
  holder_pid=$!
  for ((holder_rounds = 0; holder_rounds < 6100; holder_rounds++)); do
    [[ -s "$work/holder.pid" ]] && break
    kill -0 "$holder_pid" 2>/dev/null || break
    sleep 0.1
  done
  holder_sleep="$(cat "$work/holder.pid" 2>/dev/null)"
fi
LAYER0_SHARD_SELFTEST_BUILD_CGROUP="$work/build-cgroup" LAYER0_SHARD_SELFTEST_STDOUT_KINDS="$work/stdout-kinds" \
  env "${session_settings[@]}" bash "$driver" "$selftest_case_key" "$copy" > "$work/standalone.log" 2>&1
standalone_exit=$?
# 占位那一条只停它自己记下的 sleep（核过进程名），包装随之收尾
if [[ -n "$holder_sleep" && "$(cat "/proc/$holder_sleep/comm" 2>/dev/null)" == sleep ]]; then kill "$holder_sleep"; fi
if [[ -n "$holder_pid" ]]; then wait "$holder_pid"; fi
# ⑭ 两片的标准输出都是文件、不是管道：读输出的一方（显示用的 tail、第二台的 ssh）断了，测试进程照样写文件，
#    不会因写断掉的管道 panic、连带删掉进度文件（2026-09-28 预演第二台那一片续跑读回 0 片的成因）
stdout_kind_problems=()
for shard_name in 0/2 1/2; do
  stdout_target="$(grep "^shard=$shard_name " "$work/stdout-kinds" 2>/dev/null | head -1 | sed 's/.* stdout=//')"
  if [[ -z "$stdout_target" ]]; then stdout_kind_problems+=("第 $shard_name 片没记下标准输出")
  elif [[ "$stdout_target" != /* ]]; then stdout_kind_problems+=("第 $shard_name 片的标准输出是 $stdout_target，不是文件（管道记成 pipe:[…]；第二台那一份文件跑完随 ⑦ 删了，只看记下的是不是路径）")
  fi
done
expect "⑭ 两片的测试进程标准输出都直接写文件、不经管道" "$( (( ${#stdout_kind_problems[@]} == 0 )); echo $?)" \
  "$(IFS='；'; echo "${stdout_kind_problems[*]}")；记下的：$(tr '\n' '|' < "$work/stdout-kinds" 2>/dev/null)"
fingerprint="$(python3 "$admission_module" crash-case-manifest "$copy" "$selftest_case_key" "$work/manifest" \
  --judging-digest --toolchain --build-environment | cut -d' ' -f1)"
marker_output="$(python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest")"
marker_exit=$?
marker_count_line="$(grep '^LAYER0_SHARDED ' <<< "$marker_output")"
marker_path="$(head -1 <<< "$marker_output" | cut -d' ' -f2)"
expect "① 单独跑：退 0，两片各跑了（LAYER0_SHARD mode=run 0/2、1/2），这批输入那一格全绿标记作数" \
  "$([[ $standalone_exit == 0 && $marker_exit == 0 ]] && { [[ -n "${SINGLEFS_HEAVY_TESTS:-}" || -n "$holder_sleep" ]]; } && grep -q 'SINGLEFS_LAYER0_SHARD=0/2' "$work/standalone.log" && grep -q '④ 两片都跑完' "$work/standalone.log"; echo $?)" \
  "驱动脚本退 $standalone_exit，标记核 $marker_exit（$marker_output）；$([[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]] && { [[ -n "$holder_sleep" ]] && echo "占位那一条在跑（sleep $holder_sleep）；" || echo "占位那一条没起来：$(tail -n 2 "$work/holder.out" 2>/dev/null | tr '\n' '|')；"; })驱动脚本输出尾部：$(tail -15 "$work/standalone.log" | tr '\n' '|')"
expect "① 不分片的计数行与双机分片 merge 之后记进标记的那一行逐字相同" "$([[ -n "$unsharded_count_line" && "$marker_count_line" == "$unsharded_count_line" ]]; echo $?)" \
  "不分片「$unsharded_count_line」，标记里「$marker_count_line」"
expect "① merge 那一行按片报线程（shards=2、shard_worker_threads=），记进了标记" "$(grep -q '^parallel_finished=LAYER0_PARALLEL_FINISHED .*shards=2 shard_worker_threads=' "$marker_path"; echo $?)" \
  "标记 $marker_path 里：$(grep 'parallel_finished=' "$marker_path" 2>/dev/null)"
expect "① 清了场（回读过），第二台这一趟的目录删了" \
  "$([[ -e "$work/quiesced" ]] && grep -q '清场之后回读过了' "$work/standalone.log" \
     && [[ -z "$(ls -A "$work/peer/runs" 2>/dev/null)" ]]; echo $?)" \
  "清场标记在不在：$([[ -e "$work/quiesced" ]] && echo 在 || echo 不在)；第二台 runs/ 下：$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')"

# ⑧ 第二台那一片经内存包装起
peer_cap_problems=()
grep -qF "第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 bash research/scripts/run-with-memory-cap.sh $peer_memory_cap 起" "$work/standalone.log" \
  || peer_cap_problems+=("驱动的输出里 ④ 那一行没写第二台那一片经 run-with-memory-cap.sh $peer_memory_cap 起")
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]] && ! grep -q 'singlefs-memory-cap-' "$work/peer-cgroup" 2>/dev/null; then
  peer_cap_problems+=("假 cargo 在第二台那一片记下的 cgroup 不在 singlefs-memory-cap- 的 scope 里：$(tr '\n' ' ' < "$work/peer-cgroup" 2>/dev/null)")
fi
expect "⑧ 第二台那一片经 research/scripts/run-with-memory-cap.sh 起（上限是配置的 PEER_MEMORY_CAP=$peer_memory_cap）" "${#peer_cap_problems[@]}" \
  "$(IFS='；'; echo "${peer_cap_problems[*]}")"
unset LAYER0_SHARD_SELFTEST_PEER_CGROUP

# ⑯ 第二台编测试二进制在编译的上限里（① 那一趟的编译记录：工作目录在第二台 runs/ 底下的那一行）
build_cap_problems=()
peer_build_record="$(grep "^cwd=$work/peer/runs/" "$work/build-cgroup" 2>/dev/null | head -1)"
if [[ -z "$peer_build_record" ]]; then
  build_cap_problems+=("第二台编译那一趟没记下（记下的：$(tr '\n' '|' < "$work/build-cgroup" 2>/dev/null)）")
else
  [[ "$peer_build_record" == *" cgroup="*singlefs-memory-cap-* ]] || build_cap_problems+=("第二台编译那一趟不在 singlefs-memory-cap- 的 scope 里：$peer_build_record")
  [[ "$peer_build_record" == *" memory.max=$(gibibytes_in_bytes "$peer_build_memory_cap")" ]] \
    || build_cap_problems+=("第二台编译那一趟的 memory.max 不是编译的上限 $peer_build_memory_cap（$(gibibytes_in_bytes "$peer_build_memory_cap") 字节）：$peer_build_record")
fi
grep -qF "第二台的编译经 bash research/scripts/run-with-memory-cap.sh $peer_build_memory_cap）" "$work/standalone.log" \
  || build_cap_problems+=("驱动 ④ 那一行没写第二台的编译经 run-with-memory-cap.sh $peer_build_memory_cap")
expect "⑯ 第二台编测试二进制经 run-with-memory-cap.sh、在编译的上限 SINGLEFS_LAYER0_PEER_BUILD_MEMORY_CAP=$peer_build_memory_cap 里，不在那一片的 PEER_MEMORY_CAP=$peer_memory_cap 里" \
  "${#build_cap_problems[@]}" "$(IFS='；'; echo "${build_cap_problems[*]}")"

# ② --merged-log：门禁 checker-tier-layer0-replay 调的那一条
bash "$driver" --merged-log "$selftest_case_key" "$copy" "$fingerprint" "$work/merged.log" > "$work/merged-driver.log" 2>&1
merged_exit=$?
expect "② --merged-log：退 0，merge 那一趟的日志写进给的文件，带 LAYER0_SHARD mode=merge shards=2 与同一行计数" \
  "$([[ $merged_exit == 0 ]] && grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$work/merged.log" && [[ "$(grep '^LAYER0_SHARDED ' "$work/merged.log")" == "$unsharded_count_line" ]]; echo $?)" \
  "退 $merged_exit；驱动脚本输出尾部：$(tail -10 "$work/merged-driver.log" | tr '\n' '|')"

# ⑦ 发现日志：merge 那一份是 <日志文件>.findings.tsv（checker-tier-layer0-replay 读的那一份），两片的在各自日志旁边，第二台上那一份拷回之后删了，三份路径都打进输出
findings_problems=()
merge_findings="$work/merged.log.findings.tsv"
local_findings="$work/merged.log.shard-0-of-2.log.findings.tsv"
fetched_findings="$work/merged.log.shard-1-of-2.log.findings.tsv"
[[ "$(grep -c '^layer0_findings_begin' "$merge_findings" 2>/dev/null)" == 1 && "$(grep -c '^layer0_findings_summary' "$merge_findings" 2>/dev/null)" == 1 ]] \
  || findings_problems+=("merge 那一份 $merge_findings 不是恰好一节带 summary")
if grep -q $'^layer0_findings_begin\t.*shard=' "$merge_findings" 2>/dev/null; then findings_problems+=("merge 那一份的 begin 行带了 shard="); fi
grep -q $'^layer0_findings_begin\t.*\tshard=0/2\t' "$local_findings" 2>/dev/null || findings_problems+=("本机那一片的 $local_findings 没有 shard=0/2 那一节")
grep -q $'^layer0_findings_begin\t.*\tshard=1/2\t' "$fetched_findings" 2>/dev/null || findings_problems+=("第二台那一片没拷回到 $fetched_findings（或没有 shard=1/2 那一节）")
leftover_peer_findings="$(find "$work/peer/runs" -maxdepth 1 -name '*.findings.tsv' 2>/dev/null)"
[[ -z "$leftover_peer_findings" ]] || findings_problems+=("第二台上的发现日志没删：$leftover_peer_findings")
for findings_path in "$local_findings" "$fetched_findings" "$merge_findings"; do
  grep -qF -- "$findings_path" "$work/merged-driver.log" || findings_problems+=("驱动的输出里没打 $findings_path")
done
standalone_findings_count="$(find "$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)/singlefs-layer0-logs" -name '*.findings.tsv' -size +0 2>/dev/null | grep -c .)"
[[ "$standalone_findings_count" == 3 ]] || findings_problems+=("单独跑那一趟 common-dir 下 singlefs-layer0-logs/ 里非空的发现日志是 ${standalone_findings_count} 份，不是 3 份")
expect "⑦ 发现日志：merge 那一份在 <日志文件>.findings.tsv（一节、不带 shard=、有 summary），两片的在各自日志旁边（shard=0/2、1/2），第二台上那一份拷回后删了，三份路径都打进输出；单独跑时三份在 common-dir 下" \
  "${#findings_problems[@]}" "$(IFS='；'; echo "${findings_problems[*]}")"

# ③ 「第二台」的 rustc -Vv 第一行不同
mkdir -p "$work/other-toolchain"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$selftest_cargo" > "$work/other-toolchain/cargo"
printf '#!/usr/bin/env bash\nif [[ "${1:-}" == -Vv ]]; then %q -Vv | sed "1s/\$/ (another build)/"; else exec %q "$@"; fi\n' "$real_rustc" "$real_rustc" > "$work/other-toolchain/rustc"
chmod +x "$work/other-toolchain/cargo" "$work/other-toolchain/rustc"
write_configuration "$work/other-toolchain.env" "$work/other-toolchain"
SINGLEFS_MULTI_HOST_CONFIG="$work/other-toolchain.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/other-toolchain.log" 2>&1
toolchain_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
toolchain_marker_exit=$?
expect "③ 「第二台」rustc -Vv 第一行不同 ⇒ 拒（退 1，说工具链不同），这批输入那一格标记不动" \
  "$([[ $toolchain_exit == 1 && $toolchain_marker_exit == 0 ]] && grep -q '两台的工具链不同' "$work/other-toolchain.log"; echo $?)" \
  "退 $toolchain_exit，标记核 $toolchain_marker_exit；输出：$(tail -4 "$work/other-toolchain.log" | tr '\n' '|')"

# ④ 「第二台」那一片退 0 却没写账本
mkdir -p "$work/no-ledger"
# 驱动先 cargo test --no-run 取可执行文件、再直接执行它：--no-run 交出这一格自己的「不写账本」测试二进制
cat > "$work/no-ledger/cargo" <<NO_LEDGER_CARGO
#!/usr/bin/env bash
if [[ "\${1:-}" == test && " \$* " == *" --no-run "* ]]; then
  printf '{"reason":"compiler-artifact","target":{"name":"%s","kind":["test"]},"manifest_path":"%s/Cargo.toml","executable":"%s"}\n' $(printf '%q' "$selftest_target") "\$PWD" $(printf '%q' "$work/no-ledger/stand-in-test-binary")
  exit 0
fi
if [[ "\${1:-}" == test ]]; then echo "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"; exit 0; fi
exec $(printf '%q' "$selftest_cargo") "\$@"
NO_LEDGER_CARGO
printf '#!/usr/bin/env bash\necho "LAYER0_SHARD mode=run shard=1/2 （自证：这一片不写账本）"\nexit 0\n' > "$work/no-ledger/stand-in-test-binary"
printf '#!/usr/bin/env bash\nexec %q "$@"\n' "$real_rustc" > "$work/no-ledger/rustc"
chmod +x "$work/no-ledger/cargo" "$work/no-ledger/rustc" "$work/no-ledger/stand-in-test-binary"
write_configuration "$work/no-ledger.env" "$work/no-ledger"
git_common_directory="$(git -C "$copy" rev-parse --path-format=absolute --git-common-dir)"
rm -f -- "${git_common_directory:?}/singlefs-layer0-progress/$fingerprint"/layer0-shard-*.tally "${work:?}/peer/progress/$fingerprint"/layer0-shard-*.tally
SINGLEFS_MULTI_HOST_CONFIG="$work/no-ledger.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-ledger.log" 2>&1
no_ledger_exit=$?
python3 "$admission_module" crash-case-marker-check "$copy" "$selftest_case_key" "$fingerprint" "$work/manifest" > /dev/null
no_ledger_marker_exit=$?
expect "④ 「第二台」那一片没写账本 ⇒ 判红（退 1，说两片的账本不是各恰好一份），删这批输入那一格标记" \
  "$([[ $no_ledger_exit == 1 && $no_ledger_marker_exit == 1 ]] && grep -q '两片的账本不是各恰好一份' "$work/no-ledger.log"; echo $?)" \
  "退 $no_ledger_exit，标记核 $no_ledger_marker_exit；输出：$(tail -4 "$work/no-ledger.log" | tr '\n' '|')"

# ⑤ 配置缺键、配置文件不在 ⇒ 开跑之后头一步的配置判法拒
grep -v '^PEER_CARGO_BIN_DIRECTORY=' "$work/multi-host.env" > "$work/missing-key.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/missing-key.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-key.log" 2>&1
missing_key_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/no-such-configuration.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/no-configuration.log" 2>&1
no_configuration_exit=$?
expect "⑤ 配置缺键 PEER_CARGO_BIN_DIRECTORY、配置文件不在 ⇒ 配置判法各拒（退 1，说双机分片不能用），出路指到 multi-host.env.example" \
  "$([[ $missing_key_exit == 1 && $no_configuration_exit == 1 ]] && grep -q '双机分片不能用' "$work/missing-key.log" && grep -q '双机分片不能用' "$work/no-configuration.log" \
     && grep -q 'multi-host.env.example' "$work/missing-key.log" && grep -q 'multi-host.env.example' "$work/no-configuration.log"; echo $?)" \
  "缺键退 $missing_key_exit（$(tail -3 "$work/missing-key.log" | tr '\n' '|')），不在退 $no_configuration_exit（$(tail -3 "$work/no-configuration.log" | tr '\n' '|')）"

# ⑧ 配置缺 PEER_MEMORY_CAP、写成不带单位的数 ⇒ 配置判法拒
grep -v '^PEER_MEMORY_CAP=' "$work/multi-host.env" > "$work/missing-memory-cap.env"
sed 's/^PEER_MEMORY_CAP=.*/PEER_MEMORY_CAP=16/' "$work/multi-host.env" > "$work/unitless-memory-cap.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/missing-memory-cap.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/missing-memory-cap.log" 2>&1
missing_memory_cap_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/unitless-memory-cap.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/unitless-memory-cap.log" 2>&1
unitless_memory_cap_exit=$?
expect "⑧ 配置缺 PEER_MEMORY_CAP、写成不带单位的 16 ⇒ 配置判法各拒（退 1），说出是哪个键" \
  "$([[ $missing_memory_cap_exit == 1 && $unitless_memory_cap_exit == 1 ]] && grep -q '缺键 PEER_MEMORY_CAP' "$work/missing-memory-cap.log" \
     && grep -q 'PEER_MEMORY_CAP 要写第二台那一片的内存上限' "$work/unitless-memory-cap.log"; echo $?)" \
  "缺键退 $missing_memory_cap_exit（$(tail -3 "$work/missing-memory-cap.log" | tr '\n' '|')），不带单位退 $unitless_memory_cap_exit（$(tail -3 "$work/unitless-memory-cap.log" | tr '\n' '|')）"

# ⑭ 双机开关：写了 1 ⇒ 开；没写 ⇒ 双机：关（退 3，不是判不过）
configuration_check="$copy/research/scripts/layer0-shard-configuration-check.sh"
SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" bash "$configuration_check" "$copy" > "$work/switch-on.log" 2>&1
switch_on_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" bash "$configuration_check" --emit-assignments "$copy" > "$work/switch-on-emit.log" 2>&1
switch_on_emit_exit=$?
grep -v '^ENABLE_ACROSS_MACHINES=' "$work/multi-host.env" > "$work/switch-absent.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/switch-absent.env" bash "$configuration_check" "$copy" > "$work/switch-absent.log" 2>&1
switch_absent_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/switch-absent.env" bash "$configuration_check" --emit-assignments "$copy" > "$work/switch-absent-emit.log" 2>&1
switch_absent_emit_exit=$?
expect "⑭ 写了 ENABLE_ACROSS_MACHINES=1 ⇒ 退 0、双机：开，GPU：关，赋值里 ENABLE_GPU=0；没写开关 ⇒ 判法报「双机：关」退 3（不是判不过），--emit-assignments 同样退 3、不打赋值" \
  "$([[ $switch_on_exit == 0 && $switch_on_emit_exit == 0 && $switch_absent_exit == 3 && $switch_absent_emit_exit == 3 ]] \
     && grep -q '^双机：开，GPU：关；' "$work/switch-on.log" && grep -qx 'ENABLE_ACROSS_MACHINES=1' "$work/switch-on-emit.log" && grep -qx 'ENABLE_GPU=0' "$work/switch-on-emit.log" \
     && grep -q '^双机：关' "$work/switch-absent.log" && grep -q 'ENABLE_ACROSS_MACHINES=1' "$work/switch-absent.log" && ! grep -q '不能用' "$work/switch-absent.log" \
     && ! grep -q '^PEER_SSH_HOST=' "$work/switch-absent-emit.log"; echo $?)" \
  "写了 1：退 $switch_on_exit / emit 退 $switch_on_emit_exit（$(head -1 "$work/switch-on.log")）；没写：退 $switch_absent_exit / emit 退 $switch_absent_emit_exit（$(tr '\n' '|' < "$work/switch-absent.log")）"

# ⑮ 开关写了 0、1 之外的值 ⇒ 判不过
sed 's/^ENABLE_ACROSS_MACHINES=.*/ENABLE_ACROSS_MACHINES=yes/' "$work/multi-host.env" > "$work/switch-other.env"
{ cat "$work/multi-host.env"; echo ENABLE_GPU=2; } > "$work/gpu-other.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/switch-other.env" bash "$configuration_check" "$copy" > "$work/switch-other.log" 2>&1
switch_other_exit=$?
SINGLEFS_MULTI_HOST_CONFIG="$work/gpu-other.env" bash "$configuration_check" "$copy" > "$work/gpu-other.log" 2>&1
gpu_other_exit=$?
expect "⑮ 开关写了 0、1 之外的值（ENABLE_ACROSS_MACHINES=yes、ENABLE_GPU=2）⇒ 判不过（退 1），说出是哪个键" \
  "$([[ $switch_other_exit == 1 && $gpu_other_exit == 1 ]] && grep -q 'ENABLE_ACROSS_MACHINES 只许写 0 或 1' "$work/switch-other.log" \
     && grep -q 'ENABLE_GPU 只许写 0 或 1' "$work/gpu-other.log"; echo $?)" \
  "ENABLE_ACROSS_MACHINES=yes 退 $switch_other_exit（$(tail -2 "$work/switch-other.log" | tr '\n' '|')），ENABLE_GPU=2 退 $gpu_other_exit（$(tail -2 "$work/gpu-other.log" | tr '\n' '|')）"

# ⑥ 没登记 shard=across-machines 的用例
bash "$driver" crash-case:c561-sigma-full "$copy" > "$work/not-shardable.log" 2>&1
not_shardable_exit=$?
expect "⑥ 没登记 shard=across-machines 的用例 ⇒ 拒（退 1），出路是单机跑" \
  "$([[ $not_shardable_exit == 1 ]] && grep -q '没登记 shard=across-machines' "$work/not-shardable.log"; echo $?)" \
  "退 $not_shardable_exit；输出：$(tail -3 "$work/not-shardable.log" | tr '\n' '|')"

# ⑨ 树里没有规范副本、「第二台」那棵树找不回主仓：树是临时仓的 orphan worktree（git common-dir 指回临时仓，本机的准入模块从临时仓找得到规范副本），
# 文件按硬链接从临时仓铺过去、不带规范副本；驱动用这棵树里的那一份（与门禁 checker-tier-layer0-replay --full 调 $ROOT/research/scripts/layer0-shard-run.sh 同形）
tree_without_sop_copy="$work/tree-without-sop-copy"
peer_runs_before_sop_cell="$(ls -A "$work/peer/runs" 2>/dev/null)"
if git -C "$copy" worktree add -q --orphan -b layer0-shard-selftest-tree "$tree_without_sop_copy" > "$work/worktree-add.log" 2>&1 \
    && rsync -a --link-dest="$copy" --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop "$copy/" "$tree_without_sop_copy/" >> "$work/worktree-add.log" 2>&1; then
  bash "$tree_without_sop_copy/research/scripts/layer0-shard-run.sh" "$selftest_case_key" "$tree_without_sop_copy" > "$work/without-sop-copy.log" 2>&1
  without_sop_copy_exit=$?
else
  without_sop_copy_exit="worktree-not-built（$(tail -3 "$work/worktree-add.log" | tr '\n' '|')）"
fi
expect "⑨ 树里没有规范副本、「第二台」那棵树找不回主仓 ⇒ 驱动在 ② 把驱动所在主仓的规范副本拷过去，第二台算得出同一个输入指纹，判绿，这一趟的树删了" \
  "$([[ $without_sop_copy_exit == 0 && ! -e "$tree_without_sop_copy/.claude/singlefs-ai-sop" && -z "$peer_runs_before_sop_cell" && -z "$(ls -A "$work/peer/runs" 2>/dev/null)" ]] \
     && grep -qF "规范副本 $copy/.claude/singlefs-ai-sop 拷到第二台" "$work/without-sop-copy.log" && grep -q '③ 两台的输入指纹相同' "$work/without-sop-copy.log"; echo $?)" \
  "退 $without_sop_copy_exit；跑之前第二台 runs/ 下「${peer_runs_before_sop_cell//$'\n'/ }」，跑之后「$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')」；输出尾部：$(tail -6 "$work/without-sop-copy.log" 2>/dev/null | tr '\n' '|')"

# ⑩ ③ 之后失败：清场之后回读没过（QUIESCE_STOPPED_CHECK_COMMAND=false）；第二台上这一趟的树照 ⑦ 删掉，进度目录留着
sed 's/^QUIESCE_STOPPED_CHECK_COMMAND=.*/QUIESCE_STOPPED_CHECK_COMMAND=false/' "$work/multi-host.env" > "$work/quiesce-check-fails.env"
peer_runs_before_cleanup_cell="$(ls -A "$work/peer/runs" 2>/dev/null)"
SINGLEFS_MULTI_HOST_CONFIG="$work/quiesce-check-fails.env" bash "$driver" "$selftest_case_key" "$copy" > "$work/quiesce-check-fails.log" 2>&1
quiesce_check_fails_exit=$?
expect "⑩ ③ 之后失败（清场之后回读没过）⇒ 退 1，「第二台」上这一趟的树照 ⑦ 删掉，两片的进度目录留着" \
  "$([[ $quiesce_check_fails_exit == 1 && "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_cleanup_cell" && -d "$work/peer/progress/$fingerprint" ]] \
     && grep -q '清场之后回读没过' "$work/quiesce-check-fails.log" && grep -q '⑦ 删掉第二台上这一趟的树与编译目录' "$work/quiesce-check-fails.log"; echo $?)" \
  "退 $quiesce_check_fails_exit；跑之前第二台 runs/ 下「${peer_runs_before_cleanup_cell//$'\n'/ }」，跑之后「$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')」；进度目录 $work/peer/progress/$fingerprint $([[ -d "$work/peer/progress/$fingerprint" ]] && echo 在 || echo 不在)；输出尾部：$(tail -6 "$work/quiesce-check-fails.log" | tr '\n' '|')"

# ⑰ 编译的上限太小：驱动退 1、报「编译上限太小」，第二台这一趟的树删了（假 cargo 的编译那一趟先占 256 MiB；真 cargo 真编）
peer_runs_before_build_cap_cell="$(ls -A "$work/peer/runs" 2>/dev/null)"
build_cap_settings=(SINGLEFS_LAYER0_PEER_BUILD_MEMORY_CAP="$peer_build_memory_cap_too_small")
if [[ -n "$build_allocate_mib" ]]; then build_cap_settings+=(LAYER0_SHARD_SELFTEST_BUILD_ALLOCATE_MIB="$build_allocate_mib"); fi
env "${build_cap_settings[@]}" bash "$driver" "$selftest_case_key" "$copy" > "$work/build-cap-too-small.log" 2>&1
build_cap_too_small_exit=$?
expect "⑰ 编译的上限太小（SINGLEFS_LAYER0_PEER_BUILD_MEMORY_CAP=$peer_build_memory_cap_too_small）⇒ 驱动退 1、报「编译上限太小」，第二台这一趟的树删了" \
  "$([[ $build_cap_too_small_exit == 1 && "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_build_cap_cell" ]] && grep -q '编译上限太小' "$work/build-cap-too-small.log"; echo $?)" \
  "退 $build_cap_too_small_exit；第二台 runs/ 下「$(ls -A "$work/peer/runs" 2>/dev/null | tr '\n' ' ')」；输出尾部：$(tail -8 "$work/build-cap-too-small.log" | tr '\n' '|')"

# ⑪ ⑫ 余量与第二台上限的覆盖：设了就带给第二台那一条包装 / 覆盖配置；没设的情形核 ① 那一趟（假 cargo 记下的在 $work/peer-cgroup）
export LAYER0_SHARD_SELFTEST_PEER_CGROUP="$work/peer-cgroup-overridden"
RUN_WITH_MEMORY_CAP_RESERVE=10G SINGLEFS_LAYER0_PEER_MEMORY_CAP="$peer_memory_cap_override" bash "$driver" "$selftest_case_key" "$copy" > "$work/overridden.log" 2>&1
overridden_exit=$?
SINGLEFS_LAYER0_PEER_MEMORY_CAP=2 bash "$driver" "$selftest_case_key" "$copy" > "$work/unitless-peer-cap.log" 2>&1
unitless_peer_cap_exit=$?
unset LAYER0_SHARD_SELFTEST_PEER_CGROUP
reserve_problems=()
[[ $overridden_exit == 0 ]] || reserve_problems+=("设了两样那一趟退 $overridden_exit：$(tail -4 "$work/overridden.log" | tr '\n' '|')")
grep -q '第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 RUN_WITH_MEMORY_CAP_RESERVE=10G bash research/scripts/run-with-memory-cap.sh ' "$work/overridden.log" \
  || reserve_problems+=("设了余量那一趟，驱动 ④ 那一行没写第二台经 RUN_WITH_MEMORY_CAP_RESERVE=10G 起包装")
grep -q '第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 bash research/scripts/run-with-memory-cap.sh ' "$work/standalone.log" \
  || reserve_problems+=("没设余量的 ① 那一趟，驱动 ④ 那一行带了余量（或没经包装）")
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  grep -qx 'reserve=10G' "$work/peer-cgroup-overridden" 2>/dev/null || reserve_problems+=("设了余量那一趟，第二台那一片记下的：$(grep '^reserve=' "$work/peer-cgroup-overridden" 2>/dev/null)")
  grep -qx 'reserve=unset' "$work/peer-cgroup" 2>/dev/null || reserve_problems+=("没设余量的 ① 那一趟，第二台那一片记下的：$(grep '^reserve=' "$work/peer-cgroup" 2>/dev/null)")
fi
expect "⑪ 本机设了 RUN_WITH_MEMORY_CAP_RESERVE=10G ⇒ 第二台那一条 run-with-memory-cap.sh 带同一个值；没设就不带" "${#reserve_problems[@]}" \
  "$(IFS='；'; echo "${reserve_problems[*]}")"
peer_cap_override_problems=()
grep -qF "第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 RUN_WITH_MEMORY_CAP_RESERVE=10G bash research/scripts/run-with-memory-cap.sh $peer_memory_cap_override 起" "$work/overridden.log" \
  || peer_cap_override_problems+=("设了 SINGLEFS_LAYER0_PEER_MEMORY_CAP=$peer_memory_cap_override 那一趟，驱动 ④ 那一行上限不是 $peer_memory_cap_override")
if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
  grep -qx "memory.max=$(gibibytes_in_bytes "$peer_memory_cap_override")" "$work/peer-cgroup-overridden" 2>/dev/null \
    || peer_cap_override_problems+=("设了 $peer_memory_cap_override 那一趟，第二台那一片的 scope：$(grep '^memory.max=' "$work/peer-cgroup-overridden" 2>/dev/null)")
  grep -qx "memory.max=$(gibibytes_in_bytes "$peer_memory_cap")" "$work/peer-cgroup" 2>/dev/null \
    || peer_cap_override_problems+=("没设的 ① 那一趟，第二台那一片的 scope 不是配置的 $peer_memory_cap：$(grep '^memory.max=' "$work/peer-cgroup" 2>/dev/null)")
fi
[[ $unitless_peer_cap_exit == 1 ]] && grep -q 'SINGLEFS_LAYER0_PEER_MEMORY_CAP 要写第二台那一片的内存上限' "$work/unitless-peer-cap.log" \
  || peer_cap_override_problems+=("SINGLEFS_LAYER0_PEER_MEMORY_CAP=2（不带单位）没被拒：退 $unitless_peer_cap_exit，$(tail -3 "$work/unitless-peer-cap.log" | tr '\n' '|')")
expect "⑫ 设了 SINGLEFS_LAYER0_PEER_MEMORY_CAP=$peer_memory_cap_override ⇒ 覆盖配置的 PEER_MEMORY_CAP（第二台那一片的 scope 是 $peer_memory_cap_override）；没设照配置的 $peer_memory_cap；不带单位的拒" \
  "${#peer_cap_override_problems[@]}" "$(IFS='；'; echo "${peer_cap_override_problems[*]}")"

# ⑬ 两片跑到一半给驱动发 TERM（假 cargo 在两片里各记下进程号、停着等）；只给驱动自己发（kill "$!"），不发给它的子进程
hold_file="$work/shards-held"
: > "$hold_file"
peer_runs_before_interrupt="$(ls -A "$work/peer/runs" 2>/dev/null)"
# 等两片的进程号都记下的上限（文件头 ⑬）：假 cargo 600 秒、真 cargo 3600 秒；LAYER0_SHARD_SELFTEST_HOLD_SECONDS 设了就用它
hold_wait_seconds=600
if [[ -n "${SINGLEFS_HEAVY_TESTS:-}" ]]; then hold_wait_seconds=3600; fi
hold_wait_seconds="${LAYER0_SHARD_SELFTEST_HOLD_SECONDS:-$hold_wait_seconds}"
[[ "$hold_wait_seconds" =~ ^[1-9][0-9]*$ ]] || hold_wait_seconds=600
export LAYER0_SHARD_SELFTEST_HOLD_FILE="$hold_file" LAYER0_SHARD_SELFTEST_HOLD_SECONDS="$hold_wait_seconds"
# 第二台那一片的外层由自证按住（驱动的只供测试的开关）：这个文件在，外层就不写退出码文件；驱动要等它写完再删第二台这一趟的东西
peer_exit_hold_file="$work/peer-exit-hold"
: > "$peer_exit_hold_file"
SINGLEFS_LAYER0_SHARD_PEER_EXIT_HOLD_FILE="$peer_exit_hold_file" bash "$driver" "$selftest_case_key" "$copy" > "$work/interrupted.log" 2>&1 &
interrupted_driver=$!
unset LAYER0_SHARD_SELFTEST_HOLD_FILE LAYER0_SHARD_SELFTEST_HOLD_SECONDS
# 发 TERM 之前轮询：两片的进程号都记下了才发（held）；驱动先退了（driver-exited）、到上限还没都记下（timed-out）也发，用来收尾
hold_deadline=$((SECONDS + hold_wait_seconds))
hold_outcome=""
while [[ -z "$hold_outcome" ]]; do
  if [[ "$(grep -c '^shard=' "$hold_file")" == 2 ]]; then hold_outcome=held
  elif ! kill -0 "$interrupted_driver" 2>/dev/null; then hold_outcome=driver-exited
  elif (( SECONDS >= hold_deadline )); then hold_outcome=timed-out
  else sleep 0.1; fi
done
hold_waited_seconds=$((SECONDS + hold_wait_seconds - hold_deadline))
mapfile -t held_processes < <(sed -n 's/^shard=.* pid=//p' "$hold_file")
peer_outer="$(sed -n 's/^peer_outer=//p' "$hold_file" | head -1)"
kill -TERM "$interrupted_driver"
# 外层按住的时候（放走之前它写不出退出码文件），驱动该停在等外层那一步：至多按住 15 秒（驱动等外层的上限是 30 秒）。这期间驱动报了第二台那一片
# 停没停、或者退了，都是没等外层就往下收尾；驱动退了就放（没等的驱动删完之后，外层才写出退出码文件）。驱动退没退看 ps -o stat=（僵尸算退了）
peer_exit_release_deadline=$((SECONDS + 15))
reported_before_release="" exited_before_release=""
while [[ "$hold_outcome" == held ]]; do
  if [[ -z "$reported_before_release" ]] && grep -qE '第二台那一片（测试二进制）：|第二台那一片没停下' "$work/interrupted.log"; then reported_before_release=1; fi
  interrupted_driver_state="$(ps -o stat= -p "$interrupted_driver" 2>/dev/null)"
  if [[ -z "$interrupted_driver_state" || "$interrupted_driver_state" == Z* ]]; then exited_before_release=1; break; fi
  (( SECONDS < peer_exit_release_deadline )) || break
  sleep 0.1
done
rm -f -- "$peer_exit_hold_file"
wait "$interrupted_driver"
interrupted_exit=$?
# 等第二台那一片的外层退了再看 runs/（它退之前还可能写出退出码文件）：至多 120 秒；核命令行还是它（进程号没被重用）
peer_outer_problem=""
if [[ ! "$peer_outer" =~ ^[1-9][0-9]*$ ]]; then
  peer_outer_problem="第 1/2 片没记下第二台那一片的外层（记下的：${peer_outer:-无}）"
else
  for ((outer_rounds = 0; outer_rounds < 1200; outer_rounds++)); do
    grep -qaF '.shard-1-of-2.exit' "/proc/$peer_outer/cmdline" 2>/dev/null || break
    sleep 0.1
  done
  if grep -qaF '.shard-1-of-2.exit' "/proc/$peer_outer/cmdline" 2>/dev/null; then peer_outer_problem="第二台那一片的外层（进程 $peer_outer）驱动退了 120 秒之后还在"; fi
fi
still_running=()
for held_process in "${held_processes[@]}"; do
  held_state="$(ps -o stat= -p "$held_process" 2>/dev/null)"
  [[ -z "$held_state" || "$held_state" == Z* ]] || still_running+=("$held_process")
done
rm -f -- "$hold_file"  # 没停掉的两片看到它不在就自己退
interrupt_problems=()
[[ $interrupted_exit == 143 ]] || interrupt_problems+=("驱动退 $interrupted_exit，不是 143")
[[ ${#held_processes[@]} == 2 ]] || interrupt_problems+=("发 TERM 之前两片没都停在那里（记下 ${#held_processes[@]} 个进程号，等了 ${hold_waited_seconds} 秒：$hold_outcome）")
[[ -z "$peer_outer_problem" ]] || interrupt_problems+=("$peer_outer_problem")
(( ${#still_running[@]} == 0 )) || interrupt_problems+=("TERM 之后两片还在跑：${still_running[*]}")
if [[ -n "$reported_before_release$exited_before_release" ]]; then
  interrupt_problems+=("驱动没等第二台那一片的外层写完退出码文件就往下收尾：外层按住的那几秒里驱动$([[ -n "$reported_before_release" ]] && echo 报了第二台那一片停没停)$([[ -n "$reported_before_release" && -n "$exited_before_release" ]] && echo 、)$([[ -n "$exited_before_release" ]] && echo 退了)")
fi
[[ "$hold_outcome" != held ]] || grep -qE '第二台那一片（测试二进制）：|第二台那一片没停下' "$work/interrupted.log" || interrupt_problems+=("驱动没报第二台那一片停没停")
[[ "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_interrupt" ]] || interrupt_problems+=("第二台 runs/ 下多了：$(ls -A "$work/peer/runs" | tr '\n' ' ')")
[[ -d "$work/peer/progress/$fingerprint" && -d "$git_common_directory/singlefs-layer0-progress/$fingerprint" ]] || interrupt_problems+=("两片的进度目录不在了")
interrupt_cell="⑬ 两片跑到一半给驱动发 TERM（第二台那一片的外层由自证按住，15 秒后才放它写退出码文件）⇒ 先停两片、等那一片的外层写完退出码文件，再删第二台这一趟的树（不复原），退 143，进度目录还在"
if [[ "$hold_outcome" == timed-out ]]; then
  # 到上限两片的进程号还没都记下：这一格没跑成，不判通过（文件头 ⑬）
  checked=$((checked + 1)); not_run=$((not_run + 1))
  echo "  ✗ ${interrupt_cell}：没跑成（等了 ${hold_wait_seconds} 秒，两片的进程号只记下 ${#held_processes[@]} 个；发 TERM 收了尾，这一格没判）；驱动输出尾部：$(tail -8 "$work/interrupted.log" | tr '\n' '|')"  # gate-lint:detail
else
  expect "$interrupt_cell" "${#interrupt_problems[@]}" \
    "$(IFS='；'; echo "${interrupt_problems[*]}")；驱动输出尾部：$(tail -8 "$work/interrupted.log" | tr '\n' '|')"
fi

# ⑱ 跑完不调复原命令：① 到 ⑬ 各趟一笔复原都没记；① 与 ⑬ 那两趟报了「清了场、没复原」；复原那一对写成空串的配置照样判得过
sed -e 's/^QUIESCE_START_COMMAND=.*/QUIESCE_START_COMMAND=/' -e 's/^QUIESCE_STARTED_CHECK_COMMAND=.*/QUIESCE_STARTED_CHECK_COMMAND=/' "$work/multi-host.env" > "$work/without-restore.env"
SINGLEFS_MULTI_HOST_CONFIG="$work/without-restore.env" bash "$configuration_check" "$copy" > "$work/without-restore.log" 2>&1
without_restore_exit=$?
no_restore_problems=()
[[ ! -e "$work/restore-calls" ]] || no_restore_problems+=("配置的复原命令记了 $(grep -c . "$work/restore-calls") 笔")
grep -q '清了场、没复原' "$work/standalone.log" || no_restore_problems+=("① 那一趟没报「清了场、没复原」")
if [[ "$hold_outcome" == held ]] && ! grep -q '清了场、没复原' "$work/interrupted.log"; then no_restore_problems+=("⑬ 那一趟没报「清了场、没复原」"); fi
[[ $without_restore_exit == 0 ]] || no_restore_problems+=("复原那一对写成空串的配置判不过：退 $without_restore_exit（$(tail -2 "$work/without-restore.log" | tr '\n' '|')）")
expect "⑱ 跑完不调复原命令：① 到 ⑬ 各趟（跑完、跑红、被 TERM 打断）配置的复原命令一笔都没记，① 与 ⑬ 报了「清了场、没复原」；复原那一对写成空串的配置照样判得过" \
  "${#no_restore_problems[@]}" "$(IFS='；'; echo "${no_restore_problems[*]}")"

# ⑲ 第二台那一边的内存包装认出显存不够（退 245）：假 journalctl 问最近一条打一行、问起止之间打显卡驱动报的显存不够；
#    编测试二进制那一趟、第 1/2 片各起一趟驱动（文件头 ⑲），与 ⑳ ㉑ 一起放在最后：几趟都判红，不留给后面的格
cat > "$work/fake-journalctl" <<'FAKE_JOURNAL'
#!/usr/bin/env bash
# 自证用的假 journalctl（RUN_WITH_MEMORY_CAP_JOURNALCTL）：问最近一条打一行，问起止之间打一行显卡驱动报的显存不够
case " $* " in
  *" -n 1 "*) echo "host kernel: probe" ;;
  *) echo "host kernel: NVRM: GPU0 nvAssertOkFailedNoLog: Out of memory [NV_ERR_NO_MEMORY] (0x00000051)" ;;
esac
FAKE_JOURNAL
chmod +x "$work/fake-journalctl"
peer_runs_before_gpu_cells="$(ls -A "$work/peer/runs" 2>/dev/null)"
RUN_WITH_MEMORY_CAP_JOURNALCTL="$work/fake-journalctl" LAYER0_SHARD_SELFTEST_PEER_BUILD_FAIL=1 bash "$driver" "$selftest_case_key" "$copy" > "$work/peer-build-gpu.log" 2>&1
peer_build_gpu_exit=$?
peer_runs_after_build_gpu="$(ls -A "$work/peer/runs" 2>/dev/null)"
# 判两支各自独有的句子：「显存不够」那一支的 fail 第一句在、「撞了编译的内存上限」那一支（包装退 250）的 fail 第一句不在。
# 不拿「显存不够」单字判：第二台的内存包装自己也打「run-with-memory-cap: 显存不够」，驱动不解释时照样命中
peer_build_gpu_problems=()
[[ $peer_build_gpu_exit == 1 ]] || peer_build_gpu_problems+=("退 $peer_build_gpu_exit，不是 1")
[[ "$peer_runs_after_build_gpu" == "$peer_runs_before_gpu_cells" ]] || peer_build_gpu_problems+=("第二台 runs/ 下多了：$(tr '\n' ' ' <<< "$peer_runs_after_build_gpu")")
grep -qF '第二台编测试二进制那一趟显存不够' "$work/peer-build-gpu.log" || peer_build_gpu_problems+=("没报「第二台编测试二进制那一趟显存不够」")
! grep -qF '第二台编测试二进制撞了编译的内存上限' "$work/peer-build-gpu.log" || peer_build_gpu_problems+=("报了包装退 250 那一支的「第二台编测试二进制撞了编译的内存上限」")
! grep -qF '编译上限太小' "$work/peer-build-gpu.log" || peer_build_gpu_problems+=("说了「编译上限太小」")
expect "⑲ 第二台编测试二进制那一趟内存包装退 245 ⇒ 驱动退 1、报「第二台编测试二进制那一趟显存不够」、不报撞了编译的内存上限、不说编译上限太小，第二台这一趟的树删了" \
  "${#peer_build_gpu_problems[@]}" "$(IFS='；'; echo "${peer_build_gpu_problems[*]}")；输出尾部：$(tail -6 "$work/peer-build-gpu.log" | tr '\n' '|')"
RUN_WITH_MEMORY_CAP_JOURNALCTL="$work/fake-journalctl" LAYER0_SHARD_SELFTEST_PEER_SHARD_FAIL=1 bash "$driver" "$selftest_case_key" "$copy" > "$work/peer-shard-gpu.log" 2>&1
peer_shard_gpu_exit=$?
peer_runs_after_shard_gpu="$(ls -A "$work/peer/runs" 2>/dev/null)"
expect "⑲ 第二台那一片内存包装退 245 ⇒ 驱动退 1、报「第二台那一片退 245：显存不够」、不算结果，第二台这一趟的树删了" \
  "$([[ $peer_shard_gpu_exit == 1 && "$peer_runs_after_shard_gpu" == "$peer_runs_before_gpu_cells" ]] && grep -q '第二台那一片退 245：显存不够' "$work/peer-shard-gpu.log" \
     && grep -q '不算结果' "$work/peer-shard-gpu.log"; echo $?)" \
  "退 $peer_shard_gpu_exit；第二台 runs/ 下「$(tr '\n' ' ' <<< "$peer_runs_after_shard_gpu")」；输出尾部：$(tail -6 "$work/peer-shard-gpu.log" | tr '\n' '|')"

# ⑳ ㉑ 判第二台那一片还在不在跑看外层（驱动文件头「第二台那一片跑完没有」）：两格的第 1/2 片都记下那一片的外层，自证等它退了（至多 120 秒）再看 runs/
await_recorded_peer_outer() { # await_recorded_peer_outer <记外层进程号的文件>：等那一片的外层退了（核命令行还是它）；没记下、120 秒还在就打一句问题
  local outer outer_rounds
  outer="$(cat "$1" 2>/dev/null)"
  if [[ ! "$outer" =~ ^[1-9][0-9]*$ ]]; then echo "第 1/2 片没记下第二台那一片的外层（记下的：${outer:-无}）"; return 0; fi
  for ((outer_rounds = 0; outer_rounds < 1200; outer_rounds++)); do
    grep -qaF '.shard-1-of-2.exit' "/proc/$outer/cmdline" 2>/dev/null || return 0
    sleep 0.1
  done
  echo "第二台那一片的外层（进程 $outer）驱动退了 120 秒之后还在"
}
# ⑳ 第二台那一片的内存包装收尾拉长：假 journalctl 每次先睡 12 秒（第 1/2 片退 3，包装退非 0 时调它两次，收尾拉长约 24 秒，
#    改前的驱动连续 3 次、每次隔 5 秒看到内层没了就判 vanished）⇒ 驱动看外层还在、接着等，读到退出码 245
cat > "$work/slow-fake-journalctl" <<SLOW_FAKE_JOURNAL
#!/usr/bin/env bash
# ⑳ 用的慢假 journalctl：每次先睡 12 秒，再照 ⑲ 那一份打
sleep 12
exec $(printf '%q' "$work/fake-journalctl") "\$@"
SLOW_FAKE_JOURNAL
chmod +x "$work/slow-fake-journalctl"
peer_runs_before_slow_finish="$(ls -A "$work/peer/runs" 2>/dev/null)"
RUN_WITH_MEMORY_CAP_JOURNALCTL="$work/slow-fake-journalctl" LAYER0_SHARD_SELFTEST_PEER_SHARD_FAIL=1 LAYER0_SHARD_SELFTEST_PEER_OUTER_FILE="$work/peer-outer-slow-finish" \
  bash "$driver" "$selftest_case_key" "$copy" > "$work/peer-slow-finish.log" 2>&1
slow_finish_exit=$?
slow_finish_problems=()
slow_finish_outer_problem="$(await_recorded_peer_outer "$work/peer-outer-slow-finish")"
[[ $slow_finish_exit == 1 ]] || slow_finish_problems+=("退 $slow_finish_exit，不是 1")
if grep -q '第二台那一片退 vanished' "$work/peer-slow-finish.log"; then slow_finish_problems+=("驱动判成了 vanished：$(grep -o '第二台那一片退 vanished[^；]*' "$work/peer-slow-finish.log" | head -1)"); fi
grep -qF '第二台那一片退 245：显存不够' "$work/peer-slow-finish.log" || slow_finish_problems+=("没报「第二台那一片退 245：显存不够」")
grep -qF '不算结果' "$work/peer-slow-finish.log" || slow_finish_problems+=("没说「不算结果」")
[[ -z "$slow_finish_outer_problem" ]] || slow_finish_problems+=("$slow_finish_outer_problem")
[[ "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_slow_finish" ]] || slow_finish_problems+=("第二台 runs/ 下多了：$(ls -A "$work/peer/runs" | tr '\n' ' ')")
expect "⑳ 第二台那一片的内存包装收尾拉长到 24 秒左右（内层退了、外层还没写退出码文件）⇒ 驱动看外层还在、接着等，不判 vanished，报「第二台那一片退 245：显存不够」、不算结果，第二台这一趟的树删了" \
  "${#slow_finish_problems[@]}" "$(IFS='；'; echo "${slow_finish_problems[*]}")；输出尾部：$(tail -6 "$work/peer-slow-finish.log" | tr '\n' '|')"

# ㉑ 第二台那一片的外层没留退出码文件就没了：第 1/2 片把退出码文件换成指向不存在的目录里的符号链接，外层写不进去就退了 ⇒ 驱动判 vanished。
#    驱动放后台跑：外层没了之后等它至多 90 秒（判 vanished 要连续 3 次、每次隔 5 秒），还不退就只给它自己发 TERM（kill "$!" 记下的那个）收尾、判这一格错；
#    外层一直没记下、没退，至多等 hold_wait_seconds（同 ⑬），到了照样发 TERM 收尾，这一格判「没跑成」
peer_runs_before_outer_lost="$(ls -A "$work/peer/runs" 2>/dev/null)"
LAYER0_SHARD_SELFTEST_PEER_OUTER_FILE="$work/peer-outer-lost" LAYER0_SHARD_SELFTEST_PEER_EXIT_FILE_TARGET="$work/no-such-directory/exit" \
  bash "$driver" "$selftest_case_key" "$copy" > "$work/peer-outer-lost.log" 2>&1 &
outer_lost_driver=$!
outer_lost_deadline=$((SECONDS + hold_wait_seconds))
outer_lost_outcome="" outer_gone_at=""
while [[ -z "$outer_lost_outcome" ]]; do
  outer_lost_driver_state="$(ps -o stat= -p "$outer_lost_driver" 2>/dev/null)"
  recorded_lost_outer="$(cat "$work/peer-outer-lost" 2>/dev/null)"
  if [[ -z "$outer_gone_at" && "$recorded_lost_outer" =~ ^[1-9][0-9]*$ ]] && ! grep -qaF '.shard-1-of-2.exit' "/proc/$recorded_lost_outer/cmdline" 2>/dev/null; then
    outer_gone_at=$SECONDS
  fi
  if [[ -z "$outer_lost_driver_state" || "$outer_lost_driver_state" == Z* ]]; then outer_lost_outcome=exited
  elif [[ -n "$outer_gone_at" ]] && (( SECONDS - outer_gone_at >= 90 )); then outer_lost_outcome=hung
  elif (( SECONDS >= outer_lost_deadline )); then outer_lost_outcome=timed-out
  else sleep 0.2; fi
done
if [[ "$outer_lost_outcome" != exited ]]; then kill -TERM "$outer_lost_driver"; fi
wait "$outer_lost_driver"
outer_lost_exit=$?
outer_lost_problems=()
outer_lost_outer_problem="$(await_recorded_peer_outer "$work/peer-outer-lost")"
[[ "$outer_lost_outcome" != hung ]] || outer_lost_problems+=("外层没了 90 秒驱动还没退（还当第二台那一片在跑），发了 TERM 收尾")
[[ $outer_lost_exit == 1 ]] || outer_lost_problems+=("驱动退 $outer_lost_exit，不是 1")
grep -qF '第二台那一片退 vanished（外层进程没了、没留退出码）' "$work/peer-outer-lost.log" || outer_lost_problems+=("没报「第二台那一片退 vanished（外层进程没了、没留退出码）」")
[[ -z "$outer_lost_outer_problem" ]] || outer_lost_problems+=("$outer_lost_outer_problem")
[[ "$(ls -A "$work/peer/runs" 2>/dev/null)" == "$peer_runs_before_outer_lost" ]] || outer_lost_problems+=("第二台 runs/ 下多了：$(ls -A "$work/peer/runs" | tr '\n' ' ')")
outer_lost_cell="㉑ 第二台那一片的外层没留退出码文件就没了 ⇒ 驱动判 vanished（退 1，报「第二台那一片退 vanished（外层进程没了、没留退出码）」），第二台这一趟的树删了"
if [[ "$outer_lost_outcome" == timed-out ]]; then
  checked=$((checked + 1)); not_run=$((not_run + 1))
  echo "  ✗ ${outer_lost_cell}：没跑成（等了 ${hold_wait_seconds} 秒，第二台那一片的外层没记下或一直没退；发 TERM 收了尾，这一格没判）；驱动输出尾部：$(tail -6 "$work/peer-outer-lost.log" | tr '\n' '|')"  # gate-lint:detail
else
  expect "$outer_lost_cell" "${#outer_lost_problems[@]}" "$(IFS='；'; echo "${outer_lost_problems[*]}")；输出尾部：$(tail -6 "$work/peer-outer-lost.log" | tr '\n' '|')"
fi

if (( failures > 0 || not_run > 0 )); then
  echo "  ✗ layer0-shard-run.sh 自证没过：${failures} 格判错、${not_run} 格没跑成（共 ${checked} 格；没跑成的不算通过，照那一行写的原因重跑）"
  echo "     → 怎么办：照上面每一格的说明改驱动脚本（research/scripts/layer0-shard-run.sh）或分片那一段（crates/singlefs-checker-tier/src/layer0_progress.rs、crash.rs）"
  exit 1
fi
echo "  ✓ layer0-shard-run.sh 自证通过：${checked} 格都对（${cargo_mode}；第二台是本机上的另一个目录，没碰真的第二台）"
