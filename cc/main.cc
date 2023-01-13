#include <iostream>

#include "port.h"

using namespace evalexpr;

int main() {
    auto expr = port::parse("a + b * f(c)");
    if (!expr) {
        return -1;
    }
    // std::cout << "expr = " << port::borrow(*expr->debug_string()) << std::endl;
    auto ctx = port::make_context();
    ctx->set_int("a", 1);
    ctx->set_int("b", 2);
    ctx->set_string("c", "hello");
    ctx->set_function("f", [](const port::Value& v) -> port::ValueOr {
        auto [sv, ok] = v.get_string();
        if (!ok) {
            return port::ValueOr::empty();
        }
        return port::ValueOr::from_int(sv->length);
    });
    // std::cout << "ctx = " << port::borrow(*ctx->debug_string()) << std::endl;
    auto [res, ok2] = expr->eval_int(*ctx);
    if (!ok2) {
        return -1;
    }
    std::cout << "answer = " << res << std::endl;
    return 0;
}
