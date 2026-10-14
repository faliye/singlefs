import os


def read_env(name):
    return os.environ[name]


THREADS = int(read_env("SAMPLE_WRAP_THREADS"))
