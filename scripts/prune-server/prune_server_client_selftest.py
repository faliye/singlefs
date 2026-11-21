#!/usr/bin/env python3
# admission: always 每一格都起此刻编出的 prune-server 与此刻的三家脚本，上一次的结论不替这一次作保
"""prune_server_client.py 的自证（格、弄坏开关、三家原有自证对临时服务重跑）与临时服务的起法。与客户端分成两份文件：客户端被三家判定库 import，
它的模块级 main-guard 会随 import 进每一家判法的单元闭包，自证里起服务、起 cargo、载入 admission.py 的边不能挂在客户端上
（lib_function_units 的 main-guard 单元；lib_verdict_library 自证「判法单元指纹不含库自己」那一格就是这么红的）。
同一个缘故，三家与旁路的原有自证不 import 这一份、不起它：它们只调客户端的 selftest_service_problem() 核环境，临时服务由这一份的
--with-temporary-service 在外面起好。

  python3 scripts/prune-server/prune_server_client_selftest.py --selftest [--cells <格名>[,<格名>…]]
  python3 scripts/prune-server/prune_server_client_selftest.py --selftest-families
  python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- <命令> [<参数>…]
三条整条都不带二进制，没给 --server-binary 就现编；自己编了、两台都读得到的二进制才在子命令后另加 --server-binary <路径>。
三条都先核多机配置 HOSTS 读得到（临时服务的 TCP 绑这一台在 HOSTS 里的地址，TestServer 的 tcp_bind_address_of_this_host，读法是 scripts/hosts/multi_host_addresses.py 的 resolve）：
读不到、认不出这一台退 78（运行条件不满足），打 ✗ 与出路，一份服务、一条命令都不起，不退回回环。
格的清单、弄坏开关、末行的写法都在 scripts/prune-server/prune_server_client.py 文件头「命令行」「自证的格」两段；
弄坏开关轮怎么跑与这一份自己加的三格、十三条开关写在下面。

--cells 只走点名的格（次序照 CELLS，重复的走一次），在本进程里逐格串行跑；格名认不出、缺格名、有空项退 2，并列出可用的格名。点名了格就不跑末尾的弄坏开关轮，打一行说明本次未跑。
不给 --cells 走全部格，成功句的格数是 CELLS 的条数；各格并行跑，每格一个 --selftest --cells <格名> 子进程（有的格改本进程的环境变量，不能在同一个进程里开线程并行）：
并行度取每台默认重活线程数（scripts/hosts/heavy_threads.py，再按格数夹），每个子进程自己一个进程组、自己一个 TMPDIR、环境里带 SWEEP_TOKEN_VARIABLE 标记、
带 SWEEP_CHILD_SECONDS 秒的时限（超时、停整组、核留下的后代都与弄坏开关轮同一份实现 run_sweep_job）。各格的临时目录、socket、TCP 端口（服务绑多机配置 HOSTS 里这一台的地址、端口由内核挑，不绑回环）、服务状态目录与库都在格自己的目录里，
格之间没有共用的可写状态。收回按 CELLS 的次序逐格判、逐格打 ✓ / ✗（后面的格先跑完也等前面的）：一格要同时有它子进程打的「✓ 自证 <格名>：」行、
一格全绿的末行、子进程退 0 且退出之后没有带标记的进程留下，才算绿；红的格打子进程自己的 ✗ 块并附子进程的问题，子进程没交出结果的补一行写明退出码与输出末尾。
收回的条数不等于派出的条数，没收回的格按红算并打一行说明。末行 PRUNE_SERVER_CLIENT_SELFTEST 的写法不变。

弄坏开关轮（全绿之后）：每个开关起一个 --selftest 子进程，只交它在 BREAK_RED_CELLS 里点名的红格（同一个开关点名几格交几格），判红的依据是
「退出码不是 0，且标准输出里有每一格的『✗ 自证 <格名>：』」。各开关的子进程互相独立，并行跑，并行度取每台默认重活线程数
（scripts/hosts/heavy_threads.py，再按开关条数夹）；每个子进程自己一个进程组、自己一个 TMPDIR，环境里带 SWEEP_TOKEN_VARIABLE 标记（后代继承）。
超过 SWEEP_CHILD_SECONDS 秒没退出：对整组发 TERM，过 SWEEP_STOP_GRACE_SECONDS 秒组里还有活的发 KILL；那个开关记没对上，写明「超时」。
超时与正常退出一样，之后核带标记的后代（换了进程组、换了会话、被收养的都还带着标记）：宽限之内没自己退掉的，逐个点名（进程号与命令行）记进那个开关的没对上，
不由这一份去停（起的后代都要在父进程没了之后自己退；临时服务同理，见格 temporary-service-exits-with-parent；
自证里的假进程另盯着开关子进程的临时目录，目录一删就退）。临时目录无论如何删掉。
逐个收回每个开关的结果，收回的条数不等于派出的条数整轮报错。末行 PRUNE_SERVER_CLIENT_BREAK_SWEEP switches=… matched=… unmatched=… 的写法不变。

这一份自己加的格（run_break_sweep、--cells 的解析与 judge_cells_in_children 是它们的被测物，假子进程不起服务、不编译）：
  sweep-child-runs-only-named-cells  sweep_child_command 拼出的命令交给 parse_selftest_options，走的恰是点名的格；--cells 缺格名、认不出的格名拒并列出可用格名；不给 --cells 走全部
  sweep-stops-hung-child             三个假子进程并行：卡死的（忽略 TERM，另起两个忽略 TERM 的孙进程，一个同组、一个换了会话并盯着父进程、父进程没了就自己退）要在时限加两倍宽限之内被停、
                                     记「超时」、组里停 2 个、到点时组外带标记的 1 个、停完组之后宽限之内这个组外的自己退掉（没有点名留下的进程）、临时目录删了、
                                     整轮回来时没有带标记的进程留下；红在点名格的记对上；退 0 没红的记没对上；收回 3 条
  cells-run-in-parallel-children     judge_cells_in_children 对假的格子进程：四个假子进程互相等齐才退（一路一路跑等不齐），退出的次序与派出的次序相反；
                                     要全部等齐、各自一个进程号与进程组与临时目录（跑完删了）、都不是本进程的，逐格交出的次序是派出的次序、全绿、收回 4 条没抛错；
                                     另一趟三个假子进程各演一种红（打 ✗ 行退 1、打 ✓ 行与全绿的末行却退 1、什么都不打退 0）都判红并点名那一格
弄坏开关 PRUNE_SERVER_CLIENT_BREAK=<项>（写法同 scripts/prune-server/prune_server_client.py 文件头「弄坏开关」，一条一行，下一行写打开后红的格）：
  PRUNE_SERVER_CLIENT_BREAK=sweep-child-runs-all-cells  sweep_child_command 不带 --cells，子进程走全部格
      sweep-child-runs-only-named-cells（「✗ 自证 sweep-child-runs-only-named-cells：」）
  PRUNE_SERVER_CLIENT_BREAK=cells-option-ignored  parse_selftest_options 收下 --cells 却照走全部格
      sweep-child-runs-only-named-cells（「✗ 自证 sweep-child-runs-only-named-cells：」）
  PRUNE_SERVER_CLIENT_BREAK=unknown-cell-name-accepted  --cells 里认不出的格名悄悄丢掉、不退 2
      sweep-child-runs-only-named-cells（「✗ 自证 sweep-child-runs-only-named-cells：」）
  PRUNE_SERVER_CLIENT_BREAK=sweep-child-without-time-limit  开关子进程不设时限，等到它自己退出
      sweep-stops-hung-child（「✗ 自证 sweep-stops-hung-child：」）
  PRUNE_SERVER_CLIENT_BREAK=sweep-child-term-only  到点只发 TERM、宽限过了不发 KILL
      sweep-stops-hung-child（「✗ 自证 sweep-stops-hung-child：」）
  PRUNE_SERVER_CLIENT_BREAK=sweep-group-not-signalled  到点只给子进程自己发信号，不给整组发
      sweep-stops-hung-child（「✗ 自证 sweep-stops-hung-child：」）
  PRUNE_SERVER_CLIENT_BREAK=sweep-job-directory-kept  开关子进程的临时目录不删
      sweep-stops-hung-child（「✗ 自证 sweep-stops-hung-child：」）
  PRUNE_SERVER_CLIENT_BREAK=sweep-outcome-dropped  收回的结果丢掉最后一条再核条数
      sweep-stops-hung-child（「✗ 自证 sweep-stops-hung-child：」）
  PRUNE_SERVER_CLIENT_BREAK=sweep-descendants-left  卡死的假子进程里换了会话的孙进程不盯父进程，只带临时目录没了就退与 SWEEP_FAKE_BACKSTOP_SECONDS 秒的自退时限：
                                                   停完整组之后它还留着，被点名列出、判红，开关子进程的临时目录一删它自己退掉
      sweep-stops-hung-child（「✗ 自证 sweep-stops-hung-child：」）
  PRUNE_SERVER_CLIENT_BREAK=cells-run-serially  整份趟的各格子进程一路一路跑，不并行
      cells-run-in-parallel-children（「✗ 自证 cells-run-in-parallel-children：」）
  PRUNE_SERVER_CLIENT_BREAK=cells-merged-by-completion  整份趟的各格结果按子进程跑完的次序交，不按 CELLS 的次序
      cells-run-in-parallel-children（「✗ 自证 cells-run-in-parallel-children：」）
  PRUNE_SERVER_CLIENT_BREAK=cell-result-dropped  整份趟收回的各格结果丢掉最后一条再核条数
      cells-run-in-parallel-children（「✗ 自证 cells-run-in-parallel-children：」）
  PRUNE_SERVER_CLIENT_BREAK=cell-child-exit-code-ignored  整份趟判一格的子进程绿不看退出码、时限与留下的进程，只看它打没打 ✓ 行与末行
      cells-run-in-parallel-children（「✗ 自证 cells-run-in-parallel-children：」）

分在几份文件里：本文件留命令行入口（main）、整份自证的编排（selftest、judge_cells_in_process、default_parallel_threads、break_sweep）与 BREAK_RED_CELLS 表；其余分在同目录三份模块里，测哪一块的格跟着那一块：
  prune_server_client_selftest_service.py  临时服务的起法（编二进制、TestServer、环境、子进程）、--with-temporary-service、--selftest-families，与测二进制来处的格
  prune_server_client_selftest_cells.py   对临时服务的各格
  prune_server_client_selftest_sweep.py   弄坏开关轮、各格并行的子进程、--cells 的解析、格名表 CELLS 与格名到函数的表，与测它们的三格
本文件把三份导入并再导出；起子进程自己要跑的入口是本文件，拆出的模块里读它的路径是 prune_server_client_selftest_service.py 的 ENTRY_SCRIPT_PATH。
上面这一份自己加的十三条开关（PRUNE_SERVER_CLIENT_BREAK=sweep-child-runs-all-cells 起到 cell-child-exit-code-ignored 止）的代码都在 prune_server_client_selftest_sweep.py，拿开关名带引号的字面在那份里搜。
"""
import os
import secrets
import shutil
import subprocess
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
from prune_server_client import BINARY_VARIABLE, BREAK_VARIABLE, EXIT_USAGE, PruneServerFailure, REPOSITORY_ROOT, SELFTEST_VARIABLE
from prune_server_client_selftest_service import (
    CARGO_TARGET_DIRECTORY_VARIABLE, hosts_refusal, resolve_binary, selftest_child, selftest_families, selftest_root, with_temporary_service
)
from prune_server_client_selftest_sweep import (
    SWEEP_CHILD_SECONDS, SWEEP_STOP_GRACE_SECONDS, SweepJob, cell_child_command, judge_cells_in_children, parse_selftest_options,
    run_break_sweep, selftest_cell_functions, sweep_child_command
)
# 再导出：三份模块里定义的名字一个不少（载入本文件的地方照拆之前按名字取）
from prune_server_client_selftest_service import *  # noqa: F401,F403
from prune_server_client_selftest_cells import *  # noqa: F401,F403
from prune_server_client_selftest_sweep import *  # noqa: F401,F403


