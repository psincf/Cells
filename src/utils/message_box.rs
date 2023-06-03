use raw_window_handle::HasRawWindowHandle;

#[cfg(target_os = "windows")]
pub fn message_box(window: &winit::window::Window, text: &str) {

    let handle = window.raw_window_handle();
    let handle = match handle {
        raw_window_handle::RawWindowHandle::Windows(win) => { Some(win) }
        _ => { None }
    };

    let c_string = std::ffi::CString::new(text).unwrap();

    unsafe { winapi::um::winuser::MessageBoxA(handle.unwrap().hwnd as *mut _, c_string.as_c_str().as_ptr() as *const _, std::ptr::null(), 0); }
}
#[cfg(not(target_os = "windows"))]
pub fn message_box(window: &winit::window::Window, text: &str) {

}