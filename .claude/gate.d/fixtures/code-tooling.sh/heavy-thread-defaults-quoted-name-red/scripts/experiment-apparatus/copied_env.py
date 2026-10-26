import os

settings = os.environ.copy()
THREADS = int(settings.get("SAMPLE_COPIED_THREADS", "0") or 0)
