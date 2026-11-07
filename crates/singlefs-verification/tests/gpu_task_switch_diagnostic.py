"""Opt-in, bounded GPU task-switch diagnostic around an already-built test binary."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import threading
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', required=True)
    parser.add_argument('--host', required=True)
    parser.add_argument('--uuid', required=True)
    parser.add_argument('--pci', required=True)
    parser.add_argument('--rounds', type=int, default=3)
    parser.add_argument('--seconds', type=int, default=900)
    args = parser.parse_args()
    if socket.gethostname() != args.host:
        parser.error('wrong host')
    cards = os.environ.get('GPU_CARDS_BY_PRIORITY', '').split()
    if len(cards) != 1 or cards[0].rsplit(':', 2)[-2] != '0':
        parser.error('this diagnostic requires an explicit, single GPU 0 assignment')
    if not 1 <= args.rounds <= 10 or not 1 <= args.seconds <= 1800:
        parser.error('invalid diagnostic bounds')
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    stopped = threading.Event()
    faults, rounds, current = [], [], [None]
    started = time.time()
    deadline = time.monotonic() + args.seconds

    def stop_test(reason):
        if not stopped.is_set():
            faults.append(dict(time=time.time(), reason=reason))
            stopped.set()
            child = current[0]
            if child is not None and child.poll() is None:
                try:
                    os.killpg(child.pid, signal.SIGTERM)  # Only our start_new_session child.
                except ProcessLookupError:
                    pass

    def read_kernel(process):
        with (output / 'kernel.log').open('w', buffering=1) as log:
            for line in process.stdout:
                log.write(line)
                if 'NVRM: Xid' in line and args.pci in line:
                    stop_test(line.strip())

    def read_test(process, number, seen):
        with (output / f'round-{number}.log').open('w', buffering=1) as log:
            for line in process.stdout:
                log.write(line)
                if 'phase=queue_submit_begin' in line:
                    seen['submitted'] = True
                if 'test result: ok.' in line:
                    seen['result'] = line.strip()
                if any(marker in line for marker in ('Parent device is lost', 'DeviceLost',
                                                     'GPU_CARD_LEASE_RETURN_WAIT_FAILED',
                                                     'NOT_RUN_NO_GPU_ADAPTER')):
                    stop_test(line.strip())

    journal = subprocess.Popen(['journalctl', '-k', '-b', '-f', '-n', '0', '-o', 'short-iso', '--no-pager'],
                               stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    reader = threading.Thread(target=read_kernel, args=(journal,), daemon=True)
    reader.start()
    try:
        with (output / 'gpu.csv.jsonl').open('w', buffering=1) as samples:
            for number in range(1, args.rounds + 1):
                if stopped.is_set():
                    break
                seen = {'submitted': False}
                command = [args.binary, 'the_gpu_judge_judges_a_version_of_', '--test-threads=1', '--nocapture']
                child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                         text=True, start_new_session=True)
                current[0] = child
                if stopped.is_set():
                    os.killpg(child.pid, signal.SIGTERM)
                thread = threading.Thread(target=read_test, args=(child, number, seen), daemon=True)
                thread.start()
                round_start = time.time()
                while child.poll() is None and not stopped.is_set():
                    if journal.poll() is not None:
                        stop_test('kernel journal monitor exited')
                        break
                    if time.monotonic() >= deadline:
                        stop_test('diagnostic deadline exceeded')
                        break
                    try:
                        sample = subprocess.run(['nvidia-smi', '-i', args.uuid,
                                                 '--query-gpu=uuid,pstate,temperature.gpu,power.draw,memory.used,utilization.gpu',
                                                 '--format=csv,noheader,nounits'], capture_output=True, text=True, timeout=5)
                        samples.write(json.dumps(dict(time=time.time(), round=number,
                                                      code=sample.returncode, sample=sample.stdout.strip(),
                                                      error=sample.stderr.strip())) + '\n')
                        if sample.returncode or args.uuid not in sample.stdout or 'ERR' in sample.stdout:
                            stop_test('GPU telemetry failed: ' + sample.stdout + sample.stderr)
                    except subprocess.TimeoutExpired:
                        stop_test('GPU telemetry timed out')
                    stopped.wait(1)
                try:
                    code = child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(child.pid, signal.SIGKILL)
                    code = child.wait(timeout=5)
                thread.join(timeout=5)
                rounds.append(dict(number=number, started_at=round_start, ended_at=time.time(),
                                   exit_code=code, **seen))
                current[0] = None
                if code or not seen['submitted'] or 'result' not in seen:
                    stop_test('test round failed or did not submit GPU work')
                if stopped.wait(2):
                    break
    finally:
        stop_test_if_running = current[0]
        if stop_test_if_running is not None and stop_test_if_running.poll() is None:
            stop_test('diagnostic interrupted')
        journal.terminate()
        try:
            journal.wait(timeout=5)
        except subprocess.TimeoutExpired:
            journal.kill()
            journal.wait()
        reader.join(timeout=5)
        result = dict(host=socket.gethostname(), gpu_uuid=args.uuid, pci=args.pci,
                      started_at=started, ended_at=time.time(), rounds=rounds, faults=faults,
                      binary_sha256=hashlib.sha256(Path(args.binary).read_bytes()).hexdigest(),
                      completed=len(rounds) == args.rounds and not faults)
        (output / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
        print(json.dumps(result), flush=True)
    return 0 if result['completed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
