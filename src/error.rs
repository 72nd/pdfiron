/// This module contains the errors which can occur during the execution of pdfiron. As this
/// application is primary operated by the user itself the main focus is tho offer concise error
/// messages for humans. Some related errors are unified into their own error class.

use std::convert::From;
use std::error::Error;
use std::fmt;

/// Generic error class for all errors without their own error class. Only contains a error
/// description for the end user.
pub struct GenericError(String);

impl GenericError {
    /// Takes the error message and returns a new GenericError.
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self(msg.into())
    }
}

impl fmt::Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for GenericError {}

/// Error when an needed executable was not found on the system. Informs the user also the
/// possibilities to set an alternative name via a command line argument and/or an environment
/// variable.
pub struct ExecutableNotFound {
    /// Default name of the executable.
    name: String,
    /// Name of the command line argument to set an alternative name of the executable.
    arg: String,
    /// Name of the environment variable which can be used to alter the name of the binary.
    env: String,
    /// Binary name used while the error occurred.
    used: String,
}

impl ExecutableNotFound {
    /// Returns a new ExecutableNotFound error instance.
    pub fn new<S: Into<String>>(name: S, arg: S, env: S, used: S) -> Self {
        Self {
            name: name.into(),
            arg: arg.into(),
            env: env.into(),
            used: used.into(),
        }
    }
}

impl fmt::Display for ExecutableNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.used == self.arg {
            write!(f, "Couldn't find {} on your system under the name {} as specified by you with the --{} argument.", self.name, self.used, self.arg)
        } else if self.used == self.env {
            write!(f, "Couldn't find {} on your system under the name {} as specified by you with the {} environment variable.", self.name, self.used, self.env)
        } else {
            write!(f, "Couldn't find {} on your system, please make sure {} is installed on your system. You can use the --{} argument or the environment variable {} to set an alternative binary name.", self.name, self.name, self.arg, self.env)
        }
    }
}

impl fmt::Debug for ExecutableNotFound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

