extern crate libc;

use std::convert;
use std::error;
use std::ffi::{CStr, CString};
use std::fmt;
use std::os::raw;

mod bindings;
pub use bindings::root::{DATABLOCK_STATUS, datablock_type_t};
pub use bindings::root::__BindgenComplex as Complex;

impl fmt::Display for DATABLOCK_STATUS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for DATABLOCK_STATUS {
    fn description(&self) -> &str {
        match *self {
            DATABLOCK_STATUS::DBS_SUCCESS => "DBS_SUCCESS",
            DATABLOCK_STATUS::DBS_DATABLOCK_NULL => "DBS_DATABLOCK_NULL",
            DATABLOCK_STATUS::DBS_SECTION_NULL => "DBS_SECTION_NULL",
            DATABLOCK_STATUS::DBS_SECTION_NOT_FOUND => "DBS_SECTION_NOT_FOUND",
            DATABLOCK_STATUS::DBS_NAME_NULL => "DBS_NAME_NULL",
            DATABLOCK_STATUS::DBS_NAME_NOT_FOUND => "DBS_NAME_NOT_FOUND",
            DATABLOCK_STATUS::DBS_NAME_ALREADY_EXISTS => "DBS_NAME_ALREADY_EXISTS",
            DATABLOCK_STATUS::DBS_VALUE_NULL => "DBS_VALUE_NULL",
            DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE => "DBS_WRONG_VALUE_TYPE",
            DATABLOCK_STATUS::DBS_MEMORY_ALLOC_FAILURE => "DBS_MEMORY_ALLOC_FAILURE",
            DATABLOCK_STATUS::DBS_SIZE_NULL => "DBS_SIZE_NULL",
            DATABLOCK_STATUS::DBS_SIZE_NONPOSITIVE => "DBS_SIZE_NONPOSITIVE",
            DATABLOCK_STATUS::DBS_SIZE_INSUFFICIENT => "DBS_SIZE_INSUFFICIENT",
            DATABLOCK_STATUS::DBS_NDIM_NONPOSITIVE => "DBS_NDIM_NONPOSITIVE",
            DATABLOCK_STATUS::DBS_NDIM_OVERFLOW => "DBS_NDIM_OVERFLOW",
            DATABLOCK_STATUS::DBS_NDIM_MISMATCH => "DBS_NDIM_MISMATCH",
            DATABLOCK_STATUS::DBS_EXTENTS_NULL => "DBS_EXTENTS_NULL",
            DATABLOCK_STATUS::DBS_EXTENTS_MISMATCH => "DBS_EXTENTS_MISMATCH",
            DATABLOCK_STATUS::DBS_LOGIC_ERROR => "DBS_LOGIC_ERROR",
            DATABLOCK_STATUS::DBS_USED_DEFAULT => "DBS_USED_DEFAULT"
        }
    }
}

#[derive(Debug)]
/// Error type for CosmoSIS. Wraps a `DATABLOCK_STATUS` and contains an optional
/// error message.
pub struct CosmosisError {
    pub kind: DATABLOCK_STATUS,
    reason: Option<String>
}

impl fmt::Display for CosmosisError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CosmosisError")?;
        if self.kind == DATABLOCK_STATUS::DBS_SUCCESS {
            write!(f, "(success)")
        } else {
            write!(f, "(error: {}", error::Error::description(&self.kind))?;
            if self.reason.is_some() {
                write!(f, ", reason: {})", self.reason.as_ref().unwrap())
            } else {
                write!(f, ")")
            }
        }
    }
}

