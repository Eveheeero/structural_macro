use windows::Win32::{
    System::SystemServices::MK_LBUTTON,
    UI::WindowsAndMessaging::{
        CallNextHookEx, FindWindowExW, SendMessageW, SetWindowsHookExW, UnhookWindowsHookEx,
        WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
    },
};

pub struct Hook();

impl Hook {
    fn new() -> Self {
        Self {}
    }
}
impl Drop for Hook {
    fn drop(&mut self) {}
}
