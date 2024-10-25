use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
        System::{
            LibraryLoader::LoadLibraryW,
            ProcessStatus::EnumProcesses,
            SystemServices::MK_LBUTTON,
            Threading::{GetCurrentThreadId, OpenProcess},
        },
        UI::{
            Controls::WM_MOUSELEAVE,
            WindowsAndMessaging::{
                CallNextHookEx, FindWindowExW, FindWindowW, SendMessageW, SetWindowsHookExW,
                UnhookWindowsHookEx, HHOOK, MSG, WH_GETMESSAGE, WM_LBUTTONDOWN, WM_LBUTTONUP,
                WM_MOUSEMOVE,
            },
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
        let processes = crate::winapi::enum_processes().unwrap();
        let process = processes.iter().find(|p| p.name == "explorer.exe").unwrap();
        let module = crate::winapi::enum_process_modules(process.pid).unwrap();
        let hook = SetWindowsHookExW(WH_GETMESSAGE, Some(hook), module[0].module, 0)
            .expect("Failed to hook");
        Self(hook)
    }
}
impl Drop for Hook {
    fn drop(&mut self) {
        unsafe { UnhookWindowsHookEx(self.0) }.expect("Failed to unhook");
    }
}
