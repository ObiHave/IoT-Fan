use super::*;

pub struct CommandMsg {
    pub cmd_id: CommandID,
    pub param_id: ControlMode,
    pub param_value: u16,
}
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum CommandID {
    setControlMode = 0,
    percentSpeedMode = 1,
    temperatureSpeedMode = 2,
    invalidCommand,
}
impl From<u16> for CommandID {
    fn from(cmdid: u16) -> CommandID {
        match cmdid {
            0 => CommandID::setControlMode,
            1 => CommandID::percentSpeedMode,
            2 => CommandID::temperatureSpeedMode,
            _ => CommandID::invalidCommand,
        }
    }
}
#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum ControlMode {
    sleep = 0,
    tracking = 1,
    oscillate = 2,
    point = 3,
    night = 4,
    invalidParameter,
}
impl From<u16> for ControlMode {
    fn from(param: u16) -> ControlMode {
        match param {
            0 => ControlMode::sleep,
            1 => ControlMode::tracking,
            2 => ControlMode::oscillate,
            3 => ControlMode::point,
            4 => ControlMode::night,
            _ => ControlMode::invalidParameter,
        }
    }
}


named!(
    pub parse_message(&[u8]) -> CommandMsg,
    do_parse!(
        // Take command ID
        cmd_id: map_res!(take_until!(","), from_slice) >>
        // Take comma.
        take!(1) >>
        // Take parameter ID.
        control_mode: map_res!(take_until!(","), from_slice) >>
        // Take the comma.
        take!(1) >>
        // Take the parameter value.
        param_val: map_res!(take_until!("."), from_slice) >>
        (CommandMsg {
            cmd_id: CommandID::from(cmd_id),
            param_id: ControlMode::from(control_mode),
            param_value: param_val,
        })
    )
);
fn from_slice(input: &[u8]) -> Result<u16, Error> {
    u16::from_str_radix(std::str::from_utf8(input)?, 10).map_err(|_| FanError::ParseError.into())
}