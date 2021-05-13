//! Define `NativeArtifact` to allow compiling and instantiating to be
//! done as separate steps.

use crate::engine::{NativeEngine, NativeEngineInner};
use crate::serialize::{ArchivedModuleMetadata, ModuleMetadata};
use libloading::{Library, Symbol as LibrarySymbol};
use loupe::MemoryUsage;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[cfg(feature = "compiler")]
use std::process::Command;
use std::sync::{Arc, Mutex};
use tempfile::NamedTempFile;
#[cfg(feature = "compiler")]
use tracing::trace;
use wasmer_compiler::{CompileError, Features, OperatingSystem, Symbol, SymbolRegistry, Triple};
#[cfg(feature = "compiler")]
use wasmer_compiler::{
    CompileModuleInfo, Compiler, FunctionBodyData, ModuleEnvironment, ModuleMiddlewareChain,
    ModuleTranslationState,
};
use wasmer_engine::{
    register_frame_info, Artifact, DeserializeError, FunctionExtent, GlobalFrameInfoRegistration,
    InstantiationError, SerializeError,
};
#[cfg(feature = "compiler")]
use wasmer_engine::{Engine, Tunables};
#[cfg(feature = "compiler")]
use wasmer_object::{emit_compilation, emit_data, get_object_for_target};
use wasmer_types::entity::{BoxedSlice, PrimaryMap};
#[cfg(feature = "compiler")]
use wasmer_types::DataInitializer;
use wasmer_types::{
    FunctionIndex, LocalFunctionIndex, MemoryIndex, OwnedDataInitializer, SignatureIndex,
    TableIndex,
};
use wasmer_vm::{
    FuncDataRegistry, FunctionBodyPtr, MemoryStyle, ModuleInfo, TableStyle, VMFunctionBody,
    VMSharedSignatureIndex, VMTrampoline,
};

/// A compiled wasm module, ready to be instantiated.
#[derive(MemoryUsage)]
pub struct NativeArtifact {
    sharedobject_path: PathBuf,
    metadata: ModuleMetadata,
    finished_functions: BoxedSlice<LocalFunctionIndex, FunctionBodyPtr>,
    #[loupe(skip)]
    finished_function_call_trampolines: BoxedSlice<SignatureIndex, VMTrampoline>,
    finished_dynamic_function_trampolines: BoxedSlice<FunctionIndex, FunctionBodyPtr>,
    func_data_registry: Arc<FuncDataRegistry>,
    signatures: BoxedSlice<SignatureIndex, VMSharedSignatureIndex>,
    frame_info_registration: Mutex<Option<GlobalFrameInfoRegistration>>,
}

fn to_compile_error(err: impl Error) -> CompileError {
    CompileError::Codegen(format!("{}", err))
}

const WASMER_METADATA_SYMBOL: &[u8] = b"WASMER_METADATA";
const SERIALIZED_METADATA_LENGTH_OFFSET: usize = 0;
const SERIALIZED_METADATA_CONTENT_OFFSET: usize = 16;

impl NativeArtifact {
    // Mach-O header in Mac
    #[allow(dead_code)]
    const MAGIC_HEADER_MH_CIGAM_64: &'static [u8] = &[207, 250, 237, 254];

    // ELF Magic header for Linux (32 bit)
    #[allow(dead_code)]
    const MAGIC_HEADER_ELF_32: &'static [u8] = &[0x7f, b'E', b'L', b'F', 1];

    // ELF Magic header for Linux (64 bit)
    #[allow(dead_code)]
    const MAGIC_HEADER_ELF_64: &'static [u8] = &[0x7f, b'E', b'L', b'F', 2];

    // COFF Magic header for Windows (64 bit)
    #[allow(dead_code)]
    const MAGIC_HEADER_COFF_64: &'static [u8] = &[b'M', b'Z'];

