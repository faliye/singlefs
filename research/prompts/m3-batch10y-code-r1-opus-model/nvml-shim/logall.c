#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdio.h>
typedef void *(*dlsym_t)(void *, const char *);
void *dlsym(void *h, const char *name) {
  static dlsym_t real;
  if (!real) real = (dlsym_t)dlvsym(RTLD_NEXT, "dlsym", "GLIBC_2.34");
  if (!real) real = (dlsym_t)dlvsym(RTLD_NEXT, "dlsym", "GLIBC_2.2.5");
  fprintf(stderr, "DLSYM %s\n", name ? name : "(null)");
  return real(h, name);
}