BREAK_RED_CELLS = (("read-failure-reported-as-absent", "unreachable-exits-75"), ("reconnect-unbounded", "unreachable-exits-75"),
                   ("stop-notice-retried", "stopped-exits-75"), ("busy-retry-without-delay", "busy-retried-after-delay"),
                   ("family-prefixes-shared", "families-over-service"),
                   ("unreachable-treated-as-no-library", "unreachable-family-exits-75"),
                   ("connect-given-full-response-timeout", "connect-bounded-by-reconnect-total"),
                   ("large-value-sent-unsegmented", "large-value-round-trip"),
                   ("superseded-versions-kept", "large-value-round-trip"),
                   ("segments-assembled-without-sha256", "segmented-read-incomplete-refused"),
                   ("segmented-read-not-reread", "segmented-read-sees-a-rewrite"),
                   ("segments-overwritten-in-place", "segmented-rewrite-survives-writer-death"),
                   ("segment-count-unbounded", "segment-count-over-index-digits-refused"),
                   ("scan-row-over-frame-split", "scan-row-over-frame-refused"),
                   ("scan-split-into-next-byte-prefixes", "prefix-scan-continues-after-the-last-key"),
                   ("nul-keys-reserved-whole", "segment-key-shapes-only-reserved"),
                   ("segment-key-shapes-accepted", "segment-key-shapes-only-reserved"),
                   ("other-segment-keys-scanned-as-entries", "segment-key-shapes-only-reserved"),
                   ("oversize-frame-sent-and-retried", "oversize-frame-reported-at-once"),
                   ("temporary-service-outlives-parent", "temporary-service-exits-with-parent"),
                   ("service-problem-reads-process-environment", "service-problem-judges-given-environment"),
                   ("connection-variables-honored-outside-temporary-service", "connection-variables-refused-outside-temporary-service"),
                   ("command-line-read-absent-as-empty", "command-line-read"),
                   ("namespace-by-temporary-directory", "namespace-only-under-temporary-service"),
                   ("oversized-answer-read-unsplit", "batched-read-answer-over-frame-limit-split"),
                   ("build-without-target-directory", "binary-not-built-without-target-directory"),
                   ("delete-prefix-without-namespace", "delete-prefix-by-namespace"),
                   ("item-name-ignores-kind", "delete-by-item-name-picks-by-value"),
                   ("item-name-dry-run-deletes", "delete-by-item-name-picks-by-value"),
                   ("conclusions-kind-byte-ignored", "delete-conclusions-by-kind"),
                   ("conclusions-every-kind-deleted", "delete-conclusions-by-kind"),
                   ("conclusions-dry-run-deletes", "delete-conclusions-by-kind"),
                   ("conclusions-claims-ignored", "delete-conclusions-refuses-while-claims-are-live"),
                   ("conclusions-name-unchecked", "delete-conclusions-refuses-while-claims-are-live"),
                   ("conclusions-source-not-checked", "delete-conclusions-refuses-a-source-that-differs"),
                   ("conclusions-table-tag-drifted", "conclusion-kinds-match-the-verdict-store"),
                   ("conclusions-block-key-length-drifted", "conclusion-kinds-match-the-verdict-store"),
                   ("write-storage-failure-abandoned", "write-storage-failure-replayed"),
                   ("write-if-absent-sent-as-write", "gpu-pending-rows-written-only-if-absent"),
                   ("guard-refusal-exits", "gpu-conclusion-write-guarded"),
                   ("gpu-rejudged-row-dropped", "gpu-conclusion-write-guarded"),
                   ("claim-listing-prefix-widened", "gpu-claim-listing-lists-live-claims"),
                   ("claim-listing-resume-key-dropped", "gpu-claim-listing-lists-live-claims"),
                   ("temporary-service-binds-loopback", "temporary-service-binds-the-address-of-this-host"),
                   ("temporary-service-loopback-when-hosts-unreadable", "temporary-service-refuses-without-readable-hosts"),
                   ("sweep-child-runs-all-cells", "sweep-child-runs-only-named-cells"),
                   ("cells-option-ignored", "sweep-child-runs-only-named-cells"),
                   ("unknown-cell-name-accepted", "sweep-child-runs-only-named-cells"),
                   ("sweep-child-without-time-limit", "sweep-stops-hung-child"),
                   ("sweep-child-term-only", "sweep-stops-hung-child"),
                   ("sweep-group-not-signalled", "sweep-stops-hung-child"),
                   ("sweep-job-directory-kept", "sweep-stops-hung-child"),
                   ("sweep-outcome-dropped", "sweep-stops-hung-child"),
                   ("sweep-descendants-left", "sweep-stops-hung-child"),
                   ("cells-run-serially", "cells-run-in-parallel-children"),
                   ("cells-merged-by-completion", "cells-run-in-parallel-children"),
                   ("cell-result-dropped", "cells-run-in-parallel-children"),
                   ("cell-child-exit-code-ignored", "cells-run-in-parallel-children"))


