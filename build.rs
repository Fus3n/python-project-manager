#[cfg(windows)]
extern crate winres;

#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("ppm-icon.ico");
    res.compile().unwrap();
}
