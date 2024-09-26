use std::marker::PhantomData;

use bevy::{color::palettes::css, ecs::system::EntityCommand, prelude::*};
use sickle_ui::{prelude::*, SickleUiPlugin};

pub trait SetTitledLabelValueCommand {
    fn label(&mut self, text: impl Into<String>) -> &mut Self;
    fn title(&mut self, text: impl Into<String>) -> &mut Self;
    fn label_font_size(&mut self, size: f32) -> &mut Self;
}

impl SetTitledLabelValueCommand for UiStyle<'_, TitleLabel> {
    fn label(&mut self, text: impl Into<String>) -> &mut Self {
        let v = SetTitledLabelProperties::Label(text.into());
        self.entity_commands().add(v);
        self
    }
    fn title(&mut self, text: impl Into<String>) -> &mut Self {
        let v = SetTitledLabelProperties::Title(text.into());
        self.entity_commands().add(v);
        self
    }
    fn label_font_size(&mut self, size: f32) -> &mut Self {
        let v = SetTitledLabelProperties::LabelFontSize(size);
        self.entity_commands().add(v);
        self
    }
}

pub enum SetTitledLabelProperties<T: Component> {
    Label(String),
    Title(String),
    LabelFontSize(f32),
    Noop(PhantomData<T>),
}

impl EntityCommand for SetTitledLabelProperties<TitleLabel> {
    fn apply(self, id: Entity, world: &mut World) {
        let Some(me) = world.get::<TitleLabel>(id) else {
            return;
        };
        match self {
            SetTitledLabelProperties::Label(label) => {
                let e = me.label.clone();
                if let Some(mut text) = world.entity_mut(e).get_mut::<Text>() {
                    if let Some(section) = text.sections.first_mut() {
                        section.value = label;
                    }
                }
            }
            SetTitledLabelProperties::Title(title) => {
                let e = me.title.clone();
                if let Some(mut text) = world.entity_mut(e).get_mut::<Text>() {
                    if let Some(section) = text.sections.first_mut() {
                        section.value = title;
                    }
                }
            }
            SetTitledLabelProperties::LabelFontSize(size) => {
                let e = me.label.clone();
                if let Some(mut text) = world.entity_mut(e).get_mut::<Text>() {
                    if let Some(section) = text.sections.first_mut() {
                        section.style.font_size = size;
                    }
                }
            }
            SetTitledLabelProperties::Noop(_) => todo!(),
        }
    }
}

#[derive(Component)]
pub struct TitleLabel {
    #[allow(unused)]
    title: Entity,
    label: Entity,
}

pub trait TitledLabelExt {
    fn titled_label(
        &mut self,
        title: impl Into<String>,
        label: impl Into<String>,
    ) -> UiBuilder<Entity>;
}

impl TitledLabelExt for UiBuilder<'_, Entity> {
    fn titled_label(
        &mut self,
        title: impl Into<String>,
        label: impl Into<String>,
    ) -> UiBuilder<Entity> {
        let title: String = title.into();

        let mut t = Entity::PLACEHOLDER;
        let mut l = Entity::PLACEHOLDER;
        let mut builder = self.container(
            (
                NodeBundle::default(),
                Name::new(format!("TitledLabel: {title}")),
            ),
            |container| {
                container.style().flex_direction(FlexDirection::Column);

                t = container
                    .label(LabelConfig::from(title))
                    .style()
                    .align_self(AlignSelf::Start)
                    .font_size(25.)
                    .font_color(css::GRAY.into())
                    .id();

                l = container
                    .label(LabelConfig::from(label))
                    .style()
                    .align_self(AlignSelf::Start)
                    .font_size(20.)
                    .font_color(css::GREEN.into())
                    .id();
            },
        );
        builder.insert(TitleLabel { title: t, label: l });
        builder
    }
}
#[derive(Component)]
pub struct Root {
    pub label_one: Entity,
    pub label_two: Entity,
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Sickle UI -  Simple Editor".into(),
            resolution: (1280., 720.).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(SickleUiPlugin)
    .add_systems(Startup, setup)
    .add_systems(Update, modify_labels)
    .run();
}

fn modify_labels(
    mut commands: Commands,
    q: Query<&Root>,
    time: Res<Time>,
    mut frames: Local<usize>,
) {
    let root_node = q.single();

    *frames += 1;

    commands.style(root_node.label_one);
    // this does not exist, since I have not specifed that this is a TitledLabel and thus it would not compile
    // preventing this simple bug at compile time

    // .label(time.elapsed_seconds().to_string())

    commands
        // gets the style parameter for the container node
        .style_typed::<TitleLabel>(root_node.label_two)
        // this is only available on the UiStyle<'_, TitleLabel> that I get from the typed style, because it
        // makes assumptions on the components on this entity.
        .label(frames.to_string())
        // unfortunatly this wont work, since the font_size will try and set the font of the container node of the
        // of the label widget, but if I wished, I can create a custom command for this type of "widget"
        .font_size((*frames % 100 + 10) as f32)
        // font size label
        .label_font_size((*frames % 100 + 10) as f32);
}

fn setup(mut commands: Commands) {
    // The main camera which will render UI
    commands.spawn((Camera3dBundle {
        camera: Camera {
            order: 1,
            clear_color: Color::BLACK.into(),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(0., 30., 0.))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    },));

    // Use the UI builder with plain bundles and direct setting of bundle props

    let mut label_one = Entity::PLACEHOLDER;
    let mut label_two = Entity::PLACEHOLDER;
    commands
        .ui_builder(UiRoot)
        .container(NodeBundle::default(), |root| {
            root.style()
                .width(Val::Px(100.))
                .height(Val::Px(100.))
                .flex_direction(FlexDirection::Column)
                .justify_content(JustifyContent::Center)
                .align_content(AlignContent::Center);

            label_one = root.titled_label("Elapsed Time", "0").id();
            label_two = root.titled_label("Frames", "0").id();
        })
        .insert(Root {
            label_one,
            label_two,
        });
}
