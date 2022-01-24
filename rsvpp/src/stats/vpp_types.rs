use std::ffi::CStr;

use libc::{c_char, c_void};

#[repr(C)]
#[derive(Debug)]
pub enum StatDirectoryType {
    StatDirTypeIllegal = 0,
    StatDirTypeScalarIndex = 1,
    StatDirTypeCounterVectorSimple = 2,
    StatDirTypeCounterVectorCombined = 3,
    StatDirTypeErrorIndex = 4,
    StatDirTypeNameVector = 5,
    StatDirTypeEmpty = 6,
    StatDirTypeSymlink = 7,
}

#[repr(C)]
pub union StatDirectoryUnion {
    pub index: u64,
    pub value: u64,
    pub data: usize, // void *
}

impl std::fmt::Debug for StatDirectoryUnion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StatDirectoryUnion {{ v: {} }}", unsafe { self.data })
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct StatSegmentDirectoryEntry {
    pub r#type: StatDirectoryType,
    pub u: StatDirectoryUnion,
    pub name: [u8; 128],
}

impl StatSegmentDirectoryEntry {
    pub fn name_starts_with(&self, prefix: &str) -> bool {
        let src = &self.name as *const u8;
        let cmp = prefix.as_ptr();
        let cmp_len = prefix.len();

        (unsafe { libc::memcmp(src as *const c_void, cmp as *const c_void, cmp_len) }) == 0
    }

    pub fn name_to_str(&self) -> &str {
        let c_str = unsafe { CStr::from_ptr(&self.name as *const u8 as *const c_char) };
        c_str.to_str().unwrap_or("Utf8 error")
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct StatSegmentSharedHeader {
    pub version: u64,
    pub base: usize, // void *
    pub epoch: u64,
    pub in_progress: u64,
    pub directory_vector: usize, // stat_segment_directory_entry_t *
}

impl StatSegmentSharedHeader {
    pub fn segments(&self) -> impl Iterator<Item = &'static StatSegmentDirectoryEntry> {
        let vec_addr = self.adjust_ptr(self.directory_vector as *const c_void)
            as *const StatSegmentDirectoryEntry;
        StatSegmentDirectoryIter::new(vec_addr)
    }

    pub fn adjust_ptr(&self, ptr: *const c_void) -> *const c_void {
        let header_addr = self as *const Self as usize;
        let ptr = ptr as usize;
        if ptr > self.base {
            (header_addr + (ptr - self.base)) as *const c_void
        } else {
            0 as *const c_void
        }
    }
}

pub struct StatSegmentDirectoryIter {
    ptr: *const StatSegmentDirectoryEntry,
    current_index: usize,
    length: usize,
}

impl StatSegmentDirectoryIter {
    pub fn new(ptr: *const StatSegmentDirectoryEntry) -> Self {
        Self {
            ptr,
            current_index: 0,
            length: unsafe { super::util::vec_len(ptr as *const c_void) as usize },
        }
    }
}

impl Iterator for StatSegmentDirectoryIter {
    type Item = &'static StatSegmentDirectoryEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.length {
            None
        } else {
            let entry = unsafe { &*(self.ptr.offset(self.current_index as isize)) };
            self.current_index += 1;

            Some(entry)
        }
    }
}
