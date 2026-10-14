"""样本：装置只读自己的线程变量，capped.sh 不设它，文件里也没回退到 SINGLEFS_THREAD_CAP 或登记值。"""
import os
workers = int(os.environ["SAMPLE_DEVICE_THREADS"])
