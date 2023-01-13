#![feature(vec_into_raw_parts)]

use std::{
    marker::PhantomData,
    mem::swap,
    ptr::{null, null_mut},
};

use evalexpr::{
    ContextWithMutableFunctions, ContextWithMutableVariables, EvalexprError, Function,
    HashMapContext, Node, Value,
};

/// # In C++
///
/// ```
/// auto expr = port::parse("a + b * f(c)");
/// if (!expr) {
///     return false;
/// }
/// auto ctx = port::make_context();
/// ctx->set_int("a", 1);
/// ctx->set_int("b", 2);
/// ctx->set_string("c", "hello");
/// ctx->set_function("f", [](const port::Value& v) -> port::ValueOr {
///     auto [sv, ok] = v.get_string();
///     if (!ok) {
///         return port::ValueOr::empty();
///     }
///     return port::ValueOr::from_int(sv->length);
// });
/// auto [res, ok2] = expr->eval_int(*ctx);
/// if (!ok2) {
///     return false;
/// }
/// return true;
/// ```

pub type ExprPtr = *const (); // &Node
pub type ValuePtr = *const (); // &Value
pub type ContextPtr = *const (); // &HashMapContext
pub type ContextMutPtr = *mut (); // &mut HashMapContext

#[repr(C)]
#[derive(Clone)]
pub struct Capture {
    addr: usize,
}

#[repr(C)]
pub struct ExprPtrOr {
    expr: ExprPtr,
    ok: bool,
}

impl Default for ExprPtrOr {
    fn default() -> Self {
        Self {
            expr: null_mut(),
            ok: false,
        }
    }
}

impl From<Node> for ExprPtrOr {
    fn from(node: Node) -> Self {
        Self {
            expr: Box::leak(Box::new(node)) as *const Node as *const (),
            ok: true,
        }
    }
}

#[repr(C)]
pub struct ValuePtrOr {
    value: ValuePtr,
    ok: bool,
}

#[repr(C)]
#[derive(Default)]
pub struct BoolOr {
    value: bool,
    ok: bool,
}

impl From<bool> for BoolOr {
    fn from(v: bool) -> Self {
        Self { value: v, ok: true }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct IntOr {
    value: i64,
    ok: bool,
}

impl From<i64> for IntOr {
    fn from(v: i64) -> Self {
        Self { value: v, ok: true }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct FloatOr {
    value: f64,
    ok: bool,
}

impl From<f64> for FloatOr {
    fn from(v: f64) -> Self {
        Self { value: v, ok: true }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct StringOr {
    value: OwnedString,
    ok: bool,
}

impl From<String> for StringOr {
    fn from(v: String) -> Self {
        Self {
            value: OwnedString::from(v),
            ok: true,
        }
    }
}

#[repr(C)]
pub struct BorrowedString<'a> {
    data: *const u8,
    length: usize,
    phantom: PhantomData<&'a *const u8>,
}

impl<'a> Into<&'a str> for BorrowedString<'a> {
    fn into(self) -> &'a str {
        unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.length)) }
    }
}

impl<'a> Into<String> for BorrowedString<'a> {
    fn into(self) -> String {
        <Self as Into<&str>>::into(self).into()
    }
}

#[repr(C)]
pub struct OwnedString {
    data: *const u8,
    length: usize,
    capacity: usize,
}

impl Default for OwnedString {
    fn default() -> Self {
        Self {
            data: null(),
            length: 0,
            capacity: 0,
        }
    }
}

