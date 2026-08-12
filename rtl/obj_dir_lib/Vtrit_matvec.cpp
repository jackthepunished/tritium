// Verilated -*- C++ -*-
// DESCRIPTION: Verilator output: Model implementation (design independent parts)

#include "Vtrit_matvec__pch.h"

//============================================================
// Constructors

Vtrit_matvec::Vtrit_matvec(VerilatedContext* _vcontextp__, const char* _vcname__)
    : VerilatedModel{*_vcontextp__}
    , vlSymsp{new Vtrit_matvec__Syms(contextp(), _vcname__, this)}
    , clk{vlSymsp->TOP.clk}
    , rst_n{vlSymsp->TOP.rst_n}
    , x_we{vlSymsp->TOP.x_we}
    , x_data{vlSymsp->TOP.x_data}
    , start{vlSymsp->TOP.start}
    , w_valid{vlSymsp->TOP.w_valid}
    , w_ready{vlSymsp->TOP.w_ready}
    , y_valid{vlSymsp->TOP.y_valid}
    , err{vlSymsp->TOP.err}
    , x_addr{vlSymsp->TOP.x_addr}
    , num_cols{vlSymsp->TOP.num_cols}
    , w_data{vlSymsp->TOP.w_data}
    , y_data{vlSymsp->TOP.y_data}
    , rootp{&(vlSymsp->TOP)}
{
    // Register model with the context
    contextp()->addModel(this);
}

Vtrit_matvec::Vtrit_matvec(const char* _vcname__)
    : Vtrit_matvec(Verilated::threadContextp(), _vcname__)
{
}

//============================================================
// Destructor

Vtrit_matvec::~Vtrit_matvec() {
    delete vlSymsp;
}

//============================================================
// Evaluation function

#ifdef VL_DEBUG
void Vtrit_matvec___024root___eval_debug_assertions(Vtrit_matvec___024root* vlSelf);
#endif  // VL_DEBUG
void Vtrit_matvec___024root___eval_static(Vtrit_matvec___024root* vlSelf);
void Vtrit_matvec___024root___eval_initial(Vtrit_matvec___024root* vlSelf);
void Vtrit_matvec___024root___eval_settle(Vtrit_matvec___024root* vlSelf);
void Vtrit_matvec___024root___eval(Vtrit_matvec___024root* vlSelf);

void Vtrit_matvec::eval_step() {
    VL_DEBUG_IF(VL_DBG_MSGF("+++++TOP Evaluate Vtrit_matvec::eval_step\n"); );
#ifdef VL_DEBUG
    // Debug assertions
    Vtrit_matvec___024root___eval_debug_assertions(&(vlSymsp->TOP));
#endif  // VL_DEBUG
    vlSymsp->__Vm_deleter.deleteAll();
    if (VL_UNLIKELY(!vlSymsp->__Vm_didInit)) {
        vlSymsp->__Vm_didInit = true;
        VL_DEBUG_IF(VL_DBG_MSGF("+ Initial\n"););
        Vtrit_matvec___024root___eval_static(&(vlSymsp->TOP));
        Vtrit_matvec___024root___eval_initial(&(vlSymsp->TOP));
        Vtrit_matvec___024root___eval_settle(&(vlSymsp->TOP));
    }
    VL_DEBUG_IF(VL_DBG_MSGF("+ Eval\n"););
    Vtrit_matvec___024root___eval(&(vlSymsp->TOP));
    // Evaluate cleanup
    Verilated::endOfEval(vlSymsp->__Vm_evalMsgQp);
}

//============================================================
// Events and timing
bool Vtrit_matvec::eventsPending() { return false; }

uint64_t Vtrit_matvec::nextTimeSlot() {
    VL_FATAL_MT(__FILE__, __LINE__, "", "%Error: No delays in the design");
    return 0;
}

//============================================================
// Utilities

const char* Vtrit_matvec::name() const {
    return vlSymsp->name();
}

//============================================================
// Invoke final blocks

void Vtrit_matvec___024root___eval_final(Vtrit_matvec___024root* vlSelf);

VL_ATTR_COLD void Vtrit_matvec::final() {
    Vtrit_matvec___024root___eval_final(&(vlSymsp->TOP));
}

//============================================================
// Implementations of abstract methods from VerilatedModel

const char* Vtrit_matvec::hierName() const { return vlSymsp->name(); }
const char* Vtrit_matvec::modelName() const { return "Vtrit_matvec"; }
unsigned Vtrit_matvec::threads() const { return 1; }
void Vtrit_matvec::prepareClone() const { contextp()->prepareClone(); }
void Vtrit_matvec::atClone() const {
    contextp()->threadPoolpOnClone();
}

//============================================================
// Trace configuration

VL_ATTR_COLD void Vtrit_matvec::trace(VerilatedVcdC* tfp, int levels, int options) {
    vl_fatal(__FILE__, __LINE__, __FILE__,"'Vtrit_matvec::trace()' called on model that was Verilated without --trace option");
}
