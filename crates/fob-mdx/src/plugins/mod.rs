//! Plugin system for MDX transformations

mod heading_ids;
mod image_optimization;
mod link_validation;
mod trait_def;

pub use heading_ids::HeadingIdPlugin;
pub use image_optimization::ImageOptimizationPlugin;
pub use link_validation::LinkValidationPlugin;
pub use trait_def::MdxPlugin;
