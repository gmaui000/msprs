use std::ptr;

use ffmpeg_sys_next as ffmpeg;

use super::utils::ffmpeg_error;

// AVPacket wrapper, automatically released when the lifetime ends
#[derive(Debug)]
pub struct FFmpegPacket {
    // Raw pointer to the AVPacket
    pub ptr: *mut ffmpeg::AVPacket,
}

unsafe impl Send for FFmpegPacket {}

impl FFmpegPacket {
    // Wrapper for av_packet_alloc, creates an AVPacket
    pub fn new() -> Result<Self, &'static str> {
        let ptr = unsafe { ffmpeg::av_packet_alloc() };
        if ptr.is_null() {
            Err("Error in av_packet_alloc")
        } else {
            Ok(FFmpegPacket { ptr })
        }
    }

    /// Allocates memory for the AVPacket, then copies the specified amount of data.
    ///
    /// # Safety
    ///
    pub unsafe fn set(&mut self, data: *mut u8, nbytes: usize) {
        (*self.ptr).data = ffmpeg::av_malloc(nbytes) as *mut u8;
        std::ptr::copy_nonoverlapping(data, (*self.ptr).data, nbytes);
        (*self.ptr).stream_index = 0;
        (*self.ptr).size = nbytes as i32;
    }

    // Gets the raw pointer to the AVPacket
    pub fn as_mut_ptr(&self) -> *mut ffmpeg::AVPacket {
        self.ptr
    }

    // Gets the stream index
    pub fn stream_index(&mut self) -> i32 {
        let packet_ptr = self.as_mut_ptr();
        if !packet_ptr.is_null() {
            return unsafe { (*packet_ptr).stream_index };
        }

        -1
    }

    // Gets the frame size
    pub fn get_size(&mut self) -> i32 {
        let packet_ptr = self.as_mut_ptr();
        if !packet_ptr.is_null() {
            return unsafe { (*packet_ptr).size };
        }

        0
    }
}

impl Drop for FFmpegPacket {
    // Automatically calls av_packet_free when the lifetime ends
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ffmpeg::av_packet_free(&mut self.ptr);
                self.ptr = ptr::null_mut();
            }
        }
    }
}

// AVFrame wrapper, automatically released when the lifetime ends
#[derive(Debug)]
pub struct FFmpegFrame {
    // Raw pointer to the AVFrame
    pub ptr: *mut ffmpeg::AVFrame,
}

unsafe impl Send for FFmpegFrame {}

impl FFmpegFrame {
    // Wrapper for av_frame_alloc, creates an AVFrame
    pub fn new() -> Result<Self, &'static str> {
        let ptr = unsafe { ffmpeg::av_frame_alloc() };
        if ptr.is_null() {
            Err("Error in av_frame_alloc")
        } else {
            Ok(FFmpegFrame { ptr })
        }
    }

    // Replaces the raw pointer to the AVFrame (unref the previous pointer)
    pub fn set(&mut self, frame: *mut ffmpeg::AVFrame) {
        unsafe {
            ffmpeg::av_frame_unref(self.ptr);
            self.ptr = frame;
        }
    }

    // Gets the raw pointer to the AVFrame
    pub fn as_mut_ptr(&self) -> *mut ffmpeg::AVFrame {
        self.ptr
    }

    // Makes the AVFrame pointer writable
    pub fn make_writable(&mut self) -> Result<(), String> {
        let retval: i32 = unsafe { ffmpeg::av_frame_make_writable(self.ptr) };
        if retval < 0 {
            return Err(ffmpeg_error("av_frame_make_writable", retval));
        }
        Ok(())
    }

    // Gets the data at the specified index of the AVFrame.data
    pub fn plane(&mut self, index: usize, len: usize) -> &mut [u8] {
        unsafe {
            let data_ptr = (*self.ptr).data[index];
            std::slice::from_raw_parts_mut(data_ptr, len)
        }
    }

    // Gets the width and height
    pub fn get_resolution(&mut self) -> (i32, i32) {
        let frame = self.as_mut_ptr();
        if !frame.is_null() {
            unsafe {
                return ((*frame).width, (*frame).height);
            }
        }

        (0, 0)
    }
}

impl Drop for FFmpegFrame {
    // Automatically calls av_frame_free when the lifetime ends
    fn drop(&mut self) {
        unsafe {
            if !self.ptr.is_null() {
                ffmpeg::av_frame_free(&mut self.ptr);
                self.ptr = ptr::null_mut();
            }
        }
    }
}

#[derive(Debug)]
pub struct FrameBox {
    pub frame: FFmpegFrame,
}

unsafe impl Send for FrameBox {}
