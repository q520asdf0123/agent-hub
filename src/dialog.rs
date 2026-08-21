//! Windows 原生「选择文件夹」对话框（IFileOpenDialog 裸 COM 调用，零第三方依赖）。
//! 必须在独立线程调用：内部以 STA 初始化 COM，模态阻塞直到用户选择或取消。

#![cfg(windows)]

use std::ffi::c_void;
use std::ptr;

type Hresult = i32;

#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

// shobjidl_core.h: CLSID_FileOpenDialog / IID_IFileOpenDialog
const CLSID_FILE_OPEN_DIALOG: Guid = Guid {
    data1: 0xdc1c5a9c,
    data2: 0xe88a,
    data3: 0x4dde,
    data4: [0xa5, 0xa1, 0x60, 0xf8, 0x2a, 0x20, 0xae, 0xf7],
};
const IID_IFILE_OPEN_DIALOG: Guid = Guid {
    data1: 0xd57c7288,
    data2: 0xd4ad,
    data3: 0x4768,
    data4: [0xbe, 0x02, 0x9d, 0x96, 0x95, 0x32, 0xd9, 0x60],
};

#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> Hresult;
    fn CoUninitialize();
    fn CoCreateInstance(
        clsid: *const Guid,
        outer: *mut c_void,
        ctx: u32,
        iid: *const Guid,
        out: *mut *mut c_void,
    ) -> Hresult;
    fn CoTaskMemFree(p: *mut c_void);
}

// 仅声明用到的槽位，顺序必须与 COM 接口定义一致（IFileOpenDialog : IFileDialog
// : IModalWindow : IUnknown），未用方法以 usize 占位；结构体是完整 vtable 的前缀。
#[repr(C)]
struct DialogVtbl {
    _query_interface: usize,
    _add_ref: usize,
    release: unsafe extern "system" fn(*mut Dialog) -> u32,
    show: unsafe extern "system" fn(*mut Dialog, *mut c_void) -> Hresult,
    _set_file_types: usize,
    _set_file_type_index: usize,
    _get_file_type_index: usize,
    _advise: usize,
    _unadvise: usize,
    set_options: unsafe extern "system" fn(*mut Dialog, u32) -> Hresult,
    get_options: unsafe extern "system" fn(*mut Dialog, *mut u32) -> Hresult,
    _set_default_folder: usize,
    _set_folder: usize,
    _get_folder: usize,
    _get_current_selection: usize,
    _set_file_name: usize,
    _get_file_name: usize,
    set_title: unsafe extern "system" fn(*mut Dialog, *const u16) -> Hresult,
    _set_ok_button_label: usize,
    _set_file_name_label: usize,
    get_result: unsafe extern "system" fn(*mut Dialog, *mut *mut Item) -> Hresult,
}

#[repr(C)]
struct Dialog {
    vtbl: *const DialogVtbl,
}

// IShellItem vtable 前缀。
#[repr(C)]
struct ItemVtbl {
    _query_interface: usize,
    _add_ref: usize,
    release: unsafe extern "system" fn(*mut Item) -> u32,
    _bind_to_handler: usize,
    _get_parent: usize,
    get_display_name: unsafe extern "system" fn(*mut Item, u32, *mut *mut u16) -> Hresult,
}

#[repr(C)]
struct Item {
    vtbl: *const ItemVtbl,
}

const COINIT_APARTMENTTHREADED: u32 = 0x2;
const CLSCTX_INPROC_SERVER: u32 = 0x1;
const FOS_PICKFOLDERS: u32 = 0x20;
const FOS_FORCEFILESYSTEM: u32 = 0x40;
const SIGDN_FILESYSPATH: u32 = 0x8005_8000;
const HR_CANCELLED: Hresult = 0x8007_04c7_u32 as i32; // HRESULT_FROM_WIN32(ERROR_CANCELLED)

/// 弹出系统「选择文件夹」对话框；返回所选目录绝对路径，用户取消返回 Ok(None)。
pub fn pick_folder(title: &str) -> Result<Option<String>, String> {
    let title_w: Vec<u16> = title.encode_utf16().chain([0]).collect();
    unsafe {
        let hr = CoInitializeEx(ptr::null_mut(), COINIT_APARTMENTTHREADED);
        if hr < 0 {
            return Err(format!("COM 初始化失败: 0x{:08x}", hr as u32));
        }
        let result = show_dialog(&title_w);
        CoUninitialize();
        result
    }
}

unsafe fn show_dialog(title_w: &[u16]) -> Result<Option<String>, String> {
    let mut raw: *mut c_void = ptr::null_mut();
    let hr = CoCreateInstance(
        &CLSID_FILE_OPEN_DIALOG,
        ptr::null_mut(),
        CLSCTX_INPROC_SERVER,
        &IID_IFILE_OPEN_DIALOG,
        &mut raw,
    );
    if hr < 0 || raw.is_null() {
        return Err(format!("创建对话框失败: 0x{:08x}", hr as u32));
    }
    let dlg = raw as *mut Dialog;
    let v = &*(*dlg).vtbl;
    let mut opts = 0u32;
    (v.get_options)(dlg, &mut opts);
    (v.set_options)(dlg, opts | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM);
    (v.set_title)(dlg, title_w.as_ptr());
    let hr = (v.show)(dlg, ptr::null_mut());
    if hr == HR_CANCELLED {
        (v.release)(dlg);
        return Ok(None);
    }
    if hr < 0 {
        (v.release)(dlg);
        return Err(format!("对话框显示失败: 0x{:08x}", hr as u32));
    }
    let mut item: *mut Item = ptr::null_mut();
    let hr = (v.get_result)(dlg, &mut item);
    if hr < 0 || item.is_null() {
        (v.release)(dlg);
        return Err(format!("读取选择结果失败: 0x{:08x}", hr as u32));
    }
    let iv = &*(*item).vtbl;
    let mut pw: *mut u16 = ptr::null_mut();
    let hr = (iv.get_display_name)(item, SIGDN_FILESYSPATH, &mut pw);
    let out = if hr >= 0 && !pw.is_null() {
        let len = (0usize..).take_while(|&i| *pw.add(i) != 0).count();
        let path = String::from_utf16_lossy(std::slice::from_raw_parts(pw, len));
        CoTaskMemFree(pw as *mut c_void);
        Ok(Some(path))
    } else {
        Err(format!("获取路径失败: 0x{:08x}", hr as u32))
    };
    (iv.release)(item);
    (v.release)(dlg);
    out
}
