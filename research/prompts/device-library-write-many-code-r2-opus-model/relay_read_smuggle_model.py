#!/usr/bin/env python3
"""攻方腿模型（device-library-write-many-code-r2）：装置照 crates/singlefs-device-library/src/lib.rs 的 read 拼请求（read 不核相对键，
请求是 read <键>\\n 原样），交给 scripts/experiment-apparatus/device-code-key.py 的转交（LibraryRelay.serve，原样载入，不改一行），后面接一个只记账的假库。
看一件事：键里带换行的一条 read，转交会不会把换行后面那一段当一条合规的请求执行（写进库）。对照：键里带空格的 read（转交回 failed 3、停）。
用法：python3 -I relay_read_smuggle_model.py --repo <仓根>
每格打一行 name=<格> …，末行 name=summary。只起线程，不起子进程；每格把请求整段写完、关上请求写端，等转交读到文件尾自己收尾。"""
import argparse
import hashlib
import importlib.util
import os
import sys
import threading

CELLS = (
    # (格名, 装置调 read 时给的相对键, 这一格是不是对照)
    ("read_key_with_newline_then_empty_write", "items/a.out\nwrite items/injected.out 0", False),
    ("read_key_with_newline_then_eight_byte_write", "items/a.out\nwrite items/injected.out 8\ninjected", False),
    ("read_key_with_newline_then_write_many", "items/a.out\nwrite-many 1\nitems/injected.out 8\ninjected", False),
    ("control_read_key_with_space", "items/bad key.out", True),
    ("control_read_valid_key", "items/a.out", True),
)


def load_relay_module(repository):
    path = os.path.join(repository, "scripts", "experiment-apparatus", "device-code-key.py")
    spec = importlib.util.spec_from_file_location("device_code_key_under_attack", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module, path


class RecordingStore:
    """假库：照 LibraryStore 的 read / write / write_many / location 四样，记下每一次写进来的键。"""

    def __init__(self):
        self.rows = {}
        self.write_calls = []

    def read(self, key):
        return self.rows.get(key)

    def write(self, key, data):
        self.write_many([(key, data)])

    def write_many(self, pairs):
        pairs = list(pairs)
        self.write_calls.append([key for key, _data in pairs])
        for key, data in pairs:
            self.rows[key] = data

    def location(self):
        return "model"


def run_cell(module, key):
    store = RecordingStore()
    relay = module.LibraryRelay(store, "model")
    request_read, request_write = os.pipe()
    reply_read, reply_write = os.pipe()
    server = threading.Thread(target=relay.serve, args=(request_read, reply_write), daemon=True)
    server.start()
    os.write(request_write, f"read {key}\n".encode("utf-8"))   # lib.rs read：format!("read {relative}\n")，整条一次写
    os.close(request_write)
    replies = b""
    while True:
        chunk = os.read(reply_read, 65536)
        if not chunk:
            break
        replies += chunk
    os.close(reply_read)
    server.join(10)
    return {
        "relay_finished": not server.is_alive(),
        "replies": [line.decode("utf-8", "replace").split(" ")[0] + ("/" + line.decode("utf-8", "replace").split(" ")[1] if line.startswith(b"failed ") else "")
                    for line in replies.split(b"\n") if line],
        "store_writes": [key for call in store.write_calls for key in call],
        "relay_written": list(relay.written),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    options = parser.parse_args()
    module, path = load_relay_module(options.repo)
    with open(path, "rb") as handle:
        print(f"name=relay_source path={path} sha256={hashlib.sha256(handle.read()).hexdigest()}")
    hits = []
    for name, key, is_control in CELLS:
        seen = run_cell(module, key)
        injected = "items/injected.out" in seen["store_writes"]
        if injected and not is_control:
            hits.append(name)
        print(f"name={name} control={is_control} relay_finished={seen['relay_finished']} replies={','.join(seen['replies']) or '-'} "
              f"store_writes={','.join(seen['store_writes']) or '-'} relay_written={','.join(seen['relay_written']) or '-'} injected={injected}")
    print(f"name=summary hits={len(hits)} cells={','.join(hits) or '-'}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
