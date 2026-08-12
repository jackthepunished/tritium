// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design internal header
// See Vtrit_matvec.h for the primary calling header

#ifndef VERILATED_VTRIT_MATVEC___024ROOT_H_
#define VERILATED_VTRIT_MATVEC___024ROOT_H_  // guard

#include "verilated.h"


class Vtrit_matvec__Syms;

class alignas(VL_CACHE_LINE_BYTES) Vtrit_matvec___024root final : public VerilatedModule {
  public:

    // DESIGN SPECIFIC STATE
    VL_IN8(clk,0,0);
    VL_IN8(rst_n,0,0);
    VL_IN8(x_we,0,0);
    VL_IN8(x_data,7,0);
    VL_IN8(start,0,0);
    VL_IN8(w_valid,0,0);
    VL_OUT8(w_ready,0,0);
    VL_OUT8(y_valid,0,0);
    VL_OUT8(err,0,0);
    CData/*7:0*/ trit_matvec__DOT__beat_q;
    CData/*7:0*/ trit_matvec__DOT__beats_per_row;
    CData/*0:0*/ trit_matvec__DOT__beat_err;
    CData/*0:0*/ __VstlFirstIteration;
    CData/*0:0*/ __VicoFirstIteration;
    CData/*0:0*/ __Vtrigprevexpr___TOP__clk__0;
    CData/*0:0*/ __Vtrigprevexpr___TOP__rst_n__0;
    CData/*0:0*/ __VactContinue;
    VL_IN16(x_addr,12,0);
    VL_IN16(num_cols,13,0);
    VL_INW(w_data,127,0,4);
    VL_OUT(y_data,31,0);
    IData/*31:0*/ trit_matvec__DOT__acc_q;
    IData/*31:0*/ trit_matvec__DOT__beat_sum;
    IData/*31:0*/ __VactIterCount;
    VlUnpacked<CData/*7:0*/, 8192> trit_matvec__DOT__x_mem;
    VlTriggerVec<1> __VstlTriggered;
    VlTriggerVec<1> __VicoTriggered;
    VlTriggerVec<1> __VactTriggered;
    VlTriggerVec<1> __VnbaTriggered;

    // INTERNAL VARIABLES
    Vtrit_matvec__Syms* const vlSymsp;

    // CONSTRUCTORS
    Vtrit_matvec___024root(Vtrit_matvec__Syms* symsp, const char* v__name);
    ~Vtrit_matvec___024root();
    VL_UNCOPYABLE(Vtrit_matvec___024root);

    // INTERNAL METHODS
    void __Vconfigure(bool first);
};


#endif  // guard
