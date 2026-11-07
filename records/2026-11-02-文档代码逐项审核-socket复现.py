"""Audit only: compare existing prefix-scan test with short/long temporary roots.

Run through task-server. Uses the existing test's fake Unix service, never the
real service. Exit zero means the documented path-bound discrepancy reproduced.
"""
import hashlib
import os
from pathlib import Path
import subprocess
import tempfile


def main():
    source = Path("prune-server/tests/reliable_client_prefix_scan_after_busy.rs")
    before = hashlib.sha256(source.read_bytes()).hexdigest()
    print(f"AUDIT_SOCKET source_before_sha256={before}", flush=True)
    command = [
        "cargo", "test", "--offline", "--manifest-path", "prune-server/Cargo.toml",
        "--test", "reliable_client_prefix_scan_after_busy", "--", "--exact",
        "prefix_scan_answered_busy_once_waits_the_retry_after_and_hands_back_the_rows_of_the_resent_frame",
    ]
    with tempfile.TemporaryDirectory(prefix="audit-socket-", dir="/tmp") as root:
        short = Path(root) / "short"
        long = Path(root) / ("long-" + "x" * 120)
        for path in (short, long):
            path.mkdir()
        for label, path, expected in (("short", short, 0), ("long", long, 101)):
            completed = subprocess.run(command, env=dict(os.environ, TMPDIR=str(path), CARGO_TARGET_DIR=str(Path(root) / "target")),
                                       text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            print(f"AUDIT_SOCKET case={label} tmpdir_bytes={len(os.fsencode(path))} exit={completed.returncode}", flush=True)
            print(completed.stdout, end="", flush=True)
            if completed.returncode != expected:
                raise AssertionError(f"{label}: expected {expected}, got {completed.returncode}")
            if label == "short" and "1 passed; 0 failed" not in completed.stdout:
                raise AssertionError("short-root test did not execute successfully")
            if label == "long" and not ("绑不上" in completed.stdout and "path must be shorter than SUN_LEN" in completed.stdout):
                raise AssertionError("long-root failure was not the expected socket path refusal")
    after = hashlib.sha256(source.read_bytes()).hexdigest()
    if before != after:
        raise AssertionError("source changed during audit; results need revalidation")
    print(f"AUDIT_SOCKET discrepancy_reproduced=true source_sha256={after} temporary_roots_removed=true", flush=True)


if __name__ == "__main__":
    main()
