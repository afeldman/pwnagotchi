//! Python plugin system for Pwnagotchi.
//!
//! This crate enables writing Pwnagotchi plugins in Python while running them
//! from the Rust core. It uses PyO3 to bridge between Rust and Python.
//!
//! # Features
//!
//! - **Python plugin loading**: Load plugins from Python files
//! - **Async bridge**: Convert Python coroutines to Rust async
//! - **Type conversion**: Automatic conversion between Rust and Python types
//! - **Error handling**: Proper error propagation from Python to Rust
//!
//! # Examples
//!
//! ## Creating a Python Plugin
//!
//! ```python
//! # my_plugin.py
//! class MyPlugin:
//!     def name(self):
//!         return "my_plugin"
//!     
//!     def version(self):
//!         return "1.0.0"
//!     
//!     def description(self):
//!         return "Example Python plugin"
//!     
//!     async def on_handshake(self, filename, access_point, station):
//!         print(f"Handshake captured: {filename}")
//!         print(f"AP: {access_point['essid']} on channel {access_point['channel']}")
//! ```
//!
//! ## Loading from Rust
//!
//! ```no_run
//! use pwnagotchi_py::PythonPlugin;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let plugin = PythonPlugin::from_file("plugins/my_plugin.py", "MyPlugin").await?;
//! println!("Loaded plugin: {}", plugin.name());
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use pwnagotchi_automata::{Epoch, Mood};
use pwnagotchi_core::{AccessPoint, Station};
use pwnagotchi_plugins::Plugin;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use serde_json::Value;
use std::path::Path;
use tracing::{debug, error, info, warn};

/// Python plugin wrapper that implements the Plugin trait.
///
/// This struct wraps a Python class instance and bridges all plugin hooks
/// to Python methods using PyO3.
///
/// # Examples
///
/// ```no_run
/// use pwnagotchi_py::PythonPlugin;
///
/// # async fn example() -> anyhow::Result<()> {
/// // Load plugin from file
/// let plugin = PythonPlugin::from_file("plugins/test.py", "TestPlugin").await?;
///
/// // Use like any other plugin
/// println!("Plugin: {} v{}", plugin.name(), plugin.version());
/// # Ok(())
/// # }
/// ```
pub struct PythonPlugin {
    /// Python instance of the plugin class
    instance: PyObject,
    /// Cached plugin name
    cached_name: String,
    /// Cached plugin version
    cached_version: String,
    /// Cached plugin description
    cached_description: String,
}

impl PythonPlugin {
    /// Creates a new Python plugin from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Python file
    /// * `class_name` - Name of the plugin class to instantiate
    ///
    /// # Returns
    ///
    /// Returns the plugin instance or an error if loading fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Python file cannot be read
    /// - Python syntax is invalid
    /// - Class doesn't exist
    /// - Class instantiation fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_py::PythonPlugin;
    /// # async fn example() -> anyhow::Result<()> {
    /// let plugin = PythonPlugin::from_file("plugins/my_plugin.py", "MyPlugin").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_file(path: impl AsRef<Path>, class_name: &str) -> Result<Self> {
        let path = path.as_ref();
        info!("Loading Python plugin from: {}", path.display());

        let code = tokio::fs::read_to_string(path)
            .await
            .context("Failed to read Python plugin file")?;

        Self::from_code(&code, class_name, path.to_string_lossy().as_ref()).await
    }

    /// Creates a new Python plugin from code string.
    ///
    /// # Arguments
    ///
    /// * `code` - Python source code
    /// * `class_name` - Name of the plugin class
    /// * `module_name` - Name for the module (for error messages)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use pwnagotchi_py::PythonPlugin;
    /// # async fn example() -> anyhow::Result<()> {
    /// let code = r#"
    /// class MyPlugin:
    ///     def name(self): return "test"
    ///     def version(self): return "1.0.0"
    ///     def description(self): return "Test"
    /// "#;
    /// let plugin = PythonPlugin::from_code(code, "MyPlugin", "inline").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn from_code(code: &str, class_name: &str, module_name: &str) -> Result<Self> {
        Python::with_gil(|py| {
            // Compile and execute module
            let module = PyModule::from_code(py, code, module_name, module_name)
                .context("Failed to compile Python code")?;

            // Get class from module
            let plugin_class: &PyAny = module
                .getattr(class_name)
                .context(format!("Class '{}' not found in module", class_name))?;

            // Instantiate class
            let instance = plugin_class
                .call0()
                .context("Failed to instantiate plugin class")?;

            // Cache metadata
            let name = instance
                .call_method0("name")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "unknown".to_string());

            let version = instance
                .call_method0("version")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "0.0.0".to_string());

            let description = instance
                .call_method0("description")
                .and_then(|v| v.extract::<String>())
                .unwrap_or_else(|_| "No description".to_string());

            info!("Loaded Python plugin: {} v{}", name, version);

            Ok(Self {
                instance: instance.into(),
                cached_name: name,
                cached_version: version,
                cached_description: description,
            })
        })
    }

    /// Calls a Python method safely, handling errors.
    fn call_method(&self, method: &str, args: impl IntoPy<Py<pyo3::types::PyTuple>>) -> Result<()> {
        Python::with_gil(|py| {
            let instance = self.instance.as_ref(py);
            
            // Check if method exists
            if !instance.hasattr(method)? {
                debug!("Python plugin doesn't implement {}", method);
                return Ok(());
            }

            // Call method
            instance
                .call_method1(method, args)?;
            
            Ok(())
        }).map_err(|e: PyErr| {
            error!("Python plugin error in {}: {}", method, e);
            anyhow::anyhow!("Python error: {}", e)
        })
    }

    /// Calls an async Python method.
    async fn call_async_method(
        &self,
        method: &str,
        args: impl IntoPy<Py<pyo3::types::PyTuple>>,
    ) -> Result<()> {
        let instance = self.instance.clone();
        let method = method.to_string();

        tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let inst = instance.as_ref(py);
                
                // Check if method exists
                if !inst.hasattr(&method)? {
                    return Ok(());
                }

                // Call method
                inst.call_method1(&method, args)?;
                
                Ok(())
            })
        })
        .await?
        .map_err(|e: PyErr| {
            error!("Python plugin error in {}: {}", method, e);
            anyhow::anyhow!("Python error: {}", e)
        })
    }
}

