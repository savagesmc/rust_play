use std::sync::{Arc, Mutex};
use std::fs::{File, OpenOptions};
use std::io::{self};
use std::os::unix::prelude::*;
use std::path::PathBuf;

/// Initializes or opens a Unix shared memory object.
///
/// # Arguments
///
/// * `name` - A string slice that holds the name of the shared memory object.
/// * `size` - The size in bytes for the shared memory object. Only used when initializing.
///
/// # Returns
///
/// This function returns a result containing either a tuple with the file descriptor wrapped
/// in an `Arc<Mutex<>>` and its path or an error if something goes wrong.
pub fn init_or_open_shm(name: &str, size: Option<usize>) -> io::Result<(Arc<Mutex<File>>, PathBuf)> {
    let path = format!("/dev/shm/{}", name);

    // Try to open the shared memory object
    match OpenOptions::new().custom_flags(libc::O_EXCL).read(true).write(true).open(&path) {
        Ok(file) => {
            if let Some(size) = size {
                // Initialize the shared memory with the specified size
                file.set_len(size.try_into().unwrap())?;
            }
            Ok((Arc::new(Mutex::new(file)), PathBuf::from(path)))
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            // If not found, try to create and initialize it
            let file = File::create(&path).map_err(|e| io::Error::new(e.kind(), "Failed to create shared memory"))?;

            if let Some(size) = size {
                file.set_len(size.try_into().unwrap())?;
            }

            Ok((Arc::new(Mutex::new(file)), PathBuf::from(path)))
        },
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Failed to open or initialize shared memory: {}", e))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Write, Read};

    #[test]
    fn test_init_or_open_shm() {
        let name = "test1_shm";
        let size = 1024; // 1 KB

        // Initialize the shared memory
        let (_shm1, path1) = init_or_open_shm(name, Some(size)).expect("Failed to initialize shared memory");

        // Try opening the same shared memory
        let (_shm2, path2) = init_or_open_shm(name, None).expect("Failed to open shared memory");

        // Ensure both file descriptors point to the same path
        assert_eq!(path1, path2);

        // Clean up by removing the shared memory file
        std::fs::remove_file(&path1).expect("Failed to remove shared memory");
    }

    #[test]
    fn test_point_to_same_memory() -> Result<(), Box<dyn std::error::Error>> {
        let name = "test2_shm";
        let size = 1024; // 1 KB

        let (shm1, path1) = init_or_open_shm(name, Some(size)).expect("Failed to initialize shared memory");

        // Test message to write/read from the shared memory
        let message = b"Hello from Rust shared memory!";

        // Now write something into shared memory using shm1
        {
            let mut file1 = shm1.lock().unwrap();

            file1.write_all(message).expect("Failed to write to shared memory");
        } // Release lock on shm1

        // Now read from shm2 to verify it is the same value
        {
            let (shm2, _path2) = init_or_open_shm(name, None).expect("Failed to open shared memory");
            let mut file2 = shm2.lock().unwrap();

            // Prepare buffer to read the data back
            let mut buffer = vec![0u8; message.len()];

            // Read from the shared memory
            file2.read_exact(&mut buffer).expect("Failed to read from shared memory");

            // Verify the contents match what was written initially
            assert_eq!(&buffer[..], message);
        } // Release lock on shm2

        // Clean up by removing the shared memory file
        std::fs::remove_file(&path1).expect("Failed to remove shared memory");

        Ok(())
    }
}
