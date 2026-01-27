// src/routes/mod.rs
mod login;
mod logout;
mod root;
mod signup;
mod verify_2fa;
mod verify_token;

// re-export items from sub-modules
pub use login::*;
pub use logout::*;
pub use root::*;
pub use signup::*;
pub use verify_2fa::*;
pub use verify_token::*;
