use dbus::arg;
use dbus::arg::RefArg;
use dbus::Message;
use isahc::AsyncReadResponseExt;
use log::{debug, error, info};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::art::Art;

pub struct Track {
    pub artist: String,
    pub album: String,
    pub album_art: String,
    pub title: String,
}

impl Track {
    pub fn new(artist: &String, album: &String, album_art: &String, title: &String) -> Self {
        Self {
            artist: artist.to_string(),
            album: album.to_string(),
            album_art: album_art.to_string(),
            title: title.to_string(),
        }
    }
    pub fn is_none(&self) -> bool {
        if self.artist.len() == 0 {
            return true;
        } else if self.album.len() == 0 {
            return true;
        } else if self.title.len() == 0 {
            return true;
        } else {
            return false;
        }
    }

    pub fn equals(&self, other: &Track) -> bool {
        if self.title == other.title {
            return true;
        } else {
            return false;
        }
    }

    pub fn get_track(msg: &Message, spotify_cache_dir: Option<&str>, old_track: &Track) -> Track {
        let item1: (Option<arg::PropMap>, Option<arg::PropMap>) = msg.get2();

        if item1.1.is_none() {
            return Track {
                artist: String::new(),
                album: String::new(),
                album_art: String::new(),
                title: String::new(),
            };
        }

        let item1 = item1.1.unwrap();

        if item1.len() == 2
            && item1.contains_key("Metadata")
            && item1.contains_key("PlaybackStatus")
        {
            let metadata = item1.get("Metadata").unwrap();

            let metadata = &metadata.0;

            let iter = metadata.as_iter();
            let map = Self::get_track_metadata_map(&mut iter.unwrap());
            let mut artist = String::new();
            if map.contains_key("artist") {
                artist = format!(
                    "{:?}",
                    map.get("artist")
                        .unwrap()
                        .as_iter()
                        .unwrap()
                        .next()
                        .unwrap()
                )
                .replace('[', "")
                .replace(']', "")
                .replace('"', "");
            }

            let mut title = String::new();
            if map.contains_key("title") {
                title = format!(
                    "{:?}",
                    map.get("title").unwrap().as_iter().unwrap().next().unwrap()
                )
                .replace('[', "")
                .replace(']', "")
                .replace('"', "")
                .replace("\\", "");
            }

            let mut album = String::new();

            if map.contains_key("album") {
                album = format!(
                    "{:?}",
                    map.get("album").unwrap().as_iter().unwrap().next().unwrap()
                )
                .replace('[', "")
                .replace(']', "")
                .replace("\\", "")
                .replace('"', "");
            }

            let mut album_art_url = String::new();
            if map.contains_key("artUrl") {
                album_art_url = format!(
                    "{:?}",
                    map.get("artUrl")
                        .unwrap()
                        .as_iter()
                        .unwrap()
                        .next()
                        .unwrap()
                )
                .replace("https://open.spotify.com", "https://i.scdn.co")
                .replace('"', "");
            }

            if album_art_url.len() > 0 {
                let album_art = spotify_cache_dir.unwrap().to_owned()
                    + "/"
                    + album_art_url
                        .split_at(album_art_url.find("/image").unwrap() + 7)
                        .1;

                if album_art.len() > 0 && !album_art.contains(&old_track.album_art)
                    || old_track.is_none()
                {
                    if !Path::new(&album_art).exists() {
                        info!("This is a new track, so getting album art.");
                        // uncomment for async album art file saves
                        let art_client = Art::build_http_client().unwrap();

                        let art_data = Art::new(&album_art, &album_art_url);

                        let rt = tokio::runtime::Runtime::new().unwrap();

                        let art_bytes = rt.block_on(art_data.get_album_art(&art_client)).unwrap();

                        if art_bytes.len() > 0 {
                            info!("Album art = {} bytes, saving to disk.", art_bytes.len());

                            match art_data.save_album_art(&art_bytes) {
                                Ok(_s) => info!("Album art saved to disk"),
                                Err(e) => error!("Save to disk = failed {:?}", e), // continue showing notification minus album art
                            }
                        } else {
                            error!("Failed to get album art.");
                        }
                    } else {
                        debug!("1 Using cached album art = {}", &album_art);
                    }
                }

                Track {
                    artist,
                    album,
                    album_art,
                    title,
                }
            } else {
                return Track {
                    artist: String::new(),
                    album: String::new(),
                    album_art: String::new(),
                    title: String::new(),
                };
            }
            //Some(format!("{} - {} - {}", artist, title, album_art_url))
        } else {
            return Track {
                artist: String::new(),
                album: String::new(),
                album_art: String::new(),
                title: String::new(),
            };
        }
    }

    async fn _async_save_album_art(url: &String, path: &String) -> Result<(), isahc::Error> {
        let mut img_buffer = vec![];
        isahc::get_async(url)
            .await?
            .copy_to(&mut img_buffer)
            .await?;

        let mut file = match File::create(path) {
            Err(why) => panic!("Couldn't create file {}, {}", path, why),
            Ok(file) => {
                debug!("Album art file saved as {}", path);
                file
            }
        };

        match file.write_all(&img_buffer) {
            Err(why) => panic!("Couldn't write to file {},{}", path, why),
            Ok(_file) => (),
        };

        Ok(())
    }

    pub fn get_track_metadata_map<'a>(
        iter: &mut Box<dyn Iterator<Item = &'a dyn RefArg> + 'a>,
    ) -> HashMap<&'a str, &'a dyn RefArg> {
        let mut map = HashMap::new();
        let arr: Vec<_> = iter.collect();

        for i in (0..arr.len()).step_by(2) {
            match arr[i].as_str() {
                Some(val) => {
                    if val == "xesam:artist" {
                        map.insert("artist", arr[i + 1]);
                    } else if val == "xesam:title" {
                        map.insert("title", arr[i + 1]);
                    } else if val == "mpris:artUrl" {
                        map.insert("artUrl", arr[i + 1]);
                    } else if val == "xesam:album" {
                        map.insert("album", arr[i + 1]);
                    } else if val == "PlaybackStatus" {
                        map.insert("playbackstatus", arr[i + 1]);
                    }
                }
                None => continue,
            }
        }
        map
    }
}
