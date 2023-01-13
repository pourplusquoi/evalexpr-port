# Porting Rust `evalexpr` Library to C++

Document for Rust library: [evalexpr](https://docs.rs/evalexpr/8.1.0/evalexpr/index.html).

## Example

```cpp
#include <iostream>

#include "port.h"

using namespace evalexpr;

int main() {
    auto expr = port::parse("a + b * f(c)");
    if (!expr) {
        return -1;
    }

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

    auto [res, ok2] = expr->eval_int(*ctx);
    if (!ok2) {
        return -1;
    }

    // prints: answer = 11
    std::cout << "answer = " << res << std::endl;
    return 0;
}
```
