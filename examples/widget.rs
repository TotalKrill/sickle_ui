use bevy::{color::palettes::css, ecs::system::EntityCommand, prelude::*};
use sickle_ui::{prelude::*, SickleUiPlugin};

// Not necessary, but very nice for this kind of "widget" work
use extension_trait::extension_trait;
#[extension_trait]
/// Spawning function on general uibuilder
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

#[extension_trait]
impl TitledLabelSubExt for UiBuilder<'_, (Entity, TitleLabel)> {
    // access the different subwidgets here
    fn value(&mut self, builder: impl FnOnce(&mut UiBuilder<'_, Entity>)) -> &mut Self {
        let e = self.content().label;
        let mut vb = self.commands().ui_builder(e);
        builder(&mut vb);
        self
    }

    // access the different subwidgets here
    fn title(&mut self, builder: impl FnOnce(&mut UiBuilder<'_, Entity>)) -> &mut Self {
        let e = self.content().title;
        let mut vb = self.commands().ui_builder(e);
        builder(&mut vb);
        self
    }
}

#[extension_trait]
impl SetTextExt for UiStyle<'_> {
    fn set_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.entity_commands().add(SetText(text.into()));
        self
    }
}

pub struct SetText(String);

impl EntityCommand for SetText {
    fn apply(self, id: Entity, world: &mut World) {
        if let Some(mut text) = world.entity_mut(id).get_mut::<Text>() {
            if let Some(section) = text.sections.first_mut() {
                section.value = self.0;
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TitleLabel {
    #[allow(unused)]
    title: Entity,
    label: Entity,
}

#[derive(Component, Clone, Copy)]
pub struct Root {
    pub label_one: Entity,
    pub label_two: Entity,
}

#[extension_trait]
impl RootExt for UiBuilder<'_, (Entity, Root)> {
    fn titled_label_one(
        &mut self,
        // we borrow the query to get acces to the titlelabel
        title_labels: &Query<&TitleLabel>,
        builder: impl FnOnce(&mut UiBuilder<(Entity, TitleLabel)>),
    ) -> &mut Self {
        let entity = self.context().1.label_one;
        let tl = title_labels.get(entity).unwrap();
        let mut tl = self.commands().ui_builder((entity, tl.clone()));
        builder(&mut tl);
        self
    }

    fn titled_label_two(
        &mut self,
        title_labels: &Query<&TitleLabel>,
        builder: impl FnOnce(&mut UiBuilder<(Entity, TitleLabel)>),
    ) -> &mut Self {
        let entity = self.context().1.label_two;
        let tl = title_labels.get(entity).unwrap();
        let mut tl = self.commands().ui_builder((entity, tl.clone()));
        builder(&mut tl);
        self
    }
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

// shows how to change styles on the ui elements
fn modify_labels(
    time: Res<Time>,
    mut frames: Local<usize>,

    // these could go into a SystemParam, on which methods could be created, if wished
    mut commands: Commands,
    q: Query<(Entity, &Root)>,
    title_labels: Query<&TitleLabel>,
) {
    let (root_e, root) = q.single();

    *frames += 1;

    commands
        // make sure we get a contexted builder of type 'UiBuilder<'_, (Entity, Root)>'
        .ui_builder((root_e, root.clone()))
        // because it enables this method, that allows us to target the specific titlelabel in the closure
        .titled_label_one(&title_labels, |title_label| {
            // which enables this method to label part of the of the first 'TitleLabel' widget
            title_label.value(|value| {
                value
                    .style()
                    .set_text(frames.to_string())
                    // all the regular style values are available as well
                    .font_size((*frames % 100 + 10) as f32);
            });
        })
        // and here we acecs the second titlevalue on root
        .titled_label_two(&title_labels, |title_label| {
            // and modify its valuewe acecs the second titlevalue on root
            title_label.title(|title| {
                title
                    .style()
                    .set_text(format!("Duration: {}", time.elapsed_seconds()));
            });
        });
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

            label_one = root
                .titled_label("Changing Value Part", "this changes in the system")
                .id();
            label_two = root
                .titled_label("This changes in the system", "Changing title part")
                .id();
        })
        .insert(Root {
            label_one,
            label_two,
        });
}
