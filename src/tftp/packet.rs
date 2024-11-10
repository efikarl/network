/*++ @file

    Copyright ©2024-2024 Liu Yi, efikarl@yeah.net

    This program is just made available under the terms and conditions of the
    MIT license: http://www.efikarl.com/mit-license.html

    THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
    WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

pub const TFTP_TIMEOUT          :   u64 =                       12;
pub const TFTP_PORT             :   u16 =                   0x0045;
pub const TFTP_TID0             :   u16 =                   0x0000;
pub const TFTP_MODE             : & str =                  "octet";
pub const TFTP_SIZE_DATA_BLOCK  : usize =                   0x0200;
pub const TFTP_SIZE_PACKET_MAX  : usize = TFTP_SIZE_DATA_BLOCK + 4;

///////////////////////////////////////////////////////////////////////////////
use crate::file::extend::*;
///////////////////////////////////////////////////////////////////////////////

trait TftpTypeIntoRaw<T> {
    fn into_raw(&self) -> Vec<u8>;
    fn from_raw(i: &[u8]) -> Self;
}

impl TftpTypeIntoRaw<String> for String {
    fn into_raw(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
    fn from_raw(i: &[u8]) -> Self {
        String::from_utf8_lossy(i).to_string()
    }
}

impl TftpTypeIntoRaw<u16> for u16 {
    fn into_raw(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
    fn from_raw(i: &[u8]) -> Self {
        u16::from_be_bytes(i.try_into().unwrap())
    }
}

///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u16)]
pub enum OpCode {
    Rrq = 0x01,
    Wrq = 0x02,
    Dat = 0x03,
    Ack = 0x04,
    Err = 0x05,
}

impl From<&[u8]> for OpCode {
    fn from(i: &[u8]) -> Self {
        assert!(i.len() == 2);
        let o = u16::from_be_bytes([i[0], i[1]]);
        assert!((o >=1) && (o <= 5));
        unsafe {
            std::mem::transmute::<u16, OpCode>(o)
        }
    }
}

impl From<&OpCode> for Vec::<u8> {
    fn from(i: &OpCode) -> Self {
        let o = unsafe {
            std::mem::transmute::<OpCode, u16>(*i)
        };
        Vec::from(o.to_be_bytes())
    }
}

///////////////////////////////////////////////////////////////////////////////

type PacketFileName   = String;
type PacketOpMode     = String;
type PacketBlockId    = u16;
type PacketData       = Vec<u8>;
type PacketErrCode    = u16;
type PacketErrMsgs    = String;

#[derive(Debug, PartialEq)]
pub enum Packet {
    Rrq (PacketFileName, PacketOpMode ),
    Wrq (PacketFileName, PacketOpMode ),
    Dat (PacketBlockId , PacketData   ),
    Ack (PacketBlockId                ),
    Err (PacketErrCode , PacketErrMsgs),
}

impl Packet {
    pub fn opcode(&self) -> OpCode {
        match self {
            Packet::Rrq (..)  => OpCode::Rrq,
            Packet::Wrq (..)  => OpCode::Wrq,
            Packet::Dat (..)  => OpCode::Dat,
            Packet::Ack (..)  => OpCode::Ack,
            Packet::Err (..)  => OpCode::Err,
        }
    }

    pub fn newrrq<F: PathEx, M: ToString>(file: F, mode: M) -> Packet {
        if mode.to_string().to_lowercase() != "octet" {
            panic!("N/A");
        }
        Packet::Rrq(file.to_string(), mode.to_string())
    }
    pub fn newwrq<F: PathEx, M: ToString>(file: F, mode: M) -> Packet {
        if mode.to_string().to_lowercase() != "octet" {
            panic!("N/A");
        }
        Packet::Wrq(file.to_string(), mode.to_string())
    }
    pub fn newdat(blkid: u16, data: PacketData) -> Packet {
        Packet::Dat(blkid, data)
    }
    pub fn newack(blkid: u16) -> Packet {
        Packet::Ack(blkid)
    }
    pub fn newerr<T: ToString>(code: u16, msgs: T) -> Packet {
        Packet::Err(code, msgs.to_string())
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();

        v.append(&mut Vec::<u8>::from(&self.opcode()));
        match self {
            Packet::Rrq(file, mode) | Packet::Wrq(file, mode) => {
                v.extend_from_slice(&file.as_bytes());
                v.push(0);
                v.extend_from_slice(&mode.as_bytes());
                v.push(0);
            },
            Packet::Dat(blkid, data)  => {
                assert!(data.len() <= TFTP_SIZE_DATA_BLOCK);
                v.extend_from_slice(&blkid.into_raw());
                v.extend_from_slice(&data);
            },
            Packet::Ack(blkid)  => {
                v.extend_from_slice(&blkid.into_raw());
            },
            Packet::Err (code, msgs)  => {
                v.extend_from_slice(&code.into_raw());
                v.extend_from_slice(&msgs.as_bytes());
                v.push(0);
            },
        }
        v
    }

    pub fn decode(raw: &Vec<u8>, len: usize) -> Self {
        assert!((len == raw.len()) && (len >= std::mem::size_of::<OpCode>() + 2) && (len <= TFTP_SIZE_PACKET_MAX));

        let opcode = OpCode::from(&raw[0..2]);
        match opcode {
            OpCode::Rrq | OpCode::Wrq => {
                let s = 1 + 1;
                let e = raw[s..].iter().position(|&p| p == 0).unwrap();
                let e = s + e;
                let file = String::from_utf8_lossy(&raw[s..e]).to_string();
                let s = e + 1;
                let e = raw[s..].iter().position(|&p| p == 0).unwrap();
                let e = s + e;
                let mode = String::from_utf8_lossy(&raw[s..e]).to_string();
                if opcode == OpCode::Rrq {
                    return Packet::Rrq(file, mode);
                } else {
                    return Packet::Wrq(file, mode);
                }
            },
            OpCode::Dat => {
                let blkid = u16::from_raw(&raw[2..4]);
                let data = raw[4..len].to_vec();
                return Packet::Dat(blkid, data);
            },
            OpCode::Ack => {
                let blkid = u16::from_raw(&raw[2..4]);
                return Packet::Ack(blkid);
            },
            OpCode::Err => {
                let code = u16::from_raw(&raw[2..4]);
                let s = 2 + 2;
                let e = raw[s..].iter().position(|&p| p == 0).unwrap();
                let e = s + e;
                let msgs = String::from_utf8_lossy(&raw[s..e]).to_string();
                return Packet::Err(code, msgs);
            },
        }
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn test_encode() {
    let packet1 = Packet::newrrq("azAZ09-0.txt", "octet");
    let v0: Vec<u8> = vec![0, 1,
        'a' as u8, 'z' as u8, 'A' as u8, 'Z' as u8, '0' as u8, '9' as u8, '-' as u8, '0' as u8, '.' as u8, 't' as u8, 'x' as u8, 't' as u8, 0,
        'o' as u8, 'c' as u8, 't' as u8, 'e' as u8, 't' as u8, 0 ];
        let v1 = packet1.encode();
    assert_eq!(v0, v1);
    let packet2 = Packet::newwrq("azAZ09-0.txt", "OCTET");
    let v0: Vec<u8> = vec![0, 2,
        'a' as u8, 'z' as u8, 'A' as u8, 'Z' as u8, '0' as u8, '9' as u8, '-' as u8, '0' as u8, '.' as u8, 't' as u8, 'x' as u8, 't' as u8, 0,
        'O' as u8, 'C' as u8, 'T' as u8, 'E' as u8, 'T' as u8, 0 ];
    let v2 = packet2.encode();
    assert_eq!(v0, v2);
    let packet3 = Packet::newdat(0x10, vec!['e' as u8, 'f' as u8, 'i' as u8, '\r' as u8, 0, 'k' as u8, 'a' as u8, '\n' as u8 ]);
    let v0: Vec<u8> = vec![0, 3, 0, 0x10,
        'e' as u8, 'f' as u8, 'i' as u8, '\r' as u8, 0, 'k' as u8, 'a' as u8, '\n' as u8 ];
    let v3 = packet3.encode();
    assert_eq!(v0, v3);
    let packet4 = Packet::newack(0x10);
    let v0: Vec<u8> = vec![0, 4, 0, 0x10 ];
    let v4 = packet4.encode();
    assert_eq!(v0, v4);
    let packet5 = Packet::newerr(1, &String::from("error"));
    let v0: Vec<u8> = vec![0, 5, 0, 0x01,
        'e' as u8, 'r' as u8, 'r' as u8, 'o' as u8, 'r' as u8, 0 ];
    let v5 = packet5.encode();
    assert_eq!(v0, v5);
}

#[test]
fn test_decode() {
    let packet1 = Packet::newrrq("azAZ09-0.txt", "octet");
    let v0: Vec<u8> = vec![0, 1,
        'a' as u8, 'z' as u8, 'A' as u8, 'Z' as u8, '0' as u8, '9' as u8, '-' as u8, '0' as u8, '.' as u8, 't' as u8, 'x' as u8, 't' as u8, 0,
        'o' as u8, 'c' as u8, 't' as u8, 'e' as u8, 't' as u8, 0 ];
        let packet0 = Packet::decode(&v0, v0.len());
    assert_eq!(packet0, packet1);
    let packet2 = Packet::newwrq("azAZ09-0.txt", "OCTET");
    let v0: Vec<u8> = vec![0, 2,
        'a' as u8, 'z' as u8, 'A' as u8, 'Z' as u8, '0' as u8, '9' as u8, '-' as u8, '0' as u8, '.' as u8, 't' as u8, 'x' as u8, 't' as u8, 0,
        'O' as u8, 'C' as u8, 'T' as u8, 'E' as u8, 'T' as u8, 0 ];
        let packet0 = Packet::decode(&v0, v0.len());
    assert_eq!(packet0, packet2);
    let packet3 = Packet::newdat(0x10, vec!['e' as u8, 'f' as u8, 'i' as u8, '\r' as u8, 0, 'k' as u8, 'a' as u8, '\n' as u8 ]);
    let v0: Vec<u8> = vec![0, 3, 0, 0x10,
        'e' as u8, 'f' as u8, 'i' as u8, '\r' as u8, 0, 'k' as u8, 'a' as u8, '\n' as u8 ];
    let packet0 = Packet::decode(&v0, v0.len());
    assert_eq!(packet0, packet3);
    let packet4 = Packet::newack(0x10);
    let v0: Vec<u8> = vec![0, 4, 0, 0x10 ];
    let packet0 = Packet::decode(&v0, v0.len());
    assert_eq!(packet0, packet4);
    let packet5 = Packet::newerr(2, &String::from("error"));
    let v0: Vec<u8> = vec![0, 5, 0, 0x02,
        'e' as u8, 'r' as u8, 'r' as u8, 'o' as u8, 'r' as u8, 0 ];
    let packet0 = Packet::decode(&v0, v0.len());
    assert_eq!(packet0, packet5);
}
