// C shim over the Verilated trit_matvec core: the hardware-in-the-loop
// backend tritsim links against. Drives the DUT exactly like the Phase-1
// testbench: preload x, latch config via start, stream 16-byte beats,
// collect y_valid pulses (including the registered trailing one).
#include "trit_rtl_shim.h"

#include "Vtrit_matvec.h"
#include "verilated.h"

#include <cstring>

namespace {

struct Core {
    Vtrit_matvec top;
    void tick() {
        top.clk = 0;
        top.eval();
        top.clk = 1;
        top.eval();
    }
    void reset() {
        top.w_valid = 0;
        top.x_we = 0;
        top.start = 0;
        top.rst_n = 0;
        tick();
        tick();
        top.rst_n = 1;
        tick();
    }
};

} // namespace

extern "C" {

void* trit_rtl_new(void) {
    Core* c = new Core();
    c->reset();
    return c;
}

void trit_rtl_free(void* h) {
    delete static_cast<Core*>(h);
}

int trit_rtl_matvec(void* h, const uint8_t* beats, const int8_t* x,
                    uint32_t rows, uint32_t cols, int32_t* y_out) {
    Core* c = static_cast<Core*>(h);
    // reset clears the sticky err flag and counters from any prior call
    c->reset();

    for (uint32_t i = 0; i < cols; i++) {
        c->top.x_we = 1;
        c->top.x_addr = i;
        c->top.x_data = x[i];
        c->tick();
    }
    c->top.x_we = 0;
    c->top.num_cols = cols;
    c->top.start = 1;
    c->tick();
    c->top.start = 0;

    const uint32_t beats_total = rows * (cols / 64);
    uint32_t got = 0;
    for (uint32_t b = 0; b < beats_total; b++) {
        uint32_t words[4];
        std::memcpy(words, beats + b * 16, 16); // little-endian bytes -> LE words
        for (int w = 0; w < 4; w++) c->top.w_data[w] = words[w];
        c->top.w_valid = 1;
        c->tick();
        if (c->top.y_valid && got < rows)
            y_out[got++] = static_cast<int32_t>(c->top.y_data);
    }
    c->top.w_valid = 0;
    c->tick();
    if (c->top.y_valid && got < rows)
        y_out[got++] = static_cast<int32_t>(c->top.y_data);

    if (got != rows) return 2;
    return c->top.err ? 1 : 0;
}

} // extern "C"
