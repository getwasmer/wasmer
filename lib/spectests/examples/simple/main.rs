use std::fs::remove_file;
use wabt::wat2wasm;
use wasmer_clif_backend::CraneliftCompiler;
use wasmer_runtime_core::{
    cache::Cache,
    error::Result,
    global::Global,
    memory::Memory,
    prelude::*,
    table::Table,
    types::{ElementType, MemoryDescriptor, TableDescriptor, Value},
    units::Pages,
};

static EXAMPLE_WASM: &'static [u8] = include_bytes!("simple.wasm");

fn main() -> Result<()> {
    let compiler = CraneliftCompiler::new();
    let wasm_binary = wat2wasm(IMPORT_MODULE.as_bytes()).expect("WAST not valid or malformed");

    let inner_module = {
        use std::time::Instant;

        println!("Creating cache...");

        let start = Instant::now();
        let cache = wasmer_runtime_core::compile_to_cache_with(&wasm_binary, &compiler)?;

        let elapsed = start.elapsed();

        println!("time to compile code: {:?}", elapsed);

        println!("Writing cache to disk...");
        cache.write_to_disk("import_module.wasmer").unwrap();

        println!("Opening cache on disk...");

        let start = Instant::now();

        let cache = Cache::open("import_module.wasmer").unwrap();

        // println!("Loading cache...");
        let inner_module =
            unsafe { wasmer_runtime_core::load_cache_with(cache, &compiler).unwrap() };

        let elapsed = start.elapsed();

        println!("Time to load cache: {:?}", elapsed);

        remove_file("import_module.wasmer").unwrap();
        inner_module
    };

    // let inner_module = wasmer_runtime_core::compile_with(&wasm_binary, &CraneliftCompiler::new())?;

    let memory = Memory::new(MemoryDescriptor {
        minimum: Pages(1),
        maximum: Some(Pages(1)),
        shared: false,
    })
    .unwrap();

    let global = Global::new(Value::I32(42));

    let table = Table::new(TableDescriptor {
        element: ElementType::Anyfunc,
        minimum: 10,
        maximum: None,
    })
    .unwrap();

    memory.direct_access_mut(|slice: &mut [u32]| {
        slice[0] = 42;
    });

    let import_object = imports! {
        "env" => {
            "print_i32" => func!(print_num, [i32] -> [i32]),
            "memory" => memory,
            "global" => global,
            "table" => table,
        },
    };

    let inner_instance = inner_module.instantiate(import_object)?;

    let outer_imports = imports! {
        "env" => inner_instance,
    };

    let outer_module = wasmer_runtime_core::compile_with(EXAMPLE_WASM, &CraneliftCompiler::new())?;
    let outer_instance = outer_module.instantiate(outer_imports)?;
    let ret = outer_instance.call("main", &[Value::I32(42)])?;
    println!("ret: {:?}", ret);

    Ok(())
}

extern "C" fn print_num(n: i32, ctx: &mut vm::Ctx) -> i32 {
    println!("print_num({})", n);

    let memory = ctx.memory(0);

    let a: i32 = memory.read(0).unwrap();

    a + n + 1
}

static IMPORT_MODULE: &str = r#"
(module
  (type $t0 (func (param i32) (result i32)))
  (import "env" "memory" (memory 1 1))
  (import "env" "table" (table 10 anyfunc))
  (import "env" "global" (global i32))
  (import "env" "print_i32" (func $print_i32 (type $t0)))
  (func $print_num (export "print_num") (type $t0) (param $p0 i32) (result i32)
    get_global 0
    call $print_i32))
"#;
