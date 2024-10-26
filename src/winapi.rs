use std::rc::Rc;
use windows::{
    core::{PCSTR, PCWSTR},
    Win32::{
        Foundation::{HANDLE, HMODULE, HWND, LPARAM, WPARAM},
        System::{
            Diagnostics::{
                Debug::WriteProcessMemory,
                ToolHelp::{
                    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                    TH32CS_SNAPPROCESS,
                },
            },
            LibraryLoader::{GetModuleHandleA, GetProcAddress, LoadLibraryW},
            Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE},
            ProcessStatus::EnumProcessModules,
            SystemServices::MK_LBUTTON,
            Threading::{
                CreateRemoteThread, OpenProcess, WaitForSingleObject, INFINITE,
                LPTHREAD_START_ROUTINE, PROCESS_ALL_ACCESS,
            },
        },
        UI::WindowsAndMessaging::{
            FindWindowW, SendMessageW, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
        },
    },
};

pub struct WindowProcess {
    pub pid: u32,
    pub name: String,
    pub moudle_id: u32,
}
pub unsafe fn enum_processes() -> Result<Vec<WindowProcess>, windows::core::Error> {
    let hsnapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut result = Vec::new();
    let mut lppe: PROCESSENTRY32W = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    Process32FirstW(hsnapshot, &mut lppe)?;
    loop {
        result.push(to_process(&lppe));
        let ok = Process32NextW(hsnapshot, &mut lppe);
        if ok.is_err() {
            break;
        }
    }
    Ok(result)
}
fn to_process(pe: &PROCESSENTRY32W) -> WindowProcess {
    WindowProcess {
        pid: pe.th32ProcessID,
        name: decode_utf16_with_zero(&pe.szExeFile),
        moudle_id: pe.th32ModuleID,
    }
}
pub struct WindowProcessModule {
    pub process: Rc<HANDLE>,
    pub module: HMODULE,
}
pub unsafe fn enum_process_modules(
    pid: u32,
) -> Result<Vec<WindowProcessModule>, windows::core::Error> {
    let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
    let mut cb = 0;
    let mut cbneeded = 0;
    EnumProcessModules(process, std::ptr::null_mut(), cb, &mut cbneeded)?;
    let mut result: Vec<_> = std::iter::repeat(HMODULE::default())
        .take(
            (cbneeded / std::mem::size_of::<HMODULE>() as u32)
                .try_into()
                .unwrap(),
        )
        .collect();
    cb = result.len().try_into().unwrap();
    EnumProcessModules(process, result.as_mut_ptr(), cb, &mut cbneeded)?;
    let process = Rc::new(process);
    Ok(result
        .into_iter()
        .map(|module| WindowProcessModule {
            process: Rc::clone(&process),
            module,
        })
        .collect())
}
pub unsafe fn inject_dll(pid: u32, dll: &str) -> Result<(), windows::core::Error> {
    let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid)?;
    let data: Vec<u8> = encode_ascii_with_zero(dll);
    let dll: PCSTR = PCSTR(data.as_ptr());
    let written_address = VirtualAllocEx(process, None, data.len(), MEM_COMMIT, PAGE_READWRITE);
    WriteProcessMemory(
        process,
        written_address,
        dll.0 as *const core::ffi::c_void,
        data.len(),
        None,
    )?;
    let loading = CreateRemoteThread(
        process,
        None,
        0,
        std::mem::transmute::<_, LPTHREAD_START_ROUTINE>(GetProcAddress(
            GetModuleHandleA(windows::core::s!("kernel32.dll"))?,
            windows::core::s!("LoadLibraryA"),
        )),
        Some(written_address),
        0,
        None,
    )?;
    WaitForSingleObject(loading, INFINITE);
    VirtualFreeEx(process, written_address, 0, MEM_RELEASE)?;

    Ok(())
}

pub unsafe fn find_window(name: &str) -> Option<HWND> {
    let wstring: Vec<u16> = encode_utf16_with_zero(name);
    let Ok(result) = FindWindowW(None, PCWSTR(wstring.as_ptr())) else {
        return None;
    };
    Some(result)
}

pub unsafe fn click(window_handle: HWND, position: (isize, isize)) {
    let lparam = LPARAM(position.0 | position.1 << 16);
    SendMessageW(window_handle, WM_MOUSEMOVE, None, lparam);
    SendMessageW(
        window_handle,
        WM_LBUTTONDOWN,
        WPARAM(MK_LBUTTON.0.try_into().unwrap()),
        lparam,
    );
    SendMessageW(window_handle, WM_LBUTTONUP, None, lparam);
}

pub unsafe fn load_library(path: &str) -> Result<HMODULE, windows::core::Error> {
    let path = encode_utf16_with_zero(path);
    LoadLibraryW(PCWSTR(path.as_ptr()))
}

#[inline]
fn encode_utf16_with_zero(s: &str) -> Vec<u16> {
    widestring::encode_utf16(s.chars().chain(std::iter::once('\0'))).collect()
}
#[inline]
fn decode_utf16_with_zero(s: &[u16]) -> String {
    String::from_utf16_lossy(s)
        .trim_end_matches(char::from(0))
        .to_string()
}
#[inline]
fn encode_ascii_with_zero(s: &str) -> Vec<u8> {
    s.as_bytes()
        .iter()
        .chain(std::iter::once(&0))
        .cloned()
        .collect()
}
#[inline]
fn decode_ascii_with_zero(s: &[u8]) -> String {
    String::from_utf8_lossy(s)
        .trim_end_matches(char::from(0))
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enum_process() -> Result<(), windows::core::Error> {
        let processes = unsafe { enum_processes()? };
        assert!(!processes.is_empty());
        Ok(())
    }

    #[test]
    fn test_enum_process_modules() -> Result<(), windows::core::Error> {
        let processes = unsafe { enum_processes()? };
        assert!(!processes.is_empty());
        let process = processes
            .iter()
            .find(|p| p.name.ends_with("explorer.exe"))
            .unwrap();
        let modules = unsafe { enum_process_modules(process.pid)? };
        assert!(!modules.is_empty());
        Ok(())
    }

    #[test]
    fn test_window_click() -> Result<(), windows::core::Error> {
        let window_handle = unsafe { find_window("Structural Macro").unwrap() };
        assert_ne!(window_handle, HWND::default());
        unsafe { click(window_handle, (5, 5)) };
        Ok(())
    }

    #[test]
    fn test_inject_dll() -> Result<(), windows::core::Error> {
        let processes = unsafe { enum_processes()? };
        assert!(!processes.is_empty());
        let process = processes
            .iter()
            .find(|p| p.name.ends_with("structural_macro.exe"))
            .unwrap();
        let dll = r#"structural_macro_dll.dll"#;
        unsafe { inject_dll(process.pid, dll)? };
        Ok(())
    }
}
