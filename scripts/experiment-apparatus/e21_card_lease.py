#!/usr/bin/env python3
# admission: always 领卡每次照服务此刻交的卡现领，上一次领到的卡不替这一次作保
# run-condition: none 被 e21-compute.py、e21-prove-gpu.py、e21-transfer.py 载入的库，自己不上卡、不导入 torch；直接执行只打用法
"""E21（显存内算子的带宽与传输段）三份 torch 脚本（e21-compute.py、e21-prove-gpu.py、e21-transfer.py）共用的按需领卡。

三份脚本文件头都写 `# task-server-card-lease: on-demand`（task-server/task_server.py 文件头 ②「按需领卡」）：服务起跑不分卡、不清场，
只给这一次尝试建领卡通道；脚本在碰 CUDA 之前经客户端 task-server/task lease take 领一张，领到的卡串是 `<台地址>:<nvidia-smi 序号>:<额度>`
（与 GPU_CARDS_BY_PRIORITY 同格式）。上卡的那一段跑在 fork 出来的子进程里：子进程先把 CUDA_DEVICE_ORDER 设成 PCI_BUS_ID（让 CUDA 的序号与
nvidia-smi 的序号是同一套）、CUDA_VISIBLE_DEVICES 设成领到的那个序号，torch 里就只看得到那一张（序号 VISIBLE_DEVICE_INDEX）；
子进程退出（连同它的 CUDA 上下文与显存）之后父进程才 task-server/task lease return <卡串> 还掉，所以还卡的那一刻这一进程已经不占卡。
父进程自己不碰 CUDA，只领卡、等子进程、还卡。

用法（三份脚本里）：
    import e21_card_lease
    sys.exit(e21_card_lease.run_on_leased_card(main))     # main(card) 在领到的卡上跑，card 是 LeasedCard；它写标准输出的产物内容与格式不归本库管

几种情形（照 scripts/experiment-apparatus/replay.sh 的 replay_card_lease_channel_present、replay_card_lease_take、replay_card_lease_return）：
  没有领卡通道（TASK_SERVER_CARD_LEASE_DIRECTORY 不是在的目录，或没有 TASK_SERVER_CARD_LEASE_TOKEN；变量名照 task-server/task_server_lib.py 的
      CARD_LEASE_DIRECTORY_VARIABLE、CARD_LEASE_TOKEN_VARIABLE）：退 78，标准错误写原因与出路，一条产物都不写；
  领卡没成（客户端退非 0，或退 0 却没打卡串，或领到的卡串不是一张 `<台>:<序号>:<额度>`）：退 1，标准错误写客户端的退出码与出路，标准输出一个字不写（不写产物）；
  还卡没成（客户端退非 0；120 秒没回执退 1）：标准错误打一行「!」点名退出码，不改这一次的退出码（尝试结束时服务照样收回没还的卡）；
  跑完、出错（子进程退非 0、抛异常）、被 TERM / INT 打断：都还卡，各一次；打断时父进程把 TERM 转给自己的子进程、等它退了再还；
  环境里原有的 GPU_CARDS_BY_PRIORITY 一进来就清掉：卡只从领卡来。
退出码：0 成；子进程的退出码原样（脚本自己的 2、4 与异常的 1）；被信号打断是 128 加信号号；78 没有领卡通道；1 领卡没成。

领到的额度（卡串第三段）不在这里强制：脚本自己核显存余量（e21-compute.py 4 GiB、e21-transfer.py 3 GiB）。
客户端是起这一趟的那棵树里的 task-server/task（本文件往上三级是树根），在树根上起。

弄坏开关 E21_CARD_LEASE_BREAK=<项>（逗号分隔，只给 e21-card-lease-selftest.py 的开关轮用：环境里有自证设的 E21_CARD_LEASE_SELFTEST_MARK 才认，
平常跑设了（含单独起自证的 --selftest）本文件与自证都在一切之前退 2、点名变量只在自证里认；自证文件头写着同样的一张表，由自证核两张一致）。
逐项写开关、弄坏哪一处，下一行起写打开后自证红的格（失败行「判错的样本：<格名>（」里的格名）：
  E21_CARD_LEASE_BREAK=declaration-removed  自证拷出来的三份脚本里去掉文件头 `# task-server-card-lease: on-demand` 那一行（由自证自己弄坏，库不读）
      header-declares-on-demand
  E21_CARD_LEASE_BREAK=card-return-skipped  不调还卡
      card-taken-once-and-returned-once
  E21_CARD_LEASE_BREAK=card-returned-twice  还卡调两回
      card-taken-once-and-returned-once
  E21_CARD_LEASE_BREAK=card-returned-before-the-card-process-exits  还卡挪到等子进程之前（子进程还在占卡就还）
      card-returned-after-the-card-process-exits
  E21_CARD_LEASE_BREAK=card-kept-after-error  子进程退非 0 时不还
      card-returned-after-an-error
  E21_CARD_LEASE_BREAK=card-kept-after-interrupt  被 TERM 打断时不还
      card-returned-after-an-interrupt
  E21_CARD_LEASE_BREAK=channel-unchecked  不核领卡通道，直接去领
      no-channel-exits-78
  E21_CARD_LEASE_BREAK=refusal-ignored  领卡被拒也照起子进程
      refused-lease-runs-nothing
  E21_CARD_LEASE_BREAK=empty-card-accepted  退 0 却没打卡串的也当领到了
      empty-card-string-counts-as-refused
  E21_CARD_LEASE_BREAK=unparsed-card-run  领到的卡串认不出序号也照起子进程
      unparseable-card-string-is-returned-and-fails
  E21_CARD_LEASE_BREAK=all-cards-visible  不设 CUDA_VISIBLE_DEVICES（所有卡都看得到）
      only-the-leased-card-visible
  E21_CARD_LEASE_BREAK=device-order-unset  不设 CUDA_DEVICE_ORDER（CUDA 序号与 nvidia-smi 序号不是同一套）
      only-the-leased-card-visible
  E21_CARD_LEASE_BREAK=inherited-cards-kept  不清环境里原有的 GPU_CARDS_BY_PRIORITY
      inherited-card-variable-cleared
  E21_CARD_LEASE_BREAK=return-failure-changes-exit  还卡没成时把退出码改成 1
      failed-return-only-warns
  E21_CARD_LEASE_BREAK=return-failure-unreported  还卡没成不打「!」那一行
      failed-return-only-warns
"""
import os
import signal
import subprocess
import sys
import traceback
from typing import Callable, NamedTuple
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

