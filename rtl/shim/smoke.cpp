// Smoke test for the shim library: 2x5 matvec padded to 64 cols.
// W row0 = [1,-1,0,0,0,...], row1 = [0,0,1,0,1,...] against x=[10,20,30,40,50,0...]
// Expect y = [10-20, 30+50] = [-10, 80].
#include "trit_rtl_shim.h"

#include <cstdio>
#include <cstring>

int main() {
    uint8_t beats[2 * 16];
    std::memset(beats, 0, sizeof beats);
    // row 0: trit0=+1 (01), trit1=-1 (10) -> byte0 = 0b1001
    beats[0] = 0x09;
    // row 1: trit2=+1 (bits 5:4), trit4=+1 (bits 1:0 of byte 1)
    beats[16] = 0x10;
    beats[17] = 0x01;
    int8_t x[64] = {0};
    x[0] = 10; x[1] = 20; x[2] = 30; x[3] = 40; x[4] = 50;
    int32_t y[2] = {0, 0};

    void* h = trit_rtl_new();
    int rc = trit_rtl_matvec(h, beats, x, 2, 64, y);
    trit_rtl_free(h);

    if (rc != 0 || y[0] != -10 || y[1] != 80) {
        std::fprintf(stderr, "FAIL rc=%d y=[%d,%d] want [-10,80]\n", rc, y[0], y[1]);
        return 1;
    }
    std::printf("shim smoke OK: y=[%d,%d]\n", y[0], y[1]);
    return 0;
}
