/// Structure that represents additional configuration for Lua
/// which cannot be done with command-line definitions.
/// 
/// # Preparation of Lua source
/// This structure cannot be used with a normal Lua source distribution,
/// as the settings are not able to be modified per Lua build.
/// `luaconf.h` *must* contain (preferably, in the "local configuration" section) definitions for each field
/// with `#if defined(...)` guards.
/// 
/// For instance, the following snippet for `no_string_to_number` and `extra_space` respectively is appropriate,
/// assuming it's placed in the "local configuration" section:
/// ```c
/// #if defined(LUNKA_NOCVTS2N)
/// #define LUA_NOCVTS2N
/// #endif
/// 
/// #if defined(LUNKA_EXTRASPACE)
/// #define LUA_EXTRASPACE LUNKA_EXTRASPACE
/// #endif
/// ```
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LuaConf<S> {
	/// `true` to disable automatic coercion from numbers to strings.
	/// 
	/// This corresponds to `LUNKA_NOCVTN2S` for `LUA_NOCVTN2S`.
	pub no_number_to_string: bool,
	/// `true` to disable automatic coercion from strings to numbers.
	/// 
	/// This corresponds to `LUNKA_NOCVTS2N` for `LUA_NOCVTS2N`.
	pub no_string_to_number: bool,
	/// Size of the raw memory area associated with a Lua state with very fast access.
	/// 
	/// This corresponds to `LUNKA_EXTRASPACE` for `LUA_EXTRASPACE`.
	pub extra_space: Option<S>,
	/// Maximum size for the description of the source of a function in debug information.
	/// 
	/// This corresponds to `LUNKA_IDSIZE` for `LUA_IDSIZE`.
	pub id_size: Option<S>,
}