REPOSITORY_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
CLIENT_RELATIVE = os.path.join("task-server", "task")
CHANNEL_DIRECTORY_VARIABLE = "TASK_SERVER_CARD_LEASE_DIRECTORY"   # task-server/task_server_lib.py 的 CARD_LEASE_DIRECTORY_VARIABLE
CHANNEL_TOKEN_VARIABLE = "TASK_SERVER_CARD_LEASE_TOKEN"   # task-server/task_server_lib.py 的 CARD_LEASE_TOKEN_VARIABLE
CARDS_VARIABLE = "GPU_CARDS_BY_PRIORITY"
EXIT_NO_CHANNEL = 78
EXIT_LEASE_FAILED = 1
VISIBLE_DEVICE_INDEX = 0   # CUDA_VISIBLE_DEVICES 只留一张之后，torch 里那一张的序号
BREAK_VARIABLE = "E21_CARD_LEASE_BREAK"
# 弄坏开关只在自证里认：自证设这个标记（起的子进程环境原样继承）才认开关，外面漏进来的开关改不了脚本的跑法
SELFTEST_MARK_VARIABLE = "E21_CARD_LEASE_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "e21-card-lease-selftest"
INNER_ARGUMENT = "--selftest-inner"   # 自证的开关轮起的子进程带的参数：只有它放行弄坏开关


