use bstr::BStr;
use std::collections::HashMap;

use crate::convert::Convert;
use crate::extn::core::env::Env;
use crate::extn::core::exception::{ArgumentError, RubyException};
use crate::fs;
use crate::value::Value;
use crate::Artichoke;

#[derive(Debug, Default, Clone, Copy)]
pub struct System;

impl System {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Env for System {
    fn get(&self, interp: &Artichoke, name: &[u8]) -> Result<Value, Box<dyn RubyException>> {
        // Per Rust docs for `std::env::set_var` and `std::env::remove_var`:
        // https://doc.rust-lang.org/std/env/fn.set_var.html
        // https://doc.rust-lang.org/std/env/fn.remove_var.html
        //
        // This function may panic if key is empty, contains an ASCII equals
        // sign '=' or the NUL character '\0', or when the value contains the
        // NUL character.
        if name.is_empty() || memchr::memchr(b'=', name).is_some() {
            // This is a bit of a kludge, but MRI accepts these names on element
            // reference and should always return `nil` since they are invalid
            // at the OS level.
            return Ok(interp.convert(None::<Value>));
        }
        if memchr::memchr(b'\0', name).is_some() {
            return Err(Box::new(ArgumentError::new(
                interp,
                "bad environment variable name: contains null byte",
            )));
        }
        let name = fs::bytes_to_osstr(interp, name)?;
        if let Some(value) = std::env::var_os(name) {
            fs::osstr_to_bytes(interp, value.as_os_str()).map(|bytes| interp.convert(bytes))
        } else {
            Ok(interp.convert(None::<Value>))
        }
    }

    fn put(
        &mut self,
        interp: &Artichoke,
        name: &[u8],
        value: Option<&[u8]>,
    ) -> Result<Value, Box<dyn RubyException>> {
        // Per Rust docs for `std::env::set_var` and `std::env::remove_var`:
        // https://doc.rust-lang.org/std/env/fn.set_var.html
        // https://doc.rust-lang.org/std/env/fn.remove_var.html
        //
        // This function may panic if key is empty, contains an ASCII equals
        // sign '=' or the NUL character '\0', or when the value contains the
        // NUL character.
        if name.is_empty() || memchr::memchr(b'=', name).is_some() {
            // TODO: This should raise `Errno::EINVAL`.
            return Err(Box::new(ArgumentError::new(
                interp,
                format!("Invalid argument - setenv({:?})", <&BStr>::from(name)),
            )));
        }
        if memchr::memchr(b'\0', name).is_some() {
            return Err(Box::new(ArgumentError::new(
                interp,
                "bad environment variable name: contains null byte",
            )));
        }
        if let Some(value) = value {
            if memchr::memchr(b'\0', value).is_some() {
                Err(Box::new(ArgumentError::new(
                    interp,
                    "bad environment variable value: contains null byte",
                )))
            } else {
                std::env::set_var(
                    fs::bytes_to_osstr(interp, name)?,
                    fs::bytes_to_osstr(interp, value)?,
                );
                Ok(interp.convert(value))
            }
        } else {
            let name = fs::bytes_to_osstr(interp, name)?;
            let removed = std::env::var_os(name);
            std::env::remove_var(name);
            if let Some(removed) = removed {
                let removed = fs::osstr_to_bytes(interp, removed.as_os_str())?;
                Ok(interp.convert(removed))
            } else {
                Ok(interp.convert(None::<Value>))
            }
        }
    }

    fn as_map(
        &self,
        interp: &Artichoke,
    ) -> Result<HashMap<Vec<u8>, Vec<u8>>, Box<dyn RubyException>> {
        let mut map = HashMap::default();
        for (name, value) in std::env::vars_os() {
            let name = fs::osstr_to_bytes(interp, name.as_os_str())?;
            let value = fs::osstr_to_bytes(interp, value.as_os_str())?;
            map.insert(name.to_vec(), value.to_vec());
        }
        Ok(map)
    }
}
