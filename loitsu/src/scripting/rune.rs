use rune::{Context, Diagnostics, Source, Sources, ContextError, Module, Unit, BuildError};
use rune::diagnostics::{Diagnostic, EmitError};
use rune::runtime::RuntimeContext;
use crate::ScriptingInstance;
use crate::scripting::{ScriptingError, ScriptingSource};
use crate::log;
use rune::termcolor::{StandardStream, ColorChoice};

pub type Result<T> = std::result::Result<T, ScriptingError>;

pub struct RuneInstance {
    runtime: RuntimeContext,
    unit: Unit
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

impl From<BuildError> for ScriptingError {
    fn from(error: BuildError) -> Self {
        Self::new(&format!("Rune build error: {}", error))
    }
}

impl From<EmitError> for ScriptingError {
    fn from(error: EmitError) -> Self {
        Self::new(&format!("Rune emit error: {}", error))
    }
}

impl ScriptingInstance for RuneInstance {
    fn new_with_sources(sources: Vec<ScriptingSource>) -> Result<Self> {
        let mut context = Context::new();
        let core_module = core_module()?;
        context.install(&core_module)?;
        let runtime = context.runtime()?;
        let mut rune_sources = Sources::new();
        for source in sources {
            rune_sources.insert(Source::new(source.name, source.source)?);
        }
        let mut diagnostics = Diagnostics::new();
        let result = rune::prepare(&mut rune_sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &rune_sources)?;
        }

        let unit = result?;

        Ok(Self {
            runtime,
            unit
        })
    }
}

fn core_module() -> Result<Module> {
    let mut m = Module::new();
    m.function("print", | log: &str | crate::logging::log(log)).build()?;
    m.function("error", | log: &str | crate::logging::error(log)).build()?;
    // Math Constants
    m.constant("PI", std::f64::consts::PI).build()?;
    m.constant("E", std::f64::consts::E).build()?;

    // Platform Constants
    #[cfg(target_arch = "wasm32")]
    {
        m.constant("PLATFORM", "WEB").build()?;
        m.constant("IS_WEB", true).build()?;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        m.constant("PLATFORM", "DESKTOP").build()?;
        m.constant("IS_WEB", false).build()?;
    }
    m.constant("LOITSU_VERSION", env!("CARGO_PKG_VERSION")).build()?;

    Ok(m)
}
