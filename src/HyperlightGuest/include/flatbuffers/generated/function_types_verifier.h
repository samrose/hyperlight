#ifndef FUNCTION_TYPES_VERIFIER_H
#define FUNCTION_TYPES_VERIFIER_H

/* Generated by flatcc 0.6.2 FlatBuffers schema compiler for C by dvide.com */

#ifndef FUNCTION_TYPES_READER_H
#include "function_types_reader.h"
#endif
#include "flatcc/flatcc_verifier.h"
#include "flatcc/flatcc_prologue.h"

static int Hyperlight_Generated_hlint_verify_table(flatcc_table_verifier_descriptor_t *td);
static int Hyperlight_Generated_hllong_verify_table(flatcc_table_verifier_descriptor_t *td);
static int Hyperlight_Generated_hlstring_verify_table(flatcc_table_verifier_descriptor_t *td);
static int Hyperlight_Generated_hlbool_verify_table(flatcc_table_verifier_descriptor_t *td);
static int Hyperlight_Generated_hlvecbytes_verify_table(flatcc_table_verifier_descriptor_t *td);
static int Hyperlight_Generated_hlsizeprefixedbuffer_verify_table(flatcc_table_verifier_descriptor_t *td);
static int Hyperlight_Generated_hlvoid_verify_table(flatcc_table_verifier_descriptor_t *td);

static int Hyperlight_Generated_ParameterValue_union_verifier(flatcc_union_verifier_descriptor_t *ud)
{
    switch (ud->type) {
    case 1: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlint_verify_table); /* hlint */
    case 2: return flatcc_verify_union_table(ud, Hyperlight_Generated_hllong_verify_table); /* hllong */
    case 3: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlstring_verify_table); /* hlstring */
    case 4: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlbool_verify_table); /* hlbool */
    case 5: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlvecbytes_verify_table); /* hlvecbytes */
    default: return flatcc_verify_ok;
    }
}

static int Hyperlight_Generated_ReturnValue_union_verifier(flatcc_union_verifier_descriptor_t *ud)
{
    switch (ud->type) {
    case 1: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlint_verify_table); /* hlint */
    case 2: return flatcc_verify_union_table(ud, Hyperlight_Generated_hllong_verify_table); /* hllong */
    case 3: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlstring_verify_table); /* hlstring */
    case 4: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlbool_verify_table); /* hlbool */
    case 5: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlvoid_verify_table); /* hlvoid */
    case 6: return flatcc_verify_union_table(ud, Hyperlight_Generated_hlsizeprefixedbuffer_verify_table); /* hlsizeprefixedbuffer */
    default: return flatcc_verify_ok;
    }
}

static int Hyperlight_Generated_hlint_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    int ret;
    if ((ret = flatcc_verify_field(td, 0, 4, 4) /* value */)) return ret;
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hlint_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlint_identifier, &Hyperlight_Generated_hlint_verify_table);
}

static inline int Hyperlight_Generated_hlint_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlint_type_identifier, &Hyperlight_Generated_hlint_verify_table);
}

static inline int Hyperlight_Generated_hlint_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hlint_verify_table);
}

static inline int Hyperlight_Generated_hlint_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hlint_verify_table);
}

static int Hyperlight_Generated_hllong_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    int ret;
    if ((ret = flatcc_verify_field(td, 0, 8, 8) /* value */)) return ret;
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hllong_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hllong_identifier, &Hyperlight_Generated_hllong_verify_table);
}

static inline int Hyperlight_Generated_hllong_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hllong_type_identifier, &Hyperlight_Generated_hllong_verify_table);
}

static inline int Hyperlight_Generated_hllong_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hllong_verify_table);
}

static inline int Hyperlight_Generated_hllong_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hllong_verify_table);
}

static int Hyperlight_Generated_hlstring_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    int ret;
    if ((ret = flatcc_verify_string_field(td, 0, 0) /* value */)) return ret;
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hlstring_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlstring_identifier, &Hyperlight_Generated_hlstring_verify_table);
}

