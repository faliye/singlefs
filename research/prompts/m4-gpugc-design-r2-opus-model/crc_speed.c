/* 本机上同一份 16 MiB（第 i 字节 = i mod 251，E185 校准那一份，CRC-32C 应为 0x3430be3b）：
 * 软件八表（与 crates/singlefs-core/src/checksum.rs 同一算法）与 SSE4.2 crc32 指令各算 21 遍，线程 CPU 钟取中位。 */
#include <nmmintrin.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
static uint32_t tables[8][256];
static void build(void) {
    for (uint32_t b = 0; b < 256; b++) {
        uint32_t r = b;
        for (int k = 0; k < 8; k++) r = (r & 1) ? (r >> 1) ^ 0x82F63B78u : r >> 1;
        tables[0][b] = r;
    }
    for (uint32_t b = 0; b < 256; b++) {
        uint32_t r = tables[0][b];
        for (int t = 1; t < 8; t++) { r = tables[0][r & 0xff] ^ (r >> 8); tables[t][b] = r; }
    }
}
static uint32_t software(const uint8_t *p, size_t n) {
    uint32_t r = ~0u; size_t i = 0;
    for (; i + 8 <= n; i += 8) {
        uint32_t low, high; memcpy(&low, p + i, 4); memcpy(&high, p + i + 4, 4); low ^= r;
        r = tables[7][low & 0xff] ^ tables[6][(low >> 8) & 0xff] ^ tables[5][(low >> 16) & 0xff] ^ tables[4][low >> 24]
          ^ tables[3][high & 0xff] ^ tables[2][(high >> 8) & 0xff] ^ tables[1][(high >> 16) & 0xff] ^ tables[0][high >> 24];
    }
    for (; i < n; i++) r = tables[0][(r ^ p[i]) & 0xff] ^ (r >> 8);
    return ~r;
}
static uint32_t instruction(const uint8_t *p, size_t n) {
    uint64_t r = 0xFFFFFFFFu; size_t i = 0;
    for (; i + 8 <= n; i += 8) { uint64_t w; memcpy(&w, p + i, 8); r = _mm_crc32_u64(r, w); }
    uint32_t r32 = (uint32_t)r;
    for (; i < n; i++) r32 = _mm_crc32_u8(r32, p[i]);
    return ~r32;
}
static long long now(void) { struct timespec t; clock_gettime(CLOCK_THREAD_CPUTIME_ID, &t); return t.tv_sec * 1000000000LL + t.tv_nsec; }
static int cmp(const void *a, const void *b) { long long x = *(const long long *)a, y = *(const long long *)b; return (x > y) - (x < y); }
int main(void) {
    size_t n = 16u << 20; uint8_t *p = malloc(n);
    for (size_t i = 0; i < n; i++) p[i] = (uint8_t)(i % 251);
    build();
    const char *names[2] = {"software_eight_tables", "sse42_instruction"};
    for (int which = 0; which < 2; which++) {
        long long d[21]; uint32_t c = 0;
        for (int k = 0; k < 21; k++) { long long a = now(); c = which ? instruction(p, n) : software(p, n); d[k] = now() - a; }
        qsort(d, 21, sizeof d[0], cmp);
        printf("name=crc_speed implementation=%s bytes=%zu runs=21 crc32c=0x%08x anchor_holds=%s thread_cpu_median_nanoseconds=%lld bytes_per_second=%.0f\n",
               names[which], n, c, c == 0x3430be3bu ? "true" : "false", d[10], n * 1e9 / d[10]);
    }
    return 0;
}
