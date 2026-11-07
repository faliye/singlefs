#!/usr/bin/env python3
"""审核复现：仅使用 control 自带的临时目录、假 systemctl 与假二进制。

经 task-server 提交。退出 0 表示所记录的文档/实现差异复现成功，不是产品通过。
"""
import hashlib
import pathlib
import runpy
import tempfile


def main():
    root = pathlib.Path(__file__).resolve().parents[1]
    source = root / "prune-server/control"
    before = hashlib.sha256(source.read_bytes()).hexdigest()
    api = runpy.run_path(str(source), run_name="audit_prune_control")
    with tempfile.TemporaryDirectory(prefix="singlefs-audit-prune-control-") as base:
        world = api["SelftestWorld"](
            base, "install", api["host_configuration"](api["LOCAL_ADDRESS"]), [], recording=True
        )
        code, output, error = world.run(["install"])
        calls = pathlib.Path(world.root, "fake/systemctl-calls.log").read_text().splitlines()
        unit = pathlib.Path(world.root, "systemd", api["UNIT_NAME"]).read_text()
        assert code == 0, (code, error)
        assert "随开机自起" in output
        assert not any(" enable " in f" {line} " for line in calls)
        assert "[Install]" not in [line.strip() for line in unit.splitlines()]
        print("A31 install_exit=0 output_promises_autostart=true enable_called=false install_section=false", flush=True)
        for show_fails in (False, True):
            world = api["SelftestWorld"](
                base, "stop", api["host_configuration"](api["LOCAL_ADDRESS"]), [], recording=True
            )
            # 假 stop 后留一个进程号，用来核实是否真的读到了 cgroup.procs。
            script = api["FAKE_SYSTEMCTL_STATEFUL"].replace(': > "$processes"', 'echo 4242 > "$processes"')
            if show_fails:
                script = script.replace("show) echo /fake.slice; exit 0 ;;", "show) echo 'synthetic show failure' >&2; exit 1 ;;")
            world.write("fake-systemctl", script, executable=True)
            code, output, error = world.run(["stop"])
            remaining = pathlib.Path(world.root, "cgroup/fake.slice/cgroup.procs").read_text().strip()
            assert remaining == "4242", remaining
            assert code == (0 if show_fails else 1), (show_fails, code, output, error)
            claimed_empty = "cgroup 里没有进程" in output
            assert claimed_empty == show_fails
            print(f"A33 show_failed={str(show_fails).lower()} stop_exit={code} remaining_fake_pid=4242 claimed_empty={str(claimed_empty).lower()}", flush=True)
    assert hashlib.sha256(source.read_bytes()).hexdigest() == before, "source changed during reproduction"
    print(f"AUDIT sources_unchanged=true control_sha256={before} temporary_roots_removed=true", flush=True)


if __name__ == "__main__":
    main()
