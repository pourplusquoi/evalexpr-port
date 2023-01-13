#include "port.h"

#include <cstring>
#include <memory>
#include <string_view>
#include <utility>
#include <vector>

#include "ffi.h"

namespace evalexpr::port {
namespace {
BorrowedString from(std::string_view s) {
    return BorrowedString{ s.data(), s.size() };
}
}  // namespace

ExprPtr parse(std::string_view s) {
    auto [handle, ok] = ffi::parse(from(s));
    if (!ok) {
        return nullptr;
    }
    return ExprPtr(new Expr{ handle }, ExprDel{});
}

std::string_view borrow(const OwnedString& sv) {
    return std::string_view(sv.data, sv.length);
}

std::vector<Value> borrow(const TupleSlice& ts) {
    std::vector<Value> ret;
    ret.reserve(ts.length);
    auto ptr = ts.data;
    for (size_t i = 0; i < ts.length; i++) {
        ret.emplace_back(Value{ *ptr });
        ptr++;
    }
    return ret;
}

BoolOr Expr::eval_bool(const Context& ctx) const {
    return ffi::eval_bool(handle, ctx.handle);
}

IntOr Expr::eval_int(const Context& ctx) const {
    return ffi::eval_int(handle, ctx.handle);
}

FloatOr Expr::eval_float(const Context& ctx) const {
    return ffi::eval_float(handle, ctx.handle);
}

StringOr Expr::eval_string(const Context& ctx) const {
    auto [s, ok] = ffi::eval_string(handle, ctx.handle);
    if (!ok) {
        return StringOr { nullptr, false };
    }
    return StringOr {
        OwnedStringPtr(new OwnedString{ std::move(s) }, OwnedStringDel{}),
        true,
    };
}

OwnedStringPtr Expr::debug_string() const {
    auto s = ffi::expr_debug_string(handle);
    return OwnedStringPtr(new OwnedString{ std::move(s) }, OwnedStringDel{});
}

TupleOr TupleOr::empty() {
    return TupleOr{ nullptr, false };
}

TupleOr TupleOr::from_ffi(TupleSlice&& t) {
    return TupleOr{
        TupleSlicePtr(new TupleSlice{ std::move(t) }, TupleSliceDel{}),
        true,
    };
}

BoolOr Value::get_bool() const {
    return ffi::get_bool(handle);
}

IntOr Value::get_int() const {
    return ffi::get_int(handle);
}

FloatOr Value::get_float() const {
    return ffi::get_float(handle);
}

StringOr Value::get_string() const {
    auto [s, ok] = ffi::get_string(handle);
    if (!ok) {
        return StringOr{ nullptr, false };
    }
    return StringOr{
        OwnedStringPtr(new OwnedString{ std::move(s) }, OwnedStringDel{}),
        true,
    };
}

TupleOr Value::get_tuple() const {
    auto [t, ok] = ffi::get_tuple(handle);
    if (!ok) {
        return TupleOr::empty();
    }
    return TupleOr::from_ffi(std::move(t));
}

OwnedStringPtr Value::debug_string() const {
    auto s = ffi::value_debug_string(handle);
    return OwnedStringPtr(new OwnedString{ std::move(s) }, OwnedStringDel{});
}

ValueOr ValueOr::empty() {
    return ValueOr{ { nullptr }, false };
}

ValueOr ValueOr::from_bool(bool v) {
    return ValueOr{ make_bool(v), true };
}

ValueOr ValueOr::from_int(int64_t v) {
    return ValueOr{ make_int(v), true };
}

ValueOr ValueOr::from_float(double v) {
    return ValueOr{ make_float(v), true };
}

ValueOr ValueOr::from_string(std::string_view v) {
    return ValueOr{ make_string(v), true };
}

ContextPtr make_context() {
    return ContextPtr(new Context{ ffi::make_context() }, ContextDel{});
}

bool Context::set_bool(std::string_view k, bool v) {
    return ffi::set_bool(handle, from(k), v);
}

bool Context::set_int(std::string_view k, int64_t v) {
    return ffi::set_int(handle, from(k), v);
}

bool Context::set_float(std::string_view k, double v) {
    return ffi::set_float(handle, from(k), v);
}

bool Context::set_string(std::string_view k, std::string_view v) {
    return ffi::set_string(handle, from(k), from(v));
}

bool Context::set_function(std::string_view k, ValueOr(*fn)(const Value&)) {
    return ffi::set_function(handle, from(k), ffi::Funtion{
        [](ffi::ValuePtr ptr, ffi::Capture capture) -> ffi::ValuePtrOr {
            auto fn = reinterpret_cast<ValueOr(*)(const Value&)>(capture.addr);
            auto val = Value{ ptr };
            auto [res, ok] = fn(val);
            if (!ok) {
                return ffi::ValuePtrOr{ nullptr, false };
            }
            return ffi::ValuePtrOr{ res.handle, true };
        },
        { reinterpret_cast<size_t>(fn) },
    });
}

OwnedStringPtr Context::debug_string() const {
    auto s = ffi::context_debug_string(handle);
    return OwnedStringPtr(new OwnedString{ std::move(s) }, OwnedStringDel{});
}

Value make_bool(bool v) {
    return Value{ ffi::make_bool(v) };
}

Value make_int(int64_t v) {
    return Value{ ffi::make_int(v) };
}

Value make_float(double v) {
    return Value{ ffi::make_float(v) };
}

Value make_string(std::string_view v) {
    return Value{ ffi::make_string(from(v)) };
}

}  // namespace evalexpr::port
