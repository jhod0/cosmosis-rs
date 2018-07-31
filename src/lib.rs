extern crate libc;

use std::borrow::Borrow;
use std::convert::From;
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

impl CosmosisError {
    pub fn new(kind: DATABLOCK_STATUS) -> Self {
        CosmosisError { kind, reason: None }
    }

    pub fn with_reason(self, reason: String) -> Self {
        CosmosisError { reason: Some(reason), ..self }
    }
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

impl From<DATABLOCK_STATUS> for CosmosisError {
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

    /// Whether or not the datablock contains a value `name` in the section
    /// `section`.
    pub fn contains(&self, section: &str, name: &str) -> bool {
        unsafe {
            bindings::root::c_datablock_has_value(self.ptr,
                                                 CString::new(section).unwrap().as_ptr(),
                                                 CString::new(name).unwrap().as_ptr())
        }
    }

    /// Whether or not this `DataBlock` contains a section of the given name.
    pub fn contains_section(&self, section: &str) -> bool {
        unsafe {
            bindings::root::c_datablock_has_section(self.ptr,
                                                    CString::new(section).unwrap().as_ptr())
        }
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

    /// Whether or not the `DataBlock` contains an entry with the given `section`
    /// and `name`, of the type `C`. If the entry is of a different type, or if
    /// the there is no such entry, returns false.
    pub fn is_type<C: CosmosisGettable>(&self, section: &str, name: &str) -> bool {
        self.get_type(section, name).map(|t| t == C::InternalType::cosmosis_type()).unwrap_or(false)
    }

    /// Retrieve a value from a DataBlock.
    pub fn get<T>(&self, section: &str, name: &str) -> CosmosisResult<T>
        where T: CosmosisGettable {
        T::get_datablock(self, section, name)
    }

    /// Stores the given object into the `DataBlock`, associated with the given section and name.
    /// If an object is already stored (of the same type) in that name, replaces and returns that
    /// previous value; if the name does not exist already in the `DataBlock`, creates a new entry.
    pub fn insert<T, I>(&mut self, section: &str, name: &str, obj: I) -> CosmosisResult<Option<T::ResultType>>
        where T: CosmosisStorable,
              I: Borrow<T> {
        if self.contains(section, name) {
            T::replace_datablock(self, section, name, obj.borrow())
               .map(|s| Some(s))
        } else {
            T::put_datablock(self, section, name, obj.borrow())
               .map(|()| None)
        }
    }

    /// Store a new value in a DataBlock. Fails if an entry already exists for `(section, name)`.
    pub fn put<T, I>(&mut self, section: &str, name: &str, obj: I) -> CosmosisResult<()>
        where T: CosmosisStorable + ?Sized,
              I: Borrow<T> {
        T::put_datablock(self, section, name, obj.borrow())
    }
}

/// Types which can be stored and retrieved from `DataBlock`s are `CosmosisDataType`s.
pub trait CosmosisDataType: Sized {
    type InsertRepr: ?Sized;
    fn cosmosis_type() -> datablock_type_t;
    fn direct_get_datablock(&DataBlock, section: &str, name: &str) -> CosmosisResult<Self>;
    fn direct_put_datablock(&mut DataBlock, section: &str, name: &str, obj: &Self::InsertRepr) -> CosmosisResult<()>;
    fn direct_replace_datablock(&mut DataBlock, section: &str, name: &str, obj: &Self::InsertRepr) -> CosmosisResult<Self>;
}

/// Represents types which may be retrieved from a `DataBlock`.
pub trait CosmosisGettable: Sized {
    type InternalType: CosmosisDataType;
    fn get_datablock(&DataBlock, section: &str, name: &str) -> CosmosisResult<Self>;
}

impl<T> CosmosisGettable for T where T: CosmosisDataType {
    type InternalType = Self;
    fn get_datablock(db: &DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
        Self::direct_get_datablock(db, section, name)
    }
}

/// Represents types which may be stored in a `DataBlock`.
pub trait CosmosisStorable {
    type InternalType: CosmosisDataType;
    type ResultType: CosmosisGettable;
    fn put_datablock(&mut DataBlock, section: &str, name: &str, obj: &Self) -> CosmosisResult<()>;
    fn replace_datablock(&mut DataBlock, section: &str, name: &str, obj: &Self) -> CosmosisResult<Self::ResultType>;
}

impl<T> CosmosisStorable for T where T: CosmosisDataType<InsertRepr=T> {
    type InternalType = Self;
    type ResultType = Self;
    fn put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &Self) -> CosmosisResult<()> {
        Self::direct_put_datablock(db, section, name, obj)
    }
    fn replace_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &Self) -> CosmosisResult<Self> {
        Self::direct_replace_datablock(db, section, name, obj)
    }
}