impl error::Error for CosmosisError {
    fn description(&self) -> &str {
        if self.reason.is_some() {
            self.reason.as_ref().map(|s| &*s).unwrap()
        } else {
            self.kind.description()
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        Some(&self.kind)
    }
}

impl convert::From<DATABLOCK_STATUS> for CosmosisError {
    fn from(obj: DATABLOCK_STATUS) -> Self {
        CosmosisError { kind: obj, reason: None }
    }
}

pub type CosmosisResult<T> = Result<T, CosmosisError>;

macro_rules! wrap_cosmosis_result {
    ( $err:expr, $obj:expr ) => {
        if $err == DATABLOCK_STATUS::DBS_SUCCESS {
            Ok($obj)
        } else {
            Err(CosmosisError { kind: $err, reason: None })
        }
    };
    ( $err:expr, $obj:expr, $( $fmt_arg:expr ),* ) => {
        if $err == DATABLOCK_STATUS::DBS_SUCCESS {
            Ok($obj)
        } else {
            Err(CosmosisError { kind: $err, reason: Some(format!($( $fmt_arg ),*)) })
        }
    }
}

/// CosmoSIS Data Storage block, all input parameters and outputs are passed through
/// DataBlocks.
pub struct DataBlock {
    ptr: *mut bindings::root::c_datablock
}

impl Default for DataBlock {
    fn default() -> Self {
        DataBlock {
            ptr: unsafe { bindings::root::make_c_datablock() }
        }
    }
}

impl Clone for DataBlock {
    fn clone(&self) -> Self {
        DataBlock {
            ptr: unsafe { bindings::root::clone_c_datablock(self.ptr) }
        }
    }
}

impl Drop for DataBlock {
    fn drop(&mut self) {
        unsafe {
            bindings::root::destroy_c_datablock(self.ptr);
        }
    }
}

impl DataBlock {
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns the type of the DataBlock entry, or `None` if no such entry exists.
    pub fn get_type(&self, section: &str, name: &str) -> Option<datablock_type_t> {
        let mut ty: datablock_type_t = datablock_type_t::DBT_UNKNOWN;
        let result = unsafe {
            bindings::root::c_datablock_get_type(self.ptr,
                                                 CString::new(section).unwrap().as_ptr(),
                                                 CString::new(name).unwrap().as_ptr(),
                                                 &mut ty)
        };
        if result == DATABLOCK_STATUS::DBS_NAME_NOT_FOUND {
            None
        } else {
            Some(ty)
        }
    }

    /// Retrieve a value from a DataBlock.
    pub fn get<T>(&mut self, section: &str, name: &str) -> CosmosisResult<T>
        where T: CosmosisDataType {
        T::get_datablock(self, section, name)
    }

    /// Store a new value in a DataBlock. Fails if an entry already exists for `(section, name)`.
    pub fn put<T>(&mut self, section: &str, name: &str, obj: T) -> CosmosisResult<()>
        where T: CosmosisDataType {
        T::put_datablock(self, section, name, obj)
    }
}

/// Types which can be stored and retrieved from `DataBlock`s are `CosmosisDataType`s.
pub trait CosmosisDataType: Sized {
    fn cosmosis_type() -> datablock_type_t;
    fn get_datablock(&mut DataBlock, section: &str, name: &str) -> CosmosisResult<Self>;
    fn put_datablock(&mut DataBlock, section: &str, name: &str, Self) -> CosmosisResult<()>;
}

macro_rules! gen_cosmosis_data_type {
    ( $rust_name:ty, $cosmo_name:ident, $default_val:expr,
      // Unfortunately, concat_idents! is unstable
      $getter:path, $putter:path) => {
        impl CosmosisDataType for $rust_name {
            fn cosmosis_type() -> datablock_type_t {
                datablock_type_t::$cosmo_name
            }

            fn get_datablock(db: &mut DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
                let mut n: Self = $default_val;
                let retval = unsafe {
                    $getter(db.ptr,
                            CString::new(section).unwrap().as_ptr(),
                            CString::new(name).unwrap().as_ptr(),
                            &mut n)
                };
                wrap_cosmosis_result!(retval, n, "Could not get value at (section, name): ({}, {})",
                                      section, name)
            }

            fn put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: Self) -> CosmosisResult<()> {
                let retval = unsafe {
                    $putter(db.ptr,
                            CString::new(section).unwrap().as_ptr(),
                            CString::new(name).unwrap().as_ptr(),
                            obj)
                };
                wrap_cosmosis_result!(retval, (), "Could not put value at (section, name): ({}, {})",
                                      section, name)

            }
        }
    }
}

