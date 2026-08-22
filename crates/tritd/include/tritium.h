/* tritium.h -- C API for the Tritium ternary inference runtime.
 *
 * Link against libtritd.so (cdylib) or libtritd.a (staticlib).
 *
 * When linking the STATIC library, also link the C++ runtime and the usual
 * system libraries -- the bundled tokenizer has a C++ component:
 *
 *     cc app.c libtritd.a -lstdc++ -lpthread -ldl -lm -o app
 *
 * The shared library carries its own dependencies and needs none of that.
 *
 * Conventions
 * -----------
 *  - Handles are opaque. No Rust type crosses this boundary.
 *  - Functions returning int: 0 or 1 on success (see each), negative on error.
 *    On error, trit_last_error() returns a message for the calling thread,
 *    valid until that thread's next call into this library.
 *  - Functions returning a pointer: NULL on error, again with trit_last_error().
 *  - Every entry point catches panics internally. A panic unwinding into C
 *    would be undefined behavior, so it is converted to TRIT_ERR_PANIC.
 *
 * Threading
 * ---------
 *  A TritModel may be shared across threads. A TritSession may NOT: it owns
 *  mutable decode state. Create one session per thread.
 *
 * Lifetimes
 * ---------
 *  A TritSession borrows its model's tokenizer. Free every session created from
 *  a model BEFORE freeing that model.
 *
 * This header is written by hand rather than generated. It is kept honest by
 * tests/c_abi.rs, which compiles a C program against it and links it against
 * the real library -- if a signature here drifts from the Rust side, that test
 * fails to compile.
 */
#ifndef TRITIUM_H
#define TRITIUM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TRIT_OK          0
#define TRIT_ERR        -1
#define TRIT_ERR_NULL   -2
#define TRIT_ERR_UTF8   -3
#define TRIT_ERR_PANIC  -4
#define TRIT_ERR_BUFFER -5

/* Backend bits returned by trit_backend_mask(). */
#define TRIT_BACKEND_CPU 1u
#define TRIT_BACKEND_RTL 2u

typedef struct TritModel   TritModel;
typedef struct TritSession TritSession;

typedef struct {
    float    temperature;        /* 0 selects greedy decoding      */
    float    top_p;              /* <= 0 is treated as 1.0         */
    float    repetition_penalty; /* <= 0 is treated as 1.0         */
    uint32_t top_k;              /* 0 disables the top-k cutoff    */
    uint64_t seed;
} TritSamplerParams;

/* --- introspection; all returned strings live for the process lifetime --- */

const char *trit_version(void);
uint32_t    trit_backend_mask(void);
const char *trit_kernel_name(void);

/* Valid until this thread's next call into the library. */
const char *trit_last_error(void);

/* --- model --- */

/* tokenizer_path may be NULL to use tokenizer.json beside the model.
 * backend may be NULL for "cpu". threads = 0 means one per core.
 * Returns NULL on failure. */
TritModel *trit_model_load(const char *model_path,
                           const char *tokenizer_path,
                           const char *backend,
                           uint32_t    threads);

void     trit_model_free(TritModel *m);
uint32_t trit_model_vocab_size(const TritModel *m);
/* Weight bytes touched per decoded token: the bandwidth roofline's numerator. */
uint64_t trit_model_weight_bytes(const TritModel *m);

/* --- session --- */

/* params may be NULL for greedy decoding with default settings. */
TritSession *trit_session_new(TritModel *m, const TritSamplerParams *params);
void         trit_session_free(TritSession *s);
int          trit_session_reset(TritSession *s);

/* Tokenize and run the prompt. len is the byte length; pass 0 to treat utf8 as
 * NUL-terminated. Returns TRIT_OK or a negative code. */
int trit_session_prefill(TritSession *s, const char *utf8, size_t len);

/* Produce the next token.
 *
 * Returns 1 when a token was produced, 0 at end of stream (EOS or a full
 * context), and a negative code on error.
 *
 * out_id may be NULL. out_buf receives the token's UTF-8 text, NUL-terminated;
 * out_len (may be NULL) receives the byte length excluding the NUL.
 *
 * A return of 1 with an EMPTY string is normal, not an error: byte-level BPE
 * splits multi-byte codepoints across tokens, and the runtime holds an
 * incomplete tail until its continuation arrives. */
int trit_session_next(TritSession *s,
                      uint32_t    *out_id,
                      char        *out_buf,
                      size_t       buf_len,
                      size_t      *out_len);

#ifdef __cplusplus
}
#endif
#endif /* TRITIUM_H */
