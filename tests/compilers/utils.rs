use std::sync::Arc;
use wasmer::{ModuleMiddleware, Store};
use wasmer_compiler::CompilerConfig;
use wasmer_engine::Engine;
#[cfg(feature = "test-jit")]
use wasmer_engine_jit::JIT;
#[cfg(feature = "test-native")]
use wasmer_engine_native::Native;

pub fn get_compiler(canonicalize_nans: bool) -> impl CompilerConfig {
    cfg_if::cfg_if! {
        if #[cfg(any(
            all(feature = "test-llvm", any(feature = "test-cranelift", feature = "test-singlepass")),
            all(feature = "test-cranelift", feature = "test-singlepass")
        ))] {
            compile_error!("Only one compiler can be selected")
        } else if #[cfg(feature = "test-cranelift")] {
            let mut compiler = wasmer_compiler_cranelift::Cranelift::new();
            compiler.canonicalize_nans(canonicalize_nans);
            compiler.enable_verifier();
            compiler
        } else if #[cfg(feature = "test-llvm")] {
            let mut compiler = wasmer_compiler_llvm::LLVM::new();
            compiler.canonicalize_nans(canonicalize_nans);
            compiler.enable_verifier();
            compiler
        } else if #[cfg(feature = "test-singlepass")] {
            let mut compiler = wasmer_compiler_singlepass::Singlepass::new();
            compiler.canonicalize_nans(canonicalize_nans);
            compiler.enable_verifier();
            compiler
        } else {
            compile_error!("No compiler chosen for the tests")
        }
    }
}

#[cfg(feature = "test-jit")]
pub fn get_engine(canonicalize_nans: bool) -> impl Engine {
    let compiler_config = get_compiler(canonicalize_nans);
    JIT::new(&compiler_config).engine()
}
#[cfg(feature = "test-native")]
pub fn get_engine(canonicalize_nans: bool) -> impl Engine {
    let mut compiler_config = get_compiler(canonicalize_nans);
    Native::new(&mut compiler_config).engine()
}

pub fn get_store(canonicalize_nans: bool) -> Store {
    Store::new(&get_engine(canonicalize_nans))
}

pub fn get_store_with_middlewares<I: Iterator<Item = Arc<dyn ModuleMiddleware>>>(
    middlewares: I,
) -> Store {
    #[allow(unused_mut)]
    let mut compiler_config = get_compiler(false);
    #[cfg(feature = "test-jit")]
    let engine = JIT::new(&compiler_config).middlewares(middlewares).engine();
    #[cfg(feature = "test-native")]
    let engine = Native::new(&mut compiler_config)
        .middlewares(middlewares)
        .engine();
    Store::new(&engine)
}

#[cfg(feature = "test-jit")]
pub fn get_headless_store() -> Store {
    Store::new(&JIT::headless().engine())
}

#[cfg(feature = "test-native")]
pub fn get_headless_store() -> Store {
    Store::new(&Native::headless().engine())
}
