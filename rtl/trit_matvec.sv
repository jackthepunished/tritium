// trit_matvec: streaming ternary matrix-vector core.
// Weights arrive as 64x 2-bit trits per beat; activations are preloaded int8.
// No multipliers: each lane muxes {+x, -x, 0} into a combinational adder tree.
// Simulation-first: the single-cycle 64-term reduction and 64 parallel x_mem
// reads are fine under Verilator; FPGA timing/banking is addressed in the
// board bring-up phase.
module trit_matvec #(
    parameter int LANES    = 64,
    parameter int MAX_COLS = 8192,
    parameter int ACCW     = 32
) (
    input  logic                          clk,
    input  logic                          rst_n,
    // activation preload
    input  logic                          x_we,
    input  logic [$clog2(MAX_COLS)-1:0]   x_addr,
    input  logic signed [7:0]             x_data,
    // control
    input  logic [$clog2(MAX_COLS):0]     num_cols,  // multiple of LANES
    input  logic                          start,
    // weight stream
    input  logic                          w_valid,
    input  logic [2*LANES-1:0]            w_data,
    output logic                          w_ready,
    // row results
    output logic                          y_valid,
    output logic signed [ACCW-1:0]        y_data,
    output logic                          err
);

    localparam int BEATW = $clog2(MAX_COLS / LANES) + 1;

    logic signed [7:0] x_mem[MAX_COLS];

    logic [BEATW-1:0] beat_q, beats_per_row;
    logic signed [ACCW-1:0] acc_q;

    assign w_ready = 1'b1;

    // per-beat combinational reduction (select-accumulate, no multiplies)
    logic signed [ACCW-1:0] beat_sum;
    logic beat_err;
    always_comb begin
        beat_sum = '0;
        beat_err = 1'b0;
        for (int l = 0; l < LANES; l++) begin
            logic [1:0] code;
            logic signed [7:0] xv;
            code = w_data[2 * l +: 2];
            xv = x_mem[($clog2(MAX_COLS))'(32'(beat_q) * LANES + l)];
            unique case (code)
                2'b01:   beat_sum += ACCW'(xv);
                2'b10:   beat_sum -= ACCW'(xv);
                2'b00:   ;
                default: beat_err = 1'b1;
            endcase
        end
    end

    always_ff @(posedge clk or negedge rst_n) begin
        if (!rst_n) begin
            beat_q <= '0;
            beats_per_row <= '0;
            acc_q <= '0;
            y_valid <= 1'b0;
            y_data <= '0;
            err <= 1'b0;
        end else begin
            y_valid <= 1'b0;
            if (x_we) x_mem[x_addr] <= x_data;
            if (start) begin
                beats_per_row <= BEATW'(32'(num_cols) / LANES);
                beat_q <= '0;
                acc_q <= '0;
                err <= 1'b0;
            end else if (w_valid) begin
                if (beat_err) err <= 1'b1;
                if (32'(beat_q) == 32'(beats_per_row) - 1) begin
                    y_valid <= 1'b1;
                    y_data <= acc_q + beat_sum;
                    acc_q <= '0;
                    beat_q <= '0;
                end else begin
                    acc_q <= acc_q + beat_sum;
                    beat_q <= beat_q + 1'b1;
                end
            end
        end
    end

endmodule
