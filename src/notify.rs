use crate::track::Track;
use std::time::Duration;

use notify_rust;

const TIMEOUT: i32 = 50000;

pub struct Notification {
    pub app_name: String,
    pub summary: String,
    pub body: String,
    pub icon: String,
    pub timeout: i32,
}

impl Notification {
    pub fn display(track: &Track) {
        let notify_body = format! {"{} - {}",&track.artist,&track.album};
        //let notify_app_name = format!("Spotify - {}",&track.title);
        let mut handle = notify_rust::Notification::new()
            .appname("Spotify")
            .summary(&track.title)
            .body(&notify_body)
            .icon(&track.album_art)
            //.timeout(TIMEOUT)
            .show()
            .unwrap();

       
        std::thread::sleep(Duration::from_millis(2_000));
        handle.update();

        handle.close();

     


    }
}
