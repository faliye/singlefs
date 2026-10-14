#!/usr/bin/env python3
"""rows_orchestrator.py --selftest 用的假 multi-host-run.sh --peer：参数照真的（--peer、--memory-cap、--label、--send <本机目录> <树里的相对路径>、
--fetch <树里的相对路径>、-- <命令…>），「第二台」是 FAKE_PEER_ROOT 下的 runs/<标签>/ 一棵树：scripts/experiments、scripts
指到真仓、research/results 指到 FAKE_RESULTS_DIRECTORY（只读用），scripts/experiments/target 不建（真的树里也没有）。照真的做：标签已在退 2；取回目录（MULTI_HOST_RUN_FETCH_DIRECTORY）
里已有东西退 2；--send 在起命令之前拷进树（落点的上一级不在就像 rsync 那样失败、退 255）；命令在树根跑（环境另加 FAKE_DRIVER_HOST=<FAKE_MHR_HOST_NUMBER，默认 2>）；跑完取回 --fetch 点名的路径、
删树、照命令的退出码退出；收到 TERM 先停命令、再取回、再删树、退 143。FAKE_MHR_EXIT_FOR_LABEL=<标签片段>:<码> 时标签含那一段就不起命令、直接退那个码
（模拟工具链不同等它自己没跑成的 255）。FAKE_MHR_LOG 每次追加一行：标签与退出码。
"""
import os
import shutil
import signal
import subprocess
import sys

REAL_REPOSITORY = os.environ["FAKE_REAL_REPOSITORY"]


def main(arguments):
    label, sends, fetches, peer = "", [], [], False
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word == "--peer":
            peer, index = True, index + 1
        elif word == "--memory-cap":
            index += 2
        elif word == "--label":
            label, index = arguments[index + 1], index + 2
        elif word == "--send":
            sends.append((arguments[index + 1], arguments[index + 2]))
            index += 3
        elif word == "--fetch":
            fetches.append(arguments[index + 1])
            index += 2
        elif word == "--":
            command = arguments[index + 1:]
            break
        else:
            return 2
    assert peer, "假的只认 --peer"
    override = os.environ.get("FAKE_MHR_EXIT_FOR_LABEL", "")
    if override and override.split(":")[0] in label:
        record(label, int(override.split(":")[1]))
        return int(override.split(":")[1])
    tree = os.path.join(os.environ["FAKE_PEER_ROOT"], "runs", label)
    fetch_directory = os.environ["MULTI_HOST_RUN_FETCH_DIRECTORY"]
    if os.path.exists(tree) or (os.path.isdir(fetch_directory) and os.listdir(fetch_directory)):
        return 2
    os.makedirs(os.path.join(tree, "scripts", "experiments"))  # 照真的：scripts/experiments/target 不在 git 列得出的文件里，拷过去的树里没有它
    os.makedirs(os.path.join(tree, "research"))
    for name in os.listdir(os.path.join(REAL_REPOSITORY, "scripts", "experiments")):
        if name != "target":
            os.symlink(os.path.join(REAL_REPOSITORY, "scripts", "experiments", name), os.path.join(tree, "scripts", "experiments", name))
    for name in os.listdir(os.path.join(REAL_REPOSITORY, "scripts")):
        if name not in ("experiments", "results"):
            os.symlink(os.path.join(REAL_REPOSITORY, "scripts", name), os.path.join(tree, "scripts", name))
    os.symlink(os.environ["FAKE_RESULTS_DIRECTORY"], os.path.join(tree, "research", "results"))
    for source, relative in sends:
        if not os.path.isdir(os.path.dirname(os.path.join(tree, relative))):  # 照 rsync：落点的上一级不在就失败，真的那份退 255
            shutil.rmtree(tree)
            record(label, 255)
            return 255
        shutil.copytree(source, os.path.join(tree, relative), symlinks=True)
    environment = dict(os.environ, FAKE_DRIVER_HOST=os.environ.get("FAKE_MHR_HOST_NUMBER", "2"))

    def finish(code):
        for relative in fetches:
            source = os.path.join(tree, relative)
            if os.path.exists(source):
                shutil.copytree(source, os.path.join(fetch_directory, relative), symlinks=True)
        shutil.rmtree(tree)
        record(label, code)
        sys.exit(code)

    terminated, started = [], []

    def on_term(signal_number, frame):
        # 处理函数里只发信号、不 wait：主线程正握着 Popen.wait 的锁，在这里再 wait 会自锁。
        # 处理函数在起命令之前就装上：TERM 落在「命令已起、处理函数还没装」那一刻时，默认处置会让这一层直接退出、留下命令没人收
        if not terminated:
            terminated.append(True)
            for process in started:
                try:
                    os.killpg(process.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass

    signal.signal(signal.SIGTERM, on_term)
    if terminated:
        finish(143)
    process = subprocess.Popen(command, cwd=tree, env=environment, start_new_session=True)
    started.append(process)
    if terminated:  # TERM 落在 Popen 里面：处理函数那时还没有命令可停
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    code = process.wait()
    finish(143 if terminated else code)


def record(label, code):
    if os.environ.get("FAKE_MHR_LOG"):
        with open(os.environ["FAKE_MHR_LOG"], "a", encoding="utf-8") as handle:
            handle.write(f"label={label} exit={code}\n")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
