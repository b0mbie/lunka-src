use ::cc::Build as CcBuild;
use ::std::{
	fs::read_dir,
	io::Error as IoError,
	path::Path,
};

pub use ::cc::Error as CcError;

mod lua_conf;
pub use lua_conf::*;
pub mod platforms;

use platforms::{
	Platform, from_current_triple, CURRENT_TRIPLE,
};

/// Builder for a compilation of Lua 5.4.
#[repr(transparent)]
pub struct Build {
	cc: CcBuild,
}

impl Build {
	/// Create a new builder based on the [`Platform`] returned by [`from_current_triple`],
	/// panicking if determining the platform or setting up failed.
	pub fn for_current() -> Self {
		let Some(platform) = from_current_triple() else {
			panic!("couldn't determine platform for current target triple {CURRENT_TRIPLE:?}");
		};
		Self::new(platform)
	}
}

impl Build {
	/// Create a new builder based on a [`Platform`],
	/// panicking if setting up failed.
	/// 
	/// See also [`Build::try_new`] for the non-panicking version.
	pub fn new<P: Platform>(p: P) -> Self {
		match Self::try_new(p) {
			Ok(b) => b,
			Err(e) => panic!("{e}"),
		}
	}

	/// Create a new builder based on a [`Platform`].
	pub fn try_new<P: Platform>(p: P) -> Result<Self, CcError> {
		let mut cc = CcBuild::new();

		{
			let tool = cc.try_get_compiler()?;

			let stds = p.standards();
			let mut set_std = |std: Option<&str>| if let Some(std) = std { cc.std(std); };
			if tool.is_like_gnu() {
				set_std(stds.gnu)
			} else if tool.is_like_clang() {
				set_std(stds.clang)
			} else if tool.is_like_msvc() {
				set_std(stds.msvc)
			} else if tool.is_like_clang_cl() {
				set_std(stds.clang_cl)
			}
		}
	
		cc.warnings(true).extra_warnings(true);
		for define in p.defines() {
			cc.define(define, None);
		}
	
		Ok(Self {
			cc,
		})
	}

	/// Run the compiler, generating the file `output`,
	/// and panicking if compilation fails.
	/// 
	/// See also [`Build::try_compile`] for the non-panicking version.
	pub fn compile(&self, output: &str) {
		if let Err(e) = self.try_compile(output) {
			panic!("{e}");
		}
	}

	/// Run the compiler, generating the file `output`.
	pub fn try_compile(&self, output: &str) -> Result<(), CcError> {
		self.cc.try_compile(output)
	}

	/// Set the host assumed by this configuration.
	pub fn host(&mut self, host: &str) -> &mut Self {
		self.cc.host(host);
		self
	}

