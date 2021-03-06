mod buffer;
mod cykv;

pub use cykv::*;

use crate::*;

pub trait KvEngine: Clone + Send + 'static {
    // Get the value
    fn get(&self, key: String) -> Result<Option<String>>;

    // Scan values
    fn scan(&self, begin: String, end: String) -> Result<Vec<String>>;

    // Set the value
    // return the previous value
    fn set(&self, key: String, value: String) -> Result<()>;

    // Remove the key-value pair
    // return the removed value
    fn remove(&self, key: String) -> Result<()>;
}