gen_cosmosis_data_type!(raw::c_int, DBT_INT, 0,
                        bindings::root::c_datablock_get_int,
                        bindings::root::c_datablock_put_int);
gen_cosmosis_data_type!(bool, DBT_BOOL, false,
                        bindings::root::c_datablock_get_bool,
                        bindings::root::c_datablock_put_bool);
gen_cosmosis_data_type!(f64, DBT_DOUBLE, 0.0,
                        bindings::root::c_datablock_get_double,
                        bindings::root::c_datablock_put_double);
gen_cosmosis_data_type!(Complex<f64>, DBT_COMPLEX, Complex { re: 0.0, im: 0.0 },
                        bindings::root::c_datablock_get_complex,
                        bindings::root::c_datablock_put_complex);

impl CosmosisDataType for CString {
    fn cosmosis_type() -> datablock_type_t {
        datablock_type_t::DBT_STRING
    }

    fn get_datablock(db: &mut DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
        let mut cstr: *mut raw::c_char = std::ptr::null_mut();
        let retval = unsafe {
            bindings::root::c_datablock_get_string(db.ptr,
                                                   CString::new(section).unwrap().as_ptr(),
                                                   CString::new(name).unwrap().as_ptr(),
                                                   &mut cstr)
        };
        wrap_cosmosis_result!(retval, 
            unsafe {
                let cstr_ref = CStr::from_ptr(cstr);
                // Yes, this would be an unnecessary allocation, but we must clone
                // into Rust's heap because otherwise (i.e. CString::from_raw(cstr))
                // Rust's memory allocator would attempt to free a pointer from C's
                // heap - undefined
                let output_string = CString::new(cstr_ref.to_str().unwrap()).unwrap();
                libc::free(cstr as *mut libc::c_void);
                output_string
            },
            "Could not get value at (section, name): ({}, {})", section, name)
    }

    fn put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: Self) -> CosmosisResult<()> {
        let retval = unsafe {
            bindings::root::c_datablock_put_string(db.ptr,
                                                   CString::new(section).unwrap().as_ptr(),
                                                   CString::new(name).unwrap().as_ptr(),
                                                   obj.as_ptr())
        };
        wrap_cosmosis_result!(retval, (), "Could not put value at (section, name): ({}, {})",
                              section, name)
    }
}

#[cfg(test)]
mod tests {
    use super::{DataBlock, DATABLOCK_STATUS, datablock_type_t};
    use std::os::raw;

    #[test]
    fn test_put_get() {
        let mut db = DataBlock::new();
        let numbers: Vec<(_, raw::c_int)> = vec![("one", 1), ("two", 2), ("three", 3)];

        for (name, val) in numbers.iter() {
            assert!(db.put("my_section", name, *val).is_ok())
        }

        for (name, val) in numbers.iter() {
            assert!(db.get::<raw::c_int>("my_section", name).expect("should be present")
                    == *val);
            assert!(db.get::<f64>("my_section", name).unwrap_err().kind
                    == DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE);
        }

        for name in ["four", "five", "six", "seven", "eight"].iter() {
            assert!(db.get::<raw::c_int>("my_section", name).unwrap_err().kind
                    == DATABLOCK_STATUS::DBS_NAME_NOT_FOUND)
        }
    }

    #[test]
    fn test_wrong_type() {
        let mut db = DataBlock::new();
        let numbers: Vec<(_, f64)> = vec![("hello", 1.0), ("there", 3.2), ("pal", -1.324)];

        for (name, val) in numbers.iter() {
            assert!(db.put("my_section", name, *val).is_ok())
        }

        for (name, _) in numbers.iter() {
            assert!(db.get::<raw::c_int>("my_section", name).unwrap_err().kind
                    == DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE);
            assert!(db.get_type("my_section", name).unwrap()
                    == datablock_type_t::DBT_DOUBLE);
        }
    }
}
