#![allow(unused)]
/*
剪贴板和文件存储服务，读写本地文件，hash分流

账户登录
文件存储
加密
CRUI 轻量的界面
在用户之间分享
类似 git fork
尽量优化性能
有限制的自增和其他结合的 ID

区分创建与插入？评估性能影响
密码用hash，用户名和密码都固定宽度？优化性能？
页面缓存，LRU？

https://www.runoob.com/sqlite/sqlite-intro.html

用户名和密码都用hash

用户频率限制，空间限制，会员制？
写入原始内容，前段自行处理转义
内部用数字存储cid，文件名也是数字

/paste/raw/:id

-----
/ksite
/ksite.db

*/
use crate::{db, strip_str};
// use crate::utils::slot;
use axum::extract::{Form, Path};
use axum::response::{Html, Redirect};
use axum::routing::MethodRouter;
use axum::Router;
use serde::Deserialize;

fn db_init() {
    // uid: user id (hashed)
    // upw: user password (hashed)
    // cid: clipboard id (i64, but > 0)
    // cpw: clipboard password (hashed) (may be NULL)
    // mime: use this as the content-type
    let sqls = [
        strip_str! {"CREATE TABLE paste_user (
            uid BLOB PRIMARY KEY,
            upw BLOB
        )"},
        strip_str! {"CREATE TABLE paste_data (
            cid INTEGER PRIMARY KEY AUTOINCREMENT,
            cpw BLOB,
            uid BLOB,
            mime BLOB
        )"},
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

const CID_CHARS: [u8; 36] = *b"0123456789abcdefghijklmnopqrstuvwxyz";
fn int2str(i: i64) -> Vec<u8> {
    Default::default()
}
fn str2int(s: Vec<u8>) -> i64 {
    0
}

pub fn service() -> Router {
    db_init();
    // mentions about the path later?
    Router::new()
}
