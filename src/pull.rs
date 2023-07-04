use std::{time::Duration, fs::File, process::Command};
use reqwest::blocking::Client;
use serde_json::Value;

use crate::defs::{SOURCE_XLSXS_DIR, DEFAULT_SOURCE_SUFFIX, MAX_REQ_DELAY};

#[allow(dead_code)]
const ACCOUNT: &'static str = "public";
#[allow(dead_code)]
const PASSWD: &'static str = "5NT38Hb)m3";

pub fn update_svn() {
    let output = Command::new("cmd")
        .arg("/C")
        .arg(format!(r#"{}\update.bat"#, unsafe { SOURCE_XLSXS_DIR }))
        .output()
        .expect("Failed to execute command");

    println!("{}", String::from_utf8_lossy(&output.stdout));
}

pub fn pull_file() -> bool {
    let url = format!("https://server.conchship.com.cn:4433/drive/webapi/auth.cgi?api=SYNO.API.Auth&version=3&method=login&account={}&passwd={}&session=FileStation&format=sid", ACCOUNT, PASSWD);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(MAX_REQ_DELAY))
        .build()
        .unwrap();

    match client.get(url).send() {
        Ok(response) => {
            let v = response.text().unwrap();
            if let Ok(vals) = serde_json::from_str::<Value>(v.as_str()) {
                // login success
                if vals["success"] == true {
                    let sid = &vals["data"]["sid"].as_str();
                    sid.map(|v| {
                        download("LString", "515907215454937090", v, &client);
                    });
                    true
                } else {
                    println!("login failed, and the err code is {}", vals["error"]["code"]);
                    false
                }
            } else {
                false
            }
        }
        Err(e) => {
            println!("request failed: {}", e);
            false
        }
    }
}

fn download(name: &str, pattern: &str, sid: &str, client: &Client) {
    let output_path = format!("{}/{}.{}", unsafe { SOURCE_XLSXS_DIR }, name, DEFAULT_SOURCE_SUFFIX);
    if let Ok(mut file) = File::create(output_path) {
        let url = format!("https://server.conchship.com.cn:4433/drive/webapi/entry.cgi/{}.xlsx?api=SYNO.Office.Export&method=download&version=1&session=FileStation&path=%22id%3A{}%22&_sid={}", name, pattern, sid);
        match client.get(url).send() {
            Ok(mut response) => {
                let _ = response.copy_to(&mut file);
            }
            Err(e) => {
                println!("request failed: {}", e)
            }
        }
    }
}