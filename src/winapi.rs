use std::rc::Rc;

use windows::Win32::{
    Foundation::{HANDLE, HMODULE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        },
        ProcessStatus::EnumProcessModules,
        Threading::{OpenProcess, PROCESS_ALL_ACCESS},
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
