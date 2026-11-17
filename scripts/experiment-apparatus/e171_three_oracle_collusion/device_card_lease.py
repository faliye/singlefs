"""装置那一步按需领卡（task-server/task_server.py 文件头 ②「按需领卡」）：入口文件头写「task-server-card-lease: on-demand」的任务，服务起跑不分卡、不清场，
跑到装置上卡那一步经领卡客户端 `task-server/task lease take` 领一张、那一步跑完 `task-server/task lease return <卡串>` 还掉；编译、同步、执行器与比对器
这些不上卡的步骤进行时一张卡都不持。E171（三oracle合谋测试）逐行驱动 row_driver.py 与 E201（三oracle合谋今树整表）入口 full_table_run.py 共用这一份。
几种情形怎么办照 scripts/experiment-apparatus/replay.sh 的 replay_card_lease_channel_present、replay_card_lease_take、replay_card_lease_return：
  没有领卡通道（环境里 TASK_SERVER_CARD_LEASE_DIRECTORY 不是在的目录，或没有 TASK_SERVER_CARD_LEASE_TOKEN；不是服务照按需领卡起的）：开跑前一次说清
      （channel_problem、report_missing_channel），调用方退 EXIT_NO_CHANNEL（78），一个装置都不起；
  领卡没成（客户端退非 0，或退 0 却没打卡串）：take_card 抛 CardLeaseRefused（带客户端的退出码），调用方不起装置，记测量管子的缺陷，不当成这一步的判定；
  还卡没成（客户端退非 0；120 秒没回执退 1）：return_card 往标准错误打一行「!」点名退出码，不改装置那一步的结局（尝试结束时服务照样收回没还的卡）。
同一次尝试里几条线道同时要卡各领各的：领一回服务给一张，还卡照领到的那一串只还那一张，不碰别的线道持着的。客户端在等卡时往标准错误打的行照原样留在调用方的标准错误里。
客户端的标准输出领卡那一回收成卡串（take_card）、还卡那一回转到标准错误（return_card），都不进调用方的标准输出：scripts/gate/lib_function_units.py 因此不把
统一任务服务的代码收进 E171（三oracle合谋测试）行键（UNKEYED_PROGRAM_WORDS 里这两个函数的那两行，三格自证 CARD_LEASE_CLIENT_WORDS_*_CELL）。
客户端是起这一趟的那棵树里的 task-server/task（client_of），在那棵树的根上起。
装置分录入、判两段起（record_then_judge_on_a_leased_card；装置的点段参数 --stage record|judge，crates/singlefs-verification/src/bin/e171_three_oracle_collusion.rs）：
  录入段只在 CPU 上录、把要判的盘写进判定库，不领卡，环境里不给卡串（environment_of_the_stage 去掉 GPU_CARDS_BY_PRIORITY）；
  录入段退非 0：不领卡、判段不起，这一步的结局就是录入段的退出码；录入段退 0 才领一张卡（领卡没成同上：take 抛 CardLeaseRefused，判段不起），
  判段带这张卡起，判段结束照卡串还掉（判段退非 0、抛异常照还）。几条线道的录入段可以同时跑，判段各领各的。
  判段的参数与录入段逐字相同、只换 --stage 后面那个词（stage_arguments 接在参数末尾；装置核交接里录下的参数，对不上退 2）。
  弄坏项（调用方从自己的弄坏开关变量里认、经 broken 交进来；打开后哪一格红写在各调用方文件头）：card-held-through-the-record-stage 录入段之前就领卡、
  把卡串交给录入段；cards-left-in-the-record-environment 录入段的环境里留着调用方环境的卡串；card-returned-before-the-judge-stage 领了卡先还、再起判段；
  judge-stage-after-a-failed-record 录入段退非 0 照领卡起判段；card-kept-after-a-failed-judge 判段退非 0 不还卡。
"""

from __future__ import annotations

import os
import subprocess
import sys
from collections.abc import Callable

CHANNEL_DIRECTORY_VARIABLE = "TASK_SERVER_CARD_LEASE_DIRECTORY"   # task-server/task_server_lib.py 的 CARD_LEASE_DIRECTORY_VARIABLE
CHANNEL_TOKEN_VARIABLE = "TASK_SERVER_CARD_LEASE_TOKEN"   # task-server/task_server_lib.py 的 CARD_LEASE_TOKEN_VARIABLE
CARDS_VARIABLE = "GPU_CARDS_BY_PRIORITY"   # 装置照它取卡；领到的卡串与它同格式
EXIT_NO_CHANNEL = 78
CLIENT_RELATIVE = os.path.join("task-server", "task")
# 装置点段（文件头「装置分录入、判两段起」）：--stage 的两个取值与弄坏项的名字
STAGE_OPTION = "--stage"
RECORD_STAGE = "record"
JUDGE_STAGE = "judge"
CARD_HELD_THROUGH_THE_RECORD_STAGE = "card-held-through-the-record-stage"
CARDS_LEFT_IN_THE_RECORD_ENVIRONMENT = "cards-left-in-the-record-environment"
CARD_RETURNED_BEFORE_THE_JUDGE_STAGE = "card-returned-before-the-judge-stage"
JUDGE_STAGE_AFTER_A_FAILED_RECORD = "judge-stage-after-a-failed-record"
CARD_KEPT_AFTER_A_FAILED_JUDGE = "card-kept-after-a-failed-judge"


class CardLeaseRefused(Exception):
    """领卡没成：`exit_code` 是客户端的退出码（退 0 却没打卡串的记 1）。"""

    def __init__(self, step: str, exit_code: int) -> None:
        super().__init__(f"{step}:take_exit={exit_code}")
        self.step = step
        self.exit_code = exit_code


