use super::EntityUpdate;
use crate::ecs::{ComponentFlags, RuntimeEntity, Transform};
use crate::input::{str_to_key, InputState};
use crate::rendering::drawable::{DrawableProperty, DrawablePrototype};
use crate::scene_management::{Component, Property};
use crate::scripting::{ScriptingData, ScriptingError, ScriptingSource};
use crate::{error, log_scripting as log, ScriptingInstance};
use rune::alloc::fmt::TryWrite;
use rune::diagnostics::EmitError;
use rune::runtime::{AnyObj, Args, Protocol, Shared, Struct, Value, VmError, VmResult};
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{
    Any, BuildError, Context, ContextError, Diagnostics, Module, Source, Sources, ToValue, Vm,
};
use std::cell::RefCell;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
pub type Result<T> = std::result::Result<T, ScriptingError>;

#[cfg(feature = "scene_generation")]
static mut REQUIRED_ASSETS: Vec<String> = Vec::new();

pub struct RuneInstance {
    virtual_machine: Option<Vm>,
}

pub struct RuneComponent {
    pub data: Option<Shared<Struct>>,
}

#[derive(Debug, Clone, Any)]
struct RuneTransform {
    #[rune(get, set)]
    position: Shared<AnyObj>,
    #[rune(get, set, add_assign, sub_assign, mul_assign, div_assign)]
    rotation: f32,
    #[rune(get, set)]
    scale: Shared<AnyObj>,
}

impl RuneTransform {
    fn add_position(&mut self, other: &Vec2) {
        let mut position = as_vec2(self.position.clone());
        position.x += other.x;
        position.y += other.y;
        self.position = Shared::new(AnyObj::new(position).unwrap()).unwrap();
    }
    fn sub_position(&mut self, other: &Vec2) {
        let mut position = as_vec2(self.position.clone());
        position.x -= other.x;
        position.y -= other.y;
        self.position = Shared::new(AnyObj::new(position).unwrap()).unwrap();
    }
    fn mul_position(&mut self, other: Value) {
        let position = as_vec2(self.position.clone());
        let position = mul_vec2(position, other);
        self.position = Shared::new(AnyObj::new(position).unwrap()).unwrap();
    }
    fn div_position(&mut self, other: f64) {
        let mut position = as_vec2(self.position.clone());
        position.x /= other as f32;
        position.y /= other as f32;
        self.position = Shared::new(AnyObj::new(position).unwrap()).unwrap();
    }
    fn add_scale(&mut self, other: &Vec2) {
        let mut scale = as_vec2(self.scale.clone());
        scale.x += other.x;
        scale.y += other.y;
        self.scale = Shared::new(AnyObj::new(scale).unwrap()).unwrap();
    }
    fn sub_scale(&mut self, other: &Vec2) {
        let mut scale = as_vec2(self.scale.clone());
        scale.x -= other.x;
        scale.y -= other.y;
        self.scale = Shared::new(AnyObj::new(scale).unwrap()).unwrap();
    }
    fn mul_scale(&mut self, other: Value) {
        let scale = as_vec2(self.scale.clone());
        let scale = mul_vec2(scale, other);
        self.scale = Shared::new(AnyObj::new(scale).unwrap()).unwrap();
    }
    fn div_scale(&mut self, other: f64) {
        let mut scale = as_vec2(self.scale.clone());
        scale.x /= other as f32;
        scale.y /= other as f32;
        self.scale = Shared::new(AnyObj::new(scale).unwrap()).unwrap();
    }
}

fn mul_vec2(a: Vec2, b: Value) -> Vec2 {
    match b {
        Value::Struct(b) => {
            let b = b.borrow_ref().unwrap();
            let x = b.get("x").unwrap().as_float().unwrap();
            let y = b.get("y").unwrap().as_float().unwrap();
            Vec2::new(a.x * x as f32, a.y * y as f32)
        }
        Value::Float(b) => Vec2::new(a.x * b as f32, a.y * b as f32),
        _ => panic!("Invalid type for Vec2 multiplication"),
    }
}