def judge_cells_in_process(selected, binary):
    """selected 里的格在本进程里逐格串行跑（每格自己一个临时目录），跑完一格交一个 (格名, 是否绿, 要打的文字)。"""
    for name in selected:
        work = selftest_root()
        try:
            passed, detail = selftest_cell_functions()[name](binary, work)
        except (RuntimeError, OSError, subprocess.SubprocessError, PruneServerFailure) as error:
            passed, detail = False, f"这一格的现场出错：{type(error).__name__}：{error}"
        finally:
            shutil.rmtree(work, ignore_errors=True)
        yield name, passed, f"  {'✓' if passed else '✗'} 自证 {name}：{detail}"  # gate-lint:detail


def default_parallel_threads():
    """并行度：每台默认重活线程数（登记位 .claude/heavy-thread-default）。交 (线程数, 取不了的错误)，取得了错误是 None；不退回别的数。"""
    hosts_directory = os.path.join(REPOSITORY_ROOT, "scripts", "hosts")
    if hosts_directory not in sys.path:
        sys.path.append(hosts_directory)
    import heavy_threads
    try:
        return heavy_threads.default_heavy_threads(), None
    except heavy_threads.HeavyThreadDefaultError as error:
        return 0, error


def selftest(arguments):
    os.environ[SELFTEST_VARIABLE] = "1"
    # 三家照这个名字载入客户端：让它们用的就是这一进程里已经 import 的那一份客户端模块（同一个进程级连接，environment_for 换服务时一起丢掉）
    sys.modules.setdefault("singlefs_prune_server_client", sys.modules["prune_server_client"])
    selected, refusal = parse_selftest_options(arguments)
    if selected is None:
        print(f"  ✗ 自证的 --cells 用法不对：{refusal}")
        print("     → 怎么办：--cells 后面写 <格名>[,<格名>…]，格名取 ✗ 行里列出的可用格名；要走全部格就不给 --cells")
        return EXIT_USAGE
    hosts_exit_code = hosts_refusal(sys.stdout)
    if hosts_exit_code is not None:
        return hosts_exit_code
    binary, problem = resolve_binary(arguments)
    if binary is None or not os.access(binary, os.X_OK):
        print(f"  ✗ 自证起不了服务：{problem or binary + ' 不可执行'}")
        print(f"     → 怎么办：--server-binary <编好的 prune-server 二进制> 或 {BINARY_VARIABLE}=<路径>；只有任务服务给了 {CARGO_TARGET_DIRECTORY_VARIABLE} 才在这里现编")
        return EXIT_USAGE
    if "--cells" in arguments:
        results = judge_cells_in_process(selected, binary)
    else:
        threads, error = default_parallel_threads()
        if error is not None:
            print(f"  ✗ 自证各格并行取不了每台默认重活线程数：{error}")
            print(f"     → 怎么办：{error.howto}；不退回别的数")
            return EXIT_USAGE
        print(f"  · 各格并行：{len(selected)} 格各在一个子进程里并行 {max(1, min(len(selected), threads))} 路，每格限时 {SWEEP_CHILD_SECONDS} 秒，结果按格的次序逐格判")
        results = judge_cells_in_children(selected, lambda name: cell_child_command(binary, name), threads, SWEEP_CHILD_SECONDS, SWEEP_STOP_GRACE_SECONDS,
                                          secrets.token_hex(8))
    failed, judged = [], []
    try:
        for name, passed, text in results:
            judged.append(name)
            if not passed:
                failed.append(name)
            print(text, flush=True)  # gate-lint:detail
    except (RuntimeError, OSError, subprocess.SubprocessError) as error:
        failed.extend(name for name in selected if name not in judged)
        print(f"  ✗ 自证各格的子进程没收齐：{error}")
        print("     → 怎么办：没收回结果的格按红算；单独跑那几格看它们的子进程为什么没交结果（--cells <格名>）")
    print(f"PRUNE_SERVER_CLIENT_SELFTEST cells={len(selected)} passed={len(selected) - len(failed)} failed={len(failed)} failed_cells={','.join(failed) or '-'}")
    if failed:
        return 1
    if os.environ.get(BREAK_VARIABLE):
        return 0
    if "--cells" in arguments:
        print("  · 只走了点名的格：弄坏开关轮本次未跑")
        return 0
    return break_sweep(binary)


