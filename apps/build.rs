/*
* @file    :   build.rs
* @time    :   2024/04/19 19:18:48
* @author  :   zzjcarrot
*/

fn main() {
    println!("cargo:rustc-link-arg=-T./apps/link.ld");
    println!("build raca_std");
}
