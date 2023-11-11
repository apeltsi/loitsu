use rune::{Context, Diagnostics, Source, Sources, ContextError, Module, BuildError, Vm, ToValue};
use rune::runtime::{Value, Struct, VmError, Shared, Args, VmResult};
use rune::diagnostics::EmitError;
use crate::ScriptingInstance;
use crate::scripting::{ScriptingError, ScriptingSource, ScriptingData};
use rune::termcolor::{StandardStream, ColorChoice};
use crate::scene_management::Component;
use crate::scene_management::Property;
use std::sync::Arc;
use crate::ecs::ComponentFlags;

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[cfg(feature = "scene_generation")]
static mut REQUIRED_ASSETS: Vec<String> = Vec::new();

pub struct RuneInstance {
    virtual_machine: Option<Vm>,
}

pub struct RuneComponent {
    pub data: Option<Shared<Struct>>
}

impl ScriptingData<RuneInstance> for RuneComponent {
    fn from_component_proto(proto: Component, instance: &mut RuneInstance) -> Result<Self> {
        // lets start by initializing a new struct in the runtime
        let data = instance.virtual_machine.as_mut().unwrap().call([proto.name.as_str(), "new"], ()).expect("Error when initializing component. Did you forget to include a 'new' method?");
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
        Ok(proto)
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
            virtual_machine: Some(vm),
        })
    }

    fn new_uninitialized() -> Result<Self> {
        Ok(Self {
            virtual_machine: None,
        })
    }
    
    fn initialize(&mut self, sources: Vec<ScriptingSource>) -> Result<()> {
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

        self.virtual_machine = Some(vm);

        Ok(())
    }

    fn call<T>(&mut self, path: [&str; 2], args: T) -> Result<Value> where T: Args {
        let result = self.virtual_machine.as_mut().unwrap().execute(path, args)?.complete().into_result()?;
        Ok(result)
    }

    fn run_component_methods<RuneComponent>(&mut self, entities: &mut [crate::ecs::RuntimeEntity<Self>], method: ComponentFlags) {
        for entity in entities {
            if entity.is_new {
                if entity.component_flags & ComponentFlags::START == ComponentFlags::START {
                    self.run_component_methods_on_entity(&entity, ComponentFlags::START);
                }
                entity.is_new = false;
            }
            if entity.component_flags & method == method {
                self.run_component_methods_on_entity(&entity, method);
            }
            for child in &mut entity.children {
                if child.is_new {
                    if child.component_flags & ComponentFlags::START == ComponentFlags::START {
                        self.run_component_methods_on_entity(child, ComponentFlags::START);
                    }
                    child.is_new = false;
                }
                if child.component_flags & method == method {
                    self.run_component_methods_on_entity(child, method);
                }
            }
        }
    }

    fn get_component_flags(&self, component_name: &str) -> ComponentFlags {
        let mut flags = ComponentFlags::EMPTY;
        if let Some(vm) = &self.virtual_machine {
            let methods = vec![
                ComponentFlags::BUILD,
                ComponentFlags::FRAME,
                ComponentFlags::LATE_FRAME,
                ComponentFlags::TICK,
                ComponentFlags::START,
                ComponentFlags::DESTROY,
            ];
            for method in methods {
                let result = vm.lookup_function([component_name, flags_to_method(method)]);
                if let Ok(_) = result {
                    flags |= method;
                }
            }
        }
        flags
    }
}
impl RuneInstance {
    fn run_component_methods_on_entity(&mut self, entity: &crate::ecs::RuntimeEntity<Self>, c_flags: ComponentFlags) {
        let method = flags_to_method(c_flags);
        for component in &entity.components {
            if component.flags & c_flags != c_flags {
                continue;
            }
            match &component.data.data {
                Some(data) => {
                    let r = self.virtual_machine.as_mut().unwrap().call([component.component_proto.name.as_str(), method], (data.clone(), ));
                    if let Err(error) = r {
                        crate::logging::error(&format!("Error running method {} on component {}: {}", method, component.component_proto.name, error));
                    }
                },
                None => {
                    let r = self.virtual_machine.as_mut().unwrap().call([component.component_proto.name.as_str(), method], (rune::runtime::Value::EmptyTuple, ));
                    if let Err(error) = r {
                        crate::logging::error(&format!("Error running method {} on component {}: {}", method, component.component_proto.name, error));
                    }
                }
            }
        }
    }
}

fn flags_to_method(flags: ComponentFlags) -> &'static str {
    match flags {
        ComponentFlags::BUILD => "build",
        ComponentFlags::FRAME => "frame",
        ComponentFlags::LATE_FRAME => "late_frame",
        ComponentFlags::TICK => "tick",
        ComponentFlags::START => "start",
        ComponentFlags::DESTROY => "destroy",
        _ => panic!("Invalid component flags"),
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
    #[cfg(not(feature = "scene_generation"))]
    {
        m.function("require_asset", | _asset: &str | {
            Ok::<(), ()>(())
        }).build()?;
    }
    Ok(m)
}
