use libc::{mqd_t, mq_attr, O_CREAT, O_RDONLY, O_WRONLY};
use std::ffi::CString;
use std::os::unix::io::RawFd;
use std::sync::{Arc, Mutex};


#[repr(u8)]
enum Actions {
    Noop=0,
    Add=1,
    Delete=2,
    Query=3,
}

impl Default for Actions {
    fn default() -> Self {
        Actions::Noop
    }
}

impl Actions {
    #[allow(dead_code)]
    pub fn default_instance() -> Self {
        Actions::default()
    }
}

#[derive(Default)]
pub struct CItem {
    table_id: String,
    action: Actions,
    p: u16,
    k: Vec<u8>,
    m: Vec<u8>,
    r: Vec<u8>,
}

impl CItem {
    pub fn default_instance() -> Self {
        CItem::default()
    }

    #[allow(dead_code)]
    fn new_with_defaults() -> Self {
        CItem {
            table_id: String::new(),
            action: Actions::Add, // Default action can be set to Add or any other appropriate value
            p: 0,
            k: Vec::new(),
            m: Vec::new(),
            r: Vec::new(),
        }
    }

    pub fn pack(&self, mut buffer: Vec<u8>) {
        buffer.extend(self.table_id.len().to_le_bytes());
        buffer.extend(self.table_id.clone().into_bytes());
        buffer.push(match self.action {
            Actions::Noop => 0,
            Actions::Add => 1,
            Actions::Delete => 2,
            Actions::Query => 3,
        });
        buffer.extend(self.p.to_le_bytes());
        buffer.extend(self.k.len().to_le_bytes());
        buffer.extend(self.k.clone());
        buffer.extend(self.m.len().to_le_bytes());
        buffer.extend(self.m.clone());
        buffer.extend(self.r.len().to_le_bytes());
        buffer.extend(self.r.clone());
    }

    pub fn unpack(buffer: &Vec<u8>) -> Result<CItem, &'static str> {
        const MIN_SIZE: usize = std::mem::size_of::<usize>() * 4 // lengths
                                + std::mem::size_of::<u16>() // prio
                                + std::mem::size_of::<u8>(); // action

        if buffer.len() < MIN_SIZE { // Minimum length check for header data
            return Err("Buffer too short to contain valid data");
        }

        let mut result: CItem = Default::default();
        let mut offset = 0;

        // Extract table_id
        let table_id_length = usize::from_le_bytes(buffer[offset..offset + std::mem::size_of::<usize>()].try_into().expect("Slice with incorrect length"));
        offset += std::mem::size_of::<usize>();
        if buffer.len() < offset + table_id_length {
            return Err("Buffer too short to contain full table_id");
        }
        result.table_id = String::from_utf8_lossy(&buffer[offset..offset + table_id_length]).to_string();
        offset += table_id_length;

        // Extract action
        match buffer[offset] {
            0 => { offset += 1; result.action = Actions::Noop; },
            1 => { offset += 1; result.action = Actions::Add; },
            2 => { offset += 1; result.action = Actions::Delete; },
            3 => { offset += 1; result.action = Actions::Query; },
            _ => return Err("Invalid action value"),
        }

        result.p= u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        offset += std::mem::size_of::<u16>();

        // Extract k length and data
        let k_length = usize::from_le_bytes(buffer[offset..offset + std::mem::size_of::<usize>()].try_into().expect("Slice with incorrect length"));
        offset += std::mem::size_of::<usize>();
        if buffer.len() < offset + k_length {
            return Err("Buffer too short to contain full k");
        }
        result.k = Vec::from(&buffer[offset..offset + k_length]);
        offset += k_length;

        // Extract m length and data
        let m_length = usize::from_le_bytes(buffer[offset..offset + std::mem::size_of::<usize>()].try_into().expect("Slice with incorrect length"));
        offset += std::mem::size_of::<usize>();
        if buffer.len() < offset + m_length {
            return Err("Buffer too short to contain full m");
        }
        result.m = Vec::from(&buffer[offset..offset + m_length]);
        offset += m_length;

        // Extract r length and data
        let r_length = usize::from_le_bytes(buffer[offset..offset + std::mem::size_of::<usize>()].try_into().expect("Slice with incorrect length"));
        offset += std::mem::size_of::<usize>();
        if buffer.len() < offset + r_length {
            return Err("Buffer too short to contain full r");
        }
        result.r = Vec::from(&buffer[offset..offset + r_length]);

        Ok(result)
    }
}

#[derive(Default)]
pub struct SItem {
    table_id: String,
    action: Actions,
    index: u16,
    value: Vec<u8>,
}

impl SItem {
    pub fn default_instance() -> Self {
        SItem::default()
    }

    #[allow(dead_code)]
    fn new_with_defaults() -> Self {
        SItem {
            table_id: String::new(),
            action: Actions::Add, // Default action can be set to Add or any other appropriate value
            index: 0,
            value: Vec::new(),
        }
    }

