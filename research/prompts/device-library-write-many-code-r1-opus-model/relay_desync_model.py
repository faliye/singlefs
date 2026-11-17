#!/usr/bin/env python3
"""攻方腿模型（device-library-write-many-code-r1）：装置照 crates/singlefs-device-library/src/lib.rs 的 write_many 拼一条 write-many 请求，
交给 scripts/experiment-apparatus/device-code-key.py 的转交（LibraryRelay.serve，原样载入，不改一行），后面接一个只记账的假库。
看三件事：一批里有一条不合的相对键时，那条请求后面没读的字节会不会被当成新请求执行；一批大时会不会卡死；
prune_server_client.batches_of 把一批切成几帧（几次提交）。
用法：python3 -I relay_desync_model.py --repo <仓根>
每格打一行 name=<格> …，末行 name=summary。只起线程，不起子进程；卡住的那一格到点关管道拆掉，线程都收回。"""
import argparse
import hashlib
import importlib.util
import os
import select
import sys
import threading

SETTLE_SECONDS = 3.0


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


def write_many_request(entries):
    """照 lib.rs write_many 的拼法：write-many <条数>\\n，每条 <相对键> <内容的 UTF-8 字节数>\\n<内容>。"""
    request = f"write-many {len(entries)}\n".encode()
    for key, text in entries:
        data = text.encode("utf-8")
        request += f"{key} {len(data)}\n".encode() + data
    return request


def reply_line(descriptor, seconds):
    """读一行回应（装置的 reply_line）；到点没读到整行交 None。"""
    line = b""
    while not line.endswith(b"\n"):
        ready, _, _ = select.select([descriptor], [], [], seconds)
        if not ready:
            return None
        chunk = os.read(descriptor, 1)
        if not chunk:
            return line
        line += chunk
    return line


def run_through_relay(module, request):
    """装置一侧照 write_all 整条写完再读头一行回应，读到就退（关掉自己那两头）；→ 观测字典。"""
    store = RecordingStore()
    relay = module.LibraryRelay(store, "model")
    request_read, request_write = os.pipe()
    reply_read, reply_write = os.pipe()
    server = threading.Thread(target=relay.serve, args=(request_read, reply_write), daemon=True)
    server.start()
    finished, errors = threading.Event(), []

    def device_writes():
        view = memoryview(request)
        try:
            while view:
                view = view[os.write(request_write, view):]
        except OSError as error:
            errors.append(type(error).__name__)
        finally:
            finished.set()

    writer = threading.Thread(target=device_writes, daemon=True)
    writer.start()
    blocked = not finished.wait(SETTLE_SECONDS)
    first = None
    if not blocked:
        first = reply_line(reply_read, SETTLE_SECONDS)
    # 装置退了（或卡住之后被拆）：它那两头都关上
    os.close(reply_read)
    writer.join(SETTLE_SECONDS)
    os.close(request_write)
    server.join(SETTLE_SECONDS)
    return {
        "device_blocked_in_write_all": blocked,
        "first_reply": (first or b"").decode("utf-8", "replace").rstrip("\n").split(" ", 2)[:2],
        "writer_error": errors[0] if errors else "-",
        "relay_thread_left_running": server.is_alive() or writer.is_alive(),
        "store_write_calls": store.write_calls,
        "relay_written_keys": list(relay.written),
    }


def show(name, observed):
    calls = ";".join(",".join(keys) for keys in observed["store_write_calls"]) or "-"
    print(f"name={name} device_blocked_in_write_all={observed['device_blocked_in_write_all']} "
          f"first_reply={'/'.join(observed['first_reply']) or '-'} writer_error={observed['writer_error']} "
          f"threads_left={observed['relay_thread_left_running']} store_writes={calls} "
          f"relay_written={','.join(observed['relay_written_keys']) or '-'}")


def frame_count(module, entries):
    """prune_server_client 的 write_many 对不分段的值照 write_batches 的切法切帧；同一帧一个写批（一次提交）。"""
    client = module.prune_server_client
    return sum(1 for _batch in client.batches_of(entries, lambda entry: len(entry[0]) + len(entry[1]) + 9))


def main(arguments):
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    options = parser.parse_args(arguments)
    module, path = load_relay_module(options.repo)
    with open(path, "rb") as handle:
        print(f"name=relay_source path={path} sha256={hashlib.sha256(handle.read()).hexdigest()}")
    client_path = module.prune_server_client.__file__
    with open(client_path, "rb") as handle:
        print(f"name=client_source path={client_path} sha256={hashlib.sha256(handle.read()).hexdigest()}")
    hits = []

    observed = run_through_relay(module, write_many_request([("items/a.out", "甲\n乙"), ("items/b.out", "")]))
    show("control_valid_batch", observed)

    injected_content = "write items/injected.out 8\ninjected"
    observed = run_through_relay(module, write_many_request([("items/bad key.out", injected_content)]))
    show("bad_key_first_then_request_shaped_content", observed)
    if "items/injected.out" in [key for keys in observed["store_write_calls"] for key in keys]:
        hits.append("bad_key_first_then_request_shaped_content")

    observed = run_through_relay(module, write_many_request([("items/ok.out", "fine"), ("items/bad key.out", injected_content)]))
    show("bad_key_second_then_request_shaped_content", observed)
    if "items/injected.out" in [key for keys in observed["store_write_calls"] for key in keys]:
        hits.append("bad_key_second_then_request_shaped_content")

    many_lines = "\n".join("z" * 100 for _line in range(20))
    large = [(f"items/{index:064x}.out", many_lines) for index in range(3000)]
    request = write_many_request([("items/bad key.out", "x")] + large)
    print(f"name=large_batch_bytes bytes={len(request)}")
    observed = run_through_relay(module, request)
    show("bad_key_first_then_large_multiline_batch", observed)
    if observed["device_blocked_in_write_all"]:
        hits.append("bad_key_first_then_large_multiline_batch")

    observed = run_through_relay(module, write_many_request(large))
    show("control_large_multiline_batch", observed)

    for count in (8192, 8193):
        entries = [(f"apparatus-items/kind/items/{index:064x}.out".encode(), b"v") for index in range(count)]
        print(f"name=frames_small_values entries={count} frames={frame_count(module, entries)}")
    nine_mebibytes = b"v" * (9 * 1024 * 1024)
    entries = [(b"apparatus-items/kind/items/first.out", nine_mebibytes), (b"apparatus-items/kind/items/second.out", nine_mebibytes)]
    print(f"name=frames_two_nine_mebibyte_values entries=2 frames={frame_count(module, entries)} "
          f"segment_bytes={module.prune_server_client.SEGMENT_BYTES} frame_batch_bytes={module.prune_server_client.FRAME_BATCH_BYTES} "
          f"frame_batch_keys={module.prune_server_client.FRAME_BATCH_KEYS}")
    print(f"name=summary hits={len(hits)} cells={','.join(hits) or '-'}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
