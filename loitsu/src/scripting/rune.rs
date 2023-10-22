use rune::{Context, Diagnostics, Source, Sources, ContextError};
use rune::runtime::RuntimeContext;
use std::sync::Arc;
use crate::ScriptingInstance;
use crate::scripting::ScriptingError;

pub type Result<T> = std::result::Result<T, ScriptingError>;

pub struct RuneInstance {
    runtime: Arc<RuntimeContext>
}

impl From<rune::alloc::Error> for ScriptingError {
    fn from(error: rune::alloc::Error) -> Self {
        Self::new(&format!("Rune alloc error: {}", error))
    }
}

impl From<ContextError> for ScriptingError {
    fn from(error: ContextError) -> Self {
        Self::new(&format!("Rune context error: {}", error))
    }
}

impl ScriptingInstance for RuneInstance {
    fn new() -> Result<Self> {
        let context = Context::with_default_modules()?;
        let runtime = Arc::new(context.runtime()?);
        Ok(Self {
            runtime
        })
    }

    fn add_script(&mut self, name: &str, path: &str, script: &str) -> Result<()> {
        let mut sources = Sources::new();
        let source = Source::with_path(name, script, path)?;
        sources.insert(source);
        let mut diagnostics = Diagnostics::new();
        Ok(())
    }
}
