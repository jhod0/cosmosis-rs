use std::os::raw;
use std::ffi;

mod bindings;
pub use bindings::root::{DATABLOCK_STATUS, datablock_type_t};

type CosmosisResult<T> = Result<T, DATABLOCK_STATUS>;

fn wrap_cosmosis_result<T>(obj: T, err: DATABLOCK_STATUS) -> CosmosisResult<T> {
    if err == DATABLOCK_STATUS::DBS_SUCCESS {
        Ok(obj)
    } else {
        Err(err)
    }
}

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

    /// Retrieve a value from a DataBlock
    pub fn get<T>(&mut self, section: &str, name: &str) -> CosmosisResult<T>
        where T: CosmosisDataType {
        T::get_datablock(self, section, name)
    }

    /// Store a new value in a DataBlock. Fails if an entry already exists for `(section, name)`
    pub fn put<T>(&mut self, section: &str, name: &str, obj: T) -> CosmosisResult<()>
        where T: CosmosisDataType {
        T::put_datablock(self, section, name, obj)
    }
}

pub trait CosmosisDataType: Sized {
    fn cosmosis_type() -> datablock_type_t;
    fn get_datablock(&mut DataBlock, section: &str, name: &str) -> CosmosisResult<Self>;
    fn put_datablock(&mut DataBlock, section: &str, name: &str, Self) -> CosmosisResult<()>;
}

impl CosmosisDataType for raw::c_int {
    fn cosmosis_type() -> datablock_type_t {
        datablock_type_t::DBT_INT
    }

    fn get_datablock(db: &mut DataBlock, section: &str, name: &str) -> CosmosisResult<Self> {
        let mut n: Self = 0;
        let result = unsafe {
            bindings::root::c_datablock_get_int(db.ptr,
                                                ffi::CString::new(section).unwrap().as_ptr(),
                                                ffi::CString::new(name).unwrap().as_ptr(),
                                                &mut n)
        };
        wrap_cosmosis_result(n, result)
    }

    fn put_datablock(db: &mut DataBlock, section: &str, name: &str, n: Self) -> CosmosisResult<()> {
        let result = unsafe {
            bindings::root::c_datablock_put_int(db.ptr,
                                                ffi::CString::new(section).unwrap().as_ptr(),
                                                ffi::CString::new(name).unwrap().as_ptr(),
                                                n)
        };
        wrap_cosmosis_result((), result)
    }
}

#[cfg(test)]
mod tests {
    use super::{DataBlock, CosmosisDataType};
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
        }
    }
}
