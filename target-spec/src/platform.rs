// Copyright (c) The cargo-guppy Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Error, Triple};
use std::{borrow::Cow, collections::BTreeSet, ops::Deref};

// This is generated by the build script.
include!(concat!(env!("OUT_DIR"), "/current_platform.rs"));

/// A platform to evaluate target specifications against.
///
/// # Standard and custom platforms
///
/// `target-spec` recognizes two kinds of platforms:
///
/// * **Standard platforms:** These platforms are only specified by their triple string. For
///   example, the platform `x86_64-unknown-linux-gnu` is a standard platform since it is recognized
///   by Rust as a tier 1 platform.
///
///   All [builtin platforms](https://doc.rust-lang.org/nightly/rustc/platform-support.html) are
///   standard platforms.
///
///   By default, if a platform isn't builtin, target-spec attempts to heuristically determine the
///   characteristics of the platform based on the triple string. (Use the
///   [`new_strict`](Self::new_strict) constructor to disable this.)
///
/// * **Custom platforms:** These platforms are specified via both a triple string and a JSON file
///   in the format [defined by
///   Rust](https://docs.rust-embedded.org/embedonomicon/custom-target.html). Custom platforms are
///   used for targets not recognized by Rust.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[must_use]
pub struct Platform {
    triple: Triple,
    target_features: TargetFeatures,
    flags: BTreeSet<Cow<'static, str>>,
}

impl Platform {
    /// Creates a new standard `Platform` from the given triple and target features.
    ///
    /// Returns an error if this platform wasn't known to `target-spec`.
    pub fn new(
        triple_str: impl Into<Cow<'static, str>>,
        target_features: TargetFeatures,
    ) -> Result<Self, Error> {
        let triple = Triple::new(triple_str.into()).map_err(Error::UnknownPlatformTriple)?;
        Ok(Self::from_triple(triple, target_features))
    }

    /// Creates a new standard `Platform` from the given triple and target features.
    ///
    /// This constructor only consults the builtin platform table, and does not attempt to
    /// heuristically determine the platform's characteristics based on the triple string.
    pub fn new_strict(
        triple_str: impl Into<Cow<'static, str>>,
        target_features: TargetFeatures,
    ) -> Result<Self, Error> {
        let triple = Triple::new_strict(triple_str.into()).map_err(Error::UnknownPlatformTriple)?;
        Ok(Self::from_triple(triple, target_features))
    }

    /// Returns the current platform, as detected at build time.
    ///
    /// This is currently always a standard platform, and will return an error if the current
    /// platform was unknown to this version of `target-spec`.
    ///
    /// # Notes
    ///
    /// In the future, this constructor may also support custom platforms. This will not be
    /// considered a breaking change.
    pub fn current() -> Result<Self, Error> {
        let triple = Triple::new(CURRENT_TARGET).map_err(Error::UnknownPlatformTriple)?;
        let target_features = TargetFeatures::features(CURRENT_TARGET_FEATURES.iter().copied());
        Ok(Self {
            triple,
            target_features,
            flags: BTreeSet::new(),
        })
    }

    /// Creates a new standard platform from a `Triple` and target features.
    pub fn from_triple(triple: Triple, target_features: TargetFeatures) -> Self {
        Self {
            triple,
            target_features,
            flags: BTreeSet::new(),
        }
    }

    /// Creates a new custom `Platform` from the given triple, platform, and target features.
    #[cfg(feature = "custom")]
    pub fn new_custom(
        triple_str: impl Into<Cow<'static, str>>,
        json: &str,
        target_features: TargetFeatures,
    ) -> Result<Self, Error> {
        let triple = Triple::new_custom(triple_str, json).map_err(Error::CustomPlatformCreate)?;
        Ok(Self {
            triple,
            target_features,
            flags: BTreeSet::new(),
        })
    }

    /// Adds a set of flags to accept.
    ///
    /// A flag is a single token like the `foo` in `cfg(not(foo))`.
    ///
    /// A default `cargo build` will always evaluate flags to false, but custom wrappers may cause
    /// some flags to evaluate to true. For example, as of version 0.6, `cargo web build` will cause
    /// `cargo_web` to evaluate to true.
    pub fn add_flags(&mut self, flags: impl IntoIterator<Item = impl Into<Cow<'static, str>>>) {
        self.flags.extend(flags.into_iter().map(|s| s.into()));
    }

