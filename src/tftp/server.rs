/*++ @file

    Copyright ©2024-2024 Liu Yi, efikarl@yeah.net

    This program is just made available under the terms and conditions of the
    MIT license: http://www.efikarl.com/mit-license.html

    THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
    WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use crate::file::extend::*;
use crate::tftp::packet::*;

pub struct Server(bool);

impl Server {
    pub fn new() -> Self {
        let  info = if let Some(_) = std::env::var_os("TFTP_INFO") { true } else { false };

        Self(info)
    }

    pub fn listen(&self) {
        let svr = std::net::UdpSocket::bind(("0.0.0.0", TFTP_PORT)).unwrap();
        loop {
            let mut raw = [0u8;TFTP_SIZE_PACKET_MAX];
            let rst = svr.recv_from(&mut raw);
            if let Err(_) = rst {
                continue;
            }
            let (amt, clt) = rst.unwrap();
            let pkt = Packet::decode(&raw[0..amt].to_vec(), amt);
            match pkt {
                Packet::Rrq(file, mode) => {
                    if self.0 {
                        println!("Rrq(I): file({}) mode({})", file, mode);
                    }
                    self.send(file, clt).unwrap_or_default();
                },
                Packet::Wrq(file, mode) => {
                    if self.0 {
                        println!("Wrq(I): file({}) mode({})", file, mode);
                    }
                    self.recv(file, clt).unwrap_or_default();
                },
                _ => {
                    continue;
                }
            }
        }
    }

    fn send(&self, file: String, clt: std::net::SocketAddr) -> Result<(), std::io::Error> {
        let svr = std::net::UdpSocket::bind(("0.0.0.0", TFTP_TID0))?;
        svr.set_write_timeout(Some(std::time::Duration::new(TFTP_TIMEOUT, 0)))?;svr.set_read_timeout(Some(std::time::Duration::new(TFTP_TIMEOUT, 0)))?;
        let mut clt = clt;
        // send dat
        let mut buf;
        let mut dat = std::fs::read(&file)?;
        let mut len = dat.len();
        let mut blk = 1;
        let mut ack = [0u8;TFTP_SIZE_PACKET_MAX];
        let mut klb;
        loop {
            if len <= 0 {
                break;
            }
            if len <= TFTP_SIZE_DATA_BLOCK {
                svr.send_to(&Packet::newdat(blk, dat).encode(), clt)?;
                if self.0 {
                    println!("Dat(O): blk# = {}", blk);
                }
                (_  ,  _) = svr.recv_from(&mut ack)?;
                klb = u16::from_be_bytes([ack[2], ack[3]]);
                if self.0 {
                    println!("Ack(I): blk# = {}", klb);
                }
                if blk != klb {
                    svr.send_to(&Packet::newerr(0, "EOR($): blk != klb").encode(), clt)?;
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "EOR($): blk != klb"));
                }
                break;
            } else {
                buf = dat.split_off(TFTP_SIZE_DATA_BLOCK);
                svr.send_to(&Packet::newdat(blk, dat).encode(), clt)?;
                if self.0 {
                    println!("Dat(O): blk# = {}", blk);
                }
                (_  ,clt) = svr.recv_from(&mut ack)?;
                klb = u16::from_be_bytes([ack[2], ack[3]]);
                if self.0 {
                    println!("Ack(I): blk# = {}", klb);
                }
                if blk != klb {
                    svr.send_to(&Packet::newerr(0, "EOR($): blk != klb").encode(), clt)?;
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "EOR($): blk != klb"));
                }
                dat = buf;

                blk = blk + 1;
                len = len - TFTP_SIZE_DATA_BLOCK;
            }
        }
        if self.0 {
            println!("EOR($): size = {}", (blk as usize - 1) * TFTP_SIZE_DATA_BLOCK + len);
        }
        Ok(())
    }

    fn recv(&self, file: String, clt: std::net::SocketAddr) -> Result<(), std::io::Error>  {
        let svr = std::net::UdpSocket::bind(("0.0.0.0", TFTP_TID0))?;
        svr.set_write_timeout(Some(std::time::Duration::new(TFTP_TIMEOUT, 0)))?;svr.set_read_timeout(Some(std::time::Duration::new(TFTP_TIMEOUT, 0)))?;
        let mut clt = clt;
        let mut amt;
        // send ack
        svr.send_to(&Packet::newack(0).encode(), clt)?;
        if self.0 {
            println!("Ack(O): blk# = 0");
        }
        // recv dat
        let mut buf = vec![];
        let mut dat = [0u8;TFTP_SIZE_PACKET_MAX];
        let mut blk = 1;
        let mut klb;
        loop {
            (amt, clt) = svr.recv_from(&mut dat)?;
            klb = u16::from_be_bytes([dat[2], dat[3]]);
            if self.0 {
                println!("Dat(I): blk# = {}", klb);
            }
            if blk != klb {
                svr.send_to(&Packet::newerr(0, "EOR($): blk != klb").encode(), clt)?;
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "EOR($): blk != klb"));
            }
            svr.send_to(&Packet::newack(blk).encode(), clt)?;
            if self.0 {
                println!("Ack(O): blk# = {}", blk);
            }
            buf.extend_from_slice(&dat[4..amt]);
            if amt < TFTP_SIZE_PACKET_MAX {
                if self.0 {
                    amt = amt - 4;
                }
                break;
            }
            blk = blk + 1;
        }
        let file = file.try_create_parent(true)?;
        std::fs::write(&file, buf)?;
        if self.0 {
            println!("EOR($): size = {}", (blk as usize - 1) * TFTP_SIZE_DATA_BLOCK + amt);
        }
        Ok(())
    }
}
