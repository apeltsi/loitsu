use rune::{Context, Diagnostics, Source, Sources, ContextError, Module, BuildError, Vm, ToValue, Any};
use rune::runtime::{Value, Struct, VmError, Shared, Args, VmResult, AnyObj};
use rune::diagnostics::EmitError;
use crate::rendering::drawable::{DrawablePrototype, DrawableProperty};
use crate::{ScriptingInstance, log, error};
use crate::scripting::{ScriptingError, ScriptingSource, ScriptingData};
use rune::termcolor::{StandardStream, ColorChoice};
use crate::scene_management::{Property, Component};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use crate::ecs::{ComponentFlags, Transform, RuntimeEntity};
use rune::alloc::fmt::TryWrite;
use uuid::Uuid;
use super::EntityUpdate;
pub type Result<T> = std::result::Result<T, ScriptingError>;

#[cfg(feature = "scene_generation")]
static mut REQUIRED_ASSETS: Vec<String> = Vec::new();

pub struct RuneInstance {
    virtual_machine: Option<Vm>,
}

pub struct RuneComponent {
    pub data: Option<Shared<Struct>>
}

#[derive(Debug, Clone, Any)]
struct RuneTransform {
    #[rune(get, set)]
    position: Shared<AnyObj>,
    #[rune(get, set)]
    rotation: f32,
    #[rune(get, set)]
    scale: Shared<AnyObj>,
}

#[derive(Debug, Clone, Any, PartialEq)]
#[rune(constructor)]
pub struct Vec2 {
    #[rune(get, set, copy)]
    pub x: f32,
    #[rune(get, set, copy)]
    pub y: f32
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y
        }
    }

    pub fn from_tuple(pos: (f32, f32)) -> Self {
        Self {
            x: pos.0,
            y: pos.1
        }
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0
        }
    }

    pub fn as_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[rune::function(protocol = STRING_DISPLAY)]
    fn string_display(&self, f: &mut rune::runtime::Formatter) -> () {
        write!(f, "({}, {})", self.x, self.y).unwrap();
    }
}

impl Display for Vec2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

#[derive(Debug, Clone, Any, PartialEq)]
#[rune(constructor)]
pub struct Color {
    #[rune(get, set, copy)]
    pub r: f32,
    #[rune(get, set, copy)]
    pub g: f32,
    #[rune(get, set, copy)]
    pub b: f32,
    #[rune(get, set, copy)]
    pub a: f32
}

impl Color {
    #[rune::function(path = Self::black)]
    fn black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0
        }
    }

    #[rune::function(path = Self::white)]
    fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0
        }
    }

    #[rune::function(path = Self::rgb)]
    fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color {
            r,
            g,
            b,
            a: 1.0
        }
    }

    #[rune::function(path = Self::rgba)]
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r,
            g,
            b,
            a
        }
    }

    #[rune::function(path = Self::hex)]
    fn hex(hex: String) -> Color {
        let hex = hex.trim_start_matches("#");
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 255.0;
        let a = if hex.len() > 6 {
            u8::from_str_radix(&hex[6..8], 16).unwrap() as f32 / 255.0
        } else {
            1.0
        };
        Color {
            r,
            g,
            b,
            a
        }
    }
} 

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

