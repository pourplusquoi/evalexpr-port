#ifndef TTGAME_EVALEXPR_PORT_CC_PORT_H_
#define TTGAME_EVALEXPR_PORT_CC_PORT_H_

#include <memory>
#include <string_view>
#include <vector>

#include "ffi.h"

namespace evalexpr::port {

using ffi::BoolOr;
using ffi::IntOr;
using ffi::FloatOr;
using ffi::BorrowedString;
using ffi::OwnedString;
using ffi::TupleSlice;

struct Value;

struct OwnedStringDel {
    void operator()(OwnedString* s) {
        ffi::drop_string_view(*s);
    }
};

using OwnedStringPtr = std::unique_ptr<OwnedString, OwnedStringDel>;

std::string_view borrow(const OwnedString&);

struct StringOr {
    OwnedStringPtr ptr;
    bool ok;
};

struct TupleSliceDel {
    void operator()(TupleSlice* t) {
        ffi::drop_tuple_slice(*t);
    }
};

using TupleSlicePtr = std::unique_ptr<TupleSlice, TupleSliceDel>;

std::vector<Value> borrow(const TupleSlice&);

struct TupleOr {
    TupleSlicePtr ptr;
    bool ok;
    static TupleOr empty();
    static TupleOr from_ffi(ffi::TupleSlice&&);
};

struct Value {
    ffi::ValuePtr handle;
    Value(const Value&) = delete;
    Value& operator=(const Value&) = delete;
    Value(Value&&) = default;
    Value& operator=(Value&&) = default;
    ~Value() = default;
    BoolOr get_bool() const;
    IntOr get_int() const;
    FloatOr get_float() const;
    StringOr get_string() const;
    TupleOr get_tuple() const;
    OwnedStringPtr debug_string() const;
};

struct ValueOr {
    Value value;
    bool ok;
    static ValueOr empty();
    static ValueOr from_bool(bool);
    static ValueOr from_int(int64_t);
    static ValueOr from_float(double);
    static ValueOr from_string(std::string_view);
};

struct Context {
    ffi::ContextMutPtr handle;
    Context(const Context&) = delete;
    Context& operator=(const Context&) = delete;
    Context(Context&&) = default;
    Context& operator=(Context&&) = default;
    ~Context() = default;
    bool set_bool(std::string_view, bool);
    bool set_int(std::string_view, int64_t);
    bool set_float(std::string_view, double);
    bool set_string(std::string_view, std::string_view);
    bool set_function(std::string_view, ValueOr(*)(const Value&));
    OwnedStringPtr debug_string() const;
};

struct ContextDel {
    void operator()(Context* ctx) {
        ffi::drop_context(ctx->handle);
    }
};

using ContextPtr = std::unique_ptr<Context, ContextDel>;

ContextPtr make_context();

struct Expr {
    ffi::ExprPtr handle;
    BoolOr eval_bool(const Context&) const;
    IntOr eval_int(const Context&) const;
    FloatOr eval_float(const Context&) const;
    StringOr eval_string(const Context&) const;
    OwnedStringPtr debug_string() const;
};

struct ExprDel {
    void operator()(Expr* expr) {
        ffi::drop_expr(expr->handle);
    }
};

using ExprPtr = std::unique_ptr<Expr, ExprDel>;

ExprPtr parse(std::string_view s);

Value make_bool(bool);
Value make_int(int64_t);
Value make_float(double);
Value make_string(std::string_view);

}  // evalexpr::port

#endif  // TTGAME_EVALEXPR_PORT_CC_FFI_H_
