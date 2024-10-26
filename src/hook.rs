use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Controls::WM_MOUSELEAVE,
        WindowsAndMessaging::{
            CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, MSG, WH_GETMESSAGE,
        },
    },
};

unsafe extern "system" fn hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code < 0 {
        return CallNextHookEx(HHOOK::default(), code, wparam, lparam);
    }
    let msg = std::mem::transmute::<_, &MSG>(lparam);
    println!("{}, {}", code, msg.message);
    if msg.message != WM_MOUSELEAVE {
        return CallNextHookEx(HHOOK::default(), code, wparam, lparam);
    }
    return CallNextHookEx(HHOOK::default(), -1, wparam, lparam);
}

pub struct Hook(HHOOK);

impl Hook {
    pub unsafe fn new() -> Self {
        let dll = crate::winapi::load_library("structural_macro_dll.dll").expect("Cannot find dll");
        let hook = SetWindowsHookExW(WH_GETMESSAGE, Some(hook), dll, 0).expect("Failed to hook");
        Self(hook)
    }
}
impl Drop for Hook {
    fn drop(&mut self) {
        unsafe { UnhookWindowsHookEx(self.0) }.expect("Failed to unhook");
    }
}
