use crate::error::IoErrorIntoZhangError;
use crate::ZhangResult;
#[cfg(feature = "plugin")]
use extism::convert::Json as WasmJson;
#[cfg(feature = "plugin")]
use extism::{Manifest, Plugin as WasmPlugin, Wasm};
use log::info;
pub use semver::Version;
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use zhang_ast::{Directive, Plugin, Spanned};

pub mod http;
pub mod store;

/// indicate which type the plugin belongs to
/// the plugin can be multiple types
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// the plugin can handle batches of directive, usually used to filter or combine directives
    Processor,

    /// the plugin have the handler map directive to another directive, usually used to modify **single** directive
    /// the mapper signature would be like
    /// ```rust,ignore
    /// fn mapper(directive: Spanned<Directive>) -> Vec<Spanned<Directive>> {
    ///     // your logic here
    /// }
    /// ```
    Mapper,

    /// the plugin can handle the customized routes, usually used for new page's API
    /// like the request of URL `/api/plugins/{PLUGIN_NAME}/my-resources` will be forwarded to plugin's router by zhang-core
    Router,
}

pub trait PluginInfo {
    fn name() -> &'static str;
    fn version() -> semver::Version;
}