"""E201（三oracle合谋今树整表） 的装置那几步按需领卡（full_table_run.py 文件头「卡」）：只在起装置的那一步前领一张卡、交给装置的 GPU_CARDS_BY_PRIORITY，
那一步跑完照卡串还掉；同步、编译、E170 导出装置、执行器、O3 对照与比对器那几步一张卡都不持。领还的写法与几种情形（没有通道、领卡没成、还卡没成）在
E171（三oracle合谋测试） 研究一侧的 device_card_lease.py 文件头，这里按路径载入它（换一个不与 E171 两个目录里的模块撞名的名字）。

两处用它：
  full_table_run.py 的 run_device（各片自己起的基线、第二次基线、装置对照）经 run_on_a_leased_card；
  逐行驱动 current_tree_row_driver.py 经本脚本起：
    python3 -B device_steps_on_leased_cards.py <current_tree_row_driver.py 的参数…>
  开跑前核领卡通道（refuse_without_a_channel；没有退 78、一行都不跑），再把驱动的 run_with_time_limit（那份文件里只有 run_device 起装置时调它）换成
  device_steps_on_leased_cards 里那一层：每一回装置那一步各领各的卡（几条线道同时到装置那一步各持一张），领卡没成抛驱动自己的 MeasurementPipeDefect（`card_lease`，
  装置没起，那一行不写 row-result.tsv、各线道不再领新行、驱动退 5）。不改 current_tree_row_driver.py：它的 run_row_on_the_lane 是每行判定键里的判法
  （full_table_run.py 文件头「判定库」的 judge 一项），改了那个判法闭包里的单元，闭包含它的行键都变。
弄坏开关读 E201_FULL_TABLE_BREAK（只在 full_table_run.py 的自证里认，命令行设了退 2）。本文件认的是按需领卡那六项，开关表与 full_table_run.py 文件头「按需领卡的几处」相同，
打开后红在 full_table_run.py 自证的这几格：
  E201_FULL_TABLE_BREAK=card-held-through-the-compile  逐行驱动一起来就领一张、整趟持着（编译那几步也持卡）
      自证项 「✗ card-held-only-around-each-device-step」「✗ lanes-hold-different-cards」「✗ refused-card-lease-starts-no-device」（两条线道拿的是同一张；拒领卡时一进来就抛）
  E201_FULL_TABLE_BREAK=card-kept-after-the-device  装置那一步跑完不还卡
      自证项 「✗ card-held-only-around-each-device-step」「✗ lanes-hold-different-cards」「✗ failed-card-return-is-reported」（不还就没有还卡那一句）
  E201_FULL_TABLE_BREAK=lanes-share-one-card  几条线道同时到装置那一步，装置都拿最先领到的那一张
      自证项 「✗ lanes-hold-different-cards」「✗ card-held-only-around-each-device-step」（一条线道的后一步也拿到最先那一张）
  E201_FULL_TABLE_BREAK=card-lease-refusal-runs-the-device  领卡没成照起装置（不带卡）
      自证项 「✗ refused-card-lease-starts-no-device」
  E201_FULL_TABLE_BREAK=card-lease-channel-unchecked  开跑前不核领卡通道
      自证项 「✗ missing-lease-channel-exits-78」
  E201_FULL_TABLE_BREAK=card-return-failure-unreported  还卡没成不打那一行「!」
      自证项 「✗ failed-card-return-is-reported」
"""

from __future__ import annotations

import contextlib
import importlib.util
import os
import sys
from collections.abc import Callable, Iterator

HERE = os.path.dirname(os.path.abspath(__file__))
if HERE not in sys.path:
    sys.path.insert(0, HERE)
import e171_research_side  # noqa: E402

REPOSITORY_ROOT = e171_research_side.REPOSITORY_ROOT
DEVICE_CARD_LEASE_PATH = os.path.join(REPOSITORY_ROOT, "scripts", "experiment-apparatus", "e171_three_oracle_collusion", "device_card_lease.py")
BREAK_VARIABLE = "E201_FULL_TABLE_BREAK"
STAGE_CARD_LEASE = "card_lease"
CARDS_SHARED_BY_THE_BREAK: list[str] = []   # 只给弄坏开关 lanes-share-one-card 用：领到的卡依次记下，装置都拿最先领到的那一张


