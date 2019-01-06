// Rust test file autogenerated with cargo build (build/spectests.rs).
// Please do NOT modify it by hand, as it will be reseted on next build.
// Test based on spectests/forward.wast
#![allow(
    warnings,
    dead_code
)]
use wabt::wat2wasm;

use crate::runtime::types::Value;
use crate::webassembly::{compile, instantiate, ResultObject};

use super::_common::{spectest_importobject, NaNCheck};

// Line 1
fn create_module_1() -> ResultObject {
    let module_str = "(module
      (type (;0;) (func (param i32) (result i32)))
      (func (;0;) (type 0) (param i32) (result i32)
        get_local 0
        i32.const 0
        i32.eq
        if (result i32)  ;; label = @1
          i32.const 1
        else
          get_local 0
          i32.const 1
          i32.sub
          call 1
        end)
      (func (;1;) (type 0) (param i32) (result i32)
        get_local 0
        i32.const 0
        i32.eq
        if (result i32)  ;; label = @1
          i32.const 0
        else
          get_local 0
          i32.const 1
          i32.sub
          call 0
        end)
      (export \"even\" (func 0))
      (export \"odd\" (func 1)))
    ";
    let wasm_binary = wat2wasm(module_str.as_bytes()).expect("WAST not valid or malformed");
    instantiate(&wasm_binary[..], &spectest_importobject(), None)
        .expect("WASM can't be instantiated")
}

fn start_module_1(result_object: &mut ResultObject) {
    // TODO Review is explicit start needed? Start now called in runtime::Instance::new()
    //result_object.instance.start();
}

// Line 17
fn c1_l17_action_invoke(result_object: &mut ResultObject) {
    println!("Executing function {}", "c1_l17_action_invoke");
    let result = result_object
        .instance
        .call("even", &[Value::I32(13 as i32)])
        .expect("Missing result in c1_l17_action_invoke");
    assert_eq!(result, Some(Value::I32(0 as i32)));
}

// Line 18
fn c2_l18_action_invoke(result_object: &mut ResultObject) {
    println!("Executing function {}", "c2_l18_action_invoke");
    let result = result_object
        .instance
        .call("even", &[Value::I32(20 as i32)])
        .expect("Missing result in c2_l18_action_invoke");
    assert_eq!(result, Some(Value::I32(1 as i32)));
}

// Line 19
fn c3_l19_action_invoke(result_object: &mut ResultObject) {
    println!("Executing function {}", "c3_l19_action_invoke");
    let result = result_object
        .instance
        .call("odd", &[Value::I32(13 as i32)])
        .expect("Missing result in c3_l19_action_invoke");
    assert_eq!(result, Some(Value::I32(1 as i32)));
}

// Line 20
fn c4_l20_action_invoke(result_object: &mut ResultObject) {
    println!("Executing function {}", "c4_l20_action_invoke");
    let result = result_object
        .instance
        .call("odd", &[Value::I32(20 as i32)])
        .expect("Missing result in c4_l20_action_invoke");
    assert_eq!(result, Some(Value::I32(0 as i32)));
}

#[test]
fn test_module_1() {
    let mut result_object = create_module_1();
    // We group the calls together
    start_module_1(&mut result_object);
    c1_l17_action_invoke(&mut result_object);
    c2_l18_action_invoke(&mut result_object);
    c3_l19_action_invoke(&mut result_object);
    c4_l20_action_invoke(&mut result_object);
}