#[derive(Debug, Clone, Any, PartialEq)]
#[rune(constructor)]
pub struct Vec2 {
    #[rune(get, set, copy, add_assign, sub_assign, mul_assign, div_assign)]
    pub x: f32,
    #[rune(get, set, copy, add_assign, sub_assign, mul_assign, div_assign)]
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_tuple(pos: (f32, f32)) -> Self {
        Self { x: pos.0, y: pos.1 }
    }
    #[rune::function(path = Self::normalize)]
    pub fn normalize(&self) -> Self {
        let length = (self.x * self.x + self.y * self.y).sqrt();
        if length == 0.0 {
            return Self { x: 0.0, y: 0.0 };
        }
        Self {
            x: self.x / length,
            y: self.y / length,
        }
    }

    pub fn as_tuple(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    #[rune::function(protocol = STRING_DISPLAY)]
    fn string_display(&self, f: &mut rune::runtime::Formatter) {
        write!(f, "({}, {})", self.x, self.y).unwrap();
    }

    #[rune::function(protocol = ADD)]
    fn add(&self, other: &Vec2) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }

    #[rune::function(protocol = SUB)]
    fn sub(&self, other: &Vec2) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    #[rune::function(protocol = MUL)]
    fn mul(&self, other: Value) -> Self {
        mul_vec2(self.clone(), other)
    }

    #[rune::function(protocol = DIV)]
    fn div(&self, other: &Vec2) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
        }
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
    pub a: f32,
}

impl Color {
    #[rune::function(path = Self::black)]
    fn black() -> Color {
        Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }

    #[rune::function(path = Self::white)]
    fn white() -> Color {
        Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 1.0,
        }
    }

    #[rune::function(path = Self::rgb)]
    fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    #[rune::function(path = Self::rgba)]
    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    #[rune::function(path = Self::hex)]
    fn hex(hex: String) -> Color {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap() as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap() as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap() as f32 / 255.0;
        let a = if hex.len() > 6 {
            u8::from_str_radix(&hex[6..8], 16).unwrap() as f32 / 255.0
        } else {
            1.0
        };
        Color { r, g, b, a }
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
    drawables: Vec<(Drawable, u32)>,
    remove_drawables: Vec<u32>,
    property_updates: Vec<(u32, String, DrawableProperty)>,
}

impl RuneEntity {
    #[rune::function]
    fn register_drawable(&mut self, drawable: Drawable) -> u32 {
        let id = crate::util::id::get_unique_id();
        self.drawables.push((drawable, id));
        id
    }

    #[rune::function]
    fn unregister_drawable(&mut self, id: u32) {
        self.remove_drawables.push(id);
    }

    #[rune::function]
    fn set_drawable_color(&mut self, drawable: u32, property: &str, color: Color) {
        self.property_updates.push((
            drawable,
            property.to_string(),
            DrawableProperty::Color((&color).into()),
        ));
    }

    #[rune::function]
    fn set_drawable_sprite(&mut self, drawable: u32, property: &str, sprite: &str) {
        self.property_updates.push((
            drawable,
            property.to_string(),
            DrawableProperty::Sprite(sprite.to_string()),
        ));
    }
}

#[derive(Debug, Clone, Any)]
enum Drawable {
    #[rune(constructor)]
    Sprite(#[rune(get, set)] String, #[rune(get, set)] Color),
}

impl Drawable {
    #[rune::function(path = Self::sprite)]
    fn sprite(sprite: &str, color: Color) -> Self {
        Drawable::Sprite(sprite.to_string(), color)
    }
}

impl From<&(Drawable, u32)> for DrawablePrototype {
    fn from(drawable: &(Drawable, u32)) -> Self {
        match &drawable.0 {
            Drawable::Sprite(sprite, color) => DrawablePrototype::Sprite {
                sprite: sprite.to_string(),
                color: [color.r, color.g, color.b, color.a],
                id: drawable.1,
            },
        }
    }
}

impl From<Transform> for RuneTransform {
    fn from(transform: Transform) -> Self {
        match transform {
            Transform::Transform2D {
                position,
                rotation,
                scale,
                ..
            } => RuneTransform {
                position: Shared::new(AnyObj::new(Vec2::from_tuple(position)).unwrap()).unwrap(),
                rotation,
                scale: Shared::new(AnyObj::new(Vec2::from_tuple(scale)).unwrap()).unwrap(),
            },
            Transform::RectTransform { position, .. } => RuneTransform {
                position: Shared::new(AnyObj::new(Vec2::from_tuple(position)).unwrap()).unwrap(),
                rotation: 0.0,
                scale: Shared::new(AnyObj::new(Vec2::new(1.0, 1.0)).unwrap()).unwrap(),
            },
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
        }
    }
}

impl ScriptingData<RuneInstance> for RuneComponent {
    fn from_component_proto(proto: Component, instance: &mut RuneInstance) -> Result<Self> {
        // lets start by initializing a new struct in the runtime
        let data = instance
            .virtual_machine
            .as_mut()
            .unwrap()
            .call([proto.name.as_str(), "new"], ())
            .expect("Error when initializing component. Did you forget to include a 'new' method?");
        let component_data = match data {
            Value::Struct(data) => {
                {
                    let mut component_data = data.clone().into_mut().unwrap();
                    let component_data_obj = component_data.data_mut();
                    // lets assign all of our properties
                    for (key, value) in proto.properties {
                        component_data_obj
                            .insert_value(rune::alloc::String::try_from(key)?, value)
                            .unwrap();
                    }
                }
                Some(data)
            }
            _ => None,
        };

        Ok(RuneComponent {
            data: component_data,
        })
    }

