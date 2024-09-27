use crate::{
    ui_commands::EntityCommandsNamedExt,
    ui_style::{UiStyle, UiStyleExt, UiStyleUnchecked, UiStyleUncheckedExt},
};
use bevy::ecs::system::IntoObserverSystem;
use bevy::{
    ecs::{
        bundle::Bundle,
        entity::Entity,
        system::{Commands, EntityCommands},
    },
    hierarchy::BuildChildren,
    prelude::*,
};

/// Ghost struct to use as a type filler for root UI nodes.
///
/// i.e. `commands.ui_builder(UiRoot)` to start building without a parent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct UiRoot;

/// Used to find a root node where nodes are safe to spawn
///
/// i.e. context menus or floating panels torn off from tab containers look for this to mount.
/// This can be injected manually by the developer to indicate mount points for trees.
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct UiContextRoot;

/// The heart of `sickle_ui`
///
/// Holds a number of extension traits that map to widget creation and styling commands.
/// Acquire a builder from commands via `commands.ui_builder(UiRoot)` or
/// `commands.ui_builder(entity)`, where the entity is the UI parent node.
pub struct UiBuilder<'a, T> {
    commands: Commands<'a, 'a>,
    context: T,
}

impl<'a, T> UiBuilder<'a, T> {
    /// The current build context
    ///
    /// Actual value depends on the type of the builder, usually it is an Entity.
    /// Widgets interally can use it to pass around their main component or other data.
    pub fn context(&self) -> &T {
        &self.context
    }

    /// Return the commands used by the builder.
    pub fn commands(&mut self) -> &mut Commands<'a, 'a> {
        &mut self.commands
    }
}

impl UiBuilder<'_, UiRoot> {
    /// Spawn a bundle as a root node (without parent)
    ///
    /// The returned builder can be used to add children to the newly root node.
    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity> {
        let new_entity = self.commands().spawn(bundle).id();

        self.commands().ui_builder(new_entity)
    }
}

impl UiBuilder<'_, Entity> {
    /// The ID (Entity) of the current builder
    pub fn id(&self) -> Entity {
        *self.context()
    }

    /// The `EntityCommands` of the builder
    ///
    /// Poits to the entity currently being built upon (see [`UiBuilder<'_, Entity>::id()`]).
    pub fn entity_commands(&mut self) -> EntityCommands {
        let entity = self.id();
        self.commands().entity(entity)
    }

    /// Styling commands for UI Nodes
    ///
    /// `sickle_ui` exposes functions for all standard bevy styleable attributes.
    /// Manual extension can be done for custom styling needs via extension traits:
    ///
    /// ```rust
    /// pub trait SetMyPropExt {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self;
    /// }
    ///
    /// impl SetMyPropExt for UiStyle<'_> {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self {
    ///         // SetMyProp is assumed to be an EntityCommand
    ///         // Alternatively a closure can be supplied as per a standard bevy command
    ///         // NOTE: All built-in commands structs are public and can be re-used in extensions
    ///         self.entity_commands().add(SetMyProp {
    ///             value
    ///         });
    ///         self
    ///     }
    /// }
    /// ```
    pub fn style(&mut self) -> UiStyle {
        let entity = self.id();
        self.commands().style(entity)
    }

    /// Same as [`UiBuilder<'_, Entity>::style()`], except style commands bypass possible attribute locks.
    pub fn style_unchecked(&mut self) -> UiStyleUnchecked {
        let entity = self.id();
        self.commands().style_unchecked(entity)
    }

    /// Spawn a child node as a child of the current entity identified by [`UiBuilder<'_, Entity>::id()`]
    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity> {
        let mut new_entity = Entity::PLACEHOLDER;

        let entity = self.id();
        self.commands().entity(entity).with_children(|parent| {
            new_entity = parent.spawn(bundle).id();
        });

        self.commands().ui_builder(new_entity)
    }

    /// Inserts a [`Bundle`] of components to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        self.entity_commands().insert(bundle);
        self
    }

    /// Insert a [`Name`] component to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    pub fn named(&mut self, name: impl Into<String>) -> &mut Self {
        self.entity_commands().named(name);
        self
    }

    /// Mount an observer to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    pub fn observe<E: Event, B: Bundle, M>(
        &mut self,
        system: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        self.entity_commands().observe(system);
        self
    }
}

/// Implementations that are very useful for creating nested widgets
impl<T> UiBuilder<'_, (Entity, T)> {
    /// The ID (Entity) of the current builder
    pub fn id(&self) -> Entity {
        self.context().0
    }

    /// The extension content of the UiBuilder
    pub fn content(&self) -> &T {
        &self.context().1
    }

    /// The `EntityCommands` of the builder
    ///
    /// Poits to the entity currently being built upon (see [`UiBuilder<'_, Entity>::id()`]).
    pub fn entity_commands(&mut self) -> EntityCommands {
        let entity = self.id();
        self.commands().entity(entity)
    }

    /// Styling commands for UI Nodes
    ///
    /// `sickle_ui` exposes functions for all standard bevy styleable attributes.
    /// Manual extension can be done for custom styling needs via extension traits:
    ///
    /// ```rust
    /// pub trait SetMyPropExt {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self;
    /// }
    ///
    /// impl SetMyPropExt for UiStyle<'_> {
    ///     fn my_prop(&mut self, value: f32) -> &mut Self {
    ///         // SetMyProp is assumed to be an EntityCommand
    ///         // Alternatively a closure can be supplied as per a standard bevy command
    ///         // NOTE: All built-in commands structs are public and can be re-used in extensions
    ///         self.entity_commands().add(SetMyProp {
    ///             value
    ///         });
    ///         self
    ///     }
    /// }
    /// ```
    pub fn style(&mut self) -> UiStyle {
        let entity = self.id();
        self.commands().style(entity)
    }

    /// Same as [`UiBuilder<'_, Entity>::style()`], except style commands bypass possible attribute locks.
    pub fn style_unchecked(&mut self) -> UiStyleUnchecked {
        let entity = self.id();
        self.commands().style_unchecked(entity)
    }

    /// Spawn a child node as a child of the current entity identified by [`UiBuilder<'_, Entity>::id()`]
    pub fn spawn(&mut self, bundle: impl Bundle) -> UiBuilder<Entity> {
        let mut new_entity = Entity::PLACEHOLDER;

        let entity = self.id();
        self.commands().entity(entity).with_children(|parent| {
            new_entity = parent.spawn(bundle).id();
        });

        self.commands().ui_builder(new_entity)
    }

    /// Inserts a [`Bundle`] of components to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    pub fn insert(&mut self, bundle: impl Bundle) -> &mut Self {
        self.entity_commands().insert(bundle);
        self
    }

    /// Insert a [`Name`] component to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    pub fn named(&mut self, name: impl Into<String>) -> &mut Self {
        self.entity_commands().named(name);
        self
    }

    /// Mount an observer to the current entity (identified by [`UiBuilder<'_, Entity>::id()`])
    pub fn observe<E: Event, B: Bundle, M>(
        &mut self,
        system: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        self.entity_commands().observe(system);
        self
    }
}

pub trait UiBuilderExt {
    /// A contextual UI Builder, see [`UiBuilder<'a, T>`]
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T>;
}

impl UiBuilderExt for Commands<'_, '_> {
    fn ui_builder<T>(&mut self, context: T) -> UiBuilder<T> {
        UiBuilder {
            commands: self.reborrow(),
            context,
        }
    }
}