def break_sweep(binary):
    """弄坏开关轮：每个开关一个子进程，只走它在 BREAK_RED_CELLS 里点名的红格，并行、各带时限（文件头「弄坏开关轮」），打末行 PRUNE_SERVER_CLIENT_BREAK_SWEEP；
    全部对上退 0，有没对上的退 1，取不了并行度退 2。"""
    cells_by_switch = {}
    for switch, red_cell in BREAK_RED_CELLS:
        cells_by_switch.setdefault(switch, []).append(red_cell)
    jobs = [SweepJob(switch, tuple(cells), sweep_child_command(binary, cells), dict(os.environ, **{BREAK_VARIABLE: switch}))
            for switch, cells in cells_by_switch.items()]
    # 并行度：每台默认重活线程数，再按开关条数夹；取不了就拒
    threads, error = default_parallel_threads()
    if error is not None:
        print(f"  ✗ 弄坏开关轮取不了每台默认重活线程数：{error}")
        print(f"     → 怎么办：{error.howto}；不退回别的数")
        return EXIT_USAGE
    print(f"  · 弄坏开关轮：{len(jobs)} 个开关并行 {max(1, min(len(jobs), threads))} 路，每个开关限时 {SWEEP_CHILD_SECONDS} 秒")
    outcomes = run_break_sweep(jobs, threads, SWEEP_CHILD_SECONDS, SWEEP_STOP_GRACE_SECONDS, secrets.token_hex(8))
    unmatched = [f"{outcome.name}（{outcome.detail}）" for outcome in outcomes if not outcome.matched]
    print(f"PRUNE_SERVER_CLIENT_BREAK_SWEEP switches={len(outcomes)} matched={len(outcomes) - len(unmatched)} unmatched={'；'.join(unmatched) or '-'}")
    if unmatched:
        print("  ✗ 弄坏开关打开之后没红在点名那一格")
        print("     → 怎么办：那几个开关管的那一段没有自证盯着：补一格会被它弄错的自证，或修开关；写「超时」的是子进程卡住了，看那个开关点名的格为什么不退出")
        return 1
    return 0


def main(arguments):
    if arguments[:1] == ["--selftest"]:
        return selftest(arguments[1:])
    if arguments[:1] == ["--selftest-families"]:
        return selftest_families(arguments[1:])
    if arguments[:1] == ["--with-temporary-service"]:
        return with_temporary_service(arguments[1:])
    if arguments[:1] == ["--selftest-child"]:
        return selftest_child(arguments[1:])
    print(f"  ✗ 认不出的用法：{' '.join(arguments) or '（没给参数）'}", file=sys.stderr)
    print("     → 怎么办：--selftest [--server-binary <…>] [--cells <格名>[,<格名>…]]、--selftest-families [--server-binary <…>] 或 "
          "--with-temporary-service [--server-binary <…>] -- <命令…>", file=sys.stderr)
    return EXIT_USAGE


if __name__ == "__main__":
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
