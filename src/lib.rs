pub mod mccs;

use std::io::{Error, ErrorKind};

#[cfg(feature = "node-bindings")]
mod neon_bindings;

#[cfg(feature = "node-bindings")]
use neon_bindings::{display_get_brightness, display_set_brightness, display_info};
#[cfg(feature = "node-bindings")]
use neon::prelude::*;

use ddc::{Ddc, DdcHost, FeatureCode, VcpValue};
pub use ddc_hi::Display;

pub fn get_enhanced_displays() -> Result<Vec<Display>, Error> {
    Display::enumerate().into_iter().map(|mut display|
        display.update_capabilities()
            .map(|_| display)
            .map_err(|err| Error::new(ErrorKind::TimedOut, err.to_string()))
    ).collect()
}

pub fn get_enhanced_display(id: usize) -> Result<Display, Error> {
    get_enhanced_displays()?
        .into_iter()
        .nth(id)
        .ok_or(Error::new(ErrorKind::Unsupported, format!("There is no display with id: {}", id)))
}

pub fn get_brightness(id: usize) -> Result<VcpValue, Error> {
    let mut display = get_enhanced_display(id)?;
    let result = display.info.mccs_database
        .get(mccs::ImageAdjustment::Luminance as FeatureCode)
        .map(|feature| display.handle.get_vcp_feature(feature.code)
            .map_err(|error| Error::new(ErrorKind::TimedOut, error.to_string())))
        .or(Some(Err(Error::new(ErrorKind::Unsupported, "This display doesn't support brightness operations")))).unwrap();
    display.handle.sleep();
    result
}

pub fn set_brightness(id: usize, value: u16) -> Result<(), Error> {
    let mut display = get_enhanced_display(id)?;
    let result = display.info.mccs_database
        .get(mccs::ImageAdjustment::Luminance as FeatureCode)
        .map(|feature| display.handle.set_vcp_feature(feature.code, value)
            .map_err(|error| Error::new(ErrorKind::TimedOut, error.to_string())))
        .or(Some(Err(Error::new(ErrorKind::Unsupported, "This display doesn't support brightness operations")))).unwrap();
    display.handle.sleep();
    result
}

#[cfg(feature = "node-bindings")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("display_info", display_info)?;
    cx.export_function("display_get_brightness", display_get_brightness)?;
    cx.export_function("display_set_brightness", display_set_brightness)?;
    Ok(())
}