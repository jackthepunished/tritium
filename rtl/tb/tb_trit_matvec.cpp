// tb_trit_matvec: drives golden vector sets through Vtrit_matvec and
// requires exact i32 equality on every row output.
#include "Vtrit_matvec.h"
#include "verilated.h"

#include <cstdint>
#include <cstdio>
#include <fstream>
#include <string>
#include <vector>

namespace {

std::vector<std::string> lines(const std::string& path) {
    std::ifstream f(path);
    if (!f) { std::fprintf(stderr, "cannot open %s\n", path.c_str()); std::exit(2); }
    std::vector<std::string> out;
    for (std::string l; std::getline(f, l);)
        if (!l.empty()) out.push_back(l);
    return out;
}

struct Dut {
    Vtrit_matvec top;
    void tick() {
        top.clk = 0; top.eval();
        top.clk = 1; top.eval();
    }
    void reset() {
        top.w_valid = 0; top.x_we = 0; top.start = 0;
        top.rst_n = 0; tick(); tick();
        top.rst_n = 1; tick();
    }
};

bool run_set(Dut& d, const std::string& dir) {
    const auto meta = lines(dir + "/meta.txt");
    unsigned rows, cols;
    std::sscanf(meta.at(0).c_str(), "%u %u", &rows, &cols);
    const auto xh = lines(dir + "/x.hex");
    const auto wh = lines(dir + "/w.hex");
    const auto yh = lines(dir + "/y.hex");
    const unsigned beats = cols / 64;

    // preload activations
    for (unsigned c = 0; c < cols; c++) {
        d.top.x_we = 1;
        d.top.x_addr = c;
        d.top.x_data = static_cast<int8_t>(std::stoul(xh.at(c), nullptr, 16));
        d.tick();
    }
    d.top.x_we = 0;
    d.top.num_cols = cols;
    d.top.start = 1; d.tick(); d.top.start = 0;

    std::vector<int32_t> got;
    for (unsigned b = 0; b < rows * beats; b++) {
        const std::string& beat = wh.at(b); // 32 hex chars, MSB first
        for (int w = 0; w < 4; w++)         // w_data is 4x 32-bit words, LSW = chars 24..31
            d.top.w_data[w] = std::stoul(beat.substr(24 - 8 * w, 8), nullptr, 16);
        d.top.w_valid = 1;
        d.tick();
        if (d.top.y_valid) got.push_back(static_cast<int32_t>(d.top.y_data));
    }
    d.top.w_valid = 0;
    d.tick();
    if (d.top.y_valid) got.push_back(static_cast<int32_t>(d.top.y_data));

    bool ok = got.size() == rows && d.top.err == 0;
    for (unsigned r = 0; ok && r < rows; r++) {
        const auto want = static_cast<int32_t>(std::stoul(yh.at(r), nullptr, 16));
        if (got[r] != want) {
            std::fprintf(stderr, "%s row %u: got %d want %d\n", dir.c_str(), r, got[r], want);
            ok = false;
        }
    }
    if (got.size() != rows)
        std::fprintf(stderr, "%s: got %zu rows, want %u (err=%d)\n", dir.c_str(), got.size(), rows, (int)d.top.err);
    if (ok) std::printf("SET %s: %u rows OK\n", dir.c_str(), rows);
    return ok;
}

// invalid 0b11 code must raise the sticky err flag
bool run_invalid_code_check(Dut& d) {
    d.reset();
    d.top.x_we = 1; d.top.x_addr = 0; d.top.x_data = 1; d.tick(); d.top.x_we = 0;
    d.top.num_cols = 64;
    d.top.start = 1; d.tick(); d.top.start = 0;
    d.top.w_data[0] = 0x3; // code 0b11 in lane 0
    d.top.w_data[1] = d.top.w_data[2] = d.top.w_data[3] = 0;
    d.top.w_valid = 1; d.tick(); d.top.w_valid = 0;
    if (!d.top.err) { std::fprintf(stderr, "invalid-code check: err not raised\n"); return false; }
    std::printf("SET <invalid-code>: err raised OK\n");
    return true;
}

} // namespace

int main(int argc, char** argv) {
    Verilated::commandArgs(argc, argv);
    Dut d;
    bool ok = true;
    for (int i = 1; i < argc; i++) {
        d.reset();
        ok &= run_set(d, argv[i]);
    }
    ok &= run_invalid_code_check(d);
    d.top.final();
    return ok ? 0 : 1;
}
