# rtl/Makefile: verilate, build, and run the golden-vector regression.
VERILATOR ?= verilator
CARGO_TARGET_DIR ?= $(HOME)/.cache/tritium-target
SETS := exact_64x64 padded_5x100 extremes_4x128 zeros_3x64 wide_2x6912
SET_DIRS := $(addprefix vectors/,$(SETS))

.PHONY: test lint vectors clean

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
	rm -rf obj_dir vectors
