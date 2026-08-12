# rtl/Makefile: verilate, build, and run the golden-vector regression.
VERILATOR ?= verilator
CARGO_TARGET_DIR ?= $(HOME)/.cache/tritium-target
SETS := exact_64x64 padded_5x100 extremes_4x128 zeros_3x64 wide_2x6912
SET_DIRS := $(addprefix vectors/,$(SETS))

.PHONY: test lint vectors synth lib libtest clean

synth:
	yosys -s synth.ys

VERILATOR_ROOT_DIR := $(shell $(VERILATOR) --getenv VERILATOR_ROOT)
VINC := -I$(VERILATOR_ROOT_DIR)/include -I$(VERILATOR_ROOT_DIR)/include/vltstd

# Static library for the tritsim `rtl` feature (hardware-in-the-loop).
lib: obj_dir_lib/libtritcore_rtl.a

obj_dir_lib/libtritcore_rtl.a: trit_matvec.sv shim/trit_rtl_shim.cpp shim/trit_rtl_shim.h
	$(VERILATOR) --cc trit_matvec.sv --build -Mdir obj_dir_lib -CFLAGS -fPIC
	g++ -O2 -fPIC -std=c++17 -c shim/trit_rtl_shim.cpp -Iobj_dir_lib -Ishim $(VINC) -o obj_dir_lib/trit_rtl_shim.o
	g++ -O2 -fPIC -std=c++17 -c $(VERILATOR_ROOT_DIR)/include/verilated.cpp $(VINC) -o obj_dir_lib/verilated.o
	g++ -O2 -fPIC -std=c++17 -c $(VERILATOR_ROOT_DIR)/include/verilated_threads.cpp $(VINC) -o obj_dir_lib/verilated_threads.o
	cd obj_dir_lib && ar x Vtrit_matvec__ALL.a && ar rcs libtritcore_rtl.a *.o

libtest: lib
	g++ -O2 -std=c++17 shim/smoke.cpp -Ishim -Lobj_dir_lib -ltritcore_rtl -o obj_dir_lib/smoke
	./obj_dir_lib/smoke

# Path relative to the repo root (the vectors recipe runs after `cd ..`).
MODEL ?= models/bitnet-2b4t.trit

test: obj_dir/Vtrit_matvec vectors
	./obj_dir/Vtrit_matvec $(SET_DIRS) $$( [ -d vectors/model_k_proj_l0 ] && echo vectors/model_k_proj_l0 )

lint:
	$(VERILATOR) --lint-only -Wall trit_matvec.sv

obj_dir/Vtrit_matvec: trit_matvec.sv tb/tb_trit_matvec.cpp
	$(VERILATOR) --cc --exe --build -Wall --top-module trit_matvec \
	    trit_matvec.sv tb/tb_trit_matvec.cpp -o Vtrit_matvec

vectors:
	cd .. && CARGO_TARGET_DIR=$(CARGO_TARGET_DIR) cargo run -p tritsim --release -- vectors --out rtl/vectors \
	    $$( [ -f $(MODEL) ] && echo --model $(MODEL) )

clean:
	rm -rf obj_dir obj_dir_lib vectors
