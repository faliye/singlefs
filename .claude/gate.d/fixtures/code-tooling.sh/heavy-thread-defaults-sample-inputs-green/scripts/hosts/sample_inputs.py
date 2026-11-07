import os
import unittest


def self_test_cells():
    wrapped = ["bash", "scripts/hosts/capped.sh", "31", "cargo", "test"]
    return wrapped


def selftest():
    options = {"threads": 2, "jobs": 2}
    return options


class ArgumentTests(unittest.TestCase):
    def arguments(self):
        return {"repo_root": "/repo", "threads": 2}


def cpu_report():
    return {"load1": None, "cpus": os.cpu_count()}


def domain_variable(name, registered):
    return name.startswith(registered) and not name.endswith("THREADS")


PLAN_WORKERS_FILE_VARIABLE = "SAMPLE_PLAN_WORKERS_FILE"
