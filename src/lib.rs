use simple_error::SimpleError;
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
    SetSource(Source),
    TurnOff,
}

impl Command {
    pub fn to_bytes(&self) -> Vec<u8> {
        match *self {
            Self::SetSource(src) => src.to_bytes(),
            Self::TurnOff => [0x53, 0x30, 0x81, 0x9B, 0x0B].to_vec(),
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
