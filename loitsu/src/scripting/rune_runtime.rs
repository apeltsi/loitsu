use rune::{Context, Diagnostics, Source, Sources, ContextError, Module, BuildError, Vm, ToValue};
use rune::runtime::{Value, Struct, VmError, Shared, Args, VmResult};
use rune::diagnostics::EmitError;
use crate::ScriptingInstance;
use crate::scripting::{ScriptingError, ScriptingSource, ScriptingData};
use rune::termcolor::{StandardStream, ColorChoice};
use crate::scene_management::Component;
use crate::scene_management::Property;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[cfg(feature = "scene_generation")]
static mut REQUIRED_ASSETS: Vec<String> = Vec::new();

pub struct RuneInstance {
    virtual_machine: Vm,
}

pub struct RuneComponent {
    pub data: Option<Shared<Struct>>
}

impl ScriptingData<RuneInstance> for RuneComponent {
    fn from_component_proto(proto: Component, instance: &mut RuneInstance) -> Result<Self> {
        // lets start by initializing a new struct in the runtime
        let data = instance.virtual_machine.call([proto.name.as_str(), "new"], ())?;
        let component_data = match data {
            Value::Struct(data) => {
                {
                    let mut component_data = data.clone().into_mut().unwrap();
                    let component_data_obj = component_data.data_mut();
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

    fn to_component_proto(&self, proto: &Component) -> Result<Component> {
        let mut proto = proto.clone();
        if let Some(data) = &self.data {
            let component_data = data.clone().into_mut().unwrap();
            let component_data_obj = component_data.data();
            for (key, value) in component_data_obj.iter() {
                proto.properties.insert(key.to_string(), value.clone().into());
            }
        }
        Ok(proto.clone())
    }
}

impl From<Value> for Property {
    fn from(value: Value) -> Self {
        match value {
            Value::String(value) => {
                Property::String(value.into_ref().unwrap().to_string())
            },
            Value::Float(value) => {
                Property::Number(value as f32)
            },
            Value::Integer(value) => {
                Property::Number(value as f32)
            },
            Value::Bool(value) => {
                Property::Boolean(value)
            },
            _ => {
                Property::String("".to_string())
            }
        }
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
                    let _ = vec.push_value(item.to_value()).into_result();
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
    type Data = RuneComponent;
    fn new_with_sources(sources: Vec<ScriptingSource>) -> Result<Self> {
        let mut context = Context::new();
        let core_module = core_module()?;
        context.install(&core_module)?;
        let runtime = context.runtime()?;
        let mut rune_sources = Sources::new();
        for source in sources {
            let _ = rune_sources.insert(Source::new(source.name, source.source)?);
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

    fn run_component_methods<RuneComponent>(&mut self, entities: &[crate::ecs::RuntimeEntity<Self>], method: &str) {
        for entity in entities {
            self.run_component_methods_on_entity(&entity, method);
            for child in &entity.children {
                self.run_component_methods_on_entity(child, method);
            }
        }
    }

}
impl RuneInstance {
    fn run_component_methods_on_entity(&mut self, entity: &crate::ecs::RuntimeEntity<Self>, method: &str) {
        for component in &entity.components {
            match &component.data.data {
                Some(data) => {
                    let _ = self.virtual_machine.call([component.component_proto.name.as_str(), method], (data.clone(), )).unwrap();
                },
                None => {
                    let _ = self.virtual_machine.call([component.component_proto.name.as_str(), method], (rune::runtime::Value::EmptyTuple, )).unwrap();
                }
            }
        }
    }
}
#[cfg(feature = "scene_generation")]
pub unsafe fn get_required_assets() -> Vec<String> {
    REQUIRED_ASSETS.clone()
}

#[cfg(feature = "scene_generation")]
pub unsafe fn clear_required_assets() {
    REQUIRED_ASSETS.clear();
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

    #[cfg(feature = "scene_generation")]
    {
        m.function("require_asset", | asset: &str | {
            unsafe { REQUIRED_ASSETS.push(asset.to_string()); }
            Ok::<(), ()>(())
        }).build()?;
    }
    Ok(m)
}
