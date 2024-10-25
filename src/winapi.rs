use windows::Win32::System::{
    Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    },
    ProcessStatus::EnumProcessModules,
    Threading::{OpenProcess, PROCESS_ALL_ACCESS},
};

pub struct WindowProcess {
    pub pid: u32,
    pub name: String,
    pub moudle_id: u32,
}
pub unsafe fn enum_processes() -> Result<Vec<WindowProcess>, windows::core::Error> {
    let hsnapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?;
    let mut result = Vec::new();
    let mut lppe = std::mem::zeroed();
    Process32FirstW(hsnapshot, &mut lppe)?;
    loop {
        if &lppe as *const PROCESSENTRY32W == std::ptr::null() {
            break;
        }
        result.push(to_process(&lppe));
        Process32NextW(hsnapshot, &mut lppe)?;
    }
    Ok(result)
}
fn to_process(pe: &PROCESSENTRY32W) -> WindowProcess {
    WindowProcess {
        pid: pe.th32ProcessID,
        name: String::from_utf16(&pe.szExeFile).unwrap(),
        moudle_id: pe.th32ModuleID,
    }
}
pub unsafe fn enum_process_modules(pid: u32) {
    let process = OpenProcess(PROCESS_ALL_ACCESS, false, pid).unwrap();
    todo!()
    // EnumProcessModules(process, lphmodule, cb, lpcbneeded)
}
