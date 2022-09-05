pub mod admin;
pub mod chat;
// pub mod health;
pub mod info;
pub mod magazine;
pub mod paste;
pub mod paste_next;
pub mod qqbot;
// pub mod record;

#[cfg(not(feature = "qqbot"))]
compile_error!("you need to prune `units::qqbot` module and `ricq` dependency manually");