    /// Check if the provided bytes look like `NativeArtifact`.
    ///
    /// This means, if the bytes look like a shared object file in the target
    /// system.
    pub fn is_deserializable(bytes: &[u8]) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(all(target_pointer_width = "64", target_os="macos"))] {
                bytes.starts_with(Self::MAGIC_HEADER_MH_CIGAM_64)
            }
            else if #[cfg(all(target_pointer_width = "64", target_os="linux"))] {
                bytes.starts_with(Self::MAGIC_HEADER_ELF_64)
            }
            else if #[cfg(all(target_pointer_width = "32", target_os="linux"))] {
                bytes.starts_with(Self::MAGIC_HEADER_ELF_32)
            }
            else if #[cfg(all(target_pointer_width = "64", target_os="windows"))] {
                bytes.starts_with(Self::MAGIC_HEADER_COFF_64)
            }
            else {
                false
            }
        }
    }

    #[cfg(feature = "compiler")]
    /// Generate a compilation
    fn generate_metadata<'data>(
        data: &'data [u8],
        features: &Features,
        compiler: &dyn Compiler,
        tunables: &dyn Tunables,
    ) -> Result<
        (
            CompileModuleInfo,
            PrimaryMap<LocalFunctionIndex, FunctionBodyData<'data>>,
            Vec<DataInitializer<'data>>,
            Option<ModuleTranslationState>,
        ),
        CompileError,
    > {
        let environ = ModuleEnvironment::new();
        let translation = environ.translate(data).map_err(CompileError::Wasm)?;

        // We try to apply the middleware first
        let mut module = translation.module;
        let middlewares = compiler.get_middlewares();
        middlewares.apply_on_module_info(&mut module);

        let memory_styles: PrimaryMap<MemoryIndex, MemoryStyle> = module
            .memories
            .values()
            .map(|memory_type| tunables.memory_style(memory_type))
            .collect();
        let table_styles: PrimaryMap<TableIndex, TableStyle> = module
            .tables
            .values()
            .map(|table_type| tunables.table_style(table_type))
            .collect();

        let compile_info = CompileModuleInfo {
            module: Arc::new(module),
            features: features.clone(),
            memory_styles,
            table_styles,
        };
        Ok((
            compile_info,
            translation.function_body_inputs,
            translation.data_initializers,
            translation.module_translation_state,
        ))
    }

    /// Compile a data buffer into a `NativeArtifact`, which may then be instantiated.
    #[cfg(feature = "compiler")]
    pub fn new(
        engine: &NativeEngine,
        data: &[u8],
        tunables: &dyn Tunables,
    ) -> Result<Self, CompileError> {
        let mut engine_inner = engine.inner_mut();
        let target = engine.target();
        let compiler = engine_inner.compiler()?;
        let (compile_info, function_body_inputs, data_initializers, module_translation) =
            Self::generate_metadata(data, engine_inner.features(), compiler, tunables)?;

        let data_initializers = data_initializers
            .iter()
            .map(OwnedDataInitializer::new)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        let target_triple = target.triple();
        let frame_infos = PrimaryMap::new();

        let mut metadata = ModuleMetadata {
            compile_info,
            prefix: engine_inner.get_prefix(&data),
            data_initializers,
            frame_infos,
        };

        let metadata_serializer = |metadata: &ModuleMetadata| -> Result<Vec<u8>, CompileError> {
            let serialized_data = metadata
                .serialize()
                .map_err(|e| CompileError::Codegen(format!("{:?}", e)))?;
            let mut metadata_binary = vec![0; SERIALIZED_METADATA_CONTENT_OFFSET];
            let mut writable = &mut metadata_binary[SERIALIZED_METADATA_LENGTH_OFFSET..];
            leb128::write::unsigned(&mut writable, serialized_data.len() as u64)
                .expect("Should write number");
            metadata_binary.extend(serialized_data);
            Ok(metadata_binary)
        };

        let symbol_registry = metadata.get_symbol_registry();
        let maybe_obj_bytes = compiler.experimental_native_compile_module(
            &target,
            &metadata.compile_info,
            module_translation.as_ref().unwrap(),
            &function_body_inputs,
            &symbol_registry,
        );

        let object_filepaths = match maybe_obj_bytes {
            Some(native_compilation) => {
                let native_compilation = native_compilation?;
                let mut all_objects = native_compilation
                    .object_files
                    .into_iter()
                    .map(|content| {
                        let file = tempfile::Builder::new()
                            .prefix("wasmer_native")
                            .suffix(".o")
                            .tempfile()
                            .map_err(to_compile_error)?;

                        // Re-open it.
                        let (mut file, filepath) = file.keep().map_err(to_compile_error)?;
                        file.write(&content).map_err(to_compile_error)?;
                        Ok(filepath)
                    })
                    .collect::<Result<Vec<_>, CompileError>>()?;

                // Constructing the metadata object
                let mut obj = get_object_for_target(&target_triple).map_err(to_compile_error)?;
                metadata.frame_infos = native_compilation.frame_infos;
                let metadata_binary = metadata_serializer(&metadata)?;
                emit_data(
                    &mut obj,
                    WASMER_METADATA_SYMBOL,
                    &metadata_binary,
                    std::mem::align_of::<ArchivedModuleMetadata>() as u64,
                )
                .map_err(to_compile_error)?;
                let file = tempfile::Builder::new()
                    .prefix("wasmer_native")
                    .suffix(".o")
                    .tempfile()
                    .map_err(to_compile_error)?;

                // Re-open it.
                let (mut file, metadata_object_filepath) = file.keep().map_err(to_compile_error)?;
                let obj_bytes = obj.write().map_err(to_compile_error)?;

                file.write(&obj_bytes).map_err(to_compile_error)?;

                all_objects.push(metadata_object_filepath);
                all_objects
            }
            None => {
                let compilation = compiler.compile_module(
                    &target,
                    &metadata.compile_info,
                    module_translation.as_ref().unwrap(),
                    function_body_inputs,
                )?;
                let mut obj = get_object_for_target(&target_triple).map_err(to_compile_error)?;
                let compiled_function_infos = compilation.get_frame_info();
                emit_compilation(&mut obj, compilation, &symbol_registry, &target_triple)
                    .map_err(to_compile_error)?;
                metadata.frame_infos = compiled_function_infos;
                let metadata_binary = metadata_serializer(&metadata)?;
                emit_data(
                    &mut obj,
                    WASMER_METADATA_SYMBOL,
                    &metadata_binary,
                    std::mem::align_of::<ArchivedModuleMetadata>() as u64,
                )
                .map_err(to_compile_error)?;
                let file = tempfile::Builder::new()
                    .prefix("wasmer_native")
                    .suffix(".o")
                    .tempfile()
                    .map_err(to_compile_error)?;

                // Re-open it.
                let (mut file, filepath) = file.keep().map_err(to_compile_error)?;
                let obj_bytes = obj.write().map_err(to_compile_error)?;

                file.write(&obj_bytes).map_err(to_compile_error)?;
                vec![filepath]
            }
        };

        let shared_filepath = {
            let suffix = format!(".{}", Self::get_default_extension(&target_triple));
            let shared_file = tempfile::Builder::new()
                .prefix("wasmer_native")
                .suffix(&suffix)
                .tempfile()
                .map_err(to_compile_error)?;
            shared_file
                .into_temp_path()
                .keep()
                .map_err(to_compile_error)?
        };

        let is_cross_compiling = engine_inner.is_cross_compiling();
        let target_triple_str = {
            let into_str = target_triple.to_string();
            // We have to adapt the target triple string, because otherwise
            // Apple's clang will not recognize it.
            if into_str == "aarch64-apple-darwin" {
                "arm64-apple-darwin".to_string()
            } else {
                into_str
            }
        };

        let cross_compiling_args: Vec<String> = if is_cross_compiling {
            vec![
                format!("--target={}", target_triple_str),
                "-fuse-ld=lld".to_string(),
                "-nodefaultlibs".to_string(),
                "-nostdlib".to_string(),
            ]
        } else {
            // We are explicit on the target when the host system is
            // Apple Silicon, otherwise compilation fails.
            if target_triple_str == "arm64-apple-darwin" {
                vec![format!("--target={}", target_triple_str)]
            } else {
                vec![]
            }
        };
        let target_args = match (target_triple.operating_system, is_cross_compiling) {
            (OperatingSystem::Windows, true) => vec!["-Wl,/force:unresolved,/noentry"],
            (OperatingSystem::Windows, false) => vec!["-Wl,-undefined,dynamic_lookup"],
            _ => vec!["-nostartfiles", "-Wl,-undefined,dynamic_lookup"],
        };
        trace!(
            "Compiling for target {} from host {}",
            target_triple_str,
            Triple::host().to_string(),
        );

        let linker = engine_inner.linker().executable();
        let output = Command::new(linker)
            .args(&object_filepaths)
            .arg("-o")
            .arg(&shared_filepath)
            .args(&target_args)
            // .args(&wasmer_symbols)
            .arg("-shared")
            .args(&cross_compiling_args)
            .arg("-v")
            .output()
            .map_err(to_compile_error)?;

        if !output.status.success() {
            return Err(CompileError::Codegen(format!(
                "Shared object file generator failed with:\nstderr:{}\nstdout:{}",
                String::from_utf8_lossy(&output.stderr).trim_end(),
                String::from_utf8_lossy(&output.stdout).trim_end()
            )));
        }
        trace!("gcc command result {:?}", output);
        if is_cross_compiling {
            Self::from_parts_crosscompiled(metadata, shared_filepath)
        } else {
            let lib = unsafe { Library::new(&shared_filepath).map_err(to_compile_error)? };
            Self::from_parts(&mut engine_inner, metadata, shared_filepath, lib)
        }
    }

    /// Get the default extension when serializing this artifact
    pub fn get_default_extension(triple: &Triple) -> &'static str {
        match triple.operating_system {
            OperatingSystem::Windows => "dll",
            OperatingSystem::Darwin | OperatingSystem::Ios | OperatingSystem::MacOSX { .. } => {
                "dylib"
            }
            _ => "so",
        }
    }

    /// Construct a `NativeArtifact` from component parts.
    pub fn from_parts_crosscompiled(
        metadata: ModuleMetadata,
        sharedobject_path: PathBuf,
    ) -> Result<Self, CompileError> {
        let finished_functions: PrimaryMap<LocalFunctionIndex, FunctionBodyPtr> = PrimaryMap::new();
        let finished_function_call_trampolines: PrimaryMap<SignatureIndex, VMTrampoline> =
            PrimaryMap::new();
        let finished_dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBodyPtr> =
            PrimaryMap::new();
        let signatures: PrimaryMap<SignatureIndex, VMSharedSignatureIndex> = PrimaryMap::new();
        Ok(Self {
            sharedobject_path,
            metadata,
            finished_functions: finished_functions.into_boxed_slice(),
            finished_function_call_trampolines: finished_function_call_trampolines
                .into_boxed_slice(),
            finished_dynamic_function_trampolines: finished_dynamic_function_trampolines
                .into_boxed_slice(),
            func_data_registry: Arc::new(FuncDataRegistry::new()),
            signatures: signatures.into_boxed_slice(),
            frame_info_registration: Mutex::new(None),
        })
    }

    /// Construct a `NativeArtifact` from component parts.
    pub fn from_parts(
        engine_inner: &mut NativeEngineInner,
        metadata: ModuleMetadata,
        sharedobject_path: PathBuf,
        lib: Library,
    ) -> Result<Self, CompileError> {
        let mut finished_functions: PrimaryMap<LocalFunctionIndex, FunctionBodyPtr> =
            PrimaryMap::new();
        for (function_local_index, _) in metadata.frame_infos.iter() {
            let function_name = metadata
                .get_symbol_registry()
                .symbol_to_name(Symbol::LocalFunction(function_local_index));
            unsafe {
                // We use a fake function signature `fn()` because we just
                // want to get the function address.
                let func: LibrarySymbol<unsafe extern "C" fn()> = lib
                    .get(function_name.as_bytes())
                    .map_err(to_compile_error)?;
                finished_functions.push(FunctionBodyPtr(
                    func.into_raw().into_raw() as *const VMFunctionBody
                ));
            }
        }

        // Retrieve function call trampolines
        let mut finished_function_call_trampolines: PrimaryMap<SignatureIndex, VMTrampoline> =
            PrimaryMap::with_capacity(metadata.compile_info.module.signatures.len());
        for sig_index in metadata.compile_info.module.signatures.keys() {
            let function_name = metadata
                .get_symbol_registry()
                .symbol_to_name(Symbol::FunctionCallTrampoline(sig_index));
            unsafe {
                let trampoline: LibrarySymbol<VMTrampoline> = lib
                    .get(function_name.as_bytes())
                    .map_err(to_compile_error)?;
                let raw = *trampoline.into_raw();
                finished_function_call_trampolines.push(raw);
            }
        }

        // Retrieve dynamic function trampolines (only for imported functions)
        let mut finished_dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBodyPtr> =
            PrimaryMap::with_capacity(metadata.compile_info.module.num_imported_functions);
        for func_index in metadata
            .compile_info
            .module
            .functions
            .keys()
            .take(metadata.compile_info.module.num_imported_functions)
        {
            let function_name = metadata
                .get_symbol_registry()
                .symbol_to_name(Symbol::DynamicFunctionTrampoline(func_index));
            unsafe {
                let trampoline: LibrarySymbol<unsafe extern "C" fn()> = lib
                    .get(function_name.as_bytes())
                    .map_err(to_compile_error)?;
                finished_dynamic_function_trampolines.push(FunctionBodyPtr(
                    trampoline.into_raw().into_raw() as *const VMFunctionBody,
                ));
            }
        }

        // Compute indices into the shared signature table.
        let signatures = {
            metadata
                .compile_info
                .module
                .signatures
                .values()
                .map(|sig| engine_inner.signatures().register(sig))
                .collect::<PrimaryMap<_, _>>()
        };

        engine_inner.add_library(lib);

        Ok(Self {
            sharedobject_path,
            metadata,
            finished_functions: finished_functions.into_boxed_slice(),
            finished_function_call_trampolines: finished_function_call_trampolines
                .into_boxed_slice(),
            finished_dynamic_function_trampolines: finished_dynamic_function_trampolines
                .into_boxed_slice(),
            func_data_registry: engine_inner.func_data().clone(),
            signatures: signatures.into_boxed_slice(),
            frame_info_registration: Mutex::new(None),
        })
    }

    /// Compile a data buffer into a `NativeArtifact`, which may then be instantiated.
    #[cfg(not(feature = "compiler"))]
    pub fn new(_engine: &NativeEngine, _data: &[u8]) -> Result<Self, CompileError> {
        Err(CompileError::Codegen(
            "Compilation is not enabled in the engine".to_string(),
        ))
    }

    /// Deserialize a `NativeArtifact` from bytes.
    ///
    /// # Safety
    ///
    /// The bytes must represent a serialized WebAssembly module.
    pub unsafe fn deserialize(
        engine: &NativeEngine,
        bytes: &[u8],
    ) -> Result<Self, DeserializeError> {
        if !Self::is_deserializable(&bytes) {
            return Err(DeserializeError::Incompatible(
                "The provided bytes are not in any native format Wasmer can understand".to_string(),
            ));
        }
        // Dump the bytes into a file, so we can read it with our `dlopen`
        let named_file = NamedTempFile::new()?;
        let (mut file, path) = named_file.keep().map_err(|e| e.error)?;
        file.write_all(&bytes)?;
        // We already checked for the header, so we don't need
        // to check again.
        Self::deserialize_from_file_unchecked(&engine, &path)
    }

    /// Deserialize a `NativeArtifact` from a file path.
    ///
    /// # Safety
    ///
    /// The file's content must represent a serialized WebAssembly module.
    pub unsafe fn deserialize_from_file(
        engine: &NativeEngine,
        path: &Path,
    ) -> Result<Self, DeserializeError> {
        let mut file = File::open(&path)?;
        let mut buffer = [0; 5];
        // read up to 5 bytes
        file.read_exact(&mut buffer)?;
        if !Self::is_deserializable(&buffer) {
            return Err(DeserializeError::Incompatible(
                "The provided bytes are not in any native format Wasmer can understand".to_string(),
            ));
        }
        Self::deserialize_from_file_unchecked(&engine, &path)
    }

    /// Deserialize a `NativeArtifact` from a file path (unchecked).
    ///
    /// # Safety
    ///
    /// The file's content must represent a serialized WebAssembly module.
    pub unsafe fn deserialize_from_file_unchecked(
        engine: &NativeEngine,
        path: &Path,
    ) -> Result<Self, DeserializeError> {
        let lib = Library::new(&path).map_err(|e| {
            DeserializeError::CorruptedBinary(format!("Library loading failed: {}", e))
        })?;
        let shared_path: PathBuf = PathBuf::from(path);
        // We use 16 + 1, as the length of the module will take 16 bytes
        // (we construct it like that in `metadata_length`) and we also want
        // to take the first element of the data to construct the slice from
        // it.
        let symbol: LibrarySymbol<*mut [u8; 16 + 1]> =
            lib.get(WASMER_METADATA_SYMBOL).map_err(|e| {
                DeserializeError::CorruptedBinary(format!(
                    "The provided object file doesn't seem to be generated by Wasmer: {}",
                    e
                ))
            })?;
        use std::ops::Deref;
        use std::slice;

        let size = &mut **symbol.deref();
        let mut readable = &size[..];
        let metadata_len = leb128::read::unsigned(&mut readable).map_err(|_e| {
            DeserializeError::CorruptedBinary("Can't read metadata size".to_string())
        })?;
        let metadata_slice: &[u8] = slice::from_raw_parts(
            &size[SERIALIZED_METADATA_CONTENT_OFFSET] as *const u8,
            metadata_len as usize,
        );

        let metadata = ModuleMetadata::deserialize(metadata_slice)?;

        let mut engine_inner = engine.inner_mut();

        Self::from_parts(&mut engine_inner, metadata, shared_path, lib)
            .map_err(DeserializeError::Compiler)
    }

    /// Used in test deserialize metadata is correct
    pub fn metadata(&self) -> &ModuleMetadata {
        &self.metadata
    }
}