    pub fn pack(&self, mut buffer: Vec<u8>) {
        buffer.extend(self.table_id.len().to_le_bytes());
        buffer.extend(self.table_id.clone().into_bytes());
        buffer.push(match self.action {
            Actions::Noop => 0,
            Actions::Add => 1,
            Actions::Delete => 2,
            Actions::Query => 3,
        });
        buffer.extend(self.index.to_le_bytes());
        buffer.extend(self.value.len().to_le_bytes());
        buffer.extend(self.value.clone());
    }

    pub fn unpack(buffer: &Vec<u8>) -> Result<Self, &'static str> {
        const MIN_SIZE: usize = std::mem::size_of::<usize>() * 2 // lengths
                                + std::mem::size_of::<u16>() // index
                                + std::mem::size_of::<u8>(); // action
        if buffer.len() < MIN_SIZE { // Minimum length check for header data
            return Err("Buffer too short to contain valid data");
        }

        let mut result: SItem = Default::default();
        let mut offset = 0;

        // Extract table_id
        let table_id_length = usize::from_le_bytes(buffer[offset..offset + std::mem::size_of::<usize>()].try_into().expect("Slice with incorrect length"));
        offset += std::mem::size_of::<usize>();
        if buffer.len() < offset + table_id_length {
            return Err("Buffer too short to contain full table_id");
        }
        result.table_id = String::from_utf8_lossy(&buffer[offset..offset + table_id_length]).to_string();
        offset += table_id_length;

        // Extract action
        match buffer[offset] {
            0 => { offset += 1; result.action = Actions::Noop; },
            1 => { offset += 1; result.action = Actions::Add; },
            2 => { offset += 1; result.action = Actions::Delete; },
            3 => { offset += 1; result.action = Actions::Query; },
            _ => return Err("Invalid action value"),
        }

        // Extract index
        result.index = u16::from_le_bytes([buffer[offset], buffer[offset + 1]]);
        offset += std::mem::size_of::<u16>();

        // Extract value length and data
        let value_length = usize::from_le_bytes(buffer[offset..offset + std::mem::size_of::<usize>()].try_into().expect("Slice with incorrect length"));
        offset += std::mem::size_of::<usize>();
        if buffer.len() < offset + value_length {
            return Err("Buffer too short to contain full value");
        }
        result.value = Vec::from(&buffer[offset..offset + value_length]);

        Ok(result)
    }
}

const MAX_QITEM_SIZE:usize = 65536;
const MAX_QITEMS:usize = 1024;

pub struct TableInterface {
    handle: Arc<Mutex<i32>>,
}

impl TableInterface {
    pub fn write(&self, buffer: &Vec<u8>) -> Result<(), &'static str> {
        send_message(&self.handle.lock().unwrap(), buffer.as_slice()).expect("Failed to send message");
        Ok(())
    }

    pub fn read(&self) -> Result<Vec<u8>, &'static str> {
        let mut buffer = vec![0u8; MAX_QITEM_SIZE];
        let received_bytes = receive_message(&self.handle.lock().unwrap(), &mut buffer)
            .expect("Failed to receive message");
        let result: Vec<u8> = buffer[0..received_bytes].to_vec();
        Ok(result)
    }

    pub fn get_writer(name: &str) -> Result<TableInterface, &'static str> {
        let msgq = init_or_open_mq(name, O_CREAT | O_WRONLY, MAX_QITEM_SIZE, MAX_QITEMS);
        match msgq {
            Ok(result) => {
                let (handle, _mqd) = result;
                let table = TableInterface{
                    handle: handle,
                };
                Ok(table)
            },
            Err(_) => Err("Failed to get table writer")
        }
    }

    pub fn get_table_reader(name: &str) -> Result<TableInterface, &'static str> {
        let msgq = init_or_open_mq(name, O_CREAT | O_RDONLY, MAX_QITEM_SIZE, MAX_QITEMS);
        match msgq {
            Ok(result) => {
                let (handle, _mqd) = result;
                let table = TableInterface{
                    handle: handle,
                };
                Ok(table)
            },
            Err(_) => Err("Failed to get table reader: ")
        }
    }
}

impl Drop for TableInterface {
    fn drop(&mut self) {
        close_mq(*self.handle.lock().unwrap()).expect("Failed to close TableInterface")
    }
}

/// Opens a Unix message queue and returns a handle for reading or writing.
fn init_or_open_mq(name: &str,
                       mode: i32,
                       max_item_size: usize,
                       max_queue_size: usize,) -> Result<(Arc<Mutex<mqd_t>>, RawFd), String> {
    let c_name = CString::new(name).map_err(|_| "CString::new failed")?;
    let oflag = if mode == O_WRONLY || mode == O_CREAT { mode | O_CREAT } else { mode };
    let mut attr: mq_attr = unsafe { std::mem::zeroed() };
    attr.mq_msgsize = max_item_size as i64; // Maximum message size
    attr.mq_maxmsg = max_queue_size as i64; // Maximum number of messages

    let mqd = unsafe {
        libc::mq_open(c_name.as_ptr(), oflag, 0644, &mut attr)
    };

    if mqd == -1 {
        Err(format!("Failed to open or create message queue: {}", std::io::Error::last_os_error()))
    } else {
        Ok((Arc::new(Mutex::new(mqd)), mqd))
    }
}

