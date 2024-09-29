#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "glulx")]
extern "C" {
    pub fn glkarea_get_byte(glkaddr: u32) -> u32;
    pub fn glkarea_get_word(glkaddr: u32) -> u32;
    pub fn glkarea_put_byte(glkaddr: u32, byte: u32);
    pub fn glkarea_put_word(glkaddr: u32, word: u32);
    pub fn glkarea_get_bytes(addr: *mut u8, glkaddr: u32, n: u32);
    pub fn glkarea_get_words(addr: *mut u32, glkaddr: u32, n: u32);
    pub fn glkarea_put_bytes(glkaddr: u32, addr: *const u8, n: u32);
    pub fn glkarea_put_words(glkaddr: u32, addr: *const u32, n: u32);

    pub fn fmodf(x: f32, y: f32) -> f32;
    pub fn floorf(x: f32) -> f32;
    pub fn ceilf(x: f32) -> f32;
    pub fn expf(x: f32) -> f32;
    pub fn logf(x: f32) -> f32;
    pub fn powf(x: f32, y: f32) -> f32;
    pub fn sinf(x: f32) -> f32;
    pub fn cosf(x: f32) -> f32;
    pub fn tanf(x: f32) -> f32;
    pub fn asinf(x: f32) -> f32;
    pub fn acosf(x: f32) -> f32;
    pub fn atanf(x: f32) -> f32;
    pub fn atan2f(y: f32, x: f32) -> f32;

    pub fn fmod(x: f64, y: f64) -> f64;
    pub fn floor(x: f64) -> f64;
    pub fn ceil(x: f64) -> f64;
    pub fn exp(x: f64) -> f64;
    pub fn log(x: f64) -> f64;
    pub fn pow(x: f64, y: f64) -> f64;
    pub fn sin(x: f64) -> f64;
    pub fn cos(x: f64) -> f64;
    pub fn tan(x: f64) -> f64;
    pub fn asin(x: f64) -> f64;
    pub fn acos(x: f64) -> f64;
    pub fn atan(x: f64) -> f64;
    pub fn atan2(y: f64, x: f64) -> f64;

    pub fn restart();
    pub fn save(str: super::glk::StrId) -> i32;
    pub fn restore(str: super::glk::StrId) -> i32;
    pub fn saveundo() -> u32;
    pub fn restoreundo() -> u32;
    pub fn hasundo() -> u32;
    pub fn discardundo();
    pub fn protect(addr: *mut (), len: u32);
}
