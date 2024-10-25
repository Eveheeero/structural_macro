use std::rc::Rc;
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{HANDLE, HMODULE},
        System::{
            Diagnostics::{
                Debug::WriteProcessMemory,
                ToolHelp::{
                    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                    TH32CS_SNAPPROCESS,
                },
            },
            LibraryLoader::{GetModuleHandleA, GetProcAddress},
            Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE},
            ProcessStatus::EnumProcessModules,
            Threading::{
                CreateRemoteThread, OpenProcess, WaitForSingleObject, INFINITE,
                LPTHREAD_START_ROUTINE, PROCESS_ALL_ACCESS,
            },
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
        name: String::from_utf16(&pe.szExeFile)
            .unwrap()
            .trim_end_matches(char::from(0))
            .to_string(),
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
    let mut data: Vec<u8> = dll.into();
    data.push(b'\0');
    let dll: PCSTR = PCSTR(data.as_ptr());
    let written_address = VirtualAllocEx(
        process,
        None,
        std::mem::size_of_val(&dll),
        MEM_COMMIT,
        PAGE_READWRITE,
    );
    WriteProcessMemory(
        process,
        written_address,
        dll.0 as *const core::ffi::c_void,
        std::mem::size_of_val(&dll),
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
    VirtualFreeEx(
        process,
        written_address,
        std::mem::size_of_val(&dll),
        MEM_RELEASE,
    )?;

    Ok(())
}
