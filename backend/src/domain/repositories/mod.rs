// Repository traits are defined here
// Concrete implementations are in infrastructure/repositories/

pub mod report_repository;

pub use report_repository::*;

pub trait UserRepository {}
pub trait InvoiceRepository {}
pub trait ClientRepository {}
