# The `.trit` format, version 1

A `.trit` file is a single self-describing container holding one quantized
model: an embedded copy of the source `config.json`, a tensor table, and a
payload of ternary and dense tensors.

Its one real design commitment is that **ternary weights are stored as bit
planes, in the exact order and grouping the compute engine consumes them**. A
64-column block of one row occupies 16 contiguous bytes -- one 64-bit positive
mask and one 64-bit negative mask. That block is simultaneously:

- one cycle's weight input to the `tritcore` RTL engine, and
- one iteration of the CPU kernel's inner loop, on every supported ISA.

So a memory-mapped `.trit` is not a container the runtime decodes into some
other layout. It *is* the layout. Nothing between the page cache and the
accumulator rewrites a byte.

Version 1 is the only version this build reads or writes. Version 0, which used
interleaved 2-bit codes, is rejected with a message naming the migration
command; see [Migrating from v0](#migrating-from-v0).

## 1. Why bit planes

A ternary weight has three states, so two bits is the natural budget either way.
Version 0 spent them as an interleaved code per weight -- `00` = 0, `01` = +1,
`10` = -1 -- packed four weights to a byte. Version 1 spends the same two bits
planar: all the "is this weight +1" bits of a 64-column block in one word, all
the "is this weight -1" bits in the next.

The bit count is identical. What changes is what a machine can do with one load.

```
y[r] = sum over c of  W[r][c] * x[c]        W[r][c] in {-1, 0, +1}
     = sum of x[c] where W[r][c] = +1
     - sum of x[c] where W[r][c] = -1
```

With planes, each of those two sums is a mask applied to a vector of
activations. On AVX-512 the mask *is* a `k` register and the whole operation is
two `vmovdqu8` under mask plus two `vpdpbusd`. On NEON and AVX2 the mask is
expanded to byte lanes first. In every case the decision "does this weight
participate, and with which sign" costs no branch, no table lookup, and no
per-weight arithmetic.

With interleaved codes, extracting the sign of lane `k` means shifting by `2k`
and masking -- per weight, defeating the vectorization entirely, and the reason
the pre-pivot scalar kernel ran a branch per weight.

The RTL core wants exactly the same thing: 64 lanes each asking "am I set in
pos? in neg?" is 128 wires and a mux, with no 2-bit decode in front of it.

Ternary weights are 2.000 bits each, exactly, when the column count is a
multiple of 64 -- as every projection in BitNet b1.58 2B4T is.

## 2. File layout

All integers are little-endian. All offsets are byte offsets.

```
off   size  field
  0      4  magic, ASCII "TRIT"
  4      4  u32  version = 1
  8      4  u32  header_flags   (reserved; must be 0)
 12      8  u64  payload_hash   (FNV-1a-64 over the payload; 0 = not computed)
 20      4  u32  config_len     (<= 16 MiB)
 24      n  config             (UTF-8 JSON: the source config.json verbatim)
  .      4  u32  tensor_count   (<= 1048576)
  .      .  tensor records     (tensor_count of them, see 2.2)
  .      .  zero padding       (until the offset is a multiple of 64)
  .      .  payload
```

`payload_start` -- the offset just past the padding -- is always a multiple of
64. This is a format invariant, not an optimization: mapping bases are
page-aligned, so an aligned `payload_start` plus aligned tensor offsets is what
puts every tensor on a cache-line boundary in the mapped image. That is what
lets a dense f32 tensor be borrowed from the map instead of copied.

### 2.1 Header fields

**`header_flags`** and the per-tensor `flags` are both reserved and must be
zero. A reader rejects unknown bits rather than ignoring them, so a future
format extension cannot be silently half-understood by an old binary.

**`payload_hash`** is FNV-1a-64 over the payload region. It is cheap,
dependency-free, and detects a truncated download or a half-written file. It is
not a security primitive and is not claimed to be one. A writer that cannot or
will not compute it stores `0`, which readers report as "absent" rather than
treating as a mismatch. The hash is only checked on request -- see
[Validation](#4-validation).

**`config`** is the source `config.json`, byte for byte. Keeping the original
rather than a re-serialized subset means fields this runtime does not yet
understand survive conversion, and a `.trit` remains diffable against the
checkpoint it came from.

### 2.2 Tensor record

```
size  field
   2  u16  name_len       (1..=1024)
   n  name               (UTF-8, unique within the file)
   1  u8   dtype          (0 = F32, 1 = Trit, 2 = BF16)
   1  u8   ndim           (1..=4)
   1  u8   reserved0      (must be 0)
   1  u8   reserved1      (must be 0)
 4*d  u32  dims[ndim]     (row-major, outermost first)
   4  f32  scale          (must be finite and > 0)
   4  u32  flags          (reserved; must be 0)
   8  u64  offset         (relative to payload_start; multiple of 64)
   8  u64  byte_len
```

The two reserved bytes are not padding for its own sake: they put `dims` on a
4-byte boundary within the record and leave room for a future `layout` byte
(blocked or tiled ternary orderings, per-row scales) at no cost.

`scale` is the dequantization factor for ternary tensors and `1.0` for dense
ones. It is rejected unless finite and strictly positive -- a NaN scale would
otherwise propagate silently into every logit with no error raised anywhere.

Tensors are written **sorted by name**. The converter enforces this so that
converting the same checkpoint twice produces byte-identical files, which is
what makes a checksum of a `.trit` mean anything.

### 2.3 Ternary payload

A ternary tensor must be 2-D: `rows = dims[0]`, `cols = dims[1]`.

```
beats_per_row = ceil(cols / 64)
byte_len      = rows * beats_per_row * 16
```

Row `r` occupies `beats_per_row` consecutive beats starting at
`r * beats_per_row * 16`. Beat `b` of row `r` is the 16 bytes at
`(r * beats_per_row + b) * 16`:

```
bytes  0..8   u64 pos    bit k set  =>  W[r][b*64 + k] = +1
bytes  8..16  u64 neg    bit k set  =>  W[r][b*64 + k] = -1
```

Bit `k` is the `k`-th least significant bit. Neither bit set means the weight is
`0`. The dequantized value is `scale * trit`.

Two invariants hold for every beat:

**P1 -- the planes are disjoint: `pos & neg == 0`.**

Both bits set is invalid. It is not "reserved", not "implementation-defined",
and not silently coerced -- it is an error, and every consumer must treat it as
one. This is deliberate. A lane that tests `pos` first would produce `+x`; a CPU
kernel that accumulates the two planes separately and subtracts would produce
`+x - x = 0`. Defining the state as an error is what stops the software and
hardware paths from disagreeing on a file neither should have accepted. It is
the direct descendant of v0's illegal `0b11` code, which the RTL core already
flagged with a sticky `err`.

**P2 -- padding bits are clear.**

When `cols` is not a multiple of 64, the final beat of each row covers columns
that do not exist. Those bits must be zero in both planes. With

```
tail = (cols % 64 == 0) ? ~0 : (1 << (cols % 64)) - 1
```

every row's last beat must satisfy `(pos | neg) & ~tail == 0`.

P2 is what makes zero padding *exact* rather than merely conventional: a padded
column contributes nothing to the accumulator, so a kernel may process whole
64-column blocks unconditionally and never needs a scalar tail loop. Activation
buffers are sized to `beats_per_row * 64` with zeros in the tail for the same
reason.

Both invariants are cheap to establish at write time and O(payload) to verify,
so they are checked on demand rather than at every open.

### 2.4 Dense payload

`dtype = 0` (F32) and `dtype = 2` (BF16) store `prod(dims)` elements row-major,
little-endian, with `byte_len = prod(dims) * elem_size` where `elem_size` is 4
or 2. `scale` is `1.0`.

Dense tensors carry the embedding table, the LM head, and the normalization
gains. They are not incidental: in BitNet b1.58 2B4T they are **1,315 MB against
521 MB of ternary weights**, so at batch 1 the LM head alone moves 2.5x the
bytes that every ternary projection moves combined. BF16 storage exists to halve
that. It is opt-in (`tritc convert --embed-dtype bf16`) because it changes
numerics and therefore needs its own quality gate, not because it is
experimental.

## 3. A complete file

Two rows of three ternary weights, `W = [[+1, -1, 0], [0, +1, +1]]`, scale 0.5,
config `{"hidden_size":3}`. This is the real output of `TritWriter`, not a
reconstruction.

```
0000  54 52 49 54 01 00 00 00 00 00 00 00 60 f8 e9 b2  |TRIT........`...|
0010  58 bd 4a 4a 11 00 00 00 7b 22 68 69 64 64 65 6e  |X.JJ....{"hidden|
0020  5f 73 69 7a 65 22 3a 33 7d 01 00 00 00 01 00 77  |_size":3}......w|
0030  01 02 00 00 02 00 00 00 03 00 00 00 00 00 00 3f  |...............?|
0040  00 00 00 00 00 00 00 00 00 00 00 00 20 00 00 00  |............ ...|
0050  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  |................|
0060  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  |................|
0070  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  |................|
0080  01 00 00 00 00 00 00 00 02 00 00 00 00 00 00 00  |................|
0090  06 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00  |................|
```

| Offset | Bytes | Meaning |
|---|---|---|
| `0000` | `54 52 49 54` | magic `TRIT` |
| `0004` | `01 00 00 00` | version 1 |
| `0008` | `00 00 00 00` | header_flags 0 |
| `000c` | `60 f8 e9 b2 58 bd 4a 4a` | payload_hash `0x4a4abd58b2e9f860` |
| `0014` | `11 00 00 00` | config_len 17 |
| `0018` | `7b 22 ... 7d` | `{"hidden_size":3}` |
| `0029` | `01 00 00 00` | tensor_count 1 |
| `002d` | `01 00` | name_len 1 |
| `002f` | `77` | name `w` |
| `0030` | `01` | dtype 1 = Trit |
| `0031` | `02` | ndim 2 |
| `0032` | `00 00` | reserved |
| `0034` | `02 00 00 00` | dims[0] = 2 rows |
| `0038` | `03 00 00 00` | dims[1] = 3 cols |
| `003c` | `00 00 00 3f` | scale = 0.5f |
| `0040` | `00 00 00 00` | flags 0 |
| `0044` | `00 ... 00` | offset 0 |
| `004c` | `20 00 00 00 00 00 00 00` | byte_len 32 |
| `0054` | zeros | padding to `payload_start` |
| `0080` | | `payload_start = 128`, a multiple of 64 |

The payload is `2 rows * 1 beat * 16 bytes = 32 bytes`:

| Offset | pos | neg | Decodes to |
|---|---|---|---|
| `0080` | `0x01` (bit 0) | `0x02` (bit 1) | row 0 = `[+1, -1, 0]` |
| `0090` | `0x06` (bits 1,2) | `0x00` | row 1 = `[0, +1, +1]` |

Columns 3..63 of each beat are clear in both planes, satisfying P2.

Regenerate this dump with:

```
cargo run -p trit-core --example hexdump
```

## 4. Validation

The file is untrusted input. Checks split by cost.

**At `open()` -- O(tensor_count), header only.** Fast enough to keep
time-to-first-token unaffected even on a 1.8 GB model.

1. `magic != "TRIT"` -> `bad magic: not a .trit file`
2. `version == 0` -> names the migration command (see below)
3. `version > 1` -> `unsupported .trit version {v}`
4. `header_flags != 0` -> `unknown header flags 0x...`
5. any field read past EOF -> `truncated .trit file`
6. `config_len > 16 MiB`, or config not valid UTF-8
7. `name_len` zero or `> 1024`; duplicate tensor name
8. unknown `dtype`; `ndim` outside `1..=4`; nonzero `reserved0/1` or `flags`
9. shape product overflow
10. `scale` not finite, or `<= 0`
11. `offset % 64 != 0`
12. `offset + byte_len` past end of payload
13. two tensors' payload extents overlap
14. `dtype == Trit` with `ndim != 2`
15. `byte_len != rows * beats_per_row * 16` for ternary tensors
16. `byte_len != prod(dims) * elem_size` for dense tensors

**On request -- O(file size).** `TritFile::open_verified`, `TritFile::verify`,
`tritc verify`, `tritc convert --verify`.

17. P1 violated -> names the tensor, row, beat and column
18. P2 violated -> names the tensor and row
19. `payload_hash` mismatch (skipped when the stored hash is 0)

Because checks 1-16 run up front, every accessor afterwards is infallible by
construction: a span resolved from a validated file cannot be out of bounds, so
the hot path carries no bounds-check-and-propagate machinery. The truncation
suite cuts the example file at every byte of its header region and asserts each
prefix produces an error rather than a panic.

## 5. Migrating from v0

Version 0 stored ternary weights as interleaved 2-bit codes -- `00` = 0,
`01` = +1, `10` = -1, `11` illegal -- four weights per byte, weight `k` at bits
`2k..2k+2` of byte `k/4`. It had no alignment guarantee, no payload hash, no
reserved fields, and no defined tensor ordering.

Opening a v0 file produces:

```
.trit v0 (2-bit interleaved codes) is no longer supported;
re-run 'tritc convert', or 'tritc upgrade --input <old> --output <new>'
```

**Re-converting is the preferred path** when the source checkpoint is at hand:

```
tritc convert --input models/bitnet-2b4t --output models/bitnet-2b4t.trit --verify
```

**`tritc upgrade` covers the case where only the `.trit` survives.** It reads v0
through a frozen parser that lives in `tritc` alone -- `trit-core` contains no v0
code at all, so the runtime cannot grow a legacy path by accident.

The upgrade is a pure re-encoding: same trits, same scales, same tensor set,
only the layout differs. That claim is tested rather than asserted. On the real
BitNet b1.58 2B4T checkpoint, `tritc upgrade` of the v0 file produces a result
**byte-identical** to a fresh `tritc convert` of the original safetensors --
2,084,044,800 trits and 210 scales reproduced exactly.

## 6. What a reader is allowed to assume

For a file that opened successfully:

- `payload_start % 64 == 0`, and every tensor `offset % 64 == 0`.
- A ternary tensor's bytes are exactly `rows * beats_per_row` beats, in row
  order, each beat `{pos, neg}` little-endian.
- Every tensor's extent lies inside the file and overlaps no other tensor.
- Every `scale` is finite and positive.
- Tensor names are unique.

For a file that additionally passed verification:

- `pos & neg == 0` everywhere.
- No padding bit is set past `cols`.
- The payload matches its stored hash, when one is stored.

A reader may **not** assume the file is trusted, that tensor names match any
particular model architecture, or that the embedded config is complete or
consistent with the tensors present. Those are the model loader's business, and
it reports them as model errors rather than format errors.