impl Artifact for NativeArtifact {
    fn module(&self) -> Arc<ModuleInfo> {
        self.metadata.compile_info.module.clone()
    }

    fn module_ref(&self) -> &ModuleInfo {
        &self.metadata.compile_info.module
    }

    fn module_mut(&mut self) -> Option<&mut ModuleInfo> {
        Arc::get_mut(&mut self.metadata.compile_info.module)
    }

    fn register_frame_info(&self) {
        let mut info = self.frame_info_registration.lock().unwrap();

        if info.is_some() {
            return;
        }

        // The function sizes might not be completely accurate.
        // Because of that, we (reverse) order all the functions by their pointer.
        // [f9, f7, f6, f8...] and calculate their potential function body size by
        // getting the diff in pointers between functions.
        let mut prev_pointer = usize::MAX;

        let fp = self.finished_functions.clone();
        let mut function_pointers = fp.into_iter().collect::<Vec<_>>();
        // Sort the keys by the values in reverse order (function pointers)
        // This way we can get the maximum function lengths (since functions can't collide in memory)
        function_pointers.sort_by(|(_k1, v1), (_k2, v2)| v2.cmp(v1));
        let mut function_pointers = function_pointers
            .into_iter()
            .map(|(index, function_pointer)| {
                let fp = **function_pointer as usize;
                // This assumes we never lay any functions bodies across the usize::MAX..nullptr
                // wrapping point.
                // Which is generally true on most OSes, but certainly doesn't have to be true.
                //
                // Further reading: https://lwn.net/Articles/342330/ \
                // "There is one little problem with that reasoning, though: NULL (zero) can
                // actually be a valid pointer address."
                let current_size_by_ptr = prev_pointer - fp;
                let frame_info = &self.metadata.frame_infos[index];
                prev_pointer = fp;
                // We choose the minimum between the function size given the pointer diff
                // and the emitted size by the address map
                let ptr = function_pointer;
                let length = std::cmp::min(frame_info.address_map.body_len, current_size_by_ptr);
                (index, FunctionExtent { ptr: *ptr, length })
            })
            .collect::<Vec<_>>();
        // We sort them by key, again.
        function_pointers.sort_by(|(k1, _v1), (k2, _v2)| k1.cmp(k2));

        let finished_function_extents = function_pointers
            .into_iter()
            .map(|(_, function_extent)| function_extent)
            .collect::<PrimaryMap<LocalFunctionIndex, _>>()
            .into_boxed_slice();

        let frame_infos = &self.metadata.frame_infos;
        *info = register_frame_info(
            self.metadata.compile_info.module.clone(),
            &finished_function_extents,
            frame_infos.clone(),
        );
    }

