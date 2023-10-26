use rune::{Context, Diagnostics, Source, Sources, ContextError, Module, BuildError, Vm, ToValue};
use rune::runtime::{Value, Struct, VmError, Shared, Args, VmResult};
use rune::diagnostics::EmitError;
use crate::ScriptingInstance;
use crate::scripting::{ScriptingError, ScriptingSource, ScriptingData};
use rune::termcolor::{StandardStream, ColorChoice};
use crate::scene_management::Component;
use std::sync::Arc;
use crate::scene_management::Property;

pub type Result<T> = std::result::Result<T, ScriptingError>;

pub struct RuneInstance {
    virtual_machine: Vm,
}

pub struct RuneComponent {
    pub data: Option<Shared<Struct>>
}

impl ScriptingData for RuneComponent {
    fn from_component_proto(proto: Component, instance: &mut RuneInstance) -> Result<Self> {
        // lets start by initializing a new struct in the runtime
        let data = instance.virtual_machine.call([proto.name.as_str(), "new"], ())?;
        let component_data = match data {
            Value::Struct(data) => {
                {
                    let mut component_data = data.clone().into_mut().unwrap();
                    let mut component_data_obj = component_data.data_mut();
                    // lets assign all of our properties
                    for (key, value) in proto.properties {
                        component_data_obj.insert_value(rune::alloc::String::try_from(key)?, value).unwrap();
                    }
                }
                Some(data)
            },
            _ => {
                None
            }
        };

        Ok(RuneComponent {
            data: component_data
        })
    }
}

impl ToValue for Property {
    fn to_value(self) -> VmResult<Value> {
        match self {
            Property::String(value) => {
                rune::alloc::String::try_from(value.clone()).unwrap().to_value()
            },
            Property::Number(value) => {
                VmResult::Ok(Value::Float(value.clone() as f64))
            },
            Property::Boolean(value) => {
                VmResult::Ok(Value::Bool(value.clone()))
            },
            Property::Array(value) => {
                let mut vec = rune::runtime::Vec::new();
                for item in value {
                    vec.push_value(item.to_value()).into_result();
                }
                VmResult::Ok(Value::Vec(Shared::new(vec).unwrap()))
            },
            Property::EntityReference(value) => {
                rune::alloc::String::try_from(value.clone()).unwrap().to_value()
            },
            Property::ComponentReference(value) => {
                rune::alloc::String::try_from(value.clone()).unwrap().to_value()
            },
        }
    }
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

impl From<VmError> for ScriptingError {
    fn from(error: VmError) -> Self {
        Self::new(&format!("Rune vm error: {}", error))
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
        let mut diagnostics = Diagnostics::without_warnings();
        let result = rune::prepare(&mut rune_sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();

        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &rune_sources)?;
        }

        let unit = result?;
        let unit = Arc::new(unit);
        let runtime_context = Arc::new(runtime);
        let vm = Vm::new(runtime_context, unit);

        Ok(Self {
            virtual_machine: vm,
        })
    }

    fn call<T>(&mut self, path: [&str; 2], args: T) -> Result<Value> where T: Args {
        let result = self.virtual_machine.execute(path, args)?.complete().into_result()?;
        Ok(result)
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