static inline int Hyperlight_Generated_hlstring_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlstring_type_identifier, &Hyperlight_Generated_hlstring_verify_table);
}

static inline int Hyperlight_Generated_hlstring_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hlstring_verify_table);
}

static inline int Hyperlight_Generated_hlstring_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hlstring_verify_table);
}

static int Hyperlight_Generated_hlbool_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    int ret;
    if ((ret = flatcc_verify_field(td, 0, 1, 1) /* value */)) return ret;
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hlbool_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlbool_identifier, &Hyperlight_Generated_hlbool_verify_table);
}

static inline int Hyperlight_Generated_hlbool_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlbool_type_identifier, &Hyperlight_Generated_hlbool_verify_table);
}

static inline int Hyperlight_Generated_hlbool_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hlbool_verify_table);
}

static inline int Hyperlight_Generated_hlbool_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hlbool_verify_table);
}

static int Hyperlight_Generated_hlvecbytes_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    int ret;
    if ((ret = flatcc_verify_vector_field(td, 0, 0, 1, 1, INT64_C(4294967295)) /* value */)) return ret;
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hlvecbytes_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlvecbytes_identifier, &Hyperlight_Generated_hlvecbytes_verify_table);
}

static inline int Hyperlight_Generated_hlvecbytes_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlvecbytes_type_identifier, &Hyperlight_Generated_hlvecbytes_verify_table);
}

static inline int Hyperlight_Generated_hlvecbytes_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hlvecbytes_verify_table);
}

static inline int Hyperlight_Generated_hlvecbytes_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hlvecbytes_verify_table);
}

static int Hyperlight_Generated_hlsizeprefixedbuffer_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    int ret;
    if ((ret = flatcc_verify_field(td, 0, 4, 4) /* size */)) return ret;
    if ((ret = flatcc_verify_vector_field(td, 1, 0, 1, 1, INT64_C(4294967295)) /* value */)) return ret;
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hlsizeprefixedbuffer_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlsizeprefixedbuffer_identifier, &Hyperlight_Generated_hlsizeprefixedbuffer_verify_table);
}

static inline int Hyperlight_Generated_hlsizeprefixedbuffer_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlsizeprefixedbuffer_type_identifier, &Hyperlight_Generated_hlsizeprefixedbuffer_verify_table);
}

static inline int Hyperlight_Generated_hlsizeprefixedbuffer_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hlsizeprefixedbuffer_verify_table);
}

static inline int Hyperlight_Generated_hlsizeprefixedbuffer_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hlsizeprefixedbuffer_verify_table);
}

static int Hyperlight_Generated_hlvoid_verify_table(flatcc_table_verifier_descriptor_t *td)
{
    return flatcc_verify_ok;
}

static inline int Hyperlight_Generated_hlvoid_verify_as_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlvoid_identifier, &Hyperlight_Generated_hlvoid_verify_table);
}

static inline int Hyperlight_Generated_hlvoid_verify_as_typed_root(const void *buf, size_t bufsiz)
{
    return flatcc_verify_table_as_root(buf, bufsiz, Hyperlight_Generated_hlvoid_type_identifier, &Hyperlight_Generated_hlvoid_verify_table);
}

static inline int Hyperlight_Generated_hlvoid_verify_as_root_with_identifier(const void *buf, size_t bufsiz, const char *fid)
{
    return flatcc_verify_table_as_root(buf, bufsiz, fid, &Hyperlight_Generated_hlvoid_verify_table);
}

static inline int Hyperlight_Generated_hlvoid_verify_as_root_with_type_hash(const void *buf, size_t bufsiz, flatbuffers_thash_t thash)
{
    return flatcc_verify_table_as_typed_root(buf, bufsiz, thash, &Hyperlight_Generated_hlvoid_verify_table);
}

#include "flatcc/flatcc_epilogue.h"
#endif /* FUNCTION_TYPES_VERIFIER_H */