impl From<String> for OwnedString {
    fn from(s: String) -> Self {
        let (data, length, capacity) = s.into_raw_parts();
        Self {
            data: data,
            length: length,
            capacity: capacity,
        }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct TupleOr {
    value: TupleSlice,
    ok: bool,
}

impl From<Vec<ValuePtr>> for TupleOr {
    fn from(vec: Vec<ValuePtr>) -> Self {
        Self {
            value: TupleSlice::from(vec),
            ok: true,
        }
    }
}

#[repr(C)]
pub struct TupleSlice {
    data: *const ValuePtr,
    length: usize,
    capacity: usize,
}

impl Default for TupleSlice {
    fn default() -> Self {
        Self {
            data: null(),
            length: 0,
            capacity: 0,
        }
    }
}

impl From<Vec<ValuePtr>> for TupleSlice {
    fn from(vec: Vec<ValuePtr>) -> Self {
        let (data, length, capacity) = vec.into_raw_parts();
        Self {
            data: data,
            length: length,
            capacity: capacity,
        }
    }
}

#[repr(C)]
pub struct Closure {
    ptr: unsafe extern "C" fn(ValuePtr, Capture) -> ValuePtrOr,
    capture: Capture,
}

/// Transfers ownership of `ExprPtrOr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn parse(s: BorrowedString) -> ExprPtrOr {
    match evalexpr::build_operator_tree(s.into()) {
        Ok(node) => ExprPtrOr::from(node),
        Err(_) => ExprPtrOr::default(),
    }
}

/// Transfers ownership of `ContextMutPtr`.
#[inline]
#[no_mangle]
pub extern "C" fn make_context() -> ContextMutPtr {
    Box::leak(Box::new(HashMapContext::new())) as *mut HashMapContext as *mut ()
}

/// Takes ownership of `ExprPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn drop_expr(expr: ExprPtr) {
    drop(Box::from_raw(expr as *mut Node));
}

/// Takes ownership of `ContextMutPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn drop_context(ctx: ContextMutPtr) {
    drop(Box::from_raw(ctx as *mut HashMapContext));
}

/// Takes ownership of `OwnedString`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn drop_string_view(s: OwnedString) {
    drop(String::from_raw_parts(
        s.data as *mut u8,
        s.length,
        s.capacity,
    ));
}

/// Takes ownership of `TupleSlice`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn drop_tuple_slice(t: TupleSlice) {
    Vec::from_raw_parts(t.data as *mut ValuePtr, t.length, t.capacity)
        .into_iter()
        .for_each(|x| drop_value(x));
}

/// Takes ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn drop_value(v: ValuePtr) {
    drop(Box::from_raw(v as *mut Value));
}

/// Transfers ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub extern "C" fn make_bool(v: bool) -> ValuePtr {
    Box::leak(Box::new(Value::Boolean(v))) as *const Value as *const ()
}

/// Transfers ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub extern "C" fn make_int(v: i64) -> ValuePtr {
    Box::leak(Box::new(Value::Int(v))) as *const Value as *const ()
}

/// Transfers ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub extern "C" fn make_float(v: f64) -> ValuePtr {
    Box::leak(Box::new(Value::Float(v))) as *const Value as *const ()
}

/// Transfers ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn make_string(v: BorrowedString) -> ValuePtr {
    Box::leak(Box::new(Value::String(v.into()))) as *const Value as *const ()
}

/// Does **NOT** take ownership of `ContextMutPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn set_bool(ctx: ContextMutPtr, s: BorrowedString, v: bool) -> bool {
    (&mut *(ctx as *mut HashMapContext))
        .set_value(s.into(), Value::Boolean(v))
        .is_ok()
}

/// Does **NOT** take ownership of `ContextMutPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn set_int(ctx: ContextMutPtr, s: BorrowedString, v: i64) -> bool {
    (&mut *(ctx as *mut HashMapContext))
        .set_value(s.into(), Value::Int(v))
        .is_ok()
}

/// Does **NOT** take ownership of `ContextMutPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn set_float(ctx: ContextMutPtr, s: BorrowedString, v: f64) -> bool {
    (&mut *(ctx as *mut HashMapContext))
        .set_value(s.into(), Value::Float(v))
        .is_ok()
}

/// Does **NOT** take ownership of `ContextMutPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn set_string(
    ctx: ContextMutPtr,
    s: BorrowedString,
    v: BorrowedString,
) -> bool {
    (&mut *(ctx as *mut HashMapContext))
        .set_value(s.into(), Value::String(v.into()))
        .is_ok()
}

/// Does **NOT** take ownership of `ContextMutPtr`.
/// The callback function does **NOT** take ownership of `ValuePtr`, but transfers
/// ownership of `ValuePtrOr`.
#[no_mangle]
pub unsafe extern "C" fn set_function(
    ctx: ContextMutPtr,
    s: BorrowedString,
    closure: Closure,
) -> bool {
    (&mut *(ctx as *mut HashMapContext))
        .set_function(
            s.into(),
            Function::new(move |v| {
                let ValuePtrOr { value, ok } =
                    (closure.ptr)(v as *const Value as *const (), closure.capture.clone());
                if !ok {
                    Err(EvalexprError::CustomMessage(
                        "failed to invoke function".into(),
                    ))
                } else {
                    let mut val = Value::Empty;
                    swap(&mut val, &mut *(value as *mut Value));
                    drop_value(value);
                    Ok(val)
                }
            }),
        )
        .is_ok()
}

