use std::borrow::Cow;
use std::fmt::Debug;
use std::panic::Location;
use std::{error::Error, fmt::Display};

use serenity::prelude::{TypeMap, TypeMapKey};

pub type CommandResult = anyhow::Result<Cow<'static, str>>;

#[track_caller]
pub fn get_data<T: TypeMapKey>(typemap: &TypeMap) -> Result<&T::Value, GetDataError> {
    if let Some(result) = typemap.get::<T>() {
        return Ok(result);
    }

    let caller = Location::caller();
    Err(GetDataError { location: *caller })
}

#[derive(Clone, Copy)]
pub struct GetDataError {
    location: Location<'static>,
}

impl Display for GetDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "In file {}:{}:{}: Failed to access global data",
            self.location.file(),
            self.location.line(),
            self.location.column()
        )
    }
}
impl Debug for GetDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GetDataError").field("location", &self.location).finish()
    }
}

impl Error for GetDataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "Failed to access global data"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}
