#ifndef TTGAME_EVALEXPR_PORT_CC_FFI_H_
#define TTGAME_EVALEXPR_PORT_CC_FFI_H_

#ifdef __cplusplus
namespace evalexpr::ffi {
extern "C" {
#endif

typedef long long int int64;
typedef unsigned long usize;

typedef const void* ExprPtr;
typedef const void* ValuePtr;
typedef const void* ContextPtr;
typedef void* ContextMutPtr;

struct BorrowedString {
    const char* data;
    usize length;
};

struct OwnedString {
    const char* data;
    usize length;
    usize capacity;
};

struct TupleSlice {
    ValuePtr* data;
    usize length;
    usize capacity;
};

struct BoolOr {
    bool value;
    bool ok;
};

struct IntOr {
    int64 value;
    bool ok;
};

struct FloatOr {
    double value;
    bool ok;
};

struct StringOr {
    OwnedString value;
    bool ok;
};

struct TupleOr {
    TupleSlice value;
    bool ok;
};

struct ValuePtrOr {
    ValuePtr value;
    bool ok;
};

struct ExprPtrOr {
    ExprPtr expr;
    bool ok;
};

struct Capture {
    usize addr;
};

struct Funtion {
    ValuePtrOr(*ptr)(ValuePtr, Capture);
    Capture capture;
};

ExprPtrOr parse(BorrowedString);
ContextMutPtr make_context();

ValuePtr make_bool(bool);
ValuePtr make_int(int64);
ValuePtr make_float(double);
ValuePtr make_string(BorrowedString);

bool set_bool(ContextMutPtr, BorrowedString, bool);
bool set_int(ContextMutPtr, BorrowedString, int64);
bool set_float(ContextMutPtr, BorrowedString, double);
bool set_string(ContextMutPtr, BorrowedString, BorrowedString);
bool set_function(ContextMutPtr, BorrowedString, Funtion);

BoolOr get_bool(ValuePtr);
IntOr get_int(ValuePtr);
FloatOr get_float(ValuePtr);
StringOr get_string(ValuePtr);
TupleOr get_tuple(ValuePtr);

void drop_expr(ExprPtr);
void drop_value(ValuePtr);
void drop_context(ContextMutPtr);
void drop_string_view(OwnedString);
void drop_tuple_slice(TupleSlice);

BoolOr eval_bool(ExprPtr, ContextPtr);
IntOr eval_int(ExprPtr, ContextPtr);
FloatOr eval_float(ExprPtr, ContextPtr);
StringOr eval_string(ExprPtr, ContextPtr);

OwnedString expr_debug_string(ExprPtr);
OwnedString context_debug_string(ContextPtr);
OwnedString value_debug_string(ValuePtr);

#ifdef __cplusplus
}  // extern "C"
}  // namespace evalexpr::ffi
#endif

#endif  // TTGAME_EVALEXPR_PORT_CC_FFI_H_
