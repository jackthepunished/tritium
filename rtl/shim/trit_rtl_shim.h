#pragma once
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
void* trit_rtl_new(void);
void trit_rtl_free(void* h);
/* beats: rows * (cols/64) * 16 bytes in .trit v1 layout -- each 16-byte beat is
   { uint64_t pos; uint64_t neg; } little-endian, bit l selecting lane l. This is
   byte-for-byte what a memory-mapped .trit holds, so the caller passes the
   mapping straight through with no repacking. cols % 64 == 0.
   Returns 0 on success, 1 if the sticky err flag rose (a lane set in both
   planes), 2 if the core produced the wrong number of rows.
   Renamed from trit_rtl_matvec so a stale caller linking against the old symbol
   fails at link time rather than silently reinterpreting the bytes. */
int trit_rtl_matvec_v1(void* h, const uint8_t* beats, const int8_t* x,
                       uint32_t rows, uint32_t cols, int32_t* y_out);
#ifdef __cplusplus
}
#endif
