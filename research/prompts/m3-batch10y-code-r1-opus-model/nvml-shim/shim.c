// LD_PRELOAD 进 nvidia-smi：截 dlsym，把读显存与列进程的几个 NVML 函数换成先记时刻再转发的版本。只读，不碰卡上显存。
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>
#include <time.h>
typedef int (*f3)(void *, void *, void *);
#define N 16
static const char *names[N];
static f3 reals[N];
static int used;
static double now_us(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec * 1e6 + t.tv_nsec / 1e3; }
#define W(i) static int w##i(void *a, void *b, void *c) { double t0 = now_us(); int r = reals[i](a, b, c); fprintf(stderr, "NVMLCALL %s dev=%p t_us=%.1f..%.1f\n", names[i], a, t0, now_us()); return r; }
W(0) W(1) W(2) W(3) W(4) W(5) W(6) W(7) W(8) W(9) W(10) W(11) W(12) W(13) W(14) W(15)
static f3 wrappers[N] = {w0,w1,w2,w3,w4,w5,w6,w7,w8,w9,w10,w11,w12,w13,w14,w15};
typedef void *(*dlsym_t)(void *, const char *);
void *dlsym(void *h, const char *name) {
  static dlsym_t real;
  if (!real) real = (dlsym_t)dlvsym(RTLD_NEXT, "dlsym", "GLIBC_2.34");
  if (!real) real = (dlsym_t)dlvsym(RTLD_NEXT, "dlsym", "GLIBC_2.2.5");
  void *p = real(h, name);
  if (p && name && (strstr(name, "nvmlDeviceGetMemoryInfo") || strstr(name, "RunningProcesses") || strstr(name, "nvmlDeviceGetProcessesUtilization")) && used < N) {
    names[used] = strdup(name); reals[used] = (f3)p; return (void *)wrappers[used++];
  }
  return p;
}
