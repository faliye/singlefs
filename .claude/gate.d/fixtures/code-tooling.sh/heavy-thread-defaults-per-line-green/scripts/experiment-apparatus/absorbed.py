import os
print_cores = os.cpu_count()
workers = int(os.environ["SAMPLE_ABSORBED_WORKERS"])