/// Does **NOT** take ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn get_bool(ptr: ValuePtr) -> BoolOr {
    let val = &*(ptr as *mut Value);
    match val.as_boolean() {
        Ok(ret) => BoolOr::from(ret),
        Err(_) => BoolOr::default(),
    }
}

/// Does **NOT** take ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn get_int(ptr: ValuePtr) -> IntOr {
    let val = &*(ptr as *mut Value);
    match val.as_int() {
        Ok(ret) => IntOr::from(ret),
        Err(_) => IntOr::default(),
    }
}

/// Does **NOT** take ownership of `ValuePtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn get_float(ptr: ValuePtr) -> FloatOr {
    let val = &*(ptr as *mut Value);
    match val.as_float() {
        Ok(ret) => FloatOr::from(ret),
        Err(_) => FloatOr::default(),
    }
}

/// Does **NOT** take ownership of `ValuePtr`, transfers ownership of `StringOr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn get_string(ptr: ValuePtr) -> StringOr {
    let val = &*(ptr as *mut Value);
    match val.as_string() {
        Ok(ret) => StringOr::from(ret),
        Err(_) => StringOr::default(),
    }
}

/// Does **NOT** take ownership of `ValuePtr`, transfers ownership of `TupleOr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn get_tuple(ptr: ValuePtr) -> TupleOr {
    let val = &*(ptr as *mut Value);
    match val.as_tuple() {
        Ok(ret) => TupleOr::from(
            ret.into_iter()
                .map(|x| Box::leak(Box::new(x)) as *const Value as *const ())
                .collect::<Vec<_>>(),
        ),
        Err(_) => TupleOr::default(),
    }
}

/// Does **NOT** take ownership of `ExprPtr` and `ContextPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn eval_bool(expr: ExprPtr, ctx: ContextPtr) -> BoolOr {
    let expr = &*(expr as *const Node);
    let ctx = &*(ctx as *const HashMapContext);
    match expr.eval_boolean_with_context(ctx) {
        Ok(ret) => BoolOr::from(ret),
        Err(_) => BoolOr::default(),
    }
}

/// Does **NOT** take ownership of `ExprPtr` and `ContextPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn eval_int(expr: ExprPtr, ctx: ContextPtr) -> IntOr {
    let expr = &*(expr as *const Node);
    let ctx = &*(ctx as *const HashMapContext);
    match expr.eval_int_with_context(ctx) {
        Ok(res) => IntOr::from(res),
        Err(_) => IntOr::default(),
    }
}

/// Does **NOT** take ownership of `ExprPtr` and `ContextPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn eval_float(expr: ExprPtr, ctx: ContextPtr) -> FloatOr {
    let expr = &*(expr as *const Node);
    let ctx = &*(ctx as *const HashMapContext);
    match expr.eval_float_with_context(ctx) {
        Ok(ret) => FloatOr::from(ret),
        Err(_) => FloatOr::default(),
    }
}

/// Does **NOT** take ownership of `ExprPtr` and `ContextPtr`, transfors ownership of `StringOr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn eval_string(expr: ExprPtr, ctx: ContextPtr) -> StringOr {
    let expr = &*(expr as *const Node);
    let ctx = &*(ctx as *const HashMapContext);
    match expr.eval_string_with_context(ctx) {
        Ok(ret) => StringOr::from(ret),
        Err(_) => StringOr::default(),
    }
}

/// Does **NOT** take ownership of `ExprPtr`, transfers ownership of `OwnedString`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn expr_debug_string(expr: ExprPtr) -> OwnedString {
    let expr = &*(expr as *const Node);
    OwnedString::from(format!("{:#?}", expr))
}

/// Does **NOT** take ownership of `ExprPtr`, transfers ownership of `OwnedString`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn context_debug_string(ctx: ContextPtr) -> OwnedString {
    let ctx = &*(ctx as *const HashMapContext);
    OwnedString::from(format!("{:#?}", ctx))
}

/// Does **NOT** take ownership of `ExprPtr`.
#[inline]
#[no_mangle]
pub unsafe extern "C" fn value_debug_string(val: ValuePtr) -> OwnedString {
    let val = &*(val as *const Value);
    OwnedString::from(format!("{:#?}", val))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