/// Sends a message to the message queue.
fn send_message(handle: &mqd_t, message: &[u8]) -> Result<(), String> {
    let res = unsafe { libc::mq_send(*handle, message.as_ptr() as *const _, message.len(), 0) };
    if res == -1 {
        Err(format!("Failed to send message: {}", std::io::Error::last_os_error()))
    } else {
        Ok(())
    }
}

/// Receives a message from the message queue.
fn receive_message(handle: &mqd_t, buffer: &mut [u8]) -> Result<usize, String> {
    let res = unsafe { libc::mq_receive(*handle, buffer.as_mut_ptr() as *mut _, buffer.len(), std::ptr::null_mut()) };
    if res == -1 {
        Err(format!("Failed to receive message: {}", std::io::Error::last_os_error()))
    } else {
        Ok(res as usize)
    }
}

///  Closes a message queue.
pub fn close_mq(mqd: mqd_t) -> Result<(), String> {
    unsafe {
        if libc::mq_close(mqd) == -1 {
            return Err(format!("Failed to close message queue: {}", std::io::Error::last_os_error()));
        }
    }
    Ok(())
}

pub fn unlink_mq(name: &str) -> Result<(), String> {
    unsafe {
        let c_name = CString::new(name).map_err(|_| "CString::new failed")?;
        if libc::mq_unlink(c_name.as_ptr()) == -1 {
            Err(format!("Failed to unlink message queue: {}", std::io::Error::last_os_error()))
        } else {
            Ok(())
        }
    }
}

pub fn close_and_unlink_mq(mqd: mqd_t, name: &str) -> Result<(), String> {
    use std::thread;
    use std::time::Duration;
    close_mq(mqd).expect("Failed to close message queue");
    thread::sleep(Duration::from_millis(1));
    unlink_mq(name).expect("Failed to unlink message queue");
    Ok(())
}

#[cfg(test)]
mod mq_tests {
    use super::*;
    use libc::{O_RDWR};

    #[test]
    fn test_init_or_open_mq() {
        let name = "/init_test_queue";
        let (handle, mqd) = init_or_open_mq(name, O_CREAT | O_WRONLY, 1024, 10).expect("Failed to open message queue");

        // Ensure the handle and descriptor are valid
        assert!(mqd != -1);
        assert!(*handle.lock().unwrap() == mqd);

        // Close and unlink immediately for cleanup
        close_and_unlink_mq(mqd, name).expect("Failed to close and unlink message queue");
    }

    #[test]
    fn test_send_message() {
        let name = "/send_test_queue";
        let (handle, mqd) = init_or_open_mq(name, O_CREAT | O_WRONLY, 1024, 10).expect("Failed to open message queue");

        // Send a simple message
        send_message(&*handle.lock().unwrap(), b"Hello MQ").expect("Failed to send message");

        // Close and unlink immediately for cleanup
        close_and_unlink_mq(mqd, name).expect("Failed to close and unlink message queue");
    }

    #[test]
    fn test_receive_message() {
        let name = "/receive_test_queue";
        let test_message = b"Hello MQ";

        let (handle, mqd) = init_or_open_mq(name, O_CREAT | O_RDWR, 1024, 10)
            .expect("Failed to open message queue for writing");

        send_message(&*handle.lock().unwrap(), test_message)
            .expect("Failed to send message");

        let mut buffer = vec![0u8; 1024];

        let received_bytes = receive_message(&*handle.lock().unwrap(), &mut buffer)
            .expect("Failed to receive message");

        // Assertion: Verify that the received bytes match what was sent.
        assert_eq!(&buffer[..received_bytes], test_message,
                    "Received message did not match expected content");

        close_and_unlink_mq(mqd, name).expect("Failed to close and unlink message queue");
    }

    #[test]
    fn test_close_mq_and_unlink_mq() {
        let name = "/close_test_queue";

        let (_handle, mqd) = init_or_open_mq(name, O_CREAT | O_WRONLY, 1024, 10).expect("Failed to open message queue");
        close_mq(mqd).expect("Failed to close message queue");

        unlink_mq(name).expect("Failed unlinking close test queue");

        // Attempting to reopen should fail because it was unlinked
        let result = init_or_open_mq(name, O_RDONLY, 1024, 10);

        assert!(result.is_err(), "Expected error when reopening an unlinked message queue");
    }

    #[test]
    fn test_close_and_unlink_mq() {
        let name = "/close_and_unlink_test_queue";

        let (_handle, mqd) = init_or_open_mq(name, O_CREAT | O_WRONLY, 1024, 10).expect("Failed to open message queue");

        close_and_unlink_mq(mqd, name).expect("Failed to close and unlink message queue");

        // Attempting to reopen should fail because it was unlinked
        let result = init_or_open_mq(name, O_RDONLY, 1024, 10);

        assert!(result.is_err(), "Expected error when reopening an unlinked message queue");
    }

}

#[cfg(test)]
mod table_tests {
    #[test]
    fn test_writer() {
    }

    #[test]
    fn test_reader() {
    }
}