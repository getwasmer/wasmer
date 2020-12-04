//! The middleware parses the function binary bytecodes and transform them
//! with the chosen functions.

use smallvec::SmallVec;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::Deref;
use wasmer_types::LocalFunctionIndex;
use wasmer_vm::ModuleInfo;
use wasmparser::{BinaryReader, Operator, Result as WpResult, Type};

/// A shared builder for function middlewares.
pub trait ModuleMiddleware: Debug + Send + Sync {
    /// Generates a `FunctionMiddleware` for a given function.
    ///
    /// Here we generate a separate object for each function instead of executing directly on per-function operators,
    /// in order to enable concurrent middleware application. Takes immutable `&self` because this function can be called
    /// concurrently from multiple compilation threads.
    fn generate_function_middleware(
        &self,
        local_function_index: LocalFunctionIndex,
    ) -> Box<dyn FunctionMiddleware>;

    /// Transforms a `ModuleInfo` struct in-place. This is called before application on functions begins.
    fn transform_module_info(&self, _: &mut ModuleInfo) {}
}

/// A function middleware specialized for a single function.
pub trait FunctionMiddleware: Debug + Send + Sync {
    /// Processes the given operator.
    fn feed<'a>(
        &mut self,
        operator: Operator<'a>,
        state: &mut MiddlewareReaderState<'a>,
    ) -> WpResult<()> {
        state.push_operator(operator);
        Ok(())
    }
}

/// A Middleware binary reader of the WebAssembly structures and types.
#[derive(Debug)]
pub struct MiddlewareBinaryReader<'a> {
    /// Parsing state.
    state: MiddlewareReaderState<'a>,

    /// The backing middleware chain for this reader.
    chain: Vec<Box<dyn FunctionMiddleware>>,
}

/// The state of the binary reader. Exposed to middlewares to push their outputs.
#[derive(Debug)]
pub struct MiddlewareReaderState<'a> {
    /// Raw binary reader.
    inner: BinaryReader<'a>,

    /// The pending operations added by the middleware.
    pending_operations: VecDeque<Operator<'a>>,
}

/// Trait for generating middleware chains from "prototype" (generator) chains.
pub trait ModuleMiddlewareChain {
    /// Generates a function middleware chain.
    fn generate_function_middleware_chain(
        &self,
        local_function_index: LocalFunctionIndex,
    ) -> Vec<Box<dyn FunctionMiddleware>>;

    /// Applies the chain on a `ModuleInfo` struct.
    fn apply_on_module_info(&self, module_info: &mut ModuleInfo);
}

impl<T: Deref<Target = dyn ModuleMiddleware>> ModuleMiddlewareChain for [T] {
    /// Generates a function middleware chain.
    fn generate_function_middleware_chain(
        &self,
        local_function_index: LocalFunctionIndex,
    ) -> Vec<Box<dyn FunctionMiddleware>> {
        self.iter()
            .map(|x| x.generate_function_middleware(local_function_index))
            .collect()
    }

    /// Applies the chain on a `ModuleInfo` struct.
    fn apply_on_module_info(&self, module_info: &mut ModuleInfo) {
        for item in self {
            item.transform_module_info(module_info);
        }
    }
}

impl<'a> MiddlewareReaderState<'a> {
    /// Push an operator.
    pub fn push_operator(&mut self, operator: Operator<'a>) {
        self.pending_operations.push_back(operator);
    }
}

impl<'a> Extend<Operator<'a>> for MiddlewareReaderState<'a> {
    fn extend<I: IntoIterator<Item = Operator<'a>>>(&mut self, iter: I) {
        self.pending_operations.extend(iter);
    }
}

impl<'a: 'b, 'b> Extend<&'b Operator<'a>> for MiddlewareReaderState<'a> {
    fn extend<I: IntoIterator<Item = &'b Operator<'a>>>(&mut self, iter: I) {
        self.pending_operations.extend(iter.into_iter().cloned());
    }
}

impl<'a> MiddlewareBinaryReader<'a> {
    /// Constructs a `MiddlewareBinaryReader` with an explicit starting offset.
    pub fn new_with_offset(data: &'a [u8], original_offset: usize) -> Self {
        let inner = BinaryReader::new_with_offset(data, original_offset);
        Self {
            state: MiddlewareReaderState {
                inner,
                pending_operations: VecDeque::new(),
            },
            chain: vec![],
        }
    }

    /// Replaces the middleware chain with a new one.
    pub fn set_middleware_chain(&mut self, stages: Vec<Box<dyn FunctionMiddleware>>) {
        self.chain = stages;
    }

    /// Read a `count` indicating the number of times to call `read_local_decl`.
    pub fn read_local_count(&mut self) -> WpResult<u32> {
        self.state.inner.read_var_u32()
    }

    /// Read a `(count, value_type)` declaration of local variables of the same type.
    pub fn read_local_decl(&mut self) -> WpResult<(u32, Type)> {
        let count = self.state.inner.read_var_u32()?;
        let ty = self.state.inner.read_type()?;
        Ok((count, ty))
    }

    /// Reads the next available `Operator`.
    pub fn read_operator(&mut self) -> WpResult<Operator<'a>> {
        if self.chain.is_empty() {
            // We short-circuit in case no chain is used
            return self.state.inner.read_operator();
        }

        // Try to fill the `self.pending_operations` buffer, until it is non-empty.
        while self.state.pending_operations.is_empty() {
            let raw_op = self.state.inner.read_operator()?;

            // Fill the initial raw operator into pending buffer.
            self.state.pending_operations.push_back(raw_op);

            // Run the operator through each stage.
            for stage in &mut self.chain {
                // Take the outputs from the previous stage.
                let pending: SmallVec<[Operator<'a>; 2]> =
                    self.state.pending_operations.drain(0..).collect();

                // ...and feed them into the current stage.
                for pending_op in pending {
                    stage.feed(pending_op, &mut self.state)?;
                }
            }
        }

        Ok(self.state.pending_operations.pop_front().unwrap())
    }

    /// Returns the inner `BinaryReader`'s current position.
    pub fn current_position(&self) -> usize {
        self.state.inner.current_position()
    }

    /// Returns the inner `BinaryReader`'s original position (with the offset)
    pub fn original_position(&self) -> usize {
        self.state.inner.original_position()
    }

    /// Returns the number of bytes remaining in the inner `BinaryReader`.
    pub fn bytes_remaining(&self) -> usize {
        self.state.inner.bytes_remaining()
    }

    /// Returns the number of bytes in the `BinaryReader`.
    pub fn len(&self) -> usize {
        self.state.inner.range().end - self.state.inner.range().start
    }

    /// Returns whether the inner `BinaryReader` has reached the end of the file.
    pub fn eof(&self) -> bool {
        self.state.inner.eof()
    }
}
