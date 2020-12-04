//! Support for compiling with Singlepass.
// Allow unused imports while developing.
#![allow(unused_imports, dead_code)]

use crate::codegen_x64::{
    gen_import_call_trampoline, gen_std_dynamic_import_trampoline, gen_std_trampoline,
    CodegenError, FuncGen,
};
use crate::config::Singlepass;
use rayon::prelude::{
    IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator,
};
use std::sync::Arc;
use wasmer_compiler::wasmparser::BinaryReaderError;
use wasmer_compiler::FunctionBody;
use wasmer_compiler::TrapInformation;
use wasmer_compiler::{Compilation, CompileError, CompiledFunction, Compiler, SectionIndex};
use wasmer_compiler::{
    CompileModuleInfo, CompilerConfig, MiddlewareBinaryReader, ModuleTranslationState, Target,
};
use wasmer_types::entity::{EntityRef, PrimaryMap};
use wasmer_types::{FunctionIndex, FunctionType, LocalFunctionIndex, MemoryIndex, TableIndex};
use wasmer_vm::{ModuleInfo, TrapCode, VMOffsets};

/// A compiler that compiles a WebAssembly module with Singlepass.
/// It does the compilation in one pass
pub struct SinglepassCompiler {
    config: Singlepass,
}

impl SinglepassCompiler {
    /// Creates a new Singlepass compiler
    pub fn new(config: &Singlepass) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Gets the config for this Compiler
    fn config(&self) -> &Singlepass {
        &self.config
    }
}

impl Compiler for SinglepassCompiler {
    /// Compile the module using Singlepass, producing a compilation result with
    /// associated relocations.
    fn compile_module(
        &self,
        _target: &Target,
        compile_info: &mut CompileModuleInfo,
        _module_translation: &ModuleTranslationState,
        function_body_inputs: PrimaryMap<LocalFunctionIndex, MiddlewareBinaryReader>,
    ) -> Result<Compilation, CompileError> {
        if compile_info.features.multi_value {
            return Err(CompileError::UnsupportedFeature("multivalue".to_string()));
        }
        let memory_styles = &compile_info.memory_styles;
        let table_styles = &compile_info.table_styles;
        let vmoffsets = VMOffsets::new(8, &compile_info.module);
        let module = &compile_info.module;
        let import_trampolines: PrimaryMap<SectionIndex, _> = (0..module.num_imported_functions)
            .map(FunctionIndex::new)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|i| {
                gen_import_call_trampoline(&vmoffsets, i, &module.signatures[module.functions[i]])
            })
            .collect::<Vec<_>>()
            .into_iter()
            .collect();
        let functions = function_body_inputs
            .into_iter()
            .collect::<Vec<_>>()
            .par_iter_mut()
            .map(|(i, reader)| {
                // This local list excludes arguments.
                let mut locals = vec![];
                let num_locals = reader.read_local_count().map_err(to_compile_error)?;
                for _ in 0..num_locals {
                    let (count, ty) = reader.read_local_decl().map_err(to_compile_error)?;
                    for _ in 0..count {
                        locals.push(ty);
                    }
                }

                let mut generator = FuncGen::new(
                    module,
                    &self.config,
                    &vmoffsets,
                    &memory_styles,
                    &table_styles,
                    *i,
                    &locals,
                )
                .map_err(to_compile_error)?;

                while generator.has_control_frames() {
                    let op = reader.read_operator().map_err(to_compile_error)?;
                    generator.feed_operator(op).map_err(to_compile_error)?;
                }

                Ok(generator.finalize())
            })
            .collect::<Result<Vec<CompiledFunction>, CompileError>>()?
            .into_iter()
            .collect::<PrimaryMap<LocalFunctionIndex, CompiledFunction>>();

        let function_call_trampolines = module
            .signatures
            .values()
            .collect::<Vec<_>>()
            .par_iter()
            .cloned()
            .map(gen_std_trampoline)
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<PrimaryMap<_, _>>();

        let dynamic_function_trampolines = module
            .imported_function_types()
            .collect::<Vec<_>>()
            .par_iter()
            .map(|func_type| gen_std_dynamic_import_trampoline(&vmoffsets, &func_type))
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<PrimaryMap<FunctionIndex, FunctionBody>>();

        Ok(Compilation::new(
            functions,
            import_trampolines,
            function_call_trampolines,
            dynamic_function_trampolines,
            None,
        ))
    }
}

trait ToCompileError {
    fn to_compile_error(self) -> CompileError;
}

impl ToCompileError for BinaryReaderError {
    fn to_compile_error(self) -> CompileError {
        CompileError::Codegen(self.message().into())
    }
}

impl ToCompileError for CodegenError {
    fn to_compile_error(self) -> CompileError {
        CompileError::Codegen(self.message)
    }
}

fn to_compile_error<T: ToCompileError>(x: T) -> CompileError {
    x.to_compile_error()
}
