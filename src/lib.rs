pub mod api_contract;
pub mod application;
pub mod identity;
pub mod pairing;
mod platform_time;
pub mod results;
pub mod scheduling;
pub mod simulation;
mod table;
pub mod tournament;

#[cfg(not(target_arch = "wasm32"))]
#[path = "../server/src/lib.rs"]
pub mod backend;
