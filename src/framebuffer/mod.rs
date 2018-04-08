pub mod common;
pub mod mxcfb;
pub mod screeninfo;


pub mod io;
pub trait FramebufferIO {
    /// Writes an arbitrary length frame into the framebuffer
    fn write_frame(&mut self, frame: &[u8]);
    /// Writes a single pixel at `(y, x)` with value `v`
    fn write_pixel(&mut self, y: usize, x: usize, v: u8);
    /// Reads the value of the pixel at `(y, x)`
    fn read_pixel(&mut self, y: usize, x: usize) -> u8;
    /// Reads the value at offset `ofst` from the mmapp'ed framebuffer region
    fn read_offset(&mut self, ofst: isize) -> u8;
}

use image;
pub mod draw;
pub trait FramebufferDraw {
    /// Draws `img` at y=top, x=left coordinates with 1:1 scaling
    fn draw_image(&mut self, img: &image::DynamicImage, top: usize, left: usize) -> common::mxcfb_rect;
    /// Draws a straight line
    fn draw_line(&mut self, y0: i32, x0: i32, y1: i32, x1: i32, width: usize, color: u8) -> common::mxcfb_rect;
    /// Draws a circle using Bresenham circle algorithm
    fn draw_circle(&mut self, y: usize, x: usize, rad: usize, color: u8) -> common::mxcfb_rect;
    /// Fills a circle
    fn fill_circle(&mut self, y: usize, x: usize, rad: usize, color: u8) -> common::mxcfb_rect;
    /// Draws a bezier curve begining at `startpt`, with control point `ctrlpt`, ending at `endpt` with `color`
    fn draw_bezier(&mut self, startpt: (f32, f32), ctrlpt: (f32, f32), endpt: (f32, f32), color: u8) -> common::mxcfb_rect;
    /// Draws `text` at `(y, x)` with `color` using `scale`
    fn draw_text(
        &mut self,
        y: usize,
        x: usize,
        text: String,
        size: usize,
        color: u8,
    ) -> common::mxcfb_rect;
    /// Fills rectangle of `height` and `width` at `(y, x)`
    fn fill_rect(&mut self, y: usize, x: usize, height: usize, width: usize, color: u8);
    /// Clears the framebuffer however does not perform a refresh
    fn clear(&mut self);
}

use std;
pub mod core;
pub trait FramebufferBase<'a> {
    /// Creates a new instance of Framebuffer
    fn new(path_to_device: &str) -> core::Framebuffer;
    /// Toggles the EPD Controller (see https://wiki.mobileread.com/wiki/EPD_controller)
    fn set_epdc_access(&mut self, state: bool);
    /// Toggles autoupdate mode
    fn set_autoupdate_mode(&mut self, mode: u32);
    /// Toggles update scheme
    fn set_update_scheme(&mut self, scheme: u32);
    /// Creates a FixScreeninfo struct and fills it using ioctl
    fn get_fix_screeninfo(device: &std::fs::File) -> screeninfo::FixScreeninfo;
    /// Creates a VarScreeninfo struct and fills it using ioctl
    fn get_var_screeninfo(device: &std::fs::File) -> screeninfo::VarScreeninfo;
    /// Makes the proper ioctl call to set the VarScreenInfo.
    /// You must first update the contents of self.var_screen_info
    /// and then call this function.
    fn put_var_screeninfo(&mut self) -> bool;
}


pub mod refresh;
pub trait FramebufferRefresh {
    /// Refreshes the entire screen with the provided parameters. If `wait_completion` is
    /// set to true, doesn't return before the refresh has been completed. Returns the marker.
    fn full_refresh(&mut self,
                    waveform_mode: common::waveform_mode,
                    temperature: common::display_temp,
                    dither_mode: common::dither_mode,
                    quant_bit: i32,
                    wait_completion: bool) -> u32;

    /// Refreshes the given `region` with the provided parameters. If `mode` is `DryRun` or
    /// `Wait`, this function won't return before the `DryRun`'s collision_test or
    /// refresh has been completed. In `Async` mode, this function will return immediately
    /// and return a `marker` which can then later be fed to `wait_refresh_complete` to wait
    /// for its completion. In `DryRun`, it will return the `collision_test` result.
    ///
    /// Some additional points to note:
    ///
    ///    1) PxP must process 8x8 pixel blocks, and all pixels in each block
    ///    are considered for auto-waveform mode selection. If the
    ///    update region is not 8x8 aligned, additional unwanted pixels
    ///    will be considered in auto-waveform mode selection.
    ///
    ///    2) PxP input must be 32-bit aligned, so any update
    ///    address not 32-bit aligned must be shifted to meet the
    ///    32-bit alignment.  The PxP will thus end up processing pixels
    ///    outside of the update region to satisfy this alignment restriction,
    ///    which can affect auto-waveform mode selection.
    ///
    ///    3) If input fails 32-bit alignment, and the resulting expansion
    ///    of the processed region would add at least 8 pixels more per
    ///    line than the original update line width, the EPDC would
    ///    cause screen artifacts by incorrectly handling the 8+ pixels
    ///    at the end of each line.
    fn partial_refresh(
        &mut self,
        region: &common::mxcfb_rect,
        mode: refresh::PartialRefreshMode,
        waveform_mode: common::waveform_mode,
        temperature: common::display_temp,
        dither_mode: common::dither_mode,
        quant_bit: i32,
    ) -> u32;


    /// Takes a marker returned by `partial_refresh` and blocks until that
    /// refresh has been reflected on the display.
    /// Returns the collusion_test result which is supposed to be
    /// related to the collusion information.
    fn wait_refresh_complete(&mut self, marker: u32) -> u32;
}