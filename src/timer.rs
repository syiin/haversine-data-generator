#![feature(std_arch)]

// Imports for macOS specific OS timer (SystemTime)
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

// Imports for Linux specific OS timer (libc)
#[cfg(target_os = "linux")]
use libc::{gettimeofday, timeval}; // Only import what's directly used

pub fn get_os_timer_frequency() -> u64 {
  return 1000000;
}

pub fn read_os_timer() -> u64 {
  #[cfg(target_os = "macos")]
  {
    let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("SystemTime prior to UNIX_EPOCH");
    return duration.as_secs() * get_os_timer_frequency() + (duration.subsec_nanos() / 1000) as u64;
  }

  #[cfg(any(target_os = "linux"))]
  {    
    let mut tv: timeval = unsafe {
      std::mem::zeroed() 
    };
  
    unsafe {
      gettimeofday(&mut tv, std::ptr::null_mut());
    }
    return (tv.tv_sec as u64) * get_os_timer_frequency() + (tv.tv_usec as u64);
  }
}

#[inline]
pub unsafe fn read_cpu_timer() -> u64 {
  #[cfg(target_arch = "aarch64")]
  {
    use std::arch::aarch64::__rdtcnt_el0;
    return __rdtcnt_el0();
  
  }

  #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
  {
    use std::arch::x86_64::_rdtsc;
    return _rdtsc();
  }
}
pub fn estimate_cpu_timer_freq() -> u64 {
  let ms_to_wait = 100;
  let os_freq = get_os_timer_frequency();
  
  let cpu_start = unsafe { read_cpu_timer()} ;
  let os_start = read_os_timer();
  let mut os_end;
  let mut os_elapsed = 0;
  let os_wait_time = os_freq * ms_to_wait / 1000;
  while(os_elapsed < os_wait_time){
    os_end = read_os_timer();
    os_elapsed = os_end - os_start;
  }

  let cpu_end = unsafe { read_cpu_timer() };
  let cpu_elapsed = cpu_end - cpu_start;

  let mut cpu_freq = 0;
  if (os_elapsed > 0){
    cpu_freq = os_freq * cpu_elapsed / os_elapsed;
  }
  

  return cpu_freq;

}