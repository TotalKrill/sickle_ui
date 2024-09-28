use bevy::prelude::*;
use sickle_ui::{prelude::*, ui_commands::SetTextExt, SickleUiPlugin};

/// The traits are also required to be included to get access to the methods
use titlelabel_widget::*;

// the code to create a titlelabel widget
mod titlelabel_widget {

    use bevy::{color::palettes::css, prelude::*};
    use sickle_ui::prelude::*;

    /// keeps track of its sub widgets or in this case ui-elements
    #[derive(Component, Clone, Copy)]
    pub struct TitleLabel {
        title: Entity,
        label: Entity,
    }
    // Not necessary, but very nice for this kind of "widget" work
    use extension_trait::extension_trait;
    #[extension_trait]
    /// Spawning function on general uibuilder, spawns a container, styles it and spawns two labels inside it
    pub impl TitledLabelExt for UiBuilder<'_, Entity> {
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
    pub impl TitledLabelSubExt for UiBuilder<'_, (Entity, &TitleLabel)> {
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
}

#[derive(Component, Clone, Copy)]
/// Example widget containing two TitleLabel widgets
pub struct Root {
    label_one: Entity,
    label_two: Entity,
}

use extension_trait::extension_trait;
#[extension_trait]
/// Here we define the helper methods, to get typed UiBuilders, that gives us access to the TitleLable methods from its traits
///
/// the purpose of this is to hide complexity and to manage the deferred nature of the commands chain.
/// You could move the closure to a proper EntityCommand that would accept the regular fn, wrapped properly in an Arc.
///
/// There are examples of this around for instance where we allow custom style commands:
/// sickle_ui/crates/sickle_ui_scaffold/src/ui_style/builders.rs
///
/// and the storage struct:
/// sickle_ui/crates/sickle_ui_scaffold/src/ui_style/attribute.rs
impl RootExt for UiBuilder<'_, (Entity, Root)> {
    fn titled_label_one(
        &mut self,
        // we borrow the query to get acces to the titlelabels content, in case we wish to modify it here
        title_labels: &Query<&mut TitleLabel>,
        builder: impl FnOnce(&mut UiBuilder<(Entity, &TitleLabel)>),
    ) -> &mut Self {
        let entity = self.context().1.label_one;
        let tl = title_labels.get(entity).unwrap();
        let mut tl = self.commands().ui_builder((entity, tl));
        builder(&mut tl);
        self
    }

    /// readonly access to TitleLabel two, which only means the TitleLabel component is not modififable from this method
    /// You could also hide the complexity further, by moving the closure to a proper EntityCommand that would accept the
    fn titled_label_two(
        &mut self,
        title_labels: &Query<&TitleLabel>,
        builder: impl FnOnce(&mut UiBuilder<(Entity, &TitleLabel)>),
    ) -> &mut Self {
        let entity = self.context().1.label_two;
        let tl = title_labels.get(entity).unwrap();
        let mut tl = self.commands().ui_builder((entity, tl));
        builder(&mut tl);
        self
    }
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Sickle UI -  Widget Creation and Usage Example".into(),
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

/// shows how to change styles on the ui elements from a system
fn modify_labels(
    time: Res<Time>,
    mut frames: Local<usize>,

    // these could go into a SystemParam, on which methods could be created, if wished
    mut commands: Commands,
    q: Query<(Entity, &Root)>,
    // we get the components from the ECS world, and the Root will just help us get to the correct ones,
    // through the stored entities on the Root, and its helper methods for modifying them
    mut title_labels: Query<&mut TitleLabel>,
) {
    let (root_e, root) = q.single();

    *frames += 1;

    commands
        // make sure we get a contexted builder of type 'UiBuilder<'_, (Entity, Root)>'
        .ui_builder((root_e, root.clone()))
        // because it enables this method, that allows us to target the specific titlelabel in the closure
        .titled_label_one(&mut title_labels, |title_label| {
            // which enables this method to modify the label part of the of the first 'TitleLabel'
            title_label.value(|value| {
                value
                    .style()
                    // all the regular style values are available as well
                    .font_size((*frames % 100 + 10) as f32)
                    .entity_commands() // this erases context
                    .set_text(frames.to_string(), None);
            });
        })
        // and here we acecs the second titlevalue on root
        .titled_label_two(&title_labels.to_readonly(), |title_label| {
            // and modify its valuewe acecs the second titlevalue on root
            title_label.title(|title| {
                title
                    // this keeps context after applying the styling
                    .style_inplace(|style| {
                        style.font_color(
                            Srgba::rgb_u8(*frames as _, *frames as _, *frames as _).into(),
                        );
                    })
                    .entity_commands()
                    .set_text(format!("Duration: {}", time.elapsed_seconds()), None);
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