    fn to_component_proto(&self, proto: &Component) -> Result<Component> {
        let mut proto = proto.clone();
        if let Some(data) = &self.data {
            let component_data = data.clone().into_mut().unwrap();
            let component_data_obj = component_data.data();
            for (key, value) in component_data_obj.iter() {
                if key.to_string().starts_with("__") {
                    continue;
                }
                proto
                    .properties
                    .insert(key.to_string(), value.clone().into());
            }
        }
        Ok(proto)
    }

    fn set_property(&mut self, property: &str, value: Property) -> Result<()> {
        if let Some(data) = &mut self.data {
            let mut component_data = data.clone().into_mut().unwrap();
            let component_data_obj = component_data.data_mut();
            component_data_obj
                .insert_value(rune::alloc::String::try_from(property)?, value)
                .unwrap();
        }
        Ok(())
    }
}

impl From<Value> for Property {
    fn from(value: Value) -> Self {
        match value {
            Value::String(value) => Property::String(value.into_ref().unwrap().to_string()),
            Value::Float(value) => Property::Number(value as f32),
            Value::Integer(value) => Property::Number(value as f32),
            Value::Bool(value) => Property::Boolean(value),
            _ => Property::String("".to_string()),
        }
    }
}

impl ToValue for Property {
    fn to_value(self) -> VmResult<Value> {
        match self {
            Property::String(value) => rune::alloc::String::try_from(value.clone())
                .unwrap()
                .to_value(),
            Property::Number(value) => VmResult::Ok(Value::Float(value as f64)),
            Property::Boolean(value) => VmResult::Ok(Value::Bool(value)),
            Property::Array(value) => {
                let mut vec = rune::runtime::Vec::new();
                for item in value {
                    let _ = vec.push_value(item.to_value()).into_result();
                }
                VmResult::Ok(Value::Vec(Shared::new(vec).unwrap()))
            }
            Property::EntityReference(value) => VmResult::Ok(Value::Integer(value.into())),
            Property::ComponentReference(value) => VmResult::Ok(Value::Integer(value.into())),
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
        let core_module = core_module(None)?;
        context.install(&core_module)?;
        let runtime = context.runtime()?;
        let mut rune_sources = Sources::new();
        rune_sources
            .insert(Source::new(
                "loitsu_builtin",
                include_str!("scripts/builtin.rn"),
            )?)
            .unwrap();
        for source in sources {
            rune_sources
                .insert(Source::new(source.name, source.source)?)
                .unwrap();
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

    fn initialize(
        &mut self,
        sources: Vec<ScriptingSource>,
        input_state: Arc<Mutex<InputState>>,
    ) -> Result<()> {
        let mut context = Context::new();
        let core_module = core_module(Some(input_state))?;
        context.install(&core_module)?;
        let runtime = context.runtime()?;
        let mut rune_sources = Sources::new();
        rune_sources
            .insert(Source::new(
                "loitsu_builtin",
                include_str!("scripts/builtin.rn"),
            )?)
            .unwrap();
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

    fn call<T>(&mut self, path: [&str; 2], args: T) -> Result<Value>
    where
        T: Args,
    {
        let result = self
            .virtual_machine
            .as_mut()
            .unwrap()
            .execute(path, args)?
            .complete()
            .into_result()?;
        Ok(result)
    }

    fn run_component_methods<RuneComponent>(
        &mut self,
        entities: &[Rc<RefCell<crate::ecs::RuntimeEntity<Self>>>],
        method: ComponentFlags,
    ) -> Vec<(Arc<Mutex<crate::ecs::RuntimeTransform>>, Vec<EntityUpdate>)> {
        let mut updates = Vec::new();
        for entity in entities {
            let mut entity_updates = Vec::new();
            let mut entity = entity.borrow_mut();
            if entity.is_new {
                if entity.component_flags & ComponentFlags::START == ComponentFlags::START {
                    entity_updates.extend(
                        self.run_component_methods_on_entity(&mut entity, ComponentFlags::START),
                    );
                }
                entity.is_new = false;
            }
            if entity.component_flags & method == method {
                entity_updates.extend(self.run_component_methods_on_entity(&mut entity, method));
            }
            for child in &mut entity.children {
                let mut child = child.borrow_mut();
                let mut entity_updates = Vec::new();
                if child.is_new {
                    if child.component_flags & ComponentFlags::START == ComponentFlags::START {
                        entity_updates.extend(
                            self.run_component_methods_on_entity(&mut child, ComponentFlags::START),
                        );
                    }
                    child.is_new = false;
                }
                if child.component_flags & method == method {
                    entity_updates.extend(self.run_component_methods_on_entity(&mut child, method));
                }
                updates.push((child.transform.clone(), entity_updates));
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
                #[cfg(feature = "editor")]
                ComponentFlags::EDITOR_START,
                #[cfg(feature = "editor")]
                ComponentFlags::EDITOR_DESTROY,
                #[cfg(feature = "editor")]
                ComponentFlags::EDITOR_UPDATE,
            ];
            for method in methods {
                let result = vm.lookup_function([component_name, flags_to_method(method)]);
                if result.is_ok() {
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
            AnyObj::new(Into::<RuneTransform>::into(
                entity.transform.lock().unwrap().transform.clone(),
            ))
            .unwrap(),
        )
        .unwrap(),
        drawables: Vec::new(),
        remove_drawables: Vec::new(),
        property_updates: Vec::new(),
    }
}

impl RuneInstance {
    fn run_component_methods_on_entity(
        &mut self,
        entity: &mut crate::ecs::RuntimeEntity<Self>,
        c_flags: ComponentFlags,
    ) -> Vec<EntityUpdate> {
        #[cfg(feature = "disable_common_ecs_methods")]
        {
            if c_flags == ComponentFlags::START || c_flags == ComponentFlags::DESTROY {
                return Vec::new();
            }
        }
        let mut entity_obj = convert_entity(entity);
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
                        (data.clone(), shared.clone()),
                    );
                    if let Err(error) = r {
                        crate::logging::error(&format!(
                            "Error running method {} on component {}: {}",
                            method, component.component_proto.name, error
                        ));
                    }
                }
                None => {
                    let r = self.virtual_machine.as_mut().unwrap().call(
                        [component.component_proto.name.as_str(), method],
                        (rune::runtime::Value::EmptyTuple, shared.clone()),
                    );
                    if let Err(error) = r {
                        crate::logging::error(&format!(
                            "Error running method {} on component {}: {}",
                            method, component.component_proto.name, error
                        ));
                    }
                }
            }
        }
        let entity_obj = shared.downcast_borrow_ref::<RuneEntity>().unwrap();
        let rune_transform: RuneTransform = entity_obj.clone().transform.take_downcast().unwrap();
        let new_transform: Transform = rune_transform.into();
        let mut rtransform = entity.transform.lock().unwrap();
        if rtransform.transform != new_transform {
            rtransform.has_changed = true;
            rtransform.transform = new_transform;
        }
        let mut updates = Vec::new();
        for drawable in &entity_obj.drawables {
            updates.push(EntityUpdate::AddDrawable(Into::<DrawablePrototype>::into(
                drawable,
            )));
        }
        for drawable in &entity_obj.remove_drawables {
            updates.push(EntityUpdate::RemoveDrawable(drawable.clone()));
        }
        for property_update in &entity_obj.property_updates {
            updates.push(EntityUpdate::SetDrawableProperty(
                property_update.0.clone(),
                property_update.1.clone(),
                property_update.2.clone(),
            ));
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
        #[cfg(feature = "editor")]
        ComponentFlags::EDITOR_START => "editor_start",
        #[cfg(feature = "editor")]
        ComponentFlags::EDITOR_DESTROY => "editor_destroy",
        #[cfg(feature = "editor")]
        ComponentFlags::EDITOR_UPDATE => "editor_update",
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

fn core_module(input_state: Option<Arc<Mutex<InputState>>>) -> Result<Module> {
    let mut m = Module::new();

    // Types
    m.ty::<RuneEntity>()?;
    m.ty::<RuneTransform>()?;
    m.ty::<Vec2>()?;
    m.ty::<Color>()?;
    m.ty::<Drawable>()?;
    m.function_meta(Vec2::string_display)?;
    m.function_meta(Vec2::normalize)?;
    m.function_meta(Vec2::add)?;
    m.function_meta(Vec2::sub)?;
    m.function_meta(Vec2::mul)?;
    m.function_meta(Vec2::div)?;
    m.function_meta(RuneEntity::register_drawable)?;
    m.function_meta(RuneEntity::unregister_drawable)?;
    m.function_meta(Color::hex)?;
    m.function_meta(Color::rgba)?;
    m.function_meta(Color::rgb)?;
    m.function_meta(Color::black)?;
    m.function_meta(Color::white)?;
    m.function_meta(Drawable::sprite)?;

    m.field_function(
        Protocol::ADD_ASSIGN,
        "position",
        RuneTransform::add_position,
    )?;
    m.field_function(
        Protocol::SUB_ASSIGN,
        "position",
        RuneTransform::sub_position,
    )?;
    m.field_function(
        Protocol::MUL_ASSIGN,
        "position",
        RuneTransform::mul_position,
    )?;
    m.field_function(
        Protocol::DIV_ASSIGN,
        "position",
        RuneTransform::div_position,
    )?;
    m.field_function(Protocol::ADD_ASSIGN, "scale", RuneTransform::add_scale)?;
    m.field_function(Protocol::SUB_ASSIGN, "scale", RuneTransform::sub_scale)?;
    m.field_function(Protocol::MUL_ASSIGN, "scale", RuneTransform::mul_scale)?;
    m.field_function(Protocol::DIV_ASSIGN, "scale", RuneTransform::div_scale)?;

    m.function("print", |log: &str| log!("[RUNE] {}", log))
        .build()?;
    m.function("error", |log: &str| error!("[RUNE] {}", log))
        .build()?;
    let start = Arc::new(instant::Instant::now());
    m.function("get_time", move || {
        let duration = start.elapsed();
        duration.as_secs_f64()
    })
    .build()?;
    let input_state_clone = input_state.clone();
    m.function("get_key", move |key: &str| {
        if let Some(input_state) = &input_state_clone {
            let input_state = input_state.lock().unwrap();
            if let Some(virtual_key) = str_to_key(key) {
                return input_state.get_key(virtual_key);
            }
            false
        } else {
            false
        }
    })
    .build()?;
    let input_state_clone = input_state.clone();
    m.function("get_key_down", move |key: &str| {
        if let Some(input_state) = &input_state_clone {
            let input_state = input_state.lock().unwrap();
            if let Some(virtual_key) = str_to_key(key) {
                return input_state.get_key_down(virtual_key);
            }
            false
        } else {
            false
        }
    })
    .build()?;
    let input_state_clone = input_state.clone();
    m.function("get_key_up", move |key: &str| {
        if let Some(input_state) = &input_state_clone {
            let input_state = input_state.lock().unwrap();
            if let Some(virtual_key) = str_to_key(key) {
                return input_state.get_key_up(virtual_key);
            }
            false
        } else {
            false
        }
    })
    .build()?;
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
    m.constant("LOITSU_VERSION", env!("CARGO_PKG_VERSION"))
        .build()?;

    #[cfg(feature = "scene_generation")]
    {
        m.function("require_asset", |asset: &str| {
            unsafe {
                REQUIRED_ASSETS.push(asset.to_string());
            }
            Ok::<(), ()>(())
        })
        .build()?;
    }
    #[cfg(not(feature = "scene_generation"))]
    {
        m.function("require_asset", |_asset: &str| Ok::<(), ()>(()))
            .build()?;
    }
    Ok(m)
}