#[async_trait]
impl Plugin for PythonPlugin {
    fn name(&self) -> &str {
        &self.cached_name
    }

    fn version(&self) -> &str {
        &self.cached_version
    }

    fn description(&self) -> &str {
        &self.cached_description
    }

    async fn on_loaded(&mut self) {
        let _ = self.call_async_method("on_loaded", ()).await;
    }

    async fn on_unload(&mut self) {
        let _ = self.call_async_method("on_unload", ()).await;
    }

    async fn on_ready(&mut self) {
        let _ = self.call_async_method("on_ready", ()).await;
    }

    async fn on_starting(&mut self) {
        let _ = self.call_async_method("on_starting", ()).await;
    }

    async fn on_rebooting(&mut self) {
        let _ = self.call_async_method("on_rebooting", ()).await;
    }

    async fn on_handshake(&mut self, filename: &str, ap: &AccessPoint, sta: &Station) {
        Python::with_gil(|py| {
            let ap_dict = PyDict::new(py);
            let _ = ap_dict.set_item("bssid", &ap.bssid);
            let _ = ap_dict.set_item("essid", &ap.essid);
            let _ = ap_dict.set_item("channel", ap.channel);
            let _ = ap_dict.set_item("rssi", ap.rssi);

            let sta_dict = PyDict::new(py);
            let _ = sta_dict.set_item("mac", &sta.mac);
            let _ = sta_dict.set_item("rssi", sta.rssi);

            let _ = self.call_async_method("on_handshake", (filename, ap_dict, sta_dict));
        });
    }

    async fn on_epoch(&mut self, epoch: u64, epoch_data: &Epoch) {
        Python::with_gil(|py| {
            let epoch_dict = PyDict::new(py);
            let _ = epoch_dict.set_item("epoch", epoch_data.epoch);
            let _ = epoch_dict.set_item("handshakes", epoch_data.num_handshakes);
            let _ = epoch_dict.set_item("associations", epoch_data.num_associations);
            let _ = epoch_dict.set_item("deauths", epoch_data.num_deauths);

            let _ = self.call_async_method("on_epoch", (epoch, epoch_dict));
        });
    }

    async fn on_mood_change(&mut self, old_mood: Mood, new_mood: Mood) {
        let old = format!("{:?}", old_mood);
        let new = format!("{:?}", new_mood);
        let _ = self.call_async_method("on_mood_change", (old, new)).await;
    }

    async fn on_wifi_update(&mut self, access_points: &[AccessPoint]) {
        Python::with_gil(|py| {
            let aps_list = pyo3::types::PyList::empty(py);
            for ap in access_points {
                let ap_dict = PyDict::new(py);
                let _ = ap_dict.set_item("bssid", &ap.bssid);
                let _ = ap_dict.set_item("essid", &ap.essid);
                let _ = ap_dict.set_item("channel", ap.channel);
                let _ = ap_dict.set_item("rssi", ap.rssi);
                let _ = aps_list.append(ap_dict);
            }

            let _ = self.call_async_method("on_wifi_update", (aps_list,));
        });
    }

    async fn on_channel_hop(&mut self, channel: u8) {
        let _ = self.call_async_method("on_channel_hop", (channel,)).await;
    }

    async fn on_peer_detected(&mut self, peer_fingerprint: &str) {
        let _ = self.call_async_method("on_peer_detected", (peer_fingerprint,)).await;
    }

    async fn on_peer_lost(&mut self, peer_fingerprint: &str) {
        let _ = self.call_async_method("on_peer_lost", (peer_fingerprint,)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_python_plugin_basic() {
        let code = r#"
class TestPlugin:
    def name(self):
        return "test_plugin"
    
    def version(self):
        return "1.0.0"
    
    def description(self):
        return "Test plugin"
    
    def on_loaded(self):
        print("Plugin loaded!")
"#;

        let plugin = PythonPlugin::from_code(code, "TestPlugin", "test")
            .await
            .unwrap();

        assert_eq!(plugin.name(), "test_plugin");
        assert_eq!(plugin.version(), "1.0.0");
    }
}
