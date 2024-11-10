/*++ @file

    Copyright ©2024-2024 Liu Yi, efikarl@yeah.net

    This program is just made available under the terms and conditions of the
    MIT license: http://www.efikarl.com/mit-license.html

    THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
    WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

use crate::file::extend::*;
use crate::tftp::packet::*;

pub struct Client { server_sa: std::net::SocketAddr, client_us: std::net::UdpSocket }

impl Client {
    pub fn new<A: std::net::ToSocketAddrs>(server: A) -> Self {
        let server_sa = server.to_socket_addrs().unwrap().next().unwrap();
        let client_us = std::net::UdpSocket::bind(("0.0.0.0",0)).unwrap();

        Client { server_sa, client_us }
    }

    pub fn send<S: AsRef<std::path::Path>, D: AsRef<std::path::Path>>(&self, src: S, dst: D) {
        let mut svr = self.server_sa.clone();
        // init port:TFTP_PORT
        svr.set_port(TFTP_PORT);
        // send wrq
        let wrq = Packet::newwrq(&dst, TFTP_MODE).encode();
        self.client_us.send_to(&wrq, svr).unwrap();
        // recv ack
        let mut ack = [0u8;TFTP_SIZE_PACKET_MAX];
        (_, svr) = self.client_us.recv_from(&mut ack).unwrap();
        assert_eq!(
            0,
            u16::from_be_bytes([ack[2], ack[3]])
        );
        // send dat
        let mut buf;
        let mut dat = std::fs::read(&src).unwrap();
        let mut len = dat.len();
        let mut blk = 1;
        loop {
            if len <= 0 {
                break;
            }
            if len <= TFTP_SIZE_DATA_BLOCK {
                self.client_us.send_to(&Packet::newdat(blk, dat).encode(), svr).unwrap();
                self.client_us.recv_from(&mut ack).unwrap();
                assert_eq!(
                    blk,
                    u16::from_be_bytes([ack[2], ack[3]])
                );
                break;
            } else {
                buf = dat.split_off(TFTP_SIZE_DATA_BLOCK);
                self.client_us.send_to(&Packet::newdat(blk, dat).encode(), svr).unwrap();
                (_, svr) = self.client_us.recv_from(&mut ack).unwrap();
                assert_eq!(
                    blk,
                    u16::from_be_bytes([ack[2], ack[3]])
                );
                dat = buf;

                blk = blk + 1;
                len = len - TFTP_SIZE_DATA_BLOCK;
            }
        }
    }

    pub fn recv<S: AsRef<std::path::Path>, D: AsRef<std::path::Path>>(&self, src: S, dst: D) {
        let mut svr = self.server_sa.clone();
        // init port:TFTP_PORT
        svr.set_port(TFTP_PORT);
        let mut amt;
        // send rrq
        let mut rrq = Packet::newrrq(&src, TFTP_MODE).encode();
        self.client_us.send_to(&mut rrq, svr).unwrap();
        // recv dat
        let mut buf = vec![];
        let mut dat = [0u8;TFTP_SIZE_PACKET_MAX];
        let mut blk = 1;
        loop {
            (amt, svr) = self.client_us.recv_from(&mut dat).unwrap();
            assert_eq!(
                blk,
                u16::from_be_bytes([dat[2], dat[3]])
            );
            self.client_us.send_to(&Packet::newack(blk).encode(), svr).unwrap();
            buf.extend_from_slice(&dat[4..amt]);
            if amt < TFTP_SIZE_PACKET_MAX {
                break;
            }
            blk = blk + 1;
        }
        let file = dst.try_create_parent(true).unwrap();
        std::fs::write(&file, buf).unwrap();
    }
}