    fn features(&self) -> &Features {
        &self.metadata.compile_info.features
    }

    fn data_initializers(&self) -> &[OwnedDataInitializer] {
        &*self.metadata.data_initializers
    }

    fn memory_styles(&self) -> &PrimaryMap<MemoryIndex, MemoryStyle> {
        &self.metadata.compile_info.memory_styles
    }

    fn table_styles(&self) -> &PrimaryMap<TableIndex, TableStyle> {
        &self.metadata.compile_info.table_styles
    }

    fn finished_functions(&self) -> &BoxedSlice<LocalFunctionIndex, FunctionBodyPtr> {
        &self.finished_functions
    }

    fn finished_function_call_trampolines(&self) -> &BoxedSlice<SignatureIndex, VMTrampoline> {
        &self.finished_function_call_trampolines
    }

    fn finished_dynamic_function_trampolines(&self) -> &BoxedSlice<FunctionIndex, FunctionBodyPtr> {
        &self.finished_dynamic_function_trampolines
    }

    fn signatures(&self) -> &BoxedSlice<SignatureIndex, VMSharedSignatureIndex> {
        &self.signatures
    }

    fn func_data_registry(&self) -> &FuncDataRegistry {
        &self.func_data_registry
    }

    fn preinstantiate(&self) -> Result<(), InstantiationError> {
        Ok(())
    }

    /// Serialize a NativeArtifact
    fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        Ok(std::fs::read(&self.sharedobject_path)?)
    }
}