	/// Set the output directory where all object files and static libraries will be located.
	pub fn out_dir<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
		self.cc.out_dir(path);
		self
	}

	fn define_flag(&mut self, flag: &str) -> &mut Self {
		self.cc.define(flag, None);
		self
	}

	fn define_lit(&mut self, ident: &str, data: &str) -> &mut Self {
		self.cc.define(ident, Some(data));
		self
	}

	fn define_str(&mut self, ident: &str, data: &str) -> &mut Self {
		let data = format!("\"{}\"", data.replace('"', "\\\"").replace('\\', "\\\\"));
		self.define_lit(ident, &data)
	}

	/// Add all Lua 5.4.8 source files bundled with this crate,
	/// which allows for [`LuaConf`] to be used,
	/// panicking if an error occurs while reading the directory contents.
	pub fn add_lunka_src(&mut self) -> &mut Self {
		match self.try_add_lunka_src() {
			Ok(s) => s,
			Err(e) => panic!("{e}"),
		}
	}

	/// Add all Lua 5.4.8 source files bundled with this crate,
	/// which allows for [`LuaConf`] to be used.
	pub fn try_add_lunka_src(&mut self) -> Result<&mut Self, IoError> {
		let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("lua-5.4.8");
		self.include(root.join("include"));
		let src = {
			let mut b = root;
			b.push("src");
			b
		};
		for result in read_dir(src)? {
			let item = result?;
			if !item.file_type()?.is_file() {
				continue
			}
			self.cc.file(item.path());
		}
		Ok(self)
	}

	/// Add all Lua source files found in the specified `root`,
	/// panicking if an error occurs while reading the directory contents.
	/// 
	/// See also [`Build::try_add_lua_src`] for the non-panicking version.
	/// 
	/// `root` must be a directory containing both source files (`*.c`) and headers (`*.h`).
	/// 
	/// If [`LuaConf`] is used,
	/// then the path cannot point to a normal Lua source distribution.
	/// See the documentation for [`LuaConf`] for more details.
	pub fn add_lua_src<P: AsRef<Path>>(&mut self, root: P) -> &mut Self {
		match self.try_add_lua_src(root) {
			Ok(s) => s,
			Err(e) => panic!("{e}"),
		}
	}

	/// Add all Lua source files found in the specified `root`.
	/// 
	/// `root` must be a directory containing both source files (`*.c`) and headers (`*.h`).
	/// 
	/// If [`LuaConf`] is used,
	/// then the path cannot point to a normal Lua source distribution.
	/// See the documentation for [`LuaConf`] for more details.
	pub fn try_add_lua_src<P: AsRef<Path>>(&mut self, root: P) -> Result<&mut Self, IoError> {
		const BINARIES: [&str; 2] = ["lua.c", "luac.c"];
		for result in read_dir(root)? {
			let item = result?;
			if !item.file_type()?.is_file() {
				continue
			}

			let file_name = item.file_name();
			let Some(file_name) = file_name.to_str() else {
				continue
			};
			if !file_name.ends_with(".c") || BINARIES.contains(&file_name) {
				continue
			}

			self.cc.file(item.path());
		}
		Ok(self)
	}

	/// Add an include directory.
	pub fn include<P: AsRef<Path>>(&mut self, path: P) -> &mut Self {
		self.cc.include(path);
		self
	}

	/// Adds multiple include directories.
	pub fn includes<P>(&mut self, paths: P) -> &mut Self
	where
		P: IntoIterator,
		P::Item: AsRef<Path>,
	{
		self.cc.includes(paths);
		self
	}

	/// Set whether debug information should be emitted for this build.
	pub fn debug_info(&mut self, emit_debug_info: bool) -> &mut Self {
		self.cc.debug(emit_debug_info);
		self
	}

	/// Set the semi-arbitrary optimization level for the generated object files.
	pub fn opt_level(&mut self, opt_level: u32) -> &mut Self {
		self.cc.opt_level(opt_level);
		self
	}

	/// Enable compatibility with Lua 5.3.
	pub fn compat_lua_5_3(&mut self) -> &mut Self {
		self.define_flag("LUA_COMPAT_5_3")
	}

	/// Include several deprecated functions in the `math` library.
	pub fn compat_math_lib(&mut self) -> &mut Self {
		self.define_flag("LUA_COMPAT_MATH_LIB")
	}

	/// Emulate the `__le` metamethod using `__lt`.
	pub fn compat_lt_le(&mut self) -> &mut Self {
		self.define_flag("LUA_COMPAT_LT_LE")
	}

	/// Enable several consistency checks in the API.
	pub fn api_checks(&mut self) -> &mut Self {
		self.define_flag("LUA_USE_APICHECK")
	}

	/// Set the default path that Lua uses to look for Lua libraries.
	pub fn lua_lib_path(&mut self, path: &str) -> &mut Self {
		self.define_str("LUA_PATH_DEFAULT", path)
	}

	/// Set the default path that Lua uses to look for C libraries.
	pub fn lua_c_lib_path(&mut self, path: &str) -> &mut Self {
		self.define_str("LUA_CPATH_DEFAULT", path)
	}

	/// Set the directory separator for `require` submodules.
	pub fn dir_separator(&mut self, sep: &str) -> &mut Self {
		self.define_str("LUA_DIRSEP", sep)
	}

	/// Enable Unicode Identifiers.
	/// 
	/// This is a define that isn't explicitly mentioned in the configuration header,
	/// but is checked in `lctype.c` to build the identifier character table.
	pub fn unicode_identifiers(&mut self) -> &mut Self {
		self.define_flag("LUA_UCID")
	}

	/// Use additional configuration provided by a [`LuaConf`] in this build.
	pub fn lua_conf<S: AsRef<str>>(&mut self, lua_conf: &LuaConf<S>) -> &mut Self {
		if lua_conf.no_number_to_string {
			self.define_flag("LUNKA_NOCVTN2S");
		}
		if lua_conf.no_string_to_number {
			self.define_flag("LUNKA_NOCVTS2N");
		}
		if let Some(extra_space) = lua_conf.extra_space.as_ref().map(move |s| s.as_ref()) {
			self.define_lit("LUNKA_EXTRASPACE", extra_space);
		}
		if let Some(id_size) = lua_conf.id_size.as_ref().map(move |s| s.as_ref()) {
			self.define_lit("LUNKA_IDSIZE", id_size);
		}
		self
	}
}
