pub mod discovery;

use log::debug;
use simple_error::SimpleError;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;
use std::string::ToString;

#[derive(clap::ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Source {
    Wifi,
    Bluetooth,
    Aux,
    Opt,
    Usb,
}

impl Source {
    pub fn to_bytes(&self) -> Vec<u8> {
        match *self {
            Self::Wifi => [0x53, 0x30, 0x81, 0x12, 0x82].to_vec(),
            Self::Bluetooth => [0x53, 0x30, 0x81, 0x19, 0xad].to_vec(),
            Self::Aux => [0x53, 0x30, 0x81, 0x1a, 0x9b].to_vec(),
            Self::Opt => [0x53, 0x30, 0x81, 0x1b, 0x00].to_vec(),
            Self::Usb => [0x53, 0x30, 0x81, 0x1c, 0xf7].to_vec(),
        }
    }

    pub fn bitmask(&self, input: u8) -> u8 {
        (0b11110000 & input)
            | match *self {
                Self::Wifi => 0b0010,
                Self::Usb => 0b1100,
                Self::Bluetooth => 0b1001,
                Self::Aux => 0b1010,
                Self::Opt => 0b1011,
            }
    }
    pub fn from_mask(mask: u8) -> Self {
        match mask & 0b1111 {
            0b0010 => Self::Wifi,
            0b1100 => Self::Usb,
            0b1001 => Self::Bluetooth,
            0b1010 => Self::Aux,
            0b1011 => Self::Opt,
            _ => Self::Aux,
        }
    }
}

impl FromStr for Source {
    type Err = simple_error::SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "wifi" => Ok(Self::Wifi),
            "bluetooth" => Ok(Self::Bluetooth),
            "aux" => Ok(Self::Aux),
            "opt" => Ok(Self::Opt),
            "usb" => Ok(Self::Usb),
            _ => Err(SimpleError::new("invalid source type")),
        }
    }
}

impl ToString for Source {
    fn to_string(&self) -> String {
        match *self {
            Self::Wifi => "wifi".to_string(),
            Self::Bluetooth => "bluetooth".to_string(),
            Self::Aux => "aux".to_string(),
            Self::Opt => "opt".to_string(),
            Self::Usb => "usb".to_string(),
        }
    }
}

#[derive(clap::ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Standby {
    S0,
    S20,
    S60,
}

impl Standby {
    pub fn bitmask(&self, input: u8) -> u8 {
        (0b11001111 & input)
            | match *self {
                Self::S0 => 0b100000,
                Self::S20 => 0b000000,
                Self::S60 => 0b010000,
            }
    }
    pub fn from_mask(input: u8) -> Self {
        match 0b00110000 & input {
            0b100000 => Self::S0,
            0b000000 => Self::S20,
            0b010000 => Self::S60,
            _ => Self::S60,
        }
    }
}

#[derive(clap::ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Power {
    On,
    Off,
}

impl Power {
    pub fn bitmask(&self, input: u8) -> u8 {
        (0b01111111 & input)
            | match *self {
                Self::On => 0b00000000,
                Self::Off => 0b10000000,
            }
    }
    pub fn from_mask(input: u8) -> Self {
        match 0b10000000 & input {
            0b00000000 => Self::On,
            0b10000000 => Self::Off,
            _ => Self::Off,
        }
    }
}

#[derive(clap::ValueEnum, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Inverse {
    Right,
    Left,
}
impl Inverse {
    pub fn bitmask(&self, input: u8) -> u8 {
        (0b10111111 & input)
            | match *self {
                Self::Right => 0b0000000,
                Self::Left => 0b1000000,
            }
    }
    pub fn from_mask(input: u8) -> Self {
        match 0b01000000 & input {
            0b0000000 => Self::Right,
            0b1000000 => Self::Left,
            _ => Self::Right,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Volume {
    level: u8,
}

impl Volume {
    pub fn new(val: u8) -> Volume {
        let mut val = val;
        if val > 100 {
            val = 100
        }
        return Volume { level: val };
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return [0x53, 0x25, 0x81, self.level, 0x1a].to_vec();
    }
}

impl FromStr for Volume {
    type Err = SimpleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u8>() {
            Ok(val) => Ok(Volume::new(val)),
            Err(_) => Err(SimpleError::new("failed to parse volume")),
        }
    }
}

pub enum Command {
    GetSource,
    SetSource(Power, Inverse, Standby, Source),
    SetVolume(Volume),
    TurnOff,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CommandResult {
    Completed,
    GotSource(Power, Inverse, Standby, Source),
}

pub fn bitmask_to_source_config(bits: u8) -> (Power, Inverse, Standby, Source) {
    let source = Source::from_mask(bits);
    let standby = Standby::from_mask(bits);
    let inverse = Inverse::from_mask(bits);
    let power = Power::from_mask(bits);
    (power, inverse, standby, source)
}

impl Command {
    pub fn to_bytes(&self) -> Vec<u8> {
        match *self {
            Self::GetSource => [0x47, 0x30, 0x80].to_vec(),
            Self::SetSource(power, inverse, standby, source) => {
                let bitmask = power.bitmask(0x0);
                let bitmask = inverse.bitmask(bitmask);
                let bitmask = standby.bitmask(bitmask);
                let bitmask = source.bitmask(bitmask);
                [0x53, 0x30, 0x81, bitmask, 0x9b].to_vec() //src.to_bytes(),
            }
            Self::SetVolume(vol) => vol.to_bytes(),
            Self::TurnOff => [0x53, 0x30, 0x81, 0x9B, 0x0B].to_vec(),
        }
    }

    pub fn execute(&self, sa: &SocketAddr) -> io::Result<CommandResult> {
        let mut conn = TcpStream::connect(sa)?;
        let wbuf = self.to_bytes();
        debug!("write to speaker: {:#X?}", wbuf);
        conn.write(wbuf.as_slice())?;
        let mut rbuf: [u8; 8] = [0; 8];
        conn.read(rbuf.as_mut_slice())?;
        debug!("read from speaker: {:#X?}", rbuf);

        match *self {
            Self::GetSource => {
                let bits = rbuf[3];
                let (power, inverse, standby, source) = bitmask_to_source_config(bits);
                io::Result::Ok(CommandResult::GotSource(power, inverse, standby, source))
            }
            _ => io::Result::Ok(CommandResult::Completed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_limit() {
        let vol = Volume::new(128);
        assert!(vol.level == 100);
    }
    #[test]
    fn source_string_conv() {
        assert_eq!(
            Source::Wifi,
            Source::from_str(Source::Wifi.to_string().as_str()).unwrap()
        );
        assert_eq!(
            Source::Bluetooth,
            Source::from_str(Source::Bluetooth.to_string().as_str()).unwrap()
        );
        assert_eq!(
            Source::Aux,
            Source::from_str(Source::Aux.to_string().as_str()).unwrap()
        );
        assert_eq!(
            Source::Opt,
            Source::from_str(Source::Opt.to_string().as_str()).unwrap()
        );
        assert_eq!(
            Source::Usb,
            Source::from_str(Source::Usb.to_string().as_str()).unwrap()
        );
    }
}