    /// Returns the target triple string for this platform.
    pub fn triple_str(&self) -> &str {
        self.triple.as_str()
    }

    /// Returns the set of flags enabled for this platform.
    pub fn flags(&self) -> impl ExactSizeIterator<Item = &str> {
        self.flags.iter().map(|flag| flag.deref())
    }

    /// Returns true if this flag was set with `add_flags`.
    pub fn has_flag(&self, flag: impl AsRef<str>) -> bool {
        self.flags.contains(flag.as_ref())
    }

    /// Returns true if this is a standard platform.
    ///
    /// A standard platform can be either builtin, or heuristically determined.
    ///
    /// # Examples
    ///
    /// ```
    /// use target_spec::{Platform, TargetFeatures};
    ///
    /// // x86_64-unknown-linux-gnu is Linux x86_64.
    /// let platform = Platform::new("x86_64-unknown-linux-gnu", TargetFeatures::Unknown).unwrap();
    /// assert!(platform.is_standard());
    /// ```
    pub fn is_standard(&self) -> bool {
        self.triple.is_standard()
    }

    /// Returns true if this is a builtin platform.
    ///
    /// All builtin platforms are standard, but not all standard platforms are builtin.
    ///
    /// # Examples
    ///
    /// ```
    /// use target_spec::{Platform, TargetFeatures};
    ///
    /// // x86_64-unknown-linux-gnu is Linux x86_64, which is a Rust tier 1 platform.
    /// let platform = Platform::new("x86_64-unknown-linux-gnu", TargetFeatures::Unknown).unwrap();
    /// assert!(platform.is_builtin());
    /// ```
    pub fn is_builtin(&self) -> bool {
        self.triple.is_builtin()
    }

    /// Returns true if this is a heuristically determined platform.
    ///
    /// All heuristically determined platforms are standard, but most of the time, standard
    /// platforms are builtin.
    ///
    /// # Examples
    ///
    /// ```
    /// use target_spec::{Platform, TargetFeatures};
    ///
    /// // armv5te-apple-darwin is not a real platform, but target-spec can heuristically
    /// // guess at its characteristics.
    /// let platform = Platform::new("armv5te-apple-darwin", TargetFeatures::Unknown).unwrap();
    /// assert!(platform.is_heuristic());
    /// ```
    pub fn is_heuristic(&self) -> bool {
        self.triple.is_heuristic()
    }

    /// Returns true if this is a custom platform.
    ///
    /// This is always available, but if the `custom` feature isn't turned on this always returns
    /// false.
    pub fn is_custom(&self) -> bool {
        self.triple.is_custom()
    }

    /// Returns the underlying [`Triple`].
    pub fn triple(&self) -> &Triple {
        &self.triple
    }

    /// Returns the set of target features for this platform.
    pub fn target_features(&self) -> &TargetFeatures {
        &self.target_features
    }

    #[cfg(feature = "summaries")]
    pub(crate) fn custom_json(&self) -> Option<&str> {
        self.triple.custom_json()
    }
}

/// A set of target features to match.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum TargetFeatures {
    /// The target features are unknown.
    Unknown,
    /// Only match the specified features.
    Features(BTreeSet<Cow<'static, str>>),
    /// Match all features.
    All,
}

impl TargetFeatures {
    /// Creates a new `TargetFeatures` which matches some features.
    pub fn features(features: impl IntoIterator<Item = impl Into<Cow<'static, str>>>) -> Self {
        TargetFeatures::Features(features.into_iter().map(|s| s.into()).collect())
    }

    /// Creates a new `TargetFeatures` which doesn't match any features.
    pub fn none() -> Self {
        TargetFeatures::Features(BTreeSet::new())
    }

    /// Returns `Some(true)` if this feature is a match, `Some(false)` if it isn't, and `None` if
    /// the set of target features is unknown.
    pub fn matches(&self, feature: &str) -> Option<bool> {
        match self {
            TargetFeatures::Unknown => None,
            TargetFeatures::Features(features) => Some(features.contains(feature)),
            TargetFeatures::All => Some(true),
        }
    }
}
