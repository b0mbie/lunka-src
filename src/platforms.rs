//! Lua platform handling.

/// Trait for a Lua platform.
pub trait Platform {
	fn defines(&self) -> &[&str];
	fn standards(&self) -> &Standards<'_>;
}

/// Trait for a known, constant Lua platform.
pub trait ConstPlatform {
	const DEFINES: &'static [&'static str];
	const STANDARDS: Standards<'static> = Standards {
		gnu: Some("gnu99"),
		clang: Some("gnu99"),
		msvc: Some("c99"),
		clang_cl: Some("gnu99"),
	};
}
impl<T: ConstPlatform> Platform for T {
	fn defines(&self) -> &[&str] {
		Self::DEFINES
	}
	fn standards(&self) -> &Standards<'_> {
		&Self::STANDARDS
	}
}

macro_rules! platform {
	{
		$(#[$attr:meta])*
		$vis:vis struct $name:ident;
		DEFINES = $defines:expr;
		$(STANDARDS = $standards:expr;)?
	} => {
		#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		$(#[$attr])*
		$vis struct $name;

		impl ConstPlatform for $name {
			const DEFINES: &[&str] = $defines;
			$(const STANDARDS: Standards<'_> = $standards;)?
		}
	};
}

platform! {
	pub struct Aix;
	DEFINES = &[
		"LUA_USE_POSIX",
		"LUA_USE_DLOPEN",
	];
}

platform! {
	pub struct Bsd;
	DEFINES = &[
		"LUA_USE_POSIX",
		"LUA_USE_DLOPEN",
	];
}

platform! {
	pub struct C89;
	DEFINES = &[
		"LUA_USE_POSIX",
		"LUA_USE_DLOPEN",
	];
	STANDARDS = Standards {
		gnu: Some("c89"),
		clang: Some("c89"),
		msvc: None,
		clang_cl: Some("c89"),
	};
}

platform! {
	pub struct FreeBsd;
	DEFINES = &[
		"LUA_USE_LINUX",
	];
}

platform! {
	pub struct Ios;
	DEFINES = &[
		"LUA_USE_IOS",
	];
}

platform! {
	pub struct Linux;
	DEFINES = &[
		"LUA_USE_LINUX",
	];
}

platform! {
	pub struct MacOsX;
	DEFINES = &[
		"LUA_USE_MACOSX",
	];
}

platform! {
	pub struct MinGw;
	DEFINES = &[
		"LUA_BUILD_AS_DLL",
	];
}

platform! {
	pub struct Posix;
	DEFINES = &[
		"LUA_USE_POSIX",
	];
}

platform! {
	pub struct Solaris;
	DEFINES = &[
		"LUA_USE_POSIX",
		"LUA_USE_DLOPEN",
		"_REENTRANT",
	];
}

platform! {
	pub struct Windows;
	DEFINES = &[
		"LUA_USE_WINDOWS",
	];
}

/// Collection of C standard identifiers for different kinds of compilers.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Standards<'a> {
	pub gnu: Option<&'a str>,
	pub clang: Option<&'a str>,
	pub msvc: Option<&'a str>,
	pub clang_cl: Option<&'a str>,
}

struct DynPlatform {
	pub defines: &'static [&'static str],
	pub standards: &'static Standards<'static>,
}

impl DynPlatform {
	/// Collect information about a [`ConstPlatform`] into this structure.
	pub const fn new<P: ConstPlatform>() -> Self {
		Self {
			defines: P::DEFINES,
			standards: &P::STANDARDS,
		}
	}
}

impl Platform for DynPlatform {
	fn defines(&self) -> &[&str] {
		self.defines
	}
	fn standards(&self) -> &Standards<'_> {
		self.standards
	}
}

/// Current target triple.
pub const CURRENT_TRIPLE: &str = current_platform::CURRENT_PLATFORM;

/// Get an appropriate [`Platform`] for the target triple used for compilation.
pub fn from_current_triple() -> Option<impl Platform> {
	from_target_triple(CURRENT_TRIPLE)
}

/// Get an appropriate [`Platform`] for the given target triple.
pub fn from_target_triple(target: &str) -> Option<impl Platform> {
	if target.contains("linux") {
		Some(DynPlatform::new::<Linux>())
	} else if target.ends_with("bsd") {
		Some(DynPlatform::new::<FreeBsd>())
	} else if target.ends_with("apple-darwin") {
		Some(DynPlatform::new::<MacOsX>())
	} else if target.ends_with("apple-ios") {
		Some(DynPlatform::new::<Ios>())
	} else if target.ends_with("solaris") {
		Some(DynPlatform::new::<Solaris>())
	} else if target.contains("windows") {
		Some(DynPlatform::new::<Windows>())
	} else {
		None
	}
}
