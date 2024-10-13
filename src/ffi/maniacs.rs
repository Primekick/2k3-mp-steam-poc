use std::{mem, ptr};
use std::ffi::c_void;

use crate::ffi::{get_module_base};
use crate::ffi_call;

pub const VARIABLE_OFFSET_BASE: isize = 0x001645d4;
pub const STRING_OFFSET_BASE: isize = 0x001645E0;
pub const CREATE_EVENT_OFF: isize = 0x00059940;
pub const TLIST_ADD_OFF: isize = 0x00061e20;
pub const EVENT_CATALOG_OFF: isize = 0x001603c4;
pub const EVENT_CATALOG_OFF_DUP: isize = 0x00157104;

#[repr(C)]
struct EventData {
    id: i32,
    name: *mut c_void,
    x: i32,
    y: i32,
    pages: *mut c_void,
}

pub fn set_strvar(id: i32, value: &str) {
    let len = value.len();

    let mut area = vec![0u8; len + 9usize].into_boxed_slice();
    let mut area_ptr = area.as_mut_ptr();
    mem::forget(area);

    unsafe {
        ptr::write(area_ptr as *mut usize, 2usize);
        area_ptr = area_ptr.add(4usize);
        ptr::write(area_ptr as *mut usize, len);
        area_ptr = area_ptr.add(4usize);

        let strvar_base = get_module_base().byte_offset(STRING_OFFSET_BASE);
        let strvar_ptr: usize = ptr::read(strvar_base) + 4 * id as usize;

        let cstr = std::ffi::CString::new(value).unwrap();
        for (idx, byte) in cstr.as_bytes_with_nul().iter().enumerate() {
            ptr::write(area_ptr.add(idx), *byte);
        }

        ptr::write(strvar_ptr as *mut *mut u8, area_ptr);
    }
}

pub fn set_var(id: i32, value: i32) {
    unsafe {
        let var_base = get_module_base().byte_offset(VARIABLE_OFFSET_BASE);
        let vars_ptr: usize = ptr::read(var_base) + 4 * id as usize;
        ptr::write(vars_ptr as *mut i32, value);
    }
}

pub fn spawn_event_by_id(id: i32) -> *mut usize {
    unsafe {
        let off = get_module_base().byte_offset(CREATE_EVENT_OFF);
        let spawn = ffi_call!(off, extern "fastcall" fn(id: i32) -> *mut usize);
        spawn(id)
    }
}

pub fn add_event_to_map(id: i32) {
    unsafe {
        let ev_catalog = get_module_base().byte_offset(EVENT_CATALOG_OFF);

        // get named catalog ptr
        let ev_catalog = ptr::read(ev_catalog) as *const usize;

        // get pointer to actual list
        let ev_list_ptr = ptr::read(ev_catalog) as *const usize;
        println!("ev tlist ptr {:?}", ev_list_ptr);

        // TList.Count at +0x4
        let ev_count = ptr::read(ev_list_ptr.byte_add(0x4));
        println!("ev count {:?}", ev_count);

        // spawn from current map
        let event_ptr = spawn_event_by_id(id);
        println!("new ev ptr {:?}", event_ptr);

        // change id: in main event and in eventdata
        ptr::write(event_ptr.byte_add(0x4), ev_count + 1);
        let ev_data_ptr = ptr::read(event_ptr.byte_add(0x79)) as *mut EventData;
        let new_ev_data = Box::new(EventData {
            id: (ev_count + 1) as i32,
            name: (*ev_data_ptr).name.clone(),
            x: (*ev_data_ptr).x,
            y: (*ev_data_ptr).y,
            pages: (*ev_data_ptr).pages.clone(),
        });
        let new_ev_data_ptr = &*new_ev_data as *const EventData;
        mem::forget(new_ev_data);
        ptr::write(event_ptr.byte_add(0x79), new_ev_data_ptr as usize);

        // add the new EventData to map eventdata storage
        let map_evdata_catalog = get_module_base().byte_offset(0x00163cd0);
        let map_evdata_catalog = ptr::read(map_evdata_catalog) as *const usize;
        let map_evdata_list_ptr = ptr::read(map_evdata_catalog) as *const usize;

        let tlist_add_ptr = get_module_base().byte_offset(TLIST_ADD_OFF);
        println!("tlist add ptr {:?}", tlist_add_ptr);
        let tlist_add = ffi_call!(
            tlist_add_ptr,
            extern "fastcall" fn(list_ptr: *const usize, event_ptr: *mut usize)
        );

        tlist_add(ev_list_ptr, event_ptr);
        tlist_add(map_evdata_list_ptr, new_ev_data_ptr as *mut usize);
    }
}