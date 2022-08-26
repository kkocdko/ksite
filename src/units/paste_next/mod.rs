#![allow(unused)]
/*
剪贴板和文件存储服务，读写本地文件，hash分流

账户登录
文件存储
加密
CRUI 轻量的界面
在用户之间分享
类似 git fork，cow
尽量优化性能
有限制的自增和其他结合的 ID

密码用hash，用户名和密码都固定宽度？优化性能？
页面缓存，LRU？

https://www.runoob.com/sqlite/sqlite-intro.html

*/
use crate::db;
// use crate::utils::slot;
use axum::extract::{Form, Path};
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use serde::Deserialize;

fn db_init() {
    // uid: user id (fixed len)
    // upw: user password (hashed) (fixed len)
    // cid: clipboard id (uint)
    // cpw: clipboard password (hashed) (fixed len)

    let sqls = [
        // fixed length, fill empty space by \0
        "CREATE TABLE paste_user (uid TEXT, upw BLOB)",
        // data table
        "CREATE TABLE paste_data (cid INTEGER PRIMARY KEY AUTOINCREMENT, uid TEXT, data BLOB)",
    ];
    for sql in sqls {
        db!(sql).ok();
    }
}
fn db_user_c() {}
fn db_user_u() {}
fn db_user_r() {}
fn db_user_d() {}
fn db_data_c() {}
fn db_data_u() {}
fn db_data_r() {}
fn db_data_d() {}

pub fn service() -> Router {
    db_init();
    // mentions about the path later?
    Router::new()
}
