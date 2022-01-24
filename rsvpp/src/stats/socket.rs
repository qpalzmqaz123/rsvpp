use std::{
    ffi::CString,
    mem::{size_of, size_of_val, zeroed},
};

use libc::{c_char, c_int, c_void};

use crate::{Error, Result};

use super::vpp_types::StatSegmentSharedHeader;

pub struct Socket {
    sock: c_int,
}

impl Socket {
    pub fn connect(path: &str) -> Result<Self> {
        // Create socket
        let sock = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, 0) };
        if sock < 0 {
            return Err(Error::Internal("Create socket error".into()));
        }

        // Generate c string
        let sock_c_string = CString::new(path).unwrap();

        // Generate address
        let mut addr = unsafe { zeroed::<libc::sockaddr_un>() };
        addr.sun_family = libc::AF_UNIX as libc::sa_family_t;
        unsafe { libc::strcpy(&mut addr.sun_path as *mut c_char, sock_c_string.as_ptr()) };

        // Connect socket
        let res = unsafe {
            libc::connect(
                sock,
                &addr as *const libc::sockaddr_un as *const libc::sockaddr,
                size_of::<libc::sockaddr_un>() as libc::socklen_t,
            )
        };
        if res < 0 {
            return Err(Error::Internal(format!("Connect '{}' error", path)));
        }

        Ok(Self { sock })
    }

    pub fn get_mmap_header(self) -> Result<&'static StatSegmentSharedHeader> {
        // Get fd
        let fd = self.get_fd()?;

        // Get fd stat
        let mut stat: libc::stat = unsafe { std::mem::zeroed() };
        let res = unsafe { libc::fstat(fd, &mut stat as *mut libc::stat) };
        if res < 0 {
            unsafe { libc::close(fd) };
            return Err(Error::Internal(format!("Get fd {} state error", fd)));
        }

        // Get mmap addr
        let header: &'static StatSegmentSharedHeader = unsafe {
            &*(libc::mmap(
                0 as *mut c_void,
                stat.st_size as libc::size_t,
                libc::PROT_READ,
                libc::MAP_SHARED,
                fd,
                0,
            ) as *const StatSegmentSharedHeader)
        };

        unsafe { libc::close(fd) };

        Ok(header)
    }

    fn get_fd(self) -> Result<c_int> {
        // Generate message
        let mut io_buf = [0u8; 1];
        let mut iov = libc::iovec {
            iov_base: &mut io_buf as *mut u8 as *mut c_void,
            iov_len: 1,
        };
        let mut buf = [0u8; Self::align_size_t(size_of::<c_int>())
            + Self::align_size_t(size_of::<libc::cmsghdr>())];
        let mut msg = unsafe { zeroed::<libc::msghdr>() };
        msg.msg_iov = &mut iov as *mut libc::iovec;
        msg.msg_iovlen = 1;
        msg.msg_control = &mut buf as *mut u8 as *mut c_void;
        msg.msg_controllen = size_of_val(&buf);

        // Receive message
        let size = unsafe { libc::recvmsg(self.sock, &mut msg, 0) };
        if size < 0 {
            return Err(Error::Internal(
                "Receive message from stats socket error".into(),
            ));
        }

        // Decode message
        let cmsg = unsafe { &*(msg.msg_control as *const libc::cmsghdr) };
        let mut fd: c_int = -1;
        if cmsg.cmsg_level == libc::SOL_SOCKET && cmsg.cmsg_type == libc::SCM_RIGHTS {
            unsafe {
                let src_addr = (cmsg as *const libc::cmsghdr as usize + size_of::<libc::cmsghdr>())
                    as *const c_void;
                libc::memmove(
                    &mut fd as *mut c_int as *mut c_void,
                    src_addr,
                    size_of::<c_int>(),
                );
            }
        } else {
            return Err(Error::Internal(
                "Receive message from stats socket error1".into(),
            ));
        }

        Ok(fd)
    }

    const fn align_size_t(len: usize) -> usize {
        (len + size_of::<usize>() - 1) & !(size_of::<usize>() - 1)
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.sock);
        }
    }
}
