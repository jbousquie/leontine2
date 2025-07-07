//! Persistent storage hook for Leontine
//! Provides a way to store data in localStorage that persists across page reloads
//!
// ref : https://dioxuslabs.com/learn/0.6/cookbook/state/custom_hooks/#composing-hooks

use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
use serde::{de::DeserializeOwned, Serialize};

/// A persistent storage hook that can be used to store data across application reloads.
#[allow(clippy::needless_return)]
pub fn use_persistent<T: Serialize + DeserializeOwned + Default + Clone + 'static>(
    // A unique key for the storage entry
    key: impl ToString,
    // A function that returns the initial value if the storage entry is empty
    init: impl FnOnce() -> T,
) -> UsePersistent<T> {
    // Use the use_signal hook to create a mutable state for the storage entry
    let state = use_signal(move || {
        // This closure will run when the hook is created
        let key = key.to_string();
        let value = LocalStorage::get(key.as_str()).unwrap_or_else(|_| init());
        StorageEntry { key, value }
    });

    // Wrap the state in a new struct with a custom API
    UsePersistent { inner: state }
}

/// Internal storage entry structure
pub(crate) struct StorageEntry<T> {
    key: String,
    value: T,
}

/// Storage that persists across application reloads
#[derive(PartialEq)]
pub struct UsePersistent<T: 'static> {
    inner: Signal<StorageEntry<T>>,
}

impl<T> Clone for UsePersistent<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Copy for UsePersistent<T> {}

impl<T: Serialize + DeserializeOwned + Clone + 'static> UsePersistent<T> {
    /// Gets the current value
    pub fn get(&self) -> T {
        self.inner.read().value.clone()
    }

    /// Sets the value and persists it to localStorage
    pub fn set(&mut self, value: T) {
        let mut state = self.inner.write();
        // Write the new value to local storage
        LocalStorage::set(state.key.as_str(), &value).expect("failed to write to localStorage");
        state.value = value;
    }
}

// No longer need Deref implementation since we're using explicit get/set methods
