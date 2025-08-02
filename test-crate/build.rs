use ::lunka_src::*;

fn main() {
	let lua_conf = LuaConf::<&'static str> {
		no_number_to_string: true,
		no_string_to_number: true,
		extra_space: None,
		id_size: None,
	};

	Build::for_current()
		.add_lunka_src()
		.lua_conf(&lua_conf)
		.compat_lua_5_3()
		.unicode_identifiers()
		.compile("lua");
}