class LeasedCard(NamedTuple):
    cards: str   # 领到的卡串（还卡照它还）
    index: int   # 卡串里的 nvidia-smi 序号


class CardLeaseRefused(Exception):
    """领卡没成：exit_code 是客户端的退出码（退 0 却没打卡串的记 1）。"""

    def __init__(self, exit_code: int) -> None:
        super().__init__(f"lease take 退 {exit_code}")
        self.exit_code = exit_code


def broken(item: str) -> bool:
    if os.environ.get(SELFTEST_MARK_VARIABLE) != SELFTEST_MARK_CONTENT:
        return False
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def channel_problem() -> str | None:
    """环境里有没有领卡通道：有交 None，没有交一句点名缺了哪一样。"""
    directory = os.environ.get(CHANNEL_DIRECTORY_VARIABLE, "")
    if not os.path.isdir(directory):
        return f"{CHANNEL_DIRECTORY_VARIABLE} 不是在的目录（{directory or '没设'}）"
    if not os.environ.get(CHANNEL_TOKEN_VARIABLE):
        return f"没有 {CHANNEL_TOKEN_VARIABLE}"
    return None


def script_name() -> str:
    return os.path.basename(sys.argv[0]) or "e21 脚本"


def take_card(client: str, root: str) -> str:
    """领一张卡：交回卡串；没领到抛 CardLeaseRefused。客户端在等卡时往标准错误打的行照原样留在这一趟的标准错误里。"""
    try:
        completed = subprocess.run([client, "lease", "take"], cwd=root, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, text=True)
        exit_code, cards = completed.returncode, " ".join(completed.stdout.split())
    except OSError as error:
        print(f"  ! 领卡客户端 {client} 起不来：{error}", file=sys.stderr, flush=True)
        exit_code, cards = EXIT_LEASE_FAILED, ""
    if exit_code == 0 and not cards and broken("empty-card-accepted"):
        return cards
    if exit_code == 0 and cards:
        return cards
    exit_code = exit_code or EXIT_LEASE_FAILED
    print(f"  ✗ {script_name()}：领卡没成（task-server/task lease take 退 {exit_code}，原因在上面客户端那一句），一个产物都没写", file=sys.stderr, flush=True)
    print("  → 怎么办：照客户端那一句的出路办（不在按需领卡的尝试里就经 task-server/task submit -- python3 <这个脚本> 整条重交；"
          "这一次尝试已持着卡说明前一回没还）", file=sys.stderr, flush=True)
    raise CardLeaseRefused(exit_code)


def parse_card(cards: str) -> LeasedCard:
    """卡串必须是一张 `<台地址>:<序号>:<额度>`（地址可以带冒号，从右边切两刀）；不是抛 ValueError，写明哪里不对。"""
    items = cards.split()
    if len(items) != 1:
        raise ValueError(f"领到的卡串「{cards}」不是一张卡（要恰好一项）")
    pieces = items[0].rsplit(":", 2)
    if len(pieces) != 3 or not pieces[0] or not pieces[1].isdigit() or not pieces[2]:
        raise ValueError(f"领到的卡串「{cards}」不是 <台地址>:<nvidia-smi 序号>:<额度> 的形状")
    return LeasedCard(cards, int(pieces[1]))


def return_card(client: str, root: str, cards: str) -> int:
    """照卡串还掉这一张：交回客户端的退出码；没成只打一行「!」，不抛。客户端的标准输出转到标准错误，不进产物。"""
    if broken("card-return-skipped"):
        return 0
    try:
        completed = subprocess.run([client, "lease", "return", cards], cwd=root, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE)
        exit_code = completed.returncode
        sys.stderr.write(completed.stdout.decode("utf-8", "replace"))
        sys.stderr.flush()
    except OSError as error:
        print(f"  ! 领卡客户端 {client} 起不来：{error}", file=sys.stderr, flush=True)
        exit_code = EXIT_LEASE_FAILED
    if exit_code == 0:
        print(f"  · 还卡：{cards} 还了", file=sys.stderr, flush=True)
    elif not broken("return-failure-unreported"):
        print(f"  ! 还卡没成：task-server/task lease return 退 {exit_code}（原因在上面客户端那一句），卡 {cards} 到这一次尝试结束由服务收回；"
              "这一次的产物照已跑的算", file=sys.stderr, flush=True)
    return exit_code


