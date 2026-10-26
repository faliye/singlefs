"""样本：装置读自己的线程变量，capped.sh 不设它，但没设时回退到登记值，不点名。"""
import os
from heavy_threads import default_heavy_threads
workers = int(os.environ.get("SAMPLE_OWN_THREADS") or default_heavy_threads())