impl From<&Color> for [f32; 4] {
    fn from(color: &Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

#[derive(Debug, Clone, Any)]
struct RuneEntity {
    #[rune(get)]
    pub name: String,
    // pub components: Vec<RuneComponent>, TODO: NOT IMPLEMENTED
    // pub children: Vec<RuneEntity>, TODO: NOT IMPLEMENTED, should use a string id reference to
    // avoid circular references
    #[rune(get, set)]
    pub transform: Shared<AnyObj>,
    drawables: Vec<(Drawable, Uuid)>,
    remove_drawables: Vec<String>,
    property_updates: Vec<(String, String, DrawableProperty)>
}

impl RuneEntity {
    #[rune::function]
    fn register_drawable(&mut self, drawable: Drawable) -> String {
        let id = crate::util::random::uuid();
        self.drawables.push((drawable, id));
        id.to_string()
    }

    #[rune::function]
    fn unregister_drawable(&mut self, uuid: &str) {
        self.remove_drawables.push(uuid.to_string());
    }

    #[rune::function]
    fn set_drawable_color(&mut self, drawable: &str, property: &str, color: Color) {
        self.property_updates.push((drawable.to_string(), property.to_string(), DrawableProperty::Color((&color).into())));
    }

    #[rune::function]
    fn set_drawable_sprite(&mut self, drawable: &str, property: &str, sprite: &str) {
        self.property_updates.push((drawable.to_string(), property.to_string(), DrawableProperty::Sprite(sprite.to_string())));
    }
}

#[derive(Debug, Clone, Any)]
enum Drawable {
    #[rune(constructor)]
    Sprite (
        #[rune(get, set)]
        String,
        #[rune(get, set)]
        Color
    )
}

impl Drawable {
    #[rune::function(path = Self::sprite)]
    fn sprite(sprite: &str, color: Color) -> Self {
        Drawable::Sprite(sprite.to_string(), color)
    }
}

impl From<&(Drawable, Uuid)> for DrawablePrototype {
    fn from(drawable: &(Drawable, Uuid)) -> Self {
        match &drawable.0 {
            Drawable::Sprite(sprite, color) => {
                DrawablePrototype::Sprite {
                    sprite: sprite.to_string(),
                    color: [color.r, color.g, color.b, color.a],
                    id: drawable.1
                }
            }
        }
    }
}

impl From<Transform> for RuneTransform {
    fn from(transform: Transform) -> Self {
        match transform {
            Transform::Transform2D { position, rotation, scale, .. } => {
                RuneTransform {
                    position: Shared::new(AnyObj::new(Vec2::from_tuple(position)).unwrap()).unwrap(),
                    rotation,
                    scale: Shared::new(AnyObj::new(Vec2::from_tuple(scale)).unwrap()).unwrap()
                }
            },
            Transform::RectTransform { position, .. } => {
                RuneTransform {
                    position: Shared::new(AnyObj::new(Vec2::from_tuple(position)).unwrap()).unwrap(),
                    rotation: 0.0,
                    scale: Shared::new(AnyObj::new(Vec2::new(1.0, 1.0)).unwrap()).unwrap()
                }
            }
        }
    }
}

fn as_vec2(vec: Shared<AnyObj>) -> Vec2 {
    vec.take_downcast().unwrap()
}

impl From<RuneTransform> for Transform {
    fn from(transform: RuneTransform) -> Self {
        Transform::Transform2D {
            position: as_vec2(transform.position).as_tuple(),
            rotation: transform.rotation,
            scale: as_vec2(transform.scale).as_tuple(),
            r#static: false,
            has_changed: false,
            changed_frame: 0
        }
    }
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
        rune_sources.insert(Source::new("loitsu_builtin", include_str!("scripts/builtin.rn"))?).unwrap();
        for source in sources {
            rune_sources.insert(Source::new(source.name, source.source)?).unwrap();
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
        rune_sources.insert(Source::new("loitsu_builtin", include_str!("scripts/builtin.rn"))?).unwrap();
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

    fn run_component_methods<RuneComponent>(&mut self, entities: &mut [crate::ecs::RuntimeEntity<Self>], method: ComponentFlags) -> Vec<(Rc<RefCell<crate::ecs::Transform>>, Vec<EntityUpdate>)> {
        let mut updates = Vec::new();
        for mut entity in entities {
            let mut entity_updates = Vec::new();
            if entity.is_new {
                if entity.component_flags & ComponentFlags::START == ComponentFlags::START {
                    entity_updates.extend(self.run_component_methods_on_entity(&mut entity, ComponentFlags::START));
                }
                entity.is_new = false;
            }
            if entity.component_flags & method == method {
                entity_updates.extend(self.run_component_methods_on_entity(&mut entity, method));
            }
            for child in &mut entity.children {
                if child.is_new {
                    if child.component_flags & ComponentFlags::START == ComponentFlags::START {
                        entity_updates.extend(self.run_component_methods_on_entity(child, ComponentFlags::START));
                    }
                    child.is_new = false;
                }
                if child.component_flags & method == method {
                    entity_updates.extend(self.run_component_methods_on_entity(child, method));
                }
            }
            updates.push((entity.transform.clone(), entity_updates));
        }
        updates
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

fn convert_entity(entity: &RuntimeEntity<RuneInstance>) -> RuneEntity {
    RuneEntity {
        name: entity.get_name().to_string(),
        transform: Shared::new(
            AnyObj::new(
                Into::<RuneTransform>::into(entity.transform.borrow().clone()))
            .unwrap()
            ).unwrap(),
        drawables: Vec::new(),
        remove_drawables: Vec::new(),
        property_updates: Vec::new(),
    }
}

impl RuneInstance {
    fn run_component_methods_on_entity(&mut self, entity: &mut crate::ecs::RuntimeEntity<Self>, c_flags: ComponentFlags) -> Vec<EntityUpdate> {
        #[cfg(feature = "disable_common_ecs_methods")]
        {
            if c_flags == ComponentFlags::START || c_flags == ComponentFlags::DESTROY {
                return Vec::new();
            }
        }
        let mut entity_obj = convert_entity(&entity);
        let (shared, _guard) = unsafe { Shared::from_mut(&mut entity_obj).unwrap() };
        let method = flags_to_method(c_flags);
        for component in &entity.components {
            if component.flags & c_flags != c_flags {
                continue;
            }
            match &component.data.data {
                Some(data) => {
                    let r = self.virtual_machine.as_mut().unwrap().call(
                        [component.component_proto.name.as_str(), method],
                        (data.clone(), shared.clone()));
                    if let Err(error) = r {
                        crate::logging::error(&format!("Error running method {} on component {}: {}", method, component.component_proto.name, error));
                    }
                },
                None => {
                    let r = self.virtual_machine.as_mut().unwrap().call(
                        [component.component_proto.name.as_str(), method],
                        (rune::runtime::Value::EmptyTuple, shared.clone()));
                    if let Err(error) = r {
                        crate::logging::error(&format!("Error running method {} on component {}: {}", method, component.component_proto.name, error));
                    }
                }
            }
        }
        let entity_obj = shared.downcast_borrow_ref::<RuneEntity>().unwrap();
        let rune_transform: RuneTransform = entity_obj.clone().transform.take_downcast().unwrap();
        let mut new_transform: Transform = rune_transform.into();
        if *entity.transform.borrow() != new_transform {
            match new_transform {
                Transform::Transform2D { ref mut has_changed, ..} => {
                    *has_changed = true;
                },
                Transform::RectTransform { ref mut has_changed, .. } => {
                    *has_changed = true;
                }
            }
            *entity.transform.borrow_mut() = new_transform;
        }
        let mut updates = Vec::new();
        for drawable in &entity_obj.drawables {
            updates.push(EntityUpdate::AddDrawable(Into::<DrawablePrototype>::into(drawable)));
        }
        for drawable in &entity_obj.remove_drawables {
            updates.push(EntityUpdate::RemoveDrawable(drawable.clone()));
        }
        for property_update in &entity_obj.property_updates {
            updates.push(EntityUpdate::SetDrawableProperty(property_update.0.clone(), property_update.1.clone(), property_update.2.clone()));
        }
        updates
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

    // Types
    m.ty::<RuneEntity>()?;
    m.ty::<RuneTransform>()?;
    m.ty::<Vec2>()?;
    m.ty::<Color>()?;
    m.ty::<Drawable>()?;
    m.function_meta(Vec2::string_display)?;
    m.function_meta(RuneEntity::register_drawable)?;
    m.function_meta(RuneEntity::unregister_drawable)?;
    m.function_meta(Color::hex)?;
    m.function_meta(Color::rgba)?;
    m.function_meta(Color::rgb)?;
    m.function_meta(Color::black)?;
    m.function_meta(Color::white)?;
    m.function_meta(Drawable::sprite)?;
    
    m.function("print", | log: &str | log!("[RUNE] {}", log)).build()?;
    m.function("error", | log: &str | error!("[RUNE] {}", log)).build()?;
    let start = Arc::new(instant::Instant::now());
    m.function("get_time", move || {
        let duration = start.elapsed();
        duration.as_secs_f64()
    }).build()?;
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
