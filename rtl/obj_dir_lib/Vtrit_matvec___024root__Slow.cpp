// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Design implementation internals
// See Vtrit_matvec.h for the primary calling header

#include "Vtrit_matvec__pch.h"
#include "Vtrit_matvec__Syms.h"
#include "Vtrit_matvec___024root.h"

void Vtrit_matvec___024root___ctor_var_reset(Vtrit_matvec___024root* vlSelf);

Vtrit_matvec___024root::Vtrit_matvec___024root(Vtrit_matvec__Syms* symsp, const char* v__name)
    : VerilatedModule{v__name}
    , vlSymsp{symsp}
 {
    // Reset structure values
    Vtrit_matvec___024root___ctor_var_reset(this);
}

void Vtrit_matvec___024root::__Vconfigure(bool first) {
    if (false && first) {}  // Prevent unused
}

Vtrit_matvec___024root::~Vtrit_matvec___024root() {
}
