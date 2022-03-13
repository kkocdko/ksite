@echo off
start "" cpu-rate.exe 99
cargo build
start "" cpu-rate.exe 0 500
cargo run