def exit_status_of(code: object) -> int:
    """SystemExit 带的值换成退出码：没带是 0，整数照给，别的（一句话）打到标准错误、退 1。"""
    if code is None:
        return 0
    if isinstance(code, int):
        return code & 0xFF
    print(code, file=sys.stderr)
    return 1


def run_work_in_child(work: Callable[[LeasedCard], object], card: LeasedCard, outer_signal_mask: set[int]) -> None:
    """fork 出来的子进程：恢复信号默认处理、把 CUDA 可见的卡收成领到的那一张，跑 work，flush 之后 os._exit，不回到调用方。"""
    exit_status = 1
    try:
        signal.signal(signal.SIGTERM, signal.SIG_DFL)
        signal.signal(signal.SIGINT, signal.default_int_handler)
        signal.pthread_sigmask(signal.SIG_SETMASK, outer_signal_mask)
        if not broken("device-order-unset"):
            os.environ["CUDA_DEVICE_ORDER"] = "PCI_BUS_ID"
        if broken("all-cards-visible"):
            os.environ.pop("CUDA_VISIBLE_DEVICES", None)
        else:
            os.environ["CUDA_VISIBLE_DEVICES"] = str(card.index)
        work(card)
        exit_status = 0
    except SystemExit as stop:
        exit_status = exit_status_of(stop.code)
    except KeyboardInterrupt:
        exit_status = 128 + signal.SIGINT
    except BaseException:
        traceback.print_exc()
        exit_status = 1
    finally:
        for stream in (sys.stdout, sys.stderr):
            try:
                stream.flush()
            except OSError:
                pass
        os._exit(exit_status)


def exit_status_after_wait(wait_status: int) -> int:
    exit_code = os.waitstatus_to_exitcode(wait_status)
    return 128 - exit_code if exit_code < 0 else exit_code


def run_in_child_and_give_card_back(work: Callable[[LeasedCard], object], card: LeasedCard, client: str, root: str) -> int:
    """持着 card 起子进程跑 work、等它退出、再还卡：跑完、出错、被打断（TERM、INT 转给子进程）都还，各一次。交回这一次的退出码。"""
    interrupted: list[int] = []
    child_reaped = False
    interrupt_signals = {signal.SIGTERM, signal.SIGINT}
    outer_signal_mask = signal.pthread_sigmask(signal.SIG_BLOCK, interrupt_signals)   # fork 到装好转发之间收到的信号先挂着
    sys.stdout.flush()
    sys.stderr.flush()
    child = os.fork()
    if child == 0:
        run_work_in_child(work, card, outer_signal_mask)

    def forward_to_child(signal_number: int, _frame: object) -> None:
        interrupted.append(signal_number)
        if not child_reaped:
            os.kill(child, signal.SIGTERM)

    previous_handlers = {number: signal.signal(number, forward_to_child) for number in interrupt_signals}
    signal.pthread_sigmask(signal.SIG_SETMASK, outer_signal_mask)
    exit_status = 1
    returned_early = broken("card-returned-before-the-card-process-exits")
    try:
        if returned_early:
            return_card(client, root, card.cards)
        wait_status = os.waitpid(child, 0)[1]
        child_reaped = True
        exit_status = 128 + interrupted[0] if interrupted else exit_status_after_wait(wait_status)
    finally:
        for number, handler in previous_handlers.items():
            signal.signal(number, handler)
        keeps_the_card = (broken("card-kept-after-interrupt") and bool(interrupted)) or (broken("card-kept-after-error") and exit_status != 0 and not interrupted)
        return_failed = False
        if card.cards and not returned_early and not keeps_the_card:
            return_failed = return_card(client, root, card.cards) != 0
            if broken("card-returned-twice"):
                return_card(client, root, card.cards)
    if return_failed and broken("return-failure-changes-exit") and exit_status == 0:
        exit_status = 1
    return exit_status