def client_of(tree_root: str) -> str:
    return os.path.join(tree_root, CLIENT_RELATIVE)


def channel_problem(environment: dict[str, str] | None = None) -> str | None:
    """环境里有没有领卡通道：有交 None，没有交一句点名缺了哪一样。"""
    environment = os.environ if environment is None else environment
    directory = environment.get(CHANNEL_DIRECTORY_VARIABLE, "")
    if not os.path.isdir(directory):
        return f"{CHANNEL_DIRECTORY_VARIABLE} 不是在的目录（{directory or '没设'}）"
    if not environment.get(CHANNEL_TOKEN_VARIABLE):
        return f"没有 {CHANNEL_TOKEN_VARIABLE}"
    return None


def report_missing_channel(who: str, problem: str, entry: str) -> None:
    """没有领卡通道的那一句与出路（标准错误）。`entry` 是该整条经服务交的那一条入口。"""
    print(f"  ✗ {who}: 装置那一步要按需领卡，环境里却没有领卡通道：{problem}；一个装置都没起", file=sys.stderr)
    print(f"  → 怎么办：整条经 task-server/task submit -- {entry} 交（入口文件头的按需领卡声明让服务给这一次尝试建领卡通道），"
          "命令里不套清场包装、不写卡", file=sys.stderr)


def take_card(client: str, tree_root: str, step: str) -> str:
    """领一张卡：交回卡串；领卡没成抛 CardLeaseRefused。客户端的标准错误不接管（在等卡的行照原样留在调用方的标准错误里）。"""
    completed = subprocess.run([client, "lease", "take"], cwd=tree_root, stdout=subprocess.PIPE, text=True)
    cards = completed.stdout.replace("\n", "").strip()
    if completed.returncode == 0 and cards:
        print(f"  · 领卡：{step} 领到 {cards}", file=sys.stderr, flush=True)
        return cards
    exit_code = completed.returncode or 1
    print(f"  ✗ 领卡没成：{step}（task-server/task lease take 退 {exit_code}，原因在上面客户端那一句），装置没起", file=sys.stderr, flush=True)
    print("  → 怎么办：照客户端那一句的出路办（不在按需领卡的尝试里就经 task-server/task submit 整条重交）；这一步记测量管子的缺陷，重交同一条命令只补没做完的",
          file=sys.stderr, flush=True)
    raise CardLeaseRefused(step, exit_code)


def return_card(client: str, tree_root: str, cards: str, step: str, report_failure: bool = True) -> int:
    """照卡串还掉这一张（别的线道持着的不动）。交回客户端的退出码；没成只打一行「!」，不抛。"""
    completed = subprocess.run([client, "lease", "return", cards], cwd=tree_root, stdout=subprocess.PIPE)
    if completed.stdout:
        sys.stderr.write(completed.stdout.decode("utf-8", "replace"))
        sys.stderr.flush()
    exit_code = completed.returncode
    if exit_code == 0:
        print(f"  · 还卡：{step} 还了 {cards}", file=sys.stderr, flush=True)
    elif report_failure:
        print(f"  ! 还卡没成：{step} 的 {cards}（task-server/task lease return 退 {exit_code}，原因在上面客户端那一句），到这一次尝试结束由服务收回；"
              "这一步照已跑的算", file=sys.stderr, flush=True)
    return exit_code


def never_broken(item: str) -> bool:
    """不认任何弄坏项（平常跑的调用方交这一个）。"""
    return False


def stage_arguments(stage: str) -> list[str]:
    """接在装置参数末尾、点这一段的两个词。"""
    return [STAGE_OPTION, stage]


def environment_of_the_stage(environment: dict[str, str], stage: str | None, cards: str | None,
                             broken: Callable[[str], bool] = never_broken) -> dict[str, str]:
    """一段装置的环境：`environment` 的一份拷贝；录入段去掉 GPU_CARDS_BY_PRIORITY，给了 `cards` 就把它放进 GPU_CARDS_BY_PRIORITY。"""
    staged = dict(environment)
    if stage == RECORD_STAGE and not broken(CARDS_LEFT_IN_THE_RECORD_ENVIRONMENT):
        staged.pop(CARDS_VARIABLE, None)
    if cards is not None:
        staged[CARDS_VARIABLE] = cards
    return staged


def record_then_judge_on_a_leased_card(run_stage: Callable[[str, str | None], int], take: Callable[[], str], give_back: Callable[[str], None],
                                       broken: Callable[[str], bool] = never_broken) -> int:
    """装置那一步分两段起（文件头「装置分录入、判两段起」）：run_stage(段, 卡串或 None) 起那一段、交它的退出码；take() 领一张卡、交卡串
    （领卡没成抛 CardLeaseRefused，判段不起）；give_back(卡串) 还掉那一张。交这一步的退出码：录入段退非 0 是录入段的，否则是判段的。"""
    held = take() if broken(CARD_HELD_THROUGH_THE_RECORD_STAGE) else None
    record_exit = run_stage(RECORD_STAGE, held)
    if record_exit != 0 and not broken(JUDGE_STAGE_AFTER_A_FAILED_RECORD):
        if held is not None:
            give_back(held)
        return record_exit
    cards = take() if held is None else held
    if broken(CARD_RETURNED_BEFORE_THE_JUDGE_STAGE):
        give_back(cards)
        return run_stage(JUDGE_STAGE, cards)
    judge_exit = None
    try:
        judge_exit = run_stage(JUDGE_STAGE, cards)
    finally:
        if judge_exit == 0 or not broken(CARD_KEPT_AFTER_A_FAILED_JUDGE):
            give_back(cards)
    return judge_exit