macro_rules! gen_cosmosis_data_type {
    ( $rust_name:ty, $cosmo_name:ident, $default_val:expr,
      // Unfortunately, concat_idents! is unstable
      $getter:path, $putter:path, $replacer:path ) => {
        impl CosmosisDataType for $rust_name {
            type InsertRepr = Self;
            fn cosmosis_type() -> datablock_type_t {
                datablock_type_t::$cosmo_name
            }

            fn direct_get_datablock(db: &DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
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

            fn direct_put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &$rust_name) -> CosmosisResult<()> {
                let retval = unsafe {
                    $putter(db.ptr,
                            CString::new(section).unwrap().as_ptr(),
                            CString::new(name).unwrap().as_ptr(),
                            *obj)
                };
                wrap_cosmosis_result!(retval, (), "Could not put value at (section, name): ({}, {})",
                                      section, name)
            }

            fn direct_replace_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &$rust_name) -> CosmosisResult<Self> {
                let result = Self::direct_get_datablock(db, section, name)?;
                let retval = unsafe {
                    $replacer(db.ptr,
                              CString::new(section).unwrap().as_ptr(),
                              CString::new(name).unwrap().as_ptr(),
                              *obj)
                };
                wrap_cosmosis_result!(retval, result, "Could not get value at (section, name): ({}, {})",
                                      section, name)
            }
        }
    }
}

gen_cosmosis_data_type!(raw::c_int, DBT_INT, 0,
                        bindings::root::c_datablock_get_int,
                        bindings::root::c_datablock_put_int,
                        bindings::root::c_datablock_replace_int);
gen_cosmosis_data_type!(bool, DBT_BOOL, false,
                        bindings::root::c_datablock_get_bool,
                        bindings::root::c_datablock_put_bool,
                        bindings::root::c_datablock_replace_bool);
gen_cosmosis_data_type!(f64, DBT_DOUBLE, 0.0,
                        bindings::root::c_datablock_get_double,
                        bindings::root::c_datablock_put_double,
                        bindings::root::c_datablock_replace_double);
gen_cosmosis_data_type!(Complex<f64>, DBT_COMPLEX, Complex { re: 0.0, im: 0.0 },
                        bindings::root::c_datablock_get_complex,
                        bindings::root::c_datablock_put_complex,
                        bindings::root::c_datablock_replace_complex);

