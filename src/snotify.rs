use dbus::channel::MatchingReceiver;

use dbus::message::MatchRule;
use dbus::Message;

use dbus::arg;
use dbus::blocking::Connection;

use log::info;

use crate::track::Track;

use crate::notify::Notification;
use std::time;
const DBUS_TIMEOUT: u64 = 250;
const DBUS_PROXY_TIMEOUT: u64 = 5000;

pub struct Snotify {
    pub cache_directory: String,
}

impl Snotify {
    pub fn start_loop(dir: String) {
        let conn = Connection::new_session().expect("D-Bus connection failed");


        info!("Connected to new D-Bus connection.");
        info!("Listening for new track changes.");

        //let mut old_song = String::new();
        let mut old_track = Track::new(
            &String::new(),
            &String::new(),
            &String::new(),
            &String::new(),
        );

        let mut rule = MatchRule::new();



        rule.eavesdrop = true;

        conn.add_match(rule, move |_: (), _, msg| {
            Self::handle_message(&msg, &mut old_track, Some(&dir));
            true
        })
        .expect("add_match failed");

        
        loop {
            conn.process(time::Duration::from_millis(DBUS_TIMEOUT))
                .unwrap();
        }
    }

    // hacky ugly handling
    fn handle_message(msg: &Message, old_track: &mut Track, spotify_cache_dir: Option<&str>) {
        let item1: (Option<arg::PropMap>, Option<arg::PropMap>) = msg.get2();


        if !item1.1.is_none() {
            let item1 = item1.1.unwrap();

            if item1.len() == 2
                && item1.contains_key("Metadata")
                && item1.contains_key("PlaybackStatus")
            {
                let playback_status = &item1.get("PlaybackStatus").unwrap().0;

                if playback_status.as_str() == Some("Playing") {
                    //thread::sleep(time::Duration::from_millis(10));
                    let current_track = Track::get_track(&msg, spotify_cache_dir, old_track);

                    if current_track.is_none() {
                        return;
                    }

                    if current_track.equals(old_track) {
                        return;
                    }
                    *old_track = current_track;

                    info!("Now playing.");
                    info!(
                        "🎶 {} | {} | {} 🎶",
                        old_track.title, old_track.artist, old_track.album
                    );

                    info!("Creating notification.");

                    Notification::display(old_track);


                }
            }
        }else{
            return
        }
    }
}
