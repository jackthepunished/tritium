// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design implementation internals
// See Vtrit_matvec.h for the primary calling header

#include "Vtrit_matvec__pch.h"
#include "Vtrit_matvec___024root.h"

VL_INLINE_OPT void Vtrit_matvec___024root___ico_sequent__TOP__0(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___ico_sequent__TOP__0\n"); );
    // Init
    CData/*1:0*/ trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code;
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code = 0;
    CData/*7:0*/ trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv;
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv = 0;
    // Body
    vlSelf->trit_matvec__DOT__beat_sum = 0U;
    vlSelf->trit_matvec__DOT__beat_err = 0U;
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[0U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(1U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(2U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(3U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(4U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(5U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(6U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(7U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(8U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(9U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xaU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xbU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xcU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xdU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xeU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[0U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xfU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[1U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x10U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x11U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x12U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x13U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x14U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x15U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x16U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x17U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x18U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x19U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1aU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1bU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1cU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1dU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1eU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[1U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1fU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[2U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x20U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x21U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x22U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x23U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x24U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x25U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x26U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x27U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x28U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x29U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2aU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2bU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2cU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2dU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2eU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[2U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2fU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[3U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x30U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x31U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x32U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x33U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x34U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x35U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x36U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x37U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x38U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x39U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3aU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3bU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3cU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3dU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3eU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[3U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3fU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
}

void Vtrit_matvec___024root___eval_ico(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_ico\n"); );
    // Body
    if ((1ULL & vlSelf->__VicoTriggered.word(0U))) {
        Vtrit_matvec___024root___ico_sequent__TOP__0(vlSelf);
    }
}

void Vtrit_matvec___024root___eval_triggers__ico(Vtrit_matvec___024root* vlSelf);

bool Vtrit_matvec___024root___eval_phase__ico(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_phase__ico\n"); );
    // Init
    CData/*0:0*/ __VicoExecute;
    // Body
    Vtrit_matvec___024root___eval_triggers__ico(vlSelf);
    __VicoExecute = vlSelf->__VicoTriggered.any();
    if (__VicoExecute) {
        Vtrit_matvec___024root___eval_ico(vlSelf);
    }
    return (__VicoExecute);
}

void Vtrit_matvec___024root___eval_act(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_act\n"); );
}

VL_INLINE_OPT void Vtrit_matvec___024root___nba_sequent__TOP__0(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___nba_sequent__TOP__0\n"); );
    // Init
    CData/*1:0*/ trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code;
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code = 0;
    CData/*7:0*/ trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv;
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv = 0;
    CData/*7:0*/ __Vdly__trit_matvec__DOT__beat_q;
    __Vdly__trit_matvec__DOT__beat_q = 0;
    IData/*31:0*/ __Vdly__trit_matvec__DOT__acc_q;
    __Vdly__trit_matvec__DOT__acc_q = 0;
    SData/*12:0*/ __Vdlyvdim0__trit_matvec__DOT__x_mem__v0;
    __Vdlyvdim0__trit_matvec__DOT__x_mem__v0 = 0;
    CData/*7:0*/ __Vdlyvval__trit_matvec__DOT__x_mem__v0;
    __Vdlyvval__trit_matvec__DOT__x_mem__v0 = 0;
    CData/*0:0*/ __Vdlyvset__trit_matvec__DOT__x_mem__v0;
    __Vdlyvset__trit_matvec__DOT__x_mem__v0 = 0;
    // Body
    __Vdlyvset__trit_matvec__DOT__x_mem__v0 = 0U;
    __Vdly__trit_matvec__DOT__acc_q = vlSelf->trit_matvec__DOT__acc_q;
    __Vdly__trit_matvec__DOT__beat_q = vlSelf->trit_matvec__DOT__beat_q;
    if (vlSelf->rst_n) {
        if (vlSelf->x_we) {
            __Vdlyvval__trit_matvec__DOT__x_mem__v0 
                = vlSelf->x_data;
            __Vdlyvset__trit_matvec__DOT__x_mem__v0 = 1U;
            __Vdlyvdim0__trit_matvec__DOT__x_mem__v0 
                = vlSelf->x_addr;
        }
        vlSelf->y_valid = 0U;
        if (vlSelf->start) {
            vlSelf->err = 0U;
            __Vdly__trit_matvec__DOT__beat_q = 0U;
            __Vdly__trit_matvec__DOT__acc_q = 0U;
            vlSelf->trit_matvec__DOT__beats_per_row 
                = (0xffU & ((IData)(vlSelf->num_cols) 
                            >> 6U));
        } else if (vlSelf->w_valid) {
            if (vlSelf->trit_matvec__DOT__beat_err) {
                vlSelf->err = 1U;
            }
            if (((IData)(vlSelf->trit_matvec__DOT__beat_q) 
                 == ((IData)(vlSelf->trit_matvec__DOT__beats_per_row) 
                     - (IData)(1U)))) {
                vlSelf->y_valid = 1U;
                vlSelf->y_data = (vlSelf->trit_matvec__DOT__acc_q 
                                  + vlSelf->trit_matvec__DOT__beat_sum);
                __Vdly__trit_matvec__DOT__beat_q = 0U;
                __Vdly__trit_matvec__DOT__acc_q = 0U;
            } else {
                __Vdly__trit_matvec__DOT__acc_q = (vlSelf->trit_matvec__DOT__acc_q 
                                                   + vlSelf->trit_matvec__DOT__beat_sum);
                __Vdly__trit_matvec__DOT__beat_q = 
                    (0xffU & ((IData)(1U) + (IData)(vlSelf->trit_matvec__DOT__beat_q)));
            }
        }
    } else {
        vlSelf->err = 0U;
        __Vdly__trit_matvec__DOT__beat_q = 0U;
        __Vdly__trit_matvec__DOT__acc_q = 0U;
        vlSelf->y_valid = 0U;
        vlSelf->y_data = 0U;
        vlSelf->trit_matvec__DOT__beats_per_row = 0U;
    }
    if (__Vdlyvset__trit_matvec__DOT__x_mem__v0) {
        vlSelf->trit_matvec__DOT__x_mem[__Vdlyvdim0__trit_matvec__DOT__x_mem__v0] 
            = __Vdlyvval__trit_matvec__DOT__x_mem__v0;
    }
    vlSelf->trit_matvec__DOT__acc_q = __Vdly__trit_matvec__DOT__acc_q;
    vlSelf->trit_matvec__DOT__beat_q = __Vdly__trit_matvec__DOT__beat_q;
    vlSelf->trit_matvec__DOT__beat_sum = 0U;
    vlSelf->trit_matvec__DOT__beat_err = 0U;
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[0U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(1U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(2U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(3U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(4U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(5U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(6U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(7U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(8U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(9U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xaU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xbU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xcU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xdU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[0U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xeU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[0U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0xfU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[1U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x10U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x11U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x12U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x13U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x14U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x15U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x16U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x17U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x18U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x19U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1aU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1bU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1cU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1dU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[1U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1eU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[1U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x1fU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[2U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x20U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x21U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x22U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x23U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x24U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x25U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x26U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x27U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x28U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x29U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2aU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2bU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2cU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2dU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[2U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2eU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[2U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x2fU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & vlSelf->w_data[3U]);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x30U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 2U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x31U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 4U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x32U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 6U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x33U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 8U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x34U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0xaU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x35U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0xcU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x36U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0xeU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x37U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x10U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x38U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x12U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x39U) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x14U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3aU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x16U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3bU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x18U));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3cU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x1aU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3dU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (3U & (vlSelf->w_data[3U] >> 0x1cU));
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3eU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code 
        = (vlSelf->w_data[3U] >> 0x1eU);
    trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv 
        = vlSelf->trit_matvec__DOT__x_mem[(0x1fffU 
                                           & ((IData)(0x3fU) 
                                              + VL_SHIFTL_III(13,32,32, (IData)(vlSelf->trit_matvec__DOT__beat_q), 6U)))];
    if ((1U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              + VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((2U == (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_sum = (vlSelf->trit_matvec__DOT__beat_sum 
                                              - VL_EXTENDS_II(32,8, (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__xv)));
    } else if ((0U != (IData)(trit_matvec__DOT__unnamedblk1__DOT__unnamedblk2__DOT__code))) {
        vlSelf->trit_matvec__DOT__beat_err = 1U;
    }
}

void Vtrit_matvec___024root___eval_nba(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_nba\n"); );
    // Body
    if ((1ULL & vlSelf->__VnbaTriggered.word(0U))) {
        Vtrit_matvec___024root___nba_sequent__TOP__0(vlSelf);
    }
}

void Vtrit_matvec___024root___eval_triggers__act(Vtrit_matvec___024root* vlSelf);

bool Vtrit_matvec___024root___eval_phase__act(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_phase__act\n"); );
    // Init
    VlTriggerVec<1> __VpreTriggered;
    CData/*0:0*/ __VactExecute;
    // Body
    Vtrit_matvec___024root___eval_triggers__act(vlSelf);
    __VactExecute = vlSelf->__VactTriggered.any();
    if (__VactExecute) {
        __VpreTriggered.andNot(vlSelf->__VactTriggered, vlSelf->__VnbaTriggered);
        vlSelf->__VnbaTriggered.thisOr(vlSelf->__VactTriggered);
        Vtrit_matvec___024root___eval_act(vlSelf);
    }
    return (__VactExecute);
}

bool Vtrit_matvec___024root___eval_phase__nba(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_phase__nba\n"); );
    // Init
    CData/*0:0*/ __VnbaExecute;
    // Body
    __VnbaExecute = vlSelf->__VnbaTriggered.any();
    if (__VnbaExecute) {
        Vtrit_matvec___024root___eval_nba(vlSelf);
        vlSelf->__VnbaTriggered.clear();
    }
    return (__VnbaExecute);
}

#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__ico(Vtrit_matvec___024root* vlSelf);
#endif  // VL_DEBUG
#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__nba(Vtrit_matvec___024root* vlSelf);
#endif  // VL_DEBUG
#ifdef VL_DEBUG
VL_ATTR_COLD void Vtrit_matvec___024root___dump_triggers__act(Vtrit_matvec___024root* vlSelf);
#endif  // VL_DEBUG

void Vtrit_matvec___024root___eval(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval\n"); );
    // Init
    IData/*31:0*/ __VicoIterCount;
    CData/*0:0*/ __VicoContinue;
    IData/*31:0*/ __VnbaIterCount;
    CData/*0:0*/ __VnbaContinue;
    // Body
    __VicoIterCount = 0U;
    vlSelf->__VicoFirstIteration = 1U;
    __VicoContinue = 1U;
    while (__VicoContinue) {
        if (VL_UNLIKELY((0x64U < __VicoIterCount))) {
#ifdef VL_DEBUG
            Vtrit_matvec___024root___dump_triggers__ico(vlSelf);
#endif
            VL_FATAL_MT("trit_matvec.sv", 7, "", "Input combinational region did not converge.");
        }
        __VicoIterCount = ((IData)(1U) + __VicoIterCount);
        __VicoContinue = 0U;
        if (Vtrit_matvec___024root___eval_phase__ico(vlSelf)) {
            __VicoContinue = 1U;
        }
        vlSelf->__VicoFirstIteration = 0U;
    }
    __VnbaIterCount = 0U;
    __VnbaContinue = 1U;
    while (__VnbaContinue) {
        if (VL_UNLIKELY((0x64U < __VnbaIterCount))) {
#ifdef VL_DEBUG
            Vtrit_matvec___024root___dump_triggers__nba(vlSelf);
#endif
            VL_FATAL_MT("trit_matvec.sv", 7, "", "NBA region did not converge.");
        }
        __VnbaIterCount = ((IData)(1U) + __VnbaIterCount);
        __VnbaContinue = 0U;
        vlSelf->__VactIterCount = 0U;
        vlSelf->__VactContinue = 1U;
        while (vlSelf->__VactContinue) {
            if (VL_UNLIKELY((0x64U < vlSelf->__VactIterCount))) {
#ifdef VL_DEBUG
                Vtrit_matvec___024root___dump_triggers__act(vlSelf);
#endif
                VL_FATAL_MT("trit_matvec.sv", 7, "", "Active region did not converge.");
            }
            vlSelf->__VactIterCount = ((IData)(1U) 
                                       + vlSelf->__VactIterCount);
            vlSelf->__VactContinue = 0U;
            if (Vtrit_matvec___024root___eval_phase__act(vlSelf)) {
                vlSelf->__VactContinue = 1U;
            }
        }
        if (Vtrit_matvec___024root___eval_phase__nba(vlSelf)) {
            __VnbaContinue = 1U;
        }
    }
}

#ifdef VL_DEBUG
void Vtrit_matvec___024root___eval_debug_assertions(Vtrit_matvec___024root* vlSelf) {
    if (false && vlSelf) {}  // Prevent unused
    Vtrit_matvec__Syms* const __restrict vlSymsp VL_ATTR_UNUSED = vlSelf->vlSymsp;
    VL_DEBUG_IF(VL_DBG_MSGF("+    Vtrit_matvec___024root___eval_debug_assertions\n"); );
    // Body
    if (VL_UNLIKELY((vlSelf->clk & 0xfeU))) {
        Verilated::overWidthError("clk");}
    if (VL_UNLIKELY((vlSelf->rst_n & 0xfeU))) {
        Verilated::overWidthError("rst_n");}
    if (VL_UNLIKELY((vlSelf->x_we & 0xfeU))) {
        Verilated::overWidthError("x_we");}
    if (VL_UNLIKELY((vlSelf->x_addr & 0xe000U))) {
        Verilated::overWidthError("x_addr");}
    if (VL_UNLIKELY((vlSelf->num_cols & 0xc000U))) {
        Verilated::overWidthError("num_cols");}
    if (VL_UNLIKELY((vlSelf->start & 0xfeU))) {
        Verilated::overWidthError("start");}
    if (VL_UNLIKELY((vlSelf->w_valid & 0xfeU))) {
        Verilated::overWidthError("w_valid");}
}
#endif  // VL_DEBUG