macro_rules! gen_cosmosis_vector_type {
    ( $rust_name:ty, $cosmo_name:ident,
      $getter:path, $putter:path, $replacer:path ) => {
        impl CosmosisDataType for Vec<$rust_name> {
            type InsertRepr = [$rust_name];

            fn cosmosis_type() -> datablock_type_t {
                datablock_type_t::$cosmo_name
            }

            fn direct_get_datablock(db: &DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
                let mut size = unsafe {
                    bindings::root::c_datablock_get_array_length(db.ptr,
                                                                 CString::new(section).unwrap().as_ptr(),
                                                                 CString::new(name).unwrap().as_ptr())
                };
                if size < 0 {
                    if db.contains(section, name) {
                        Err(CosmosisError::new(DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE)
                                          .with_reason(format!("Not a 1D Double array at (section, name): ({}, {})",
                                                               section, name)))
                    } else {
                        Err(CosmosisError::new(DATABLOCK_STATUS::DBS_NAME_NOT_FOUND)
                                          .with_reason(format!("No value at (section, name): ({}, {})",
                                                               section, name)))
                    }
                } else {
                    let mut vec = Vec::with_capacity(size as usize);
                    let retval = unsafe {
                        vec.set_len(size as usize);
                        $getter(db.ptr,
                                CString::new(section).unwrap().as_ptr(),
                                CString::new(name).unwrap().as_ptr(),
                                vec.as_mut_ptr(),
                                &mut size,
                                size)
                    };
                    wrap_cosmosis_result!(retval, vec,
                                          "Could not get value at (section, name): ({}, {})", section, name)
                }
            }

            fn direct_put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &Self::InsertRepr) -> CosmosisResult<()> {
                let retval = unsafe {
                    $putter(db.ptr,
                            CString::new(section).unwrap().as_ptr(),
                            CString::new(name).unwrap().as_ptr(),
                            obj.as_ptr(),
                            obj.len() as raw::c_int)
                };
                wrap_cosmosis_result!(retval, (), "Could not put value at (section, name): ({}, {})",
                                      section, name)
            }

            fn direct_replace_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &Self::InsertRepr) -> CosmosisResult<Self> {
                let result = Self::direct_get_datablock(db, section, name)?;
                let retval = unsafe {
                    $replacer(db.ptr,
                              CString::new(section).unwrap().as_ptr(),
                              CString::new(name).unwrap().as_ptr(),
                              obj.as_ptr(),
                              obj.len() as raw::c_int)
                };
                wrap_cosmosis_result!(retval, result, "Could not replace value at (section, name): ({}, {})",
                                      section, name)
            }
        }

        impl CosmosisStorable for [$rust_name] {
            type InternalType = Vec<$rust_name>;
            type ResultType = Vec<$rust_name>;
            fn put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &Self) -> CosmosisResult<()> {
                Self::InternalType::direct_put_datablock(db, section, name, obj)
            }
            fn replace_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &Self) -> CosmosisResult<Self::ResultType> {
                Self::InternalType::direct_replace_datablock(db, section, name, obj)
            }
        }
    }
}

gen_cosmosis_vector_type!(f64, DBT_DOUBLE1D,
                          bindings::root::c_datablock_get_double_array_1d_preallocated,
                          bindings::root::c_datablock_put_double_array_1d,
                          bindings::root::c_datablock_replace_double_array_1d);
gen_cosmosis_vector_type!(raw::c_int, DBT_INT1D,
                          bindings::root::c_datablock_get_int_array_1d_preallocated,
                          bindings::root::c_datablock_put_int_array_1d,
                          bindings::root::c_datablock_replace_int_array_1d);
gen_cosmosis_vector_type!(Complex<f64>, DBT_COMPLEX1D,
                          bindings::root::c_datablock_get_complex_array_1d_preallocated,
                          bindings::root::c_datablock_put_complex_array_1d,
                          bindings::root::c_datablock_replace_complex_array_1d);

impl CosmosisDataType for CString {
    type InsertRepr = CStr;

    fn cosmosis_type() -> datablock_type_t {
        datablock_type_t::DBT_STRING
    }

    fn direct_get_datablock(db: &DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
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

    fn direct_put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &CStr) -> CosmosisResult<()> {
        let retval = unsafe {
            bindings::root::c_datablock_put_string(db.ptr,
                                                   CString::new(section).unwrap().as_ptr(),
                                                   CString::new(name).unwrap().as_ptr(),
                                                   obj.as_ptr())
        };
        wrap_cosmosis_result!(retval, (), "Could not put value at (section, name): ({}, {})",
                              section, name)
    }

    fn direct_replace_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &CStr) -> CosmosisResult<Self> {
        let result = Self::direct_get_datablock(db, section, name)?;
        let retval = unsafe {
            bindings::root::c_datablock_replace_string(db.ptr,
                                                       CString::new(section).unwrap().as_ptr(),
                                                       CString::new(name).unwrap().as_ptr(),
                                                       obj.as_ptr())
        };
        wrap_cosmosis_result!(retval, result,
                              "Could not replace value at (section, name): ({}, {})", section, name)
    }
}

