//! Tools for sapiens

/// Hue tools
#[cfg(feature = "hue")]
pub mod hue;

/// Tool to conclude a chain
pub mod conclude;

/// Tool to run some (limited) python
pub mod python;

/// Tool to test stuffs
pub mod dummy;

/// Tools related to mediawiki: Wikipedia, Wikidata, etc.
#[cfg(feature = "wiki")]
pub mod wiki;
