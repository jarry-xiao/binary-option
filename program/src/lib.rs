pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;
pub mod utils;
// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;

solana_program::declare_id!("5UBdr5zwP8kN54xeTRZ6LyAUqLyRMMuP77v3r3jQowMr");