def load_device_card_lease():
    specification = importlib.util.spec_from_file_location("e201_device_card_lease", DEVICE_CARD_LEASE_PATH)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


device_card_lease = load_device_card_lease()
CARD_LEASE_CLIENT = device_card_lease.client_of(REPOSITORY_ROOT)


def broken(item: str) -> bool:
    return os.environ.get(BREAK_VARIABLE, "") == item


def refuse_without_a_channel(entry: str) -> int | None:
    """开跑前核领卡通道：没有就打一句与出路、交 78，有交 None。"""
    problem = device_card_lease.channel_problem()
    if problem is None or broken("card-lease-channel-unchecked"):
        return None
    device_card_lease.report_missing_channel("E201", problem, entry)
    return device_card_lease.EXIT_NO_CHANNEL


def run_on_a_leased_card(run: Callable[[dict[str, str]], int], environment: dict[str, str], step: str, client: str | None = None) -> int:
    """装置那一步：领一张卡、把卡串放进这一步的环境起 run，跑完照卡串还掉（还卡没成只打一行「!」）。领卡没成抛 device_card_lease.CardLeaseRefused，run 不起。"""
    client = client or CARD_LEASE_CLIENT
    try:
        cards = device_card_lease.take_card(client, REPOSITORY_ROOT, step)
    except device_card_lease.CardLeaseRefused:
        if not broken("card-lease-refusal-runs-the-device"):
            raise
        return run(environment)
    taken = cards
    if broken("lanes-share-one-card"):
        CARDS_SHARED_BY_THE_BREAK.append(taken)
        cards = CARDS_SHARED_BY_THE_BREAK[0]
    try:
        return run(dict(environment, **{device_card_lease.CARDS_VARIABLE: cards}))
    finally:
        if not broken("card-kept-after-the-device"):
            device_card_lease.return_card(client, REPOSITORY_ROOT, taken, step, report_failure=not broken("card-return-failure-unreported"))


def device_step_name(command: list[str]) -> str:
    """装置那一步的称呼（领还卡那几行里用）：行目录名，取不到写「装置」。"""
    if "--output-directory" in command[:-1]:
        return f"装置 {os.path.basename(os.path.dirname(command[command.index('--output-directory') + 1]))}"
    return "装置"


@contextlib.contextmanager
def device_steps_on_leased_cards(driver, client: str | None = None) -> Iterator[None]:
    """把逐行驱动（current_tree_row_driver 模块）的 run_with_time_limit 换成每一回先领一张卡、跑完还的那一层，出来时换回原件。
    弄坏开关 card-held-through-the-compile 下进来就领一张、整趟持着（编译那几步也持卡），各步都用它、出来也不还。"""
    unwrapped = driver.run_with_time_limit
    held = device_card_lease.take_card(client or CARD_LEASE_CLIENT, REPOSITORY_ROOT, "整趟逐行驱动") if broken("card-held-through-the-compile") else None

    def device_step(command: list[str], limit_seconds: int, **popen_arguments) -> int:
        environment = popen_arguments.pop("env", None) or dict(os.environ)

        def run(step_environment: dict[str, str]) -> int:
            return unwrapped(command, limit_seconds, env=step_environment, **popen_arguments)

        if held is not None:
            return run(dict(environment, **{device_card_lease.CARDS_VARIABLE: held}))
        try:
            return run_on_a_leased_card(run, environment, device_step_name(command), client)
        except device_card_lease.CardLeaseRefused as refused:
            raise driver.MeasurementPipeDefect(STAGE_CARD_LEASE, [f"take_exit={refused.exit_code}"]) from refused

    driver.run_with_time_limit = device_step
    try:
        yield
    finally:
        driver.run_with_time_limit = unwrapped


def main(argument_list: list[str]) -> int:
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✗ {BREAK_VARIABLE} 只在 full_table_run.py --selftest 里认，这一趟不是自证", file=sys.stderr)
        print(f"     → 怎么办：去掉 {BREAK_VARIABLE} 再交", file=sys.stderr)
        return 2
    refusal = refuse_without_a_channel("python3 scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree/full_table_run.py run")
    if refusal is not None:
        return refusal
    import current_tree_row_driver as driver
    with device_steps_on_leased_cards(driver):
        return driver.main(argument_list)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
