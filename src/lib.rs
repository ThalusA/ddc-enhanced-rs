pub mod mccs;

use std::io::{Error, ErrorKind};

#[cfg(feature = "node-bindings")]
mod neon_bindings;

#[cfg(feature = "node-bindings")]
use neon_bindings::{display_get_brightness, display_set_brightness, displays_info};
#[cfg(feature = "node-bindings")]
use neon::prelude::*;

use ddc::{Ddc, DdcHost, FeatureCode, VcpValue};
use ddc_hi::Backend;
pub use ddc_hi::Display;

pub fn get_displays(needs_caps: bool) -> Result<Vec<Display>, Error> {
    let mut displays = Display::enumerate().into_iter().map(|mut display|
        if needs_caps && display.info.backend == Backend::WinApi {
            display.update_capabilities()
                .map(|_| display)
                .map_err(|err| Error::new(ErrorKind::TimedOut, err.to_string()))
        } else {
            Ok(display)
        }
    );
    if displays.find(|display| if let &Ok(ref display) = display {
        (&display.info).backend == Backend::Nvapi
    } else {
        false
    }).is_some() {
        let displays = displays.filter(|display|
            if let &Ok(ref display) = display {
                (&display.info).backend != Backend::WinApi
            } else {
                true
            }
        );
        displays.collect()
    } else {
        displays.collect()
    }
}

pub fn get_display(needs_caps: bool, id: usize) -> Result<Display, Error> {
    get_displays(needs_caps)?
        .into_iter()
        .nth(id)
        .ok_or(Error::new(ErrorKind::Unsupported,
                          format!("There is no display with id: {}", id)))
}

pub fn get_brightness(id: usize) -> Result<VcpValue, Error> {
    let mut display = get_display(true, id)?;
    let result = display.info.mccs_database
        .get(mccs::ImageAdjustment::Luminance as FeatureCode)
        .map(|feature| display.handle.get_vcp_feature(feature.code)
            .map_err(|error| Error::new(ErrorKind::TimedOut, error.to_string())))
        .or(Some(Err(Error::new(ErrorKind::Unsupported,
                                "This display doesn't support brightness operations")))).unwrap();
    display.handle.sleep();
    result
}

pub fn set_brightness(id: usize, value: u16) -> Result<(), Error> {
    let mut display = get_display(true, id)?;
    let result = display.info.mccs_database
        .get(mccs::ImageAdjustment::Luminance as FeatureCode)
        .map(|feature| display.handle.set_vcp_feature(feature.code, value)
            .map_err(|error| Error::new(ErrorKind::TimedOut, error.to_string())))
        .or(Some(Err(Error::new(ErrorKind::Unsupported,
                                "This display doesn't support brightness operations")))).unwrap();
    display.handle.sleep();
    result
}

#[cfg(feature = "node-bindings")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("displays_info", displays_info)?;
    cx.export_function("display_get_brightness", display_get_brightness)?;
    cx.export_function("display_set_brightness", display_set_brightness)?;
    Ok(())
}