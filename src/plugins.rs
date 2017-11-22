use failure::Error;

pub trait Plugin {
    fn name(&self) -> &str;
    fn config_keys(&self) -> Vec<String>;
    fn config_callback(&mut self, key: String, value: String) -> Result<(), Error>;
    fn report_values(&self) -> Result<(), Error>;
}

#[macro_export]
macro_rules! collectd_plugin {
    ($plugin:ident) => {
        use std::os::raw::{c_char, c_int};
        use collectd_plugin::bindings::{plugin_register_config, plugin_register_read};
        use std::ffi::{CString, CStr};
        use std::mem;
        use collectd_plugin::{LogLevel, collectd_log};

        #[no_mangle]
        pub extern "C" fn module_register2() {
            let pl = $plugin.lock().unwrap();

            // Use expects for assertions -- no one really should be passing us strings that
            // contain nulls
            let s = CString::new(pl.name()).expect("Plugin name to not contain nulls");

            let ck: Vec<CString> = pl.config_keys()
                .into_iter()
                .map(|x| CString::new(x).expect("Config key to not contain nulls"))
                .collect();

            // Now grab all the pointers to the c strings for ffi
            let mut pointers: Vec<*const c_char> = ck.iter().map(|arg| arg.as_ptr()).collect();

            unsafe {
                plugin_register_read(s.as_ptr(), Some(my_plugin_read));
                plugin_register_config(
                    s.as_ptr(),
                    Some(my_config),
                    pointers.as_mut_ptr(),
                    pointers.len() as i32,
                );
            }

            // We must forget the vector as collectd hangs on to the info and if we were to drop
            // it, collectd would segfault trying to read the newly freed up data structure
            mem::forget(ck);
            mem::forget(pointers);
        }

        #[no_mangle]
        pub extern "C" fn my_plugin_read() -> c_int {
            if let Err(ref e) = $plugin.lock().unwrap().report_values() {
                collectd_log(LogLevel::Error, &format!("read error: {}", e));
                return -1;
            }
            0
        }

        #[no_mangle]
        pub unsafe extern "C" fn my_config(key: *const c_char, value: *const c_char) -> c_int {
            if let Ok(key) = CStr::from_ptr(key).to_owned().into_string() {
                if let Ok(value) = CStr::from_ptr(value).to_owned().into_string() {
                    if let Err(ref e) = $plugin.lock().unwrap().config_callback(key, value) {
                        collectd_log(LogLevel::Error, &format!("config error: {}", e));
                        return -1;
                    } else {
                        return 0;
                    }
                }
            }
            -1
        }
    };
}