def run_on_leased_card(work: Callable[[LeasedCard], object], root: str = REPOSITORY_ROOT) -> int:
    """三份脚本的入口：领一张卡、在那一张卡上跑 work、还卡；交回退出码（文件头「几种情形」）。work 在子进程里跑，可以直接 sys.exit。"""
    if not broken("inherited-cards-kept"):
        os.environ.pop(CARDS_VARIABLE, None)
    problem = None if broken("channel-unchecked") else channel_problem()
    if problem is not None:
        print(f"  ✗ {script_name()}：要按需领卡，环境里却没有领卡通道：{problem}；一个产物都没写", file=sys.stderr)
        print(f"  → 怎么办：整条经 task-server/task submit -- python3 scripts/experiment-apparatus/{script_name()} 交"
              "（文件头 `# task-server-card-lease: on-demand` 让服务给这一次尝试建领卡通道，上卡那一段脚本自己领、自己还）；命令里不套清场包装、不写卡",
              file=sys.stderr)
        return EXIT_NO_CHANNEL
    client = os.path.join(root, CLIENT_RELATIVE)
    try:
        cards = take_card(client, root)
    except CardLeaseRefused:
        if not broken("refusal-ignored"):
            return EXIT_LEASE_FAILED
        cards = ""
    try:
        card = parse_card(cards)
    except ValueError as unparseable:
        if not (broken("refusal-ignored") or broken("empty-card-accepted") or broken("unparsed-card-run")):
            print(f"  ✗ {script_name()}：{unparseable}，一个产物都没写", file=sys.stderr)
            print("  → 怎么办：服务给的卡串形状变了，查 task-server/task lease take 打的那一行；这一张已照原串还掉", file=sys.stderr)
            if cards:
                return_card(client, root, cards)
            return EXIT_LEASE_FAILED
        card = LeasedCard(cards, VISIBLE_DEVICE_INDEX)
    print(f"  · 领卡：{script_name()} 领到 {card.cards or '（没有）'}，跑完就还", file=sys.stderr, flush=True)
    return run_in_child_and_give_card_back(work, card, client, root)


def refuse_break_outside_selftest(arguments: list[str]) -> int | None:
    """弄坏开关只在自证的开关轮里认：命令行不是单独一个 --selftest-inner（e21-card-lease-selftest.py 的 --selftest 逐个设上开关起的那一趟）、
    环境里也没有自证留的标记而 E21_CARD_LEASE_BREAK 非空，参数解析之前就拒，交回 2；放行交回 None。"""
    value = os.environ.get(BREAK_VARIABLE, "")
    if not value or arguments == [INNER_ARGUMENT] or os.environ.get(SELFTEST_MARK_VARIABLE) == SELFTEST_MARK_CONTENT:
        return None
    print(f"  ✗ {BREAK_VARIABLE}={value} 是弄坏开关，只在自证（e21-card-lease-selftest.py）的开关轮里认，平常跑设了（含单独起它的 --selftest）就拒，这一趟什么都没做", file=sys.stderr)
    print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再跑；要逐个证红就跑 python3 scripts/experiment-apparatus/e21-card-lease-selftest.py --selftest（它自己逐项设上开关），"
          f"单跑一项是 {BREAK_VARIABLE}=<项> python3 scripts/experiment-apparatus/e21-card-lease-selftest.py {INNER_ARGUMENT}",
          file=sys.stderr)
    return 2


def main(arguments: list[str]) -> int:
    refused = refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    print("用法：本文件是 e21-compute.py、e21-prove-gpu.py、e21-transfer.py 载入的库，不单独跑；自证是 python3 scripts/experiment-apparatus/e21-card-lease-selftest.py --selftest",
          file=sys.stderr)
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