impl CosmosisGettable for String {
    type InternalType = CString;
    fn get_datablock(db: &DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
        CString::direct_get_datablock(db, section, name)
                .map(|cstr| cstr.into_string().expect("DataBlock should contain valid UTF-8"))
    }
}

impl CosmosisStorable for str {
    type InternalType = CString;
    type ResultType = String;

    fn put_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &str) -> CosmosisResult<()> {
        CString::direct_put_datablock(db, section, name, &CString::new(obj).unwrap())
    }

    fn replace_datablock(db: &mut DataBlock, section: &str, name: &str, obj: &str) -> CosmosisResult<String> {
        CString::direct_replace_datablock(db, section, name, &CString::new(obj).unwrap())
                .map(|cstr| cstr.into_string().expect("DataBlock should contain valid UTF-8"))
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
            assert!(db.put("my_section", name, *val).is_ok());
            assert!(db.contains("my_section", name));
        }

        for (name, val) in numbers.iter() {
            assert!(db.contains("my_section", name));
            assert_eq!(db.get::<raw::c_int>("my_section", name).expect("should be present"),
                       *val);
            assert_eq!(db.get::<f64>("my_section", name).unwrap_err().kind,
                       DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE);
        }

        for name in ["four", "five", "six", "seven", "eight"].iter() {
            assert_eq!(db.get::<raw::c_int>("my_section", name).unwrap_err().kind,
                       DATABLOCK_STATUS::DBS_NAME_NOT_FOUND);
            assert!(!db.contains("my_section", name));
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
            assert_eq!(db.get::<raw::c_int>("my_section", name).unwrap_err().kind,
                       DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE);
            assert_eq!(db.get_type("my_section", name).unwrap(),
                       datablock_type_t::DBT_DOUBLE);
        }

        for name in ["four", "five", "six", "seven", "eight"].iter() {
            assert!(db.get_type("my_section", name).is_none());
        }
    }

    #[test]
    fn test_replace() {
        let mut db = DataBlock::new();
        let numbers: Vec<(_, raw::c_int)> = vec![("one", 1), ("two", 2), ("three", 3)];

        for (name, val) in numbers.iter() {
            assert!(db.put("my_section", name, *val).is_ok());
            assert!(db.contains("my_section", name));
        }

        for (name, val) in numbers.iter() {
            let inserted = db.insert("my_section", name, *val + 1).unwrap();
            assert!(inserted.is_some());
            assert_eq!(inserted.unwrap(), *val);
            assert_eq!(db.get::<raw::c_int>("my_section", name).unwrap(), *val + 1);
        }
    }

    #[test]
    fn test_put_get_vec() {
        let mut db = DataBlock::new();
        let data: Vec<(_, Vec<f64>)> = vec![("one", vec![1.0, 2.0, 3.0]),
                                            ("two", vec![0.0, 2.0, 4.0]),
                                            ("archnemesis", vec![5.0, 6.0, 7.0])];

        for (name, val) in data.iter() {
            assert!(db.put::<[f64], &[f64]>("my_section", name, val).is_ok());
            assert!(db.contains("my_section", name));
        }

        for (name, val) in data.iter() {
            assert!(db.contains("my_section", name));
            assert_eq!(db.get::<Vec<f64>>("my_section", name).expect("should be present"), &val[..]);
            assert_eq!(db.get::<f64>("my_section", name).unwrap_err().kind, DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE);
        }
    }

    #[test]
    fn test_put_get_string() {
        let mut db = DataBlock::new();
        let data: Vec<(&'static str, &'static str)> = vec![("a", "artichoke"),
                                                           ("b", "bear"),
                                                           ("c", "caterpillar"),
                                                           ("d", "dandelion")];

        for (name, val) in data.iter() {
            assert!(db.put::<str, _>("my_section", name, *val).is_ok());
            assert!(db.contains("my_section", name));
        }

        for (name, val) in data.iter() {
            assert!(db.contains("my_section", name));
            assert_eq!(db.get::<String>("my_section", name).expect("should be present"), *val);
            assert_eq!(db.get::<f64>("my_section", name).unwrap_err().kind, DATABLOCK_STATUS::DBS_WRONG_VALUE_TYPE);
        }
    }
}
