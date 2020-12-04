// Allow unused imports while developing
#![allow(unused_imports, dead_code)]

use crate::compiler::SinglepassCompiler;
use std::sync::Arc;
use wasmer_compiler::{Compiler, CompilerConfig, CpuFeature, Target};
use wasmer_types::Features;

#[derive(Debug, Clone)]
pub struct Singlepass {
    pub(crate) enable_nan_canonicalization: bool,
    pub(crate) enable_stack_check: bool,
}

impl Singlepass {
    /// Creates a new configuration object with the default configuration
    /// specified.
    pub fn new() -> Self {
        Self {
            enable_nan_canonicalization: true,
            enable_stack_check: false,
        }
    }

    /// Enable stack check.
    ///
    /// When enabled, an explicit stack depth check will be performed on entry
    /// to each function to prevent stack overflow.
    ///
    /// Note that this doesn't guarantee deterministic execution across
    /// different platforms.
    pub fn enable_stack_check(&mut self, enable: bool) -> &mut Self {
        self.enable_stack_check = enable;
        self
    }

    /// Enable NaN canonicalization.
    ///
    /// NaN canonicalization is useful when trying to run WebAssembly
    /// deterministically across different architectures.
    pub fn canonicalize_nans(&mut self, enable: bool) -> &mut Self {
        self.enable_nan_canonicalization = enable;
        self
    }
}

impl CompilerConfig for Singlepass {
    fn enable_pic(&mut self) {
        // Do nothing, since singlepass already emits
        // PIC code.
    }

    /// Transform it into the compiler
    fn compiler(&self) -> Box<dyn Compiler + Send> {
        Box::new(SinglepassCompiler::new(&self))
    }

    /// Gets the default features for this compiler in the given target
    fn default_features_for_target(&self, _target: &Target) -> Features {
        let mut features = Features::default();
        features.multi_value(false);
        features
    }
}

impl Default for Singlepass {
    fn default() -> Singlepass {
        Self::new()
    }
}
