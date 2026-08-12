// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design implementation internals
// See Vtrit_matvec.h for the primary calling header

#include "Vtrit_matvec__pch.h"
#include "Vtrit_matvec___024root.h"

VL_ATTR_COLD void Vtrit_matvec___024root___eval_static(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_static\n"); );
}

VL_ATTR_COLD void Vtrit_matvec___024root___eval_initial__TOP(Vtrit_matvec___024root* vlSelf);

VL_ATTR_COLD void Vtrit_matvec___024root___eval_initial(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_initial\n"); );
    // Body
    Vtrit_matvec___024root___eval_initial__TOP(vlSelf);
    vlSelf->__Vtrigprevexpr___TOP__clk__0 = vlSelf->clk;
    vlSelf->__Vtrigprevexpr___TOP__rst_n__0 = vlSelf->rst_n;
}

VL_ATTR_COLD void Vtrit_matvec___024root___eval_initial__TOP(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_initial__TOP\n"); );
    // Body
    vlSelf->w_ready = 1U;
}

VL_ATTR_COLD void Vtrit_matvec___024root___eval_final(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_final\n"); );
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__stl(Vtrit_matvec___024root* vlSelf);
#endif  // VL_DEBUG
VL_ATTR_COLD bool Vtrit_matvec___024root___eval_phase__stl(Vtrit_matvec___024root* vlSelf);

VL_ATTR_COLD void Vtrit_matvec___024root___eval_settle(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_settle\n"); );
    // Init
    IData/*31:0*/ __VstlIterCount;
    CData/*0:0*/ __VstlContinue;
    // Body
    __VstlIterCount = 0U;
    vlSelf->__VstlFirstIteration = 1U;
    __VstlContinue = 1U;
    while (__VstlContinue) {
        if (VL_UNLIKELY((0x64U < __VstlIterCount))) {
#ifdef VL_DEBUG
            Vtrit_matvec___024root___dump_triggers__stl(vlSelf);
#endif
            VL_FATAL_MT("trit_matvec.sv", 7, "", "Settle region did not converge.");
        }
        __VstlIterCount = ((IData)(1U) + __VstlIterCount);
        __VstlContinue = 0U;
        if (Vtrit_matvec___024root___eval_phase__stl(vlSelf)) {
            __VstlContinue = 1U;
        }
        vlSelf->__VstlFirstIteration = 0U;
    }
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__stl(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___dump_triggers__stl\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VstlTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VstlTriggered.word(0U))) {
        VL_DBG_MSGF("         'stl' region trigger index 0 is active: Internal 'stl' trigger - first iteration\n");
    }
}
#endif  // VL_DEBUG

void Vtrit_matvec___024root___ico_sequent__TOP__0(Vtrit_matvec___024root* vlSelf);

VL_ATTR_COLD void Vtrit_matvec___024root___eval_stl(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_stl\n"); );
    // Body
    if ((1ULL & vlSelf->__VstlTriggered.word(0U))) {
        Vtrit_matvec___024root___ico_sequent__TOP__0(vlSelf);
    }
}

VL_ATTR_COLD void Vtrit_matvec___024root___eval_triggers__stl(Vtrit_matvec___024root* vlSelf);

VL_ATTR_COLD bool Vtrit_matvec___024root___eval_phase__stl(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_phase__stl\n"); );
    // Init
    CData/*0:0*/ __VstlExecute;
    // Body
    Vtrit_matvec___024root___eval_triggers__stl(vlSelf);
    __VstlExecute = vlSelf->__VstlTriggered.any();
    if (__VstlExecute) {
        Vtrit_matvec___024root___eval_stl(vlSelf);
    }
    return (__VstlExecute);
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__ico(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___dump_triggers__ico\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VicoTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VicoTriggered.word(0U))) {
        VL_DBG_MSGF("         'ico' region trigger index 0 is active: Internal 'ico' trigger - first iteration\n");
    }
}
#endif  // VL_DEBUG

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__act(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___dump_triggers__act\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VactTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VactTriggered.word(0U))) {
        VL_DBG_MSGF("         'act' region trigger index 0 is active: @(posedge clk or negedge rst_n)\n");
    }
}
#endif  // VL_DEBUG

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__nba(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___dump_triggers__nba\n"); );
    // Body
    if ((1U & (~ (IData)(vlSelf->__VnbaTriggered.any())))) {
        VL_DBG_MSGF("         No triggers active\n");
    }
    if ((1ULL & vlSelf->__VnbaTriggered.word(0U))) {
        VL_DBG_MSGF("         'nba' region trigger index 0 is active: @(posedge clk or negedge rst_n)\n");
    }
}
#endif  // VL_DEBUG

VL_ATTR_COLD void Vtrit_matvec___024root___ctor_var_reset(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___ctor_var_reset\n"); );
    // Body
    vlSelf->clk = VL_RAND_RESET_I(1);
    vlSelf->rst_n = VL_RAND_RESET_I(1);
    vlSelf->x_we = VL_RAND_RESET_I(1);
    vlSelf->x_addr = VL_RAND_RESET_I(13);
    vlSelf->x_data = VL_RAND_RESET_I(8);
    vlSelf->num_cols = VL_RAND_RESET_I(14);
    vlSelf->start = VL_RAND_RESET_I(1);
    vlSelf->w_valid = VL_RAND_RESET_I(1);
    VL_RAND_RESET_W(128, vlSelf->w_data);
    vlSelf->w_ready = VL_RAND_RESET_I(1);
    vlSelf->y_valid = VL_RAND_RESET_I(1);
    vlSelf->y_data = VL_RAND_RESET_I(32);
    vlSelf->err = VL_RAND_RESET_I(1);
    for (int __Vi0 = 0; __Vi0 < 8192; ++__Vi0) {
        vlSelf->trit_matvec__DOT__x_mem[__Vi0] = VL_RAND_RESET_I(8);
    }
    vlSelf->trit_matvec__DOT__beat_q = VL_RAND_RESET_I(8);
    vlSelf->trit_matvec__DOT__beats_per_row = VL_RAND_RESET_I(8);
    vlSelf->trit_matvec__DOT__acc_q = VL_RAND_RESET_I(32);
    vlSelf->trit_matvec__DOT__beat_sum = VL_RAND_RESET_I(32);
    vlSelf->trit_matvec__DOT__beat_err = VL_RAND_RESET_I(1);
    vlSelf->__Vtrigprevexpr___TOP__clk__0 = VL_RAND_RESET_I(1);
    vlSelf->__Vtrigprevexpr___TOP__rst_n__0 = VL_RAND_RESET_I(1);
}
