use libc::pid_t;
use futures::sync::mpsc::Receiver;
use failure::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct WindowMetaData {
    pub id: u64,
    pub pid: Option<pid_t>,
    pub title: Option<String>,
}


pub enum WindowEvent {
    Commit,
}

pub enum Buffer {
    Update{data: Vec<u8>, size: (u32, u32)},
    Erase,
}

trait WindowListener {
    fn handle(&mut self, event: WindowEvent);
}

pub trait WindowManager: Sync + Send {
    fn get_window(&self, id: u64) -> Option<WindowMetaData>;

    fn all_windows(&self) -> Vec<WindowMetaData>;

    fn get_event_stream(&self, id: u64) -> Option<Receiver<WindowEvent>>;

    fn next_frame(&self, id: u64) -> Result<(), Error>;

    fn release_buffer(&self, id: u64) -> Result<(), Error>;

    fn get_buffer(&self, id: u64) -> Result<Buffer, Error>;
}
