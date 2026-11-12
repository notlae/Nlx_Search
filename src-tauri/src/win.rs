//! Small Win32 helpers declared by hand (no crate-version drift).
use serde::Serialize;
use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::process::Command;

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[link(name = "kernel32")]
extern "system" {
    fn GetLogicalDrives() -> u32;
    fn GetDriveTypeW(root: *const u16) -> u32;
    fn GetDiskFreeSpaceExW(dir: *const u16, avail: *mut u64, total: *mut u64, free: *mut u64) -> i32;
}

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, flags: u32) -> i32;
}

#[link(name = "user32")]
extern "system" {
    fn PeekMessageW(msg: *mut [u64; 6], hwnd: isize, min: u32, max: u32, remove: u32) -> i32;
    fn TranslateMessage(msg: *const [u64; 6]) -> i32;
    fn DispatchMessageW(msg: *const [u64; 6]) -> isize;
}

#[repr(C)]
struct ShellExecuteInfoW {
    cb_size: u32,
    mask: u32,
    hwnd: isize,
    verb: *const u16,
    file: *const u16,
    parameters: *const u16,
    directory: *const u16,
    show: i32,
    inst_app: isize,
    id_list: *mut c_void,
    class: *const u16,
    hkey_class: isize,
    hot_key: u32,
    icon_or_monitor: isize,
    process: isize,
}

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteExW(info: *mut ShellExecuteInfoW) -> i32;
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Drive {
    pub letter: String,
    pub free: u64,
    pub total: u64,
}

/// Roots of fixed (non-removable, non-network) drives, e.g. "C:\\".
pub fn fixed_drives() -> Vec<String> {
    let mask = unsafe { GetLogicalDrives() };
    (0u32..26)
        .filter(|i| mask & (1u32 << i) != 0)
        .map(|i| format!("{}:\\", (b'A' + i as u8) as char))
        .filter(|root| {
            let w = wide(root);
            unsafe { GetDriveTypeW(w.as_ptr()) == 3 } // DRIVE_FIXED
        })
        .collect()
}

pub fn drive_info() -> Vec<Drive> {
    fixed_drives()
        .into_iter()
        .map(|root| {
            let w = wide(&root);
            let (mut avail, mut total, mut free) = (0u64, 0u64, 0u64);
            unsafe { GetDiskFreeSpaceExW(w.as_ptr(), &mut avail, &mut total, &mut free) };
            Drive { letter: root[..2].to_string(), free: avail, total }
        })
        .collect()
}

/// Hidden-window command builder.
pub fn hidden(program: &str) -> Command {
    let mut c = Command::new(program);
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

/// Opens the Explorer "Properties" dialog for a path (best effort).
pub fn show_properties(path: String) {
    std::thread::spawn(move || unsafe {
        CoInitializeEx(std::ptr::null_mut(), 2); // COINIT_APARTMENTTHREADED
        let verb = wide("properties");
        let file = wide(&path);
        let mut info: ShellExecuteInfoW = std::mem::zeroed();
        info.cb_size = std::mem::size_of::<ShellExecuteInfoW>() as u32;
        info.mask = 0x0000_000C | 0x0000_0100; // SEE_MASK_INVOKEIDLIST | SEE_MASK_NOASYNC
        info.verb = verb.as_ptr();
        info.file = file.as_ptr();
        info.show = 5; // SW_SHOW
        if ShellExecuteExW(&mut info) == 0 {
            return;
        }
        // keep this thread alive (and pumping) so the dialog is not torn down
        let mut msg = [0u64; 6];
        for _ in 0..600 {
            while PeekMessageW(&mut msg, 0, 0, 0, 1) != 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    });
}
