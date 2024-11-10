/*++ @file

    Copyright ©2024-2024 Liu Yi, efikarl@yeah.net

    This program is just made available under the terms and conditions of the
    MIT license: http://www.efikarl.com/mit-license.html

    THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
    WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use network::prelude::*;

fn main() {
    let client = Client::new("127.0.0.1:0");

    let mut text = String::new();
    text.push_str(&"1".repeat(512));
    text.push('\n');
    text.push_str(&"2".repeat(512));
    text.push('\n');

    let target = "target/client/tftp.0.log".try_create_parent(true).unwrap();
    let mut file = std::fs::OpenOptions::new().create(true).read(true).append(true).open(&target).unwrap();
    file.write(text.as_bytes()).unwrap();

    client.send("target/client/tftp.0.log", "target/server/tftp.x.log");
    // std::thread::sleep(std::time::Duration::from_secs(1));
    client.recv("target/server/tftp.x.log", "target/client/tftp.1.log");

    let send = std::fs::read("target/client/tftp.0.log").unwrap();
    let text = std::fs::read("target/server/tftp.x.log").unwrap();
    let recv = std::fs::read("target/client/tftp.1.log").unwrap();

    assert_eq!(send, text);
    assert_eq!(text, recv);
}
