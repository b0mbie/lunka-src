use ::std::{
	ffi::{
		CStr, c_char, c_int, c_void,
	},
	ops::{
		Deref, DerefMut,
	},
	mem::{
		MaybeUninit, align_of, size_of,
	},
	slice::from_raw_parts,
};

#[repr(transparent)]
struct State(c_void);

const LUA_OK: c_int = 0;

type CFunction = unsafe extern "C-unwind" fn(l: *mut State) -> c_int;
type KContext = isize;
type KFunction = unsafe extern "C-unwind" fn(l: *mut State, status: c_int, ctx: KContext) -> c_int;

unsafe extern "C-unwind" {
	fn luaL_newstate() -> *mut State;
	fn lua_close(l: *mut State);
	fn lua_pcallk(
		l: *mut State,
		n_args: c_int, n_results: c_int, err_func: c_int,
		ctx: KContext, k: Option<KFunction>,
	) -> c_int;
	fn luaL_loadstring(l: *mut State, s: *const c_char) -> c_int;
	fn luaL_openlibs(l: *mut State);
	fn lua_tolstring(l: *mut State, idx: c_int, out_len: *mut usize) -> *const c_char;
	fn lua_pushcclosure(l: *mut State, func: CFunction, n_upvalues: c_int);
}

#[repr(transparent)]
struct Lua {
	thread: &'static mut Thread,
}

impl Drop for Lua {
	fn drop(&mut self) {
		unsafe { lua_close(self.thread.as_ptr_mut()) }
	}
}

impl Deref for Lua {
	type Target = Thread;
	fn deref(&self) -> &Self::Target {
		self.thread
	}
}
impl DerefMut for Lua {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.thread
	}
}

impl Lua {
	pub fn new() -> Option<Self> {
		let l = unsafe { luaL_newstate() };
		if !l.is_null() {
			Some(Self {
				thread: unsafe { Thread::from_ptr_mut(l) },
			})
		} else {
			None
		}
	}
}

#[repr(transparent)]
struct Thread(c_void);

impl Thread {
	pub const unsafe fn as_ptr(&self) -> *mut State {
		self as *const Self as *mut State
	}

	pub const unsafe fn as_ptr_mut(&mut self) -> *mut State {
		self as *mut Self as *mut State
	}
	
	pub const unsafe fn from_ptr_mut<'a>(l: *mut State) -> &'a mut Self {
		unsafe { &mut *(l as *mut Self) }
	}

	pub unsafe fn open_libs(&mut self) {
		unsafe { luaL_openlibs(self.as_ptr_mut()) }
	}

	pub fn load_string(&self, s: &CStr) -> bool {
		(unsafe { luaL_loadstring(self.as_ptr(), s.as_ptr()) }) == LUA_OK
	}

	pub unsafe fn pcall(&mut self, n_args: c_int, n_results: c_int, err_func: c_int) -> bool {
		(unsafe { lua_pcallk(self.as_ptr_mut(), n_args, n_results, err_func, 0, None) }) == LUA_OK
	}

	pub unsafe fn do_string(&mut self, s: &CStr) -> bool {
		self.load_string(s) && unsafe { self.pcall(0, 0, 0) }
	}

	pub fn val_to_bytes(&mut self, idx: c_int) -> Option<&[u8]> {
		let mut len_c_chars = MaybeUninit::uninit();
		let ptr = unsafe { lua_tolstring(self.as_ptr_mut(), idx, len_c_chars.as_mut_ptr()) };
		if ptr.is_null() {
			return None
		}

		let len_bytes = {
			let len_c_chars = unsafe { len_c_chars.assume_init() };
			const { assert!(align_of::<c_char>() == align_of::<u8>()) }
			len_c_chars * size_of::<c_char>() / size_of::<u8>()
		};
		unsafe { Some(from_raw_parts(ptr as *const u8, len_bytes)) }
	}

	pub fn push_c_function(&mut self, func: CFunction) {
		unsafe { lua_pushcclosure(self.as_ptr_mut(), func, 0) }
	}
}

fn main() {
	std::panic::catch_unwind(move || unsafe {
		let mut lua = Lua::new().expect("failed to create Lua state");
	
		lua.open_libs();
		assert!(lua.do_string(cr#"print("Hello, world!")"#));

		assert!(!lua.do_string(cr#"nonexistent()"#));
		let error = lua.val_to_bytes(-1).unwrap();
		eprintln!("{}", String::from_utf8_lossy(error));
	
		unsafe extern "C-unwind" fn panicking(l: *mut State) -> c_int {
			let _ = l;
			panic!("panicked!");
		}
		lua.push_c_function(panicking);
		assert!(lua.pcall(0, 0, 0));
	}).unwrap_err();
}
