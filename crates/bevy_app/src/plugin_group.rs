use crate::{AppBuilder, Plugin, BoxedPlugin};
use bevy_utils::{tracing::debug, HashMap};
use std::any::TypeId;

pub trait PluginGroup {
    fn build(self, group: &mut PluginGroupBuilder);
}

struct PluginEntry {
    plugin: BoxedPlugin,
    enabled: bool,
}

#[derive(Default)]
pub struct PluginGroupBuilder {
    plugins: HashMap<TypeId, PluginEntry>,
    order: Vec<TypeId>,
}

impl PluginGroupBuilder {
    pub fn add<T: Plugin>(&mut self, plugin: T) -> &mut Self {
        self.order.push(TypeId::of::<T>());
        self.plugins.insert(
            TypeId::of::<T>(),
            PluginEntry {
                plugin: Box::new(plugin),
                enabled: true,
            },
        );
        self
    }

    pub fn add_before<Target: Plugin, T: Plugin>(&mut self, plugin: T) -> &mut Self {
        let target_index = self
            .order
            .iter()
            .enumerate()
            .find(|(_i, ty)| **ty == TypeId::of::<Target>())
            .map(|(i, _)| i)
            .unwrap_or_else(|| {
                panic!(
                    "Plugin does not exist: {}.",
                    std::any::type_name::<Target>()
                )
            });
        self.order.insert(target_index, TypeId::of::<T>());
        self.plugins.insert(
            TypeId::of::<T>(),
            PluginEntry {
                plugin: Box::new(plugin),
                enabled: true,
            },
        );
        self
    }

    pub fn add_after<Target: Plugin, T: Plugin>(&mut self, plugin: T) -> &mut Self {
        let target_index = self
            .order
            .iter()
            .enumerate()
            .find(|(_i, ty)| **ty == TypeId::of::<Target>())
            .map(|(i, _)| i)
            .unwrap_or_else(|| {
                panic!(
                    "Plugin does not exist: {}.",
                    std::any::type_name::<Target>()
                )
            });
        self.order.insert(target_index + 1, TypeId::of::<T>());
        self.plugins.insert(
            TypeId::of::<T>(),
            PluginEntry {
                plugin: Box::new(plugin),
                enabled: true,
            },
        );
        self
    }

    pub fn enable<T: Plugin>(&mut self) -> &mut Self {
        let mut plugin_entry = self
            .plugins
            .get_mut(&TypeId::of::<T>())
            .expect("Cannot enable a plugin that does not exist.");
        plugin_entry.enabled = true;
        self
    }

    pub fn disable<T: Plugin>(&mut self) -> &mut Self {
        let mut plugin_entry = self
            .plugins
            .get_mut(&TypeId::of::<T>())
            .expect("Cannot disable a plugin that does not exist.");
        plugin_entry.enabled = false;
        self
    }

    pub fn finish(mut self, app: &mut AppBuilder) {
        for ty in self.order.iter() {
            if let Some(PluginEntry{plugin,enabled}) = self.plugins.remove(ty) {
                if enabled {
                    debug!("added plugin: {}", plugin.name());
                    plugin.unbox_and_build(app);
                }
            }
        }
    }
}
