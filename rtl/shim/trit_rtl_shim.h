#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
void* trit_rtl_new(void);
void trit_rtl_free(void* h);
/* beats: rows * (cols/64) * 16 bytes, .trit packed layout (2 bits/trit,
   4 trits/byte, little-endian within the byte); cols % 64 == 0.
   Returns 0 on success, 1 if the sticky err flag rose (invalid trit code). */
int trit_rtl_matvec(void* h, const uint8_t* beats, const int8_t* x,
                    uint32_t rows, uint32_t cols, int32_t* y_out);
#ifdef __cplusplus
}
#endif
